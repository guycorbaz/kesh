// Story 38-11d (AC-1) — le compte PayPal à l'écran des comptes bancaires : case
// « Compte PayPal » sous le type *Compte bancaire*, champs masqués, IBAN
// facultatif, comptes du plan en devise offerts seulement case cochée, compte
// de frais recalculé, charges `paypal` (cochée, décochée, inchangée, carte),
// case désactivée sur le compte principal, ligne de la liste, pas de « Lier ».
//
// Patron et mocks de `bank-accounts-page-credit-card.test.ts`.

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
const PAYPAL_CHF = account({ id: 21, number: '1030', name: 'PayPal CHF' });
const PAYPAL_EUR = account({ id: 22, number: '1031', name: 'PayPal EUR', currency: 'EUR' });
const FRAIS_CHF = account({ id: 60, number: '6940', name: 'Frais PayPal', accountType: 'Expense' });
const FRAIS_EUR = account({ id: 61, number: '6941', name: 'Frais PayPal EUR', accountType: 'Expense', currency: 'EUR' });
const FRAIS_USD = account({ id: 62, number: '6942', name: 'Frais PayPal USD', accountType: 'Expense', currency: 'USD' });
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

const principal = bank({ id: 1, isPrimary: true });
const paypalEur = bank({
	id: 2,
	bankName: 'PayPal',
	iban: null,
	provider: 'paypal',
	paypal: { account: 'PayPal www', feeAccountId: FRAIS_EUR.id },
	journalAccountId: PAYPAL_EUR.id,
	version: 4,
});
const ordinaire = bank({ id: 3, bankName: 'Raiffeisen', journalAccountId: BANQUE.id });

const options = (el: HTMLElement) =>
	Array.from((el as HTMLSelectElement).options).map((o) => o.value);

async function fill(el: HTMLElement, value: string) {
	await fireEvent.input(el, { target: { value } });
}

describe('page des comptes bancaires — le compte PayPal (Story 38-11d, AC-1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue([BANQUE, PAYPAL_CHF, PAYPAL_EUR, FRAIS_CHF, FRAIS_EUR, FRAIS_USD, PRODUIT]);
		listBankAccountsMock.mockResolvedValue([principal, paypalEur, ordinaire]);
		createBankAccountMock.mockResolvedValue({});
		updateBankAccountMock.mockResolvedValue({});
	});

	it('la case n’existe que pour un compte bancaire, jamais pour une carte', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		expect(r.queryByTestId('form-is-paypal')).not.toBeNull();
		await fireEvent.click(await r.findByTestId('form-kind-credit-card'));
		expect(r.queryByTestId('form-is-paypal')).toBeNull();
	});

	it('cochée : QR-IBAN et principal masqués, IBAN facultatif, comptes en devise offerts', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		expect(options(await r.findByTestId('form-journal-account'))).not.toContain('22');
		expect((await r.findByTestId('form-iban')).hasAttribute('required')).toBe(true);
		await fireEvent.click(await r.findByTestId('form-is-paypal'));
		expect(r.queryByTestId('form-qr-iban')).toBeNull();
		expect(r.queryByTestId('form-is-primary')).toBeNull();
		expect((await r.findByTestId('form-iban')).hasAttribute('required')).toBe(false);
		expect(options(await r.findByTestId('form-journal-account'))).toContain('22');
		// Le datalist propose les libellés déjà portés.
		const labels = Array.from((await r.findByTestId('form-paypal-labels')).querySelectorAll('option')).map(
			(o) => o.getAttribute('value'),
		);
		expect(labels).toEqual(['PayPal www']);
	});

	it('décochée : un compte du plan en devise choisi est retiré', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-paypal'));
		await fireEvent.change(await r.findByTestId('form-journal-account'), { target: { value: '22' } });
		await fireEvent.click(await r.findByTestId('form-is-paypal'));
		expect((r.getByTestId('form-journal-account') as HTMLSelectElement).value).not.toBe('22');
		expect(options(r.getByTestId('form-journal-account'))).not.toContain('22');
	});

	// Revue P1 (L7) — le compte de frais choisi avant le compte du plan n'est pas effacé en
	// silence ; il ne l'est que si la devise du compte du plan ne l'admet plus.
	it('compte de frais choisi avant le compte du plan : gardé s’il reste admis', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-paypal'));
		await fireEvent.change(await r.findByTestId('form-fee-account'), { target: { value: '60' } });
		await fireEvent.change(await r.findByTestId('form-journal-account'), { target: { value: '22' } });
		await waitFor(() => expect((r.getByTestId('form-fee-account') as HTMLSelectElement).value).toBe('60'));
	});

	it('compte de frais : charges CHF ou de la devise du compte du plan, vidé quand la devise ne l’admet plus', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-paypal'));
		await fireEvent.change(await r.findByTestId('form-journal-account'), { target: { value: '22' } });
		const fee = await r.findByTestId('form-fee-account');
		// Première option : « Non configuré » ; puis les charges en CHF ou en EUR, triées.
		expect(options(fee).slice(1)).toEqual(['60', '61']);
		await fireEvent.change(fee, { target: { value: '61' } });
		expect((fee as HTMLSelectElement).value).toBe('61');
		// Compte du plan en CHF : le compte de frais EUR n'est plus admis, il se vide.
		await fireEvent.change(r.getByTestId('form-journal-account'), { target: { value: '21' } });
		await waitFor(() => expect(options(r.getByTestId('form-fee-account'))).not.toContain('61'));
		expect((r.getByTestId('form-fee-account') as HTMLSelectElement).selectedIndex).toBe(0);
	});

	it('création : charge `paypal`, IBAN vide → null, ni QR-IBAN ni principal ; pas de solde d’ouverture en devise', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-paypal'));
		await fill(await r.findByTestId('form-bank-name'), 'PayPal');
		await fill(await r.findByTestId('form-paypal-account'), '  PayPal www ');
		await fireEvent.change(await r.findByTestId('form-journal-account'), { target: { value: '22' } });
		expect(r.queryByTestId('form-opening-balance')).toBeNull();
		await fireEvent.change(await r.findByTestId('form-fee-account'), { target: { value: '61' } });
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(createBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = createBankAccountMock.mock.calls[0][0] as Record<string, unknown>;
		expect(payload.paypal).toEqual({ account: 'PayPal www', feeAccountId: 61 });
		expect(payload.iban).toBeNull();
		expect(payload.qrIban).toBeNull();
		expect(payload.isPrimary).toBe(false);
		expect(payload.journalAccountId).toBe(22);
		expect('openingBalance' in payload).toBe(false);
	});

	it('création : libellé vide ou compte du plan absent, refus à l’écran sans appel', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fireEvent.click(await r.findByTestId('form-is-paypal'));
		await fill(await r.findByTestId('form-bank-name'), 'PayPal');
		await fireEvent.click(await r.findByTestId('form-submit'));
		expect((await r.findByTestId('form-error')).textContent).toContain('libellé du compte PayPal');
		await fill(await r.findByTestId('form-paypal-account'), 'PayPal www');
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(r.getByTestId('form-error').textContent).toContain('compte du plan lié'));
		expect(createBankAccountMock).not.toHaveBeenCalled();
	});

	it('création d’un compte ordinaire et d’une carte : aucune charge `paypal`', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('create-bank-account-button'));
		await fill(await r.findByTestId('form-bank-name'), 'UBS');
		await fill(await r.findByTestId('form-iban'), 'CH9300762011623852957');
		await fireEvent.click(await r.findByTestId('form-submit'));
		await waitFor(() => expect(createBankAccountMock).toHaveBeenCalledTimes(1));
		expect('paypal' in (createBankAccountMock.mock.calls[0][0] as object)).toBe(false);
	});

	it('modification d’un compte PayPal : champs préremplis, charge `paypal` envoyée', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		expect((await r.findByTestId('edit-is-paypal') as HTMLInputElement).checked).toBe(true);
		expect(((await r.findByTestId('edit-paypal-account')) as HTMLInputElement).value).toBe('PayPal www');
		expect(((await r.findByTestId('edit-fee-account')) as HTMLSelectElement).value).toBe('61');
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const [id, payload] = updateBankAccountMock.mock.calls[0] as [number, Record<string, unknown>];
		expect(id).toBe(2);
		expect(payload.paypal).toEqual({ account: 'PayPal www', feeAccountId: 61 });
		expect(payload.iban).toBeNull();
		expect(payload.isPrimary).toBe(false);
	});

	// Revue P2 (L3) — un compte de frais archivé ou non imputable n'est pas vidé en silence à
	// l'ouverture de la modification : le serveur le juge, le champ ne change pas.
	it('modification : un compte de frais archivé reste envoyé, non vidé', async () => {
		fetchAccountsMock.mockResolvedValue([
			BANQUE, PAYPAL_CHF, PAYPAL_EUR, FRAIS_CHF, { ...FRAIS_EUR, active: false }, FRAIS_USD, PRODUIT,
		]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		await r.findByTestId('edit-is-paypal');
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = updateBankAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect(payload.paypal).toEqual({ account: 'PayPal www', feeAccountId: 61 });
	});

	it('modification : décochée sur un compte PayPal → `paypal: null`', async () => {
		listBankAccountsMock.mockResolvedValue([
			bank({ ...paypalEur, journalAccountId: PAYPAL_CHF.id, iban: 'CH9300762011623852957' }),
		]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		await fireEvent.click(await r.findByTestId('edit-is-paypal'));
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = updateBankAccountMock.mock.calls[0][1] as Record<string, unknown>;
		expect('paypal' in payload).toBe(true);
		expect(payload.paypal).toBeNull();
	});

	it('modification d’un compte ordinaire : `paypal` absent (inchangé)', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-3'));
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		expect('paypal' in (updateBankAccountMock.mock.calls[0][1] as object)).toBe(false);
	});

	it('compte principal : la case est désactivée, avec son motif', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-1'));
		expect(((await r.findByTestId('edit-is-paypal')) as HTMLInputElement).disabled).toBe(true);
		expect((await r.findByTestId('edit-is-paypal-locked')).textContent).toContain('compte principal');
	});

	it('liste : « PayPal · <compte> · <devise> » sous le nom, pas de « Lier » sur un compte PayPal', async () => {
		const r = render(Page);
		expect((await r.findByTestId('bank-account-paypal-2')).textContent).toBe('PayPal · PayPal www · EUR');
		expect(r.queryByTestId('bank-account-paypal-3')).toBeNull();
		expect(r.queryByTestId('link-button-2')).toBeNull();
		expect(r.queryByTestId('link-button-3')).not.toBeNull();
	});
});
