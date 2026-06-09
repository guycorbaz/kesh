// Story 17-3b — Tests Vitest pour `downloadFullExport` + parser Content-Disposition.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import { downloadFullExport, parseContentDispositionFilename } from './admin-backup.api';

describe('admin-backup.api — Story 17-3b', () => {
	describe('parseContentDispositionFilename', () => {
		it('RFC 6266 ASCII : filename="…"', () => {
			expect(
				parseContentDispositionFilename(
					'attachment; filename="kesh-installation-2026-06-09.keshbackup"',
				),
			).toBe('kesh-installation-2026-06-09.keshbackup');
		});

		it('RFC 5987 UTF-8 percent-decoded avec tag langue (fr-CH)', () => {
			expect(
				parseContentDispositionFilename(
					"attachment; filename=\"kesh-installation-2026-06-09.keshbackup\"; filename*=UTF-8'fr-CH'kesh-installation-2026-06-09.keshbackup",
				),
			).toBe('kesh-installation-2026-06-09.keshbackup');
		});

		it('null/empty → null', () => {
			expect(parseContentDispositionFilename(null)).toBeNull();
			expect(parseContentDispositionFilename('')).toBeNull();
		});

		it('header sans filename → null', () => {
			expect(parseContentDispositionFilename('attachment')).toBeNull();
		});

		it('filename*=UTF-8\'\' (valeur vide) → null (lookahead négatif)', () => {
			expect(parseContentDispositionFilename("attachment; filename*=UTF-8''")).toBeNull();
		});
	});

	describe('downloadFullExport', () => {
		let createObjectURLSpy: ReturnType<typeof vi.fn>;
		let revokeObjectURLSpy: ReturnType<typeof vi.fn>;

		beforeEach(() => {
			authState.clearSession();
			authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 900 });
			createObjectURLSpy = vi.fn().mockReturnValue('blob:http://localhost/keshbackup');
			revokeObjectURLSpy = vi.fn();
			vi.stubGlobal('URL', {
				createObjectURL: createObjectURLSpy,
				revokeObjectURL: revokeObjectURLSpy,
			});
		});

		afterEach(() => {
			vi.unstubAllGlobals();
			authState.clearSession();
		});

		it('appelle GET /api/v1/admin/full-export + déclenche le download via Blob', async () => {
			const zipBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
			const mockBlob = new Blob([zipBytes], { type: 'application/octet-stream' });
			const mockFetch = vi.fn().mockResolvedValue({
				ok: true,
				status: 200,
				blob: () => Promise.resolve(mockBlob),
				headers: new Headers({
					'content-type': 'application/octet-stream',
					'content-disposition':
						'attachment; filename="kesh-installation-2026-06-09.keshbackup"',
				}),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			await downloadFullExport();

			expect(mockFetch).toHaveBeenCalledTimes(1);
			expect(mockFetch.mock.calls[0][0] as string).toContain('/api/v1/admin/full-export');
			expect(createObjectURLSpy).toHaveBeenCalledOnce();
			expect(revokeObjectURLSpy).toHaveBeenCalledOnce();
		});

		it('rejette quand le backend retourne 403 (anti-PAT / non-Admin) — pas de download', async () => {
			const mockFetch = vi.fn().mockResolvedValue({
				ok: false,
				status: 403,
				json: () =>
					Promise.resolve({
						error: { code: 'API_KEY_MANAGEMENT_FORBIDDEN', message: 'Interdit.' },
					}),
				headers: new Headers({ 'content-type': 'application/json' }),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			await expect(downloadFullExport()).rejects.toThrow();
			expect(createObjectURLSpy).not.toHaveBeenCalled();
		});
	});
});
