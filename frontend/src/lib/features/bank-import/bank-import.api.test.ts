// Story 8-1b T7.8 — Test unitaire Vitest sur le client API bank-import.
//
// Vérifie que :
// - `previewBankImport` envoie un FormData (Content-Type multipart, pas JSON)
// - `createBankImport` ajoute `confirmBalanceMismatch` quand demandé
// - Le browser pose un `Content-Type: multipart/form-data; boundary=...`
//   (M3 §api-client : on NE force PAS `application/json` pour FormData).

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	createBankImport,
	previewBankImport,
} from './bank-import.api';

function fakeJwt(): string {
	const part = btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' }))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
	const payload = btoa(JSON.stringify({ sub: '1', role: 'Comptable', exp: 9999999999 }))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
	return `${part}.${payload}.sig`;
}

const FAKE_FILE = new File(['<Document/>'], 'stmt.xml', { type: 'application/xml' });

describe('bank-import.api', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		authState.clearSession();
		authState.login(fakeJwt(), 'refresh-uuid', 900);
		mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			status: 201,
			json: () => Promise.resolve({ id: 1 }),
			headers: new Headers(),
		} as Response);
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	it('previewBankImport envoie un FormData sans forcer application/json', async () => {
		await previewBankImport(FAKE_FILE, 42);
		expect(mockFetch).toHaveBeenCalled();
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(init.body).toBeInstanceOf(FormData);
		const headers = init.headers as Record<string, string>;
		// Le browser pose lui-même Content-Type: multipart/form-data; boundary=...
		// L'api-client NE force PAS application/json pour FormData (§api-client).
		expect(headers['Content-Type']).toBeUndefined();
		expect(headers['Authorization']).toMatch(/^Bearer /);
	});

	it('createBankImport ajoute confirmBalanceMismatch quand demandé', async () => {
		await createBankImport(FAKE_FILE, 42, { confirmBalanceMismatch: true });
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('confirmBalanceMismatch')).toBe('true');
		expect(form.get('bankAccountId')).toBe('42');
	});

	it("createBankImport sans confirm n'ajoute pas le champ", async () => {
		await createBankImport(FAKE_FILE, 42);
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('confirmBalanceMismatch')).toBeNull();
	});

	// Story 8-3 T7.5 — 4 nouveaux tests sur les flags étendus.
	it('createBankImport ajoute confirmDuplicateFile quand demandé', async () => {
		await createBankImport(FAKE_FILE, 42, { confirmDuplicateFile: true });
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('confirmDuplicateFile')).toBe('true');
		expect(form.get('confirmDuplicateLines')).toBeNull();
	});

	it('createBankImport sérialise confirmDuplicateLines (skip|import)', async () => {
		await createBankImport(FAKE_FILE, 42, { confirmDuplicateLines: 'import' });
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('confirmDuplicateLines')).toBe('import');
	});

	// L9 (Pass 1 review) — la valeur par défaut backend est `Skip`. Le
	// frontend n'envoie PAS le champ quand l'utilisateur a explicitement
	// choisi `'skip'` (équivalent sémantique à omettre le champ).
	it("createBankImport n'envoie pas confirmDuplicateLines='skip' (default backend)", async () => {
		await createBankImport(FAKE_FILE, 42, { confirmDuplicateLines: 'skip' });
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('confirmDuplicateLines')).toBeNull();
	});

	// L1 (Pass 1 review) — bankProfileId === 0 doit être appended (defensive
	// même si DB AUTO_INCREMENT commence à 1).
	it('createBankImport sérialise bankProfileId === 0 (truthy guard L1)', async () => {
		await createBankImport(FAKE_FILE, 42, { bankProfileId: 0 });
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('bankProfileId')).toBe('0');
	});

	it('createBankImport ajoute confirmPartialImport + bankProfileId combinés', async () => {
		await createBankImport(FAKE_FILE, 42, {
			confirmPartialImport: true,
			bankProfileId: 77,
		});
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('confirmPartialImport')).toBe('true');
		expect(form.get('bankProfileId')).toBe('77');
	});

	it('previewBankImport propage le bankProfileId explicite (KF #70)', async () => {
		await previewBankImport(FAKE_FILE, 42, { bankProfileId: 99 });
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		const form = init.body as FormData;
		expect(form.get('bankProfileId')).toBe('99');
	});
});
