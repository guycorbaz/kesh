import { describe, expect, it } from 'vitest';
import Big from 'big.js';
import {
	buildPurchaseVatLines,
	isDraftLineNonEmpty,
	lineVatAmount,
	type PurchaseVatParams
} from './vat-purchase';

const CHARGE = 6000;
const RECOVERABLE = 1171;
const COUNTERPARTY = 2000;

function base(overrides: Partial<PurchaseVatParams> = {}): PurchaseVatParams {
	return {
		chargeAccountId: CHARGE,
		htAmount: '1000',
		ratePercent: '8.10',
		counterpartyAccountId: COUNTERPARTY,
		recoverableAccountId: RECOVERABLE,
		...overrides
	};
}

function sum(lines: { debit: string; credit: string }[], field: 'debit' | 'credit'): string {
	return lines
		.reduce((acc, l) => acc.plus(l[field] === '' ? new Big(0) : new Big(l[field])), new Big(0))
		.toFixed(2);
}

describe('lineVatAmount (parité backend line_vat_amount)', () => {
	it('taux normal 8.10 % sur 1000 → 81.00', () => {
		expect(lineVatAmount('1000', '8.10')).toBe('81.00');
	});

	it('taux réduit 2.60 % sur 1000 → 26.00', () => {
		expect(lineVatAmount('1000', '2.60')).toBe('26.00');
	});

	it('taux spécial 3.80 % sur 1000 → 38.00', () => {
		expect(lineVatAmount('1000', '3.80')).toBe('38.00');
	});

	it('taux exempt 0 → 0.00', () => {
		expect(lineVatAmount('1000', '0')).toBe('0.00');
	});

	it('normalise la virgule décimale (1000,50 @ 8.10 → 81.04)', () => {
		// 1000.50 × 8.10 / 100 = 81.0405 → round₂ = 81.04
		expect(lineVatAmount('1000,50', '8.10')).toBe('81.04');
	});

	it('tie-break half-up away-from-zero (0.10 @ 5 = 0.005 exact → 0.01)', () => {
		// 0.10 × 5 / 100 = 0.0050 exact ; half-up → 0.01 (half-even donnerait 0.00).
		expect(lineVatAmount('0.10', '5')).toBe('0.01');
	});
});

describe('buildPurchaseVatLines', () => {
	it('vat > 0 → 3 lignes (charge, impôt préalable, contrepartie) équilibrées', () => {
		const lines = buildPurchaseVatLines(base());
		expect(lines).toHaveLength(3);
		expect(lines[0]).toEqual({ accountId: CHARGE, debit: '1000.00', credit: '' });
		expect(lines[1]).toEqual({ accountId: RECOVERABLE, debit: '81.00', credit: '' });
		expect(lines[2]).toEqual({ accountId: COUNTERPARTY, debit: '', credit: '1081.00' });
		expect(sum(lines, 'debit')).toBe(sum(lines, 'credit'));
	});

	it('vat == 0 (taux exempt) → 2 lignes, aucune ligne sur le compte impôt préalable', () => {
		const lines = buildPurchaseVatLines(base({ ratePercent: '0' }));
		expect(lines).toHaveLength(2);
		expect(lines.some((l) => l.accountId === RECOVERABLE)).toBe(false);
		expect(lines[0]).toEqual({ accountId: CHARGE, debit: '1000.00', credit: '' });
		expect(lines[1]).toEqual({ accountId: COUNTERPARTY, debit: '', credit: '1000.00' });
		expect(sum(lines, 'debit')).toBe(sum(lines, 'credit'));
	});

	it('ordre canonique : charge → impôt préalable → contrepartie', () => {
		const lines = buildPurchaseVatLines(base());
		expect(lines.map((l) => l.accountId)).toEqual([CHARGE, RECOVERABLE, COUNTERPARTY]);
	});

	it('HT à 3-4 décimales (100.005 @ 8.10) reste équilibré et charge à 2 décimales (F-OPUS-C1)', () => {
		// charge = round₂(100.005) = 100.01 ; vat = round₂(100.005 × 8.10 / 100 = 8.1004…) = 8.10 ;
		// ttc = 100.01 + 8.10 = 108.11 ; Σdebit = 108.11 = Σcredit.
		const lines = buildPurchaseVatLines(base({ htAmount: '100.005' }));
		expect(lines[0].debit).toBe('100.01');
		// charge a exactement 2 décimales (pas le HT brut à 3 décimales).
		expect(lines[0].debit.split('.')[1]).toHaveLength(2);
		expect(sum(lines, 'debit')).toBe(sum(lines, 'credit'));
	});

	it('virgule décimale dans le HT est normalisée', () => {
		const lines = buildPurchaseVatLines(base({ htAmount: '1000,50' }));
		expect(lines[0].debit).toBe('1000.50');
		expect(lines[1].debit).toBe('81.04');
		expect(lines[2].credit).toBe('1081.54');
		expect(sum(lines, 'debit')).toBe(sum(lines, 'credit'));
	});
});

describe('isDraftLineNonEmpty', () => {
	it('ligne vierge initiale → false', () => {
		expect(isDraftLineNonEmpty({ accountId: null, debit: '', credit: '' })).toBe(false);
	});

	it('compte choisi → true', () => {
		expect(isDraftLineNonEmpty({ accountId: 5, debit: '', credit: '' })).toBe(true);
	});

	it('débit saisi → true', () => {
		expect(isDraftLineNonEmpty({ accountId: null, debit: '100', credit: '' })).toBe(true);
	});

	it('crédit saisi → true', () => {
		expect(isDraftLineNonEmpty({ accountId: null, debit: '', credit: '50' })).toBe(true);
	});

	it('espaces seuls → false', () => {
		expect(isDraftLineNonEmpty({ accountId: null, debit: '  ', credit: ' ' })).toBe(false);
	});
});
