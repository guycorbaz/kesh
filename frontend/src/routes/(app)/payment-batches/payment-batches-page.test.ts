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
import { pinClock } from '$lib/shared/utils/pin-clock.test-helper';

// Story v0174-correctifs-3 (#889, AC-3.6, site 11) — à 00 h 30 le 5 novembre à Zurich, la
// date d'exécution proposée est le 5, non la veille (UTC).
describe('page des lots de virements — date d’exécution locale (#889)', () => {
	pinClock();
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([]);
		listSupplierInvoicesMock.mockResolvedValue({ items: [], total: 0 });
	});
	it('la date d’exécution par défaut est le 5 novembre', async () => {
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('payment-batch-new'));
		const form = await findByTestId('payment-batch-form');
		expect((form.querySelector('input[type="date"]') as HTMLInputElement).value).toBe(
			'2026-11-05',
		);
	});
});

function compte(id: number, description: string | null): BankAccountSummary {
	return {
		id,
		bankName: 'UBS',
		description,
		iban: `CH93007620116238529${50 + id}`,
		qrIban: null,
		accountKind: 'bank',
		cardLast4: null,
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

	// Story 34-11a1 (#972, AC-5) — un compte sans IBAN ne débite pas un pain.001 : il
	// n'est pas proposé comme compte source.
	it('un compte sans IBAN n’est pas proposé', async () => {
		const carte: BankAccountSummary = {
			...compte(3, null),
			bankName: 'Visa',
			iban: null,
			accountKind: 'credit_card',
			cardLast4: '1234',
			isPrimary: false,
		};
		listBankAccountsMock.mockResolvedValue([compte(1, 'Personnel'), carte]);
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('payment-batch-new'));
		const form = await findByTestId('payment-batch-form');
		const labels = Array.from(form.querySelectorAll('select option')).map(
			(o) => o.textContent?.trim() ?? '',
		);
		expect(labels).toContain(`UBS — Personnel — ${compte(1, null).iban}`);
		expect(labels.some((l) => l.includes('Visa'))).toBe(false);
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
		// #888 (0.17.4) : la première page porte désormais son `offset`.
		expect(listSupplierInvoicesMock.mock.calls[0][0]).toEqual({
			limit: 200,
			offset: 0,
			status: ['open'],
		});
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

// Story v0174-correctifs-3 (#888, AC-3.1 à 3.3) — le formulaire d'un lot propose TOUTES
// les factures validées : l'écran lit les pages de 200 jusqu'à `total`, s'arrête sur une
// page vide, et déduplique par `id` (une facture validée entre deux pages décale l'offset).
describe('page des lots de virements — toutes les factures validées (#888)', () => {
	function facture(id: number) {
		return {
			id,
			contactId: 1,
			contactName: `Fournisseur ${id}`,
			supplierInvoiceNumber: `FF-${id}`,
			status: 'open' as const,
			invoiceDate: '2026-05-10',
			dueDate: null,
			totalAmount: '100.00',
		};
	}
	function range(from: number, count: number) {
		return Array.from({ length: count }, (_, i) => facture(from + i));
	}
	function pages(total: number, ...items: ReturnType<typeof facture>[][]) {
		for (const page of items) {
			listSupplierInvoicesMock.mockResolvedValueOnce({ items: page, total, offset: 0, limit: 200 });
		}
	}
	async function boxesOnForm() {
		const { findByTestId, findAllByTestId } = render(Page);
		await fireEvent.click(await findByTestId('payment-batch-new'));
		return findAllByTestId('payment-batch-invoice-checkbox');
	}

	beforeEach(() => {
		vi.clearAllMocks();
		listSupplierInvoicesMock.mockReset();
		listBankAccountsMock.mockResolvedValue([compte(1, 'Personnel')]);
	});

	it('AC-3.1 — 450 factures en trois pages : trois appels, 450 cases', async () => {
		pages(450, range(1, 200), range(201, 200), range(401, 50));
		const boxes = await boxesOnForm();
		expect(boxes).toHaveLength(450);
		expect(listSupplierInvoicesMock.mock.calls.map((c) => c[0])).toEqual([
			{ limit: 200, offset: 0, status: ['open'] },
			{ limit: 200, offset: 200, status: ['open'] },
			{ limit: 200, offset: 400, status: ['open'] },
		]);
	});

	it('AC-3.2 — une page vide arrête la lecture (pas de boucle)', async () => {
		pages(450, range(1, 200), []);
		const boxes = await boxesOnForm();
		expect(boxes).toHaveLength(200);
		expect(listSupplierInvoicesMock).toHaveBeenCalledTimes(2);
	});

	it('AC-3.2 bis — une page qui recouvre la précédente : aucune case en double', async () => {
		pages(450, range(1, 200), range(191, 200), range(391, 50));
		const boxes = await boxesOnForm();
		expect(boxes).toHaveLength(440);
	});

	it('AC-3.3 — aucune facture : un appel, le message d’absence', async () => {
		pages(0, []);
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('payment-batch-new'));
		const form = await findByTestId('payment-batch-form');
		expect(form.textContent).toContain('Aucune facture fournisseur validée à payer.');
		expect(listSupplierInvoicesMock).toHaveBeenCalledTimes(1);
	});
});
