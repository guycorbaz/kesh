// Story 43-3 (#823, D4) — les gardes de concurrence de la modale du règlement groupé.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createLatestRequest, createSessionGuard } from './modal-guards';

/** Une promesse dont on tient la résolution. */
function deferred<T>() {
	let resolve!: (v: T) => void;
	let reject!: (e: unknown) => void;
	const promise = new Promise<T>((res, rej) => {
		resolve = res;
		reject = rej;
	});
	return { promise, resolve, reject };
}

describe('SessionGuard', () => {
	it('chaque session neuve périme les précédentes', () => {
		const s = createSessionGuard();
		const a = s.next();
		expect(s.isCurrent(a)).toBe(true);
		const b = s.next();
		expect(s.isCurrent(a)).toBe(false);
		expect(s.isCurrent(b)).toBe(true);
		expect(s.current()).toBe(b);
	});
});

describe('LatestRequest', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});
	afterEach(() => {
		vi.useRealTimers();
	});

	it('attend le délai anti-rebond, et seule la dernière frappe part', async () => {
		const latest = createLatestRequest<string>(250);
		const load = vi.fn((signal: AbortSignal) => {
			void signal;
			return Promise.resolve('r');
		});
		const onResult = vi.fn();
		latest.run(load, { onResult, onError: vi.fn() });
		latest.run(load, { onResult, onError: vi.fn() });
		latest.run(load, { onResult, onError: vi.fn() });
		expect(load).not.toHaveBeenCalled();
		await vi.advanceTimersByTimeAsync(249);
		expect(load).not.toHaveBeenCalled();
		await vi.advanceTimersByTimeAsync(1);
		expect(load).toHaveBeenCalledTimes(1);
		await vi.runAllTimersAsync();
		expect(onResult).toHaveBeenCalledWith('r');
	});

	it('une réponse hors d’ordre est ignorée, et la requête dépassée est avortée', async () => {
		const latest = createLatestRequest<string>(0);
		const first = deferred<string>();
		const second = deferred<string>();
		const signals: AbortSignal[] = [];
		const onResult = vi.fn();
		latest.run(
			(signal) => {
				signals.push(signal);
				return first.promise;
			},
			{ onResult, onError: vi.fn() },
		);
		latest.run(
			(signal) => {
				signals.push(signal);
				return second.promise;
			},
			{ onResult, onError: vi.fn() },
		);
		expect(signals[0].aborted).toBe(true);
		expect(signals[1].aborted).toBe(false);
		second.resolve('récente');
		await vi.runAllTimersAsync();
		first.resolve('ancienne');
		await vi.runAllTimersAsync();
		expect(onResult).toHaveBeenCalledTimes(1);
		expect(onResult).toHaveBeenCalledWith('récente');
	});

	it('une erreur dépassée est ignorée, une erreur courante est rendue', async () => {
		const latest = createLatestRequest<string>(0);
		const first = deferred<string>();
		const onError = vi.fn();
		latest.run(() => first.promise, { onResult: vi.fn(), onError });
		latest.run(() => Promise.reject(new Error('courante')), { onResult: vi.fn(), onError });
		first.reject(new Error('dépassée'));
		await vi.runAllTimersAsync();
		expect(onError).toHaveBeenCalledTimes(1);
		expect((onError.mock.calls[0][0] as Error).message).toBe('courante');
	});

	it('cancel annule le délai et périme la requête en vol', async () => {
		const latest = createLatestRequest<string>(250);
		const load = vi.fn(() => Promise.resolve('r'));
		const onResult = vi.fn();
		latest.run(load, { onResult, onError: vi.fn() });
		latest.cancel();
		await vi.runAllTimersAsync();
		expect(load).not.toHaveBeenCalled();

		const pending = deferred<string>();
		latest.run(() => pending.promise, { onResult, onError: vi.fn() }, true);
		expect(latest.busy()).toBe(true);
		latest.cancel();
		expect(latest.busy()).toBe(false);
		pending.resolve('trop tard');
		await vi.runAllTimersAsync();
		expect(onResult).not.toHaveBeenCalled();
	});

	it('immediate part sans délai et prévient onStart', async () => {
		const latest = createLatestRequest<string>(250);
		const onStart = vi.fn();
		const load = vi.fn(() => Promise.resolve('r'));
		latest.run(load, { onResult: vi.fn(), onError: vi.fn(), onStart }, true);
		expect(load).toHaveBeenCalledTimes(1);
		expect(onStart).toHaveBeenCalledTimes(1);
	});

	it('une fonction de chargement qui lève est rendue comme une erreur', async () => {
		const latest = createLatestRequest<string>(0);
		const onError = vi.fn();
		latest.run(
			() => {
				throw new Error('synchrone');
			},
			{ onResult: vi.fn(), onError },
		);
		await vi.runAllTimersAsync();
		expect(onError).toHaveBeenCalledTimes(1);
	});
});
