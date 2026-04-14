/**
 * Client API typé pour les factures brouillon (Story 5.1).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateInvoiceRequest,
	InvoiceListItemResponse,
	InvoiceResponse,
	ListInvoicesQuery,
	ListResponse,
	UpdateInvoiceRequest,
} from './invoices.types';

function buildQueryString(q: ListInvoicesQuery): string {
	const p = new URLSearchParams();
	if (q.search) p.set('search', q.search);
	if (q.status) p.set('status', q.status);
	if (q.contactId !== undefined) p.set('contactId', String(q.contactId));
	if (q.dateFrom) p.set('dateFrom', q.dateFrom);
	if (q.dateTo) p.set('dateTo', q.dateTo);
	if (q.sortBy) p.set('sortBy', q.sortBy);
	if (q.sortDirection) p.set('sortDirection', q.sortDirection);
	if (q.limit !== undefined) p.set('limit', String(q.limit));
	if (q.offset !== undefined) p.set('offset', String(q.offset));
	const s = p.toString();
	return s ? `?${s}` : '';
}

export async function listInvoices(
	query: ListInvoicesQuery = {},
): Promise<ListResponse<InvoiceListItemResponse>> {
	return apiClient.get(`/api/v1/invoices${buildQueryString(query)}`);
}

export async function getInvoice(id: number): Promise<InvoiceResponse> {
	return apiClient.get(`/api/v1/invoices/${id}`);
}

export async function createInvoice(req: CreateInvoiceRequest): Promise<InvoiceResponse> {
	return apiClient.post('/api/v1/invoices', req);
}

export async function updateInvoice(
	id: number,
	req: UpdateInvoiceRequest,
): Promise<InvoiceResponse> {
	return apiClient.put(`/api/v1/invoices/${id}`, req);
}

export async function deleteInvoice(id: number): Promise<void> {
	return apiClient.delete(`/api/v1/invoices/${id}`);
}
