// Story 39-11d2 (AC-2) — la page commune des prestataires **sans revue** (`api.getReview`
// absent), mode générique qu'aucun prestataire n'emploie plus depuis la revue Wise : ni exercice
// ni propositions, et les exercices ne sont pas lus — leur lecture en échec ne masque pas
// l'import (revue P1 de la 39-11d, L7, test repris de la page Wise).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

const listBankAccountsMock = vi.fn(async () => [{ id: 1, provider: 'x' }]);
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => listBankAccountsMock(),
}));
const listFiscalYearsMock = vi.fn(async () => [
	{ id: 5, companyId: 1, name: '2020', startDate: '2020-01-01', endDate: '2020-12-31', status: 'Open', multiCurrency: true },
]);
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({ listFiscalYears: () => listFiscalYearsMock() }));
vi.mock('$lib/features/reconciliation/reconciliation.api', () => ({
	getLinkCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	getEntryCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	linkEntry: vi.fn(),
	linkTransaction: vi.fn(),
}));

import ProviderReviewPage from './ProviderReviewPage.svelte';

const props = () => ({
	testId: 'generique',
	texts: {
		title: 'Prestataire',
		subtitle: 'Sous-titre',
		noAccount: 'Aucun compte',
		accountLabel: 'Compte',
		importButton: 'Importer',
		idColumn: 'Identifiant',
	},
	api: { importFile: vi.fn() },
	accountLabels: () => ['Compte Fictif'],
	importErrorLines: () => [],
});

describe('page commune sans revue (Story 39-11d2)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('ni exercice ni propositions ; les exercices ne sont pas lus', async () => {
		const r = render(ProviderReviewPage, { props: props() });
		expect(await r.findByTestId('generique-import-button')).toBeTruthy();
		expect(r.queryByTestId('generique-fiscal-year')).toBeNull();
		expect(r.queryByTestId('generique-proposals')).toBeNull();
		expect(r.queryByTestId('generique-to-review')).toBeNull();
		expect(listFiscalYearsMock).not.toHaveBeenCalled();
	});

	it('une lecture des exercices en échec ne masque pas l’import', async () => {
		listFiscalYearsMock.mockRejectedValueOnce(new Error('exercices indisponibles'));
		const r = render(ProviderReviewPage, { props: props() });
		expect(await r.findByTestId('generique-file-input')).toBeTruthy();
		expect(r.queryByTestId('generique-load-error')).toBeNull();
		listFiscalYearsMock.mockReset();
	});
});
