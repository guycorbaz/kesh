/**
 * Story 37-3b (AC-5) — les écritures en devise à l'écran, de l'écran au serveur.
 *
 * Société `with-company` (exercice ouvert au 01.01.2020) passée en multi-devises
 * par l'API, comptes de gains (6840), de pertes (6940) et de conversion (1090)
 * désignés ; un compte USD « par défaut » (1025), un compte USD à comptes
 * **propres** (1026 : 6841 / 6941), un compte EUR (1027). Les cours sont servis
 * par la source factice du mode test (cours par défaut : USD 0.8800 mensuel,
 * 0.89120 journalier ; EUR 0.9300) ; `_test/seed` vide la table des cours, si
 * bien que le premier `lookup` de chaque mois rend 404 et que l'écran exerce son
 * repli (`POST /exchange-rates/fetch`, copie dans les cours de la société,
 * relecture).
 *
 * 1. Soldes d'ouverture d'un compte USD : cours journalier de la veille
 *    (31.12.2019) proposé, puis, l'exercice passé en genre `month`, cours mensuel
 *    de décembre 2019 ; l'ouverture s'enregistre (devise du compte, `manual`).
 * 2. Écriture USD au cours AFC (cours proposé, CHF calculé, `afc` relu).
 * 3. Ligne d'écart par le bouton : compte « par défaut » → comptes de la
 *    société ; compte à comptes propres → ses comptes.
 * 4. Saisie du CHF au lieu du cours ; changement de compte USD → EUR (montant et
 *    cours repartent à vide, cours EUR proposé, l'écriture s'enregistre en EUR) ;
 *    le sélecteur n'offre pas de compte d'une autre devise (R7).
 * 5. Achat USD avec TVA 8,1 % par l'assistant (ligne d'impôt préalable en CHF au
 *    cours de l'achat, écart d'un centime soldé par le bouton d'écart).
 * 6. Conversion EUR → USD par l'assistant (deux écritures, compte de conversion à 0).
 * 7. Écriture à deux devises refusée par l'API (`FX_MULTIPLE_CURRENCIES`).
 * 8. Cours saisi par la société (`PUT /exchange-rates/rates`) : proposé, « cours
 *    saisi », la ligne part `manual` ; après le ↑ de la table des cours, `afc`.
 * 9. Cours `manual` apparu après l'écriture : modifier le seul montant d'une
 *    ligne `afc` → `FX_RATE_NOT_AFC` sans cours attendu, « Garder mon cours ».
 * 10. Story 53-1b (#1365, AC-7) : une conversion dont la réponse est perdue après le
 *    `COMMIT` (K6, rien n'est renvoyé), reprise sans requête, puis le doublon refusé (K5) et
 *    sa confirmation ; le journal porte deux écritures de CETTE conversion, non quatre.
 *
 * L'activation est irréversible : la spec reseede `with-company` en fin de parcours.
 * Captures d'écran (thème clair) dans `KESH_CAPTURES_DIR` s'il est défini.
 */

import { expect, test, type APIRequestContext, type Page, type Request } from '@playwright/test';
import { join } from 'node:path';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

test.describe.configure({ mode: 'serial' });

const CAPTURES = process.env.KESH_CAPTURES_DIR ?? process.env.KESH_E2E_CAPTURES_DIR;

async function capture(page: Page, name: string): Promise<void> {
	if (CAPTURES) await page.screenshot({ path: join(CAPTURES, `${name}.png`), fullPage: true });
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
	usd: number;
	usdOwn: number;
	eur: number;
	transit: number;
	gain: number;
	loss: number;
	ownGain: number;
	ownLoss: number;
	equity: number;
	cash: number;
	expense: number;
}
let ids: Ids;

async function setup(page: Page): Promise<Ids> {
	const ctx = await authedApiContext(page);
	try {
		const create = async (number: string, name: string, accountType: string, extra: object = {}) =>
			(await api<{ id: number }>(ctx, 'post', '/api/v1/accounts', { number, name, accountType, ...extra })).id;
		const transit = await create('1090', 'Compte de conversion', 'Asset');
		const gain = await create('6840', 'Gains de change', 'Revenue');
		const loss = await create('6940', 'Pertes de change', 'Expense');
		const ownGain = await create('6841', 'Gains de change PayPal', 'Revenue');
		const ownLoss = await create('6941', 'Pertes de change PayPal', 'Expense');
		const company = await api<{ company: { version: number } }>(ctx, 'get', '/api/v1/companies/current');
		await api(ctx, 'put', '/api/v1/companies/current/currency-settings', {
			multiCurrencyEnabled: true,
			fxGainAccountId: gain,
			fxLossAccountId: loss,
			fxTransitAccountId: transit,
			version: company.company.version,
		});
		const usd = await create('1025', 'Banque USD', 'Asset', { currency: 'USD' });
		const usdOwn = await create('1026', 'PayPal USD', 'Asset', {
			currency: 'USD',
			fxGainAccountId: ownGain,
			fxLossAccountId: ownLoss,
		});
		const eur = await create('1027', 'Banque EUR', 'Asset', { currency: 'EUR' });
		const accounts = await api<Array<{ id: number; number: string }>>(ctx, 'get', '/api/v1/accounts');
		const byNumber = (n: string) => accounts.find((a) => a.number === n)!.id;
		return {
			usd,
			usdOwn,
			eur,
			transit,
			gain,
			loss,
			ownGain,
			ownLoss,
			equity: byNumber('2000'),
			cash: byNumber('1100'),
			expense: byNumber('4000'),
		};
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Choisit le compte `number` dans le `index`-ième sélecteur de compte de la page. */
async function pickAccount(page: Page, index: number, number: string): Promise<void> {
	const input = page.locator('input[aria-autocomplete="list"]').nth(index);
	await input.fill(number);
	await page.getByRole('listbox').getByRole('option').filter({ hasText: number }).first().click();
}

async function openNewEntry(page: Page, date: string, description: string): Promise<void> {
	await page.goto('/journal-entries');
	await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
	await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();
	await page.fill('#entry-date', date);
	await page.fill('#entry-description', description);
}

async function submitEntry(page: Page): Promise<number> {
	const created = page.waitForResponse(
		(r) => r.url().endsWith('/api/v1/journal-entries') && r.request().method() === 'POST',
	);
	await page.getByTestId('journal-entry-submit').click();
	const res = await created;
	expect(res.status(), await res.text()).toBe(201);
	return ((await res.json()) as { id: number }).id;
}

async function getEntry(page: Page, id: number) {
	const ctx = await authedApiContext(page);
	try {
		return await api<{
			version: number;
			entryDate: string;
			lines: Array<{
				accountId: number;
				debit: string;
				credit: string;
				foreign?: { currency: string; amount: string; rate: string; rateSource: string };
			}>;
		}>(ctx, 'get', `/api/v1/journal-entries/${id}`);
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.beforeAll(async ({ browser }) => {
	await seedTestState('with-company');
	const page = await browser.newPage();
	try {
		await login(page);
		ids = await setup(page);
	} finally {
		await clearAuthStorage(page);
		await page.close();
	}
});

test.afterAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('1 — soldes d’ouverture USD : cours journalier de la veille, puis mensuel de décembre 2019', async ({ page }) => {
	await login(page);
	await page.goto('/settings/opening-balances');
	await expect(page.getByTestId('opening-balances-grid')).toBeVisible();
	const rate = page.getByTestId('opening-balances-fx-rate-1025');
	// Genre `day` (défaut) : la veille de l'ouverture, cours journalier (repli du fetch).
	await expect(rate).toHaveValue('0.8912');
	await expect(page.getByTestId('opening-balances-fx-provenance-1025')).toContainText(
		'cours journalier du 31.12.2019',
	);
	await capture(page, '01-ouverture-genre-day');

	// Le genre d'ouverture passe à `month` (le genre de clôture reste celui lu).
	const ctx = await authedApiContext(page);
	try {
		const years = await api<Array<{ id: number; startDate: string; closingRateKind: string }>>(
			ctx,
			'get',
			'/api/v1/fiscal-years',
		);
		const fy = years.find((y) => y.startDate === '2020-01-01')!;
		await api(ctx, 'put', `/api/v1/fiscal-years/${fy.id}/rate-kinds`, {
			openingRateKind: 'month',
			closingRateKind: fy.closingRateKind,
		});
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.reload();
	await expect(rate).toHaveValue('0.88');
	await expect(page.getByTestId('opening-balances-fx-provenance-1025')).toContainText('AFC 12.2019');
	await page.getByTestId('opening-balances-fx-debit-1025').fill('1000');
	await expect(page.getByTestId('opening-balances-fx-chf-1025')).toHaveValue('880.00');
	await page.getByTestId('opening-balances-credit-2000').fill('880.00');
	await capture(page, '02-ouverture-genre-month');
	const generated = page.waitForResponse(
		(r) => r.url().endsWith('/api/v1/opening-balances') && r.request().method() === 'POST',
	);
	await page.getByTestId('opening-balances-generate').click();
	const res = await generated;
	expect(res.status(), await res.text()).toBe(201);
	const body = (await res.json()) as { lines: Array<{ accountId: number; foreign?: { currency: string; rateSource: string } }> };
	const usdLine = body.lines.find((l) => l.accountId === ids.usd)!;
	expect(usdLine.foreign).toMatchObject({ currency: 'USD', rateSource: 'manual' });
});

test('2 — écriture USD au cours AFC proposé', async ({ page }) => {
	await login(page);
	await openNewEntry(page, '2026-03-15', 'Encaissement USD');
	await pickAccount(page, 0, '1025');
	await expect(page.getByTestId('journal-entry-line-fx-rate-0')).toHaveValue('0.88');
	await expect(page.getByTestId('journal-entry-line-fx-indicator-0')).toHaveText('AFC');
	await page.getByTestId('journal-entry-line-fx-debit-0').fill('100');
	await expect(page.getByTestId('journal-entry-line-fx-chf-0')).toHaveValue('88.00');
	await pickAccount(page, 1, '3000');
	await page.getByTestId('journal-entry-line-credit-1').fill('88.00');
	await capture(page, '03-ecriture-usd-afc');
	const id = await submitEntry(page);
	const entry = await getEntry(page, id);
	expect(entry.lines[0].foreign).toEqual({ currency: 'USD', amount: '100', rate: '0.88', rateSource: 'afc' });
});

test('3 — ligne d’écart : comptes de la société, puis comptes propres', async ({ page }) => {
	await login(page);
	for (const [account, expected] of [
		['1025', ids.loss],
		['1026', ids.ownLoss],
	] as const) {
		await openNewEntry(page, '2026-03-16', `Écart ${account}`);
		await pickAccount(page, 0, account);
		await expect(page.getByTestId('journal-entry-line-fx-rate-0')).toHaveValue('0.88');
		await page.getByTestId('journal-entry-line-fx-debit-0').fill('100');
		await pickAccount(page, 1, '3000');
		await page.getByTestId('journal-entry-line-credit-1').fill('90.00');
		await page.getByTestId('journal-entry-add-fx-difference').click();
		await expect(page.getByTestId('journal-entry-fx-difference-note')).toHaveCount(0);
		if (account === '1026') await capture(page, '04-ligne-ecart-comptes-propres');
		const id = await submitEntry(page);
		const entry = await getEntry(page, id);
		expect(entry.lines).toHaveLength(3);
		expect(entry.lines[2]).toMatchObject({ accountId: expected, debit: '2.0000' });
	}
});

test('4 — CHF saisi au lieu du cours ; USD → EUR ; une seule devise au sélecteur', async ({ page }) => {
	await login(page);
	await openNewEntry(page, '2026-03-17', 'Changement de compte');
	await pickAccount(page, 0, '1025');
	await page.getByTestId('journal-entry-line-fx-debit-0').fill('100');
	// R7 : l'autre ligne n'offre pas le compte EUR.
	await page.locator('input[aria-autocomplete="list"]').nth(1).fill('1027');
	await expect(page.getByRole('listbox').getByRole('option').filter({ hasText: '1027' })).toHaveCount(0);
	await page.locator('input[aria-autocomplete="list"]').nth(1).fill('');
	await expect(page.getByTestId('journal-entry-conversion-link')).toBeVisible();
	// CHF au lieu du cours.
	await page.getByTestId('journal-entry-line-fx-chf-0').fill('91.19');
	await expect(page.getByTestId('journal-entry-line-fx-rate-0')).toHaveValue('0.9119');
	await capture(page, '05-chf-saisi');
	// USD → EUR : montant vide, cours EUR proposé.
	await pickAccount(page, 0, '1027');
	await expect(page.getByTestId('journal-entry-line-fx-debit-0')).toHaveValue('');
	await expect(page.getByTestId('journal-entry-line-fx-rate-0')).toHaveValue('0.93');
	await page.getByTestId('journal-entry-line-fx-debit-0').fill('100');
	await pickAccount(page, 1, '3000');
	await page.getByTestId('journal-entry-line-credit-1').fill('93.00');
	const id = await submitEntry(page);
	const entry = await getEntry(page, id);
	expect(entry.lines[0].foreign).toMatchObject({ currency: 'EUR', amount: '100', rate: '0.93' });
});

test('5 — achat USD avec TVA 8,1 % par l’assistant, centime soldé par le bouton d’écart', async ({ page }) => {
	await login(page);
	await openNewEntry(page, '2026-03-18', '');
	await page.getByRole('button', { name: /Assistant TVA achat/ }).click();
	const panel = page.getByTestId('vat-purchase-assistant');
	await pickAccount(page, 0, '4000');
	await pickAccount(page, 1, '1025');
	await expect(panel.getByTestId('vat-purchase-fx-rate')).toHaveValue('0.88');
	await panel.getByTestId('vat-purchase-fx-rate').fill('0.8850');
	await panel.getByPlaceholder('0.00', { exact: true }).fill('12.29');
	await panel.getByTestId('vat-purchase-rate').click();
	await page.getByRole('option', { name: /8\.10/ }).click();
	await capture(page, '06-assistant-tva-usd');
	await panel.getByTestId('vat-purchase-insert').click();
	// TVA CHF 0.89, charge 10.88, contrepartie USD 13.29 → 11.76 : un centime d'écart.
	await expect(page.getByTestId('journal-entry-fx-difference-note')).toContainText('arrondi');
	await page.getByTestId('journal-entry-add-fx-difference').click();
	await capture(page, '07-assistant-tva-centime-solde');
	const id = await submitEntry(page);
	const entry = await getEntry(page, id);
	expect(entry.lines.map((l) => l.debit !== '0.0000' ? l.debit : l.credit)).toEqual([
		'10.8800',
		'0.8900',
		'11.7600',
		'0.0100',
	]);
	expect(entry.lines[2].foreign).toMatchObject({ currency: 'USD', amount: '13.29', rate: '0.885' });
});

test('6 — conversion EUR → USD par l’assistant : deux écritures, compte de conversion à 0', async ({ page }) => {
	await login(page);
	await page.goto('/journal-entries');
	await page.getByTestId('journal-entries-conversion').click();
	await expect(page.getByTestId('conversion-assistant')).toBeVisible();
	await page.getByTestId('conversion-debit-date').fill('2026-03-05');
	await page.getByTestId('conversion-credit-date').fill('2026-03-06');
	const debitCol = page.getByTestId('conversion-column-debit');
	const creditCol = page.getByTestId('conversion-column-credit');
	await debitCol.locator('input[aria-autocomplete="list"]').fill('1027');
	await page.getByRole('listbox').getByRole('option').filter({ hasText: '1027' }).first().click();
	await creditCol.locator('input[aria-autocomplete="list"]').fill('1025');
	await page.getByRole('listbox').getByRole('option').filter({ hasText: '1025' }).first().click();
	await expect(page.getByTestId('conversion-debit-rate')).toHaveValue('0.93');
	await expect(page.getByTestId('conversion-credit-rate')).toHaveValue('0.88');
	await page.getByTestId('conversion-debit-amount').fill('1000');
	await page.getByTestId('conversion-credit-amount').fill('1060');
	// 1000 × 0.93 = 930.00 ; 1060 × 0.88 = 932.80 : gain 2.80.
	await expect(page.getByTestId('conversion-preview-2-difference')).toContainText('2.80');
	await capture(page, '08-assistant-conversion');
	const posted = page.waitForResponse((r) => r.url().endsWith('/api/v1/journal-entries/conversions'));
	await page.getByTestId('conversion-submit').click();
	const res = await posted;
	expect(res.status(), await res.text()).toBe(201);
	await expect(page).toHaveURL(/\/journal-entries$/);
	const ctx = await authedApiContext(page);
	try {
		const list = await api<{ items: Array<{ lines: Array<{ accountId: number; debit: string; credit: string }> }> }>(
			ctx,
			'get',
			`/api/v1/journal-entries?accountId=${ids.transit}&limit=50`,
		);
		const net = list.items
			.flatMap((e) => e.lines)
			.filter((l) => l.accountId === ids.transit)
			.reduce((s, l) => s + Number(l.debit) - Number(l.credit), 0);
		expect(Math.abs(net)).toBeLessThan(0.0001);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('7 — une écriture à deux devises est refusée par l’API', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/journal-entries', {
			data: {
				entryDate: '2026-03-20',
				journal: 'OD',
				description: 'Deux devises',
				lines: [
					{ accountId: ids.usd, debit: '88.00', credit: '0', foreign: { currency: 'USD', amount: '100', rate: '0.88', rateSource: 'manual' } },
					{ accountId: ids.eur, debit: '0', credit: '88.00', foreign: { currency: 'EUR', amount: '94.6237', rate: '0.93', rateSource: 'manual' } },
				],
			},
		});
		expect(res.status()).toBe(400);
		expect(((await res.json()) as { error: { code: string } }).error.code).toBe('FX_MULTIPLE_CURRENCIES');
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('8 — cours saisi par la société : proposé, « cours saisi », manual ; après le ↑, afc', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const MONTH = '2026-04-01';
	try {
		// Le mois récupéré d'abord (copie `afc`, valeur publiée), puis un cours saisi.
		await api(ctx, 'post', '/api/v1/exchange-rates/fetch', { year: 2026, month: 4 });
		const list = await api<{ rates: Array<{ currency: string; row: { version: number } | null }> }>(
			ctx,
			'get',
			`/api/v1/exchange-rates?month=2026-04`,
		);
		const version = list.rates.find((r) => r.currency === 'USD')!.row!.version;
		await api(ctx, 'put', '/api/v1/exchange-rates/rates', {
			currency: 'USD',
			kind: 'month',
			rateDate: MONTH,
			rate: '0.9',
			version,
		});
	} finally {
		await disposeContextSafe(ctx);
	}
	await openNewEntry(page, '2026-04-10', 'Cours saisi');
	await pickAccount(page, 0, '1025');
	await expect(page.getByTestId('journal-entry-line-fx-rate-0')).toHaveValue('0.9');
	await expect(page.getByTestId('journal-entry-line-fx-indicator-0')).toHaveText('cours saisi');
	await page.getByTestId('journal-entry-line-fx-debit-0').fill('100');
	await pickAccount(page, 1, '3000');
	await page.getByTestId('journal-entry-line-credit-1').fill('90.00');
	await capture(page, '09-cours-saisi-societe');
	const manualId = await submitEntry(page);
	expect((await getEntry(page, manualId)).lines[0].foreign?.rateSource).toBe('manual');

	// ↑ de la table des cours : la ligne de la société repasse à la valeur publiée (afc).
	const ctx2 = await authedApiContext(page);
	try {
		const list = await api<{
			rates: Array<{ currency: string; row: { version: number } | null; published: { rate: string } | null }>;
		}>(ctx2, 'get', `/api/v1/exchange-rates?month=2026-04`);
		const usd = list.rates.find((r) => r.currency === 'USD')!;
		await api(ctx2, 'post', '/api/v1/exchange-rates/rates/reset', {
			currency: 'USD',
			kind: 'month',
			rateDate: MONTH,
			version: usd.row!.version,
			publishedRate: usd.published!.rate,
		});
	} finally {
		await disposeContextSafe(ctx2);
	}
	await openNewEntry(page, '2026-04-11', 'Après le ↑');
	await pickAccount(page, 0, '1025');
	await expect(page.getByTestId('journal-entry-line-fx-rate-0')).toHaveValue('0.88');
	await page.getByTestId('journal-entry-line-fx-debit-0').fill('100');
	await pickAccount(page, 1, '3000');
	await page.getByTestId('journal-entry-line-credit-1').fill('88.00');
	const afcId = await submitEntry(page);
	expect((await getEntry(page, afcId)).lines[0].foreign?.rateSource).toBe('afc');
});

test('9 — cours manual apparu après l’écriture : FX_RATE_NOT_AFC sans cours attendu, « Garder mon cours »', async ({ page }) => {
	await login(page);
	// Une écriture `afc` de mai, puis la société saisit son propre cours pour mai.
	await openNewEntry(page, '2026-05-12', 'Ligne AFC de mai');
	await pickAccount(page, 0, '1025');
	await expect(page.getByTestId('journal-entry-line-fx-rate-0')).toHaveValue('0.88');
	await page.getByTestId('journal-entry-line-fx-debit-0').fill('100');
	await pickAccount(page, 1, '3000');
	await page.getByTestId('journal-entry-line-credit-1').fill('88.00');
	const id = await submitEntry(page);
	const ctx = await authedApiContext(page);
	try {
		const list = await api<{ rates: Array<{ currency: string; row: { version: number } | null }> }>(
			ctx,
			'get',
			`/api/v1/exchange-rates?month=2026-05`,
		);
		await api(ctx, 'put', '/api/v1/exchange-rates/rates', {
			currency: 'USD',
			kind: 'month',
			rateDate: '2026-05-01',
			rate: '0.95',
			version: list.rates.find((r) => r.currency === 'USD')!.row!.version,
		});
	} finally {
		await disposeContextSafe(ctx);
	}
	await page.goto(`/journal-entries/${id}`);
	await page.getByTestId('edit-entry').click();
	await page.getByTestId('journal-entry-line-fx-debit-0').fill('200');
	await page.getByTestId('journal-entry-line-credit-1').fill('176.00');
	const refused = page.waitForResponse((r) => r.url().includes(`/api/v1/journal-entries/${id}`) && r.request().method() === 'PUT');
	await page.getByTestId('journal-entry-submit').click();
	expect((await refused).status()).toBe(400);
	await expect(page.getByTestId('journal-entry-line-refusal-0')).toContainText('propre cours');
	await expect(page.getByTestId('journal-entry-line-take-expected-0')).toHaveCount(0);
	await capture(page, '10-rate-not-afc-garder');
	await page.getByTestId('journal-entry-line-keep-rate-0').click();
	const saved = page.waitForResponse((r) => r.url().includes(`/api/v1/journal-entries/${id}`) && r.request().method() === 'PUT');
	await page.getByTestId('journal-entry-submit').click();
	expect((await saved).status()).toBe(200);
	const entry = await getEntry(page, id);
	expect(entry.lines[0].foreign).toMatchObject({ amount: '200', rate: '0.88', rateSource: 'manual' });
	await page.goto(`/journal-entries/${id}`);
	await expect(page.getByTestId(/^entry-line-fx-/).first()).toContainText('USD 200.00 × 0.88 · manuel');
	await capture(page, '11-fiche-ligne-devise');
});

test('10 — la conversion ne passe pas deux fois : réponse perdue (K6), reprise sans requête, doublon (K5), confirmation', async ({
	page,
}) => {
	await login(page);
	// Dates et montants **distincts** de ceux du test 6 (2026-03-05/06, EUR 1000 / USD 1060) : le
	// libellé de la conversion est calculé par l'assistant, il ne distingue pas deux conversions ;
	// ce décor est partagé (mode `serial`), le journal porte déjà les deux écritures du test 6.
	const D1 = '2026-03-12';
	const D2 = '2026-03-13';
	await page.goto('/journal-entries');
	await page.getByTestId('journal-entries-conversion').click();
	await expect(page.getByTestId('conversion-assistant')).toBeVisible();
	await page.getByTestId('conversion-debit-date').fill(D1);
	await page.getByTestId('conversion-credit-date').fill(D2);
	await page.getByTestId('conversion-column-debit').locator('input[aria-autocomplete="list"]').fill('1027');
	await page.getByRole('listbox').getByRole('option').filter({ hasText: '1027' }).first().click();
	await page.getByTestId('conversion-column-credit').locator('input[aria-autocomplete="list"]').fill('1025');
	await page.getByRole('listbox').getByRole('option').filter({ hasText: '1025' }).first().click();
	await expect(page.getByTestId('conversion-debit-rate')).toHaveValue('0.93');
	await expect(page.getByTestId('conversion-credit-rate')).toHaveValue('0.88');
	await page.getByTestId('conversion-debit-amount').fill('500');
	await page.getByTestId('conversion-credit-amount').fill('530');
	await expect(page.getByTestId('conversion-preview-2-difference')).toBeVisible();
	await capture(page, '12-k1-conversion-saisie');

	// Le serveur écrit, la réponse est jetée (la première fois seulement).
	const posts: Request[] = [];
	let cut = true;
	await page.route('**/api/v1/journal-entries/conversions', async (route) => {
		if (route.request().method() !== 'POST') return route.fallback();
		posts.push(route.request());
		if (!cut) return route.fallback();
		const response = await route.fetch();
		expect(response.status(), 'le serveur a bien écrit').toBe(201);
		await route.abort('connectionreset');
	});
	const submit = page.getByTestId('conversion-submit');
	const fields = page.getByTestId('conversion-debit-date');
	const ofThisConversion = async (): Promise<number> => {
		const ctx = await authedApiContext(page);
		try {
			const list = await api<{ items: Array<{ description: string }> }>(
				ctx,
				'get',
				`/api/v1/journal-entries?dateFrom=${D1}&dateTo=${D2}&limit=50`,
			);
			return list.items.filter((e) => /^Conversion EUR → USD/.test(e.description)).length;
		} finally {
			await disposeContextSafe(ctx);
		}
	};

	await submit.click();

	// K6 : figé, avis, lien sur la plage des deux dates, aucun message d'échec, aucun toast.
	const unknown = page.getByTestId('conversion-unknown-outcome');
	await expect(unknown).toBeVisible();
	await expect(page.getByTestId('conversion-unknown-outcome-text')).toHaveText(
		'La conversion a peut-être été passée. Vérifiez dans le journal si ses deux écritures existent avant de la passer de nouveau.',
	);
	// Aucun message d'échec (le faux « n'a pas pu être passée » de l'ancien écran) : ni erreur globale ni de colonne.
	await expect(page.getByTestId('conversion-global-error')).toHaveCount(0);
	await expect(page.getByTestId('conversion-debit-error')).toHaveCount(0);
	await expect(page.getByTestId('conversion-credit-error')).toHaveCount(0);
	const link = page.getByTestId('conversion-unknown-outcome-link');
	await expect(link).toHaveAttribute('href', `/journal-entries?dateFrom=${D1}&dateTo=${D2}`);
	await expect(link).toHaveAttribute('target', '_blank');
	await expect(fields).toBeDisabled();
	await expect(submit).toHaveCount(0);
	await expect(page.locator('[data-sonner-toast]')).toHaveCount(0);
	await capture(page, '13-k6-issue-inconnue');
	expect(await ofThisConversion(), 'le COMMIT a eu lieu : deux écritures de cette conversion').toBe(2);

	// La reprise rend la saisie intacte, sans requête.
	await page.getByTestId('conversion-resume-entry').click();
	expect(posts).toHaveLength(1);
	await expect(unknown).toHaveCount(0);
	await expect(fields).toBeEnabled();
	await expect(page.getByTestId('conversion-debit-amount')).toHaveValue('500');
	await expect(submit).toBeEnabled();

	// L'envoi suivant : le doublon (K5), aucune écriture de plus.
	cut = false;
	await submit.click();
	const dup = page.getByTestId('conversion-probable-duplicate');
	await expect(dup).toBeVisible();
	expect(posts).toHaveLength(2);
	await expect(page.getByTestId('conversion-probable-duplicate-text')).toHaveText(
		'L’écriture de la colonne Débit de cette conversion existe déjà à l’identique. Si la conversion est déjà passée, ne la passez pas une seconde fois ; sinon, confirmez que ce n’est pas un doublon.',
	);
	const cited = page.getByTestId('conversion-duplicate-link');
	await expect(cited).toHaveAttribute('href', /^\/journal-entries\/\d+$/);
	await expect(cited).toHaveAttribute('target', '_blank');
	await expect(cited).toContainText('Écriture n°');
	await expect(fields).toBeDisabled();
	await expect(submit).toHaveCount(0);
	await capture(page, '14-k5-doublon');
	expect(await ofThisConversion(), 'deux écritures de cette conversion, non quatre').toBe(2);

	// « Modifier la saisie » rend K1 ; l'envoi suivant est ordinaire (refusé encore).
	await page.getByTestId('conversion-modify-entry').click();
	await expect(fields).toBeEnabled();
	await submit.click();
	await expect(dup).toBeVisible();
	expect(posts).toHaveLength(3);
	expect(posts[2].postDataJSON().confirmNotDuplicate).toBeUndefined();

	// La confirmation renvoie le corps gardé avec confirmNotDuplicate, une requête.
	await page.getByTestId('conversion-confirm-not-duplicate').click();
	await expect(page).toHaveURL(/\/journal-entries$/);
	expect(posts).toHaveLength(4);
	expect(posts[3].postDataJSON().confirmNotDuplicate).toBe(true);
	expect(posts[3].postDataJSON().entries).toEqual(posts[0].postDataJSON().entries);
	expect(await ofThisConversion(), 'la seconde conversion, confirmée, s’ajoute').toBe(4);
});
