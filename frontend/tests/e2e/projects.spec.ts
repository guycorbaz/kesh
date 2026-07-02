import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Gestion des projets analytiques (Epic 19, Story 19-1).
 * Valide le câblage réel formulaire → API → arbre (create racine + sous-projet).
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test.describe('Projets analytiques', () => {
	test('créer un projet racine puis un sous-projet → arbre 2 niveaux', async ({ page }) => {
		const uniq = `${Date.now()}`;
		await login(page);
		await page.goto('/settings/projects');
		await expect(page.getByRole('heading', { level: 1 })).toContainText('Projets');

		// Projet racine.
		await page.getByTestId('project-new').click();
		await page.getByTestId('project-code').fill(`RENOV-${uniq}`);
		await page.getByTestId('project-name').fill('Rénovation chalet');
		await page.getByTestId('project-submit').click();

		const root = page
			.getByTestId('project-row')
			.filter({ hasText: `RENOV-${uniq}` });
		await expect(root).toBeVisible();

		// Sous-projet rattaché à la racine.
		await page.getByTestId('project-new').click();
		await page.getByTestId('project-code').fill(`TOIT-${uniq}`);
		await page.getByTestId('project-name').fill('Toiture');
		await page.locator('select').selectOption({ label: `RENOV-${uniq} — Rénovation chalet` });
		await page.getByTestId('project-submit').click();

		// Le sous-projet apparaît indenté sous sa racine.
		const child = page.getByTestId('project-child').filter({ hasText: `TOIT-${uniq}` });
		await expect(child).toBeVisible();
	});
});
