// Story 40-7d (AC-1) — le compte Stripe à l'écran des comptes bancaires : case « Compte
// Stripe » à côté de « Compte PayPal » (l'une exclut l'autre), libellé, compte de frais,
// compte bancaire de versement (même devise, ni Stripe ni PayPal, vidé quand la devise du
// compte du plan change), charges `stripe` (création, modification, décochée), ligne de la
// liste, pas de « Lier ».
//
// Patron et mocks de `bank-accounts-page-paypal.test.ts` (38-11d).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

const fetchAccountsMock = vi.fn<() => Promise<AccountResponse[]>>();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: () => fetchAccountsMock(),
}));

const listBankAccountsMock = vi.fn<() => Promise<BankAccountSummary[]>>();
const createBankAccountMock = vi.fn<(payload: unknown) => Promise<unknown>>();
const updateBankAccountMock = vi.fn<(id: number, payload: unknown) => Promise<unknown>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => listBankAccountsMock(),
	createBankAccount: (payload: unknown) => createBankAccountMock(payload),
	updateBankAccount: (id: number, payload: unknown) => updateBankAccountMock(id, payload),
	archiveBankAccount: vi.fn(),
	closeBankAccount: vi.fn(),
	reopenBankAccount: vi.fn(),
	updateBankAccountJournalLink: vi.fn(),
}));

vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoiceSettings: async () => ({ defaultReceivableAccountId: null, defaultPayableAccountId: null }),
}));

import Page from './+page.svelte';

function account(overrides: Partial<AccountResponse>): AccountResponse {
	return {
		id: 0,
		companyId: 1,
		number: '',
		name: '',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: false,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...overrides,
	};
}

const BANQUE = account({ id: 20, number: '1020', name: 'Banque' });
const STRIPE_CHF = account({ id: 21, number: '1030', name: 'Stripe CHF' });
const STRIPE_EUR = account({ id: 22, number: '1031', name: 'Stripe EUR', currency: 'EUR' });
const WISE_EUR = account({ id: 23, number: '1025', name: 'Wise EUR', currency: 'EUR' });
const FRAIS_CHF = account({ id: 60, number: '6940', name: 'Frais Stripe', accountType: 'Expense' });
const FRAIS_EUR = account({ id: 61, number: '6941', name: 'Frais Stripe EUR', accountType: 'Expense', currency: 'EUR' });
const PRODUIT = account({ id: 70, number: '3200', name: 'Ventes', accountType: 'Revenue' });

function bank(overrides: Partial<BankAccountSummary>): BankAccountSummary {
	return {
		id: 0,
		bankName: 'UBS',
		description: null,
		iban: 'CH9300762011623852957',
		qrIban: null,
		accountKind: 'bank',
		cardLast4: null,
		provider: null,
		paypal: null,
		stripe: null,
		isPrimary: false,
		journalAccountId: BANQUE.id,
		version: 1,
		archived: false,
		closedOn: null,
		currentBalance: '0.00',
		lastTransactionDate: '2026-01-02',
		statementClosingBalance: null,
		statementDate: null,
		ledgerBalanceAtStatement: null,
		statementGap: null,
		...overrides,
	};
}

const principal = bank({ id: 1, isPrimary: true, bankName: 'UBS' });
const stripeEur = bank({
	id: 2,
	bankName: 'Stripe',
	iban: null,
	provider: 'stripe',
	stripe: { account: 'Stripe boutique', feeAccountId: FRAIS_EUR.id, payoutBankAccountId: 4 },
	journalAccountId: STRIPE_EUR.id,
	version: 4,
});
const ordinaire = bank({ id: 3, bankName: 'Raiffeisen', journalAccountId: BANQUE.id });
const wiseEur = bank({ id: 4, bankName: 'Wise', iban: null, provider: 'wise', journalAccountId: WISE_EUR.id });
const paypalChf = bank({
	id: 5,
	bankName: 'PayPal',
	iban: null,
	provider: 'paypal',
	paypal: { account: 'PayPal www', feeAccountId: null },
	journalAccountId: STRIPE_CHF.id,
});

const options = (el: HTMLElement) =>
	Array.from((el as HTMLSelectElement).options).map((o) => o.value);

async function fill(el: HTMLElement, value: string) {
	await fireEvent.input(el, { target: { value } });
}

describe('page des comptes bancaires — le compte Stripe (Story 40-7d, AC-1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue([BANQUE, STRIPE_CHF, STRIPE_EUR, WISE_EUR, FRAIS_CHF, FRAIS_EUR, PRODUIT]);
		listBankAccountsMock.mockResolvedValue([principal, stripeEur, ordinaire, wiseEur, paypalChf]);
		createBankAccountMock.mockResolvedValue({});
		updateBankAccountMock.mockResolvedValue({});
	});

	it('les cases PayPal et Stripe s’excluent ; le compte de versement n’existe que pour Stripe', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		const paypal = (await r.findByTestId('form-is-paypal')) as HTMLInputElement;
		const stripe = (await r.findByTestId('form-is-stripe')) as HTMLInputElement;
		await fireEvent.click(stripe);
		expect(stripe.checked).toBe(true);
		expect(r.queryByTestId('form-payout-account')).not.toBeNull();
		expect(r.queryByTestId('form-stripe-account')).not.toBeNull();
		await fireEvent.click(paypal);
		expect(paypal.checked).toBe(true);
		expect(stripe.checked).toBe(false);
		expect(r.queryByTestId('form-payout-account')).toBeNull();
		expect(r.queryByTestId('form-paypal-account')).not.toBeNull();
		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		expect(r.queryByTestId('form-is-stripe')).toBeNull();
	});

	it('cochée : QR-IBAN et principal masqués, comptes en devise offerts, libellés Stripe seuls au datalist', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-stripe'));
		expect(r.queryByTestId('form-qr-iban')).toBeNull();
		expect(r.queryByTestId('form-is-primary')).toBeNull();
		expect(options(await r.findByTestId('form-journal-account'))).toContain('22');
		const labels = Array.from((await r.findByTestId('form-stripe-labels')).querySelectorAll('option')).map(
			(o) => o.getAttribute('value'),
		);
		expect(labels).toEqual(['Stripe boutique']);
	});

	it('compte de versement : même devise que le compte du plan, ni Stripe ni PayPal ; vidé quand la devise change', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-stripe'));
		await fireEvent.change(await r.findByTestId('form-journal-account'), { target: { value: '22' } });
		const payout = await r.findByTestId('form-payout-account');
		// « Non configuré », puis le seul compte EUR admis : Wise (le compte Stripe EUR est exclu).
		await waitFor(() => expect(options(payout).slice(1)).toEqual(['4']));
		await fireEvent.change(payout, { target: { value: '4' } });
		expect((payout as HTMLSelectElement).value).toBe('4');
		await fireEvent.change(r.getByTestId('form-journal-account'), { target: { value: '21' } });
		// CHF : les banques CHF (UBS, Raiffeisen), jamais le compte PayPal CHF ; le Wise EUR se vide.
		await waitFor(() => expect(options(r.getByTestId('form-payout-account')).slice(1)).toEqual(['3', '1']));
		expect((r.getByTestId('form-payout-account') as HTMLSelectElement).selectedIndex).toBe(0);
	});

	it('création : charge `stripe` avec compte de frais et de versement, IBAN vide → null', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-stripe'));
		await fill(await r.findByTestId('form-bank-name'), 'Stripe');
		await fill(await r.findByTestId('form-stripe-account'), ' Stripe boutique ');
		await fireEvent.change(await r.findByTestId('form-journal-account'), { target: { value: '22' } });
		await fireEvent.change(await r.findByTestId('form-fee-account'), { target: { value: '61' } });
		await fireEvent.change(await r.findByTestId('form-payout-account'), { target: { value: '4' } });
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(createBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = createBankAccountMock.mock.calls[0][0] as Record<string, unknown>;
		expect(payload.stripe).toEqual({ account: 'Stripe boutique', feeAccountId: 61, payoutBankAccountId: 4 });
		expect('paypal' in payload).toBe(false);
		expect(payload.iban).toBeNull();
		expect(payload.isPrimary).toBe(false);
	});

	it('création : libellé vide, refus à l’écran nommant Stripe, sans appel', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-stripe'));
		await fill(await r.findByTestId('form-bank-name'), 'Stripe');
		await fireEvent.click(await r.findByTestId('form-submit'));
		expect((await r.findByTestId('form-error')).textContent).toContain('libellé du compte Stripe');
		await fill(await r.findByTestId('form-stripe-account'), 'Stripe boutique');
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(r.getByTestId('form-error').textContent).toContain('solde Stripe'));
		expect(createBankAccountMock).not.toHaveBeenCalled();
	});

	it('modification d’un compte Stripe : champs préremplis, charge `stripe` renvoyée', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		expect(((await r.findByTestId('edit-is-stripe')) as HTMLInputElement).checked).toBe(true);
		expect(((await r.findByTestId('edit-stripe-account')) as HTMLInputElement).value).toBe('Stripe boutique');
		expect(((await r.findByTestId('edit-fee-account')) as HTMLSelectElement).value).toBe('61');
		await waitFor(() => expect(((r.getByTestId('edit-payout-account')) as HTMLSelectElement).value).toBe('4'));
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const [id, payload] = updateBankAccountMock.mock.calls[0] as [number, Record<string, unknown>];
		expect(id).toBe(2);
		expect(payload.stripe).toEqual({ account: 'Stripe boutique', feeAccountId: 61, payoutBankAccountId: 4 });
	});

	it('modification : décochée sur un compte Stripe → `stripe: null`, `paypal` absent', async () => {
		listBankAccountsMock.mockResolvedValue([
			bank({ ...stripeEur, journalAccountId: STRIPE_CHF.id, iban: 'CH9300762011623852957' }),
		]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		await fireEvent.click(await r.findByTestId('edit-is-stripe'));
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = updateBankAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect('stripe' in payload).toBe(true);
		expect(payload.stripe).toBeNull();
		expect('paypal' in payload).toBe(false);
	});

	it('liste : « Stripe · <compte> · <devise> », pas de « Lier » sur un compte Stripe', async () => {
		const r = render(Page);
		expect((await r.findByTestId('bank-account-stripe-2')).textContent).toBe('Stripe · Stripe boutique · EUR');
		expect(r.queryByTestId('link-button-2')).toBeNull();
		expect(r.queryByTestId('link-button-3')).not.toBeNull();
	});
});
