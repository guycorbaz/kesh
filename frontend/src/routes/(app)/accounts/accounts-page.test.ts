// Story 25-2-a (#382, #274) — tests Vitest pour l'avertissement bloquant au
// retypage d'un compte mouvementé.
//
// ⚠️ Cette page n'avait AUCUN test de page avant cette story, contrairement à
// `contacts/`, `users/` ou `settings/fiscal-years/`. Le harnais est calqué sur
// celui des exercices comptables : mocks hoistés AVANT l'import du composant,
// `authState` piloté via `login`.
//
// ⛔ `isApiError` n'est PAS mocké — c'est la vraie fonction, et elle exige
// `code` (string) ET `status` (number). Une erreur simulée à laquelle il
// manquerait `status` serait silencieusement rejetée par le garde de type, et
// l'écran retomberait sur « Erreur inattendue » : le test passerait à côté de la
// branche qu'il prétend couvrir, sans rien signaler.
//
// ⚠️ `i18nMsg` rend le repli SANS interpoler les arguments. Les assertions
// portent donc sur le COMPORTEMENT — présence du bloc des exercices clos, appel
// de `updateAccount` avec le drapeau — et jamais sur le texte interpolé, qui est
// affaire de traduction et non de code.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { authState } from '$lib/app/stores/auth.svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
const updateAccountMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
	updateAccount: (id: number, req: unknown) => updateAccountMock(id, req),
	createAccount: vi.fn(),
	archiveAccount: vi.fn(),
	reactivateAccount: vi.fn(),
}));

import Page from './+page.svelte';

function account(overrides: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id: 42,
		companyId: 1,
		number: '6500',
		name: 'Frais de bureau',
		accountType: 'Expense',
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

/** Le 409 que rend l'API quand le compte porte des écritures. */
function accountHasEntries(details: Record<string, unknown>) {
	return {
		code: 'ACCOUNT_HAS_ENTRIES',
		message: 'Ce compte porte des écritures.',
		status: 409,
		details,
	};
}

const IMPACT = {
	entryCount: 3,
	closedFiscalYears: ['Exercice 2025'],
	fromType: 'Expense',
	toType: 'Asset',
};

/** Ouvre la boîte de modification du compte 6500 et soumet. */
async function openEditAndSubmit() {
	const edit = await screen.findByTestId('account-row-6500-edit-button');
	await fireEvent.click(edit);
	const submit = await screen.findByTestId('account-edit-dialog-submit');
	await fireEvent.click(submit);
}

beforeEach(() => {
	fetchAccountsMock.mockReset();
	updateAccountMock.mockReset();
	fetchAccountsMock.mockResolvedValue([account()]);
	authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
});

afterEach(async () => {
	await authState.logout();
});

describe('retypage d’un compte mouvementé — avertissement bloquant', () => {
	it('un 409 ACCOUNT_HAS_ENTRIES ouvre l’avertissement et nomme les exercices clos', async () => {
		updateAccountMock.mockRejectedValue(accountHasEntries(IMPACT));

		render(Page);
		await openEditAndSubmit();

		// L'avertissement s'ouvre — et ce n'est pas un message d'erreur, c'est
		// une question posée à l'utilisateur.
		expect(await screen.findByTestId('account-retype-dialog-confirm')).toBeTruthy();
		// Et il NOMME les exercices clos : c'est tout l'écart entre un
		// avertissement bloquant et un refus sec.
		expect(await screen.findByTestId('account-retype-closed-years')).toBeTruthy();
	});

	it('sans exercice clos, l’avertissement s’ouvre mais ne prétend pas en nommer', async () => {
		updateAccountMock.mockRejectedValue(
			accountHasEntries({ ...IMPACT, closedFiscalYears: [] })
		);

		render(Page);
		await openEditAndSubmit();

		expect(await screen.findByTestId('account-retype-dialog-confirm')).toBeTruthy();
		expect(screen.queryByTestId('account-retype-closed-years')).toBeNull();
	});

	it('confirmer rejoue la requête AVEC confirmAccountRetype', async () => {
		updateAccountMock
			.mockRejectedValueOnce(accountHasEntries(IMPACT))
			.mockResolvedValueOnce(account({ accountType: 'Asset', version: 2 }));

		render(Page);
		await openEditAndSubmit();

		const confirm = await screen.findByTestId('account-retype-dialog-confirm');
		await fireEvent.click(confirm);

		await waitFor(() => expect(updateAccountMock).toHaveBeenCalledTimes(2));

		// Le premier appel ne porte PAS le drapeau, le second si : c'est
		// exactement le contrat de l'avertissement bloquant.
		const [, premier] = updateAccountMock.mock.calls[0];
		const [, second] = updateAccountMock.mock.calls[1];
		expect((premier as Record<string, unknown>).confirmAccountRetype).toBeUndefined();
		expect((second as Record<string, unknown>).confirmAccountRetype).toBe(true);
	});

	it('renoncer ne rejoue RIEN', async () => {
		updateAccountMock.mockRejectedValue(accountHasEntries(IMPACT));

		render(Page);
		await openEditAndSubmit();

		const cancel = await screen.findByTestId('account-retype-dialog-cancel');
		await fireEvent.click(cancel);

		// Un seul appel : celui qui a été refusé. Le compte n'est pas retypé.
		await waitFor(() => expect(updateAccountMock).toHaveBeenCalledTimes(1));
	});

	it('un details illisible retombe sur le message générique, sans avertissement à trous', async () => {
		// `entryCount` n'est pas un nombre : `readRetypeImpact` rend `null`.
		updateAccountMock.mockRejectedValue(accountHasEntries({ entryCount: 'beaucoup' }));

		render(Page);
		await openEditAndSubmit();

		await waitFor(() =>
			expect(screen.queryByTestId('account-retype-dialog-confirm')).toBeNull()
		);
		// Et le message du serveur est affiché tel quel plutôt qu'un
		// « undefined écriture(s) ».
		expect(await screen.findByText('Ce compte porte des écritures.')).toBeTruthy();
	});
});
