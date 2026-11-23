// Story 40-9 (intégration 38/40) — la page commune des prestataires choisit l'exercice
// par défaut au jour de Zurich, comme la période proposée au téléchargement. Le fuseau
// du processus est posé à UTC par le test (comme `settings/api-keys/api-keys-page.test.ts`) :
// le 31.12.2025 à 23 h 30 UTC est déjà le 01.01.2026 à Zurich.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

const listBankAccountsMock = vi.fn<(archived?: boolean, closed?: boolean) => Promise<BankAccountSummary[]>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: (archived?: boolean, closed?: boolean) => listBankAccountsMock(archived, closed),
}));
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({
	listFiscalYears: async () => [
		{ id: 25, companyId: 1, name: '2025', startDate: '2025-01-01', endDate: '2025-12-31', status: 'Open', multiCurrency: true },
		{ id: 26, companyId: 1, name: '2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Open', multiCurrency: true },
	],
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: async () => [
		{ id: 31, number: '1030', name: 'Stripe CHF', accountType: 'Asset', active: true, postable: true },
		{ id: 60, number: '6940', name: 'Frais Stripe', accountType: 'Expense', active: true, postable: true },
	],
}));
vi.mock('$lib/features/reconciliation/reconciliation.api', () => ({
	getLinkCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	getEntryCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	linkEntry: vi.fn(),
	linkTransaction: vi.fn(),
}));

const importMock = vi.fn();
const reviewMock = vi.fn();
const acceptMock = vi.fn();
vi.mock('$lib/features/stripe/stripe.api', () => ({
	importStripeReport: (...a: unknown[]) => importMock(...a),
	getStripeReview: (...a: unknown[]) => reviewMock(...a),
	acceptStripeProposals: (...a: unknown[]) => acceptMock(...a),
}));

// Story 40-8b — le dialogue de téléchargement : seules ses routes sont simulées.
const startDownloadMock = vi.fn();
const getDownloadMock = vi.fn();
vi.mock('$lib/features/provider-download/provider-download.api', async (importOriginal) => ({
	...(await importOriginal<typeof import('$lib/features/provider-download/provider-download.api')>()),
	startProviderDownload: (...a: unknown[]) => startDownloadMock(...a),
	getProviderDownload: (...a: unknown[]) => getDownloadMock(...a),
}));

import Page from './+page.svelte';

function stripeAccount(id: number, account: string): BankAccountSummary {
	return {
		id,
		bankName: 'Stripe',
		description: null,
		iban: null,
		qrIban: null,
		accountKind: 'bank',
		cardLast4: null,
		provider: 'stripe',
		paypal: null,
		stripe: { account, feeAccountId: 60, payoutBankAccountId: 9 },
		wise: null,
		isPrimary: false,
		journalAccountId: 31,
		version: 1,
		archived: false,
		closedOn: null,
		currentBalance: null,
		lastTransactionDate: null,
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		statementGap: null,
	};
}

describe('page Stripe — exercice par défaut au jour de Zurich (Story 40-9)', () => {
	let tz: string | undefined;
	beforeEach(() => {
		vi.clearAllMocks();
		tz = process.env.TZ;
		process.env.TZ = 'UTC';
		vi.useFakeTimers({ toFake: ['Date'] });
		vi.setSystemTime(new Date('2025-12-31T23:30:00Z'));
		listBankAccountsMock.mockResolvedValue([stripeAccount(1, 'Stripe boutique')]);
		reviewMock.mockResolvedValue({ account: 'Stripe boutique', fiscalYearId: 26, proposals: [], gaps: [], toReview: [] });
	});
	afterEach(() => {
		vi.useRealTimers();
		if (tz === undefined) delete process.env.TZ;
		else process.env.TZ = tz;
	});

	it('le 31.12 à 23 h 30 UTC : la revue est demandée pour l’exercice 2026', async () => {
		// Montage : le jour local du processus est encore le 31.12.2025.
		expect(new Date().getDate()).toBe(31);
		render(Page);
		await waitFor(() => expect(reviewMock).toHaveBeenCalled());
		expect(reviewMock.mock.calls[0][1]).toBe(26);
	});
});
