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
import { toast } from 'svelte-sonner';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
const updateAccountMock = vi.fn();
const archiveAccountMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
	updateAccount: (id: number, req: unknown) => updateAccountMock(id, req),
	createAccount: vi.fn(),
	archiveAccount: (id: number, req: unknown) => archiveAccountMock(id, req),
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
		letterable: false,
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

// Story 33-8 (#486, AC-8) — archiver un compte désigné dans les paramètres.
describe('archivage d’un compte désigné — avertissement, puis confirmation', () => {
	function designated(settings: unknown) {
		return {
			code: 'ACCOUNT_DESIGNATED_IN_SETTINGS',
			message: 'Ce compte est désigné.',
			status: 409,
			details: { settings },
		};
	}

	async function openAndConfirm() {
		await fireEvent.click(await screen.findByTestId('account-row-6500-archive-button'));
		await fireEvent.click(await screen.findByTestId('account-archive-dialog-confirm'));
	}

	beforeEach(() => {
		archiveAccountMock.mockReset();
		vi.mocked(toast.error).mockClear();
	});

	it('Administrateur : les réglages par leur libellé, le lien, et la confirmation archive', async () => {
		archiveAccountMock.mockRejectedValueOnce(designated(['discount', 'vatPayable']));
		archiveAccountMock.mockResolvedValueOnce(account({ active: false }));
		render(Page);
		await openAndConfirm();

		const items = await screen.findAllByTestId('account-archive-designated-setting');
		expect(items[0].textContent).toContain("Compte d'escompte accordé");
		expect(items[1].textContent).toContain('Compte TVA due (Passif)');
		expect(screen.getByTestId('account-archive-designated-link').getAttribute('href')).toBe(
			'/settings/invoicing',
		);
		expect(screen.queryByTestId('account-archive-designated-admin-only')).toBeNull();
		expect(toast.error).not.toHaveBeenCalled();
		expect(archiveAccountMock).toHaveBeenCalledWith(42, { version: 1 });

		await fireEvent.click(screen.getByTestId('account-archive-dialog-anyway'));
		await waitFor(() => expect(archiveAccountMock).toHaveBeenCalledTimes(2));
		expect(archiveAccountMock).toHaveBeenLastCalledWith(42, {
			version: 1,
			confirmDesignated: true,
		});
		await waitFor(() =>
			expect(screen.queryByTestId('account-archive-designated')).toBeNull(),
		);
	});

	it('Comptable : aucun lien, le texte « Un administrateur… », et sa confirmation archive', async () => {
		await authState.logout();
		authState.login({ userId: '2', username: 'compta', role: 'Comptable', expiresIn: 3600 });
		archiveAccountMock.mockRejectedValueOnce(designated(['rounding']));
		archiveAccountMock.mockResolvedValueOnce(account({ active: false }));
		render(Page);
		await openAndConfirm();

		await screen.findByTestId('account-archive-designated');
		expect(screen.queryByTestId('account-archive-designated-link')).toBeNull();
		expect(screen.getByTestId('account-archive-designated-admin-only')).toBeTruthy();

		await fireEvent.click(screen.getByTestId('account-archive-dialog-anyway'));
		await waitFor(() =>
			expect(archiveAccountMock).toHaveBeenLastCalledWith(42, {
				version: 1,
				confirmDesignated: true,
			}),
		);
	});

	it('un 409 de version garde son message, sans liste ni confirmation', async () => {
		archiveAccountMock.mockRejectedValueOnce({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			message: 'conflit',
			status: 409,
		});
		render(Page);
		await openAndConfirm();
		await waitFor(() =>
			expect(toast.error).toHaveBeenCalledWith('Les données ont été modifiées. Rechargez la page.'),
		);
		expect(screen.queryByTestId('account-archive-designated')).toBeNull();
		expect(screen.queryByTestId('account-archive-dialog-anyway')).toBeNull();
	});

	// Revue P1 (A L-2) : le Comptable n'est jamais enfermé, même si `details` est illisible.
	it('details illisible : le message du serveur, sans liste, et la confirmation reste offerte', async () => {
		archiveAccountMock.mockRejectedValueOnce(designated('receivable'));
		archiveAccountMock.mockResolvedValueOnce(account({ active: false }));
		render(Page);
		await openAndConfirm();
		const avis = await screen.findByTestId('account-archive-designated');
		expect(avis.textContent).toContain('Ce compte est désigné.');
		expect(screen.queryByTestId('account-archive-designated-setting')).toBeNull();
		await fireEvent.click(screen.getByTestId('account-archive-dialog-anyway'));
		await waitFor(() =>
			expect(archiveAccountMock).toHaveBeenLastCalledWith(42, {
				version: 1,
				confirmDesignated: true,
			}),
		);
	});

	// Revue P1 (A M-2, E-2, E-3) : l'effet se dit réglage par réglage, et n'en promet aucun
	// pour le décompte TVA, qu'aucune écriture ne lit.
	it('chaque réglage dit son propre effet', async () => {
		archiveAccountMock.mockRejectedValueOnce(
			designated([
				'receivable',
				'revenue',
				'vatPayable',
				'vatRecoverable',
				'vatDecompte',
				'payable',
				'rounding',
				'discount',
				'bankFees',
				'badDebt',
			]),
		);
		render(Page);
		await openAndConfirm();
		const items = await screen.findAllByTestId('account-archive-designated-setting');
		const textes = items.map((i) => i.textContent ?? '');
		expect(textes).toHaveLength(10);
		for (const texte of textes) expect(texte).toContain(' : ');
		expect(textes[0]).toContain('validation d’une facture');
		expect(textes[1]).toContain('sans compte de produit');
		expect(textes[2]).toContain('portant de la TVA');
		expect(textes[3]).toContain('fournisseur portant de la TVA');
		expect(textes[4]).toContain('Aucune écriture');
		expect(textes[5]).toContain('facture fournisseur');
		expect(textes[6]).toContain('écart d’arrondi');
		for (const i of [7, 8, 9]) expect(textes[i]).toContain('solde');
		const avis = screen.getByTestId('account-archive-designated');
		expect(avis.getAttribute('role')).toBe('alert');
	});

	it('rouvrir le dialogue repart de la première question', async () => {
		archiveAccountMock.mockRejectedValueOnce(designated(['discount']));
		render(Page);
		await openAndConfirm();
		await screen.findByTestId('account-archive-designated');
		await fireEvent.click(screen.getByTestId('account-archive-dialog-cancel'));
		await fireEvent.click(await screen.findByTestId('account-row-6500-archive-button'));
		await screen.findByTestId('account-archive-dialog-confirm');
		expect(screen.queryByTestId('account-archive-designated')).toBeNull();
	});
});
