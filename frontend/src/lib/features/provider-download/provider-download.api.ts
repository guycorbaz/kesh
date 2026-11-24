/**
 * Story 38-12b (AC-1 à AC-3, socle S7) — client **générique** des routes de
 * téléchargement par l'API d'un prestataire (38-12a, AC-8) :
 * `POST /api/v1/<prestataire>-downloads` et
 * `GET /api/v1/<prestataire>-downloads/{jobId}`. PayPal s'en sert ici ; Wise
 * (39-12b) et Stripe (40-8b) le reprennent avec leur segment de route.
 *
 * Les deux routes sont réservées aux sessions (une clé d'API est refusée en
 * `403 API_KEY_OPERATION_FORBIDDEN`) : l'écran n'atteint jamais ce refus.
 */
import { apiClient } from '$lib/shared/utils/api-client';

/**
 * Ce que la liste des comptes bancaires dit du téléchargement d'un compte de
 * prestataire (`providerApi`, 38-12b AC-1) : une série d'identifiants porte son
 * libellé ; `mode` absent pour un hôte ni réel ni d'essai (serveur simulé).
 * Jamais un secret.
 */
export interface ProviderApiStatus {
	configured: boolean;
	mode?: 'live' | 'sandbox';
}

/** Corps du `POST` : le libellé du compte du prestataire et la période (`AAAA-MM-JJ`). */
export interface ProviderDownloadRequest {
	account: string;
	from: string;
	to: string;
}

/** État d'une tâche (forme commune S3). */
export type ProviderDownloadState = 'running' | 'done' | 'failed';

/** L'erreur d'une tâche en échec : `{ code, message, details? }` (`details.provider` toujours). */
export interface ProviderDownloadError {
	code: string;
	/** Rendu et traduit par le serveur. */
	message: string;
	details?: Record<string, unknown>;
}

/**
 * Réponse de `GET /api/v1/<prestataire>-downloads/{jobId}` (forme commune S3) ;
 * `R` est le compte rendu propre au prestataire (PayPal : la réponse `201` de
 * l'import CSV plus `lastRefreshed`).
 */
export interface ProviderDownloadView<R> {
	state: ProviderDownloadState;
	/** Étapes faites et en tout (`total` nul : inconnu d'avance). */
	steps: { done: number; total: number | null };
	/** Secondes de pause restantes (limitation de débit), `null` hors pause. */
	pausedUntilSeconds: number | null;
	result: R | null;
	error: ProviderDownloadError | null;
}

/** La route d'un prestataire : `/api/v1/<prestataire>-downloads`. */
export function downloadsRoute(provider: string): string {
	return `/api/v1/${encodeURIComponent(provider)}-downloads`;
}

/** `POST` — lance la tâche ; `202 { jobId }`. Refus : `ApiError` (voir `startRefusal`). */
export async function startProviderDownload(
	provider: string,
	body: ProviderDownloadRequest,
): Promise<{ jobId: number }> {
	return apiClient.post<{ jobId: number }>(downloadsRoute(provider), body);
}

/** `GET` — la vue d'une tâche de la société de l'appelant (`404` : purgée ou serveur redémarré). */
export async function getProviderDownload<R>(provider: string, jobId: number): Promise<ProviderDownloadView<R>> {
	return apiClient.get<ProviderDownloadView<R>>(`${downloadsRoute(provider)}/${jobId}`);
}
