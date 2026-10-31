/**
 * Story v017-4b (#849, AC-22, AC-25) — la fiche d'une facture fournisseur : les actions
 * par statut et par rôle (brouillon, validée, payée, annulée × Administrateur, Comptable,
 * Consultation), puis les gestes Valider, Dévalider, Supprimer le brouillon.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (M21, M21-bis, M22). Patron :
 * `supplier-settlement-page.test.ts` — mocks hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type {
	SupplierInvoiceResponse,
	SupplierInvoiceStatus,
} from '$lib/features/supplier-invoices/supplier-invoices.types';

const gotoMock = vi.fn();
vi.mock('$app/navigation', () => ({ goto: (u: string) => gotoMock(u) }));
vi.mock('$app/state', () => ({
	page: { params: { id: '9' }, url: new URL('http://localhost/supplier-invoices/9') },
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
const notifySuccess = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: vi.fn(),
	notifySuccess: (m: string) => notifySuccess(m),
	notifyWarning: vi.fn(),
	notifyInfo: vi.fn(),
}));
const role = { value: 'Comptable' };
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return { role: role.value, username: 'c', userId: '1' };
		},
	},
}));

const getMock = vi.fn();
const validateMock = vi.fn();
const unvalidateMock = vi.fn();
const deleteMock = vi.fn();
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	getSupplierInvoice: (id: number) => getMock(id),
	paySupplierInvoice: vi.fn(),
	cancelSupplierInvoice: vi.fn(),
	cancelSupplierInvoiceSettlement: vi.fn(),
	validateSupplierInvoice: (id: number, v: number) => validateMock(id, v),
	unvalidateSupplierInvoice: (id: number, v: number) => unvalidateMock(id, v),
	deleteSupplierInvoice: (id: number) => deleteMock(id),
}));
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: vi.fn(async () => []),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: vi.fn(async () => []) }));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: vi.fn(async () => []) }));
vi.mock('$lib/features/imported-supplier-invoices/imported-supplier-invoices.api', () => ({
	downloadSupplierInvoiceSourceDocument: vi.fn(),
	fetchSupplierInvoiceSourceDocument: vi.fn(),
	attachSourceDocument: vi.fn(),
	detachSourceDocument: vi.fn(),
	listImported: vi.fn(async () => []),
	fetchImportedSourceDocument: vi.fn(),
}));

import Page from './+page.svelte';

function inv(partial: Partial<SupplierInvoiceResponse> = {}): SupplierInvoiceResponse {
	return {
		id: 9,
		contactId: 1,
		supplierInvoiceNumber: 'FF-9',
		status: 'draft',
		invoiceDate: '2026-06-15',
		dueDate: null,
		totalAmount: '100.00',
		creditorIban: null,
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
		version: 3,
		createdAt: '2026-06-15T00:00:00',
		lines: [],
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

/** Une facture du statut donné, champs cohérents avec lui. */
function ofStatus(status: SupplierInvoiceStatus): SupplierInvoiceResponse {
	switch (status) {
		case 'draft':
			return inv();
		case 'open':
			return inv({
				status: 'open',
				purchaseJournalEntryId: 41,
				cancellable: true,
				cancelBlockedBy: null,
			});
		case 'paid':
			return inv({
				status: 'paid',
				purchaseJournalEntryId: 41,
				settlementType: 'internal_account',
				settlementAccountId: 1,
				settlementJournalEntryId: 42,
				paidAt: '2026-06-20T00:00:00',
				settlementCancellable: true,
				settlementCancelBlockedBy: null,
				cancellable: true,
				cancelBlockedBy: null,
			});
		case 'cancelled':
			return inv({
				status: 'cancelled',
				purchaseJournalEntryId: 41,
				cancellable: false,
				cancelBlockedBy: 'SUPPLIER_INVOICE_CANCELLED',
			});
	}
}

beforeEach(() => {
	role.value = 'Comptable';
});
afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

async function rendered(status: SupplierInvoiceStatus, r: string) {
	role.value = r;
	getMock.mockResolvedValue(ofStatus(status));
	const view = render(Page);
	await view.findByTestId('supplier-invoice-status');
	return view;
}

const present = (v: Awaited<ReturnType<typeof rendered>>, id: string) => v.queryByTestId(id) !== null;

describe('fiche — actions par statut et par rôle', () => {
	it.each(['Admin', 'Comptable', 'Consultation'])(
		'brouillon (%s) : ni bloc Payer (M21) ni bloc Annuler la facture (M21-bis), ni lien d’écriture d’achat',
		async (r) => {
			const v = await rendered('draft', r);
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Brouillon');
			expect(present(v, 'supplier-invoice-pay')).toBe(false);
			expect(present(v, 'supplier-invoice-cancel-block')).toBe(false);
			expect(present(v, 'supplier-invoice-cancel')).toBe(false);
			expect(present(v, 'supplier-invoice-purchase-entry')).toBe(false);
			expect(present(v, 'supplier-invoice-unvalidate')).toBe(false);
		},
	);

	it.each([
		['Admin', true, true],
		['Comptable', true, false],
		['Consultation', false, false],
	])(
		'brouillon (%s) : Modifier et Valider %s, Supprimer le brouillon à l’Administrateur seul (M22)',
		async (r, writes, deletes) => {
			const v = await rendered('draft', r);
			expect(present(v, 'supplier-invoice-edit')).toBe(writes);
			expect(present(v, 'supplier-invoice-validate')).toBe(writes);
			expect(present(v, 'supplier-invoice-delete')).toBe(deletes);
			if (writes) {
				expect(v.getByTestId('supplier-invoice-edit').getAttribute('href')).toBe(
					'/supplier-invoices/9/edit',
				);
			}
		},
	);

	it.each([
		['Admin', true],
		['Comptable', true],
		['Consultation', false],
	])('validée (%s) : Dévalider %s ; Payer et Annuler comme avant ; libellé « Validée »', async (r, writes) => {
		const v = await rendered('open', r);
		expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Validée');
		expect(present(v, 'supplier-invoice-unvalidate')).toBe(writes);
		expect(present(v, 'supplier-invoice-pay')).toBe(true);
		expect(present(v, 'supplier-invoice-cancel')).toBe(writes);
		expect(present(v, 'supplier-invoice-edit')).toBe(false);
		expect(present(v, 'supplier-invoice-validate')).toBe(false);
		expect(present(v, 'supplier-invoice-delete')).toBe(false);
	});

	it.each(['open', 'paid', 'cancelled'] as const)(
		'%s : le lien « Voir l’écriture d’achat » mène à l’écriture',
		async (s) => {
			const v = await rendered(s, 'Comptable');
			expect(v.getByTestId('supplier-invoice-purchase-entry').getAttribute('href')).toBe(
				'/journal-entries/41',
			);
		},
	);

	it.each(['paid', 'cancelled'] as const)('%s : ni Dévalider, ni Modifier, ni Valider, ni Supprimer', async (s) => {
		const v = await rendered(s, 'Admin');
		for (const id of [
			'supplier-invoice-unvalidate',
			'supplier-invoice-edit',
			'supplier-invoice-validate',
			'supplier-invoice-delete',
		]) {
			expect(present(v, id), id).toBe(false);
		}
	});
});

describe('fiche — les gestes du cycle', () => {
	it('Valider : confirmation, appel avec la version lue, la facture rendue s’affiche', async () => {
		const v = await rendered('draft', 'Comptable');
		validateMock.mockResolvedValue(ofStatus('open'));
		await fireEvent.click(v.getByTestId('supplier-invoice-validate'));
		const dialog = await v.findByTestId('supplier-invoice-validate-dialog');
		expect(dialog.textContent).toContain('L’écriture d’achat sera passée au journal');
		await fireEvent.click(v.getByTestId('supplier-invoice-validate-confirm'));
		await waitFor(() => expect(validateMock).toHaveBeenCalledWith(9, 3));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Validée'),
		);
		expect(notifySuccess).toHaveBeenCalled();
	});

	it('un refus de la validation s’affiche sous son bouton, par le message du serveur', async () => {
		const v = await rendered('draft', 'Comptable');
		validateMock.mockRejectedValue({
			code: 'FISCAL_YEAR_INVALID',
			status: 400,
			message: 'Aucun exercice ouvert pour cette date.',
		});
		await fireEvent.click(v.getByTestId('supplier-invoice-validate'));
		await v.findByTestId('supplier-invoice-validate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-validate-confirm'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-validate-error').textContent?.trim()).toBe(
				'Aucun exercice ouvert pour cette date.',
			),
		);
		expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Brouillon');
	});

	it('Dévalider : la confirmation dit les trois effets ; la facture redevient brouillon', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockResolvedValue(ofStatus('draft'));
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		const dialog = await v.findByTestId('supplier-invoice-unvalidate-dialog');
		expect(dialog.textContent).toContain('supprimée');
		expect(dialog.textContent).toContain('trou');
		expect(dialog.textContent).toContain('modifiable');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		await waitFor(() => expect(unvalidateMock).toHaveBeenCalledWith(9, 3));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Brouillon'),
		);
	});

	it('un refus de la dévalidation dit son motif, suffixé du numéro de la pièce (AC-25)', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockRejectedValue({
			code: 'SUPPLIER_INVOICE_IN_PAYMENT_BATCH',
			status: 409,
			message: 'serveur',
			details: { documentId: 4, documentNumber: 'Lot 4' },
		});
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-unvalidate-error').textContent?.trim()).toBe(
				'Cette facture figure dans un lot de paiement en cours : annulez d\'abord le lot. (Lot 4)',
			),
		);
	});

	it('Supprimer le brouillon (Administrateur) : confirmation, suppression, retour à la liste', async () => {
		const v = await rendered('draft', 'Admin');
		deleteMock.mockResolvedValue(undefined);
		await fireEvent.click(v.getByTestId('supplier-invoice-delete'));
		await v.findByTestId('supplier-invoice-delete-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-delete-confirm'));
		await waitFor(() => expect(deleteMock).toHaveBeenCalledWith(9));
		await waitFor(() => expect(gotoMock).toHaveBeenCalledWith('/supplier-invoices'));
		expect(notifySuccess).toHaveBeenCalled();
	});

	it('un refus de la suppression s’affiche sous son bouton, par le message du serveur', async () => {
		const v = await rendered('draft', 'Admin');
		deleteMock.mockRejectedValue({
			code: 'SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY',
			status: 409,
			message: 'Ce brouillon figure dans un lot de paiement.',
		});
		await fireEvent.click(v.getByTestId('supplier-invoice-delete'));
		await v.findByTestId('supplier-invoice-delete-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-delete-confirm'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-delete-error').textContent?.trim()).toBe(
				'Ce brouillon figure dans un lot de paiement.',
			),
		);
		expect(gotoMock).not.toHaveBeenCalled();
	});
});
