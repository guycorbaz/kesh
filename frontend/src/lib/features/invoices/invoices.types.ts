/**
 * Types TS miroir des DTOs API pour les factures brouillon (Story 5.1).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/invoices.rs`
 * (serde `rename_all = "camelCase"`). Montants (`quantity`, `unitPrice`,
 * `vatRate`, `lineTotal`, `totalAmount`) transportés en **string décimale**
 * via `rust_decimal::serde-str`. Ne JAMAIS convertir en `number`.
 */

export type InvoiceSortBy = 'Date' | 'TotalAmount' | 'ContactName' | 'CreatedAt';
export type SortDirection = 'Asc' | 'Desc';
export type InvoiceStatus = 'draft' | 'validated' | 'cancelled';

export interface InvoiceLineResponse {
	id: number;
	invoiceId: number;
	position: number;
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
	lineTotal: string;
	createdAt: string;
}

export interface InvoiceResponse {
	id: number;
	companyId: number;
	contactId: number;
	invoiceNumber: string | null;
	status: InvoiceStatus;
	date: string;
	dueDate: string | null;
	paymentTerms: string | null;
	totalAmount: string;
	journalEntryId: number | null;
	version: number;
	createdAt: string;
	updatedAt: string;
	lines: InvoiceLineResponse[];
}

// Story 5.2 — Configuration facturation
export type JournalCode = 'Achats' | 'Ventes' | 'Banque' | 'Caisse' | 'OD';

export interface InvoiceSettingsResponse {
	companyId: number;
	invoiceNumberFormat: string;
	defaultReceivableAccountId: number | null;
	defaultRevenueAccountId: number | null;
	defaultSalesJournal: JournalCode;
	journalEntryDescriptionTemplate: string;
	version: number;
}

export interface UpdateInvoiceSettingsRequest {
	invoiceNumberFormat: string;
	defaultReceivableAccountId: number | null;
	defaultRevenueAccountId: number | null;
	defaultSalesJournal: JournalCode;
	journalEntryDescriptionTemplate: string;
	version: number;
}

export interface InvoiceListItemResponse {
	id: number;
	companyId: number;
	contactId: number;
	contactName: string;
	invoiceNumber: string | null;
	status: InvoiceStatus;
	date: string;
	dueDate: string | null;
	paymentTerms: string | null;
	totalAmount: string;
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateInvoiceLineRequest {
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
}

export interface CreateInvoiceRequest {
	contactId: number;
	date: string;
	dueDate?: string | null;
	paymentTerms?: string | null;
	lines: CreateInvoiceLineRequest[];
}

export interface UpdateInvoiceRequest extends CreateInvoiceRequest {
	version: number;
}

export interface ListInvoicesQuery {
	search?: string;
	status?: InvoiceStatus;
	contactId?: number;
	dateFrom?: string;
	dateTo?: string;
	sortBy?: InvoiceSortBy;
	sortDirection?: SortDirection;
	limit?: number;
	offset?: number;
}

export interface ListResponse<T> {
	items: T[];
	total: number;
	limit: number;
	offset: number;
}
