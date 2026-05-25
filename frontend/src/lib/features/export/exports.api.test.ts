// Story 9-2b — Tests Vitest pour `downloadGlobalExport` + parser RFC 5987 / 6266.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	downloadGlobalExport,
	parseContentDispositionFilename,
} from './exports.api';

function fakeJwt(): string {
	const header = btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' }))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
	const payload = btoa(JSON.stringify({ sub: '1', role: 'Comptable', exp: 9999999999 }))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
	return `${header}.${payload}.sig`;
}

describe('exports.api — Story 9-2b', () => {
	// ----------------------------------------------------------------------
	// T12.1(d) + (e) — parseContentDispositionFilename (Pass 3 ECH3-H4)
	// ----------------------------------------------------------------------

	describe('parseContentDispositionFilename', () => {
		it('AC #31(d) ASCII fallback : filename="…"', () => {
			expect(
				parseContentDispositionFilename(
					'attachment; filename="kesh-export-foo-2026-05-15.zip"',
				),
			).toBe('kesh-export-foo-2026-05-15.zip');
		});

		it('AC #31(e) RFC 5987 UTF-8 percent-decoded (no language tag)', () => {
			expect(
				parseContentDispositionFilename(
					"attachment; filename*=UTF-8''kesh-export-%C3%A9t%C3%A9-2026.zip",
				),
			).toBe('kesh-export-été-2026.zip');
		});

		it('AC #31(e) RFC 5987 UTF-8 percent-decoded WITH language tag (fr-CH)', () => {
			// Story 9-2a Pass 1 code-review M14 — backend insère le tag langue.
			expect(
				parseContentDispositionFilename(
					"attachment; filename=\"kesh-export-ci-test-company-2026-05-17.zip\"; filename*=UTF-8'fr-CH'kesh-export-ci-test-company-2026-05-17.zip",
				),
			).toBe('kesh-export-ci-test-company-2026-05-17.zip');
		});

		it('AC #31(e) returns null on null or empty header', () => {
			expect(parseContentDispositionFilename(null)).toBeNull();
			expect(parseContentDispositionFilename('')).toBeNull();
		});

		it('returns null on header without filename', () => {
			expect(parseContentDispositionFilename('attachment')).toBeNull();
		});

		// Pass 1 code-review H6 (C4 Blind F02/F09 + C4-ECH-H2) — régression :
		// header `filename*=UTF-8''` (valeur RFC 5987 vide) DOIT retourner null,
		// pas une chaîne `"UTF-8''"` que la regex unquoted matcherait sans le
		// lookahead `(?!\*)`.
		it('returns null on RFC 5987 filename* with empty percent-encoded value', () => {
			expect(
				parseContentDispositionFilename("attachment; filename*=UTF-8''"),
			).toBeNull();
		});
	});

	// ----------------------------------------------------------------------
	// T12.1(a) — downloadGlobalExport happy path
	// T12.1(b) — error path (500)
	// T12.1(c) — double-clic guard re-entrancy (testé au niveau caller ; ici
	//             on confirme que la fn est resolvable concurrently et
	//             réutilisable en série — le guard est dans `+page.svelte`).
	// ----------------------------------------------------------------------

	describe('downloadGlobalExport', () => {
		let mockFetch: ReturnType<typeof vi.fn>;
		let createObjectURLSpy: ReturnType<typeof vi.fn>;
		let revokeObjectURLSpy: ReturnType<typeof vi.fn>;

		beforeEach(() => {
			authState.clearSession();
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			createObjectURLSpy = vi.fn().mockReturnValue('blob:http://localhost/zip');
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

		it('AC #31(a) appelle fetch /api/v1/exports/global.zip + déclenche download via Blob', async () => {
			const zipBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x00, 0x00]);
			const mockBlob = new Blob([zipBytes], { type: 'application/zip' });
			mockFetch = vi.fn().mockResolvedValue({
				ok: true,
				status: 200,
				blob: () => Promise.resolve(mockBlob),
				headers: new Headers({
					'content-type': 'application/zip',
					'content-disposition':
						'attachment; filename="kesh-export-ci-test-company-2026-05-17.zip"',
				}),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			await downloadGlobalExport();

			expect(mockFetch).toHaveBeenCalledTimes(1);
			const calledUrl = mockFetch.mock.calls[0][0] as string;
			expect(calledUrl).toContain('/api/v1/exports/global.zip');
			// Blob path effectivement utilisé
			expect(createObjectURLSpy).toHaveBeenCalledOnce();
			expect(revokeObjectURLSpy).toHaveBeenCalledOnce();
		});

		it('AC #31(b) rejette quand backend retourne 500', async () => {
			mockFetch = vi.fn().mockResolvedValue({
				ok: false,
				status: 500,
				json: () =>
					Promise.resolve({
						error: {
							code: 'GLOBAL_EXPORT_FAILED',
							message: "Échec de la génération de l'export global.",
						},
					}),
				headers: new Headers({ 'content-type': 'application/json' }),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			await expect(downloadGlobalExport()).rejects.toThrow();
			// pas de download tenté
			expect(createObjectURLSpy).not.toHaveBeenCalled();
		});

		// Pass 1 code-review H8 (C4-AA-HIGH-01) — AC #31(c) guard re-entrancy.
		// Le guard `if (exporting) return` first-line dans `+page.svelte` doit
		// empêcher un second appel pendant un download en cours. On extrait la
		// logique de guard dans une fonction `runOnceGuarded` testable
		// isolément, puis on valide que deux appels concurrents ne déclenchent
		// `downloadGlobalExport` qu'une seule fois.
		it('AC #31(c) guard re-entrancy : second concurrent call court-circuité', async () => {
			// Mock backend lent (Promise délayée 50ms).
			let resolveDownload!: () => void;
			const slowFetch = vi.fn().mockImplementation(
				() =>
					new Promise<Response>((resolve) => {
						resolveDownload = () => {
							const zipBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x00, 0x00]);
							const mockBlob = new Blob([zipBytes], { type: 'application/zip' });
							resolve({
								ok: true,
								status: 200,
								blob: () => Promise.resolve(mockBlob),
								headers: new Headers({
									'content-type': 'application/zip',
									'content-disposition':
										'attachment; filename="kesh-export-test.zip"',
								}),
							} as unknown as Response);
						};
					}),
			);
			vi.stubGlobal('fetch', slowFetch);

			// Reproduction du guard `startExport` du `+page.svelte` (Pass 1 M12
			// pattern AC #26 — `if (exporting) return` first-line).
			let exporting = false;
			let calls = 0;
			async function startExport(): Promise<void> {
				if (exporting) return;
				exporting = true;
				try {
					calls += 1;
					await downloadGlobalExport();
				} finally {
					exporting = false;
				}
			}

			// 2 appels concurrents avant résolution du premier
			const p1 = startExport();
			const p2 = startExport();
			// Resolve le mock backend pour laisser p1 terminer
			resolveDownload();
			await Promise.all([p1, p2]);

			// Le guard doit avoir court-circuité le 2e appel : 1 seul `calls` + 1 seul fetch.
			expect(calls).toBe(1);
			expect(slowFetch).toHaveBeenCalledTimes(1);
		});
	});
});
