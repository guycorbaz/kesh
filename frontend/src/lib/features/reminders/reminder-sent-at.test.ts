import { describe, expect, it } from 'vitest';
import { reminderSentAtDateTime } from './reminder-sent-at';

describe('reminderSentAtDateTime (#259 — anti-futur)', () => {
	it("pour aujourd'hui AVANT midi UTC, renvoie l'instant courant (jamais midi, jamais futur)", () => {
		const now = new Date('2026-07-20T08:30:00Z');
		const result = reminderSentAtDateTime('2026-07-20', now);
		// Régression #259 : l'ancien code renvoyait '2026-07-20T12:00:00' → futur → 422.
		expect(result).toBe('2026-07-20T08:30:00');
		expect(result).not.toBe('2026-07-20T12:00:00');
		// Invariant : jamais dans le futur par rapport à `now`.
		expect(new Date(`${result}Z`).getTime()).toBeLessThanOrEqual(now.getTime());
	});

	it("pour aujourd'hui APRÈS midi UTC, renvoie aussi l'instant courant (≤ now)", () => {
		const now = new Date('2026-07-20T15:45:12Z');
		const result = reminderSentAtDateTime('2026-07-20', now);
		expect(result).toBe('2026-07-20T15:45:12');
		expect(new Date(`${result}Z`).getTime()).toBeLessThanOrEqual(now.getTime());
	});

	it('pour une date passée, renvoie midi (anti-décalage d\'affichage, jamais futur)', () => {
		const now = new Date('2026-07-20T08:30:00Z');
		const result = reminderSentAtDateTime('2026-07-18', now);
		expect(result).toBe('2026-07-18T12:00:00');
		expect(new Date(`${result}Z`).getTime()).toBeLessThan(now.getTime());
	});

	it('conserve le suffixe heure (#249 — date nue rejetée par NaiveDateTime)', () => {
		const now = new Date('2026-07-20T08:30:00Z');
		// Aujourd'hui comme date passée : jamais de forme « YYYY-MM-DD » seule.
		expect(reminderSentAtDateTime('2026-07-20', now)).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$/);
		expect(reminderSentAtDateTime('2026-07-01', now)).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$/);
	});
});
