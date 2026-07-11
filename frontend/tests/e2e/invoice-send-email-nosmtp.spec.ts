/**
 * Tests E2E — Fallback zéro-config SMTP (Story 20-4, décision #3 epic-20).
 *
 * ⚠️ Exige un backend démarré **SANS** vars SMTP (NoopMailer,
 * `/health.smtpConfigured=false`) — configuration INVERSE du run principal.
 * Gaté par `KESH_E2E_NO_SMTP=1` pour ne pas échouer dans le run standard.
 * Recette des deux runs : docs/testing.md.
 */
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

const BACKEND_URL = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1';

test.skip(
	process.env.KESH_E2E_NO_SMTP !== '1',
	'Run dédié : backend SANS vars SMTP + KESH_E2E_NO_SMTP=1 (cf. docs/testing.md)',
);

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test.describe('Envoi de facture — fallback zéro-config SMTP (Story 20-4)', () => {
	test('backend sans SMTP : /health smtpConfigured=false, bouton grisé + tooltip', async ({
		page,
	}) => {
		// Garde de recette : ce run DOIT tourner contre un backend sans SMTP.
		const health = await page.request.get(`${BACKEND_URL}/health`);
		const body = await health.json();
		expect(
			body.smtpConfigured,
			'backend démarré AVEC SMTP — ce spec exige la config inverse (cf. docs/testing.md)',
		).toBe(false);

		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(
			page,
			`NoSmtp ${Date.now()}`,
			`n-${Date.now()}@example.ch`,
		);
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		await page.goto(`/invoices/${invoiceId}`);
		const button = page.getByTestId('send-email-button');
		await expect(button).toBeVisible();
		await expect(button).toBeDisabled();

		// Le tooltip s'ouvre au hover du wrapper (un <button disabled> ne fire
		// pas les events — d'où le span trigger, cf. 20-3b2).
		await page.getByTestId('send-email-disabled-wrapper').hover();
		await expect(page.getByText(/KESH_SMTP_/)).toBeVisible({ timeout: 5000 });
	});
});
