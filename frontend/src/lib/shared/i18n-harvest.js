/**
 * **Moisson des replis** — story 23-1b (#316). Logique pure, sans I/O de sortie.
 *
 * ⚠️ **Pourquoi un module et non un script** : `AC10 (d)` exige que le moissonneur porte
 * son propre test vitest, et `vite.config.ts` borne vitest à `src/**\/*.test.ts`. Un
 * script de `scripts/` ne serait exécuté par aucun gate — toute la substance d'AC10
 * n'aurait alors pour preuve que deux nombres recopiés à la main. Même raisonnement que
 * `i18n-literal-reader.js` (23-1a, D1-bis).
 *
 * ⚠️ **Il PROPOSE, il n'écrit jamais.** Un repli est écrit dans le feu de l'action —
 * souvent sans majuscule, sans point final. Le laisser devenir un libellé de catalogue
 * sans relecture, c'est faire entrer 285 approximations dans le produit.
 */

// ⚠️ **Le MÊME lecteur que la garde** (23-1a, D1-bis). Une copie qui dérive est
// exactement le défaut que ce module partagé existe pour empêcher — et la classe de
// caractères négative qu'une copie naïve emploierait a coûté cinq erreurs à cette story.
import { findCallSites, readFallback } from './i18n-literal-reader.js';

/**
 * Vrai si ce nom de fichier entre dans le périmètre de la moisson.
 *
 * ⚠️ **Cette règle vivait dans le script, donc hors de tout gate** — et le docstring du
 * test revendiquait pourtant l'« exclusion des tests » parmi ce qu'il prouvait. Elle est
 * ici pour être testée : `i18n.svelte.test.ts` demande `une-cle` et `compteur`, clés
 * fictives qui doivent le rester. (Revue de code 23-1b, passe 1.)
 *
 * @param {string} nom
 * @returns {boolean}
 */
export function dansLePerimetreDeFichier(nom) {
	return /\.(svelte|ts)$/.test(nom) && !nom.includes('.test.');
}

/**
 * @typedef {{ chemin: string, source: string }} Fichier
 * @typedef {{
 *   replis: Map<string, Map<string, string[]>>,
 *   sansRepli: Map<string, string[]>,
 *   divergents: [string, Map<string, string[]>][],
 *   aEchapper: [string, Map<string, string[]>][]
 * }} Moisson
 */

/**
 * Moissonne les replis des clés **absentes du catalogue**.
 *
 * @param {Fichier[]} fichiers   les fichiers à parcourir. ⚠️ **Le périmètre est appliqué
 *                               ICI**, pas laissé à l'appelant : la passe 1 avait déplacé
 *                               `dansLePerimetreDeFichier` dans ce module pour le rendre
 *                               testable, mais la GARANTIE reposait encore sur la politesse
 *                               du script. Retirer l'appel là-bas laissait les 8 tests VERTS
 *                               — vérifié par mutation. Un maillon prouvé ne prouve pas la
 *                               chaîne. (Revue de code 23-1b, passe 2.)
 * @param {(cle: string) => boolean} existeAuCatalogue
 * @param {string[]} [prefixes]  restreint aux clés commençant par l'un d'eux
 * @returns {Moisson}
 */
export function moissonner(fichiers, existeAuCatalogue, prefixes = []) {
	const dansLePerimetre = (/** @type {string} */ cle) =>
		prefixes.length === 0 || prefixes.some((p) => cle.startsWith(p));

	/** @type {Map<string, Map<string, string[]>>} */
	const replis = new Map();
	/** @type {Map<string, string[]>} */
	const sansRepli = new Map();

	for (const { chemin, source } of fichiers) {
		// ⚠️ `split` rend toujours au moins une chaîne, donc le cas `undefined` est inatteignable
		// à l'EXÉCUTION — mais `.pop()` est typé `string | undefined` et `npm run check` le refuse.
		// L'indexation par la longueur dit la même chose sans faire croire à une défense qui
		// n'existe pas. (Le `\\` est là parce que `node:path.join` le produit ailleurs.) Passe 3.
		const morceaux = chemin.split(/[\\/]/);
		if (!dansLePerimetreDeFichier(morceaux[morceaux.length - 1])) continue;
		for (const site of findCallSites(source)) {
			if (site.arg?.kind !== 'literal') continue;
			const cle = site.arg.value;
			if (existeAuCatalogue(cle) || !dansLePerimetre(cle)) continue;

			const repli = readFallback(source, site.afterFirstArg);
			const ou = `${chemin}:${site.line}`;
			if (repli === null || repli.kind !== 'literal') {
				if (!sansRepli.has(cle)) sansRepli.set(cle, []);
				/** @type {string[]} */ (sansRepli.get(cle)).push(ou);
				continue;
			}
			if (!replis.has(cle)) replis.set(cle, new Map());
			const parTexte = /** @type {Map<string, string[]>} */ (replis.get(cle));
			if (!parTexte.has(repli.value)) parTexte.set(repli.value, []);
			/** @type {string[]} */ (parTexte.get(repli.value)).push(ou);
		}
	}

	// Une clé qui a AU MOINS un repli littéral quelque part n'est pas « sans repli ».
	for (const cle of replis.keys()) sansRepli.delete(cle);

	const divergents = [...replis].filter(([, parTexte]) => parTexte.size > 1);
	// Replis qui ne peuvent pas entrer tels quels dans un `.ftl` — à échapper à la main.
	// ⚠️ Les DEUX motifs d'écartement de `fragmentFtl`, pas seulement celui de la valeur.
	// La passe 2 avait ajouté le contrôle de clé sans lui donner de canal de signalement :
	// une clé mal formée disparaissait du fragment sans figurer NULLE PART — le silence même
	// que cette story combat, rouvert par le correctif censé le fermer. (Passe 3.)
	const aEchapper = [...replis].filter(
		([cle, parTexte]) => !estCleFtlSaine(cle) || !estFtlSain([...parTexte.keys()][0])
	);
	return { replis, sansRepli, divergents, aEchapper };
}

/**
 * Vrai si `texte` peut entrer tel quel dans un `.ftl`.
 *
 * ⚠️ **Un repli invalide ne casse pas une ligne, il casse TOUTE la locale** :
 * `loader.rs:71` propage l'erreur de `FluentResource::try_new` sans tri partiel. Deux
 * formes cassent — un **retour à la ligne** (Fluent y attend une continuation indentée)
 * et une **accolade non appariée** (Fluent y attend un placeable).
 *
 * ⚠️ **Les accolades APPARIÉES, elles, sont légitimes et doivent passer** : six replis du
 * dépôt portent de vrais placeables — `Facture #{$id} enregistrée.`, `{$n} facture(s)
 * importée(s).` — que le frontend interpole lui-même. Un correctif qui échapperait toutes
 * les accolades les casserait. (Revue de code 23-1b, passe 1.)
 *
 * @param {string} texte
 * @returns {boolean}
 */
export function estFtlSain(texte) {
	// ⚠️ Le VIDE est le tueur que la version d'origine laissait passer : `cle = `, `cle =` et
	// `cle =    ` sont TOUS rejetés par Fluent (`ExpectedMessageField`). La garde qui existe
	// pour éviter qu'une ligne emporte la locale laissait donc passer la ligne qui l'emporte.
	if (texte.trim() === '') return false;
	if (texte.includes('\n')) return false;

	// ⚠️ Compter les accolades ne suffit pas : `a {} b`, `JSON {"a": 1}` et `code {1,2}` sont
	// APPARIÉS et pourtant rejetés par Fluent. On valide donc le CONTENU ENTIER de chaque
	// placeable, pas son amorce.
	//
	// Le jeu accepté est celui que le moissonneur peut légitimement produire : une variable
	// (`{$id}`), une référence de message (`{marque}`) ou de terme (`{-produit}`), et le
	// LITTÉRAL DE CHAÎNE — `{"{"}` est la façon dont Fluent échappe une accolade, et `fr-CH`
	// s'en sert réellement (`invoice-numbering-format-hint`).
	//
	// ⚠️ **L'asymétrie des coûts commande la sévérité.** Un refus injustifié coûte un repli à
	// recopier à la main ; un accord injustifié coûte une locale entière qui ne charge plus,
	// `loader.rs` propageant l'erreur de parse sans tri. En cas de doute : refuser.
	let i = 0;
	while (i < texte.length) {
		const c = texte[i];
		if (c === '}') return false; // fermante orpheline
		if (c !== '{') {
			i += 1;
			continue;
		}
		const fin = finDuPlaceable(texte, i);
		if (fin === -1) return false;
		const dedans = texte.slice(i + 1, fin);
		const estVariableOuReference = /^\s*(\$[a-zA-Z][\w-]*|-?[a-zA-Z][\w-]*)\s*$/.test(dedans);
		if (!estVariableOuReference && !estLitteralChaineValide(dedans.trim())) return false;
		i = fin + 1;
	}
	return true;
}

/**
 * Index de l'accolade fermante du placeable ouvert en `debut`, ou `-1` s'il n'y en a pas.
 *
 * ⚠️ Un `}` peut vivre DANS un littéral de chaîne — `{"}"}` est du Fluent valide —, donc on
 * balaie caractère par caractère au lieu de chercher la prochaine fermante. Une première
 * rédaction s'y est laissé prendre. (Passe 3.)
 *
 * @param {string} texte
 * @param {number} debut  index de l'accolade ouvrante
 * @returns {number}
 */
function finDuPlaceable(texte, debut) {
	let j = debut + 1;
	while (j < texte.length && texte[j] !== '}') {
		if (texte[j] !== '"') {
			j += 1;
			continue;
		}
		j += 1; // on entre dans le littéral
		while (j < texte.length && texte[j] !== '"') j += texte[j] === '\\' ? 2 : 1;
		if (j >= texte.length) return -1; // guillemet jamais refermé
		j += 1; // on sort du littéral
	}
	return j < texte.length ? j : -1;
}

/**
 * Vrai si `dedans` est un littéral de chaîne Fluent **entièrement valide**, échappements compris.
 *
 * ⚠️ **C'est le trou que la passe 3 avait laissé béant**, et dans le sens dangereux. Fluent
 * n'admet dans un littéral que quatre échappements — `\\`, `\"`, `\uXXXX` et `\UXXXXXX`. Toute
 * autre paire est une `UnknownEscapeSequence`, donc une erreur de parse, donc **une locale
 * entière qui ne charge plus**. Or le motif `\\.` de la passe 3 acceptait n'importe quel
 * caractère après la barre : `{"a\nb"}` passait la garde et tuait la locale.
 *
 * ⚠️ **Pourquoi l'audit des 5001 entrées ne pouvait PAS le trouver** — et le point vaut au-delà
 * de cette fonction : ces entrées sont, par construction, des valeurs qui **chargent déjà**.
 * Elles ne peuvent donc contenir aucun échappement illégal. Un corpus de valeurs valides ne
 * révèle que les faux REFUS ; il est structurellement aveugle aux faux ACCORDS. Éprouver une
 * garde contre ce qu'elle doit accepter ne dit rien de ce qu'elle doit refuser.
 *
 * @param {string} dedans  le contenu du placeable, déjà détouré
 * @returns {boolean}
 */
function estLitteralChaineValide(dedans) {
	if (dedans.length < 2 || !dedans.startsWith('"') || !dedans.endsWith('"')) return false;
	const corps = dedans.slice(1, -1);
	let k = 0;
	while (k < corps.length) {
		if (corps[k] === '"') return false; // guillemet nu au milieu
		if (corps[k] !== '\\') {
			k += 1;
			continue;
		}
		const suivant = corps[k + 1];
		if (suivant === '\\' || suivant === '"') k += 2;
		else if (suivant === 'u' && /^[0-9a-fA-F]{4}/.test(corps.slice(k + 2))) k += 6;
		else if (suivant === 'U' && /^[0-9a-fA-F]{6}/.test(corps.slice(k + 2))) k += 8;
		else return false; // UnknownEscapeSequence / InvalidUnicodeEscapeSequence
	}
	return true;
}

/**
 * Vrai si cet identifiant peut servir de CLÉ à une entrée Fluent.
 *
 * ⚠️ **La passe 1 n'avait gardé qu'un côté du signe `=`.** `estFtlSain` contrôlait la
 * valeur ; la clé, elle, entrait telle quelle. Or une clé vide, numérique, ou portant un
 * `.` produit une ligne que le parseur rejette — et `loader.rs` propageant l'erreur sans
 * tri, **une seule ligne invalide empêche le chargement de TOUTE la locale**. Exactement le
 * raisonnement qui avait motivé `estFtlSain`, appliqué à sa moitié manquante.
 *
 * ⚠️ Le `.` n'est pas un caractère interdit par étourderie : en Fluent il introduit un
 * **attribut**, si bien que `foo.bar = x` ne définit pas la clé `foo.bar` mais l'attribut
 * `bar` d'un message `foo` — le fragment serait donc accepté ET faux.
 *
 * @param {string} cle
 * @returns {boolean}
 */
export function estCleFtlSaine(cle) {
	// Fluent : un identifiant commence par une lettre ASCII, puis lettres, chiffres, `_`, `-`.
	// Un `.` y introduit un ATTRIBUT, pas une clé — `foo.bar = x` ne définit pas `foo.bar`.
	return /^[a-zA-Z][a-zA-Z0-9_-]*$/.test(cle);
}

/** Rend le fragment `.ftl` trié — à RELIRE avant d'être collé, jamais écrit d'office. */
export function fragmentFtl(/** @type {Moisson} */ moisson, /** @type {string} */ date) {
	/** @type {string[]} */
	const corps = [];
	let emises = 0;
	let conflitsEmis = 0;
	for (const [cle, parTexte] of [...moisson.replis].sort(([a], [b]) => a.localeCompare(b))) {
		const texte = [...parTexte.keys()][0];
		if (!estCleFtlSaine(cle) || !estFtlSain(texte)) {
			// Écarté du fragment plutôt qu'injecté tel quel — et RECENSÉ dans `aEchapper`,
			// que le script imprime sur la sortie d'erreur. Rien ne disparaît en silence.
			continue;
		}
		if (parTexte.size > 1) {
			corps.push(`# CONFLIT — ${parTexte.size} replis, cf. sortie d'erreur`);
			conflitsEmis += 1;
		}
		corps.push(`${cle} = ${texte}`);
		emises += 1;
	}
	// ⚠️ L'en-tête compte les clés RÉELLEMENT ÉMISES, pas les clés moissonnées : compter les
	// secondes faisait annoncer « 2 clés » à un fragment qui en portait une. (Passe 3.)
	return [
		`# Fragment moissonné le ${date} — À RELIRE AVANT DE COLLER.`,
		`# ${emises} clés émises ; ${conflitsEmis} portent la mention CONFLIT ; ` +
			`${moisson.replis.size - emises} écartées, cf. sortie d'erreur.`,
		...corps
	].join('\n');
}
