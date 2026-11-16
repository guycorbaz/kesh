// Story 37-1 (AC-7) — la devise des comptes au *Plan comptable*.
//
// En multi-devises : champ « Devise » à la création et à la modification
// (désactivé si verrouillé), colonne « Devise », et, pour un compte en devise
// seulement, le choix « par défaut / propres » des comptes de différence de
// change — envoyés seulement s'ils changent. En mono-devise : aucun changement
// visible, aucun champ envoyé.
//
// ⛔ `isApiError` n'est pas mocké : une erreur simulée porte `code` ET `status`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { CompanyCurrentResponse, CompanyJson } from '$lib/features/settings/settings.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
const createAccountMock = vi.fn();
const updateAccountMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
	createAccount: (req: unknown) => createAccountMock(req),
	updateAccount: (id: number, req: unknown) => updateAccountMock(id, req),
	archiveAccount: vi.fn(),
	reactivateAccount: vi.fn(),
}));
const fetchCompanyCurrentMock = vi.fn<() => Promise<CompanyCurrentResponse>>();
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => fetchCompanyCurrentMock(),
}));

import Page from './+page.svelte';

function account(overrides: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id: 42,
		companyId: 1,
		number: '1021',
		name: 'Banque USD',
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

const GAIN = account({ id: 68, number: '6840', name: 'Différences de change', accountType: 'Expense' });
const OTHER = account({ id: 69, number: '6950', name: 'Gains de change', accountType: 'Revenue' });
const CHF = account({ id: 50, number: '6500', name: 'Frais de bureau', accountType: 'Expense' });

function company(multi: boolean): CompanyCurrentResponse {
	const c: CompanyJson = {
		id: 1,
		name: 'Test SA',
		address: '',
		addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
		addressCountryKnown: true,
		ideNumber: null,
		orgType: 'Pme',
		accountingLanguage: 'FR',
		instanceLanguage: 'FR',
		email: null,
		phone: null,
		website: null,
		booksLockedThrough: null,
		multiCurrencyEnabled: multi,
		fxGainAccountId: multi ? 68 : null,
		fxLossAccountId: multi ? 68 : null,
		fxTransitAccountId: multi ? 10 : null,
		baseCurrency: 'CHF',
		version: 1,
	};
	return { company: c, bankAccounts: [] };
}

async function openEdit(number: string) {
	await fireEvent.click(await screen.findByTestId(`account-row-${number}-edit-button`));
	await screen.findByTestId('account-edit-dialog-submit');
}

beforeEach(() => {
	fetchAccountsMock.mockReset();
	createAccountMock.mockReset();
	updateAccountMock.mockReset();
	fetchCompanyCurrentMock.mockReset();
	updateAccountMock.mockResolvedValue({});
	createAccountMock.mockResolvedValue({});
	authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
});

afterEach(async () => {
	await authState.logout();
});

describe('mono-devise : aucun changement visible', () => {
	beforeEach(() => {
		fetchCompanyCurrentMock.mockResolvedValue(company(false));
		// Même un compte qui porterait une devise (état impossible) n'en montre aucune.
		fetchAccountsMock.mockResolvedValue([CHF, account({ currency: 'USD' })]);
	});

	it('ni colonne, ni champ « Devise » à la création ou à la modification, rien envoyé', async () => {
		render(Page);
		await screen.findByTestId('account-row-6500-edit-button');
		await waitFor(() => expect(fetchCompanyCurrentMock).toHaveBeenCalled());
		expect(screen.queryByTestId('account-row-1021-currency')).toBeNull();

		await fireEvent.click(screen.getByTestId('account-create-button'));
		await screen.findByTestId('account-create-dialog-submit');
		expect(screen.queryByTestId('account-create-dialog-currency')).toBeNull();
		await fireEvent.click(screen.getByTestId('account-create-dialog-cancel'));

		await openEdit('6500');
		expect(screen.queryByTestId('account-edit-dialog-currency')).toBeNull();
		await fireEvent.click(screen.getByTestId('account-edit-dialog-submit'));
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalledTimes(1));
		const body = updateAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect(Object.keys(body).sort()).toEqual(['accountType', 'name', 'postable', 'role', 'version']);
	});

	it('société illisible : l’écran reste celui du mono-devise', async () => {
		fetchCompanyCurrentMock.mockRejectedValue(new Error('down'));
		render(Page);
		await openEdit('6500');
		expect(screen.queryByTestId('account-edit-dialog-currency')).toBeNull();
	});
});

describe('multi-devises', () => {
	beforeEach(() => {
		fetchCompanyCurrentMock.mockResolvedValue(company(true));
	});

	it('colonne « Devise » : le code pour un compte en devise, rien pour le CHF', async () => {
		fetchAccountsMock.mockResolvedValue([CHF, account({ currency: 'USD' })]);
		render(Page);
		expect((await screen.findByTestId('account-row-1021-currency')).textContent?.trim()).toBe('USD');
		expect(screen.queryByTestId('account-row-6500-currency')).toBeNull();
	});

	it('le choix des comptes de différence n’apparaît que pour un compte en devise', async () => {
		fetchAccountsMock.mockResolvedValue([CHF, GAIN, OTHER, account({ currency: 'USD' })]);
		render(Page);
		await openEdit('6500');
		expect(screen.getByTestId('account-edit-dialog-currency')).toBeTruthy();
		expect(screen.queryByTestId('account-edit-dialog-fx')).toBeNull();
		await fireEvent.click(screen.getByTestId('account-edit-dialog-cancel'));

		await openEdit('1021');
		expect(await screen.findByTestId('account-edit-dialog-fx')).toBeTruthy();
		// « Par défaut » rappelle les comptes de la société.
		expect(screen.getByTestId('account-edit-dialog-fx-company').textContent).toContain(
			'6840 Différences de change',
		);
	});

	it('« par défaut » sans changement : ni devise ni comptes envoyés', async () => {
		fetchAccountsMock.mockResolvedValue([GAIN, OTHER, account({ currency: 'USD' })]);
		render(Page);
		await openEdit('1021');
		await fireEvent.click(screen.getByTestId('account-edit-dialog-submit'));
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalledTimes(1));
		const body = updateAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect(body).not.toHaveProperty('currency');
		expect(body).not.toHaveProperty('fxGainAccountId');
		expect(body).not.toHaveProperty('fxLossAccountId');
	});

	it('« propres » envoie les deux comptes', async () => {
		fetchAccountsMock.mockResolvedValue([GAIN, OTHER, account({ currency: 'USD' })]);
		render(Page);
		await openEdit('1021');
		await fireEvent.click(screen.getByTestId('account-edit-dialog-fx-own'));
		const gain = (await screen.findByTestId('account-edit-dialog-fx-gain')) as HTMLSelectElement;
		await fireEvent.change(gain, { target: { value: '68' } });
		const loss = screen.getByTestId('account-edit-dialog-fx-loss') as HTMLSelectElement;
		await fireEvent.change(loss, { target: { value: '69' } });
		await fireEvent.click(screen.getByTestId('account-edit-dialog-submit'));
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalledTimes(1));
		const body = updateAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect(body.fxGainAccountId).toBe(68);
		expect(body.fxLossAccountId).toBe(69);
		expect(body).not.toHaveProperty('currency');
	});

	it('« propres » sans les deux comptes : message, aucun envoi', async () => {
		fetchAccountsMock.mockResolvedValue([GAIN, OTHER, account({ currency: 'USD' })]);
		render(Page);
		await openEdit('1021');
		await fireEvent.click(screen.getByTestId('account-edit-dialog-fx-own'));
		await fireEvent.click(screen.getByTestId('account-edit-dialog-submit'));
		expect(await screen.findByText(/Choisissez les deux comptes/)).toBeTruthy();
		expect(updateAccountMock).not.toHaveBeenCalled();
	});

	it('devise remise à vide : choix repassé à « par défaut », currency et comptes envoyés à null', async () => {
		fetchAccountsMock.mockResolvedValue([
			GAIN,
			OTHER,
			account({ currency: 'USD', fxGainAccountId: 68, fxLossAccountId: 69 }),
		]);
		render(Page);
		await openEdit('1021');
		expect((screen.getByTestId('account-edit-dialog-fx-own') as HTMLInputElement).checked).toBe(true);
		await fireEvent.change(screen.getByTestId('account-edit-dialog-currency'), { target: { value: '' } });
		expect(screen.queryByTestId('account-edit-dialog-fx')).toBeNull();
		await fireEvent.click(screen.getByTestId('account-edit-dialog-submit'));
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalledTimes(1));
		const body = updateAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect(body.currency).toBeNull();
		expect(body.fxGainAccountId).toBeNull();
		expect(body.fxLossAccountId).toBeNull();
	});

	it('devise verrouillée : champ désactivé avec l’explication', async () => {
		fetchAccountsMock.mockResolvedValue([account({ currency: 'USD', currencyLocked: true })]);
		render(Page);
		await openEdit('1021');
		expect((screen.getByTestId('account-edit-dialog-currency') as HTMLSelectElement).disabled).toBe(true);
		expect(screen.getByTestId('account-edit-dialog-currency-locked')).toBeTruthy();
	});

	it('devise non verrouillée : champ actif, sans explication', async () => {
		fetchAccountsMock.mockResolvedValue([account({ currency: 'USD' })]);
		render(Page);
		await openEdit('1021');
		expect((screen.getByTestId('account-edit-dialog-currency') as HTMLSelectElement).disabled).toBe(false);
		expect(screen.queryByTestId('account-edit-dialog-currency-locked')).toBeNull();
	});

	it('création d’un compte en devise « par défaut » : la devise seule', async () => {
		fetchAccountsMock.mockResolvedValue([GAIN]);
		render(Page);
		await fireEvent.click(await screen.findByTestId('account-create-button'));
		await fireEvent.input(await screen.findByLabelText('Numéro'), { target: { value: '1022' } });
		await fireEvent.input(screen.getByLabelText('Nom'), { target: { value: 'Banque EUR' } });
		await fireEvent.change(screen.getByTestId('account-create-dialog-currency'), {
			target: { value: 'EUR' },
		});
		expect(await screen.findByTestId('account-create-dialog-fx')).toBeTruthy();
		await fireEvent.click(screen.getByTestId('account-create-dialog-submit'));
		await waitFor(() => expect(createAccountMock).toHaveBeenCalledTimes(1));
		const body = createAccountMock.mock.calls[0][0] as Record<string, unknown>;
		expect(body.currency).toBe('EUR');
		expect(body).not.toHaveProperty('fxGainAccountId');
	});

	it('FX_ACCOUNT_INVALID : le message s’affiche sous le sélecteur désigné', async () => {
		fetchAccountsMock.mockResolvedValue([GAIN, OTHER, account({ currency: 'USD' })]);
		updateAccountMock.mockRejectedValue({
			code: 'FX_ACCOUNT_INVALID',
			status: 400,
			message: 'Compte de pertes archivé.',
			details: { field: 'fxLossAccountId', reason: 'archived' },
		});
		render(Page);
		await openEdit('1021');
		await fireEvent.click(screen.getByTestId('account-edit-dialog-fx-own'));
		await fireEvent.change(await screen.findByTestId('account-edit-dialog-fx-gain'), {
			target: { value: '68' },
		});
		await fireEvent.change(screen.getByTestId('account-edit-dialog-fx-loss'), { target: { value: '69' } });
		await fireEvent.click(screen.getByTestId('account-edit-dialog-submit'));
		expect((await screen.findByTestId('account-edit-dialog-fx-loss-error')).textContent).toContain(
			'Compte de pertes archivé.',
		);
	});
});
