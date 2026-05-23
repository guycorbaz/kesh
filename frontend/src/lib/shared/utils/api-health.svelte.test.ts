/**
 * Tests Vitest du store `apiHealth` (Story 10.3 AC #13bis).
 *
 * Exerce l'**implémentation réelle** du store (pas mockée) pour couvrir les
 * invariants critiques que `api-client.test.ts` ne peut pas tester car il
 * mocke `apiHealth` via `vi.mock` :
 *
 * - (a) Initial state : `isDegraded === false`, aucun timer.
 * - (b) `setDegraded()` idempotence : pas de leak de timer si appelé 2×.
 * - (c) `clearDegraded()` : remet état clean ET clear le timer.
 * - (d) `pollHealth()` recovery : tick `setInterval` détecte `{ db: true }` → exit.
 * - (e) `pollHealth()` resiliency : fetch qui throw ne propage pas
 *       d'`unhandledrejection` et l'état reste `isDegraded === true`.
 *
 * Cleanup obligatoire en `afterEach` (clearDegraded + useRealTimers +
 * unstubAllGlobals) sinon pollution cross-test.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { apiHealth, HEALTH_POLL_INTERVAL_MS } from './api-health.svelte';

describe('apiHealth (Story 10.3)', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		apiHealth.clearDegraded();
		vi.useRealTimers();
		vi.unstubAllGlobals();
	});

	it('(a) initial state : isDegraded false, aucun timer actif', () => {
		expect(apiHealth.isDegraded).toBe(false);
		expect(vi.getTimerCount()).toBe(0);
	});

	it('(b) setDegraded() idempotence : 2 appels consécutifs → 1 seul timer', () => {
		apiHealth.setDegraded();
		expect(apiHealth.isDegraded).toBe(true);
		expect(vi.getTimerCount()).toBe(1);

		// 2e appel idempotent : pas de leak
		apiHealth.setDegraded();
		expect(apiHealth.isDegraded).toBe(true);
		expect(vi.getTimerCount()).toBe(1);
	});

	it('(c) clearDegraded() après setDegraded() : reset état + clear timer', () => {
		apiHealth.setDegraded();
		expect(vi.getTimerCount()).toBe(1);

		apiHealth.clearDegraded();
		expect(apiHealth.isDegraded).toBe(false);
		expect(vi.getTimerCount()).toBe(0);

		// Idempotent : 2e appel no-op
		apiHealth.clearDegraded();
		expect(apiHealth.isDegraded).toBe(false);
		expect(vi.getTimerCount()).toBe(0);
	});

	it('(d) pollHealth() recovery : tick avec { db: true } → exit état dégradé', async () => {
		const okResponse = {
			ok: true,
			status: 200,
			json: () =>
				Promise.resolve({ status: 'ok', db: true, version: '0.1.0' }),
		} as Response;
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue(okResponse));

		apiHealth.setDegraded();
		expect(apiHealth.isDegraded).toBe(true);

		// Avance le timer pour déclencher le 1er tick de pollHealth
		await vi.advanceTimersByTimeAsync(HEALTH_POLL_INTERVAL_MS);

		expect(apiHealth.isDegraded).toBe(false);
		expect(vi.getTimerCount()).toBe(0);
	});

	it("(e) pollHealth() resiliency : fetch qui throw ne pollue pas unhandledrejection, isDegraded reste true", async () => {
		// Pass 1 code review F8 : on **ajoute** un spy à la liste des listeners existants
		// (incluant celui de Vitest) sans les retirer. Si pollHealth's try/catch swallow
		// fonctionne correctement, aucun unhandledRejection ne fire → ni Vitest ni notre
		// spy ne sont appelés. L'ancien pattern `removeAllListeners + restore in finally`
		// créait une fenêtre où une rejection legitime d'un autre test concurrent serait
		// masquée. process.on append-only n'a pas ce risque (et permet de restore proprement
		// avec un seul removeListener ciblé).
		const unhandledSpy = vi.fn();
		process.on('unhandledRejection', unhandledSpy);

		try {
			vi.stubGlobal(
				'fetch',
				vi.fn().mockRejectedValue(new TypeError('network')),
			);

			apiHealth.setDegraded();
			expect(apiHealth.isDegraded).toBe(true);

			// Avance le timer pour déclencher le pollHealth qui va throw
			await vi.advanceTimersByTimeAsync(HEALTH_POLL_INTERVAL_MS);

			// Force la résolution des microtasks pour que tout unhandled rejection
			// éventuel ait eu le temps de se propager
			await Promise.resolve();
			await Promise.resolve();

			expect(unhandledSpy).not.toHaveBeenCalled();
			expect(apiHealth.isDegraded).toBe(true);
		} finally {
			process.removeListener('unhandledRejection', unhandledSpy);
		}
	});
});
