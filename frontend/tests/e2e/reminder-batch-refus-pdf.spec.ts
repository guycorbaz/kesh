/**
 * Story 35-2b (#955, AC5) — le rapport d'un lot de rappels dit pourquoi le PDF
 * d'une facture est refusé.
 *
 * Le client habite `Ve–vey` (tiret demi-cadratin, U+2013, que la QR-facture
 * refuse) : la facture échue est proposée au lot, son PDF est refusé, et la
 * ligne du rapport porte la phrase du serveur (`failed[].details.message`) après
 * le libellé. Seul un E2E voit la valeur traverser la frontière HTTP jusqu'à
 * l'écran ; Vitest ne teste que l'affichage, Rust que la réponse.
 *
 * Montage sur le modèle de `dunning-roundtrip.spec.ts` (niveaux semés au premier
 * GET de `/settings/dunning`).
 */
import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	createAndValidateInvoiceViaApi,
	ensurePrimaryBankAccountViaApi,
	overdueDate,
} from './helpers/api-fixtures';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('le rapport du lot nomme la localité refusée par la QR-facture', async ({ page }) => {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');

	await ensurePrimaryBankAccountViaApi(page);
	await page.goto('/settings/dunning');
	await expect(page.getByTestId('dunning-level-row')).toHaveCount(3);

	const nom = `Débiteur Vevey ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
	// L'e-mail est fourni : sans lui, la ligne serait CONTACT_EMAIL_MISSING.
	const contactId = await createContactWithAddressViaApi(
		page,
		nom,
		'debiteur-vevey@example.ch',
		undefined,
		undefined,
		'Ve–vey',
	);
	await createAndValidateInvoiceViaApi(page, contactId, overdueDate());

	await page.goto('/invoices/reminders');
	const groupe = page.locator('div.rounded', { hasText: nom });
	await groupe.getByTestId('reminder-row').first().getByTestId('reminder-batch-checkbox').check();
	await expect(page.getByTestId('reminder-selected-count')).toContainText('1');
	await page.getByTestId('reminder-batch-send').click();

	const ligne = page.getByTestId('reminder-batch-failed-row');
	await expect(ligne).toHaveCount(1, { timeout: 10000 });
	await expect(ligne).toContainText('Facture non imprimable en PDF');
	await expect(ligne).toContainText('localité du client');
	await expect(ligne).toContainText('U+2013');
});
