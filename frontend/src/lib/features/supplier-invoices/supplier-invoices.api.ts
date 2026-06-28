/**
 * Client API typé pour les factures fournisseurs & règlement — Story 12.2 (#191).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateSupplierInvoiceRequest,
	ListResponse,
	ListSupplierInvoicesQuery,
	PaySupplierInvoiceRequest,
	SupplierInvoiceListItemResponse,
	SupplierInvoiceResponse,
} from './supplier-invoices.types';

function buildQueryString(q: ListSupplierInvoicesQuery): string {
	const p = new URLSearchParams();
	if (q.limit !== undefined) p.set('limit', String(q.limit));
	if (q.offset !== undefined) p.set('offset', String(q.offset));
	const s = p.toString();
	return s ? `?${s}` : '';
}

export async function listSupplierInvoices(
	query: ListSupplierInvoicesQuery = {},
): Promise<ListResponse<SupplierInvoiceListItemResponse>> {
	return apiClient.get(`/api/v1/supplier-invoices${buildQueryString(query)}`);
}

export async function getSupplierInvoice(id: number): Promise<SupplierInvoiceResponse> {
	return apiClient.get(`/api/v1/supplier-invoices/${id}`);
}

/** Enregistre une facture fournisseur (poste l'écriture d'achat, statut « ouverte »). */
export async function createSupplierInvoice(
	req: CreateSupplierInvoiceRequest,
): Promise<SupplierInvoiceResponse> {
	return apiClient.post('/api/v1/supplier-invoices', req);
}

/** Règle une facture fournisseur (choix binaire virement / compte interne). */
export async function paySupplierInvoice(
	id: number,
	req: PaySupplierInvoiceRequest,
): Promise<SupplierInvoiceResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${id}/pay`, req);
}

/** Annule une facture fournisseur « ouverte » (contre-passe l'écriture d'achat). */
export async function cancelSupplierInvoice(id: number): Promise<SupplierInvoiceResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${id}/cancel`, {});
}
