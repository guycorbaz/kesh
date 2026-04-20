import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.beforeEach(async ({ page }) => {
	// Clear localStorage to isolate each test and prevent token bleed from previous tests
	await page.context().clearCookies();
});

/**
 * Tests E2E — Carnet d'adresses (Story 4.1)
 *
 * Prérequis seed DB :
 * - un admin bootstrap (admin / admin123)
 * - une `companies` active (créée par seed_demo ou onboarding)
 * - Pattern identique à `journal-entries.spec.ts` — les helpers `login` +
 *   `seed_demo` sont déjà utilisés par les autres specs.
 */

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function goToContacts(page: import('@playwright/test').Page) {
	await login(page);
	await page.goto('/contacts');
	await expect(page).toHaveURL(/\/contacts/);
}

test.describe('Page contacts — affichage', () => {
	test("affiche le titre et le bouton Nouveau contact", async ({ page }) => {
		await goToContacts(page);
		await expect(page.getByRole('heading', { name: /Carnet d'adresses/ })).toBeVisible();
		await expect(page.getByRole('button', { name: /Nouveau contact/ })).toBeVisible();
	});
});

test.describe('Page contacts — CRUD', () => {
	test('création nominale d\'un contact Entreprise', async ({ page }) => {
		await goToContacts(page);

		const uniqueName = `TestContact E2E ${Date.now()}`;

		await page.getByRole('button', { name: /Nouveau contact/ }).click();
		await expect(page.getByRole('heading', { name: /Nouveau contact/ })).toBeVisible();

		await page.fill('#form-name', uniqueName);
		await page.fill('#form-email', 'test@example.ch');
		await page.getByRole('button', { name: 'Créer' }).click();

		// Le nouveau contact apparaît dans la liste.
		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

		// Cleanup : archiver le contact pour éviter de polluer les tests suivants.
		const row = page.locator('tr', { hasText: uniqueName }).first();
		await row.getByRole('button', { name: /Archiver/ }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
	});

	test("validation IDE invalide affiche un message d'erreur", async ({ page }) => {
		await goToContacts(page);

		await page.getByRole('button', { name: /Nouveau contact/ }).click();

		const uniqueName = `TestContact IDE ${Date.now()}`;
		await page.fill('#form-name', uniqueName);
		// CHE-109.322.552 = checksum invalide (dernier chiffre décalé).
		// CHE-000.000.000 est VALIDE (checksum 0 modulo 11) — ne PAS l'utiliser ici.
		await page.fill('#form-ide', 'CHE-109.322.552');

		// La validation client-side accepte le format, le bouton est actif.
		const submitBtn = page.getByRole('button', { name: 'Créer' });
		await submitBtn.click();

		// Le backend rejette avec message d'erreur (toast ou inline).
		// Le message peut venir de notifyError (toast) ou du formError inline.
		await expect(
			page.getByText(/IDE|invalid/i).first()
		).toBeVisible({ timeout: 5000 });
	});

	test('archivage avec confirmation et disparition de la liste', async ({ page }) => {
		await goToContacts(page);

		// Créer un contact ad-hoc.
		const uniqueName = `TestContact Archive ${Date.now()}`;
		await page.getByRole('button', { name: /Nouveau contact/ }).click();
		await page.fill('#form-name', uniqueName);
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

		// Archiver.
		const row = page.locator('tr', { hasText: uniqueName }).first();
		await row.getByRole('button', { name: /Archiver/ }).click();

		// Confirmation dialog.
		await expect(page.getByRole('dialog').getByText(/Archiver le contact/)).toBeVisible();
		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();

		// Le contact disparaît de la liste par défaut.
		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
	});

	test('filtre par type Entreprise', async ({ page }) => {
		await goToContacts(page);

		// Sélectionner le filtre type = Entreprise.
		await page.locator('#filter-type').selectOption('Entreprise');

		// Attendre le rechargement (debounce 300ms pour search, mais filtres type
		// déclenchent immédiatement via $effect).
		await page.waitForTimeout(500);

		// URL reflète le filtre.
		await expect(page).toHaveURL(/contactType=Entreprise/);
	});

	test('URL state préservé après reload', async ({ page }) => {
		await goToContacts(page);

		// Appliquer un filtre.
		await page.locator('#filter-type').selectOption('Entreprise');
		await page.waitForTimeout(500);
		expect(page.url()).toContain('contactType=Entreprise');

		// Reload.
		await page.reload();
		await page.waitForTimeout(500);

		// Le filtre est restauré.
		const selectedValue = await page.locator('#filter-type').inputValue();
		expect(selectedValue).toBe('Entreprise');
	});

	// Reportés à Story 4.2 ou post-MVP.
	test.skip('filtre combinés (type + client + search)', async () => {});
	test.skip('pagination navigation précédent/suivant', async () => {});
});

test.describe('Page contacts — accessibilité', () => {
	test('axe-core sans violations sur la liste contacts', async ({ page }) => {
		await goToContacts(page);
		await page.waitForLoadState('networkidle');
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});
});
