/**
 * Types pour la feature `email-templates` (Epic 20 #224, Story 20-2).
 *
 * Consomme l'API `email_templates` livrée par la Story 20-1 (backend). Un
 * template « effectif » est soit un override en base (`isDefault: false`,
 * `version` entière), soit le texte par défaut applicatif (`isDefault: true`,
 * `version: null`). Le fallback défaut est le comportement zéro-config normal,
 * jamais une erreur.
 */

/** Code langue tel qu'exposé par l'API — TOUJOURS en majuscules. */
export type EmailTemplateLanguage = 'FR' | 'DE' | 'IT' | 'EN';

/** Les 4 langues supportées, dans l'ordre d'affichage des onglets. */
export const EMAIL_TEMPLATE_LANGUAGES: EmailTemplateLanguage[] = ['FR', 'DE', 'IT', 'EN'];

/** Un template effectif (override ou défaut) renvoyé par l'API. */
export type EmailTemplateResponse = {
	/** Type de template (v1 : `'invoice_send'` uniquement). */
	templateType: string;
	language: EmailTemplateLanguage;
	subject: string;
	body: string;
	/** Verrou optimiste — `null` quand `isDefault: true` (aucune ligne en base). */
	version: number | null;
	/** `true` = texte par défaut applicatif ; `false` = override personnalisé. */
	isDefault: boolean;
	/** Variables `{var}` autorisées dans subject/body de ce type. */
	allowedVariables: string[];
};

/**
 * Corps de `PUT /api/v1/admin/email-templates/{type}/{LANG}`.
 *
 * `expectedVersion` porte la sémantique double du backend :
 * - `null`/absent → le client croit qu'aucun override n'existe (création).
 * - entier → modification de l'override existant à cette version.
 */
export type UpdateEmailTemplatePayload = {
	subject: string;
	body: string;
	expectedVersion?: number | null;
};
