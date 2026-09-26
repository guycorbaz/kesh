import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

/**
 * Garde Comptable+ (Story 25-1c-b1, #378) : le journal d'audit se consulte par
 * les rôles Admin et Comptable. Un rôle Consultation est redirigé vers
 * l'accueil. ⚠️ Le masquage d'interface ne protège rien : c'est le 403 de la
 * route (`/api/v1/audit-log`) qui fait foi.
 */
export function load() {
	const role = authState.currentUser?.role;
	if (browser && role !== 'Admin' && role !== 'Comptable') {
		throw redirect(302, '/');
	}
}
