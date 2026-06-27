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
