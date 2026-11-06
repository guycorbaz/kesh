/**
 * Story v0174-correctifs-3 (#889) — épingle l'horloge (`Date` seul, les minuteries restent
 * réelles) et le fuseau du processus le temps des tests du `describe` qui l'appelle.
 *
 * Défaut : `2026-11-04T23:30:00Z` à `Europe/Zurich`, soit 00 h 30 le 5 novembre à Zurich
 * — l'heure où la date UTC est encore la veille. Un écran qui propose « aujourd'hui »
 * doit y proposer le 5.
 *
 * ⚠️ Module de test : il n'est importé que par des `*.test.ts`.
 */
import { afterEach, beforeEach, vi } from 'vitest';
import { pinTimeZone } from './pin-time-zone.test-helper';

/** L'instant d'épingle par défaut : 00 h 30 le 5 novembre 2026 à Zurich. */
export const AFTER_MIDNIGHT_ZURICH = '2026-11-04T23:30:00Z';

export function pinClock(instant: string = AFTER_MIDNIGHT_ZURICH, zone = 'Europe/Zurich'): void {
	pinTimeZone(zone);
	beforeEach(() => {
		vi.useFakeTimers({ toFake: ['Date'] });
		vi.setSystemTime(new Date(instant));
	});
	afterEach(() => {
		vi.useRealTimers();
	});
}
