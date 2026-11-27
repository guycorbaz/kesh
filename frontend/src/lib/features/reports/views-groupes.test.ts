// Story 46-2 (AC-4) — le bilan et le compte de résultat en vue groupée : les
// sections rendent `groupedRows`, les totaux de section, les lignes calculées du
// bilan et l'équation restent à leur place ; sans `groupedRows`, la vue à plat.

import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));

import BalanceSheetView from './BalanceSheetView.svelte';
import IncomeStatementView from './IncomeStatementView.svelte';
import GroupingControls from './GroupingControls.svelte';
import { defaultGroupingView } from './reports-grouping';
import type { AccountBalance, BalanceSheetDto, GroupedRow, IncomeStatementDto } from './reports.types';

const period = { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' };

const compte = (id: number, number: string, name: string, balance: string, extra: Partial<AccountBalance> = {}): AccountBalance => ({
	accountId: id,
	accountNumber: number,
	accountName: name,
	accountType: 'Asset',
	active: true,
	balance,
	role: null,
	...extra,
});
const groupe = (id: number, number: string, name: string, balance: string, depth: number, open: boolean): GroupedRow => ({
	kind: 'group', depth, accountId: id, accountNumber: number, accountName: name, balance, open,
});
const ligne = (a: AccountBalance, depth: number): GroupedRow => ({
	kind: 'account', depth, accountId: a.accountId, accountNumber: a.accountNumber, accountName: a.accountName, balance: a.balance, active: a.active, role: a.role,
});

const caisse = compte(1, '1000', 'Caisse', '200');
const banque = compte(2, '1020', 'Banque', '800');
const fournisseurs = compte(3, '2000', 'Fournisseurs', '300', { accountType: 'Liability', role: 'Payable' });
const capital = compte(4, '2800', 'Capital', '500', { accountType: 'Liability', role: 'EquityCapital' });

function bilan(grouped: boolean, position: 'header' | 'footer' = 'header'): BalanceSheetDto {
	const dto: BalanceSheetDto = {
		period,
		assets: [caisse, banque],
		liabilities: [fournisseurs],
		equity: [capital],
		totalAssets: '1000',
		totalLiabilities: '300',
		totalEquity: '500',
		retainedEarnings: '50',
		equityResult: '150',
		equationHolds: true,
	};
	if (!grouped) return dto;
	return {
		...dto,
		grouping: { detailLevel: 'all', subtotalPosition: position, maxDepth: 2, open: [], closed: [] },
		groupedRows: {
			assets: [groupe(90, '10', 'Actifs circulants', '1000', 1, true), ligne(caisse, 2), ligne(banque, 2),
				...(position === 'footer' ? [{ ...groupe(90, '10', 'Actifs circulants', '1000', 1, true), kind: 'subtotal' as const, open: undefined }] : [])],
			liabilities: [groupe(91, '20', 'Dettes à court terme', '300', 1, false)],
			equity: [ligne(capital, 1)],
		},
	};
}

describe('BalanceSheetView — vue groupée', () => {
	it('les trois sections rendent leurs lignes groupées ; groupe fermé sans descendance', () => {
		const { getByTestId, queryByTestId } = render(BalanceSheetView, { dto: bilan(true) });
		expect(getByTestId('grouped-group-10').textContent).toContain('Actifs circulants');
		expect(getByTestId('grouped-account-1000')).toBeTruthy();
		expect(getByTestId('grouped-group-20').textContent).toContain('300.00');
		expect(queryByTestId('grouped-account-2000')).toBeNull();
		expect(getByTestId('grouped-account-2800')).toBeTruthy();
	});

	it('totaux, lignes calculées et équation à leur place', () => {
		const { getByText, getByTestId, queryByRole } = render(BalanceSheetView, { dto: bilan(true) });
		expect(getByText('Total actifs').closest('tr')?.textContent).toContain('1’000.00');
		expect(getByText('Total passifs').closest('tr')?.textContent).toContain('300.00');
		const equity = getByTestId('balance-sheet-equity');
		expect(getByTestId('balance-sheet-retained-earnings').textContent).toContain('50.00');
		expect(equity.textContent).toContain("Bénéfice de l'exercice");
		// lignes calculées après les comptes de fonds propres
		const rows = [...equity.querySelectorAll('tbody tr')].map((tr) => tr.getAttribute('data-testid'));
		expect(rows.indexOf('grouped-account-2800')).toBeLessThan(rows.indexOf('balance-sheet-retained-earnings'));
		expect(getByText('Total capitaux propres').closest('tr')?.textContent).toContain('700.00');
		expect(queryByRole('alert')).toBeNull();
	});

	it('vue groupée : pas de sous-titre de rôle (D4, vue à plat seulement)', () => {
		const groupee = render(BalanceSheetView, { dto: bilan(true) });
		expect(groupee.queryByText('EquityCapital')).toBeNull();
		groupee.unmount();
		const plate = render(BalanceSheetView, { dto: bilan(false) });
		expect(plate.getByText('EquityCapital')).toBeTruthy();
		expect(plate.queryByTestId('grouped-account-1000')).toBeNull();
	});

	it('footer : ligne « Total <groupe> » sous ses comptes', () => {
		const { getByTestId } = render(BalanceSheetView, { dto: bilan(true, 'footer') });
		expect(getByTestId('grouped-subtotal-10').textContent).toContain('Total Actifs circulants');
	});

	it('clic sur un accordéon transmis à la page', async () => {
		const onToggleGroup = vi.fn();
		const { getByTestId } = render(BalanceSheetView, { dto: bilan(true), onToggleGroup });
		await fireEvent.click(getByTestId('group-toggle-20'));
		expect(onToggleGroup).toHaveBeenCalledWith(expect.objectContaining({ accountId: 91, open: false }));
	});
});

describe('IncomeStatementView — vue groupée', () => {
	const vente = compte(5, '3200', 'Ventes', '900', { accountType: 'Revenue' });
	const loyer = compte(6, '6000', 'Loyer', '400', { accountType: 'Expense' });
	const dto = (grouped: boolean): IncomeStatementDto => {
		const base: IncomeStatementDto = {
			period, revenues: [vente], expenses: [loyer], totalRevenues: '900', totalExpenses: '400', netResult: '500',
		};
		return grouped
			? {
					...base,
					grouping: { detailLevel: 1, subtotalPosition: 'header', maxDepth: 2, open: [], closed: [] },
					groupedRows: {
						revenues: [groupe(70, '3', "Chiffre d'affaires", '900', 1, false)],
						expenses: [groupe(71, '6', 'Autres charges', '400', 1, true), ligne(loyer, 2)],
					},
				}
			: base;
	};

	it('Produits et Charges groupés, résultat net inchangé', () => {
		const { getByTestId, queryByTestId, getByText } = render(IncomeStatementView, { dto: dto(true) });
		expect(getByTestId('grouped-group-3').textContent).toContain('900.00');
		expect(queryByTestId('grouped-account-3200')).toBeNull();
		expect(getByTestId('grouped-account-6000').querySelector('a')).toBeTruthy();
		expect(getByText('Total produits').closest('tr')?.textContent).toContain('900.00');
		expect(getByText(/Résultat net/).parentElement?.textContent).toContain('500.00');
	});

	it('sans groupedRows : la vue à plat d’avant', () => {
		const { queryByTestId, getByText } = render(IncomeStatementView, { dto: dto(false) });
		expect(queryByTestId('grouped-group-3')).toBeNull();
		expect(getByText('Ventes')).toBeTruthy();
	});
});

describe('GroupingControls — accessibilité (revue P1 L2)', () => {
	it('le sélecteur de niveau a un nom court ; l’info-bulle en est la description seule', () => {
		const r = render(GroupingControls, { view: defaultGroupingView(), maxDepth: 3, onChange: () => {} });
		const select = r.getByRole('combobox', { name: 'Niveau de détail' });
		expect(select).toBe(r.getByTestId('grouping-level'));
		const hint = document.getElementById(select.getAttribute('aria-describedby')!);
		expect(hint?.textContent).toMatch(/Les sections légales du bilan restent/);
		expect(hint?.closest('label')).toBeNull();
	});
});
