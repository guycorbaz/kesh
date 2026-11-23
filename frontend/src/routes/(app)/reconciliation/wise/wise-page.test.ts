// Story 39-11d (AC-2) — la page Wise, sur la page commune des prestataires : aucun compte,
// consigne d'export à l'écran, choix multiple (zip ou CSV), compte rendu par fichier
// (archive, doublons, soldes, avertissement de continuité, entrées du zip non lues), refus
// traduits (frais non séparés, zip illisible, deux fichiers d'une devise). Story 39-11d2 —
// la revue (tests retournés : l'exercice et les propositions sont là) : genres et « ce qui sera
// fait », paiement par carte groupé et ses sous-lignes, phrases des motifs, acceptation.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import type { BankAccountSummary } from '$lib/features/bank-accounts/bank-accounts.api';
import type { WiseImportResponse } from '$lib/features/wise/wise.types';

vi.mock('$app/environment', () => ({ browser: true }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

const listBankAccountsMock = vi.fn<(archived?: boolean, closed?: boolean) => Promise<BankAccountSummary[]>>();
vi.mock('$lib/features/bank-accounts/bank-accounts.api', () => ({
	listBankAccounts: (archived?: boolean, closed?: boolean) => listBankAccountsMock(archived, closed),
}));
const listFiscalYearsMock = vi.fn(async () => [
	{ id: 5, companyId: 1, name: '2020', startDate: '2020-01-01', endDate: '2020-12-31', status: 'Open', multiCurrency: true },
]);
vi.mock('$lib/features/fiscal-years/fiscal-years.api', () => ({ listFiscalYears: () => listFiscalYearsMock() }));
vi.mock('$lib/features/reconciliation/reconciliation.api', () => ({
	getLinkCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	getEntryCandidates: vi.fn(async () => ({ bankTransactionId: 0, candidates: [] })),
	linkEntry: vi.fn(),
	linkTransaction: vi.fn(),
}));

const importMock = vi.fn();
const reviewMock = vi.fn();
const acceptMock = vi.fn();
vi.mock('$lib/features/wise/wise.api', () => ({
	importWiseFiles: (...a: unknown[]) => importMock(...a),
	getWiseReview: (...a: unknown[]) => reviewMock(...a),
	acceptWiseProposals: (...a: unknown[]) => acceptMock(...a),
}));
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: async () => [
		{ id: 60, number: '6947', name: 'Frais Wise' },
		{ id: 40, number: '4000', name: 'Achats' },
	],
}));
vi.mock('$lib/features/reconciliation/rules/rules.api', () => ({
	listRules: async () => ({ items: [{ id: 2, label: 'Boutique' }], total: 1 }),
}));

import Page from './+page.svelte';
import { getEntryCandidates } from '$lib/features/reconciliation/reconciliation.api';

function wiseAccount(id: number, account: string, journalAccountId = 31): BankAccountSummary {
	return {
		id,
		bankName: 'Wise',
		description: null,
		iban: null,
		qrIban: null,
		accountKind: 'bank',
		cardLast4: null,
		provider: 'wise',
		paypal: null,
		wise: { account, feeAccountId: 60 },
		stripe: null,
		isPrimary: false,
		journalAccountId,
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

function report(): WiseImportResponse {
	return {
		account: 'Wise Fictif',
		files: [
			{
				filename: 'wise-releve-chf-fictif.csv',
				archive: 'statement_csv.zip',
				currency: 'CHF',
				bankAccountId: 1,
				importId: 7,
				rows: 10,
				inserted: 10,
				duplicates: 0,
				openingBalance: '0.00',
				closingBalance: '1234.50',
				warnings: [],
			},
			{
				filename: 'wise-releve-eur-fictif.csv',
				archive: 'statement_csv.zip',
				currency: 'EUR',
				bankAccountId: 2,
				importId: null,
				rows: 5,
				inserted: 0,
				duplicates: 5,
				openingBalance: '10.00',
				closingBalance: '20.00',
				warnings: [{ code: 'gap', expected: '0.00', found: '10.00', previousImportId: 3 }],
			},
		],
		ignoredEntries: [{ archive: 'statement_csv.zip', entry: 'lisez-moi.pdf' }],
	};
}

function emptyReview() {
	return { account: 'Wise Fictif', fiscalYearId: 5, proposals: [], gaps: [], toReview: [] };
}

const sum = (id: number, more: Record<string, unknown> = {}) => ({
	id,
	bankAccountId: 1,
	date: '2020-03-10',
	amount: '-86.60',
	currency: 'CHF',
	providerType: 'CARD',
	name: 'Boutique Exemple',
	transactionId: 'CARD-90000020',
	counterpartyEmail: null,
	...more,
});
const usd = sum(12, { bankAccountId: 2, amount: '-13.85', currency: 'USD' });
const feeUsd = sum(11, { bankAccountId: 2, amount: '-0.04', currency: 'USD', transactionId: 'FEE-CARD-90000020', name: null });

function review() {
	return {
		account: 'Wise Fictif',
		fiscalYearId: 5,
		proposals: [
			{
				type: 'card_group',
				bankTransactionIds: [10, 11, 12],
				ruleId: 2,
				counterpartyAccountId: 40,
				lines: [sum(10), feeUsd, usd],
				merchantAmount: '97.95',
				transaction: sum(10),
			},
			{
				type: 'fee',
				bankTransactionId: 3,
				feeAccountId: 60,
				transaction: sum(3, { date: '2020-02-12', amount: '-1.20', transactionId: 'FEE-TRANSFER-1', providerType: 'TRANSFER' }),
			},
		],
		gaps: [],
		toReview: [
			{ code: 'stripe_payout_unmatched', transaction: sum(20, { amount: '1500.00', transactionId: 'BALANCE-8' }) },
			{
				code: 'card_group_statements_missing',
				transaction: sum(30, { transactionId: 'CARD-5' }),
				details: { lines: [sum(30), sum(31, { transactionId: 'FEE-CARD-5' })], missingCurrencies: ['CHF'] },
			},
			{ code: 'card_group_partly_reconciled', transaction: sum(40), details: { journalEntryIds: [77] } },
		],
	};
}

async function choose(r: { findByTestId: (id: string) => Promise<HTMLElement> }, files: File[]) {
	const input = (await r.findByTestId('wise-file-input')) as HTMLInputElement;
	await fireEvent.change(input, { target: { files } });
	return input;
}

describe('page Wise (Story 39-11d)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([wiseAccount(1, 'Wise Fictif'), wiseAccount(2, 'Wise Fictif', 32)]);
		reviewMock.mockResolvedValue(emptyReview());
	});

	it('sans compte Wise : le message et le lien vers les comptes bancaires ; un compte Stripe ne compte pas', async () => {
		listBankAccountsMock.mockResolvedValue([
			{ ...wiseAccount(1, 'x'), provider: 'stripe', wise: null, stripe: { account: 'S', feeAccountId: null, payoutBankAccountId: null } },
		]);
		const r = render(Page);
		expect((await r.findByTestId('wise-no-account')).textContent).toContain('Aucun compte Wise');
		expect(r.getByTestId('wise-no-account-link').getAttribute('href')).toBe('/bank-accounts');
	});

	it('la consigne d’export est à l’écran, le choix prend le zip et plusieurs CSV ; l’exercice et la revue', async () => {
		const r = render(Page);
		const input = (await r.findByTestId('wise-file-input')) as HTMLInputElement;
		expect(input.getAttribute('accept')).toBe('.csv,text/csv,.zip,application/zip');
		expect(input.multiple).toBe(true);
		const aide = r.getByTestId('wise-import-help').textContent ?? '';
		expect(aide).toContain('Afficher les transactions dont les frais sont indiqués séparément');
		expect(aide).toContain('zip');
		expect(r.getByTestId('wise-import-button').textContent).toContain('Importer des relevés Wise');
		expect(r.getByTestId('wise-fiscal-year')).toBeTruthy();
		expect(await r.findByTestId('wise-proposals')).toBeTruthy();
		await waitFor(() => expect(reviewMock).toHaveBeenCalledWith('Wise Fictif', 5));
		expect(document.body.textContent).not.toMatch(/automatiquement/i);
	});

	it('import : tous les fichiers en un envoi, compte rendu par fichier, avertissement et entrées non lues', async () => {
		importMock.mockResolvedValue(report());
		const r = render(Page);
		const zip = new File(['PK'], 'statement_csv.zip', { type: 'application/zip' });
		const csv = new File(['x'], 'autre.csv', { type: 'text/csv' });
		await choose(r, [zip, csv]);
		await waitFor(() => expect(importMock).toHaveBeenCalledWith([zip, csv], 'Wise Fictif'));
		const chf = await r.findByTestId('wise-import-report-file-CHF');
		expect(chf.textContent).toContain('wise-releve-chf-fictif.csv (archive statement_csv.zip)');
		expect(chf.textContent).toContain('CHF : 10 transaction(s) insérée(s), 0 doublon(s) ; solde de 0.00 à 1234.50 CHF');
		expect(chf.querySelector('[data-testid="wise-import-report-warning-gap"]')).toBeNull();
		const eur = r.getByTestId('wise-import-report-file-EUR');
		expect(eur.textContent).toContain('0 transaction(s) insérée(s), 5 doublon(s)');
		expect(eur.querySelector('[data-testid="wise-import-report-file-all-known"]')).not.toBeNull();
		expect(eur.querySelector('[data-testid="wise-import-report-warning-gap"]')?.textContent).toContain(
			'solde d’ouverture 10.00 EUR, au lieu de 0.00 EUR',
		);
		expect(r.getByTestId('wise-import-report-ignored').textContent).toContain('lisez-moi.pdf');
	});

	it('refus : relevé exporté sans les frais séparés — le fichier, son zip, la consigne et les lignes', async () => {
		importMock.mockRejectedValue({
			code: 'WISE_CSV_FEES_NOT_SEPARATED',
			status: 422,
			message: 'refus du serveur',
			details: { filename: 'chf.csv', archive: 'statement_csv.zip', lines: [3, 7], count: 2 },
		});
		const r = render(Page);
		await choose(r, [new File(['PK'], 'statement_csv.zip')]);
		const texte = (await r.findByTestId('wise-import-error')).textContent ?? '';
		expect(texte).toContain('« chf.csv » (archive « statement_csv.zip »)');
		expect(texte).toContain('en cochant « Afficher les transactions dont les frais sont indiqués séparément »');
		expect(texte).toContain('2 opération(s) à frais sans leur ligne de frais (lignes 3, 7)');
		expect(r.queryByTestId('wise-import-report')).toBeNull();
	});

	it('refus du zip : le message du serveur, tel quel', async () => {
		importMock.mockRejectedValue({
			code: 'WISE_ZIP_INVALID',
			status: 422,
			message: 'L’archive « x.zip » ne contient aucun relevé CSV : exportez de Wise le relevé au format CSV.',
			details: { filename: 'x.zip', reason: 'NO_CSV' },
		});
		const r = render(Page);
		await choose(r, [new File(['PK'], 'x.zip')]);
		expect((await r.findByTestId('wise-import-error')).textContent).toContain('ne contient aucun relevé CSV');
	});

	// Revue P1 de la 39-11d, L7, retourné (39-11d2) : la page a sa revue, elle lit les exercices ;
	// la page commune sans revue garde l'ancien comportement (`ProviderReviewPage.test.ts`).
	it('les exercices sont lus : leur échec se dit', async () => {
		listFiscalYearsMock.mockRejectedValueOnce(new Error('exercices indisponibles'));
		const r = render(Page);
		expect(await r.findByTestId('wise-load-error')).toBeTruthy();
		expect(listFiscalYearsMock).toHaveBeenCalled();
	});

	it('refus : relevé déjà importé, le nom sous lequel il l’a été', async () => {
		importMock.mockRejectedValue({
			code: 'BANK_IMPORT_DUPLICATE_FILE',
			status: 422,
			message: 'Ce fichier a déjà été importé.',
			details: { existingImportId: 9, existingFilename: 'statement_1_CHF_2020-01-01_2020-12-31.csv' },
		});
		const r = render(Page);
		await choose(r, [new File(['PK'], 'statement_csv.zip')]);
		expect((await r.findByTestId('wise-import-error')).textContent).toContain(
			'sous le nom « statement_1_CHF_2020-01-01_2020-12-31.csv » : rien n’a été importé',
		);
		expect(r.queryByTestId('wise-import-report')).toBeNull();
	});

	it('refus : deux fichiers d’une même devise, nommés', async () => {
		importMock.mockRejectedValue({
			code: 'WISE_DUPLICATE_CURRENCY_FILE',
			status: 422,
			message: 'refus',
			details: { currency: 'EUR', filenames: ['a.csv', 'b.csv'] },
		});
		const r = render(Page);
		await choose(r, [new File(['x'], 'a.csv'), new File(['x'], 'b.csv')]);
		expect((await r.findByTestId('wise-import-error')).textContent).toContain('en EUR (a.csv, b.csv) : un seul fichier par devise');
	});
});

describe('page Wise — la revue (Story 39-11d2)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listBankAccountsMock.mockResolvedValue([wiseAccount(1, 'Wise Fictif'), wiseAccount(2, 'Wise Fictif', 32)]);
		reviewMock.mockResolvedValue(review());
	});

	it('genres et « ce qui sera fait » ; le paiement groupé et ses sous-lignes ; rien de coché', async () => {
		const r = render(Page);
		const groupe = await r.findByTestId('wise-proposal-10');
		expect(groupe.textContent).toContain('Paiement par carte');
		await waitFor(() =>
			expect(groupe.textContent).toContain(
				'Une écriture : 97.95 CHF au compte 4000 — Achats (règle Boutique), chaque solde crédité de sa part',
			),
		);
		expect(r.getByTestId('wise-proposal-line-11').textContent).toContain('-0.04 USD');
		expect(r.getByTestId('wise-proposal-line-11').textContent).toContain('frais');
		expect(r.getByTestId('wise-proposal-line-12').textContent).toContain('-13.85 USD');
		expect(r.queryByTestId('wise-proposal-line-10')).toBeNull();
		expect(r.queryByTestId('wise-check-11')).toBeNull();
		expect(r.getByTestId('wise-proposal-3').textContent).toContain('Écriture au compte de frais 6947 — Frais Wise');
		expect((r.getByTestId('wise-check-10') as HTMLInputElement).checked).toBe(false);
		expect(r.getByTestId('wise-gaps-empty').textContent).toContain('Aucun écart.');
	});

	it('« à examiner » : la phrase de chaque motif, les sous-lignes et leur bouton, les liens', async () => {
		const r = render(Page);
		const versement = await r.findByTestId('wise-to-review-20');
		expect(versement.textContent).toContain('Versement Stripe sans l’écriture de son versement');
		// Le nom et l'identifiant Wise de la ligne, et de chaque sous-ligne : de quoi la retrouver.
		expect(versement.textContent).toContain('BALANCE-8');
		expect(versement.textContent).toContain('Boutique Exemple');
		// Sans compte Stripe, aucun lien vers la page Stripe.
		expect(r.queryByTestId('wise-to-review-stripe-link')).toBeNull();
		expect(r.getByTestId('wise-to-review-30').textContent).toContain('le relevé du solde CHF ne couvre pas cette date');
		expect(r.getByTestId('wise-to-review-line-31').textContent).toContain('FEE-CARD-5');
		expect(r.getByTestId('wise-link-line-31')).toBeTruthy();
		expect(r.getByTestId('wise-link-30')).toBeTruthy();
		const part = r.getByTestId('wise-to-review-40');
		expect(part.textContent).toContain('Une part de ce paiement est déjà rapprochée');
		expect(r.getByTestId('wise-to-review-entry-77').getAttribute('href')).toBe('/journal-entries/77');
	});

	it('« Rattacher » d\u2019une sous-ligne ouvre la modale sur SA transaction (31), non sur la ligne principale (30)', async () => {
		const r = await (async () => {
			const x = render(Page);
			await x.findByTestId('wise-to-review-30');
			return x;
		})();
		vi.mocked(getEntryCandidates).mockClear();
		await fireEvent.click(r.getByTestId('wise-link-line-31'));
		await waitFor(() => expect(getEntryCandidates).toHaveBeenCalled());
		expect(getEntryCandidates).toHaveBeenLastCalledWith(31);
		expect(getEntryCandidates).not.toHaveBeenCalledWith(30);
	});

	it('avec un compte Stripe, le versement sans son écriture mène à la page Stripe', async () => {
		listBankAccountsMock.mockResolvedValue([
			wiseAccount(1, 'Wise Fictif'),
			{ ...wiseAccount(9, 'x'), provider: 'stripe', wise: null, stripe: { account: 'S', feeAccountId: null, payoutBankAccountId: null } },
		]);
		const r = render(Page);
		expect((await r.findByTestId('wise-to-review-stripe-link')).getAttribute('href')).toBe('/reconciliation/stripe');
	});

	it('acceptation : les corps dans l’ordre des dates (un groupe sans ses lignes), le compte rendu, la revue relue', async () => {
		acceptMock.mockResolvedValue({
			accepted: [{ type: 'fee', bankTransactionIds: [3], journalEntryIds: [1] }],
			failed: [
				{
					bankTransactionId: 10,
					errorCode: 'PROVIDER_PROPOSAL_STALE',
					details: { reason: 'card_group_statements_missing', missingBankAccountIds: [1], missingCurrencies: ['CHF'] },
				},
			],
		});
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('wise-select-all'));
		await fireEvent.click(r.getByTestId('wise-accept'));
		await waitFor(() => expect(acceptMock).toHaveBeenCalledTimes(1));
		expect(acceptMock).toHaveBeenCalledWith('Wise Fictif', [
			{ type: 'fee', bankTransactionId: 3, feeAccountId: 60 },
			{ type: 'card_group', bankTransactionIds: [10, 11, 12], ruleId: 2, counterpartyAccountId: 40 },
		]);
		const compteRendu = await r.findByTestId('wise-accept-report');
		expect(compteRendu.textContent).toContain('1 proposition(s) acceptée(s), 1 refusée(s).');
		expect(r.getByTestId('wise-accept-failed').textContent).toContain(
			'#10 — Une ligne de ce paiement peut manquer : le relevé du solde CHF ne couvre pas cette date',
		);
		await waitFor(() => expect(reviewMock).toHaveBeenCalledTimes(2));
	});
});
