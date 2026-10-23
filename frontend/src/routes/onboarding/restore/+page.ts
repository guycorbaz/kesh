// Story 31-1b (AC15, #503) — garde de rôle de `/onboarding/restore`, copie de
// `(app)/admin/restore/+page.ts` (DC-D1) : sans elle, un non-Administrateur
// atteignant la page par URL directe verrait l'UI destructrice. Un non-Admin
// retourne à l'assistant (`/onboarding`), qu'il peut poursuivre ; le RBAC backend
// `require_admin_role` reste l'autorité de sécurité (403).
//
// La garde d'authentification et la garde inverse « assistant terminé → / » sont
// celles de `routes/onboarding/+layout.ts`, qui couvre cette page comme
// `/onboarding`.
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';

export const ssr = false;

export function load() {
	if (browser && authState.currentUser?.role !== 'Admin') {
		throw redirect(302, '/onboarding');
	}
}
