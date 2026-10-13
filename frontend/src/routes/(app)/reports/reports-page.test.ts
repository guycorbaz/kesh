// Story 28-2 (#520, AC9 — B15) — premier test de la page des rapports : un
// rejet d'un rapport qui n'est pas une `ApiError` est rendu par `formatError`,
// dont la dernière branche passe désormais par `errorMessageOf` — le générique
// traduit, ni le message technique ni « [object Object] ». La branche
// `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` est vérifiée préservée.
//
// ⚠️ Le rejet passe par `generate()` (ici le grand livre, ouvert par un lien
// profond `?tab=general-ledger&from=…&to=…`, qui le génère d'office), dont le
// `catch` appelle `formatError` sans condition — et non par un export, qui ne
// l'appelle que pour une `ApiError` porteuse d'un `code`.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';

const { getGeneralLedgerMock } = vi.hoisted(() => ({ getGeneralLedgerMock: vi.fn() }));

vi.mock('$app/state', () => ({
	page: {
		url: new URL('http://localhost/reports?tab=general-ledger&from=2026-01-01&to=2026-12-31'),
	},
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));
vi.mock('$lib/features/projects/projects.api', () => ({ listProjects: async () => [] }));
vi.mock('$lib/features/accounts/accounts.api', () => ({ fetchAccounts: async () => [] }));
vi.mock('$lib/features/reports/reports.api', async (importOriginal) => {
	const vrai = await importOriginal<typeof import('$lib/features/reports/reports.api')>();
	return {
		buildExportFilename: vrai.buildExportFilename,
		formatSwissDate: vrai.formatSwissDate,
		downloadReport: vi.fn(),
		downloadProjectReport: vi.fn(),
		getBalanceSheet: vi.fn(),
		getIncomeStatement: vi.fn(),
		getJournalReport: vi.fn(),
		getProjectExpenses: vi.fn(),
		getProjectReturn: vi.fn(),
		getTrialBalance: vi.fn(),
		getVatReport: vi.fn(),
		getGeneralLedger: (q: unknown) => getGeneralLedgerMock(q),
		downloadGeneralLedger: vi.fn(),
		getAgedReceivables: vi.fn(),
		downloadAgedReceivables: vi.fn(),
	};
});

import Page from './+page.svelte';

const data = { fiscalYears: [], companyName: 'Test SA' };

describe('page des rapports — erreur d’un rapport (Story 28-2, B15)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	it('rejet non-ApiError de generate() → le générique (M21)', async () => {
		getGeneralLedgerMock.mockRejectedValue(new Error('boom'));
		const { findByRole } = render(Page, { data });
		const alerte = await findByRole('alert');
		expect(getGeneralLedgerMock).toHaveBeenCalledTimes(1);
		expect(alerte.textContent).toBe('Erreur inattendue.');
		expect(alerte.textContent).not.toContain('boom');
		expect(alerte.textContent).not.toContain('[object Object]');
	});

	it('REPORT_PERIOD_OUT_OF_FISCAL_YEAR → les bornes de l’exercice, branche préservée', async () => {
		getGeneralLedgerMock.mockRejectedValue({
			code: 'REPORT_PERIOD_OUT_OF_FISCAL_YEAR',
			message: 'serveur',
			status: 400,
			details: { fyStart: '2026-01-01', fyEnd: '2026-12-31' },
		});
		const { findByRole } = render(Page, { data });
		const alerte = await findByRole('alert');
		expect(alerte.textContent).toContain('dépasse les bornes');
		expect(alerte.textContent).toContain('01.01.2026');
	});

	it('ApiError ordinaire → son message', async () => {
		getGeneralLedgerMock.mockRejectedValue({ code: 'X', message: 'Refus du serveur.', status: 400 });
		const { findByRole } = render(Page, { data });
		expect((await findByRole('alert')).textContent).toBe('Refus du serveur.');
	});
});
