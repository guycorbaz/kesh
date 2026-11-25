// Story 46-2 (AC-3, AC-6) — les paramètres de groupement dans l'URL du rapport et
// dans celle de l'export : noms et formes du contrat de la 46-1, que l'export de
// la 46-3 lit tels quels (`detailLevel`, `subtotalPosition`, `open`, `closed`,
// identifiants séparés par des virgules). Le rendu groupé du fichier est exercé de
// bout en bout après la fusion de la 46-3.

import { beforeEach, describe, expect, it, vi } from 'vitest';

const { getMock, getBlobMock } = vi.hoisted(() => ({
	getMock: vi.fn(async (_url: string) => ({})),
	getBlobMock: vi.fn(async (_url: string) => ({ blob: async () => new Blob(['x']) })),
}));
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: getMock, getBlob: getBlobMock },
}));

import {
	downloadReport,
	getBalanceSheet,
	getIncomeStatement,
	getReportExportUrl,
	FLAT_DETAIL_LEVEL,
	getTrialBalance,
	groupingQueryParams,
} from './reports.api';
import type { GroupingParams } from './reports.types';

const params = (url: string) => new URL(url, 'http://localhost').searchParams;
const g: GroupingParams = { detailLevel: 2, subtotalPosition: 'footer', open: [12, 7], closed: [40] };

describe('groupingQueryParams', () => {
	it('sans groupement : aucun paramètre', () => {
		expect(groupingQueryParams(undefined)).toEqual({});
	});
	it('listes vides omises (une liste vide serait refusée en 400)', () => {
		const p = groupingQueryParams({ detailLevel: 'all', subtotalPosition: 'header', open: [], closed: [] });
		expect(p).toEqual({ detailLevel: 'all', subtotalPosition: 'header', open: undefined, closed: undefined });
	});
});

describe('requêtes du bilan et du compte de résultat', () => {
	beforeEach(() => getMock.mockClear());

	it('getBalanceSheet transmet niveau, position et listes', async () => {
		await getBalanceSheet({ fiscalYearId: 3, periodEnd: '2026-06-30' }, g);
		const p = params(getMock.mock.calls[0][0] as string);
		expect(p.get('fiscalYearId')).toBe('3');
		expect(p.get('periodEnd')).toBe('2026-06-30');
		expect(p.get('detailLevel')).toBe('2');
		expect(p.get('subtotalPosition')).toBe('footer');
		expect(p.get('open')).toBe('12,7');
		expect(p.get('closed')).toBe('40');
	});

	it('sans groupement : la requête d’avant l’Epic 46, à l’identique', async () => {
		await getIncomeStatement({ fiscalYearId: 3 });
		expect(getMock.mock.calls[0][0]).toBe('/api/v1/reports/income-statement?fiscalYearId=3');
	});

	it('getIncomeStatement transmet le niveau « all »', async () => {
		await getIncomeStatement({ fiscalYearId: 3 }, { ...g, detailLevel: 'all', open: [], closed: [] });
		const p = params(getMock.mock.calls[0][0] as string);
		expect(p.get('detailLevel')).toBe('all');
		expect(p.has('open')).toBe(false);
		expect(p.has('closed')).toBe(false);
	});
});

describe('requête de la balance (Story 46-6b)', () => {
	beforeEach(() => getMock.mockClear());

	it('getTrialBalance transmet niveau, position et listes', async () => {
		await getTrialBalance({ fiscalYearId: 3 }, g);
		const url = getMock.mock.calls[0][0] as string;
		expect(url.startsWith('/api/v1/reports/trial-balance?')).toBe(true);
		const p = params(url);
		expect(p.get('detailLevel')).toBe('2');
		expect(p.get('subtotalPosition')).toBe('footer');
		expect(p.get('open')).toBe('12,7');
		expect(p.get('closed')).toBe('40');
	});

	it('sans groupement : la requête d’avant, à l’identique', async () => {
		await getTrialBalance({ fiscalYearId: 3 });
		expect(getMock.mock.calls[0][0]).toBe('/api/v1/reports/trial-balance?fiscalYearId=3');
	});
});

describe("URL d'export (AC-6, paramètres de la 46-3 et de la 46-6b)", () => {
	it('bilan, compte de résultat et balance : les mêmes paramètres que l’écran', () => {
		for (const type of ['balance-sheet', 'income-statement', 'trial-balance'] as const) {
			const url = getReportExportUrl(type, { fiscalYearId: 1 }, 'csv', g);
			expect(url.startsWith(`/api/v1/reports/${type}/export?`)).toBe(true);
			const p = params(url);
			expect(p.get('format')).toBe('csv');
			expect(p.get('detailLevel')).toBe('2');
			expect(p.get('subtotalPosition')).toBe('footer');
			expect(p.get('open')).toBe('12,7');
			expect(p.get('closed')).toBe('40');
		}
	});

	// Avant la décision du mainteneur (l'export de l'API rend la présentation de
	// l'écran par défaut), ce test attendait une URL sans paramètre de groupement :
	// c'était l'export à plat. Sans paramètre, l'export est désormais groupé ;
	// l'écran à plat demande donc `detailLevel=flat`, et rien d'autre.
	it('hiérarchie masquée : detailLevel=flat seul, pour les trois rapports groupés', () => {
		expect(getReportExportUrl('balance-sheet', { fiscalYearId: 1 }, 'pdf')).toBe(
			'/api/v1/reports/balance-sheet/export?fiscalYearId=1&format=pdf&detailLevel=flat',
		);
		for (const type of ['balance-sheet', 'income-statement', 'trial-balance'] as const) {
			const p = params(getReportExportUrl(type, { fiscalYearId: 1 }, 'csv'));
			expect(p.get('detailLevel')).toBe(FLAT_DETAIL_LEVEL);
			expect(p.has('subtotalPosition')).toBe(false);
			expect(p.has('open')).toBe(false);
			expect(p.has('closed')).toBe(false);
		}
	});

	it('hiérarchie masquée : la lecture JSON reste sans paramètre (à plat par défaut)', async () => {
		await getBalanceSheet({ fiscalYearId: 1 });
		const p = params(getMock.mock.calls.at(-1)![0] as string);
		expect(p.has('detailLevel')).toBe(false);
	});

	it('les autres rapports ne reçoivent jamais de paramètre de groupement', () => {
		// Story 46-6b : la balance a quitté cette liste (elle reçoit les paramètres).
		for (const type of ['journals', 'vat'] as const) {
			const p = params(getReportExportUrl(type, { fiscalYearId: 1 }, 'csv', g));
			expect(p.has('detailLevel')).toBe(false);
			// …pas même `flat` sans groupement.
			expect(params(getReportExportUrl(type, { fiscalYearId: 1 }, 'csv')).has('detailLevel')).toBe(false);
			expect(p.has('open')).toBe(false);
		}
	});

	it('downloadReport passe le groupement à l’URL téléchargée', async () => {
		// jsdom n'a pas d'URL d'objet : posées le temps du téléchargement.
		const avant = { c: URL.createObjectURL, r: URL.revokeObjectURL };
		URL.createObjectURL = () => 'blob:x';
		URL.revokeObjectURL = () => {};
		try {
			await downloadReport('balance-sheet', { fiscalYearId: 1 }, 'pdf', 'f.pdf', g);
		} finally {
			URL.createObjectURL = avant.c;
			URL.revokeObjectURL = avant.r;
		}
		const p = params(getBlobMock.mock.calls[0][0] as string);
		expect(p.get('detailLevel')).toBe('2');
		expect(p.get('open')).toBe('12,7');
	});
});
