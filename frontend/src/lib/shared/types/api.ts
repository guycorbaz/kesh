/**
 * Types partagés pour la communication avec l'API Kesh.
 *
 * - `ApiError` : erreur structurée retournée par le wrapper fetch.
 * - `PaginatedResponse<T>` : enveloppe de pagination standard.
 */

/** Erreur structurée conforme au format API `{ error: { code, message, details? } }`. */
export interface ApiError {
	code: string;
	message: string;
	details?: Record<string, unknown>;
	status: number;
}

/** Réponse paginée standard de l'API. */
export interface PaginatedResponse<T> {
	items: T[];
	total: number;
	offset: number;
	limit: number;
}
