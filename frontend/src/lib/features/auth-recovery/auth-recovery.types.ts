/**
 * Types du module recovery de mot de passe self-service (Story 17-4d).
 *
 * Contrat API figé par la Story 17-4c (backend) — serde camelCase.
 */

/** Corps de `POST /api/v1/auth/forgot-password`. */
export interface ForgotPasswordRequest {
	/** Nom d'utilisateur OU email (un `@` aiguille vers le lookup email, DC6). */
	identifier: string;
}

/** Corps de `POST /api/v1/auth/reset-password`. */
export interface ResetPasswordRequest {
	/** Token brut reçu dans l'URL de l'email (query param `token`). */
	token: string;
	newPassword: string;
}

/** Réponse de `POST /api/v1/auth/reset-password` (succès). */
export interface ResetPasswordResponse {
	status: string;
}
