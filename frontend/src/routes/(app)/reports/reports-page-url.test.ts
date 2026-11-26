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
// Story 55-1b (D4) — la présentation des contrôles (vue par défaut : mémoire vide), écrite après les paramètres.
const PRES = '&view=grouped&level=all&subtotals=header';
const URL_BILAN_26_P = URL_BILAN_26 + PRES;

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
		expect(rechercheCourante()).toBe(URL_BILAN_26_P);
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
		// Story 55-1b — une seule normalisation, sans entrée : la présentation des contrôles s'ajoute.
		expect(m.goto).toHaveBeenCalledTimes(1);
		expect(rechercheCourante()).toBe('tab=income-statement&fy=1&from=2025-02-01&to=2025-03-31' + PRES);
		expect(historique.entrees).toHaveLength(1);
	});

	it.each([
		['tab=xyz', URL_BILAN_26_P],
		['tab=balance-sheet&fy=999999', URL_BILAN_26_P],
		['tab=balance-sheet&fy=abc', URL_BILAN_26_P],
		['tab=balance-sheet&from=2026-02-30&to=2026-03-31', URL_BILAN_26_P],
		['tab=balance-sheet&fy=2&from=2026-01-01&to=2026-12-31&accountId=4', URL_BILAN_26_P],
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
		expect(rechercheCourante()).toBe('tab=balance-sheet&fy=2&from=2026-01-01&to=2026-06-30' + PRES);
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
		expect(rechercheCourante()).toBe('tab=income-statement&fy=2&from=2026-01-01&to=2026-12-31' + PRES);
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

	it('Story 55-4 (L2) — grand livre de tous les comptes : « Du » tapé sans Générer, aller-retour par un onglet : la date tapée reste, 0 requête', async () => {
		// Le clic sur l'onglet du grand livre (tous les comptes, qui garde *Générer*) écrit la clé AFFICHÉE, non celle
		// des champs (`urlForTab`) : la page ne doit pas réappliquer l'adresse qu'elle vient d'écrire.
		// Mutation tuée : `writeUrl` qui ne pose pas `lastHandledKey` (« clé écrite non enregistrée ») → l'effet
		// réapplique l'adresse du clic, « Du » revient au 2026-01-01, rouge.
		const r = await monter('/reports?tab=general-ledger&from=2026-01-01&to=2026-06-30');
		const lDu = () => r.getByTestId('ledger-from') as HTMLInputElement;
		expect(appels(m.getGeneralLedger)).toBe(1);
		await fireEvent.input(lDu(), { target: { value: '2026-03-01' } });
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		await fireEvent.click(onglet(r, 'Grand livre'));
		await tick();
		await pause();
		expect(rechercheCourante()).toBe('tab=general-ledger&from=2026-01-01&to=2026-06-30');
		expect(lDu().value).toBe('2026-03-01');
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
		expect(rechercheCourante()).toBe('tab=balance-sheet&fy=2&from=2026-04-01&to=2026-12-31' + PRES);
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
		expect(rechercheCourante()).toBe('tab=income-statement&fy=2&from=2026-01-01&to=2026-06-30' + PRES);
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
		expect(rechercheCourante()).toBe(URL_BILAN_26_P);
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
		expect(rechercheCourante()).toBe(URL_BILAN_26 + '&view=grouped&level=1&subtotals=header'); // 55-1b : la réponse écrit la clé affichée
	});
});

// =====================================================================================
// Story 55-1b (D4) — la présentation des rapports groupés dans l'URL : lue à l'arrivée et à chaque
// navigation (l'URL prime sur la mémoire), écrite par la seule réponse affichée (écho compris).
// =====================================================================================

describe('la présentation dans l’URL (Story 55-1b)', () => {
	const CLE_MEMOIRE = 'kesh-report-grouping:7:balance-sheet';
	/** Un bilan groupé : l'écho de l'API rend les paramètres reçus, le groupe 77 « disparu du plan » en moins. */
	const groupe = (q: ReportQuery, p?: Record<string, unknown>) => ({
		...bilan(q),
		...(p
			? {
					grouping: {
						...p,
						open: ((p.open as number[]) ?? []).filter((id) => id !== 77),
						closed: ((p.closed as number[]) ?? []).filter((id) => id !== 77),
						maxDepth: 5,
					},
					groupedRows: {
						assets: [{ kind: 'group', depth: 1, accountId: 90, accountNumber: '10', accountName: 'Actifs', balance: '100', open: true }],
						liabilities: [],
						equity: [],
					},
				}
			: {}),
	});
	const niveau = (r: R) => r.getByTestId('grouping-level') as HTMLSelectElement;
	const changerNiveau = (r: R, v: string) => fireEvent.change(niveau(r), { target: { value: v } });
	/** Les paramètres de regroupement de la dernière demande du bilan. */
	const dernierBilan = () => m.getBalanceSheet.mock.calls.at(-1)?.[1];
	/** Une redemande du bilan suspendue. */
	function bilanSuspendu() {
		let finir!: () => void;
		m.getBalanceSheet.mockImplementationOnce(
			(q: ReportQuery, p?: Record<string, unknown>) => new Promise((res) => (finir = () => res(groupe(q, p)))),
		);
		return () => finir();
	}
	const memoire = () => JSON.parse(localStorage.getItem(CLE_MEMOIRE) ?? 'null');
	const avec = (pres: string) => `${URL_BILAN_26}${pres}`;
	const L2 = '&view=grouped&level=2&subtotals=header';
	const L3 = '&view=grouped&level=3&subtotals=header';

	beforeEach(() => {
		m.getBalanceSheet.mockImplementation(async (q: ReportQuery, p?: Record<string, unknown>) => groupe(q, p));
	});

	it('AC-1, AC-2 — une adresse avec présentation l’emporte sur la mémoire, qui la prend ensuite', async () => {
		localStorage.setItem(CLE_MEMOIRE, JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'header', open: [], closed: [] }));
		const r = await monter(`/reports?${avec('&view=grouped&level=3&subtotals=footer&open=10')}`);
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(dernierBilan()).toEqual({ detailLevel: 3, subtotalPosition: 'footer', open: [10], closed: [] });
		expect(niveau(r).value).toBe('3');
		expect((r.getByTestId('grouping-position') as HTMLSelectElement).value).toBe('footer');
		expect(memoire()).toMatchObject({ detailLevel: 3, subtotalPosition: 'footer', open: [10] });
		expect(m.goto).not.toHaveBeenCalled(); // canonique, la réponse égale l'adresse
	});

	it('AC-3 — adresse complète sans présentation : la vue des contrôles (mémoire), 1 requête, 1 goto', async () => {
		localStorage.setItem(CLE_MEMOIRE, JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'header', open: [], closed: [] }));
		await monter(`/reports?${URL_BILAN_26}`);
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(dernierBilan()).toMatchObject({ detailLevel: 2 });
		expect(m.goto).toHaveBeenCalledTimes(1);
		expect(rechercheCourante()).toBe(avec(L2));
	});

	it('AC-3 — bilan gardé par la page avec cette vue : 0 requête, 1 goto', async () => {
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		m.goto.mockClear();
		const avant = appels(m.getBalanceSheet);
		naviguer(`/reports?${URL_BILAN_26}`); // un lien sans présentation
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant);
		expect(m.goto).toHaveBeenCalledTimes(1);
		expect(rechercheCourante()).toBe(URL_BILAN_26_P);
	});

	it('AC-4, AC-6 — Retour vers une autre présentation : le Bilan au niveau 2, contrôles au niveau 2', async () => {
		const r = await monter('/reports');
		await r.findByTestId('grouped-group-10');
		await changerNiveau(r, '2');
		await pause();
		expect(rechercheCourante()).toBe(avec(L2)); // la réponse a écrit la présentation
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		expect(rechercheCourante()).toBe(avec(L2)); // le clic empile la clé affichée
		await changerNiveau(r, '3');
		await pause();
		expect(rechercheCourante()).toBe(avec(L3));
		const avant = appels(m.getBalanceSheet);
		retour();
		await tick();
		await pause();
		retour();
		await tick();
		await pause();
		expect(onglet(r, 'Bilan').getAttribute('aria-selected')).toBe('true');
		expect(niveau(r).value).toBe('2');
		expect(appels(m.getBalanceSheet)).toBe(avant + 1);
		expect(dernierBilan()).toMatchObject({ detailLevel: 2 });
		expect(rechercheCourante()).toBe(avec(L2));
	});

	it('AC-6 — un geste de présentation n’écrit pas l’URL : 0 goto avant la réponse, 1 à la réponse, 2 écritures en mémoire', async () => {
		const r = await monter('/reports');
		await r.findByTestId('grouped-group-10');
		m.goto.mockClear();
		const ecrire = vi.spyOn(Storage.prototype, 'setItem');
		const finir = bilanSuspendu();
		await changerNiveau(r, '2');
		await tick();
		expect(m.goto).not.toHaveBeenCalled();
		expect(appels(m.getBalanceSheet)).toBe(2);
		finir();
		await pause();
		expect(m.goto).toHaveBeenCalledTimes(1);
		expect(m.goto.mock.calls[0][1]).toMatchObject({ replaceState: true });
		expect(rechercheCourante()).toBe(avec(L2));
		expect(ecrire.mock.calls.filter((c) => c[0] === CLE_MEMOIRE)).toHaveLength(2);
		ecrire.mockRestore();
	});

	it('AC-6 — sans rapport affiché ni chargement : un geste de présentation ne fait rien (0 goto, 0 requête)', async () => {
		m.getBalanceSheet.mockImplementationOnce(async () => {
			throw new Error('réseau');
		});
		const r = await monter('/reports');
		m.goto.mockClear();
		await changerNiveau(r, 'all');
		await fireEvent.click(r.getByTestId('grouping-shown'));
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(m.goto).not.toHaveBeenCalled();
	});

	it('AC-5 — X1 par Retour : la vue de l’entrée, 0 requête, l’autre chargement n’est pas interrompu', async () => {
		const r = await monter('/reports');
		await r.findByTestId('grouped-group-10');
		const finirResultat = suspendre(m.getIncomeStatement, resultat);
		bilanSuspendu();
		await changerNiveau(r, '2'); // redemande en vol
		await fireEvent.click(onglet(r, 'Compte de résultat')); // la périme
		const avant = appels(m.getBalanceSheet);
		retour(); // vers le Bilan tel qu'affiché
		await tick();
		await pause();
		expect(niveau(r).value).toBe('all');
		expect(appels(m.getBalanceSheet)).toBe(avant);
		finirResultat();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant); // la règle de la 46-2 ne redemande rien : accord
		// le compte de résultat a bien été accepté : revenir dessus ne le redemande pas
		const avantResultat = appels(m.getIncomeStatement);
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		expect(appels(m.getIncomeStatement)).toBe(avantResultat);
	});

	it('V5-5 (a) — clic, contrôles différents de l’affiché, rien en vol : 1 requête, 2 goto (clé affichée, puis réponse)', async () => {
		const r = await monter('/reports');
		await r.findByTestId('grouped-group-10');
		bilanSuspendu();
		await changerNiveau(r, '2');
		await fireEvent.click(onglet(r, 'Compte de résultat')); // périme la redemande
		await pause();
		m.goto.mockClear();
		const avant = appels(m.getBalanceSheet);
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant + 1);
		expect(dernierBilan()).toMatchObject({ detailLevel: 2 });
		expect(m.goto).toHaveBeenCalledTimes(2);
		expect(m.goto.mock.calls[0][0]).toBe(`?${URL_BILAN_26_P}`);
		expect(m.goto.mock.calls[0][1]).toMatchObject({ replaceState: false });
		expect(rechercheCourante()).toBe(avec(L2));
	});

	it('flèches du clavier comme le clic : la clé affichée, présentation comprise, en remplaçant', async () => {
		const r = await monter(`/reports?${avec(L2)}`);
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		const longueur = historique.entrees.length;
		await fireEvent.keyDown(onglet(r, 'Compte de résultat'), { key: 'ArrowLeft' });
		await pause();
		expect(rechercheCourante()).toBe(avec(L2));
		expect(historique.entrees).toHaveLength(longueur);
	});

	it('AC-7 — écho : un groupe disparu sort de l’URL et de la mémoire ; Retour puis Suivant : 0 requête', async () => {
		const r = await monter(`/reports?${avec('&view=grouped&level=2&subtotals=header&open=10%2C77')}`);
		expect(dernierBilan()).toMatchObject({ open: [10, 77] });
		expect(rechercheCourante()).toBe(avec('&view=grouped&level=2&subtotals=header&open=10'));
		expect(memoire()).toMatchObject({ open: [10] });
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		const avant = appels(m.getBalanceSheet);
		retour();
		await tick();
		await pause();
		suivant();
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant);
	});

	it.each([
		['&level=0', PRES],
		['&level=21', PRES],
		['&level=03', PRES],
		['&level=3.0', PRES],
		['&level=%2B3', PRES],
		['&level=ALL', PRES],
		['&open=1,a', PRES],
		['&open=1,,2', PRES],
		['&open=99999999999999999999', PRES],
		['&closed=x', PRES],
		['&view=xyz', PRES],
		['&open=', PRES],
		['&open=5&closed=5', PRES],
		['&view=flat&level=3', '&view=flat'],
	])('AC-8 — %s : ignoré, adresse normalisée, rien d’invalide envoyé', async (pres, attendu) => {
		await monter(`/reports?${URL_BILAN_26}${pres}`);
		expect(appels(m.getBalanceSheet)).toBe(1);
		expect(dernierBilan()).toEqual(
			attendu === '&view=flat' ? undefined : { detailLevel: 'all', subtotalPosition: 'header', open: [], closed: [] },
		);
		expect(rechercheCourante()).toBe(URL_BILAN_26 + attendu);
		expect(historique.entrees).toHaveLength(1);
	});

	it('AC-8 — 501 identifiants : la liste est ignorée', async () => {
		const ids = Array.from({ length: 501 }, (_, i) => i + 1).join(',');
		await monter(`/reports?${URL_BILAN_26}&open=${ids}`);
		expect(dernierBilan()).toMatchObject({ open: [] });
		expect(rechercheCourante()).toBe(URL_BILAN_26_P);
	});

	it('AC-8 — un paramètre invalide seul laisse la vue des contrôles (mémoire)', async () => {
		localStorage.setItem(CLE_MEMOIRE, JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'header', open: [], closed: [] }));
		await monter(`/reports?${URL_BILAN_26}&level=0&open=5&closed=5`);
		expect(dernierBilan()).toMatchObject({ detailLevel: 2 });
		expect(rechercheCourante()).toBe(avec(L2));
	});

	it('C-55-1b-4 — une adresse incomplète ne lit pas sa présentation : le rapport des champs, vue des contrôles', async () => {
		const r = await monter('/reports?tab=balance-sheet&level=3');
		expect(dernierBilan()).toMatchObject({ detailLevel: 'all' });
		expect(niveau(r).value).toBe('all');
		expect(rechercheCourante()).toBe(URL_BILAN_26_P);
	});

	it('AC-9 — view=flat sur des contrôles au niveau 2 : à plat, puis « hiérarchie montrée » rend le niveau 2', async () => {
		localStorage.setItem(CLE_MEMOIRE, JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'footer', open: [], closed: [] }));
		const r = await monter(`/reports?${avec('&view=flat')}`);
		expect(dernierBilan()).toBeUndefined();
		expect((r.getByTestId('grouping-shown') as HTMLInputElement).checked).toBe(false);
		expect(rechercheCourante()).toBe(avec('&view=flat'));
		await fireEvent.click(r.getByTestId('grouping-shown'));
		await pause();
		expect(dernierBilan()).toEqual({ detailLevel: 2, subtotalPosition: 'footer', open: [], closed: [] });
		expect(rechercheCourante()).toBe(avec('&view=grouped&level=2&subtotals=footer'));
	});

	it('AC-10 — Retour deux fois pendant une redemande de l’onglet : 0 requête, redemande périmée, écran et contrôles au niveau 2', async () => {
		const r = await monter(`/reports?${avec(L2)}`);
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		const finir = bilanSuspendu();
		await changerNiveau(r, '3');
		const avant = appels(m.getBalanceSheet);
		retour();
		await tick();
		await pause();
		retour();
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant);
		expect(niveau(r).value).toBe('2');
		finir(); // périmée : ni affichée, ni écrite
		await pause();
		expect(rechercheCourante()).toBe(avec(L2));
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false');
		expect(memoire()).toMatchObject({ detailLevel: 2 });
	});

	it('AC-10 — de même avec une autre période en vol (dates changées et quittées), Retour deux fois : 0 requête', async () => {
		const r = await monter(`/reports?${avec(L2)}`);
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		const finir = bilanSuspendu();
		await fireEvent.input(du(r), { target: { value: '2026-04-01' } });
		await fireEvent.focusOut(du(r), { relatedTarget: null });
		const avant = appels(m.getBalanceSheet);
		retour();
		await tick();
		await pause();
		expect(onglet(r, 'Compte de résultat').getAttribute('aria-selected')).toBe('true');
		retour();
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant);
		finir();
		await pause();
		expect(r.getByTestId('report-period-heading').textContent).toContain('Exercice 2026 — du 01.01.2026 au 31.12.2026');
		expect(rechercheCourante()).toBe(avec(L2));
	});

	it('AC-10 — une adresse égale à la clé en vol : 0 requête ; la réponse égale l’adresse', async () => {
		const r = await monter(`/reports?${avec(L2)}`);
		const finir = bilanSuspendu();
		await changerNiveau(r, '3');
		const avant = appels(m.getBalanceSheet);
		naviguer(`/reports?${avec(L3)}`);
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant);
		m.goto.mockClear();
		finir();
		await pause();
		expect(m.goto).not.toHaveBeenCalled();
		expect(rechercheCourante()).toBe(avec(L3));
	});

	it('V6-2 — une troisième clé pendant une redemande de l’onglet : 1 requête, la redemande ancienne ne s’affiche pas', async () => {
		const r = await monter(`/reports?${avec(L2)}`);
		const finir = bilanSuspendu();
		await changerNiveau(r, '3');
		const avant = appels(m.getBalanceSheet);
		naviguer(`/reports?${avec('&view=grouped&level=4&subtotals=header')}`);
		await tick();
		await pause();
		expect(appels(m.getBalanceSheet)).toBe(avant + 1);
		expect(dernierBilan()).toMatchObject({ detailLevel: 4 });
		finir(); // la redemande du niveau 3 répond après : périmée
		await pause();
		expect(niveau(r).value).toBe('4');
		expect(rechercheCourante()).toBe(avec('&view=grouped&level=4&subtotals=header'));
	});

	it('R-2 — Retour vers d’autres paramètres pendant une redemande : le rapport reste affiché, grisé, puis celui de l’adresse', async () => {
		const r = await monter(`/reports?${avec(L2)}`);
		bilanSuspendu();
		await changerNiveau(r, '3');
		const finirAutre = bilanSuspendu();
		naviguer('/reports?tab=balance-sheet&fy=2&from=2026-01-01&to=2026-06-30');
		await tick();
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('true');
		expect(r.getByTestId('report-content').className).toContain('opacity-60');
		finirAutre();
		await pause();
		expect(r.getByTestId('report-period-heading').textContent).toContain('30.06.2026');
		expect(rechercheCourante()).toBe('tab=balance-sheet&fy=2&from=2026-01-01&to=2026-06-30' + L3);
	});

	it.each([
		['un autre onglet', 'tab=income-statement&fy=2&from=2026-01-01&to=2026-12-31'],
		['des dates dans deux exercices', 'tab=balance-sheet&from=2025-06-01&to=2026-06-30'],
	])('R-1 — navigation vers %s pendant une redemande : la réponse ne réécrit pas l’adresse', async (_nom, cible) => {
		const r = await monter(`/reports?${avec(L2)}`);
		const finir = bilanSuspendu();
		await changerNiveau(r, '3');
		naviguer(`/reports?${cible}`);
		await tick();
		await pause();
		const ici = rechercheCourante();
		expect(reportPartDe(ici)).toBe(reportPartDe(cible));
		finir();
		await pause();
		expect(rechercheCourante()).toBe(ici);
	});

	it('T0 — un contrôle changé pendant le rechargement d’une autre période : la période en vol est gardée', async () => {
		const r = await monter(`/reports?${avec(L2)}`);
		bilanSuspendu();
		await fireEvent.input(du(r), { target: { value: '2026-04-01' } });
		await fireEvent.focusOut(du(r), { relatedTarget: null });
		await changerNiveau(r, '3');
		await pause();
		expect(m.getBalanceSheet.mock.calls.at(-1)?.[0]).toMatchObject({ periodStart: '2026-04-01' });
		expect(dernierBilan()).toMatchObject({ detailLevel: 3 });
		expect(rechercheCourante()).toBe('tab=balance-sheet&fy=2&from=2026-04-01&to=2026-12-31' + L3);
	});

	it('AC-11 — export désactivé pendant un chargement lancé par Retour, puis la présentation de l’entrée', async () => {
		const r = await monter('/reports');
		await r.findByTestId('grouped-group-10');
		await changerNiveau(r, '2');
		await pause();
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		await changerNiveau(r, '3');
		await pause();
		const exporter = () => r.getByRole('button', { name: 'Export CSV' }) as HTMLButtonElement;
		const finir = bilanSuspendu();
		retour();
		await tick();
		retour(); // vers l'entrée du niveau 2 : redemandée
		await tick();
		await pause();
		expect(exporter().disabled).toBe(true);
		finir();
		await pause();
		expect(exporter().disabled).toBe(false);
		await fireEvent.click(exporter());
		await waitFor(() => expect(m.downloadReport).toHaveBeenCalledTimes(1));
		expect((m.downloadReport.mock.calls[0] as unknown[])[4]).toMatchObject({ detailLevel: 2 });
	});

	it('stockage refusé, puis Retour vers une entrée sans présentation : les contrôles ne changent pas', async () => {
		const refus = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new Error('refusé');
		});
		const r = await monter('/reports');
		await r.findByTestId('grouped-group-10');
		await changerNiveau(r, '2');
		await pause();
		await fireEvent.click(onglet(r, 'Compte de résultat'));
		await pause();
		naviguer(`/reports?${URL_BILAN_26}`);
		await tick();
		await pause();
		expect(niveau(r).value).toBe('2');
		expect(rechercheCourante()).toBe(avec(L2));
		refus.mockRestore();
	});
});

/** La partie « rapport » d'une recherche (sans la présentation), pour les tests de la garde R-1. */
function reportPartDe(s: string): string {
	const qs = new URLSearchParams(s);
	for (const n of ['view', 'level', 'subtotals', 'open', 'closed']) qs.delete(n);
	return qs.toString();
}
