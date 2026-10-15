// Story 30-2b (#685, AC11, AC12, AC14 à AC16) — le formulaire de facture fournisseur.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape. i18nMsg rend son repli, variables
// substituées : les textes vérifiés sont ceux du repli français (= catalogue fr-CH).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor, cleanup } from '@testing-library/svelte';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';
import type { ImportedSupplierInvoice } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
import type { CreateSupplierInvoiceRequest } from './supplier-invoices.types';
import type { SupplierProposal } from './supplier-proposals';

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
vi.mock('./supplier-invoices.api', () => ({ scanQrSupplierInvoice: vi.fn() }));
vi.mock('./supplier-invoice-scan', () => ({ decodeQrFromImageFile: vi.fn(), scanToPrefill: vi.fn() }));
const createContactMock = vi.fn();
const listContactsMock = vi.fn();
vi.mock('$lib/features/contacts/contacts.api', () => ({
	createContact: (req: unknown) => createContactMock(req),
	listContacts: (q: unknown) => listContactsMock(q),
}));

import SupplierInvoiceForm from './SupplierInvoiceForm.svelte';

function contact(id: number, name: string): ContactResponse {
	return {
		id,
		companyId: 1,
		contactType: 'Entreprise',
		name,
		firstName: null,
		lastName: null,
		isClient: false,
		isSupplier: true,
		address: null,
		addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
		email: null,
		phone: null,
		ideNumber: null,
		clientNumber: null,
		defaultPaymentTerms: null,
		defaultPaymentTermsDays: null,
		defaultPaymentTermsLabel: null,
		language: null,
		salutation: 'Neutre',
		active: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
	};
}

function qrDoc(over: Partial<ImportedSupplierInvoice> = {}): ImportedSupplierInvoice {
	return {
		id: 7,
		companyId: 1,
		status: 'to_complete',
		supplierInvoiceId: null,
		fileHash: 'h',
		storagePath: 'h.png',
		originalFilename: 'facture.png',
		mimeType: 'image/png',
		byteSize: 10,
		creditorIban: 'CH9300762011623852957',
		isQrIban: false,
		creditorAddressType: 'S',
		creditorName: 'Dubarde SA',
		creditorLine1: 'Rue du Lac',
		creditorLine2: '12',
		creditorPostalCode: '1000',
		creditorTown: 'Lausanne',
		creditorCountry: 'CH',
		referenceType: 'SCOR',
		referenceValue: 'RF18539007547034',
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

const ACCOUNT = {
	id: 40,
	number: '4000',
	name: 'Achats',
	accountType: 'Expense',
	active: true,
	postable: true,
};

type Submit = (req: CreateSupplierInvoiceRequest) => Promise<void>;

function renderForm(opts: {
	document?: ImportedSupplierInvoice | null;
	proposals?: SupplierProposal[];
	suppliers?: ContactResponse[];
	onSubmit?: Submit;
}) {
	const onSubmit = vi.fn(opts.onSubmit ?? (async () => {}));
	const r = render(SupplierInvoiceForm, {
		props: {
			suppliers: opts.suppliers ?? [contact(1, 'Alpha SA')],
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			expenseAccounts: [ACCOUNT as any],
			vatRates: [],
			projects: [],
			document: opts.document ?? null,
			proposals: opts.proposals ?? [],
			onSubmit,
		},
	});
	return { ...r, onSubmit };
}

/** Remplit la première ligne : description, quantité 1, prix, compte. */
async function fillLine(container: HTMLElement, price: string) {
	const q = (id: string) => container.querySelector(`[data-testid="${id}"]`) as HTMLInputElement;
	await fireEvent.input(q('supplier-invoice-line-desc'), { target: { value: 'Prestation' } });
	await fireEvent.input(q('supplier-invoice-line-unit-price'), { target: { value: price } });
	const account = container.querySelector('[data-testid="supplier-invoice-line-account"]') as HTMLSelectElement;
	account.value = '40';
	await fireEvent.change(account);
}

async function selectSupplier(container: HTMLElement, id: string) {
	const sel = container.querySelector('[data-testid="supplier-invoice-supplier"]') as HTMLSelectElement;
	sel.value = id;
	await fireEvent.change(sel);
}

beforeEach(() => {
	createContactMock.mockReset();
	listContactsMock.mockReset();
	listContactsMock.mockResolvedValue({ items: [], total: 0, limit: 20, offset: 0 });
});
afterEach(() => cleanup());

describe('SupplierInvoiceForm — document avec QR (AC11)', () => {
	it('affiche IBAN, référence et montant du QR en lecture seule, sans scan (M-b8)', () => {
		const { getByTestId, queryByTestId } = renderForm({ document: qrDoc() });
		const iban = getByTestId('supplier-invoice-iban') as HTMLInputElement;
		expect(iban.value).toBe('CH9300762011623852957');
		expect(iban.readOnly).toBe(true);
		expect((getByTestId('supplier-invoice-reference') as HTMLInputElement).readOnly).toBe(true);
		expect((getByTestId('supplier-invoice-expected-amount') as HTMLInputElement).readOnly).toBe(true);
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
	});

	it('n’envoie aucune coordonnée de paiement (le serveur refuserait QR_FIELDS_FROM_DOCUMENT)', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: qrDoc() });
		await selectSupplier(container, '1');
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		const req = onSubmit.mock.calls[0][0];
		for (const k of ['creditorIban', 'creditorQrIban', 'paymentReference', 'expectedPaymentAmount']) {
			expect(k in req, k).toBe(false);
		}
		expect(req.contactId).toBe(1);
	});

	it('compare le total TTC vivant au montant du QR', async () => {
		const { container, queryByTestId } = renderForm({ document: qrDoc() });
		await fillLine(container, '90.00');
		expect(queryByTestId('supplier-invoice-mismatch')).not.toBeNull();
		await fillLine(container, '100.00');
		expect(queryByTestId('supplier-invoice-mismatch')).toBeNull();
	});
});

describe('SupplierInvoiceForm — fournisseur proposé (AC14)', () => {
	it('affiche les propositions avec leur motif, sans en présélectionner aucune (M-b4)', () => {
		const p = contact(5, 'Dubarde SA');
		const { getAllByTestId, getByTestId } = renderForm({
			document: qrDoc(),
			proposals: [{ contact: p, reason: 'iban' }],
		});
		const items = getAllByTestId('supplier-proposal');
		expect(items).toHaveLength(1);
		expect(items[0].textContent).toContain('Proposé : Dubarde SA');
		expect(items[0].textContent).toContain('même IBAN');
		const sel = getByTestId('supplier-invoice-supplier') as HTMLSelectElement;
		expect(sel.value).not.toBe('5');
		expect(sel.selectedIndex).toBe(0);
	});

	it('« Retenir » ajoute le contact à la liste et le sélectionne', async () => {
		const p = contact(5, 'Dubarde SA');
		const { getByTestId, container, onSubmit } = renderForm({
			document: qrDoc(),
			proposals: [{ contact: p, reason: 'name' }],
		});
		await fireEvent.click(getByTestId('supplier-proposal-retain'));
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		expect(onSubmit.mock.calls[0][0].contactId).toBe(5);
	});
});

describe('SupplierInvoiceForm — saisie à la main (AC12)', () => {
	it('propose le scan sans document, et route un QR-IBAN vers creditorQrIban (M-b9)', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: null });
		expect(getByTestId('supplier-invoice-scan-btn')).not.toBeNull();
		await selectSupplier(container, '1');
		await fillLine(container, '10.00');
		await fireEvent.input(getByTestId('supplier-invoice-iban'), {
			target: { value: 'ch44 3199 9123 0008 8901 2' },
		});
		await fireEvent.input(getByTestId('supplier-invoice-reference'), {
			target: { value: '21 00000 00003 13947 14300 09017' },
		});
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		const req = onSubmit.mock.calls[0][0];
		expect(req.creditorQrIban).toBe('CH4431999123000889012');
		expect(req.creditorIban).toBeNull();
		expect(req.paymentReference).toBe('21 00000 00003 13947 14300 09017');
	});

	it('un document sans QR laisse tout saisir, sans scan, et route un IBAN ordinaire vers creditorIban', async () => {
		const { container, getByTestId, queryByTestId, onSubmit } = renderForm({
			document: qrDoc({
				creditorIban: null,
				isQrIban: null,
				creditorAddressType: null,
				creditorName: null,
				creditorCountry: null,
				referenceType: null,
				referenceValue: null,
				amount: null,
				currency: null,
			}),
		});
		expect(queryByTestId('supplier-invoice-scan-btn')).toBeNull();
		expect(queryByTestId('supplier-invoice-qr-locked')).toBeNull();
		await selectSupplier(container, '1');
		await fillLine(container, '10.00');
		await fireEvent.input(getByTestId('supplier-invoice-iban'), {
			target: { value: 'CH93 0076 2011 6238 5295 7' },
		});
		await fireEvent.input(getByTestId('supplier-invoice-expected-amount'), { target: { value: '10.00' } });
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		const req = onSubmit.mock.calls[0][0];
		expect(req.creditorIban).toBe('CH9300762011623852957');
		expect(req.creditorQrIban).toBeNull();
		expect(req.expectedPaymentAmount).toBe('10.00');
	});
});

describe('SupplierInvoiceForm — refus du serveur (AC16)', () => {
	it('affiche le motif sous le champ nommé, saisie et lignes intactes', async () => {
		const refus = {
			code: 'INVALID_PAYMENT_DETAILS',
			status: 400,
			message: 'Coordonnées de paiement invalides.',
			details: { field: 'paymentReference', reason: 'INVALID_QRR' },
		};
		const { container, getByTestId, onSubmit } = renderForm({
			document: null,
			onSubmit: async () => {
				throw refus;
			},
		});
		await selectSupplier(container, '1');
		await fillLine(container, '10.00');
		await fireEvent.input(getByTestId('supplier-invoice-iban'), {
			target: { value: 'CH4431999123000889012' },
		});
		await fireEvent.input(getByTestId('supplier-invoice-reference'), { target: { value: '123' } });
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		await waitFor(() =>
			expect(getByTestId('supplier-invoice-reference-error').textContent).toContain(
				'Référence QRR invalide',
			),
		);
		expect((getByTestId('supplier-invoice-iban') as HTMLInputElement).value).toBe('CH4431999123000889012');
		expect((getByTestId('supplier-invoice-reference') as HTMLInputElement).value).toBe('123');
		expect((getByTestId('supplier-invoice-line-desc') as HTMLInputElement).value).toBe('Prestation');
		expect((getByTestId('supplier-invoice-line-unit-price') as HTMLInputElement).value).toBe('10.00');
	});

	it('un autre refus s’affiche en tête, sans rien perdre (IMPORT_NOT_PENDING_COMPLETION)', async () => {
		const { container, getByTestId } = renderForm({
			document: qrDoc(),
			onSubmit: async () => {
				throw { code: 'IMPORT_NOT_PENDING_COMPLETION', status: 409, message: 'x' };
			},
		});
		await selectSupplier(container, '1');
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() =>
			expect(getByTestId('supplier-invoice-form-error').textContent).toContain('déjà été traité'),
		);
		expect((getByTestId('supplier-invoice-line-desc') as HTMLInputElement).value).toBe('Prestation');
	});
});

describe('SupplierInvoiceForm — nouveau fournisseur (AC15)', () => {
	it('pré-remplit nom et adresse d’un QR de type S, crée, sélectionne, et garde les lignes (M-b6)', async () => {
		createContactMock.mockResolvedValue(contact(9, 'Dubarde SA'));
		const { container, getByTestId, queryByTestId, onSubmit } = renderForm({ document: qrDoc() });
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-new-open'));
		expect((getByTestId('supplier-new-name') as HTMLInputElement).value).toBe('Dubarde SA');
		expect((getByTestId('supplier-new-street') as HTMLInputElement).value).toBe('Rue du Lac');
		expect((getByTestId('supplier-new-postal-code') as HTMLInputElement).value).toBe('1000');
		expect((getByTestId('supplier-new-city') as HTMLInputElement).value).toBe('Lausanne');
		await fireEvent.click(getByTestId('supplier-new-create'));
		await waitFor(() => expect(createContactMock).toHaveBeenCalledTimes(1));
		const req = createContactMock.mock.calls[0][0];
		expect(req.isSupplier).toBe(true);
		expect(req.contactType).toBe('Entreprise');
		expect(req.name).toBe('Dubarde SA');
		await waitFor(() => expect(queryByTestId('supplier-new-panel')).toBeNull());
		expect((getByTestId('supplier-invoice-line-desc') as HTMLInputElement).value).toBe('Prestation');
		expect((getByTestId('supplier-invoice-line-unit-price') as HTMLInputElement).value).toBe('100.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		expect(onSubmit.mock.calls[0][0].contactId).toBe(9);
	});

	it('n’invente pas d’adresse pour un QR de type K', async () => {
		const { getByTestId } = renderForm({ document: qrDoc({ creditorAddressType: 'K' }) });
		await fireEvent.click(getByTestId('supplier-new-open'));
		expect((getByTestId('supplier-new-street') as HTMLInputElement).value).toBe('');
	});

	it('une erreur de création s’affiche dans le panneau, rien n’est perdu ; Annuler ne crée rien', async () => {
		createContactMock.mockRejectedValue({ code: 'CONFLICT', status: 409, message: 'IDE déjà utilisé' });
		const { container, getByTestId, queryByTestId } = renderForm({ document: qrDoc() });
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-new-open'));
		await fireEvent.click(getByTestId('supplier-new-create'));
		await waitFor(() => expect(getByTestId('supplier-new-error').textContent).toContain('IDE déjà utilisé'));
		expect((getByTestId('supplier-new-name') as HTMLInputElement).value).toBe('Dubarde SA');
		expect((getByTestId('supplier-invoice-line-desc') as HTMLInputElement).value).toBe('Prestation');
		createContactMock.mockClear();
		await fireEvent.click(getByTestId('supplier-new-cancel'));
		expect(queryByTestId('supplier-new-panel')).toBeNull();
		expect(createContactMock).not.toHaveBeenCalled();
	});
});
