import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
	ensurePrimaryBankAccountViaApi,
} from './helpers/api-fixtures';

/**
 * Tests E2E — une QR-facture dont le débiteur porte « & » (Story 33-12a, #941).
 *
 * `Müller & Söhne` est au jeu de caractères de la directive (§ 4.1.1) : le PDF se
 * produit, là où Kesh répondait « caractère non autorisé » à un nom réel. Seul un
 * E2E voit le nom traverser la route jusqu'au fichier.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('un débiteur au nom portant « & » reçoit sa QR-facture', async ({ page }) => {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');

	await ensurePrimaryBankAccountViaApi(page);
	const nom = `Müller & Söhne ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
	const contactId = await createContactWithAddressViaApi(page, nom);
	const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

	const ctx = await authedApiContext(page);
	try {
		const pdf = await ctx.get(`/api/v1/invoices/${invoiceId}/pdf`);
		expect(pdf.status(), await pdf.text()).toBe(200);
		expect(pdf.headers()['content-type']).toContain('application/pdf');
		expect((await pdf.body()).subarray(0, 5).toString()).toBe('%PDF-');
	} finally {
		await disposeContextSafe(ctx);
	}
});
