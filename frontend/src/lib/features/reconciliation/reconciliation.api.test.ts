// Story 8-4 T7.5 — Test unitaire Vitest sur le client API reconciliation.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	acceptProposals,
	getProposals,
	rejectProposals,
} from './reconciliation.api';

function fakeJwt(): string {
	const part = btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' }))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
	const payload = btoa(
		JSON.stringify({ sub: '1', role: 'Comptable', exp: 9999999999 }),
	)
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
	return `${part}.${payload}.sig`;
}

describe('reconciliation.api', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		authState.clearSession();
		authState.login(fakeJwt(), 'refresh-uuid', 900);
		mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ proposals: [], accepted: [], failed: [], rejected: [] }),
			headers: new Headers(),
		} as Response);
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	it('getProposals appelle GET avec bankAccountId + limit', async () => {
		await getProposals(42);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toContain('/api/v1/reconciliation/proposals');
		expect(url).toContain('bankAccountId=42');
		expect(url).toContain('limit=100');
		expect(init.method).toBe('GET');
	});

	it('acceptProposals envoie body JSON avec proposals[]', async () => {
		await acceptProposals(7, [{ bankTransactionId: 1, invoiceId: 2 }]);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toContain('/api/v1/reconciliation/accept');
		expect(init.method).toBe('POST');
		const body = JSON.parse(init.body as string);
		expect(body.bankAccountId).toBe(7);
		expect(body.proposals).toEqual([{ bankTransactionId: 1, invoiceId: 2 }]);
	});

	it('rejectProposals envoie body JSON avec bankTransactionIds[]', async () => {
		await rejectProposals(7, [10, 11]);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toContain('/api/v1/reconciliation/reject');
		expect(init.method).toBe('POST');
		const body = JSON.parse(init.body as string);
		expect(body.bankAccountId).toBe(7);
		expect(body.bankTransactionIds).toEqual([10, 11]);
	});
});
