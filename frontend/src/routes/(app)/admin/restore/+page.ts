// Story 17-3d — Guard de route Admin (DC-D1, copie de admin/backup/+page.ts).
// La visibilité sidebar (`isAdmin`) est une garde UX, pas une garde d'accès.
// Sans ce `+page.ts`, un non-Admin atteignant `/admin/restore` par URL directe
// verrait l'UI destructrice. Pattern identique à `(app)/users/+page.ts`. Le
// RBAC backend `require_admin_role` reste l'autorité de sécurité.
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

export function load() {
	if (browser && authState.currentUser?.role !== 'Admin') {
		throw redirect(302, '/');
	}
}
