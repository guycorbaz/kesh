import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

// Story 21-4 — réglages des rappels débiteurs réservés à l'Administrateur.
// Le backend applique aussi `require_admin_role` (défense en profondeur).
export function load() {
	if (browser && authState.currentUser?.role !== 'Admin') {
		throw redirect(302, '/');
	}
}
