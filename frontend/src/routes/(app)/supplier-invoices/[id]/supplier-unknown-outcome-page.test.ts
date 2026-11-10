/**
 * La fiche facture fournisseur après une réponse perdue — Story 34-8 (#938, AC-8).
 *
 * Sur les sept gestes de la fiche (payer, annuler le règlement, annuler, valider,
 * repasser en brouillon, joindre et détacher un document), un échec dont l'issue
 * n'est pas connue (`isSettlementOutcomeUnknown` : réponse perdue, 5xx, erreur hors
 * `ApiError`) relit la facture et pose un encadré persistant
 * (`supplier-unknown-outcome`) ; un 4xx garde son traitement, sans relecture. Si la
 * relecture échoue, le second avis : l'écran n'affirme jamais une relecture qui n'a
 * pas eu lieu.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (M6 de la fiche : la fiche relit
 * seulement sur `ApiError`). Patron : `supplier-document-page.test.ts` — mocks
 * hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type { SupplierInvoiceResponse } from '$lib/features/supplier-invoices/supplier-invoices.types';
import type { ImportedSupplierInvoice } from '$lib/features/imported-supplier-invoices/imported-supplier-invoices.types';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({
	page: {
		params: { id: '9' },
		url: new URL('http://localhost/supplier-invoices/9'),
	},
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: vi.fn(),
	notifySuccess: vi.fn(),
	notifyWarning: vi.fn(),
	notifyInfo: vi.fn(),
}));
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return { role: 'Comptable', username: 'c', userId: '1' };
		},
	},
}));

const getMock = vi.fn();
const payMock = vi.fn();
const cancelMock = vi.fn();
const cancelSettlementMock = vi.fn();
const validateMock = vi.fn();
const unvalidateMock = vi.fn();
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	getSupplierInvoice: (id: number) => getMock(id),
	paySupplierInvoice: (id: number, req: unknown) => payMock(id, req),
	cancelSupplierInvoice: (id: number) => cancelMock(id),
	cancelSupplierInvoiceSettlement: (id: number) => cancelSettlementMock(id),
	validateSupplierInvoice: (id: number, v: number) => validateMock(id, v),
	unvalidateSupplierInvoice: (id: number, v: number) => unvalidateMock(id, v),
	deleteSupplierInvoice: vi.fn(),
}));
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: vi.fn(async () => []),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: vi.fn(async () => [
		{ id: 1, number: '1000', name: 'Caisse', active: true, postable: true },
	]),
}));
vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: vi.fn(async () => []),
}));
const attachMock = vi.fn();
const detachMock = vi.fn();
const listImportedMock = vi.fn();
vi.mock('$lib/features/imported-supplier-invoices/imported-supplier-invoices.api', () => ({
	downloadSupplierInvoiceSourceDocument: vi.fn(),
	fetchSupplierInvoiceSourceDocument: vi.fn(),
	attachSourceDocument: (id: number, doc: number) => attachMock(id, doc),
	detachSourceDocument: (id: number) => detachMock(id),
	listImported: (status: string) => listImportedMock(status),
	fetchImportedSourceDocument: vi.fn(),
}));

import Page from './+page.svelte';

const VERIFY = 'L’issue n’est pas connue : la facture a été relue, vérifiez-la avant de réessayer.';
const REREAD_FAILED =
	'L’issue n’est pas connue, et la facture n’a pas pu être relue : rechargez la page avant de réessayer.';

const DOC = {
	importedSupplierInvoiceId: 31,
	originalFilename: 'piece.png',
	mimeType: 'image/png',
	byteSize: 10,
	linkOrigin: 'attachment' as const,
	qr: null,
};

function inv(partial: Partial<SupplierInvoiceResponse> = {}): SupplierInvoiceResponse {
	return {
		id: 9,
		contactId: 1,
		supplierInvoiceNumber: 'FF-9',
		internalNumber: null,
		status: 'open',
		invoiceDate: '2026-06-15',
		dueDate: null,
		totalAmount: '100.00',
		creditorIban: null,
		creditorQrIban: null,
		paymentReference: null,
		expectedPaymentAmount: null,
		projectId: null,
		purchaseJournalEntryId: 4,
		settlementType: null,
		settlementBankAccountId: null,
		settlementAccountId: null,
		settlementJournalEntryId: null,
		paidAt: null,
		version: 3,
		createdAt: '2026-06-15T00:00:00',
		lines: [],
		settlementCancellable: false,
		settlementCancelBlockedBy: null,
		settlementCancelBlockedLabel: null,
		lastConfirmedBatch: null,
		cancellable: true,
		cancelBlockedBy: null,
		cancelBlockedLabel: null,
		sourceDocument: null,
		...partial,
	};
}

const PAID: Partial<SupplierInvoiceResponse> = {
	status: 'paid',
	settlementType: 'internal_account',
	settlementAccountId: 1,
	settlementJournalEntryId: 5,
	paidAt: '2026-06-20T00:00:00',
	settlementCancellable: true,
};

function staged(): ImportedSupplierInvoice {
	return {
		id: 31,
		companyId: 1,
		status: 'to_complete',
		supplierInvoiceId: null,
		fileHash: 'h',
		storagePath: 'h.png',
		originalFilename: 'piece.png',
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
		createdAt: '2026-06-16T08:00:00',
		updatedAt: '2026-06-16T08:00:00',
	};
}

/** Ce dont un geste a besoin du rendu (le type de `render` ne se laisse pas nommer ici). */
interface View {
	findByTestId: (id: string) => Promise<HTMLElement>;
	getByTestId: (id: string) => HTMLElement;
}

/** Un geste de la fiche : son état de départ, l'état relu après succès, l'appel. */
interface Gesture {
	name: string;
	start: SupplierInvoiceResponse;
	written: SupplierInvoiceResponse;
	mock: ReturnType<typeof vi.fn>;
	act: (v: View) => Promise<void>;
	/** Où un 4xx s'affiche. */
	refusal: string;
	/** Le texte attendu d'un 4xx : le message du serveur, sauf au repassage en brouillon. */
	refusalText?: string;
}

const GESTURES: Gesture[] = [
	{
		name: 'payer',
		start: inv(),
		written: inv(PAID),
		mock: payMock,
		refusal: 'supplier-invoice-pay-error',
		act: async (v) => {
			await v.findByTestId('supplier-invoice-pay');
			const radios = document.querySelectorAll('input[type="radio"]');
			await fireEvent.click(radios[1]);
			const select = v.getByTestId('pay-internal-account') as HTMLSelectElement;
			await waitFor(() => expect(select.options.length).toBeGreaterThan(1));
			select.value = select.options[1].value;
			await fireEvent.change(select);
			await fireEvent.click(v.getByTestId('supplier-invoice-pay-submit'));
		},
	},
	{
		name: 'annuler le règlement',
		start: inv(PAID),
		written: inv({ settlementCancelBlockedBy: 'SUPPLIER_INVOICE_NOT_PAID' }),
		mock: cancelSettlementMock,
		refusal: 'supplier-invoice-settlement-cancel-error',
		act: async (v) => {
			await fireEvent.click(await v.findByTestId('supplier-invoice-settlement-cancel'));
		},
	},
	{
		name: 'annuler la facture',
		start: inv(),
		written: inv({ status: 'cancelled', cancellable: false }),
		mock: cancelMock,
		refusal: 'supplier-invoice-cancel-error',
		act: async (v) => {
			await fireEvent.click(await v.findByTestId('supplier-invoice-cancel'));
		},
	},
	{
		name: 'valider',
		start: inv({ status: 'draft', purchaseJournalEntryId: null }),
		written: inv({ version: 4 }),
		mock: validateMock,
		refusal: 'supplier-invoice-validate-error',
		act: async (v) => {
			await fireEvent.click(await v.findByTestId('supplier-invoice-validate'));
			await v.findByTestId('supplier-invoice-validate-dialog');
			await fireEvent.click(v.getByTestId('supplier-invoice-validate-confirm'));
		},
	},
	{
		name: 'repasser en brouillon',
		start: inv(),
		written: inv({ status: 'draft', purchaseJournalEntryId: null, version: 4 }),
		mock: unvalidateMock,
		refusal: 'supplier-invoice-unvalidate-error',
		// `supplierInvoiceUnvalidateMessage` traduit le code ; un code inconnu, son repli.
		refusalText: 'La facture n’a pas pu repasser en brouillon.',
		act: async (v) => {
			await fireEvent.click(await v.findByTestId('supplier-invoice-unvalidate'));
			await v.findByTestId('supplier-invoice-unvalidate-dialog');
			await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		},
	},
	{
		name: 'joindre un document',
		start: inv(),
		written: inv({ sourceDocument: DOC }),
		mock: attachMock,
		refusal: 'attach-document-error',
		act: async (v) => {
			await fireEvent.click(await v.findByTestId('supplier-invoice-attach'));
			await fireEvent.click(await v.findByTestId('attach-document-submit'));
		},
	},
	{
		name: 'détacher le document',
		start: inv({ sourceDocument: DOC }),
		written: inv(),
		mock: detachMock,
		refusal: 'supplier-invoice-detach-error',
		act: async (v) => {
			await fireEvent.click(await v.findByTestId('supplier-invoice-detach'));
		},
	},
];

/** Ce que lève le client quand la réponse se perd (`status: 0`). */
const LOST = { code: 'NETWORK_ERROR', status: 0, message: 'Réseau indisponible' };
/** Un refus certain. */
const REFUSED = { code: 'VALIDATION_ERROR', status: 400, message: 'Refus du serveur.' };

beforeEach(() => {
	for (const m of [
		getMock,
		payMock,
		cancelMock,
		cancelSettlementMock,
		validateMock,
		unvalidateMock,
		attachMock,
		detachMock,
		listImportedMock,
	])
		m.mockReset();
	listImportedMock.mockResolvedValue([staged()]);
	vi.spyOn(window, 'confirm').mockReturnValue(true);
});
afterEach(() => {
	cleanup();
	vi.clearAllMocks();
	vi.restoreAllMocks();
});

describe('fiche fournisseur — une réponse perdue relit la facture et le dit (AC-8)', () => {
	for (const g of GESTURES) {
		it(`${g.name} : réponse perdue ⇒ relecture et avis « vérifiez » (mutation : aucune relecture)`, async () => {
			getMock.mockResolvedValueOnce(g.start).mockResolvedValueOnce(g.written);
			g.mock.mockRejectedValue(LOST);
			const v = render(Page);
			await g.act(v);
			const notice = await v.findByTestId('supplier-unknown-outcome');
			expect(notice.textContent?.trim()).toBe(VERIFY);
			expect(getMock).toHaveBeenCalledTimes(2);
			expect(v.queryByTestId(g.refusal)).toBeNull();
		});

		it(`${g.name} : refus 400 ⇒ le message, sans relecture ni avis (mutation : tout échec relu)`, async () => {
			getMock.mockResolvedValue(g.start);
			g.mock.mockRejectedValue(REFUSED);
			const v = render(Page);
			await g.act(v);
			expect((await v.findByTestId(g.refusal)).textContent).toContain(
				g.refusalText ?? 'Refus du serveur.',
			);
			expect(getMock).toHaveBeenCalledTimes(1);
			expect(v.queryByTestId('supplier-unknown-outcome')).toBeNull();
		});
	}

	it('payer : une erreur hors `ApiError` n’est plus muette — relecture et avis (M6)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockResolvedValueOnce(inv(PAID));
		payMock.mockRejectedValue(new TypeError('Failed to fetch'));
		const v = render(Page);
		await GESTURES[0].act(v);
		expect((await v.findByTestId('supplier-unknown-outcome')).textContent?.trim()).toBe(VERIFY);
		expect(getMock).toHaveBeenCalledTimes(2);
		expect(await v.findByTestId('supplier-invoice-status')).toBeTruthy();
		expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Payée');
	});

	it('annuler : un 503 se lit « issue inconnue » (mutation : 5xx traité comme un refus)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockResolvedValueOnce(inv({ status: 'cancelled' }));
		cancelMock.mockRejectedValue({ code: 'SERVICE_UNAVAILABLE', status: 503, message: 'x' });
		const v = render(Page);
		await GESTURES[2].act(v);
		expect((await v.findByTestId('supplier-unknown-outcome')).textContent?.trim()).toBe(VERIFY);
		expect(v.queryByTestId('supplier-invoice-cancel-error')).toBeNull();
	});

	it('la relecture échoue ⇒ le second avis, jamais « la facture a été relue » (mutation : avis unique)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockRejectedValueOnce(LOST);
		payMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[0].act(v);
		const notice = await v.findByTestId('supplier-unknown-outcome');
		expect(notice.textContent?.trim()).toBe(REREAD_FAILED);
		expect(notice.getAttribute('data-outcome')).toBe('reread-failed');
	});

	it('joindre : relue sans document ⇒ le sélecteur garde l’issue inconnue, sans encadré', async () => {
		getMock.mockResolvedValue(inv());
		attachMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[5].act(v);
		expect((await v.findByTestId('attach-document-error')).textContent?.trim()).toBe(VERIFY);
		expect(v.getByTestId('attach-document-picker')).toBeTruthy();
		expect(getMock).toHaveBeenCalledTimes(2);
		expect(v.queryByTestId('supplier-unknown-outcome')).toBeNull();
	});

	it('joindre : relue avec le document ⇒ le sélecteur se ferme sur lui, encadré « vérifiez »', async () => {
		getMock.mockResolvedValueOnce(inv()).mockResolvedValueOnce(inv({ sourceDocument: DOC }));
		attachMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[5].act(v);
		expect((await v.findByTestId('supplier-unknown-outcome')).textContent?.trim()).toBe(VERIFY);
		expect(v.queryByTestId('attach-document-picker')).toBeNull();
		expect(v.getByTestId('supplier-invoice-document-name').textContent).toBe('piece.png');
	});

	it('joindre : la relecture échoue ⇒ le sélecteur se ferme, encadré « rechargez »', async () => {
		getMock.mockResolvedValueOnce(inv()).mockRejectedValueOnce(LOST);
		attachMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[5].act(v);
		expect((await v.findByTestId('supplier-unknown-outcome')).textContent?.trim()).toBe(
			REREAD_FAILED,
		);
		await waitFor(() => expect(v.queryByTestId('attach-document-picker')).toBeNull());
	});

	it('l’encadré part au geste suivant (mutation : avis jamais retiré)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockResolvedValueOnce(inv({ status: 'draft' }));
		unvalidateMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[4].act(v);
		await v.findByTestId('supplier-unknown-outcome');
		validateMock.mockResolvedValue(inv({ version: 5 }));
		await fireEvent.click(await v.findByTestId('supplier-invoice-validate'));
		await v.findByTestId('supplier-invoice-validate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-validate-confirm'));
		await waitFor(() => expect(v.queryByTestId('supplier-unknown-outcome')).toBeNull());
	});
});
