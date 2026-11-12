/**
 * Types DTO de l'import de factures fournisseurs depuis un dossier — Story 12.5d (#194).
 * Miroir camelCase des réponses backend (kesh-api routes/imported_supplier_invoices.rs
 * + inbox_import.rs). Décimaux sérialisés en `string` (rust_decimal).
 */

import type { SupplierInvoiceResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';
import { formatSwissDate } from '$lib/features/reports/reports.api';

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
 * Date d'import (`createdAt`, horodatage naïf UTC) au format suisse — partagée par la
 * liste « Documents à traiter » et le sélecteur de la fiche (Story 30-2c, AC20).
 * Depuis la 0.17.4 (#889), la date suisse de l'instant (horodatage entier) — non plus
 * celle d'UTC, l'angle mort assumé de la revue P1 de la 30-2a.
 */
export function importDateOf(row: ImportedSupplierInvoice): string {
	return formatSwissDate(row.createdAt);
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

/**
 * Échec per-fichier (identifiant business = `fileName`, `errorCode` constante).
 *
 * Story 34-9 (#969) — `details.movedTo` (tout échec) : le nom sous lequel le fichier a été
 * rangé dans `failed/` ; un `DUPLICATE` porte en plus `existingDocumentId`,
 * `existingStatus` et `supplierInvoiceId` (lus par {@link duplicateOf}).
 */
export interface FailedFile {
	fileName: string;
	errorCode: string;
	details?: Record<string, unknown> | null;
}

/** État d'un document importé (`imported_supplier_invoices.status`). */
export type ImportedDocumentStatus = 'to_complete' | 'completed' | 'discarded';

/** Story 34-9 (AC-1) — le document déjà importé qu'un doublon désigne. */
export interface ExistingDocument {
	existingDocumentId: number;
	existingStatus: ImportedDocumentStatus;
	supplierInvoiceId: number | null;
}

/** Story 34-9 (#974) — pourquoi un fichier n'a pas été traité. */
export type SkipReason = 'UNSTABLE' | 'VANISHED';

/** Story 34-9 (#974) — un fichier ni importé ni déplacé. */
export interface SkippedFile {
	fileName: string;
	reason: SkipReason;
}

/** Rapport d'un run d'import (`POST /inbox-import`, HTTP 200). */
export interface InboxImportReport {
	accepted: AcceptedFile[];
	failed: FailedFile[];
	/** Story 34-9 (#974) — fichiers non traités. */
	skipped: SkippedFile[];
	/** Story 34-9 (#974) — la limite de fichiers par import, si elle a été atteinte. */
	truncatedAt: number | null;
	/** @deprecated Story 34-9 (#974) : messages du serveur en français ; l'écran lit `skipped` et `truncatedAt`. */
	warnings: string[];
}

/** Story 34-9 (AC-2) — le nom rangé dans `failed/`, ou `null` s'il manque. */
export function movedToOf(f: FailedFile): string | null {
	const v = f.details?.movedTo;
	return typeof v === 'string' && v !== '' ? v : null;
}

const STATUTS_DOCUMENT: readonly string[] = ['to_complete', 'completed', 'discarded'];

/**
 * Story 34-9 (AC-1) — le document existant d'un doublon, ou `null` si l'entrée n'en porte
 * pas (document introuvable à la relecture, ou serveur antérieur à la 0.17.6).
 */
export function duplicateOf(f: FailedFile): ExistingDocument | null {
	const d = f.details;
	if (f.errorCode !== 'DUPLICATE' || !d) return null;
	const id = d.existingDocumentId;
	const status = d.existingStatus;
	const invoice = d.supplierInvoiceId;
	if (typeof id !== 'number' || typeof status !== 'string' || !STATUTS_DOCUMENT.includes(status)) {
		return null;
	}
	return {
		existingDocumentId: id,
		existingStatus: status as ImportedDocumentStatus,
		supplierInvoiceId: typeof invoice === 'number' ? invoice : null,
	};
}

/** Story 34-9 (AC-4) — état d'une ligne du rapport. */
export type ReportRowState = 'imported' | 'imported-no-qr' | 'duplicate' | 'refused' | 'not-processed';

/** Story 34-9 (AC-4) — une ligne du rapport : un fichier et ce qui lui est arrivé. */
export type ReportRow =
	| { state: 'imported' | 'imported-no-qr'; fileName: string; accepted: AcceptedFile }
	| { state: 'duplicate' | 'refused'; fileName: string; failed: FailedFile }
	| { state: 'not-processed'; fileName: string; skipped: SkippedFile };

/**
 * Story 34-9 (AC-4, C-e342-12) — une ligne par fichier, `accepted`, `failed` et `skipped`
 * fusionnés, dans l'ordre du nom de fichier (`read_dir` n'en garantit aucun).
 */
export function reportRows(report: InboxImportReport): ReportRow[] {
	const rows: ReportRow[] = [
		...report.accepted.map(
			(a): ReportRow => ({
				state: a.qr === 'present' ? 'imported' : 'imported-no-qr',
				fileName: a.fileName,
				accepted: a,
			}),
		),
		...report.failed.map(
			(f): ReportRow => ({
				state: f.errorCode === 'DUPLICATE' ? 'duplicate' : 'refused',
				fileName: f.fileName,
				failed: f,
			}),
		),
		...(report.skipped ?? []).map(
			(s): ReportRow => ({ state: 'not-processed', fileName: s.fileName, skipped: s }),
		),
	];
	return rows.sort(
		(a, b) =>
			a.fileName.localeCompare(b.fileName, 'fr', { numeric: true, sensitivity: 'base' }) ||
			(a.fileName < b.fileName ? -1 : a.fileName > b.fileName ? 1 : 0),
	);
}

/** Story 34-9 (AC-4) — les quatre segments du résumé, comptés sur les lignes. */
export function reportCounts(report: InboxImportReport): {
	imported: number;
	duplicates: number;
	refused: number;
	notProcessed: number;
} {
	const duplicates = report.failed.filter((f) => f.errorCode === 'DUPLICATE').length;
	return {
		imported: report.accepted.length,
		duplicates,
		refused: report.failed.length - duplicates,
		notProcessed: (report.skipped ?? []).length,
	};
}

/**
 * Corps de `POST /imported-supplier-invoices/{id}/complete` : depuis la Story 30-2b
 * (AC13), **celui de** `POST /supplier-invoices` — `CreateSupplierInvoiceRequest`. Avec
 * QR, il ne porte aucune coordonnée de paiement (`QR_FIELDS_FROM_DOCUMENT`).
 * La complétion retourne la facture fournisseur réelle créée, mise en forme comme par le
 * GET (lue avant le commit depuis la Story 34-8).
 */
export type CompleteImportResponse = SupplierInvoiceResponse;
