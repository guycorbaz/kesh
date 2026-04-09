import { test, expect } from '@playwright/test';

/**
 * Tests E2E — Page d'accueil & Paramètres (Story 2.4)
 * Requiert backend + frontend running avec onboarding complété.
 */

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
