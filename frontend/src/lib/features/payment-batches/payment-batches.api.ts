/**
 * Client API typé pour les lots de paiement pain.001 — Story 12.3 (#191).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	ConfirmPaymentBatchRequest,
	CreatePaymentBatchRequest,
	CreatePaymentBatchResponse,
	ListPaymentBatchesQuery,
	ListResponse,
	PaymentBatchListItemResponse,
	PaymentBatchResponse,
} from './payment-batches.types';

function buildQueryString(q: ListPaymentBatchesQuery): string {
	const p = new URLSearchParams();
	if (q.limit !== undefined) p.set('limit', String(q.limit));
	if (q.offset !== undefined) p.set('offset', String(q.offset));
	const s = p.toString();
	return s ? `?${s}` : '';
}

export async function listPaymentBatches(
	query: ListPaymentBatchesQuery = {},
): Promise<ListResponse<PaymentBatchListItemResponse>> {
	return apiClient.get(`/api/v1/payment-batches${buildQueryString(query)}`);
}

export async function getPaymentBatch(id: number): Promise<PaymentBatchResponse> {
	return apiClient.get(`/api/v1/payment-batches/${id}`);
}

/** Crée un lot (génération) ; retourne le lot créé + les factures refusées. */
export async function createPaymentBatch(
	req: CreatePaymentBatchRequest,
): Promise<CreatePaymentBatchResponse> {
	return apiClient.post('/api/v1/payment-batches', req);
}

/** Confirme un lot : poste les règlements et marque les factures payées. */
export async function confirmPaymentBatch(
	id: number,
	req: ConfirmPaymentBatchRequest,
): Promise<PaymentBatchResponse> {
	return apiClient.post(`/api/v1/payment-batches/${id}/confirm`, req);
}

/** Annule un lot généré (déverrouille les factures). */
export async function cancelPaymentBatch(id: number): Promise<PaymentBatchResponse> {
	return apiClient.post(`/api/v1/payment-batches/${id}/cancel`, {});
}

/** URL de téléchargement du fichier pain.001 (GET direct, cookies inclus). */
export function pain001DownloadUrl(id: number): string {
	return `/api/v1/payment-batches/${id}/pain001`;
}
