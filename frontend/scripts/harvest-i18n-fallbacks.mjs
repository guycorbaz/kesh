#!/usr/bin/env node
/**
 * Enveloppe en ligne de commande du moissonneur (story 23-1b, AC10).
 *
 * La logique vit dans `src/lib/shared/i18n-harvest.js` — importable par vitest ET par
 * `node` —, parce qu'un script de `scripts/` n'est exécuté par aucun gate : sa substance
 * n'aurait alors pour preuve que des nombres recopiés à la main.
 *
 * Usage : `node scripts/harvest-i18n-fallbacks.mjs [préfixe…]`
 *   Le fragment `.ftl` part sur la sortie STANDARD ; ce qui demande un arbitrage humain
 *   — replis interpolés, replis divergents — part sur la sortie d'ERREUR.
 */

import { readFileSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { moissonner, fragmentFtl, dansLePerimetreDeFichier } from '../src/lib/shared/i18n-harvest.js';

const RACINE = join(dirname(fileURLToPath(import.meta.url)), '..');
const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'];
const RACINE_FTL = join(RACINE, '..', 'crates', 'kesh-i18n', 'locales');

const catalogues = LOCALES.map((locale) => {
	const cles = new Set();
	for (const ligne of readFileSync(join(RACINE_FTL, locale, 'messages.ftl'), 'utf-8').split('\n')) {
		const m = /^([a-zA-Z][\w-]*)\s*=/.exec(ligne);
		if (m) cles.add(m[1]);
	}
	return cles;
});

/** Parcourt `src` en excluant les fichiers de test (23-1a, D5-bis). */
function fichiers(rep, acc = []) {
	for (const e of readdirSync(rep, { withFileTypes: true })) {
		const chemin = join(rep, e.name);
		if (e.isDirectory()) fichiers(chemin, acc);
		else if (dansLePerimetreDeFichier(e.name)) {
			acc.push({ chemin: chemin.slice(RACINE.length + 1), source: readFileSync(chemin, 'utf-8') });
		}
	}
	return acc;
}

const moisson = moissonner(
	fichiers(join(RACINE, 'src')),
	(cle) => catalogues.some((c) => c.has(cle)),
	process.argv.slice(2)
);

console.log(fragmentFtl(moisson, new Date().toISOString().slice(0, 10)));

const err = (s) => process.stderr.write(s + '\n');
err('');
err(`── ${moisson.sansRepli.size} clés SANS REPLI LITTÉRAL (repli interpolé → entrée Fluent à variables) ──`);
for (const [cle, sites] of [...moisson.sansRepli].sort(([a], [b]) => a.localeCompare(b))) {
	err(`  ${cle}   ${sites.join(', ')}`);
}
err('');
err(`── ${moisson.divergents.length} clés à REPLI DIVERGENT (arbitrage humain requis) ──`);
for (const [cle, parTexte] of moisson.divergents.sort(([a], [b]) => a.localeCompare(b))) {
	err(`  ${cle}`);
	for (const [texte, sites] of parTexte) err(`      « ${texte} »   ${sites.join(', ')}`);
}
err('');
if (moisson.aEchapper.length > 0) {
	err('');
	err(`── ${moisson.aEchapper.length} clés dont le repli NE PEUT PAS entrer tel quel dans un .ftl ──`);
	err('   (retour à la ligne, ou accolade non appariée — une seule casserait le chargement de TOUTE la locale)');
	for (const [cle, parTexte] of moisson.aEchapper) err(`  ${cle}   « ${[...parTexte.keys()][0]} »`);
}
err('');
err(`── total : ${moisson.replis.size} clés moissonnées, ${moisson.sansRepli.size} sans repli, ${moisson.divergents.length} en conflit, ${moisson.aEchapper.length} à échapper ──`);
