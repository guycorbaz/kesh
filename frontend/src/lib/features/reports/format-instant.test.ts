/**
 * Story 50-6 (#1511, AC2 à AC4) — `formatInstant` : un instant du serveur montré à l'heure de
 * Zurich, quel que soit le fuseau du poste (le processus, que `pinTimeZone` épingle).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape (fiche 50-6, M1 et M2). Valeurs d'ICU de Node
 * (fiche, prémisse P2) : `fr-CH` medium = « 25 nov. 2026, 00:30 », `de-CH` = « 25.11.2026, 00:30 ».
 */
import { describe, expect, it } from 'vitest';
import { formatInstant } from './reports.api';
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';

for (const poste of ['America/New_York', 'UTC']) {
	describe(`formatInstant, poste à ${poste}`, () => {
		pinTimeZone(poste);

		it('AC2 : le jour suivant, l’hiver (mutation M1 : `timeZone` retiré)', () => {
			expect(formatInstant('2026-11-24T23:30:00Z', 'dateHeure', 'fr-CH')).toBe('25 nov. 2026, 00:30');
			expect(formatInstant('2026-11-24T23:30:00Z', 'date', 'fr-CH')).toBe('25 nov. 2026');
			expect(formatInstant('2026-11-24T23:30:00Z', 'dateHeure', 'de-CH')).toBe('25.11.2026, 00:30');
		});

		it('AC2 : la forme naïve est lue UTC, et neuf décimales passent (mutation M2 : `Z` non ajouté)', () => {
			const attendu = '25 nov. 2026, 00:30';
			expect(formatInstant('2026-11-24T23:30:00.123', 'dateHeure', 'fr-CH')).toBe(attendu);
			expect(formatInstant('2026-11-24T23:30:00.123456789Z', 'dateHeure', 'fr-CH')).toBe(attendu);
			expect(formatInstant('2026-11-24 23:30:00', 'dateHeure', 'fr-CH')).toBe(attendu);
		});

		it('AC3 : l’été et les deux bascules', () => {
			expect(formatInstant('2026-07-14T22:30:00Z', 'dateHeure', 'fr-CH')).toBe('15 juil. 2026, 00:30');
			expect(formatInstant('2026-03-29T00:59:00Z', 'dateHeure', 'fr-CH')).toBe('29 mars 2026, 01:59');
			expect(formatInstant('2026-03-29T01:00:00Z', 'dateHeure', 'fr-CH')).toBe('29 mars 2026, 03:00');
			expect(formatInstant('2026-10-25T00:30:00Z', 'dateHeure', 'fr-CH')).toBe('25 oct. 2026, 02:30');
			expect(formatInstant('2026-10-25T01:30:00Z', 'dateHeure', 'fr-CH')).toBe('25 oct. 2026, 02:30');
		});

		it('AC4 : l’absent et l’illisible rendent null', () => {
			for (const v of [null, undefined, '', 'abc', '2026-13-45T99:99:99Z']) {
				expect(formatInstant(v, 'dateHeure', 'fr-CH'), String(v)).toBeNull();
				expect(formatInstant(v, 'date', 'fr-CH'), String(v)).toBeNull();
			}
		});
	});
}

describe('formatInstant, poste à Europe/Zurich (AC9 : filet de non-régression, vert avant et après)', () => {
	pinTimeZone('Europe/Zurich');

	it('rend la même chose que le poste suisse d’avant la story', () => {
		expect(formatInstant('2026-11-24T23:30:00Z', 'dateHeure', 'fr-CH')).toBe('25 nov. 2026, 00:30');
		expect(formatInstant('2026-11-24T11:00:00Z', 'date', 'fr-CH')).toBe('24 nov. 2026');
	});
});
