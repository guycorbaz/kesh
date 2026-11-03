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
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
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
const cancelMock = vi.fn();
const cancelSettlementMock = vi.fn();
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	getSupplierInvoice: (id: number) => getMock(id),
	paySupplierInvoice: vi.fn(),
	cancelSupplierInvoice: (id: number) => cancelMock(id),
	cancelSupplierInvoiceSettlement: (id: number) => cancelSettlementMock(id),
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

	// Revue P1 (A-7) : chaque statut × chaque rôle.
	it.each([
		['paid', 'Admin'],
		['paid', 'Comptable'],
		['paid', 'Consultation'],
		['cancelled', 'Admin'],
		['cancelled', 'Comptable'],
		['cancelled', 'Consultation'],
	] as const)('%s (%s) : ni Dévalider, ni Modifier, ni Valider, ni Supprimer', async (s, r) => {
		const v = await rendered(s, r);
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

// ---------------------------------------------------------------------------
// Revue de code P1 (E-1 = A-6, B-2, B-3 = E-3, E-2 = A-1)
// ---------------------------------------------------------------------------

describe('fiche — refus d’écran périmé et lot confirmé (revue P1)', () => {
	it('un refus de Valider qui relit une facture devenue validée reste lisible (E-1)', async () => {
		const v = await rendered('draft', 'Comptable');
		validateMock.mockRejectedValue({
			code: 'ILLEGAL_STATE_TRANSITION',
			status: 409,
			message: 'Transition d’état interdite',
		});
		getMock.mockResolvedValue(ofStatus('open'));
		await fireEvent.click(v.getByTestId('supplier-invoice-validate'));
		await v.findByTestId('supplier-invoice-validate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-validate-confirm'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Validée'),
		);
		expect(v.getByTestId('supplier-invoice-validate-error').textContent?.trim()).toBe(
			'Transition d’état interdite',
		);
	});

	it('un refus de Supprimer « déjà validée » relit la fiche et reste lisible (E-1, E-3)', async () => {
		const v = await rendered('draft', 'Admin');
		deleteMock.mockRejectedValue({
			code: 'SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST',
			status: 409,
			message: 'Cette facture fournisseur est validée.',
		});
		getMock.mockResolvedValue(ofStatus('open'));
		await fireEvent.click(v.getByTestId('supplier-invoice-delete'));
		await v.findByTestId('supplier-invoice-delete-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-delete-confirm'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Validée'),
		);
		expect(v.getByTestId('supplier-invoice-delete-error').textContent?.trim()).toBe(
			'Cette facture fournisseur est validée.',
		);
		expect(v.queryByTestId('supplier-invoice-delete')).toBeNull();
	});

	it('un refus de Dévalider « payée » relit la fiche, qui ne propose plus de payer (B-3)', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockRejectedValue({
			code: 'SUPPLIER_INVOICE_PAID',
			status: 409,
			message: 'serveur',
			details: { documentId: null, documentNumber: null },
		});
		getMock.mockResolvedValue(ofStatus('paid'));
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Payée'),
		);
		expect(v.queryByTestId('supplier-invoice-pay')).toBeNull();
		expect(v.getByTestId('supplier-invoice-unvalidate-error').textContent?.trim()).toBe(
			"Cette facture fournisseur est payée : annulez d'abord son règlement.",
		);
	});

	it('un refus périmé s’efface au geste réussi suivant (B-2)', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockRejectedValueOnce({
			code: 'ILLEGAL_STATE_TRANSITION',
			status: 409,
			message: 'x',
		});
		getMock.mockResolvedValue(ofStatus('draft'));
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		await v.findByTestId('supplier-invoice-unvalidate-error');
		validateMock.mockResolvedValue(ofStatus('open'));
		await fireEvent.click(await v.findByTestId('supplier-invoice-validate'));
		await v.findByTestId('supplier-invoice-validate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-validate-confirm'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Validée'),
		);
		expect(v.queryByTestId('supplier-invoice-unvalidate-error')).toBeNull();
	});

	// Revue P2 (B2-1 = A2-1) : un fait historique, jamais « payée par ce lot »
	// (`payment_batches::last_confirmed_batch_for_invoice`).
	const FAIT = 'Cette facture figure dans le lot de paiement n° 12, confirmé le 20.06.2026.';
	const CONSIGNE =
		'Si la banque a exécuté cet ordre, la facture a peut-être déjà été payée : vérifiez le relevé avant de changer son montant.';
	const texte = (el: HTMLElement) => (el.textContent ?? '').replace(/\s+/g, ' ').trim();

	it('une facture validée qui figure dans un lot confirmé : la fiche et la confirmation nomment le lot (E-2 = A-1, B2-1)', async () => {
		role.value = 'Comptable';
		getMock.mockResolvedValue({
			...ofStatus('open'),
			lastConfirmedBatch: { id: 12, confirmedAt: '2026-06-20T10:00:00' },
		});
		const v = render(Page);
		const warning = await v.findByTestId('supplier-invoice-confirmed-batch-warning');
		expect(texte(warning)).toBe(`${FAIT} ${CONSIGNE}`);
		expect(texte(warning)).not.toMatch(/payée par|règlement a été annulé/);
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		expect(texte(v.getByTestId('supplier-invoice-unvalidate-dialog-batch-warning'))).toBe(
			`${FAIT} ${CONSIGNE}`,
		);
	});

	it('les catalogues disent le fait, jamais « payée par ce lot », aux quatre locales (B2-1)', () => {
		const valeur = (loc: string, cle: string) =>
			readFileSync(join('../crates/kesh-i18n/locales', loc, 'messages.ftl'), 'utf-8')
				.split('\n')
				.find((l) => l.startsWith(`${cle} = `))
				?.slice(cle.length + 3);
		expect(valeur('fr-CH', 'supplier-invoices-unvalidate-batch-warning')).toBe(
			'Cette facture figure dans le lot de paiement n° { $batch }, confirmé le { $date }.',
		);
		expect(valeur('fr-CH', 'supplier-invoices-unvalidate-batch-warning-undated')).toBe(
			'Cette facture figure dans le lot de paiement confirmé n° { $batch }.',
		);
		expect(valeur('fr-CH', 'supplier-invoices-unvalidate-batch-warning-action')).toBe(CONSIGNE);
		for (const loc of ['fr-CH', 'de-CH', 'it-CH', 'en-CH']) {
			for (const cle of [
				'supplier-invoices-unvalidate-batch-warning',
				'supplier-invoices-unvalidate-batch-warning-undated',
			]) {
				const v = valeur(loc, cle);
				expect(v, `${loc} ${cle}`).toBeDefined();
				expect(v, `${loc} ${cle}`).not.toMatch(/pay[ée]|bezahlt|paid|pagat|storn|annul/i);
			}
		}
	});

	it('sans date de confirmation, la phrase sans date (B2-3)', async () => {
		role.value = 'Comptable';
		getMock.mockResolvedValue({
			...ofStatus('open'),
			lastConfirmedBatch: { id: 12, confirmedAt: null },
		});
		const v = render(Page);
		const warning = await v.findByTestId('supplier-invoice-confirmed-batch-warning');
		expect(texte(warning)).toBe(
			`Cette facture figure dans le lot de paiement confirmé n° 12. ${CONSIGNE}`,
		);
	});

	it('sans lot confirmé, aucun avertissement', async () => {
		const v = await rendered('open', 'Comptable');
		expect(v.queryByTestId('supplier-invoice-confirmed-batch-warning')).toBeNull();
	});

	it('le rôle Consultation voit le lot confirmé, sans la consigne ni Dévalider (E-2 = A-1, B2-5)', async () => {
		role.value = 'Consultation';
		getMock.mockResolvedValue({
			...ofStatus('open'),
			lastConfirmedBatch: { id: 12, confirmedAt: '2026-06-20T10:00:00' },
		});
		const v = render(Page);
		const warning = await v.findByTestId('supplier-invoice-confirmed-batch-warning');
		expect(texte(warning)).toBe(FAIT);
		expect(v.queryByTestId('supplier-invoice-unvalidate')).toBeNull();
	});

	it('Consultation, facture validée sans lot : aucun bloc vide (A2-4 = E2-3)', async () => {
		const v = await rendered('open', 'Consultation');
		expect(v.queryByTestId('supplier-invoice-unvalidate-block')).toBeNull();
	});

	// Revue P2 (B2-2 = A2-2 = E2-1) : les gestes qui changent le statut effacent les refus du
	// cycle, rendus hors des blocs de statut.
	it('Dévalider refusé « payée », puis Annuler le règlement : le refus disparaît', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockRejectedValue({
			code: 'SUPPLIER_INVOICE_PAID',
			status: 409,
			message: 'serveur',
			details: { documentId: null, documentNumber: null },
		});
		getMock.mockResolvedValue(ofStatus('paid'));
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		await v.findByTestId('supplier-invoice-unvalidate-error');
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Payée'),
		);
		const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
		cancelSettlementMock.mockResolvedValue({ invoice: ofStatus('open') });
		await fireEvent.click(v.getByTestId('supplier-invoice-settlement-cancel'));
		await waitFor(() =>
			expect(v.getByTestId('supplier-invoice-status').textContent?.trim()).toBe('Validée'),
		);
		expect(v.queryByTestId('supplier-invoice-unvalidate-error')).toBeNull();
		confirmSpy.mockRestore();
	});

	it('Dévalider refusé, puis Annuler la facture : le refus disparaît', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockRejectedValue({ code: 'PERIOD_LOCKED', status: 409, message: 'serveur' });
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		await v.findByTestId('supplier-invoice-unvalidate-error');
		const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
		cancelMock.mockResolvedValue(ofStatus('cancelled'));
		await fireEvent.click(v.getByTestId('supplier-invoice-cancel'));
		await waitFor(() => expect(cancelMock).toHaveBeenCalled());
		await waitFor(() => expect(v.queryByTestId('supplier-invoice-unvalidate-error')).toBeNull());
		confirmSpy.mockRestore();
	});

	it('Dévalider refusé, puis Payer : le refus disparaît dès le geste', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockRejectedValue({ code: 'PERIOD_LOCKED', status: 409, message: 'serveur' });
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		await v.findByTestId('supplier-invoice-unvalidate-error');
		await fireEvent.click(v.getByTestId('supplier-invoice-pay-submit'));
		await waitFor(() => expect(v.queryByTestId('supplier-invoice-unvalidate-error')).toBeNull());
	});

	it('LATER_FISCAL_YEAR_CLOSED : le refus nomme l’exercice postérieur clos (E-4)', async () => {
		const v = await rendered('open', 'Comptable');
		unvalidateMock.mockRejectedValue({
			code: 'LATER_FISCAL_YEAR_CLOSED',
			status: 400,
			message: 'serveur',
			details: { fiscalYearId: 7, fiscalYearName: 'Exercice 2027' },
		});
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate'));
		await v.findByTestId('supplier-invoice-unvalidate-dialog');
		await fireEvent.click(v.getByTestId('supplier-invoice-unvalidate-confirm'));
		const err = await v.findByTestId('supplier-invoice-unvalidate-error');
		expect(err.textContent?.trim()).toBe(
			'L’exercice postérieur « Exercice 2027 » est clôturé : pour pouvoir dévalider cette facture, un administrateur doit rouvrir les exercices postérieurs clôturés, en commençant par le plus récent.',
		);
	});
});
