/**
 * Mapping `errorCode` (FailedReminder d'un lot) → libellé i18n (Story 21-6b).
 *
 * Testable unitairement (AC 28). Les codes proviennent de `send_one_batch_reminder`
 * (backend 21-5b, `invoice_email.rs`).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { FailedReminder } from './reminders.types';

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
	// Story 25-4-b2 (#416) : reste dû nul — rien à réclamer.
	REMINDER_NOTHING_DUE: ['nothing-due', 'Rien à réclamer'],
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

/**
 * Texte d'un échec du lot de rappels (Story 35-2b, #955) : le libellé du code, suivi,
 * pour un PDF refusé (`INVOICE_NOT_PDF_READY`), de la phrase du serveur qui dit quoi
 * corriger (`details.message`), comme sur la fiche de la facture.
 *
 * Le message n'est lu que pour ce code — le `details.message` d'autres lots est un
 * texte technique — et seulement s'il est une chaîne non vide. Le résultat est du
 * texte : le composant l'affiche par `{…}`, jamais `{@html}`.
 *
 * Limite assumée (fiche, D2.4) : le séparateur « : » est écrit en dur dans les quatre
 * langues, et la phrase est dans la langue du serveur, le libellé dans celle de
 * l'interface.
 */
export function reminderFailureText(
	failed: Pick<FailedReminder, 'errorCode' | 'details'>,
): string {
	const libelle = reminderErrorLabel(failed.errorCode);
	if (failed.errorCode !== 'INVOICE_NOT_PDF_READY') return libelle;
	const message = failed.details?.message;
	if (typeof message !== 'string' || message.trim() === '') return libelle;
	return `${libelle} : ${message.trim()}`;
}
