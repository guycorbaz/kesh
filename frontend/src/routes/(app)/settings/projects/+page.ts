import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

// Story 19-1 — gestion des projets analytiques réservée à Comptable+ (mutations
// gardées `require_comptable_role` côté backend). Défense en profondeur client.
export function load() {
	const role = authState.currentUser?.role;
	if (browser && role !== 'Admin' && role !== 'Comptable') {
		throw redirect(302, '/');
	}
}
