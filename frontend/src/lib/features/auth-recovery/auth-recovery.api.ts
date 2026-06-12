/**
 * Story 17-4d — wrappers API du recovery de mot de passe self-service.
 *
 * Endpoints **publics** (pré-login, pas de cookie de session requis), montés
 * par le backend seulement si `KESH_FEATURE_FORGOT_PASSWORD=true` — sinon
 * `404` (le caller affiche un message générique d'indisponibilité).
 *
 * Anti-énumération (DC4) : `forgot-password` retourne TOUJOURS `200`, que le
 * compte existe ou non — ne jamais afficher de message différencié.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type { ResetPasswordResponse } from './auth-recovery.types';

/**
 * Demande l'envoi d'un lien de réinitialisation (`POST /auth/forgot-password`).
 *
 * Le `200` de succès a un **corps vide** (anti-énum 17-4c) — `request<T>` du
 * client le tolère (`res.json()` échoue → `undefined`, cf. api-client.ts:429).
 *
 * Erreurs propagées (`ApiError`) : `429 RATE_LIMITED` (5 req / 15 min / IP),
 * `404` si feature désactivé, `NETWORK_ERROR`/`TIMEOUT`.
 */
export async function requestPasswordReset(identifier: string): Promise<void> {
	await apiClient.post<void>('/api/v1/auth/forgot-password', { identifier });
}

/**
 * Consomme le token de reset et pose le nouveau mot de passe
 * (`POST /auth/reset-password`).
 *
 * Erreurs propagées (`ApiError`) :
 * - `400 INVALID_OR_EXPIRED_TOKEN` — token inconnu/expiré/déjà utilisé
 *   (générique par design, DC4 : ne pas tenter de distinguer les cas).
 * - `400 VALIDATION_ERROR` — politique de mot de passe (≥ 12 chars).
 * - `429 RATE_LIMITED` — limiter partagé avec forgot-password.
 */
export async function resetPassword(token: string, newPassword: string): Promise<void> {
	await apiClient.post<ResetPasswordResponse>('/api/v1/auth/reset-password', {
		token,
		newPassword,
	});
}
