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

/**
 * Pose ou **avance** le verrou de période (Admin + Comptable, Story 24-4c).
 *
 * ⚠️ Ce point d'entrée ne peut pas reculer la borne : le serveur refuse toute
 * date antérieure ou égale à la borne courante. Reculer relève de
 * `releaseBooksLock`, réservé à l'Admin avec motif.
 */
export async function lockBooks(through: string): Promise<CompanyJson> {
	return apiClient.post<CompanyJson>('/api/v1/companies/current/books-lock', { through });
}

/**
 * **Recule ou retire** le verrou de période (Admin seul, motif obligatoire).
 * `through = null` retire le verrou entièrement.
 */
export async function releaseBooksLock(
	through: string | null,
	motif: string
): Promise<CompanyJson> {
	return apiClient.post<CompanyJson>('/api/v1/companies/current/books-lock/release', {
		through,
		motif
	});
}
