/**
 * La liste des règlements et son bouton d'annulation — Story 25-3-a-1 (#414).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import InvoiceSettlements from './InvoiceSettlements.svelte';
import type { InvoiceSettlementResponse } from './invoices.types';

function s(partial: Partial<InvoiceSettlementResponse> & { id: number }): InvoiceSettlementResponse {
	return {
		journalEntryId: 100 + partial.id,
		amount: '40.00',
		settledOn: '2026-03-05',
		settlementType: 'internal_account',
		cancellable: true,
		cancelBlockedBy: null,
		cancelBlockedLabel: null,
		cancelBlockedDocumentId: null,
		...partial,
	};
}

afterEach(() => cleanup());

describe('InvoiceSettlements', () => {
	it('rien à afficher sans règlement (mutation : titre vide affiché)', () => {
		const { queryByTestId } = render(InvoiceSettlements, {
			settlements: [],
			canManage: true,
			onCancel: vi.fn(),
		});
		expect(queryByTestId('invoice-settlements')).toBeNull();
	});

	it('une ligne par règlement, avec le lien vers son écriture', () => {
		const { getAllByTestId, container } = render(InvoiceSettlements, {
			settlements: [s({ id: 1 }), s({ id: 2, settlementType: 'bank_transfer' })],
			canManage: true,
			onCancel: vi.fn(),
		});
		expect(getAllByTestId('invoice-settlement-row')).toHaveLength(2);
		expect(container.querySelector('a[href="/journal-entries/101"]')).not.toBeNull();
		expect(container.textContent).toContain('Virement bancaire');
	});

	it('annulable + droit d’écriture ⇒ le bouton, qui remonte le règlement (mutation : mauvais règlement)', async () => {
		const onCancel = vi.fn();
		const cible = s({ id: 7 });
		const { getByTestId } = render(InvoiceSettlements, {
			settlements: [cible],
			canManage: true,
			onCancel,
		});
		await fireEvent.click(getByTestId('invoice-settlement-cancel'));
		expect(onCancel).toHaveBeenCalledWith(cible);
	});

	it('sans droit d’écriture ⇒ pas de bouton (mutation : `canManage` ignoré)', () => {
		const { queryByTestId } = render(InvoiceSettlements, {
			settlements: [s({ id: 1 })],
			canManage: false,
			onCancel: vi.fn(),
		});
		expect(queryByTestId('invoice-settlement-cancel')).toBeNull();
	});

	it('non annulable ⇒ le MOTIF à la place du bouton, pour chaque code (mutation : bouton affiché qui échouerait)', () => {
		const cas: [InvoiceSettlementResponse['cancelBlockedBy'], string][] = [
			['INVOICE_CREDITED', 'paiement à lettrer'],
			['FISCAL_YEAR_CLOSED', "rouvrir l'exercice"],
			['MATCHED_BANK_TRANSACTION', 'annulez d\'abord le rapprochement'],
			['ACCOUNT_ARCHIVED', '(1000)'],
			['FISCAL_YEAR_INVALID', 'Aucun exercice ouvert'],
		];
		for (const [code, extrait] of cas) {
			const { queryByTestId, getByTestId } = render(InvoiceSettlements, {
				settlements: [
					s({ id: 1, cancellable: false, cancelBlockedBy: code, cancelBlockedLabel: '1000' }),
				],
				canManage: true,
				onCancel: vi.fn(),
			});
			expect(queryByTestId('invoice-settlement-cancel')).toBeNull();
			expect(getByTestId('invoice-settlement-cancel-blocked').textContent).toContain(extrait);
			cleanup();
		}
	});
});
