/**
 * Client API typé pour les contacts (Story 4.1).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	ArchiveContactRequest,
	ContactResponse,
	CreateContactRequest,
	ListContactsQuery,
	ListResponse,
	UpdateContactRequest
} from './contacts.types';

function buildQueryString(q: ListContactsQuery): string {
	const params = new URLSearchParams();
	if (q.search) params.set('search', q.search);
	if (q.contactType) params.set('contactType', q.contactType);
	if (q.isClient !== undefined) params.set('isClient', String(q.isClient));
	if (q.isSupplier !== undefined) params.set('isSupplier', String(q.isSupplier));
	if (q.includeArchived) params.set('includeArchived', 'true');
	if (q.sortBy) params.set('sortBy', q.sortBy);
	if (q.sortDirection) params.set('sortDirection', q.sortDirection);
	if (q.limit !== undefined) params.set('limit', String(q.limit));
	if (q.offset !== undefined) params.set('offset', String(q.offset));
	const s = params.toString();
	return s ? `?${s}` : '';
}

export async function listContacts(
	query: ListContactsQuery = {}
): Promise<ListResponse<ContactResponse>> {
	return apiClient.get<ListResponse<ContactResponse>>(
		`/api/v1/contacts${buildQueryString(query)}`
	);
}

export async function getContact(id: number): Promise<ContactResponse> {
	return apiClient.get<ContactResponse>(`/api/v1/contacts/${id}`);
}

export async function createContact(req: CreateContactRequest): Promise<ContactResponse> {
	return apiClient.post<ContactResponse>('/api/v1/contacts', req);
}

export async function updateContact(
	id: number,
	req: UpdateContactRequest
): Promise<ContactResponse> {
	return apiClient.put<ContactResponse>(`/api/v1/contacts/${id}`, req);
}

export async function archiveContact(
	id: number,
	req: ArchiveContactRequest
): Promise<ContactResponse> {
	return apiClient.put<ContactResponse>(`/api/v1/contacts/${id}/archive`, req);
}
