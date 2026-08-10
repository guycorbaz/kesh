import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CompanyCurrentResponse,
	CompanyJson,
	UpdateCompanyContactDetailsRequest,
	UpdateCompanyEmailRequest
} from './settings.types';

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

/**
 * Met à jour le téléphone et le site web de la société — rendus sur le PDF de
 * facture (Story 16-3a, #151). Admin-only, verrou optimiste, même patron que
 * `updateCompanyEmail`.
 */
export async function updateCompanyContactDetails(
	req: UpdateCompanyContactDetailsRequest
): Promise<CompanyJson> {
	return apiClient.put<CompanyJson>('/api/v1/companies/current/contact-details', req);
}
