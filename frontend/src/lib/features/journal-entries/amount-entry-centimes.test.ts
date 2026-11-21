// Story 48-10 (revue P1, B4, B5) — la lecture au centime des frais de rappel et du minimum.
import { describe, expect, it } from 'vitest';
import { centimesEntryProblem, readCentimesEntry } from './amount-entry-centimes';

describe('readCentimesEntry', () => {
	it('vide (ou fait de blancs) → empty', () => {
		expect(readCentimesEntry('')).toEqual({ kind: 'empty' });
		expect(readCentimesEntry('   ')).toEqual({ kind: 'empty' });
	});

	it('ambigu et illisible : la lecture du journal', () => {
		expect(readCentimesEntry('1.250')).toEqual({ kind: 'ambiguous' });
		expect(readCentimesEntry(' 1,250 ')).toEqual({ kind: 'ambiguous' });
		for (const raw of ["12'34", '1e3', '-5', '.5', '5.', '10.01000']) {
			expect(readCentimesEntry(raw), raw).toEqual({ kind: 'unreadable' });
		}
	});

	it('plus de deux décimales significatives → illisible (le message le dit)', () => {
		for (const raw of ['0.125', '10.1250', '1.2345', '1234.125', '10.0050']) {
			expect(readCentimesEntry(raw), raw).toEqual({ kind: 'unreadable' });
		}
	});

	it('clair : charge normalisée, zéros de fin au-delà du centime retirés', () => {
		const cas: [string, string][] = [
			["1'250", '1250'],
			['1’250.50', '1250.50'],
			['1,25', '1.25'],
			[' 12.50 ', '12.50'],
			['0', '0'],
			['10.0100', '10.01'],
			['1250.0000', '1250.00'],
			['5.0000', '5.00'],
			['1.2500', '1.25'],
		];
		for (const [raw, payload] of cas) {
			expect(readCentimesEntry(raw), raw).toEqual({ kind: 'clear', payload });
		}
	});
});

describe('centimesEntryProblem', () => {
	it('rend le refus, ou null', () => {
		expect(centimesEntryProblem({ kind: 'ambiguous' })).toBe('ambiguous');
		expect(centimesEntryProblem({ kind: 'unreadable' })).toBe('unreadable');
		expect(centimesEntryProblem({ kind: 'empty' })).toBeNull();
		expect(centimesEntryProblem({ kind: 'clear', payload: '1' })).toBeNull();
	});
});
