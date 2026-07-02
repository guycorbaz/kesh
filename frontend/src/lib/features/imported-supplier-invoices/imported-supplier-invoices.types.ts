/**
 * Types DTO de l'import de factures fournisseurs depuis un dossier — Story 12.5d (#194).
 * Miroir camelCase des réponses backend (kesh-api routes/imported_supplier_invoices.rs
 * + inbox_import.rs). Décimaux sérialisés en `string` (rust_decimal).
 */

import type { SupplierInvoiceResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';

export type ImportedStatus = 'to_complete' | 'completed' | 'discarded';

/** Facture importée en staging (coordonnées QR parsées + lien justificatif). */
export interface ImportedSupplierInvoice {
	id: number;
	companyId: number;
	status: ImportedStatus;
	supplierInvoiceId: number | null;
	fileHash: string;
	storagePath: string;
	originalFilename: string;
	mimeType: string;
	byteSize: number;
	creditorIban: string;
	isQrIban: boolean;
	/** 'K' (combinée) ou 'S' (structurée). */
	creditorAddressType: string;
	creditorName: string;
	creditorLine1: string | null;
	creditorLine2: string | null;
	creditorPostalCode: string | null;
	creditorTown: string | null;
	creditorCountry: string;
	/** 'QRR' | 'SCOR' | 'NON'. */
	referenceType: string;
	referenceValue: string | null;
	/** Montant TTC du QR (`null` = montant ouvert). */
	amount: string | null;
	currency: string;
	unstructuredMessage: string | null;
	billingInformation: string | null;
	version: number;
	createdAt: string;
	updatedAt: string;
}

/** Fichier importé avec succès dans le rapport batch. */
export interface AcceptedFile {
	importedSupplierInvoiceId: number;
	fileName: string;
}

/** Échec per-fichier (identifiant business = `fileName`, `errorCode` constante). */
export interface FailedFile {
	fileName: string;
	errorCode: string;
	details?: Record<string, unknown> | null;
}

/** Rapport d'un run d'import (`POST /inbox-import`, HTTP 200). */
export interface InboxImportReport {
	accepted: AcceptedFile[];
	failed: FailedFile[];
	warnings: string[];
}

/** Ligne du formulaire de complétion (décimaux en string, calcul big.js). */
export interface CompleteImportLineRequest {
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
	expenseAccountId: number;
}

/** Corps de `POST /imported-supplier-invoices/{id}/complete`. */
export interface CompleteImportRequest {
	contactId: number;
	invoiceDate: string;
	supplierInvoiceNumber?: string | null;
	dueDate?: string | null;
	lines: CompleteImportLineRequest[];
}

/** La complétion retourne la facture fournisseur réelle créée (12-2). */
export type CompleteImportResponse = SupplierInvoiceResponse;
