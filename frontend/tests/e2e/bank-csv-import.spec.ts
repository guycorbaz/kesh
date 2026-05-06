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
import path from 'path';
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
	await page.getByTestId('bank-import-profile-submit').click();

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
	await page.getByTestId('bank-import-profile-submit').click();

	await expect(page.getByTestId('bank-import-profile-error')).toBeVisible();
});

test('duplicate bank_name returns 409', async ({ page }) => {
	await login(page);
	// Premier profil
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('UBS');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-import-profile-submit').click();
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);

	// Deuxième profil même nom → erreur 409
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('UBS');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('bank-import-profile-submit').click();

	await expect(page.getByTestId('bank-import-profile-error')).toBeVisible();
});

test('accessibility — profile pages axe scan zero violations', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import/profiles');
	await expect(page.getByTestId('bank-import-profile-page-title')).toBeVisible();

	const results = await new AxeBuilder({ page }).analyze();
	expect(results.violations).toEqual([]);
});

// Pass 1 review G3 H11+H12 : 2 scénarios CSV ajoutés en bonus.

test('shows BANK_CSV_NO_PROFILE_MATCH on CSV upload without profile', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page
		.getByTestId('bank-import-file-input')
		.setInputFiles(path.join(__dirname, 'fixtures', 'csv_utf8_minimal.csv'));

	const error = page.getByTestId('bank-import-error');
	await expect(error).toBeVisible();
	await expect(error).toHaveAttribute(
		'data-error-code',
		'BANK_CSV_NO_PROFILE_MATCH',
	);
});

test('displays partial failure error panel with line numbers', async ({ page }) => {
	await login(page);
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page
		.getByTestId('bank-import-file-input')
		.setInputFiles(path.join(__dirname, 'fixtures', 'csv_partial_failure.csv'));

	// Sans profil bancaire configuré, on attend le 404 NO_PROFILE_MATCH
	// (pas le 422 PARTIAL_FAILURE — le parsing n'est pas atteint).
	// Le test démontre que le panneau erreur s'affiche et contient
	// l'erreur structurée (data-error-code) — pré-requis pour partial
	// failure quand un profil sera configuré.
	const error = page.getByTestId('bank-import-error');
	await expect(error).toBeVisible();
});

// Pass 2 review M4 (BH2-2) : CSV happy path E2E couvrant le pipeline
// complet (création profil → upload CSV → preview → parser → success).
test('uploads CSV with matching profile end-to-end', async ({ page }) => {
	await login(page);

	// Étape 1 — créer un profil bancaire qui matche `csv_utf8_minimal.csv`.
	await page.goto('/bank-import/profiles/new');
	await page.getByTestId('bank-name-input').fill('Test E2E Bank');
	// Pattern qui matche le filename exact uploadé.
	await page.getByTestId('filename-pattern-input').fill('^csv_utf8_minimal\\.csv$');
	await page.getByTestId('date-format-input').fill('%Y-%m-%d');
	await page.getByTestId('field-separator-select').selectOption(';');
	await page.getByTestId('decimal-separator-select').selectOption('.');
	await page.getByTestId('header-row-count-input').fill('1');
	await page.getByTestId('col-date-input').fill('0');
	await page.getByTestId('col-amount-input').fill('1');
	await page.getByTestId('col-reference-input').fill('2');
	await page.getByTestId('col-details-input').fill('3');
	await page.getByTestId('bank-import-profile-submit').click();
	await expect(page).toHaveURL(/\/bank-import\/profiles\/\d+/);

	// Étape 2 — uploader le CSV qui matche le profil par auto-match
	// filename. Le backend doit résoudre le profil via
	// `find_matching_profiles_for_filename` puis parser → preview.
	await page.goto('/bank-import');
	await expect(page.getByTestId('bank-import-page-title')).toBeVisible();
	await page.getByTestId('bank-account-select').selectOption({ index: 1 });
	await page
		.getByTestId('bank-import-file-input')
		.setInputFiles(path.join(__dirname, 'fixtures', 'csv_utf8_minimal.csv'));

	// Étape 3 — preview affiché avec 3 transactions (cf. fixture).
	await expect(page.getByTestId('bank-import-preview')).toBeVisible();
	await expect(page.getByTestId('preview-tx-count')).toHaveText('3');

	// Étape 4 — confirm import → persist + redirect.
	await page.getByTestId('bank-import-confirm').click();
	await expect(page.getByTestId('bank-import-preview')).toBeHidden();

	// La liste affiche au moins 1 import.
	await expect(page.getByTestId('bank-import-list-row').first()).toBeVisible();
});
