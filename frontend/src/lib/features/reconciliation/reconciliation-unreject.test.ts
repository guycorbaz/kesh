/**
 * Story 28-5b (#526) — V1 : « rejetée », côté écran. Chaque test nomme la
 * MUTATION qu'il attrape.
 */
import { describe, it, expect } from 'vitest';
import { isRejected } from './reconciliation-unreject';
import type { BankTransactionDto } from '$lib/features/bank-import/bank-import.types';

function tx(
	status: string,
	matchedEntryId: number | null,
	autoMatchRejectedAt: string | null | undefined,
): BankTransactionDto {
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
		// `undefined` simule une réponse d'un serveur antérieur, sans le champ.
		autoMatchRejectedAt: autoMatchRejectedAt as string | null,
	};
}

describe('isRejected', () => {
	it('vrai pour une transaction `pending`, sans lien, marquée', () => {
		expect(isRejected(tx('pending', null, '2026-10-13T08:00:00Z'))).toBe(true);
	});

	it('faux pour une transaction rapprochée même marquée (mutation M9 : `!isReconciled` retiré)', () => {
		expect(isRejected(tx('reconciled', 12, '2026-10-13T08:00:00Z'))).toBe(false);
		// Un lien seul suffit à la dire rapprochée (définition d'`isReconciled`).
		expect(isRejected(tx('pending', 12, '2026-10-13T08:00:00Z'))).toBe(false);
	});

	it('faux pour une transaction non marquée (`null`)', () => {
		expect(isRejected(tx('pending', null, null))).toBe(false);
	});

	it('faux quand le champ est ABSENT (`undefined`) (mutation M16 : `!== null` au lieu de `!= null`)', () => {
		expect(isRejected(tx('pending', null, undefined))).toBe(false);
	});
});
