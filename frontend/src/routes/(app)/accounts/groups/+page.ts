import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

/**
 * Garde Comptable+ (Story 46-5b, AC-1) — patron de `bank-import/manual/+page.ts` :
 * créer des groupes est réservé aux rôles Admin et Comptable, ceux qui modifient
 * déjà le plan (le backend garde aussi la route). Un rôle Consultation est
 * redirigé vers l'écran du plan, qui masque ses gestes sans rien refuser.
 */
export function load() {
	const role = authState.currentUser?.role;
	if (browser && role !== 'Admin' && role !== 'Comptable') {
		throw redirect(302, '/accounts');
	}
}
