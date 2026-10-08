// Story 15-5c (AC1, AC2, #492) — libellés des refus d'un lot de rapprochement.
//
// ⚠️ La liste des codes est ÉCRITE EN DUR, et non lue depuis le module : un test qui
// itérerait sur la table de production serait vert par construction (fiche, § « Tests — ce qui
// rendrait un test vert sans rien prouver »). Relevé : les 25 littéraux de
// `crates/kesh-api/src/routes/reconciliation.rs` + `ACCOUNT_NOT_POSTABLE` (DbError::error_code).

import { describe, it, expect, vi, beforeEach } from 'vitest';

const calls: string[] = [];
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) => {
		calls.push(key);
		return args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? ''))
			: fallback;
	},
}));

import { failedProposalLabel, rejectedAccountNumbers } from './failed-proposal-label';

/** Code → clé lue (réutilisée `error-*` / `reconciliation-*`, ou neuve), choix C37. */
const EXPECTED_KEYS: Record<string, string> = {
	ACCOUNT_NOT_FOUND: 'reconciliation-failed-account-not-found',
	ACCOUNT_NOT_POSTABLE: 'reconciliation-failed-account-not-postable-generic',
	BANK_ACCOUNT_NOT_CONFIGURED: 'reconciliation-failed-bank-account-not-configured',
	BANK_ACCOUNT_NOT_FOUND: 'reconciliation-failed-bank-account-not-found',
	BANK_TRANSACTION_NOT_FOUND: 'reconciliation-failed-bank-transaction-not-found',
	DATABASE_ERROR: 'reconciliation-failed-database-error',
	FISCAL_YEAR_INVALID: 'error-fiscal-year-invalid',
	INTERNAL_ERROR: 'error-internal',
	INVOICE_NOT_FOUND: 'reconciliation-failed-invoice-not-found',
	INVOICE_SALE_ENTRY_MALFORMED: 'reconciliation-failed-invoice-sale-entry-malformed',
	PERIOD_LOCKED: 'reconciliation-failed-period-locked',
	PROJECT_ARCHIVED: 'reconciliation-failed-project-archived',
	PROJECT_NOT_FOUND: 'reconciliation-failed-project-not-found',
	RECONCILIATION_ALREADY_RECONCILED: 'reconciliation-errors-already-reconciled',
	RECONCILIATION_CURRENCY_MISMATCH: 'reconciliation-failed-currency-mismatch',
	RECONCILIATION_FISCAL_YEAR_CLOSED: 'error-fiscal-year-invalid',
	RECONCILIATION_INVOICE_NOT_ELIGIBLE: 'reconciliation-errors-invoice-not-eligible',
	RECONCILIATION_OVERPAYMENT: 'reconciliation-failed-overpayment',
	RECONCILIATION_RULE_MISMATCH: 'reconciliation-failed-rule-mismatch',
	RECONCILIATION_RULE_NO_LONGER_MATCHES: 'reconciliation-failed-rule-no-longer-matches',
	RECONCILIATION_RULE_NOT_FOUND: 'reconciliation-failed-rule-not-found',
	RECONCILIATION_SCORE_TOO_LOW: 'reconciliation-failed-score-too-low',
	RECONCILIATION_SPLIT_IMBALANCE: 'reconciliation-split-error-imbalance',
	RECONCILIATION_TRANSACTION_NOT_PENDING: 'reconciliation-failed-transaction-not-pending',
	ROUNDING_ACCOUNT_NOT_CONFIGURED: 'error-rounding-account-not-configured',
	VALIDATION_ERROR: 'error-validation',
};

describe('failedProposalLabel', () => {
	beforeEach(() => {
		calls.length = 0;
	});

	it('couvre les 26 codes relevés', () => {
		expect(Object.keys(EXPECTED_KEYS)).toHaveLength(26);
	});

	for (const [code, key] of Object.entries(EXPECTED_KEYS)) {
		it(`${code} → ${key}, libellé distinct du repli`, () => {
			const label = failedProposalLabel(code);
			expect(calls).toEqual([key]);
			// Le repli cite le code brut : un libellé traduit ne le cite pas.
			expect(label).not.toContain(code);
			expect(label).not.toBe(failedProposalLabel('__CODE_INCONNU__'));
			expect(label.trim().length).toBeGreaterThan(0);
		});
	}

	it('un code inconnu rend le repli, avec le code brut', () => {
		const label = failedProposalLabel('NEW_SERVER_CODE');
		expect(calls).toEqual(['reconciliation-failed-unknown']);
		expect(label).toContain('NEW_SERVER_CODE');
	});

	it('ACCOUNT_NOT_POSTABLE nomme les numéros de details.rejected', () => {
		const label = failedProposalLabel('ACCOUNT_NOT_POSTABLE', {
			rejected: [
				{ accountId: 1, accountNumber: '1000' },
				{ accountId: 2, accountNumber: '3000' },
			],
		});
		expect(calls).toEqual(['reconciliation-failed-account-not-postable']);
		expect(label).toContain('1000, 3000');
		expect(label).not.toContain('undefined');
	});

	it.each([
		['absent', undefined],
		['null', null],
		['une chaîne', 'rejected'],
		['un objet sans rejected', { lockedThrough: '2026-01-31' }],
		['rejected non tableau', { rejected: 'x' }],
		['rejected vide', { rejected: [] }],
		['entrées sans accountNumber chaîne', { rejected: [null, 3, { accountId: 1 }, { accountNumber: 42 }] }],
	])('ACCOUNT_NOT_POSTABLE, details %s → libellé sans numéros, sans exception', (_, details) => {
		const label = failedProposalLabel('ACCOUNT_NOT_POSTABLE', details);
		expect(calls).toEqual(['reconciliation-failed-account-not-postable-generic']);
		expect(label).not.toContain('undefined');
		expect(label).not.toContain('null');
	});

	it('PERIOD_LOCKED ne lit pas son details (limite assumée, C32)', () => {
		const label = failedProposalLabel('PERIOD_LOCKED', {
			lockedThrough: '2026-03-31',
			attempted: '2026-03-15',
		});
		expect(label).not.toContain('2026-03-31');
	});
});

describe('rejectedAccountNumbers', () => {
	it('garde les seules entrées de la forme attendue', () => {
		expect(
			rejectedAccountNumbers({
				rejected: [{ accountNumber: '1000' }, { accountNumber: '' }, { accountNumber: 7 }, 'x'],
			}),
		).toEqual(['1000']);
	});
});
