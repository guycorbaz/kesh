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
		cancellable: true,
		cancelBlockedBy: null,
		cancelBlockedLabel: null,
		cancelBlockedDocumentId: null,
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
		expect(mockApi.cancelReconciliation).toHaveBeenCalledWith(42);
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
			"rouvrir l'exercice",
		);
		expect(queryByTestId('reconciliation-cancel-submit')).toBeNull();
	});

	it('refusé AU CLIC par un code hors motifs ⇒ le message du serveur, dans le dialogue (mutation : erreur avalée ou générique)', async () => {
		mockApi.getReconciliationTransaction.mockResolvedValue(view({}));
		mockApi.cancelReconciliation.mockRejectedValue({
			code: 'PERIOD_LOCKED',
			message: 'La période du jour est verrouillée.',
			status: 400,
		});
		const onSuccess = vi.fn();
		const { findByTestId } = monter(onSuccess);
		await fireEvent.click(await findByTestId('reconciliation-cancel-submit'));
		expect((await findByTestId('reconciliation-cancel-error')).textContent).toContain(
			'La période du jour est verrouillée.',
		);
		expect(onSuccess).not.toHaveBeenCalled();
	});
});
