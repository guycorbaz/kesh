/**
 * L'API du journal d'audit — Story 25-1c-b1 (#378).
 *
 * Patron : `features/export/exports.api.test.ts` — `fetch` intercepté par
 * `vi.stubGlobal`, l'URL appelée relue. ⚠️ C'est le premier test du dépôt qui
 * observe l'URL passée à `getBlob`.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import { exportAuditLogCsv, getAuditLogVocabulary, listAuditLog } from './audit-log.api';

let fetchMock: ReturnType<typeof vi.fn>;

function jsonResponse(body: unknown) {
	return {
		ok: true,
		status: 200,
		headers: new Headers({ 'content-type': 'application/json' }),
		json: () => Promise.resolve(body),
		text: () => Promise.resolve(JSON.stringify(body)),
	} as unknown as Response;
}

beforeEach(() => {
	authState.clearSession();
	authState.login({ userId: '1', username: 'test', role: 'Comptable', expiresIn: 900 });
	fetchMock = vi.fn();
	vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
	vi.unstubAllGlobals();
	authState.clearSession();
});

function calledUrl(): URL {
	return new URL(fetchMock.mock.calls[0][0] as string, 'http://localhost');
}

describe('audit-log.api', () => {
	it("n'envoie pas un paramètre vide", async () => {
		fetchMock.mockResolvedValue(jsonResponse({ items: [], total: 0, offset: 0, limit: 50 }));
		await listAuditLog({ action: '', entityType: '', dateFrom: '2026-09-01' });
		const url = calledUrl();
		expect(url.pathname).toBe('/api/v1/audit-log');
		expect(url.searchParams.has('action')).toBe(false);
		expect(url.searchParams.has('entityType')).toBe(false);
		expect(url.searchParams.get('dateFrom')).toBe('2026-09-01');
	});

	it('getAuditLogVocabulary appelle la route de vocabulaire', async () => {
		fetchMock.mockResolvedValue(jsonResponse({ entityTypes: [], actions: [] }));
		await getAuditLogVocabulary();
		expect(calledUrl().pathname).toBe('/api/v1/audit-log/vocabulary');
	});

	it('⛔ l’URL passée à getBlob porte les filtres, sans offset ni limit (mutation : filtres non sérialisés)', async () => {
		// Une SOUS-CLASSE, jamais `Object.assign(URL, …)` : muter le vrai `URL`
		// survivrait à `vi.unstubAllGlobals()` (revue P1). `new URL(...)` reste
		// utilisable plus bas.
		class FakeURL extends URL {
			static createObjectURL = vi.fn().mockReturnValue('blob:x');
			static revokeObjectURL = vi.fn();
		}
		vi.stubGlobal('URL', FakeURL);
		vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});
		fetchMock.mockResolvedValue({
			ok: true,
			status: 200,
			blob: () => Promise.resolve(new Blob(['a'])),
			headers: new Headers({
				'content-disposition': 'attachment; filename="kesh-journal-audit-ci-2026-09-16.csv"',
			}),
		} as unknown as Response);
		let named = '';
		vi.spyOn(document.body, 'appendChild').mockImplementation(<T extends Node>(n: T) => {
			named = (n as unknown as HTMLAnchorElement).download;
			return n;
		});

		await exportAuditLogCsv({
			dateFrom: '2026-09-01',
			dateTo: '2026-09-16',
			entityType: 'contact',
			entityId: 5,
			action: 'contact.created',
			offset: 100,
			limit: 50,
		});

		const url = new URL(fetchMock.mock.calls[0][0] as string, 'http://localhost');
		expect(url.pathname).toBe('/api/v1/audit-log/export.csv');
		expect(Object.fromEntries(url.searchParams)).toEqual({
			dateFrom: '2026-09-01',
			dateTo: '2026-09-16',
			entityType: 'contact',
			entityId: '5',
			action: 'contact.created',
		});
		// Le nom vient de `Content-Disposition`.
		expect(named).toBe('kesh-journal-audit-ci-2026-09-16.csv');
	});
});
