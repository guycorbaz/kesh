// Story 30-2b (#685, AC11, AC12, AC14 à AC16) — le formulaire de facture fournisseur.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape. i18nMsg rend son repli, variables
// substituées : les textes vérifiés sont ceux du repli français (= catalogue fr-CH).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';
import type { ImportedSupplierInvoice } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';
import type { CreateSupplierInvoiceRequest } from './supplier-invoices.types';
import type { SupplierProposal } from './supplier-proposals';
import type { SupplierInvoiceFormInitial } from './supplier-invoice-form';

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
import { todayLocal } from '$lib/features/open-items/open-items';

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
	initial?: SupplierInvoiceFormInitial | null;
	hasDocument?: boolean;
	vatRates?: { id: number; rate: string }[];
}) {
	const onSubmit = vi.fn(opts.onSubmit ?? (async () => {}));
	const r = render(SupplierInvoiceForm, {
		props: {
			suppliers: opts.suppliers ?? [contact(1, 'Alpha SA')],
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			expenseAccounts: [ACCOUNT as any],
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			vatRates: (opts.vatRates ?? []) as any,
			projects: [],
			document: opts.document ?? null,
			proposals: opts.proposals ?? [],
			onSubmit,
			// Story v017-4b : props neuves, passées seulement quand le test les donne
			// (leurs défauts — `null`, `false` — sont ce que la page de création passe).
			...(opts.initial !== undefined ? { initial: opts.initial } : {}),
			...(opts.hasDocument !== undefined ? { hasDocument: opts.hasDocument } : {}),
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

/**
 * Story v017-4b (#853, AC-24 (b)) — avec un document, la date de facture démarre VIDE et
 * se saisit : les tests qui enregistrent à côté d'un document la posent d'abord.
 */
async function setDate(container: HTMLElement, value = '2026-10-15') {
	const d = container.querySelector('[data-testid="supplier-invoice-date"]') as HTMLInputElement;
	await fireEvent.input(d, { target: { value } });
}

async function setDue(container: HTMLElement, value: string) {
	const d = container.querySelector('[data-testid="supplier-invoice-due"]') as HTMLInputElement;
	await fireEvent.input(d, { target: { value } });
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
		await setDate(container);
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
		await setDate(container);
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
		await setDate(container);
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
		await setDate(container);
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
		await setDate(container);
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

/**
 * Appuie sur Entrée dans `el` comme le ferait un navigateur : `keydown` annulable, puis,
 * s'il n'a pas été annulé, la soumission implicite du formulaire qui contient le champ.
 * jsdom n'implémente pas cette soumission implicite : sans ce modèle, aucun test ne
 * verrait la facture partir.
 */
async function pressEnter(el: HTMLInputElement) {
	const ev = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true });
	el.dispatchEvent(ev);
	if (!ev.defaultPrevented) el.form?.requestSubmit();
	await tick();
}

describe('SupplierInvoiceForm — Entrée dans le panneau « Nouveau fournisseur » (revue P1, B1 = E1 = A1)', () => {
	it('Entrée dans CHAQUE champ du panneau crée le fournisseur et ne soumet jamais la facture (M-b20)', async () => {
		// Le pire cas : un fournisseur est déjà sélectionné et les lignes sont valides —
		// une soumission partirait sur ce fournisseur-là.
		createContactMock.mockRejectedValue({ code: 'VALIDATION_ERROR', status: 400, message: 'refus' });
		const { container, getByTestId, onSubmit } = renderForm({ document: qrDoc() });
		// Story v017-4b : la date posée, sans quoi la facture ne partirait jamais et le
		// test passerait à vide.
		await setDate(container);
		await fillLine(container, '100.00');
		await selectSupplier(container, '1');
		await fireEvent.click(getByTestId('supplier-new-open'));
		const panel = getByTestId('supplier-new-panel');
		const visit = async () => {
			const inputs = Array.from(panel.querySelectorAll('input'));
			expect(inputs.length).toBeGreaterThan(0);
			for (const input of inputs) {
				createContactMock.mockClear();
				await pressEnter(input);
				await waitFor(() => expect(createContactMock).toHaveBeenCalledTimes(1));
				expect(onSubmit).not.toHaveBeenCalled();
			}
			return inputs.length;
		};
		// Entreprise : nom, rue, N°, NPA, localité, pays.
		expect(await visit()).toBe(6);
		// Personne : prénom et nom de famille en plus de l'adresse.
		const type = getByTestId('supplier-new-type') as HTMLSelectElement;
		type.value = 'Personne';
		await fireEvent.change(type);
		const firstLast = Array.from(panel.querySelectorAll('input')).slice(0, 2);
		await fireEvent.input(firstLast[0], { target: { value: 'Jean' } });
		await fireEvent.input(firstLast[1], { target: { value: 'Dupont' } });
		expect(await visit()).toBe(7);
		expect(onSubmit).not.toHaveBeenCalled();
	});

	it('Entrée dans le nom crée et sélectionne le fournisseur, sans enregistrer la facture', async () => {
		createContactMock.mockResolvedValue(contact(9, 'Dubarde SA'));
		const { container, getByTestId, queryByTestId, onSubmit } = renderForm({ document: qrDoc() });
		await setDate(container);
		await fillLine(container, '100.00');
		await selectSupplier(container, '1');
		await fireEvent.click(getByTestId('supplier-new-open'));
		await pressEnter(getByTestId('supplier-new-name') as HTMLInputElement);
		await waitFor(() => expect(queryByTestId('supplier-new-panel')).toBeNull());
		expect(createContactMock).toHaveBeenCalledTimes(1);
		expect(onSubmit).not.toHaveBeenCalled();
		expect((getByTestId('supplier-invoice-supplier') as HTMLSelectElement).value).toBe('9');
	});

	it('témoin du modèle : Entrée dans un champ de la facture la soumet bien', async () => {
		// Sans ce témoin, `pressEnter` pourrait ne rien soumettre du tout et les deux
		// tests ci-dessus passeraient à vide.
		const { container, getByTestId, onSubmit } = renderForm({ document: qrDoc() });
		await setDate(container);
		await fillLine(container, '100.00');
		await selectSupplier(container, '1');
		await pressEnter(getByTestId('supplier-invoice-number') as HTMLInputElement);
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
	});
});

describe('SupplierInvoiceForm — une ligne au centime (Story v017-3b, #687, AC-12)', () => {
	const DEDIE =
		'Une ligne n’est pas au centime : saisissez-la au montant facturé par le fournisseur, au centime.';

	async function setQty(container: HTMLElement, value: string) {
		const qty = container.querySelector('[data-testid="supplier-invoice-line-qty"]') as HTMLInputElement;
		await fireEvent.input(qty, { target: { value } });
	}

	it('affiche le message de ligne sous une ligne qui n’est pas au centime', async () => {
		const { container, getByTestId, queryByTestId } = renderForm({});
		await fillLine(container, '33.3333');
		await setQty(container, '3');
		expect(getByTestId('supplier-invoice-line-subcentime').textContent?.trim()).toBe(
			'Le montant de la ligne (99.9999) n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.',
		);
		await setQty(container, '1');
		await fillLine(container, '99.99');
		expect(queryByTestId('supplier-invoice-line-subcentime')).toBeNull();
	});

	it('au clic, affiche le message DÉDIÉ et n’envoie rien (M8, M12) ; corrigée, la saisie part', async () => {
		const { container, getByTestId, onSubmit } = renderForm({});
		await selectSupplier(container, '1');
		await fillLine(container, '33.3333');
		await setQty(container, '3');
		expect((getByTestId('supplier-invoice-submit') as HTMLButtonElement).disabled).toBe(false);
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await tick();
		expect(getByTestId('supplier-invoice-form-error').textContent?.trim()).toBe(DEDIE);
		expect(onSubmit).not.toHaveBeenCalled();

		await setQty(container, '1');
		await fillLine(container, '99.99');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
	});

	it('une ligne à la fois incomplète et sous le centime reçoit le message générique', async () => {
		const { container, getByTestId, onSubmit } = renderForm({});
		await selectSupplier(container, '1');
		await fillLine(container, '33.3333');
		await setQty(container, '3');
		await fireEvent.input(getByTestId('supplier-invoice-line-desc'), { target: { value: '' } });
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await tick();
		expect(getByTestId('supplier-invoice-form-error').textContent?.trim()).toBe(
			'Chaque ligne requiert une description, un montant et un compte de charge.',
		);
		expect(onSubmit).not.toHaveBeenCalled();
	});

	it('un refus serveur désigne sa ligne, traduit, sans perdre la saisie', async () => {
		const { container, getByTestId, getAllByTestId } = renderForm({
			onSubmit: async () => {
				throw {
					code: 'SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES',
					status: 400,
					message: 'serveur',
					details: { line: 1, lineTotal: '0.005' },
				};
			},
		});
		await selectSupplier(container, '1');
		await fillLine(container, '10.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() =>
			expect(getAllByTestId('supplier-invoice-line-server-error')[0].textContent?.trim()).toBe(
				'La ligne 1 donne un montant de 0.005, qui n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.',
			),
		);
		expect((getByTestId('supplier-invoice-line-unit-price') as HTMLInputElement).value).toBe('10.00');
	});
});

// ---------------------------------------------------------------------------
// Story v017-4b (#849, #853, #830, #835 ; AC-23, AC-24)
// ---------------------------------------------------------------------------

/** Ce que la page d'édition passe au formulaire : la facture relue (brouillon). */
function initial(over: Partial<SupplierInvoiceFormInitial> = {}): SupplierInvoiceFormInitial {
	return {
		contactId: 1,
		supplierInvoiceNumber: 'FF-1',
		invoiceDate: '2026-05-10',
		dueDate: '2026-06-09',
		creditorIban: 'CH9300762011623852957',
		creditorQrIban: null,
		paymentReference: null,
		expectedPaymentAmount: null,
		projectId: null,
		lines: [
			{
				position: 0,
				description: 'Fourniture',
				quantity: '1.0000',
				unitPrice: '100.2000',
				vatRate: '0.00',
				lineTotal: '100.20',
				expenseAccountId: 40,
			},
		],
		...over,
	};
}

describe('SupplierInvoiceForm — la date de facture (#853, AC-24 (b))', () => {
	// v0172 (#885) : avec un document, la date ne démarre plus vide — elle vaut la date du
	// jour, signalée « à vérifier » (C-v0172b-15). Le test de #853 (« démarre VIDE ») est
	// remplacé par AC-B9 ci-dessous ; ne reste ici que le cas sans document.
	it('sans document, la date du jour, sans aide ni avis (M23)', () => {
		const without = renderForm({ document: null });
		expect((without.getByTestId('supplier-invoice-date') as HTMLInputElement).value).toBe(
			todayLocal(),
		);
		expect(without.queryByTestId('supplier-invoice-date-help')).toBeNull();
		expect(without.queryByTestId('supplier-invoice-date-to-verify')).toBeNull();
	});

	it('une date vide a son propre message, sous le champ, et rien ne part', async () => {
		const { container, getByTestId, queryByTestId, onSubmit } = renderForm({ document: qrDoc() });
		await selectSupplier(container, '1');
		// v0172 (#885) : la date démarre au jour même ; on la vide.
		await setDate(container, '');
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await tick();
		expect(getByTestId('supplier-invoice-date-error').textContent?.trim()).toBe(
			'Saisissez la date de la facture, telle qu’imprimée sur le document.',
		);
		// Pas le message des lignes, qui parlait d'autre chose.
		expect(queryByTestId('supplier-invoice-form-error')).toBeNull();
		expect(onSubmit).not.toHaveBeenCalled();
	});

	it('une échéance antérieure à la date se signale sous le champ et bloque l’envoi (M24)', async () => {
		const { container, getByTestId, queryByTestId, onSubmit } = renderForm({ document: null });
		await selectSupplier(container, '1');
		await setDate(container, '2026-05-10');
		await fillLine(container, '100.00');
		await setDue(container, '2026-05-09');
		expect(getByTestId('supplier-invoice-due-error').textContent?.trim()).toBe(
			"L'échéance (09.05.2026) précède la date de la facture (10.05.2026).",
		);
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await tick();
		expect(onSubmit).not.toHaveBeenCalled();
		// Égale : elle passe.
		await setDue(container, '2026-05-10');
		expect(queryByTestId('supplier-invoice-due-error')).toBeNull();
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		expect(onSubmit.mock.calls[0][0].dueDate).toBe('2026-05-10');
	});

	it('le refus serveur de l’échéance s’affiche au même endroit, saisie intacte', async () => {
		const { container, getByTestId } = renderForm({
			document: null,
			onSubmit: async () => {
				throw {
					code: 'SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE',
					status: 400,
					message: 'serveur',
					details: { invoiceDate: '2026-05-10', dueDate: '2026-05-01' },
				};
			},
		});
		await selectSupplier(container, '1');
		await fillLine(container, '10.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() =>
			expect(getByTestId('supplier-invoice-due-error').textContent?.trim()).toBe(
				"L'échéance (01.05.2026) précède la date de la facture (10.05.2026).",
			),
		);
		expect((getByTestId('supplier-invoice-line-unit-price') as HTMLInputElement).value).toBe('10.00');
	});
});

describe('SupplierInvoiceForm — la virgule décimale (#830, AC-24 (c))', () => {
	it('quantité, prix et montant attendu partent avec un point (M25, M32)', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: null });
		await selectSupplier(container, '1');
		await fillLine(container, '12,50');
		await fireEvent.input(getByTestId('supplier-invoice-line-qty'), { target: { value: '2,5' } });
		await fireEvent.input(getByTestId('supplier-invoice-expected-amount'), {
			target: { value: '31,25' },
		});
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		const req = onSubmit.mock.calls[0][0];
		expect(req.lines[0].unitPrice).toBe('12.50');
		expect(req.lines[0].quantity).toBe('2.5');
		expect(req.expectedPaymentAmount).toBe('31.25');
	});

	it('une quantité vide vaut 1 au total vivant comme à l’envoi', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: null });
		await selectSupplier(container, '1');
		await fillLine(container, '12,50');
		await fireEvent.input(getByTestId('supplier-invoice-line-qty'), { target: { value: '' } });
		expect(getByTestId('supplier-invoice-sum-ttc').textContent).toContain('12.50');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		expect(onSubmit.mock.calls[0][0].lines[0].quantity).toBe('1');
	});

	it('un prix vide est refusé par le message des lignes, rien ne part (M33)', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: null });
		await selectSupplier(container, '1');
		await fillLine(container, '');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await tick();
		expect(getByTestId('supplier-invoice-form-error').textContent?.trim()).toBe(
			'Chaque ligne requiert une description, un montant et un compte de charge.',
		);
		expect(onSubmit).not.toHaveBeenCalled();
	});
});

describe('SupplierInvoiceForm — le total vivant aux décimales exactes (#835, AC-24 (d))', () => {
	it('une ligne à 99.9999 se lit 99.9999, sous son message de refus (M26)', async () => {
		const { container, getByTestId } = renderForm({ document: null });
		await fillLine(container, '33.3333');
		await fireEvent.input(getByTestId('supplier-invoice-line-qty'), { target: { value: '3' } });
		expect(getByTestId('supplier-invoice-sum-ttc').textContent).toContain('99.9999');
		expect(getByTestId('supplier-invoice-line-subcentime')).not.toBeNull();
	});

	it('le montant du QR s’affiche aux décimales exactes', () => {
		const { getByTestId } = renderForm({ document: qrDoc({ amount: '100.0050' }) });
		expect((getByTestId('supplier-invoice-expected-amount') as HTMLInputElement).value).toContain(
			'100.0050',
		);
	});
});

describe('SupplierInvoiceForm — le bouton d’enregistrement (AC-24 (a))', () => {
	it('« Enregistrer le brouillon » à la création, « Enregistrer » à l’édition', () => {
		const create = renderForm({ document: null });
		expect(create.getByTestId('supplier-invoice-submit').textContent?.trim()).toBe(
			'Enregistrer le brouillon',
		);
		cleanup();
		const edit = renderForm({ document: null, initial: initial() });
		expect(edit.getByTestId('supplier-invoice-submit').textContent?.trim()).toBe('Enregistrer');
	});
});

describe('SupplierInvoiceForm — l’édition d’un brouillon (AC-23)', () => {
	it('pré-remplit fournisseur, numéro, dates, coordonnées et lignes', () => {
		const { getByTestId } = renderForm({ document: null, initial: initial() });
		expect((getByTestId('supplier-invoice-supplier') as HTMLSelectElement).value).toBe('1');
		expect((getByTestId('supplier-invoice-number') as HTMLInputElement).value).toBe('FF-1');
		expect((getByTestId('supplier-invoice-date') as HTMLInputElement).value).toBe('2026-05-10');
		expect((getByTestId('supplier-invoice-due') as HTMLInputElement).value).toBe('2026-06-09');
		expect((getByTestId('supplier-invoice-iban') as HTMLInputElement).value).toBe(
			'CH9300762011623852957',
		);
		expect((getByTestId('supplier-invoice-line-desc') as HTMLInputElement).value).toBe('Fourniture');
		expect((getByTestId('supplier-invoice-line-qty') as HTMLInputElement).value).toBe('1');
		expect((getByTestId('supplier-invoice-line-unit-price') as HTMLInputElement).value).toBe('100.20');
	});

	it('la date de l’initial prime sur la règle « date vide avec un document »', () => {
		const { getByTestId } = renderForm({ document: qrDoc(), initial: initial() });
		expect((getByTestId('supplier-invoice-date') as HTMLInputElement).value).toBe('2026-05-10');
	});

	it('le corps reprend la saisie corrigée, coordonnées comprises', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: null, initial: initial() });
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		const req = onSubmit.mock.calls[0][0];
		expect(req.contactId).toBe(1);
		expect(req.supplierInvoiceNumber).toBe('FF-1');
		expect(req.invoiceDate).toBe('2026-05-10');
		expect(req.creditorIban).toBe('CH9300762011623852957');
		expect(req.lines).toHaveLength(1);
		expect(req.lines[0].unitPrice).toBe('100.00');
		expect(req.lines[0].vatRate).toBe('0');
	});

	it('scan offert ssi `document === null && !hasDocument` (M35)', () => {
		const sans = renderForm({ document: null, initial: initial(), hasDocument: false });
		expect(sans.queryByTestId('supplier-invoice-scan-btn')).not.toBeNull();
		cleanup();
		const avec = renderForm({ document: null, initial: initial(), hasDocument: true });
		expect(avec.queryByTestId('supplier-invoice-scan-btn')).toBeNull();
		// Et saisie libre : les coordonnées restent modifiables.
		expect((avec.getByTestId('supplier-invoice-iban') as HTMLInputElement).readOnly).toBe(false);
	});
});

describe('SupplierInvoiceForm — revue de code P1 (B-8, B-9 = E-5, E-6, E-7)', () => {
	it('sans document, une date vide a un message qui ne parle pas d’un document (E-6)', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: null });
		await selectSupplier(container, '1');
		await setDate(container, '');
		await fillLine(container, '10.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await tick();
		expect(getByTestId('supplier-invoice-date-error').textContent?.trim()).toBe(
			'Saisissez la date de la facture.',
		);
		expect(onSubmit).not.toHaveBeenCalled();
	});

	it('le montant attendu du brouillon se pré-remplit lisible (B-8)', () => {
		const { getByTestId } = renderForm({
			document: null,
			initial: initial({ expectedPaymentAmount: '100.2000' }),
		});
		expect((getByTestId('supplier-invoice-expected-amount') as HTMLInputElement).value).toBe(
			'100.20',
		);
	});

	it('corriger la date de facture efface le refus serveur de l’échéance (B-9 = E-5)', async () => {
		const { container, getByTestId, queryByTestId } = renderForm({
			document: null,
			onSubmit: async () => {
				throw {
					code: 'SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE',
					status: 400,
					message: 'serveur',
					details: { invoiceDate: '2026-05-10', dueDate: '2026-05-01' },
				};
			},
		});
		await selectSupplier(container, '1');
		await fillLine(container, '10.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(getByTestId('supplier-invoice-due-error')).not.toBeNull());
		await setDate(container, '2026-04-30');
		expect(queryByTestId('supplier-invoice-due-error')).toBeNull();
	});

	it('le taux d’une ligne absent des taux actifs reste proposé et choisi (E-7)', () => {
		const { getByTestId } = renderForm({
			document: null,
			vatRates: [{ id: 1, rate: '8.10' }],
			initial: initial({
				lines: [
					{
						position: 0,
						description: 'Ancienne',
						quantity: '1.0000',
						unitPrice: '10.0000',
						vatRate: '7.70',
						lineTotal: '10.00',
						expenseAccountId: 40,
					},
				],
			}),
		});
		const sel = getByTestId('supplier-invoice-line-vat') as HTMLSelectElement;
		expect([...sel.options].map((o) => o.value)).toContain('7.70');
		expect(sel.value).toBe('7.70');
	});

	// Revue P2 (E2-5) : un taux expiré à l'échelle du serveur s'affiche « 7.70% », sa valeur
	// reste celle du serveur.
	it('le taux expiré à quatre décimales s’affiche à deux (E2-5)', () => {
		const { getByTestId } = renderForm({
			document: null,
			vatRates: [{ id: 1, rate: '8.10' }],
			initial: initial({
				lines: [
					{
						position: 0,
						description: 'Ancienne',
						quantity: '1.0000',
						unitPrice: '10.0000',
						vatRate: '7.7000',
						lineTotal: '10.00',
						expenseAccountId: 40,
					},
				],
			}),
		});
		const sel = getByTestId('supplier-invoice-line-vat') as HTMLSelectElement;
		const opt = [...sel.options].find((o) => o.value === '7.7000');
		expect(opt?.textContent?.trim()).toBe('7.70%');
		expect(sel.value).toBe('7.7000');
	});
});

// v0172-correctifs (#885, C-v0172b-15, C-v0172b-16) — à côté d'un document, la date est
// proposée au jour même (date LOCALE) et marquée « à vérifier » ; l'échéance suit la date,
// plus le délai du fournisseur, tant qu'elle n'a pas été saisie.
describe('SupplierInvoiceForm — date proposée et échéance qui suit (v0172, #885)', () => {
	let tz: string | undefined;
	beforeEach(() => {
		// Fuseau épinglé ET horloge fixée : 2026-11-04T23:30Z = le 5 novembre, 00:30 à
		// Zurich ; sous `toISOString`, la date serait le 4 (MB5, sonde MB10).
		tz = process.env.TZ;
		process.env.TZ = 'Europe/Zurich';
		vi.useFakeTimers({ toFake: ['Date'] });
		vi.setSystemTime(new Date('2026-11-04T23:30:00Z'));
	});
	afterEach(() => {
		vi.useRealTimers();
		if (tz === undefined) delete process.env.TZ;
		else process.env.TZ = tz;
	});

	function withTerms(id: number, name: string, days: number | null): ContactResponse {
		return { ...contact(id, name), defaultPaymentTermsDays: days };
	}
	const dateInput = (c: HTMLElement) =>
		c.querySelector('[data-testid="supplier-invoice-date"]') as HTMLInputElement;
	const dueInput = (c: HTMLElement) =>
		c.querySelector('[data-testid="supplier-invoice-due"]') as HTMLInputElement;

	it('AC-B9 — avec un document : la date LOCALE du jour (2026-11-05), l’avis « à vérifier » visible (MB5)', () => {
		expect(new Date().toISOString().slice(0, 10), 'montage : la date UTC est la veille').toBe('2026-11-04');
		const { container, getByTestId } = renderForm({ document: qrDoc() });
		expect(dateInput(container).value).toBe('2026-11-05');
		const notice = getByTestId('supplier-invoice-date-to-verify');
		expect(notice.textContent).toContain(
			'Date du jour, proposée par défaut — ce n’est pas celle du document. Vérifiez-la.',
		);
		expect(dateInput(container).getAttribute('aria-describedby')).toBe(notice.id);
		expect(getByTestId('supplier-invoice-date-confirm').textContent?.trim()).toBe('C’est la bonne date');
	});

	it('AC-B9 — sans document : 2026-11-05 et aucun avis ; à l’édition : la date de `initial`, aucun avis', () => {
		const a = renderForm({ document: null });
		expect(dateInput(a.container).value).toBe('2026-11-05');
		expect(a.queryByTestId('supplier-invoice-date-to-verify')).toBeNull();
		cleanup();
		const b = renderForm({
			document: qrDoc(),
			initial: initial({ invoiceDate: '2026-06-01', dueDate: '2026-06-30' }),
		});
		expect(dateInput(b.container).value).toBe('2026-06-01');
		expect(dueInput(b.container).value).toBe('2026-06-30');
		expect(b.queryByTestId('supplier-invoice-date-to-verify')).toBeNull();
	});

	it('AC-B10 — l’avis disparaît après une saisie dans la date, et ne revient pas (MB7)', async () => {
		const { container, queryByTestId } = renderForm({ document: qrDoc() });
		await setDate(container, '2026-10-20');
		expect(queryByTestId('supplier-invoice-date-to-verify')).toBeNull();
		await setDate(container, '2026-11-05');
		expect(queryByTestId('supplier-invoice-date-to-verify')).toBeNull();
	});

	it('AC-B10 — « C’est la bonne date » retire l’avis, la date reste', async () => {
		const { container, getByTestId, queryByTestId } = renderForm({ document: qrDoc() });
		await fireEvent.click(getByTestId('supplier-invoice-date-confirm'));
		expect(queryByTestId('supplier-invoice-date-to-verify')).toBeNull();
		expect(dateInput(container).value).toBe('2026-11-05');
	});

	it('AC-B11 — fournisseur à 30 jours : échéance 2026-12-05, puis 2026-12-10 quand la date passe au 2026-11-10', async () => {
		const { container } = renderForm({
			document: qrDoc(),
			suppliers: [withTerms(1, 'Alpha SA', 30), withTerms(2, 'Beta SA', 10)],
		});
		await selectSupplier(container, '1');
		expect(dueInput(container).value).toBe('2026-12-05');
		await setDate(container, '2026-11-10');
		expect(dueInput(container).value).toBe('2026-12-10');
		// Changer de fournisseur (30 → 10 jours) recalcule tant que l'échéance suit.
		await selectSupplier(container, '2');
		expect(dueInput(container).value).toBe('2026-11-20');
	});

	it('AC-B11 — fournisseur sans délai (ou aucun) : échéance = date, et elle suit la date ; date vidée ⇒ échéance vide', async () => {
		const { container } = renderForm({ document: qrDoc(), suppliers: [withTerms(1, 'Alpha SA', null)] });
		expect(dueInput(container).value, 'aucun fournisseur choisi').toBe('2026-11-05');
		await selectSupplier(container, '1');
		expect(dueInput(container).value).toBe('2026-11-05');
		await setDate(container, '2026-11-12');
		expect(dueInput(container).value).toBe('2026-11-12');
		await setDate(container, '');
		expect(dueInput(container).value).toBe('');
	});

	it('AC-B11 — échéance saisie à la main : changer la date ou le fournisseur ne la touche plus (MB6)', async () => {
		const { container } = renderForm({
			document: qrDoc(),
			suppliers: [withTerms(1, 'Alpha SA', 30), withTerms(2, 'Beta SA', 10)],
		});
		await selectSupplier(container, '1');
		// 2026-11-23 : aucune des valeurs que le calcul produirait ici (12-05, 12-10, 11-20),
		// sans quoi le test passerait sous MB6 par coïncidence (constaté à la première
		// rédaction, qui saisissait 2026-11-20 = 11-10 + 10 jours).
		await setDue(container, '2026-11-23');
		await setDate(container, '2026-11-10');
		expect(dueInput(container).value).toBe('2026-11-23');
		await selectSupplier(container, '2');
		expect(dueInput(container).value).toBe('2026-11-23');
	});

	it('AC-B12 — #853 inchangé : échéance 2026-11-01 pour une date 2026-11-05 ⇒ message, rien ne part', async () => {
		const { container, getByTestId, onSubmit } = renderForm({ document: qrDoc() });
		await selectSupplier(container, '1');
		await fillLine(container, '100.00');
		await setDue(container, '2026-11-01');
		expect(getByTestId('supplier-invoice-due-error').textContent?.trim()).toBe(
			"L'échéance (01.11.2026) précède la date de la facture (05.11.2026).",
		);
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await tick();
		expect(onSubmit).not.toHaveBeenCalled();
	});

	it('AC-B12 — sans rien toucher, la requête porte la date proposée et l’échéance calculée (date non confirmée : pas de blocage)', async () => {
		const { container, getByTestId, onSubmit } = renderForm({
			document: qrDoc(),
			suppliers: [withTerms(1, 'Alpha SA', 30)],
		});
		await selectSupplier(container, '1');
		await fillLine(container, '100.00');
		await fireEvent.click(getByTestId('supplier-invoice-submit'));
		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
		expect(onSubmit.mock.calls[0][0]).toMatchObject({ invoiceDate: '2026-11-05', dueDate: '2026-12-05' });
	});
});
