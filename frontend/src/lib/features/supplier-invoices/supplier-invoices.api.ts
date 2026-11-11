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
	UpdateSupplierInvoiceRequest,
} from './supplier-invoices.types';

function buildQueryString(q: ListSupplierInvoicesQuery): string {
	const p = new URLSearchParams();
	if (q.limit !== undefined) p.set('limit', String(q.limit));
	if (q.offset !== undefined) p.set('offset', String(q.offset));
	// Story v017-4b (AC-15) : valeurs séparées par des virgules ; une liste vide n'est
	// pas envoyée (le serveur refuserait `status=` vide en 400).
	if (q.status !== undefined && q.status.length > 0) p.set('status', q.status.join(','));
	if (q.q !== undefined && q.q.trim() !== '') p.set('q', q.q);
	if (q.sort !== undefined) p.set('sort', q.sort);
	if (q.order !== undefined) p.set('order', q.order);
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

/**
 * Enregistre un **brouillon** de facture fournisseur, sans écriture (Story v017-4a,
 * #849 : changement de contrat — jusqu'à la 0.17.0-rc1, la création passait
 * l'écriture d'achat). La facture se valide ensuite ({@link validateSupplierInvoice}).
 */
export async function createSupplierInvoice(
	req: CreateSupplierInvoiceRequest,
): Promise<SupplierInvoiceResponse> {
	return apiClient.post('/api/v1/supplier-invoices', req);
}

/**
 * Remplace un brouillon (Story v017-4b, AC-23) : en-tête et toutes les lignes, avec la
 * `version` lue. Rend la facture telle que le geste l'a écrite (lue avant le commit,
 * Story 34-8).
 */
export async function updateSupplierInvoice(
	id: number,
	req: UpdateSupplierInvoiceRequest,
): Promise<SupplierInvoiceResponse> {
	return apiClient.put(`/api/v1/supplier-invoices/${id}`, req);
}

/** Supprime un brouillon (Administrateur ; Story v017-4b, AC-22). */
export async function deleteSupplierInvoice(id: number): Promise<void> {
	await apiClient.delete(`/api/v1/supplier-invoices/${id}`);
}

/**
 * Valide un brouillon : l'écriture d'achat est passée (Story v017-4b, AC-22). Rend la
 * facture telle que le geste l'a écrite, `status: "open"` (affiché « Validée »).
 */
export async function validateSupplierInvoice(
	id: number,
	version: number,
): Promise<SupplierInvoiceResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${id}/validate`, { version });
}

/**
 * Repasse en brouillon une facture validée — ou, depuis la Story v0172-annulees (#868),
 * annulée : son écriture d'achat (et sa contre-passation) est **supprimée**, elle redevient
 * un brouillon modifiable (Story v017-4b, AC-22). Rend la facture telle que le geste l'a
 * écrite.
 */
export async function unvalidateSupplierInvoice(
	id: number,
	version: number,
): Promise<SupplierInvoiceResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${id}/unvalidate`, { version });
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
 * jour, et la ramène à « validée », à payer — Story 25-3-a-2 (#414). Distinct de
 * `cancelSupplierInvoice`, qui annule la facture elle-même.
 */
export async function cancelSupplierInvoiceSettlement(
	id: number,
): Promise<CancelSupplierSettlementResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${id}/settlement/cancel`, {});
}

/**
 * Annule une facture fournisseur, validée ou payée (Story 25-3-c) : contre-passe
 * l'écriture d'achat ; payée, son règlement reste au grand livre, détaché.
 * Rend la facture telle que le geste l'a écrite, avec ses champs de lecture.
 */
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
