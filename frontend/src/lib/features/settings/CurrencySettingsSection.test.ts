// Story 37-1 (AC-7) — bloc « Comptabilité multi-devises » de Paramètres →
// Comptabilité : confirmation de l'activation (irréversible), *Annuler*
// n'envoie rien, case désactivée une fois actif, sélecteurs toujours
// modifiables, erreurs du serveur, lecture seule hors Administrateur.
//
// ⛔ `isApiError` n'est pas mocké : une erreur simulée porte `code` ET `status`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { CompanyJson } from './settings.types';

const updateMock = vi.fn();
const fetchCompanyMock = vi.fn();
vi.mock('./settings.api', () => ({
	updateCurrencySettings: (...a: unknown[]) => updateMock(...a),
	fetchCompanyCurrent: (...a: unknown[]) => fetchCompanyMock(...a),
}));
const fetchAccountsMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: (...a: unknown[]) => fetchAccountsMock(...a),
}));
const notifySuccessMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: (...a: unknown[]) => notifySuccessMock(...a),
}));

import CurrencySettingsSection from './CurrencySettingsSection.svelte';

function acc(id: number, number: string, accountType: AccountResponse['accountType']): AccountResponse {
	return {
		id,
		companyId: 1,
		number,
		name: `Compte ${number}`,
		accountType,
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	};
}

const ACCOUNTS = [acc(68, '6840', 'Expense'), acc(69, '6950', 'Revenue'), acc(10, '1090', 'Asset')];

function company(overrides: Partial<CompanyJson> = {}): CompanyJson {
	return {
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
		multiCurrencyEnabled: false,
		fxGainAccountId: null,
		fxLossAccountId: null,
		fxTransitAccountId: null,
		baseCurrency: 'CHF',
		version: 3,
		...overrides,
	};
}

const ACTIVE = { multiCurrencyEnabled: true, fxGainAccountId: 68, fxLossAccountId: 69, fxTransitAccountId: 10 };

async function monter(c: CompanyJson, isAdmin = true) {
	const onSaved = vi.fn();
	const r = render(CurrencySettingsSection, { company: c, isAdmin, onSaved });
	await waitFor(() => expect(fetchAccountsMock).toHaveBeenCalled());
	return { ...r, onSaved };
}

/** Coche la case et choisit les trois comptes. */
async function remplir(r: Awaited<ReturnType<typeof monter>>) {
	await fireEvent.click(r.getByTestId('settings-currency-enabled'));
	const gain = (await r.findByTestId('settings-currency-gain')) as HTMLSelectElement;
	await waitFor(() => expect(gain.options.length).toBeGreaterThan(1));
	await fireEvent.change(gain, { target: { value: gain.options[1].value } });
	const loss = r.getByTestId('settings-currency-loss') as HTMLSelectElement;
	await fireEvent.change(loss, { target: { value: loss.options[2].value } });
	const transit = r.getByTestId('settings-currency-transit') as HTMLSelectElement;
	await fireEvent.change(transit, { target: { value: transit.options[1].value } });
}

describe('CurrencySettingsSection (Story 37-1, AC-7)', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		fetchAccountsMock.mockResolvedValue(ACCOUNTS);
	});

	it('mono-devise : case décochée, aucun sélecteur, textes de l’AC-7', async () => {
		const r = await monter(company());
		const box = r.getByTestId('settings-currency-enabled') as HTMLInputElement;
		expect(box.checked).toBe(false);
		expect(box.disabled).toBe(false);
		expect(r.queryByTestId('settings-currency-gain')).toBeNull();
		const help = r.getByTestId('settings-currency-help').textContent ?? '';
		expect(help).toContain('CHF');
		expect(help).toContain('exercices ouverts');
		expect(help).toContain('compte de conversion');
		expect(r.getByTestId('settings-currency-help-definitive').textContent).toContain('définitive');
	});

	it('cochée : les sélecteurs filtrent les comptes (gains/pertes : charges et produits ; conversion : bilan)', async () => {
		const r = await monter(company());
		await fireEvent.click(r.getByTestId('settings-currency-enabled'));
		const gain = (await r.findByTestId('settings-currency-gain')) as HTMLSelectElement;
		await waitFor(() => expect(gain.options.length).toBe(3));
		const transit = r.getByTestId('settings-currency-transit') as HTMLSelectElement;
		expect([...transit.options].map((o) => o.textContent?.trim())).toEqual([
			'— Sélectionner —',
			'1090 — Compte 1090',
		]);
	});

	it('enregistrer l’activation ouvre la confirmation, sans rien envoyer', async () => {
		const r = await monter(company());
		await remplir(r);
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		expect(await r.findByTestId('settings-currency-confirm-activate')).toBeTruthy();
		expect(updateMock).not.toHaveBeenCalled();
	});

	it('*Annuler* ferme la confirmation et n’envoie rien', async () => {
		const r = await monter(company());
		await remplir(r);
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		await fireEvent.click(await r.findByTestId('settings-currency-confirm-cancel'));
		await waitFor(() => expect(r.queryByTestId('settings-currency-confirm-activate')).toBeNull());
		expect(updateMock).not.toHaveBeenCalled();
	});

	it('*Activer* envoie le PUT ; la case reste cochée et désactivée, les sélecteurs modifiables', async () => {
		const updated = company({ ...ACTIVE, version: 4 });
		updateMock.mockResolvedValue(updated);
		const r = await monter(company());
		await remplir(r);
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		await fireEvent.click(await r.findByTestId('settings-currency-confirm-activate'));
		await waitFor(() => expect(updateMock).toHaveBeenCalledTimes(1));
		expect(updateMock).toHaveBeenCalledWith({
			multiCurrencyEnabled: true,
			fxGainAccountId: 68,
			fxLossAccountId: 69,
			fxTransitAccountId: 10,
			version: 3,
		});
		await waitFor(() => expect(r.onSaved).toHaveBeenCalledWith(updated));
		await r.rerender({ company: updated, isAdmin: true, onSaved: r.onSaved });
		const box = r.getByTestId('settings-currency-enabled') as HTMLInputElement;
		expect(box.checked).toBe(true);
		expect(box.disabled).toBe(true);
		expect((r.getByTestId('settings-currency-gain') as HTMLSelectElement).disabled).toBe(false);
	});

	it('actif : changer un compte envoie le PUT directement, sans confirmation', async () => {
		updateMock.mockResolvedValue(company({ ...ACTIVE, fxLossAccountId: 68, version: 6 }));
		const r = await monter(company({ ...ACTIVE, version: 5 }));
		const box = r.getByTestId('settings-currency-enabled') as HTMLInputElement;
		expect(box.checked).toBe(true);
		expect(box.disabled).toBe(true);
		const loss = (await r.findByTestId('settings-currency-loss')) as HTMLSelectElement;
		await waitFor(() => expect(loss.options.length).toBe(3));
		await fireEvent.change(loss, { target: { value: loss.options[1].value } });
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		await waitFor(() => expect(updateMock).toHaveBeenCalledTimes(1));
		expect(updateMock.mock.calls[0][0]).toEqual({
			multiCurrencyEnabled: true,
			fxGainAccountId: 68,
			fxLossAccountId: 68,
			fxTransitAccountId: 10,
			version: 5,
		});
		expect(r.queryByTestId('settings-currency-confirm-activate')).toBeNull();
	});

	it('activer sans les trois comptes : message, aucune confirmation ni PUT', async () => {
		const r = await monter(company());
		await fireEvent.click(r.getByTestId('settings-currency-enabled'));
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		expect((await r.findByTestId('settings-currency-error')).textContent).toContain('trois comptes');
		expect(r.queryByTestId('settings-currency-confirm-activate')).toBeNull();
		expect(updateMock).not.toHaveBeenCalled();
	});

	it('FX_ACCOUNT_INVALID : le message s’affiche sous le sélecteur désigné par details.field', async () => {
		updateMock.mockRejectedValue({
			code: 'FX_ACCOUNT_INVALID',
			status: 400,
			message: 'Compte de pertes archivé.',
			details: { field: 'fxLossAccountId', reason: 'archived' },
		});
		const r = await monter(company({ ...ACTIVE }));
		const gain = (await r.findByTestId('settings-currency-gain')) as HTMLSelectElement;
		await waitFor(() => expect(gain.options.length).toBe(3));
		await fireEvent.change(gain, { target: { value: gain.options[2].value } });
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		expect((await r.findByTestId('settings-currency-loss-error')).textContent).toContain(
			'Compte de pertes archivé.',
		);
		expect(r.queryByTestId('settings-currency-gain-error')).toBeNull();
	});

	it('409 de version : la société est relue et l’écran le dit', async () => {
		updateMock.mockRejectedValue({ code: 'OPTIMISTIC_LOCK_CONFLICT', status: 409, message: 'conflit' });
		const fresh = company({ ...ACTIVE, version: 9 });
		fetchCompanyMock.mockResolvedValue({ company: fresh, bankAccounts: [] });
		const r = await monter(company({ ...ACTIVE }));
		const gain = (await r.findByTestId('settings-currency-gain')) as HTMLSelectElement;
		await waitFor(() => expect(gain.options.length).toBe(3));
		await fireEvent.change(gain, { target: { value: gain.options[2].value } });
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		expect((await r.findByTestId('settings-currency-error')).textContent).toContain('Conflit de version');
		expect(r.onSaved).toHaveBeenCalledWith(fresh);
	});

	it('actif : vider un compte est refusé par l’écran, sans PUT', async () => {
		const r = await monter(company({ ...ACTIVE }));
		const transit = (await r.findByTestId('settings-currency-transit')) as HTMLSelectElement;
		await waitFor(() => expect(transit.options.length).toBe(2));
		await fireEvent.change(transit, { target: { value: transit.options[0].value } });
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		expect((await r.findByTestId('settings-currency-error')).textContent).toContain('trois comptes');
		expect(updateMock).not.toHaveBeenCalled();
	});

	it('un autre refus (MULTI_CURRENCY_IRREVERSIBLE…) : le message du serveur est affiché', async () => {
		updateMock.mockRejectedValue({
			code: 'MULTI_CURRENCY_IRREVERSIBLE',
			status: 409,
			message: 'Le multi-devises ne peut plus être désactivé.',
			details: {},
		});
		const r = await monter(company({ ...ACTIVE }));
		const gain = (await r.findByTestId('settings-currency-gain')) as HTMLSelectElement;
		await waitFor(() => expect(gain.options.length).toBe(3));
		await fireEvent.change(gain, { target: { value: gain.options[2].value } });
		await fireEvent.click(r.getByTestId('settings-currency-save'));
		expect((await r.findByTestId('settings-currency-error')).textContent).toContain(
			'ne peut plus être désactivé',
		);
	});

	it('hors Administrateur : lecture seule — case et sélecteurs désactivés, aucun bouton', async () => {
		const r = await monter(company({ ...ACTIVE }), false);
		expect((r.getByTestId('settings-currency-enabled') as HTMLInputElement).disabled).toBe(true);
		expect((r.getByTestId('settings-currency-gain') as HTMLSelectElement).disabled).toBe(true);
		expect((r.getByTestId('settings-currency-transit') as HTMLSelectElement).disabled).toBe(true);
		expect(r.queryByTestId('settings-currency-save')).toBeNull();
		expect(r.getByTestId('settings-currency-admin-only')).toBeTruthy();
	});

	it('hors Administrateur, mono-devise : la case ne se coche pas', async () => {
		const r = await monter(company(), false);
		expect((r.getByTestId('settings-currency-enabled') as HTMLInputElement).disabled).toBe(true);
	});
});
