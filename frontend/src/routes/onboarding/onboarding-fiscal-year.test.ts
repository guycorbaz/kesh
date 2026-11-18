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
	sessionStorage.clear();
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

	// Intégration de la v0.18.0 (Story 36-17, #1187, D2) : l'échec de la finalisation
	// s'affiche à l'écran de l'étape 7, avec *Réessayer* — plus en toast. Et *Réessayer*
	// renvoie les dates choisies : sans elles, l'exercice serait créé aux dates par
	// défaut du serveur, en silence.
	it('un refus 400 de la finalisation s’affiche avec le motif du serveur ; Réessayer renvoie les dates choisies', async () => {
		vi.mocked(api.finalize).mockRejectedValueOnce({
			code: 'VALIDATION_ERROR',
			message: 'MOTIF-DATES',
			status: 400,
		});
		const { unmount } = await rendre();
		await saisir('fy-start', '2026-07-01');
		await saisir('fy-end', '2027-06-30');
		await fireEvent.click(boutonPlusTard());
		const motif = await screen.findByTestId('onboarding-finalize-motif');
		expect(motif.textContent).toContain('MOTIF-DATES');
		const retry = screen.getByTestId('onboarding-finalize-retry') as HTMLButtonElement;
		await waitFor(() => expect(retry.disabled).toBe(false));
		await fireEvent.click(retry);
		await waitFor(() => expect(api.finalize).toHaveBeenCalledTimes(2));
		for (const call of vi.mocked(api.finalize).mock.calls) {
			expect(call[0]).toEqual({
				fiscalYearStartDate: '2026-07-01',
				fiscalYearEndDate: '2027-06-30',
			});
		}
		unmount();
		await new Promise((r) => setTimeout(r, 0));
	});

	// Revue P1 de l'intégration de la v0.18.0 (H1) : le lien de remède d'un refus mène hors
	// de l'assistant ; au retour, la page est recréée. Les dates choisies doivent revenir,
	// affichées sur l'écran de reprise, et partir avec *Terminer la configuration* — sinon
	// l'exercice serait créé à l'année civile, sans retour possible.
	it('refus, départ par le lien de remède puis retour : les dates choisies sont affichées et envoyées', async () => {
		vi.mocked(api.finalize).mockRejectedValueOnce({
			code: 'VALIDATION_ERROR',
			message: 'MOTIF-REMEDE',
			status: 400,
		});
		const premier = await rendre();
		await saisir('fy-start', '2026-04-01');
		await fireEvent.click(boutonPlusTard());
		await screen.findByTestId('onboarding-finalize-motif');
		expect(screen.getByTestId('onboarding-finalize-remedy-link')).toBeTruthy();
		// Les dates restent sous les yeux de l'utilisateur sur l'écran d'échec.
		expect(champ('fy-start').value).toBe('2026-04-01');
		expect(champ('fy-end').value).toBe('2027-03-31');
		premier.unmount();
		await new Promise((r) => setTimeout(r, 0));

		// Retour par la bannière : page neuve, étape 7 complétée (écran de reprise).
		vi.mocked(api.fetchState).mockResolvedValue(etat(7));
		await onboardingState.fetchState();
		const second = render(Page);
		const finir = (await screen.findByTestId('onboarding-resume-finish')) as HTMLButtonElement;
		expect(champ('fy-start').value).toBe('2026-04-01');
		expect(champ('fy-end').value).toBe('2027-03-31');
		expect(screen.getByTestId('onboarding-fiscal-year-summary').textContent).toContain(
			'Du 01.04.2026 au 31.03.2027'
		);
		await waitFor(() => expect(finir.disabled).toBe(false));
		await fireEvent.click(finir);
		await waitFor(() => expect(api.finalize).toHaveBeenCalledTimes(2));
		expect(vi.mocked(api.finalize).mock.calls[1][0]).toEqual({
			fiscalYearStartDate: '2026-04-01',
			fiscalYearEndDate: '2027-03-31',
		});
		// Finalisation réussie : la saisie gardée est effacée.
		await waitFor(() =>
			expect(sessionStorage.getItem('kesh-onboarding-first-fiscal-year')).toBeNull()
		);
		second.unmount();
		await new Promise((r) => setTimeout(r, 0));
	});

	it('sur l’écran d’échec, une fin qui ne suit pas le début désactive Réessayer, et la touche Entrée ne finalise pas', async () => {
		vi.mocked(api.finalize).mockRejectedValueOnce({
			code: 'VALIDATION_ERROR',
			message: 'MOTIF',
			status: 400,
		});
		const { unmount } = await rendre();
		await fireEvent.click(boutonPlusTard());
		const retry = (await screen.findByTestId('onboarding-finalize-retry')) as HTMLButtonElement;
		await waitFor(() => expect(retry.disabled).toBe(false));
		await saisir('fy-end', '2025-12-31');
		expect((await screen.findByTestId('onboarding-fiscal-year-problem')).textContent).toContain(
			'La date de fin doit suivre la date de début.'
		);
		expect(retry.disabled).toBe(true);
		await fireEvent.submit(screen.getByTestId('onboarding-finalize-form'));
		await new Promise((r) => setTimeout(r, 0));
		expect(api.finalize).toHaveBeenCalledTimes(1);
		// Corrigée, la fin rend le bouton, et Entrée relance avec les dates affichées.
		await saisir('fy-end', '2026-06-30');
		await waitFor(() => expect(retry.disabled).toBe(false));
		await fireEvent.submit(screen.getByTestId('onboarding-finalize-form'));
		await waitFor(() => expect(api.finalize).toHaveBeenCalledTimes(2));
		expect(vi.mocked(api.finalize).mock.calls[1][0]).toEqual({
			fiscalYearStartDate: '2026-01-01',
			fiscalYearEndDate: '2026-06-30',
		});
		unmount();
		await new Promise((r) => setTimeout(r, 0));
	});

	it('un stockage illisible est ignoré : les dates par défaut', async () => {
		sessionStorage.setItem('kesh-onboarding-first-fiscal-year', '{pas du json');
		const { unmount } = await rendre();
		expect(champ('fy-start').value).toBe('2026-01-01');
		expect(champ('fy-end').value).toBe('2026-12-31');
		unmount();
	});
});
