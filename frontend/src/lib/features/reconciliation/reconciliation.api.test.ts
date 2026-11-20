// Story 8-4 T7.5 — Test unitaire Vitest sur le client API reconciliation.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	acceptProposals,
	getGroupCandidates,
	getLinkCandidates,
	getProposals,
	linkTransaction,
	manualMatchTransaction,
	rejectProposals,
	splitTransaction,
	unrejectTransaction,
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
		authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
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

	it('acceptProposals envoie body JSON avec proposals[] + type="invoice"', async () => {
		await acceptProposals(7, [
			{ type: 'invoice', bankTransactionId: 1, invoiceId: 2 },
		]);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toContain('/api/v1/reconciliation/accept');
		expect(init.method).toBe('POST');
		const body = JSON.parse(init.body as string);
		expect(body.bankAccountId).toBe(7);
		// Story 8-5a-bis Q2 breaking — chaque proposal porte un discriminator `type`.
		expect(body.proposals).toEqual([
			{ type: 'invoice', bankTransactionId: 1, invoiceId: 2 },
		]);
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

	// Story 28-5b (#526, V5) — annuler le rejet : POST sans corps utile, réponse 204.
	it('unrejectTransaction poste {} à /reconciliation/transactions/{id}/unreject et accepte un 204', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 204,
			json: () => Promise.reject(new Error('204 sans corps')),
			headers: new Headers(),
		} as unknown as Response);
		await expect(unrejectTransaction(7)).resolves.toBeUndefined();
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toMatch(/\/api\/v1\/reconciliation\/transactions\/7\/unreject$/);
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({});
	});

	// Story 29-8 (#670, AC4) — rattacher une transaction au règlement saisi depuis la fiche.
	it('getLinkCandidates appelle GET /reconciliation/transactions/{id}/link-candidates', async () => {
		await getLinkCandidates(7);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toMatch(/\/api\/v1\/reconciliation\/transactions\/7\/link-candidates$/);
		expect(init.method).toBe('GET');
	});

	it('linkTransaction poste { settlementId } à /reconciliation/transactions/{id}/link', async () => {
		await linkTransaction(7, 4);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toMatch(/\/api\/v1\/reconciliation\/transactions\/7\/link$/);
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ settlementId: 4 });
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

	// Story 19-5 — tag analytique document-level dans le body manual.
	it('manualMatchTransaction inclut projectId quand fourni, l’omet si null', async () => {
		await manualMatchTransaction(17, 42, 6810, undefined, undefined, 7);
		let body = JSON.parse(mockFetch.mock.calls[0][1].body as string);
		expect(body.projectId).toBe(7);

		mockFetch.mockClear();
		await manualMatchTransaction(17, 42, 6810, undefined, undefined, null);
		body = JSON.parse(mockFetch.mock.calls[0][1].body as string);
		expect(body.projectId).toBeUndefined();
	});

	// Story 19-5 — tag analytique par ligne de ventilation dans le body split.
	it('splitTransaction propage projectId par ligne', async () => {
		await splitTransaction(17, 42, [
			{ counterpartyAccountId: 5000, amount: '100', description: 'A', projectId: 3 },
			{ counterpartyAccountId: 6900, amount: '50', description: 'B' },
		]);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toContain('/api/v1/reconciliation/split');
		const body = JSON.parse(init.body as string);
		expect(body.splits[0].projectId).toBe(3);
		expect(body.splits[1].projectId).toBeUndefined();
	});

	// Story 43-3 (#823) — la recherche des factures d'un règlement groupé (Story 43-2).
	it('getGroupCandidates : chemin, q taillé, exclude joint par des virgules, sans rejeu', async () => {
		await getGroupCandidates(41, { q: '  Exemple SA ', exclude: [12, 15] });
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe(
			'/api/v1/reconciliation/transactions/41/group-candidates?q=Exemple+SA&exclude=12%2C15',
		);
		expect(init.method).toBe('GET');
	});

	it('getGroupCandidates : sans terme ni exclusion, aucun paramètre ; limit passé tel quel', async () => {
		await getGroupCandidates(41);
		expect(mockFetch.mock.calls[0][0]).toBe('/api/v1/reconciliation/transactions/41/group-candidates');
		mockFetch.mockClear();
		await getGroupCandidates(41, { q: '   ', exclude: [], limit: 100 });
		expect(mockFetch.mock.calls[0][0]).toBe(
			'/api/v1/reconciliation/transactions/41/group-candidates?limit=100',
		);
	});

	it('getGroupCandidates rend direction: null et une liste vide telles quelles', async () => {
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 200,
			json: () =>
				Promise.resolve({
					direction: null,
					transactionAmount: '20.00',
					maxDifference: '5.00',
					candidates: [],
					hasMore: false,
				}),
			headers: new Headers(),
		} as Response);
		const r = await getGroupCandidates(41);
		expect(r.direction).toBeNull();
		expect(r.candidates).toEqual([]);
	});

	// L'élément d'`accepted[]` d'un groupe (43-2, M1) : cinq champs additifs, lus tels quels.
	it('acceptProposals : invoice_group envoyé, élément d’accepted[] d’un groupe désérialisé', async () => {
		const element = {
			bankTransactionId: 41,
			invoiceId: 0,
			supplierInvoiceId: null,
			journalEntryId: 905,
			score: { total: 1, amountScore: 0, referenceScore: 0, contactScore: 0 },
			difference: '-0.4000',
			roundingDifference: '0.0000',
			gap: '-0.40',
			differenceAccountId: 6940,
			settledInvoices: [
				{ invoiceId: 12, amount: '500.0000' },
				{ invoiceId: 15, amount: '300.0000' },
			],
		};
		mockFetch.mockResolvedValueOnce({
			ok: true,
			status: 200,
			json: () => Promise.resolve({ accepted: [element], failed: [] }),
			headers: new Headers(),
		} as Response);
		const r = await acceptProposals(3, [
			{ type: 'invoice_group', bankTransactionId: 41, invoiceIds: [12, 15], differenceAccountId: 6940 },
		]);
		const body = JSON.parse(mockFetch.mock.calls[0][1].body as string);
		expect(body.proposals[0]).toEqual({
			type: 'invoice_group',
			bankTransactionId: 41,
			invoiceIds: [12, 15],
			differenceAccountId: 6940,
		});
		const a = r.accepted[0];
		expect(a.gap).toBe('-0.40');
		expect(a.roundingDifference).toBe('0.0000');
		expect(a.difference).toBe('-0.4000');
		expect(a.differenceAccountId).toBe(6940);
		expect(a.settledInvoices?.map((s) => s.invoiceId)).toEqual([12, 15]);
	});
});
