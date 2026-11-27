import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Story 64-1 (#1178) : *Paramètres → TVA* par le formulaire réel.
 *
 * Svelte 5 lie un `<input type="number">` à un nombre : le taux saisi ne partait pas
 * (`TypeError` sur `.trim()`, « La création a échoué. »). La création contre le vrai serveur
 * prouve que la chaîne décimale est acceptée.
 *
 * Spec à part, avec son propre `seedTestState` : `vat-rates.spec.ts` compte exactement
 * quatre options dans les listes de taux, et ces listes prennent tout taux actif — un taux
 * créé ici les rougirait s'il était dans le même fichier (le reseed isole les fichiers).
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

test.describe('Taux TVA — formulaire de création (64-1)', () => {
	test('un taux saisi à 8.1 est créé et apparaît dans la liste', async ({ page }) => {
		await login(page);
		await page.goto('/settings/vat-rates');
		await page.getByTestId('vat-rates-new').click();
		await page.getByTestId('vat-form-category').fill('e2e-64-1');
		await page.getByTestId('vat-form-rate').fill('8.1');
		await page.getByTestId('vat-form-from').fill('2031-01-01');
		await page.getByTestId('vat-form-submit').click();
		// Sonde positive : le formulaire se referme, jamais « La création a échoué. ».
		await expect(page.getByTestId('vat-form-submit')).toHaveCount(0);
		await expect(page.getByTestId('vat-form-error')).toHaveCount(0);
		const row = page.getByTestId('vat-rate-row').filter({ hasText: '8.1' });
		await expect(row.first()).toBeVisible();
	});

	test('un taux vide est refusé par le formulaire, sans envoi', async ({ page }) => {
		await login(page);
		await page.goto('/settings/vat-rates');
		await page.getByTestId('vat-rates-new').click();
		await page.getByTestId('vat-form-from').fill('2031-01-01');
		await page.getByTestId('vat-form-submit').click();
		await expect(page.getByTestId('vat-form-error')).toHaveText('Saisissez un taux.');
	});
});
