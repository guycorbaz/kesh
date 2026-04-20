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
 * Tests E2E — Catalogue produits (Story 4.2)
 *
 * Prérequis seed DB identiques à contacts.spec.ts :
 * - admin bootstrap (admin / admin123)
 * - une `companies` active
 */

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function goToProducts(page: import('@playwright/test').Page) {
	await login(page);
	await page.goto('/products');
	await expect(page).toHaveURL(/\/products/);
}

function uniq(prefix: string): string {
	// Suffixe unique tolérant aux exécutions parallèles (ms + pid + random).
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

async function createProduct(
	page: import('@playwright/test').Page,
	name: string,
	price: string,
	vatValue?: '0.00' | '2.60' | '3.80' | '8.10'
) {
	await page.getByRole('button', { name: /Nouveau produit/ }).click();
	await page.fill('#form-name', name);
	await page.fill('#form-price', price);
	if (vatValue) {
		// `selectOption` avec `{ value }` est déterministe ; `{ label }` ne matche
		// pas les regex et dépend de la locale courante — les valeurs TVA sont
		// stables côté backend et constituent un point d'ancrage sûr.
		await page.locator('#form-vat-rate').selectOption(vatValue);
	}
	await page.getByRole('button', { name: 'Créer' }).click();
	await expect(page.locator('tbody').getByText(name)).toBeVisible({ timeout: 5000 });
}

async function archiveRow(page: import('@playwright/test').Page, name: string) {
	const row = page.locator('tr', { hasText: name }).first();
	await row.getByRole('button', { name: /Archiver/ }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
	// Scopé à `tbody` pour éviter les faux positifs avec le toast de succès
	// qui peut répéter brièvement le nom du produit.
	await expect(page.locator('tbody').getByText(name)).toHaveCount(0, { timeout: 5000 });
}

test.describe('Page catalogue — affichage', () => {
	test('affiche le titre et le bouton Nouveau produit', async ({ page }) => {
		await goToProducts(page);
		await expect(page.getByRole('heading', { name: /Catalogue/ })).toBeVisible();
		await expect(page.getByRole('button', { name: /Nouveau produit/ })).toBeVisible();
	});
});

test.describe('Page catalogue — accessibilité', () => {
	test('axe-core sans violations sur la liste produits', async ({ page }) => {
		await goToProducts(page);
		await page.waitForLoadState('networkidle');
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
	});
});

test.describe('Page catalogue — CRUD', () => {
	test('création nominale d\'un produit', async ({ page }) => {
		await goToProducts(page);

		const uniqueName = `TestProduct E2E ${Date.now()}`;

		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await expect(page.getByRole('heading', { name: /Nouveau produit/ })).toBeVisible();

		await page.fill('#form-name', uniqueName);
		await page.fill('#form-price', '1500.00');
		// Taux TVA 8.10 % est sélectionné par défaut.
		await page.getByRole('button', { name: 'Créer' }).click();

		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

		// Cleanup : archiver pour ne pas polluer les tests suivants.
		const row = page.locator('tr', { hasText: uniqueName }).first();
		await row.getByRole('button', { name: /Archiver/ }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
	});

	test('archivage avec confirmation et disparition de la liste', async ({ page }) => {
		await goToProducts(page);

		const uniqueName = `TestProduct Arch ${Date.now()}`;
		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await page.fill('#form-name', uniqueName);
		await page.fill('#form-price', '42.00');
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

		const row = page.locator('tr', { hasText: uniqueName }).first();
		await row.getByRole('button', { name: /Archiver/ }).click();

		await expect(page.getByRole('dialog').getByText(/Archiver le produit/)).toBeVisible();
		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();

		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
	});

	test('filtre recherche reflété dans URL et résultats', async ({ page }) => {
		await goToProducts(page);

		const uniqueName = `TestProduct Search ${Date.now()}`;
		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await page.fill('#form-name', uniqueName);
		await page.fill('#form-price', '10.00');
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

		// Rechercher par nom unique → URL contient `search=` (attente event-driven
		// plutôt que `waitForTimeout`, pour éviter les flakes CI sur machines lentes).
		await page.fill('#filter-search', uniqueName);
		await page.waitForURL(/search=/, { timeout: 2000 });
		await expect(page.locator('tbody').getByText(uniqueName)).toBeVisible();

		// Cleanup.
		const row = page.locator('tr', { hasText: uniqueName }).first();
		await row.getByRole('button', { name: /Archiver/ }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
	});
});

test.describe('Page catalogue — validation & erreurs', () => {
	test('format prix invalide affiche un message inline et désactive Créer', async ({ page }) => {
		await goToProducts(page);
		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await page.fill('#form-name', uniq('TestProduct Invalid'));
		await page.fill('#form-price', '10.123456'); // > 4 décimales
		// Feedback inline visible sans avoir cliqué sur Créer.
		await expect(page.getByText(/prix invalide/i)).toBeVisible();
		await expect(page.getByRole('button', { name: 'Créer' })).toBeDisabled();
		// Correction → activation.
		await page.fill('#form-price', '10.50');
		await expect(page.getByRole('button', { name: 'Créer' })).toBeEnabled();
		await page.getByRole('button', { name: 'Annuler' }).click();
	});

	test('création d\'un nom en doublon remonte une erreur', async ({ page }) => {
		await goToProducts(page);
		const name = uniq('TestProduct Dup');
		await createProduct(page, name, '5.00');

		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await page.fill('#form-name', name);
		await page.fill('#form-price', '7.00');
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(/existe déjà|already exists/i)).toBeVisible({ timeout: 5000 });
		await page.getByRole('button', { name: 'Annuler' }).click();

		await archiveRow(page, name);
	});
});

test.describe('Page catalogue — filtres, tri & pagination', () => {
	test('toggle "Inclure archivés" réaffiche un produit archivé', async ({ page }) => {
		await goToProducts(page);
		const name = uniq('TestProduct Arch Toggle');
		await createProduct(page, name, '12.00');

		// Archiver.
		const row = page.locator('tr', { hasText: name }).first();
		await row.getByRole('button', { name: /Archiver/ }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
		await expect(page.getByText(name)).toHaveCount(0, { timeout: 5000 });

		// Activer toggle → produit ré-apparaît (opacité réduite).
		await page.getByLabel(/Inclure archivés|Include archived|Archivierte|archiviati/i).check();
		await expect(page.getByText(name)).toBeVisible({ timeout: 5000 });
		await expect(page).toHaveURL(/includeArchived=true/);
	});

	test('tri par nom : clic sur en-tête bascule Asc/Desc et URL', async ({ page }) => {
		await goToProducts(page);
		const header = page.getByRole('button', { name: /^Nom|^Name|^Nome$/ }).first();
		await header.click();
		await expect(page).toHaveURL(/sortDirection=Desc/);
		await header.click();
		await expect(page).not.toHaveURL(/sortDirection=Desc/);
	});

	test('AC #9 : filtres/tri/pagination restaurés depuis l\'URL après reload', async ({
		page
	}) => {
		await goToProducts(page);
		const name = uniq('TestProduct URLState');
		await createProduct(page, name, '10.00');

		// Applique un filtre recherche + tri Desc → URL doit porter les deux.
		await page.fill('#filter-search', name);
		await page.waitForURL(/search=/, { timeout: 2000 });
		const header = page.getByRole('button', { name: /^Nom|^Name|^Nome$/ }).first();
		await header.click();
		await page.waitForURL(/sortDirection=Desc/, { timeout: 2000 });

		const urlBefore = page.url();

		// Reload : l'état doit être reconstruit depuis les query params.
		await page.reload();
		await expect(page).toHaveURL(urlBefore);
		await expect(page.locator('#filter-search')).toHaveValue(name);
		await expect(page.locator('tbody').getByText(name)).toBeVisible({ timeout: 5000 });

		await archiveRow(page, name);
	});

	test('sélection taux TVA 2.60 % persiste après édition', async ({ page }) => {
		await goToProducts(page);
		const name = uniq('TestProduct VAT');
		await page.getByRole('button', { name: /Nouveau produit/ }).click();
		await page.fill('#form-name', name);
		await page.fill('#form-price', '100.00');
		await page.locator('#form-vat-rate').selectOption('2.60');
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(name)).toBeVisible({ timeout: 5000 });

		const row = page.locator('tr', { hasText: name }).first();
		await row.getByRole('button', { name: /Modifier/ }).click();
		await expect(page.locator('#form-vat-rate')).toHaveValue('2.60');
		await page.getByRole('button', { name: 'Annuler' }).click();

		await archiveRow(page, name);
	});
});

test.describe('Contact — conditions de paiement (Story 4.2 T1)', () => {
	test('le champ conditions de paiement persiste après création+édition', async ({ page }) => {
		await login(page);
		await page.goto('/contacts');
		await expect(page).toHaveURL(/\/contacts/);

		const uniqueName = `TestContact PT ${Date.now()}`;

		await page.getByRole('button', { name: /Nouveau contact/ }).click();
		await page.fill('#form-name', uniqueName);
		await page.fill('#form-payment-terms', '30 jours net');
		await page.getByRole('button', { name: 'Créer' }).click();
		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });

		// Rouvrir en édition → la valeur doit être restaurée.
		const row = page.locator('tr', { hasText: uniqueName }).first();
		await row.getByRole('button', { name: /Modifier/ }).click();
		await expect(page.locator('#form-payment-terms')).toHaveValue('30 jours net');
		await page.getByRole('button', { name: 'Annuler' }).click();

		// Cleanup.
		await row.getByRole('button', { name: /Archiver/ }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
	});
});
