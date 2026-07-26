// Story 9-2a — Tests Vitest pour les helpers d'export PDF & CSV.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	buildExportFilename,
	downloadReport,
	getProjectReportExportUrl,
	getReportExportUrl,
	slugify,
	getAgedReceivables,
	downloadAgedReceivables,
	isReportEmpty,
} from './reports.api';
import type { BalanceSheetDto } from './reports.types';

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

describe('reports.api — Story 9-2a', () => {
	// ----------------------------------------------------------------------
	// T11.1 — buildExportFilename : 4 types + slug edge cases
	// ----------------------------------------------------------------------

	describe('buildExportFilename', () => {
		const period = { start: '2026-01-01', end: '2026-12-31' };

		it('produit la string attendue pour les 4 types', () => {
			// Pass 1 code-review H10 + M16 : typeSlug est résolu via i18nMsg
			// (fallback FR par défaut quand le store i18n n'est pas chargé en
			// jsdom). Le fallback `'balance'` est devenu `'balance-comptes'`
			// (M16 — distinct de `bilan` pour éviter l'ambiguïté FR).
			expect(
				buildExportFilename('balance-sheet', 'CI Test Company', period, 'pdf'),
			).toBe('kesh-bilan-ci-test-company-2026-01-01_2026-12-31.pdf');
			expect(
				buildExportFilename('income-statement', 'CI Test Company', period, 'csv'),
			).toBe(
				'kesh-compte-resultat-ci-test-company-2026-01-01_2026-12-31.csv',
			);
			expect(
				buildExportFilename('trial-balance', 'CI Test Company', period, 'pdf'),
			).toBe('kesh-balance-comptes-ci-test-company-2026-01-01_2026-12-31.pdf');
			expect(buildExportFilename('journals', 'CI Test Company', period, 'csv')).toBe(
				'kesh-journaux-ci-test-company-2026-01-01_2026-12-31.csv',
			);
		});

		it('slug strip diacritics : "Müller AG" → "muller-ag"', () => {
			expect(buildExportFilename('balance-sheet', 'Müller AG', period, 'pdf')).toBe(
				'kesh-bilan-muller-ag-2026-01-01_2026-12-31.pdf',
			);
		});

		it('slug collapse repeated dashes : "Kesh ---   SA" → "kesh-sa"', () => {
			expect(buildExportFilename('balance-sheet', 'Kesh ---   SA', period, 'csv')).toBe(
				'kesh-bilan-kesh-sa-2026-01-01_2026-12-31.csv',
			);
		});

		it('slug truncate à 20 chars + strip trailing dash (Pass 1 ECH-H3)', () => {
			// "Acme SA Fribourg Extension Long" → "acme-sa-fribourg-extension-long"
			// slice(20) → "acme-sa-fribourg-ext" (pas de trailing `-` ici, 20 chars exacts)
			const slug = slugify('Acme SA Fribourg Extension Long', 'company');
			expect(slug.length).toBeLessThanOrEqual(20);
			expect(slug.endsWith('-')).toBe(false);
			expect(slug).toBe('acme-sa-fribourg-ext');
		});

		it('slug truncate strip trailing dash quand truncate tombe sur "-"', () => {
			// "Foo Bar Baz Qux Quux Corge" → après nonalnum→`-` : "foo-bar-baz-qux-quux-corge"
			// slice(20) → "foo-bar-baz-qux-quux" — pas de trailing `-`, ok.
			// Mais "Foo Bar Baz Qux Quu-x" → "foo-bar-baz-qux-quu--x" → slice(20) → "foo-bar-baz-qux-quu-"
			// → strip trailing → "foo-bar-baz-qux-quu" (19 chars, pas de `-` final).
			const slug = slugify('aaaaa bbbb ccc d e fg', 'company');
			expect(slug.endsWith('-')).toBe(false);
		});

		it('slug vide → fallback "company"', () => {
			expect(buildExportFilename('balance-sheet', '', period, 'pdf')).toBe(
				'kesh-bilan-company-2026-01-01_2026-12-31.pdf',
			);
		});

		it('nom CJK chinois → fallback "company"', () => {
			expect(buildExportFilename('balance-sheet', '北京公司', period, 'pdf')).toBe(
				'kesh-bilan-company-2026-01-01_2026-12-31.pdf',
			);
		});

		it('slugify : strip leading and trailing dashes', () => {
			// "---hello---" → "hello"
			expect(slugify('---hello---', 'fb')).toBe('hello');
		});

		it('slugify : multiple slashes / underscores → un seul dash', () => {
			expect(slugify('a/b\\c_d', 'fb')).toBe('a-b-c-d');
		});
	});

	// ----------------------------------------------------------------------
	// T11.1 — getReportExportUrl : construction URL avec query string
	// ----------------------------------------------------------------------

	describe('getReportExportUrl', () => {
		it("construit l'URL pour balance-sheet PDF avec format=pdf", () => {
			const url = getReportExportUrl(
				'balance-sheet',
				{ fiscalYearId: 7 },
				'pdf',
			);
			expect(url).toBe('/api/v1/reports/balance-sheet/export?fiscalYearId=7&format=pdf');
		});

		it("inclut periodStart/periodEnd si fournis", () => {
			const url = getReportExportUrl(
				'trial-balance',
				{
					fiscalYearId: 12,
					periodStart: '2026-03-01',
					periodEnd: '2026-06-30',
				},
				'csv',
			);
			expect(url).toContain('fiscalYearId=12');
			expect(url).toContain('periodStart=2026-03-01');
			expect(url).toContain('periodEnd=2026-06-30');
			expect(url).toContain('format=csv');
		});

		it("inclut le filtre journal pour le rapport journals", () => {
			const url = getReportExportUrl(
				'journals',
				{ fiscalYearId: 1, journal: 'Ventes' },
				'pdf',
			);
			expect(url).toContain('journal=Ventes');
		});
	});

	// Story 19-6a — URL export rapport projet, 2 modes.
	describe('getProjectReportExportUrl', () => {
		it('mode fiscal_year inclut projectId + mode + fiscalYearId', () => {
			const url = getProjectReportExportUrl(
				'project-expenses',
				{ projectId: 3, mode: 'fiscal_year', fiscalYearId: 7 },
				'pdf',
			);
			expect(url).toContain('/api/v1/reports/project-expenses/export');
			expect(url).toContain('projectId=3');
			expect(url).toContain('mode=fiscal_year');
			expect(url).toContain('fiscalYearId=7');
			expect(url).toContain('format=pdf');
		});

		it('project-return : URL export construite', () => {
			const url = getProjectReportExportUrl(
				'project-return',
				{ projectId: 9, mode: 'fiscal_year', fiscalYearId: 3 },
				'csv',
			);
			expect(url).toContain('/api/v1/reports/project-return/export');
			expect(url).toContain('projectId=9');
			expect(url).toContain('format=csv');
		});

		it('mode cumulative omet fiscalYearId', () => {
			const url = getProjectReportExportUrl(
				'project-expenses',
				{ projectId: 3, mode: 'cumulative', fiscalYearId: 7 },
				'csv',
			);
			expect(url).toContain('mode=cumulative');
			expect(url).not.toContain('fiscalYearId');
			expect(url).toContain('format=csv');
		});
	});

	// ----------------------------------------------------------------------
	// T11.1 — downloadReport : mock fetch + Blob path + error path
	// ----------------------------------------------------------------------

	describe('downloadReport', () => {
		let mockFetch: ReturnType<typeof vi.fn>;
		let createObjectURLSpy: ReturnType<typeof vi.fn>;
		let revokeObjectURLSpy: ReturnType<typeof vi.fn>;

		beforeEach(() => {
			authState.clearSession();
			authState.login({ userId: '1', username: 'test', role: 'Admin', expiresIn: 900 });
			// Mock URL.createObjectURL / revokeObjectURL (pas dispo dans jsdom par défaut)
			createObjectURLSpy = vi.fn().mockReturnValue('blob:http://localhost/abc');
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

		it('appelle fetch avec URL correcte et retourne via Blob (mock fetch)', async () => {
			const mockBlob = new Blob([new Uint8Array([0x25, 0x50, 0x44, 0x46])], {
				type: 'application/pdf',
			}); // %PDF
			mockFetch = vi.fn().mockResolvedValue({
				ok: true,
				status: 200,
				blob: () => Promise.resolve(mockBlob),
				headers: new Headers({ 'content-type': 'application/pdf' }),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			await downloadReport(
				'balance-sheet',
				{ fiscalYearId: 1 },
				'pdf',
				'kesh-bilan-test-2026-01-01_2026-12-31.pdf',
			);

			expect(mockFetch).toHaveBeenCalledTimes(1);
			const call = mockFetch.mock.calls[0];
			const calledUrl = call[0] as string;
			expect(calledUrl).toContain('/api/v1/reports/balance-sheet/export');
			expect(calledUrl).toContain('fiscalYearId=1');
			expect(calledUrl).toContain('format=pdf');
			expect(createObjectURLSpy).toHaveBeenCalledOnce();
			expect(revokeObjectURLSpy).toHaveBeenCalledOnce();
		});

		it('rejette quand status 500 avec message d\'erreur formaté', async () => {
			mockFetch = vi.fn().mockResolvedValue({
				ok: false,
				status: 500,
				json: () =>
					Promise.resolve({
						error: { code: 'INTERNAL_ERROR', message: 'Internal server error' },
					}),
				headers: new Headers({ 'content-type': 'application/json' }),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			await expect(
				downloadReport(
					'balance-sheet',
					{ fiscalYearId: 1 },
					'pdf',
					'kesh-bilan-test-2026-01-01_2026-12-31.pdf',
				),
			).rejects.toThrow();
		});
	});

	// ----------------------------------------------------------------------
	// Story 21-7 — Balance âgée : wrappers dédiés
	// ----------------------------------------------------------------------
	describe('aged receivables (Story 21-7)', () => {
		let mockFetch: ReturnType<typeof vi.fn>;

		beforeEach(() => {
			authState.clearSession();
			authState.login({ userId: '1', username: 'test', role: 'Comptable', expiresIn: 900 });
			vi.stubGlobal('URL', {
				createObjectURL: vi.fn().mockReturnValue('blob:http://localhost/x'),
				revokeObjectURL: vi.fn(),
			});
		});
		afterEach(() => {
			vi.unstubAllGlobals();
			authState.clearSession();
		});

		it('getAgedReceivables GET sur le bon path', async () => {
			mockFetch = vi.fn().mockResolvedValue({
				ok: true,
				status: 200,
				json: () =>
					Promise.resolve({ asOf: '2026-07-20', rows: [], totals: { total: '0.00' } }),
				headers: new Headers({ 'content-type': 'application/json' }),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			const dto = await getAgedReceivables();
			expect(dto.asOf).toBe('2026-07-20');
			const url = mockFetch.mock.calls[0][0] as string;
			expect(url).toContain('/api/v1/reports/aged-receivables');
			expect(url).not.toContain('/export');
		});

		it('downloadAgedReceivables GET blob sur le path export?format=csv', async () => {
			const blob = new Blob(['Contact;Total\r\n'], { type: 'text/csv' });
			mockFetch = vi.fn().mockResolvedValue({
				ok: true,
				status: 200,
				blob: () => Promise.resolve(blob),
				headers: new Headers({ 'content-type': 'text/csv' }),
			} as unknown as Response);
			vi.stubGlobal('fetch', mockFetch);

			await downloadAgedReceivables();
			const url = mockFetch.mock.calls[0][0] as string;
			expect(url).toContain('/api/v1/reports/aged-receivables/export');
			expect(url).toContain('format=csv');
		});
	});

	// Story 14-1 review Pass 1 — `isReportEmpty('balance-sheet', …)` doit tenir
	// compte de retainedEarnings/equityResult, sinon la section fonds propres est
	// masquée quand actif/passif retombent à 0 mais report/résultat sont non nuls.
	describe('isReportEmpty — balance-sheet (Story 14-1)', () => {
		function bs(partial: Partial<BalanceSheetDto>): BalanceSheetDto {
			return {
				period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
				assets: [],
				liabilities: [],
				equity: [],
				totalAssets: '0',
				totalLiabilities: '0',
				totalEquity: '0',
				retainedEarnings: '0',
				equityResult: '0',
				equationHolds: true,
				...partial,
			};
		}

		it('vide quand actif/passif ET report/résultat sont tous nuls', () => {
			expect(isReportEmpty('balance-sheet', bs({}))).toBe(true);
		});

		it("PAS vide quand actif/passif nuls mais report/résultat non nuls et opposés", () => {
			const dto = bs({ retainedEarnings: '1000', equityResult: '-1000' });
			expect(isReportEmpty('balance-sheet', dto)).toBe(false);
		});

		it('PAS vide quand il y a des comptes de bilan', () => {
			const dto = bs({
				assets: [
					{
						accountId: 1,
						accountNumber: '1000',
						accountName: 'Banque',
						accountType: 'Asset',
						active: true,
						balance: '500',
						role: null,
					},
				],
				totalAssets: '500',
			});
			expect(isReportEmpty('balance-sheet', dto)).toBe(false);
		});

		it('PAS vide quand seuls des comptes de fonds propres sont presents (reclassement pur equity, Story 14-3c)', () => {
			// Reclassement pur entre 2 comptes de fonds propres qui s'annulent (net 0)
			// mais la section equity est peuplee -> NON vide (garde P1-F1).
			const dto = bs({
				equity: [
					{
						accountId: 10,
						accountNumber: '2800',
						accountName: 'Capital',
						accountType: 'Liability',
						active: true,
						balance: '1000',
						role: 'EquityCapital',
					},
					{
						accountId: 11,
						accountNumber: '2900',
						accountName: 'Reserves',
						accountType: 'Liability',
						active: true,
						balance: '-1000',
						role: 'EquityOther',
					},
				],
				totalEquity: '0',
			});
			expect(isReportEmpty('balance-sheet', dto)).toBe(false);
		});
	});
});
