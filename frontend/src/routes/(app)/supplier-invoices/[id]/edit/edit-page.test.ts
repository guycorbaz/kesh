/**
 * Story v017-4b (#849, AC-23) — la page `/supplier-invoices/[id]/edit` : un brouillon
 * pré-rempli, enregistré par `PUT` avec la version lue ; le mode « document avec QR »
 * seulement pour une facture NÉE de la complétion d'un document à QR ; le scan caché dès
 * que la facture a un document.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (M34, M35). Patron des
 * `*-page.test.ts` : mocks hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type {
	SupplierInvoiceResponse,
	SupplierInvoiceSourceDocument,
	SupplierInvoiceSourceQr,
} from '$lib/features/supplier-invoices/supplier-invoices.types';

const gotoMock = vi.fn();
vi.mock('$app/navigation', () => ({ goto: (u: string) => gotoMock(u) }));
vi.mock('$app/state', () => ({
	page: { params: { id: '9' }, url: new URL('http://localhost/supplier-invoices/9/edit') },
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
const notifySuccess = vi.fn();
const notifyWarning = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: vi.fn(),
	notifySuccess: (m: string) => notifySuccess(m),
	notifyWarning: (m: string) => notifyWarning(m),
	notifyInfo: vi.fn(),
}));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { currentUser: { role: 'Comptable', username: 'c', userId: '1' } },
}));

const getMock = vi.fn();
const updateMock = vi.fn();
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	getSupplierInvoice: (id: number) => getMock(id),
	updateSupplierInvoice: (id: number, req: unknown) => updateMock(id, req),
	scanQrSupplierInvoice: vi.fn(),
}));
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: vi.fn(async () => ({
		items: [
			{
				id: 1,
				name: 'Dubarde SA',
				isSupplier: true,
				addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
			},
		],
		total: 1,
		limit: 200,
		offset: 0,
	})),
	getContact: vi.fn(),
	createContact: vi.fn(),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: vi.fn(async () => [
		{ id: 40, number: '4000', name: 'Achats', accountType: 'Expense', active: true, postable: true },
	]),
}));
vi.mock('$lib/features/vat-rates/vat-rates.api', () => ({ listVatRates: vi.fn(async () => []) }));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: vi.fn(async () => []) }));

import Page from './+page.svelte';

const QR: SupplierInvoiceSourceQr = {
	creditorIban: 'CH4431999123000889012',
	isQrIban: true,
	referenceType: 'QRR',
	referenceValue: '210000000003139471430009017',
	amount: '100.00',
	currency: 'CHF',
	creditorName: 'Dubarde SA',
	creditorAddressType: 'S',
	creditorLine1: 'Rue du Lac',
	creditorLine2: '12',
	creditorPostalCode: '1000',
	creditorTown: 'Lausanne',
	creditorCountry: 'CH',
};

function doc(over: Partial<SupplierInvoiceSourceDocument>): SupplierInvoiceSourceDocument {
	return {
		importedSupplierInvoiceId: 7,
		originalFilename: 'facture.pdf',
		mimeType: 'application/pdf',
		byteSize: 10,
		linkOrigin: 'completion',
		qr: QR,
		...over,
	};
}

function draft(partial: Partial<SupplierInvoiceResponse> = {}): SupplierInvoiceResponse {
	return {
		id: 9,
		contactId: 1,
		supplierInvoiceNumber: 'FF-9',
		status: 'draft',
		invoiceDate: '2026-05-10',
		dueDate: null,
		totalAmount: '100.00',
		creditorIban: 'CH9300762011623852957',
		creditorQrIban: null,
		paymentReference: null,
		expectedPaymentAmount: null,
		projectId: null,
		purchaseJournalEntryId: null,
		settlementType: null,
		settlementBankAccountId: null,
		settlementAccountId: null,
		settlementJournalEntryId: null,
		paidAt: null,
		version: 5,
		createdAt: '2026-05-10T00:00:00',
		lines: [
			{
				position: 0,
				description: 'Fourniture',
				quantity: '1.0000',
				unitPrice: '100.0000',
				vatRate: '0.00',
				lineTotal: '100.00',
				expenseAccountId: 40,
			},
		],
		settlementCancellable: false,
		settlementCancelBlockedBy: 'SUPPLIER_INVOICE_NOT_PAID',
		settlementCancelBlockedLabel: null,
		lastConfirmedBatch: null,
		cancellable: false,
		cancelBlockedBy: 'SUPPLIER_INVOICE_DRAFT',
		cancelBlockedLabel: null,
		sourceDocument: null,
		...partial,
	};
}

beforeEach(() => {
	getMock.mockReset();
	updateMock.mockReset();
});
afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('page d’édition d’un brouillon', () => {
	it('une facture qui n’est pas brouillon renvoie à sa fiche, avec un message', async () => {
		getMock.mockResolvedValue(draft({ status: 'open', purchaseJournalEntryId: 41 }));
		render(Page);
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/supplier-invoices/9'));
		expect(notifyWarning).toHaveBeenCalled();
	});

	it('sans document : formulaire pré-rempli, scan offert ; Enregistrer envoie le PUT avec la version lue', async () => {
		getMock.mockResolvedValue(draft());
		updateMock.mockResolvedValue(draft({ version: 6 }));
		const { findByTestId, getByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(getByTestId('supplier-invoice-scan-btn')).not.toBeNull();
		expect((getByTestId('supplier-invoice-line-unit-price') as HTMLInputElement).value).toBe('100.00');
		await fireEvent.input(getByTestId('supplier-invoice-line-unit-price'), {
			target: { value: '99,95' },
		});
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(updateMock).toHaveBeenCalledTimes(1));
		const [id, req] = updateMock.mock.calls[0];
		expect(id).toBe(9);
		expect(req.version).toBe(5);
		expect(req.lines[0].unitPrice).toBe('99.95');
		expect(req.creditorIban).toBe('CH9300762011623852957');
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/supplier-invoices/9'));
		expect(notifySuccess).toHaveBeenCalled();
	});

	it('née de la complétion d’un document à QR : coordonnées du QR en lecture seule, aucun envoi de coordonnées, pas de scan', async () => {
		getMock.mockResolvedValue(
			draft({
				creditorIban: null,
				creditorQrIban: QR.creditorIban,
				paymentReference: QR.referenceValue,
				sourceDocument: doc({ linkOrigin: 'completion' }),
			}),
		);
		updateMock.mockResolvedValue(draft());
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-qr-locked');
		expect((getByTestId('supplier-invoice-iban') as HTMLInputElement).readOnly).toBe(true);
		expect((getByTestId('supplier-invoice-iban') as HTMLInputElement).value).toBe(QR.creditorIban);
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(updateMock).toHaveBeenCalledTimes(1));
		const req = updateMock.mock.calls[0][1];
		for (const k of ['creditorIban', 'creditorQrIban', 'paymentReference', 'expectedPaymentAmount']) {
			expect(k in req, k).toBe(false);
		}
	});

	it('document joint après coup, même avec QR : saisie libre, sans mode QR (M34) ni scan (M35)', async () => {
		getMock.mockResolvedValue(draft({ sourceDocument: doc({ linkOrigin: 'attachment' }) }));
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(queryByTestId('supplier-invoice-qr-locked')).toBeNull();
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
		const iban = getByTestId('supplier-invoice-iban') as HTMLInputElement;
		expect(iban.readOnly).toBe(false);
		expect(iban.value).toBe('CH9300762011623852957');
	});

	it('lien antérieur à la 0.17.0 (`linkOrigin` nul) : saisie libre, sans scan', async () => {
		getMock.mockResolvedValue(draft({ sourceDocument: doc({ linkOrigin: null }) }));
		const { findByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(queryByTestId('supplier-invoice-qr-locked')).toBeNull();
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
	});

	it('née d’un document SANS QR : tout se saisit, sans scan (M35)', async () => {
		getMock.mockResolvedValue(
			draft({ sourceDocument: doc({ linkOrigin: 'completion', qr: null }) }),
		);
		const { findByTestId, getByTestId, queryByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		expect(queryByTestId('supplier-invoice-qr-locked')).toBeNull();
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
		expect((getByTestId('supplier-invoice-iban') as HTMLInputElement).readOnly).toBe(false);
		// La date de la facture reste celle du brouillon (l'initial prime).
		expect((getByTestId('supplier-invoice-date') as HTMLInputElement).value).toBe('2026-05-10');
	});

	it('un conflit de version laisse la saisie et invite à recharger', async () => {
		getMock.mockResolvedValue(draft());
		updateMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'Conflit de version',
		});
		const { findByTestId, getByTestId } = render(Page);
		await findByTestId('supplier-invoice-form');
		await fireEvent.input(getByTestId('supplier-invoice-line-unit-price'), {
			target: { value: '80.00' },
		});
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() =>
			expect(getByTestId('supplier-invoice-form-error').textContent).toMatch(/rechargez la page/),
		);
		expect((getByTestId('supplier-invoice-line-unit-price') as HTMLInputElement).value).toBe('80.00');
		expect(gotoMock).not.toHaveBeenCalled();
	});
});
