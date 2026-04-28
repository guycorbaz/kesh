/**
 * Client API pour les taux TVA (Story 7.2 — KF-003 closure).
 *
 * Endpoint backend `GET /api/v1/vat-rates` : tous rôles authentifiés.
 * Pas de pagination — la liste tient en 4-10 entrées maxi.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type { VatRateResponse } from './vat-rates.types';

export async function listVatRates(): Promise<VatRateResponse[]> {
	return apiClient.get<VatRateResponse[]>('/api/v1/vat-rates');
}
