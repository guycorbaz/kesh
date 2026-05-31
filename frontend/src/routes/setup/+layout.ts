/**
 * Story v011-5 — route /setup : route publique, accessible sans auth.
 *
 * Si l'utilisateur est déjà authentifié → redirect `/` (le 1er admin a déjà
 * été créé, plus rien à faire ici).
 *
 * Si la table users est vide côté backend → `/me` retourne 423 → l'auth store
 * a déjà set `_setupRequired = true` au boot, l'utilisateur arrive
 * naturellement ici via la redirection api-client.ts.
 *
 * AC #15 — pas de check auth qui force redirect /login depuis ici.
 */

import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;
export const prerender = false;

export function load() {
	if (browser && authState.isAuthenticated) {
		throw redirect(302, '/');
	}
}
