/**
 * Client API typé pour les exercices comptables (Story 3.7).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	ClosingChecksResponse,
	CreateFiscalYearRequest,
	FiscalYearResponse,
	ReopenFiscalYearRequest,
	RevaluationPostResponse,
	RevaluationPreview,
	RevaluationRequest,
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
 * (lecture seule, tout rôle). Le panneau ne bloque rien par lui-même : deux
 * contrôles, K1 et K9 (Story 37-5), sont bloquants parce que la clôture
 * refuse réellement dans ces cas — elle reste l'autorité.
 */
export async function getClosingChecks(id: number): Promise<ClosingChecksResponse> {
	return apiClient.get<ClosingChecksResponse>(`/api/v1/fiscal-years/${id}/closing-checks`);
}

/**
 * Story 37-5 — aperçu de la réévaluation des comptes en devise (Comptable et
 * Administrateur ; n'écrit rien).
 */
export async function previewRevaluation(
	id: number,
	req: RevaluationRequest
): Promise<RevaluationPreview> {
	return apiClient.post<RevaluationPreview>(`/api/v1/fiscal-years/${id}/fx-revaluation/preview`, req);
}

/** Story 37-5 — passe la réévaluation : une écriture par compte réévalué. */
export async function postRevaluation(
	id: number,
	req: RevaluationRequest
): Promise<RevaluationPostResponse> {
	return apiClient.post<RevaluationPostResponse>(`/api/v1/fiscal-years/${id}/fx-revaluation`, req);
}
