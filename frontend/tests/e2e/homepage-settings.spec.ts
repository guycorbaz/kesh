import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Page d'accueil & Paramètres (Story 2.4)
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	// Clear localStorage after each test to prevent token bleed to next test
	await clearAuthStorage(page);
});

/** Helper : login as admin (admin/admin123 — bootstrap test). */
async function loginAsAdmin(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test.describe('Homepage', () => {
	test.beforeEach(async ({ page }) => {
		await loginAsAdmin(page);
	});

	test('affiche les widgets inconditionnels sur la page d\'accueil', async ({ page }) => {
		// Story v014-1 AC#29 — widget bank-accounts est conditionnel
		// (`{#if bankAccounts.length > 0}`) ; le seed `with-company` ne crée
		// pas de bank_account donc widget absent. Option (b) AC#31 adopté :
		// asserter explicitement l'absence.
		await expect(page).toHaveURL('/');
		await expect(page.locator('[data-testid="homepage-card-recent-entries"]')).toBeVisible();
		await expect(page.locator('[data-testid="homepage-card-open-invoices"]')).toBeVisible();
		await expect(page.locator('[data-testid="homepage-card-bank-accounts"]')).toHaveCount(0);
	});
});

test.describe('Settings', () => {
	test.beforeEach(async ({ page }) => {
		await loginAsAdmin(page);
	});

	test('page Paramètres affiche 4 sections', async ({ page }) => {
		await page.goto('/settings');
		// Sections rendues comme <h2> sur la page /settings (heading scope par level + nom unique)
		await expect(page.getByRole('heading', { level: 2, name: 'Organisation' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 2, name: 'Comptabilité' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 2, name: 'Comptes bancaires' })).toBeVisible();
		await expect(page.getByRole('heading', { level: 2, name: 'Utilisateurs' })).toBeVisible();
	});
});

test.describe('Homepage — accessibilité', () => {
	test.beforeEach(async ({ page }) => {
		await loginAsAdmin(page);
	});

	test('axe-core sans violations sur la page d\'accueil', async ({ page }) => {
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});
});
