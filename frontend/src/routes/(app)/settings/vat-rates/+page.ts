import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

// Story 11-1 — gestion des taux TVA réservée à l'Administrateur (FR54).
// Le backend applique aussi `require_admin_role` (défense en profondeur).
export function load() {
	if (browser && authState.currentUser?.role !== 'Admin') {
		throw redirect(302, '/');
	}
}
