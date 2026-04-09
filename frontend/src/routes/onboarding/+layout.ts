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
			if (onboardingState.stepCompleted >= 3) {
				throw redirect(302, '/');
			}
		} catch (err) {
			// Re-throw SvelteKit redirects
			if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
				throw err;
			}
			// API error — rester sur le wizard, l'utilisateur peut réessayer
			console.error('[onboarding layout] fetchState failed:', err);
		}
	}
}
