/**
 * Client API des projets analytiques (Epic 19, Story 19-1).
 *
 * - `GET /api/v1/projects` (tous rôles) — `?includeArchived=true` inclut les archivés.
 * - `POST`/`PUT`/`archive`/`unarchive` : Comptable+ (guard backend `require_comptable_role`).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type { NewProjectPayload, ProjectResponse, UpdateProjectPayload } from './projects.types';
import type { InvoiceResponse } from '$lib/features/invoices/invoices.types';

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

// ---------------------------------------------------------------------------
// Story 36-16a (#1167) — changer le compte analytique après coup. Un seul module
// d'API pour le geste : 36-16b (facture fournisseur) et 36-16c (écriture) y
// ajoutent leur fonction.
// ---------------------------------------------------------------------------

/** Réponse de `POST /api/v1/invoices/{id}/reassign-project`. */
export type ReassignInvoiceProjectResponse = {
	/** La facture à jour — la même qu'un `GET`, version comprise. */
	invoice: InvoiceResponse;
	/** Les écritures qui ont changé de compte analytique (vide : rien à faire, ou brouillon). */
	journalEntryIds: number[];
};

/**
 * Change le compte analytique d'une facture client et de toutes ses écritures,
 * quel que soit son statut (Comptable+). `projectId: null` retire le compte.
 *
 * Refus de période close : `409 PROJECT_REASSIGNMENT_CLOSED_PERIOD` (un
 * Administrateur confirme avec `acknowledge`), `403
 * PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY` (tout autre auteur).
 */
export async function reassignInvoiceProject(
	invoiceId: number,
	projectId: number | null,
	version: number,
	acknowledge = false,
): Promise<ReassignInvoiceProjectResponse> {
	return apiClient.post<ReassignInvoiceProjectResponse>(
		`/api/v1/invoices/${invoiceId}/reassign-project`,
		{ projectId, version, acknowledgeClosedPeriods: acknowledge },
	);
}
