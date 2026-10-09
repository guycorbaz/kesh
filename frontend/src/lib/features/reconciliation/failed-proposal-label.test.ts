// Story 15-5c (AC1, AC2, #492) — libellés des refus d'un lot de rapprochement.
//
// ⚠️ La liste des codes est ÉCRITE EN DUR, et non lue depuis le module : un test qui
// itérerait sur la table de production serait vert par construction (fiche, § « Tests — ce qui
// rendrait un test vert sans rien prouver »). Relevé : les 25 littéraux de
// `crates/kesh-api/src/routes/reconciliation.rs` + les deux codes de `DbError::error_code()` :
// `ACCOUNT_NOT_POSTABLE` et `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (Story 15-6b).

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

import {
	failedProposalLabel,
	failureReason,
	rejectedAccountNumbers,
} from './failed-proposal-label';

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
	SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT: 'reconciliation-failed-counterparty-is-claim-account',
	VALIDATION_ERROR: 'error-validation',
};

describe('failedProposalLabel', () => {
	beforeEach(() => {
		calls.length = 0;
	});

	it('couvre les 27 codes relevés', () => {
		expect(Object.keys(EXPECTED_KEYS)).toHaveLength(27);
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

	// Revue de code P1 (E1, choix C-15-5c-3) : la proposition peut offrir une facture
	// postérieure au paiement que l'acceptation refuse (#548) ; cette raison-là a son libellé.
	it('RECONCILIATION_INVOICE_NOT_ELIGIBLE : libellé dédié au paiement antérieur à la facture', () => {
		const label = failedProposalLabel('RECONCILIATION_INVOICE_NOT_ELIGIBLE', {
			reason: 'payment_date_before_invoice_date',
		});
		expect(calls).toEqual(['reconciliation-failed-payment-before-invoice']);
		expect(label).toContain('plus d’un jour avant la facture');
	});

	// Story 15-6b (#474, AC7) — deux remèdes selon `details.role`.
	it('SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT, role counterparty : relier le compte bancaire', () => {
		const label = failedProposalLabel('SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT', {
			bankAccountId: 3,
			accountId: 7,
			accountNumber: '1100',
			claim: 'receivable',
			role: 'counterparty',
		});
		expect(calls).toEqual(['reconciliation-failed-counterparty-is-claim-account']);
		expect(label).toContain('propre compte de banque');
	});

	it('SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT, role rounding : renvoi aux réglages (mutation : rôle ignoré)', () => {
		const label = failedProposalLabel('SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT', {
			accountId: 7,
			accountNumber: '1100',
			claim: 'receivable',
			role: 'rounding',
		});
		expect(calls).toEqual(['reconciliation-failed-rounding-account-is-claim-account']);
		expect(label).toContain('Paramètres → Facturation');
	});

	it.each([
		['role non chaîne', { role: 1 }],
		['details absent', null],
	])('SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT (%s) → libellé du compte bancaire', (_cas, details) => {
		failedProposalLabel('SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT', details);
		expect(calls).toEqual(['reconciliation-failed-counterparty-is-claim-account']);
	});

	it.each([
		['invoice_already_paid', { reason: 'invoice_already_paid' }],
		['payment_date_outside_window', { reason: 'payment_date_outside_window', window_days: 30 }],
		['raison non chaîne', { reason: 1 }],
		['details absent', null],
	])('RECONCILIATION_INVOICE_NOT_ELIGIBLE (%s) garde le libellé générique', (_cas, details) => {
		failedProposalLabel('RECONCILIATION_INVOICE_NOT_ELIGIBLE', details);
		expect(calls).toEqual(['reconciliation-errors-invoice-not-eligible']);
	});
});

describe('failureReason', () => {
	it('ne rend que la raison chaîne d’un objet', () => {
		expect(failureReason({ reason: 'x' })).toBe('x');
		expect(failureReason({ reason: 3 })).toBeNull();
		expect(failureReason('x')).toBeNull();
		expect(failureReason(null)).toBeNull();
		expect(failureReason([])).toBeNull();
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
