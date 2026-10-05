/**
 * Story 25-4-e (#495) — un montant minimum configurable sous lequel une facture
 * n'est pas émise.
 *
 * Fixer un seuil de 500.00 dans Paramètres → Facturation, tenter de valider un
 * brouillon de 100.00 (taux 0 %, total déjà rond : aucun compte d'arrondi requis),
 * voir le refus nommant les deux montants, puis effacer le seuil. Le test remet le
 * seuil à vide à la fin : la base est partagée entre specs.
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

test('une facture sous le montant minimum n’est pas validée', async ({ page }) => {
	await login(page);
	try {
		await page.goto('/settings/invoicing');
		await page.getByTestId('settings-minimum-invoice-amount').fill('500.00');
		await saveSettings(page);

		const contactId = await createContactWithAddressViaApi(page, 'Client Minimum');
		const ctx = await authedApiContext(page);
		let invoiceId = 0;
		try {
			const today = new Date().toISOString().slice(0, 10);
			const created = await ctx.post('/api/v1/invoices', {
				data: {
					contactId,
					date: today,
					dueDate: today,
					lines: [{ description: 'Petite prestation', quantity: '1', unitPrice: '100.00', vatRate: '0' }],
				},
			});
			expect(created.ok(), `création : ${created.status()}`).toBeTruthy();
			invoiceId = (await created.json()).id;
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto(`/invoices/${invoiceId}`);
		await page.getByTestId('invoice-validate-button').click();
		await page.getByTestId('invoice-validate-confirm').click();
		const error = page.getByTestId('invoice-validate-error');
		await expect(error).toBeVisible();
		await expect(error).toContainText('100.00');
		await expect(error).toContainText('500.00');
	} finally {
		await page.goto('/settings/invoicing');
		await page.getByTestId('settings-minimum-invoice-amount').fill('');
		await saveSettings(page);
	}
});
