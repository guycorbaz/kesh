/**
 * Story 64-1 (#1178, D1/D2) — conversion et validation du taux saisi, fonctions pures.
 */
import { describe, it, expect } from 'vitest';
import { decimalOf, validateRateInput, validateDateInput } from './vat-rates.validate';

describe('decimalOf — le taux lié à un <input type="number"> part en chaîne décimale', () => {
	it.each([
		[8.1, '8.1'],
		[7.7, '7.7'],
		['7.70', '7.70'],
		[' 8.1 ', '8.1'],
		[0, '0'],
		[-0, '0'],
		[100, '100'],
		[null, ''],
		['', ''],
		[undefined, ''],
	])('%j → %j', (input, expected) => {
		expect(decimalOf(input as string | number | null)).toBe(expected);
	});
});

describe('validateRateInput — les cas évidents que le formulaire arrête avant tout envoi', () => {
	it.each([[0], [8.1], [7.7], ['7.70'], [100], [2.5], ['8.10'], ['8.1']])('%j est admis', (v) => {
		expect(validateRateInput(v as string | number | null)).toBeNull();
	});
	it.each([
		[null, 'required'],
		['', 'required'],
		['  ', 'required'],
		['abc', 'invalid'],
		[Number.NaN, 'invalid'],
		[Number.POSITIVE_INFINITY, 'invalid'],
		[150, 'range'],
		[100.01, 'range'],
		[-1, 'range'],
		[8.125, 'decimals'],
		[0.0000001, 'decimals'],
		// Revue 64-1 (R1-2) : jugés sur le texte envoyé, non sur `n × 100` avec tolérance.
		[8.12000000001, 'decimals'],
		[8.100000000001, 'decimals'],
		[1e-12, 'decimals'],
		['8.12000000001', 'decimals'],
	])('%j est refusé : %s', (v, code) => {
		expect(validateRateInput(v as string | number | null)).toBe(code);
	});
});

describe('validateDateInput', () => {
	it('une date vide est refusée, une date saisie est admise', () => {
		expect(validateDateInput('')).toBe('required');
		expect(validateDateInput('2027-01-01')).toBeNull();
	});
});
