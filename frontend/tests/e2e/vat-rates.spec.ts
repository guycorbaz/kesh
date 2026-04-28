import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Story 7.2 (KF-003 closure) : taux TVA configurés en DB et
 * lus dynamiquement par le frontend (store de session).
 *
 * Vérifie que :
 * - Le `<select>` du formulaire produit est peuplé depuis le store (≥4 options).
 * - Le `<select>` ligne du formulaire facture est peuplé depuis le store (≥4).
 * - Le store n'est pas re-fetched à chaque navigation (cache de session).
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

test.describe('Taux TVA — chargement dynamique depuis le backend', () => {
	test('formulaire produit : le <select> contient les 4 taux suisses 2024+', async ({ page }) => {
		await login(page);
		await page.goto('/products');
		await page.getByRole('button', { name: /Nouveau produit/ }).click();

		// Attendre que le store ait peuplé le <select> (au moins 1 option).
		const select = page.locator('#form-vat-rate');
		await expect(select.locator('option')).toHaveCount(4, { timeout: 5000 });

		// Les 4 valeurs (taux suisses 2024+) doivent toutes être présentes.
		const values = await select.locator('option').evaluateAll((opts) =>
			(opts as HTMLOptionElement[]).map((o) => o.value),
		);
		expect(values).toEqual(expect.arrayContaining(['8.10', '3.80', '2.60', '0.00']));
	});

	test('formulaire facture : le <select> ligne contient les 4 taux', async ({ page }) => {
		await login(page);
		await page.goto('/invoices/new');

		// Attendre la 1re ligne par défaut (créée à l'ouverture du formulaire).
		const lineSelect = page.locator('select').filter({ has: page.locator('option') }).first();
		await expect(lineSelect.locator('option')).toHaveCount(4, { timeout: 5000 });
	});

	test('le store ne re-fetch pas entre /products et /invoices/new (cache de session)', async ({
		page,
	}) => {
		await login(page);

		// Capturer les requêtes /api/v1/vat-rates pendant la navigation croisée.
		const requests: string[] = [];
		page.on('request', (req) => {
			if (req.url().includes('/api/v1/vat-rates')) {
				requests.push(req.url());
			}
		});

		await page.goto('/products');
		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await expect(page.locator('#form-vat-rate option')).toHaveCount(4, { timeout: 5000 });
		await page.keyboard.press('Escape');

		await page.goto('/invoices/new');
		await expect(
			page.locator('select').filter({ has: page.locator('option') }).first().locator('option'),
		).toHaveCount(4, { timeout: 5000 });

		await page.goto('/products');
		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await expect(page.locator('#form-vat-rate option')).toHaveCount(4, { timeout: 5000 });

		// Le store doit avoir fait au plus 1 fetch sur tout le parcours (inflight-promise + cache).
		expect(requests.length).toBeLessThanOrEqual(1);
	});
});
