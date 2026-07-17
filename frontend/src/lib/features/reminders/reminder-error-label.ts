/**
 * Mapping `errorCode` (FailedReminder d'un lot) → libellé i18n (Story 21-6b).
 *
 * Testable unitairement (AC 28). Les codes proviennent de `send_one_batch_reminder`
 * (backend 21-5b, `invoice_email.rs`).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/**
 * Codes signifiant que l'e-mail EST (peut-être) parti chez le débiteur.
 * L'UI ne doit JAMAIS proposer « Réessayer » sur ces codes (leçon 21-5b) : un
 * ré-essai renverrait un vrai second e-mail.
 */
export const EMAIL_SENT_CODES = new Set<string>([
	'REMINDER_SENT_BUT_INVOICE_GONE',
	'RECORD_FAILED_EMAIL_SENT',
	'SMTP_SEND_FAILED',
]);

/** `true` si le code signifie que l'e-mail est parti (pas de ré-essai). */
export function isEmailSent(code: string): boolean {
	return EMAIL_SENT_CODES.has(code);
}

const LABELS: Record<string, [string, string]> = {
	INVOICE_NOT_FOUND: ['invoice-not-found', 'Facture introuvable'],
	INVOICE_NOT_VALIDATED: ['invoice-not-validated', 'Facture non validée'],
	INVOICE_ALREADY_PAID: ['invoice-already-paid', 'Facture déjà payée'],
	DUNNING_PAUSED: ['dunning-paused', 'Rappels suspendus'],
	NO_NEXT_LEVEL: ['no-next-level', 'Dernier niveau atteint'],
	CONTACT_ARCHIVED: ['contact-archived', 'Contact archivé'],
	CONTACT_EMAIL_MISSING: ['contact-email-missing', "Contact sans adresse e-mail"],
	REMINDER_CONTENT_EMPTY: ['content-empty', 'Modèle de rappel vide'],
	REMINDER_CONTENT_TOO_LONG: ['content-too-long', 'Contenu du rappel trop long'],
	INVOICE_NOT_PDF_READY: ['not-pdf-ready', 'Facture non imprimable en PDF'],
	RATE_LIMITED: ['rate-limited', "Limite d'envoi atteinte"],
	DATABASE_ERROR: ['database-error', 'Erreur technique'],
	// Codes « e-mail parti » : le message dit explicitement qu'il a été envoyé.
	SMTP_SEND_FAILED: ['smtp-failed', "Échec de l'envoi e-mail"],
	REMINDER_SENT_BUT_INVOICE_GONE: [
		'sent-but-gone',
		'E-mail envoyé, mais la facture a disparu entre-temps (non enregistré)',
	],
	RECORD_FAILED_EMAIL_SENT: [
		'sent-not-recorded',
		'E-mail envoyé, mais non enregistré (erreur technique)',
	],
};

/** Libellé traduit d'un `errorCode` de rappel ; fallback avec le code brut. */
export function reminderErrorLabel(code: string): string {
	const entry = LABELS[code];
	if (entry) return i18nMsg(`reminders-error-${entry[0]}`, entry[1]);
	return i18nMsg('reminders-error-unknown', 'Échec ({ $code })', { code });
}
