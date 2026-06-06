// Story 17-2b — Tests Vitest pour le client API api-keys. Calque
// `bank-accounts.api.test.ts`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import { listApiKeys, createApiKey, revokeApiKey } from './api-keys.api';

describe('api-keys.api', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		authState.clearSession();
		authState.login({ userId: '1', username: 'test', role: 'Comptable', expiresIn: 900 });
		mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	it('listApiKeys appelle GET sur /api/v1/settings/api-keys', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 200,
			json: () => Promise.resolve([]),
			headers: new Headers(),
		} as Response);
		await listApiKeys();
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/settings/api-keys');
		expect(init.method).toBe('GET');
	});

	it('createApiKey envoie POST avec body camelCase et retourne le secret une fois', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 201,
			json: () =>
				Promise.resolve({
					id: 7,
					name: 'CI bot',
					scope: 'read-write',
					createdAt: '2026-06-06T10:00:00',
					key: 'kesh_pat_abc123',
				}),
			headers: new Headers(),
		} as Response);
		const created = await createApiKey({ name: 'CI bot', scope: 'read-write' });
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/settings/api-keys');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ name: 'CI bot', scope: 'read-write' });
		expect(created.key).toBe('kesh_pat_abc123');
	});

	it('createApiKey transmet expiresAt quand fourni', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 201,
			json: () =>
				Promise.resolve({
					id: 8,
					name: 'temp',
					scope: 'read',
					createdAt: '2026-06-06T10:00:00',
					key: 'kesh_pat_x',
				}),
			headers: new Headers(),
		} as Response);
		await createApiKey({ name: 'temp', scope: 'read', expiresAt: '2027-01-01T00:00:00Z' });
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(JSON.parse(init.body as string)).toEqual({
			name: 'temp',
			scope: 'read',
			expiresAt: '2027-01-01T00:00:00Z',
		});
	});

	it('revokeApiKey envoie DELETE avec version dans le body', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 204,
			json: () => Promise.reject(new Error('no body')),
			headers: new Headers(),
		} as Response);
		await revokeApiKey(7, 3);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/settings/api-keys/7');
		expect(init.method).toBe('DELETE');
		expect(JSON.parse(init.body as string)).toEqual({ version: 3 });
	});
});
