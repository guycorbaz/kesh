/**
 * Story 36-17 (#1187, D4, AC5) — la garde de `/onboarding` (`+layout.ts`).
 *
 * Avant la story, elle renvoyait le chemin production à `/` dès l'étape 7, finalisation
 * faite ou non : une installation restée à 7 entrait dans l'application sans réglages de
 * facturation, taux de TVA ni exercice, et la page « Configuration terminée » aurait été
 * invisible au rechargement. Désormais : production 7 → écran de reprise, ≥ 8 → page
 * terminée (aucun renvoi) ; démo ≥ 3 → `/`, inchangé.
 *
 * ⚠️ Sous Vitest, `browser` (`$app/environment`) vaut faux et `load` ne fait RIEN : sans
 * le simulacre `browser: true`, « pas de redirect » passerait à vide. Sonde positive : les
 * cas « démo 3 → / » et « non authentifié → /login » sont verts AVANT le correctif, ce qui
 * prouve que `load` s'exécute bien sous ce simulacre.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

const { auth } = vi.hoisted(() => ({ auth: { isAuthenticated: true } }));

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/app/stores/auth.svelte', () => ({ authState: auth }));
vi.mock('$lib/features/onboarding/onboarding.api', () => ({
	fetchState: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
	loadI18nMessages: vi.fn(),
}));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import { load } from './+layout';

async function au(stepCompleted: number, isDemo: boolean) {
	vi.mocked(api.fetchState).mockResolvedValueOnce({ stepCompleted, isDemo, uiMode: null, isStub: false });
	await onboardingState.fetchState();
}

/** Le `redirect` levé par `load`, ou `null`. */
async function renvoi(): Promise<{ status: number; location: string } | null> {
	try {
		await load();
		return null;
	} catch (e) {
		const r = e as { status: number; location: string };
		return { status: r.status, location: r.location };
	}
}

beforeEach(() => {
	auth.isAuthenticated = true;
});

describe('garde de /onboarding (#1187, D4)', () => {
	it.each([7, 8, 10])('production, étape %i → pas de renvoi (l’écran de l’état)', async (step) => {
		await au(step, false);
		expect(await renvoi()).toBeNull();
	});

	it('démo, étape 3 → / (inchangé)', async () => {
		await au(3, true);
		expect(await renvoi()).toEqual({ status: 302, location: '/' });
	});

	it('non authentifié → /login (inchangé)', async () => {
		auth.isAuthenticated = false;
		expect(await renvoi()).toEqual({ status: 302, location: '/login' });
	});
});
