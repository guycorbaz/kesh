// Story 8-4 T7.5 — Test unitaire Vitest sur le client API reconciliation.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	acceptProposals,
	getProposals,
	manualMatchTransaction,
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
			json: () =>
				Promise.resolve({
					proposals: [],
					hasMore: false,
					accepted: [],
					failed: [],
					rejected: [],
				}),
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

	// Story 8-5a-base FR45 — manual match.
	it('manualMatchTransaction appelle POST /reconciliation/manual sans bankLedgerAccountId', async () => {
		await manualMatchTransaction(17, 42, 6810, 'Frais TWINT mai', '2026-05-15');
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toContain('/api/v1/reconciliation/manual');
		expect(init.method).toBe('POST');
		const body = JSON.parse(init.body as string);
		expect(body.bankAccountId).toBe(17);
		expect(body.bankTransactionId).toBe(42);
		expect(body.counterpartyAccountId).toBe(6810);
		expect(body.description).toBe('Frais TWINT mai');
		expect(body.valueDate).toBe('2026-05-15');
		// Démarcation explicite vs spec 8-5a unifiée : PAS de
		// `bankLedgerAccountId` dans le body (résolu serveur-side via
		// `bank_account.journal_account_id` foundation 8-5a-zero).
		expect(body.bankLedgerAccountId).toBeUndefined();
	});

	it('manualMatchTransaction omet description / valueDate quand absents', async () => {
		await manualMatchTransaction(17, 42, 6810);
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const body = JSON.parse(init.body as string);
		expect(body.description).toBeUndefined();
		expect(body.valueDate).toBeUndefined();
		// Verbe HTTP doit rester POST (cf. Pass 4 Sonnet F1 + Pass 5 Haiku
		// M1 — naming clarification).
		expect(init.method).toBe('POST');
	});
});
