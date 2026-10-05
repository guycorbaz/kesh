/**
 * Le message d'un refus de téléchargement de PDF — facture ou avoir.
 *
 * Partagé par les deux fiches (`invoices/[id]`, `credit-notes/[id]`), qui
 * portaient chacune leur table : la même correction devait y être faite deux
 * fois (Story 25-6-b, revue P2).
 */
import { isApiError } from '$lib/shared/utils/api-client';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/**
 * Codes de refus → clé FTL. Explicite et non construite depuis `err.code`,
 * pour qu'un code inconnu du backend ne fabrique pas une clé inexistante — le
 * mismatch serait silencieux.
 */
const PDF_ERROR_KEYS: Record<string, string> = {
	INVOICE_NOT_VALIDATED: 'invoice-pdf-error-invoice-not-validated',
	INVOICE_NOT_PDF_READY: 'invoice-pdf-error-invoice-not-pdf-ready',
	// Story 16-3a (#151) — sans cette entrée, le message retombe sur le
	// générique et l'utilisateur ne sait pas QUOI raccourcir.
	INVOICE_PDF_HEADER_OVERFLOW: 'error-invoice-pdf-header-overflow',
	PDF_GENERATION_FAILED: 'invoice-pdf-error-pdf-generation-failed',
	NOT_FOUND: 'invoice-pdf-error-not-found',
	// Story 25-6-b (#387) — le PDF figé.
	INVOICE_CHANGED: 'error-invoice-changed',
	INVOICE_CANCELLED: 'error-invoice-cancelled',
};

/**
 * ⛔ Codes dont le message porte une **variable** (`{ $count }`, `{ $sha256 }`).
 * Le catalogue servi au frontend est résolu SANS arguments : la clé y rendrait
 * la variable brute (« La facture contient { $count } lignes »). Le message du
 * serveur, résolu avec ses arguments dans la même langue d'installation, est
 * donc repris tel quel.
 */
const SERVER_RESOLVED_CODES = new Set(['INVOICE_TOO_MANY_LINES_FOR_PDF', 'INVOICE_PDF_GONE']);

/** Le texte à afficher pour un échec de téléchargement de PDF. */
export function pdfErrorMessage(err: unknown): string {
	if (!isApiError(err)) {
		return i18nMsg('invoice-pdf-error-generic', 'Erreur lors du téléchargement du PDF');
	}
	if (SERVER_RESOLVED_CODES.has(err.code)) return err.message;
	const key = PDF_ERROR_KEYS[err.code] ?? 'invoice-pdf-error-generic';
	return i18nMsg(key, err.message);
}
