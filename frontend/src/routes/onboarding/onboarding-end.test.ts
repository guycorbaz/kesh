/**
 * Story 36-17 (#1187) — la fin de l'assistant : « Configuration en cours… », la page
 * « Configuration terminée » et son récapitulatif, l'échec et *Réessayer*, la reprise
 * après un rechargement, le chemin démo.
 *
 * Avant la story, la page n'avait aucun écran aux étapes 7 et 8 : la finalisation
 * tournait sur une page vide, le succès naviguait par un `goto('/')` non attendu et
 * l'échec n'était qu'un toast passager.
 *
 * Fixtures synthétiques seulement (« Exemple Banque SA », IBAN de démonstration).
 *
 * ⚠️ `_loading` est un état de module, et `finalize` relâche son verrou par
 * `setTimeout(0)` après un rejet : chaque cas relit l'état (`au`) et attend le tour
 * suivant après le démontage.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import type { OnboardingState } from '$lib/features/onboarding/onboarding.types';
import type { CompanyCurrentResponse } from '$lib/features/settings/settings.types';
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';

const { toastError, gotoMock, companyMock, yearsMock } = vi.hoisted(() => ({
	toastError: vi.fn(),
	gotoMock: vi.fn(),
	companyMock: vi.fn(),
	yearsMock: vi.fn(),
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
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: () => companyMock(),
}));
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({
	listFiscalYears: () => yearsMock(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
	loadI18nMessages: vi.fn(),
}));
vi.mock('$app/navigation', () => ({ goto: gotoMock }));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: toastError } }));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import Page from './+page.svelte';

const IBAN_DEMO = 'CH93 0076 2011 6238 5295 7';
const REMEDE_DEBUT = 'Si les comptes 1100 et 3000 figurent dans votre plan comptable';

const etat = (stepCompleted: number, isDemo = false): OnboardingState => ({
	stepCompleted,
	isDemo,
	uiMode: null,
	isStub: false,
});

function societe(bank: boolean): CompanyCurrentResponse {
	return {
		company: {
			id: 1,
			name: 'Exemple Conseil Sàrl',
			address: 'Rue d’Exemple 1, 1000 Lausanne',
			addressStructured: { street: 'Rue d’Exemple', building: '1', postalCode: '1000', city: 'Lausanne', country: 'CH' },
			addressCountryKnown: true,
			ideNumber: null,
			orgType: 'Pme',
			accountingLanguage: 'DE',
			instanceLanguage: 'FR',
			email: null,
			phone: null,
			website: null,
			booksLockedThrough: null,
			version: 3,
		},
		bankAccounts: bank
			? [
					{
						id: 1,
						bankName: 'Exemple Banque SA',
						description: 'Compte courant',
						iban: 'CH9300762011623852957',
						qrIban: null,
						isPrimary: true,
					},
				]
			: [],
	};
}

const EXERCICE: FiscalYearResponse = {
	id: 1,
	companyId: 1,
	name: 'Exercice 2026',
	startDate: '2026-01-01',
	endDate: '2026-12-31',
	status: 'Open',
	createdAt: '2026-11-15T10:00:00',
	updatedAt: '2026-11-15T10:00:00',
};

async function au(step: number, isDemo = false) {
	vi.mocked(api.fetchState).mockResolvedValueOnce(etat(step, isDemo));
	await onboardingState.fetchState();
}

function ecrans(container: HTMLElement): string[] {
	return Array.from(container.querySelectorAll('[data-testid^="onboarding-screen-"]')).map(
		(e) => (e.getAttribute('data-testid') ?? '').replace('onboarding-screen-', '')
	);
}

/** Une promesse dont on tient la résolution et le rejet. */
function enAttente<T>() {
	let resoudre: (v: T) => void = () => {};
	let rejeter: (e: unknown) => void = () => {};
	const promesse = new Promise<T>((r, j) => {
		resoudre = r;
		rejeter = j;
	});
	return { promesse, resoudre, rejeter };
}

let consoleError: ReturnType<typeof vi.spyOn>;
beforeEach(() => {
	vi.clearAllMocks();
	companyMock.mockResolvedValue(societe(true));
	yearsMock.mockResolvedValue([EXERCICE]);
	consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
});
afterEach(async () => {
	consoleError.mockRestore();
	// Laisser `finalize` relâcher son verrou (`setTimeout(0)`) avant le cas suivant.
	await new Promise((r) => setTimeout(r, 0));
});

describe('fin nominale (AC1)', () => {
	it('Configurer plus tard : « en cours » pendant la finalisation, puis la page terminée, sans navigation', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockResolvedValueOnce(etat(7));
		const fin = enAttente<OnboardingState>();
		vi.mocked(api.finalize).mockReturnValueOnce(fin.promesse);
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(ecrans(container)).toEqual(['finalizing']));
		const enCours = screen.getByTestId('onboarding-screen-finalizing');
		expect(enCours.getAttribute('role')).toBe('status');
		expect(enCours.getAttribute('aria-busy')).toBe('true');
		fin.resoudre(etat(8));
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		expect(screen.getByTestId('onboarding-done-start').getAttribute('href')).toBe('/');
		expect(screen.getByTestId('onboarding-done-start').hasAttribute('data-sveltekit-reload')).toBe(true);
		expect(gotoMock).not.toHaveBeenCalled();
		unmount();
	});

	it('Enregistrer (compte saisi) : même fin', async () => {
		await au(6);
		vi.mocked(api.setBankAccount).mockResolvedValueOnce(etat(7));
		vi.mocked(api.finalize).mockResolvedValueOnce(etat(8));
		const { container, unmount } = render(Page);
		await waitFor(() => expect((screen.getByText('Enregistrer') as HTMLButtonElement).disabled).toBe(false));
		const champ = (id: string, v: string) => {
			const el = document.getElementById(id) as HTMLInputElement;
			el.value = v;
			return fireEvent.input(el);
		};
		await champ('bank-name', 'Exemple Banque SA');
		await champ('bank-iban', IBAN_DEMO);
		await fireEvent.submit(document.querySelector('form') as HTMLFormElement);
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		expect(gotoMock).not.toHaveBeenCalled();
		unmount();
	});
});

describe('récapitulatif (AC3)', () => {
	it('société, type, langue comptable, exercice en date suisse, banque et description, sans IBAN', async () => {
		await au(8);
		const { container, unmount } = render(Page);
		await waitFor(() => expect(screen.queryByTestId('onboarding-done-summary')).not.toBeNull());
		const texte = screen.getByTestId('onboarding-done-summary').textContent ?? '';
		expect(texte).toContain('Exemple Conseil Sàrl');
		expect(texte).toContain('PME');
		expect(texte).toContain('Deutsch');
		expect(texte).toContain('Exercice 2026');
		expect(texte).toContain('01.01.2026');
		expect(texte).toContain('31.12.2026');
		expect(texte).toContain('Exemple Banque SA');
		expect(texte).toContain('Compte courant');
		expect(container.textContent).not.toContain('CH93');
		expect(container.textContent).not.toContain('CH9300762011623852957');
		expect(screen.getByTestId('onboarding-done-todo-vat')).toBeTruthy();
		expect(screen.queryByTestId('onboarding-done-todo-bank')).toBeNull();
		unmount();
	});

	it('sans compte bancaire : la phrase d’ajout', async () => {
		companyMock.mockResolvedValue(societe(false));
		await au(8);
		const { unmount } = render(Page);
		await waitFor(() => expect(screen.queryByTestId('onboarding-done-todo-bank')).not.toBeNull());
		expect(screen.getByTestId('onboarding-done-todo-vat')).toBeTruthy();
		unmount();
	});

	it('lecture en échec : la ligne et le motif ; titre, À faire fixe et Commencer restent', async () => {
		companyMock.mockRejectedValue({ code: 'INTERNAL_ERROR', message: 'MOTIF-RECAP', status: 500 });
		await au(8);
		const { container, unmount } = render(Page);
		await waitFor(() => expect(screen.queryByTestId('onboarding-done-summary-error')).not.toBeNull());
		expect(screen.getByTestId('onboarding-done-summary-error').textContent).toContain('MOTIF-RECAP');
		expect(ecrans(container)).toEqual(['done']);
		expect(screen.getByTestId('onboarding-done-todo-vat')).toBeTruthy();
		expect(screen.getByTestId('onboarding-done-start')).toBeTruthy();
		unmount();
	});
});

describe('échec et Réessayer (AC4)', () => {
	async function echouer(rejet: unknown) {
		await au(6);
		vi.mocked(api.skipBank).mockResolvedValueOnce(etat(7));
		vi.mocked(api.finalize).mockRejectedValueOnce(rejet);
		const r = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(ecrans(r.container)).toEqual(['finalize-error']));
		return r;
	}

	it('refus VALIDATION_ERROR : la phrase de remède AVANT le motif, sans toast', async () => {
		const { unmount } = await echouer({ code: 'VALIDATION_ERROR', message: 'MOTIF-FIN', status: 400 });
		const ecran = screen.getByTestId('onboarding-screen-finalize-error');
		expect(ecran.querySelector('[role="alert"]')).not.toBeNull();
		const remede = screen.getByTestId('onboarding-finalize-remedy');
		const motif = screen.getByTestId('onboarding-finalize-motif');
		expect(remede.textContent).toContain(REMEDE_DEBUT);
		expect(motif.textContent).toContain('MOTIF-FIN');
		// Ordre des nœuds : le remède précède le motif.
		expect(remede.compareDocumentPosition(motif) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
		// Revue P1 (E10) : le chemin du remède, d'un clic.
		expect(screen.getByTestId('onboarding-finalize-remedy-link').getAttribute('href')).toBe('/settings/invoicing');
		expect(toastError).not.toHaveBeenCalled();
		unmount();
	});

	it('TypeError : « Finalisation échouée — réessayez. », sans phrase de remède', async () => {
		const { unmount } = await echouer(new TypeError('Failed to fetch'));
		expect(screen.getByTestId('onboarding-finalize-motif').textContent).toContain('Finalisation échouée — réessayez.');
		expect(screen.queryByTestId('onboarding-finalize-remedy')).toBeNull();
		expect(toastError).not.toHaveBeenCalled();
		unmount();
	});

	it('autre code : le motif, sans phrase de remède', async () => {
		const { unmount } = await echouer({ code: 'INTERNAL_ERROR', message: 'MOTIF-500', status: 500 });
		expect(screen.getByTestId('onboarding-finalize-motif').textContent).toContain('MOTIF-500');
		expect(screen.queryByTestId('onboarding-finalize-remedy')).toBeNull();
		unmount();
	});

	it('Réessayer : désactivé pendant le verrou, puis « en cours » (erreur effacée), puis la page terminée', async () => {
		const { container, unmount } = await echouer(new TypeError('Failed to fetch'));
		const bouton = () => screen.getByTestId('onboarding-finalize-retry') as HTMLButtonElement;
		// Le verrou d'un tour du store (`setTimeout(0)`) : le bouton est désactivé juste après l'échec.
		expect(bouton().disabled).toBe(true);
		await waitFor(() => expect(bouton().disabled).toBe(false));
		const relance = enAttente<OnboardingState>();
		vi.mocked(api.finalize).mockReturnValueOnce(relance.promesse);
		await fireEvent.click(bouton());
		await waitFor(() => expect(ecrans(container)).toEqual(['finalizing']));
		expect(screen.queryByTestId('onboarding-screen-finalize-error')).toBeNull();
		relance.resoudre(etat(8));
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		expect(api.finalize).toHaveBeenCalledTimes(2);
		unmount();
	});

	it('ONBOARDING_STEP_ALREADY_COMPLETED : l’état est relu et son écran s’affiche', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockResolvedValueOnce(etat(7));
		vi.mocked(api.finalize).mockRejectedValueOnce({
			code: 'ONBOARDING_STEP_ALREADY_COMPLETED',
			message: 'MOTIF-DEJA',
			status: 400,
		});
		vi.mocked(api.fetchState).mockResolvedValueOnce(etat(6));
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(api.fetchState).toHaveBeenCalledTimes(2));
		await waitFor(() => expect(ecrans(container)).toEqual(['6']));
		expect(toastError).not.toHaveBeenCalled();
		unmount();
	});

	it('ONBOARDING_STEP_ALREADY_COMPLETED et relecture en échec : le motif à l’écran, avec Réessayer', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockResolvedValueOnce(etat(7));
		vi.mocked(api.finalize).mockRejectedValueOnce({
			code: 'ONBOARDING_STEP_ALREADY_COMPLETED',
			message: 'MOTIF-DEJA',
			status: 400,
		});
		vi.mocked(api.fetchState).mockRejectedValueOnce({ code: 'X', message: 'MOTIF-RELECTURE', status: 500 });
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(ecrans(container)).toEqual(['finalize-error']));
		expect(screen.getByTestId('onboarding-finalize-motif').textContent).toContain('MOTIF-RELECTURE');
		expect(screen.getByTestId('onboarding-finalize-retry')).toBeTruthy();
		unmount();
	});
});

describe('la fenêtre entre l’étape bancaire et la finalisation (D3, V1-3, V2-5)', () => {
	it('l’écran de reprise n’apparaît à aucun instant', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockResolvedValueOnce(etat(7));
		const fin = enAttente<OnboardingState>();
		vi.mocked(api.finalize).mockReturnValueOnce(fin.promesse);
		const { container, unmount } = render(Page);
		const apparitions: string[] = [];
		const observateur = new MutationObserver((enregistrements) => {
			for (const e of enregistrements) {
				for (const n of Array.from(e.addedNodes)) {
					if (!(n instanceof HTMLElement)) continue;
					if (n.matches('[data-testid="onboarding-screen-resume"]')) apparitions.push('resume');
					if (n.querySelector('[data-testid="onboarding-screen-resume"]')) apparitions.push('resume');
				}
			}
		});
		observateur.observe(container, { childList: true, subtree: true });
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(ecrans(container)).toEqual(['finalizing']));
		fin.resoudre(etat(8));
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		observateur.disconnect();
		expect(apparitions).toEqual([]);
		unmount();
	});
});

describe('reprise au rechargement (AC5)', () => {
	it('état 7 sans appel : l’écran de reprise, aucune finalisation au rendu ; Terminer la finalise', async () => {
		await au(7);
		vi.mocked(api.finalize).mockResolvedValueOnce(etat(8));
		const { container, unmount } = render(Page);
		expect(ecrans(container)).toEqual(['resume']);
		await new Promise((r) => setTimeout(r, 0));
		expect(api.finalize).not.toHaveBeenCalled();
		expect(screen.getByTestId('onboarding-screen-resume').textContent).toContain(
			"Configuration incomplète : terminez-la avant d'utiliser Kesh."
		);
		await fireEvent.click(screen.getByTestId('onboarding-resume-finish'));
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		expect(api.finalize).toHaveBeenCalledTimes(1);
		unmount();
	});

	it('Terminer la configuration est désactivé pendant un appel du store', async () => {
		await au(7);
		const lecture = enAttente<OnboardingState>();
		vi.mocked(api.fetchState).mockReturnValueOnce(lecture.promesse);
		const enCours = onboardingState.fetchState();
		const { unmount } = render(Page);
		expect((screen.getByTestId('onboarding-resume-finish') as HTMLButtonElement).disabled).toBe(true);
		lecture.resoudre(etat(7));
		await enCours;
		await waitFor(() =>
			expect((screen.getByTestId('onboarding-resume-finish') as HTMLButtonElement).disabled).toBe(false)
		);
		unmount();
	});
});

describe('chemin démo (AC8)', () => {
	it('chargement : son indicateur ; ensuite navigation attendue vers /', async () => {
		await au(2);
		const graine = enAttente<OnboardingState>();
		vi.mocked(api.seedDemo).mockReturnValueOnce(graine.promesse);
		gotoMock.mockResolvedValueOnce(undefined);
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-path-demo'));
		await waitFor(() => expect(ecrans(container)).toEqual(['demo-loading']));
		expect(screen.getByTestId('onboarding-screen-demo-loading').getAttribute('role')).toBe('status');
		graine.resoudre(etat(3, true));
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/'));
		expect(container.querySelector('[data-testid^="onboarding-org-type-"]')).toBeNull();
		unmount();
	});

	it('navigation en échec : l’écran « démo prête », Commencer et le motif', async () => {
		await au(2);
		vi.mocked(api.seedDemo).mockResolvedValueOnce(etat(3, true));
		gotoMock.mockRejectedValueOnce(new TypeError('navigation'));
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-path-demo'));
		await waitFor(() => expect(screen.queryByTestId('onboarding-demo-motif')).not.toBeNull());
		expect(ecrans(container)).toEqual(['demo-ready']);
		expect(screen.getByTestId('onboarding-demo-start').getAttribute('href')).toBe('/');
		unmount();
	});

	it('pendant le chargement, le lien de restauration disparaît (revue P1, A2)', async () => {
		await au(2);
		const graine = enAttente<OnboardingState>();
		vi.mocked(api.seedDemo).mockReturnValueOnce(graine.promesse);
		gotoMock.mockResolvedValueOnce(undefined);
		const { unmount } = render(Page);
		expect(screen.queryByTestId('onboarding-restore-link')).not.toBeNull();
		await fireEvent.click(screen.getByTestId('onboarding-path-demo'));
		await waitFor(() => expect(screen.queryByTestId('onboarding-screen-demo-loading')).not.toBeNull());
		expect(screen.queryByTestId('onboarding-restore-link')).toBeNull();
		graine.resoudre(etat(3, true));
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/'));
		unmount();
	});
});

/** Remplit l'étape bancaire et l'envoie (*Enregistrer*), une fois la liste des comptes chargée. */
async function enregistrerLeCompte() {
	await waitFor(() =>
		expect((screen.getByTestId('onboarding-bank-save') as HTMLButtonElement).disabled).toBe(false)
	);
	for (const [id, v] of [
		['bank-name', 'Exemple Banque SA'],
		['bank-iban', IBAN_DEMO],
	]) {
		const el = document.getElementById(id) as HTMLInputElement;
		el.value = v;
		await fireEvent.input(el);
	}
	await fireEvent.submit(document.querySelector('form') as HTMLFormElement);
}

function boutonsBancairesActifs() {
	expect((screen.getByTestId('onboarding-bank-save') as HTMLButtonElement).disabled).toBe(false);
	expect((screen.getByTestId('onboarding-skip-bank') as HTMLButtonElement).disabled).toBe(false);
}

describe('les drapeaux se relâchent après un refus (revue P1, B2, E1)', () => {
	it('refus de l’étape bancaire (Enregistrer) : écran 6, boutons de nouveau actifs', async () => {
		await au(6);
		vi.mocked(api.setBankAccount).mockRejectedValueOnce({ code: 'VALIDATION_ERROR', message: 'IBAN', status: 400 });
		const { container, unmount } = render(Page);
		await enregistrerLeCompte();
		await waitFor(() => expect(toastError).toHaveBeenCalledTimes(1));
		expect(ecrans(container)).toEqual(['6']);
		await waitFor(() => boutonsBancairesActifs());
		expect(api.finalize).not.toHaveBeenCalled();
		unmount();
	});

	it('refus de Configurer plus tard : écran 6, boutons de nouveau actifs', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockRejectedValueOnce({ code: 'X', message: 'REFUS', status: 400 });
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(toastError).toHaveBeenCalledTimes(1));
		expect(ecrans(container)).toEqual(['6']);
		await waitFor(() => boutonsBancairesActifs());
		unmount();
	});

	it('refus du chargement de la démo : écran 2, bouton de nouveau actif, pas d’indicateur', async () => {
		await au(2);
		vi.mocked(api.seedDemo).mockRejectedValueOnce({ code: 'X', message: 'REFUS-DEMO', status: 500 });
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-path-demo'));
		await waitFor(() => expect(toastError).toHaveBeenCalledWith('REFUS-DEMO'));
		await waitFor(() => expect(ecrans(container)).toEqual(['2']));
		expect((screen.getByTestId('onboarding-path-demo') as HTMLButtonElement).disabled).toBe(false);
		expect(gotoMock).not.toHaveBeenCalled();
		unmount();
	});

	it('étape bancaire qui ne rend pas l’étape 7 : aucune finalisation, écran de l’état (E5)', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockResolvedValueOnce(etat(6));
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(api.skipBank).toHaveBeenCalledTimes(1));
		await waitFor(() => boutonsBancairesActifs());
		expect(ecrans(container)).toEqual(['6']);
		expect(api.finalize).not.toHaveBeenCalled();
		unmount();
	});

	it('étape bancaire qui rend l’étape 8 (finalisée ailleurs) : la page terminée, sans finalisation (revue P2, LOW-1)', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockResolvedValueOnce(etat(8));
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		expect(api.finalize).not.toHaveBeenCalled();
		expect(toastError).not.toHaveBeenCalled();
		unmount();
	});

	it('double clic sur Configurer plus tard : un seul saut (B3, E3)', async () => {
		await au(6);
		const saut = enAttente<OnboardingState>();
		vi.mocked(api.skipBank).mockReturnValueOnce(saut.promesse);
		vi.mocked(api.finalize).mockResolvedValueOnce(etat(8));
		const { container, unmount } = render(Page);
		const bouton = screen.getByTestId('onboarding-skip-bank');
		// Deux événements dans la même tâche, avant toute mise à jour du DOM.
		bouton.click();
		bouton.click();
		saut.resoudre(etat(7));
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		expect(api.skipBank).toHaveBeenCalledTimes(1);
		expect(toastError).not.toHaveBeenCalled();
		unmount();
	});
});

describe('compléments de la revue P1 (A5, A6, E6, B7/E7)', () => {
	it('Enregistrer : « Configuration en cours… » pendant la finalisation (A5)', async () => {
		await au(6);
		vi.mocked(api.setBankAccount).mockResolvedValueOnce(etat(7));
		const fin = enAttente<OnboardingState>();
		vi.mocked(api.finalize).mockReturnValueOnce(fin.promesse);
		const { container, unmount } = render(Page);
		await enregistrerLeCompte();
		await waitFor(() => expect(ecrans(container)).toEqual(['finalizing']));
		fin.resoudre(etat(8));
		await waitFor(() => expect(ecrans(container)).toEqual(['done']));
		unmount();
	});

	it('lecture des exercices en échec : la ligne et le motif (A6)', async () => {
		yearsMock.mockRejectedValue({ code: 'INTERNAL_ERROR', message: 'MOTIF-EXERCICES', status: 500 });
		await au(8);
		const { unmount } = render(Page);
		await waitFor(() => expect(screen.queryByTestId('onboarding-done-summary-error')).not.toBeNull());
		expect(screen.getByTestId('onboarding-done-summary-error').textContent).toContain('MOTIF-EXERCICES');
		expect(screen.getByTestId('onboarding-done-start')).toBeTruthy();
		unmount();
	});

	it('exercices rendus dans l’ordre des dates (E6)', async () => {
		yearsMock.mockResolvedValue([
			{ ...EXERCICE, id: 2, name: 'Exercice 2027', startDate: '2027-01-01', endDate: '2027-12-31' },
			EXERCICE,
		]);
		await au(8);
		const { unmount } = render(Page);
		await waitFor(() => expect(screen.queryByTestId('onboarding-done-summary')).not.toBeNull());
		const texte = screen.getByTestId('onboarding-done-summary').textContent ?? '';
		expect(texte.indexOf('Exercice 2026')).toBeGreaterThanOrEqual(0);
		expect(texte.indexOf('Exercice 2026')).toBeLessThan(texte.indexOf('Exercice 2027'));
		unmount();
	});

	it('une carte de crédit se nomme comme telle et ne tient pas lieu de compte bancaire (B7, E7)', async () => {
		const avecCarte = societe(false);
		avecCarte.bankAccounts = [
			{
				id: 2,
				bankName: 'Exemple Carte',
				description: null,
				iban: null,
				qrIban: null,
				accountKind: 'credit_card',
				cardLast4: '4242',
				isPrimary: false,
			},
		];
		companyMock.mockResolvedValue(avecCarte);
		await au(8);
		const { unmount } = render(Page);
		await waitFor(() => expect(screen.queryByTestId('onboarding-done-summary')).not.toBeNull());
		const texte = screen.getByTestId('onboarding-done-summary').textContent ?? '';
		expect(texte).toContain('Carte de crédit');
		expect(texte).toContain('Exemple Carte');
		expect(texte).not.toContain('Compte bancaire');
		expect(screen.getByTestId('onboarding-done-todo-bank')).toBeTruthy();
		unmount();
	});
});
