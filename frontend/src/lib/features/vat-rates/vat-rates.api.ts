/**
 * Client API pour les taux TVA (Story 7.2 — KF-003 ; CRUD admin Story 11-1).
 *
 * - `GET /api/v1/vat-rates` (tous rôles) : taux actifs. `?history=true` →
 *   liste complète (actifs + expirés) pour la vue admin.
 * - `POST`/`PUT`/`DELETE` : Administrateur uniquement (guard backend
 *   `require_admin_role`, FR54). `DELETE` = désactivation soft.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	VatRateResponse,
	NewVatRatePayload,
	UpdateVatRatePayload,
} from './vat-rates.types';

export type {
	VatRateResponse,
	NewVatRatePayload,
	UpdateVatRatePayload,
} from './vat-rates.types';

const BASE = '/api/v1/vat-rates';

/** Liste les taux. `history=true` → actifs + expirés (vue admin). */
export async function listVatRates(history = false): Promise<VatRateResponse[]> {
	const url = history ? `${BASE}?history=true` : BASE;
	return apiClient.get<VatRateResponse[]>(url);
}

/** Crée un taux (Administrateur). Retourne le taux créé. */
export async function createVatRate(payload: NewVatRatePayload): Promise<VatRateResponse> {
	return apiClient.post<VatRateResponse>(BASE, payload);
}

/** Modifie un taux (label/validTo/active) avec verrou optimiste (Administrateur). */
export async function updateVatRate(
	id: number,
	payload: UpdateVatRatePayload,
): Promise<VatRateResponse> {
	return apiClient.put<VatRateResponse>(`${BASE}/${id}`, payload);
}

/** Désactive (soft-delete) un taux avec verrou optimiste (Administrateur). */
export async function deactivateVatRate(id: number, version: number): Promise<VatRateResponse> {
	return apiClient.delete<VatRateResponse>(`${BASE}/${id}`, { version });
}
