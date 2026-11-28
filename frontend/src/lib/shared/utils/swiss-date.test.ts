/**
 * Le formateur de date suisse partagé — revue P1 de la 63-1 (L1).
 *
 * Chaque test nomme la mutation qu'il attrape.
 */
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it, expect } from 'vitest';
import { formatSwissDate, formatSwissDateTime } from './swiss-date';
import * as reportsApi from '$lib/features/reports/reports.api';

describe('swiss-date', () => {
	it('rend une date pure sans fuseau, un horodatage à Zurich, le reste tel quel', () => {
		expect(formatSwissDate('2025-12-20')).toBe('20.12.2025');
		expect(formatSwissDate('2026-11-03T17:40:45')).toBe('03.11.2026');
		expect(formatSwissDate('n’importe quoi')).toBe('n’importe quoi');
		expect(formatSwissDateTime('2026-11-24T23:30:00Z')).toBe('25.11.2026 00:30');
	});

	it('reports.api ré-exporte le même formateur (mutation : une seconde définition dans reports.api)', () => {
		expect(reportsApi.formatSwissDate).toBe(formatSwissDate);
		expect(reportsApi.formatSwissDateTime).toBe(formatSwissDateTime);
	});

	it('la queue commune n’importe plus rien de features/ (mutation : formatSwissDate importé de reports.api)', () => {
		const source = readFileSync(
			resolve(dirname(fileURLToPath(import.meta.url)), 'settlement-cancel-blocked.ts'),
			'utf-8',
		);
		const imports = source.split('\n').filter((l) => /^\s*import\b/.test(l));
		expect(imports.length).toBeGreaterThan(0);
		expect(imports.filter((l) => l.includes('$lib/features/'))).toEqual([]);
	});
});
