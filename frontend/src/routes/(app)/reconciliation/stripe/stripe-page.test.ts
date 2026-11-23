// Story 40-7d (AC-2, AC-3) — la page Stripe, sur la page commune des prestataires :
// aucun compte, rappel des ventes, import et compte rendu (devises, versements), refus du
// fichier sans source_id, propositions (genres, action), acceptation (corps du versement),
// écarts (charge sans vente, vente sans charge, versement sans réception et ses liens),
// à examiner, états vides. Story 40-8b : le bouton « Télécharger depuis Stripe » et son dialogue.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import type { StripeReview } from '$lib/features/stripe/stripe.types';

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
		{ id: 5, companyId: 1, name: '2020', startDate: '2020-01-01', endDate: '2020-12-31', status: 'Open', multiCurrency: true },
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

import { authState } from '$lib/app/stores/auth.svelte';
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
		wise: null,
		stripe: { account, feeAccountId: 60, payoutBankAccountId: 9 },
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

function tx(id: number, date: string, extra: Record<string, unknown> = {}) {
	return {
		id,
		bankAccountId: 1,
		date,
		amount: '100.00',
		currency: 'CHF',
		providerType: 'charge',
		name: null,
		transactionId: `txn_TEST000${id}`,
		counterpartyEmail: null,
		...extra,
	};
}

function review(): StripeReview {
	return {
		account: 'Stripe boutique',
		fiscalYearId: 5,
		proposals: [
			{
				type: 'link_entry',
				bankTransactionId: 1,
				journalEntryId: 40,
				transaction: tx(1, '2020-01-29'),
				entry: { journalEntryId: 40, entryNumber: 12, entryDate: '2020-01-29', description: 'Vente ch_TEST0001', ids: ['ch_TEST0001'] },
			},
			{ type: 'fee', bankTransactionId: 2, feeAccountId: 60, transaction: tx(2, '2020-01-29', { amount: '-3.20' }) },
			{
				type: 'payout',
				bankTransactionId: 3,
				payoutBankAccountId: 9,
				payoutAccountLabel: 'Banque · Principal',
				transaction: tx(3, '2020-02-03', { amount: '-125.15', providerType: 'payout', transactionId: 'po_TEST0001' }),
			},
		],
		gaps: [
			{ code: 'receipt_without_sale', transaction: tx(4, '2020-02-06', { currency: 'EUR', amount: '80.00' }) },
			{
				code: 'sale_without_receipt',
				entries: [{ journalEntryId: 41, entryNumber: 13, entryDate: '2020-02-07', description: 'Vente ch_TEST0050', ids: ['ch_TEST0050'] }],
			},
			{
				code: 'payout_without_receipt',
				transaction: tx(8, '2020-02-10', { amount: '-270.30', currency: 'EUR', providerType: 'payout', transactionId: 'po_TEST0002' }),
				details: { payoutId: 'po_TEST0002', journalEntryId: 77, payoutBankAccountId: 9 },
			},
		],
		toReview: [
			{ code: 'refund', transaction: tx(5, '2020-01-31', { amount: '-20.00', providerType: 'refund' }), details: { category: 'refund' } },
			{ code: 'payout_account_missing', transaction: tx(6, '2020-02-10', { providerType: 'payout' }), details: { payoutId: 'po_TEST0002' } },
		],
	};
}

const emptyReview = (): StripeReview => ({ account: 'Stripe boutique', fiscalYearId: 5, proposals: [], gaps: [], toReview: [] });

describe('page Stripe (Story 40-7d)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([stripeAccount(1, 'Stripe boutique'), stripeAccount(2, 'Stripe boutique')]);
		reviewMock.mockResolvedValue(review());
		acceptMock.mockResolvedValue({ accepted: [{ type: 'payout', bankTransactionIds: [3], journalEntryIds: [9] }], failed: [] });
	});

	it('sans compte Stripe : le message et le lien vers les comptes bancaires ; un compte PayPal ne compte pas', async () => {
		listBankAccountsMock.mockResolvedValue([
			{ ...stripeAccount(1, 'x'), provider: 'paypal', stripe: null, paypal: { account: 'PayPal www', feeAccountId: null } },
		]);
		const r = render(Page);
		expect((await r.findByTestId('stripe-no-account')).textContent).toContain('Aucun compte Stripe');
		expect(r.getByTestId('stripe-no-account-link').getAttribute('href')).toBe('/bank-accounts');
		expect(reviewMock).not.toHaveBeenCalled();
	});

	it('revue : un libellé par genre, ce qui sera fait, écarts et motifs libellés pour Stripe', async () => {
		const r = render(Page);
		expect((await r.findByTestId('stripe-account')) as HTMLSelectElement).toBeTruthy();
		expect(reviewMock).toHaveBeenCalledWith('Stripe boutique', 5);
		expect(r.getByTestId('stripe-hint').textContent).toContain('importez-les avant le rapport Stripe');
		await waitFor(() => expect(r.getByTestId('stripe-proposal-1')).toBeTruthy());
		expect(r.getByTestId('stripe-proposal-1').textContent).toContain('Rattachement à la vente n° 12');
		expect(r.getByTestId('stripe-proposal-2').textContent).toContain('6940 — Frais Stripe');
		expect(r.getByTestId('stripe-proposal-3').textContent).toContain('Versement vers Banque · Principal');
		expect(r.getByTestId('stripe-gap-receipt_without_sale').textContent).toContain('Charge sans vente');
		expect(r.getByTestId('stripe-gap-sale_without_receipt').textContent).toContain('Vente sans charge');
		expect(r.getByTestId('stripe-gap-sale_without_receipt').textContent).toContain('ch_TEST0050');
		const sansReception = r.getByTestId('stripe-gap-payout_without_receipt');
		expect(sansReception.textContent).toContain('Versement sans réception');
		const liens = Array.from(sansReception.querySelectorAll('[data-testid="stripe-gap-receipt-links"] a')).map((a) =>
			a.getAttribute('href'),
		);
		expect(liens).toEqual(['/journal-entries/77', '/reconciliation']);
		expect(r.queryAllByTestId('stripe-gap-receipt-links')).toHaveLength(1);
		expect(r.getByTestId('stripe-to-review-5').textContent).toContain('Remboursement');
		expect(r.getByTestId('stripe-to-review-6').textContent).toContain('compte de versement');
		expect(r.getByTestId('stripe-to-review-6').textContent).toContain('txn_TEST0006');
		expect(document.body.textContent).not.toMatch(/automatiquement/i);
	});

	// Revue P1 de 40-7d (M3) — un versement vers un compte Wise : la réconciliation bancaire
	// n'offre pas ce compte, le renvoi dit d'importer son relevé ; vers une banque ordinaire,
	// le lien vers la réconciliation reste.
	it('versement sans réception : lien vers la réconciliation pour une banque, vers la page Wise pour un compte Wise', async () => {
		const destination = (provider: 'wise' | null) => ({
			...stripeAccount(9, 'x'),
			bankName: provider === 'wise' ? 'Wise' : 'Banque',
			provider,
			wise: null,
			stripe: null,
			iban: provider === 'wise' ? null : 'CH9300762011623852957',
		});
		listBankAccountsMock.mockResolvedValue([stripeAccount(1, 'Stripe boutique'), destination('wise')]);
		const wise = render(Page);
		await waitFor(() => expect(wise.getByTestId('stripe-gap-payout_without_receipt')).toBeTruthy());
		const bloc = wise.getByTestId('stripe-gap-payout_without_receipt');
		expect(bloc.querySelector('[data-testid="stripe-gap-receipt-import"]')?.textContent).toContain('Importez le relevé');
		// Story 39-11d — le texte mène à la page Wise, où le relevé du compte de destination s'importe.
		expect(Array.from(bloc.querySelectorAll('a')).map((a) => a.getAttribute('href'))).toEqual(['/journal-entries/77', '/reconciliation/wise']);
		wise.unmount();
		listBankAccountsMock.mockResolvedValue([stripeAccount(1, 'Stripe boutique'), destination(null)]);
		const banque = render(Page);
		await waitFor(() => expect(banque.getByTestId('stripe-gap-payout_without_receipt')).toBeTruthy());
		const bloc2 = banque.getByTestId('stripe-gap-payout_without_receipt');
		expect(bloc2.querySelector('[data-testid="stripe-gap-receipt-import"]')).toBeNull();
		expect(Array.from(bloc2.querySelectorAll('a')).map((a) => a.getAttribute('href'))).toEqual(['/journal-entries/77', '/reconciliation']);
	});

	// Revue P2 de 40-7d (P2-3) — une destination Wise archivée ou close n'est pas offerte par la
	// réconciliation non plus : la résolution relit aussi les comptes archivés et clos.
	it('versement vers un compte Wise archivé : la page Wise, non la réconciliation (comptes archivés et clos relus)', async () => {
		const wiseArchive = { ...stripeAccount(9, 'x'), bankName: 'Wise', provider: 'wise' as const, stripe: null, iban: null, archived: true };
		listBankAccountsMock.mockImplementation(async (archived) =>
			archived ? [stripeAccount(1, 'Stripe boutique'), wiseArchive] : [stripeAccount(1, 'Stripe boutique')],
		);
		const r = render(Page);
		await waitFor(() => expect(r.getByTestId('stripe-gap-payout_without_receipt')).toBeTruthy());
		const bloc = r.getByTestId('stripe-gap-payout_without_receipt');
		await waitFor(() => expect(bloc.querySelector('[data-testid="stripe-gap-receipt-import"]')).not.toBeNull());
		// Story 39-11d — le texte mène à la page Wise, où le relevé du compte de destination s'importe.
		expect(Array.from(bloc.querySelectorAll('a')).map((a) => a.getAttribute('href'))).toEqual(['/journal-entries/77', '/reconciliation/wise']);
		expect(listBankAccountsMock).toHaveBeenCalledWith(true, true);
	});

	it('accepter la sélection : corps par genre, versement avec son compte de versement, ordre des dates', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('stripe-select-all'));
		await fireEvent.click(r.getByTestId('stripe-accept'));
		await waitFor(() => expect(acceptMock).toHaveBeenCalledTimes(1));
		const [account, bodies] = acceptMock.mock.calls[0];
		expect(account).toBe('Stripe boutique');
		expect(bodies).toEqual([
			{ type: 'link_entry', bankTransactionId: 1, journalEntryId: 40 },
			{ type: 'fee', bankTransactionId: 2, feeAccountId: 60 },
			{ type: 'payout', bankTransactionId: 3, payoutBankAccountId: 9 },
		]);
		expect((await r.findByTestId('stripe-accept-report')).textContent).toContain('1 proposition(s) acceptée(s), 0 refusée(s)');
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(2));
	});

	it('refus de l’acceptation : la proposition périmée par la clé commune', async () => {
		acceptMock.mockResolvedValue({
			accepted: [],
			failed: [{ bankTransactionId: 3, errorCode: 'PROVIDER_PROPOSAL_STALE', details: { reason: 'provider_settings_changed' } }],
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('stripe-check-3'));
		await fireEvent.click(r.getByTestId('stripe-accept'));
		expect((await r.findByTestId('stripe-accept-failed')).textContent).toContain('n\'est plus valable');
	});

	it('import : compte rendu par devise et versements lus, puis la revue relue', async () => {
		importMock.mockResolvedValue({
			account: 'Stripe boutique',
			currencies: [{ currency: 'CHF', bankAccountId: 1, importId: 4, inserted: 8, duplicates: 0 }],
			payouts: [{ payoutId: 'po_TEST0001', currency: 'CHF', lines: 4, net: '125.15' }],
		});
		const r = render(Page);
		const input = (await r.findByTestId('stripe-file-input')) as HTMLInputElement;
		expect(input.getAttribute('accept')).toBe('.csv,text/csv');
		const file = new File(['x'], 'stripe.csv', { type: 'text/csv' });
		await fireEvent.change(input, { target: { files: [file] } });
		await waitFor(() => expect(importMock).toHaveBeenCalledWith(file, 'Stripe boutique'));
		expect((await r.findByTestId('stripe-import-report-CHF')).textContent).toContain('8 transaction(s) insérée(s), 0 doublon(s)');
		expect(r.getByTestId('stripe-import-report-payout-po_TEST0001').textContent).toContain('4 ligne(s), net 125.15 CHF');
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(2));
	});

	it('refus du fichier sans source_id : la marche à suivre dans Stripe', async () => {
		importMock.mockRejectedValue({
			code: 'STRIPE_CSV_INVALID',
			status: 422,
			message: 'refus',
			details: { reason: 'STRIPE_CSV_MISSING_COLUMNS', columns: ['source_id|charge_id'] },
		});
		const r = render(Page);
		const input = (await r.findByTestId('stripe-file-input')) as HTMLInputElement;
		await fireEvent.change(input, { target: { files: [new File(['x'], 'a.csv')] } });
		expect((await r.findByTestId('stripe-import-error')).textContent).toContain('ajoutez la colonne source_id');
		// Story 40-9 — un refus de format ne propose pas le lien « Comptes bancaires ».
		expect(r.queryByTestId('stripe-import-error-accounts-link')).toBeNull();
	});

	it('refus du compte (aucun compte pour une devise) : le lien « Comptes bancaires » (Story 40-9)', async () => {
		importMock.mockRejectedValue({
			code: 'PROVIDER_CURRENCY_ACCOUNT_MISSING',
			status: 422,
			message: 'Aucun compte bancaire pour la devise EUR.',
			details: { account: 'Stripe boutique', currencies: ['EUR'] },
		});
		const r = render(Page);
		const input = (await r.findByTestId('stripe-file-input')) as HTMLInputElement;
		await fireEvent.change(input, { target: { files: [new File(['x'], 'a.csv')] } });
		await r.findByTestId('stripe-import-error');
		expect(r.getByTestId('stripe-import-error-accounts-link').getAttribute('href')).toBe('/bank-accounts');
	});

	it('refus du compte (compte du fournisseur introuvable) : le lien « Comptes bancaires » (Story 40-9, AC-4)', async () => {
		importMock.mockRejectedValue({
			code: 'PROVIDER_ACCOUNT_NOT_FOUND',
			status: 404,
			message: 'Compte introuvable.',
		});
		const r = render(Page);
		const input = (await r.findByTestId('stripe-file-input')) as HTMLInputElement;
		await fireEvent.change(input, { target: { files: [new File(['x'], 'a.csv')] } });
		await r.findByTestId('stripe-import-error');
		expect(r.getByTestId('stripe-import-error-accounts-link').getAttribute('href')).toBe('/bank-accounts');
	});

	it('états vides : aucune proposition, aucun écart, rien à examiner', async () => {
		reviewMock.mockResolvedValue(emptyReview());
		const r = render(Page);
		expect((await r.findByTestId('stripe-proposals-empty')).textContent).toContain('Aucune proposition');
		expect(r.getByTestId('stripe-gaps-empty').textContent).toContain('Aucun écart');
		expect(r.getByTestId('stripe-to-review-empty').textContent).toContain('Rien à examiner');
	});
});

describe('page Stripe — téléchargement par l’API (Story 40-8b, AC-4)', () => {
	const configured = (mode?: 'live' | 'sandbox') => ({
		...stripeAccount(1, 'Stripe boutique'),
		providerApi: mode ? { configured: true, mode } : { configured: true },
	});
	const notConfigured = () => ({ ...stripeAccount(1, 'Stripe boutique'), providerApi: { configured: false } });

	beforeEach(() => {
		vi.clearAllMocks();
		reviewMock.mockResolvedValue(emptyReview());
	});

	async function as(role: string) {
		await authState.logout();
		authState.login({ userId: '1', username: 'u', role, expiresIn: 3600 });
	}

	it('compte configuré, Comptable : le bouton, le dialogue par versement, le compte rendu et les versements manuels, puis la revue relue', async () => {
		await as('Comptable');
		listBankAccountsMock.mockResolvedValue([configured('live')]);
		startDownloadMock.mockResolvedValue({ jobId: 3 });
		getDownloadMock
			.mockResolvedValueOnce({ state: 'running', steps: { done: 1, total: 3 }, pausedUntilSeconds: null, result: null, error: null })
			.mockResolvedValue({
				state: 'done',
				steps: { done: 3, total: 3 },
				pausedUntilSeconds: null,
				result: {
					account: 'Stripe boutique',
					currencies: [{ currency: 'CHF', bankAccountId: 1, importId: 4, inserted: 6, duplicates: 0 }],
					payouts: [{ payoutId: 'po_TEST0001', currency: 'CHF', lines: 4, net: '125.15' }],
					manualPayouts: ['po_TEST0005'],
				},
				error: null,
			});
		const r = render(Page);
		const button = await r.findByTestId('stripe-download-button');
		expect(button.textContent).toContain('Télécharger depuis Stripe');
		expect(r.queryByTestId('stripe-download-sandbox')).toBeNull();
		expect(r.queryByTestId('stripe-download-not-configured')).toBeNull();
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(1));
		await fireEvent.click(button);
		const from = (await r.findByTestId('provider-download-from')) as HTMLInputElement;
		expect(from.value).toBe('2020-01-01');
		expect((r.getByTestId('provider-download-to') as HTMLInputElement).value).toBe('2020-12-31');
		expect(r.getByTestId('provider-download-notice').textContent).toContain('Les versements manuels ne se téléchargent pas');
		await fireEvent.click(r.getByTestId('provider-download-start'));
		await waitFor(() =>
			expect(startDownloadMock).toHaveBeenCalledWith('stripe', { account: 'Stripe boutique', from: '2020-01-01', to: '2020-12-31' }),
		);
		expect((await r.findByTestId('provider-download-progress')).textContent).toContain('versement 1/3');
		expect((await r.findByTestId('stripe-download-report-CHF', {}, { timeout: 3000 })).textContent).toContain(
			'6 transaction(s) insérée(s)',
		);
		expect(r.getByTestId('stripe-download-report-payout-po_TEST0001').textContent).toContain('4 ligne(s), net 125.15 CHF');
		expect(r.getByTestId('stripe-download-report-manual').textContent).toContain('po_TEST0005');
		expect(getDownloadMock).toHaveBeenCalledWith('stripe', 3);
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(2));
	});

	it('clé de test (bac à sable) : la mention, sur la page et dans le dialogue', async () => {
		await as('Comptable');
		listBankAccountsMock.mockResolvedValue([configured('sandbox')]);
		const r = render(Page);
		expect((await r.findByTestId('stripe-download-sandbox')).textContent).toContain('bac à sable');
		await fireEvent.click(r.getByTestId('stripe-download-button'));
		expect((await r.findByTestId('provider-download-sandbox')).textContent).toContain('bac à sable');
	});

	it('non configuré : pas de bouton ; la ligne d’aide pour l’Administrateur seulement', async () => {
		await as('Admin');
		listBankAccountsMock.mockResolvedValue([notConfigured()]);
		const r = render(Page);
		expect((await r.findByTestId('stripe-download-not-configured')).textContent).toContain('manuel d’administration');
		expect(r.queryByTestId('stripe-download-button')).toBeNull();
		expect(r.getByTestId('stripe-import-button')).toBeTruthy();
		r.unmount();

		await as('Comptable');
		const r2 = render(Page);
		await r2.findByTestId('stripe-import-button');
		await waitFor(() => expect(reviewMock).toHaveBeenCalled());
		expect(r2.queryByTestId('stripe-download-not-configured')).toBeNull();
		expect(r2.queryByTestId('stripe-download-button')).toBeNull();
	});

	it('Consultation : jamais le bouton, même configuré', async () => {
		await as('Consultation');
		listBankAccountsMock.mockResolvedValue([configured('live')]);
		const r = render(Page);
		await r.findByTestId('stripe-import-button');
		expect(r.queryByTestId('stripe-download-button')).toBeNull();
		await authState.logout();
	});
});
