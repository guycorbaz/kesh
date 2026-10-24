// Story 31-1b (AC15, #503) — le lien « Restaurer une sauvegarde » de l'assistant :
// étapes 0 à 2 seulement (avant le choix démo / production), Administrateur seulement.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';

const { auth } = vi.hoisted(() => ({
	auth: { currentUser: { role: 'Admin' } as { role: string } | null, isAuthenticated: true },
}));

vi.mock('$lib/app/stores/auth.svelte', () => ({ authState: auth }));
vi.mock('$lib/features/onboarding/onboarding.api', () => ({
	fetchState: vi.fn(),
	setLanguage: vi.fn(),
	setMode: vi.fn(),
	seedDemo: vi.fn(),
	resetDemo: vi.fn(),
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

async function atStep(stepCompleted: number, isDemo = false) {
	vi.mocked(api.fetchState).mockResolvedValue({
		stepCompleted,
		isDemo,
		uiMode: null,
		isStub: false,
	});
	await onboardingState.fetchState();
}

describe('assistant — lien « Restaurer une sauvegarde » (Story 31-1b, AC15)', () => {
	beforeEach(() => {
		auth.currentUser = { role: 'Admin' };
	});

	for (const step of [0, 1, 2]) {
		it(`Administrateur, étape ${step} : le lien mène à /onboarding/restore`, async () => {
			await atStep(step);
			const { unmount } = render(Page);
			const lien = screen.getByTestId('onboarding-restore-link');
			expect(lien.getAttribute('href')).toBe('/onboarding/restore');
			expect(lien.textContent).toContain('Restaurer une sauvegarde');
			unmount();
		});
	}

	for (const step of [3, 5, 6]) {
		it(`Administrateur, étape ${step} (après le choix démo / production) : pas de lien`, async () => {
			await atStep(step);
			const { unmount } = render(Page);
			expect(screen.queryByTestId('onboarding-restore-link')).toBeNull();
			unmount();
		});
	}

	for (const role of ['Comptable', 'Consultation']) {
		it(`${role}, étape 0 : pas de lien`, async () => {
			auth.currentUser = { role };
			await atStep(0);
			const { unmount } = render(Page);
			expect(screen.queryByTestId('onboarding-restore-link')).toBeNull();
			unmount();
		});
	}

	it('session sans utilisateur connu : pas de lien', async () => {
		auth.currentUser = null;
		await atStep(0);
		const { unmount } = render(Page);
		expect(screen.queryByTestId('onboarding-restore-link')).toBeNull();
		unmount();
	});
});
