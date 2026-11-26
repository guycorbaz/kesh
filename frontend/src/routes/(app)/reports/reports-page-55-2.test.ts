// Story 55-2 — au niveau de la page des Rapports : le sélecteur d'exercice du Grand livre (AC-11 à AC-14),
// le chargement du Grand livre filtré (D16), les comptes de TVA lus au premier affichage du décompte
// (AC-6) et la carte numéro → compte des contreparties (AC-4).
//
// Même assistant que `reports-page-url.test.ts` (55-1a : `page.url` réactive, historique simulé) ; fichier
// à part parce qu'il simule en plus la liste des comptes, les réglages de facturation, le journal et la
// TVA. ⚠️ Chaque test nomme la mutation qu'il attrape.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor, within } from '@testing-library/svelte';
import { tick } from 'svelte';
import type { GeneralLedgerDto, ReportQuery, VatReportDto } from '$lib/features/reports/reports.types';
import { arriverSur, naviguer, pageState, rechercheCourante } from './page-url.test-helper.svelte';

const m = vi.hoisted(() => ({
	getBalanceSheet: vi.fn(),
	getGeneralLedger: vi.fn(),
	getVatReport: vi.fn(),
	getProjectExpenses: vi.fn(),
	listProjects: vi.fn(async () => [] as unknown[]),
	fetchAccounts: vi.fn(),
	getInvoiceSettings: vi.fn(),
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
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: m.fetchAccounts }));
vi.mock('$lib/features/invoices/invoices.api', () => ({ getInvoiceSettings: m.getInvoiceSettings }));
vi.mock('$lib/features/reports/reports.api', async (importOriginal) => {
	const vrai = await importOriginal<typeof import('$lib/features/reports/reports.api')>();
	return {
		...vrai,
		getBalanceSheet: m.getBalanceSheet,
		getGeneralLedger: m.getGeneralLedger,
		getVatReport: m.getVatReport,
		getProjectExpenses: m.getProjectExpenses,
	};
});

import Page from './+page.svelte';

const FY25 = { id: 1, companyId: 7, name: 'Exercice 2025', startDate: '2025-01-01', endDate: '2025-12-31', status: 'Open', multiCurrency: false };
const FY26 = { id: 2, companyId: 7, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Open', multiCurrency: false };
const data = { fiscalYears: [FY26, FY25], companyName: 'Test SA', companyId: 7 } as never;

const COMPTES = [
	{ id: 11, number: '1170', name: 'Impôt préalable', active: true, accountType: 'Asset' },
	{ id: 20, number: '2200', name: 'TVA due', active: true, accountType: 'Liability' },
	{ id: 60, number: '6000', name: 'Loyer', active: true, accountType: 'Expense' },
	{ id: 7, number: '1020', name: 'Banque', active: true, accountType: 'Asset' },
];

function bilan(q: ReportQuery) {
	return {
		period: { fiscalYearId: q.fiscalYearId, startDate: q.periodStart!, endDate: q.periodEnd! },
		assets: [],
		liabilities: [],
		equity: [],
		totalAssets: '0',
		totalLiabilities: '0',
		totalEquity: '0',
		retainedEarnings: '0',
		equityResult: '0',
		equationHolds: true,
	};
}
function tva(q: ReportQuery): VatReportDto {
	return {
		period: { fiscalYearId: q.fiscalYearId, startDate: q.periodStart!, endDate: q.periodEnd! },
		rows: [{ rate: '8.10', category: null, baseHt: '1000.00', vatDue: '81.00' }],
		totalBaseHt: '1000.00',
		totalVatDue: '81.00',
		writeOffRows: [],
		totalVatWriteOff: '0.00',
		totalVatDueNet: '81.00',
		totalVatRecoverable: '0.00',
		vatBalance: '81.00',
		reconciliationDelta: '0.00',
		reconciliationStatus: 'ok',
	} as VatReportDto;
}
/** Un grand livre d'une section (le compte 1020), dont la ligne a la contrepartie 6000. */
const livre = (q: { from: string; to: string }): GeneralLedgerDto =>
	({
		period: { from: q.from, to: q.to },
		sections: [
			{
				accountId: 7,
				accountNumber: '1020',
				accountName: 'Banque',
				accountType: 'Asset',
				active: true,
				balanceSide: 'debit',
				opening: '0',
				lines: [
					{
						lineId: 1,
						entryId: 1,
						entryDate: q.from,
						fiscalYearId: 2,
						fiscalYearName: 'Exercice 2026',
						entryNumber: 1,
						journal: 'OD',
						description: 'Loyer',
						counterpart: ['6000'],
						debit: '0',
						credit: '100',
						runningBalance: '-100',
						letteringCode: null,
					},
				],
				totalDebit: '0',
				totalCredit: '100',
				closing: '-100',
				unnaturalBalance: false,
				fiscalYearBreaks: [],
				lineCount: 1,
			},
		],
	}) as GeneralLedgerDto;

const appels = (fn: ReturnType<typeof vi.fn>) => fn.mock.calls.length;
const pause = () => new Promise((res) => setTimeout(res, 10));
type R = Awaited<ReturnType<typeof monter>>;
const onglet = (r: R, nom: string) => r.getByRole('tab', { name: nom });
const lDu = (r: R) => r.getByTestId('ledger-from') as HTMLInputElement;
const lAu = (r: R) => r.getByTestId('ledger-to') as HTMLInputElement;
const menuLivre = (r: R) =>
	within(r.getByTestId('ledger-controls')).getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement;

async function monter(href = '/reports', d = data) {
	arriverSur(href);
	const r = render(Page, { data: d });
	await tick();
	await pause();
	return r;
}

/** Saisit deux dates du grand livre puis quitte le groupe (focus vers le titre, hors du groupe). */
async function datesLivre(r: R, du: string, au: string) {
	await fireEvent.input(lDu(r), { target: { value: du } });
	await fireEvent.input(lAu(r), { target: { value: au } });
	await fireEvent.focusOut(lAu(r), { relatedTarget: document.body });
	await pause();
}

beforeEach(() => {
	vi.clearAllMocks();
	localStorage.clear();
	m.getBalanceSheet.mockImplementation(async (q: ReportQuery) => bilan(q));
	m.getVatReport.mockImplementation(async (q: ReportQuery) => tva(q));
	m.getGeneralLedger.mockImplementation(async (q: { from: string; to: string }) => livre(q));
	m.fetchAccounts.mockResolvedValue(COMPTES);
	m.getInvoiceSettings.mockResolvedValue({ defaultVatPayableAccountId: 20, defaultVatRecoverableAccountId: 11 });
	m.listProjects.mockResolvedValue([]);
	m.goto.mockImplementation((href: string, opts?: { replaceState?: boolean }) => {
		naviguer(href, opts?.replaceState === true);
		return Promise.resolve();
	});
	vi.spyOn(console, 'error').mockImplementation(() => {});
});

describe('Grand livre — le sélecteur d’exercice (AC-11 à AC-14)', () => {
	it('le bandeau est ReportSelector : menu « Exercice » dans ledger-controls ; arrivée par clic, dates vides → bornes de l’exercice choisi, Générer actif, aucune requête', async () => {
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'Grand livre'));
		await pause();
		// Rouge sans le correctif : aucun menu dans ledger-controls.
		expect(menuLivre(r).value).toBe('2');
		// Mutation : préremplissage retiré → dates vides, rouge.
		expect(lDu(r).value).toBe('2026-01-01');
		expect(lAu(r).value).toBe('2026-12-31');
		expect((r.getByTestId('ledger-generate') as HTMLButtonElement).disabled).toBe(false);
		// Tous les comptes : aucune requête, Générer attend.
		expect(appels(m.getGeneralLedger)).toBe(0);
	});

	it('adresse incomplète ?tab=general-ledger : dates pré-remplies, aucune requête', async () => {
		const r = await monter('/reports?tab=general-ledger');
		expect(lDu(r).value).toBe('2026-01-01');
		expect(lAu(r).value).toBe('2026-12-31');
		expect(appels(m.getGeneralLedger)).toBe(0);
		expect((r.getByTestId('ledger-generate') as HTMLButtonElement).disabled).toBe(false);
	});

	it('dates déjà remplies : elles restent à l’arrivée (garde du préremplissage inconditionnel)', async () => {
		const r = await monter('/reports?tab=general-ledger&from=2025-03-01&to=2025-04-30');
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		await fireEvent.click(onglet(r, 'Grand livre'));
		await pause();
		// Mutation : préremplissage inconditionnel → 2026-01-01, rouge.
		expect(lDu(r).value).toBe('2025-03-01');
		expect(lAu(r).value).toBe('2025-04-30');
		// Le menu en dérive : une partie de 2025 (et « Exercice entier »).
		expect(menuLivre(r).value).toBe('1');
		expect(r.getByTestId('report-fy-whole')).toBeTruthy();
	});

	it('deux dates vidées à la main restent vides (jamais de préremplissage à la saisie)', async () => {
		const r = await monter('/reports?tab=general-ledger');
		await datesLivre(r, '', '');
		expect(lDu(r).value).toBe('');
		expect(lAu(r).value).toBe('');
		expect((r.getByTestId('ledger-generate') as HTMLButtonElement).disabled).toBe(true);
	});

	it('choisir un exercice remplit « Du » et « Au » de ses bornes', async () => {
		const r = await monter('/reports?tab=general-ledger');
		await fireEvent.change(menuLivre(r), { target: { value: '1' } });
		await pause();
		expect(lDu(r).value).toBe('2025-01-01');
		expect(lAu(r).value).toBe('2025-12-31');
		expect(appels(m.getGeneralLedger)).toBe(0);
	});

	it('dates à cheval sur deux exercices : « Plusieurs » sélectionnée et désactivée, aucun message, Générer actif et générant', async () => {
		const r = await monter('/reports?tab=general-ledger');
		await datesLivre(r, '2025-07-01', '2026-06-30');
		expect(menuLivre(r).value).toBe('__multiple');
		expect((menuLivre(r).querySelector('option[value="__multiple"]') as HTMLOptionElement).disabled).toBe(true);
		// Mutation : `unSeulExercice` vrai → message affiché, rouge.
		expect(r.queryByTestId('report-needs-one-fy')).toBeNull();
		const generer = r.getByTestId('ledger-generate') as HTMLButtonElement;
		expect(generer.disabled).toBe(false);
		await fireEvent.click(generer);
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(1));
		expect(m.getGeneralLedger.mock.calls[0][0]).toMatchObject({ from: '2025-07-01', to: '2026-06-30' });
		await r.findByTestId('general-ledger');
	});

	it('date hors de tout exercice : « Hors exercice », Générer actif', async () => {
		const r = await monter('/reports?tab=general-ledger');
		await datesLivre(r, '2024-03-01', '2024-03-31');
		expect(menuLivre(r).value).toBe('__outside');
		expect((r.getByTestId('ledger-generate') as HTMLButtonElement).disabled).toBe(false);
	});

	it('le Grand livre n’écrit jamais l’exercice des rapports classiques : le rapport par compte analytique en mémoire reste', async () => {
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
		await r.findByTestId('project-expenses-view');
		await fireEvent.click(onglet(r, 'Grand livre'));
		await pause();
		// Un AUTRE exercice que celui des rapports classiques (2026).
		await fireEvent.change(menuLivre(r), { target: { value: '1' } });
		await pause();
		await fireEvent.click(onglet(r, 'Dépenses par compte analytique'));
		await pause();
		// Mutation : écriture de `selectedFiscalYearId` → `fiscalYearChanged()` vide le rapport, rouge.
		expect(r.queryByTestId('project-expenses-view')).not.toBeNull();
		expect((r.getByTestId('report-from') as HTMLInputElement).value).toBe('2026-01-01');
		expect(appels(m.getProjectExpenses)).toBe(1);
	});

	it('société sans exercice : menu désactivé, dates libres, Générer utilisable une fois les dates remplies', async () => {
		const r = await monter('/reports?tab=general-ledger', { fiscalYears: [], companyName: 'X', companyId: 7 } as never);
		expect(menuLivre(r).disabled).toBe(true);
		expect(lDu(r).disabled).toBe(false);
		expect(r.queryByRole('alert')).toBeNull();
		expect((r.getByTestId('ledger-generate') as HTMLButtonElement).disabled).toBe(true);
		await datesLivre(r, '2026-01-01', '2026-03-31');
		const generer = r.getByTestId('ledger-generate') as HTMLButtonElement;
		expect(generer.disabled).toBe(false);
		await fireEvent.click(generer);
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(1));
	});
});

describe('Grand livre filtré — chargé sans Générer (D16, AC-11, AC-12)', () => {
	it('choix d’un compte : une requête, l’adresse remplacée par sa clé ; plus de bouton Générer', async () => {
		const r = await monter('/reports?tab=general-ledger');
		await waitFor(() => expect(r.getByTestId('ledger-account').querySelectorAll('option').length).toBe(5));
		await fireEvent.change(r.getByTestId('ledger-account'), { target: { value: '7' } });
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(1));
		expect(m.getGeneralLedger.mock.calls[0][0]).toMatchObject({ from: '2026-01-01', to: '2026-12-31', accountIds: [7] });
		expect(rechercheCourante()).toBe('tab=general-ledger&accountId=7&from=2026-01-01&to=2026-12-31');
		expect(r.queryByTestId('ledger-generate')).toBeNull();
	});

	it('filtré : sortie des dates changées → une requête ; mêmes dates → aucune ; début après fin → aucune, message', async () => {
		const r = await monter('/reports?tab=general-ledger&accountId=7&from=2026-01-01&to=2026-12-31');
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(1));
		await datesLivre(r, '2026-01-01', '2026-12-31');
		expect(appels(m.getGeneralLedger)).toBe(1);
		await datesLivre(r, '2026-02-01', '2026-12-31');
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(2));
		await datesLivre(r, '2026-12-31', '2026-02-01');
		expect(appels(m.getGeneralLedger)).toBe(2);
		expect(r.getByTestId('report-period-invalid')).toBeTruthy();
	});

	it('filtré : « comptes sans mouvement » recharge ; le choix d’un exercice aussi', async () => {
		const r = await monter('/reports?tab=general-ledger&accountId=7&from=2026-01-01&to=2026-12-31');
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(1));
		await fireEvent.click(r.getByTestId('ledger-include-zero'));
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(2));
		expect(m.getGeneralLedger.mock.calls[1][0]).toMatchObject({ includeZero: true });
		await fireEvent.change(menuLivre(r), { target: { value: '1' } });
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(3));
		expect(m.getGeneralLedger.mock.calls[2][0]).toMatchObject({ from: '2025-01-01', to: '2025-12-31' });
	});

	it('arrivée par clic sur l’onglet, compte déjà choisi, dates vides : préremplies puis UNE requête', async () => {
		const r = await monter('/reports?tab=general-ledger');
		await waitFor(() => expect(r.getByTestId('ledger-account').querySelectorAll('option').length).toBe(5));
		await datesLivre(r, '', '');
		await fireEvent.change(r.getByTestId('ledger-account'), { target: { value: '7' } });
		expect(appels(m.getGeneralLedger)).toBe(0);
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		await fireEvent.click(onglet(r, 'Grand livre'));
		await waitFor(() => expect(appels(m.getGeneralLedger)).toBe(1));
		expect(lDu(r).value).toBe('2026-01-01');
	});
});

describe('Contreparties : la carte numéro → compte (AC-4)', () => {
	it('liste des comptes arrivée APRÈS le rapport : la contrepartie devient un lien', async () => {
		let livrer!: (v: unknown) => void;
		m.fetchAccounts.mockReturnValue(new Promise((res) => (livrer = res)));
		const r = await monter('/reports?tab=general-ledger&accountId=7&from=2026-01-01&to=2026-12-31');
		await r.findByTestId('general-ledger');
		expect(r.getByTestId('ledger-counterpart-1').querySelector('a')).toBeNull();
		livrer(COMPTES);
		await pause();
		// Mutation : carte calculée une fois (non dérivée) → texte, rouge.
		expect(r.getByTestId('ledger-counterpart-1').querySelector('a')?.getAttribute('href')).toBe(
			'/reports?tab=general-ledger&accountId=60&from=2026-01-01&to=2026-12-31',
		);
	});
});

describe('TVA — les comptes désignés, lus au premier affichage du décompte (AC-6)', () => {
	it('aucune lecture des réglages tant qu’aucun décompte n’est affiché', async () => {
		const r = await monter('/reports');
		await r.findByText(/Total actifs|Aucune écriture/);
		// Mutation : lecture au montage → rouge.
		expect(appels(m.getInvoiceSettings)).toBe(0);
	});

	it('par une adresse : une lecture, la ligne des deux comptes ; Retour et retour sur l’onglet : aucune de plus', async () => {
		const r = await monter('/reports?tab=vat&fy=2&from=2026-01-01&to=2026-12-31');
		const ligne = await r.findByTestId('vat-accounts');
		expect([...ligne.querySelectorAll('a')].map((a) => a.textContent)).toEqual(['2200', '1170']);
		expect(appels(m.getInvoiceSettings)).toBe(1);
		await fireEvent.click(onglet(r, 'Bilan'));
		await pause();
		await fireEvent.click(onglet(r, 'TVA'));
		await pause();
		expect(appels(m.getInvoiceSettings)).toBe(1);
	});

	it('par un clic sur l’onglet : une lecture', async () => {
		const r = await monter('/reports');
		await fireEvent.click(onglet(r, 'TVA'));
		await r.findByTestId('vat-accounts');
		expect(appels(m.getInvoiceSettings)).toBe(1);
	});

	it('réglages en échec : silence, la ligne est omise, le décompte s’affiche', async () => {
		m.getInvoiceSettings.mockRejectedValue(new Error('403'));
		const r = await monter('/reports?tab=vat&fy=2&from=2026-01-01&to=2026-12-31');
		await r.findByTestId('vat-total-vat-due');
		await pause();
		expect(r.queryByTestId('vat-accounts')).toBeNull();
		expect(r.queryByRole('alert')).toBeNull();
	});

	it('compte désigné absent de la liste des comptes : sa paire est omise', async () => {
		m.getInvoiceSettings.mockResolvedValue({ defaultVatPayableAccountId: 999, defaultVatRecoverableAccountId: 11 });
		const r = await monter('/reports?tab=vat&fy=2&from=2026-01-01&to=2026-12-31');
		const ligne = await r.findByTestId('vat-accounts');
		expect([...ligne.querySelectorAll('a')].map((a) => a.textContent)).toEqual(['1170']);
	});
});
