// Story 30-2a (AC5, T-a5) — la page « Documents à traiter ».
//
// i18nMsg rend son repli, variables substituées : les textes vérifiés sont ceux du
// repli français, qui égale le catalogue fr-CH.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor, cleanup } from '@testing-library/svelte';
import type {
	ImportedSupplierInvoice,
	InboxImportReport,
} from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
import { notifySuccess, notifyWarning } from '$lib/shared/utils/notify';
import { catalogueDe } from '$lib/shared/i18n-repli-catalogue';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifySuccess: vi.fn(),
	notifyError: vi.fn(),
	notifyWarning: vi.fn(),
}));

const listImportedMock = vi.fn();
const triggerMock = vi.fn();
const discardMock = vi.fn();
const fetchDocMock = vi.fn();
vi.mock('$lib/features/imported-supplier-invoices/imported-supplier-invoices.api', () => ({
	listImported: (s: string) => listImportedMock(s),
	triggerInboxImport: () => triggerMock(),
	completeImport: vi.fn(),
	discardImport: (id: number) => discardMock(id),
	downloadImportedSourceDocument: vi.fn(),
	fetchImportedSourceDocument: (id: number) => fetchDocMock(id),
}));
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: () => Promise.resolve({ items: [], total: 0, limit: 200, offset: 0 }),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => Promise.resolve([]),
}));
vi.mock('$lib/features/vat-rates/vat-rates.api', () => ({
	listVatRates: () => Promise.resolve([]),
}));

import Page from './+page.svelte';

function row(over: Partial<ImportedSupplierInvoice>): ImportedSupplierInvoice {
	return {
		id: 1,
		companyId: 1,
		status: 'to_complete',
		supplierInvoiceId: null,
		fileHash: 'h',
		storagePath: 'h.png',
		originalFilename: 'doc.png',
		mimeType: 'image/png',
		byteSize: 10,
		creditorIban: null,
		isQrIban: null,
		creditorAddressType: null,
		creditorName: null,
		creditorLine1: null,
		creditorLine2: null,
		creditorPostalCode: null,
		creditorTown: null,
		creditorCountry: null,
		referenceType: null,
		referenceValue: null,
		amount: null,
		currency: null,
		unstructuredMessage: null,
		billingInformation: null,
		version: 1,
		createdAt: '2026-10-15T08:30:00.000',
		updatedAt: '2026-10-15T08:30:00.000',
		...over,
	};
}

const AVEC_QR = row({
	id: 7,
	originalFilename: 'facture-qr.png',
	creditorIban: 'CH9300762011623852957',
	isQrIban: false,
	creditorAddressType: 'K',
	creditorName: 'Fournisseur QR SA',
	creditorCountry: 'CH',
	referenceType: 'SCOR',
	referenceValue: 'RF18539007547034',
	amount: '100.0000',
	currency: 'CHF',
});
const SANS_QR = row({ id: 8, originalFilename: 'ticket.png' });

beforeEach(() => {
	listImportedMock.mockReset();
	triggerMock.mockReset();
	discardMock.mockReset();
	fetchDocMock.mockReset();
	vi.mocked(notifySuccess).mockClear();
	vi.mocked(notifyWarning).mockClear();
	vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:x');
	vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
});

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

describe('Documents à traiter', () => {
	it('titre, nom du fichier, date d’import, badge « QR-facture » ou « Aucune QR-facture exploitable »', async () => {
		listImportedMock.mockResolvedValue([AVEC_QR, SANS_QR]);
		const { findAllByTestId, getByRole, getAllByTestId } = render(Page);
		const rows = await findAllByTestId('imported-row');
		expect(getByRole('heading', { level: 1 }).textContent).toMatch(/Documents à traiter/);
		expect(listImportedMock).toHaveBeenCalledWith('to_complete');
		expect(rows).toHaveLength(2);
		expect(rows[0].textContent).toMatch(/facture-qr\.png/);
		expect(rows[0].textContent).toMatch(/Importé le 15\.10\.2026/);
		const qrBadge = rows[0].querySelector('[data-testid="imported-badge-qr"]');
		expect(qrBadge?.textContent).toMatch(/QR-facture/);
		expect(qrBadge?.textContent).toMatch(/Fournisseur QR SA/);
		expect(qrBadge?.textContent).toMatch(/RF18539007547034/);
		expect(rows[0].querySelector('[data-testid="imported-badge-no-qr"]')).toBeNull();
		expect(rows[1].querySelector('[data-testid="imported-badge-no-qr"]')?.textContent).toMatch(
			/Aucune QR-facture exploitable/,
		);
		expect(rows[1].querySelector('[data-testid="imported-badge-qr"]')).toBeNull();
		expect(getAllByTestId('imported-discard')[0].textContent).toMatch(/Retirer de la liste/);
	});

	it('« Aperçu » affiche le document sous sa ligne, puis se referme', async () => {
		listImportedMock.mockResolvedValue([SANS_QR]);
		fetchDocMock.mockResolvedValue(new Blob(['x'], { type: 'image/png' }));
		const { findByTestId, queryByTestId, getByTestId } = render(Page);
		await fireEvent.click(await findByTestId('imported-preview'));
		expect(await findByTestId('document-preview-image')).toBeTruthy();
		expect(fetchDocMock).toHaveBeenCalledWith(8);
		expect(getByTestId('imported-preview').textContent).toMatch(/Fermer l’aperçu/);
		await fireEvent.click(getByTestId('imported-preview'));
		await waitFor(() => expect(queryByTestId('imported-preview-panel')).toBeNull());
	});

	it('l’aperçu ouvert ne relit pas le document quand la liste se recharge (fonction de lecture stable, M-a17)', async () => {
		// Deux lectures de la liste, deux objets distincts pour la même ligne.
		listImportedMock
			.mockResolvedValueOnce([{ ...SANS_QR }])
			.mockResolvedValueOnce([{ ...SANS_QR }]);
		triggerMock.mockResolvedValue({
			accepted: [],
			failed: [],
			warnings: [],
			skipped: [],
			truncatedAt: null,
		});
		fetchDocMock.mockResolvedValue(new Blob(['x'], { type: 'image/png' }));
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('imported-preview'));
		await findByTestId('document-preview-image');
		await fireEvent.click(await findByTestId('inbox-import-trigger'));
		await waitFor(() => expect(listImportedMock).toHaveBeenCalledTimes(2));
		await findByTestId('inbox-import-report');
		await findByTestId('document-preview-image');
		expect(fetchDocMock).toHaveBeenCalledTimes(1);
	});

	it('le rapport dit « aucune QR-facture exploitable » ou « QR illisible » pour un document accepté sans QR', async () => {
		listImportedMock.mockResolvedValue([]);
		const report: InboxImportReport = {
			accepted: [
				{ importedSupplierInvoiceId: 1, fileName: 'avec.png', qr: 'present' },
				{ importedSupplierInvoiceId: 2, fileName: 'sans.png', qr: 'none' },
				{
					importedSupplierInvoiceId: 3,
					fileName: 'abime.png',
					qr: 'unreadable',
					qrError: 'QR_AMOUNT_OUT_OF_RANGE',
				},
			],
			failed: [],
			warnings: [],
			skipped: [],
			truncatedAt: null,
		};
		triggerMock.mockResolvedValue(report);
		const { findByTestId, findAllByTestId } = render(Page);
		await fireEvent.click(await findByTestId('inbox-import-trigger'));
		const panel = await findByTestId('inbox-import-report');
		// Story 34-9 (AC-6, P2-9-1) : le résumé composé, et une ligne par fichier — le
		// document avec QR compris —, dans l'ordre des noms.
		expect(panel.textContent).toMatch(/3 document\(s\) importé\(s\)\./);
		const lines = await findAllByTestId('inbox-import-accepted-row');
		expect(lines).toHaveLength(3);
		expect(lines[0].textContent).toMatch(/abime\.png.*QR illisible/s);
		expect(lines[0].textContent).toMatch(/Montant du QR hors des limites admises/);
		// Story 30-2b : la saisie de l'IBAN et de la référence existe désormais.
		expect(lines[0].textContent).toMatch(/saisissez l’IBAN et la référence à la main/);
		expect(lines[1].textContent).toMatch(/avec\.png.*Importé\./s);
		expect(lines[2].textContent).toMatch(/sans\.png.*aucune QR-facture exploitable/s);
	});

	it('« Créer une facture » mène à la page de création à côté du document (Story 30-2b, AC10)', async () => {
		listImportedMock.mockResolvedValue([SANS_QR]);
		const { findByTestId, queryByTestId } = render(Page);
		const link = await findByTestId('imported-create-invoice');
		expect(link.getAttribute('href')).toBe('/supplier-invoices/new?document=8');
		expect(link.textContent).toContain('Créer une facture');
		expect(queryByTestId('imported-complete-form')).toBeNull();
	});

	it('« Retirer de la liste » demande confirmation, puis retire la ligne', async () => {
		listImportedMock.mockResolvedValue([SANS_QR]);
		discardMock.mockResolvedValue(undefined);
		const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
		const { findByTestId, queryAllByTestId } = render(Page);
		await fireEvent.click(await findByTestId('imported-discard'));
		expect(confirmSpy).toHaveBeenCalledWith(
			'Retirer ce document de la liste ? Le fichier reste conservé.',
		);
		await waitFor(() => expect(discardMock).toHaveBeenCalledWith(8));
		await waitFor(() => expect(queryAllByTestId('imported-row')).toHaveLength(0));
	});

	it('liste vide : « Aucun document à traiter. »', async () => {
		listImportedMock.mockResolvedValue([]);
		const { findByTestId } = render(Page);
		expect((await findByTestId('imported-empty')).textContent).toMatch(
			/Aucun document à traiter\./,
		);
	});
});

// Story 34-9 (#969, #974, AC-4 à AC-6) — une ligne par fichier, le doublon et son lien,
// l'avertissement.
describe('le rapport d’import nomme chaque fichier (Story 34-9)', () => {
	/** Rapport vide, complété par `over`. */
	function rapport(over: Partial<InboxImportReport>): InboxImportReport {
		return { accepted: [], failed: [], warnings: [], skipped: [], truncatedAt: null, ...over };
	}

	const MIXTE = rapport({
		accepted: [
			{ importedSupplierInvoiceId: 1, fileName: 'sans.png', qr: 'none' },
			{ importedSupplierInvoiceId: 2, fileName: 'avec.png', qr: 'present' },
		],
		failed: [
			{
				fileName: 'gros.png',
				errorCode: 'FILE_TOO_LARGE',
				details: { byteSize: 30, maxBytes: 10, movedTo: 'gros_0f0f0f0f.png' },
			},
			{
				fileName: 'dup-facture.png',
				errorCode: 'DUPLICATE',
				details: {
					existingDocumentId: 9,
					existingStatus: 'completed',
					supplierInvoiceId: 42,
					movedTo: 'dup-facture_1a2b3c4d.png',
				},
			},
			{
				fileName: 'dup-attente.png',
				errorCode: 'DUPLICATE',
				details: {
					existingDocumentId: 7,
					existingStatus: 'to_complete',
					supplierInvoiceId: null,
					movedTo: 'dup-attente_1a2b3c4d.png',
				},
			},
		],
		warnings: ['message du serveur, que l’écran n’affiche plus'],
		skipped: [{ fileName: 'en-cours.png', reason: 'UNSTABLE' }],
		truncatedAt: 200,
	});

	async function importer(r: InboxImportReport) {
		triggerMock.mockResolvedValue(r);
		const vue = render(Page);
		await fireEvent.click(await vue.findByTestId('inbox-import-trigger'));
		const panel = await vue.findByTestId('inbox-import-report');
		await waitFor(() => expect(listImportedMock).toHaveBeenCalledTimes(2));
		return { ...vue, panel };
	}

	it('rapport mixte : six lignes dans l’ordre des noms, états, liens, failed/, résumé, troncature, un avertissement (n = 4)', async () => {
		listImportedMock.mockResolvedValue([]);
		const { panel, getAllByTestId } = await importer(MIXTE);
		const rows = getAllByTestId('inbox-import-file-row');
		expect(rows.map((r) => r.querySelector('.font-mono')?.textContent)).toEqual([
			'avec.png',
			'dup-attente.png',
			'dup-facture.png',
			'en-cours.png',
			'gros.png',
			'sans.png',
		]);
		expect(rows.map((r) => r.getAttribute('data-state'))).toEqual([
			'imported',
			'duplicate',
			'duplicate',
			'not-processed',
			'refused',
			'imported-no-qr',
		]);

		// Doublon en attente : lien vers la ligne de la liste, plus bas sur la page.
		expect(rows[1].textContent).toMatch(/Déjà importé :.*en attente dans Documents à traiter/s);
		expect(rows[1].querySelector('a')?.getAttribute('href')).toBe('#imported-7');
		expect(rows[1].textContent).toMatch(
			/Le fichier a été déplacé dans failed\/ sous le nom dup-attente_1a2b3c4d\.png ; supprimez-le à la main\./,
		);
		// Doublon transformé en facture : lien vers la facture.
		expect(rows[2].textContent).toMatch(/Déjà importé :.*transformé en facture/s);
		expect(rows[2].querySelector('a')?.getAttribute('href')).toBe('/supplier-invoices/42');
		expect(rows[2].textContent).toMatch(/dup-facture_1a2b3c4d\.png/);
		// Non traité : motif traduit, aucune phrase sur failed/.
		expect(rows[3].textContent).toMatch(/en cours d’écriture, laissé dans le dossier : relancez l’import/);
		expect(rows[3].textContent).not.toMatch(/failed\//);
		// Refus : motif et nom rangé.
		expect(rows[4].textContent).toMatch(/Refusé : Fichier trop volumineux/);
		expect(rows[4].textContent).toMatch(/gros_0f0f0f0f\.png/);
		expect(rows[5].textContent).toMatch(/aucune QR-facture exploitable/);

		// Anciens testids gardés sur les lignes concernées.
		expect(getAllByTestId('inbox-import-accepted-row')).toHaveLength(2);
		expect(getAllByTestId('inbox-import-failed-row')).toHaveLength(3);

		expect(panel.textContent).toMatch(
			/2 document\(s\) importé\(s\), 2 déjà importé\(s\), 1 refusé\(s\), 1 non traité\(s\)\./,
		);
		expect(panel.textContent).toMatch(
			/La limite de 200 fichiers par import est atteinte : relancez l’import pour traiter le reste\./,
		);
		// `warnings[]` n'est plus affiché (#974).
		expect(panel.textContent).not.toMatch(/message du serveur/);
		expect(panel.className).toMatch(/border-warning/);

		expect(notifyWarning).toHaveBeenCalledTimes(1);
		expect(vi.mocked(notifyWarning).mock.calls[0][0]).toBe(
			'Import terminé : 4 fichier(s) non importé(s). Voir le rapport.',
		);
		expect(notifySuccess).not.toHaveBeenCalled();
	});

	it('doublon retiré de la liste, déjà traité sans facture, ou sans document relu : sans lien', async () => {
		listImportedMock.mockResolvedValue([]);
		const { getAllByTestId } = await importer(
			rapport({
				failed: [
					{
						fileName: 'a.png',
						errorCode: 'DUPLICATE',
						details: {
							existingDocumentId: 3,
							existingStatus: 'discarded',
							supplierInvoiceId: null,
							movedTo: 'a_11111111.png',
						},
					},
					{
						fileName: 'b.png',
						errorCode: 'DUPLICATE',
						details: {
							existingDocumentId: 4,
							existingStatus: 'completed',
							supplierInvoiceId: null,
							movedTo: 'b_22222222.png',
						},
					},
					{ fileName: 'c.png', errorCode: 'DUPLICATE', details: { movedTo: 'c_33333333.png' } },
				],
			}),
		);
		const rows = getAllByTestId('inbox-import-file-row');
		expect(rows[0].textContent).toMatch(/Déjà importé :.*retiré de la liste/s);
		expect(rows[1].textContent).toMatch(/Déjà importé :.*déjà traité/s);
		expect(rows[2].textContent).toMatch(/Déjà importé\./);
		expect(rows[2].textContent).toMatch(/c_33333333\.png/);
		for (const r of rows) expect(r.querySelector('a')).toBeNull();
	});

	it('fichier disparu pendant l’import : ligne « non traité » avec son motif', async () => {
		listImportedMock.mockResolvedValue([]);
		const { getAllByTestId } = await importer(
			rapport({ skipped: [{ fileName: 'parti.pdf', reason: 'VANISHED' }] }),
		);
		const rows = getAllByTestId('inbox-import-file-row');
		expect(rows).toHaveLength(1);
		expect(rows[0].getAttribute('data-state')).toBe('not-processed');
		expect(rows[0].textContent).toMatch(/parti\.pdf.*disparu pendant l’import/s);
		expect(notifyWarning).toHaveBeenCalledTimes(1);
		expect(vi.mocked(notifyWarning).mock.calls[0][0]).toBe(
			'Import terminé : 1 fichier(s) non importé(s). Voir le rapport.',
		);
	});

	it('rapport sans échec : notification de succès, encadré sans bordure d’avertissement', async () => {
		listImportedMock.mockResolvedValue([]);
		const { panel } = await importer(
			rapport({ accepted: [{ importedSupplierInvoiceId: 1, fileName: 'a.png', qr: 'none' }] }),
		);
		expect(notifySuccess).toHaveBeenCalledTimes(1);
		expect(vi.mocked(notifySuccess).mock.calls[0][0]).toBe(
			'Import terminé : 1 document(s) importé(s).',
		);
		expect(notifyWarning).not.toHaveBeenCalled();
		expect(panel.className).not.toMatch(/border-warning/);
	});

	it('rapport vide : aucune notification, résumé à zéro', async () => {
		listImportedMock.mockResolvedValue([]);
		const { panel, queryAllByTestId } = await importer(rapport({}));
		expect(notifySuccess).not.toHaveBeenCalled();
		expect(notifyWarning).not.toHaveBeenCalled();
		expect(queryAllByTestId('inbox-import-file-row')).toHaveLength(0);
		expect(panel.textContent).toMatch(/0 document\(s\) importé\(s\)\./);
	});

	it('limite atteinte seule : avertissement « la limite par import est atteinte »', async () => {
		listImportedMock.mockResolvedValue([]);
		await importer(
			rapport({
				accepted: [{ importedSupplierInvoiceId: 1, fileName: 'a.png', qr: 'none' }],
				truncatedAt: 1,
			}),
		);
		expect(notifyWarning).toHaveBeenCalledTimes(1);
		expect(vi.mocked(notifyWarning).mock.calls[0][0]).toBe(
			'Import terminé : la limite par import est atteinte. Voir le rapport.',
		);
		expect(notifySuccess).not.toHaveBeenCalled();
	});

	it('rechargement de la liste en échec : une seule notification, qui dit les deux', async () => {
		listImportedMock.mockResolvedValueOnce([]).mockRejectedValueOnce(new Error('réseau'));
		await importer(MIXTE);
		expect(notifyWarning).toHaveBeenCalledTimes(1);
		const [titre, description] = vi.mocked(notifyWarning).mock.calls[0];
		expect(titre).toBe('Import terminé : 4 fichier(s) non importé(s). Voir le rapport.');
		expect(description).toMatch(/La liste n’a pas pu être rechargée/);
		expect(notifySuccess).not.toHaveBeenCalled();
	});

	it('rechargement en échec après un import réussi : un seul avertissement, qui dit les deux', async () => {
		listImportedMock.mockResolvedValueOnce([]).mockRejectedValueOnce(new Error('réseau'));
		await importer(
			rapport({ accepted: [{ importedSupplierInvoiceId: 1, fileName: 'a.png', qr: 'none' }] }),
		);
		expect(notifyWarning).toHaveBeenCalledTimes(1);
		const [titre, description] = vi.mocked(notifyWarning).mock.calls[0];
		expect(titre).toBe('Import terminé : 1 document(s) importé(s).');
		expect(description).toMatch(/La liste n’a pas pu être rechargée/);
		expect(notifySuccess).not.toHaveBeenCalled();
	});

	it('chaque ligne de la liste porte l’ancre que vise le lien d’un doublon', async () => {
		listImportedMock.mockResolvedValue([SANS_QR]);
		const { findAllByTestId } = render(Page);
		const rows = await findAllByTestId('imported-row');
		expect(rows[0].id).toBe('imported-8');
	});
});

// Story 34-9 (AC-4, F9-1, C-e342-21) — les clés neuves existent dans les quatre catalogues,
// et aucune ne porte de sélecteur Fluent : le client reçoit les messages pré-résolus, un
// sélecteur y arriverait figé sur sa variante par défaut.
describe('les textes du rapport d’import aux catalogues (Story 34-9)', () => {
	const CLES_NEUVES = [
		'imported-supplier-invoices-report-summary-imported',
		'imported-supplier-invoices-report-summary-duplicates',
		'imported-supplier-invoices-report-summary-refused',
		'imported-supplier-invoices-report-summary-not-processed',
		'imported-supplier-invoices-report-state-imported',
		'imported-supplier-invoices-report-state-imported-no-qr',
		'imported-supplier-invoices-report-state-not-processed',
		'imported-supplier-invoices-report-duplicate',
		'imported-supplier-invoices-report-duplicate-unknown',
		'imported-supplier-invoices-report-duplicate-pending',
		'imported-supplier-invoices-report-duplicate-invoice',
		'imported-supplier-invoices-report-duplicate-discarded',
		'imported-supplier-invoices-report-duplicate-processed',
		'imported-supplier-invoices-report-refused',
		'imported-supplier-invoices-report-moved-to',
		'imported-supplier-invoices-report-skipped-unstable',
		'imported-supplier-invoices-report-skipped-vanished',
		'imported-supplier-invoices-report-truncated',
		'imported-supplier-invoices-notice-not-imported',
		'imported-supplier-invoices-notice-truncated',
		'imported-supplier-invoices-notice-imported',
	];
	for (const locale of ['fr-CH', 'de-CH', 'it-CH', 'en-CH']) {
		it(`${locale} : présentes, sans sélecteur`, () => {
			const cat = catalogueDe('../crates/kesh-i18n/locales', locale);
			for (const cle of CLES_NEUVES) {
				const v = cat.get(cle);
				expect(v, `${locale} ${cle}`).toBeTruthy();
				expect(v, `${locale} ${cle}`).not.toMatch(/->/);
			}
		});
	}
});
