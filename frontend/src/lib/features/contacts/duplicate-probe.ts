/**
 * Socle d'appariement des contacts — Story 22-2a (#301).
 *
 * Module **pur** : aucune dépendance réseau, DOM, horloge ni i18n. C'est sa
 * raison d'être — les défauts du mécanisme se paient en secondes de vitest, et
 * non en passes de revue adversariale. La 22-2b le consomme ; lui ne consomme
 * rien.
 *
 * ## L'ordre des opérations est la partie fragile
 *
 * `normaliser → mesurer → décider`, jamais l'inverse. `probeTerm` compose les
 * trois de sorte que l'ordre soit **dans le module**, donc testable : quand il
 * vivait chez l'appelant, la mutation « mesurer avant de normaliser » ne
 * faisait tomber aucune preuve des deux moitiés.
 *
 * ## La double normalisation n'est pas un doublon
 *
 * `normalizeTerm` applique **NFC** — pour que `.length` soit stable, `aé`
 * valant 2 unités en NFC et 3 en NFD. `fold` applique ensuite **NFD + strip**
 * pour le repli d'accents. L'une stabilise une *mesure*, l'autre produit une
 * *comparaison* ; composer les deux est inoffensif.
 *
 * ⚠️ `fold` fait l'**inverse** de `kesh_core::text::canonical_key`, et c'est
 * voulu : `canonical_key` sert l'**unicité** d'un identifiant, où un accent
 * doit distinguer ; `fold` sert la **ressemblance**, où un accent doit
 * rapprocher.
 *
 * ⚠️ `fold` passe par `normalizeTerm`. Sans cela, le terme perdrait ses traits
 * d'union et pas les noms comparés : retaper `Jean-Marc Zwahlen` à l'identique
 * ne remonterait pas `Jean-Marc Zwahlen`, qui sortirait **dernier**. C'est le
 * défaut fondateur de la story, et il se referme par la **symétrie**.
 *
 * ⚠️ `İ` (U+0130) se décompose en NFD en `I` + point combinant ; le strip
 * retire le point, et le `toLowerCase` final rend `i`. L'ordre compte :
 * minusculer **avant** le strip rendrait deux caractères. Ne pas réordonner.
 */

import type { ContactResponse, ContactType } from './contacts.types';

/**
 * Les dix opérateurs `BOOLEAN MODE` de MariaDB, miroir de `BOOLEAN_FT_OPERATORS`
 * (`crates/kesh-db/src/util/search.rs:41`).
 */
const BOOLEAN_FT_OPERATORS = /[+\-><()~*"\\]/g;

/**
 * Caractères invisibles **de largeur nulle** — miroir d'`is_zero_width`
 * (`crates/kesh-core/src/text.rs`, Story 22-1).
 *
 * ⚠️ Les blancs en sont **exclus**, et la distinction est ce qui empêche de
 * confondre `CLI 1` et `CLI1` : une espace *marque*, un ZWSP ne marque rien.
 */
const ZERO_WIDTH = /[\u00AD\u200B-\u200F\u2060-\u2064\uFEFF]/g;

/** Longueur minimale d'un token indexé — `innodb_ft_min_token_size`, mesuré à 3. */
export const MIN_TOKEN_LENGTH = 3;

/**
 * Forme normalisée d'un terme de recherche.
 *
 * NFC, puis les dix opérateurs **remplacés par une espace** — jamais
 * supprimés. Supprimer est ce que fait `escape_boolean_ft` côté serveur, et
 * c'est la cause de l'issue #314 : `Coop-Vaud` devient `CoopVaud`, qui ne
 * matche ni `Coop` ni `Vaud`, les deux tokens réellement indexés.
 */
export function normalizeTerm(raw: string): string {
	return raw
		.replace(ZERO_WIDTH, '')
		.normalize('NFC')
		.replace(BOOLEAN_FT_OPERATORS, ' ')
		.replace(/\s+/g, ' ')
		.trim();
}

/**
 * Repli de comparaison : normalisation du terme, puis accents et casse.
 *
 * Appliqué **symétriquement** au terme et à chaque nom comparé (cf. l'en-tête
 * du module). Limites assumées : `œ` et `ß` ne se replient pas vers `oe` et
 * `ss`. Elles ne mordent pas sur un carnet suisse ordinaire — `ü`, `ä`, `ö`,
 * `é`, `è` se replient tous correctement, et l'allemand de Suisse n'emploie pas
 * `ß`.
 */
export function fold(s: string): string {
	return normalizeTerm(s)
		.normalize('NFKC')
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '')
		.toLowerCase();
}

/**
 * Compose le terme de recherche selon le type de contact.
 *
 * ⚠️ Le serveur recompose `name` de la même façon pour une `Personne`
 * (`routes/contacts.rs`, `format!("{f} {l}")`). Une sonde branchée sur la seule
 * raison sociale serait **entièrement inopérante pour les personnes
 * physiques**, sans rien casser ni faire échouer la compilation.
 *
 * Pendant la frappe, l'un des deux champs d'une `Personne` est presque toujours
 * vide : c'est l'état **normal**, pas un cas dégradé.
 */
export function buildTerm(
	type: ContactType,
	name: string,
	firstName: string,
	lastName: string
): string {
	const sain = (v: string) => (typeof v === 'string' ? v : '');
	return type === 'Personne'
		? `${sain(firstName)} ${sain(lastName)}`.trim()
		: sain(name).trim();
}

/**
 * Vrai si le terme **déjà normalisé** porte un token d'au moins
 * {@link MIN_TOKEN_LENGTH} caractères.
 *
 * ⚠️ **`isArmed` NE normalise PAS.** Elle reçoit la sortie de
 * {@link normalizeTerm}. Le seuil porte sur le **plus long token** et non sur
 * la longueur du terme : la `Personne` « An Li » fait cinq caractères mais
 * aucun token indexable, et la requête serait un silence garanti.
 *
 * Le seuil de trois est une **politique d'ergonomie** — limiter le bruit et le
 * nombre de requêtes — et non une contrainte du moteur :
 * `innodb_ft_min_token_size` gouverne les tokens *exacts*, pas les préfixes, et
 * le dépôt appose toujours `*`.
 */
export function isArmed(normalized: string): boolean {
	if (normalized === '') return false;
	return normalized.split(/\s+/).some((t) => t.length >= MIN_TOKEN_LENGTH);
}

/** Résultat de {@link probeTerm} : le terme à envoyer, et s'il faut l'envoyer. */
export interface ProbeTerm {
	normalized: string;
	armed: boolean;
}

/**
 * Compose `buildTerm → normalizeTerm → isArmed`, **dans cet ordre**.
 *
 * ⚠️ C'est la raison d'être de cette fonction : tant que l'ordre vivait chez
 * l'appelant, la mutation « mesurer le seuil avant de normaliser » n'avait
 * aucun site d'application dans un module testable — elle traversait les deux
 * moitiés de la story sans faire tomber une seule preuve.
 */
export function probeTerm(
	type: ContactType,
	name: string,
	firstName: string,
	lastName: string
): ProbeTerm {
	const normalized = normalizeTerm(buildTerm(type, name, firstName, lastName));
	return { normalized, armed: isArmed(normalized) };
}

/** Longueur du plus long préfixe commun entre deux chaînes **repliées**. */
function commonPrefixLength(a: string, b: string): number {
	let i = 0;
	while (i < a.length && i < b.length && a[i] === b[i]) i++;
	return i;
}

/**
 * Classe les candidats du plus proche au plus lointain.
 *
 * Cinq critères, **dans cet ordre**, sur des chaînes passées par {@link fold} :
 *
 * 1. le nom **commence par** le terme complet ;
 * 2. puis le nombre de tokens **distincts** du terme présents dans le nom,
 *    décroissant — un terme qui répète un token (`jean jean`) ne le compte
 *    qu'une fois, sans quoi le classement récompenserait une faute de frappe ;
 * 3. puis la longueur du plus long préfixe commun **entre `fold(name)` et
 *    `fold(terme)`**, décroissante ;
 * 4. puis l'ordre alphabétique ;
 * 5. puis l'`id`, croissant.
 *
 * ⚠️ Les critères 4 et 5 ne sont pas des critères de pertinence : ce sont ceux
 * qui rendent le classement **déterministe**, donc testable. Le 5 est
 * indispensable — deux contacts strictement homonymes (un père et son fils)
 * sont ex æquo sur les quatre premiers, et un tri stable rendrait l'ordre
 * d'*entrée*.
 *
 * ⚠️ **`rank` CLASSE et ne FILTRE pas.** Elle rend exactement les éléments
 * reçus, dans un autre ordre. C'est ce dont dépend l'arithmétique de
 * {@link countOthers}, qui soustrait d'un total serveur.
 *
 * @param items candidats, tels que rendus par le serveur
 * @param normalizedTerm terme **déjà** passé par {@link normalizeTerm}
 */
export function rank(items: ContactResponse[], normalizedTerm: string): ContactResponse[] {
	const term = fold(normalizedTerm);
	const termTokens = [...new Set(term.split(/\s+/).filter(Boolean))];

	const startsWith = (name: string) => (name.startsWith(term) ? 0 : 1);
	const sharedTokens = (name: string) => {
		const nameTokens = new Set(name.split(/\s+/).filter(Boolean));
		return termTokens.filter((t) => nameTokens.has(t)).length;
	};

	return [...items].sort((a, b) => {
		const fa = fold(a.name);
		const fb = fold(b.name);
		return (
			startsWith(fa) - startsWith(fb) ||
			sharedTokens(fb) - sharedTokens(fa) ||
			commonPrefixLength(fb, term) - commonPrefixLength(fa, term) ||
			(fa < fb ? -1 : fa > fb ? 1 : 0) ||
			a.id - b.id
		);
	});
}

/**
 * Retire le contact en cours d'édition de ses propres résultats.
 *
 * Sans cette garde, ouvrir une fiche et toucher son nom afficherait « un
 * contact au nom proche existe » en désignant **la fiche elle-même** — la façon
 * la plus rapide d'apprendre à l'utilisateur à ne plus lire les avertissements.
 *
 * @param editingId `null` en création. **Jamais `undefined`** : `null` est
 *   l'unique convention d'absence de ce module.
 */
export function excludeSelf(items: ContactResponse[], editingId: number | null): ContactResponse[] {
	if (editingId === null) return [...items];
	return items.filter((c) => c.id !== editingId);
}

/**
 * Nombre de correspondances **non affichées**, pour la mention « et N autres ».
 *
 * ⚠️ `total` est le `COUNT(*)` **serveur**, qui ne connaît pas la notion de
 * « soi » : l'exclusion du contact édité n'a lieu qu'au client. Sans la
 * soustraction, corriger un caractère du nom d'une fiche existante afficherait
 * « et 1 autre » **au-dessus d'une liste vide**, en désignant la fiche qu'on
 * modifie.
 *
 * ⚠️ **Précondition** : si le contact édité figure dans `total`, il figure
 * aussi dans `items`. Au-delà de la fenêtre d'interrogation, le compteur
 * sur-compte d'une unité — écart **assumé**, il n'affecte qu'un compteur
 * indicatif, jamais une proposition affichée ni une décision.
 *
 * @param items les candidats **non filtrés**, tels que reçus du serveur
 * @param affiches les propositions **effectivement affichées**, pas la liste
 *   classée complète
 */
export function countOthers(
	total: number,
	items: ContactResponse[],
	affiches: ContactResponse[],
	editingId: number | null
): number {
	if (!Number.isFinite(total)) return 0;
	const self = editingId !== null && items.some((c) => c.id === editingId) ? 1 : 0;
	return Math.max(0, total - self - affiches.length);
}

/**
 * Le contact qui porte **exactement** cet IDE, hors contact édité.
 *
 * ⚠️ Un contact peut remonter parce que la chaîne figure dans son nom ou son
 * email, **sans porter cet IDE** : émettre le signal franc sur le seul fait
 * qu'un résultat existe produirait un message faux et péremptoire.
 *
 * @param normalized la valeur **envoyée avec la requête**, jamais une relecture
 *   du champ à l'arrivée de la réponse : `normalizeIdeForApi('')` rend `null`,
 *   et `null === null` désignerait n'importe quel contact sans IDE.
 */
export function findIdeHolder(
	items: ContactResponse[],
	normalized: string | null,
	editingId: number | null
): ContactResponse | undefined {
	if (normalized === null || normalized === '') return undefined;
	return items.find((c) => c.ideNumber === normalized && c.id !== editingId);
}
