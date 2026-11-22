// Story 38-11d (AC-3 à AC-5) — la page PayPal : aucun compte, import et compte rendu,
// refus traduits, propositions à cocher, retrait incomplet, acceptation par paquets,
// écarts, à examiner et rattachement par la modale.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import type { PaypalReview } from '$lib/features/paypal/paypal.types';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

const listBankAccountsMock = vi.fn<() => Promise<BankAccountSummary[]>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: () => listBankAccountsMock(),
}));
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({
	listFiscalYears: async () => [
		{ id: 5, companyId: 1, name: '2020', startDate: '2020-01-01', endDate: '2020-12-31', status: 'Open', multiCurrency: true },
	],
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: async () => [
		{ id: 20, number: '1020', name: 'Banque', accountType: 'Asset', active: true, postable: true },
		{ id: 31, number: '1053', name: 'Solde PayPal', accountType: 'Asset', active: true, postable: true },
		{ id: 60, number: '6940', name: 'Frais PayPal', accountType: 'Expense', active: true, postable: true },
	],
}));
vi.mock('$lib/features/reconciliation/rules/rules.api', () => ({
	listRules: async () => ({ items: [{ id: 3, label: 'Hébergement' }], total: 1, limit: 200, offset: 0 }),
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
vi.mock('$lib/features/paypal/paypal.api', async (importOriginal) => {
	const real = await importOriginal<typeof import('$lib/features/paypal/paypal.api')>();
	return {
		...real,
		importPaypalCsv: (...a: unknown[]) => importMock(...a),
		getPaypalReview: (...a: unknown[]) => reviewMock(...a),
		acceptPaypalProposals: (...a: unknown[]) => acceptMock(...a),
	};
});

import * as reconciliationApi from '$lib/features/reconciliation/reconciliation.api';
import Page from './+page.svelte';

function paypalAccount(id: number, account: string): BankAccountSummary {
	return {
		id,
		bankName: 'PayPal',
		description: null,
		iban: null,
		qrIban: null,
		accountKind: 'bank',
		cardLast4: null,
		provider: 'paypal',
		paypal: { account, feeAccountId: 60 },
		stripe: null,
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
		amount: '10.00',
		currency: 'EUR',
		providerType: 'Website Payment',
		name: 'Client fictif',
		transactionId: `0TESTTXN00000${id}`,
		counterpartyEmail: null,
		...extra,
	};
}

function review(): PaypalReview {
	return {
		account: 'PayPal www',
		fiscalYearId: 5,
		proposals: [
			{ type: 'link_entry', bankTransactionId: 1, journalEntryId: 40, transaction: tx(1, '2020-03-05'), entry: { journalEntryId: 40, entryNumber: 12, entryDate: '2020-03-04', description: 'Vente', ids: ['X'] } },
			{ type: 'fee', bankTransactionId: 2, feeAccountId: 60, transaction: tx(2, '2020-03-05', { amount: '-0.65' }) },
			{ type: 'withdrawal', bankTransactionId: 3, counterpartyAccountId: null, transaction: tx(3, '2020-03-06', { currency: 'CHF' }) },
			{ type: 'rule', bankTransactionId: 6, ruleId: 3, counterpartyAccountId: 60, transaction: tx(6, '2020-03-09') },
		],
		gaps: [
			{ code: 'receipt_without_sale', transaction: tx(4, '2020-03-07') },
			{ code: 'amount_or_currency_mismatch', transaction: tx(7, '2020-03-07'), entries: [{ journalEntryId: 41, entryNumber: 13, entryDate: '2020-03-07', description: 'Vente', ids: ['Y'] }], details: { expectedAmount: '10.00', expectedCurrency: 'EUR', foundAmount: '9.00', foundCurrency: 'EUR' } },
		],
		toReview: [
			{
				code: 'no_rule',
				transaction: tx(5, '2020-03-08', { providerType: 'General Payment', counterpartyEmail: 'client@example.invalid' }),
			},
		],
	};
}

describe('page PayPal (Story 38-11d)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([paypalAccount(1, 'PayPal www'), paypalAccount(2, 'PayPal www')]);
		reviewMock.mockResolvedValue(review());
		acceptMock.mockResolvedValue({ accepted: [{ type: 'fee', bankTransactionIds: [2], journalEntryIds: [9] }], failed: [] });
	});

	it('sans compte PayPal : le message et le lien vers les comptes bancaires', async () => {
		listBankAccountsMock.mockResolvedValue([]);
		const r = render(Page);
		expect((await r.findByTestId('paypal-no-account')).textContent).toContain('Aucun compte PayPal');
		expect(r.getByTestId('paypal-no-account-link').getAttribute('href')).toBe('/bank-accounts');
		expect(reviewMock).not.toHaveBeenCalled();
	});

	it('libellés distincts, exercice courant ; la revue est lue pour ce couple', async () => {
		const r = render(Page);
		const select = (await r.findByTestId('paypal-account')) as HTMLSelectElement;
		expect(Array.from(select.options).map((o) => o.value)).toEqual(['PayPal www']);
		await waitFor(() => expect(reviewMock).toHaveBeenCalledWith('PayPal www', 5));
		expect(await r.findByTestId('paypal-proposal-1')).toBeTruthy();
		expect(r.getByTestId('paypal-gap-receipt_without_sale').textContent).toContain('Encaissement sans vente');
		expect(r.getByTestId('paypal-gap-details').textContent).toContain('Attendu 10.00 EUR, trouvé 9.00 EUR');
		expect(r.getByTestId('paypal-proposal-6').textContent).toContain('Règle Hébergement');
		expect(r.getByTestId('paypal-to-review-5').textContent).toContain('Aucune règle');
		expect(r.getByTestId('paypal-to-review-5').textContent).toContain('client@example.invalid');
	});

	it('import : compte rendu par devise, écartées, puis revue relue', async () => {
		importMock.mockResolvedValue({
			account: 'PayPal www',
			currencies: [{ currency: 'EUR', bankAccountId: 1, importId: 3, inserted: 8, duplicates: 1 }],
			skipped: { notCompleted: [{ transactionId: '0TESTX', type: 'Website Payment', status: 'Pending' }], memo: 2, neutralPairs: 1 },
		});
		const r = render(Page);
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(1));
		const input = (await r.findByTestId('paypal-file-input')) as HTMLInputElement;
		const file = new File(['x'], 'paypal.csv', { type: 'text/csv' });
		await fireEvent.change(input, { target: { files: [file] } });
		await waitFor(() => expect(importMock).toHaveBeenCalledWith(file, 'PayPal www'));
		expect((await r.findByTestId('paypal-import-report-EUR')).textContent).toContain('8 transaction(s) insérée(s), 1 doublon(s)');
		expect(r.getByTestId('paypal-import-report-skipped').textContent).toContain('2 ligne(s) Memo');
		expect(r.getByTestId('paypal-import-report-not-completed').textContent).toContain('Pending');
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(2));
	});

	it('import refusé : une phrase par cause', async () => {
		importMock.mockRejectedValue({
			code: 'PAYPAL_CSV_INVALID',
			status: 422,
			message: 'x',
			details: { reason: 'PAYPAL_CSV_MISSING_COLUMNS', columns: ['Gross'] },
		});
		const r = render(Page);
		const input = (await r.findByTestId('paypal-file-input')) as HTMLInputElement;
		await fireEvent.change(input, { target: { files: [new File(['x'], 'a.csv')] } });
		expect((await r.findByTestId('paypal-import-error')).textContent).toContain('Colonnes manquantes : Gross');
	});

	it('retrait sans compte : non cochable, « Tout cocher » l’omet ; accepté une fois choisi', async () => {
		const r = render(Page);
		const retrait = (await r.findByTestId('paypal-check-3')) as HTMLInputElement;
		expect(retrait.disabled).toBe(true);
		await fireEvent.click(r.getByTestId('paypal-select-all'));
		expect((r.getByTestId('paypal-check-1') as HTMLInputElement).checked).toBe(true);
		expect(retrait.checked).toBe(false);
		await fireEvent.click(r.getByTestId('paypal-check-6'));
		await fireEvent.change(r.getByTestId('paypal-withdrawal-3'), { target: { value: '20' } });
		expect(retrait.disabled).toBe(false);
		await fireEvent.click(retrait);
		await fireEvent.click(r.getByTestId('paypal-accept'));
		await waitFor(() => expect(acceptMock).toHaveBeenCalledTimes(1));
		const [account, bodies] = acceptMock.mock.calls[0];
		expect(account).toBe('PayPal www');
		expect(bodies).toEqual([
			{ type: 'link_entry', bankTransactionId: 1, journalEntryId: 40 },
			{ type: 'fee', bankTransactionId: 2, feeAccountId: 60 },
			{ type: 'withdrawal', bankTransactionId: 3, counterpartyAccountId: 20 },
		]);
		expect((await r.findByTestId('paypal-accept-report')).textContent).toContain('1 proposition(s) acceptée(s), 0 refusée(s)');
	});

	it('refus de l’acceptation : chacun avec son message', async () => {
		acceptMock.mockResolvedValue({
			accepted: [],
			failed: [{ bankTransactionId: 1, errorCode: 'PROVIDER_PROPOSAL_STALE', details: null }],
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('paypal-check-1'));
		await fireEvent.click(r.getByTestId('paypal-accept'));
		expect((await r.findByTestId('paypal-accept-failed')).textContent).toContain('La proposition ne vaut plus');
	});

	it('plus de 500 propositions : un paquet à la fois, compte rendu cumulé', async () => {
		const many = review();
		many.proposals = Array.from({ length: 501 }, (_, i) => ({
			type: 'fee' as const,
			bankTransactionId: 100 + i,
			feeAccountId: 60,
			transaction: tx(100 + i, '2020-03-05'),
		}));
		reviewMock.mockResolvedValue(many);
		acceptMock.mockImplementation(async (_a: string, bodies: unknown[]) => ({
			accepted: bodies.map(() => ({ type: 'fee', bankTransactionIds: [], journalEntryIds: [] })),
			failed: [],
		}));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('paypal-select-all'));
		await fireEvent.click(r.getByTestId('paypal-accept'));
		await waitFor(() => expect(acceptMock).toHaveBeenCalledTimes(2));
		expect(acceptMock.mock.calls[0][1]).toHaveLength(500);
		expect(acceptMock.mock.calls[1][1]).toHaveLength(1);
		expect((await r.findByTestId('paypal-accept-report')).textContent).toContain('501 proposition(s) acceptée(s)');
	});

	it('à examiner : « Rattacher à une écriture » ouvre la modale sur la transaction, en devise', async () => {
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('paypal-link-5'));
		await waitFor(() => expect(reconciliationApi.getEntryCandidates).toHaveBeenCalledWith(5));
		expect(reconciliationApi.getLinkCandidates).not.toHaveBeenCalled();
		expect((await r.findByTestId('link-settlement-modal')).textContent).toContain('EUR');
	});
});

// Revue P1 (L1, L2) — changement de compte, garde de génération, erreur au paquet 2,
// relecture après l'acceptation, sélecteurs désactivés, retrait effacé.
describe('page PayPal — robustesse (revue P1)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([paypalAccount(1, 'PayPal www'), paypalAccount(2, 'PayPal boutique')]);
		reviewMock.mockResolvedValue(review());
		acceptMock.mockResolvedValue({ accepted: [], failed: [] });
	});

	it('changer de compte relit la revue pour ce compte', async () => {
		const r = render(Page);
		await waitFor(() => expect(reviewMock).toHaveBeenCalledWith('PayPal boutique', 5));
		await fireEvent.change(r.getByTestId('paypal-account'), { target: { value: 'PayPal www' } });
		await waitFor(() => expect(reviewMock).toHaveBeenLastCalledWith('PayPal www', 5));
	});

	it('une réponse périmée (compte quitté) ne remplace pas la revue courante', async () => {
		let resolveFirst: (v: PaypalReview) => void = () => {};
		const stale = review();
		stale.proposals = [];
		reviewMock
			.mockImplementationOnce(() => new Promise<PaypalReview>((res) => (resolveFirst = res)))
			.mockResolvedValueOnce(review());
		const r = render(Page);
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(1));
		await fireEvent.change(r.getByTestId('paypal-account'), { target: { value: 'PayPal www' } });
		await waitFor(() => expect(r.getByTestId('paypal-proposal-1')).toBeTruthy());
		resolveFirst(stale);
		await new Promise((res) => setTimeout(res, 0));
		expect(r.getByTestId('paypal-proposal-1')).toBeTruthy();
		expect(r.queryByTestId('paypal-proposals-empty')).toBeNull();
	});

	// Revue P2 (L1) — pendant la relecture après un changement de compte, l'ancienne sélection
	// ne s'accepte pas : le bouton est grisé jusqu'à la réponse.
	it('changement de compte : « Accepter la sélection » grisé pendant la relecture', async () => {
		const r = render(Page);
		await waitFor(() => expect(r.getByTestId('paypal-proposal-1')).toBeTruthy());
		await fireEvent.click(r.getByTestId('paypal-check-2'));
		expect((r.getByTestId('paypal-accept') as HTMLButtonElement).disabled).toBe(false);
		let resolveNext: (v: PaypalReview) => void = () => {};
		reviewMock.mockImplementationOnce(() => new Promise<PaypalReview>((res) => (resolveNext = res)));
		await fireEvent.change(r.getByTestId('paypal-account'), { target: { value: 'PayPal www' } });
		await waitFor(() => expect((r.getByTestId('paypal-accept') as HTMLButtonElement).disabled).toBe(true));
		resolveNext(review());
		await waitFor(() => expect(r.getByTestId('paypal-proposal-2')).toBeTruthy());
	});

	// Revue P2 (L2) — après une acceptation, rien n'est coché d'office, même la proposition
	// refusée qui reparaît sous la même clé.
	it('après l’acceptation d’une proposition refusée : aucune case cochée', async () => {
		acceptMock.mockResolvedValue({
			accepted: [],
			failed: [{ bankTransactionId: 2, errorCode: 'PROVIDER_PROPOSAL_STALE', details: null }],
		});
		const r = render(Page);
		await waitFor(() => expect(r.getByTestId('paypal-proposal-2')).toBeTruthy());
		await fireEvent.click(r.getByTestId('paypal-check-2'));
		expect((r.getByTestId('paypal-check-2') as HTMLInputElement).checked).toBe(true);
		await fireEvent.click(r.getByTestId('paypal-accept'));
		await waitFor(() => expect(acceptMock).toHaveBeenCalledTimes(1));
		await r.findByTestId('paypal-accept-report');
		await waitFor(() => expect((r.getByTestId('paypal-check-2') as HTMLInputElement).checked).toBe(false));
	});

	it('erreur au paquet 2 : le paquet 1 compté, l’erreur dite, la revue relue', async () => {
		const many = review();
		many.proposals = Array.from({ length: 501 }, (_, i) => ({
			type: 'fee' as const,
			bankTransactionId: 100 + i,
			feeAccountId: 60,
			transaction: tx(100 + i, '2020-03-05'),
		}));
		reviewMock.mockResolvedValue(many);
		acceptMock
			.mockImplementationOnce(async (_a: string, bodies: unknown[]) => ({
				accepted: bodies.map(() => ({ type: 'fee', bankTransactionIds: [], journalEntryIds: [] })),
				failed: [],
			}))
			.mockRejectedValueOnce({ code: 'INTERNAL_ERROR', status: 500, message: 'Erreur du serveur' });
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('paypal-select-all'));
		const calls = reviewMock.mock.calls.length;
		await fireEvent.click(r.getByTestId('paypal-accept'));
		expect((await r.findByTestId('paypal-accept-error')).textContent).toContain('Erreur du serveur');
		expect(r.getByTestId('paypal-accept-report').textContent).toContain('500 proposition(s) acceptée(s)');
		await waitFor(() => expect(reviewMock.mock.calls.length).toBe(calls + 1));
	});

	it('pendant l’acceptation : sélecteurs et import désactivés, chaque paquet au compte de départ', async () => {
		let release: () => void = () => {};
		acceptMock.mockImplementationOnce(
			() => new Promise((res) => (release = () => res({ accepted: [], failed: [] }))),
		);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('paypal-check-1'));
		await fireEvent.click(r.getByTestId('paypal-accept'));
		await waitFor(() => expect((r.getByTestId('paypal-account') as HTMLSelectElement).disabled).toBe(true));
		expect((r.getByTestId('paypal-fiscal-year') as HTMLSelectElement).disabled).toBe(true);
		expect((r.getByTestId('paypal-import-button') as HTMLButtonElement).disabled).toBe(true);
		// Revue P4, P4-3 : les cases par ligne sont grisées aussi.
		expect((r.getByTestId('paypal-check-1') as HTMLInputElement).disabled).toBe(true);
		release();
		await waitFor(() => expect((r.getByTestId('paypal-account') as HTMLSelectElement).disabled).toBe(false));
		expect(acceptMock.mock.calls[0][0]).toBe('PayPal boutique');
	});

	it('retrait : le compte du plan du compte PayPal n’est pas offert (revue P3, L1)', async () => {
		const r = render(Page);
		const select = (await r.findByTestId('paypal-withdrawal-3')) as HTMLSelectElement;
		const values = [...select.options].map((o) => o.value);
		expect(values).toContain('20');
		expect(values).not.toContain('31');
	});

	it('pendant l’import : « Accepter la sélection » grisé ; pendant l’acceptation : « Tout cocher » grisé (revue P3, L2)', async () => {
		let releaseImport: () => void = () => {};
		importMock.mockImplementationOnce(
			() =>
				new Promise((res) => {
					releaseImport = () => res({ account: 'PayPal www', currencies: [], skipped: { notCompleted: [], memo: 0, neutralPairs: 0 } });
				}),
		);
		let releaseAccept: () => void = () => {};
		acceptMock.mockImplementationOnce(
			() => new Promise((res) => (releaseAccept = () => res({ accepted: [], failed: [] }))),
		);
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('paypal-check-1'));
		expect((r.getByTestId('paypal-accept') as HTMLButtonElement).disabled).toBe(false);
		const input = (await r.findByTestId('paypal-file-input')) as HTMLInputElement;
		await fireEvent.change(input, { target: { files: [new File(['x'], 'p.csv', { type: 'text/csv' })] } });
		await waitFor(() => expect((r.getByTestId('paypal-accept') as HTMLButtonElement).disabled).toBe(true));
		releaseImport();
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(2));
		await waitFor(() => expect((r.getByTestId('paypal-import-button') as HTMLButtonElement).disabled).toBe(false));
		const check = (await r.findByTestId('paypal-check-1')) as HTMLInputElement;
		if (!check.checked) await fireEvent.click(check);
		await waitFor(() => expect((r.getByTestId('paypal-accept') as HTMLButtonElement).disabled).toBe(false));
		await fireEvent.click(r.getByTestId('paypal-accept'));
		await waitFor(() => expect((r.getByTestId('paypal-select-all') as HTMLButtonElement).disabled).toBe(true));
		expect((r.getByTestId('paypal-select-none') as HTMLButtonElement).disabled).toBe(true);
		releaseAccept();
		await waitFor(() => expect((r.getByTestId('paypal-select-all') as HTMLButtonElement).disabled).toBe(false));
	});

	it('retrait : effacer le compte choisi décoche la proposition', async () => {
		const r = render(Page);
		const retrait = (await r.findByTestId('paypal-check-3')) as HTMLInputElement;
		await fireEvent.change(r.getByTestId('paypal-withdrawal-3'), { target: { value: '20' } });
		await fireEvent.click(retrait);
		expect(retrait.checked).toBe(true);
		await fireEvent.change(r.getByTestId('paypal-withdrawal-3'), { target: { value: '' } });
		expect(retrait.checked).toBe(false);
		expect(retrait.disabled).toBe(true);
	});

	it('écart d’une vente déjà rattachée : son libellé propre', async () => {
		const r1 = review();
		r1.gaps = [
			{
				code: 'amount_or_currency_mismatch',
				transaction: tx(7, '2020-03-07'),
				details: { expectedAmount: '10.00', expectedCurrency: 'EUR', foundAmount: '10.00', foundCurrency: 'EUR', alreadyLinked: true },
			},
		];
		reviewMock.mockResolvedValue(r1);
		const r = render(Page);
		const row = await r.findByTestId('paypal-gap-amount_or_currency_mismatch');
		expect(row.textContent).toContain('Vente déjà rattachée à une autre transaction');
		expect(row.textContent).not.toContain('Montant ou devise différents');
		expect(r.queryByTestId('paypal-gap-details')).toBeNull();
	});
});
