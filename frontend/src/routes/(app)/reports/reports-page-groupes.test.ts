// Story 46-2 — la page des rapports et la vue groupée du bilan (AC-1, AC-3, AC-5,
// AC-6, AC-10) : vue restaurée avant la première requête, redemande à chaque
// changement sans effacer l'affichage, seule la dernière réponse retenue, écho
// mémorisé, export aux paramètres affichés, hiérarchie masquée sans paramètre.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import type { BalanceSheetDto, GroupingParams, TrialBalanceDto } from '$lib/features/reports/reports.types';

const {
	getBalanceSheetMock,
	getIncomeStatementMock,
	getTrialBalanceMock,
	downloadReportMock,
	getAgedReceivablesMock,
	getGeneralLedgerMock,
	getProjectExpensesMock,
	listProjectsMock,
} = vi.hoisted(() => ({
		getGeneralLedgerMock: vi.fn(),
		getProjectExpensesMock: vi.fn(),
		listProjectsMock: vi.fn(async () => [] as unknown[]),
		getBalanceSheetMock: vi.fn(),
		getIncomeStatementMock: vi.fn(),
		getTrialBalanceMock: vi.fn(),
		downloadReportMock: vi.fn(async () => {}),
		getAgedReceivablesMock: vi.fn(async () => ({ asOf: '2026-12-31', rows: [], totals: {} })),
	}));

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/reports') } }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: listProjectsMock }));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: async () => [] }));
vi.mock('$lib/features/reports/reports.api', async (importOriginal) => {
	const vrai = await importOriginal<typeof import('$lib/features/reports/reports.api')>();
	return {
		...vrai,
		downloadReport: downloadReportMock,
		getAgedReceivables: getAgedReceivablesMock,
		getBalanceSheet: getBalanceSheetMock,
		getGeneralLedger: getGeneralLedgerMock,
		getProjectExpenses: getProjectExpensesMock,
		getIncomeStatement: getIncomeStatementMock,
		getTrialBalance: getTrialBalanceMock,
	};
});

import Page from './+page.svelte';

const data = {
	fiscalYears: [
		{
			id: 1,
			companyId: 7,
			name: '2026',
			startDate: '2026-01-01',
			endDate: '2026-12-31',
			status: 'Open',
			multiCurrency: false,
		},
	],
	companyName: 'Test SA',
	companyId: 7,
} as never;

const KEY = 'kesh-report-grouping:7:balance-sheet';

/** Bilan groupé : groupe 10 (id 90) ouvert ou fermé selon `open`, écho de `params`. */
function bilan(params: GroupingParams | undefined, open = true, echo?: Partial<GroupingParams>): BalanceSheetDto {
	const base: BalanceSheetDto = {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		assets: [
			{ accountId: 1, accountNumber: '1000', accountName: 'Caisse', accountType: 'Asset', active: true, balance: '100', role: null },
		],
		liabilities: [],
		equity: [],
		totalAssets: '100',
		totalLiabilities: '0',
		totalEquity: '0',
		retainedEarnings: '0',
		equityResult: '100',
		equationHolds: true,
	};
	if (!params) return base;
	return {
		...base,
		grouping: { ...params, maxDepth: 2, ...echo },
		groupedRows: {
			assets: [
				{ kind: 'group', depth: 1, accountId: 90, accountNumber: '10', accountName: 'Actifs circulants', balance: '100', open },
				...(open
					? [{ kind: 'account' as const, depth: 2, accountId: 1, accountNumber: '1000', accountName: 'Caisse', balance: '100', active: true }]
					: []),
			],
			liabilities: [],
			equity: [],
		},
	};
}

async function generer(r: { getByRole: (role: 'button', opts: { name: string }) => HTMLElement }) {
	await fireEvent.click(r.getByRole('button', { name: 'Générer' }));
}

describe('page des rapports — vue groupée du bilan (Story 46-2)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => bilan(p, true));
	});
	afterEach(() => vi.restoreAllMocks());

	it('première visite : bilan groupé, tous les comptes, sous-totaux sur le titre (AC-1)', async () => {
		const r = render(Page, { data });
		await generer(r);
		expect(getBalanceSheetMock).toHaveBeenCalledWith(expect.objectContaining({ fiscalYearId: 1 }), {
			detailLevel: 'all',
			subtotalPosition: 'header',
			open: [],
			closed: [],
		});
		await r.findByTestId('grouped-group-10');
		expect((r.getByTestId('grouping-shown') as HTMLInputElement).checked).toBe(true);
	});

	it('vue mémorisée relue AVANT la première requête (AC-10)', async () => {
		localStorage.setItem(
			KEY,
			JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'footer', open: [90, 555], closed: [] }),
		);
		const r = render(Page, { data });
		await generer(r);
		expect(getBalanceSheetMock.mock.calls[0][1]).toEqual({
			detailLevel: 2,
			subtotalPosition: 'footer',
			open: [90, 555],
			closed: [],
		});
	});

	it("groupe disparu : l'écho le retire de la mémoire après la réponse (AC-10)", async () => {
		localStorage.setItem(KEY, JSON.stringify({ shown: true, detailLevel: 1, subtotalPosition: 'header', open: [90, 555], closed: [] }));
		getBalanceSheetMock.mockImplementation(async (_q, p) => bilan(p, true, { open: [90] }));
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		expect(JSON.parse(localStorage.getItem(KEY)!).open).toEqual([90]);
	});

	it('mémoire illisible ou refusée : vue par défaut, sans erreur (AC-5)', async () => {
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new DOMException('refusé', 'SecurityError');
		});
		const r = render(Page, { data });
		await generer(r);
		expect(getBalanceSheetMock.mock.calls[0][1]).toMatchObject({ detailLevel: 'all' });
		await r.findByTestId('grouped-group-10');
		// la redemande et l'affichage marchent quand même
		await fireEvent.click(r.getByTestId('group-toggle-10'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(r.queryByRole('alert')).toBeNull();
	});

	it('clic sur un accordéon : mémorisé, redemandé avec closed, affichage gardé (AC-3)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		let resoudre!: (v: BalanceSheetDto) => void;
		getBalanceSheetMock.mockImplementationOnce(() => new Promise((res) => (resoudre = res)));
		await fireEvent.click(r.getByTestId('group-toggle-10'));
		expect(getBalanceSheetMock.mock.calls[1][1]).toMatchObject({ open: [], closed: [90] });
		expect(JSON.parse(localStorage.getItem(KEY)!).closed).toEqual([90]);
		// pendant la redemande : l'affichage précédent reste, marqué occupé
		expect(r.queryByTestId('grouped-account-1000')).not.toBeNull();
		expect(r.queryByText('Génération du rapport en cours…')).toBeNull();
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('true');
		resoudre(bilan(getBalanceSheetMock.mock.calls[1][1], false, { closed: [90] }));
		await waitFor(() => expect(r.queryByTestId('grouped-account-1000')).toBeNull());
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false');
	});

	it('requêtes concurrentes : seule la dernière réponse s’affiche (AC-3)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const attentes: ((v: BalanceSheetDto) => void)[] = [];
		getBalanceSheetMock.mockImplementation(() => new Promise((res) => attentes.push(res)));
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await fireEvent.change(r.getByTestId('grouping-position'), { target: { value: 'footer' } });
		expect(attentes).toHaveLength(2);
		// la seconde arrive d'abord, puis la première, périmée
		attentes[1](bilan({ detailLevel: 1, subtotalPosition: 'footer', open: [], closed: [] }, false));
		await waitFor(() => expect(r.queryByTestId('grouped-account-1000')).toBeNull());
		attentes[0](bilan({ detailLevel: 1, subtotalPosition: 'header', open: [], closed: [] }, true));
		await new Promise((res) => setTimeout(res, 0));
		expect(r.queryByTestId('grouped-account-1000')).toBeNull();
	});

	it('changement de niveau : listes vidées, mémorisé ; tout ouvrir et tout fermer', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(getBalanceSheetMock.mock.calls[1][1]).toMatchObject({ detailLevel: 1, open: [], closed: [] });
		expect(JSON.parse(localStorage.getItem(KEY)!).detailLevel).toBe(1);
		await fireEvent.click(r.getByTestId('grouping-open-all'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(3));
		expect(getBalanceSheetMock.mock.calls[2][1]).toMatchObject({ detailLevel: 'all' });
		await fireEvent.click(r.getByTestId('grouping-close-all'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(4));
		expect(getBalanceSheetMock.mock.calls[3][1]).toMatchObject({ detailLevel: 1 });
	});

	it('position des sous-totaux mémorisée et transmise', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.change(r.getByTestId('grouping-position'), { target: { value: 'footer' } });
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(getBalanceSheetMock.mock.calls[1][1]).toMatchObject({ subtotalPosition: 'footer' });
		expect(JSON.parse(localStorage.getItem(KEY)!).subtotalPosition).toBe('footer');
	});

	it('hiérarchie masquée : requête sans paramètre, écran à plat, choix gardés (C-46-2-2)', async () => {
		localStorage.setItem(KEY, JSON.stringify({ shown: true, detailLevel: 2, subtotalPosition: 'footer', open: [], closed: [] }));
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.click(r.getByTestId('grouping-shown'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(getBalanceSheetMock.mock.calls[1][1]).toBeUndefined();
		await waitFor(() => expect(r.queryByTestId('grouped-group-10')).toBeNull());
		expect(r.getByText('Caisse')).toBeTruthy();
		expect((r.getByTestId('grouping-level') as HTMLSelectElement).disabled).toBe(true);
		expect(JSON.parse(localStorage.getItem(KEY)!)).toMatchObject({ shown: false, detailLevel: 2, subtotalPosition: 'footer' });
	});

	it('export : les paramètres de la dernière réponse affichée (AC-6)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.click(r.getByTestId('group-toggle-10'));
		await waitFor(() => expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false'));
		await fireEvent.click(r.getByRole('button', { name: 'Export CSV' }));
		await waitFor(() => expect(downloadReportMock).toHaveBeenCalledTimes(1));
		const args = downloadReportMock.mock.calls[0] as unknown[];
		expect(args[0]).toBe('balance-sheet');
		expect(args[2]).toBe('csv');
		expect(args[4]).toEqual({ detailLevel: 'all', subtotalPosition: 'header', open: [], closed: [90] });
	});

	it('sélecteur de niveau : 1 à maxDepth − 1, puis Tous les comptes', async () => {
		const r = render(Page, { data });
		expect((r.getByTestId('grouping-level') as HTMLSelectElement).disabled).toBe(true);
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const select = r.getByTestId('grouping-level') as HTMLSelectElement;
		expect(select.disabled).toBe(false);
		expect([...select.options].map((o) => o.textContent)).toEqual(['Niveau 1', 'Tous les comptes']);
	});
});

/** Ce que les scénarios demandent au rendu : onglets et contrôles par rôle ou identifiant. */
type Ecran = {
	getByRole: (role: string, opts: { name: string }) => HTMLElement;
	getByTestId: (id: string) => HTMLElement;
};

/** Compte de résultat vide, sans groupe : seule la génération compte ici. */
const resultatVide = async () => ({
	period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
	revenues: [],
	expenses: [],
	totalRevenues: '0',
	totalExpenses: '0',
	netResult: '0',
});

/** Prochain appel du bilan suspendu ; rend de quoi le résoudre avec ses paramètres. */
function bilanSuspendu(): { resoudre: (open?: boolean) => void; params: () => GroupingParams | undefined } {
	let res!: (v: BalanceSheetDto) => void;
	let p: GroupingParams | undefined;
	getBalanceSheetMock.mockImplementationOnce((_q, params?: GroupingParams) => {
		p = params;
		return new Promise((r) => (res = r));
	});
	return { resoudre: (open = true) => res(bilan(p, open)), params: () => p };
}

const pause = () => new Promise((res) => setTimeout(res, 10));

describe('page des rapports — demandes périmées et vue affichée (Story 46-2, revue P1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => bilan(p, true));
		getIncomeStatementMock.mockImplementation(resultatVide);
	});
	afterEach(() => vi.restoreAllMocks());

	it('S1 — hiérarchie masquée pendant le premier chargement : redemandée, écran et case concordent (M2 a)', async () => {
		const premier = bilanSuspendu();
		const r = render(Page, { data });
		await generer(r);
		await fireEvent.click(r.getByTestId('grouping-shown'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(getBalanceSheetMock.mock.calls[1][1]).toBeUndefined();
		premier.resoudre(); // la première réponse, partie groupée, est périmée
		await r.findByText('Caisse');
		await pause();
		expect((r.getByTestId('grouping-shown') as HTMLInputElement).checked).toBe(false);
		expect(r.queryByTestId('grouped-group-10')).toBeNull();
		expect(JSON.parse(localStorage.getItem(KEY)!).shown).toBe(false);
	});

	it('S2 — redemande jetée par « Générer » sur un autre onglet : redemandée au retour (M2 b)', async () => {
		// au niveau 1, le groupe 10 est fermé par l'API
		getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => bilan(p, p?.detailLevel !== 1));
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const jetee = bilanSuspendu();
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await fireEvent.click(r.getByRole('tab', { name: 'Compte de résultat' }));
		await generer(r);
		await waitFor(() => expect(getIncomeStatementMock).toHaveBeenCalled());
		jetee.resoudre(false);
		await pause();
		expect(getBalanceSheetMock).toHaveBeenCalledTimes(2);
		await fireEvent.click(r.getByRole('tab', { name: 'Bilan' }));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(3));
		expect(getBalanceSheetMock.mock.calls[2][1]).toMatchObject({ detailLevel: 1 });
		await waitFor(() => expect(r.queryByTestId('grouped-account-1000')).toBeNull());
		expect((r.getByTestId('grouping-level') as HTMLSelectElement).value).toBe('1');
	});

	it('retour sur un onglet déjà d’accord avec ses contrôles : rien n’est redemandé', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.click(r.getByRole('tab', { name: 'Compte de résultat' }));
		await fireEvent.click(r.getByRole('tab', { name: 'Bilan' }));
		await pause();
		expect(getBalanceSheetMock).toHaveBeenCalledTimes(1);
	});

	it('S3 — redemande en erreur : l’export reprend les paramètres affichés, non la vue demandée (AC-6)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		getBalanceSheetMock.mockImplementationOnce(async () => {
			throw new Error('réseau');
		});
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await waitFor(() => expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false'));
		await fireEvent.click(r.getByRole('button', { name: 'Export CSV' }));
		await waitFor(() => expect(downloadReportMock).toHaveBeenCalledTimes(1));
		expect((downloadReportMock.mock.calls[0] as unknown[])[4]).toEqual({
			detailLevel: 'all',
			subtotalPosition: 'header',
			open: [],
			closed: [],
		});
	});

	it('export désactivé pendant une redemande, rendu à sa réponse (AC-3, AC-6)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const exporter = () => r.getByRole('button', { name: 'Export CSV' }) as HTMLButtonElement;
		expect(exporter().disabled).toBe(false);
		const enVol = bilanSuspendu();
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		expect(exporter().disabled).toBe(true);
		enVol.resoudre(false);
		await waitFor(() => expect(exporter().disabled).toBe(false));
	});

	it('S4 — redemande du bilan périmée par la balance âgée : drapeaux rabaissés (M1)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const jetee = bilanSuspendu();
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await fireEvent.click(r.getByRole('tab', { name: 'Balance âgée' }));
		await fireEvent.click(r.getByTestId('aged-report-generate'));
		await waitFor(() => expect(getAgedReceivablesMock).toHaveBeenCalled());
		jetee.resoudre(false);
		await pause();
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false');
		await fireEvent.click(r.getByRole('tab', { name: 'Bilan' }));
		// le retour redemande la vue jetée (M2 b), puis l'export revient
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(3));
		await waitFor(() =>
			expect((r.getByRole('button', { name: 'Export CSV' }) as HTMLButtonElement).disabled).toBe(false),
		);
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false');
	});

	it('changement d’exercice pendant une redemande : drapeaux rabaissés (M1)', async () => {
		const deux = {
			...(data as { fiscalYears: unknown[] }),
			fiscalYears: [
				...(data as { fiscalYears: unknown[] }).fiscalYears,
				{ id: 2, companyId: 7, name: '2027', startDate: '2027-01-01', endDate: '2027-12-31', status: 'Open', multiCurrency: false },
			],
		} as never;
		const r = render(Page, { data: deux });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		bilanSuspendu();
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('true');
		await fireEvent.change(r.getByRole('combobox', { name: 'Exercice' }), { target: { value: '2' } });
		await waitFor(() => expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false'));
		expect((r.getByRole('button', { name: 'Générer' }) as HTMLButtonElement).disabled).toBe(false);
	});

	it('X1 — retour sur le bilan désaccordé PENDANT le chargement d’un autre onglet : redemandé à sa fin (revue P2 M1)', async () => {
		getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => bilan(p, p?.detailLevel !== 1));
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		const jetee = bilanSuspendu();
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await fireEvent.click(r.getByRole('tab', { name: 'Compte de résultat' }));
		let resoudreResultat!: () => void;
		getIncomeStatementMock.mockImplementationOnce(
			() => new Promise((res) => (resoudreResultat = () => res(resultatVide()))),
		);
		await generer(r);
		await waitFor(() => expect(getIncomeStatementMock).toHaveBeenCalledTimes(1));
		jetee.resoudre(false); // périmée par « Générer » sur le compte de résultat
		await fireEvent.click(r.getByRole('tab', { name: 'Bilan' }));
		await pause();
		// le chargement du compte de résultat n'est pas périmé par le retour
		expect(getBalanceSheetMock).toHaveBeenCalledTimes(2);
		resoudreResultat();
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(3));
		expect(getBalanceSheetMock.mock.calls[2][1]).toMatchObject({ detailLevel: 1 });
		await waitFor(() => expect(r.queryByTestId('grouped-account-1000')).toBeNull());
		expect((r.getByTestId('grouping-level') as HTMLSelectElement).value).toBe('1');
		await pause();
		expect(getBalanceSheetMock).toHaveBeenCalledTimes(3);
	});

	// Revue P3 L1 : le `finally` de chaque branche appelle `afterGeneration` ; X1 ne garde que celle des rapports
	// classiques. Même scénario avec la balance âgée, le grand livre et un rapport par projet comme « autre chargement ».
	describe.each([
		{
			nom: 'la balance âgée',
			ouvrir: async (r: Ecran) => {
				await fireEvent.click(r.getByRole('tab', { name: 'Balance âgée' }));
				return () => r.getByTestId('aged-report-generate');
			},
			mock: getAgedReceivablesMock,
		},
		{
			nom: 'le grand livre',
			ouvrir: async (r: Ecran) => {
				await fireEvent.click(r.getByRole('tab', { name: 'Grand livre' }));
				await fireEvent.input(r.getByTestId('ledger-from'), { target: { value: '2026-01-01' } });
				await fireEvent.input(r.getByTestId('ledger-to'), { target: { value: '2026-12-31' } });
				return () => r.getByTestId('ledger-generate');
			},
			mock: getGeneralLedgerMock,
		},
		{
			nom: 'un rapport par projet',
			ouvrir: async (r: Ecran) => {
				await fireEvent.click(r.getByRole('tab', { name: 'Dépenses par compte analytique' }));
				await fireEvent.change(r.getByTestId('project-report-project'), { target: { value: '5' } });
				return () => r.getByTestId('project-report-generate');
			},
			mock: getProjectExpensesMock,
		},
	])('retour sur le bilan désaccordé pendant $nom (revue P3 L1)', ({ ouvrir, mock }) => {
		it('redemandé à la fin de ce chargement', async () => {
			listProjectsMock.mockResolvedValue([{ id: 5, code: 'P5', name: 'Projet', parentId: null }]);
			getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => bilan(p, p?.detailLevel !== 1));
			const r = render(Page, { data });
			await generer(r);
			await r.findByTestId('grouped-group-10');
			const jetee = bilanSuspendu();
			await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
			const generer2 = await ouvrir(r);
			let finir!: () => void;
			mock.mockImplementationOnce(() => new Promise((_res, rej) => (finir = () => rej(new Error('autre')))));
			await fireEvent.click(generer2());
			await waitFor(() => expect(mock).toHaveBeenCalledTimes(1));
			jetee.resoudre(false);
			await fireEvent.click(r.getByRole('tab', { name: 'Bilan' }));
			await pause();
			expect(getBalanceSheetMock).toHaveBeenCalledTimes(2);
			finir();
			await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(3));
			expect(getBalanceSheetMock.mock.calls[2][1]).toMatchObject({ detailLevel: 1 });
			await waitFor(() => expect(r.queryByTestId('grouped-account-1000')).toBeNull());
		});
	});

	it('redemande en erreur : pas de redemande en boucle (revue P2 M1)', async () => {
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		// en erreur cinq fois, puis de nouveau servi : une boucle se verrait au compte
		let echecs = 0;
		getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => {
			if (++echecs <= 5) throw new Error('réseau');
			return bilan(p, true);
		});
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await waitFor(() => expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false'));
		await pause();
		expect(getBalanceSheetMock).toHaveBeenCalledTimes(2);
		expect(r.getByRole('alert')).toBeTruthy();
	});

	it('chargement périmé par un autre onglet : un changement de vue sans rapport affiché ne demande rien (revue P2 L1)', async () => {
		bilanSuspendu(); // premier chargement du bilan, jamais affiché
		const r = render(Page, { data });
		await generer(r);
		await fireEvent.click(r.getByRole('tab', { name: 'Balance âgée' }));
		getAgedReceivablesMock.mockImplementationOnce(() => new Promise(() => {}));
		await fireEvent.click(r.getByTestId('aged-report-generate'));
		await waitFor(() => expect(getAgedReceivablesMock).toHaveBeenCalledTimes(1));
		await fireEvent.click(r.getByRole('tab', { name: 'Bilan' }));
		await fireEvent.click(r.getByTestId('grouping-shown'));
		await pause();
		expect(getBalanceSheetMock).toHaveBeenCalledTimes(1);
	});

	it('export après un écho qui retire un groupe disparu : les listes de l’écho (revue P2 L1)', async () => {
		localStorage.setItem(KEY, JSON.stringify({ shown: true, detailLevel: 'all', subtotalPosition: 'header', open: [90, 555], closed: [] }));
		getBalanceSheetMock.mockImplementation(async (_q, p) => bilan(p, true, { open: [90] }));
		const r = render(Page, { data });
		await generer(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.click(r.getByRole('button', { name: 'Export CSV' }));
		await waitFor(() => expect(downloadReportMock).toHaveBeenCalledTimes(1));
		expect((downloadReportMock.mock.calls[0] as unknown[])[4]).toEqual({
			detailLevel: 'all',
			subtotalPosition: 'header',
			open: [90],
			closed: [],
		});
	});

	it('changement de vue pendant le premier chargement : chargement plein, non redemande (revue P2 L1)', async () => {
		bilanSuspendu();
		const r = render(Page, { data });
		await generer(r);
		expect(r.queryByText('Génération du rapport en cours…')).not.toBeNull();
		bilanSuspendu();
		await fireEvent.click(r.getByTestId('grouping-shown'));
		await waitFor(() => expect(getBalanceSheetMock).toHaveBeenCalledTimes(2));
		expect(r.queryByText('Génération du rapport en cours…')).not.toBeNull();
		expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('true');
	});
});

/** Balance groupée (Story 46-6b) : groupe 10 (id 90) ouvert ou fermé, écho de `params`. */
function balance(params: GroupingParams | undefined, open = true): TrialBalanceDto {
	const caisse = {
		accountId: 1,
		accountNumber: '1000',
		accountName: 'Caisse',
		openingBalance: '40',
		totalDebit: '100',
		totalCredit: '0',
		balance: '100',
		closingBalance: '140',
	};
	const base: TrialBalanceDto = {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		rows: [{ ...caisse, accountType: 'Asset', active: true }],
		totalDebit: '100',
		totalCredit: '100',
		balanced: true,
		retainedEarnings: '40',
		openingBalanced: true,
	};
	if (!params) return base;
	return {
		...base,
		grouping: { ...params, maxDepth: 2 },
		groupedRows: [
			{ ...caisse, kind: 'group', depth: 1, accountId: 90, accountNumber: '10', accountName: 'Actifs circulants', open },
			...(open ? [{ ...caisse, kind: 'account' as const, depth: 2, active: true }] : []),
		],
	};
}

describe('page des rapports — vue groupée de la balance (Story 46-6b)', () => {
	const TB_KEY = 'kesh-report-grouping:7:trial-balance';

	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		getBalanceSheetMock.mockImplementation(async (_q, p?: GroupingParams) => bilan(p, true));
		getTrialBalanceMock.mockImplementation(async (_q, p?: GroupingParams) => balance(p, true));
	});

	async function ouvrirBalance(r: {
		getByRole: (role: 'tab' | 'button', opts: { name: string }) => HTMLElement;
	}): Promise<void> {
		await fireEvent.click(r.getByRole('tab', { name: 'Balance' }));
		await generer(r);
	}

	it('première visite : balance groupée, tous les comptes, sous-totaux sur le titre (AC-1)', async () => {
		const r = render(Page, { data });
		await ouvrirBalance(r);
		expect(getTrialBalanceMock).toHaveBeenCalledWith(expect.objectContaining({ fiscalYearId: 1 }), {
			detailLevel: 'all',
			subtotalPosition: 'header',
			open: [],
			closed: [],
		});
		await r.findByTestId('grouped-group-10');
		expect(r.getByTestId('grouping-controls')).toBeTruthy();
		// L'info-bulle du niveau est celle de la balance, non celle des sections du bilan.
		expect(r.container.querySelector('#reports-groups-level-hint')?.textContent).toContain(
			'Le total des débits et celui des crédits',
		);
	});

	it('mémoire propre à la balance : le réglage du bilan ne la touche pas (AC-1)', async () => {
		localStorage.setItem(
			KEY,
			JSON.stringify({ shown: true, detailLevel: 1, subtotalPosition: 'footer', open: [], closed: [] }),
		);
		const r = render(Page, { data });
		await ouvrirBalance(r);
		expect(getTrialBalanceMock.mock.calls[0][1]).toMatchObject({ detailLevel: 'all', subtotalPosition: 'header' });
		await r.findByTestId('grouped-group-10');
		await fireEvent.click(r.getByTestId('group-toggle-10'));
		await waitFor(() => expect(getTrialBalanceMock).toHaveBeenCalledTimes(2));
		expect(getTrialBalanceMock.mock.calls[1][1]).toMatchObject({ closed: [90] });
		expect(JSON.parse(localStorage.getItem(TB_KEY)!).closed).toEqual([90]);
		expect(JSON.parse(localStorage.getItem(KEY)!).detailLevel).toBe(1);
	});

	it('export : les paramètres de la dernière réponse affichée de la balance (AC-2)', async () => {
		const r = render(Page, { data });
		await ouvrirBalance(r);
		await r.findByTestId('grouped-group-10');
		await fireEvent.change(r.getByTestId('grouping-level'), { target: { value: '1' } });
		await waitFor(() => expect(r.getByRole('tabpanel').getAttribute('aria-busy')).toBe('false'));
		await fireEvent.click(r.getByRole('button', { name: 'Export CSV' }));
		await waitFor(() => expect(downloadReportMock).toHaveBeenCalledTimes(1));
		const args = downloadReportMock.mock.calls[0] as unknown[];
		expect(args[0]).toBe('trial-balance');
		expect(args[4]).toEqual({ detailLevel: 1, subtotalPosition: 'header', open: [], closed: [] });
	});

	// L'export reçoit `undefined` pour le groupement ; c'est `getReportExportUrl` qui en
	// fait `detailLevel=flat` (reports.api.groupes.test.ts), l'export de l'API étant
	// groupé sans paramètre.
	it('hiérarchie masquée : requête sans paramètre, export sans groupement, écran à plat', async () => {
		localStorage.setItem(
			TB_KEY,
			JSON.stringify({ shown: false, detailLevel: 'all', subtotalPosition: 'header', open: [], closed: [] }),
		);
		const r = render(Page, { data });
		await ouvrirBalance(r);
		await r.findByTestId('tb-row');
		expect(getTrialBalanceMock.mock.calls[0][1]).toBeUndefined();
		expect(r.queryByTestId('grouped-group-10')).toBeNull();
		await fireEvent.click(r.getByRole('button', { name: 'Export PDF' }));
		await waitFor(() => expect(downloadReportMock).toHaveBeenCalledTimes(1));
		expect((downloadReportMock.mock.calls[0] as unknown[])[4]).toBeUndefined();
	});
});
