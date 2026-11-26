/**
 * Story 37-4 (AC-2, AC-3, AC-4) — les rapports d'un compte en devise, de la base
 * à l'écran.
 *
 * Société passée en multi-devises par l'API (comptes de conversion et de
 * différences de change créés d'abord), un compte USD, une écriture en devise
 * (USD 1234.56 au cours 0.91193 = CHF 1125.83, contre un compte CHF) passée par
 * l'API. Le grand livre du compte USD montre sa devise, la seconde ligne du
 * mouvement et les soldes en devise ; la balance et le bilan portent le solde en
 * devise ; puis une vente USD sur un compte de produits USD, lue au compte de
 * résultat et au journal (revue de code P1). La valeur traverse ainsi la
 * frontière HTTP (JSON omis hors devise,
 * présent ici).
 *
 * L'activation est irréversible : la spec repart d'un `with-company` neuf et en
 * reseede un en fin de parcours.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { todayInZurich } from './helpers/api-fixtures';

test.describe.configure({ mode: 'serial' });

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

const USD_NUMBER = '1026';
const USD_REVENUE_NUMBER = '3206';

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function createAccount(
	ctx: APIRequestContext,
	data: Record<string, unknown>,
): Promise<number> {
	const res = await ctx.post('/api/v1/accounts', { data });
	expect(res.ok(), `compte ${data.number} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return ((await res.json()) as { id: number }).id;
}

async function accountId(ctx: APIRequestContext, number: string): Promise<number> {
	const res = await ctx.get('/api/v1/accounts');
	const found = ((await res.json()) as { id: number; number: string }[]).find(
		(a) => a.number === number,
	);
	expect(found, `compte ${number}`).toBeTruthy();
	return found!.id;
}

test('grand livre, balance, bilan, compte de résultat et journal d’un compte USD', async ({ page }) => {
	await login(page);
	const today = todayInZurich();
	const year = today.slice(0, 4);
	const ctx = await authedApiContext(page);
	try {
		const transit = await createAccount(ctx, {
			number: '1091',
			name: 'Conversion',
			accountType: 'Asset',
		});
		const fx = await createAccount(ctx, {
			number: '6841',
			name: 'Différences de change',
			accountType: 'Expense',
		});
		const company = await ctx.get('/api/v1/companies/current');
		const version = ((await company.json()) as { company: { version: number } }).company.version;
		const activate = await ctx.put('/api/v1/companies/current/currency-settings', {
			data: {
				multiCurrencyEnabled: true,
				fxGainAccountId: fx,
				fxLossAccountId: fx,
				fxTransitAccountId: transit,
				version,
			},
		});
		expect(activate.ok(), `activation : ${activate.status()} ${await activate.text()}`).toBeTruthy();

		const usd = await createAccount(ctx, {
			number: USD_NUMBER,
			name: 'Banque USD',
			accountType: 'Asset',
			currency: 'USD',
		});
		const capital = await accountId(ctx, '2000');
		const entry = await ctx.post('/api/v1/journal-entries', {
			data: {
				entryDate: today,
				journal: 'OD',
				description: 'E2E 37-4 — apport en USD',
				lines: [
					{
						accountId: usd,
						debit: '1125.83',
						credit: '0',
						foreign: { amount: '1234.56', rate: '0.91193', rateSource: 'manual' },
					},
					{ accountId: capital, debit: '0', credit: '1125.83' },
				],
			},
		});
		expect(entry.ok(), `écriture : ${entry.status()} ${await entry.text()}`).toBeTruthy();

		// Grand livre.
		await page.goto('/reports');
		await page.waitForLoadState('networkidle');
		await page.locator('#reports-tab-general-ledger').click();
		await expect(page.getByTestId('ledger-controls')).toBeVisible();
		await page.getByTestId('ledger-from').fill(`${year}-01-01`);
		await page.getByTestId('ledger-to').fill(`${year}-12-31`);
		await page.getByTestId('ledger-account').selectOption(String(usd));
		await page.getByTestId('ledger-generate').click();
		const section = page.getByTestId(`ledger-section-${USD_NUMBER}`);
		await expect(section).toBeVisible({ timeout: 5000 });
		await expect(page.getByTestId(`ledger-currency-${USD_NUMBER}`)).toHaveText('USD');
		await expect(section).toContainText('1’125.83');
		await expect(section).toContainText('USD 1’234.56 × 0.91193 · solde USD 1’234.56');
		await expect(page.getByTestId('ledger-opening-fx')).toHaveText('USD 0.00');
		await expect(page.getByTestId('ledger-closing-fx')).toHaveText('USD 1’234.56');
		await expect(page.getByTestId('ledger-totals-fx')).toHaveText(
			'débit USD 1’234.56 · crédit USD 0.00',
		);
		await page.screenshot({
			path: `${process.env.KESH_CAPTURES_DIR ?? 'test-results'}/37-4-grand-livre-usd.png`,
			fullPage: true,
		});
	} finally {
		await disposeContextSafe(ctx);
	}

	// Balance : seconde ligne en devise sous le compte USD.
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	// Story 55-1a (D16) — plus de *Générer* : le clic sur l'onglet charge le rapport.
	await page.locator('#reports-tab-trial-balance').click();
	await expect(page.getByTestId(`tb-fx-row-${USD_NUMBER}`)).toContainText(
		'USD ouverture 0.00 · débit 1’234.56 · crédit 0.00 · clôture 1’234.56',
		{ timeout: 5000 },
	);
	await page.screenshot({
		path: `${process.env.KESH_CAPTURES_DIR ?? 'test-results'}/37-4-balance-usd.png`,
		fullPage: true,
	});

	// Bilan : « USD 1’234.56 » à côté du nom.
	await page.locator('#reports-tab-balance-sheet').click();
	await expect(page.getByTestId(`account-fx-${USD_NUMBER}`)).toHaveText('USD 1’234.56', {
		timeout: 5000,
	});
	await page.screenshot({
		path: `${process.env.KESH_CAPTURES_DIR ?? 'test-results'}/37-4-bilan-usd.png`,
		fullPage: true,
	});

	// Revue de code P1 — compte de résultat et journal en devise : un compte de
	// produits USD, une vente USD 100 au cours 0.9119 = CHF 91.19.
	const ctx2 = await authedApiContext(page);
	try {
		const usd = await accountId(ctx2, USD_NUMBER);
		const ventes = await createAccount(ctx2, {
			number: USD_REVENUE_NUMBER,
			name: 'Ventes USD',
			accountType: 'Revenue',
			currency: 'USD',
		});
		const foreign = { amount: '100', rate: '0.9119', rateSource: 'manual' };
		const sale = await ctx2.post('/api/v1/journal-entries', {
			data: {
				entryDate: today,
				journal: 'Ventes',
				description: 'E2E 37-4 — vente en USD',
				lines: [
					{ accountId: usd, debit: '91.19', credit: '0', foreign },
					{ accountId: ventes, debit: '0', credit: '91.19', foreign },
				],
			},
		});
		expect(sale.ok(), `vente : ${sale.status()} ${await sale.text()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx2);
	}

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	await page.locator('#reports-tab-income-statement').click();
	await expect(page.getByTestId(`account-fx-${USD_REVENUE_NUMBER}`)).toHaveText('USD 100.00', {
		timeout: 5000,
	});
	await page.screenshot({
		path: `${process.env.KESH_CAPTURES_DIR ?? 'test-results'}/37-4-resultat-usd.png`,
		fullPage: true,
	});

	await page.locator('#reports-tab-journals').click();
	await expect(page.getByTestId('journal-fx-line').first()).toBeVisible({ timeout: 5000 });
	await expect(
		page.getByTestId('journal-fx-line').filter({ hasText: 'USD 100.00 × 0.9119' }),
	).toHaveCount(2);
	await page.screenshot({
		path: `${process.env.KESH_CAPTURES_DIR ?? 'test-results'}/37-4-journal-usd.png`,
		fullPage: true,
	});
});
