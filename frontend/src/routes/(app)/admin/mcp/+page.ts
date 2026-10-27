// Story 32-7 (refs #723) — garde de route Administrateur de `/admin/mcp` (AC-1),
// copie de `admin/backup/+page.ts` : la visibilité du menu est une garde d'écran,
// non une garde d'accès. Les routes `/api/v1/admin/mcp/*` refusent tout autre rôle
// et restent l'autorité.
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

export function load() {
	if (browser && authState.currentUser?.role !== 'Admin') {
		throw redirect(302, '/');
	}
}
