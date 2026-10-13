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

// Story 28-4b — le repli est interpolé (`{ $date }`, `{ $amount }`), comme le
// ferait Fluent : le badge et l'avertissement de solde portent leur valeur.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));

// Story 28-4b (#642) — les deux booléens du filtre sont transmis au mock (T-b9).
const listBankAccountsMock =
	vi.fn<(includeArchived?: boolean, includeClosed?: boolean) => Promise<BankAccountSummary[]>>();
// Story 26-2a — les payloads de création et de modification sont inspectés.
const createBankAccountMock = vi.fn<(payload: unknown) => Promise<unknown>>();
const updateBankAccountMock = vi.fn<(id: number, payload: unknown) => Promise<unknown>>();
// Story 28-2 (revue P1 E-L5) — l'archivage est piloté pour le cas B4.
const archiveBankAccountMock = vi.fn<(id: number, version: number) => Promise<unknown>>();
// Story 28-4b (#642, AC11) — clôture et réouverture.
const closeBankAccountMock =
	vi.fn<(id: number, version: number, closedOn: string) => Promise<unknown>>();
const reopenBankAccountMock = vi.fn<(id: number, version: number) => Promise<unknown>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: (includeArchived?: boolean, includeClosed?: boolean) =>
		listBankAccountsMock(includeArchived, includeClosed),
	createBankAccount: (payload: unknown) => createBankAccountMock(payload),
	updateBankAccount: (id: number, payload: unknown) => updateBankAccountMock(id, payload),
	archiveBankAccount: (id: number, version: number) => archiveBankAccountMock(id, version),
	closeBankAccount: (id: number, version: number, closedOn: string) =>
		closeBankAccountMock(id, version, closedOn),
	reopenBankAccount: (id: number, version: number) => reopenBankAccountMock(id, version),
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
	closedOn: null,
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

// Story 28-4b (#642, AC11) — clore et rouvrir un compte bancaire. T-b5 à T-b9 de la
// fiche. Trois comptes synthétiques : le principal (5), un compte ouvert (6), un
// compte clos (7). IBAN de test, aucune donnée réelle.
describe('page des comptes bancaires — clôture (Story 28-4b, AC11)', () => {
	const PRINCIPAL: BankAccountSummary = { ...bankAccount, id: 5, isPrimary: true, version: 2 };
	const OUVERT: BankAccountSummary = {
		...bankAccount,
		id: 6,
		bankName: 'UBS',
		iban: 'CH5604835012345678009',
		isPrimary: false,
		journalAccountId: null,
		version: 3,
		currentBalance: '125.50',
	};
	const CLOS: BankAccountSummary = {
		...bankAccount,
		id: 7,
		bankName: 'PostFinance',
		iban: 'CH4431999123000889012',
		isPrimary: false,
		journalAccountId: null,
		version: 4,
		closedOn: '2026-09-30',
	};

	/** La date locale du jour, `AAAA-MM-JJ` (ce que le champ doit préremplir). */
	function aujourdhui(): string {
		const d = new Date();
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	}

	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
		fetchAccountsMock.mockResolvedValue(accounts);
		listBankAccountsMock.mockResolvedValue([PRINCIPAL, OUVERT, CLOS]);
		getInvoiceSettingsMock.mockResolvedValue({
			defaultReceivableAccountId: null,
			defaultPayableAccountId: null,
		});
		closeBankAccountMock.mockResolvedValue({});
		reopenBankAccountMock.mockResolvedValue({});
	});

	// T-b5 — une ligne close : badge daté au format suisse, seule action « Rouvrir ».
	it('une ligne close porte le badge daté et la seule action Rouvrir', async () => {
		const r = render(Page);
		const badge = await r.findByTestId('closed-badge-7');
		expect(badge.textContent?.trim()).toBe('Clos le 30.09.2026');
		expect(r.getByTestId('reopen-7')).toBeTruthy();
		for (const absent of ['close-7', 'link-button-7', 'edit-button-7', 'archive-button-7']) {
			expect(r.queryByTestId(absent)).toBeNull();
		}
		// Atténuée comme une archivée — ses cellules de données, non son action : « Rouvrir »
		// reste pleinement lisible (recette visuelle, C-28-4-46).
		expect(r.getByTestId('balance-cell-7').className).toContain('opacity-50');
		expect(r.getByTestId('reopen-7').closest('td')?.className).not.toContain('opacity-50');
		expect(r.getByTestId('bank-account-row-7').className).not.toContain('opacity-50');
		// Une ligne ouverte : « Clore », et pas « Rouvrir ».
		expect(r.getByTestId('close-6')).toBeTruthy();
		expect(r.queryByTestId('reopen-6')).toBeNull();
		expect(r.queryByTestId('closed-badge-6')).toBeNull();
	});

	// T-b6 — le principal : « Clore » désactivé, expliqué en `title` et en aide.
	it('sur le compte principal, Clore est désactivé et expliqué', async () => {
		const r = render(Page);
		const bouton = (await r.findByTestId('close-5')) as HTMLButtonElement;
		expect(bouton.disabled).toBe(true);
		expect(bouton.getAttribute('title')).toContain("désignez d'abord un autre compte principal");
		expect(r.getByTestId('close-primary-hint-5').textContent).toContain(
			"désignez d'abord un autre compte principal",
		);
		expect((r.getByTestId('close-6') as HTMLButtonElement).disabled).toBe(false);
		expect(r.queryByTestId('close-primary-hint-6')).toBeNull();
	});

	// T-b7 — dialogue : date locale préremplie ; avertissement si solde ≠ 0.
	it('le dialogue préremplit la date du jour et avertit d’un solde non nul', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		const date = (await r.findByTestId('close-date')) as HTMLInputElement;
		expect(date.type).toBe('date');
		expect(date.value).toBe(aujourdhui());
		const avertissement = r.getByTestId('close-balance-warning');
		expect(avertissement.textContent).toContain('125.50');
		expect(avertissement.textContent).toContain('vérifiez-le avant de clore');
	});

	it.each([
		['nul', '0.00'],
		['nul négatif', '-0.00'],
		['absent', null],
	])('solde %s : aucun avertissement', async (_cas, solde) => {
		listBankAccountsMock.mockResolvedValue([PRINCIPAL, { ...OUVERT, currentBalance: solde }]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		await r.findByTestId('close-confirm');
		expect(r.queryByTestId('close-balance-warning')).toBeNull();
	});

	// T-b3 côté écran — la date saisie part au serveur avec la version.
	it('confirmer envoie la version et la date choisie', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		await fireEvent.input(await r.findByTestId('close-date'), { target: { value: '2026-08-31' } });
		await fireEvent.click(r.getByTestId('close-confirm-button'));
		await waitFor(() => expect(closeBankAccountMock).toHaveBeenCalledWith(6, 3, '2026-08-31'));
	});

	it('Annuler ferme le dialogue sans rien envoyer', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		await fireEvent.click(await r.findByTestId('close-cancel-button'));
		expect(r.queryByTestId('close-confirm')).toBeNull();
		expect(closeBankAccountMock).not.toHaveBeenCalled();
	});

	// T-b8 — un refus du serveur s'affiche par son message, jamais « [object Object] ».
	it('un refus 412 s’affiche par le message du serveur', async () => {
		const message =
			"Ce compte porte des transactions importées qui ne sont pas rapprochées. Rapprochez-les, ou supprimez l'import s'il ne contient aucune transaction rapprochée, avant de clore le compte.";
		closeBankAccountMock.mockRejectedValue({
			code: 'BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS',
			message,
			status: 412,
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		await fireEvent.click(await r.findByTestId('close-confirm-button'));
		const erreur = await r.findByTestId('close-error');
		expect(erreur.textContent).toBe(message);
		expect(erreur.textContent).not.toContain('[object Object]');
		// Le dialogue reste ouvert : l'utilisateur lit le refus là où il a agi.
		expect(r.getByTestId('close-confirm')).toBeTruthy();
	});

	it('un rejet hors ApiError s’affiche par le générique', async () => {
		closeBankAccountMock.mockRejectedValue(new Error('boom'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		await fireEvent.click(await r.findByTestId('close-confirm-button'));
		expect((await r.findByTestId('close-error')).textContent).toBe('Erreur inattendue.');
	});

	// T-b9 — la bascule des clos : masqués par défaut, demandés au clic.
	it('la bascule Afficher les comptes clos recharge la liste avec includeClosed', async () => {
		const r = render(Page);
		const bascule = await r.findByTestId('toggle-closed');
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenCalledWith(false, false));
		expect(bascule.textContent?.trim()).toBe('Afficher les comptes clos');
		await fireEvent.click(bascule);
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenLastCalledWith(false, true));
		expect((await r.findByTestId('toggle-closed')).textContent?.trim()).toBe('Masquer les comptes clos');
	});

	// Toast de succès : il dit où retrouver le compte quand les clos sont masqués.
	it('après clôture, clos masqués : le toast dit comment retrouver le compte', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		await fireEvent.click(await r.findByTestId('close-confirm-button'));
		await waitFor(() =>
			expect(toast.success).toHaveBeenCalledWith(
				'Compte clos. Affichez les comptes clos pour le retrouver.',
			),
		);
	});

	it('après clôture, clos affichés : le toast simple', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('toggle-closed'));
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenLastCalledWith(false, true));
		await fireEvent.click(await r.findByTestId('close-6'));
		await fireEvent.click(await r.findByTestId('close-confirm-button'));
		await waitFor(() => expect(toast.success).toHaveBeenCalledWith('Compte bancaire clos.'));
	});

	// Réouverture : une phrase, Annuler / Rouvrir ; le geste part avec la version.
	it('rouvrir un compte clos envoie sa version', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('reopen-7'));
		expect((await r.findByTestId('reopen-confirm')).textContent).toContain('Rouvrir ce compte bancaire');
		await fireEvent.click(r.getByTestId('reopen-confirm-button'));
		await waitFor(() => expect(reopenBankAccountMock).toHaveBeenCalledWith(7, 4));
		await waitFor(() => expect(toast.success).toHaveBeenCalledWith('Compte bancaire rouvert.'));
	});

	it('un refus de la réouverture s’affiche par le message du serveur', async () => {
		reopenBankAccountMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'Le compte a été modifié entre-temps.',
			status: 409,
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('reopen-7'));
		await fireEvent.click(await r.findByTestId('reopen-confirm-button'));
		expect((await r.findByTestId('reopen-error')).textContent).toBe(
			'Le compte a été modifié entre-temps.',
		);
	});

	// ── Revue de code P1 (2026-10-13) ────────────────────────────────────────────

	// B-1 = E-4 — une bascule referme le dialogue ouvert : sans quoi, la ligne masquée,
	// le mode restait ouvert sur un compte absent et les actions de TOUTES les lignes
	// disparaissaient (garde `mode.kind === 'none'`).
	it.each([['toggle-closed'], ['toggle-archived']])(
		'%s referme le dialogue ouvert et rend les actions des lignes',
		async (bascule) => {
			const r = render(Page);
			await fireEvent.click(await r.findByTestId('reopen-7'));
			await r.findByTestId('reopen-confirm');
			await fireEvent.click(r.getByTestId(bascule));
			await waitFor(() => expect(r.queryByTestId('reopen-confirm')).toBeNull());
			expect(await r.findByTestId('close-6')).toBeTruthy();
		},
	);

	// B-2 — un conflit de version recharge la liste : le second essai part avec la
	// version relue, non avec la périmée.
	it('un conflit de version recharge la liste, et le second essai envoie la version relue', async () => {
		closeBankAccountMock.mockRejectedValueOnce({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'Conflit de version — la ressource a été modifiée',
			status: 409,
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		listBankAccountsMock.mockResolvedValue([PRINCIPAL, { ...OUVERT, version: 9 }, CLOS]);
		await fireEvent.click(await r.findByTestId('close-confirm-button'));
		expect((await r.findByTestId('close-error')).textContent).toContain('Conflit de version');
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenCalledTimes(2));
		await fireEvent.click(r.getByTestId('close-confirm-button'));
		await waitFor(() => expect(closeBankAccountMock).toHaveBeenLastCalledWith(6, 9, expect.any(String)));
	});

	// E-1 — la date préremplie est la date LOCALE : à 00:30 à Zurich, l'UTC dit encore la
	// veille. Horloge figée (seul `Date` est simulé, les attentes de testing-library
	// gardent leurs minuteries) et fuseau épinglé par `process.env.TZ`, que Node relit
	// à chaque calcul de date.
	it('la date préremplie est la date locale, non la date UTC (00:30 à Zurich)', async () => {
		const tz = process.env.TZ;
		process.env.TZ = 'Europe/Zurich';
		vi.useFakeTimers({ toFake: ['Date'] });
		vi.setSystemTime(new Date('2026-10-13T22:30:00Z'));
		try {
			// Le montage est-il celui que l'on croit ? 22:30 UTC = 00:30 le 14 à Zurich.
			expect(new Date().getDate()).toBe(14);
			expect(new Date().toISOString().slice(0, 10)).toBe('2026-10-13');
			const r = render(Page);
			await fireEvent.click(await r.findByTestId('close-6'));
			expect(((await r.findByTestId('close-date')) as HTMLInputElement).value).toBe('2026-10-14');
		} finally {
			vi.useRealTimers();
			if (tz === undefined) delete process.env.TZ;
			else process.env.TZ = tz;
		}
	});

	// E-2 — une date vidée désactive la confirmation.
	it('une date de clôture vidée désactive Clore le compte', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('close-6'));
		const bouton = (await r.findByTestId('close-confirm-button')) as HTMLButtonElement;
		expect(bouton.disabled).toBe(false);
		await fireEvent.input(r.getByTestId('close-date'), { target: { value: '' } });
		await waitFor(() => expect(bouton.disabled).toBe(true));
	});

	// E-3 — les deux bascules se composent.
	it('les deux bascules se composent : archivés puis clos → (true, true)', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('toggle-archived'));
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenLastCalledWith(true, false));
		await fireEvent.click(await r.findByTestId('toggle-closed'));
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenLastCalledWith(true, true));
	});

	// E-3 — un compte archivé ET clos (base restaurée) : deux badges, aucune action.
	it('un compte archivé et clos porte les deux badges et aucune action', async () => {
		listBankAccountsMock.mockResolvedValue([PRINCIPAL, { ...CLOS, archived: true }]);
		const r = render(Page);
		expect(await r.findByTestId('archived-badge-7')).toBeTruthy();
		expect(r.getByTestId('closed-badge-7')).toBeTruthy();
		for (const absent of ['reopen-7', 'close-7', 'archive-button-7', 'edit-button-7']) {
			expect(r.queryByTestId(absent)).toBeNull();
		}
	});

	// A-L6 — liste vide, comptes clos ou archivés masqués : le message dit où les trouver.
	it('liste vide avec les clos masqués : le message renvoie aux bascules', async () => {
		listBankAccountsMock.mockResolvedValue([]);
		const r = render(Page);
		expect((await r.findByTestId('bank-accounts-empty')).textContent).toContain(
			'Les comptes clos ou archivés éventuels sont masqués',
		);
	});

	it('liste vide, clos et archivés affichés : aucun compte configuré', async () => {
		listBankAccountsMock.mockResolvedValue([]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('toggle-closed'));
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenLastCalledWith(false, true));
		await fireEvent.click(await r.findByTestId('toggle-archived'));
		await waitFor(() => expect(listBankAccountsMock).toHaveBeenLastCalledWith(true, true));
		expect((await r.findByTestId('bank-accounts-empty')).textContent?.trim()).toBe(
			'Aucun compte bancaire configuré.',
		);
	});
});
