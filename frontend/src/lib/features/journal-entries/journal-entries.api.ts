/**
 * Client API pour les écritures comptables.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateJournalEntryRequest,
	JournalEntryDetailResponse,
	JournalEntryListQuery,
	JournalEntryResponse,
	ListResponse,
	UpdateJournalEntryRequest
} from './journal-entries.types';
import { serializeQuery } from './query-helpers';

/**
 * Récupère les écritures selon les filtres/tri/pagination fournis.
 *
 * **Story 3.4** : le type de retour est une envelope `ListResponse<T>`
 * `{ items, total, offset, limit }`. Les appelants qui consommaient
 * un tableau direct doivent faire `result.items`.
 */
export async function fetchJournalEntries(
	query: JournalEntryListQuery = {}
): Promise<ListResponse<JournalEntryResponse>> {
	const params = serializeQuery(query);
	const qs = params.toString();
	const url = qs ? `/api/v1/journal-entries?${qs}` : '/api/v1/journal-entries';
	return apiClient.get<ListResponse<JournalEntryResponse>>(url);
}

/**
 * Récupère le détail d'une écriture (lignes incluses) par son id.
 * `404` si l'écriture n'existe pas ou appartient à une autre company.
 */
export async function getJournalEntry(id: number): Promise<JournalEntryDetailResponse> {
	return apiClient.get<JournalEntryDetailResponse>(`/api/v1/journal-entries/${id}`);
}

/**
 * Contre-passe une écriture (Story 24-4a, #380).
 *
 * ⛔ Ne modifie rien : crée l'écriture inverse et rend celle-ci. L'origine
 * demeure — c'est la correction qui doit se voir, pas remplacer ce qu'elle
 * corrige.
 */
export async function reverseJournalEntry(id: number): Promise<JournalEntryResponse> {
	return apiClient.post<JournalEntryResponse>(`/api/v1/journal-entries/${id}/reverse`, {});
}

export async function createJournalEntry(
	req: CreateJournalEntryRequest
): Promise<JournalEntryResponse> {
	return apiClient.post<JournalEntryResponse>('/api/v1/journal-entries', req);
}

export async function updateJournalEntry(
	id: number,
	req: UpdateJournalEntryRequest
): Promise<JournalEntryResponse> {
	return apiClient.put<JournalEntryResponse>(`/api/v1/journal-entries/${id}`, req);
}

export async function deleteJournalEntry(id: number): Promise<void> {
	return apiClient.delete(`/api/v1/journal-entries/${id}`);
}
