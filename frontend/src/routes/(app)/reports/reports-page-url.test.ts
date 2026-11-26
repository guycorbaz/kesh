// Story 55-1a — l'état des Rapports dans l'URL (D1 à D5), le sélecteur d'exercice (D15) et le
// chargement sans *Générer* (D16), au niveau de la page : `page.url` RÉACTIVE (helper), `goto` simulé qui
// l'écrit, historique simulé (Retour, Suivant), requêtes comptées par rapport.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import { tick } from 'svelte';
import type {
	BalanceSheetDto,
	GeneralLedgerDto,
	IncomeStatementDto,
	ReportQuery,
} from '$lib/features/reports/reports.types';
import {
	arriverSur,
	historique,
	naviguer,
	pageState,
	rechercheCourante,
	retour,
	suivant,
} from './page-url.test-helper.svelte';

const m = vi.hoisted(() => ({
	getBalanceSheet: vi.fn(),
	getIncomeStatement: vi.fn(),
	getTrialBalance: vi.fn(),
	getGeneralLedger: vi.fn(),
	getAgedReceivables: vi.fn(),
	getProjectExpenses: vi.fn(),
	downloadReport: vi.fn(async () => {}),
	downloadProjectReport: vi.fn(async () => {}),
	listProjects: vi.fn(async () => [] as unknown[]),
	goto: vi.fn(),
}));

vi.mock('$app/state', () => ({ page: pageState }));
vi.mock('$app/navigation', () => ({ goto: m.goto }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: m.listProjects }));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: async () => [] }));
vi.mock('$lib/features/reports/reports.api', async (importOriginal) => {
	const vrai = await importOriginal<typeof import('$lib/features/reports/reports.api')>();
	return {
		...vrai,
		getBalanceSheet: m.getBalanceSheet,
		getIncomeStatement: m.getIncomeStatement,
		getTrialBalance: m.getTrialBalance,
		getGeneralLedger: m.getGeneralLedger,
		getAgedReceivables: m.getAgedReceivables,
		getProjectExpenses: m.getProjectExpenses,
		downloadReport: m.downloadReport,
		downloadProjectReport: m.downloadProjectReport,
	};
});

import Page from './+page.svelte';

const FY25 = { id: 1, companyId: 7, name: 'Exercice 2025', startDate: '2025-01-01', endDate: '2025-12-31', status: 'Open', multiCurrency: false };
const FY26 = { id: 2, companyId: 7, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Open', multiCurrency: false };
// `data.fiscalYears[0]` est l'exercice par défaut : 2026 en tête, comme l'API (le plus récent d'abord).
const data = { fiscalYears: [FY26, FY25], companyName: 'Test SA', companyId: 7 } as never;

const URL_BILAN_26 = 'tab=balance-sheet&fy=2&from=2026-01-01&to=2026-12-31';

function bilan(q: ReportQuery): BalanceSheetDto {
	return {
		period: { fiscalYearId: q.fiscalYearId, startDate: q.periodStart!, endDate: q.periodEnd! },
		assets: [{ accountId: 1, accountNumber: '1000', accountName: 'Caisse', accountType: 'Asset', active: true, balance: '100', role: null }],
		liabilities: [],
		equity: [],
		totalAssets: '100',
		totalLiabilities: '0',
		totalEquity: '0',
		retainedEarnings: '0',
		equityResult: '100',
		equationHolds: true,
	} as BalanceSheetDto;
}
function resultat(q: ReportQuery): IncomeStatementDto {
	return {
		period: { fiscalYearId: q.fiscalYearId, startDate: q.periodStart!, endDate: q.periodEnd! },
		revenues: [],
		expenses: [],
		totalRevenues: '0',
		totalExpenses: '0',
		netResult: '0',
	} as IncomeStatementDto;
}
const livre = (q: { from: string; to: string }): GeneralLedgerDto => ({ period: { from: q.from, to: q.to }, sections: [] }) as GeneralLedgerDto;

/** Une réponse suspendue : la promesse et de quoi la résoudre. */
function suspendre<T>(fn: ReturnType<typeof vi.fn>, faire: (q: never) => T) {
	let resoudre!: () => void;
	fn.mockImplementationOnce((q: never) => new Promise<T>((res) => (resoudre = () => res(faire(q)))));
	return () => resoudre();
}

const appels = (fn: ReturnType<typeof vi.fn>) => fn.mock.calls.length;
const pause = () => new Promise((res) => setTimeout(res, 10));
type R = Awaited<ReturnType<typeof monter>>;
const onglet = (r: R, nom: string) => r.getByRole('tab', { name: nom });
const du = (r: R) => r.getByTestId('report-from') as HTMLInputElement;
const au = (r: R) => r.getByTestId('report-to') as HTMLInputElement;

/** Rend la page à une adresse d'arrivée, et attend le premier chargement. */
async function monter(href = '/reports') {
	arriverSur(href);
	const r = render(Page, { data });
	await tick();
	await pause();
	return r;
}

beforeEach(() => {
	vi.clearAllMocks();
	localStorage.clear();
	m.getBalanceSheet.mockImplementation(async (q: ReportQuery) => bilan(q));
	m.getIncomeStatement.mockImplementation(async (q: ReportQuery) => resultat(q));
	m.getTrialBalance.mockImplementation(async () => {
		throw new Error('non servi');
	});
	m.getGeneralLedger.mockImplementation(async (q: { from: string; to: string }) => livre(q));
	m.getAgedReceivables.mockImplementation(async () => ({ asOf: '2026-12-31', rows: [], totals: {} }));
	// Le `goto` de la page écrit l'adresse et l'historique, comme SvelteKit.
	m.goto.mockImplementation((href: string, opts?: { replaceState?: boolean }) => {
		naviguer(href, opts?.replaceState === true);
		return Promise.resolve();
	});
	vi.spyOn(console, 'error').mockImplementation(() => {});
});

describe('arrivée, rechargement, lien partagé (AC-4, AC-5, D16)', () => {
	it('/reports : le Bilan de l’exercice par défaut se charge, l’adresse est normalisée sans entrée', async () => {
		const r = await monter('/reports');
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(m.getBalanceSheet.mock.calls[0][0]).toEqual({ fiscalYearId: 2, periodStart: '2026-01-01', periodEnd: '2026-12-31' });
		expect(rechercheCourante()).toBe(URL_BILAN_26);
		expect(historique.entrees).toHaveLength(1);
		expect(onglet(r, 'Bilan').getAttribute('aria-selected')).toBe('true');
		expect(du(r).value).toBe('2026-01-01');
		expect(r.getByTestId('report-period-heading').textContent).toContain('Exercice 2026 — du 01.01.2026 au 31.12.2026');
	});

	it('adresse complète collée ou rechargée : le rapport de l’adresse, champs remplis, sans clic', async () => {
		const r = await monter('/reports?tab=income-statement&fy=1&from=2025-02-01&to=2025-03-31');
		expect(appels(m.getBalanceSheet)).toBe(0);
		expect(m.getIncomeStatement.mock.calls[0][0]).toEqual({ fiscalYearId: 1, periodStart: '2025-02-01', periodEnd: '2025-03-31' });
		expect(onglet(r, 'Compte de résultat').getAttribute('aria-selected')).toBe('true');
		expect(du(r).value).toBe('2025-02-01');
		expect(au(r).value).toBe('2025-03-31');
		expect((r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement).value).toBe('1');
		expect(m.goto).not.toHaveBeenCalled(); // déjà canonique
	});

	it.each([
		['tab=xyz', URL_BILAN_26],
		['tab=balance-sheet&fy=999999', URL_BILAN_26],
		['tab=balance-sheet&fy=abc', URL_BILAN_26],
		['tab=balance-sheet&from=2026-02-30&to=2026-03-31', URL_BILAN_26],
		['tab=balance-sheet&fy=2&from=2026-01-01&to=2026-12-31&accountId=4', URL_BILAN_26],
		['tab=general-ledger&from=2026-01-01&to=2026-12-31&accountId=-3', 'tab=general-ledger&from=2026-01-01&to=2026-12-31'],
		['tab=general-ledger&from=2026-01-01&to=2026-06-30&accountId=5', 'tab=general-ledger&accountId=5&from=2026-01-01&to=2026-06-30'],
	])('valeur invalide ou ordre non canonique (%s) : ignorée, adresse normalisée sans entrée', async (entree, canonique) => {
		const r = await monter(`/reports?${entree}`);
		expect(rechercheCourante()).toBe(canonique);
		expect(historique.entrees).toHaveLength(1);
		expect(r.queryByRole('alert')).toBeNull();
		expect(m.goto.mock.calls.every((c) => (c[1] as { replaceState?: boolean }).replaceState === true)).toBe(true);
	});

	it('un fy qui contredit les dates est réécrit : les dates décident (R-9)', async () => {
		await monter('/reports?tab=balance-sheet&fy=1&from=2026-01-01&to=2026-06-30');
		expect(m.getBalanceSheet.mock.calls[0][0]).toMatchObject({ fiscalYearId: 2 });
		expect(rechercheCourante()).toBe('tab=balance-sheet&fy=2&from=2026-01-01&to=2026-06-30');
	});

	it('balance âgée par son adresse : chargée, 1 requête, 0 goto (AC-7)', async () => {
		const r = await monter('/reports?tab=aged-receivables');
		expect(appels(m.getAgedReceivables)).toBe(1);
		expect(m.goto).not.toHaveBeenCalled();
		expect(r.queryByRole('button', { name: 'Générer' })).toBeNull();
	});
});

describe('Retour et Suivant (AC-1, AC-2, AC-13)', () => {
	it('AC-1 — Retour après un lien de compte : le Bilan, sans requête ; Suivant : le grand livre, sans requête', async () => {
		const r = await monter('/reports');
		naviguer('/reports?tab=general-ledger&accountId=1&from=2026-01-01&to=2026-12-31'); // le <a href> du bilan
		await tick();
		await pause();
		expect(appels(m.getGeneralLedger)).toBe(1);
		expect(onglet(r, 'Grand livre').getAttribute('aria-selected')).toBe('true');
		retour();
		await tick();
		await pause();
		// assertion décisive (V8-3) : onglet et rapport affichés sont ceux de l'adresse
		expect(onglet(r, 'Bilan').getAttribute('aria-selected')).toBe('true');
		expect(r.getByText('Caisse')).toBeTruthy();
		expect(appels(m.getBalanceSheet)).toBe(1);
		suivant();
		await tick();
		await pause();
		expect(onglet(r, 'Grand livre').getAttribute('aria-selected')).toBe('true');
		expect(appels(m.getGeneralLedger)).toBe(1);
	});

	it('AC-2, AC-22 — clic sur un onglet jamais affiché : il se charge, l’adresse complète est empilée ; Retour et Suivant sans requête', async () => {
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		expect(appels(m.getIncomeStatement)).toBe(1);
		expect(rechercheCourante()).toBe('tab=income-statement&fy=2&from=2026-01-01&to=2026-12-31');
		expect(historique.entrees).toHaveLength(2);
		retour();
		await tick();
		await pause();
		expect(onglet(r, 'Bilan').getAttribute('aria-selected')).toBe('true');
		expect(appels(m.getBalanceSheet)).toBe(1);
		suivant();
		await tick();
		await pause();
		expect(onglet(r, 'Compte de résultat').getAttribute('aria-selected')).toBe('true');
		expect(appels(m.getIncomeStatement)).toBe(1);
	});

	it('clic sur un onglet déjà affiché aux mêmes champs : 0 requête (non-régression)', async () => {
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(r.getByText('Caisse')).toBeTruthy();
	});

	it('chaque goto de la page reçoit keepFocus et noScroll (AC-10)', async () => {
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await fireEvent.keyDown(onglet(r, 'Compte de résultat'), { key: 'ArrowRight' });
		await pause();
		expect(m.goto.mock.calls.length).toBeGreaterThan(1);
		for (const c of m.goto.mock.calls) expect(c[1]).toMatchObject({ keepFocus: true, noScroll: true });
		// les flèches remplacent (C-55-1-5) : une entrée pour le clic, aucune pour la flèche
		expect(historique.entrees).toHaveLength(2);
	});

	it('AC-10 — échec sur le Compte de résultat, puis Retour au Bilan : aucun message', async () => {
		m.getIncomeStatement.mockImplementationOnce(async () => {
			throw new Error('réseau');
		});
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await r.findByRole('alert');
		retour();
		await tick();
		await pause();
		expect(r.queryByRole('alert')).toBeNull();
	});

	it('AC-8 b, AC-13 — Retour vers un rapport absent de la mémoire pendant le chargement d’un autre : demandé, l’autre périmé', async () => {
		m.getIncomeStatement.mockImplementationOnce(async () => {
			throw new Error('réseau'); // le compte de résultat n'est pas en mémoire
		});
		const r = await monter('/reports?tab=income-statement&fy=2&from=2026-01-01&to=2026-12-31');
		await r.findByRole('alert');
		const finirBilan = suspendre(m.getBalanceSheet, bilan);
		naviguer(`/reports?${URL_BILAN_26}`);
		await tick();
		expect(appels(m.getBalanceSheet)).toBe(1); // en vol
		retour();
		await tick();
		await pause();
		expect(appels(m.getIncomeStatement)).toBe(2); // demandé, sans clic
		expect(onglet(r, 'Compte de résultat').getAttribute('aria-selected')).toBe('true');
		finirBilan(); // réponse périmée : jetée
		await pause();
		suivant();
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(2); // le bilan n'était pas en mémoire : redemandé
	});

	it('point 5 — un Retour qui supplante un goto différé de la page est appliqué', async () => {
		const r = await monter('/reports');
		// le goto du clic ne se résout pas : l'adresse ne change pas encore
		m.goto.mockImplementationOnce(() => new Promise(() => {}));
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		naviguer('/reports?tab=general-ledger&accountId=1&from=2026-01-01&to=2026-12-31');
		await tick();
		await pause();
		expect(onglet(r, 'Grand livre').getAttribute('aria-selected')).toBe('true');
		expect(appels(m.getGeneralLedger)).toBe(1);
	});
});

describe('le chargement sans Générer (D16, AC-20 à AC-26)', () => {
	it('AC-20 — Du changé puis Tab vers Au : rien ; sortie du groupe : 1 requête, adresse remplacée', async () => {
		const r = await monter('/reports');
		await fireEvent.input(du(r), { target: { value: '2026-04-01' } });
		await fireEvent.focusOut(du(r), { relatedTarget: au(r) });
		expect(appels(m.getBalanceSheet)).toBe(1);
		await fireEvent.focusOut(au(r), { relatedTarget: null });
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(2);
		expect(m.getBalanceSheet.mock.calls[1][0]).toMatchObject({ periodStart: '2026-04-01', periodEnd: '2026-12-31' });
		expect(rechercheCourante()).toBe('tab=balance-sheet&fy=2&from=2026-04-01&to=2026-12-31');
		expect(historique.entrees).toHaveLength(1);
	});

	it('AC-20 — dates inchangées puis sortie : 0 ; Entrée : 1 ; Entrée puis sortie : 1 seule', async () => {
		const r = await monter('/reports');
		await fireEvent.focusOut(au(r), { relatedTarget: null });
		expect(appels(m.getBalanceSheet)).toBe(1);
		await fireEvent.input(au(r), { target: { value: '2026-06-30' } });
		await fireEvent.keyDown(au(r), { key: 'Enter' });
		await fireEvent.focusOut(au(r), { relatedTarget: null });
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(2);
	});

	it('AC-20 — début après fin : 0 requête, message ; le rapport affiché reste, non grisé', async () => {
		const r = await monter('/reports');
		await fireEvent.input(du(r), { target: { value: '2026-12-01' } });
		await fireEvent.input(au(r), { target: { value: '2026-02-01' } });
		await fireEvent.focusOut(au(r), { relatedTarget: null });
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(r.getByTestId('report-period-invalid')).toBeTruthy();
		expect(r.getByText('Caisse')).toBeTruthy();
		expect(r.getByTestId('report-content').className).not.toContain('opacity-60');
	});

	it('AC-20 — choix d’un exercice dans le menu : ses bornes, 1 requête ; le rapport classique reste, grisé (P17 retiré)', async () => {
		const r = await monter('/reports');
		const finir = suspendre(m.getBalanceSheet, bilan);
		await fireEvent.change(r.getByRole('combobox', { name: 'Exercice' }), { target: { value: '1' } });
		await tick();
		expect(r.queryByText('Caisse')).not.toBeNull();
		expect(r.getByTestId('report-content').className).toContain('opacity-60');
		finir();
		await pause();
		expect(du(r).value).toBe('2025-01-01');
		expect(au(r).value).toBe('2025-12-31');
		expect(m.getBalanceSheet.mock.calls[1][0]).toEqual({ fiscalYearId: 1, periodStart: '2025-01-01', periodEnd: '2025-12-31' });
	});

	it('AC-21 — la dernière demande l’emporte : la première, qui répond après, est ignorée', async () => {
		const r = await monter('/reports');
		const finirPremiere = suspendre(m.getBalanceSheet, bilan);
		await fireEvent.input(au(r), { target: { value: '2026-03-31' } });
		await fireEvent.keyDown(au(r), { key: 'Enter' });
		const finirSeconde = suspendre(m.getBalanceSheet, bilan);
		await fireEvent.input(au(r), { target: { value: '2026-06-30' } });
		await fireEvent.keyDown(au(r), { key: 'Enter' });
		finirSeconde();
		await pause();
		finirPremiere();
		await pause();
		expect(r.getByTestId('report-period-heading').textContent).toContain('au 30.06.2026');
		expect(r.queryByRole('alert')).toBeNull();
	});

	it('R-17 — retour aux dates du rapport affiché pendant une demande en vol : elle est périmée, aucune requête', async () => {
		const r = await monter('/reports');
		const finir = suspendre(m.getBalanceSheet, bilan);
		await fireEvent.input(au(r), { target: { value: '2026-03-31' } });
		await fireEvent.keyDown(au(r), { key: 'Enter' });
		await fireEvent.input(au(r), { target: { value: '2026-12-31' } });
		await fireEvent.keyDown(au(r), { key: 'Enter' });
		expect(appels(m.getBalanceSheet)).toBe(2);
		finir();
		await pause();
		expect(r.getByTestId('report-period-heading').textContent).toContain('au 31.12.2026');
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false');
	});

	it('AC-22, R-19 — dates tapées puis clic sur un autre onglet (relatedTarget nul, Safari) : l’onglet quitté ne charge pas, le visé charge avec ces dates', async () => {
		const r = await monter('/reports');
		await fireEvent.input(au(r), { target: { value: '2026-06-30' } });
		const cible = onglet(r, 'Compte de résultat');
		await fireEvent.pointerDown(cible);
		await fireEvent.focusOut(au(r), { relatedTarget: null });
		await fireEvent.click(cible);
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(m.getIncomeStatement.mock.calls[0][0]).toMatchObject({ periodEnd: '2026-06-30' });
		expect(rechercheCourante()).toBe('tab=income-statement&fy=2&from=2026-01-01&to=2026-06-30');
		retour(); // l'onglet quitté garde son rapport et sa clé : sans requête
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(1);
	});

	it('AC-22 — focus qui part vers un onglet (relatedTarget dans la liste) : l’onglet quitté ne charge pas', async () => {
		const r = await monter('/reports');
		await fireEvent.input(au(r), { target: { value: '2026-06-30' } });
		await fireEvent.focusOut(au(r), { relatedTarget: onglet(r, 'Balance') });
		expect(appels(m.getBalanceSheet)).toBe(1);
	});

	it('AC-24 — premier chargement « Chargement… » ; rechargement : ancien rapport grisé, aria-busy', async () => {
		const finir1 = suspendre(m.getBalanceSheet, bilan);
		const r = await monter('/reports');
		expect(r.getByTestId('report-loading').textContent).toContain('Chargement…');
		finir1();
		await pause();
		const finir2 = suspendre(m.getBalanceSheet, bilan);
		await fireEvent.input(au(r), { target: { value: '2026-06-30' } });
		await fireEvent.keyDown(au(r), { key: 'Enter' });
		await tick();
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('true');
		expect(r.getByTestId('report-content').className).toContain('opacity-60');
		expect(r.getByText('Caisse')).toBeTruthy();
		finir2();
		await pause();
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false');
		expect(r.getByTestId('report-content').className).not.toContain('opacity-60');
	});

	it('R-18 — menu et dates restent actifs pendant un premier chargement : le focus reste dans « Au »', async () => {
		const r = await monter('/reports');
		suspendre(m.getBalanceSheet, bilan);
		au(r).focus();
		await fireEvent.input(au(r), { target: { value: '2026-05-31' } });
		await fireEvent.keyDown(au(r), { key: 'Enter' });
		await tick();
		expect(au(r).disabled).toBe(false);
		expect((r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement).disabled).toBe(false);
		expect(document.activeElement).toBe(au(r));
	});

	it('AC-25, R-20 — dates tapées puis clic sur Export : aucun export de l’ancienne période ; l’export suit le rapport affiché', async () => {
		const r = await monter('/reports');
		const exporter = () => r.getByRole('button', { name: 'Export CSV' }) as HTMLButtonElement;
		expect(exporter().disabled).toBe(false);
		const finir = suspendre(m.getBalanceSheet, bilan);
		await fireEvent.input(au(r), { target: { value: '2026-06-30' } });
		expect(exporter().disabled).toBe(true); // champs ≠ rapport affiché
		await fireEvent.focusOut(au(r), { relatedTarget: exporter() });
		await fireEvent.click(exporter());
		expect(m.downloadReport).not.toHaveBeenCalled();
		finir();
		await waitFor(() => expect(exporter().disabled).toBe(false));
		await fireEvent.click(exporter());
		await waitFor(() => expect(m.downloadReport).toHaveBeenCalledTimes(1));
		expect((m.downloadReport.mock.calls[0] as unknown[])[1]).toEqual({ fiscalYearId: 2, periodStart: '2026-01-01', periodEnd: '2026-06-30' });
	});

	it('AC-26 — dates dans deux exercices : rien n’est chargé, rapport retiré, message ; adresse inchangée ; un seul exercice : chargé', async () => {
		const r = await monter('/reports');
		await fireEvent.input(du(r), { target: { value: '2025-06-01' } });
		await fireEvent.focusOut(du(r), { relatedTarget: null });
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(r.queryByText('Caisse')).toBeNull();
		expect(r.getByTestId('report-needs-one-fy')).toBeTruthy();
		const select = r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement;
		expect(select.value).toBe('__multiple');
		expect(rechercheCourante()).toBe(URL_BILAN_26);
		await fireEvent.input(du(r), { target: { value: '2026-06-01' } });
		await fireEvent.focusOut(du(r), { relatedTarget: null });
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(2);
		expect(r.queryByTestId('report-needs-one-fy')).toBeNull();
	});

	it('« Exercice entier » : les bornes, et le rapport de l’exercice repris de la mémoire (0 requête)', async () => {
		const r = await monter('/reports');
		await fireEvent.input(du(r), { target: { value: '2026-04-01' } });
		await fireEvent.click(r.getByTestId('report-fy-whole'));
		expect(du(r).value).toBe('2026-01-01');
		expect(r.queryByTestId('report-fy-whole')).toBeNull();
		expect(appels(m.getBalanceSheet)).toBe(1);
	});
});

describe('exercice et rapports par compte analytique (R-8, R-10, AC-9)', () => {
	it('changement d’exercice : le rapport par compte analytique est vidé, son compte reste ; l’export suit le rapport affiché', async () => {
		m.listProjects.mockResolvedValue([{ id: 5, code: 'P5', name: 'Projet', parentId: null }]);
		m.getProjectExpenses.mockImplementation(async () => ({
			reportType: 'project-expenses',
			project: { id: 5, code: 'P5', name: 'Projet' },
			mode: 'fiscal_year',
			periodLabel: '2026',
			sections: [],
			grandTotal: '0',
		}));
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'Dépenses par compte analytique'));
		await fireEvent.change(r.getByTestId('project-report-project'), { target: { value: '5' } });
		await fireEvent.click(r.getByTestId('project-report-generate'));
		await waitFor(() => expect(appels(m.getProjectExpenses)).toBe(1));
		await waitFor(() => expect((r.getByTestId('project-report-export-csv') as HTMLButtonElement).disabled).toBe(false));
		// dates changées sans Générer : l'export garde la requête affichée
		await fireEvent.input(au(r), { target: { value: '2026-03-31' } });
		await fireEvent.click(r.getByTestId('project-report-export-csv'));
		await waitFor(() => expect(m.downloadProjectReport).toHaveBeenCalledTimes(1));
		expect((m.downloadProjectReport.mock.calls[0] as unknown[])[1]).toMatchObject({ periodEnd: '2026-12-31' });
		await fireEvent.change(r.getByRole('combobox', { name: 'Exercice' }), { target: { value: '1' } });
		await tick();
		expect((r.getByTestId('project-report-export-csv') as HTMLButtonElement).disabled).toBe(true);
		expect((r.getByTestId('project-report-project') as HTMLSelectElement).value).toBe('5');
	});

	it('AC-9 — une adresse d’un autre exercice : la demande qu’elle lance n’est pas périmée par le changement d’exercice', async () => {
		const r = await monter('/reports');
		naviguer('/reports?tab=balance-sheet&fy=1&from=2025-01-01&to=2025-12-31');
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(2);
		expect(r.getByTestId('report-period-heading').textContent).toContain('Exercice 2025');
	});
});

describe('X1 de la 46-2 rejoué avec une adresse qui suit goto (AC-6)', () => {
	it('niveau changé sur le Bilan, chargement du Compte de résultat, retour sur le Bilan : redemandé à la fin', async () => {
		const groupe = (q: ReportQuery, p?: Record<string, unknown>) => ({
			...bilan(q),
			...(p
				? {
						grouping: { ...p, maxDepth: 2 },
						groupedRows: {
							assets: [{ kind: 'group', depth: 1, accountId: 90, accountNumber: '10', accountName: 'Actifs', balance: '100', open: true }],
							liabilities: [],
							equity: [],
						},
					}
				: {}),
		});
		m.getBalanceSheet.mockImplementation(async (q: ReportQuery, p?: Record<string, unknown>) => groupe(q, p));
		const r = await monter('/reports');
		await r.findByTestId('grouped-group-10');
		const finirResultat = suspendre(m.getIncomeStatement, resultat);
		let finirRedemande!: () => void;
		m.getBalanceSheet.mockImplementationOnce(
			(q: ReportQuery, p?: Record<string, unknown>) => new Promise((res) => (finirRedemande = () => res(groupe(q, p)))),
		);
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		expect(appels(m.getBalanceSheet)).toBe(2);
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		finirRedemande(); // périmée par le chargement du compte de résultat
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		expect(onglet(r, 'Bilan').getAttribute('aria-selected')).toBe('true');
		const avant = appels(m.getBalanceSheet);
		finirResultat();
		await pause();
		// la règle de la 46-2 redemande le bilan à la fin de l'autre chargement, sans passer à vide
		expect(appels(m.getBalanceSheet)).toBe(avant + 1);
		expect(m.getBalanceSheet.mock.calls.at(-1)?.[1]).toMatchObject({ detailLevel: 1 });
		expect(rechercheCourante()).toBe(URL_BILAN_26);
	});
});
