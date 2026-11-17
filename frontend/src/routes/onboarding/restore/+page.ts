// Story 31-1b (AC15, #503) — garde de `/onboarding/restore`, copie de
// `(app)/admin/restore/+page.ts` (DC-D1) : sans elle, un non-Administrateur
// atteignant la page par URL directe verrait l'UI destructrice. Un non-Admin
// retourne à l'assistant (`/onboarding`), qu'il peut poursuivre ; le RBAC backend
// `require_admin_role` reste l'autorité de sécurité (403).
//
// Revue P1 (E-LOW-2) : la page suit aussi la borne du lien de l'assistant —
// étapes 0 à 2, avant le choix démo / production. Au-delà, l'installation porte
// déjà des données ; sa restauration passe par `Administration → Restaurer /
// Importer` une fois l'assistant terminé. La page renvoie donc à l'assistant.
//
// La garde d'authentification et la garde inverse de la démo (étape ≥ 3 → `/`) sont
// celles de `routes/onboarding/+layout.ts`, qui couvre cette page comme
// `/onboarding` ; `await parent()` attend qu'elle ait chargé l'état d'onboarding.
// Depuis la Story 36-17 (#1187, D4), le chemin production n'est plus renvoyé à `/` :
// `/onboarding` rend l'écran de son état (reprise à 7, page terminée à ≥ 8).
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { authState } from '$lib/app/stores/auth.svelte';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import type { PageLoad } from './$types';

export const ssr = false;

/** Dernière étape de l'assistant où la restauration est proposée (cf. le lien de `../+page.svelte`). */
const DERNIERE_ETAPE_RESTAURABLE = 2;

export const load: PageLoad = async ({ parent }) => {
	await parent();
	if (!browser) return;
	if (authState.currentUser?.role !== 'Admin') {
		throw redirect(302, '/onboarding');
	}
	if (onboardingState.loaded && onboardingState.stepCompleted > DERNIERE_ETAPE_RESTAURABLE) {
		throw redirect(302, '/onboarding');
	}
};
