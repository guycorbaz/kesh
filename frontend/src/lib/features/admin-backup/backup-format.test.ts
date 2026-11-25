/**
 * Story 50-6 (#1511, AC7 et AC9) — `formatBackupDate` à l'heure de Zurich, quel que soit le
 * fuseau du poste. Contrat inchangé : `''` pour une valeur absente ou illisible.
 */
import { describe, expect, it } from 'vitest';
import { formatBackupDate } from './backup-format';
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';

describe('formatBackupDate, poste à America/New_York', () => {
	pinTimeZone('America/New_York');

	it('AC7 : Zurich, non le poste (mutation M3 : `new Date(iso)` et `Intl` sans zone)', () => {
		expect(formatBackupDate('2026-11-24T23:30:00Z', 'fr-CH')).toBe('25 nov. 2026, 00:30');
		expect(formatBackupDate('2026-11-24T23:30:00.123456789Z', 'fr-CH')).toBe('25 nov. 2026, 00:30');
	});

	it('AC7 : le contrat de repli est inchangé', () => {
		expect(formatBackupDate(null, 'fr-CH')).toBe('');
		expect(formatBackupDate('', 'fr-CH')).toBe('');
		expect(formatBackupDate('abc', 'fr-CH')).toBe('');
	});
});

describe('formatBackupDate, poste à Europe/Zurich (AC9 : filet, vert avant et après)', () => {
	pinTimeZone('Europe/Zurich');

	it('rend ce qu’il rendait', () => {
		expect(formatBackupDate('2026-11-24T23:30:00Z', 'fr-CH')).toBe('25 nov. 2026, 00:30');
	});
});
