// Story v0171 (#871) — la date d'exécution d'un lot s'affiche en forme suisse dans la
// liste des lots (`29.01.2026`, jamais `2026-01-29`). Mutation M5 : retirer
// `formatSwissDate` du site rougit ce test.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/features/payment-batches/payment-batches.api', () => ({
	createPaymentBatch: vi.fn(),
	listPaymentBatches: () =>
		Promise.resolve({
			items: [
				{
					id: 7,
					bankAccountId: 1,
					status: 'confirmed',
					requestedExecutionDate: '2026-01-29',
					totalAmount: '100.00',
					createdAt: '2026-01-20T08:00:00',
				},
			],
			total: 1,
		}),
}));
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	listSupplierInvoices: () => Promise.resolve({ items: [], total: 0 }),
}));
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => Promise.resolve([]),
}));

import Page from './+page.svelte';

describe('liste des lots de virements — date d’exécution (v0171)', () => {
	it('`2026-01-29` s’affiche `29.01.2026` (mutation M5)', async () => {
		const { findByTestId, container } = render(Page);
		const row = await findByTestId('payment-batch-row');
		expect(row.textContent).toContain('29.01.2026');
		expect(container.textContent).not.toContain('2026-01-29');
	});
});
