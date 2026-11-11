/**
 * Types DTO des factures fournisseurs & règlement binaire — Story 12.2 (#191).
 * Miroir camelCase des réponses backend (kesh-api routes/supplier_invoices.rs).
 */

import type { SupplierSettlementCancelCode } from './settlement-cancel';
import type { SupplierInvoiceCancelCode } from './invoice-cancel';

/**
 * Statut d'une facture fournisseur. Story v017-4b (#849, AC-21) : `draft` — un
 * brouillon, sans écriture ; `open` est la valeur **stockée** de l'état « validée,
 * à payer », qui s'affiche « Validée » (décision Q4, C-v017-4-24).
 */
export type SupplierInvoiceStatus = 'draft' | 'open' | 'paid' | 'cancelled';
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
	/** Le numéro interne (Story 33-1a, #926) : `null` pour un brouillon jamais validé. */
	internalNumber: string | null;
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
	/** L'écriture d'achat ; `null` pour un brouillon (Story v017-4a, AC-4). */
	purchaseJournalEntryId: number | null;
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
	 * ⚠️ **`null` veut dire « non calculé », jamais « non »** : seuls le GET et les
	 * neuf gestes qui écrivent le calculent (la création, non). Calculé par la fonction
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
	/**
	 * Story 30-2c (AC18) — le document de la facture, à **trois états** :
	 * - champ **absent** (`undefined`) : non calculé (réponse de la création) ;
	 * - `null` : la facture n'a pas de document ;
	 * - l'objet : son nom, son type et sa taille.
	 *
	 * ⚠️ `undefined` ne veut jamais dire « sans document » : la fiche ne propose
	 * « Joindre un document » que sur `null`.
	 */
	sourceDocument?: SupplierInvoiceSourceDocument | null;
}

/** Le document d'une facture fournisseur (Story 30-2c, AC18). */
export interface SupplierInvoiceSourceDocument {
	/** Story v017-4a (contrat, F5) : l'identifiant du document importé. */
	importedSupplierInvoiceId: number;
	originalFilename: string;
	mimeType: string;
	byteSize: number;
	/**
	 * D'où vient le lien (Story v017-4a, C-v017-4-29) : `completion` — la facture est
	 * née de la complétion du document ; `attachment` — document joint après coup ;
	 * `null` — lien antérieur à la 0.17.0.
	 */
	linkOrigin: 'completion' | 'attachment' | null;
	/** Les données du QR du document, ou `null` pour un document sans QR. */
	qr: SupplierInvoiceSourceQr | null;
}

/**
 * Les données du QR d'un document, telles que le serveur les expose dans
 * `sourceDocument.qr` (Story v017-4a) — jamais le chemin de stockage ni l'empreinte.
 * Montant en `string` (parité `serde-str`).
 */
export interface SupplierInvoiceSourceQr {
	creditorIban: string | null;
	isQrIban: boolean | null;
	referenceType: string | null;
	referenceValue: string | null;
	amount: string | null;
	currency: string | null;
	creditorName: string | null;
	creditorAddressType: string | null;
	creditorLine1: string | null;
	creditorLine2: string | null;
	creditorPostalCode: string | null;
	creditorTown: string | null;
	creditorCountry: string | null;
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
	internalNumber: string | null;
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
 * Story v017-4b (#849, AC-23) — le corps du `PUT` d'un brouillon : celui de la
 * création, plus la `version` lue (verrou optimiste).
 */
export interface UpdateSupplierInvoiceRequest extends CreateSupplierInvoiceRequest {
	version: number;
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
	/**
	 * Story v017-4b (AC-15, AC-21, AC-27) — filtre serveur sur un ou plusieurs statuts
	 * (`?status=draft,open,paid`) ; absent ou vide : tous.
	 */
	status?: SupplierInvoiceStatus[];
	/** Story 33-1b (#926) — recherche : numéro interne, numéro et nom du fournisseur. */
	q?: string;
	/** Story 33-1b (#926) — tri ; absent : la date, de la plus récente à la plus ancienne. */
	sort?: 'internalNumber';
	order?: 'asc' | 'desc';
}

export interface ListResponse<T> {
	items: T[];
	total: number;
	offset: number;
	limit: number;
}
