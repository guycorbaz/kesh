/**
 * Client API typé pour les exercices comptables (Story 3.7).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	ClosingChecksResponse,
	CreateFiscalYearRequest,
	FiscalYearResponse,
	ReopenFiscalYearRequest,
	UpdateFiscalYearRequest
} from './fiscal-years.types';

export async function listFiscalYears(): Promise<FiscalYearResponse[]> {
	return apiClient.get<FiscalYearResponse[]>('/api/v1/fiscal-years');
}

export async function getFiscalYear(id: number): Promise<FiscalYearResponse> {
	return apiClient.get<FiscalYearResponse>(`/api/v1/fiscal-years/${id}`);
}

export async function createFiscalYear(
	req: CreateFiscalYearRequest
): Promise<FiscalYearResponse> {
	return apiClient.post<FiscalYearResponse>('/api/v1/fiscal-years', req);
}

export async function updateFiscalYear(
	id: number,
	req: UpdateFiscalYearRequest
): Promise<FiscalYearResponse> {
	return apiClient.put<FiscalYearResponse>(`/api/v1/fiscal-years/${id}`, req);
}

export async function closeFiscalYear(id: number): Promise<FiscalYearResponse> {
	return apiClient.post<FiscalYearResponse>(`/api/v1/fiscal-years/${id}/close`, {});
}

/** Story 14-2 — rouvre un exercice clôturé (Admin, motif obligatoire). */
export async function reopenFiscalYear(
	id: number,
	req: ReopenFiscalYearRequest
): Promise<FiscalYearResponse> {
	return apiClient.post<FiscalYearResponse>(`/api/v1/fiscal-years/${id}/reopen`, req);
}

/**
 * Story 29-1a (#232) — les contrôles préalables à la clôture de l'exercice
 * (lecture seule, tout rôle). Ils ne bloquent rien : la clôture reste
 * l'autorité.
 */
export async function getClosingChecks(id: number): Promise<ClosingChecksResponse> {
	return apiClient.get<ClosingChecksResponse>(`/api/v1/fiscal-years/${id}/closing-checks`);
}
