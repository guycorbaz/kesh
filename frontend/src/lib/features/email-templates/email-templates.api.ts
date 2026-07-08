/**
 * Client API pour les modèles d'e-mail (Epic 20 #224, Story 20-2).
 *
 * Tous les endpoints sont **Admin uniquement** (guard backend
 * `require_admin_role`). La langue est passée en MAJUSCULES dans l'URL
 * (`Language::from_str` backend est case-sensitive — `fr` donnerait un 400).
 *
 * - `GET /api/v1/admin/email-templates` : les 4 combinaisons type×langue
 *   résolues (override ou défaut). Jamais vide.
 * - `GET/PUT/DELETE .../{type}/{LANG}` : un template unique. `DELETE` restaure
 *   le défaut (supprime l'override) et retourne `204`.
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

/** Construit l'URL d'un template unique — langue forcée en MAJUSCULES. */
function templateUrl(templateType: string, language: EmailTemplateLanguage): string {
	return `${BASE}/${templateType}/${language.toUpperCase()}`;
}

/** Liste les 4 templates effectifs (override ou défaut) de la company. */
export async function listEmailTemplates(): Promise<EmailTemplateResponse[]> {
	return apiClient.get<EmailTemplateResponse[]>(BASE);
}

/** Récupère un template effectif unique. */
export async function getEmailTemplate(
	templateType: string,
	language: EmailTemplateLanguage,
): Promise<EmailTemplateResponse> {
	return apiClient.get<EmailTemplateResponse>(templateUrl(templateType, language));
}

/** Crée ou modifie l'override (verrou optimiste via `expectedVersion`). */
export async function updateEmailTemplate(
	templateType: string,
	language: EmailTemplateLanguage,
	payload: UpdateEmailTemplatePayload,
): Promise<EmailTemplateResponse> {
	return apiClient.put<EmailTemplateResponse>(templateUrl(templateType, language), payload);
}

/** Restaure le défaut (supprime l'override) — idempotent, `204 No Content`. */
export async function restoreEmailTemplateDefault(
	templateType: string,
	language: EmailTemplateLanguage,
): Promise<void> {
	return apiClient.delete<void>(templateUrl(templateType, language));
}
