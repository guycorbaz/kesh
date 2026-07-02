import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

/**
 * Garde Comptable+ (Story 12.5d) : l'import et la complétion de factures sont
 * réservés aux rôles Admin et Comptable (le backend garde aussi via
 * `require_comptable_role`). Un rôle Consultation est redirigé vers l'accueil.
 */
export function load() {
	const role = authState.currentUser?.role;
	if (browser && role !== 'Admin' && role !== 'Comptable') {
		throw redirect(302, '/');
	}
}
