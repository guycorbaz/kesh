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
	getSupplierInvoice: (id: number, call?: unknown) => getMock(id, call),
	paySupplierInvoice: (id: number, req: unknown) => payMock(id, req),
	cancelSupplierInvoice: (id: number) => cancelMock(id),
	cancelSupplierInvoiceSettlement: (id: number) => cancelSettlementMock(id),
	validateSupplierInvoice: (id: number, v: number) => validateMock(id, v),
	// Story 48-12b (règle du drapeau) : le mock transmet TOUS ses arguments — un troisième
	// argument inattendu rougit les assertions `(9, 3)`.
	unvalidateSupplierInvoice: (...args: unknown[]) => unvalidateMock(...args),
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
import { notifySuccess, notifyWarning } from '$lib/shared/utils/notify';

const VERIFY = 'L’issue n’est pas connue : la facture a été relue, vérifiez-la avant de réessayer.';
const REREAD_FAILED =
	'L’issue n’est pas connue, et la facture n’a pas pu être relue : rechargez la page avant de réessayer.';
/** Revue P3 (B-3 = E-2) : après un refus connu, la relecture en échec n'a pas le texte de l'issue inconnue. */
const REFUSED_REREAD_FAILED = 'La facture n’a pas pu être relue : rechargez la page avant de réessayer.';

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

/**
 * Revue P2 de la 34-8 (E-1 ; mutation M8) — le second clic après une réponse perdue puis une
 * relecture en échec. Le geste était écrit : le serveur refuse le second envoi par un refus
 * de statut. L'écran relit alors la facture, et n'affiche pas un échec sur la fiche d'avant.
 */
describe('fiche fournisseur — le second clic après une relecture en échec (revue P2, E-1)', () => {
	const STALE: Array<[Gesture, string]> = [
		[GESTURES[0], 'ILLEGAL_STATE_TRANSITION'],
		[GESTURES[1], 'SUPPLIER_INVOICE_NOT_PAID'],
		[GESTURES[2], 'SUPPLIER_INVOICE_CANCELLED'],
	];
	for (const [g, code] of STALE) {
		it(`${g.name} : le refus « ${code} » du second clic relit la facture (mutation M8 : aucune relecture)`, async () => {
			getMock
				.mockResolvedValueOnce(g.start)
				.mockRejectedValueOnce(LOST)
				.mockResolvedValueOnce(g.written);
			g.mock
				.mockRejectedValueOnce(LOST)
				.mockRejectedValueOnce({ code, status: 409, message: 'Transition d’état interdite.' });
			const v = render(Page);
			await g.act(v);
			expect((await v.findByTestId('supplier-unknown-outcome')).textContent?.trim()).toBe(
				REREAD_FAILED,
			);
			await g.act(v);
			await waitFor(() => expect(getMock).toHaveBeenCalledTimes(3));
			// La fiche relue montre l'état écrit : ni l'encadré, ni le refus sur la fiche d'avant.
			await waitFor(() => expect(v.queryByTestId(g.refusal)).toBeNull());
			expect(v.queryByTestId('supplier-unknown-outcome')).toBeNull();
		});
	}

	it('payer : le second clic refusé et une relecture encore en échec ⇒ l’encadré « rechargez », sans « issue inconnue » (revue P3, B-3 = E-2)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockRejectedValueOnce(LOST).mockRejectedValueOnce(LOST);
		payMock
			.mockRejectedValueOnce(LOST)
			.mockRejectedValueOnce({ code: 'ILLEGAL_STATE_TRANSITION', status: 409, message: 'Refus.' });
		const v = render(Page);
		await GESTURES[0].act(v);
		await v.findByTestId('supplier-unknown-outcome');
		await GESTURES[0].act(v);
		await waitFor(() => expect(getMock).toHaveBeenCalledTimes(3));
		const notice = await v.findByTestId('supplier-unknown-outcome');
		await waitFor(() => expect(notice.textContent?.trim()).toBe(REFUSED_REREAD_FAILED));
		expect(notice.getAttribute('data-outcome')).toBe('refused-reread-failed');
	});

	/**
	 * Revue P4 ciblée (C-1 ; mutation M13) — un refus d'écran périmé dont la relecture rend le
	 * **même** statut : le bloc reste affiché, et son refus avec lui.
	 */
	it('payer : un refus périmé relu au même statut reste sous son bouton (mutation M13)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockResolvedValueOnce(inv({ version: 4 }));
		payMock.mockRejectedValue({
			code: 'OPTIMISTIC_LOCK_CONFLICT',
			status: 409,
			message: 'La facture a été modifiée entre-temps.',
		});
		const v = render(Page);
		await GESTURES[0].act(v);
		await waitFor(() => expect(getMock).toHaveBeenCalledTimes(2));
		expect((await v.findByTestId('supplier-invoice-pay-error')).textContent).toContain(
			'La facture a été modifiée entre-temps.',
		);
	});

	/** Revue P5 ciblée (S-2 ; M14, forme « suppression ») — un détachement réussi efface le refus d'avant. */
	it('détacher : refus, puis détachement réussi ⇒ le refus d’avant disparaît (M14, forme « suppression »)', async () => {
		getMock.mockResolvedValue(inv({ sourceDocument: DOC }));
		detachMock
			.mockRejectedValueOnce({ code: 'FORBIDDEN', status: 403, message: 'Refus d’avant.' })
			.mockResolvedValueOnce(inv());
		const v = render(Page);
		await GESTURES[6].act(v);
		expect((await v.findByTestId('supplier-invoice-detach-error')).textContent).toContain(
			'Refus d’avant.',
		);
		await fireEvent.click(v.getByTestId('supplier-invoice-detach'));
		await waitFor(() => expect(detachMock).toHaveBeenCalledTimes(2));
		expect(await v.findByTestId('supplier-invoice-attach')).toBeTruthy();
		expect(v.queryByTestId('supplier-invoice-detach-error')).toBeNull();
	});

	it('détacher : une confirmation refusée garde le refus d’avant (revue P4 ciblée, C-2)', async () => {
		getMock.mockResolvedValue(inv({ sourceDocument: DOC }));
		detachMock.mockRejectedValue({ code: 'FORBIDDEN', status: 403, message: 'Refus d’avant.' });
		const v = render(Page);
		await GESTURES[6].act(v);
		expect((await v.findByTestId('supplier-invoice-detach-error')).textContent).toContain(
			'Refus d’avant.',
		);
		(window.confirm as unknown as ReturnType<typeof vi.fn>).mockReturnValue(false);
		await fireEvent.click(v.getByTestId('supplier-invoice-detach'));
		expect(v.getByTestId('supplier-invoice-detach-error').textContent).toContain('Refus d’avant.');
	});

	/**
	 * Revue P3 (M10b) — un bloc rouvert par une **relecture**, sans geste de l'utilisateur qui
	 * passe par `clearCycleErrors` : un refus de paiement certain (400), puis deux gestes de
	 * document dont la réponse se perd ; la première relecture montre la facture payée ailleurs
	 * (bloc de paiement masqué), la seconde la montre de nouveau validée (bloc rouvert). Le refus
	 * d'avant ne reparaît pas : la relecture réussie l'a vidé.
	 */
	it('payer : un refus ne reparaît pas dans un bloc rouvert par une relecture (mutation M10b)', async () => {
		getMock
			.mockResolvedValueOnce(inv())
			.mockResolvedValueOnce(inv({ ...PAID, sourceDocument: DOC }))
			.mockResolvedValueOnce(inv());
		payMock.mockRejectedValue(REFUSED);
		attachMock.mockRejectedValue(LOST);
		detachMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[0].act(v);
		expect((await v.findByTestId('supplier-invoice-pay-error')).textContent).toContain(
			'Refus du serveur.',
		);
		await GESTURES[5].act(v);
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Payée'),
		);
		await fireEvent.click(await v.findByTestId('supplier-invoice-detach'));
		expect(await v.findByTestId('supplier-invoice-pay-submit')).toBeTruthy();
		expect(v.queryByTestId('supplier-invoice-pay-error')).toBeNull();
	});

	/**
	 * Revue P3 de la 34-8 (A-1 = B-1 = E-1 ; mutation M10) — la séquence entière, en DOM visible :
	 * le refus du second clic, relu « Payée », ne reparaît pas sous le formulaire de paiement
	 * quand l'annulation du règlement le rouvre.
	 */
	it('payer : le refus relu ne reparaît pas quand l’annulation du règlement rouvre le paiement (mutation M10)', async () => {
		getMock
			.mockResolvedValueOnce(inv())
			.mockRejectedValueOnce(LOST)
			.mockResolvedValueOnce(inv(PAID));
		payMock
			.mockRejectedValueOnce(LOST)
			.mockRejectedValueOnce({
				code: 'ILLEGAL_STATE_TRANSITION',
				status: 409,
				message: 'Transition d’état interdite.',
			});
		cancelSettlementMock.mockResolvedValue({ invoice: inv(), reversalJournalEntryId: 77 });
		const v = render(Page);
		await GESTURES[0].act(v);
		await v.findByTestId('supplier-unknown-outcome');
		await GESTURES[0].act(v);
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Payée'),
		);
		await fireEvent.click(await v.findByTestId('supplier-invoice-settlement-cancel'));
		// Le formulaire de paiement est rendu de nouveau, visible : aucun refus sous lui.
		expect(await v.findByTestId('supplier-invoice-pay-submit')).toBeTruthy();
		expect(v.queryByTestId('supplier-invoice-pay-error')).toBeNull();
	});

	/** Revue P3 de la 34-8 (A-2 ; mutations M11, M12) — valider et repasser en brouillon. */
	const CYCLE: Array<[Gesture, string]> = [
		[GESTURES[3], 'ILLEGAL_STATE_TRANSITION'],
		[GESTURES[4], 'OPTIMISTIC_LOCK_CONFLICT'],
	];
	for (const [g, code] of CYCLE) {
		it(`${g.name} : refus « ${code} » après une relecture en échec ⇒ relecture, et l’encadré si elle échoue encore`, async () => {
			getMock.mockResolvedValueOnce(g.start).mockRejectedValueOnce(LOST).mockRejectedValueOnce(LOST);
			g.mock
				.mockRejectedValueOnce(LOST)
				.mockRejectedValueOnce({ code, status: 409, message: 'Refus.' });
			const v = render(Page);
			await g.act(v);
			await v.findByTestId('supplier-unknown-outcome');
			await g.act(v);
			await waitFor(() => expect(getMock).toHaveBeenCalledTimes(3));
			const notice = await v.findByTestId('supplier-unknown-outcome');
			await waitFor(() => expect(notice.getAttribute('data-outcome')).toBe('refused-reread-failed'));
		});
	}

	it('payer : un champ manquant n’efface pas l’encadré d’une issue inconnue (revue P3, B-2)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockRejectedValueOnce(LOST);
		payMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[0].act(v);
		await v.findByTestId('supplier-unknown-outcome');
		const select = v.getByTestId('pay-internal-account') as HTMLSelectElement;
		select.value = select.options[0].value;
		await fireEvent.change(select);
		await fireEvent.click(v.getByTestId('supplier-invoice-pay-submit'));
		expect(await v.findByTestId('supplier-invoice-pay-error')).toBeTruthy();
		expect(v.getByTestId('supplier-unknown-outcome')).toBeTruthy();
	});

	it('détacher : une confirmation refusée n’efface pas l’encadré (revue P3, B-2)', async () => {
		getMock.mockResolvedValueOnce(inv({ sourceDocument: DOC })).mockRejectedValueOnce(LOST);
		detachMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[6].act(v);
		await v.findByTestId('supplier-unknown-outcome');
		(window.confirm as unknown as ReturnType<typeof vi.fn>).mockReturnValue(false);
		await fireEvent.click(v.getByTestId('supplier-invoice-detach'));
		expect(v.getByTestId('supplier-unknown-outcome')).toBeTruthy();
		expect(detachMock).toHaveBeenCalledTimes(1);
	});

	it('joindre : le second clic refusé « déjà traité » sur SON document se dit joint, non en avertissement (revue P2, B-4)', async () => {
		getMock
			.mockResolvedValueOnce(inv())
			.mockResolvedValueOnce(inv())
			.mockResolvedValueOnce(inv({ sourceDocument: DOC }));
		attachMock.mockRejectedValueOnce(LOST).mockRejectedValueOnce({
			code: 'IMPORT_NOT_PENDING_COMPLETION',
			status: 409,
			message: 'Ce document a déjà été traité.',
		});
		const v = render(Page);
		await GESTURES[5].act(v);
		await v.findByTestId('attach-document-error');
		await fireEvent.click(v.getByTestId('attach-document-submit'));
		await waitFor(() => expect(v.queryByTestId('attach-document-picker')).toBeNull());
		expect(notifySuccess).toHaveBeenCalledWith('Document joint à la facture.');
		expect(notifyWarning).not.toHaveBeenCalled();
	});

	it('la relecture se fait sans rejeu (`retry: false`, revue P2, E-2)', async () => {
		getMock.mockResolvedValueOnce(inv()).mockResolvedValueOnce(inv(PAID));
		payMock.mockRejectedValue(LOST);
		const v = render(Page);
		await GESTURES[0].act(v);
		await v.findByTestId('supplier-unknown-outcome');
		expect(getMock).toHaveBeenLastCalledWith(9, { retry: false });
	});
});

// ---------------------------------------------------------------------------
// Story 48-12b (#975, table B, relevé D) — le repassage en brouillon d'une annulée en
// doublon : l'avis F4, son bouton, et chaque site qui l'efface.
// ---------------------------------------------------------------------------

const CANCELLED: Partial<SupplierInvoiceResponse> = { status: 'cancelled', cancellable: false };
const F4_TEXT =
	'Une facture équivalente non annulée existe. Repasser celle-ci en brouillon en ferait un doublon : ne le faites que pour la retirer ou la corriger ensuite.';
const dupRefusal = (code: string, confirmable = true, details = true) => ({
	code,
	status: 409,
	message: 'Message du serveur.',
	...(details ? { details: { supplierInvoiceId: 21, internalNumber: 'FF-2026-021', status: 'open', confirmable } } : {}),
});

/** Ouvre la fiche d'une annulée, repasse en brouillon, et attend l'avis F4. */
async function enF4(start: Partial<SupplierInvoiceResponse> = {}) {
	getMock.mockResolvedValue(inv({ ...CANCELLED, ...start }));
	unvalidateMock.mockRejectedValueOnce(dupRefusal('SUPPLIER_INVOICE_DUPLICATE'));
	const v = render(Page);
	await GESTURES[4].act(v);
	await v.findByTestId('supplier-invoice-unvalidate-duplicate');
	return v;
}

describe('fiche d’une annulée — le refus de doublon au repassage (table B, Story 48-12b)', () => {
	for (const code of ['SUPPLIER_INVOICE_DUPLICATE', 'SUPPLIER_INVOICE_PROBABLE_DUPLICATE']) {
		it(`${code} : avis F4 (texte du repassage, facture active, nouvel onglet), facture toujours annulée`, async () => {
			getMock.mockResolvedValue(inv(CANCELLED));
			unvalidateMock.mockRejectedValueOnce(dupRefusal(code));
			const v = render(Page);
			await GESTURES[4].act(v);
			const notice = await v.findByTestId('supplier-invoice-unvalidate-duplicate');
			// Mutation « texte de création (15-b) sur la fiche ».
			expect(v.getByTestId('supplier-invoice-unvalidate-duplicate-text').textContent).toBe(F4_TEXT);
			expect(notice.textContent).not.toContain('15-b');
			const link = v.getByTestId('supplier-invoice-unvalidate-duplicate-link');
			expect(link.getAttribute('href')).toBe('/supplier-invoices/21');
			expect(link.getAttribute('target')).toBe('_blank');
			expect(link.getAttribute('rel')).toBe('noopener');
			expect(link.textContent).toBe('FF-2026-021');
			expect(v.getByTestId('supplier-invoice-status').textContent).toBe('Annulée');
			// Les deux codes ne passent pas par `supplierInvoiceUnvalidateMessage`.
			expect(v.queryByTestId('supplier-invoice-unvalidate-error')).toBeNull();
			// Mutation « bouton absent sur SUPPLIER_INVOICE_DUPLICATE ».
			expect(v.getByTestId('supplier-invoice-unvalidate-anyway')).toBeTruthy();
		});
	}

	it('le dialogue confirmé appelle (id, version), sans troisième argument (règle du drapeau)', async () => {
		await enF4();
		// Mutation « gestionnaire passé tel quel et drapeau testé par sa vérité » :
		// l'événement du clic partirait en drapeau, (9, 3, true).
		expect(unvalidateMock).toHaveBeenCalledTimes(1);
		expect(unvalidateMock.mock.calls[0]).toEqual([9, 3]);
	});

	it('« Repasser en brouillon malgré tout » renvoie la version affichée avec le drapeau, puis F3', async () => {
		const v = await enF4();
		unvalidateMock.mockResolvedValueOnce(inv({ status: 'draft', purchaseJournalEntryId: null, version: 4 }));
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-anyway'));
		await waitFor(() => expect(unvalidateMock).toHaveBeenCalledTimes(2));
		expect(unvalidateMock.mock.calls[1]).toEqual([9, 3, true]);
		await waitFor(() => expect(v.getByTestId('supplier-invoice-status').textContent).toBe('Brouillon'));
		expect(v.queryByTestId('supplier-invoice-unvalidate-duplicate')).toBeNull();
	});

	it('D2 : au départ du « malgré tout », l’avis et le bouton disparaissent', async () => {
		const v = await enF4();
		let finir: (x: unknown) => void = () => {};
		unvalidateMock.mockReturnValueOnce(new Promise((r) => (finir = r)));
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-anyway'));
		await waitFor(() => expect(v.queryByTestId('supplier-invoice-unvalidate-duplicate')).toBeNull());
		expect(v.queryByTestId('supplier-invoice-unvalidate-anyway')).toBeNull();
		finir(inv({ status: 'draft', purchaseJournalEntryId: null, version: 4 }));
	});

	it('confirmable: false : avis sans bouton', async () => {
		getMock.mockResolvedValue(inv(CANCELLED));
		unvalidateMock.mockRejectedValueOnce(dupRefusal('SUPPLIER_INVOICE_DUPLICATE', false));
		const v = render(Page);
		await GESTURES[4].act(v);
		await v.findByTestId('supplier-invoice-unvalidate-duplicate');
		expect(v.queryByTestId('supplier-invoice-unvalidate-anyway')).toBeNull();
	});

	it('F6 : un des deux codes sans details lisibles → le message du serveur, sans avis F4', async () => {
		getMock.mockResolvedValue(inv(CANCELLED));
		unvalidateMock.mockRejectedValueOnce(dupRefusal('SUPPLIER_INVOICE_PROBABLE_DUPLICATE', true, false));
		const v = render(Page);
		await GESTURES[4].act(v);
		// Mutation « doublon illisible affiché en F4 ».
		expect((await v.findByTestId('supplier-invoice-unvalidate-error')).textContent?.trim()).toBe(
			'Message du serveur.',
		);
		expect(v.queryByTestId('supplier-invoice-unvalidate-duplicate')).toBeNull();
	});

	it('D1 : repasser de nouveau efface l’avis avant l’envoi', async () => {
		const v = await enF4();
		let finir: (x: unknown) => void = () => {};
		unvalidateMock.mockReturnValueOnce(new Promise((r) => (finir = r)));
		await GESTURES[4].act(v);
		await waitFor(() => expect(v.queryByTestId('supplier-invoice-unvalidate-duplicate')).toBeNull());
		expect(unvalidateMock.mock.calls[1]).toEqual([9, 3]);
		finir(inv({ status: 'draft', purchaseJournalEntryId: null, version: 4 }));
	});

	for (const [issue, reponse] of [
		['succès', 'ok'],
		['refus', 'refus'],
		['issue inconnue', 'perdue'],
	] as const) {
		it(`D4 : joindre un document (${issue}) efface l’avis (mutation « avis non effacé par attach »)`, async () => {
			const v = await enF4();
			if (reponse === 'ok') attachMock.mockResolvedValue(inv({ ...CANCELLED, sourceDocument: DOC }));
			else if (reponse === 'refus') attachMock.mockRejectedValue(REFUSED);
			else attachMock.mockRejectedValue(LOST);
			await GESTURES[5].act(v);
			await waitFor(() => expect(attachMock).toHaveBeenCalled());
			await waitFor(() => expect(v.queryByTestId('supplier-invoice-unvalidate-duplicate')).toBeNull());
		});

		it(`D5 : détacher le document, après confirmation (${issue}), efface l’avis`, async () => {
			const v = await enF4({ sourceDocument: DOC });
			if (reponse === 'ok') detachMock.mockResolvedValue(inv(CANCELLED));
			else if (reponse === 'refus') detachMock.mockRejectedValue(REFUSED);
			else detachMock.mockRejectedValue(LOST);
			await GESTURES[6].act(v);
			await waitFor(() => expect(detachMock).toHaveBeenCalled());
			await waitFor(() => expect(v.queryByTestId('supplier-invoice-unvalidate-duplicate')).toBeNull());
		});
	}

	it('D5 : une boîte de détachement annulée garde l’avis (mutation « avis effacé par une boîte annulée »)', async () => {
		const v = await enF4({ sourceDocument: DOC });
		(window.confirm as unknown as ReturnType<typeof vi.fn>).mockReturnValue(false);
		await fireEvent.click(await v.findByTestId('supplier-invoice-detach'));
		expect(detachMock).not.toHaveBeenCalled();
		expect(v.getByTestId('supplier-invoice-unvalidate-duplicate')).toBeTruthy();
	});

	it('D6 : une relecture (issue inconnue d’un geste suivant) efface l’avis (mutation « avis non effacé par reloadAfterRefusal »)', async () => {
		const v = await enF4();
		// Un repassage dont la réponse se perd : relecture, encadré « vérifiez », plus d'avis F4.
		unvalidateMock.mockRejectedValueOnce(LOST);
		getMock.mockResolvedValue(inv(CANCELLED));
		await GESTURES[4].act(v);
		await v.findByTestId('supplier-unknown-outcome');
		expect(v.queryByTestId('supplier-invoice-unvalidate-duplicate')).toBeNull();
	});
});
