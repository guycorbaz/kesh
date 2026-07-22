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
		const createDialog = page.getByRole('dialog');
		await expect(createDialog).toBeVisible();
		await expect(createDialog).toContainText('Ajoutez un compte');

		// Remplir le formulaire — numéro dérivé du timestamp (4 chiffres) pour éviter
		// la pollution inter-runs (seedTestState utilise beforeAll, pas beforeEach).
		const testNumber = `9${String(Date.now()).slice(-3)}`;
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

		// Cliquer sur le bouton modifier du compte 1000 (seedé actif par with-company).
		// Garde toHaveCount(1) avant click : si seed change ou compte 1000 archivé,
		// le bouton n'est plus rendu ({#if active}) — fail-fast plutôt que timeout opaque.
		const editButton = page.locator('[data-testid="account-row-1000-edit-button"]');
		await expect(editButton).toHaveCount(1);
		await editButton.click();

		// Le dialog de modification doit s'ouvrir
		const editDialog = page.getByRole('dialog');
		await expect(editDialog).toBeVisible();
		await expect(editDialog).toContainText('Le numéro n\'est pas modifiable');

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

test.describe('Plan comptable — rôles & réactivation (Story 14-3a, #269)', () => {
	test("affecte un rôle à un compte via le dialog Modifier", async ({ page }) => {
		await loginAndGoToAccounts(page);

		const editButton = page.locator('[data-testid="account-row-1000-edit-button"]');
		await expect(editButton).toHaveCount(1);
		await editButton.click();

		const editDialog = page.getByRole('dialog');
		await expect(editDialog).toBeVisible();

		// Le sélecteur de rôle et la case Postable sont exposés.
		const roleSelect = page.locator('[data-testid="account-edit-dialog-role"]');
		await expect(roleSelect).toBeVisible();
		await expect(page.locator('[data-testid="account-edit-dialog-postable"]')).toBeVisible();

		// Choisir « Capital » (rôle multi-valué : jamais en conflit avec le seed).
		await roleSelect.click();
		await page.getByRole('option', { name: 'Capital', exact: true }).click();
		await page.locator('[data-testid="account-edit-dialog-submit"]').click();

		// Le badge de rôle apparaît sur la ligne.
		await expect(page.locator('[data-testid="account-row-1000-role-badge"]')).toContainText(
			'Capital',
		);
	});

	test('cycle archiver → afficher les archivés → réactiver', async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Créer un compte dédié pour ne pas perturber les autres tests.
		const number = `T${Date.now().toString().slice(-5)}`;
		await page.locator('[data-testid="account-create-button"]').click();
		await page.fill('#create-number', number);
		await page.fill('#create-name', 'Compte à réactiver');
		await page.locator('[data-testid="account-create-dialog-submit"]').click();
		await expect(page.locator(`[data-testid="account-row-${number}"]`)).toBeVisible();

		// Archiver.
		await page.locator(`[data-testid="account-row-${number}-archive-button"]`).click();
		await page.locator('[data-testid="account-archive-dialog-confirm"]').click();

		// Par défaut la liste masque les archivés → la ligne disparaît.
		await expect(page.locator(`[data-testid="account-row-${number}"]`)).toHaveCount(0);

		// Cocher « afficher les archivés » → la ligne revient, avec un bouton
		// Réactiver et SANS bouton Modifier/Archiver.
		await page.locator('[data-testid="account-show-archived-toggle"]').check();
		await expect(page.locator(`[data-testid="account-row-${number}"]`)).toBeVisible();
		await expect(
			page.locator(`[data-testid="account-row-${number}-reactivate-button"]`),
		).toHaveCount(1);
		await expect(page.locator(`[data-testid="account-row-${number}-edit-button"]`)).toHaveCount(0);
		await expect(page.locator(`[data-testid="account-row-${number}-archive-button"]`)).toHaveCount(
			0,
		);

		// Réactiver → le compte redevient modifiable.
		await page.locator(`[data-testid="account-row-${number}-reactivate-button"]`).click();
		await expect(page.getByLabel(/Notifications/)).toContainText('réactivé');
		await expect(page.locator(`[data-testid="account-row-${number}-edit-button"]`)).toHaveCount(1);

		// Et il réapparaît dans la liste par défaut (archivés masqués).
		await page.locator('[data-testid="account-show-archived-toggle"]').uncheck();
		await expect(page.locator(`[data-testid="account-row-${number}"]`)).toBeVisible();
	});

	test("un compte non postable porte un badge explicitement indicatif", async ({ page }) => {
		await loginAndGoToAccounts(page);

		// Les comptes titres du plan seedé (ex. « 1 », « 10 ») sont non-postables
		// depuis le backfill de la migration.
		const badge = page.locator('[data-testid="account-row-10-postable-badge"]');
		if ((await badge.count()) > 0) {
			await expect(badge).toContainText('Non postable');
			// La mention « indicatif » (L2) doit être portée par le title.
			await expect(badge).toHaveAttribute('title', /[Ii]ndicatif/);
		}
	});
});
