// Story 8-5a-zero — Tests Vitest pour le client API bank-accounts.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	closeBankAccount,
	listBankAccounts,
	reopenBankAccount,
	updateBankAccountJournalLink,
} from './bank-accounts.api';

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
					description: null,
					iban: 'CH4431999123000889012',
					qrIban: null,
					isPrimary: true,
					journalAccountId: 1020,
					version: 4,
					archived: false,
					currentBalance: null,
					statementClosingBalance: null,
					statementDate: null,
					ledgerBalanceAtStatement: null,
					lastTransactionDate: null,
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

// Story 25-6-a (#389) — les trois champs du relevé traversent le mappage TS ; une
// faute de frappe dans une clé `Raw` les rendrait toujours `null`, tous gates verts.
describe('bank-accounts.api — champs du relevé dont la période se termine le plus tard (25-6-a)', () => {
	beforeEach(() => {
		authState.clearSession();
		authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
	});
	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	function stubJson(body: unknown) {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				status: 200,
				json: () => Promise.resolve(body),
				headers: new Headers(),
			} as Response),
		);
	}

	it('listBankAccounts rend les chaînes du relevé telles quelles', async () => {
		stubJson([
			{
				id: 1,
				bankName: 'UBS',
				description: null,
				iban: 'CH4431999123000889012',
				qrIban: null,
				isPrimary: true,
				journalAccountId: 10,
				version: 1,
				archived: false,
				currentBalance: '800.0000',
				lastTransactionDate: '2026-03-10',
				statementClosingBalance: '1000.00',
				statementDate: '2026-03-31',
				ledgerBalanceAtStatement: '1000.0000',
			},
		]);
		const [a] = await listBankAccounts();
		expect(a.currentBalance).toBe('800.0000');
		expect(a.statementClosingBalance).toBe('1000.00');
		expect(a.statementDate).toBe('2026-03-31');
		expect(a.ledgerBalanceAtStatement).toBe('1000.0000');
	});

	it('une réponse de mutation sans ces champs les rend à null', async () => {
		stubJson({
			id: 1,
			bankName: 'UBS',
			description: null,
			iban: 'CH4431999123000889012',
			qrIban: null,
			isPrimary: true,
			journalAccountId: 10,
			version: 2,
			archived: false,
		});
		const a = await updateBankAccountJournalLink(1, 10, 1);
		expect(a.statementClosingBalance).toBeNull();
		expect(a.statementDate).toBeNull();
		expect(a.ledgerBalanceAtStatement).toBeNull();
	});

	// Story 26-2a (#622) — la description est mappée telle quelle ; absente
	// d'une réponse (serveur antérieur), elle vaut `null`.
	it('mappe la description, et la rend à null si la réponse l’omet', async () => {
		stubJson([
			{
				id: 1,
				bankName: 'UBS',
				description: 'Personnel',
				iban: 'CH4431999123000889012',
				qrIban: null,
				isPrimary: true,
				journalAccountId: null,
				version: 1,
				archived: false,
				currentBalance: null,
				lastTransactionDate: null,
			},
			{
				id: 2,
				bankName: 'UBS',
				iban: 'CH9300762011623852957',
				qrIban: null,
				isPrimary: false,
				journalAccountId: null,
				version: 1,
				archived: false,
				currentBalance: null,
				lastTransactionDate: null,
			},
		]);
		const [a, b] = await listBankAccounts();
		expect(a.description).toBe('Personnel');
		expect(b.description).toBeNull();
	});
});

// Story 28-4b (#642, AC11) — clôture d'un compte bancaire : le filtre des clos,
// `closedOn`, et les deux gestes. T-b1 à T-b4 de la fiche.
describe('bank-accounts.api — comptes clos (28-4b)', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	const COMPTE = {
		id: 7,
		bankName: 'UBS',
		description: null,
		iban: 'CH9300762011623852957',
		qrIban: null,
		isPrimary: false,
		journalAccountId: null,
		version: 3,
		archived: false,
		closedOn: '2026-09-30',
	};

	function reponse(body: unknown) {
		return {
			ok: true,
			status: 200,
			json: () => Promise.resolve(body),
			headers: new Headers(),
		} as Response;
	}

	beforeEach(() => {
		authState.clearSession();
		authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
		mockFetch = vi.fn().mockResolvedValue(reponse(COMPTE));
		vi.stubGlobal('fetch', mockFetch);
	});
	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	// T-b1 — les combinaisons des deux booléens (C-28-4-20) ; les appelants
	// existants (aucun argument, `false`) gardent l'URL d'avant.
	it.each([
		[undefined, undefined, '/api/v1/bank-accounts'],
		[false, false, '/api/v1/bank-accounts'],
		[true, false, '/api/v1/bank-accounts?includeArchived=true'],
		[false, true, '/api/v1/bank-accounts?includeClosed=true'],
		[true, true, '/api/v1/bank-accounts?includeArchived=true&includeClosed=true'],
	])('listBankAccounts(%s, %s) → %s', async (archives, clos, attendu) => {
		mockFetch.mockResolvedValueOnce(reponse([]));
		await listBankAccounts(archives, clos);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe(attendu);
		expect(init.method).toBe('GET');
	});

	// T-b2 — `closedOn` traverse le mappage ; absent (serveur antérieur) → null.
	it('parseBankAccount porte closedOn, et le rend à null s’il manque', async () => {
		mockFetch.mockResolvedValueOnce(reponse([COMPTE, { ...COMPTE, id: 8, closedOn: undefined }]));
		const [clos, sans] = await listBankAccounts(false, true);
		expect(clos.closedOn).toBe('2026-09-30');
		expect(sans.closedOn).toBeNull();
	});

	// T-b3 — clôture : POST, corps { version, closedOn }.
	it('closeBankAccount envoie POST /close avec version et closedOn', async () => {
		const a = await closeBankAccount(7, 3, '2026-09-30');
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/bank-accounts/7/close');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ version: 3, closedOn: '2026-09-30' });
		expect(a.closedOn).toBe('2026-09-30');
	});

	// T-b4 — réouverture : POST, corps { version } seul.
	it('reopenBankAccount envoie POST /reopen avec la seule version', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ ...COMPTE, version: 4, closedOn: null }));
		const a = await reopenBankAccount(7, 3);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/bank-accounts/7/reopen');
		expect(init.method).toBe('POST');
		expect(JSON.parse(init.body as string)).toEqual({ version: 3 });
		expect(a.closedOn).toBeNull();
	});
});
