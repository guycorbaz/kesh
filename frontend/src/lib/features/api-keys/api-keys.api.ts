// Story 17-2b — Client API pour les clés API (PAT). Calque
// `features/bank-accounts/bank-accounts.api.ts`.
//
// Route backend : `/api/v1/settings/api-keys` (préfixe `settings/`, guard
// `require_comptable_role`, DC4). Le secret en clair n'est retourné qu'à la
// création (`CreatedApiKey.key`) ; le GET ne le renvoie jamais.

import { apiClient } from '$lib/shared/utils/api-client';
import type { ApiKey, CreatedApiKey, NewApiKeyPayload } from './api-keys.types';

// Ré-export pour que les consommateurs (page) importent types + fonctions
// depuis un seul module (ergonomie calquée sur `bank-accounts.api`).
export type { ApiKey, ApiKeyScope, CreatedApiKey, NewApiKeyPayload } from './api-keys.types';

const BASE = '/api/v1/settings/api-keys';

/**
 * GET /api/v1/settings/api-keys — liste **toutes** les clés de la company
 * (actives ET révoquées, triées `createdAt DESC` côté serveur). Multi-tenant
 * scoping serveur-side via la session JWT.
 */
export async function listApiKeys(): Promise<ApiKey[]> {
	return apiClient.get<ApiKey[]>(BASE);
}

/**
 * POST /api/v1/settings/api-keys — crée une clé et retourne le secret en clair
 * **une seule fois** (`CreatedApiKey.key`). Le backend valide `name` (non-vide,
 * ≤ 255) et `expiresAt` (futur uniquement) → `400` sinon.
 */
export async function createApiKey(payload: NewApiKeyPayload): Promise<CreatedApiKey> {
	return apiClient.post<CreatedApiKey>(BASE, payload);
}

/**
 * DELETE /api/v1/settings/api-keys/{id} — révocation soft-delete avec
 * optimistic lock (`version` dans le body, convention projet `bank-accounts`).
 * Backend retourne `204 No Content`. `404` si clé absente/autre company,
 * `409 OPTIMISTIC_LOCK_CONFLICT` si `version` périmée.
 */
export async function revokeApiKey(id: number, version: number): Promise<void> {
	await apiClient.delete<void>(`${BASE}/${id}`, { version });
}
