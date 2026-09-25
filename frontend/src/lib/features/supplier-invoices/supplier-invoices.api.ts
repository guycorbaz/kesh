/**
 * Client API typé pour les factures fournisseurs & règlement — Story 12.2 (#191).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CancelSupplierSettlementResponse,
	CreateSupplierInvoiceRequest,
	ListResponse,
	ListSupplierInvoicesQuery,
	PaySupplierInvoiceRequest,
	ScanQrResponse,
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

/**
 * Annule le **règlement** d'une facture « payée » par contre-passation datée du
 * jour, et la ramène à « ouverte » — Story 25-3-a-2 (#414). Distinct de
 * `cancelSupplierInvoice`, qui annule la facture elle-même.
 */
export async function cancelSupplierInvoiceSettlement(
	id: number,
): Promise<CancelSupplierSettlementResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${id}/settlement/cancel`, {});
}

/** Annule une facture fournisseur « ouverte » (contre-passe l'écriture d'achat). */
export async function cancelSupplierInvoice(id: number): Promise<SupplierInvoiceResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${id}/cancel`, {});
}

/**
 * Pré-remplissage par scan QR (Story 12.4) : envoie le texte SPC décodé côté
 * navigateur (jsQR) et récupère les coordonnées de paiement. Lecture seule.
 */
export async function scanQrSupplierInvoice(spcText: string): Promise<ScanQrResponse> {
	return apiClient.post('/api/v1/supplier-invoices/scan-qr', { spcText });
}
