// Story 38-11d (AC-3 à AC-5) — libellés de l'écran PayPal : refus de l'import, genres,
// écarts, motifs, refus de l'acceptation (codes propres, délégation au reste).

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) =>
		`${key}|` + (args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback),
}));

import {
	findingGapDetails,
	findingGapLabel,
	gapKindLabel,
	paypalFailedLabel,
	paypalImportErrorLines,
	paypalLineErrorLabel,
	toReviewReasonLabel,
} from './paypal-labels';

const apiError = (code: string, details?: Record<string, unknown>, message = `message ${code}`) => ({
	code,
	status: 422,
	message,
	details,
});

describe('paypal-labels — import', () => {
	it('colonnes manquantes nommées', () => {
		const [line] = paypalImportErrorLines(
			apiError('PAYPAL_CSV_INVALID', { reason: 'PAYPAL_CSV_MISSING_COLUMNS', columns: ['Gross', 'Net'] }),
		);
		expect(line).toContain('paypal-import-error-missing-columns');
		expect(line).toContain('Gross, Net');
	});

	it('lignes en erreur : une phrase par ligne, numéro, champ et cause', () => {
		const lines = paypalImportErrorLines(
			apiError('PAYPAL_CSV_INVALID', {
				reason: 'PAYPAL_CSV_LINES',
				lines: [
					{ line: 4, field: 'Gross', code: 'AMOUNT', value: 'x' },
					{ line: 9, field: null, code: 'FIELD_COUNT', value: null },
				],
			}),
		);
		expect(lines).toHaveLength(3);
		expect(lines[1]).toContain('Ligne 4, Gross');
		expect(lines[1]).toContain('paypal-csv-line-amount');
		expect(lines[2]).toContain('Ligne 9, —');
	});

	it('ordre des dates ambigu : le remède', () => {
		const [line] = paypalImportErrorLines(apiError('PAYPAL_CSV_INVALID', { reason: 'PAYPAL_CSV_DATE_ORDER_AMBIGUOUS' }));
		expect(line).toContain('période plus longue');
	});

	it('autre refus : le message du serveur (devises nommées)', () => {
		expect(
			paypalImportErrorLines(apiError('PROVIDER_CURRENCY_ACCOUNT_MISSING', { currencies: ['GBP'] }, 'GBP sans compte')),
		).toEqual(['GBP sans compte']);
	});

	it('code de ligne inconnu : brut', () => {
		expect(paypalLineErrorLabel('NOUVEAU')).toBe('NOUVEAU');
	});
});

describe('paypal-labels — revue', () => {
	it('écarts et motifs traduits ; inconnus bruts', () => {
		expect(gapKindLabel('receipt_without_sale')).toContain('Encaissement sans vente');
		expect(gapKindLabel('sale_without_receipt')).toContain('Vente sans encaissement');
		expect(gapKindLabel('amount_or_currency_mismatch')).toContain('Montant ou devise différents');
		expect(gapKindLabel('autre')).toBe('autre');
		for (const r of ['ambiguous_sale', 'fee_account_missing', 'conversion_unpaired', 'withdrawal_foreign_currency', 'no_rule']) {
			expect(toReviewReasonLabel(r)).toMatch(/^paypal-to-review-/);
		}
		expect(toReviewReasonLabel('autre')).toBe('autre');
	});
});

describe('paypal-labels — refus de l’acceptation', () => {
	it('tous les refus du moteur commun passent par les clés provider-failed-*', () => {
		expect(paypalFailedLabel('PROVIDER_PROPOSAL_STALE')).toMatch(/^provider-failed-proposal-stale\|/);
		expect(paypalFailedLabel('NOT_FOUND')).toMatch(/^provider-failed-not-found\|/);
	});

	it('les autres codes sont délégués au rapprochement générique', () => {
		expect(paypalFailedLabel('RATE_UNAVAILABLE', { currency: 'EUR', month: '2020-03' })).not.toMatch(/^paypal-/);
		expect(paypalFailedLabel('BANK_ACCOUNT_CLOSED')).not.toMatch(/^paypal-/);
		expect(paypalFailedLabel('CODE_INCONNU')).toContain('reconciliation-failed-unknown');
	});
});

/**
 * Story 38-11d (AC-4) — les codes que la route PayPal peut poser dans `failed[]`, ÉCRITS
 * EN DUR (jamais lus depuis le module : un test qui itérerait sur la table de production
 * serait vert par construction). Relevé : les littéraux de `crates/kesh-api/src/routes/
 * paypal.rs` et `provider_accept.rs`
 * (`grep -ohE '"[A-Z][A-Z_]{4,}"' … | sort -u`, 16 codes, après la fusion de la 38-11c2,
 * `1e91465b`). Chacun a un libellé traduit : aucun ne retombe sur le repli « Refus non
 * reconnu ».
 */
const CODES_ROUTE_PAYPAL = [
	'ACCOUNT_NOT_FOUND',
	'COUNTERPARTY_CURRENCY_MISMATCH',
	'BANK_ACCOUNT_NOT_CONFIGURED',
	'BANK_TRANSACTION_NOT_FOUND',
	'DATABASE_ERROR',
	'FX_ACCOUNT_INVALID',
	'FX_RATE_INVALID',
	'NOT_FOUND',
	'OPTIMISTIC_LOCK_CONFLICT',
	'PROVIDER_PROPOSAL_STALE',
	'RATE_UNAVAILABLE',
	'RECONCILIATION_ALREADY_RECONCILED',
	'RECONCILIATION_ENTRY_NOT_LINKABLE',
	'RECONCILIATION_FISCAL_YEAR_CLOSED',
	'RECONCILIATION_TRANSACTION_NOT_PENDING',
	'VALIDATION_ERROR',
];

describe('paypal-labels — refus de la devise de la contrepartie d’une règle', () => {
	it('nomme les deux devises, par la clé commune', () => {
		const l = paypalFailedLabel('COUNTERPARTY_CURRENCY_MISMATCH', {
			accountId: 7,
			accountCurrency: 'EUR',
			transactionCurrency: 'USD',
		});
		expect(l).toMatch(/^provider-failed-counterparty-currency-mismatch\|/);
		expect(l).toContain('EUR');
		expect(l).toContain('USD');
	});
});

describe('paypal-labels — codes de la route PayPal', () => {
	it.each(CODES_ROUTE_PAYPAL)('%s a un libellé traduit', (code) => {
		expect(paypalFailedLabel(code, {})).not.toContain('reconciliation-failed-unknown');
	});
});

// Story 38-11d (revue P1, M3) — l'écriture déjà rattachée a son propre libellé ; jamais
// « trouvé » suivi de rien.
describe('paypal-labels — écart montant ou devise différents', () => {
	const mismatch = (details: Record<string, unknown>) => ({ code: 'amount_or_currency_mismatch', details });

	it('montant différent : attendu et trouvé', () => {
		const f = mismatch({ expectedAmount: '10.00', expectedCurrency: 'EUR', foundAmount: '9.00', foundCurrency: 'EUR', alreadyLinked: false });
		expect(findingGapLabel(f)).toContain('Montant ou devise différents');
		expect(findingGapDetails(f)).toContain('Attendu 10.00 EUR, trouvé 9.00 EUR');
	});

	it('déjà rattachée, même montant : son libellé, aucun détail', () => {
		const f = mismatch({ expectedAmount: '10.00', expectedCurrency: 'EUR', foundAmount: '10.00', foundCurrency: 'EUR', alreadyLinked: true });
		expect(findingGapLabel(f)).toContain('Vente déjà rattachée à une autre transaction');
		expect(findingGapDetails(f)).toBeNull();
	});

	it('déjà rattachée, autre montant : son libellé et l’écart', () => {
		const f = mismatch({ expectedAmount: '10.00', expectedCurrency: 'EUR', foundAmount: '8.00', foundCurrency: 'EUR', alreadyLinked: true });
		expect(findingGapLabel(f)).toContain('Vente déjà rattachée');
		expect(findingGapDetails(f)).toContain('trouvé 8.00 EUR');
	});

	it('aucun débit trouvé : jamais « trouvé » suivi de rien', () => {
		const f = mismatch({ expectedAmount: '10.00', expectedCurrency: 'EUR', foundAmount: null, foundCurrency: null, alreadyLinked: false });
		expect(findingGapDetails(f)).toContain('Attendu 10.00 EUR, aucun débit trouvé');
		expect(findingGapDetails(f)).not.toMatch(/trouvé\s*$/);
	});

	it('autre écart : aucun détail', () => {
		expect(findingGapDetails({ code: 'receipt_without_sale' })).toBeNull();
		expect(findingGapLabel({ code: 'receipt_without_sale' })).toContain('Encaissement sans vente');
	});
});

/**
 * Revue P1 (L3) — les codes que l'acceptation PayPal pose par conversion d'une erreur
 * d'écriture (`entry_refusal`, `provider_accept.rs` : verrou de période, exercice postérieur
 * clôturé, compte clos, refus du contrôle des lignes en devise par `fx_failed_proposal`),
 * ÉCRITS EN DUR : aucun ne retombe sur le repli.
 */
const CODES_CONVERTIS = [
	'PERIOD_LOCKED',
	'LATER_FISCAL_YEAR_CLOSED',
	'BANK_ACCOUNT_CLOSED',
	'MULTI_CURRENCY_DISABLED',
	'ACCOUNT_REQUIRES_CURRENCY',
	'FX_MULTIPLE_CURRENCIES',
	'ENTRY_IS_FX_REVALUATION',
	'FX_ACCOUNT_IN_CHF',
	'FX_CURRENCY_MISMATCH',
	'FX_RATE_NOT_AFC',
];

describe('paypal-labels — codes convertis des refus d’écriture', () => {
	it.each(CODES_CONVERTIS)('%s a un libellé traduit', (code) => {
		expect(paypalFailedLabel(code, {})).not.toContain('reconciliation-failed-unknown');
	});
});
