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
 * @typedef {{ chemin: string, source: string }} Fichier
 * @typedef {{
 *   replis: Map<string, Map<string, string[]>>,
 *   sansRepli: Map<string, string[]>,
 *   divergents: [string, Map<string, string[]>][]
 * }} Moisson
 */

/**
 * Moissonne les replis des clés **absentes du catalogue**.
 *
 * @param {Fichier[]} fichiers   les fichiers à parcourir (déjà filtrés : ni `.test.`, ni
 *                               autre extension que `.svelte`/`.ts`)
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
	return { replis, sansRepli, divergents };
}

/** Rend le fragment `.ftl` trié — à RELIRE avant d'être collé, jamais écrit d'office. */
export function fragmentFtl(/** @type {Moisson} */ moisson, /** @type {string} */ date) {
	const lignes = [
		`# Fragment moissonné le ${date} — À RELIRE AVANT DE COLLER.`,
		`# ${moisson.replis.size} clés ; les ${moisson.divergents.length} à repli divergent portent la mention CONFLIT.`
	];
	for (const [cle, parTexte] of [...moisson.replis].sort(([a], [b]) => a.localeCompare(b))) {
		if (parTexte.size > 1) lignes.push(`# CONFLIT — ${parTexte.size} replis, cf. sortie d'erreur`);
		lignes.push(`${cle} = ${[...parTexte.keys()][0]}`);
	}
	return lignes.join('\n');
}
