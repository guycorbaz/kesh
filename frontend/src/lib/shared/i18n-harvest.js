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
		if (!dansLePerimetreDeFichier(chemin.split('/').pop() ?? chemin)) continue;
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
	const aEchapper = [...replis].filter(([, parTexte]) => !estFtlSain([...parTexte.keys()][0]));
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
	if (texte.includes('\n')) return false;
	let profondeur = 0;
	for (const c of texte) {
		if (c === '{') profondeur += 1;
		else if (c === '}') {
			profondeur -= 1;
			if (profondeur < 0) return false;
		}
	}
	return profondeur === 0;
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
	const lignes = [
		`# Fragment moissonné le ${date} — À RELIRE AVANT DE COLLER.`,
		`# ${moisson.replis.size} clés ; les ${moisson.divergents.length} à repli divergent portent la mention CONFLIT.`
	];
	for (const [cle, parTexte] of [...moisson.replis].sort(([a], [b]) => a.localeCompare(b))) {
		const texte = [...parTexte.keys()][0];
		if (!estCleFtlSaine(cle) || !estFtlSain(texte)) {
			// Écarté du fragment plutôt qu'injecté tel quel : cf. `aEchapper`.
			continue;
		}
		if (parTexte.size > 1) lignes.push(`# CONFLIT — ${parTexte.size} replis, cf. sortie d'erreur`);
		lignes.push(`${cle} = ${texte}`);
	}
	return lignes.join('\n');
}
