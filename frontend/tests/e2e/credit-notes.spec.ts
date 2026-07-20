import { expect, test } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';
// Story 21-8 (A4) — fixtures partagées (fin de la copie locale des helpers).
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

/**
 * Tests E2E — Avoirs / notes de crédit (Story 12.1).
 *
 * Parcours : créer + valider une facture (API), créer un avoir depuis l'UI de
 * la facture, vérifier que la facture passe « annulée » et que l'avoir est créé
 * avec son numéro + PDF téléchargeable.
 */

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

test.describe('Avoirs — création depuis une facture validée', () => {
	test('crée un avoir, annule la facture, PDF téléchargeable', async ({ page }) => {
		await login(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('Avoir Client'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		// Ouvre la facture validée et crée l'avoir.
		await page.goto(`/invoices/${invoiceId}`);
		await page.getByRole('button', { name: /Créer un avoir/i }).click();
		// Dialog de confirmation → bouton « Créer un avoir ».
		await page
			.getByRole('dialog')
			.getByRole('button', { name: /Créer un avoir/i })
			.click();

		// Redirection vers le détail de l'avoir.
		await expect(page).toHaveURL(/\/credit-notes\/\d+$/);
		await expect(page.getByTestId('credit-note-number')).toContainText(/AV-/);

		// La facture d'origine est désormais annulée.
		const ctx = await authedApiContext(page);
		try {
			const invRes = await ctx.get(`/api/v1/invoices/${invoiceId}`);
			expect(invRes.ok()).toBeTruthy();
			expect((await invRes.json()).status).toBe('cancelled');

			// PDF de l'avoir : 200 + %PDF.
			const cnUrl = page.url();
			const cnId = cnUrl.split('/').pop();
			const pdfRes = await ctx.get(`/api/v1/credit-notes/${cnId}/pdf`);
			expect(pdfRes.status()).toBe(200);
			expect(pdfRes.headers()['content-type']).toContain('application/pdf');
			const buf = await pdfRes.body();
			expect(buf.slice(0, 7).toString('utf8')).toMatch(/^%PDF-1\./);
		} finally {
			await disposeContextSafe(ctx);
		}
	});

	test('l’avoir apparaît dans la liste des avoirs', async ({ page }) => {
		await login(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('Avoir Liste'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
		const ctx = await authedApiContext(page);
		try {
			const res = await ctx.post('/api/v1/credit-notes', {
				data: { invoiceId, date: new Date().toISOString().slice(0, 10) },
			});
			expect(res.ok(), `create credit note failed: ${res.status()}`).toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto('/credit-notes');
		await expect(page.getByTestId('credit-notes-table')).toBeVisible();
		await expect(page.getByTestId('credit-note-row').first()).toBeVisible();
	});
});
