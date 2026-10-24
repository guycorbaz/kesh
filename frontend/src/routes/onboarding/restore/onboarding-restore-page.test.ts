// Story 31-1b (AC15, #503) — garde de `/onboarding/restore`, même règle de rôle
// que `(app)/admin/restore/+page.ts` : la page ne s'affiche pas non plus par URL
// directe pour un non-Administrateur (L7, P2). Le serveur refuse de toute façon (403).
// Revue P1 (E-LOW-2) : la page suit aussi la borne d'étape du lien (0 à 2).
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/features/onboarding/onboarding.api', () => ({
	fetchState: vi.fn(),
	setLanguage: vi.fn(),
	setMode: vi.fn(),
	seedDemo: vi.fn(),
	resetDemo: vi.fn(),
}));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';

/** Charge la page comme SvelteKit : `parent()` résolu (le layout a chargé l'état). */
async function charger(): Promise<unknown> {
	const { load } = await import('./+page');
	return (load as unknown as (e: { parent: () => Promise<unknown> }) => Promise<unknown>)({
		parent: async () => ({}),
	});
}

async function aEtape(stepCompleted: number) {
	vi.mocked(api.fetchState).mockResolvedValue({
		stepCompleted,
		isDemo: false,
		uiMode: null,
		isStub: false,
	});
	await onboardingState.fetchState();
}

async function attendRedirection(promesse: Promise<unknown>) {
	try {
		await promesse;
		expect.unreachable('redirection attendue');
	} catch (err: unknown) {
		const e = err as { status: number; location: string };
		expect(e.status).toBe(302);
		expect(e.location).toBe('/onboarding');
	}
}

describe('/onboarding/restore — garde', () => {
	beforeEach(async () => {
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }));
		await authState.logout();
		await aEtape(0);
	});

	for (const role of ['Comptable', 'Consultation']) {
		it(`${role} → redirection vers /onboarding`, async () => {
			authState.login({ userId: '2', username: 'u', role, expiresIn: 3600 });
			await attendRedirection(charger());
		});
	}

	for (const step of [0, 1, 2]) {
		it(`Administrateur, étape ${step} → la page se charge`, async () => {
			authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
			await aEtape(step);
			await expect(charger()).resolves.toBeUndefined();
		});
	}

	for (const step of [3, 6]) {
		it(`Administrateur, étape ${step} (après le choix démo / production) → redirection vers /onboarding`, async () => {
			authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 3600 });
			await aEtape(step);
			await attendRedirection(charger());
		});
	}
});
