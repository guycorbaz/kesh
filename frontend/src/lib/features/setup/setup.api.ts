/**
 * Story v011-5 — wrapper API pour POST /api/v1/setup/admin.
 *
 * Le 1er admin créé via web reçoit les cookies HttpOnly (session immédiate),
 * puis le caller broadcaste login() pour synchroniser les autres onglets et
 * redirige vers /onboarding.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import { authState, type LoginPayload } from '$lib/app/stores/auth.svelte';

interface SetupAdminResponse {
	accessToken: string;
	refreshToken: string;
	expiresIn: number;
	userId: number;
	username: string;
	role: string;
}

/**
 * Crée le 1er admin via POST /api/v1/setup/admin et synchronise le state d'auth.
 *
 * Le backend émet les cookies HttpOnly automatiquement. Le caller doit
 * `await` cette fonction puis `goto('/onboarding')`.
 *
 * Erreurs propagées :
 * - 410 SETUP_ALREADY_COMPLETE → caller doit rediriger /login.
 * - 400 VALIDATION_ERROR → caller affiche message backend.
 * - 429 RATE_LIMITED → caller affiche message setup-error-rate-limit.
 */
export async function setupAdmin(username: string, password: string): Promise<void> {
	const data = await apiClient.post<SetupAdminResponse>('/api/v1/setup/admin', {
		username,
		password,
	});

	const payload: LoginPayload = {
		userId: String(data.userId),
		username: data.username,
		role: data.role,
		expiresIn: data.expiresIn,
	};

	// Story v011-5 ECH1-6 : broadcast cross-tab AVANT le redirect (le goto vient
	// du caller, après cette fonction). `authState.login` broadcast par défaut.
	authState.login(payload);
}
