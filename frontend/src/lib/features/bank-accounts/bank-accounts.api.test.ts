// Story 8-5a-zero — Tests Vitest pour le client API bank-accounts.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import { listBankAccounts, updateBankAccountJournalLink } from './bank-accounts.api';

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

describe('bank-accounts.api', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		authState.clearSession();
		authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
		mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 200,
			json: () =>
				Promise.resolve({
					id: 17,
					bankName: 'UBS',
					iban: 'CH4431999123000889012',
					qrIban: null,
					isPrimary: true,
					journalAccountId: 1020,
					version: 4,
				}),
			headers: new Headers(),
		} as Response);
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	it('listBankAccounts appelle GET sur /api/v1/bank-accounts', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 200,
			json: () => Promise.resolve([]),
			headers: new Headers(),
		} as Response);
		await listBankAccounts();
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/bank-accounts');
		expect(init.method).toBe('GET');
	});

	it('updateBankAccountJournalLink envoie PATCH avec body camelCase + version', async () => {
		await updateBankAccountJournalLink(17, 1020, 3);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/bank-accounts/17');
		expect(init.method).toBe('PATCH');
		const body = JSON.parse(init.body as string);
		expect(body).toEqual({ journalAccountId: 1020, version: 3 });
	});

	it('updateBankAccountJournalLink supporte journalAccountId=null pour délier', async () => {
		await updateBankAccountJournalLink(17, null, 4);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/bank-accounts/17');
		expect(init.method).toBe('PATCH');
		const body = JSON.parse(init.body as string);
		expect(body).toEqual({ journalAccountId: null, version: 4 });
	});
});
