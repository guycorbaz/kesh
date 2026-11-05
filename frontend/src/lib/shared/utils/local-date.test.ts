// Story v0174-correctifs-3 (#889, AC-3.5) — la date du jour du poste, `AAAA-MM-JJ`.
// Déplacé de `open-items.test.ts` avec `todayLocal` (C-v0174c-13).
import { afterEach, describe, expect, it, vi } from 'vitest';
import { todayLocal } from '$lib/shared/utils/local-date';
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
