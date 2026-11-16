// Story 36-3 (#1051, AC1) — le prédicat d'ambiguïté des milliers, partagé par le
// solde d'ouverture (35-6) et le journal.

import { describe, expect, it } from 'vitest';
import { isAmbiguousThousands } from './amount-ambiguity';

describe('isAmbiguousThousands (Story 36-3, AC1)', () => {
	it.each([
		['1', '234'],
		['12', '500'],
		['001', '234']
	])('ambigu : partie entière %s, fraction %s, sans groupe', (integerDigits, fraction) => {
		expect(isAmbiguousThousands({ integerDigits, fraction, grouped: false })).toBe(true);
	});

	it.each([
		['1234', '567', false, 'quatre chiffres entiers'],
		['0', '500', false, 'partie entière nulle'],
		['1', '234', true, 'partie entière groupée'],
		['1', '23', false, 'deux décimales'],
		['1', '2340', false, 'quatre décimales'],
		['1', '', false, 'sans fraction'],
		['000', '123', false, 'zéros seulement']
	])('non ambigu : %s / %s (groupé : %s) — %s', (integerDigits, fraction, grouped) => {
		expect(isAmbiguousThousands({ integerDigits, fraction, grouped })).toBe(false);
	});
});
