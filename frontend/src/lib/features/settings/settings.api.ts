import { apiClient } from '$lib/shared/utils/api-client';
import type { CompanyCurrentResponse, CompanyJson, UpdateCompanyEmailRequest } from './settings.types';

export async function fetchCompanyCurrent(): Promise<CompanyCurrentResponse> {
	return apiClient.get<CompanyCurrentResponse>('/api/v1/companies/current');
}

/**
 * Met à jour l'e-mail de contact de la société (Reply-To des factures
 * envoyées par e-mail) — Admin-only, verrou optimiste (Story 20-3b2).
 */
export async function updateCompanyEmail(req: UpdateCompanyEmailRequest): Promise<CompanyJson> {
	return apiClient.put<CompanyJson>('/api/v1/companies/current/email', req);
}
