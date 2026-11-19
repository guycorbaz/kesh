import { describe, expect, it } from 'vitest';
import {
	classifyPriceInput,
	formatPrice,
	formatVatRate,
	normalizePriceInput
} from './product-helpers';

describe('formatPrice', () => {
	it('formate un prix standard avec apostrophe typographique U+2019', () => {
		// U+2019 == \u2019 (pas ASCII \u0027).
		expect(formatPrice('1500.0000')).toBe('1\u2019500.00');
	});

	it('gère les montants < 1000 sans séparateur de milliers', () => {
		expect(formatPrice('42.5000')).toBe('42.50');
	});

	it('gère les grands montants avec précision exacte (> MAX_SAFE_INTEGER)', () => {
		// big.js préserve la précision au-delà de 9×10¹⁵.
		expect(formatPrice('99999999999999.99')).toBe(
			'99\u2019999\u2019999\u2019999\u2019999.99'
		);
	});

	it('retourne "" pour null/undefined/vide', () => {
		expect(formatPrice(null)).toBe('');
		expect(formatPrice(undefined)).toBe('');
		expect(formatPrice('')).toBe('');
	});

	it('retourne la string brute si non parsable (défensif)', () => {
		expect(formatPrice('not-a-number')).toBe('not-a-number');
	});
});

describe('formatVatRate', () => {
	it('formate un taux standard avec %', () => {
		expect(formatVatRate('8.10')).toBe('8.10%');
		expect(formatVatRate('0.00')).toBe('0.00%');
		expect(formatVatRate('2.60')).toBe('2.60%');
	});

	it('retourne "" pour null/undefined/vide', () => {
		expect(formatVatRate(null)).toBe('');
		expect(formatVatRate(undefined)).toBe('');
		expect(formatVatRate('')).toBe('');
	});
});

// Story 48-10 (#1173, D1) : `isValidPrice` et `PRICE_RE`, sans appelant de production, sont
// retirés avec leurs tests ; le prix se lit comme le montant d'une écriture (`readJournalAmount`).
describe('normalizePriceInput', () => {
	it('remplace la virgule décimale par un point (clavier mobile suisse)', () => {
		expect(normalizePriceInput('10,50')).toBe('10.50');
		expect(normalizePriceInput('1,5')).toBe('1.5');
	});

	it('trim les espaces', () => {
		expect(normalizePriceInput('  10.50  ')).toBe('10.50');
		// AC5 : une espace avant et après, collée depuis un tableur (M14).
		expect(normalizePriceInput(' 1250 ')).toBe('1250');
	});

	it('laisse inchangé un input déjà normalisé', () => {
		expect(normalizePriceInput('10.50')).toBe('10.50');
		expect(normalizePriceInput('0')).toBe('0');
	});

	// Story 48-10 (AC1, AC5) — la charge d'une lecture claire ; `null` sinon (M2).
	it("retire l'apostrophe des milliers : « 1'250 » → 1250, « 1’250.50 » → 1250.50", () => {
		expect(normalizePriceInput("1'250")).toBe('1250');
		expect(normalizePriceInput('1’250.50')).toBe('1250.50');
		expect(normalizePriceInput("1'250.50")).toBe('1250.50');
	});

	it('rend null pour le vide, l’ambigu et l’illisible', () => {
		expect(normalizePriceInput('')).toBeNull();
		expect(normalizePriceInput('   ')).toBeNull();
		expect(normalizePriceInput('1.250')).toBeNull();
		expect(normalizePriceInput('1,250')).toBeNull();
		expect(normalizePriceInput('abc')).toBeNull();
		expect(normalizePriceInput("12'34")).toBeNull();
	});

	it('garde les zéros de tête et de fin (« 007.50 », « 1250.0000 »)', () => {
		expect(normalizePriceInput('007.50')).toBe('007.50');
		expect(normalizePriceInput('1250.0000')).toBe('1250.0000');
	});
});

describe('classifyPriceInput', () => {
	it('classe vide comme empty', () => {
		expect(classifyPriceInput('')).toBe('empty');
		expect(classifyPriceInput('   ')).toBe('empty');
	});

	it('classe les négatifs comme negative', () => {
		expect(classifyPriceInput('-1')).toBe('negative');
		expect(classifyPriceInput('-0.5')).toBe('negative');
	});

	it('classe les valides comme ok, y compris virgule normalisée', () => {
		expect(classifyPriceInput('10.50')).toBe('ok');
		expect(classifyPriceInput('10,50')).toBe('ok');
		expect(classifyPriceInput('0')).toBe('ok');
	});

	it('classe les autres comme invalid', () => {
		expect(classifyPriceInput('abc')).toBe('invalid');
		expect(classifyPriceInput('1.23456')).toBe('invalid');
		expect(classifyPriceInput('.5')).toBe('invalid');
	});

	// Story 48-10 (#1173, AC1) — rouge avant le correctif : « 1.250 » était `ok`, lu 1,25 (M1).
	it('« 1.250 » et « 1,250 » sont ambigus', () => {
		expect(classifyPriceInput('1.250')).toBe('ambiguous');
		expect(classifyPriceInput('1,250')).toBe('ambiguous');
		expect(classifyPriceInput('12.125')).toBe('ambiguous');
		expect(classifyPriceInput(' 1.250 ')).toBe('ambiguous');
	});

	// AC1 — rouge avant le correctif : « 1'250 » était `invalid`, « 007.50 » aussi.
	it("« 1'250 », « 007.50 » sont acceptés", () => {
		expect(classifyPriceInput("1'250")).toBe('ok');
		expect(classifyPriceInput('007.50')).toBe('ok');
	});

	// AC5 — rien de juste ne devient refusé (filets, sauf les formes groupées).
	it('les formes justes restent acceptées', () => {
		for (const raw of [
			'1.25',
			'1250',
			"1'250.50",
			'1’250',
			'1,25',
			'0.50',
			' 1250 ',
			'1250.0000',
			'12.1250',
			'0.125',
			'1234.125',
			"1'234.125",
		]) {
			expect(classifyPriceInput(raw), raw).toBe('ok');
		}
	});

	// AC5 — ce que la grammaire du journal ne lit pas devient illisible.
	it('exposant, point sans chiffre, blanc intérieur, plus de quatre décimales : invalid', () => {
		for (const raw of ['1e3', '.5', '5.', '1 250', '10.01000', "12'34"]) {
			expect(classifyPriceInput(raw), raw).toBe('invalid');
		}
	});
});
