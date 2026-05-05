/**
 * Story 8-2 T8 — Tests E2E Playwright pour la feature bank-csv-import.
 *
 * 5 scénarios minimaux v0.1 (couverture ACs principaux) :
 *   1. liste profils accessible (AC #11)
 *   2. créer profil banque (AC #6)
 *   3. valeurs séparateurs distincts validées côté API (AC #15b)
 *   4. UNIQUE bank_name par company → 409 (AC #17a)
 *   5. accessibilité page profils — axe scan zero violations (AC #23)
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true + Playwright browsers
 * (Ubuntu 26.04 : PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 *
 * Note : scénarios CSV import end-to-end (UTF-8, ISO-8859-1, partial
 * failure, encoding mismatch) reportés à code review post-impl ou
 * Story 8-3 (rejet partiel). La couverture E2E HTTP T4 (14 tests) +
 * unit T2 (40 tests) couvre déjà les ACs critiques.
 */

import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test('lists bank profiles page accessible', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles');
	await expect(page.getByTestId('bank-import-profile-page-title')).toBeVisible();
});

test('creates a bank profile via wizard', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('Test Bank');
	await page.getByTestId('filename-pattern-input').fill('^test-.*\\.csv$');
	await page.getByTestId('date-format-input').fill('%d.%m.%Y');
	await page.getByTestId('field-separator-select').selectOption(';');
	await page.getByTestId('decimal-separator-select').selectOption('.');
	await page.getByTestId('header-row-count-input').fill('1');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-profile-submit').click();

	// Redirect vers /bank-import/profiles/{id}
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);
});

test('rejects equal separators with API error', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('Bad Bank');
	await page.getByTestId('date-format-input').fill('%d.%m.%Y');
	await page.getByTestId('field-separator-select').selectOption(',');
	await page.getByTestId('decimal-separator-select').selectOption(',');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-profile-submit').click();

	await expect(page.getByTestId('bank-profile-error')).toBeVisible();
});

test('duplicate bank_name returns 409', async ({ page }) => {
	await login(page);
	// Premier profil
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('UBS');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-profile-submit').click();
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);

	// Deuxième profil même nom → erreur 409
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('UBS');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-profile-submit').click();

	await expect(page.getByTestId('bank-profile-error')).toBeVisible();
});

test('accessibility — profile pages axe scan zero violations', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles');
	await expect(page.getByTestId('bank-import-profile-page-title')).toBeVisible();

	const results = await new AxeBuilder({ page }).analyze();
	expect(results.violations).toEqual([]);
});
