/**
 * Le module de téléchargement partagé — Story 25-1c-b1 (#378).
 *
 * Les cas de `parseContentDispositionFilename` sont recopiés de
 * `features/export/exports.api.test.ts` **sans modification de leurs
 * assertions** (leur suppression là-bas relève de #438). `triggerDownload`, lui,
 * n'avait aucun test direct.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { parseContentDispositionFilename, triggerDownload } from './download';

describe('parseContentDispositionFilename', () => {
	it('AC #31(d) ASCII fallback : filename="…"', () => {
		expect(
			parseContentDispositionFilename('attachment; filename="kesh-export-foo-2026-05-15.zip"'),
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

	it('returns null on RFC 5987 filename* with empty percent-encoded value', () => {
		expect(parseContentDispositionFilename("attachment; filename*=UTF-8''")).toBeNull();
	});
});

describe('triggerDownload', () => {
	let createObjectURL: ReturnType<typeof vi.fn>;
	let revokeObjectURL: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		createObjectURL = vi.fn().mockReturnValue('blob:http://localhost/x');
		revokeObjectURL = vi.fn();
		vi.stubGlobal('URL', { createObjectURL, revokeObjectURL });
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
	});

	it("pose le nom et l'URL objet, clique, puis retire l'ancre et libère l'URL", () => {
		let clicked: HTMLAnchorElement | null = null;
		const click = vi
			.spyOn(HTMLAnchorElement.prototype, 'click')
			.mockImplementation(function (this: HTMLAnchorElement) {
				clicked = this;
			});
		triggerDownload(new Blob(['a;b']), 'journal.csv');

		expect(click).toHaveBeenCalledOnce();
		expect(clicked!.download).toBe('journal.csv');
		expect(clicked!.getAttribute('href')).toBe('blob:http://localhost/x');
		expect(clicked!.parentNode).toBeNull();
		expect(revokeObjectURL).toHaveBeenCalledWith('blob:http://localhost/x');
	});

	it("libère l'URL et retire l'ancre MÊME si click() jette (mutation : revoke hors du finally)", () => {
		let clicked: HTMLAnchorElement | null = null;
		vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (
			this: HTMLAnchorElement,
		) {
			clicked = this;
			throw new Error('bloqué');
		});
		expect(() => triggerDownload(new Blob(['x']), 'x.csv')).toThrow('bloqué');
		expect(clicked!.parentNode).toBeNull();
		expect(revokeObjectURL).toHaveBeenCalledWith('blob:http://localhost/x');
	});
});
