/**
 * Types DTO des factures fournisseurs & règlement binaire — Story 12.2 (#191).
 * Miroir camelCase des réponses backend (kesh-api routes/supplier_invoices.rs).
 */

import type { SupplierSettlementCancelCode } from './settlement-cancel';
import type { SupplierInvoiceCancelCode } from './invoice-cancel';

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
	/** Projet analytique affecté (Epic 19, Story 19-3), ou `null`. */
	projectId: number | null;
	purchaseJournalEntryId: number;
	settlementType: SettlementType | null;
	settlementBankAccountId: number | null;
	settlementAccountId: number | null;
	settlementJournalEntryId: number | null;
	paidAt: string | null;
	version: number;
	createdAt: string;
	lines: SupplierInvoiceLineResponse[];
	/**
	 * Story 25-3-a-2 (#414) — le règlement peut-il être annulé ?
	 *
	 * ⚠️ **`null` veut dire « non calculé », jamais « non »** : seuls le GET,
	 * `pay` et l'annulation du règlement le calculent. Calculé par la fonction
	 * même qui refuse l'annulation ; ne tient pas compte du rôle.
	 */
	settlementCancellable: boolean | null;
	settlementCancelBlockedBy: SupplierSettlementCancelCode | null;
	/** Le numéro du compte archivé, quand c'est le motif. */
	settlementCancelBlockedLabel: string | null;
	/**
	 * Le lot de paiement **confirmé le plus récent** qui contient la facture.
	 * ⛔ **Historique** : il ne dit PAS d'où vient le règlement courant — il sert
	 * à prévenir un double paiement avant d'annuler.
	 */
	lastConfirmedBatch: { id: number; confirmedAt: string | null } | null;
	/**
	 * Story 25-3-c (#454) — la **facture** peut-elle être annulée ?
	 *
	 * ⚠️ Même discipline que `settlementCancellable` : `null` = non calculé,
	 * jamais « non ». Calculé par la fonction même qui refuse ; ne tient pas
	 * compte du rôle.
	 */
	cancellable: boolean | null;
	cancelBlockedBy: SupplierInvoiceCancelCode | null;
	/** Le numéro du compte archivé, quand c'est le motif. */
	cancelBlockedLabel: string | null;
}

export interface CancelSupplierSettlementResponse {
	invoice: SupplierInvoiceResponse;
	reversalJournalEntryId: number;
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
	/** Projet analytique (Epic 19, Story 19-3) — document-level, optionnel. */
	projectId?: number | null;
	lines: CreateSupplierInvoiceLineRequest[];
}

/**
 * Coordonnées extraites d'une QR-facture (Story 12.4) — pré-remplissage seul.
 * Exactement l'un de `creditorIban` / `creditorQrIban` est renseigné.
 * Montant en `string` (parité `serde-str` backend / big.js frontend).
 */
export interface ScanQrResponse {
	creditorIban: string | null;
	creditorQrIban: string | null;
	paymentReference: string | null;
	expectedPaymentAmount: string | null;
	currency: string;
	creditorName: string;
	creditorAddress: string | null;
	unstructuredMessage: string | null;
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
