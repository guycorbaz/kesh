/**
 * Story 25-4-d2b (#490 ; API : 25-4-d2a, #384) — solder le reste d'une facture
 * depuis sa fiche.
 *
 * Montage : un compte d'escompte désigné par le formulaire des paramètres (le
 * 4000 *Charges* du seed), une facture validée de 972.90 TTC à 8.1 % (multiple
 * de 0.05 : aucun compte d'arrondi requis), réglée de 500.— par l'API. Puis, à
 * l'écran : solder le reste en escompte, vérifier la liste et le motif qui
 * bloque l'annulation du règlement antérieur, annuler le solde.
 *
 * Nettoyage : le compte d'escompte remis à vide — la base est partagée entre
 * specs (patron `invoice-settings-write-off-accounts.spec.ts`).
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
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

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

async function setDiscountAccount(page: Page, number: string | null): Promise<void> {
	await page.goto('/settings/invoicing');
	const select = page.getByTestId('settings-discount-account');
	// Les comptes arrivent après le rendu : attendre une option de compte.
	await expect(select.locator('option').nth(1)).toBeAttached();
	if (number === null) {
		await select.selectOption({ index: 0 });
	} else {
		const value = await select
			.locator('option', { hasText: number })
			.first()
			.getAttribute('value');
		expect(value, `compte ${number} proposé`).toBeTruthy();
		await select.selectOption(value!);
	}
	const saved = page.waitForResponse(
		(r) => r.url().includes('/api/v1/company/invoice-settings') && r.request().method() === 'PUT',
	);
	await page.getByTestId('settings-invoicing-save').click();
	expect((await saved).status()).toBe(200);
}

test('solder le reste en escompte depuis la fiche, puis annuler le solde', async ({ page }) => {
	await login(page);
	try {
		await setDiscountAccount(page, '4000');

		const contactId = await createContactWithAddressViaApi(page, 'Client Escompte');
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
		const ctx = await authedApiContext(page);
		try {
			const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as {
				id: number;
				number: string;
			}[];
			const caisse = accounts.find((a) => a.number === '1000');
			expect(caisse, 'compte 1000 du seed').toBeTruthy();
			const settled = await ctx.post(`/api/v1/invoices/${invoiceId}/settlements`, {
				data: {
					settlementType: 'internal_account',
					accountId: caisse!.id,
					amount: '500.00',
					settledOn: new Date().toISOString().slice(0, 10),
				},
			});
			expect(settled.ok(), `règlement : ${settled.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByTestId('invoice-amount-due')).toContainText('472.90');

		// Solder le reste en escompte.
		await page.getByTestId('write-off-open').click();
		await expect(page.getByTestId('write-off-amount')).toContainText('472.90');
		await page.getByTestId('write-off-nature-discount').check();
		await page.getByTestId('write-off-confirm').click();

		await expect(page.getByTestId('invoice-amount-due')).toContainText('0.00');
		await expect(page.getByTestId('invoice-amount-settled')).toContainText('500.00');
		await expect(page.getByTestId('invoice-amount-written-off')).toContainText('472.90');
		const modes = page.getByTestId('invoice-settlement-mode');
		await expect(modes).toHaveCount(2);
		await expect(modes.nth(1)).toContainText('Escompte accordé');
		// Le règlement antérieur ne s'annule pas tant que le solde existe.
		await expect(page.getByTestId('invoice-settlement-cancel-blocked')).toContainText(
			"annulez d'abord le solde",
		);

		// Story 25-4-d2c — AVANT l'annulation (un solde annulé quitte le rapport) :
		// le rapport TVA de la période retranche la TVA du solde. Seul test qui voit
		// les champs des diminutions traverser la frontière HTTP.
		await page.goto('/reports');
		await page.waitForLoadState('networkidle');
		await page.getByRole('tab', { name: /^TVA$/ }).click();
		await page.getByRole('button', { name: /générer/i }).click();
		await expect(page.getByTestId('vat-write-off-section')).toBeVisible();
		await expect(page.getByTestId('vat-total-vat-due-net')).toBeVisible();
		await page.goto(`/invoices/${invoiceId}`);

		// Annuler le solde : le reste réapparaît.
		const cancel = page.getByTestId('invoice-settlement-cancel');
		await expect(cancel).toHaveCount(1);
		await expect(cancel).toContainText('Annuler le solde');
		await cancel.click();
		await page.getByTestId('invoice-settlement-cancel-confirm').click();
		await expect(page.getByTestId('invoice-amount-due')).toContainText('472.90');
		await expect(page.getByTestId('invoice-settlement-mode')).toHaveCount(1);
	} finally {
		await setDiscountAccount(page, null);
	}
});
