/**
 * Types de la feature `reminders` (envoi des rappels débiteurs — Story 21-6b, #231).
 *
 * Miroir camelCase des DTO backend (livrés par 21-5a/21-5b). Distinct de la
 * feature `dunning` (21-4), qui couvre la CONFIGURATION des niveaux ; ici c'est
 * l'ENVOI. Les montants décimaux sont des `string` (rust_decimal `serde-str`).
 */

// --- Liste des factures à rappeler (GET /api/v1/dunning/reminders) ---

/** Une facture éligible à un rappel, dans un groupe de contact. */
export interface ReminderCandidate {
	invoiceId: number;
	invoiceNumber: string | null;
	/** Échéance (YYYY-MM-DD). */
	dueDate: string;
	/** Niveau courant = MAX(level_number) des rappels non annulés (0 si aucun). */
	currentLevel: number;
	/** Prochain niveau à envoyer ; `null` si état terminal (dernier niveau atteint). */
	nextLevel: number | null;
	/** `true` = dernier niveau atteint, poursuite à envisager (jamais d'envoi auto). */
	terminal: boolean;
	/** Horodatage du dernier rappel non annulé (`null` si aucun). */
	lastReminderAt: string | null;
}

/** Les factures à rappeler d'un même débiteur. */
export interface ContactGroup {
	contactId: number;
	contactName: string;
	/** `false` = pas d'e-mail sur la fiche → envoi e-mail impossible (manuel OK). */
	hasEmail: boolean;
	invoices: ReminderCandidate[];
}

export interface ReminderListResponse {
	groups: ContactGroup[];
}

// --- Aperçu (GET /api/v1/invoices/{id}/reminder-preview?level=N) ---

/** Aperçu serveur d'un rappel : subject/body rendus, destinataire verrouillé. */
export interface ReminderPreviewResponse {
	/** Destinataire = contacts.email (verrouillé serveur) ; `null` si absent. */
	to: string | null;
	language: string;
	level: number;
	subject: string;
	body: string;
}

// --- Envoi unitaire (POST /api/v1/invoices/{id}/reminders/send) ---

/** Payload d'envoi unitaire. PAS de champ `to` (destinataire verrouillé serveur). */
export interface SendReminderRequest {
	levelNumber: number;
	subject: string;
	body: string;
}

/** Un rappel enregistré (réponse d'envoi unitaire / manuel, 201). */
export interface ReminderResponse {
	id: number;
	levelNumber: number;
	/** Frais snapshotés en CHF (décimale string). */
	feeAmount: string;
	sentAt: string;
	/** `"email"` | `"manual"`. */
	channel: string;
	/** Destinataire snapshoté ; `null` si canal manuel. */
	sentTo: string | null;
	subject: string;
	body: string;
	note: string | null;
	/** Annulation douce (Admin) ; `null` = actif. */
	cancelledAt: string | null;
}

// --- Envoi par lot (POST /api/v1/dunning/reminders/send-batch) ---

/** Payload de lot : uniquement des IDs (chaque template rendu serveur, prochain niveau). */
export interface SendReminderBatchRequest {
	invoiceIds: number[];
}

/** Un rappel envoyé avec succès dans un lot. */
export interface AcceptedReminder {
	invoiceId: number;
	reminderId: number;
	/** Niveau effectivement envoyé (= prochain niveau). */
	levelNumber: number;
}

/** Un échec per-facture d'un lot (pattern FailedProposal, HTTP 200 en succès partiel). */
export interface FailedReminder {
	invoiceId: number;
	/** Code d'erreur canonique (SCREAMING_SNAKE_CASE), à traduire côté UI. */
	errorCode: string;
	details: Record<string, unknown> | null;
}

export interface SendReminderBatchResponse {
	accepted: AcceptedReminder[];
	failed: FailedReminder[];
}

// --- Suspension des rappels (PUT /api/v1/invoices/{id}/dunning-pause|dunning-resume) ---

/**
 * Réponse de suspension/reprise (Story 21-6c).
 *
 * ⚠️ N'est PAS un `InvoiceResponse` complet — contrairement à `mark_as_paid`.
 * La suspension **incrémente `version`** (verrou optimiste). Après un appel,
 * l'UI DOIT ré-appliquer `{ version, dunningPausedAt, dunningPausedNote }` à son
 * état `invoice` local, sinon la prochaine action (mark-paid, unmark, validate,
 * delete) enverra une `version` périmée → 409 `OPTIMISTIC_LOCK_CONFLICT`.
 */
export interface DunningPauseResponse {
	invoiceId: number;
	dunningPausedAt: string | null;
	dunningPausedNote: string | null;
	version: number;
}

/** Payload de suspension : `note` optionnel (borne backend `PAUSE_NOTE_MAX` = 500). */
export interface PauseDunningRequest {
	version: number;
	note: string | null;
}

/** Payload de reprise : la reprise nulle serveur `dunningPausedAt` ET `dunningPausedNote`. */
export interface ResumeDunningRequest {
	version: number;
}

// --- Rappel manuel (POST /api/v1/invoices/{id}/reminders/manual) ---

/**
 * Payload d'un rappel papier (déjà envoyé hors Kesh).
 *
 * ⚠️ `sentAt` DOIT être un `NaiveDateTime` (format `YYYY-MM-DDTHH:MM:SS`), jamais
 * une date nue : le backend rejette `"YYYY-MM-DD"` (bug #249, cf. MarkPaidDialog).
 */
export interface ManualReminderRequest {
	levelNumber: number;
	sentAt: string;
	note: string | null;
}
