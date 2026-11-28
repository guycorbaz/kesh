/**
 * Le dialogue « Annuler le rapprochement » — Story 25-3-b (#418).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('./reconciliation.api', () => ({
	getReconciliationTransaction: vi.fn(),
	cancelReconciliation: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));

import * as api from './reconciliation.api';
import CancelReconciliationDialog from './CancelReconciliationDialog.svelte';
import type { ReconciliationTransactionResponse } from './reconciliation.types';

const mockApi = vi.mocked(api);

function view(partial: Partial<ReconciliationTransactionResponse>): ReconciliationTransactionResponse {
	return {
		id: 42,
		status: 'reconciled',
		amount: '80.00',
		currency: 'CHF',
		bookingDate: '2026-05-15',
		matchedEntryId: 7,
		kind: 'entry',
		invoiceId: null,
		invoiceNumber: null,
		supplierInvoiceId: null,
		supplierInvoiceNumber: null,
		internalNumber: null,
		cancellable: true,
		unlinkOnly: false,
		cancelBlockedBy: null,
		cancelBlockedLabel: null,
		cancelBlockedDocumentId: null,
		reversalDate: '2026-03-14',
		reversalFiscalYearName: '2026',
		...partial,
	};
}

function monter(onSuccess = vi.fn()) {
	return render(CancelReconciliationDialog, {
		bankTransactionId: 42,
		open: true,
		onClose: vi.fn(),
		onSuccess,
	});
}

beforeEach(() => vi.clearAllMocks());
afterEach(() => cleanup());

describe('CancelReconciliationDialog', () => {
	it('écriture propre ⇒ la confirmation dit l’écriture inverse, et Confirmer appelle l’API sur la transaction (mutation : mauvais identifiant)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(view({}));
		const result = {
			bankTransaction: { id: 42, status: 'pending', matchedEntryId: null },
			reversalJournalEntryId: 9,
			invoiceId: null,
			supplierInvoiceId: null,
		};
		mockApi.cancelReconciliation.mockResolvedValue(result);
		const onSuccess = vi.fn();
		const { findByTestId } = monter(onSuccess);
		expect((await findByTestId('reconciliation-cancel-confirm-text')).textContent).toContain(
			'écriture inverse',
		);
		expect(mockApi.getReconciliationTransaction).toHaveBeenCalledWith(42);
		await fireEvent.click(await findByTestId('reconciliation-cancel-submit'));
		await waitFor(() => expect(onSuccess).toHaveBeenCalledWith(result));
		// Story 34-11b (revue P1 E1-1) : le geste porte la décision montrée (`unlinkOnly: false`).
		expect(mockApi.cancelReconciliation).toHaveBeenCalledWith(42, false);
	});

	it('règlement de facture ⇒ la confirmation NOMME la facture (mutation : texte de l’écriture propre)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ kind: 'invoice_settlement', invoiceId: 5, invoiceNumber: 'F-2026-0012' }),
		);
		const { findByTestId } = monter();
		const texte = (await findByTestId('reconciliation-cancel-confirm-text')).textContent ?? '';
		expect(texte).toContain('F-2026-0012');
		expect(texte).toContain('redeviendra à régler');
	});

	it('refusé à la lecture ⇒ le MOTIF, et pas de bouton Confirmer (mutation : bouton affiché qui échouerait)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ cancellable: false, cancelBlockedBy: 'FISCAL_YEAR_CLOSED' }),
		);
		const { findByTestId, queryByTestId } = monter();
		expect((await findByTestId('reconciliation-cancel-motif')).textContent).toContain(
			"en commençant par le plus récent",
		);
		expect(queryByTestId('reconciliation-cancel-submit')).toBeNull();
	});

	it('refusé AU CLIC par un code hors motifs ⇒ le message du serveur, dans le dialogue (mutation : erreur avalée ou générique)', async () => {
		// Story 53-6a : `PERIOD_LOCKED` est un motif ; `LATER_FISCAL_YEAR_CLOSED` reprend l'exemple.
		mockApi.getReconciliationTransaction.mockResolvedValue(view({}));
		mockApi.cancelReconciliation.mockRejectedValue({
			code: 'LATER_FISCAL_YEAR_CLOSED',
			message: 'Un exercice postérieur est clôturé.',
			status: 400,
		});
		const onSuccess = vi.fn();
		const { findByTestId } = monter(onSuccess);
		await fireEvent.click(await findByTestId('reconciliation-cancel-submit'));
		expect((await findByTestId('reconciliation-cancel-error')).textContent).toContain(
			'Un exercice postérieur est clôturé.',
		);
		expect(onSuccess).not.toHaveBeenCalled();
	});

	// Story 53-6a (#1674, AC-10, D4) — la date et l'exercice de la contre-passation.
	it('la confirmation dit la DATE de l’écriture et son EXERCICE, jamais « aujourd’hui », et la phrase TVA (mutation : `reversalDate` ignoré)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ reversalDate: '2025-12-20', reversalFiscalYearName: 'FY 2025' }),
		);
		const { findByTestId } = monter();
		const t = (await findByTestId('reconciliation-cancel-confirm-text')).textContent ?? '';
		expect(t).toContain('datée du 20.12.2025');
		expect(t).toContain("dans l'exercice « FY 2025 »");
		expect(t).not.toContain("aujourd'hui");
		const tva = (await findByTestId('reconciliation-cancel-vat')).textContent ?? '';
		// Story 63-1 (revue P3, M2) — une écriture propre ne change la TVA que si elle porte le
		// compte d'impôt préalable : la phrase est conditionnelle, jamais « TVA comprise ».
		expect(tva).toContain("Si cette écriture porte le compte d'impôt préalable");
		expect(tva).not.toContain('TVA comprise');
		// Revue P1, L2 — le conseil dit aussi ce qui s'ensuit : un refus, qui protège la déclaration.
		expect(tva).toContain("l'annulation sera alors refusée, ce qui protège la déclaration");
	});

	// Story 63-1 — un règlement client, un paiement fournisseur ou un règlement groupé ne portent
	// aucune ligne de TVA : leur dé-rapprochement ne change aucun décompte, la phrase de TVA se tait
	// (mutation : phrase montrée pour tous les genres, comme avant).
	it('aucune phrase de TVA pour un règlement, un paiement ou un règlement groupé', async () => {
		const cas: Partial<ReconciliationTransactionResponse>[] = [
			{ kind: 'invoice_settlement', invoiceId: 5, invoiceNumber: 'F-1' },
			{ kind: 'supplier_settlement', supplierInvoiceId: 31, supplierInvoiceNumber: 'FF-1' },
			{
				kind: 'grouped_settlement',
				groupedInvoices: [{ kind: 'invoice', id: 1, number: 'F-1', amount: '10.00' }],
			},
		];
		for (const partial of cas) {
			mockApi.getReconciliationTransaction.mockResolvedValue(view(partial));
			const { findByTestId, queryByTestId } = monter();
			await findByTestId('reconciliation-cancel-confirm-text');
			expect(queryByTestId('reconciliation-cancel-vat'), JSON.stringify(partial)).toBeNull();
			cleanup();
		}
	});

	it('Revue P1 (L4) — une vue sans date ni exercice (serveur antérieur) ne tronque pas la phrase', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ reversalDate: undefined, reversalFiscalYearName: undefined }),
		);
		const { findByTestId } = monter();
		const t = (await findByTestId('reconciliation-cancel-confirm-text')).textContent ?? '';
		expect(t).toContain('datée du —');
		expect(t).toContain("dans l'exercice « — »");
	});

	it('Story 53-6a — les cinq genres nomment la date (mutation : un message oublié)', async () => {
		const cas: Partial<ReconciliationTransactionResponse>[] = [
			{ kind: 'entry' },
			{ kind: 'invoice_settlement', invoiceId: 5, invoiceNumber: 'F-1' },
			{ kind: 'supplier_settlement', supplierInvoiceId: 31, supplierInvoiceNumber: 'FF-1' },
			{
				kind: 'grouped_settlement',
				groupedInvoices: [{ kind: 'invoice', id: 1, number: 'F-1', amount: '10.00' }],
			},
			{
				kind: 'grouped_settlement',
				groupedInvoices: [{ kind: 'supplier_invoice', id: 2, number: 'FF-2', amount: '10.00' }],
			},
		];
		for (const partial of cas) {
			mockApi.getReconciliationTransaction.mockResolvedValue(view(partial));
			const { findByTestId } = monter();
			const t = (await findByTestId('reconciliation-cancel-confirm-text')).textContent ?? '';
			expect(t, JSON.stringify(partial)).toContain('datée du 14.03.2026');
			expect(t, JSON.stringify(partial)).toContain("dans l'exercice « 2026 »");
			cleanup();
		}
	});

	// Story 53-6a (AC-9) — la période verrouillée, à la lecture et au clic.
	it('PERIOD_LOCKED à la lecture ⇒ le motif avec la borne en date suisse, sans Confirmer (mutation : motif non traduit)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ cancellable: false, cancelBlockedBy: 'PERIOD_LOCKED', cancelBlockedLabel: '2026-06-30' }),
		);
		const { findByTestId, queryByTestId } = monter();
		const motif = (await findByTestId('reconciliation-cancel-motif')).textContent ?? '';
		expect(motif).toContain('période verrouillée');
		expect(motif).toContain("verrouillée jusqu'au 30.06.2026 inclus");
		expect(queryByTestId('reconciliation-cancel-submit')).toBeNull();
		expect(queryByTestId('reconciliation-cancel-vat')).toBeNull();
	});

	it('PERIOD_LOCKED au clic (400 du socle) ⇒ le motif, la borne de `details.lockedThrough` (mutation : message du serveur)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(view({}));
		mockApi.cancelReconciliation.mockRejectedValue({
			code: 'PERIOD_LOCKED',
			message: 'texte serveur',
			status: 400,
			details: { lockedThrough: '2026-06-30', attempted: '2026-06-15' },
		});
		const { findByTestId } = monter();
		await fireEvent.click(await findByTestId('reconciliation-cancel-submit'));
		const erreur = (await findByTestId('reconciliation-cancel-error')).textContent ?? '';
		expect(erreur).toContain('période verrouillée');
		expect(erreur).toContain("verrouillée jusqu'au 30.06.2026 inclus");
	});
});

// Story 28-2 (#520, AC9 — B10) — le rejet de `cancelReconciliation` à la
// confirmation (`:97`), et non celui du chargement (B9), par une valeur qui n'est
// pas une `ApiError` : le générique traduit.
describe('CancelReconciliationDialog — échec imprévu à la confirmation (Story 28-2, B10)', () => {
	it('rejet new Error AU CLIC ⇒ le générique (mutation : repli String(err) rétabli — M14)', async () => {
		vi.spyOn(console, 'error').mockImplementation(() => {});
		mockApi.getReconciliationTransaction.mockResolvedValue(view({}));
		mockApi.cancelReconciliation.mockRejectedValue(new Error('boom'));
		const { findByTestId } = monter();
		await fireEvent.click(await findByTestId('reconciliation-cancel-submit'));
		const erreur = await findByTestId('reconciliation-cancel-error');
		expect(erreur.textContent).toContain('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
		expect(erreur.textContent).not.toContain('[object Object]');
	});

	it('B9 — rejet new Error AU CHARGEMENT ⇒ le générique (revue P1 E-L5)', async () => {
		vi.spyOn(console, 'error').mockImplementation(() => {});
		mockApi.getReconciliationTransaction.mockRejectedValue(new Error('boom'));
		const { findByTestId } = monter();
		const erreur = await findByTestId('reconciliation-cancel-error');
		expect(mockApi.getReconciliationTransaction).toHaveBeenCalledWith(42);
		expect(erreur.textContent).toContain('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
	});
});

// Story 30-1b (#686, AC-b11) — T-b40 (ex-T-a9) : les trois genres de rapprochement, et le
// repli `#id` d'une facture fournisseur sans numéro.
describe('CancelReconciliationDialog — règlement fournisseur (Story 30-1b, T-b40)', () => {
	async function texte(partial: Partial<ReconciliationTransactionResponse>) {
		mockApi.getReconciliationTransaction.mockResolvedValue(view(partial));
		const { findByTestId } = monter();
		const t = (await findByTestId('reconciliation-cancel-confirm-text')).textContent ?? '';
		cleanup();
		return t;
	}

	it('supplier_settlement ⇒ le texte fournisseur, qui nomme la facture (M-b26 : texte client)', async () => {
		const t = await texte({
			kind: 'supplier_settlement',
			supplierInvoiceId: 31,
			supplierInvoiceNumber: 'FF-2026-0042',
			internalNumber: null,
		});
		expect(t).toContain('facture fournisseur FF-2026-0042');
		// Story v017-4b (#849, décision Q4) : l'état `open` se nomme « validée ».
		expect(t).toContain('redeviendra validée, à payer');
		expect(t).not.toContain('redeviendra à régler');
	});

	it('supplier_settlement sans numéro ⇒ repli #id', async () => {
		const t = await texte({
			kind: 'supplier_settlement',
			supplierInvoiceId: 31,
			supplierInvoiceNumber: null,
			internalNumber: null,
		});
		expect(t).toContain('facture fournisseur #31');
	});

	it('les trois genres donnent trois textes distincts', async () => {
		const client = await texte({ kind: 'invoice_settlement', invoiceId: 5, invoiceNumber: 'F-1' });
		const fournisseur = await texte({
			kind: 'supplier_settlement',
			supplierInvoiceId: 31,
			supplierInvoiceNumber: 'FF-1',
			internalNumber: null,
		});
		const ecriture = await texte({ kind: 'entry' });
		expect(new Set([client, fournisseur, ecriture]).size).toBe(3);
	});

	// Story 34-11b (#972, AC-4b) — un côté d'un virement rattaché des deux côtés.
	it('unlinkOnly ⇒ la confirmation dit que seul le lien est retiré, sans écriture inverse (mutation : `unlinkOnly` ignoré)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(view({ unlinkOnly: true }));
		const result = {
			bankTransaction: { id: 42, status: 'pending', matchedEntryId: null },
			reversalJournalEntryId: null,
			invoiceId: null,
			supplierInvoiceId: null,
		};
		mockApi.cancelReconciliation.mockResolvedValue(result);
		const onSuccess = vi.fn();
		const { findByTestId, queryByTestId } = monter(onSuccess);
		const t = (await findByTestId('reconciliation-cancel-confirm-text')).textContent ?? '';
		expect(t).toContain('Seul le lien de cette transaction');
		expect(t).toContain('aucune écriture inverse');
		// Story 53-6a : réécrit sur le texte neuf (l'ancien passerait à vide).
		expect(t).not.toContain('Une écriture inverse datée du');
		expect(queryByTestId('reconciliation-cancel-vat')).toBeNull();
		expect(queryByTestId('reconciliation-cancel-motif')).toBeNull();
		await fireEvent.click(await findByTestId('reconciliation-cancel-submit'));
		await waitFor(() => expect(onSuccess).toHaveBeenCalledWith(result));
		expect(mockApi.cancelReconciliation).toHaveBeenCalledWith(42, true);
	});

	// Story 34-11b (revue P1 E1-1) — la décision a changé entre la lecture et le geste.
	it('409 RECONCILIATION_CANCEL_CHANGED ⇒ la vue est relue, la confirmation juste remontrée, le refus affiché, rien de réussi (mutation : pas de relecture)', async () => {
		mockApi.getReconciliationTransaction
			.mockResolvedValueOnce(view({ unlinkOnly: true }))
			.mockResolvedValueOnce(view({ unlinkOnly: false }));
		mockApi.cancelReconciliation.mockRejectedValue({
			code: 'RECONCILIATION_CANCEL_CHANGED',
			message: 'Ce rapprochement a changé depuis l’ouverture de la fenêtre.',
			status: 409,
		});
		const onSuccess = vi.fn();
		const { findByTestId } = monter(onSuccess);
		expect((await findByTestId('reconciliation-cancel-confirm-text')).textContent).toContain(
			'Seul le lien',
		);
		await fireEvent.click(await findByTestId('reconciliation-cancel-submit'));
		await waitFor(() =>
			expect((document.querySelector('[data-testid="reconciliation-cancel-confirm-text"]')?.textContent ?? '')).toContain(
				'écriture inverse',
			),
		);
		expect(mockApi.getReconciliationTransaction).toHaveBeenCalledTimes(2);
		expect((await findByTestId('reconciliation-cancel-error')).textContent).toContain(
			'a changé depuis',
		);
		expect(onSuccess).not.toHaveBeenCalled();
	});
});

// Story 43-3 (#823, AC5) — l'annulation d'un règlement groupé : les N factures rouvertes,
// l'écart et son compte tels que le serveur les rend, et le refus par facture qui la nomme.
describe('CancelReconciliationDialog — règlement groupé (Story 43-3)', () => {
	const groupe = {
		kind: 'grouped_settlement' as const,
		groupedInvoices: [
			{ kind: 'invoice' as const, id: 12, number: 'F-12', amount: '500.0000' },
			{ kind: 'invoice' as const, id: 15, number: null, amount: '300.0000' },
		],
		gap: '-0.40',
		differenceAccount: { id: 6940, number: '6940', name: 'Frais bancaires' },
		cancelBlockedInvoice: null,
	};

	it('les N factures listées redeviendront ouvertes ; écart et compte nommés (mutation : liste omise)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(view(groupe));
		const { findByTestId, getAllByTestId, getByTestId } = monter();
		const texte = (await findByTestId('reconciliation-cancel-confirm-text')).textContent ?? '';
		expect(texte).toContain('le règlement des factures F-12, #15 sera retiré');
		expect(getAllByTestId('reconciliation-cancel-group-invoice').map((li) => li.textContent?.trim())).toEqual([
			'F-12 — 500.00',
			'#15 — 300.00',
		]);
		// Revue P1 (A L-2, B-L6, B-L7) : montants au centime, écart dit avec son sens.
		expect(getByTestId('reconciliation-cancel-group-gap').textContent).toBe(
			'Moins-payé de 0.40, passé au compte 6940 Frais bancaires, sera contre-passé avec l’écriture.',
		);
	});

	// Revue P1 (E-L11) : un écart dont le compte n'a pas pu être relu reste annoncé.
	it('écart sans compte relu : la contre-passation de l’écart reste annoncée', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ ...groupe, gap: '0.40', differenceAccount: null }),
		);
		const { findByTestId } = monter();
		expect((await findByTestId('reconciliation-cancel-group-gap')).textContent).toBe(
			'Trop-perçu de 0.40 sera contre-passé avec l’écriture.',
		);
	});

	it('écart nul (gap 0.00, compte null) : aucune phrase d’écart', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ ...groupe, gap: '0.00', differenceAccount: null }),
		);
		const { findByTestId, queryByTestId } = monter();
		await findByTestId('reconciliation-cancel-confirm-text');
		expect(queryByTestId('reconciliation-cancel-group-gap')).toBeNull();
	});

	// D6 — un groupe fournisseur (commit retirable).
	it('groupe fournisseur : le paiement des factures fournisseur est retiré', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({
				...groupe,
				groupedInvoices: [
					{ kind: 'supplier_invoice', id: 7, number: 'FF-7', amount: '100.0000' },
					{ kind: 'supplier_invoice', id: 8, number: 'FF-8', amount: '50.0000' },
				],
			}),
		);
		const { findByTestId } = monter();
		expect((await findByTestId('reconciliation-cancel-confirm-text')).textContent).toContain(
			'le paiement des factures fournisseur FF-7, FF-8 sera retiré',
		);
	});

	it.each([
		['INVOICE_CREDITED', 'Une des factures de ce rapprochement, F-15, a été créditée'],
		['INVOICE_WRITTEN_OFF', "Le reste d'une des factures de ce rapprochement, F-15, a été soldé"],
		['LETTERING_ALL_LINES_IN_CLOSED_PERIODS', 'Une des factures de ce rapprochement, F-15, est lettrée'],
	] as const)('refus %s par facture ⇒ la facture est nommée (mutation : texte au singulier)', async (code, phrase) => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({
				...groupe,
				cancellable: false,
				cancelBlockedBy: code,
				cancelBlockedInvoice: { kind: 'invoice', id: 15, number: 'F-15' },
			}),
		);
		const { findByTestId, queryByTestId } = monter();
		expect((await findByTestId('reconciliation-cancel-motif')).textContent).toContain(phrase);
		expect(queryByTestId('reconciliation-cancel-submit')).toBeNull();
	});

	it('un motif qui ne porte pas sur une facture garde son texte (exercice clôturé)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(
			view({ ...groupe, cancellable: false, cancelBlockedBy: 'FISCAL_YEAR_CLOSED' }),
		);
		const { findByTestId } = monter();
		expect((await findByTestId('reconciliation-cancel-motif')).textContent).toContain(
			'exercice clôturé',
		);
	});
});
