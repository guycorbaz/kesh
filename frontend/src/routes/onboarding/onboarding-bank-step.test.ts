/**
 * Story 34-1 (#966) — l'étape bancaire de l'assistant pose la description et le
 * compte comptable lié du compte bancaire.
 *
 * - AC-6 : la liste des comptes liables (actifs, imputables, actif ou passif, sans
 *   les comptes de rôle `Receivable` et `Payable`) ; **présélection du compte
 *   numéroté `1020`** s'il y figure, « Non configuré » sinon ; liste en échec →
 *   « Non configuré », motif sous le champ, l'étape reste utilisable.
 *   (« Aucun » de la fiche = l'option « Non configuré » de la page *Comptes
 *   bancaires*, clé réutilisée, T5.)
 * - AC-1 : le corps envoyé porte toujours `description` et `journalAccountId`
 *   (`null` pour « Non configuré »).
 * - AC-7 : la phrase « modifiable plus tard ».
 *
 * ⛔ Le numéro `1020` n'est qu'une présélection d'écran (décision de Guy, Story
 * 14-3a) : le serveur n'en sait rien.
 *
 * Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

const { i18nMsgMock, toastError, fetchAccountsMock } = vi.hoisted(() => ({
	i18nMsgMock: vi.fn((_key: string, fallback: string) => fallback),
	toastError: vi.fn(),
	fetchAccountsMock: vi.fn<() => Promise<AccountResponse[]>>(),
}));

vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { currentUser: { role: 'Admin' }, isAuthenticated: true },
}));
vi.mock('$lib/features/onboarding/onboarding.api', () => ({
	fetchState: vi.fn(),
	setLanguage: vi.fn(),
	setMode: vi.fn(),
	seedDemo: vi.fn(),
	resetDemo: vi.fn(),
	startProduction: vi.fn(),
	setOrgType: vi.fn(),
	setAccountingLanguage: vi.fn(),
	setCoordinates: vi.fn(),
	setBankAccount: vi.fn(),
	skipBank: vi.fn(),
	finalize: vi.fn(),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: i18nMsgMock,
	loadI18nMessages: vi.fn(),
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: toastError } }));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import Page from './+page.svelte';

const etat = (stepCompleted: number) => ({
	stepCompleted,
	isDemo: false,
	uiMode: null,
	isStub: false,
});

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

/** Un plan PME réduit : de quoi éprouver chaque condition du filtre. */
const PLAN: AccountResponse[] = [
	account({ id: 10, number: '10', name: 'Actifs circulants', postable: false }),
	account({ id: 11, number: '1000', name: 'Caisse' }),
	account({ id: 12, number: '1020', name: 'Banque' }),
	account({ id: 13, number: '1100', name: 'Débiteurs', role: 'Receivable' }),
	account({ id: 14, number: '1150', name: 'Ancien compte', active: false }),
	account({ id: 20, number: '2000', name: 'Créanciers', accountType: 'Liability', role: 'Payable' }),
	account({ id: 21, number: '2100', name: 'Dettes bancaires', accountType: 'Liability' }),
	account({ id: 30, number: '3000', name: 'Ventes', accountType: 'Revenue' }),
];

async function atBankStep() {
	vi.mocked(api.fetchState).mockResolvedValue(etat(6));
	await onboardingState.fetchState();
}

function select(): HTMLSelectElement {
	return document.getElementById('bank-journal-account') as HTMLSelectElement;
}

function optionLabels(): string[] {
	return Array.from(select().options).map((o) => o.textContent?.trim() ?? '');
}

function remplir(id: string, valeur: string) {
	const el = document.getElementById(id) as HTMLInputElement;
	el.value = valeur;
	return fireEvent.input(el);
}

async function soumettre() {
	await remplir('bank-name', 'Banque Exemple');
	await remplir('bank-iban', 'CH93 0076 2011 6238 5295 7');
	const form = document.querySelector('form') as HTMLFormElement;
	return fireEvent.submit(form);
}

beforeEach(() => {
	vi.clearAllMocks();
	i18nMsgMock.mockImplementation((_key: string, fallback: string) => fallback);
	vi.mocked(api.setBankAccount).mockResolvedValue(etat(7));
	vi.mocked(api.finalize).mockResolvedValue(etat(8));
});

describe('étape bancaire de l’assistant — compte lié (Story 34-1, AC-6)', () => {
	it('ne propose que les comptes liables, et présélectionne le 1020', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN);
		await atBankStep();
		const { unmount } = render(Page);
		await waitFor(() => expect(select().selectedOptions[0]?.textContent).toContain('1020'));
		expect(optionLabels()).toEqual([
			'Non configuré',
			'1000 — Caisse',
			'1020 — Banque',
			'2100 — Dettes bancaires',
		]);
		unmount();
	});

	it('sans 1020, le champ vaut « Non configuré » (pas le premier compte, M4)', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN.filter((a) => a.number !== '1020'));
		await atBankStep();
		const { unmount } = render(Page);
		await waitFor(() => expect(optionLabels()).toContain('1000 — Caisse'));
		expect(select().selectedOptions[0]?.textContent?.trim()).toBe('Non configuré');
		unmount();
	});

	it('un 1020 non imputable n’est ni proposé ni présélectionné (M5)', async () => {
		fetchAccountsMock.mockResolvedValue(
			PLAN.map((a) => (a.number === '1020' ? { ...a, postable: false } : a))
		);
		await atBankStep();
		const { unmount } = render(Page);
		await waitFor(() => expect(optionLabels()).toContain('1000 — Caisse'));
		expect(optionLabels()).not.toContain('1020 — Banque');
		expect(select().selectedOptions[0]?.textContent?.trim()).toBe('Non configuré');
		unmount();
	});

	it('un 1020 de rôle Receivable n’est pas proposé : le rôle prime sur le numéro', async () => {
		fetchAccountsMock.mockResolvedValue(
			PLAN.map((a) => (a.number === '1020' ? { ...a, role: 'Receivable' as const } : a))
		);
		await atBankStep();
		const { unmount } = render(Page);
		await waitFor(() => expect(optionLabels()).toContain('1000 — Caisse'));
		expect(optionLabels()).not.toContain('1020 — Banque');
		expect(optionLabels()).not.toContain('1100 — Débiteurs');
		expect(select().selectedOptions[0]?.textContent?.trim()).toBe('Non configuré');
		unmount();
	});

	it('la liste en échec : « Non configuré », motif sous le champ, sans toast, et l’étape s’envoie', async () => {
		const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
		fetchAccountsMock.mockRejectedValue(new TypeError('Failed to fetch'));
		await atBankStep();
		const { unmount, findByTestId } = render(Page);
		const motif = await findByTestId('onboarding-bank-ledger-error');
		expect(motif.textContent).toContain('peut être lié plus tard');
		expect(i18nMsgMock).toHaveBeenCalledWith(
			'onboarding-bank-ledger-load-error',
			expect.stringContaining('pas pu être chargée')
		);
		expect(select().selectedOptions[0]?.textContent?.trim()).toBe('Non configuré');
		expect(select().options).toHaveLength(1);
		consoleError.mockRestore();
		await soumettre();
		await waitFor(() => expect(api.setBankAccount).toHaveBeenCalledTimes(1));
		expect(vi.mocked(api.setBankAccount).mock.calls[0][0]).toMatchObject({
			journalAccountId: null,
		});
		expect(toastError).not.toHaveBeenCalled();
		unmount();
	});
});

describe('étape bancaire de l’assistant — corps envoyé (Story 34-1, AC-1, M6)', () => {
	it('porte la description saisie et le compte présélectionné', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN);
		await atBankStep();
		const { unmount } = render(Page);
		await waitFor(() => expect(select().selectedOptions[0]?.textContent).toContain('1020'));
		await remplir('bank-description', '  Personnel  ');
		await soumettre();
		await waitFor(() => expect(api.setBankAccount).toHaveBeenCalledTimes(1));
		expect(vi.mocked(api.setBankAccount).mock.calls[0][0]).toEqual({
			bankName: 'Banque Exemple',
			iban: 'CH93 0076 2011 6238 5295 7',
			qrIban: null,
			description: 'Personnel',
			journalAccountId: 12,
		});
		unmount();
	});

	it('« Non configuré » choisi et description vide : deux null explicites', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN);
		await atBankStep();
		const { unmount } = render(Page);
		await waitFor(() => expect(select().selectedOptions[0]?.textContent).toContain('1020'));
		select().selectedIndex = 0;
		await fireEvent.change(select());
		await soumettre();
		await waitFor(() => expect(api.setBankAccount).toHaveBeenCalledTimes(1));
		const corps = vi.mocked(api.setBankAccount).mock.calls[0][0] as Record<string, unknown>;
		expect(corps).toHaveProperty('description', null);
		expect(corps).toHaveProperty('journalAccountId', null);
		unmount();
	});

	it('un refus du serveur s’affiche avec son motif (AC-8)', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN);
		await atBankStep();
		vi.mocked(api.setBankAccount).mockRejectedValueOnce({
			code: 'BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT',
			message: 'MOTIF-CREANCE',
			status: 400,
		});
		const { unmount } = render(Page);
		await waitFor(() => expect(select().selectedOptions[0]?.textContent).toContain('1020'));
		await soumettre();
		await waitFor(() => expect(toastError).toHaveBeenCalledWith('MOTIF-CREANCE'));
		expect(api.finalize).not.toHaveBeenCalled();
		unmount();
	});
});

describe('étape bancaire de l’assistant — champs et phrase (Story 34-1, AC-6, AC-7)', () => {
	it('la description a la limite et l’aide de la page Comptes bancaires', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN);
		await atBankStep();
		const { unmount } = render(Page);
		const champ = document.getElementById('bank-description') as HTMLInputElement;
		expect(champ.getAttribute('maxlength')).toBe('60');
		expect(i18nMsgMock).toHaveBeenCalledWith(
			'bank-accounts-help-description',
			expect.stringContaining('60 caractères au plus')
		);
		unmount();
	});

	it('dit que description et compte lié se modifient ensuite', async () => {
		fetchAccountsMock.mockResolvedValue(PLAN);
		await atBankStep();
		const { unmount, getByTestId } = render(Page);
		expect(i18nMsgMock).toHaveBeenCalledWith(
			'onboarding-bank-editable-later',
			expect.stringContaining('Comptes bancaires')
		);
		expect(getByTestId('onboarding-bank-editable-later').textContent).toContain(
			'Administration → Comptes bancaires'
		);
		unmount();
	});
});
