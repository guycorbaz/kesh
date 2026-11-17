// Story 37-4 — tests Vitest pour IncomeStatementView.svelte (absents jusque-là) :
// produits, charges, résultat net, et le solde en devise d'un compte en devise ;
// mono-devise, rien de visible ne change (AC-6).

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import IncomeStatementView from './IncomeStatementView.svelte';
import type { AccountBalance, IncomeStatementDto } from './reports.types';

function compte(over: Partial<AccountBalance> = {}): AccountBalance {
	return {
		accountId: 1,
		accountNumber: '3000',
		accountName: 'Ventes',
		accountType: 'Revenue',
		active: true,
		balance: '1000.00',
		role: null,
		...over,
	};
}

function makeDto(over: Partial<IncomeStatementDto> = {}): IncomeStatementDto {
	return {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		revenues: [compte()],
		expenses: [
			compte({
				accountId: 2,
				accountNumber: '4000',
				accountName: 'Charges',
				accountType: 'Expense',
				balance: '400.00',
			}),
		],
		totalRevenues: '1000.00',
		totalExpenses: '400.00',
		netResult: '600.00',
		...over,
	};
}

describe('IncomeStatementView', () => {
	it('affiche produits, charges, totaux et résultat net', () => {
		const { container } = render(IncomeStatementView, { dto: makeDto() });
		const text = container.textContent ?? '';
		expect(text).toContain('Ventes');
		expect(text).toContain('Charges');
		expect(text).toContain('1’000.00');
		expect(text).toContain('600.00');
	});

	it('affiche le message vide sans compte', () => {
		const { getByRole } = render(IncomeStatementView, {
			dto: makeDto({ revenues: [], expenses: [] }),
		});
		expect(getByRole('status').textContent).toContain('Aucune écriture');
	});

	it('Story 37-4 — un compte en devise porte « USD 9.00 » à côté du nom, même à CHF 0', () => {
		const usd = compte({
			accountId: 3,
			accountNumber: '3200',
			accountName: 'Ventes USD',
			balance: '0.00',
			currency: 'USD',
			balanceCurrency: '9.0000',
		});
		const { getByTestId, queryByTestId, container } = render(IncomeStatementView, {
			dto: makeDto({ revenues: [compte(), usd] }),
		});
		expect(getByTestId('account-fx-3200').textContent).toBe('USD 9.00');
		expect(queryByTestId('account-fx-3000')).toBeNull();
		// Les totaux restent en devise principale.
		expect(container.querySelector('tfoot')?.textContent).not.toContain('USD');
	});

	it('mono-devise : rien de visible ne change (aucune mention en devise)', () => {
		const { container } = render(IncomeStatementView, { dto: makeDto() });
		expect(container.innerHTML).not.toContain('account-fx-');
		expect(container.textContent).not.toContain('USD');
	});
});
