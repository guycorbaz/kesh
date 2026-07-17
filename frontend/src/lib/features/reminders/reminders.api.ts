/**
 * Client API pour l'envoi des rappels débiteurs (Story 21-6b, #231).
 *
 * Endpoints livrés par 21-5a/21-5b, tous **Comptable+** :
 * - `GET  /api/v1/dunning/reminders` : factures à rappeler, groupées par contact.
 * - `GET  /api/v1/invoices/{id}/reminder-preview?level=N` : aperçu serveur.
 * - `POST /api/v1/invoices/{id}/reminders/send` : envoi unitaire (niveau ≤ prochain).
 * - `POST /api/v1/dunning/reminders/send-batch` : envoi lot (prochain niveau, cap 20).
 * - `POST /api/v1/invoices/{id}/reminders/manual` : rappel papier.
 *
 * Aucun payload d'envoi ne porte de champ `to` (destinataire = contacts.email
 * verrouillé serveur, anti-exfiltration).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	ReminderListResponse,
	ReminderPreviewResponse,
	SendReminderRequest,
	ReminderResponse,
	SendReminderBatchResponse,
	ManualReminderRequest,
} from './reminders.types';

export type {
	ReminderCandidate,
	ContactGroup,
	ReminderListResponse,
	ReminderPreviewResponse,
	SendReminderRequest,
	ReminderResponse,
	AcceptedReminder,
	FailedReminder,
	SendReminderBatchResponse,
	ManualReminderRequest,
} from './reminders.types';

/** Factures à rappeler, groupées par débiteur. Liste vide si dunning désactivé. */
export async function listReminders(): Promise<ReminderListResponse> {
	return apiClient.get<ReminderListResponse>('/api/v1/dunning/reminders');
}

/** Aperçu serveur d'un rappel de niveau `level`. `level` toujours fourni (backend: absent → 400). */
export async function getReminderPreview(
	invoiceId: number,
	level: number,
): Promise<ReminderPreviewResponse> {
	return apiClient.get<ReminderPreviewResponse>(
		`/api/v1/invoices/${invoiceId}/reminder-preview?level=${level}`,
	);
}

/** Envoi unitaire d'un rappel par e-mail (subject/body édités, 201). */
export async function sendReminder(
	invoiceId: number,
	payload: SendReminderRequest,
): Promise<ReminderResponse> {
	return apiClient.post<ReminderResponse>(`/api/v1/invoices/${invoiceId}/reminders/send`, payload);
}

/** Envoi par lot (prochain niveau de chaque facture). HTTP 200 en succès partiel. */
export async function sendReminderBatch(
	invoiceIds: number[],
): Promise<SendReminderBatchResponse> {
	return apiClient.post<SendReminderBatchResponse>('/api/v1/dunning/reminders/send-batch', {
		invoiceIds,
	});
}

/** Enregistre un rappel papier (déjà envoyé hors Kesh). `sentAt` = NaiveDateTime (bug #249). */
export async function recordManualReminder(
	invoiceId: number,
	payload: ManualReminderRequest,
): Promise<ReminderResponse> {
	return apiClient.post<ReminderResponse>(
		`/api/v1/invoices/${invoiceId}/reminders/manual`,
		payload,
	);
}
