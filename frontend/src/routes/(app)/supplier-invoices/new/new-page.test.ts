// Story 30-2b (#685, AC10, AC14, AC16) — la page `/supplier-invoices/new`.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape. Patron des `*-page.test.ts` : mocks
// hoistés AVANT l'import du composant.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor, cleanup } from '@testing-library/svelte';
import type { ImportedSupplierInvoice } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';

const url = { value: new URL('http://localhost/supplier-invoices/new') };
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
	page: {
		get url() {
			return url.value;
		},
	},
}));
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
const role = { value: 'Comptable' };
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return { role: role.value, username: 'c', userId: '1' };
		},
	},
}));

const listImportedMock = vi.fn();
const candidatesMock = vi.fn();
const fetchDocMock = vi.fn();
vi.mock('$lib/features/imported-supplier-invoices/imported-supplier-invoices.api', () => ({
	listImported: (s: string) => listImportedMock(s),
	getSupplierCandidates: (id: number) => candidatesMock(id),
	fetchImportedSourceDocument: (id: number) => fetchDocMock(id),
	completeImport: vi.fn(),
}));
const listContactsMock = vi.fn();
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: (q: unknown) => listContactsMock(q),
	createContact: vi.fn(),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: () => Promise.resolve([]) }));
vi.mock('$lib/features/vat-rates/vat-rates.api', () => ({ listVatRates: () => Promise.resolve([]) }));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: () => Promise.resolve([]) }));
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	createSupplierInvoice: vi.fn(),
	scanQrSupplierInvoice: vi.fn(),
}));

import Page from './+page.svelte';

function doc(over: Partial<ImportedSupplierInvoice> = {}): ImportedSupplierInvoice {
	return {
		id: 7,
		companyId: 1,
		status: 'to_complete',
		supplierInvoiceId: null,
		fileHash: 'h',
		storagePath: 'h.png',
		originalFilename: 'facture-dubarde.png',
		mimeType: 'image/png',
		byteSize: 10,
		creditorIban: 'CH9300762011623852957',
		isQrIban: false,
		creditorAddressType: 'K',
		creditorName: 'Dubarde SA',
		creditorLine1: 'Rue du Lac 1',
		creditorLine2: '1000 Lausanne',
		creditorPostalCode: null,
		creditorTown: null,
		creditorCountry: 'CH',
		referenceType: 'NON',
		referenceValue: null,
		amount: '100.00',
		currency: 'CHF',
		unstructuredMessage: null,
		billingInformation: null,
		version: 1,
		createdAt: '2026-10-15T08:30:00',
		updatedAt: '2026-10-15T08:30:00',
		...over,
	};
}

beforeEach(() => {
	role.value = 'Comptable';
	url.value = new URL('http://localhost/supplier-invoices/new');
	listImportedMock.mockReset();
	candidatesMock.mockReset();
	fetchDocMock.mockReset();
	listContactsMock.mockReset();
	listContactsMock.mockResolvedValue({ items: [], total: 0, limit: 200, offset: 0 });
	candidatesMock.mockResolvedValue({ candidates: [] });
	fetchDocMock.mockResolvedValue(new Blob(['x'], { type: 'image/png' }));
});
afterEach(() => cleanup());

describe('page /supplier-invoices/new', () => {
	it('sans document : le formulaire seul, avec le scan, sans lire les documents à traiter', async () => {
		const { findByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(queryByTestId('supplier-invoice-scan-btn')).not.toBeNull();
		expect(queryByTestId('supplier-invoice-new-preview')).toBeNull();
		expect(listImportedMock).not.toHaveBeenCalled();
	});

	it('à côté du document : aperçu et formulaire, sans scan', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		listImportedMock.mockResolvedValue([doc()]);
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(getByTestId('supplier-invoice-new-preview').textContent).toContain('facture-dubarde.png');
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
		await waitFor(() => expect(fetchDocMock).toHaveBeenCalledWith(7));
	});

	it.each([
		['inconnu', 'document=99'],
		['illisible', 'document=abc'],
	])('document %s ou plus à traiter : le même message, un lien, aucun formulaire', async (_l, q) => {
		url.value = new URL(`http://localhost/supplier-invoices/new?${q}`);
		listImportedMock.mockResolvedValue([doc()]);
		const { findByTestId, queryByTestId } = render(Page);
		const gone = await findByTestId('supplier-invoice-new-doc-gone');
		expect(gone.textContent).toContain('Ce document n’est plus à traiter.');
		expect(gone.querySelector('a')?.getAttribute('href')).toBe('/supplier-invoices/import');
		expect(queryByTestId('supplier-invoice-form')).toBeNull();
	});

	it('état de chargement, puis état d’erreur de la lecture', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		let rejectList: (e: unknown) => void = () => {};
		listImportedMock.mockReturnValue(
			new Promise((_, reject) => {
				rejectList = reject;
			}),
		);
		const { findByTestId, getByTestId } = render(Page);
		expect(getByTestId('supplier-invoice-new-loading')).not.toBeNull();
		rejectList({ code: 'INTERNAL_ERROR', status: 500, message: 'panne' });
		const err = await findByTestId('supplier-invoice-new-error');
		expect(err.textContent).toContain('panne');
	});

	it('propose par IBAN puis par nom, la recherche filtrée sur les fournisseurs (M-b5)', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		listImportedMock.mockResolvedValue([doc()]);
		const { findAllByTestId } = render(Page);
		await findAllByTestId('supplier-invoice-form');
		await waitFor(() => expect(candidatesMock).toHaveBeenCalledWith(7));
		const searchCall = listContactsMock.mock.calls.find(
			(c) => (c[0] as { search?: string }).search !== undefined,
		);
		expect(searchCall?.[0]).toEqual({ search: 'dubarde', isSupplier: true, limit: 20 });
	});

	it('un document sans QR ne déclenche aucune proposition', async () => {
		url.value = new URL('http://localhost/supplier-invoices/new?document=7');
		listImportedMock.mockResolvedValue([
			doc({
				creditorIban: null,
				isQrIban: null,
				creditorAddressType: null,
				creditorName: null,
				creditorCountry: null,
				referenceType: null,
				amount: null,
				currency: null,
			}),
		]);
		const { findByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(candidatesMock).not.toHaveBeenCalled();
	});

	it('pour un rôle Consultation, la page ne propose rien', async () => {
		role.value = 'Consultation';
		const { findByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-new-forbidden');
		expect(queryByTestId('supplier-invoice-form')).toBeNull();
		expect(listContactsMock).not.toHaveBeenCalled();
	});
});
