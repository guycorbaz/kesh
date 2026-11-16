/**
 * Story 37-1 (AC-11) — les dates proposées pour le premier exercice. Les cas limites
 * sont ceux du serveur (`kesh_core::accounting::exercice`, test d'API
 * `fiscal_years_e2e.rs`) : les deux côtés rendent la même fin.
 */
import { describe, expect, it } from 'vitest';
import {
	defaultFiscalYearEnd,
	defaultFiscalYearStart,
	fiscalYearRangeProblem,
	isIsoDate,
} from './premier-exercice';

describe('premier exercice — fin proposée (miroir du serveur)', () => {
	it.each([
		['2026-01-01', '2026-12-31'],
		['2026-04-01', '2027-03-31'],
		['2026-07-01', '2027-06-30'],
		['2028-02-29', '2029-02-28'],
		['2027-03-01', '2028-02-29'],
		['2028-02-28', '2029-02-27'],
		['2026-12-31', '2027-12-30'],
	])('début %s → fin %s', (start, end) => {
		expect(defaultFiscalYearEnd(start)).toBe(end);
	});

	it('rend null pour ce qui n’est pas une date', () => {
		expect(defaultFiscalYearEnd('')).toBeNull();
		expect(defaultFiscalYearEnd('2026-02-30')).toBeNull();
		expect(defaultFiscalYearEnd('01.04.2026')).toBeNull();
	});

	it('lit une année à deux chiffres comme telle, non comme 19xx', () => {
		expect(defaultFiscalYearEnd('0050-01-01')).toBe('0050-12-31');
	});
});

describe('premier exercice — début proposé', () => {
	it('est le 1er janvier de l’année du jour', () => {
		expect(defaultFiscalYearStart('2026-11-16')).toBe('2026-01-01');
		expect(defaultFiscalYearStart('2027-01-01')).toBe('2027-01-01');
	});
});

describe('premier exercice — motif de refus', () => {
	it('admet toute durée, de six à dix-huit mois et au-delà', () => {
		expect(fiscalYearRangeProblem('2026-01-01', '2026-06-30')).toBeNull();
		expect(fiscalYearRangeProblem('2026-07-01', '2027-12-31')).toBeNull();
	});

	it('refuse une fin qui ne suit pas strictement le début', () => {
		expect(fiscalYearRangeProblem('2026-04-01', '2026-04-01')).toBe('end-not-after-start');
		expect(fiscalYearRangeProblem('2026-04-01', '2026-03-31')).toBe('end-not-after-start');
	});

	it('refuse une date vide, impossible ou hors des années 1000 à 9999', () => {
		expect(fiscalYearRangeProblem('', '2026-12-31')).toBe('invalid-date');
		expect(fiscalYearRangeProblem('2026-01-01', '')).toBe('invalid-date');
		expect(fiscalYearRangeProblem('2026-02-30', '2026-12-31')).toBe('invalid-date');
		expect(fiscalYearRangeProblem('0999-01-01', '2026-12-31')).toBe('invalid-date');
	});

	it('isIsoDate suit le calendrier', () => {
		expect(isIsoDate('2028-02-29')).toBe(true);
		expect(isIsoDate('2027-02-29')).toBe(false);
	});
});
