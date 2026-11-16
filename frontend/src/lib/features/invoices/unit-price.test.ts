// Story 36-3b (#1171, AC1) — le prix unitaire d'une ligne de facture client : la charge
// envoyée au serveur (`unitPricePayload`), la forme d'un prix rouvert dans son champ
// (`unitPriceFieldValue`) et le refus affiché sous le champ (`unitPriceProblem`).

import { describe, it, expect } from 'vitest';
import { readJournalAmount } from '$lib/features/journal-entries/balance';
import { unitPriceFieldValue, unitPricePayload, unitPriceProblem } from './unit-price';

describe('unitPricePayload (36-3b, D1, D3)', () => {
	it.each([
		["1'250", '1250'],
		['1’250.50', '1250.50'],
		['1,25', '1.25'],
		['100.00', '100.00'],
		['33.3333', '33.3333'],
		['0.00', '0.00'],
		['0.125', '0.125'],
	])('« %s » part `%s`', (raw, attendu) => {
		expect(unitPricePayload(raw)).toBe(attendu);
	});

	it.each(['1.250', '12,500', '001.250', "12'50", '-5', '1e3', '.5', '5.', '1.25000', ''])(
		'« %s » ne part pas (`null`)',
		(raw) => {
			expect(unitPricePayload(raw)).toBeNull();
		},
	);
});

describe('unitPriceProblem (36-3b, D3)', () => {
	it.each(['1.250', '12,500', '001.250'])('« %s » est ambigu', (raw) => {
		expect(unitPriceProblem(raw)).toBe('ambiguous');
	});
	it.each(["12'34", "12'50", '-5', '1e3', '.5', '5.', '1.25000'])('« %s » est illisible', (raw) => {
		expect(unitPriceProblem(raw)).toBe('unreadable');
	});
	it.each(['', "1'250", '1,25', '1.2340', '0.125'])('« %s » ne se signale pas', (raw) => {
		expect(unitPriceProblem(raw)).toBeNull();
	});
});

describe('unitPriceFieldValue (36-3b, D2)', () => {
	// La table de D2, et les cas de la validation (V1-4) : la borne haute de la partie
	// entière ambiguë (`999.1230`), la partie nulle (`0.0010`), l'entier (`1.0000`).
	it.each([
		['1.2340', '1.2340'],
		['12.3450', '12.3450'],
		['1.2500', '1.25'],
		['1250.0000', '1250.00'],
		['100.2000', '100.20'],
		['33.3333', '33.3333'],
		['0.0100', '0.01'],
		['0.1250', '0.125'],
		['0.0010', '0.001'],
		['1000.0000', '1000.00'],
		['1.0000', '1.00'],
		['999.1230', '999.1230'],
	])('« %s » se rouvre `%s`, forme jamais ambiguë', (raw, attendu) => {
		const sortie = unitPriceFieldValue(raw);
		expect(sortie).toBe(attendu);
		expect(readJournalAmount(sortie).kind).not.toBe('ambiguous');
	});

	it('une valeur illisible pour big.js est rendue telle quelle', () => {
		expect(unitPriceFieldValue('abc')).toBe('abc');
	});
});
