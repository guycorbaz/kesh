import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;
export const prerender = false;

/**
 * Story v011-5 (AC #19) — au boot racine : si `hydrate()` a détecté un 423
 * Locked (table `users` vide côté backend), `authState.isSetupRequired` est
 * `true` → redirect `/setup`. Pas de redirect si déjà sur `/setup` (évite
 * la boucle). L'api-client interceptor 423 gère également ce flow en
 * dynamique (e.g. user truncate DB pendant la session — théoretical).
 */
export function load({ url }: { url: URL }) {
	if (browser && authState.isSetupRequired && url.pathname !== '/setup') {
		throw redirect(302, '/setup');
	}
}
