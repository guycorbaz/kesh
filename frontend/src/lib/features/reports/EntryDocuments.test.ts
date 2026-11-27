/**
 * Story 55-3b (AC-1, AC-2, AC-7, R-11) — l'icône des justificatifs d'une écriture et son dialogue, montés dans
 * la vue du Journal (une instance du dialogue par vue).
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
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

import * as api from './reports.api';
import * as nav from '$app/navigation';
import JournalReportView from './JournalReportView.svelte';
import type { EntryDocument, JournalReportDto } from './reports.types';

const listDocs = vi.mocked(api.fetchEntryDocuments);
const invoicePdf = vi.mocked(api.fetchInvoicePdf);

const facture = (id: number, number: string): EntryDocument => ({
	kind: 'invoice',
	id,
	number,
	pdfStatus: 'frozen',
});

/** Un journal de trois écritures : 7 et 9 ont des justificatifs, 8 n'en a pas. */
function journal(): JournalReportDto {
	const entry = (entryId: number, entryNumber: number, hasDocuments?: boolean) => ({
		entryId,
		entryNumber,
		entryDate: '2026-02-10',
		description: `Écriture ${entryNumber}`,
		lines: [
			{ accountId: 1, accountNumber: '1100', accountName: 'Débiteurs', debit: '100.00', credit: '0.00', lineOrder: 0 },
			{ accountId: 2, accountNumber: '3200', accountName: 'Ventes', debit: '0.00', credit: '100.00', lineOrder: 1 },
		],
		...(hasDocuments ? { hasDocuments: true } : {}),
	});
	return {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		journals: [
			{
				journal: 'Ventes',
				entries: [entry(70, 7, true), entry(80, 8), entry(90, 9, true)],
				sectionTotalDebit: '300.00',
				sectionTotalCredit: '300.00',
			},
		],
		grandTotalDebit: '300.00',
		grandTotalCredit: '300.00',
	} as JournalReportDto;
}

/** Une promesse dont l'appelant décide de l'issue. */
function deferred<T>() {
	let resolve!: (v: T) => void;
	const promise = new Promise<T>((r) => (resolve = r));
	return { promise, resolve };
}

beforeEach(() => {
	vi.clearAllMocks();
	let n = 0;
	vi.spyOn(URL, 'createObjectURL').mockImplementation(() => `blob:doc-${++n}`);
	vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
	invoicePdf.mockImplementation(async () => new Blob(['%PDF'], { type: 'application/pdf' }));
	listDocs.mockImplementation(async () => [facture(42, 'F-1')]);
});
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

const dialog = () => document.querySelector('[data-testid="entry-documents-dialog"]');

describe('EntryDocuments — l’icône', () => {
	it('une icône par écriture à justificatif, aucune sans ; aucune requête avant le clic (mutation : condition `hasDocuments` retirée)', async () => {
		const r = render(JournalReportView, { dto: journal() });
		expect(r.getByTestId('journal-entry-documents-70')).toBeTruthy();
		expect(r.getByTestId('journal-entry-documents-90')).toBeTruthy();
		expect(r.queryByTestId('journal-entry-documents-80')).toBeNull();
		// Une seule icône par écriture, quel que soit le nombre de justificatifs (aucun compteur).
		expect(r.getByTestId('journal-entry-header-70').querySelectorAll('button')).toHaveLength(1);
		expect(r.getByTestId('journal-entry-documents-70').textContent?.trim()).toBe('');
		expect(listDocs).not.toHaveBeenCalled();
	});

	it('libellé accessible, icône masquée aux lecteurs d’écran', () => {
		const r = render(JournalReportView, { dto: journal() });
		const b = r.getByTestId('journal-entry-documents-70');
		expect(b.getAttribute('aria-label')).toBe("Justificatifs de l'écriture n° 7");
		expect(b.querySelector('svg')?.getAttribute('aria-hidden')).toBe('true');
	});

	it('R-8 — monochrome : aucune classe de couleur sur le bouton ni le SVG, trait `currentColor` (mutation : `text-primary` ajouté)', () => {
		const r = render(JournalReportView, { dto: journal() });
		const b = r.getByTestId('journal-entry-documents-70');
		const svg = b.querySelector('svg')!;
		// Motif ouvert (R-8) : tout préfixe de couleur, variantes comprises — aucune liste de couleurs interdites.
		const couleur = /^(?:[a-z-]+:)*(?:text|fill|stroke|bg)-/;
		for (const el of [b, svg]) {
			const classes = (el.getAttribute('class') ?? '').split(/\s+/).filter(Boolean);
			expect(classes.filter((c) => couleur.test(c))).toEqual([]);
		}
		expect(svg.getAttribute('stroke')).toBe('currentColor');
	});
});

describe('EntryDocuments — le dialogue', () => {
	it('au clic : une lecture de la liste, un indicateur, puis la visionneuse ; l’URL ne change pas (mutation : `goto` appelé)', async () => {
		const d = deferred<EntryDocument[]>();
		listDocs.mockReturnValue(d.promise);
		const r = render(JournalReportView, { dto: journal() });
		await fireEvent.click(r.getByTestId('journal-entry-documents-70'));
		await waitFor(() => expect(dialog()).not.toBeNull());
		expect(r.getByTestId('entry-documents-loading')).toBeTruthy();
		expect(listDocs).toHaveBeenCalledTimes(1);
		expect(listDocs).toHaveBeenCalledWith(70);
		d.resolve([facture(42, 'F-1')]);
		await r.findByTestId('document-preview-frame');
		expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Justificatifs de l'écriture n° 7");
		expect(nav.goto).not.toHaveBeenCalled();
	});

	it('liste vide : « plus de justificatif » ; erreur de la route : son message', async () => {
		listDocs.mockResolvedValueOnce([]);
		const r = render(JournalReportView, { dto: journal() });
		await fireEvent.click(r.getByTestId('journal-entry-documents-70'));
		expect((await r.findByTestId('entry-documents-none')).textContent).toContain('plus de justificatif');
		await fireEvent.click(r.getByTestId('entry-documents-close'));
		await waitFor(() => expect(dialog()).toBeNull());
		listDocs.mockRejectedValueOnce({ code: 'NOT_FOUND', status: 404, message: 'Écriture introuvable' });
		await fireEvent.click(r.getByTestId('journal-entry-documents-70'));
		expect((await r.findByTestId('entry-documents-error')).textContent?.trim()).toBe('Écriture introuvable');
	});

	it('fermé puis rouvert pendant la lecture de la liste : la liste de la SECONDE ouverture s’affiche (mutation : compteur d’ouverture retiré)', async () => {
		const premiere = deferred<EntryDocument[]>();
		const seconde = deferred<EntryDocument[]>();
		listDocs.mockReturnValueOnce(premiere.promise).mockReturnValueOnce(seconde.promise);
		const r = render(JournalReportView, { dto: journal() });
		await fireEvent.click(r.getByTestId('journal-entry-documents-70'));
		await waitFor(() => expect(dialog()).not.toBeNull());
		await fireEvent.click(r.getByTestId('entry-documents-close'));
		await waitFor(() => expect(dialog()).toBeNull());
		await fireEvent.click(r.getByTestId('journal-entry-documents-90'));
		await waitFor(() => expect(dialog()).not.toBeNull());
		seconde.resolve([facture(1, 'F-SECONDE'), facture(2, 'F-SECONDE-B')]);
		await r.findByTestId('entry-documents-item-invoice-1');
		premiere.resolve([facture(3, 'F-PREMIERE'), facture(4, 'F-PREMIERE-B')]);
		await new Promise((res) => setTimeout(res, 10));
		expect(r.queryByTestId('entry-documents-item-invoice-3')).toBeNull();
		expect(r.getByTestId('entry-documents-list').textContent).toContain('F-SECONDE');
		expect(r.getByTestId('entry-documents-list').textContent).not.toContain('F-PREMIERE');
	});

	it('la liste est relue à chaque ouverture', async () => {
		const r = render(JournalReportView, { dto: journal() });
		for (let i = 0; i < 2; i++) {
			await fireEvent.click(r.getByTestId('journal-entry-documents-70'));
			await r.findByTestId('document-preview-frame');
			await fireEvent.click(r.getByTestId('entry-documents-close'));
			await waitFor(() => expect(dialog()).toBeNull());
		}
		expect(listDocs).toHaveBeenCalledTimes(2);
	});

	it('« Fermer » rend le focus à l’icône qui a ouvert (la seconde) — mutation : rendu explicite du focus retiré', async () => {
		const r = render(JournalReportView, { dto: journal() });
		const premiere = r.getByTestId('journal-entry-documents-70');
		const seconde = r.getByTestId('journal-entry-documents-90');
		premiere.focus();
		// `fireEvent.click` ne donne pas le focus au bouton : sans le rendu explicite, bits-ui le rendrait à
		// la première icône (l'élément actif avant l'ouverture).
		await fireEvent.click(seconde);
		await r.findByTestId('document-preview-frame');
		await fireEvent.click(r.getByTestId('entry-documents-close'));
		await waitFor(() => expect(dialog()).toBeNull());
		await waitFor(() => expect(document.activeElement).toBe(seconde));
	});

	it('Échap ferme et rend le focus à l’icône d’origine', async () => {
		const r = render(JournalReportView, { dto: journal() });
		const premiere = r.getByTestId('journal-entry-documents-70');
		const seconde = r.getByTestId('journal-entry-documents-90');
		premiere.focus();
		await fireEvent.click(seconde);
		await r.findByTestId('document-preview-frame');
		await fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Escape' });
		await waitFor(() => expect(dialog()).toBeNull());
		await waitFor(() => expect(document.activeElement).toBe(seconde));
	});

	it('un clic à côté (sur le fond) ferme et rend le focus (mutation : `interactOutsideBehavior="ignore"`)', async () => {
		const r = render(JournalReportView, { dto: journal() });
		const premiere = r.getByTestId('journal-entry-documents-70');
		const seconde = r.getByTestId('journal-entry-documents-90');
		premiere.focus();
		await fireEvent.click(seconde);
		await r.findByTestId('document-preview-frame');
		const fond = document.querySelector('[data-slot="dialog-overlay"]') ?? document.body;
		// bits-ui (`use-dismissable-layer.svelte.js`) : `pointerdown` hors du contenu, jugé par les
		// coordonnées (`isClickTrulyOutside`, rectangle nul sous jsdom : un point positif est « dehors »),
		// rappel retardé de 10 ms.
		await new Promise((res) => setTimeout(res, 20));
		await fireEvent.pointerDown(fond, { button: 0, pointerType: 'mouse', clientX: 400, clientY: 400 });
		await fireEvent.pointerUp(fond, { button: 0, pointerType: 'mouse', clientX: 400, clientY: 400 });
		await waitFor(() => expect(dialog()).toBeNull());
		await waitFor(() => expect(document.activeElement).toBe(seconde));
	});

	it('R-11 — rapport rechargé (objet égal) dialogue ouvert : une seule lecture, titre inchangé (mutation : sélection tenue par l’objet)', async () => {
		listDocs.mockImplementation(async () => [facture(42, 'F-1'), facture(43, 'F-2')]);
		const r = render(JournalReportView, { dto: journal() });
		await fireEvent.click(r.getByTestId('journal-entry-documents-70'));
		await r.findByTestId('document-preview-frame');
		const autre = journal();
		autre.journals[0].entries[0].entryNumber = 99;
		await r.rerender({ dto: journal() });
		await r.rerender({ dto: autre });
		await new Promise((res) => setTimeout(res, 10));
		expect(invoicePdf).toHaveBeenCalledTimes(1);
		expect(document.querySelector('[role="dialog"]')?.textContent).toContain("Justificatifs de l'écriture n° 7");
	});
});
