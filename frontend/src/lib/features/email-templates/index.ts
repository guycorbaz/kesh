/**
 * Re-exports publics de la feature `email-templates` (Epic 20 #224, Story 20-2).
 */

export type {
	EmailTemplateLanguage,
	EmailTemplateResponse,
	UpdateEmailTemplatePayload,
} from './email-templates.types';
export { EMAIL_TEMPLATE_LANGUAGES } from './email-templates.types';
export {
	listEmailTemplates,
	getEmailTemplate,
	updateEmailTemplate,
	restoreEmailTemplateDefault,
} from './email-templates.api';
