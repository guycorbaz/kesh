/**
 * Story 55-3b (D9 à D12) — les règles des justificatifs d'une écriture, sans composant : quel lecteur lit
 * quel genre, sous quel nom, quand une facture n'est **pas** lue, et quel texte rend une erreur.
 *
 * ⚠️ **Afficher n'est pas figer** (D12) : le PDF d'une facture n'est lu que pour `pdfStatus === 'frozen'` ;
 * toute autre valeur — `'notFrozen'`, champ absent, valeur future de l'énumération ouverte — ne lit rien
 * (« PDF pas encore émis »), parce que `GET /invoices/{id}/pdf` figerait la facture.
 */
import { fetchSupplierInvoiceSourceDocument } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.api';
import { isApiError } from '$lib/shared/utils/api-client';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { pdfErrorMessage } from '$lib/shared/utils/pdf-error';
import { fetchCreditNotePdf, fetchInvoicePdf } from './reports.api';
import type { EntryDocument, EntryDocumentKind } from './reports.types';

/** Le nom de l'erreur « document vide » (une `Error` nommée, pas de classe : garde des libellés en dur). */
const EMPTY_DOCUMENT = 'EmptyDocumentError';

/**
 * Un document reçu vide (0 octet) : `DocumentPreview` ne contrôle pas la taille, le lecteur le refuse
 * donc lui-même (AC-3) ; `entryDocumentErrorMessage` le reconnaît avant tout autre cas.
 */
export function emptyDocumentError(): Error {
	const err = new Error('document vide');
	err.name = EMPTY_DOCUMENT;
	return err;
}

/** Vrai pour l'erreur « document vide » d'`entryDocumentLoader`. */
export function isEmptyDocumentError(err: unknown): boolean {
	return err instanceof Error && err.name === EMPTY_DOCUMENT;
}

/** Vrai pour une facture dont le PDF n'est pas encore émis : elle n'est pas lue (AC-5). */
export function isNotIssuedInvoice(kind: EntryDocumentKind, pdfStatus: string | undefined): boolean {
	return kind === 'invoice' && pdfStatus !== 'frozen';
}

/** Le lecteur brut d'un genre (route de la pièce). */
function readerOf(kind: EntryDocumentKind): (id: number) => Promise<Blob> {
	switch (kind) {
		case 'invoice':
			return fetchInvoicePdf;
		case 'creditNote':
			return fetchCreditNotePdf;
		case 'supplierDocument':
			return fetchSupplierInvoiceSourceDocument;
	}
}

/**
 * La fonction `load` passée à `DocumentPreview` pour un justificatif : elle lit la pièce et **rejette** un
 * `Blob` vide par `emptyDocumentError()`. Ne s'appelle jamais pour une facture non émise (voir
 * `isNotIssuedInvoice`) — l'appelant ne monte alors pas `DocumentPreview`.
 */
export function entryDocumentLoader(kind: EntryDocumentKind, id: number): () => Promise<Blob> {
	const read = readerOf(kind);
	return async () => {
		const blob = await read(id);
		if (blob.size === 0) throw emptyDocumentError();
		return blob;
	};
}

/**
 * Le nom passé à `DocumentPreview` (titre du cadre, nom téléchargé) : `facture-<numéro>.pdf`,
 * `avoir-<numéro>.pdf`, à défaut du numéro l'identifiant ; le nom importé d'un document fournisseur, à
 * défaut `justificatif-<id>`.
 */
export function entryDocumentFileName(doc: EntryDocument): string {
	const ref = doc.number !== null && doc.number.trim() !== '' ? doc.number : String(doc.id);
	switch (doc.kind) {
		case 'invoice':
			return `facture-${ref}.pdf`;
		case 'creditNote':
			return `avoir-${ref}.pdf`;
		case 'supplierDocument':
			return doc.fileName && doc.fileName.trim() !== '' ? doc.fileName : `justificatif-${doc.id}`;
	}
}

/**
 * Le libellé de l'icône d'une écriture, qui est aussi le titre du dialogue (une clé pour les deux, T0) :
 * « Justificatifs de l'écriture n° N » — `N` est le numéro affiché (au grand livre, préfixé de l'exercice
 * quand la période en traverse plusieurs).
 */
export function entryDocumentsTitle(number: string | number): string {
	return i18nMsg('reports-nav-documents-title', "Justificatifs de l'écriture n° { $number }", {
		number: String(number),
	});
}

/** Le libellé du genre — un appel littéral par clé, que lisent les gardes i18n. */
export function entryDocumentKindLabel(kind: EntryDocumentKind): string {
	switch (kind) {
		case 'invoice':
			return i18nMsg('reports-nav-documents-kind-invoice', 'Facture');
		case 'creditNote':
			return i18nMsg('reports-nav-documents-kind-credit-note', 'Avoir');
		case 'supplierDocument':
			return i18nMsg('reports-nav-documents-kind-supplier-document', 'Document fournisseur');
	}
}

/**
 * Le libellé d'un élément de la liste : genre et numéro (l'identifiant à défaut), suivi du nom du fichier
 * d'un document fournisseur.
 */
export function entryDocumentItemLabel(doc: EntryDocument): string {
	const ref = doc.number !== null && doc.number.trim() !== '' ? doc.number : `#${doc.id}`;
	const base = `${entryDocumentKindLabel(doc.kind)} ${ref}`;
	return doc.kind === 'supplierDocument' && doc.fileName ? `${base} — ${doc.fileName}` : base;
}

/**
 * Le texte d'erreur d'une lecture (propriété `errorMessage` de `DocumentPreview`, AC-4) : document vide →
 * `reports-nav-documents-empty` ; facture ou avoir refusés par l'API → `pdfErrorMessage` (le `410
 * INVOICE_PDF_GONE` nomme le fichier à restaurer) ; sinon `undefined`, le texte de `DocumentPreview`.
 */
export function entryDocumentErrorMessage(kind: EntryDocumentKind, err: unknown): string | undefined {
	if (isEmptyDocumentError(err)) {
		return i18nMsg('reports-nav-documents-empty', 'Le document reçu est vide.');
	}
	if ((kind === 'invoice' || kind === 'creditNote') && isApiError(err)) return pdfErrorMessage(err);
	return undefined;
}
