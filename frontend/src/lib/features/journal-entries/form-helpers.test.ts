import { describe, expect, it } from 'vitest';
import { fromJournalEntryResponse, lineResponseToDraft } from './form-helpers';
import type { JournalEntryResponse } from './journal-entries.types';

function mockEntry(
	lines: Array<{ lineOrder: number; accountId: number; debit: string; credit: string }>
): JournalEntryResponse {
	return {
		id: 1,
		companyId: 1,
		fiscalYearId: 1,
		entryNumber: 1,
		entryDate: '2026-04-10',
		journal: 'Banque',
		description: 'Test',
		version: 1,
		lines: lines.map((l) => ({ id: l.lineOrder, ...l })),
		createdAt: '2026-04-10T10:00:00',
		updatedAt: '2026-04-10T10:00:00'
	};
}

describe('lineResponseToDraft', () => {
	it('convertit une ligne débit en draft avec crédit vide', () => {
		const draft = lineResponseToDraft({
			id: 1,
			accountId: 42,
			lineOrder: 1,
			debit: '100.00',
			credit: '0.0000'
		});
		expect(draft).toEqual({ accountId: 42, debit: '100.00', credit: '' });
	});

	it('convertit une ligne crédit en draft avec débit vide', () => {
		const draft = lineResponseToDraft({
			id: 2,
			accountId: 43,
			lineOrder: 2,
			debit: '0.0000',
			credit: '100.00'
		});
		expect(draft).toEqual({ accountId: 43, debit: '', credit: '100.00' });
	});

	it('préserve les montants avec 4 décimales', () => {
		const draft = lineResponseToDraft({
			id: 3,
			accountId: 99,
			lineOrder: 1,
			debit: '10.1234',
			credit: '0'
		});
		expect(draft.debit).toBe('10.1234');
		expect(draft.credit).toBe('');
	});
});

describe('fromJournalEntryResponse', () => {
	it('reconstitue les lignes triées par lineOrder', () => {
		const entry = mockEntry([
			{ lineOrder: 2, accountId: 20, debit: '0', credit: '100' },
			{ lineOrder: 1, accountId: 10, debit: '100', credit: '0' }
		]);
		const drafts = fromJournalEntryResponse(entry);
		expect(drafts).toHaveLength(2);
		expect(drafts[0].accountId).toBe(10);
		expect(drafts[0].debit).toBe('100');
		expect(drafts[1].accountId).toBe(20);
		expect(drafts[1].credit).toBe('100');
	});

	it('gère les écritures multi-lignes débit', () => {
		const entry = mockEntry([
			{ lineOrder: 1, accountId: 1, debit: '30', credit: '0' },
			{ lineOrder: 2, accountId: 2, debit: '20', credit: '0' },
			{ lineOrder: 3, accountId: 3, debit: '0', credit: '50' }
		]);
		const drafts = fromJournalEntryResponse(entry);
		expect(drafts).toHaveLength(3);
		expect(drafts[0].debit).toBe('30');
		expect(drafts[1].debit).toBe('20');
		expect(drafts[2].credit).toBe('50');
	});
});
