/**
 * Client API typé pour les avoirs (notes de crédit) — Story 12.1.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateCreditNoteRequest,
	CreditNoteListItemResponse,
	CreditNoteResponse,
	ListCreditNotesQuery,
	ListResponse,
} from './credit-notes.types';

function buildQueryString(q: ListCreditNotesQuery): string {
	const p = new URLSearchParams();
	if (q.limit !== undefined) p.set('limit', String(q.limit));
	if (q.offset !== undefined) p.set('offset', String(q.offset));
	const s = p.toString();
	return s ? `?${s}` : '';
}

export async function listCreditNotes(
	query: ListCreditNotesQuery = {},
): Promise<ListResponse<CreditNoteListItemResponse>> {
	return apiClient.get(`/api/v1/credit-notes${buildQueryString(query)}`);
}

export async function getCreditNote(id: number): Promise<CreditNoteResponse> {
	return apiClient.get(`/api/v1/credit-notes/${id}`);
}

/** Crée et émet un avoir total contre-passant une facture validée (single-step). */
export async function createCreditNote(
	req: CreateCreditNoteRequest,
): Promise<CreditNoteResponse> {
	return apiClient.post('/api/v1/credit-notes', req);
}
