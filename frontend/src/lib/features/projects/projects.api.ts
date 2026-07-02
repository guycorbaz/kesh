/**
 * Client API des projets analytiques (Epic 19, Story 19-1).
 *
 * - `GET /api/v1/projects` (tous rôles) — `?includeArchived=true` inclut les archivés.
 * - `POST`/`PUT`/`archive`/`unarchive` : Comptable+ (guard backend `require_comptable_role`).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type { NewProjectPayload, ProjectResponse, UpdateProjectPayload } from './projects.types';

export type { NewProjectPayload, ProjectResponse, UpdateProjectPayload } from './projects.types';

const BASE = '/api/v1/projects';

/** Liste les projets. `includeArchived=true` → inclut les projets archivés. */
export async function listProjects(includeArchived = false): Promise<ProjectResponse[]> {
	const url = includeArchived ? `${BASE}?includeArchived=true` : BASE;
	return apiClient.get<ProjectResponse[]>(url);
}

/** Crée un projet (Comptable+). */
export async function createProject(payload: NewProjectPayload): Promise<ProjectResponse> {
	return apiClient.post<ProjectResponse>(BASE, payload);
}

/** Modifie un projet avec verrou optimiste (Comptable+). */
export async function updateProject(
	id: number,
	payload: UpdateProjectPayload,
): Promise<ProjectResponse> {
	return apiClient.put<ProjectResponse>(`${BASE}/${id}`, payload);
}

/** Archive un projet (soft-delete) avec verrou optimiste (Comptable+). */
export async function archiveProject(id: number, version: number): Promise<ProjectResponse> {
	return apiClient.post<ProjectResponse>(`${BASE}/${id}/archive`, { version });
}

/** Désarchive un projet avec verrou optimiste (Comptable+). */
export async function unarchiveProject(id: number, version: number): Promise<ProjectResponse> {
	return apiClient.post<ProjectResponse>(`${BASE}/${id}/unarchive`, { version });
}
