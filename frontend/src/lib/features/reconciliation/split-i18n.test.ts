// @vitest-environment node
/**
 * Story 36-2 (#1150, D3, D5) — les textes du dialogue *Éclater*, lus au **catalogue réel**.
 *
 * Le catalogue l'emporte sur le repli (`i18n.svelte.ts` : `_messages[key] || fallback`) : un
 * repli qui seul porte le montant attendu et l'écart (`Balance non équilibrée — attendu …`)
 * ne s'affiche jamais, et l'écran ne disait que « Balance ». D'où :
 *
 * 1. le repli de chaque clé `reconciliation-split-*` du dialogue égale sa valeur fr-CH — dans
 *    ce sens seulement : d'autres sites lisent des clés du préfixe
 *    (`reconciliation-split-button-label` par `ReconciliationProposals.svelte`,
 *    `reconciliation-split-error-imbalance` par `failed-proposal-label.ts`) ;
 * 2. les clés de la story existent dans les quatre locales, avec leurs variables ;
 * 3. `reconciliation-split-error-imbalance`, message de la `400 RECONCILIATION_SPLIT_IMBALANCE`
 *    (`errors.rs`) et de l'étiquette d'échec partiel, ne porte **aucune** variable : ces trois
 *    sites — dont le dialogue, quand le montant de la transaction est illisible (C-36-2-21) —
 *    l'appellent sans arguments (validation P1, V1-1).
 */
import { describe, it, expect } from 'vitest';
import { catalogueDe, ecartsAuCatalogue, releveDesReplis } from '$lib/shared/i18n-repli-catalogue';
import { readFileSync } from 'node:fs';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LOCALES = ['fr-CH', 'de-CH', 'en-CH', 'it-CH'] as const;
const PREFIXE = 'reconciliation-split-';
const DIALOGUE = 'src/lib/features/reconciliation/TransactionSplitModal.svelte';

/** Les clés de la story et leurs variables (D3, D5). */
const CLES: Array<[string, string[]]> = [
	['reconciliation-split-balance-indicator', ['expected']],
	['reconciliation-split-balance-gap', ['expected', 'diff']],
	// Revue P1 (B3) : l'écart non calculable se dit en toutes lettres.
	['reconciliation-split-balance-incomplete', ['expected']],
	['reconciliation-split-error-imbalance-detail', ['expected', 'diff']],
	['reconciliation-split-error-amount-unreadable', ['line']],
	['reconciliation-split-error-amount-ambiguous', ['line']],
	['reconciliation-split-error-amount-cents', ['line']],
];

describe('Éclater — catalogues (Story 36-2)', () => {
	it('chaque repli du dialogue égale sa valeur fr-CH', () => {
		const releve = releveDesReplis([{ chemin: DIALOGUE, texte: readFileSync(DIALOGUE, 'utf-8') }]);
		const fr = catalogueDe(RACINE_FTL, 'fr-CH');
		const ecarts = ecartsAuCatalogue(releve, fr, PREFIXE).filter(
			(e) => !e.endsWith('demandée par aucun site littéral'),
		);
		expect(ecarts).toEqual([]);
		for (const [cle] of CLES) expect(releve.has(cle), cle).toBe(true);
	});

	it.each(LOCALES)('%s : les clés de la story existent, avec leurs variables', (locale) => {
		const catalogue = catalogueDe(RACINE_FTL, locale);
		for (const [cle, variables] of CLES) {
			const valeur = catalogue.get(cle);
			expect(valeur, `${locale} ${cle}`).toBeTruthy();
			for (const v of variables) expect(valeur, `${locale} ${cle}`).toContain(`{ $${v} }`);
		}
	});

	it.each(LOCALES)('%s : reconciliation-split-error-imbalance ne porte aucune variable', (locale) => {
		const valeur = catalogueDe(RACINE_FTL, locale).get('reconciliation-split-error-imbalance');
		expect(valeur?.trim(), locale).toBeTruthy();
		expect(valeur, locale).not.toContain('{');
	});
});
