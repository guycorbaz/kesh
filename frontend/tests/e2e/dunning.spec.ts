import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Story 21-4 : page Réglages > Rappels débiteurs (settings/dunning).
 * Le 1er GET des réglages déclenche le seed lazy des 3 niveaux par défaut (backend 21-3).
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function loginAsAdmin(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test.describe('Réglages — Rappels débiteurs', () => {
	test('affiche les 3 niveaux seedés, la grâce et l’exemple calculé', async ({ page }) => {
		await loginAsAdmin(page);
		await page.goto('/settings/dunning');

		// Le seed lazy pose 3 niveaux au 1er GET.
		await expect(page.getByTestId('dunning-level-row')).toHaveCount(3, { timeout: 5000 });
		// Grâce = 5 par défaut.
		await expect(page.getByTestId('dunning-grace-input')).toHaveValue('5');
		// Exemple des délais cumulés visible (D7).
		await expect(page.getByTestId('dunning-example')).toBeVisible();
		// Hint CGV visible (D13).
		await expect(page.getByTestId('dunning-cgv-hint')).toBeVisible();
	});

	test('CRUD : ajouter, modifier, supprimer un niveau', async ({ page }) => {
		await loginAsAdmin(page);
		await page.goto('/settings/dunning');
		await expect(page.getByTestId('dunning-level-row')).toHaveCount(3, { timeout: 5000 });

		// Ajouter un 4e niveau.
		await page.getByTestId('dunning-level-new').click();
		await page.getByTestId('dunning-form-delay').fill('7');
		await page.getByTestId('dunning-form-fee').fill('60.00');
		await page.getByTestId('dunning-form-submit').click();
		await expect(page.getByTestId('dunning-level-row')).toHaveCount(4, { timeout: 5000 });

		// Supprimer le dernier niveau (revient à 3, renumérotation backend).
		await page.getByTestId('dunning-level-row').last().getByRole('button', { name: /Supprimer/ }).click();
		await page.getByTestId('dunning-level-delete-confirm').getByRole('button', { name: /Supprimer/ }).click();
		await expect(page.getByTestId('dunning-level-row')).toHaveCount(3, { timeout: 5000 });
	});

	test('éditer la période de grâce', async ({ page }) => {
		await loginAsAdmin(page);
		await page.goto('/settings/dunning');
		await expect(page.getByTestId('dunning-grace-input')).toHaveValue('5', { timeout: 5000 });

		await page.getByTestId('dunning-grace-input').fill('10');
		await page.getByTestId('dunning-grace-save').click();
		await page.reload();
		await expect(page.getByTestId('dunning-grace-input')).toHaveValue('10', { timeout: 5000 });
	});

	test('a11y : page réglages rappels sans violation axe', async ({ page }) => {
		await loginAsAdmin(page);
		await page.goto('/settings/dunning');
		await expect(page.getByTestId('dunning-example')).toBeVisible({ timeout: 5000 });
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});
});
