// Story 15-5b (AC13, choix C12 et C26) — tests Vitest de la page des comptes
// bancaires. Aucun test de page n'existait.
//
// Le défaut couvert est celui de #271, sur deux des trois surfaces du compte
// lié : un compte comptable lié AVANT de devenir non imputable (scindé en
// sous-comptes, règle 14-3a) disparaissait des options filtrées
// `active && postable`. Svelte 5 pose alors `selectedIndex = -1` — le champ
// s'affiche VIDE pour un lien qui existe — sans réécrire la variable.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe
// `+` dans `src/routes/` (patron des autres tests de page, `accounts-page.test.ts`).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));

const listBankAccountsMock = vi.fn<() => Promise<BankAccountSummary[]>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => listBankAccountsMock(),
	createBankAccount: vi.fn(),
	updateBankAccount: vi.fn(),
	archiveBankAccount: vi.fn(),
	updateBankAccountJournalLink: vi.fn(),
}));

import Page from './+page.svelte';

function account(overrides: Partial<AccountResponse>): AccountResponse {
	return {
		id: 0,
		companyId: 1,
		number: '',
		name: '',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

/** 1020 lié, puis devenu non imputable ; 1021 est son sous-compte imputable. */
const LINKED_NON_POSTABLE = account({ id: 20, number: '1020', name: 'Banque', postable: false });
const accounts: AccountResponse[] = [
	LINKED_NON_POSTABLE,
	account({ id: 21, number: '1021', name: 'Banque BCV', parentId: 20 }),
];

const bankAccount: BankAccountSummary = {
	id: 5,
	bankName: 'BCV',
	iban: 'CH9300762011623852957',
	qrIban: null,
	isPrimary: true,
	journalAccountId: LINKED_NON_POSTABLE.id,
	version: 2,
	archived: false,
	currentBalance: '0.00',
	lastTransactionDate: null,
	statementClosingBalance: null,
	statementDate: null,
	ledgerBalanceAtStatement: null,
};

describe('page des comptes bancaires — compte lié devenu non imputable (Story 15-5b, AC13)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue(accounts);
		listBankAccountsMock.mockResolvedValue([bankAccount]);
	});

	it('reste affiché et sélectionné dans le formulaire de modification', async () => {
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('edit-button-5'));
		const select = (await findByTestId('edit-journal-account')) as HTMLSelectElement;
		expect(select.selectedIndex).toBeGreaterThan(-1);
		expect(select.value).toBe(String(LINKED_NON_POSTABLE.id));
		expect(select.selectedOptions[0].textContent).toContain('1020');
	});

	it('reste affiché et sélectionné dans le formulaire de lien', async () => {
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('link-button-5'));
		const select = (await findByTestId('journal-account-select')) as HTMLSelectElement;
		expect(select.selectedIndex).toBeGreaterThan(-1);
		expect(select.selectedOptions[0].textContent).toContain('1020');
	});

	it("n'offre pas le compte non imputable quand il n'est pas la valeur en place", async () => {
		listBankAccountsMock.mockResolvedValue([{ ...bankAccount, journalAccountId: null }]);
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('edit-button-5'));
		const select = (await findByTestId('edit-journal-account')) as HTMLSelectElement;
		const labels = Array.from(select.options).map((o) => o.textContent ?? '');
		expect(labels.some((l) => l.includes('1020'))).toBe(false);
		expect(labels.some((l) => l.includes('1021'))).toBe(true);
	});
});
