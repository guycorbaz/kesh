// @vitest-environment node
/**
 * Story 34-4 (#828, AC-5) — les textes du signal « transaction déjà comptabilisée », lus au
 * **catalogue réel**, et non au seul repli.
 *
 * 1. Le repli de chaque clé `reconciliation-labels-already-booked-…` égale sa valeur fr-CH, et
 *    l'ensemble des clés relevées dans l'écran égale celui du catalogue (`ecartsAuCatalogue`,
 *    les deux sens) : un texte de l'écran ne peut pas diverger du catalogue en silence.
 * 2. Aucune de ces clés ne porte de sélecteur Fluent (`->`), dans aucune locale : le client
 *    reçoit les messages pré-résolus, un sélecteur y arriverait figé sur sa variante par défaut
 *    (choix C-e342-21).
 * 3. Les huit clés existent dans les quatre locales, et `-other` porte `{ $count }`.
 */
import { describe, it, expect } from 'vitest';
import {
	catalogueDe,
	ecartsAuCatalogue,
	releveDesReplis,
	sourcesDesDossiers,
} from '$lib/shared/i18n-repli-catalogue';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'en-CH', 'it-CH'] as const;
const PREFIXE = 'reconciliation-labels-already-booked-';

/** Les clés de la story, liste fermée. */
const CLES = [
	'reconciliation-labels-already-booked-one',
	'reconciliation-labels-already-booked-other',
	// Story 34-11b (#972, AC-10) : `-elsewhere` (b) et `-combined` (c) retirées, remplacées par
	// `-transfer` (b′) et `-transfer-among` (c′).
	'reconciliation-labels-already-booked-transfer',
	'reconciliation-labels-already-booked-transfer-among',
	'reconciliation-labels-already-booked-confirm-title',
	'reconciliation-labels-already-booked-confirm-intro',
	'reconciliation-labels-already-booked-confirm-cancel',
	'reconciliation-labels-already-booked-confirm-continue',
] as const;

describe('Déjà comptabilisée — catalogues (Story 34-4)', () => {
	it('chaque repli de l’écran égale le catalogue fr-CH, et l’écran demande toutes les clés du préfixe', () => {
		const releve = releveDesReplis(sourcesDesDossiers(['src/lib/features/reconciliation']));
		const fr = catalogueDe(RACINE_FTL, 'fr-CH');
		expect(ecartsAuCatalogue(releve, fr, PREFIXE)).toEqual([]);
		for (const cle of CLES) expect(releve.has(cle), cle).toBe(true);
	});

	it.each(LOCALES)('%s : les huit clés existent, sans sélecteur Fluent', (locale) => {
		const catalogue = catalogueDe(RACINE_FTL, locale);
		const duPrefixe = [...catalogue.keys()].filter((c) => c.startsWith(PREFIXE)).sort();
		expect(duPrefixe).toEqual([...CLES].sort());
		for (const cle of CLES) {
			const valeur = catalogue.get(cle)!;
			expect(valeur.trim(), `${locale} ${cle}`).not.toBe('');
			expect(valeur, `${locale} ${cle}`).not.toContain('->');
		}
		expect(catalogue.get('reconciliation-labels-already-booked-other')).toContain('{ $count }');
		expect(catalogue.get('reconciliation-labels-already-booked-one')).not.toContain('{ $count }');
	});
});
