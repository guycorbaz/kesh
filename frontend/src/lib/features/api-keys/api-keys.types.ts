// Story 17-2b — Types pour la feature « clés API (PAT) ».
//
// Calque `features/bank-accounts/`. Le contrat backend (17-2a,
// `crates/kesh-api/src/routes/api_keys.rs`) sérialise en camelCase ; les dates
// sont des `NaiveDateTime` (sans timezone) → strings côté TS.

/** Scope d'une clé API. Valeurs exactes acceptées par le backend. */
export type ApiKeyScope = 'read' | 'read-write';

/**
 * Clé API telle que retournée par `GET /api/v1/settings/api-keys`.
 * Le secret n'est **jamais** présent ici (uniquement à la création, une fois).
 */
export interface ApiKey {
	id: number;
	name: string;
	scope: ApiKeyScope;
	/** ISO sans timezone (`NaiveDateTime`), ex. `2026-06-06T12:34:56.789`. */
	createdAt: string;
	lastUsedAt: string | null;
	revokedAt: string | null;
	expiresAt: string | null;
	/** Optimistic lock. */
	version: number;
}

/** Payload de création — `POST /api/v1/settings/api-keys`. */
export interface NewApiKeyPayload {
	name: string;
	scope: ApiKeyScope;
	/** RFC 3339 (`DateTime<Utc>`), optionnel. Backend refuse une date passée (400). */
	expiresAt?: string | null;
}

/**
 * Réponse de création — contient le secret en clair `key` (`kesh_pat_…`),
 * affiché **une seule fois**, jamais re-récupérable.
 */
export interface CreatedApiKey {
	id: number;
	name: string;
	scope: ApiKeyScope;
	createdAt: string;
	key: string;
}
