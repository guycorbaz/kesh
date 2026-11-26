/**
 * Story 55-3b (AC-2, AC-3, AC-5, AC-11) — la visionneuse des justificatifs : la liste au-dessus de
 * `DocumentPreview`, le lecteur de chaque genre, « PDF pas encore émis » sans rien lire, la lecture unique.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		fallback.replace(/\{ \$(\w+) \}/g, (_: string, k: string) => String(args?.[k] ?? '')),
}));
vi.mock('./reports.api', async (importOriginal) => ({
	...(await importOriginal<typeof import('./reports.api')>()),
	fetchInvoicePdf: vi.fn(),
	fetchCreditNotePdf: vi.fn(),
	fetchEntryDocuments: vi.fn(),
}));
vi.mock('$lib/features/imported-supplier-invoices/imported-supplier-invoices.api', () => ({
	fetchSupplierInvoiceSourceDocument: vi.fn(),
}));

import * as api from './reports.api';
import * as supplierApi from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.api';
import EntryDocumentsViewer from './EntryDocumentsViewer.svelte';
import type { EntryDocument } from './reports.types';

const invoicePdf = vi.mocked(api.fetchInvoicePdf);
const creditNotePdf = vi.mocked(api.fetchCreditNotePdf);
const supplierDoc = vi.mocked(supplierApi.fetchSupplierInvoiceSourceDocument);

const pdf = () => new Blob(['%PDF-1.7'], { type: 'application/pdf' });
const vide = () => new Blob([], { type: 'application/pdf' });

const facture = (over: Partial<EntryDocument> = {}): EntryDocument => ({
	kind: 'invoice',
	id: 42,
	number: 'F-2026-0007',
	pdfStatus: 'frozen',
	...over,
});
const avoir = (over: Partial<EntryDocument> = {}): EntryDocument => ({
	kind: 'creditNote',
	id: 3,
	number: 'AV-2026-0001',
	...over,
});
const fournisseur = (over: Partial<EntryDocument> = {}): EntryDocument => ({
	kind: 'supplierDocument',
	id: 18,
	number: 'FF-2026-004',
	fileName: 'facture-mai.pdf',
	...over,
});

beforeEach(() => {
	vi.clearAllMocks();
	let n = 0;
	vi.spyOn(URL, 'createObjectURL').mockImplementation(() => `blob:doc-${++n}`);
	vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
	invoicePdf.mockImplementation(async () => pdf());
	creditNotePdf.mockImplementation(async () => pdf());
	supplierDoc.mockImplementation(async () => pdf());
});
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

describe('EntryDocumentsViewer', () => {
	it('un seul justificatif : aucune liste, la visionneuse l’ouvre directement (mutation : liste rendue à un élément)', async () => {
		const r = render(EntryDocumentsViewer, { documents: [facture()] });
		expect(r.queryByTestId('entry-documents-list')).toBeNull();
		const frame = (await r.findByTestId('document-preview-frame')) as HTMLIFrameElement;
		expect(frame.getAttribute('title')).toBe('facture-F-2026-0007.pdf');
		expect(invoicePdf).toHaveBeenCalledTimes(1);
		expect(invoicePdf).toHaveBeenCalledWith(42);
	});

	it('AC-11 — le PDF s’ouvre sans le panneau des vignettes : fragment `#pagemode=none&navpanes=0` (mutation : `hideThumbnails` retiré)', async () => {
		const r = render(EntryDocumentsViewer, { documents: [facture()] });
		const frame = (await r.findByTestId('document-preview-frame')) as HTMLIFrameElement;
		expect(frame.getAttribute('src')).toMatch(/^blob:doc-\d+#pagemode=none&navpanes=0$/);
	});

	it('deux justificatifs : la liste en haut, le premier affiché et marqué ; un clic sur le second ne lit que lui (mutation : sélection ignorée)', async () => {
		const r = render(EntryDocumentsViewer, { documents: [facture(), avoir()] });
		const list = r.getByTestId('entry-documents-list');
		expect(list.querySelectorAll('li')).toHaveLength(2);
		expect(r.getByTestId('entry-documents-item-invoice-42').getAttribute('aria-current')).toBe('true');
		expect(r.getByTestId('entry-documents-item-creditNote-3').getAttribute('aria-current')).toBeNull();
		await r.findByTestId('document-preview-frame');
		expect(invoicePdf).toHaveBeenCalledTimes(1);
		await fireEvent.click(r.getByTestId('entry-documents-item-creditNote-3'));
		await waitFor(() => expect(creditNotePdf).toHaveBeenCalledWith(3));
		expect(creditNotePdf).toHaveBeenCalledTimes(1);
		expect(invoicePdf).toHaveBeenCalledTimes(1);
		expect(r.getByTestId('entry-documents-item-creditNote-3').getAttribute('aria-current')).toBe('true');
		await waitFor(() =>
			expect(r.getByTestId('document-preview-frame').getAttribute('title')).toBe('avoir-AV-2026-0001.pdf'),
		);
	});

	it('libellés de la liste : genre, numéro (l’identifiant à défaut), nom du fichier fournisseur', () => {
		const r = render(EntryDocumentsViewer, {
			documents: [facture({ number: null }), avoir(), fournisseur()],
		});
		expect(r.getByTestId('entry-documents-item-invoice-42').textContent?.trim()).toBe('Facture #42');
		expect(r.getByTestId('entry-documents-item-creditNote-3').textContent?.trim()).toBe('Avoir AV-2026-0001');
		expect(r.getByTestId('entry-documents-item-supplierDocument-18').textContent?.trim()).toBe(
			'Document fournisseur FF-2026-004 — facture-mai.pdf',
		);
	});

	for (const [cas, status] of [
		['notFrozen', 'notFrozen'],
		['champ absent', undefined],
		['valeur inconnue', 'external'],
	] as const) {
		it(`facture « ${cas} » : « PDF pas encore émis », aucun lecteur, aucun DocumentPreview (mutation : lecteur de facture appelé)`, async () => {
			const r = render(EntryDocumentsViewer, { documents: [facture({ pdfStatus: status })] });
			await new Promise((res) => setTimeout(res, 10));
			// Assertions d'abord sur la lecture : c'est elle qui figerait la facture.
			expect(invoicePdf).not.toHaveBeenCalled();
			expect(r.queryByTestId('document-preview')).toBeNull();
			const msg = r.queryByTestId('entry-documents-not-issued');
			expect(msg?.getAttribute('role')).toBe('status');
			expect(msg?.textContent).toContain('PDF pas encore émis');
		});
	}

	for (const [cas, doc] of [
		['facture figée', facture()],
		['avoir', avoir()],
		['document fournisseur', fournisseur()],
	] as const) {
		it(`${cas} : aucun message « pas encore émis » (mutation : message toujours affiché)`, async () => {
			const r = render(EntryDocumentsViewer, { documents: [doc] });
			expect(r.queryByTestId('entry-documents-not-issued')).toBeNull();
			await r.findByTestId('document-preview-frame');
		});
	}

	it('le message suit l’élément CHOISI : notFrozen puis frozen, un clic sur la seconde ouvre son PDF (mutation : état tenu par list[0])', async () => {
		const r = render(EntryDocumentsViewer, {
			documents: [facture({ id: 1, number: 'F-1', pdfStatus: 'notFrozen' }), facture({ id: 2, number: 'F-2' })],
		});
		expect(r.getByTestId('entry-documents-not-issued')).toBeTruthy();
		await fireEvent.click(r.getByTestId('entry-documents-item-invoice-2'));
		expect(r.queryByTestId('entry-documents-not-issued')).toBeNull();
		await r.findByTestId('document-preview-frame');
		expect(invoicePdf).toHaveBeenCalledTimes(1);
		expect(invoicePdf).toHaveBeenCalledWith(2);
	});

	it('les trois lecteurs et leurs noms de repli (numéro nul : identifiant ; fichier fournisseur absent : justificatif-<id>)', async () => {
		for (const [doc, lecteur, nom] of [
			[facture({ number: null }), invoicePdf, 'facture-42.pdf'],
			[avoir({ number: null }), creditNotePdf, 'avoir-3.pdf'],
			[fournisseur({ fileName: undefined }), supplierDoc, 'justificatif-18'],
			[fournisseur(), supplierDoc, 'facture-mai.pdf'],
		] as const) {
			vi.clearAllMocks();
			const r = render(EntryDocumentsViewer, { documents: [doc] });
			const frame = await r.findByTestId('document-preview-frame');
			expect(frame.getAttribute('title')).toBe(nom);
			expect(lecteur).toHaveBeenCalledTimes(1);
			expect(lecteur).toHaveBeenCalledWith(doc.id);
			r.unmount();
		}
	});

	it('un Blob vide, pour chacun des trois genres : « Le document reçu est vide. » (mutation : contrôle de taille retiré)', async () => {
		invoicePdf.mockImplementation(async () => vide());
		creditNotePdf.mockImplementation(async () => vide());
		supplierDoc.mockImplementation(async () => vide());
		for (const doc of [facture(), avoir(), fournisseur()]) {
			const r = render(EntryDocumentsViewer, { documents: [doc] });
			await waitFor(() =>
				expect(r.queryByTestId('document-preview-error')?.textContent?.trim()).toBe(
					'Le document reçu est vide.',
				),
			);
			r.unmount();
		}
	});

	it('410 INVOICE_PDF_GONE : le message du serveur, qui nomme le fichier (pdfErrorMessage)', async () => {
		invoicePdf.mockRejectedValue({
			code: 'INVOICE_PDF_GONE',
			status: 410,
			message: 'Le PDF émis de cette facture est introuvable : restaurez abc.pdf.',
		});
		const r = render(EntryDocumentsViewer, { documents: [facture()] });
		expect((await r.findByTestId('document-preview-error')).textContent?.trim()).toBe(
			'Le PDF émis de cette facture est introuvable : restaurez abc.pdf.',
		);
	});

	it('document fournisseur en 410 SOURCE_DOCUMENT_GONE : l’état d’aujourd’hui de DocumentPreview (`gone`)', async () => {
		supplierDoc.mockRejectedValue({ code: 'SOURCE_DOCUMENT_GONE', status: 410, message: 'parti' });
		const r = render(EntryDocumentsViewer, { documents: [fournisseur()] });
		expect(await r.findByTestId('document-preview-gone')).toBeTruthy();
	});

	it('lecture unique : la liste remplacée par des objets égaux ne relit pas le document (mutation : sélection tenue par l’objet)', async () => {
		const r = render(EntryDocumentsViewer, { documents: [facture(), avoir()] });
		await r.findByTestId('document-preview-frame');
		await r.rerender({ documents: [facture(), avoir()] });
		await r.rerender({ documents: [facture(), avoir()] });
		await new Promise((res) => setTimeout(res, 10));
		expect(invoicePdf).toHaveBeenCalledTimes(1);
	});
});
