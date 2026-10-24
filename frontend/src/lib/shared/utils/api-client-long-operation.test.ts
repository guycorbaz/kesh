// Story 31-1b (AC18b, #503) — les appels longs (export et import d'une
// sauvegarde complète) échappent au délai de 30 s du client HTTP, sans rejeu.
//
// Horloge simulée : le délai du client est un `setTimeout` qui abandonne le
// `fetch` par son `AbortSignal`. Le `fetch` simulé ne répond jamais et ne
// rejette qu'à l'abandon — exactement ce que fait un serveur qui assemble une
// archive de plusieurs Gio avant d'envoyer ses en-têtes.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('$lib/shared/utils/api-health.svelte', () => ({
	apiHealth: { isDegraded: false, setDegraded: vi.fn(), clearDegraded: vi.fn() },
	HEALTH_POLL_INTERVAL_MS: 5000,
}));

import { apiClient, LONG_OPERATION_TIMEOUT_MS } from './api-client';
import { authState } from '$lib/app/stores/auth.svelte';

/** `fetch` qui ne répond jamais et rejette en `AbortError` à l'abandon. */
function fetchQuiPend(): ReturnType<typeof vi.fn> {
	return vi.fn(
		(_url: string, init: RequestInit) =>
			new Promise<Response>((_resolve, reject) => {
				init.signal?.addEventListener('abort', () =>
					reject(new DOMException('The operation was aborted.', 'AbortError')),
				);
			}),
	);
}

/** Suit l'issue d'une promesse sans l'attendre (rejet détaché). */
function suivre<T>(p: Promise<T>): { etat: () => 'pending' | 'resolved' | 'rejected'; erreur: () => unknown } {
	let etat: 'pending' | 'resolved' | 'rejected' = 'pending';
	let erreur: unknown;
	p.then(
		() => {
			etat = 'resolved';
		},
		(e) => {
			etat = 'rejected';
			erreur = e;
		},
	);
	return { etat: () => etat, erreur: () => erreur };
}

const MINUTE = 60_000;

describe('api-client — appels longs (Story 31-1b, AC18b)', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		vi.useFakeTimers();
		authState.clearSession();
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 900 });
		// Les délais de rejeu de production (300 ms … 10 s) : un rejeu se verrait.
		delete (window as unknown as { __KESH_RETRY_DELAYS?: readonly number[] }).__KESH_RETRY_DELAYS;
		mockFetch = fetchQuiPend();
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	it('LONG_OPERATION_TIMEOUT_MS vaut une heure', () => {
		expect(LONG_OPERATION_TIMEOUT_MS).toBe(60 * MINUTE);
	});

	it('par défaut, un appel est abandonné à 30 s (TIMEOUT)', async () => {
		const appel = suivre(apiClient.postFormData('/api/v1/admin/full-import', new FormData()));
		await vi.advanceTimersByTimeAsync(29_999);
		expect(appel.etat()).toBe('pending');
		await vi.advanceTimersByTimeAsync(1);
		expect(appel.etat()).toBe('rejected');
		expect(appel.erreur()).toMatchObject({ code: 'TIMEOUT', status: 0 });
	});

	it('avec LONG_OPERATION_TIMEOUT_MS, un POST n’est abandonné ni à 30 s ni à 59 min, mais à 60 min', async () => {
		const appel = suivre(
			apiClient.postFormData('/api/v1/admin/full-import', new FormData(), {
				timeoutMs: LONG_OPERATION_TIMEOUT_MS,
				retry: false,
			}),
		);
		await vi.advanceTimersByTimeAsync(30_000);
		expect(appel.etat()).toBe('pending');
		await vi.advanceTimersByTimeAsync(59 * MINUTE - 30_000);
		expect(appel.etat()).toBe('pending');
		await vi.advanceTimersByTimeAsync(MINUTE);
		expect(appel.etat()).toBe('rejected');
		expect(appel.erreur()).toMatchObject({ code: 'TIMEOUT' });
		expect(mockFetch).toHaveBeenCalledTimes(1);
	});

	it('avec LONG_OPERATION_TIMEOUT_MS, un getBlob n’est pas abandonné à 59 min, l’est à 60 min', async () => {
		const appel = suivre(
			apiClient.getBlob('/api/v1/admin/full-export', {
				timeoutMs: LONG_OPERATION_TIMEOUT_MS,
				retry: false,
			}),
		);
		await vi.advanceTimersByTimeAsync(59 * MINUTE);
		expect(appel.etat()).toBe('pending');
		await vi.advanceTimersByTimeAsync(MINUTE);
		expect(appel.etat()).toBe('rejected');
		expect(appel.erreur()).toMatchObject({ code: 'TIMEOUT' });
	});

	it('getBlob avec retry: false n’est PAS rejoué sur TIMEOUT (un seul fetch)', async () => {
		mockFetch.mockRejectedValue(new DOMException('The operation was aborted.', 'AbortError'));
		const appel = suivre(apiClient.getBlob('/api/v1/admin/full-export', { retry: false }));
		// Plus que la somme des délais de rejeu de production (14,3 s).
		await vi.advanceTimersByTimeAsync(20_000);
		expect(appel.etat()).toBe('rejected');
		expect(appel.erreur()).toMatchObject({ code: 'TIMEOUT' });
		expect(mockFetch).toHaveBeenCalledTimes(1);
	});

	it('témoin : getBlob par défaut est rejoué sur TIMEOUT (cinq fetch)', async () => {
		mockFetch.mockRejectedValue(new DOMException('The operation was aborted.', 'AbortError'));
		const appel = suivre(apiClient.getBlob('/api/v1/admin/full-export'));
		await vi.advanceTimersByTimeAsync(20_000);
		expect(appel.etat()).toBe('rejected');
		expect(mockFetch).toHaveBeenCalledTimes(5);
	});
});
