/**
 * Client API pour les rappels débiteurs (dunning — Story 21-4, #231).
 *
 * - `GET /api/v1/dunning-levels` (tous rôles) : niveaux configurés.
 * - `POST`/`PUT`/`DELETE /api/v1/dunning-levels[/{id}]` : Administrateur (guard
 *   backend `require_admin_role`). Création = append (backend calcule le niveau) ;
 *   suppression = hard-delete + renumérotation.
 * - `GET`/`PUT /api/v1/company/dunning-settings` : période de grâce (singleton
 *   get-or-create + verrou optimiste).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	DunningLevelResponse,
	CreateDunningLevelPayload,
	UpdateDunningLevelPayload,
	DunningSettingsResponse,
	UpdateDunningSettingsRequest,
} from './dunning.types';

export type {
	DunningLevelResponse,
	CreateDunningLevelPayload,
	UpdateDunningLevelPayload,
	DunningSettingsResponse,
	UpdateDunningSettingsRequest,
} from './dunning.types';

const LEVELS_BASE = '/api/v1/dunning-levels';
const SETTINGS_BASE = '/api/v1/company/dunning-settings';

/** Liste les niveaux de rappel configurés (triés par niveau côté backend). */
export async function listDunningLevels(): Promise<DunningLevelResponse[]> {
	return apiClient.get<DunningLevelResponse[]>(LEVELS_BASE);
}

/** Ajoute un niveau à la suite (Administrateur). */
export async function createDunningLevel(
	payload: CreateDunningLevelPayload,
): Promise<DunningLevelResponse> {
	return apiClient.post<DunningLevelResponse>(LEVELS_BASE, payload);
}

/** Modifie un niveau (délai + frais) avec verrou optimiste (Administrateur). */
export async function updateDunningLevel(
	id: number,
	payload: UpdateDunningLevelPayload,
): Promise<DunningLevelResponse> {
	return apiClient.put<DunningLevelResponse>(`${LEVELS_BASE}/${id}`, payload);
}

/** Supprime un niveau + renumérote (Administrateur). `version` dans le body. */
export async function deleteDunningLevel(id: number, version: number): Promise<void> {
	return apiClient.delete<void>(`${LEVELS_BASE}/${id}`, { version });
}

/** Réglages de rappel (grâce). Le GET déclenche le seed lazy des défauts côté backend. */
export async function getDunningSettings(): Promise<DunningSettingsResponse> {
	return apiClient.get<DunningSettingsResponse>(SETTINGS_BASE);
}

/** Modifie la période de grâce avec verrou optimiste (Administrateur). */
export async function updateDunningSettings(
	req: UpdateDunningSettingsRequest,
): Promise<DunningSettingsResponse> {
	return apiClient.put<DunningSettingsResponse>(SETTINGS_BASE, req);
}
