/**
 * Story 34-6 (#953) — la règle du conflit de version des trois éditions de la société
 * (e-mail, téléphone et site, adresse), dans *Paramètres* → *Organisation*.
 *
 * Chaque édition envoie sa saisie avec la version de la société. Un `409
 * OPTIMISTIC_LOCK_CONFLICT` dit que la société a changé depuis : l'écran la relit. Avant
 * cette story, il gardait la saisie sans rien montrer, et le clic suivant, parti avec la
 * version fraîche, remplaçait l'écriture concurrente que personne n'avait vue.
 *
 * Cette fonction compare, champ par champ :
 * - `base`, la copie des valeurs enregistrées prise à l'ouverture de l'édition (et remise
 *   à jour après chaque relecture réussie) ;
 * - `stored`, les valeurs relues après le 409 ;
 * - `input`, la saisie.
 *
 * Définitions (fiche 34-6, § Conception ; C-e343-6, C-e343-14) :
 * - `merged = { f : input[f] = base[f] ∧ stored[f] ≠ base[f] }` — un champ que
 *   l'Administrateur n'a pas touché prend la valeur relue (fusion à trois voies) : aucun
 *   travail n'est perdu, et le second clic ne rétablit pas l'ancienne valeur sur
 *   l'écriture d'un collègue ;
 * - `changed = { f : input[f] ≠ base[f] ∧ stored[f] ≠ base[f] ∧ stored[f] ≠ input[f] }` —
 *   modifié des deux côtés, à des valeurs différentes : seul cas où enregistrer de
 *   nouveau remplacera quelque chose ;
 * - `already-saved` si, après la fusion, chaque champ relu égale la saisie, ET qu'au
 *   moins un champ avait été modifié avant elle (sans modification, l'Administrateur n'a
 *   rien enregistré) ;
 * - `unchanged` si `changed = ∅` hors `already-saved` ; `changed` sinon.
 *
 * Les comparaisons portent sur les valeurs telles que le serveur les range : `trim()`, et
 * chaîne vide ⇔ `null`. Le pays de l'adresse se normalise en plus comme
 * `normalized_country` (`crates/kesh-api/src/address_input.rs`).
 */

/** Une valeur de champ telle que l'écran ou le serveur la porte. */
export type CompanyFieldValue = string | null | undefined;

/** Normalisation d'un champ avant comparaison. */
export type CompanyFieldNormalizer = (value: CompanyFieldValue) => string;

/** Les trois issues d'un conflit de version, du point de vue d'un bloc. */
export type CompanyConflictKind = 'unchanged' | 'already-saved' | 'changed';

/** Résultat de {@link companyEditConflict}. */
export interface CompanyEditConflict<K extends string> {
	kind: CompanyConflictKind;
	/** Champs modifiés des deux côtés, à des valeurs différentes, dans l'ordre de `fields`. */
	changed: K[];
	/** Champs non modifiés, repris de la valeur relue, dans l'ordre de `fields`. */
	merged: K[];
	/** La saisie après fusion : `input`, où chaque champ de `merged` vaut `stored`. */
	input: Record<K, string>;
}

/** `trim()`, et l'absence de valeur vaut la chaîne vide. */
export function normalizeCompanyValue(value: CompanyFieldValue): string {
	return (value ?? '').trim();
}

/**
 * Le pays comme le serveur le range : `trim()`, ASCII en majuscules (et lui seul, comme
 * `to_ascii_uppercase`), vide ou absent valant `CH`.
 */
export function normalizeCountry(value: CompanyFieldValue): string {
	const trimmed = normalizeCompanyValue(value).replace(/[a-z]/g, (c) => c.toUpperCase());
	return trimmed || 'CH';
}

/** Les normalisations propres au bloc adresse : le pays seul diffère de la règle commune. */
export const ADDRESS_NORMALIZERS = { country: normalizeCountry } as const;

/**
 * Juge un `409` d'une édition de la société, champ par champ. Fonction pure.
 *
 * @param base       copie des valeurs enregistrées à l'ouverture (ou à la dernière relecture)
 * @param stored     valeurs relues après le 409
 * @param input      la saisie
 * @param fields     les champs du bloc, dans l'ordre d'affichage
 * @param normalizers normalisation par champ ; {@link normalizeCompanyValue} par défaut
 */
export function companyEditConflict<K extends string>(
	base: Record<K, CompanyFieldValue>,
	stored: Record<K, CompanyFieldValue>,
	input: Record<K, CompanyFieldValue>,
	fields: readonly K[],
	normalizers: Partial<Record<K, CompanyFieldNormalizer>> = {},
): CompanyEditConflict<K> {
	const norm = (f: K, v: CompanyFieldValue) => (normalizers[f] ?? normalizeCompanyValue)(v);
	const same = (f: K, a: CompanyFieldValue, b: CompanyFieldValue) => norm(f, a) === norm(f, b);

	const merged: K[] = [];
	const changed: K[] = [];
	let modifiedBefore = false;
	const mergedInput = {} as Record<K, string>;

	for (const f of fields) {
		const touched = !same(f, input[f], base[f]);
		const movedElsewhere = !same(f, stored[f], base[f]);
		if (touched) modifiedBefore = true;
		if (!touched && movedElsewhere) {
			merged.push(f);
			mergedInput[f] = stored[f] ?? '';
		} else {
			mergedInput[f] = input[f] ?? '';
		}
		if (touched && movedElsewhere && !same(f, stored[f], input[f])) changed.push(f);
	}

	const alreadySaved = modifiedBefore && fields.every((f) => same(f, stored[f], mergedInput[f]));
	const kind: CompanyConflictKind = alreadySaved
		? 'already-saved'
		: changed.length > 0
			? 'changed'
			: 'unchanged';
	return { kind, changed, merged, input: mergedInput };
}

/**
 * Vrai si les valeurs d'un bloc dans `stored` égalent sa copie `base`, champ par champ, aux
 * mêmes normalisations que {@link companyEditConflict}.
 *
 * Revue de code P1 (B-1, E1-1 ; C-e346-8) : quand la page reçoit une société plus récente
 * (succès ou relecture d'un AUTRE bloc), une édition ouverte n'adopte la nouvelle version que
 * si ses propres champs n'ont pas bougé. Sinon elle garde la version de sa copie, et son
 * enregistrement reçoit son propre 409 : l'écriture concurrente est alors montrée, au lieu
 * d'être écrasée sous une version fraîche que l'écran n'a jamais affichée.
 */
export function sameBlockValues<K extends string>(
	base: Record<K, CompanyFieldValue>,
	stored: Record<K, CompanyFieldValue>,
	fields: readonly K[],
	normalizers: Partial<Record<K, CompanyFieldNormalizer>> = {},
): boolean {
	return fields.every((f) => {
		const norm = normalizers[f] ?? normalizeCompanyValue;
		return norm(base[f]) === norm(stored[f]);
	});
}

/** La version d'une société et la séquence de départ de la requête qui l'a rendue. */
export interface CompanyStamp {
	version: number;
	seq: number;
}

/**
 * Faut-il adopter une société reçue, et quelle séquence la page retient-elle ensuite ?
 * Fonction pure (revues P3 et P4 ; C-e346-10, C-e346-11).
 *
 * - Version inférieure à celle de la page : refusée, quelle que soit la séquence.
 * - Version égale : adoptée seulement si sa requête est partie après celle dont vient la
 *   page. Le verrou de période ne change pas la version, si bien que l'égalité ne départage
 *   pas deux réponses.
 * - Version supérieure : toujours adoptée.
 *
 * La séquence retenue ne recule jamais (`Math.max`, revue P4 E4-2) : une société de version
 * supérieure partie plus tôt ne rouvre pas la porte à une réponse de même version partie
 * entre les deux.
 */
export function adoptionDecision(current: CompanyStamp | null, incoming: CompanyStamp): { adopt: boolean; seq: number } {
	if (current === null) return { adopt: true, seq: incoming.seq };
	if (incoming.version < current.version) return { adopt: false, seq: current.seq };
	if (incoming.version === current.version && incoming.seq < current.seq) {
		return { adopt: false, seq: current.seq };
	}
	return { adopt: true, seq: Math.max(current.seq, incoming.seq) };
}

/**
 * Classe des champs d'une édition de la société pendant l'envoi et la relecture, où ils sont
 * en lecture seule (C-e346-9). Revue P4 (B4-1 = E4-1 = A4-1) : `bg-input` n'est pas un jeton
 * du thème (`src/app.css`), et Tailwind 4 ne produit aucune règle pour lui. Le fond
 * `surface-alt` seul ne se distingue guère du blanc (1,05:1) ; le texte passe donc aussi en
 * `text-muted` (4,55:1 sur `surface-alt`, au-dessus du seuil AA). Un test vérifie que chaque
 * couleur nommée ici est un jeton défini.
 */
export const PENDING_FIELD_CLASS = 'read-only:bg-surface-alt read-only:text-text-muted read-only:cursor-progress';

/**
 * Ce que l'écran affiche d'un conflit : les champs nommés et les valeurs relues.
 * `null` quand rien n'est à montrer (aucun champ en conflit ni repris).
 */
export interface CompanyConflictNoticeState {
	changed: string[];
	merged: string[];
	/** Valeurs relues, par champ, telles qu'affichées sous chaque champ nommé. */
	stored: Record<string, string>;
}

/** L'état d'avis d'un conflit, ou `null` s'il n'y a rien à nommer. */
export function conflictNotice<K extends string>(
	conflict: CompanyEditConflict<K>,
	stored: Record<K, CompanyFieldValue>,
): CompanyConflictNoticeState | null {
	if (conflict.changed.length === 0 && conflict.merged.length === 0) return null;
	const values: Record<string, string> = {};
	for (const f of [...conflict.changed, ...conflict.merged]) values[f] = normalizeCompanyValue(stored[f]);
	return { changed: [...conflict.changed], merged: [...conflict.merged], stored: values };
}
