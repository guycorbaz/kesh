/**
 * Story 40-7d (AC-5) — l'écran Stripe, de l'écran au serveur.
 *
 * Société `with-company` (exercice ouvert au 01.01.2020) passée en multi-devises
 * par l'API (comptes de change désignés), comptes du plan CHF, EUR et USD des
 * soldes Stripe, compte de frais en CHF, compte bancaire de versement CHF (une
 * banque ordinaire), cours mensuels de janvier à avril 2020 copiés dans les
 * cours de la société (`POST /exchange-rates/fetch`, source factice du mode test).
 *
 * 1. Avant tout compte Stripe, la page le dit. Le compte Stripe CHF se crée **par
 *    l'écran** (case « Compte Stripe », compte de frais, compte de versement) ;
 *    EUR (sans compte de versement) et USD (sans compte de frais) par l'API, même
 *    libellé.
 * 2. Écritures de vente par l'API (patron du script des boutiques, identifiant
 *    `ch_…` au libellé) : trois justes (CHF, CHF, EUR, USD), une sans charge.
 * 3. Le rapport sans `source_id` est refusé avec sa marche à suivre ; la fixture
 *    fictive de 40-7a (lue par chemin, ni copie ni seconde fixture) s'importe
 *    **par l'écran** : compte rendu par devise et versements lus.
 * 4. Revue : rattachements aux ventes, frais, versement ; écarts « Charge sans
 *    vente » et « Vente sans charge » ; lignes « à examiner ». « Tout cocher »,
 *    « Accepter la sélection » par l'écran : aucun refus.
 * 5. L'écart « Versement sans réception » paraît ; la réception, saisie par un
 *    relevé à la main sur la banque CHF, est rattachée à l'écriture du versement
 *    par `POST …/link-entry` (le geste à l'écran est celui de la réconciliation) ;
 *    la revue relue, l'écart a disparu.
 *
 * Écart à la fiche (AC-5) : la réception devait être celle d'un compte Wise EUR,
 * importée par la route Wise (39-11b) ; cette route n'est pas dans la branche de
 * l'Epic 40, et un relevé en devise ne se saisit pas hors prestataire. La
 * réception est donc celle du versement CHF, sur une banque, par un relevé saisi
 * à la main — même geste de rattachement, même écart levé.
 *
 * L'activation du multi-devises est irréversible : la spec reseede `with-company`
 * en fin de parcours. Captures d'écran (thème clair) dans `KESH_CAPTURES_DIR`.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { join } from 'node:path';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';

test.describe.configure({ mode: 'serial' });

const CAPTURES = process.env.KESH_CAPTURES_DIR;
const FIXTURES = join(process.cwd(), '../crates/kesh-import/tests/fixtures');
const FIXTURE = join(FIXTURES, 'stripe-itemized-payout-fictif.csv');
const FIXTURE_SANS_SOURCE = join(FIXTURES, 'stripe-itemized-payout-sans-source-fictif.csv');
const STRIPE = 'Stripe boutique';

async function capture(page: Page, name: string): Promise<void> {
	if (CAPTURES) await page.screenshot({ path: join(CAPTURES, `${name}.png`), fullPage: true });
}

async function captureOf(page: Page, testId: string, name: string): Promise<void> {
	if (CAPTURES) await page.getByTestId(testId).screenshot({ path: join(CAPTURES, `${name}.png`) });
}

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function api<T>(ctx: APIRequestContext, method: 'get' | 'post' | 'put', path: string, data?: unknown): Promise<T> {
	const res = await ctx[method](path, data === undefined ? undefined : { data });
	expect(res.ok(), `${method.toUpperCase()} ${path} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return (await res.json()) as T;
}

interface Ids {
	chf: number;
	eur: number;
	usd: number;
	fee: number;
	revenue: number;
	bankLedger: number;
	bank: number;
}
let ids: Ids;

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterAll(async () => {
	await seedTestState('with-company');
});

test.beforeEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('1 — préparation ; compte Stripe CHF par l’écran, avec son compte de versement', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/stripe');
	await expect(page.getByTestId('stripe-no-account')).toBeVisible();
	await expect(page.getByTestId('stripe-no-account-link')).toHaveAttribute('href', '/bank-accounts');
	await capture(page, '00-aucun-compte-stripe');
	const ctx = await authedApiContext(page);
	try {
		const create = async (number: string, name: string, accountType: string, extra: object = {}) =>
			(await api<{ id: number }>(ctx, 'post', '/api/v1/accounts', { number, name, accountType, ...extra })).id;
		const transit = await create('1090', 'Compte de conversion', 'Asset');
		const gain = await create('6840', 'Gains de change', 'Revenue');
		const loss = await create('6940', 'Pertes de change', 'Expense');
		const company = await api<{ company: { version: number } }>(ctx, 'get', '/api/v1/companies/current');
		await api(ctx, 'put', '/api/v1/companies/current/currency-settings', {
			multiCurrencyEnabled: true,
			fxGainAccountId: gain,
			fxLossAccountId: loss,
			fxTransitAccountId: transit,
			version: company.company.version,
		});
		const chf = await create('1061', 'Stripe CHF', 'Asset');
		const eur = await create('1062', 'Stripe EUR', 'Asset', { currency: 'EUR' });
		const usd = await create('1063', 'Stripe USD', 'Asset', { currency: 'USD' });
		const fee = await create('6946', 'Frais Stripe', 'Expense');
		const bankLedger = await create('1029', 'Banque des versements', 'Asset');
		const bank = (
			await api<{ id: number }>(ctx, 'post', '/api/v1/bank-accounts', {
				bankName: 'Banque des versements',
				iban: 'CH9300762011623852957',
				journalAccountId: bankLedger,
			})
		).id;
		const accounts = await api<Array<{ id: number; accountType: string; postable: boolean; active: boolean; currency?: string | null }>>(
			ctx,
			'get',
			'/api/v1/accounts',
		);
		const revenue = accounts.find((a) => a.accountType === 'Revenue' && a.postable && a.active && !a.currency && a.id !== gain)!.id;
		ids = { chf, eur, usd, fee, revenue, bankLedger, bank };
		for (let month = 1; month <= 4; month++) {
			await api(ctx, 'post', '/api/v1/exchange-rates/fetch', { year: 2020, month });
		}
	} finally {
		await disposeContextSafe(ctx);
	}

	// Le compte Stripe CHF, par l'écran.
	await page.goto('/bank-accounts');
	await page.getByTestId('create-bank-account-button').click();
	await page.getByTestId('form-bank-name').fill('Stripe');
	await page.getByTestId('form-is-stripe').check();
	await expect(page.getByTestId('form-is-paypal')).not.toBeChecked();
	await page.getByTestId('form-stripe-account').fill(STRIPE);
	await page.getByTestId('form-journal-account').selectOption(String(ids.chf));
	await page.getByTestId('form-fee-account').selectOption(String(ids.fee));
	await page.getByTestId('form-payout-account').selectOption(String(ids.bank));
	await expect(page.getByTestId('form-qr-iban')).toHaveCount(0);
	await capture(page, '01-formulaire-compte-stripe');
	await page.getByTestId('form-submit').click();
	await expect(page.getByTestId('create-bank-account-form')).toHaveCount(0);
	await expect(page.locator('[data-testid^="bank-account-stripe-"]').filter({ hasText: `${STRIPE} · CHF` })).toHaveCount(1);

	// EUR (sans compte de versement) et USD (sans compte de frais) par l'API, même libellé.
	const ctx2 = await authedApiContext(page);
	try {
		await api(ctx2, 'post', '/api/v1/bank-accounts', {
			bankName: 'Stripe EUR',
			journalAccountId: ids.eur,
			stripe: { account: STRIPE, feeAccountId: ids.fee, payoutBankAccountId: null },
		});
		await api(ctx2, 'post', '/api/v1/bank-accounts', {
			bankName: 'Stripe USD',
			journalAccountId: ids.usd,
			stripe: { account: STRIPE, feeAccountId: null, payoutBankAccountId: null },
		});
	} finally {
		await disposeContextSafe(ctx2);
	}
	await page.reload();
	await expect(page.locator('[data-testid^="bank-account-stripe-"]').filter({ hasText: `${STRIPE} · USD` })).toHaveCount(1);
	await capture(page, '02-liste-comptes-stripe');
});

test('2 — ventes des boutiques ; rapport sans source_id refusé ; import par l’écran', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const sale = (entryDate: string, description: string, ledger: number, amount: string, currency?: string, rate?: string) => {
			const chf = rate ? (Number(amount) * Number(rate)).toFixed(2) : amount;
			return api(ctx, 'post', '/api/v1/journal-entries', {
				entryDate,
				journal: 'Ventes',
				description,
				lines: [
					{
						accountId: ledger,
						debit: chf,
						credit: '0',
						...(currency ? { foreign: { amount, rate, rateSource: 'manual' } } : {}),
					},
					{ accountId: ids.revenue, debit: '0', credit: chf },
				],
			});
		};
		await sale('2020-01-29', 'Commande A ch_TEST0001', ids.chf, '100.00');
		await sale('2020-01-30', 'Commande B ch_TEST0002', ids.chf, '50.00');
		await sale('2020-02-05', 'Commande C ch_TEST0003', ids.eur, '200.00', 'EUR', '0.95');
		await sale('2020-02-12', 'Commande E ch_TEST0004', ids.usd, '40.00', 'USD', '0.90');
		// Une vente qu'aucune charge du rapport ne porte.
		await sale('2020-02-14', 'Commande Z ch_TEST0050', ids.chf, '30.00');
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.goto('/reconciliation/stripe');
	await expect(page.getByTestId('stripe-account')).toHaveValue(STRIPE);
	await expect(page.getByTestId('stripe-hint')).toContainText('importez-les avant le rapport Stripe');
	const fy = page.getByTestId('stripe-fiscal-year');
	await fy.selectOption({ label: (await fy.locator('option').first().textContent())!.trim() });
	// Le rapport sans source_id : refusé, la marche à suivre dans Stripe, rien d'importé.
	await page.getByTestId('stripe-file-input').setInputFiles(FIXTURE_SANS_SOURCE);
	await expect(page.getByTestId('stripe-import-error')).toContainText('ajoutez la colonne source_id');
	await captureOf(page, 'stripe-import-error', '03-rapport-sans-source-id');
	await page.getByTestId('stripe-file-input').setInputFiles(FIXTURE);
	await expect(page.getByTestId('stripe-import-report')).toBeVisible();
	await expect(page.getByTestId('stripe-import-error')).toHaveCount(0);
	await expect(page.getByTestId('stripe-import-report-CHF')).toContainText('insérée');
	await expect(page.getByTestId('stripe-import-report-EUR')).toContainText('insérée');
	await expect(page.getByTestId('stripe-import-report-USD')).toContainText('insérée');
	await expect(page.getByTestId('stripe-import-report-payout-po_TEST0001')).toContainText('4 ligne(s), net 125.15 CHF');
	await capture(page, '04-compte-rendu-import');
});

test('3 — revue : propositions, écarts, à examiner ; tout accepter', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/stripe');
	const proposals = page.getByTestId('stripe-proposals-table');
	await expect(proposals).toBeVisible();
	await expect(proposals).toContainText('Rattachement à la vente n°');
	await expect(proposals).toContainText('Frais');
	await expect(proposals).toContainText('Versement vers Banque des versements');
	await expect(proposals.locator('tbody tr').filter({ hasText: 'Versement vers' })).toHaveCount(1);
	await capture(page, '05-propositions');
	const gaps = page.getByTestId('stripe-gaps-table');
	await expect(gaps).toContainText('Charge sans vente');
	await expect(gaps).toContainText('Vente sans charge');
	await expect(gaps).toContainText('ch_TEST0050');
	await captureOf(page, 'stripe-gaps', '06-ecarts');
	const toReview = page.getByTestId('stripe-to-review-table');
	await expect(toReview).toContainText('Remboursement');
	await expect(toReview).toContainText('compte de versement');
	await expect(toReview).toContainText('compte de frais');
	await captureOf(page, 'stripe-to-review', '07-a-examiner');
	await expect(page.locator('body')).not.toContainText(/automatiquement/i);

	await page.getByTestId('stripe-select-all').click();
	await page.getByTestId('stripe-accept').click();
	await expect(page.getByTestId('stripe-accept-report')).toContainText('0 refusée(s)');
	await expect(page.getByTestId('stripe-accept-failed')).toHaveCount(0);
	await expect(page.getByTestId('stripe-proposals-empty')).toBeVisible();
	await capture(page, '08-compte-rendu-acceptation');
});

test('4 — versement sans réception : rattaché à sa réception, l’écart disparaît', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/stripe');
	const gap = page.getByTestId('stripe-gap-payout_without_receipt');
	await expect(gap).toHaveCount(1);
	await expect(gap).toContainText('Versement sans réception');
	await expect(gap.getByTestId('stripe-gap-receipt-links').locator('a').last()).toHaveAttribute('href', '/reconciliation');
	await captureOf(page, 'stripe-gaps', '09-versement-sans-reception');

	const ctx = await authedApiContext(page);
	try {
		const years = await api<Array<{ id: number; startDate: string; endDate: string }>>(ctx, 'get', '/api/v1/fiscal-years');
		const year = years.find((y) => y.startDate <= '2020-02-03' && '2020-02-03' <= y.endDate)!.id;
		const review = await api<{ gaps: Array<{ code: string; details?: { journalEntryId?: number } }> }>(
			ctx,
			'get',
			`/api/v1/stripe/review?account=${encodeURIComponent(STRIPE)}&fiscalYearId=${year}`,
		);
		const entryId = review.gaps.find((g) => g.code === 'payout_without_receipt')!.details!.journalEntryId!;
		// La réception du versement CHF, saisie à la main sur la banque des versements.
		const imported = await api<{ id: number }>(ctx, 'post', '/api/v1/bank-imports/manual', {
			bankAccountId: ids.bank,
			periodFrom: '2020-02-04',
			periodTo: '2020-02-04',
			statementNumber: null,
			openingBalance: '0.00',
			closingBalance: '125.15',
			lines: [
				{
					bookingDate: '2020-02-04',
					valueDate: '2020-02-04',
					details: 'Versement Stripe po_TEST0001',
					counterpartyName: 'Stripe',
					reference: null,
					amount: '125.15',
					direction: 'credit',
				},
			],
			confirmDuplicateFile: false,
			confirmDuplicateLines: false,
			confirmOpeningBalanceGap: false,
		});
		const detail = await api<{ transactions: Array<{ id: number }> }>(ctx, 'get', `/api/v1/bank-imports/${imported.id}`);
		await api(ctx, 'post', `/api/v1/reconciliation/transactions/${detail.transactions[0].id}/link-entry`, {
			journalEntryId: entryId,
		});
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.reload();
	await expect(page.getByTestId('stripe-gaps')).toBeVisible();
	await expect(page.getByTestId('stripe-gap-payout_without_receipt')).toHaveCount(0);
	await capture(page, '10-apres-reception');
});
