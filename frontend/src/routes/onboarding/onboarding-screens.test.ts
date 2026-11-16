/**
 * Story 36-17 (#1187, AC7) — le détecteur : un écran pour chaque état de l'assistant.
 *
 * Pour chaque état chargé `stepCompleted ∈ {−1, 0, …, 10}` × `isDemo` × `loading`, la
 * page rend **exactement un** élément `[data-testid^="onboarding-screen-"]`, celui de la
 * correspondance de la fiche (D8), et la zone de l'assistant n'est pas vide. Aux étapes 6
 * et 7 du chemin production, les drapeaux locaux de la page (étape bancaire en attente,
 * finalisation en attente, rejetée, relancée) sont pilotés par les promesses de l'API
 * simulée. L'état **non chargé** a son propre fichier (`onboarding-unloaded.test.ts`) :
 * `_loaded` est un état de module qui ne repasse jamais à faux.
 *
 * Sonde positive (journal `kesh-gate-logs/e36-17/rouges.log`) : sur le code d'avant la
 * story, (7, production) et (8, production) rougissent sur « zone non vide », (3, démo)
 * sur l'écran du type d'organisation, et (0, production) PASSE « zone non vide » — preuve
 * que cette assertion n'est pas rouge partout.
 *
 * ⚠️ `_loading` est un état de module, et `finalize` relâche son verrou par
 * `setTimeout(0)` après un rejet : chaque cas relit l'état avant le rendu et attend le
 * tour suivant après le démontage.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import type { OnboardingState } from '$lib/features/onboarding/onboarding.types';

const { toastError } = vi.hoisted(() => ({ toastError: vi.fn() }));

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
// Le récapitulatif de la page terminée (D6) : ses lectures restent en attente, hors réseau.
vi.mock('$lib/features/settings/settings.api', () => ({
	fetchCompanyCurrent: vi.fn(() => new Promise(() => {})),
}));
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({
	listFiscalYears: vi.fn(() => new Promise(() => {})),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
	loadI18nMessages: vi.fn(),
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn(() => new Promise(() => {})) }));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: toastError } }));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import Page from './+page.svelte';

const etat = (stepCompleted: number, isDemo = false): OnboardingState => ({
	stepCompleted,
	isDemo,
	uiMode: null,
	isStub: false,
});

/** Pose l'état chargé ; si `loading`, laisse une relecture en attente (rendue ensuite par `liberer`). */
async function poser(stepCompleted: number, isDemo: boolean, loading: boolean): Promise<() => void> {
	vi.mocked(api.fetchState).mockResolvedValueOnce(etat(stepCompleted, isDemo));
	await onboardingState.fetchState();
	if (!loading) return () => {};
	let liberer: (v: OnboardingState) => void = () => {};
	vi.mocked(api.fetchState).mockReturnValueOnce(
		new Promise<OnboardingState>((r) => {
			liberer = r;
		})
	);
	const enCours = onboardingState.fetchState();
	return () => {
		liberer(etat(stepCompleted, isDemo));
		void enCours;
	};
}

/** La correspondance de la fiche (D8), pour un état chargé sans drapeau local. */
function attendu(step: number, isDemo: boolean): string {
	if (step < 0) return 'fallback';
	if (isDemo) return step >= 3 ? 'demo-ready' : String(step);
	if (step <= 6) return String(step);
	if (step === 7) return 'resume';
	return 'done';
}

function ecrans(container: HTMLElement): string[] {
	return Array.from(container.querySelectorAll('[data-testid^="onboarding-screen-"]')).map(
		(e) => (e.getAttribute('data-testid') ?? '').replace('onboarding-screen-', '')
	);
}

const ETAPES = [-1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const CAS = ETAPES.flatMap((step) =>
	[false, true].flatMap((isDemo) => [false, true].map((loading) => ({ step, isDemo, loading })))
);

beforeEach(() => {
	vi.clearAllMocks();
});

describe('assistant — un écran pour chaque état chargé (#1187, AC7)', () => {
	it.each(CAS.map((c) => [`étape ${c.step}, ${c.isDemo ? 'démo' : 'production'}, ${c.loading ? 'en chargement' : 'au repos'}`, c] as const))(
		'%s',
		async (_nom, { step, isDemo, loading }) => {
			const liberer = await poser(step, isDemo, loading);
			const { container, unmount } = render(Page);
			// Zone de l'assistant non vide : au moins un élément rendu.
			expect(container.querySelector('*'), 'zone de l’assistant vide').not.toBeNull();
			if (isDemo) {
				// L'écran du type d'organisation n'apparaît jamais en démo.
				expect(
					container.querySelector('[data-testid^="onboarding-org-type-"]'),
					'écran du type d’organisation en démo'
				).toBeNull();
			}
			expect(ecrans(container)).toEqual([attendu(step, isDemo)]);
			unmount();
			liberer();
			await new Promise((r) => setTimeout(r, 0));
		}
	);
});

describe('assistant — les drapeaux locaux aux étapes 6 et 7 (#1187, AC7, D3)', () => {
	async function au(step: number) {
		vi.mocked(api.fetchState).mockResolvedValueOnce(etat(step));
		await onboardingState.fetchState();
	}

	it('étape 6, étape bancaire en attente → l’écran bancaire reste', async () => {
		await au(6);
		vi.mocked(api.skipBank).mockReturnValueOnce(new Promise(() => {}));
		const { container, unmount } = render(Page);
		await fireEvent.click(screen.getByTestId('onboarding-skip-bank'));
		await waitFor(() => expect(api.skipBank).toHaveBeenCalledTimes(1));
		expect(ecrans(container)).toEqual(['6']);
		unmount();
	});

	it('étape 7 : finalisation en attente → en cours ; rejetée → échec ; relancée → en cours', async () => {
		await au(7);
		let rejeter: (e: unknown) => void = () => {};
		vi.mocked(api.finalize).mockReturnValueOnce(
			new Promise<OnboardingState>((_r, j) => {
				rejeter = j;
			})
		);
		const { container, unmount } = render(Page);
		expect(ecrans(container)).toEqual(['resume']);
		await fireEvent.click(screen.getByTestId('onboarding-resume-finish'));
		await waitFor(() => expect(ecrans(container)).toEqual(['finalizing']));
		rejeter({ code: 'INTERNAL_ERROR', message: 'MOTIF', status: 500 });
		await waitFor(() => expect(ecrans(container)).toEqual(['finalize-error']));
		// Le verrou d'un tour du store (`setTimeout(0)`) relâché, *Réessayer* redevient actif.
		await waitFor(() =>
			expect((screen.getByTestId('onboarding-finalize-retry') as HTMLButtonElement).disabled).toBe(false)
		);
		vi.mocked(api.finalize).mockReturnValueOnce(new Promise(() => {}));
		await fireEvent.click(screen.getByTestId('onboarding-finalize-retry'));
		await waitFor(() => expect(ecrans(container)).toEqual(['finalizing']));
		unmount();
		await new Promise((r) => setTimeout(r, 0));
	});
});
