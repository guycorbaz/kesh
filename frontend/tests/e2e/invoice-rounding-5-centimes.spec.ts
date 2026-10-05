/**
 * Story 25-4-c4-b (#494) — l'arrondi à 5 centimes, réglable et visible.
 *
 * Désactiver puis réactiver la case de Paramètres → Facturation, puis valider une
 * facture au TTC de 123.44 (114.19 HT à 8.1 %) : sa fiche porte la ligne
 * « Arrondi » (+0.01) et un total de 123.45.
 *
 * Le compte d'arrondi est la charge imputable du jeu `with-company` (4000). Le test
 * remet le compte à vide à la fin : la base est partagée entre specs.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import { createContactWithAddressViaApi } from './helpers/api-fixtures';

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

test('la case d’arrondi se règle, et la facture validée porte la ligne « Arrondi »', async ({ page }) => {
	await login(page);
	await page.goto('/settings/invoicing');
	const select = page.getByTestId('settings-rounding-account');
	const box = page.getByTestId('settings-round-to-5-centimes');
	const chargeId = await select.locator('option', { hasText: '4000' }).getAttribute('value');
	expect(chargeId, 'le compte 4000 est proposé').toBeTruthy();
	try {
		// Désactiver, enregistrer, retrouver désactivé.
		await select.selectOption(chargeId!);
		await box.uncheck();
		await saveSettings(page);
		await page.reload();
		await expect(page.getByTestId('settings-round-to-5-centimes')).not.toBeChecked();

		// Réactiver.
		await page.getByTestId('settings-round-to-5-centimes').check();
		await saveSettings(page);

		const contactId = await createContactWithAddressViaApi(page, 'Client Arrondi');
		const ctx = await authedApiContext(page);
		let invoiceId = 0;
		try {
			const today = new Date().toISOString().slice(0, 10);
			const created = await ctx.post('/api/v1/invoices', {
				data: {
					contactId,
					date: today,
					dueDate: today,
					lines: [
						{ description: 'Prestation', quantity: '1', unitPrice: '114.19', vatRate: '8.10' },
					],
				},
			});
			expect(created.ok(), `création : ${created.status()}`).toBeTruthy();
			invoiceId = (await created.json()).id;
			const validated = await ctx.post(`/api/v1/invoices/${invoiceId}/validate`);
			expect(validated.ok(), `validation : ${validated.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto(`/invoices/${invoiceId}`);
		const row = page.getByTestId('invoice-detail-rounding');
		await expect(row).toBeVisible();
		await expect(row).toContainText('+0.01');
		await expect(page.locator('tfoot')).toContainText('123.45');
	} finally {
		// Remise à vide du compte (le réglage, lui, reste actif — sa valeur par défaut).
		await page.goto('/settings/invoicing');
		await page.getByTestId('settings-round-to-5-centimes').check();
		await page.getByTestId('settings-rounding-account').selectOption({ index: 0 });
		await saveSettings(page);
	}
});
