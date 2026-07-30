/**
 * Types DTO des avoirs (notes de crédit) — Story 12.1.
 * Miroir camelCase des réponses backend (kesh-api routes/credit_notes.rs).
 */

export type CreditNoteStatus = 'draft' | 'issued' | 'cancelled';

export interface CreditNoteLineResponse {
	position: number;
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
	lineTotal: string;
	/**
	 * Compte de produit extourné par cette ligne d'avoir (Story 16-1a D5).
	 *
	 * **Lecture seule** : recopié depuis la ligne de facture au moment de
	 * l'émission, jamais choisi par le client — `CreateCreditNoteRequest` ne le
	 * porte pas.
	 *
	 * `null` quand la ligne de facture d'origine l'était elle-même. Depuis le
	 * backfill de 16-1a-bis, ce cas se limite aux pièces dont l'écriture a été
	 * retouchée à la main. ⚠️ Ne PAS afficher « (défaut) » sur un avoir : aucun
	 * repli n'aura lieu, l'écriture est déjà passée — afficher un tiret.
	 */
	revenueAccountId: number | null;
}

export interface CreditNoteResponse {
	id: number;
	contactId: number;
	invoiceId: number;
	creditNoteNumber: string | null;
	status: CreditNoteStatus;
	date: string;
	totalAmount: string;
	journalEntryId: number | null;
	version: number;
	createdAt: string;
	lines: CreditNoteLineResponse[];
}

export interface CreditNoteListItemResponse {
	id: number;
	contactId: number;
	invoiceId: number;
	creditNoteNumber: string | null;
	status: CreditNoteStatus;
	date: string;
	totalAmount: string;
}

export interface CreateCreditNoteRequest {
	invoiceId: number;
	date: string;
}

export interface ListCreditNotesQuery {
	limit?: number;
	offset?: number;
}

export interface ListResponse<T> {
	items: T[];
	total: number;
	offset: number;
	limit: number;
}
