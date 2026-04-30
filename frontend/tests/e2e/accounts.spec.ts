import { test, expect } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Plan comptable (Story 3.1)
 *
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true` + `KESH_HOST=127.0.0.1`.
 * Le `beforeAll` truncate la DB et re-seed via l'endpoint `/api/v1/_test/seed`
 * → état déterministe indépendant de l'ordre des specs.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	// Clear auth tokens after each test to prevent token bleed to next test
	await clearAuthStorage(page);
});

/** Helper : login as admin and navigate to /accounts. */
async function loginAndGoToAccounts(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
	await page.goto('/accounts');
	await expect(page).toHaveURL('/accounts');
}

test.describe('Page plan comptable — affichage', () => {
	test('affiche le titre Plan comptable', async ({ page }) => {
		await loginAndGoToAccounts(page);
		await expect(page.getByRole('heading', { name: 'Plan comptable', level: 1 })).toBeVisible();
	});

	test('affiche l\'arborescence des comptes avec numeros', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Les comptes du plan PME doivent etre visibles (testid scope ligne)
		await expect(page.locator('[data-testid="account-row-1000"]')).toBeVisible();
		await expect(page.locator('[data-testid="account-row-2000"]')).toBeVisible();
	});

	test('affiche le type de compte (badge)', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Les badges de type sur les lignes 1000 (Actif) et 2000 (Passif)
		await expect(page.locator('[data-testid="account-row-1000-type-badge"]')).toContainText('Actif');
		await expect(page.locator('[data-testid="account-row-2000-type-badge"]')).toContainText('Passif');
	});

	test('affiche le compteur de comptes', async ({ page }) => {
		await loginAndGoToAccounts(page);
		await expect(page.getByText(/\d+ comptes/)).toBeVisible();
	});
});

test.describe('Page plan comptable — CRUD', () => {
	test('ajout d\'un compte via dialog', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Ouvrir le dialog de creation (description visible = dialog ouvert)
		await page.locator('[data-testid="account-create-button"]').click();
		await expect(page.getByRole('dialog')).toContainText('Ajoutez un compte');

		// Remplir le formulaire
		const testNumber = `9999`;
		await page.fill('#create-number', testNumber);
		await page.fill('#create-name', 'Compte de test E2E');

		// Soumettre
		await page.locator('[data-testid="account-create-dialog-submit"]').click();

		// Le toast de succes doit apparaitre (svelte-sonner — region Notifications)
		await expect(page.getByLabel(/Notifications/)).toContainText(`Compte ${testNumber} créé`);

		// Le compte doit apparaitre dans la liste — testid scope ligne (évite collision avec le toast)
		await expect(page.locator(`[data-testid="account-row-${testNumber}"]`)).toBeVisible();
		await expect(page.locator(`[data-testid="account-row-${testNumber}-name"]`)).toContainText('Compte de test E2E');
	});

	test('modification d\'un compte via dialog', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Cliquer sur le bouton modifier du premier compte visible (1000 du plan PME seedé)
		await page.locator('[data-testid="account-row-1000-edit-button"]').click();

		// Le dialog de modification doit s'ouvrir
		await expect(page.getByText('Le numéro n\'est pas modifiable')).toBeVisible();

		// Le champ numero doit etre desactive
		const numberField = page.locator('#edit-number');
		await expect(numberField).toBeDisabled();

		// Fermer sans modifier — testid scoped au dialog Modifier (pas de collision avec Créer/Archiver)
		await page.locator('[data-testid="account-edit-dialog-cancel"]').click();
	});

	test('toggle afficher les archives', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// La checkbox "Afficher les archives" doit exister
		await expect(page.locator('[data-testid="account-show-archived-toggle"]')).toBeVisible();
	});
});
