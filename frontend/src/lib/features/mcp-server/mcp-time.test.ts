/**
 * Story 32-7 (refs #723) — les dates de l'écran « Serveur MCP » (AC-4 ; #738 B7 = E6).
 *
 * Sous `TZ = 'Europe/Zurich'`, posé ici et rétabli après (précédent :
 * `bank-accounts-page.test.ts`), avec un TÉMOIN que le fuseau a pris : sans lui, la CI
 * en UTC ferait passer à vide la mutation M7 (`depuisUtcNaif` sans `Z`).
 */
import { describe, it, expect, beforeAll, afterAll, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({ i18nLocale: () => 'fr-CH' }));

import { dateAffichee, depuisUtcNaif, estPasse, finDeJourneeLocale, lendemainLocal } from './mcp-time';

const TZ_AVANT = process.env.TZ;

describe('mcp-time (TZ Europe/Zurich)', () => {
	beforeAll(() => {
		process.env.TZ = 'Europe/Zurich';
	});
	afterAll(() => {
		if (TZ_AVANT === undefined) delete process.env.TZ;
		else process.env.TZ = TZ_AVANT;
	});

	it('témoin : le fuseau de Zurich a pris (été : UTC+2)', () => {
		expect(new Date(2026, 6, 1, 12).getTimezoneOffset()).toBe(-120);
	});

	it('une date naïve de clé est lue comme de l’UTC', () => {
		expect(depuisUtcNaif('2026-10-25T10:00:00.123')?.getTime()).toBe(Date.UTC(2026, 9, 25, 10, 0, 0, 123));
	});

	it('une date qui porte déjà un fuseau n’est pas décalée deux fois', () => {
		expect(depuisUtcNaif('2026-10-25T10:00:00.123Z')?.getTime()).toBe(Date.UTC(2026, 9, 25, 10, 0, 0, 123));
		expect(depuisUtcNaif('2026-10-25T12:00:00+02:00')?.getTime()).toBe(Date.UTC(2026, 9, 25, 10, 0, 0, 0));
	});

	it('null, vide ou illisible → null, jamais Invalid Date', () => {
		expect(depuisUtcNaif(null)).toBeNull();
		expect(depuisUtcNaif('')).toBeNull();
		expect(depuisUtcNaif('pas une date')).toBeNull();
		expect(dateAffichee('pas une date')).toBeNull();
	});

	it('l’expiration : fin de la journée locale, et l’aller-retour redonne la même date', () => {
		const iso = finDeJourneeLocale('2026-12-31');
		// 31.12.2026 23:59:59.999 à Zurich (hiver, UTC+1) = 22:59:59.999 UTC
		expect(iso).toBe('2026-12-31T22:59:59.999Z');
		expect(dateAffichee(iso)).toBe('31.12.2026');
		// en été, la journée finit à 21:59:59.999 UTC
		expect(finDeJourneeLocale('2026-07-01')).toBe('2026-07-01T21:59:59.999Z');
		expect(finDeJourneeLocale('')).toBeNull();
	});

	it('le min du champ est le lendemain local', () => {
		expect(lendemainLocal(new Date(2026, 11, 31, 23, 30))).toBe('2027-01-01');
	});

	it('estPasse lit la date naïve comme de l’UTC', () => {
		const maintenant = new Date(Date.UTC(2026, 9, 25, 10, 30));
		expect(estPasse('2026-10-25T10:00:00', maintenant)).toBe(true);
		expect(estPasse('2026-10-25T11:00:00', maintenant)).toBe(false);
		expect(estPasse(null, maintenant)).toBe(false);
	});
});
