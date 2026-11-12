/**
 * Story 34-11a2 (#972, T7) — la carte de crédit, de l'écran au serveur.
 *
 * Une carte se crée sur *Comptes bancaires*, liée à un passif créé par l'API ;
 * elle s'affiche « •••• 4821 » dans la liste et dans le sélecteur de l'import,
 * où le CAMT ne lui est pas offert ; elle n'est pas proposée comme compte
 * source d'un lot de paiement ; un numéro complet est refusé avec son motif ;
 * sa description se modifie.
 *
 * Fixtures synthétiques : le numéro complet est le numéro de test public Visa,
 * les quatre chiffres valent 4821.
 */

import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function loginAsAdmin(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test('une carte de crédit : création, affichage, sélecteurs, refus du numéro complet, modification', async ({
	page,
}) => {
	await loginAsAdmin(page);
	const ctx = await authedApiContext(page);
	// Nom et quatre chiffres propres à ce passage : la spec se rejoue sur la même base.
	const number = `2${Date.now().toString().slice(-6)}`;
	const nom = `Visa ${number}`;
	const last4 = number.slice(-4);
	const cree = await ctx.post('/api/v1/accounts', {
		data: { number, name: 'Carte Visa E2E', accountType: 'Liability' },
	});
	expect(cree.ok(), `création du passif : ${cree.status()}`).toBeTruthy();
	const { id: passifId } = (await cree.json()) as { id: number };

	await page.goto('/bank-accounts');
	await expect(page.getByTestId('bank-accounts-page-title')).toBeVisible();
	await page.getByTestId('create-bank-account-button').click();
	await page.getByTestId('form-kind-credit-card').check();
	await expect(page.getByTestId('form-iban')).toHaveCount(0);
	await expect(page.getByTestId('form-is-primary')).toHaveCount(0);

	// Un numéro complet dans la description : refus nommé, rien de créé.
	await page.getByTestId('form-bank-name').fill(nom);
	await page.getByTestId('form-description').fill('Carte 4111 1111 1111 1111');
	await page.getByTestId('form-card-last4').fill(last4);
	await page.getByTestId('form-journal-account').selectOption(String(passifId));
	await page.getByTestId('form-submit').click();
	await expect(page.getByTestId('form-error')).toContainText('numéro de carte complet');

	// Corrigée, la carte se crée.
	await page.getByTestId('form-description').fill('Personnel');
	await page.getByTestId('form-submit').click();
	const ligne = page.locator('[data-testid^="bank-account-row-"]', { hasText: nom });
	await expect(ligne).toBeVisible({ timeout: 5000 });
	await expect(ligne.locator('[data-testid^="iban-cell-"]')).toContainText(`•••• ${last4}`);

	// Modifier sa description.
	await ligne.locator('[data-testid^="edit-button-"]').click();
	await expect(page.getByTestId('edit-kind')).toContainText('Carte de crédit');
	await expect(page.getByTestId('edit-iban')).toHaveCount(0);
	await page.getByTestId('edit-description').fill('Professionnel');
	await page.getByTestId('edit-form-submit').click();
	await expect(
		page.locator('[data-testid^="bank-account-row-"]', { hasText: `${nom} — Professionnel` }),
	).toBeVisible({ timeout: 5000 });

	// Import : la carte est proposée, « •••• 1234 », et le CAMT ne lui est pas offert.
	await page.goto('/bank-import');
	const select = page.getByTestId('bank-account-select');
	const option = select.locator('option', { hasText: nom });
	await expect(option).toHaveCount(1);
	await expect(option).toContainText(`•••• ${last4}`);
	await select.selectOption({ label: (await option.textContent())?.trim() ?? '' });
	await expect(page.getByTestId('bank-import-drop-zone-text')).toContainText("n'a pas d'IBAN");
	expect(await page.getByTestId('bank-import-file-input').getAttribute('accept')).not.toContain(
		'.xml',
	);

	// Lot de paiement : la carte n'est pas un compte source.
	await page.goto('/payment-batches');
	await page.getByTestId('payment-batch-new').click();
	const source = page.getByTestId('payment-batch-form').locator('select').first();
	await expect(source).toBeVisible();
	await expect(source.locator('option', { hasText: nom })).toHaveCount(0);
	await ctx.dispose();
});
