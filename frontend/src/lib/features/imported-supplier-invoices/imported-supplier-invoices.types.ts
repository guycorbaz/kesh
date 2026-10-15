/**
 * Types DTO de l'import de factures fournisseurs depuis un dossier — Story 12.5d (#194).
 * Miroir camelCase des réponses backend (kesh-api routes/imported_supplier_invoices.rs
 * + inbox_import.rs). Décimaux sérialisés en `string` (rust_decimal).
 */

import type { SupplierInvoiceResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';

export type ImportedStatus = 'to_complete' | 'completed' | 'discarded';

/**
 * Document à traiter en staging (coordonnées QR parsées + lien du document).
 *
 * Story 30-2a : un document **sans** QR (ou au QR inexploitable) entre aussi ; ses
 * champs QR sont alors tous `null`. Les sept champs `creditorIban`, `isQrIban`,
 * `creditorAddressType`, `creditorName`, `creditorCountry`, `referenceType` et
 * `currency` sont `null` ensemble ou renseignés ensemble : « le document porte un
 * QR » se lit `creditorIban !== null` (cf. `hasQr`).
 */
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
	creditorIban: string | null;
	isQrIban: boolean | null;
	/** 'K' (combinée) ou 'S' (structurée). */
	creditorAddressType: string | null;
	creditorName: string | null;
	creditorLine1: string | null;
	creditorLine2: string | null;
	creditorPostalCode: string | null;
	creditorTown: string | null;
	creditorCountry: string | null;
	/** 'QRR' | 'SCOR' | 'NON'. */
	referenceType: string | null;
	referenceValue: string | null;
	/** Montant TTC du QR (`null` = montant ouvert, ou document sans QR). */
	amount: string | null;
	currency: string | null;
	unstructuredMessage: string | null;
	billingInformation: string | null;
	version: number;
	createdAt: string;
	updatedAt: string;
}

/** Le document porte-t-il des données QR ? (Story 30-2a.) */
export function hasQr(row: ImportedSupplierInvoice): boolean {
	return row.creditorIban !== null;
}

/**
 * Ce que le document apporte de son QR, au rapport d'import (Story 30-2a) :
 * `present` (données stockées), `none` (aucun QR), `unreadable` (QR trouvé mais
 * inexploitable — code en `qrError`, aucune donnée stockée).
 */
export type QrStatus = 'present' | 'none' | 'unreadable';

/** Fichier importé avec succès dans le rapport batch. */
export interface AcceptedFile {
	importedSupplierInvoiceId: number;
	fileName: string;
	qr: QrStatus;
	/** Présent seulement quand `qr` vaut `unreadable`. */
	qrError?: string;
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

/**
 * Corps de `POST /imported-supplier-invoices/{id}/complete` : depuis la Story 30-2b
 * (AC13), **celui de** `POST /supplier-invoices` — `CreateSupplierInvoiceRequest`. Avec
 * QR, il ne porte aucune coordonnée de paiement (`QR_FIELDS_FROM_DOCUMENT`).
 * La complétion retourne la facture fournisseur réelle créée, relue comme par le GET.
 */
export type CompleteImportResponse = SupplierInvoiceResponse;
