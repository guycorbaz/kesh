/**
 * Types TS miroir des DTOs API pour les factures brouillon (Story 5.1).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/invoices.rs`
 * (serde `rename_all = "camelCase"`). Montants (`quantity`, `unitPrice`,
 * `vatRate`, `lineTotal`, `totalAmount`) transportés en **string décimale**
 * via `rust_decimal::serde-str`. Ne JAMAIS convertir en `number`.
 */

export type InvoiceSortBy = 'Date' | 'DueDate' | 'TotalAmount' | 'ContactName' | 'CreatedAt';
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
	paidAt: string | null;
	/** Calculé backend (P6 review pass 2). Source unique de vérité pour le badge « en retard ». */
	isOverdue: boolean;
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
	paidAt: string | null;
	version: number;
	createdAt: string;
	updatedAt: string;
}

// Story 5.4 — Échéancier
export type PaymentStatusFilter = 'all' | 'paid' | 'unpaid' | 'overdue';

export interface DueDateItem extends InvoiceListItemResponse {
	isOverdue: boolean;
}

export interface DueDatesSummary {
	unpaidCount: number;
	unpaidTotal: string;
	overdueCount: number;
	overdueTotal: string;
}

export interface DueDatesResponse {
	items: DueDateItem[];
	total: number;
	offset: number;
	limit: number;
	summary: DueDatesSummary;
}

export interface DueDatesQuery {
	search?: string;
	contactId?: number;
	dateFrom?: string;
	dateTo?: string;
	dueBefore?: string;
	paymentStatus?: PaymentStatusFilter;
	sortBy?: InvoiceSortBy;
	sortDirection?: SortDirection;
	limit?: number;
	offset?: number;
}

export interface MarkPaidRequest {
	paidAt?: string;
	version: number;
}

export interface UnmarkPaidRequest {
	version: number;
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
