/**
 * Story 38-11d (AC-8) — l'écran PayPal, de l'écran au serveur.
 *
 * Société `with-company` (exercice ouvert au 01.01.2020) passée en multi-devises
 * par l'API (compte de conversion désigné), comptes du plan EUR et USD du solde
 * PayPal (le CHF prend un compte d'actif sans devise), compte de frais en CHF,
 * cours mensuels de janvier à avril 2020 copiés dans les cours de la société
 * (`POST /exchange-rates/fetch`, source factice du mode test) puis le cours EUR de
 * février posé par `PUT /exchange-rates/rates`.
 *
 * 1. Le compte PayPal EUR se crée **par l'écran** (case « Compte PayPal », compte
 *    du plan en devise, compte de frais) ; USD et CHF par l'API, même libellé.
 * 2. Une règle « Fournisseur » avec le filtre « E-mail exact », **par l'écran**.
 * 3. Trois écritures de vente « TXN … » par l'API (patron du script des
 *    boutiques) : une juste (EUR), une de montant faux (CHF), une sans encaissement.
 * 4. Import de la fixture fictive de 38-11a **par l'écran** (lue par chemin) :
 *    compte rendu (insérées, doublons, écartées).
 * 5. Revue : propositions, un écart de chaque genre, une ligne « à examiner » ;
 *    « Tout cocher », « Accepter la sélection » : aucun refus, plus de proposition.
 * 6. La ligne « à examiner » (remboursement) se rattache par la modale à une
 *    écriture en EUR passée à la main.
 *
 * L'activation du multi-devises est irréversible : la spec reseede `with-company`
 * en fin de parcours. Captures d'écran (thème clair) dans `KESH_CAPTURES_DIR`.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { join } from 'node:path';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';

test.describe.configure({ mode: 'serial' });

const CAPTURES = process.env.KESH_CAPTURES_DIR;
const FIXTURE = join(process.cwd(), '../crates/kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv');
const PAYPAL = 'PayPal www';

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
	eur: number;
	usd: number;
	chf: number;
	fee: number;
	revenue: number;
	expense: number;
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

test('1 — préparation : multi-devises, comptes, cours ; compte PayPal EUR par l’écran', async ({ page }) => {
	await login(page);
	// Avant tout compte PayPal : la page le dit, avec le lien vers les comptes bancaires.
	await page.goto('/reconciliation/paypal');
	await expect(page.getByTestId('paypal-no-account')).toBeVisible();
	await capture(page, '00-aucun-compte-paypal');
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
		const eur = await create('1051', 'PayPal EUR', 'Asset', { currency: 'EUR' });
		const usd = await create('1052', 'PayPal USD', 'Asset', { currency: 'USD' });
		const chf = await create('1053', 'PayPal CHF', 'Asset');
		const fee = await create('6945', 'Frais PayPal', 'Expense');
		const accounts = await api<Array<{ id: number; number: string; accountType: string; postable: boolean; active: boolean; currency?: string | null }>>(
			ctx,
			'get',
			'/api/v1/accounts',
		);
		const firstOf = (type: string) =>
			accounts.find((a) => a.accountType === type && a.postable && a.active && !a.currency && a.id !== fee && a.id !== gain && a.id !== loss)!.id;
		ids = { eur, usd, chf, fee, revenue: firstOf('Revenue'), expense: firstOf('Expense') };
		for (let month = 1; month <= 4; month++) {
			await api(ctx, 'post', '/api/v1/exchange-rates/fetch', { year: 2020, month });
		}
		const list = await api<{ rates: Array<{ currency: string; row: { version: number } | null }> }>(
			ctx,
			'get',
			'/api/v1/exchange-rates?month=2020-02',
		);
		const version = list.rates.find((r) => r.currency === 'EUR')!.row!.version;
		await api(ctx, 'put', '/api/v1/exchange-rates/rates', {
			currency: 'EUR',
			kind: 'month',
			rateDate: '2020-02-01',
			rate: '0.94',
			version,
		});
	} finally {
		await disposeContextSafe(ctx);
	}

	// Le compte PayPal EUR, par l'écran.
	await page.goto('/bank-accounts');
	await page.getByTestId('create-bank-account-button').click();
	await page.getByTestId('form-bank-name').fill('PayPal');
	await page.getByTestId('form-is-paypal').check();
	await page.getByTestId('form-paypal-account').fill(PAYPAL);
	await page.getByTestId('form-journal-account').selectOption(String(ids.eur));
	await page.getByTestId('form-fee-account').selectOption(String(ids.fee));
	await expect(page.getByTestId('form-qr-iban')).toHaveCount(0);
	await expect(page.getByTestId('form-is-primary')).toHaveCount(0);
	await capture(page, '01-formulaire-compte-paypal');
	await page.getByTestId('form-submit').click();
	await expect(page.getByTestId('create-bank-account-form')).toHaveCount(0);
	await expect(page.locator('[data-testid^="bank-account-paypal-"]').filter({ hasText: `${PAYPAL} · EUR` })).toHaveCount(1);

	// USD et CHF par l'API, même libellé.
	const ctx2 = await authedApiContext(page);
	try {
		for (const [ledger, label] of [
			[ids.usd, 'USD'],
			[ids.chf, 'CHF'],
		] as const) {
			await api(ctx2, 'post', '/api/v1/bank-accounts', {
				bankName: `PayPal ${label}`,
				journalAccountId: ledger,
				paypal: { account: PAYPAL, feeAccountId: ids.fee },
			});
		}
	} finally {
		await disposeContextSafe(ctx2);
	}
	await page.reload();
	await expect(page.locator('[data-testid^="bank-account-paypal-"]').filter({ hasText: `${PAYPAL} · USD` })).toHaveCount(1);
	await capture(page, '02-liste-comptes-paypal');
});

test('2 — règle « Fournisseur » avec le filtre « E-mail exact », par l’écran', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/rules');
	await page.getByTestId('rules-create-button').click();
	await page.getByTestId('rule-form-label').fill('Licences');
	await page.getByTestId('rule-form-match-type').selectOption('counterparty_contains');
	await page.getByTestId('rule-form-match-value').fill('Fournisseur');
	await page.getByTestId('rule-form-counterparty-email').fill('Fournisseur@Example.invalid');
	await page.getByTestId('rule-form-direction').selectOption('debit');
	await page.getByTestId('rule-form-counterparty').selectOption(String(ids.expense));
	await capture(page, '03-regle-avec-filtres');
	await page.getByTestId('rule-form-submit').click();
	const filters = page.locator('[data-testid^="rule-filter-"]');
	await expect(filters.filter({ hasText: 'fournisseur@example.invalid' })).toHaveCount(1);
	await expect(filters).toHaveCount(2);
	await capture(page, '04-liste-regles-filtres');
});

test('3 — écritures de vente des boutiques, import par l’écran, compte rendu', async ({ page }) => {
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
		// Juste : même TXN, même brut, même devise.
		await sale('2020-01-03', 'Commande 1001 TXN 0TEST00000000001A', ids.eur, '100.00', 'EUR', '0.93');
		// Montant faux (la boutique a écrit 1200 au lieu de 1234.56).
		await sale('2020-02-14', 'Commande 1002 TXN 0TEST00000000006F', ids.chf, '1200.00');
		// Sans encaissement.
		await sale('2020-03-20', 'Commande 1009 TXN 0TEST0000000000ZZ', ids.eur, '45.00', 'EUR', '0.93');
	} finally {
		await disposeContextSafe(ctx);
	}

	await page.goto('/reconciliation/paypal');
	await expect(page.getByTestId('paypal-account')).toHaveValue(PAYPAL);
	await page.getByTestId('paypal-fiscal-year').selectOption({ index: 0 });
	// Un fichier refusé d'abord : une phrase par cause, rien d'importé.
	const dir = mkdtempSync(join(tmpdir(), 'kesh-paypal-'));
	const broken = join(dir, 'paypal-colonnes.csv');
	writeFileSync(broken, '"Date","Name","Type"\n"03/01/2020","Alice","Website Payment"\n');
	await page.getByTestId('paypal-file-input').setInputFiles(broken);
	await expect(page.getByTestId('paypal-import-error')).toContainText('Colonnes manquantes');
	await captureOf(page, 'paypal-import-error', '05a-import-refuse');
	await page.getByTestId('paypal-file-input').setInputFiles(FIXTURE);
	const report = page.getByTestId('paypal-import-report');
	await expect(report).toBeVisible();
	await expect(page.getByTestId('paypal-import-report-EUR')).toContainText('insérée');
	await expect(page.getByTestId('paypal-import-report-USD')).toContainText('insérée');
	await expect(page.getByTestId('paypal-import-report-CHF')).toContainText('insérée');
	await expect(page.getByTestId('paypal-import-report-skipped')).toContainText('2 ligne(s) non terminée(s), 1 ligne(s) Memo, 1 paire(s) neutre(s)');
	await capture(page, '05-compte-rendu-import');
});

test('4 — revue : propositions, écarts, à examiner ; tout accepter', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/paypal');
	await expect(page.getByTestId('paypal-proposals-table')).toBeVisible();
	const proposals = page.getByTestId('paypal-proposals-table');
	await expect(proposals).toContainText('Vente rattachée');
	await expect(proposals).toContainText('Frais');
	await expect(proposals).toContainText('Règle Licences');
	await expect(proposals).toContainText('Conversion');
	await expect(proposals).toContainText('Retrait');
	await capture(page, '06-propositions');
	const gaps = page.getByTestId('paypal-gaps-table');
	await expect(gaps).toContainText('Encaissement sans vente');
	await expect(gaps).toContainText('Vente sans encaissement');
	await expect(gaps).toContainText('Montant ou devise différents');
	await captureOf(page, 'paypal-gaps', '07-ecarts');
	const toReview = page.getByTestId('paypal-to-review-table');
	await expect(toReview).toContainText('Aucune règle');
	await expect(toReview).toContainText('daniel@example.invalid');
	await captureOf(page, 'paypal-to-review', '08-a-examiner');

	// Sans compte bancaire principal, le retrait demande son compte de destination : la case
	// reste grisée tant qu'il n'est pas choisi.
	const withdrawalSelect = page.locator('[data-testid^="paypal-withdrawal-"]');
	await expect(withdrawalSelect).toHaveCount(1);
	const withdrawalRow = proposals.locator('tbody tr').filter({ has: withdrawalSelect });
	await expect(withdrawalRow.locator('input[type="checkbox"]')).toBeDisabled();
	await withdrawalRow.screenshot(CAPTURES ? { path: join(CAPTURES, '06a-retrait-a-choisir.png') } : {});
	await withdrawalSelect.selectOption({ index: 1 });
	await page.getByTestId('paypal-select-all').click();
	// Les frais du remboursement restent décochés : ils serviront au refus de l'étape 6.
	await proposals.locator('tbody tr').filter({ hasText: '0TEST0000000000BL:FEE' }).locator('input[type="checkbox"]').uncheck();
	await page.getByTestId('paypal-accept').click();
	const acceptReport = page.getByTestId('paypal-accept-report');
	await expect(acceptReport).toContainText('0 refusée(s)');
	await expect(page.getByTestId('paypal-accept-failed')).toHaveCount(0);
	await expect(proposals.locator('tbody tr')).toHaveCount(1);
	await capture(page, '09-compte-rendu-acceptation');
});

test('5 — la ligne « à examiner » se rattache par la modale, en EUR', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		// Le remboursement de Daniel, passé à la main en EUR : 30 EUR rendus.
		await api(ctx, 'post', '/api/v1/journal-entries', {
			entryDate: '2020-03-30',
			journal: 'Banque',
			description: 'Remboursement Daniel Fictif',
			lines: [
				{ accountId: ids.revenue, debit: '27.90', credit: '0' },
				{ accountId: ids.eur, debit: '0', credit: '27.90', foreign: { amount: '30.00', rate: '0.93', rateSource: 'manual' } },
			],
		});
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.goto('/reconciliation/paypal');
	const refundRows = () => page.getByTestId('paypal-to-review').locator('tbody tr').filter({ hasText: 'Payment Refund' });
	const row = refundRows();
	await expect(row).toBeVisible();
	await row.locator('[data-testid^="paypal-link-"]').click();
	const modal = page.getByTestId('link-settlement-modal');
	await expect(modal).toBeVisible();
	await expect(modal.getByTestId('link-settlements-section')).toHaveCount(0);
	const entryRow = modal.getByTestId('entry-candidate-row').filter({ hasText: 'Remboursement Daniel Fictif' });
	await expect(entryRow).toContainText('-30.00 EUR');
	await entryRow.getByTestId('entry-candidate-radio').check();
	await capture(page, '10-rattachement-modale');
	await modal.getByTestId('link-submit').click();
	await expect(modal).toHaveCount(0);
	await expect(refundRows()).toHaveCount(0);
	await capture(page, '11-apres-rattachement');
});

test('6 — une proposition acceptée ailleurs entre-temps est refusée, avec son motif', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/paypal');
	const proposals = page.getByTestId('paypal-proposals-table');
	const row = proposals.locator('tbody tr').filter({ hasText: '0TEST0000000000BL:FEE' });
	await expect(row).toBeVisible();
	// Pendant que la page est ouverte, la même proposition est acceptée par l'API.
	const ctx = await authedApiContext(page);
	try {
		const fy = await api<Array<{ id: number; startDate: string }>>(ctx, 'get', '/api/v1/fiscal-years');
		const year = fy.find((y) => y.startDate <= '2020-03-30')!.id;
		const review = await api<{ proposals: Array<{ type: string; bankTransactionId: number; feeAccountId: number }> }>(
			ctx,
			'get',
			`/api/v1/paypal/review?account=${encodeURIComponent(PAYPAL)}&fiscalYearId=${year}`,
		);
		const fee = review.proposals.find((p) => p.type === 'fee')!;
		const res = await api<{ accepted: unknown[]; failed: unknown[] }>(ctx, 'post', '/api/v1/paypal/accept', {
			account: PAYPAL,
			proposals: [{ type: 'fee', bankTransactionId: fee.bankTransactionId, feeAccountId: fee.feeAccountId }],
		});
		expect(res.accepted).toHaveLength(1);
	} finally {
		await disposeContextSafe(ctx);
	}
	await row.locator('input[type="checkbox"]').check();
	await page.getByTestId('paypal-accept').click();
	await expect(page.getByTestId('paypal-accept-report')).toContainText('1 refusée(s)');
	await expect(page.getByTestId('paypal-accept-failed')).toContainText("n'est plus valable");
	await captureOf(page, 'paypal-accept-report', '12-acceptation-avec-refus');
	await expect(page.getByTestId('paypal-proposals-empty')).toBeVisible();
});

test('7 — compte principal : la case « Compte PayPal » est désactivée, avec son motif', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	let primaryId: number;
	try {
		const ledger = (
			await api<{ id: number }>(ctx, 'post', '/api/v1/accounts', { number: '1059', name: 'Banque principale', accountType: 'Asset' })
		).id;
		primaryId = (
			await api<{ id: number }>(ctx, 'post', '/api/v1/bank-accounts', {
				bankName: 'Banque principale',
				iban: 'CH9300762011623852957',
				isPrimary: true,
				journalAccountId: ledger,
			})
		).id;
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.goto('/bank-accounts');
	await page.getByTestId(`edit-button-${primaryId}`).click();
	await expect(page.getByTestId('edit-is-paypal')).toBeDisabled();
	await expect(page.getByTestId('edit-is-paypal-locked')).toBeVisible();
	await captureOf(page, 'edit-bank-account-form', '13-principal-case-desactivee');
});

test('8 — retrait détaché côté PayPal, réception rattachée : « Déjà comptabilisée », jamais écrit deux fois', async ({
	page,
}) => {
	// Correctif « une transaction de prestataire n'est écrite qu'une fois ».
	await login(page);
	const ctx = await authedApiContext(page);
	let withdrawalId = 0;
	let entryId = 0;
	let destination = 0;
	try {
		const banks = await api<Array<{ id: number; journalAccountId: number | null }>>(ctx, 'get', '/api/v1/bank-accounts');
		const paypalChf = banks.find((b) => b.journalAccountId === ids.chf)!.id;
		const imports = await api<{ items: Array<{ id: number }> }>(ctx, 'get', `/api/v1/bank-imports?bankAccountId=${paypalChf}`);
		for (const imp of imports.items) {
			const detail = await api<{
				transactions: Array<{ id: number; transactionId: string | null; matchedEntryId: number | null }>;
			}>(ctx, 'get', `/api/v1/bank-imports/${imp.id}`);
			const t = detail.transactions.find((x) => x.transactionId === '0TEST00000000007G');
			if (t) {
				withdrawalId = t.id;
				entryId = t.matchedEntryId ?? 0;
			}
		}
		expect(entryId, 'le retrait accepté à l’étape 4').toBeGreaterThan(0);
		// La réception : un compte bancaire relié au compte de destination du retrait.
		const entry = await api<{ lines: Array<{ accountId: number }> }>(ctx, 'get', `/api/v1/journal-entries/${entryId}`);
		destination = entry.lines.find((l) => l.accountId !== ids.chf)!.accountId;
		const bank = await api<{ id: number }>(ctx, 'post', '/api/v1/bank-accounts', {
			bankName: 'Banque de réception',
			iban: 'CH5604835012345678009',
			journalAccountId: destination,
		});
		const imported = await api<{ id: number }>(ctx, 'post', '/api/v1/bank-imports/manual', {
			bankAccountId: bank.id,
			periodFrom: '2020-02-28',
			periodTo: '2020-02-28',
			statementNumber: null,
			openingBalance: '0.00',
			closingBalance: '1000.00',
			lines: [
				{
					bookingDate: '2020-02-28',
					valueDate: '2020-02-28',
					details: 'Virement PayPal',
					counterpartyName: 'PayPal',
					reference: null,
					amount: '1000.00',
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
		// Le côté PayPal annulé seul : son lien retiré, l'écriture reste.
		await api(ctx, 'post', `/api/v1/reconciliation/transactions/${withdrawalId}/cancel`, {});
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.goto('/reconciliation/paypal');
	const row = page.getByTestId(`paypal-to-review-${withdrawalId}`);
	await expect(row).toContainText('Déjà comptabilisée');
	await expect(page.getByTestId(`paypal-to-review-entry-${withdrawalId}`)).toHaveAttribute(
		'href',
		`/journal-entries/${entryId}`,
	);
	await captureOf(page, 'paypal-to-review', '14-deja-comptabilisee');
	// L'ancienne proposition, renvoyée : refusée, rien d'écrit.
	const ctx2 = await authedApiContext(page);
	try {
		const res = await api<{ accepted: unknown[]; failed: Array<{ errorCode: string }> }>(ctx2, 'post', '/api/v1/paypal/accept', {
			account: PAYPAL,
			proposals: [{ type: 'withdrawal', bankTransactionId: withdrawalId, counterpartyAccountId: destination }],
		});
		expect(res.accepted).toEqual([]);
		expect(res.failed.map((f) => f.errorCode)).toEqual(['PROVIDER_TRANSACTION_ALREADY_POSTED']);
	} finally {
		await disposeContextSafe(ctx2);
	}
});
