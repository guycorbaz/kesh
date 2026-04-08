import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

export function load() {
	if (browser && !authState.isAuthenticated) {
		throw redirect(302, '/login');
	}
}
