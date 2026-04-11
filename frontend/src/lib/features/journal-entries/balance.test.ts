import { describe, expect, it } from 'vitest';
import Big from 'big.js';
import {
	classifyLine,
	computeBalance,
	formatSwissAmount,
	isValidAmount,
	parseAmount
} from './balance';

describe('isValidAmount', () => {
	it('accepte les entiers', () => {
		expect(isValidAmount('10')).toBe(true);
		expect(isValidAmount('123456789')).toBe(true);
	});

	it('accepte 1 à 4 décimales', () => {
		expect(isValidAmount('10.1')).toBe(true);
		expect(isValidAmount('10.12')).toBe(true);
		expect(isValidAmount('10.1234')).toBe(true);
	});

	it('rejette plus de 4 décimales', () => {
		expect(isValidAmount('10.12345')).toBe(false);
		expect(isValidAmount('10.99999')).toBe(false);
	});

	it('accepte virgule et point', () => {
		expect(isValidAmount('10,50')).toBe(true);
		expect(isValidAmount('10.50')).toBe(true);
	});

	it('accepte chaîne vide', () => {
		expect(isValidAmount('')).toBe(true);
	});

	it('rejette texte arbitraire', () => {
		expect(isValidAmount('abc')).toBe(false);
		expect(isValidAmount('10.5.0')).toBe(false);
	});

	it('P8 : rejette séparateur sans chiffre après', () => {
		expect(isValidAmount('100,')).toBe(false);
		expect(isValidAmount('100.')).toBe(false);
	});
});

describe('formatSwissAmount (P9)', () => {
	it('formate un petit montant avec 2 décimales', () => {
		expect(formatSwissAmount(new Big('1234.5'))).toBe("1\u2019234.50");
	});

	it('ajoute les apostrophes suisses par groupes de 3', () => {
		expect(formatSwissAmount(new Big('1234567.89'))).toBe("1\u2019234\u2019567.89");
	});

	it('zéro formaté correctement', () => {
		expect(formatSwissAmount(new Big('0'))).toBe('0.00');
	});

	it('montants négatifs', () => {
		expect(formatSwissAmount(new Big('-1234.56'))).toBe("-1\u2019234.56");
	});

	it('P9 : montants très grands sans perte de précision', () => {
		// Au-delà de Number.MAX_SAFE_INTEGER (~9×10^15) la conversion
		// en f64 perdrait de la précision — formatSwissAmount reste exact.
		const huge = new Big('99999999999999.99');
		const formatted = formatSwissAmount(huge);
		expect(formatted).toBe("99\u2019999\u2019999\u2019999\u2019999.99");
	});

	it('arrondit à 2 décimales', () => {
		expect(formatSwissAmount(new Big('100.12345'))).toBe('100.12');
	});
});

describe('parseAmount', () => {
	it('convertit virgule en point', () => {
		expect(parseAmount('10,50').eq(new Big('10.50'))).toBe(true);
	});

	it('retourne 0 pour chaîne vide', () => {
		expect(parseAmount('').eq(new Big(0))).toBe(true);
	});
});

describe('computeBalance', () => {
	it('équilibre nominal 2 lignes', () => {
		const lines = [
			{ debit: '100.00', credit: '' },
			{ debit: '', credit: '100.00' }
		];
		const r = computeBalance(lines);
		expect(r.isBalanced).toBe(true);
		expect(r.diff.eq(0)).toBe(true);
	});

	it('équilibre décimal exact (19.95 + 0.05 = 20.00)', () => {
		const lines = [
			{ debit: '19.95', credit: '' },
			{ debit: '0.05', credit: '' },
			{ debit: '', credit: '20.00' }
		];
		const r = computeBalance(lines);
		expect(r.isBalanced).toBe(true);
		expect(r.totalDebit.eq(new Big('20'))).toBe(true);
	});

	it('déséquilibre', () => {
		const lines = [
			{ debit: '100', credit: '' },
			{ debit: '', credit: '80' }
		];
		const r = computeBalance(lines);
		expect(r.isBalanced).toBe(false);
		expect(r.diff.eq(new Big(20))).toBe(true);
	});

	it('champs vides uniquement → non équilibré (total 0)', () => {
		const r = computeBalance([
			{ debit: '', credit: '' },
			{ debit: '', credit: '' }
		]);
		expect(r.isBalanced).toBe(false);
	});

	it('hasInvalidAmount true si plus de 4 décimales', () => {
		const r = computeBalance([
			{ debit: '10.99999', credit: '' },
			{ debit: '', credit: '10.99999' }
		]);
		expect(r.hasInvalidAmount).toBe(true);
		expect(r.isBalanced).toBe(false);
	});

	it('virgule acceptée', () => {
		const r = computeBalance([
			{ debit: '10,50', credit: '' },
			{ debit: '', credit: '10,50' }
		]);
		expect(r.isBalanced).toBe(true);
	});
});

describe('classifyLine', () => {
	it('vide', () => {
		expect(classifyLine({ accountId: null, debit: '', credit: '' })).toBe('empty');
	});

	it('valide avec débit seul', () => {
		expect(classifyLine({ accountId: 1, debit: '100', credit: '' })).toBe('valid');
	});

	it('valide avec crédit seul', () => {
		expect(classifyLine({ accountId: 1, debit: '', credit: '50' })).toBe('valid');
	});

	it('partial : compte sans montant', () => {
		expect(classifyLine({ accountId: 1, debit: '', credit: '' })).toBe('partial');
	});

	it('partial : montant sans compte', () => {
		expect(classifyLine({ accountId: null, debit: '100', credit: '' })).toBe('partial');
	});

	it('partial : débit ET crédit > 0', () => {
		expect(classifyLine({ accountId: 1, debit: '50', credit: '50' })).toBe('partial');
	});

	it('partial : montant invalide', () => {
		expect(classifyLine({ accountId: 1, debit: '10.99999', credit: '' })).toBe('partial');
	});

	it('partial : montant = 0', () => {
		expect(classifyLine({ accountId: 1, debit: '0', credit: '' })).toBe('partial');
	});
});
