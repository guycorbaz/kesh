// Story 39-11d (AC-1) — le compte Wise à l'écran des comptes bancaires : case « Compte Wise »
// à côté de « Compte PayPal » et « Compte Stripe » (l'une exclut les autres), champs masqués,
// IBAN facultatif, comptes du plan en devise offerts, charges `wise` (cochée, décochée,
// modification), textes qui nomment Wise, ligne de la liste, pas de « Lier ».
//
// Patron et mocks de `bank-accounts-page-paypal.test.ts`.

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
const WISE_CHF = account({ id: 21, number: '1030', name: 'Wise CHF' });
const WISE_EUR = account({ id: 22, number: '1031', name: 'Wise EUR', currency: 'EUR' });
const FRAIS_CHF = account({ id: 60, number: '6940', name: 'Frais Wise', accountType: 'Expense' });
const FRAIS_EUR = account({ id: 61, number: '6941', name: 'Frais Wise EUR', accountType: 'Expense', currency: 'EUR' });
const FRAIS_USD = account({ id: 62, number: '6942', name: 'Frais Wise USD', accountType: 'Expense', currency: 'USD' });
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
		wise: null,
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

const principal = bank({ id: 1, isPrimary: true });
const wiseEur = bank({
	id: 2,
	bankName: 'Wise',
	iban: null,
	provider: 'wise',
	wise: { account: 'Wise Fictif', feeAccountId: FRAIS_EUR.id },
	journalAccountId: WISE_EUR.id,
	version: 4,
});
const paypalChf = bank({
	id: 4,
	bankName: 'PayPal',
	iban: null,
	provider: 'paypal',
	paypal: { account: 'PayPal www', feeAccountId: null },
	journalAccountId: WISE_CHF.id,
});
const ordinaire = bank({ id: 3, bankName: 'Raiffeisen', journalAccountId: BANQUE.id });

const options = (el: HTMLElement) =>
	Array.from((el as HTMLSelectElement).options).map((o) => o.value);

async function fill(el: HTMLElement, value: string) {
	await fireEvent.input(el, { target: { value } });
}

describe('page des comptes bancaires — le compte Wise (Story 39-11d, AC-1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue([BANQUE, WISE_CHF, WISE_EUR, FRAIS_CHF, FRAIS_EUR, FRAIS_USD, PRODUIT]);
		listBankAccountsMock.mockResolvedValue([principal, wiseEur, ordinaire, paypalChf]);
		createBankAccountMock.mockResolvedValue({});
		updateBankAccountMock.mockResolvedValue({});
	});

	it('trois cases de prestataire, chacune exclut les autres ; aucune pour une carte', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		const wise = (await r.findByTestId('form-is-wise')) as HTMLInputElement;
		const paypal = r.getByTestId('form-is-paypal') as HTMLInputElement;
		await fireEvent.click(paypal);
		await fireEvent.click(wise);
		expect(wise.checked).toBe(true);
		expect(paypal.checked).toBe(false);
		expect((r.getByTestId('form-is-stripe') as HTMLInputElement).checked).toBe(false);
		expect(r.getByTestId('form-is-wise-help').textContent).toContain('page Wise');
		expect(r.queryByTestId('form-payout-account')).toBeNull();
		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		expect(r.queryByTestId('form-is-wise')).toBeNull();
	});

	it('cochée : QR-IBAN et principal masqués, IBAN facultatif, comptes en devise offerts, libellés Wise seuls', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		expect(options(await r.findByTestId('form-journal-account'))).not.toContain('22');
		await fireEvent.click(await r.findByTestId('form-is-wise'));
		expect(r.queryByTestId('form-qr-iban')).toBeNull();
		expect(r.queryByTestId('form-is-primary')).toBeNull();
		expect((await r.findByTestId('form-iban')).hasAttribute('required')).toBe(false);
		expect(options(await r.findByTestId('form-journal-account'))).toContain('22');
		expect((r.getByTestId('form-journal-account-help')).textContent).toContain('solde Wise');
		const labels = Array.from((await r.findByTestId('form-wise-labels')).querySelectorAll('option')).map((o) =>
			o.getAttribute('value'),
		);
		expect(labels).toEqual(['Wise Fictif']);
	});

	it('création : charge `wise`, ni `paypal` ni `stripe`, IBAN vide → null', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-wise'));
		await fill(await r.findByTestId('form-bank-name'), 'Wise');
		await fill(await r.findByTestId('form-wise-account'), '  Wise Fictif ');
		await fireEvent.change(await r.findByTestId('form-journal-account'), { target: { value: '22' } });
		await fireEvent.change(await r.findByTestId('form-fee-account'), { target: { value: '61' } });
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(createBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = createBankAccountMock.mock.calls[0][0] as Record<string, unknown>;
		expect(payload.wise).toEqual({ account: 'Wise Fictif', feeAccountId: 61 });
		expect('paypal' in payload).toBe(false);
		expect('stripe' in payload).toBe(false);
		expect(payload.iban).toBeNull();
		expect(payload.isPrimary).toBe(false);
		expect(payload.journalAccountId).toBe(22);
	});

	it('création : libellé vide puis compte du plan absent, refus à l’écran nommant Wise, sans appel', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-wise'));
		await fill(await r.findByTestId('form-bank-name'), 'Wise');
		await fireEvent.click(await r.findByTestId('form-submit'));
		expect((await r.findByTestId('form-error')).textContent).toContain('libellé du compte Wise');
		await fill(await r.findByTestId('form-wise-account'), 'Wise Fictif');
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(r.getByTestId('form-error').textContent).toContain('solde Wise'));
		expect(createBankAccountMock).not.toHaveBeenCalled();
	});

	it('modification d’un compte Wise : champs préremplis, charge `wise` envoyée', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		expect(((await r.findByTestId('edit-is-wise')) as HTMLInputElement).checked).toBe(true);
		expect(((await r.findByTestId('edit-wise-account')) as HTMLInputElement).value).toBe('Wise Fictif');
		expect(((await r.findByTestId('edit-fee-account')) as HTMLSelectElement).value).toBe('61');
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const [id, payload] = updateBankAccountMock.mock.calls[0] as [number, Record<string, unknown>];
		expect(id).toBe(2);
		expect(payload.wise).toEqual({ account: 'Wise Fictif', feeAccountId: 61 });
		expect('paypal' in payload).toBe(false);
	});

	it('modification : décochée sur un compte Wise → `wise: null`, `paypal` et `stripe` absents', async () => {
		listBankAccountsMock.mockResolvedValue([
			bank({ ...wiseEur, journalAccountId: WISE_CHF.id, iban: 'CH9300762011623852957' }),
		]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		await fireEvent.click(await r.findByTestId('edit-is-wise'));
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = updateBankAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect('wise' in payload).toBe(true);
		expect(payload.wise).toBeNull();
		expect('paypal' in payload).toBe(false);
		expect('stripe' in payload).toBe(false);
	});

	it('compte principal : la case Wise est désactivée, le motif nomme Wise', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-1'));
		expect(((await r.findByTestId('edit-is-wise')) as HTMLInputElement).disabled).toBe(true);
		expect((await r.findByTestId('edit-is-paypal-locked')).textContent).toContain('Wise');
	});

	it('liste : « Wise · <compte> · <devise> » sous le nom, pas de « Lier » sur un compte Wise', async () => {
		const r = render(Page);
		expect((await r.findByTestId('bank-account-wise-2')).textContent).toBe('Wise · Wise Fictif · EUR');
		expect(r.queryByTestId('link-button-2')).toBeNull();
		expect(r.queryByTestId('link-button-3')).not.toBeNull();
	});
});
