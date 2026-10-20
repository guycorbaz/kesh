/**
 * Client API typé pour l'import de factures fournisseurs depuis un dossier — Story 12.5d (#194).
 *
 * Consomme les 6 endpoints livrés par 12-5c, plus `supplier-candidates` (Story 30-2b)
 * et le rattachement / détachement d'un document (Story 30-2c) — aucune logique
 * transactionnelle ici.
 * Les downloads utilisent le **Pattern A** (`apiClient.getBlob` → blob → ancre
 * éphémère) qui gère le 401-refresh (cookie httpOnly), contrairement à une ancre
 * `<a download>` directe.
 */

import { apiClient } from '$lib/shared/utils/api-client';
import { parseContentDispositionFilename } from '$lib/features/export/exports.api';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';
import type {
	CreateSupplierInvoiceRequest,
	SupplierInvoiceResponse,
} from '$lib/features/supplier-invoices/supplier-invoices.types';
import type {
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

/**
 * Crée la facture d'un document à traiter (atomique DC6). Story 30-2b (AC13) : le corps
 * est celui de `POST /supplier-invoices` ; avec QR, sans coordonnées de paiement.
 */
export async function completeImport(
	id: number,
	req: CreateSupplierInvoiceRequest,
): Promise<CompleteImportResponse> {
	return apiClient.post(`/api/v1/imported-supplier-invoices/${id}/complete`, req);
}

/**
 * Fournisseurs proposés par l'IBAN du document (Story 30-2b, AC14) : ceux dont une
 * facture passée porte cet IBAN, trois au plus. Liste vide pour un document sans QR.
 * L'IBAN est lu par le serveur sur le document — il ne passe pas dans l'URL.
 */
export async function getSupplierCandidates(id: number): Promise<{ candidates: ContactResponse[] }> {
	return apiClient.get(`/api/v1/imported-supplier-invoices/${id}/supplier-candidates`);
}

/** Écarte une facture importée (to_complete → discarded). 204 No Content. */
export async function discardImport(id: number): Promise<void> {
	return apiClient.post(`/api/v1/imported-supplier-invoices/${id}/discard`, {});
}

/**
 * Lit le document d'une importée en `Blob` (Story 30-2a, aperçu). Même chemin que le
 * téléchargement — `apiClient.getBlob`, qui gère le 401-refresh —, sans l'enregistrer :
 * l'aperçu en fait une URL `blob:` (C-30-2-6). Le type du blob vient du `Content-Type`
 * de la réponse, lui-même dérivé de l'extension à l'import, jamais du contenu.
 */
export async function fetchImportedSourceDocument(id: number): Promise<Blob> {
	const response = await apiClient.getBlob(
		`/api/v1/imported-supplier-invoices/${id}/source-document`,
	);
	return response.blob();
}

/** Télécharge le justificatif d'une importée (avant complétion). Pattern A (gère 401-refresh). */
export async function downloadImportedSourceDocument(id: number): Promise<void> {
	const response = await apiClient.getBlob(
		`/api/v1/imported-supplier-invoices/${id}/source-document`,
	);
	await saveBlobResponse(response, `justificatif-import-${id}`);
}

/** Télécharge le document d'une facture fournisseur. */
export async function downloadSupplierInvoiceSourceDocument(id: number): Promise<void> {
	const response = await apiClient.getBlob(`/api/v1/supplier-invoices/${id}/source-document`);
	await saveBlobResponse(response, `justificatif-${id}`);
}

/**
 * Lit le document d'une facture fournisseur en `Blob`, pour l'aperçu de la fiche et
 * du détail de l'écriture (Story 30-2c, AC18, AC21). Lecture ouverte à tout rôle
 * authentifié (AC22).
 */
export async function fetchSupplierInvoiceSourceDocument(id: number): Promise<Blob> {
	const response = await apiClient.getBlob(`/api/v1/supplier-invoices/${id}/source-document`);
	return response.blob();
}

/**
 * Joint le document à traiter `importedSupplierInvoiceId` à la facture `invoiceId`
 * (Story 30-2c, AC19, Comptable+). Rend la facture relue, `sourceDocument` calculé.
 * Refus : `404 IMPORTED_INVOICE_NOT_FOUND`, `404 NOT_FOUND`,
 * `409 IMPORT_NOT_PENDING_COMPLETION`, `409 SOURCE_DOCUMENT_ALREADY_ATTACHED`.
 */
export async function attachSourceDocument(
	invoiceId: number,
	importedSupplierInvoiceId: number,
): Promise<SupplierInvoiceResponse> {
	return apiClient.post(`/api/v1/supplier-invoices/${invoiceId}/source-document`, {
		importedSupplierInvoiceId,
	});
}

/**
 * Détache le document de la facture (Story 30-2c, AC24, Comptable+) : il revient dans
 * « Documents à traiter ». Rend la facture relue, `sourceDocument: null`.
 */
export async function detachSourceDocument(invoiceId: number): Promise<SupplierInvoiceResponse> {
	return apiClient.delete<SupplierInvoiceResponse>(
		`/api/v1/supplier-invoices/${invoiceId}/source-document`,
	);
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
 * Duplication locale (~10 lignes), antérieure au module partagé
 * `lib/shared/utils/download.ts` (Story 25-1c-b1) — sa migration est suivie par
 * l'issue #438. try/finally pour le cleanup même si `a.click()` jette.
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
