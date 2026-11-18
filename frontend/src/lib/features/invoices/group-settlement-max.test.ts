import { describe, expect, it } from 'vitest';
import { groupMaxChanged, readGroupMax } from './group-settlement-max';

describe('groupMaxChanged (Story 43-7, P2 B2-1 / P3 L3-1)', () => {
	it.each([
		['5.01', '5.01'],
		['5,01', '5.01'],
		['5,00', '5.00'],
		['5.0', '5.00'],
		['5', '5.00'],
		['5.0100', '5.01'],
	])('« %s » contre « %s » rendu : égal, pas un changement', (typed, stored) => {
		expect(groupMaxChanged(typed, stored)).toBe(false);
	});

	it.each([
		['5.02', '5.01'],
		['0', '5.00'],
		['100', '99.99'],
	])('« %s » contre « %s » rendu : différent', (typed, stored) => {
		expect(groupMaxChanged(typed, stored)).toBe(true);
	});

	it('une saisie illisible ou ambiguë compte pour un changement (l’écran la refuse avant l’envoi)', () => {
		expect(groupMaxChanged('abc', '5.00')).toBe(true);
		expect(groupMaxChanged('1.250', '5.00')).toBe(true);
		expect(groupMaxChanged('', '5.00')).toBe(true);
	});

	it('une valeur rendue absente ou illisible compte pour un changement', () => {
		expect(groupMaxChanged('5.00', undefined)).toBe(true);
		expect(groupMaxChanged('5.00', 'n/a')).toBe(true);
	});
});

describe('readGroupMax (Story 43-7)', () => {
	it.each([
		['5.00', '5.00'],
		['0', '0'],
		['0.01', '0.01'],
		['100', '100'],
		['100.00', '100.00'],
		['2,5', '2.5'],
		['5.0000', '5.0000'],
		[' 7.25 ', '7.25'],
	])('lit %s', (raw, payload) => {
		expect(readGroupMax(raw)).toEqual({ ok: true, payload });
	});

	it('refuse le vide', () => {
		expect(readGroupMax('  ')).toEqual({ ok: false, problem: 'empty' });
	});

	it('refuse « 1.250 » comme ambigu, « 1\'250 » est lu puis refusé hors plage', () => {
		expect(readGroupMax('1.250')).toEqual({ ok: false, problem: 'ambiguous' });
		expect(readGroupMax("1'250")).toEqual({ ok: false, problem: 'range' });
	});

	it('refuse les saisies illisibles et le signe', () => {
		expect(readGroupMax('abc')).toEqual({ ok: false, problem: 'unreadable' });
		expect(readGroupMax('-1')).toEqual({ ok: false, problem: 'unreadable' });
	});

	it('refuse plus de deux décimales et plus de 100', () => {
		expect(readGroupMax('1.005')).toEqual({ ok: false, problem: 'ambiguous' });
		expect(readGroupMax('1.0051')).toEqual({ ok: false, problem: 'scale' });
		expect(readGroupMax('100.01')).toEqual({ ok: false, problem: 'range' });
		expect(readGroupMax('1000')).toEqual({ ok: false, problem: 'range' });
	});
});
