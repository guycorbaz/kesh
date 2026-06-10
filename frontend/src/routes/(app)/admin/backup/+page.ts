// Story 17-3b — Guard de route Admin (review Pass 1) : la visibilité sidebar
// (`isAdmin`) est une garde UX, pas une garde d'accès. Sans ce `+page.ts`, un
// non-Admin atteignant `/admin/backup` par URL directe verrait la page (le
// backend renverrait 403 à l'export, mais l'UI admin ne doit pas s'afficher).
// Pattern identique à `(app)/users/+page.ts`. Défense en profondeur : le RBAC
// backend `require_admin_role` reste l'autorité de sécurité.
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

export function load() {
	if (browser && authState.currentUser?.role !== 'Admin') {
		throw redirect(302, '/');
	}
}
