/**
 * Client API typé pour les exercices comptables (Story 3.7).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CreateFiscalYearRequest,
	FiscalYearResponse,
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
