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
	vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:x');
	vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
});

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

describe('Documents à traiter', () => {
	it('titre, nom du fichier, date d’import, badge « QR-facture » ou « Sans QR-code »', async () => {
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
			/Sans QR-code/,
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
		triggerMock.mockResolvedValue({ accepted: [], failed: [], warnings: [] });
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

	it('le rapport dit « sans QR-code » ou « QR illisible » pour un document accepté sans QR', async () => {
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
		};
		triggerMock.mockResolvedValue(report);
		const { findByTestId, findAllByTestId } = render(Page);
		await fireEvent.click(await findByTestId('inbox-import-trigger'));
		const panel = await findByTestId('inbox-import-report');
		expect(panel.textContent).toMatch(/3 document\(s\) importé\(s\)/);
		const lines = await findAllByTestId('inbox-import-accepted-row');
		expect(lines).toHaveLength(2);
		expect(lines[0].textContent).toMatch(/sans\.png.*sans QR-code/s);
		expect(lines[1].textContent).toMatch(/abime\.png.*QR illisible/s);
		expect(lines[1].textContent).toMatch(/Montant du QR hors des limites admises/);
		expect(lines[1].textContent).toMatch(/la facture créée sera sans IBAN ni référence/);
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
