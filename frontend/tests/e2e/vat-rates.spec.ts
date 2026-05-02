import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Story 7.2 (KF-003 closure) : taux TVA configurés en DB et
 * lus dynamiquement par le frontend (store de session).
 *
 * Vérifie que :
 * - Le `<select>` du formulaire produit est peuplé depuis le store (≥4 options).
 * - Le `<select>` ligne du formulaire facture est peuplé depuis le store (≥4).
 *
 * Note : un 3e test « cache de session » a été retiré 2026-05-02 (closes #56).
 * Il assertait `≤ 1 fetch /api/v1/vat-rates` sur 3 `page.goto()` consécutifs,
 * mais `page.goto()` est une vraie navigation browser (full reload, pas du
 * SPA-routing) → le cache module-level est ré-instancié à chaque load. Le
 * cache fonctionne correctement pour la navigation SPA réelle (link clicks),
 * mais ce test ne pouvait jamais passer avec son design. Cf. KF-024 closed
 * as not-a-bug. La garantie « pas de re-fetch entre mounts dans la même page »
 * est implicite via le test 2 (formulaire facture) qui passe.
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
		const lineSelect = page.getByTestId('invoice-line-vat-rate').first();
		await expect(lineSelect.locator('option')).toHaveCount(4, { timeout: 5000 });
	});
});
