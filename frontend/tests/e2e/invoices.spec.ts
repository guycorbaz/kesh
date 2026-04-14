import { expect, test } from '@playwright/test';

/**
 * Tests E2E — Factures brouillon (Story 5.1)
 *
 * Prérequis seed DB :
 * - admin bootstrap (admin / admin123)
 * - une `companies` active
 *
 * Les tests créent leurs propres contacts et factures avec suffixes uniques.
 */

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

function uniq(prefix: string): string {
	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
}

async function createContactViaApi(page: import('@playwright/test').Page, name: string): Promise<number> {
	const res = await page.request.post('/api/v1/contacts', {
		data: {
			contactType: 'Entreprise',
			name,
			isClient: true,
			isSupplier: false,
			defaultPaymentTerms: '30 jours net',
		},
	});
	expect(res.ok(), `createContactViaApi failed: ${res.status()}`).toBeTruthy();
	const json = await res.json();
	return json.id as number;
}

async function createProductViaApi(
	page: import('@playwright/test').Page,
	name: string,
	unitPrice: string,
	vatRate: string,
): Promise<number> {
	const res = await page.request.post('/api/v1/products', {
		data: { name, unitPrice, vatRate },
	});
	expect(res.ok(), `createProductViaApi failed: ${res.status()}`).toBeTruthy();
	const json = await res.json();
	return json.id as number;
}

test.describe('Factures — liste', () => {
	test('affiche le titre et le bouton Nouvelle facture', async ({ page }) => {
		await login(page);
		await page.goto('/invoices');
		await expect(page.getByRole('heading', { name: 'Factures' })).toBeVisible();
		await expect(page.getByRole('button', { name: /Nouvelle facture/ })).toBeVisible();
	});
});

test.describe('Factures — création brouillon', () => {
	test('crée une facture avec une ligne libre et la persiste', async ({ page }) => {
		await login(page);
		const contactName = uniq('Client');
		await createContactViaApi(page, contactName);

		await page.goto('/invoices/new');
		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();

		// Sélection du contact via le combobox
		await page.getByRole('combobox').click();
		await page.getByRole('combobox').fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Ligne libre par défaut : remplir description, quantité, prix
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Conseil stratégique');
		const numericInputs = firstRow.locator('input[inputmode="decimal"]');
		await numericInputs.nth(0).fill('4.5');
		await numericInputs.nth(1).fill('200.00');

		await page.getByRole('button', { name: 'Créer la facture' }).click();
		await expect(page).toHaveURL('/invoices');
		await expect(page.locator('tbody').getByText(contactName)).toBeVisible({ timeout: 5000 });
	});

	test('crée une facture avec 1 ligne libre + 1 ligne catalogue et persiste après reload (AC #1, #2)', async ({
		page,
	}) => {
		await login(page);
		const contactName = uniq('ClientCombo');
		const productName = uniq('Prod');
		await createContactViaApi(page, contactName);
		await createProductViaApi(page, productName, '150.00', '8.10');

		await page.goto('/invoices/new');

		// Contact
		await page.getByRole('combobox').click();
		await page.getByRole('combobox').fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Ligne libre (celle par défaut)
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Prestation libre');
		const firstRowNumerics = firstRow.locator('input[inputmode="decimal"]');
		await firstRowNumerics.nth(0).fill('2');
		await firstRowNumerics.nth(1).fill('100.00');

		// Ligne depuis catalogue
		await page.getByRole('button', { name: /Depuis catalogue/ }).click();
		await expect(page.getByRole('dialog')).toBeVisible();
		await page.getByPlaceholder(/Rechercher un produit/).fill(productName);
		await page
			.getByRole('dialog')
			.getByRole('button')
			.filter({ hasText: productName })
			.first()
			.click();

		// Le formulaire doit maintenant contenir 2 lignes, la 2e pré-remplie
		await expect(page.locator('tbody tr')).toHaveCount(2);
		const secondRow = page.locator('tbody tr').nth(1);
		await expect(secondRow.locator('input[type="text"]').first()).toHaveValue(productName);
		const secondRowNumerics = secondRow.locator('input[inputmode="decimal"]');
		await expect(secondRowNumerics.nth(1)).toHaveValue('150.00');

		// Soumettre
		await page.getByRole('button', { name: 'Créer la facture' }).click();
		await expect(page).toHaveURL('/invoices');

		// Ouvrir la facture créée pour vérifier la persistance après reload
		const row = page.locator('tbody tr', { hasText: contactName }).first();
		await row.getByRole('button').first().click();
		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();

		// Reload dur — l'état doit être identique
		await page.reload();
		await expect(page.getByText('Prestation libre')).toBeVisible();
		await expect(page.getByText(productName)).toBeVisible();
	});
});
