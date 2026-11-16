/**
 * Story 37-1 (AC-11) — la section « Premier exercice » de la dernière page de
 * l'assistant : début au 1er janvier de l'année du poste, fin recalculée tant
 * qu'elle n'est pas retouchée, boutons désactivés avec le motif si la fin ne suit
 * pas le début, dates envoyées à `finalize` par les deux boutons.
 *
 * Le jour du poste est figé (`todayLocal` simulé) ; la simulation d'`i18nMsg`
 * substitue les variables, pour lire la forme suisse du résumé.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor, screen } from '@testing-library/svelte';

const { i18nMsgMock, toastError } = vi.hoisted(() => ({
	i18nMsgMock: vi.fn(
		(_key: string, fallback: string, args?: Record<string, string | number>) =>
			args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback
	),
	toastError: vi.fn(),
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
	fetchAccounts: vi.fn().mockResolvedValue([]),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: i18nMsgMock,
	loadI18nMessages: vi.fn(),
}));
vi.mock('$lib/shared/utils/local-date', async (original) => ({
	...(await original<typeof import('$lib/shared/utils/local-date')>()),
	todayLocal: () => '2026-11-16',
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

async function atBankStep() {
	vi.mocked(api.fetchState).mockResolvedValue(etat(6));
	await onboardingState.fetchState();
}

function champ(id: string): HTMLInputElement {
	return document.getElementById(id) as HTMLInputElement;
}

function saisir(id: string, valeur: string) {
	const el = champ(id);
	el.value = valeur;
	return fireEvent.input(el);
}

function boutonEnregistrer(): HTMLButtonElement {
	return document.querySelector('form button[type="submit"]') as HTMLButtonElement;
}

function boutonPlusTard(): HTMLButtonElement {
	return screen.getByText('Configurer plus tard').closest('button') as HTMLButtonElement;
}

async function rendre() {
	await atBankStep();
	const r = render(Page);
	// Fin du chargement de la liste des comptes (le bouton Enregistrer l'attend).
	await waitFor(() => expect(boutonEnregistrer().disabled).toBe(false));
	return r;
}

beforeEach(() => {
	vi.clearAllMocks();
	vi.mocked(api.setBankAccount).mockResolvedValue(etat(7));
	vi.mocked(api.skipBank).mockResolvedValue(etat(7));
	vi.mocked(api.finalize).mockResolvedValue(etat(8));
});

describe('assistant — premier exercice (Story 37-1, AC-11)', () => {
	it('propose l’année civile du poste, en forme suisse, et dit que les dates sont définitives', async () => {
		const { unmount, getByTestId } = await rendre();
		expect(champ('fy-start').value).toBe('2026-01-01');
		expect(champ('fy-end').value).toBe('2026-12-31');
		expect(getByTestId('onboarding-fiscal-year-summary').textContent).toContain(
			'Du 01.01.2026 au 31.12.2026'
		);
		expect(getByTestId('onboarding-fiscal-year-immutable').textContent).toContain(
			'ne se modifient plus une fois créé'
		);
		expect(i18nMsgMock).toHaveBeenCalledWith('onboarding-fiscal-year-title', 'Premier exercice');
		// La section précède le formulaire bancaire.
		const section = getByTestId('onboarding-fiscal-year');
		const form = document.querySelector('form') as HTMLFormElement;
		expect(section.compareDocumentPosition(form) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
		unmount();
	});

	it.each([
		['2026-04-01', '2027-03-31'],
		['2028-02-29', '2029-02-28'],
		['2027-03-01', '2028-02-29'],
	])('un début au %s recalcule la fin au %s', async (start, end) => {
		const { unmount } = await rendre();
		await saisir('fy-start', start);
		await waitFor(() => expect(champ('fy-end').value).toBe(end));
		unmount();
	});

	it('une fin retouchée n’est plus recalculée', async () => {
		const { unmount } = await rendre();
		await saisir('fy-end', '2027-06-30');
		await saisir('fy-start', '2026-07-01');
		await waitFor(() => expect(champ('fy-start').value).toBe('2026-07-01'));
		expect(champ('fy-end').value).toBe('2027-06-30');
		unmount();
	});

	it('une fin qui ne suit pas le début désactive les deux boutons, avec le motif', async () => {
		const { unmount, findByTestId, queryByTestId } = await rendre();
		await saisir('fy-end', '2026-01-01');
		expect((await findByTestId('onboarding-fiscal-year-problem')).textContent).toContain(
			'La date de fin doit suivre la date de début.'
		);
		expect(queryByTestId('onboarding-fiscal-year-summary')).toBeNull();
		expect(boutonEnregistrer().disabled).toBe(true);
		expect(boutonPlusTard().disabled).toBe(true);
		// La touche Entrée ne contourne pas le contrôle.
		await saisir('bank-name', 'Banque Exemple');
		await saisir('bank-iban', 'CH93 0076 2011 6238 5295 7');
		await fireEvent.submit(document.querySelector('form') as HTMLFormElement);
		await new Promise((r) => setTimeout(r, 0));
		expect(api.setBankAccount).not.toHaveBeenCalled();
		expect(api.finalize).not.toHaveBeenCalled();
		// Corrigée, la fin rend les boutons.
		await saisir('fy-end', '2026-12-31');
		await waitFor(() => expect(boutonPlusTard().disabled).toBe(false));
		expect(boutonEnregistrer().disabled).toBe(false);
		unmount();
	});

	it('une date effacée désactive les boutons, avec son motif', async () => {
		const { unmount, findByTestId } = await rendre();
		await saisir('fy-start', '');
		expect((await findByTestId('onboarding-fiscal-year-problem')).textContent).toContain(
			'Saisissez une date de début et une date de fin valides.'
		);
		expect(boutonPlusTard().disabled).toBe(true);
		unmount();
	});

	it('Enregistrer envoie les dates choisies à finalize', async () => {
		const { unmount } = await rendre();
		await saisir('fy-start', '2026-04-01');
		await saisir('bank-name', 'Banque Exemple');
		await saisir('bank-iban', 'CH93 0076 2011 6238 5295 7');
		await fireEvent.submit(document.querySelector('form') as HTMLFormElement);
		await waitFor(() => expect(api.finalize).toHaveBeenCalledTimes(1));
		expect(vi.mocked(api.finalize).mock.calls[0][0]).toEqual({
			fiscalYearStartDate: '2026-04-01',
			fiscalYearEndDate: '2027-03-31',
		});
		unmount();
		await new Promise((r) => setTimeout(r, 0));
	});

	it('Configurer plus tard envoie aussi les dates choisies', async () => {
		const { unmount } = await rendre();
		await saisir('fy-start', '2026-07-01');
		await saisir('fy-end', '2027-12-31');
		await fireEvent.click(boutonPlusTard());
		await waitFor(() => expect(api.finalize).toHaveBeenCalledTimes(1));
		expect(api.skipBank).toHaveBeenCalledTimes(1);
		expect(vi.mocked(api.finalize).mock.calls[0][0]).toEqual({
			fiscalYearStartDate: '2026-07-01',
			fiscalYearEndDate: '2027-12-31',
		});
		unmount();
		await new Promise((r) => setTimeout(r, 0));
	});

	it('un refus 400 de la finalisation s’affiche avec le motif du serveur', async () => {
		vi.mocked(api.finalize).mockRejectedValueOnce({
			code: 'VALIDATION_ERROR',
			message: 'MOTIF-DATES',
			status: 400,
		});
		const { unmount } = await rendre();
		await fireEvent.click(boutonPlusTard());
		await waitFor(() => expect(toastError).toHaveBeenCalledWith('MOTIF-DATES'));
		unmount();
		await new Promise((r) => setTimeout(r, 0));
	});
});
