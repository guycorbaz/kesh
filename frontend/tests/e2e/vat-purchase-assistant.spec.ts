import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Assistant TVA achat (Story 18-1c).
 *
 * Vérifie le round-trip : ouvrir l'assistant → saisir charge/HT/taux/contrepartie
 * → insérer les 3 lignes (D charge / D impôt préalable / C contrepartie TTC) →
 * soumettre via le flux POST /journal-entries existant → l'écriture apparaît.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true + seed `with-company`. Le seed
 * `seed_accounting_company` configure désormais `default_vat_recoverable_account_id`
 * = compte 1000 (Story 18-1c, fixture), sans quoi l'assistant serait en mode
 * « config requise ».
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function goToCreateForm(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	await page.goto('/journal-entries');
	await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
	await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();
}

test.describe('Assistant TVA achat', () => {
	test('insère 3 lignes équilibrées et soumet l’écriture', async ({ page }) => {
		await goToCreateForm(page);

		const panel = page.getByTestId('vat-purchase-assistant');
		await expect(panel).toBeVisible();

		// Ouvrir le panneau.
		await panel.getByRole('button', { name: /Assistant TVA achat/ }).click();

		// Compte de charge = 4000 (Charges CI, Expense).
		const accountInputs = panel.locator('input[aria-autocomplete="list"]');
		await accountInputs.nth(0).fill('4000');
		await panel.getByRole('option').first().click();

		// Montant HT.
		await panel.locator('input[inputmode="decimal"]').first().fill('1000');

		// Taux TVA 8.10 % (le contenu du Select est rendu en portail → page-level).
		await panel.locator('[data-slot="select-trigger"]').click();
		await page.getByRole('option', { name: /8\.10/ }).click();

		// Contrepartie = 1100 (Banque CI). Différent de charge (4000) et du
		// compte d'impôt préalable (1000).
		await accountInputs.nth(1).fill('1100');
		await panel.getByRole('option').first().click();

		// Insérer les lignes.
		await panel.getByRole('button', { name: /Insérer les lignes/ }).click();

		// Le libellé est auto-rempli (le brouillon était vierge). NB : Fluent entoure
		// la variable interpolée de marques d'isolation directionnelle (U+2068/U+2069),
		// d'où les `.*` autour du taux.
		await expect(page.locator('#entry-description')).toHaveValue(/TVA.*8\.10.*récupérable/);

		// L'écriture générée est équilibrée (créance 1081 = charge 1000 + TVA 81).
		await expect(page.getByText(/✓ Équilibré/)).toBeVisible();

		// Le journal est forcé à « Achats ».
		await expect(page.locator('#entry-journal')).toContainText('Achats');

		// Soumettre.
		await page.getByRole('button', { name: 'Valider' }).click();

		// L'écriture apparaît dans la liste.
		await expect(page.getByText(/TVA.*8\.10.*récupérable/).first()).toBeVisible({
			timeout: 5000
		});
	});

	test('taux exempt (0 %) → 2 lignes, aucune ligne d’impôt préalable', async ({ page }) => {
		await goToCreateForm(page);

		const panel = page.getByTestId('vat-purchase-assistant');
		await panel.getByRole('button', { name: /Assistant TVA achat/ }).click();

		const accountInputs = panel.locator('input[aria-autocomplete="list"]');
		await accountInputs.nth(0).fill('4000');
		await panel.getByRole('option').first().click();
		await panel.locator('input[inputmode="decimal"]').first().fill('500');

		await panel.locator('[data-slot="select-trigger"]').click();
		// Le taux exempt (0 %) — libellé « Exonéré / 0 % ».
		await page.getByRole('option', { name: /0\.00|Exonéré/ }).first().click();

		await accountInputs.nth(1).fill('1100');
		await panel.getByRole('option').first().click();

		await panel.getByRole('button', { name: /Insérer les lignes/ }).click();

		// 2 lignes seulement : la créance = HT (pas de TVA), équilibré.
		await expect(page.getByText(/✓ Équilibré/)).toBeVisible();
		await page.fill('#entry-description', 'Achat exempt E2E');
		await page.getByRole('button', { name: 'Valider' }).click();
		await expect(page.getByText(/Achat exempt E2E/).first()).toBeVisible({ timeout: 5000 });
	});
});
