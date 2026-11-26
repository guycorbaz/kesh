// Story 55-1a (D1, D5, AC-5, AC-6, AC-15, R-9) — lecture et écriture de l'URL des Rapports.

import { describe, expect, it } from 'vitest';
import {
	buildReportSearch,
	parseReportUrl,
	reportHref,
	resolveClassicPeriod,
	REPORT_TABS,
} from './reports-url';
import { ledgerHref } from './reports.api';

const FYS = [
	{ id: 1, name: 'Exercice 2025', startDate: '2025-01-01', endDate: '2025-12-31' },
	{ id: 2, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31' },
];
const IDS = FYS.map((f) => f.id);
const lire = (s: string) => parseReportUrl(new URLSearchParams(s), IDS);

describe('reports-url — lecture (D5)', () => {
	it('chaque paramètre valide est lu', () => {
		expect(lire('tab=balance-sheet&fy=2&from=2026-01-01&to=2026-03-31')).toEqual({
			tab: 'balance-sheet',
			fy: 2,
			from: '2026-01-01',
			to: '2026-03-31',
			accountId: null,
			includeZero: false,
		});
		expect(lire('tab=general-ledger&accountId=17&from=2026-01-01&to=2026-12-31&includeZero=1')).toEqual({
			tab: 'general-ledger',
			fy: null,
			from: '2026-01-01',
			to: '2026-12-31',
			accountId: 17,
			includeZero: true,
		});
	});

	it.each([
		['tab=xyz', 'tab', null],
		['tab=balance-sheet&fy=999999', 'fy', null],
		['tab=balance-sheet&fy=abc', 'fy', null],
		['tab=balance-sheet&fy=0', 'fy', null],
		['tab=balance-sheet&from=2026-02-30', 'from', null],
		['tab=balance-sheet&to=0202-01-01', 'to', null],
		['tab=general-ledger&accountId=-3', 'accountId', null],
		['tab=general-ledger&accountId=9007199254740993', 'accountId', null],
		['tab=general-ledger&includeZero=true', 'includeZero', false],
	] as const)('%s → %s ignoré', (s, champ, attendu) => {
		expect(lire(s)[champ]).toBe(attendu);
	});

	it('un paramètre étranger à l’onglet est ignoré', () => {
		const p = lire('tab=balance-sheet&accountId=5&includeZero=1');
		expect(p.accountId).toBeNull();
		expect(p.includeZero).toBe(false);
		expect(lire('tab=general-ledger&fy=2').fy).toBeNull();
		expect(lire('tab=aged-receivables&from=2026-01-01').from).toBeNull();
	});

	it('mutation tuée : la validation de fy (un exercice inconnu est invalide)', () => {
		expect(lire('tab=vat&fy=3').fy).toBeNull();
		expect(lire('tab=vat&fy=1').fy).toBe(1);
	});
});

describe('reports-url — écriture canonique', () => {
	it('ordre canonique et paramètres de l’onglet seulement', () => {
		expect(
			buildReportSearch({ tab: 'general-ledger', includeZero: true, to: '2026-12-31', from: '2026-01-01', accountId: 4, fy: 2 }),
		).toBe('tab=general-ledger&accountId=4&from=2026-01-01&to=2026-12-31&includeZero=1');
		expect(buildReportSearch({ tab: 'balance-sheet', to: '2026-12-31', from: '2026-01-01', fy: 2, accountId: 4 })).toBe(
			'tab=balance-sheet&fy=2&from=2026-01-01&to=2026-12-31',
		);
		expect(buildReportSearch({ tab: 'aged-receivables', from: '2026-01-01' })).toBe('tab=aged-receivables');
		expect(buildReportSearch({ tab: 'general-ledger', includeZero: false })).toBe('tab=general-ledger');
	});

	it.each([
		'tab=balance-sheet&fy=2&from=2026-01-01&to=2026-12-31',
		'tab=income-statement&fy=1&from=2025-02-01&to=2025-02-28',
		'tab=general-ledger&accountId=17&from=2025-06-01&to=2026-03-31&includeZero=1',
		'tab=general-ledger&from=2026-01-01&to=2026-12-31',
		'tab=aged-receivables',
		'tab=project-expenses',
	])('aller-retour : build(parse(%s)) est l’URL elle-même', (s) => {
		const p = lire(s);
		expect(buildReportSearch({ ...p, tab: p.tab! })).toBe(s);
	});

	it('ledgerHref rend la même chaîne qu’avant (AC-6, trois cas)', () => {
		const avant = (a: number, f: string, t: string) =>
			`/reports?${new URLSearchParams({ tab: 'general-ledger', accountId: String(a), from: f, to: t }).toString()}`;
		for (const [a, f, t] of [
			[17, '2026-01-01', '2026-12-31'],
			[1, '2025-03-01', '2026-02-28'],
			[90071992, '1999-12-31', '2000-01-01'],
		] as const) {
			expect(ledgerHref(a, f, t)).toBe(avant(a, f, t));
			expect(reportHref({ tab: 'general-ledger', accountId: a, from: f, to: t })).toBe(avant(a, f, t));
		}
	});

	it('les neuf onglets', () => {
		expect(REPORT_TABS).toHaveLength(9);
	});
});

describe('reports-url — période d’un onglet classique (AC-15, R-9)', () => {
	const periode = (s: string) => resolveClassicPeriod(lire(s), FYS);

	it('fy seul : ses bornes', () => {
		expect(periode('tab=balance-sheet&fy=1')).toEqual({ from: '2025-01-01', to: '2025-12-31', fy: 1 });
	});
	it('une date absente prend la borne de fy', () => {
		expect(periode('tab=balance-sheet&fy=1&from=2025-04-01')).toEqual({ from: '2025-04-01', to: '2025-12-31', fy: 1 });
	});
	it('les dates décident : un fy qui les contredit est remplacé', () => {
		expect(periode('tab=balance-sheet&fy=1&from=2026-01-01&to=2026-06-30')).toEqual({
			from: '2026-01-01',
			to: '2026-06-30',
			fy: 2,
		});
	});
	it('dates dans deux exercices : posées, aucun exercice (« Plusieurs », non complète)', () => {
		expect(periode('tab=balance-sheet&from=2025-06-01&to=2026-06-30')).toEqual({
			from: '2025-06-01',
			to: '2026-06-30',
			fy: null,
		});
	});
	it('date hors de tout exercice : aucun exercice (« Hors exercice »)', () => {
		expect(periode('tab=balance-sheet&from=2024-06-01&to=2024-06-30')?.fy).toBeNull();
	});
	it('début après fin : aucun exercice (non complète)', () => {
		expect(periode('tab=balance-sheet&from=2026-06-30&to=2026-01-01')?.fy).toBeNull();
	});
	it('sans date ni fy valide : rien (aucun champ modifié)', () => {
		expect(periode('tab=balance-sheet')).toBeNull();
		expect(periode('tab=balance-sheet&fy=abc')).toBeNull();
		expect(periode('tab=balance-sheet&from=2026-01-01')).toBeNull();
	});
	it('« Plusieurs » s’écrit sans fy', () => {
		expect(buildReportSearch({ tab: 'balance-sheet', fy: null, from: '2025-06-01', to: '2026-06-30' })).toBe(
			'tab=balance-sheet&from=2025-06-01&to=2026-06-30',
		);
	});
});
