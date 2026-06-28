/**
 * Types DTO des factures fournisseurs & règlement binaire — Story 12.2 (#191).
 * Miroir camelCase des réponses backend (kesh-api routes/supplier_invoices.rs).
 */

export type SupplierInvoiceStatus = 'open' | 'paid' | 'cancelled';
export type SettlementType = 'bank_transfer' | 'internal_account';

export interface SupplierInvoiceLineResponse {
	position: number;
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
	lineTotal: string;
	expenseAccountId: number;
}

export interface SupplierInvoiceResponse {
	id: number;
	contactId: number;
	supplierInvoiceNumber: string | null;
	status: SupplierInvoiceStatus;
	invoiceDate: string;
	dueDate: string | null;
	totalAmount: string;
	creditorIban: string | null;
	creditorQrIban: string | null;
	paymentReference: string | null;
	expectedPaymentAmount: string | null;
	purchaseJournalEntryId: number;
	settlementType: SettlementType | null;
	settlementBankAccountId: number | null;
	settlementAccountId: number | null;
	settlementJournalEntryId: number | null;
	paidAt: string | null;
	version: number;
	createdAt: string;
	lines: SupplierInvoiceLineResponse[];
}

export interface SupplierInvoiceListItemResponse {
	id: number;
	contactId: number;
	contactName: string;
	supplierInvoiceNumber: string | null;
	status: SupplierInvoiceStatus;
	invoiceDate: string;
	dueDate: string | null;
	totalAmount: string;
}

export interface CreateSupplierInvoiceLineRequest {
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
	expenseAccountId: number;
}

export interface CreateSupplierInvoiceRequest {
	contactId: number;
	supplierInvoiceNumber?: string | null;
	invoiceDate: string;
	dueDate?: string | null;
	creditorIban?: string | null;
	creditorQrIban?: string | null;
	paymentReference?: string | null;
	expectedPaymentAmount?: string | null;
	lines: CreateSupplierInvoiceLineRequest[];
}

/** Règlement binaire : virement (bankAccountId) ou compte interne (accountId). */
export interface PaySupplierInvoiceRequest {
	settlementType: SettlementType;
	bankAccountId?: number;
	accountId?: number;
	paymentDate: string;
}

export interface ListSupplierInvoicesQuery {
	limit?: number;
	offset?: number;
}

export interface ListResponse<T> {
	items: T[];
	total: number;
	offset: number;
	limit: number;
}
