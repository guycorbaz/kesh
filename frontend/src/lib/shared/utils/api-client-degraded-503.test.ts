// Story 37-2 (AC-5) — l'option `degradedOn503: false` de `CallOptions` : un 503
// (refus métier explicite, p. ex. `RATE_SOURCE_UNAVAILABLE` quand la source des cours
// de change ne répond pas) n'allume pas le bandeau « service dégradé ». Elle ne
// couvre QUE le 503 : un TIMEOUT ou une coupure du client allument toujours le
// bandeau, par la branche `catch` — comportement voulu (le serveur ne répond pas).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

const { setDegraded, clearDegraded } = vi.hoisted(() => ({
	setDegraded: vi.fn(),
	clearDegraded: vi.fn(),
}));
vi.mock('$lib/shared/utils/api-health.svelte', () => ({
	apiHealth: { isDegraded: false, setDegraded, clearDegraded },
	HEALTH_POLL_INTERVAL_MS: 5000,
}));

import { apiClient } from './api-client';
import { authState } from '$lib/app/stores/auth.svelte';

function reponse503(): Response {
	return new Response(
		JSON.stringify({
			error: { code: 'RATE_SOURCE_UNAVAILABLE', message: 'La source ne répond pas.' },
		}),
		{ status: 503, headers: { 'Content-Type': 'application/json' } },
	);
}

describe('api-client — degradedOn503 (Story 37-2, AC-5)', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		vi.clearAllMocks();
		authState.clearSession();
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 900 });
		(window as unknown as { __KESH_RETRY_DELAYS?: readonly number[] }).__KESH_RETRY_DELAYS = [0];
		mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		delete (window as unknown as { __KESH_RETRY_DELAYS?: readonly number[] }).__KESH_RETRY_DELAYS;
		authState.clearSession();
	});

	it('par défaut, un 503 allume le bandeau', async () => {
		mockFetch.mockResolvedValue(reponse503());
		await expect(apiClient.post('/api/v1/exchange-rates/fetch', {})).rejects.toMatchObject({
			code: 'RATE_SOURCE_UNAVAILABLE',
			status: 503,
		});
		expect(setDegraded).toHaveBeenCalledTimes(1);
	});

	it('avec degradedOn503: false, un 503 rend son erreur sans allumer le bandeau', async () => {
		mockFetch.mockResolvedValue(reponse503());
		await expect(
			apiClient.post('/api/v1/exchange-rates/fetch', {}, { degradedOn503: false }),
		).rejects.toMatchObject({ code: 'RATE_SOURCE_UNAVAILABLE', status: 503 });
		expect(setDegraded).not.toHaveBeenCalled();
	});

	it('avec degradedOn503: false, un TIMEOUT allume quand même le bandeau (voulu)', async () => {
		mockFetch.mockRejectedValue(new DOMException('The operation was aborted.', 'AbortError'));
		await expect(
			apiClient.post('/api/v1/exchange-rates/fetch', {}, { degradedOn503: false }),
		).rejects.toMatchObject({ code: 'TIMEOUT' });
		expect(setDegraded).toHaveBeenCalledTimes(1);
	});

	it('avec degradedOn503: false, une coupure réseau allume quand même le bandeau (voulu)', async () => {
		mockFetch.mockRejectedValue(new TypeError('Failed to fetch'));
		await expect(
			apiClient.post('/api/v1/exchange-rates/fetch', {}, { degradedOn503: false }),
		).rejects.toMatchObject({ code: 'NETWORK_ERROR' });
		expect(setDegraded).toHaveBeenCalledTimes(1);
	});
});
