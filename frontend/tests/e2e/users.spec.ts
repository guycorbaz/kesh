import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	// Clear localStorage after each test to prevent token bleed to next test
	await clearAuthStorage(page);
});

/**
 * Tests E2E — Gestion des utilisateurs (Story 1.12)
 *
 * Ces tests nécessitent un backend Kesh fonctionnel sur localhost:3000
 * avec un admin bootstrap (admin / admin123).
 */

/** Helper : login as admin and navigate to /users. */
async function loginAndGoToUsers(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	await page.goto('/users');
	await expect(page).toHaveURL('/users');
}

test.describe('Page utilisateurs — CRUD', () => {
	test('admin voit le lien Utilisateurs dans le sidebar', async ({ page }) => {
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');

		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		await expect(sidebar.locator('[data-testid="nav-link-users"]')).toBeVisible();
	});

	test('liste des utilisateurs affichée avec tableau', async ({ page }) => {
		await loginAndGoToUsers(page);

		// Le tableau doit être visible
		await expect(page.locator('[data-testid="user-table"]')).toBeVisible();

		// Au moins l'admin bootstrap doit apparaître
		await expect(page.locator('[data-testid="user-row-admin"]')).toBeVisible();

		// Badge "Vous" sur l'admin connecté
		await expect(page.locator('[data-testid="current-user-badge"]')).toBeVisible();
	});

	test('création d\'un utilisateur', async ({ page }) => {
		await loginAndGoToUsers(page);

		// Ouvrir le dialog de création
		await page.locator('[data-testid="user-create-button"]').click();
		await expect(page.getByRole('dialog')).toContainText('Créez un nouveau compte');

		// Remplir le formulaire
		const testUser = `test-${Date.now()}`;
		await page.fill('#create-username', testUser);
		await page.fill('#create-password', 'MotDePasse12345');
		await page.fill('#create-confirm', 'MotDePasse12345');

		// Soumettre — bouton scoped au dialog (Dialog.Footer parent + texte 'Créer')
		await page.getByRole('dialog').getByRole('button', { name: 'Créer' }).click();

		// L'utilisateur doit apparaître dans le tableau (testid scope ligne, pas de collision avec le dialog)
		await expect(page.locator(`[data-testid="user-row-${testUser}-username-cell"]`)).toBeVisible({ timeout: 5000 });
	});

	test('validation mot de passe trop court', async ({ page }) => {
		await loginAndGoToUsers(page);

		await page.locator('[data-testid="user-create-button"]').click();
		await page.fill('#create-username', 'test-short-pw');
		await page.fill('#create-password', 'short');
		await page.fill('#create-confirm', 'short');

		await page.getByRole('dialog').getByRole('button', { name: 'Créer' }).click();

		// Message d'erreur de validation (scope au dialog)
		await expect(page.getByRole('dialog')).toContainText('au moins 12 caractères');
	});

	test('validation mots de passe non identiques', async ({ page }) => {
		await loginAndGoToUsers(page);

		await page.locator('[data-testid="user-create-button"]').click();
		await page.fill('#create-username', 'test-mismatch');
		await page.fill('#create-password', 'MotDePasse12345');
		await page.fill('#create-confirm', 'Différent12345!');

		await page.getByRole('dialog').getByRole('button', { name: 'Créer' }).click();

		await expect(page.getByRole('dialog')).toContainText('ne correspondent pas');
	});
});

test.describe('Page utilisateurs — Erreurs', () => {
	test('le bouton désactiver est absent pour soi-même', async ({ page }) => {
		await loginAndGoToUsers(page);

		// La ligne de l'admin connecté (testid stable) ne doit pas avoir de bouton désactiver
		const adminRow = page.locator('[data-testid="user-row-admin"]');
		await expect(adminRow.getByLabel(/Désactiver/)).not.toBeVisible();
	});
});

test.describe('Page utilisateurs — Accessibilité', () => {
	test('axe-core : pas de violations critiques', async ({ page }) => {
		await loginAndGoToUsers(page);

		const results = await new AxeBuilder({ page })
			.disableRules(['color-contrast']) // tokens custom, vérifiés manuellement
			.analyze();

		expect(results.violations.filter((v) => v.impact === 'critical')).toHaveLength(0);
	});
});
