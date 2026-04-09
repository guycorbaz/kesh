import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';
import { syncModeFromServer } from '$lib/app/stores/mode.svelte';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';

export const ssr = false;

export async function load() {
	if (browser && !authState.isAuthenticated) {
		throw redirect(302, '/login');
	}

	// Check onboarding après auth — court-circuit si déjà chargé
	if (browser && authState.isAuthenticated && !onboardingState.loaded) {
		try {
			await onboardingState.fetchState();
		} catch (err) {
			if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
				throw err;
			}
			console.error('[onboarding guard] fetchState failed:', err);
			return;
		}
	}

	// Sync mode Guidé/Expert depuis le serveur (Story 2.5)
	if (browser && onboardingState.loaded && onboardingState.uiMode) {
		syncModeFromServer(onboardingState.uiMode);
	}

	// Seuil d'accès conditionnel Path A / Path B
	if (browser && onboardingState.loaded) {
		const step = onboardingState.stepCompleted;
		const isDemo = onboardingState.isDemo;

		// Path A (demo) : step < 3 → wizard obligatoire
		// Path B (production) : step < 6 → wizard obligatoire (banque optionnelle)
		if (isDemo && step < 3) {
			throw redirect(302, '/onboarding');
		}
		if (!isDemo && step < 6) {
			throw redirect(302, '/onboarding');
		}
	}
}
