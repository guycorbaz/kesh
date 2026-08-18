/**
 * **Les clés que le code DEMANDE existent-elles dans le catalogue ?** — passe 4 (#301).
 *
 * ⚠️ `i18nMsg(clé, repli)` retombe **silencieusement** sur son repli — français —
 * si la clé n'existe pas. Rien ne rougit : ni `npm run check`, ni
 * `lint-i18n-ownership` (qui contrôle l'*appartenance* d'un namespace à un
 * dossier, jamais l'*existence* d'une clé), ni l'E2E, qui tourne en français où
 * le repli est indiscernable de la traduction.
 *
 * Le dispositif anti-doublon repose donc sur des noms de clés **recopiés à
 * l'identique de part et d'autre de la frontière HTTP**, au seul endroit où
 * personne ne regardait. Une faute de frappe faisait lire le français à un
 * utilisateur germanophone, tous gates verts.
 *
 * ⚠️ **Portée délibérément bornée au domaine `contact-duplicate-*`**, celui que
 * cette story possède. Le contrôle général vaudrait pour tout le frontend, mais
 * il y a **250 clés** aujourd'hui employées sans exister au catalogue — un état
 * antérieur à cette story, de la famille de la KF #283, qu'il ne revient pas à
 * cette PR de trancher.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'];
const RACINE = '../crates/kesh-i18n/locales';
const DOMAINE = /^contact-duplicate-/;

/** Les clés déclarées dans un `.ftl` — une clé ouvre une ligne NON indentée. */
function clesDe(locale: string): Set<string> {
	const texte = readFileSync(join(RACINE, locale, 'messages.ftl'), 'utf-8');
	const cles = new Set<string>();
	for (const ligne of texte.split('\n')) {
		const m = /^([a-zA-Z][\w-]*)\s*=/.exec(ligne);
		if (m) cles.add(m[1]);
	}
	return cles;
}

/** Les clés du domaine effectivement DEMANDÉES par du code (hors tests). */
function clesDemandees(): Map<string, string> {
	const trouvees = new Map<string, string>();
	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) {
				parcourir(chemin);
				continue;
			}
			if (!/\.(svelte|ts)$/.test(e.name) || e.name.includes('.test.')) continue;
			const texte = readFileSync(chemin, 'utf-8');
			for (const m of texte.matchAll(/i18nMsg\(\s*'([^']+)'/g)) {
				if (DOMAINE.test(m[1]) && !trouvees.has(m[1])) trouvees.set(m[1], chemin);
			}
		}
	};
	parcourir('src');
	return trouvees;
}

describe('les clés `contact-duplicate-*` demandées par le code', () => {
	it('existent toutes, et dans les QUATRE locales', () => {
		const demandees = clesDemandees();
		// Sans cette borne, le test passerait à vide si le motif cessait de matcher.
		expect(demandees.size).toBeGreaterThanOrEqual(5);

		const manquantes: string[] = [];
		for (const locale of LOCALES) {
			const catalogue = clesDe(locale);
			for (const [cle, ou] of demandees) {
				if (!catalogue.has(cle)) manquantes.push(`${cle} absente de ${locale} (demandée par ${ou})`);
			}
		}
		expect(manquantes).toEqual([]);
	});

	it('le catalogue n’a pas de clé du domaine que PERSONNE ne demande', () => {
		// L'autre sens : une clé orpheline est du poids mort qu'on traduit en pure
		// perte dans quatre locales. Moins grave, mais aussi peu visible.
		const demandees = new Set(clesDemandees().keys());
		const orphelines = [...clesDe('fr-CH')].filter((k) => DOMAINE.test(k) && !demandees.has(k));
		expect(orphelines).toEqual([]);
	});
});
