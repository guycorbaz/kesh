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
			// Path B (production) : step >= 7 → onboarding terminé
			// step 6 = coordonnées saisies → app accessible avec bannière bleue
			if (!isDemo && step >= 7) {
				throw redirect(302, '/');
			}
		} catch (err) {
			if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
				throw err;
			}
			console.error('[onboarding layout] fetchState failed:', err);
		}
	}
}
