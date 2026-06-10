// Story 17-3d — Tests Vitest pour `uploadFullImport`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import { uploadFullImport } from './admin-restore.api';

describe('admin-restore.api — Story 17-3d', () => {
	beforeEach(() => {
		authState.clearSession();
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 900 });
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	it('POST /api/v1/admin/full-import avec FormData (champ file) + retourne la réponse', async () => {
		let capturedBody: unknown;
		const mockFetch = vi.fn().mockImplementation((_url: string, opts: RequestInit) => {
			capturedBody = opts.body;
			return Promise.resolve({
				ok: true,
				status: 200,
				json: () =>
					Promise.resolve({
						backupCreated: true,
						tablesRestored: 21,
						rowsRestored: 42,
						sourceVersion: '0.1.8',
						sessionInvalidated: true,
					}),
				headers: new Headers({ 'content-type': 'application/json' }),
			} as unknown as Response);
		});
		vi.stubGlobal('fetch', mockFetch);

		const file = new File([new Uint8Array([0x50, 0x4b])], 'inst.keshbackup');
		const res = await uploadFullImport(file);

		expect(mockFetch).toHaveBeenCalledTimes(1);
		expect(mockFetch.mock.calls[0][0] as string).toContain('/api/v1/admin/full-import');
		expect(mockFetch.mock.calls[0][1].method).toBe('POST');
		// Le corps est un FormData contenant le champ `file`.
		expect(capturedBody).toBeInstanceOf(FormData);
		expect((capturedBody as FormData).get('file')).toBeInstanceOf(File);
		// Réponse parsée.
		expect(res.sessionInvalidated).toBe(true);
		expect(res.tablesRestored).toBe(21);
	});

	it('rejette quand le backend retourne 409 (version incompatible)', async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 409,
			json: () =>
				Promise.resolve({
					error: {
						code: 'IMPORT_VERSION_INCOMPATIBLE',
						message: 'Version incompatible.',
						details: { sourceMinRequired: '99.0.0', binaryVersion: '0.1.8' },
					},
				}),
			headers: new Headers({ 'content-type': 'application/json' }),
		} as unknown as Response);
		vi.stubGlobal('fetch', mockFetch);

		const file = new File([new Uint8Array([0x50, 0x4b])], 'inst.keshbackup');
		await expect(uploadFullImport(file)).rejects.toThrow();
	});
});
