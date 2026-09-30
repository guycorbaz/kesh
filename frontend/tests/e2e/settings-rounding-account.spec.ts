/**
 * Story 25-4-c3-a1 (#476) — désigner le compte de différences d'arrondi dans
 * Paramètres → Facturation, l'enregistrer, et le retrouver après rechargement.
 *
 * Le compte choisi est la charge imputable du jeu `with-company` (4000). Le test remet
 * le réglage à vide à la fin : la base est partagée entre specs.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page } from '@playwright/test';
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

async function saveSettings(page: Page): Promise<void> {
	const saved = page.waitForResponse(
		(r) => r.url().includes('/api/v1/company/invoice-settings') && r.request().method() === 'PUT',
	);
	await page.getByTestId('settings-invoicing-save').click();
	expect((await saved).status()).toBe(200);
}

test('désigner le compte de différences d’arrondi et le retrouver', async ({ page }) => {
	await login(page);
	await page.goto('/settings/invoicing');

	const select = page.getByTestId('settings-rounding-account');
	await expect(select).toBeVisible();
	await expect(page.getByTestId('settings-rounding-hint')).toBeVisible();

	// La charge imputable du jeu `with-company` (qui ne porte que 3000 et 4000).
	const charge = select.locator('option', { hasText: '4000' });
	const chargeId = await charge.getAttribute('value');
	expect(chargeId, 'le compte 4000 est proposé').toBeTruthy();
	await select.selectOption(chargeId!);
	await saveSettings(page);

	await page.reload();
	await expect(page.getByTestId('settings-rounding-account')).toHaveValue(chargeId!);

	// Remise à vide : le réglage ne doit pas fuir vers les autres specs.
	await page.getByTestId('settings-rounding-account').selectOption({ index: 0 });
	await saveSettings(page);
});
