import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';

export const ssr = false;
export const prerender = false;

export async function load() {
	if (browser && !authState.isAuthenticated) {
		throw redirect(302, '/login');
	}

	// Guard inverse : si onboarding complété, redirect vers app
	if (browser && authState.isAuthenticated) {
		try {
			if (!onboardingState.loaded) {
				await onboardingState.fetchState();
			}

			const step = onboardingState.stepCompleted;
			const isDemo = onboardingState.isDemo;

			// Path A (demo) : step >= 3 → onboarding terminé
			if (isDemo && step >= 3) {
				throw redirect(302, '/');
			}
			// Path B (production) : AUCUN renvoi (Story 36-17, #1187, D4). La garde
			// renvoyait à `/` dès l'étape 7, finalisation faite ou non : une installation
			// restée à 7 entrait dans l'application sans exercice, taux de TVA ni réglages
			// de facturation, et la page « Configuration terminée » aurait été invisible au
			// rechargement. La page rend désormais l'écran de l'état : reprise à 7, page
			// terminée à ≥ 8. L'application, elle, reste ouverte dès l'étape 6
			// (`(app)/+layout.ts`), avec la bannière « Configuration incomplète » aux
			// étapes 6 et 7.
		} catch (err) {
			if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
				throw err;
			}
			console.error('[onboarding layout] fetchState failed:', err);
		}
	}
}
