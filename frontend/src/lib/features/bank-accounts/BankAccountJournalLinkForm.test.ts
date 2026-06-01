// Story 8-5a-zero — Tests Vitest pour `BankAccountJournalLinkForm.svelte`.
//
// Couvre :
// - Filtre dropdown classe 1/2 actifs Asset|Liability.
// - Submit désactivé sur no-op (selection identique à valeur initiale).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { BankAccountSummary } from './bank-accounts.api';

vi.mock('./bank-accounts.api', () => ({
	updateBankAccountJournalLink: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import BankAccountJournalLinkForm from './BankAccountJournalLinkForm.svelte';

function makeAccount(
	id: number,
	number: string,
	name: string,
	accountType: AccountResponse['accountType'],
	active = true,
): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name,
		accountType,
		parentId: null,
		active,
		version: 1,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z',
	};
}

function makeBankAccount(journalAccountId: number | null): BankAccountSummary {
	return {
		id: 17,
		bankName: 'UBS',
		iban: 'CH4431999123000889012',
		qrIban: null,
		isPrimary: true,
		journalAccountId,
		version: 3,
		archived: false,
		currentBalance: null,
		lastTransactionDate: null,
	};
}

describe('BankAccountJournalLinkForm', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.resetAllMocks();
	});

	it('filters dropdown to Asset|Liability accounts class 1 or 2', () => {
		const accounts: AccountResponse[] = [
			makeAccount(1, '1020', 'Caisse banque', 'Asset'),
			makeAccount(2, '1030', 'Banque', 'Asset'),
			makeAccount(3, '2100', 'Banque (découvert)', 'Liability'),
			// Ne devrait PAS apparaître :
			makeAccount(4, '3000', 'Ventes', 'Revenue'), // Revenue
			makeAccount(5, '4000', 'Achats', 'Expense'), // Expense
			makeAccount(6, '5000', 'Charges', 'Asset'), // class 5 (pas 1 ou 2)
			makeAccount(7, '1099', 'Archived asset', 'Asset', false), // archivé
		];

		const { container } = render(BankAccountJournalLinkForm, {
			props: {
				bankAccount: makeBankAccount(null),
				accounts,
				onSuccess: vi.fn(),
				onCancel: vi.fn(),
			},
		});

		const select = container.querySelector(
			'[data-testid="journal-account-select"]',
		) as HTMLSelectElement;
		expect(select).not.toBeNull();
		const optionTexts = Array.from(select.options).map((o) => o.textContent ?? '');
		// Une option « Non configuré » + 3 comptes éligibles.
		expect(select.options.length).toBe(4);
		expect(optionTexts.some((t) => t.includes('1020'))).toBe(true);
		expect(optionTexts.some((t) => t.includes('1030'))).toBe(true);
		expect(optionTexts.some((t) => t.includes('2100'))).toBe(true);
		expect(optionTexts.some((t) => t.includes('3000'))).toBe(false);
		expect(optionTexts.some((t) => t.includes('4000'))).toBe(false);
		expect(optionTexts.some((t) => t.includes('5000'))).toBe(false);
		expect(optionTexts.some((t) => t.includes('1099'))).toBe(false);
	});

	it('disables submit when selection equals initial value (no-op)', () => {
		const accounts: AccountResponse[] = [
			makeAccount(1, '1020', 'Caisse banque', 'Asset'),
		];

		// bank_account déjà lié à id=1 → la sélection initiale est 1, no-op
		// par défaut au premier render.
		const { container } = render(BankAccountJournalLinkForm, {
			props: {
				bankAccount: makeBankAccount(1),
				accounts,
				onSuccess: vi.fn(),
				onCancel: vi.fn(),
			},
		});

		const submitBtn = container.querySelector(
			'[data-testid="submit-link"]',
		) as HTMLButtonElement;
		expect(submitBtn).not.toBeNull();
		expect(submitBtn.disabled).toBe(true);
	});

	it('shows unlink button only when bank_account is currently linked', () => {
		const accounts: AccountResponse[] = [
			makeAccount(1, '1020', 'Caisse banque', 'Asset'),
		];

		// Cas non lié : pas de bouton « Délier ».
		const { container: c1 } = render(BankAccountJournalLinkForm, {
			props: {
				bankAccount: makeBankAccount(null),
				accounts,
				onSuccess: vi.fn(),
				onCancel: vi.fn(),
			},
		});
		expect(c1.querySelector('[data-testid="unlink-button"]')).toBeNull();

		// Cas lié : bouton « Délier » présent.
		const { container: c2 } = render(BankAccountJournalLinkForm, {
			props: {
				bankAccount: makeBankAccount(1),
				accounts,
				onSuccess: vi.fn(),
				onCancel: vi.fn(),
			},
		});
		expect(c2.querySelector('[data-testid="unlink-button"]')).not.toBeNull();
	});
});
