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
import { render, fireEvent, waitFor } from '@testing-library/svelte';
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
// Story 26-2a — les payloads de création et de modification sont inspectés.
const createBankAccountMock = vi.fn<(payload: unknown) => Promise<unknown>>();
const updateBankAccountMock = vi.fn<(id: number, payload: unknown) => Promise<unknown>>();
// Story 28-2 (revue P1 E-L5) — l'archivage est piloté pour le cas B4.
const archiveBankAccountMock = vi.fn<(id: number, version: number) => Promise<unknown>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => listBankAccountsMock(),
	createBankAccount: (payload: unknown) => createBankAccountMock(payload),
	updateBankAccount: (id: number, payload: unknown) => updateBankAccountMock(id, payload),
	archiveBankAccount: (id: number, version: number) => archiveBankAccountMock(id, version),
	updateBankAccountJournalLink: vi.fn(),
}));

// Story 15-6c (AC7) : la page lit les réglages pour écarter les comptes de
// créance désignés. Par défaut, aucun compte désigné : les tests de la 15-5b
// gardent leur sens.
const getInvoiceSettingsMock = vi.fn<() => Promise<Record<string, unknown>>>();
vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: () => getInvoiceSettingsMock(),
}));

import { toast } from 'svelte-sonner';
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
		letterable: false,
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
	description: null,
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
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: null,
			defaultPayableAccountId: null,
		});
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

describe('page des comptes bancaires — comptes de créance désignés (Story 15-6c, AC7)', () => {
	const RECEIVABLE = account({ id: 30, number: '1100', name: 'Débiteurs' });
	const PAYABLE = account({ id: 31, number: '2000', name: 'Créanciers', accountType: 'Liability' });
	const BANK = account({ id: 32, number: '1020', name: 'Banque' });

	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue([BANK, RECEIVABLE, PAYABLE]);
		listBankAccountsMock.mockResolvedValue([{ ...bankAccount, journalAccountId: null }]);
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: RECEIVABLE.id,
			defaultPayableAccountId: PAYABLE.id,
		});
	});

	/** Ouvre le formulaire `button` et rend son `<select>` (un rendu par formulaire : les modes s'excluent). */
	async function openSelect(button: string, select: string): Promise<HTMLSelectElement> {
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId(button));
		return (await findByTestId(select)) as HTMLSelectElement;
	}

	function labelsOf(select: HTMLSelectElement): string[] {
		return Array.from(select.options).map((o) => o.textContent ?? '');
	}

	// Création : revue de code P1 (A3) — le select du formulaire de création de la
	// page, ajouté aux deux autres (modification ; lien, dans BankAccountJournalLinkForm).
	it.each([
		['création', 'create-bank-account-button', 'form-journal-account'],
		['modification', 'edit-button-5', 'edit-journal-account'],
		['lien', 'link-button-5', 'journal-account-select'],
	])("le <select> du formulaire de %s n'offre ni le compte débiteurs ni le compte créanciers", async (_cas, button, select) => {
		const labels = labelsOf(await openSelect(button, select));
		expect(labels.some((l) => l.includes('1020'))).toBe(true);
		expect(labels.some((l) => l.includes('1100'))).toBe(false);
		expect(labels.some((l) => l.includes('2000'))).toBe(false);
	});

	it.each([
		['modification', 'edit-button-5', 'edit-journal-account'],
		['lien', 'link-button-5', 'journal-account-select'],
	])('un lien existant vers le compte débiteurs reste affiché (formulaire de %s)', async (_cas, button, select) => {
		listBankAccountsMock.mockResolvedValue([{ ...bankAccount, journalAccountId: RECEIVABLE.id }]);
		const el = await openSelect(button, select);
		await waitFor(() => expect(el.selectedIndex).toBeGreaterThan(-1));
		expect(el.selectedOptions[0].textContent).toContain('1100');
	});

	it('réglages illisibles : menus non filtrés, page affichée', async () => {
		getInvoiceSettingsMock.mockRejectedValue(new Error('403'));
		const labels = labelsOf(await openSelect('edit-button-5', 'edit-journal-account'));
		expect(labels.some((l) => l.includes('1100'))).toBe(true);
		expect(labels.some((l) => l.includes('2000'))).toBe(true);
	});
});

describe('page des comptes bancaires — description (Story 26-2a, AC7)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue(accounts);
		listBankAccountsMock.mockResolvedValue([bankAccount]);
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: null,
			defaultPayableAccountId: null,
		});
		createBankAccountMock.mockResolvedValue({});
		updateBankAccountMock.mockResolvedValue({});
	});

	async function fill(el: HTMLElement, value: string) {
		await fireEvent.input(el, { target: { value } });
	}

	async function openCreateAndFill(description: string) {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fill(await r.findByTestId('form-bank-name'), 'UBS');
		await fill(await r.findByTestId('form-description'), description);
		await fill(await r.findByTestId('form-iban'), 'CH3908704016075473007');
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(createBankAccountMock).toHaveBeenCalledTimes(1));
		return createBankAccountMock.mock.calls[0][0] as Record<string, unknown>;
	}

	it('création avec description : le payload la porte', async () => {
		const payload = await openCreateAndFill('Épargne');
		expect(payload.description).toBe('Épargne');
	});

	it('création sans description : le champ est présent, à null', async () => {
		const payload = await openCreateAndFill('   ');
		expect('description' in payload).toBe(true);
		expect(payload.description).toBeNull();
	});

	it('le champ de création borne la saisie à 60 caractères et porte son aide', async () => {
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('create-bank-account-button'));
		expect((await findByTestId('form-description')).getAttribute('maxlength')).toBe('60');
		expect((await findByTestId('form-description-help')).textContent).toContain(
			'60 caractères au plus',
		);
	});

	it('édition : préremplit la description, puis l’efface (payload null)', async () => {
		listBankAccountsMock.mockResolvedValue([{ ...bankAccount, description: 'Personnel' }]);
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('edit-button-5'));
		const input = (await findByTestId('edit-description')) as HTMLInputElement;
		expect(input.value).toBe('Personnel');
		expect(input.getAttribute('maxlength')).toBe('60');
		await fill(input, '');
		await fireEvent.click(await findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const [id, payload] = updateBankAccountMock.mock.calls[0] as [number, Record<string, unknown>];
		expect(id).toBe(5);
		expect('description' in payload).toBe(true);
		expect(payload.description).toBeNull();
	});

	it('édition qui garde la description : le payload la renvoie', async () => {
		listBankAccountsMock.mockResolvedValue([{ ...bankAccount, description: 'Personnel' }]);
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('edit-button-5'));
		await fireEvent.click(await findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const [, payload] = updateBankAccountMock.mock.calls[0] as [number, Record<string, unknown>];
		expect(payload.description).toBe('Personnel');
	});

	it('la cellule « Banque » nomme le compte avec sa description', async () => {
		listBankAccountsMock.mockResolvedValue([{ ...bankAccount, description: 'Personnel' }]);
		const { findByTestId } = render(Page);
		expect((await findByTestId('bank-account-name-5')).textContent).toBe('BCV — Personnel');
	});

	it('sans description, la cellule affiche la banque seule', async () => {
		const { findByTestId } = render(Page);
		expect((await findByTestId('bank-account-name-5')).textContent).toBe('BCV');
	});
});

// Story 28-2 (#520, AC9 — B1) — un rejet du chargement de la liste qui n'est pas
// une `ApiError` affiche le générique traduit, ni « Error: boom » ni
// « [object Object] » (site `Promise.allSettled`, `baResult.reason`).
describe('page des comptes bancaires — échec imprévu du chargement (Story 28-2, B1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
		fetchAccountsMock.mockResolvedValue(accounts);
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: null,
			defaultPayableAccountId: null,
		});
	});

	it('rejet non-ApiError de listBankAccounts → le générique', async () => {
		listBankAccountsMock.mockRejectedValue(new Error('boom'));
		const { findByTestId } = render(Page);
		const erreur = await findByTestId('load-error');
		expect(erreur.textContent).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
		expect(erreur.textContent).not.toContain('[object Object]');
	});

	it('rejet ApiError de listBankAccounts → son message', async () => {
		listBankAccountsMock.mockRejectedValue({ code: 'FORBIDDEN', message: 'Accès refusé.', status: 403 });
		const { findByTestId } = render(Page);
		expect((await findByTestId('load-error')).textContent).toBe('Accès refusé.');
	});
});

// Story 28-2 (revue P1 E-L5 — B2, B3, B4) — les trois autres sites du fichier, sur
// un rejet qui n'est pas une `ApiError` : création, modification, archivage.
describe('page des comptes bancaires — échec imprévu d’une écriture (Story 28-2, B2-B4)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
		fetchAccountsMock.mockResolvedValue(accounts);
		listBankAccountsMock.mockResolvedValue([bankAccount]);
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: null,
			defaultPayableAccountId: null,
		});
	});

	async function fill(el: HTMLElement, value: string) {
		await fireEvent.input(el, { target: { value } });
	}

	it('B2 — création rejetée hors ApiError → le générique dans le formulaire', async () => {
		createBankAccountMock.mockRejectedValue(new Error('boom'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fill(await r.findByTestId('form-bank-name'), 'UBS');
		await fill(await r.findByTestId('form-iban'), 'CH3908704016075473007');
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(createBankAccountMock).toHaveBeenCalledTimes(1));
		const erreur = await r.findByTestId('form-error');
		expect(erreur.textContent).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
	});

	it('B3 — modification rejetée hors ApiError → le générique dans le formulaire', async () => {
		updateBankAccountMock.mockRejectedValue(new TypeError('Failed to fetch'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-5'));
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const erreur = await r.findByTestId('edit-form-error');
		expect(erreur.textContent).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('Failed to fetch');
	});

	it('B4 — archivage rejeté hors ApiError → le générique en toast', async () => {
		archiveBankAccountMock.mockRejectedValue(new Error('boom'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('archive-button-5'));
		await fireEvent.click(await r.findByTestId('archive-confirm-button'));
		await waitFor(() => expect(archiveBankAccountMock).toHaveBeenCalledWith(5, 2));
		await waitFor(() => expect(toast.error).toHaveBeenCalledWith('Erreur inattendue.'));
	});
});
