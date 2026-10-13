/**
 * Tests du helper de contraste (Story 28-1, #640, AC8 § Helper de contraste).
 *
 * Vitest le ramasse par `tests/**\/*.test.ts` ; Playwright l'ignore par
 * `testMatch` (précédent : `test-state.test.ts`).
 *
 * Les cas « foncé en premier » tuent une mutation qui ne trierait pas les
 * luminances (M16) ; blanc/noir tue celle qui oublierait le `+ 0,05` (M15).
 */
import { describe, it, expect } from 'vitest';
import { blend, contrastRatio, parseHex } from './contrast';

describe('parseHex', () => {
	it('lit #rrggbb', () => {
		expect(parseHex('#b91c1c')).toEqual([185, 28, 28]);
		expect(parseHex('#FFFFFF')).toEqual([255, 255, 255]);
	});

	it('refuse une chaîne rgb(…) en la nommant', () => {
		expect(() => parseHex('rgb(185, 28, 28)')).toThrow(/rgb\(185, 28, 28\)/);
	});

	it('refuse un #fff court', () => {
		expect(() => parseHex('#fff')).toThrow(/#fff/);
	});
});

describe('blend', () => {
	it('mélange non arrondi : destructive à 10 % sur blanc = (248 ; 232,3 ; 232,3)', () => {
		const [r, g, b] = blend('#b91c1c', '#ffffff', 0.1);
		expect(r).toBeCloseTo(248, 2);
		expect(g).toBeCloseTo(232.3, 2);
		expect(b).toBeCloseTo(232.3, 2);
	});
});

describe('contrastRatio', () => {
	const cas: Array<[string, string, number]> = [
		['#ffffff', '#000000', 21],
		['#000000', '#ffffff', 21],
		['#ffffff', '#ffffff', 1],
		['#b91c1c', '#ffffff', 6.47],
	];
	for (const [a, b, attendu] of cas) {
		it(`${a} / ${b} ≈ ${attendu}`, () => {
			expect(Math.abs(contrastRatio(a, b) - attendu)).toBeLessThan(0.01);
		});
	}

	it('destructive sur bandeau /10 ≈ 5,46 (foncé en premier)', () => {
		const bandeau = blend('#b91c1c', '#ffffff', 0.1);
		expect(Math.abs(contrastRatio('#b91c1c', bandeau) - 5.46)).toBeLessThan(0.01);
	});
});
