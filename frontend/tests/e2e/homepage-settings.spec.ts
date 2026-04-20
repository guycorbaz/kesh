import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState } from './helpers/test-state';

/**
 * Tests E2E — Page d'accueil & Paramètres (Story 2.4)
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.beforeEach(async ({ page }) => {
	// Clear localStorage to isolate each test and prevent token bleed from previous tests
	await page.context().clearCookies();
});

test.describe('Homepage', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
	});

	test('affiche 3 widgets sur la page d\'accueil', async ({ page }) => {
		await expect(page).toHaveURL('/');
		await expect(page.getByText('Dernières écritures')).toBeVisible();
		await expect(page.getByText('Factures ouvertes')).toBeVisible();
		await expect(page.getByText('Comptes bancaires')).toBeVisible();
	});
});

test.describe('Settings', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
	});

	test('page Paramètres affiche 4 sections', async ({ page }) => {
		await page.goto('/settings');
		await expect(page.getByText('Organisation')).toBeVisible();
		await expect(page.getByText('Comptabilité')).toBeVisible();
		await expect(page.getByText('Comptes bancaires')).toBeVisible();
		await expect(page.getByText('Utilisateurs')).toBeVisible();
	});
});

test.describe('Homepage — accessibilité', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
	});

	test('axe-core sans violations sur la page d\'accueil', async ({ page }) => {
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});
});
