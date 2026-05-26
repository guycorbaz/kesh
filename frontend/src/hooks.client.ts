/**
 * SvelteKit client hooks — Story 10-5 (T6.3bis) async auth hydration.
 *
 * Le hook `init` (SvelteKit ClientInit) est garanti exécuté AVANT toute
 * `load()` function et tout render. C'est OBLIGATOIRE pour Story 10-5 :
 * `authState.hydrate()` est maintenant async (fait `await fetch('/api/v1/auth/me')`)
 * pour restaurer l'identité utilisateur depuis le cookie HttpOnly. Sans
 * `await` ici, les load() de `(app)/+layout.ts:10` et `onboarding/+layout.ts:10`
 * s'exécuteraient avant que `_currentUser` soit peuplé → redirect /login
 * systématique pour TOUS les utilisateurs authentifiés (régression UX catastrophique).
 *
 * Pré-Story 10-5 : `authState.hydrate()` lisait synchronement localStorage,
 * donc l'appel synchrone fire-and-forget marchait. Post-Story 10-5 : le hook
 * doit être async pour attendre le round-trip /me.
 */

import type { ClientInit } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const init: ClientInit = async () => {
	try {
		await authState.hydrate();
	} catch (error) {
		// Hydration failure non-fatale — l'app continue avec state non-auth.
		console.error(
			'[auth] Hydration via /me failed at app startup:',
			error instanceof Error ? error.message : String(error),
		);
	}
};
