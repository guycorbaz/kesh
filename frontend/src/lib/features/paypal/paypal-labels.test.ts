// Story 38-11d (AC-3 à AC-5) — libellés de l'écran PayPal : refus de l'import, genres,
// écarts, motifs, refus de l'acceptation (codes propres, délégation au reste).

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) =>
		`${key}|` + (args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback),
}));

import {
	gapKindLabel,
	paypalFailedLabel,
	paypalImportErrorLines,
	paypalLineErrorLabel,
	PAYPAL_OWN_FAILED_CODES,
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
	it('codes propres : écrits en dur, traduits par paypal-failed-*', () => {
		expect([...PAYPAL_OWN_FAILED_CODES]).toEqual([
			'PROVIDER_PROPOSAL_STALE',
			'FX_ACCOUNT_INVALID',
			'FX_RATE_INVALID',
			'NOT_FOUND',
			'OPTIMISTIC_LOCK_CONFLICT',
			'RECONCILIATION_ENTRY_NOT_LINKABLE',
		]);
		expect(paypalFailedLabel('PROVIDER_PROPOSAL_STALE')).toMatch(/^paypal-failed-proposal-stale\|/);
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
 * (`grep -ohE '"[A-Z][A-Z_]{4,}"' … | sort -u`, 15 codes, après la fusion de la 38-11c2,
 * `ecd54a38`). Chacun a un libellé traduit : aucun ne retombe sur le repli « Refus non
 * reconnu ».
 */
const CODES_ROUTE_PAYPAL = [
	'ACCOUNT_NOT_FOUND',
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

describe('paypal-labels — codes de la route PayPal', () => {
	it.each(CODES_ROUTE_PAYPAL)('%s a un libellé traduit', (code) => {
		expect(paypalFailedLabel(code, {})).not.toContain('reconciliation-failed-unknown');
	});
});
