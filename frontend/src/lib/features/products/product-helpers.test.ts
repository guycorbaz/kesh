import { describe, expect, it } from 'vitest';
import {
	classifyPriceInput,
	formatPrice,
	formatVatRate,
	isValidPrice,
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

describe('isValidPrice', () => {
	it('accepte les entiers et décimaux valides', () => {
		expect(isValidPrice('0')).toBe(true);
		expect(isValidPrice('100')).toBe(true);
		expect(isValidPrice('1500.00')).toBe(true);
		expect(isValidPrice('0.0001')).toBe(true);
	});

	it('rejette la chaîne vide (prix requis)', () => {
		expect(isValidPrice('')).toBe(false);
		expect(isValidPrice('   ')).toBe(false);
	});

	it('tolère les espaces en tête/queue (paste)', () => {
		expect(isValidPrice(' 10.50 ')).toBe(true);
		expect(isValidPrice('\t100\n')).toBe(true);
	});

	it('rejette les zéros en tête superflus', () => {
		expect(isValidPrice('007.50')).toBe(false);
		expect(isValidPrice('01')).toBe(false);
	});

	it('rejette le point sans partie entière', () => {
		expect(isValidPrice('.5')).toBe(false);
	});

	it('rejette plus de 4 décimales', () => {
		expect(isValidPrice('1.23456')).toBe(false);
	});

	it('rejette les nombres négatifs ou avec caractères parasites', () => {
		expect(isValidPrice('-1')).toBe(false);
		expect(isValidPrice('1,50')).toBe(false);
		expect(isValidPrice('abc')).toBe(false);
	});
});

describe('normalizePriceInput', () => {
	it('remplace la virgule décimale par un point (clavier mobile suisse)', () => {
		expect(normalizePriceInput('10,50')).toBe('10.50');
		expect(normalizePriceInput('1,5')).toBe('1.5');
	});

	it('trim les espaces', () => {
		expect(normalizePriceInput('  10.50  ')).toBe('10.50');
	});

	it('laisse inchangé un input déjà normalisé', () => {
		expect(normalizePriceInput('10.50')).toBe('10.50');
		expect(normalizePriceInput('0')).toBe('0');
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
});
