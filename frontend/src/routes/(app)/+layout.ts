import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';

export const ssr = false;

export async function load() {
	if (browser && !authState.isAuthenticated) {
		throw redirect(302, '/login');
	}

	// Check onboarding après auth — si pas complété, redirect wizard.
	// Court-circuit si déjà chargé (évite double fetch sur navigation intra-app).
	if (browser && authState.isAuthenticated && !onboardingState.loaded) {
		try {
			await onboardingState.fetchState();
		} catch (err) {
			// Si c'est un redirect SvelteKit (throw redirect()), le re-throw
			if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
				throw err;
			}
			// Erreur réseau/API — ne pas bloquer l'accès, logguer et continuer.
			// Le apiClient gère 401 (refresh + redirect login) automatiquement.
			// Pour 503/timeout : on laisse l'app charger, la page affichera son
			// propre état d'erreur si nécessaire.
			console.error('[onboarding guard] fetchState failed:', err);
			return;
		}
	}

	if (browser && onboardingState.loaded && onboardingState.stepCompleted < 3) {
		throw redirect(302, '/onboarding');
	}
}
