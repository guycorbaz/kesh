import { describe, expect, it } from 'vitest';
import { creditNoteStatusLabel, formatCreditNoteTotal } from './credit-note-helpers';

describe('formatCreditNoteTotal', () => {
	it('formats a swiss amount with thousands separator', () => {
		expect(formatCreditNoteTotal('1000.0000')).toBe('1’000.00');
	});

	it('returns empty string for null/undefined/empty', () => {
		expect(formatCreditNoteTotal(null)).toBe('');
		expect(formatCreditNoteTotal(undefined)).toBe('');
		expect(formatCreditNoteTotal('')).toBe('');
	});

	it('returns empty string for invalid input', () => {
		expect(formatCreditNoteTotal('not-a-number')).toBe('');
	});
});

describe('creditNoteStatusLabel', () => {
	it('maps known statuses to french labels', () => {
		expect(creditNoteStatusLabel('draft')).toBe('Brouillon');
		expect(creditNoteStatusLabel('issued')).toBe('Émis');
		expect(creditNoteStatusLabel('cancelled')).toBe('Annulé');
	});
});
