/**
 * Story 33-3b (#918) — les deux graphiques de l'accueil : les valeurs traversent la
 * frontière HTTP, le clic de souris et le clavier mènent au même détail.
 *
 * Montage : `with-company-no-fy`, puis un exercice civil de douze mois créé ici — le
 * preset `with-company` en sème un de 132 mois. ⚠️ La route lit `Utc::now()` : dans la
 * première ou les deux premières heures du 1er janvier à Zurich, elle lit encore l'an
 * passé et cette spec rougit (`noOpenFiscalYear`). Ce rouge relève de l'issue de la date
 * UTC, non de cette spec.
 *
 * Sélecteurs `data-testid` seulement (garde `e2e-selecteurs-traduits`, #326).
 */
import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {
	authedApiContext,
	clearAuthStorage,
	disposeContextSafe,
	seedTestState,
} from './helpers/test-state';
import {
	createExpenseEntryViaApi,
	createFiscalYearViaApi,
	createJournalEntryViaApi,
	todayInZurich,
} from './helpers/api-fixtures';

async function login(page: Page, username = 'admin', password = 'admin123') {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

/** Tabule jusqu'à `target`, avec une borne : un élément hors de l'ordre échoue au lieu de boucler. */
async function tabTo(page: Page, target: Locator) {
	const handle = await target.elementHandle();
	for (let i = 0; i < 200; i++) {
		await page.keyboard.press('Tab');
		const reached = await page.evaluate((el) => document.activeElement === el, handle);
		if (reached) return;
	}
	throw new Error('élément jamais atteint au clavier');
}

const currentMonth = () => todayInZurich().slice(0, 7);

test.describe.serial('Graphiques de l’accueil, exercice ouvert (33-3b)', () => {
	const expenseLabel = uniq('Charge graphiques');
	let expenseAccountId = 0;
	let expenseAccountNumber = '';
	let fiscalYear = '';

	test.beforeAll(async () => {
		await seedTestState('with-company-no-fy');
	});

	test.afterEach(async ({ page }) => {
		await clearAuthStorage(page);
	});

	test('1. les valeurs traversent : tableau au bon mois, au bon poste, sur douze mois', async ({ page }) => {
		await login(page);
		fiscalYear = await createFiscalYearViaApi(page);
		const posted = await createExpenseEntryViaApi(page, expenseLabel, '123.45');
		expenseAccountId = posted.accountId;
		expenseAccountNumber = posted.accountNumber;
		await createJournalEntryViaApi(page, uniq('Produit graphiques'), '321.40');
		await page.goto('/');

		await expect(page.getByTestId('homepage-chart-month-mark')).toHaveCount(12);
		await page.getByTestId('homepage-chart-months-details').locator('summary').click();
		const row = page.locator(`tr[data-month="${currentMonth()}"]`);
		expect(Number(await row.getAttribute('data-revenues'))).toBe(321.4);
		expect(Number(await row.getAttribute('data-expenses'))).toBe(123.45);
		const post = page.locator(`tr[data-account-id="${expenseAccountId}"]`);
		expect(Number(await post.getAttribute('data-amount'))).toBe(123.45);
		expect(await post.getAttribute('data-share')).toBe('100.0');
	});

	test('2. clic de souris sur la marque d’un mois : ses écritures, et elles seules', async ({ page }) => {
		await login(page);
		// Une écriture d'un autre mois du même exercice : janvier, ou décembre en janvier.
		const month = currentMonth();
		const other = month.endsWith('-01') ? `${fiscalYear}-12-15` : `${fiscalYear}-01-15`;
		const otherLabel = uniq('Charge autre mois');
		await createExpenseEntryViaApi(page, otherLabel, '1.00', { number: '6990', date: other });
		await page.goto('/');

		const mark = page.locator(`[data-testid="homepage-chart-month-mark"][data-month="${month}"]`);
		await mark.click();
		const [y, m] = month.split('-').map(Number);
		const lastDay = String(new Date(Date.UTC(y, m, 0)).getUTCDate()).padStart(2, '0');
		await expect(page).toHaveURL(new RegExp(`/journal-entries\\?dateFrom=${month}-01&dateTo=${month}-${lastDay}$`));
		await expect(page.locator('tr').filter({ hasText: expenseLabel })).toHaveCount(1);
		await expect(page.locator('tr').filter({ hasText: otherLabel })).toHaveCount(0);
	});

	test('3. clic de souris sur une part dessinée en arc : le grand livre du compte, généré', async ({ page }) => {
		await login(page);
		const mark = page.getByTestId('homepage-chart-post-mark').first();
		await expect(mark.locator('path'), 'le navigateur clique un arc, non un disque').toHaveCount(1);
		// La part fait presque le tour depuis midi : le centre de sa boîte est sa pointe.
		// Le point à neuf heures, à mi-rayon, est dans la part.
		const box = (await mark.boundingBox())!;
		await mark.click({ position: { x: box.width * 0.25, y: box.height * 0.5 } });
		await expect(page).toHaveURL(
			new RegExp(`/reports\\?tab=general-ledger&accountId=${expenseAccountId}&from=${fiscalYear}-01-01&to=${fiscalYear}-12-31`),
		);
		await expect(page.locator('tr').filter({ hasText: expenseLabel })).toHaveCount(1);
	});

	test('4a. au clavier : le lien du poste, même destination', async ({ page }) => {
		await login(page);
		const link = page.getByTestId('homepage-chart-post-link').first();
		await expect(link).toBeVisible();
		await expect(link).toContainText(expenseAccountNumber);
		await tabTo(page, link);
		await page.keyboard.press('Enter');
		await expect(page).toHaveURL(
			new RegExp(`/reports\\?tab=general-ledger&accountId=${expenseAccountId}&from=${fiscalYear}-01-01&to=${fiscalYear}-12-31`),
		);
	});

	test('4b. au clavier : le résumé s’ouvre, puis le lien du mois', async ({ page }) => {
		await login(page);
		const summary = page.getByTestId('homepage-chart-months-details').locator('summary');
		await expect(summary).toBeVisible();
		await tabTo(page, summary);
		await page.keyboard.press('Enter');
		const link = page.locator(`tr[data-month="${currentMonth()}"]`).getByTestId('homepage-chart-month-link');
		await expect(link).toBeVisible();
		await tabTo(page, link);
		await page.keyboard.press('Enter');
		await expect(page).toHaveURL(new RegExp(`/journal-entries\\?dateFrom=${currentMonth()}-01&dateTo=`));
	});

	test('5. une contre-passation dans l’exercice : le poste et le mois reviennent', async ({ page }) => {
		await login(page);
		const extra = await createExpenseEntryViaApi(page, uniq('Charge corrigée'), '77.77');
		await page.goto('/');
		const post = page.locator(`tr[data-account-id="${expenseAccountId}"]`);
		await expect(post).toBeVisible();
		expect(Number(await post.getAttribute('data-amount'))).toBe(201.22);

		const ctx = await authedApiContext(page);
		try {
			const res = await ctx.post(`/api/v1/journal-entries/${extra.id}/reverse`);
			expect(res.ok(), `reverse: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await page.goto('/');
		await expect(post).toBeVisible();
		expect(Number(await post.getAttribute('data-amount'))).toBe(123.45);
		const row = page.locator(`tr[data-month="${currentMonth()}"]`);
		expect(Number(await row.getAttribute('data-expenses'))).toBe(123.45);
	});

	test('6. un rôle Consultation voit les mêmes graphiques et les mêmes liens', async ({ page }) => {
		await login(page);
		const username = `consult-charts-${Date.now()}`;
		const ctx = await authedApiContext(page);
		try {
			const res = await ctx.post('/api/v1/users', {
				data: { username, password: 'MotDePasse12345', role: 'Consultation' },
			});
			expect(res.ok(), `create user: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}
		await clearAuthStorage(page);
		await login(page, username, 'MotDePasse12345');
		await expect(page.getByTestId('homepage-chart-month-mark')).toHaveCount(12);
		await expect(page.getByTestId('homepage-chart-post-link')).toHaveCount(2);
		await expect(page.getByTestId('homepage-chart-month-link')).toHaveCount(12);
	});

	test('6b. six parts depuis l’API : cinq comptes et « Autres », et le clic sur un arc', async ({ page }) => {
		await login(page);
		let largest = 0;
		for (const [number, amount] of [
			['6100', '900.00'],
			['6200', '700.00'],
			['6300', '500.00'],
			['6400', '300.00'],
			['6500', '200.00'],
			['6600', '100.00'],
		]) {
			const posted = await createExpenseEntryViaApi(page, uniq(`Charge ${number}`), amount, { number });
			if (number === '6100') largest = posted.accountId;
		}
		await page.goto('/');

		// Base : 2'700, plus 123.45 (le compte du scénario 1) et 1.00 (celui du scénario 2).
		const marks = page.getByTestId('homepage-chart-post-mark');
		await expect(marks).toHaveCount(5);
		await expect(page.getByTestId('homepage-chart-others-mark')).toHaveCount(1);
		await expect(
			page.getByTestId('homepage-card-expense-pie').locator('svg[role="img"] path'),
			'six arcs réellement rendus, aucun disque',
		).toHaveCount(6);
		const first = page.locator(`tr[data-account-id="${largest}"]`);
		expect(await first.getAttribute('data-share')).toBe('31.9');
		const others = page.getByTestId('homepage-chart-others-row');
		expect(Number(await others.getAttribute('data-amount'))).toBe(224.45);
		await expect(others).toContainText('3');

		await marks.first().click();
		await expect(page).toHaveURL(new RegExp(`/reports\\?tab=general-ledger&accountId=${largest}&`));
	});

	test('8a. axe-core sans violation, tableau des mois fermé puis ouvert', async ({ page }) => {
		await login(page);
		await expect(page.getByTestId('homepage-chart-post-mark').first()).toBeVisible();
		await expect(page.getByTestId('homepage-card-recent-entries').locator('ul, p').first()).toBeVisible();
		expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
		await page.getByTestId('homepage-chart-months-details').locator('summary').click();
		await expect(page.getByTestId('homepage-chart-month-link').first()).toBeVisible();
		expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
	});
});

test.describe('Graphiques de l’accueil, sans exercice (33-3b)', () => {
	test.beforeAll(async () => {
		await seedTestState('with-company-no-fy');
	});

	test.afterEach(async ({ page }) => {
		await clearAuthStorage(page);
	});

	test('7. aucun exercice ouvert : la phrase, aucun graphique', async ({ page }) => {
		await login(page);
		await expect(page.getByTestId('homepage-charts-no-open-year')).toBeVisible();
		await expect(page.getByTestId('homepage-card-monthly-bars')).toHaveCount(0);
		await expect(page.getByTestId('homepage-card-expense-pie')).toHaveCount(0);
	});

	test('8b. axe-core sans violation', async ({ page }) => {
		await login(page);
		await expect(page.getByTestId('homepage-charts-no-open-year')).toBeVisible();
		expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([]);
	});
});
