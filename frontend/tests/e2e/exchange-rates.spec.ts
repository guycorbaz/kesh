/**
 * Story 37-2 (AC-6) — l'écran *Paramètres → Cours de change*, de l'écran au serveur.
 *
 * Société multi-devises montée par l'API (`with-company-no-fy`, exercice civil de
 * l'année du jour, comptes de conversion et de différences de change, activation,
 * compte en USD). La source des cours est la source factice du mode test, pilotée
 * par `POST /api/v1/_test/exchange-rates/source` ; `_test/seed` la remet à son
 * défaut et vide les valeurs publiées (AC-7).
 *
 * 1. Un cours saisi dans la case de février (rien de publié) devient le cours
 *    retenu, que `lookup` propose ; « AFC : non publié » et ↑ « Retirer le cours
 *    saisi », qui le supprime (`lookup` → 404).
 * 2. Janvier récupéré (copie `afc`), puis récupéré de nouveau à une autre valeur :
 *    case « AFC corrigé » en rouge et bandeau ; ↑ direct → cours retenu changé,
 *    rouge disparu ; une autre correction (« Revérifier les cours publiés ») →
 *    « Acquitter » → bandeau disparu, rouge encore là.
 * 3. « Récupérer les cours manquants » remplit les cases récupérables (février,
 *    jour d'ouverture) par la copie des cours publiés.
 *
 * L'activation est irréversible : la spec reseede `with-company` en fin de
 * parcours, pour que les specs suivantes retrouvent une société mono-devise.
 */

import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { createFiscalYearViaApi, todayInZurich } from './helpers/api-fixtures';

test.describe.configure({ mode: 'serial' });

/** Année de l'exercice civil créé (celle du jour, à Zurich). */
const YEAR = todayInZurich().slice(0, 4);
const JAN = `${YEAR}-01-01`;
const FEB = `${YEAR}-02-01`;
/** Jour d'ouverture en genre `day` : la veille du 1er janvier. */
const OPENING_DAY = `${Number(YEAR) - 1}-12-31`;

/** Cours USD du XML mensuel par défaut de la source factice (`MOCK_MONTHLY_RATES`). */
const USD_DEFAULT = '0.8800';

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function post(ctx: APIRequestContext, path: string, data: unknown): Promise<unknown> {
	const res = await ctx.post(path, { data });
	expect(res.ok(), `POST ${path} : ${res.status()} ${await res.text()}`).toBeTruthy();
	return res.json();
}

/** XML `xmlavgmonth` d'un mois, avec le seul cours USD donné (une unité). */
function monthlyXml(month: string, usd: string): string {
	return (
		'<?xml version="1.0" encoding="UTF-8"?>' +
		'<monatsmittelkurs xmlns="https://www.backend-rates.bazg.admin.ch/avgratesmonth">' +
		`<monat>${month.slice(0, 7)}</monat>` +
		'<devise code="usd"><land_de/><land_fr/><land_it/>' +
		`<waehrung>1 USD</waehrung><kurs>${usd}</kurs></devise>` +
		'</monatsmittelkurs>'
	);
}

/** Pilote la source factice : le mois `month` rendra ce cours USD. */
async function pilotMonth(ctx: APIRequestContext, month: string, usd: string): Promise<void> {
	await post(ctx, '/api/v1/_test/exchange-rates/source', {
		kind: 'month',
		reply: 'xml',
		xml: monthlyXml(month, usd),
		date: month,
	});
}

/** Monte la société multi-devises : exercice de l'année, trois comptes, activation, USD. */
async function setupMultiCurrency(page: Page): Promise<void> {
	await createFiscalYearViaApi(page);
	const ctx = await authedApiContext(page);
	try {
		const transit = (await post(ctx, '/api/v1/accounts', {
			number: '1090',
			name: 'Compte de conversion',
			accountType: 'Asset',
		})) as { id: number };
		const fx = (await post(ctx, '/api/v1/accounts', {
			number: '6840',
			name: 'Différences de change',
			accountType: 'Expense',
		})) as { id: number };
		const current = await ctx.get('/api/v1/companies/current');
		expect(current.ok()).toBeTruthy();
		const version = ((await current.json()) as { company: { version: number } }).company
			.version;
		const activated = await ctx.put('/api/v1/companies/current/currency-settings', {
			data: {
				multiCurrencyEnabled: true,
				fxGainAccountId: fx.id,
				fxLossAccountId: fx.id,
				fxTransitAccountId: transit.id,
				version,
			},
		});
		expect(activated.ok(), `activation : ${activated.status()} ${await activated.text()}`).toBe(
			true,
		);
		await post(ctx, '/api/v1/accounts', {
			number: '1025',
			name: 'Banque USD',
			accountType: 'Asset',
			currency: 'USD',
		});
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function openScreen(page: Page): Promise<void> {
	await page.goto('/settings/exchange-rates');
	await expect(page.getByTestId('exchange-rates-table')).toBeVisible();
	await expect(page.getByTestId('exchange-rates-row-USD')).toBeVisible();
}

/** Vrai si une couleur CSS `rgb(r, g, b)` est rouge (r nettement au-dessus de g et b). */
function isRed(color: string): boolean {
	const m = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
	if (!m) return false;
	const [r, g, b] = [Number(m[1]), Number(m[2]), Number(m[3])];
	return r > 150 && r > g + 60 && r > b + 60;
}

test.beforeAll(async ({ browser }) => {
	await seedTestState('with-company-no-fy');
	const page = await browser.newPage();
	try {
		await login(page);
		await setupMultiCurrency(page);
	} finally {
		await clearAuthStorage(page);
		await page.close();
	}
});

test.afterAll(async () => {
	// L'activation ne se défait pas : seule la remise à zéro rend le mono-devise.
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('un cours saisi devient le cours retenu proposé, ↑ le retire', async ({ page }) => {
	await login(page);
	await openScreen(page);
	const key = `USD-month-${FEB}`;
	const input = page.getByTestId(`exchange-rates-input-${key}`);
	await expect(input).toHaveValue('');
	await expect(page.getByTestId(`exchange-rates-mention-${key}`)).toHaveAttribute(
		'data-kind',
		'none',
	);

	const put = page.waitForResponse(
		(r) => r.url().includes('/api/v1/exchange-rates/rates') && r.request().method() === 'PUT',
	);
	await input.fill('0.9123');
	await input.press('Enter');
	expect((await put).status()).toBe(200);
	await expect(input).toHaveValue('0.9123');
	const mention = page.getByTestId(`exchange-rates-mention-${key}`);
	await expect(mention).toHaveAttribute('data-kind', 'saisi-unpublished');
	await expect(mention).toHaveText('AFC : non publié');
	const arrow = page.getByTestId(`exchange-rates-arrow-${key}`);
	await expect(arrow).toHaveAttribute('data-arrow', 'remove');
	await expect(arrow).toHaveAttribute('aria-label', 'Retirer le cours saisi');

	// Proposé : `lookup` rend le cours saisi, source `manual`.
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.get(`/api/v1/exchange-rates/lookup?currency=USD&date=${FEB}`);
		expect(res.status(), await res.text()).toBe(200);
		const body = (await res.json()) as { effectiveRate: string; source: string };
		expect(Number(body.effectiveRate)).toBe(0.9123);
		expect(body.source).toBe('manual');
	} finally {
		await disposeContextSafe(ctx);
	}

	const reset = page.waitForResponse(
		(r) => r.url().includes('/api/v1/exchange-rates/rates/reset') && r.request().method() === 'POST',
	);
	await arrow.click();
	expect((await reset).status()).toBe(200);
	await expect(input).toHaveValue('');
	await expect(mention).toHaveAttribute('data-kind', 'none');

	const after = await authedApiContext(page);
	try {
		const res = await after.get(`/api/v1/exchange-rates/lookup?currency=USD&date=${FEB}`);
		expect(res.status(), 'cours retiré : plus rien à proposer').toBe(404);
	} finally {
		await disposeContextSafe(after);
	}
});

test('correction de l’AFC : rouge et bandeau, ↑ direct, puis acquittement', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	try {
		// Janvier récupéré au cours par défaut : copié dans la case (ligne `afc`).
		await post(ctx, '/api/v1/exchange-rates/fetch', { year: Number(YEAR), month: 1 });
		// L'AFC republie janvier à une autre valeur.
		await pilotMonth(ctx, JAN, '0.8850');
		const fetched = (await post(ctx, '/api/v1/exchange-rates/fetch', {
			year: Number(YEAR),
			month: 1,
		})) as { corrected: number };
		expect(fetched.corrected).toBe(1);
	} finally {
		await disposeContextSafe(ctx);
	}

	await openScreen(page);
	const key = `USD-month-${JAN}`;
	const input = page.getByTestId(`exchange-rates-input-${key}`);
	const mention = page.getByTestId(`exchange-rates-mention-${key}`);
	// La société garde son cours ; la case dit la correction, en rouge.
	await expect(input).toHaveValue(USD_DEFAULT);
	await expect(mention).toHaveAttribute('data-kind', 'afc-corrected');
	await expect(mention).toContainText('AFC corrigé : 0.8850');
	const color = await mention.evaluate((el) => getComputedStyle(el).color);
	expect(isRed(color), `couleur de « AFC corrigé » : ${color}`).toBe(true);
	const banner = page.getByTestId('exchange-rate-notices-banner');
	await expect(banner).toBeVisible();
	await expect(page.getByTestId(`exchange-rate-notice-USD-month-${JAN}`)).toBeVisible();

	// ↑ direct, sans dialogue (A23) : la valeur corrigée devient le cours retenu.
	const arrow = page.getByTestId(`exchange-rates-arrow-${key}`);
	await expect(arrow).toHaveAttribute('data-arrow', 'corrected');
	const reset = page.waitForResponse(
		(r) => r.url().includes('/api/v1/exchange-rates/rates/reset') && r.request().method() === 'POST',
	);
	await arrow.click();
	expect((await reset).status()).toBe(200);
	await expect(page.getByTestId('exchange-rates-correction-dialog')).toHaveCount(0);
	await expect(input).toHaveValue('0.8850');
	await expect(mention).toHaveAttribute('data-kind', 'afc-ok');
	await expect(page.getByTestId(`exchange-rates-arrow-${key}`)).toHaveCount(0);

	// Une autre correction, découverte par « Revérifier les cours publiés ».
	const pilot = await authedApiContext(page);
	try {
		await pilotMonth(pilot, JAN, '0.8870');
	} finally {
		await disposeContextSafe(pilot);
	}
	await page.getByTestId('exchange-rates-recheck').click();
	await expect(mention).toHaveAttribute('data-kind', 'afc-corrected');
	await expect(mention).toContainText('AFC corrigé : 0.8870');
	await expect(input).toHaveValue('0.8850');

	// Le bandeau se lit au montage : rechargement, puis « Acquitter ».
	await openScreen(page);
	await expect(banner).toBeVisible();
	const ack = page.waitForResponse(
		(r) =>
			r.url().includes('/api/v1/exchange-rates/notices/acknowledge') &&
			r.request().method() === 'POST',
	);
	await page.getByTestId('exchange-rate-notices-banner-acknowledge').click();
	expect((await ack).status()).toBe(200);
	await expect(banner).toHaveCount(0);
	// L'acquittement n'efface que l'avertissement : le rouge reste.
	await expect(mention).toHaveAttribute('data-kind', 'afc-corrected');

	await openScreen(page);
	await expect(page.getByTestId(`exchange-rates-mention-${key}`)).toHaveAttribute(
		'data-kind',
		'afc-corrected',
	);
	await expect(page.getByTestId('exchange-rate-notices-banner')).toHaveCount(0);
});

test('« Récupérer les cours manquants » remplit les cases récupérables', async ({ page }) => {
	await login(page);
	await openScreen(page);
	const febInput = page.getByTestId(`exchange-rates-input-USD-month-${FEB}`);
	const openingInput = page.getByTestId(`exchange-rates-input-USD-opening-${OPENING_DAY}`);
	await expect(febInput).toHaveValue('');
	await expect(openingInput).toHaveValue('');

	await page.getByTestId('exchange-rates-fetch-missing').click();
	await expect(page.getByTestId('exchange-rates-progress')).toHaveCount(0, { timeout: 60_000 });

	// Février n'est récupérable que s'il est au plus le mois suivant (toujours vrai,
	// sauf en janvier où il est justement le mois suivant).
	await expect(febInput).toHaveValue(USD_DEFAULT);
	await expect(page.getByTestId(`exchange-rates-mention-USD-month-${FEB}`)).toHaveAttribute(
		'data-kind',
		'afc-ok',
	);
	// Jour d'ouverture (genre `day` par défaut) : cours journalier par défaut.
	await expect(openingInput).toHaveValue('0.8912');
	await expect(
		page.getByTestId(`exchange-rates-mention-USD-opening-${OPENING_DAY}`),
	).toHaveAttribute('data-kind', 'afc-ok');
	// Janvier, présent, n'est pas redemandé : sa correction non reprise reste en rouge.
	await expect(page.getByTestId(`exchange-rates-mention-USD-month-${JAN}`)).toHaveAttribute(
		'data-kind',
		'afc-corrected',
	);
});
