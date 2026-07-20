import { describe, expect, it } from 'vitest';
import {
	addDaysIso,
	computeInvoiceTotal,
	computeLineTotal,
	formatInvoiceTotal
} from './invoice-helpers';

describe('computeLineTotal', () => {
	it('multiplies quantity by unit price with 4 decimals', () => {
		expect(computeLineTotal('4.5', '200.00')).toBe('900.0000');
	});
	it('preserves precision across small decimals', () => {
		// Classic float trap: 0.1 + 0.2 != 0.3 — Big handles it.
		expect(computeLineTotal('0.1', '0.2')).toBe('0.0200');
	});
	it('returns 0.0000 on invalid input', () => {
		expect(computeLineTotal('abc', '10')).toBe('0.0000');
	});
});

describe('computeInvoiceTotal', () => {
	it('sums line totals', () => {
		const lines = [
			{ quantity: '4.5', unitPrice: '200.00' },
			{ quantity: '1', unitPrice: '500.00' },
			{ quantity: '3', unitPrice: '10.00' },
		];
		expect(computeInvoiceTotal(lines)).toBe('1430.0000');
	});
	it('returns 0.0000 for empty lines', () => {
		expect(computeInvoiceTotal([])).toBe('0.0000');
	});
});

describe('formatInvoiceTotal', () => {
	it('formats with Swiss apostrophe thousand separator', () => {
		expect(formatInvoiceTotal('1500.0000')).toContain('500');
	});
	it('returns empty string on null/undefined', () => {
		expect(formatInvoiceTotal(null)).toBe('');
		expect(formatInvoiceTotal(undefined)).toBe('');
	});
});

describe('addDaysIso (#245)', () => {
	it('adds days within a month', () => {
		expect(addDaysIso('2026-07-01', 30)).toBe('2026-07-31');
	});
	it('rolls over month and year boundaries', () => {
		expect(addDaysIso('2026-12-15', 30)).toBe('2027-01-14');
	});
	it('handles leap years', () => {
		expect(addDaysIso('2028-02-28', 1)).toBe('2028-02-29');
		expect(addDaysIso('2027-02-28', 1)).toBe('2027-03-01');
	});
	it('returns the same date for 0 days (comptant)', () => {
		expect(addDaysIso('2026-07-01', 0)).toBe('2026-07-01');
	});
	it('returns input unchanged on invalid iso or non-integer days', () => {
		expect(addDaysIso('', 30)).toBe('');
		expect(addDaysIso('not-a-date', 30)).toBe('not-a-date');
		expect(addDaysIso('2026-07-01', 1.5)).toBe('2026-07-01');
	});
});
