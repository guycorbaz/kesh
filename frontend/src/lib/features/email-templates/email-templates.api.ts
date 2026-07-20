/**
 * Client API pour les modèles d'e-mail (Epic 20 #224, Story 20-2).
 *
 * Tous les endpoints sont **Admin uniquement** (guard backend
 * `require_admin_role`). La langue est passée en MAJUSCULES dans l'URL
 * (`Language::from_str` backend est case-sensitive — `fr` donnerait un 400).
 *
 * - `GET /api/v1/admin/email-templates` : tous les templates effectifs
 *   (type × langue × niveau) résolus (override ou défaut). Jamais vide. En config
 *   zéro-config = 20 (4 langues × [1 invoice_send niv.0 + 4 invoice_reminder niv.0-3]),
 *   dynamique au-delà (Story 21-4).
 * - `GET/PUT/DELETE .../{type}/{LANG}?level=<N>` : un template unique à un niveau
 *   (défaut 0). `DELETE` restaure le défaut (supprime l'override) et retourne `204`.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	EmailTemplateLanguage,
	EmailTemplateResponse,
	UpdateEmailTemplatePayload,
} from './email-templates.types';

export type {
	EmailTemplateLanguage,
	EmailTemplateResponse,
	UpdateEmailTemplatePayload,
} from './email-templates.types';

const BASE = '/api/v1/admin/email-templates';

/** Construit l'URL d'un template unique — langue MAJUSCULES + `?level=<N>` (défaut 0). */
function templateUrl(
	templateType: string,
	language: EmailTemplateLanguage,
	level = 0,
): string {
	return `${BASE}/${templateType}/${language.toUpperCase()}?level=${level}`;
}

/** Liste tous les templates effectifs (type × langue × niveau) de la company. */
export async function listEmailTemplates(): Promise<EmailTemplateResponse[]> {
	return apiClient.get<EmailTemplateResponse[]>(BASE);
}

/** Récupère un template effectif unique à un niveau (défaut 0). */
export async function getEmailTemplate(
	templateType: string,
	language: EmailTemplateLanguage,
	level = 0,
): Promise<EmailTemplateResponse> {
	return apiClient.get<EmailTemplateResponse>(templateUrl(templateType, language, level));
}

/** Crée ou modifie l'override à un niveau (verrou optimiste via `expectedVersion`). */
export async function updateEmailTemplate(
	templateType: string,
	language: EmailTemplateLanguage,
	payload: UpdateEmailTemplatePayload,
	level = 0,
): Promise<EmailTemplateResponse> {
	return apiClient.put<EmailTemplateResponse>(templateUrl(templateType, language, level), payload);
}

/** Restaure le défaut d'un niveau (supprime l'override) — idempotent, `204`. */
export async function restoreEmailTemplateDefault(
	templateType: string,
	language: EmailTemplateLanguage,
	level = 0,
): Promise<void> {
	return apiClient.delete<void>(templateUrl(templateType, language, level));
}
