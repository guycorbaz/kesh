// Story 40-7d (AC-1) — le compte Stripe à l'écran des comptes bancaires : case « Compte
// Stripe » à côté de « Compte PayPal » (l'une exclut l'autre), libellé, compte de frais,
// compte bancaire de versement (même devise, ni Stripe ni PayPal, vidé seulement quand
// l'utilisateur change le compte du plan et que la destination n'y est plus admise ; archivée ou
// close, gardée et montrée tant que le compte du plan ne change pas), charges `stripe` (création,
// modification, décochée), ligne de la liste, pas de « Lier ».
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

const listBankAccountsMock = vi.fn<(includeArchived?: boolean, includeClosed?: boolean) => Promise<BankAccountSummary[]>>();
const createBankAccountMock = vi.fn<(payload: unknown) => Promise<unknown>>();
const updateBankAccountMock = vi.fn<(id: number, payload: unknown) => Promise<unknown>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: (includeArchived?: boolean, includeClosed?: boolean) =>
		listBankAccountsMock(includeArchived, includeClosed),
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
		bankName: 'Banque Zeta',
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

const principal = bank({ id: 1, isPrimary: true, bankName: 'Banque Zeta' });
const stripeEur = bank({
	id: 2,
	bankName: 'Stripe',
	iban: null,
	provider: 'stripe',
	stripe: { account: 'Stripe boutique', feeAccountId: FRAIS_EUR.id, payoutBankAccountId: 4 },
	journalAccountId: STRIPE_EUR.id,
	version: 4,
});
const ordinaire = bank({ id: 3, bankName: 'Banque Alpha', journalAccountId: BANQUE.id });
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

/**
 * Revue P3 de 40-7d (P3-1) — le serveur des comptes bancaires, qui respecte ses arguments comme
 * `GET /bank-accounts` : sans `includeArchived`, pas de compte archivé ; sans `includeClosed`, pas de
 * compte clos. Le mock d'avant rendait tout, comme si les deux cases de l'écran étaient cochées.
 */
function serve(all: BankAccountSummary[]) {
	listBankAccountsMock.mockImplementation(async (includeArchived, includeClosed) =>
		all.filter((b) => (includeArchived || !b.archived) && (includeClosed || b.closedOn === null)),
	);
}

describe('page des comptes bancaires — le compte Stripe (Story 40-7d, AC-1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		fetchAccountsMock.mockResolvedValue([BANQUE, STRIPE_CHF, STRIPE_EUR, WISE_EUR, FRAIS_CHF, FRAIS_EUR, PRODUIT]);
		serve([principal, stripeEur, ordinaire, wiseEur, paypalChf]);
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
		// CHF : les banques CHF (Banque Alpha, Banque Zeta), jamais le compte PayPal CHF ; le Wise EUR se vide.
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
		serve([
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

	it.each([
		['archivée', { archived: true }],
		['close', { closedOn: '2026-01-31' }],
	])('modification : une destination %s depuis reste choisie, marquée, et renvoyée au serveur', async (_nom, etat) => {
		serve([principal, stripeEur, ordinaire, { ...wiseEur, ...etat }]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		const payout = (await r.findByTestId('edit-payout-account')) as HTMLSelectElement;
		await waitFor(() => expect(payout.value).toBe('4'));
		expect(payout.options[payout.selectedIndex].textContent).toMatch(/Wise.*\((Archivé|Clos le)/);
		await fireEvent.click(await r.findByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = updateBankAccountMock.mock.calls[0][1] as { stripe: { payoutBankAccountId: number | null } };
		expect(payload.stripe.payoutBankAccountId).toBe(4);
	});

	it('modification : le changement de devise du compte du plan efface même une destination archivée', async () => {
		serve([principal, stripeEur, ordinaire, { ...wiseEur, archived: true }]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		const payout = (await r.findByTestId('edit-payout-account')) as HTMLSelectElement;
		await waitFor(() => expect(payout.value).toBe('4'));
		await fireEvent.change(r.getByTestId('edit-journal-account'), { target: { value: '21' } });
		await waitFor(() => expect((r.getByTestId('edit-payout-account') as HTMLSelectElement).selectedIndex).toBe(0));
	});

	// Revue P2 de 40-7d (P2-1) — le serveur rejuge la destination dès que le compte du plan change
	// (`payout_is_unchanged`) : archivée ou close, elle n'est plus offerte ; active, elle reste.
	it.each([
		// Colonnes dans l'ordre du titre (revue P3, P3-5) : nom, valeur attendue, puis l'état.
		['archivée', '', { archived: true }],
		['active', '4', {}],
	])('modification : compte du plan changé à même devise, destination %s → valeur « %s »', async (_nom, attendu, etat) => {
		const AUTRE_EUR = account({ id: 24, number: '1032', name: 'Stripe EUR bis', currency: 'EUR' });
		fetchAccountsMock.mockResolvedValue([BANQUE, STRIPE_CHF, STRIPE_EUR, WISE_EUR, AUTRE_EUR, FRAIS_CHF, FRAIS_EUR, PRODUIT]);
		serve([principal, stripeEur, ordinaire, { ...wiseEur, ...etat }]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		const payout = (await r.findByTestId('edit-payout-account')) as HTMLSelectElement;
		await waitFor(() => expect(payout.value).toBe('4'));
		await fireEvent.change(r.getByTestId('edit-journal-account'), { target: { value: '24' } });
		await waitFor(() => expect((r.getByTestId('edit-payout-account') as HTMLSelectElement).value).toBe(attendu));
	});

	// Revue P3 de 40-7d (P3-1) — vue par défaut (ni archivés ni clos) : la destination archivée ou
	// close n'est pas dans la liste de l'écran. Un simple renommage la renvoie telle quelle, en CHF
	// (l'ouverture ne change pas la devise) comme en EUR (elle la fait passer de CHF à EUR), et le
	// sélecteur la montre, marquée, au lieu de « Non configuré ».
	const stripeChf = bank({
		id: 6,
		bankName: 'Stripe',
		iban: null,
		provider: 'stripe',
		stripe: { account: 'Stripe CH', feeAccountId: FRAIS_CHF.id, payoutBankAccountId: 3 },
		journalAccountId: STRIPE_CHF.id,
		version: 2,
	});
	it.each([
		['CHF', 'archivée', 6, 3, [principal, stripeChf, { ...ordinaire, archived: true }], /Banque Alpha.*\(Archivé/],
		['CHF', 'close', 6, 3, [principal, stripeChf, { ...ordinaire, closedOn: '2026-01-31' }], /Banque Alpha.*\(Clos le/],
		['EUR', 'archivée', 2, 4, [principal, stripeEur, ordinaire, { ...wiseEur, archived: true }], /Wise.*\(Archivé/],
		['EUR', 'close', 2, 4, [principal, stripeEur, ordinaire, { ...wiseEur, closedOn: '2026-01-31' }], /Wise.*\(Clos le/],
	])(
		'renommage d’un compte Stripe %s, destination %s absente de la vue par défaut : gardée et montrée',
		async (_devise, _etat, id, destination, comptes, libelle) => {
			serve(comptes);
			const r = render(Page);
			await fireEvent.click(await r.findByTestId(`edit-button-${id}`));
			const payout = (await r.findByTestId('edit-payout-account')) as HTMLSelectElement;
			await waitFor(() => expect(payout.value).toBe(String(destination)));
			expect(payout.options[payout.selectedIndex].textContent).toMatch(libelle);
			await fill(r.getByTestId('edit-stripe-account'), 'Stripe renommé');
			await fireEvent.click(r.getByTestId('edit-form-submit'));
			await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
			const payload = updateBankAccountMock.mock.calls[0][1] as {
				stripe: { account: string; payoutBankAccountId: number | null };
			};
			expect(payload.stripe.account).toBe('Stripe renommé');
			expect(payload.stripe.payoutBankAccountId).toBe(destination);
		},
	);

	it('lecture des destinations en échec : la destination absente n’est pas vidée au renommage', async () => {
		listBankAccountsMock.mockImplementation(async (includeArchived, includeClosed) => {
			if (includeArchived && includeClosed) throw new Error('boom');
			return [principal, stripeEur, ordinaire];
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		await r.findByTestId('edit-payout-account');
		await fireEvent.click(r.getByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = updateBankAccountMock.mock.calls[0][1] as { stripe: { payoutBankAccountId: number | null } };
		expect(payload.stripe.payoutBankAccountId).toBe(4);
	});

	it('lecture des destinations en échec : changer le compte du plan ne vide pas une destination inconnue', async () => {
		listBankAccountsMock.mockImplementation(async (includeArchived, includeClosed) => {
			if (includeArchived && includeClosed) throw new Error('boom');
			return [principal, stripeEur, ordinaire];
		});
		const AUTRE_EUR = account({ id: 24, number: '1032', name: 'Stripe EUR bis', currency: 'EUR' });
		fetchAccountsMock.mockResolvedValue([BANQUE, STRIPE_CHF, STRIPE_EUR, WISE_EUR, AUTRE_EUR, FRAIS_CHF, FRAIS_EUR, PRODUIT]);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		await r.findByTestId('edit-payout-account');
		await fireEvent.change(r.getByTestId('edit-journal-account'), { target: { value: '24' } });
		await fireEvent.click(r.getByTestId('edit-form-submit'));
		await waitFor(() => expect(updateBankAccountMock).toHaveBeenCalledTimes(1));
		const payload = updateBankAccountMock.mock.calls[0][1] as { stripe: { payoutBankAccountId: number | null } };
		// Le serveur rejuge la destination (compte du plan changé) et refuse s'il le faut : le client ne
		// retire pas ce qu'il ne voit pas.
		expect(payload.stripe.payoutBankAccountId).toBe(4);
	});

	it('lecture des destinations en échec : la destination absente se nomme « introuvable », les candidates de l’écran restent (Story 40-9)', async () => {
		const wiseBis = bank({ id: 6, bankName: 'Wise bis', iban: null, provider: 'wise', journalAccountId: WISE_EUR.id });
		listBankAccountsMock.mockImplementation(async (includeArchived, includeClosed) => {
			if (includeArchived && includeClosed) throw new Error('boom');
			return [principal, stripeEur, ordinaire, wiseBis];
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		const payout = (await r.findByTestId('edit-payout-account')) as HTMLSelectElement;
		await waitFor(() => expect(payout.value).toBe('4'));
		expect(r.getByTestId('edit-payout-missing').textContent).toContain('Destination actuelle (introuvable)');
		expect((r.getByTestId('edit-payout-missing') as HTMLOptionElement).selected).toBe(true);
		// Le repli sur la liste de l'écran offre encore ses candidates (un repli vide ne les offrirait pas).
		expect(options(payout)).toContain('6');
	});

	it('destination lue : pas d’option « introuvable » (Story 40-9)', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('edit-button-2'));
		await waitFor(() => expect(((r.getByTestId('edit-payout-account')) as HTMLSelectElement).value).toBe('4'));
		expect(r.queryByTestId('edit-payout-missing')).toBeNull();
	});

	it('liste : « Stripe · <compte> · <devise> », pas de « Lier » sur un compte Stripe', async () => {
		const r = render(Page);
		expect((await r.findByTestId('bank-account-stripe-2')).textContent).toBe('Stripe · Stripe boutique · EUR');
		expect(r.queryByTestId('link-button-2')).toBeNull();
		expect(r.queryByTestId('link-button-3')).not.toBeNull();
	});
});
