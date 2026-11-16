// Story v0174-correctifs-3 (#889, AC-3.5) — la date du jour du poste, `AAAA-MM-JJ`.
// Déplacé de `open-items.test.ts` avec `todayLocal` (C-v0174c-13).
import { afterEach, describe, expect, it, vi } from 'vitest';
import { dayBefore, todayInZurich, todayLocal } from '$lib/shared/utils/local-date';
import { pinTimeZone } from './pin-time-zone.test-helper';

describe('todayLocal — la date LOCALE, jamais l’UTC', () => {
	pinTimeZone('Europe/Zurich');
	afterEach(() => {
		vi.useRealTimers();
	});

	it('lit les champs locaux de l’instant donné (mutation : l’UTC)', () => {
		const d = new Date(2026, 11, 31, 23, 30);
		expect(todayLocal(d)).toBe('2026-12-31');
		expect(todayLocal(new Date(2026, 0, 5, 0, 10))).toBe('2026-01-05');
	});

	it('à 00 h 30 le 5 novembre à Zurich (23 h 30 UTC la veille), rend le 5', () => {
		vi.useFakeTimers({ toFake: ['Date'] });
		vi.setSystemTime(new Date('2026-11-04T23:30:00Z'));
		expect(todayLocal()).toBe('2026-11-05');
	});
});

// Story 35-5a (#907, revue P1 E1) — le jour de Zurich, quel que soit le fuseau du poste.
describe('todayInZurich — le jour de Zurich, non celui du poste', () => {
	const cas: [string, string, string][] = [
		['2025-12-31T23:30:00Z', '2026-01-01', '00 h 30 CET le 1er janvier'],
		['2026-07-01T22:30:00Z', '2026-07-02', '00 h 30 CEST'],
		['2026-07-01T21:59:59Z', '2026-07-01', '23 h 59 min 59 s CEST'],
		['2026-03-10T22:30:00Z', '2026-03-10', '23 h 30 CET (00 h 30 le 11 à Athènes)'],
	];
	for (const zone of ['Europe/Zurich', 'America/New_York', 'Europe/Athens', 'UTC']) {
		describe(`poste ${zone}`, () => {
			pinTimeZone(zone);
			for (const [instant, attendu, motif] of cas) {
				it(`${instant} → ${attendu} (${motif})`, () => {
					expect(todayInZurich(new Date(instant))).toBe(attendu);
				});
			}
		});
	}
});

describe('dayBefore — la veille d’une date pure', () => {
	it.each([
		['2026-03-11', '2026-03-10'],
		['2026-03-01', '2026-02-28'],
		['2024-03-01', '2024-02-29'],
		['2026-01-01', '2025-12-31'],
		['2026-03-30', '2026-03-29'],
		['2026-10-26', '2026-10-25'],
	])('%s → %s', (jour, veille) => {
		expect(dayBefore(jour)).toBe(veille);
	});
});
