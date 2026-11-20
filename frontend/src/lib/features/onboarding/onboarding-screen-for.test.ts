/**
 * Story 36-17 (#1187, D8) — `onboardingScreenFor`, branches que la matrice de la page ne
 * joue pas (revue P1, E9) : une étape non entière tombe sur l'écran de repli, jamais sur
 * l'écran d'une étape voisine.
 */
import { describe, it, expect } from 'vitest';
import { onboardingScreenFor } from './onboarding-screens';

const base = { loaded: true, isDemo: false, finalizing: false, finalizeFailed: false, demoLoading: false };

describe('onboardingScreenFor — étapes non entières (E9)', () => {
	it.each([1.5, 7.5, Number.NaN, Number.POSITIVE_INFINITY])('étape %s → repli', (stepCompleted) => {
		expect(onboardingScreenFor({ ...base, stepCompleted })).toBe('fallback');
	});

	it('démo à l’étape 1,5 → repli aussi', () => {
		expect(onboardingScreenFor({ ...base, isDemo: true, stepCompleted: 1.5 })).toBe('fallback');
	});
});
