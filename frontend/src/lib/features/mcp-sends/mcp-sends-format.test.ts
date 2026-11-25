/**
 * Story 50-6 (#1511, AC8 et AC9) — `dateLocale` (envois et prompts du serveur MCP) à l'heure de
 * Zurich, quel que soit le fuseau du poste. Contrat inchangé : `—` si vide, valeur brute si illisible.
 * `i18nLocale()` vaut `fr-CH` par défaut (`i18n.svelte.ts`).
 */
import { describe, expect, it } from 'vitest';
import { dateLocale } from './mcp-sends-format';
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';

describe('dateLocale, poste à America/New_York', () => {
	pinTimeZone('America/New_York');

	it('AC8 : la forme naïve de la base est lue UTC et montrée à Zurich (mutations M1 et M2)', () => {
		expect(dateLocale('2026-11-24T23:30:00.000')).toBe('25 nov. 2026, 00:30');
		expect(dateLocale('2026-11-24T23:30:00.000Z')).toBe('25 nov. 2026, 00:30');
	});

	it('AC8 : le contrat de repli est inchangé', () => {
		expect(dateLocale(null)).toBe('—');
		expect(dateLocale('')).toBe('—');
		expect(dateLocale('abc')).toBe('abc');
	});
});

describe('dateLocale, poste à Europe/Zurich (AC9 : filet, vert avant et après)', () => {
	pinTimeZone('Europe/Zurich');

	it('rend ce qu’il rendait', () => {
		expect(dateLocale('2026-11-24T23:30:00.000')).toBe('25 nov. 2026, 00:30');
	});
});
