// Story 26-2b (#622, AC9, site D4) — le sélecteur du compte bancaire source d'un
// lot de virements nomme le compte avec sa description.
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
vi.mock('$lib/features/payment-batches/payment-batches.api', () => ({
	createPaymentBatch: vi.fn(),
	listPaymentBatches: () => Promise.resolve({ items: [], total: 0 }),
}));
const listSupplierInvoicesMock = vi.fn();
vi.mock('$lib/features/supplier-invoices/supplier-invoices.api', () => ({
	listSupplierInvoices: (q: unknown) => listSupplierInvoicesMock(q),
}));
const listBankAccountsMock = vi.fn<() => Promise<BankAccountSummary[]>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => listBankAccountsMock(),
}));

import Page from './+page.svelte';

function compte(id: number, description: string | null): BankAccountSummary {
	return {
		id,
		bankName: 'UBS',
		description,
		iban: `CH93007620116238529${50 + id}`,
		qrIban: null,
		isPrimary: id === 1,
		journalAccountId: 1020,
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

describe('page des lots de virements — compte source (Story 26-2b, D4)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([compte(1, 'Personnel'), compte(2, null)]);
		listSupplierInvoicesMock.mockResolvedValue({ items: [], total: 0 });
	});

	it('l’option nomme le compte décrit, et garde « banque — IBAN » sans description', async () => {
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('payment-batch-new'));
		const form = await findByTestId('payment-batch-form');
		const labels = Array.from(form.querySelectorAll('select option')).map(
			(o) => o.textContent?.trim() ?? '',
		);
		expect(labels).toContain(`UBS — Personnel — ${compte(1, null).iban}`);
		expect(labels).toContain(`UBS — ${compte(2, null).iban}`);
	});
});

// Story v017-4b (#849, AC-27) — l'écran des lots ne propose jamais un brouillon : il
// demande au serveur les seules factures validées (`status=open`, AC-15), et garde le
// filtre client en défense.
describe('page des lots de virements — factures proposées (Story v017-4b, AC-27)', () => {
	function facture(id: number, status: 'draft' | 'open' | 'paid' | 'cancelled') {
		return {
			id,
			contactId: 1,
			contactName: `Fournisseur ${id}`,
			supplierInvoiceNumber: `FF-${id}`,
			status,
			invoiceDate: '2026-05-10',
			dueDate: null,
			totalAmount: '100.00',
		};
	}

	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([compte(1, 'Personnel')]);
	});

	it('demande les factures validées au serveur (M30)', async () => {
		listSupplierInvoicesMock.mockResolvedValue({ items: [], total: 0 });
		render(Page);
		await vi.waitFor(() => expect(listSupplierInvoicesMock).toHaveBeenCalled());
		expect(listSupplierInvoicesMock.mock.calls[0][0]).toEqual({ limit: 200, status: ['open'] });
	});

	it('un brouillon rendu malgré tout n’est jamais proposé (défense)', async () => {
		listSupplierInvoicesMock.mockResolvedValue({
			items: [facture(1, 'draft'), facture(2, 'open')],
			total: 2,
		});
		const { findByTestId, findAllByTestId } = render(Page);
		await fireEvent.click(await findByTestId('payment-batch-new'));
		const boxes = await findAllByTestId('payment-batch-invoice-checkbox');
		expect(boxes).toHaveLength(1);
		const form = await findByTestId('payment-batch-form');
		expect(form.textContent).toContain('Fournisseur 2');
		expect(form.textContent).not.toContain('Fournisseur 1');
	});
});
