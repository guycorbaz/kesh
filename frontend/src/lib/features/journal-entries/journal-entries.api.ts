/**
 * Client API pour les écritures comptables.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateJournalEntryRequest,
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
