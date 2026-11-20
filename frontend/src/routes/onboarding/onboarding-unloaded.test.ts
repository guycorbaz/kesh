/**
 * Story 36-17 (#1187, AC7, D9) — un état de l'assistant **non lu** a son écran.
 *
 * La garde de `/onboarding` avale l'échec de `fetchState()` (`+layout.ts`) : l'état reste
 * celui du module (`stepCompleted: 0`, `_loaded` faux). La page rendait alors l'écran de
 * l'étape 0 (choix de la langue) pour une installation peut-être à l'étape 7. Elle rend
 * désormais `onboarding-screen-unloaded` — « Chargement… » pendant une lecture, sinon le
 * constat et *Réessayer*, qui relit l'état.
 *
 * ⚠️ `_loaded` est un état de MODULE qui ne repasse jamais à faux : chaque fichier de test
 * a son propre module (isolation de Vitest), mais dans ce fichier les cas « non chargé »
 * doivent précéder toute lecture réussie. Ils forment donc **un seul** `it`, dans l'ordre.
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import type { OnboardingState } from '$lib/features/onboarding/onboarding.types';

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
	i18nMsg: (_key: string, fallback: string) => fallback,
	loadI18nMessages: vi.fn(),
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import Page from './+page.svelte';

const etat = (stepCompleted: number): OnboardingState => ({
	stepCompleted,
	isDemo: false,
	uiMode: null,
	isStub: false,
});

function ecrans(container: HTMLElement): string[] {
	return Array.from(container.querySelectorAll('[data-testid^="onboarding-screen-"]')).map(
		(e) => (e.getAttribute('data-testid') ?? '').replace('onboarding-screen-', '')
	);
}

describe('assistant — l’état non lu (#1187, D9, AC7)', () => {
	it('non lu : son écran, jamais celui de la langue ; Réessayer relit l’état', async () => {
		vi.spyOn(console, 'error').mockImplementation(() => {});
		// 1. Aucune lecture encore : loaded faux, loading faux (la garde a avalé l'échec).
		expect(onboardingState.loaded).toBe(false);
		const premier = render(Page);
		expect(ecrans(premier.container)).toEqual(['unloaded']);
		expect(premier.container.querySelector('[data-testid="onboarding-screen-0"]')).toBeNull();
		premier.unmount();

		// 2. Lecture en cours (chargement initial) : même écran, en son état « Chargement… ».
		let rejeter: (e: unknown) => void = () => {};
		vi.mocked(api.fetchState).mockReturnValueOnce(
			new Promise<OnboardingState>((_r, j) => {
				rejeter = j;
			})
		);
		const lecture = onboardingState.fetchState().catch(() => {});
		const second = render(Page);
		expect(ecrans(second.container)).toEqual(['unloaded']);
		expect(screen.getByTestId('onboarding-screen-unloaded').getAttribute('aria-busy')).toBe('true');
		// Pendant la lecture, pas de *Réessayer* actif.
		expect(screen.queryByTestId('onboarding-unloaded-retry')).toBeNull();
		second.unmount();
		// La lecture échoue (comme la garde l'aurait avalée) : toujours non lu.
		rejeter(new TypeError('Failed to fetch'));
		await lecture;
		expect(onboardingState.loaded).toBe(false);

		// 3. Réessayer : une relecture rejetée garde l'écran et dit le motif.
		const troisieme = render(Page);
		vi.mocked(api.fetchState).mockRejectedValueOnce({ code: 'X', message: 'MOTIF-LECTURE', status: 500 });
		await waitFor(() => expect(screen.queryByTestId('onboarding-unloaded-retry')).not.toBeNull());
		await fireEvent.click(screen.getByTestId('onboarding-unloaded-retry'));
		await waitFor(() => expect(screen.getByTestId('onboarding-unloaded-motif').textContent).toContain('MOTIF-LECTURE'));
		expect(ecrans(troisieme.container)).toEqual(['unloaded']);

		// 4. Réessayer encore : la relecture réussit, l'écran de l'état lu s'affiche.
		vi.mocked(api.fetchState).mockResolvedValueOnce(etat(7));
		await fireEvent.click(screen.getByTestId('onboarding-unloaded-retry'));
		await waitFor(() => expect(ecrans(troisieme.container)).toEqual(['resume']));
		expect(api.fetchState).toHaveBeenCalledTimes(3);
		troisieme.unmount();
	});
});
