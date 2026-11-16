/**
 * Story 36-17 (#1187, D5, AC6) — la bannière « Configuration incomplète ».
 *
 * L'application reste ouverte aux étapes 6 et 7 (garde `(app)/+layout.ts` inchangée) ;
 * avant la story, la bannière ne s'affichait qu'à l'étape 6, si bien qu'une installation
 * restée à l'étape 7 (finalisation interrompue) entrait dans Kesh sans rien qui le
 * signale. La condition est extraite en fonction pure, lue par `(app)/+layout.svelte` ;
 * le texte dit désormais de terminer avant d'utiliser Kesh (la limite #1200 qui le
 * motivait est corrigée par la 36-19, même release ; la consigne reste juste).
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
	loadI18nMessages: vi.fn(),
}));

import { showIncompleteBanner } from './onboarding-screens';
import IncompleteBanner from '$lib/shared/components/IncompleteBanner.svelte';

const etat = (stepCompleted: number, isDemo = false, loaded = true) => ({ stepCompleted, isDemo, loaded });

describe('showIncompleteBanner (#1187, D5)', () => {
	it.each([
		[5, false],
		[6, true],
		[7, true],
		[8, false],
		[10, false],
	])('production, étape %i → %s', (step, attendu) => {
		expect(showIncompleteBanner(etat(step))).toBe(attendu);
	});

	it('démo → jamais', () => {
		expect(showIncompleteBanner(etat(6, true))).toBe(false);
		expect(showIncompleteBanner(etat(7, true))).toBe(false);
	});

	it('état non lu → jamais', () => {
		expect(showIncompleteBanner(etat(7, false, false))).toBe(false);
	});
});

describe('IncompleteBanner — le texte neuf (#1187, AC6)', () => {
	it('dit de terminer avant d’utiliser Kesh ; le lien mène à /onboarding', () => {
		const { unmount } = render(IncompleteBanner);
		const banniere = screen.getByTestId('incomplete-config-banner');
		expect(banniere.textContent).toContain("Configuration incomplète : terminez-la avant d'utiliser Kesh.");
		expect(banniere.querySelector('a')?.getAttribute('href')).toBe('/onboarding');
		unmount();
	});
});
