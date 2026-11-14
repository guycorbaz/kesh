/**
 * Story 35-5c (#929, D3) — `ajouterJoursIso`, l'arithmétique de jours des specs E2E.
 */
import { describe, expect, it } from 'vitest';
import { ajouterJoursIso } from './dates';

describe('ajouterJoursIso', () => {
	it('passe la fin de février d’une année commune', () => {
		expect(ajouterJoursIso('2026-02-28', 1)).toBe('2026-03-01');
	});

	it('s’arrête au 29 février d’une année bissextile', () => {
		expect(ajouterJoursIso('2024-02-28', 1)).toBe('2024-02-29');
	});

	it('ignore le passage à l’heure d’été', () => {
		expect(ajouterJoursIso('2026-03-29', 1)).toBe('2026-03-30');
	});

	it('recule d’une année au 1er janvier', () => {
		expect(ajouterJoursIso('2026-01-01', -1)).toBe('2025-12-31');
	});

	it('part du jour reçu, non de l’horloge', () => {
		expect(ajouterJoursIso('2026-03-10', 0)).toBe('2026-03-10');
		expect(ajouterJoursIso('1999-12-31', 40)).toBe('2000-02-09');
	});

	it('refuse ce qui n’est pas un jour AAAA-MM-JJ', () => {
		expect(() => ajouterJoursIso('2026-3-1', 1)).toThrow(/AAAA-MM-JJ/);
	});

	// Revue P1 de la 35-5c (A5, B4, E3) — les entrées que `Date.UTC` normalisait en silence.
	it('refuse une année hors de 1000 à 9999, à l’entrée comme au résultat', () => {
		expect(() => ajouterJoursIso('0050-01-01', 1)).toThrow(/1000 à 9999/);
		expect(() => ajouterJoursIso('0999-12-31', 1)).toThrow(/1000 à 9999/);
		expect(() => ajouterJoursIso('1000-01-01', -1)).toThrow(/1000 à 9999/);
		expect(() => ajouterJoursIso('9999-12-31', 1)).toThrow(/1000 à 9999/);
		expect(ajouterJoursIso('1000-01-01', 0)).toBe('1000-01-01');
	});

	it('refuse un jour impossible', () => {
		expect(() => ajouterJoursIso('2026-02-30', 0)).toThrow(/jour impossible/);
		expect(() => ajouterJoursIso('2026-13-01', 0)).toThrow(/jour impossible/);
		expect(() => ajouterJoursIso('2025-02-29', 1)).toThrow(/jour impossible/);
		expect(() => ajouterJoursIso('2026-04-00', 1)).toThrow(/jour impossible/);
	});

	it('refuse un décalage qui n’est pas un entier fini', () => {
		expect(() => ajouterJoursIso('2026-03-10', Number.NaN)).toThrow(/entier/);
		expect(() => ajouterJoursIso('2026-03-10', 0.5)).toThrow(/entier/);
		expect(() => ajouterJoursIso('2026-03-10', -0.5)).toThrow(/entier/);
		expect(() => ajouterJoursIso('2026-03-10', Number.POSITIVE_INFINITY)).toThrow(/entier/);
	});
});
