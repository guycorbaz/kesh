import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState } from './helpers/test-state';

/**
 * Tests E2E — Authentification & Accessibilité (Story 1.11)
 *
 * Ces tests nécessitent un backend Kesh fonctionnel sur localhost:3000
 * avec `KESH_TEST_MODE=true` (cf. Story 6.4).
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.describe('Login', () => {
	test('login réussi → redirection accueil, affichage header/sidebar', async ({ page }) => {
		await page.goto('/login');

		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');

		// Redirection vers l'accueil
		await expect(page).toHaveURL('/');

		// Header visible avec logo Kesh
		await expect(page.locator('header')).toBeVisible();
		await expect(page.locator('header').getByText('Kesh')).toBeVisible();

		// Sidebar navigation visible
		await expect(page.locator('nav[aria-label="Navigation principale"]')).toBeVisible();
	});

	test('login échoué → message d\'erreur affiché', async ({ page }) => {
		await page.goto('/login');

		await page.fill('#username', 'wrong');
		await page.fill('#password', 'wrong');
		await page.click('button[type="submit"]');

		// Rester sur la page login
		await expect(page).toHaveURL(/\/login/);

		// Message d'erreur visible
		await expect(page.locator('#login-error')).toContainText('Identifiant ou mot de passe incorrect');
	});

	test('accès page protégée sans auth → redirect login', async ({ page }) => {
		// Tenter d'accéder à l'accueil sans être connecté
		await page.goto('/');

		// Doit être redirigé vers /login
		await expect(page).toHaveURL(/\/login/);
	});

	test('raccourci Ctrl+S déclenche l\'événement kesh:save', async ({ page }) => {
		// Se connecter d'abord
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');

		// Attacher le listener AVANT de presser la touche (évite la race condition)
		await page.evaluate(() => {
			(window as unknown as Record<string, boolean>).__keshSaveFired = false;
			window.addEventListener('kesh:save', () => {
				(window as unknown as Record<string, boolean>).__keshSaveFired = true;
			}, { once: true });
		});

		// Presser Ctrl+S
		await page.keyboard.press('Control+s');

		// Vérifier que l'événement a été déclenché
		const eventFired = await page.evaluate(
			() => (window as unknown as Record<string, boolean>).__keshSaveFired,
		);
		expect(eventFired).toBe(true);
	});
});

test.describe('Accessibilité', () => {
	test('page login — axe-core sans violations', async ({ page }) => {
		await page.goto('/login');

		const results = await new AxeBuilder({ page }).analyze();

		expect(results.violations).toEqual([]);
	});

	test('layout principal — axe-core sans violations', async ({ page }) => {
		// Se connecter d'abord
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');

		const results = await new AxeBuilder({ page }).analyze();

		expect(results.violations).toEqual([]);
	});
});
