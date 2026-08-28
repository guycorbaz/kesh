/**
 * Client API typé pour les factures brouillon (Story 5.1).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateInvoiceRequest,
	DueDatesQuery,
	DueDatesResponse,
	EmailPreviewResponse,
	InvoiceListItemResponse,
	InvoiceResponse,
	InvoiceSettingsResponse,
	ListInvoicesQuery,
	ListResponse,
	SettleInvoiceRequest,
	SendInvoiceEmailRequest,
	SettleInvoiceResponse,
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
	// Story 21-6a : `all` est le défaut backend (no-op) → jamais sérialisé,
	// cohérent avec la convention « on n'écrit que le non-défaut » de syncUrl.
	if (q.paused && q.paused !== 'all') p.set('paused', q.paused);
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

/**
 * Enregistre un règlement — Story 24-3 (#372).
 *
 * ⛔ **Remplace `markInvoicePaid` / `unmarkInvoicePaid`, supprimés avec leurs
 * routes.** Un marquage qui n'écrivait rien s'annulait gratuitement ; un
 * règlement qui produit son écriture se **contre-passe** (issue #414).
 *
 * ⚠️ Pas de `version` : enregistrer un règlement n'est pas modifier la facture,
 * c'est y ajouter un fait. Le garde qui compte est le refus du trop-perçu,
 * calculé côté serveur sous verrou.
 */
export async function settleInvoice(
	id: number,
	req: SettleInvoiceRequest,
): Promise<SettleInvoiceResponse> {
	return apiClient.post(`/api/v1/invoices/${id}/settlements`, req);
}

// Story 20-3b2 — Envoi de facture par e-mail (#224)

/** Preview pré-remplie (template rendu dans la langue du contact, destinataire verrouillé). */
export async function getInvoiceEmailPreview(id: number): Promise<EmailPreviewResponse> {
	return apiClient.get(`/api/v1/invoices/${id}/email-preview`);
}

/** Envoie la facture (PDF QR joint) au contact. Retourne la facture marquée (`emailedAt`/`emailedTo`). */
export async function sendInvoiceEmail(
	id: number,
	req: SendInvoiceEmailRequest,
): Promise<InvoiceResponse> {
	return apiClient.post(`/api/v1/invoices/${id}/send-email`, req);
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
