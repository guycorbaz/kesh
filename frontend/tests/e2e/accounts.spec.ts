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
		await expect(page.getByText('Plan comptable')).toBeVisible();
	});

	test('affiche l\'arborescence des comptes avec numeros', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Les comptes du plan PME doivent etre visibles
		await expect(page.getByText('1000')).toBeVisible();
		await expect(page.getByText('2000')).toBeVisible();
	});

	test('affiche le type de compte (badge)', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Les badges de type doivent etre presents
		await expect(page.getByText('Actif').first()).toBeVisible();
		await expect(page.getByText('Passif').first()).toBeVisible();
	});

	test('affiche le compteur de comptes', async ({ page }) => {
		await loginAndGoToAccounts(page);
		await expect(page.getByText(/\d+ comptes/)).toBeVisible();
	});
});

test.describe('Page plan comptable — CRUD', () => {
	test('ajout d\'un compte via dialog', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Ouvrir le dialog de creation
		await page.getByText('Nouveau compte').click();
		await expect(page.getByText('Ajoutez un compte')).toBeVisible();

		// Remplir le formulaire
		const testNumber = `9999`;
		await page.fill('#create-number', testNumber);
		await page.fill('#create-name', 'Compte de test E2E');

		// Soumettre
		await page.getByRole('button', { name: 'Créer' }).click();

		// Le toast de succes doit apparaitre
		await expect(page.getByText(`Compte ${testNumber} créé`)).toBeVisible();

		// Le compte doit apparaitre dans la liste
		await expect(page.getByText(testNumber)).toBeVisible();
		await expect(page.getByText('Compte de test E2E')).toBeVisible();
	});

	test('modification d\'un compte via dialog', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Cliquer sur le bouton modifier du premier compte visible
		const editButton = page.getByLabel(/Modifier/).first();
		await editButton.click();

		// Le dialog de modification doit s'ouvrir
		await expect(page.getByText('Le numéro n\'est pas modifiable')).toBeVisible();

		// Le champ numero doit etre desactive
		const numberField = page.locator('#edit-number');
		await expect(numberField).toBeDisabled();

		// Fermer sans modifier
		await page.getByRole('button', { name: 'Annuler' }).click();
	});

	test('toggle afficher les archives', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// La checkbox "Afficher les archives" doit exister
		await expect(page.getByText('Afficher les archivés')).toBeVisible();
	});
});
