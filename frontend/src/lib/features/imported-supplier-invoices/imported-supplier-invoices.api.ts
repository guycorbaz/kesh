/**
 * Client API typé pour l'import de factures fournisseurs depuis un dossier — Story 12.5d (#194).
 *
 * Consomme les 6 endpoints livrés par 12-5c (aucune logique transactionnelle ici).
 * Les downloads utilisent le **Pattern A** (`apiClient.getBlob` → blob → ancre
 * éphémère) qui gère le 401-refresh (cookie httpOnly), contrairement à une ancre
 * `<a download>` directe.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import { parseContentDispositionFilename } from '$lib/features/export/exports.api';
import type {
	CompleteImportRequest,
	CompleteImportResponse,
	ImportedStatus,
	ImportedSupplierInvoice,
	InboxImportReport,
} from './imported-supplier-invoices.types';

/** Déclenche un run d'import du dossier inbox. HTTP 200 `{accepted, failed, warnings}` ; 409 si déjà en cours. */
export async function triggerInboxImport(): Promise<InboxImportReport> {
	return apiClient.post('/api/v1/inbox-import', {});
}

/** Liste les factures importées filtrées par statut (status obligatoire côté API). */
export async function listImported(status: ImportedStatus): Promise<ImportedSupplierInvoice[]> {
	return apiClient.get(`/api/v1/imported-supplier-invoices?status=${encodeURIComponent(status)}`);
}

/** Complète une facture importée (crée la facture fournisseur réelle, atomique DC6). */
export async function completeImport(
	id: number,
	req: CompleteImportRequest,
): Promise<CompleteImportResponse> {
	return apiClient.post(`/api/v1/imported-supplier-invoices/${id}/complete`, req);
}

/** Écarte une facture importée (to_complete → discarded). 204 No Content. */
export async function discardImport(id: number): Promise<void> {
	return apiClient.post(`/api/v1/imported-supplier-invoices/${id}/discard`, {});
}

/** Télécharge le justificatif d'une importée (avant complétion). Pattern A (gère 401-refresh). */
export async function downloadImportedSourceDocument(id: number): Promise<void> {
	const response = await apiClient.getBlob(
		`/api/v1/imported-supplier-invoices/${id}/source-document`,
	);
	await saveBlobResponse(response, `justificatif-import-${id}`);
}

/** Télécharge le justificatif d'une facture fournisseur complétée (après complétion). */
export async function downloadSupplierInvoiceSourceDocument(id: number): Promise<void> {
	const response = await apiClient.getBlob(`/api/v1/supplier-invoices/${id}/source-document`);
	await saveBlobResponse(response, `justificatif-${id}`);
}

/** Lit le blob + filename du header `Content-Disposition` et déclenche le download. */
async function saveBlobResponse(response: Response, fallbackBase: string): Promise<void> {
	const blob = await response.blob();
	const filename =
		parseContentDispositionFilename(response.headers.get('Content-Disposition')) ?? fallbackBase;
	triggerDownload(blob, filename);
}

/**
 * Déclenche le download navigateur via une ancre `<a download>` éphémère.
 * Duplication locale (~10 lignes) cohérente avec la décision projet (cf.
 * `exports.api.ts::triggerDownload`, non exportée) — try/finally pour le cleanup
 * même si `a.click()` jette.
 */
function triggerDownload(blob: Blob, filename: string): void {
	const objectUrl = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = objectUrl;
	a.download = filename;
	try {
		document.body.appendChild(a);
		a.click();
	} finally {
		if (a.parentNode) a.parentNode.removeChild(a);
		URL.revokeObjectURL(objectUrl);
	}
}
