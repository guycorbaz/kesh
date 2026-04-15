/**
 * Client API typé pour les factures brouillon (Story 5.1).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateInvoiceRequest,
	DueDatesQuery,
	DueDatesResponse,
	InvoiceListItemResponse,
	InvoiceResponse,
	InvoiceSettingsResponse,
	ListInvoicesQuery,
	ListResponse,
	MarkPaidRequest,
	UnmarkPaidRequest,
	UpdateInvoiceRequest,
	UpdateInvoiceSettingsRequest,
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

// --- Story 5.2 : validation + config ---

export async function validateInvoice(id: number): Promise<InvoiceResponse> {
	return apiClient.post(`/api/v1/invoices/${id}/validate`, {});
}

export async function getInvoiceSettings(): Promise<InvoiceSettingsResponse> {
	return apiClient.get('/api/v1/company/invoice-settings');
}

export async function updateInvoiceSettings(
	req: UpdateInvoiceSettingsRequest,
): Promise<InvoiceSettingsResponse> {
	return apiClient.put('/api/v1/company/invoice-settings', req);
}

// --- Story 5.4 : échéancier ---

function buildDueDatesQueryString(q: DueDatesQuery): string {
	const p = new URLSearchParams();
	if (q.search) p.set('search', q.search);
	if (q.contactId !== undefined) p.set('contactId', String(q.contactId));
	if (q.dateFrom) p.set('dateFrom', q.dateFrom);
	if (q.dateTo) p.set('dateTo', q.dateTo);
	if (q.dueBefore) p.set('dueBefore', q.dueBefore);
	if (q.paymentStatus) p.set('paymentStatus', q.paymentStatus);
	if (q.sortBy) p.set('sortBy', q.sortBy);
	if (q.sortDirection) p.set('sortDirection', q.sortDirection);
	if (q.limit !== undefined) p.set('limit', String(q.limit));
	if (q.offset !== undefined) p.set('offset', String(q.offset));
	const s = p.toString();
	return s ? `?${s}` : '';
}

export async function listDueDates(query: DueDatesQuery = {}): Promise<DueDatesResponse> {
	return apiClient.get(`/api/v1/invoices/due-dates${buildDueDatesQueryString(query)}`);
}

export async function markInvoicePaid(id: number, req: MarkPaidRequest): Promise<InvoiceResponse> {
	return apiClient.post(`/api/v1/invoices/${id}/mark-paid`, req);
}

export async function unmarkInvoicePaid(
	id: number,
	req: UnmarkPaidRequest,
): Promise<InvoiceResponse> {
	return apiClient.post(`/api/v1/invoices/${id}/unmark-paid`, req);
}

/**
 * Télécharge l'export CSV échéancier (BOM UTF-8, `;`, CRLF, montants suisses).
 * Passe par `apiClient.getBlob()` pour conserver l'auth JWT + le refresh 401.
 */
export async function exportDueDatesCsv(query: DueDatesQuery = {}): Promise<Blob> {
	const res = await apiClient.getBlob(
		`/api/v1/invoices/due-dates/export.csv${buildDueDatesQueryString(query)}`,
	);
	return res.blob();
}
