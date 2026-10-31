import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

/**
 * Garde Comptable+ (Story v016-2b, #816) — patron de
 * `supplier-invoices/import/+page.ts` : saisir un relevé est réservé aux rôles
 * Admin et Comptable (le backend garde aussi les deux routes, C-v016-2-9). Un
 * rôle Consultation est redirigé vers l'accueil.
 */
export function load() {
	const role = authState.currentUser?.role;
	if (browser && role !== 'Admin' && role !== 'Comptable') {
		throw redirect(302, '/');
	}
}
