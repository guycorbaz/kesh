/**
 * Story 27-1 (#632) — « rapprochée », côté écran. Chaque test nomme la
 * MUTATION qu'il attrape.
 */
import { describe, it, expect } from 'vitest';
import { isReconciled, reconciledCount } from './bank-import-deletion';
import type { BankTransactionDto } from './bank-import.types';

function tx(status: string, matchedEntryId: number | null): BankTransactionDto {
	return {
		id: 1,
		bookingDate: '2026-03-02',
		valueDate: null,
		amount: '10.00',
		currency: 'CHF',
		reference: null,
		details: 'test',
		endToEndId: null,
		transactionId: null,
		counterpartyIban: null,
		counterpartyName: null,
		status,
		matchedEntryId,
	};
}

describe('isReconciled', () => {
	it('vrai pour `reconciled` sans lien (mutation : branche du statut retirée)', () => {
		expect(isReconciled(tx('reconciled', null))).toBe(true);
	});
	it('vrai pour `pending` avec lien (mutation : branche matchedEntryId retirée — M11)', () => {
		expect(isReconciled(tx('pending', 7))).toBe(true);
	});
	it('faux pour `pending` sans lien, rejetée ou non (mutation : prédicat toujours vrai)', () => {
		expect(isReconciled(tx('pending', null))).toBe(false);
	});
});

describe('reconciledCount', () => {
	it('compte les rapprochées d’une liste mixte', () => {
		expect(
			reconciledCount([tx('pending', null), tx('reconciled', 3), tx('pending', 4), tx('pending', null)]),
		).toBe(2);
		expect(reconciledCount([])).toBe(0);
	});
});
