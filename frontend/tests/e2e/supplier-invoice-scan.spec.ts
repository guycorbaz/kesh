import { expect, test } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Scan QR-facture au formulaire de facture fournisseur (Story 12.4, #191).
 *
 * Seul test qui exerce le **décodage jsQR réel** (navigateur) : on charge la
 * fixture QR PNG dans l'input fichier caché, jsQR décode le payload SPC, le
 * backend le parse, et le formulaire est pré-rempli. La fixture réutilise
 * `spc_e2e_invoice.png` (IBAN classique CH93…957, montant 100.00,
 * créancier « Fournisseur E2E SA »).
 */

const FIXTURE = path.join(
	path.dirname(fileURLToPath(import.meta.url)),
	'fixtures',
	'spc_e2e_invoice.png',
);

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

test.describe('Scan QR-facture (pré-remplissage)', () => {
	test('charger une image QR pré-remplit IBAN, montant et créancier', async ({ page }) => {
		await login(page);
		await page.goto('/supplier-invoices');

		// Ouvrir le formulaire de saisie.
		await page.getByTestId('supplier-invoice-new').click();
		await expect(page.getByTestId('supplier-invoice-form')).toBeVisible();

		// Charger la fixture QR dans l'input caché (jsQR décode côté navigateur).
		await page.getByTestId('supplier-invoice-scan-file').setInputFiles(FIXTURE);

		// Le créancier détecté s'affiche → le scan + parse a réussi.
		await expect(page.getByTestId('supplier-invoice-scan-creditor')).toContainText(
			'Fournisseur E2E SA',
		);

		// IBAN classique + montant attendu pré-remplis (toHaveValue = propriété .value).
		await expect(page.getByTestId('supplier-invoice-iban')).toHaveValue('CH9300762011623852957');
		await expect(page.getByTestId('supplier-invoice-expected-amount')).toHaveValue('100.00');
	});
});
