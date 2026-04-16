import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

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

	// Note (D-6-1-D) : avec le seed E2E actuel (bootstrap admin seul), la liste
	// /invoices est vide → ce test axe valide uniquement l'empty state. Une fois
	// Story 6-4 (`seed_accounting_company`) en place, étendre pour couvrir l'état
	// peuplé (badges statut, contraste lignes de tableau, etc.).
	test('axe-core sans violations sur la liste factures (empty state)', async ({ page }) => {
		await login(page);
		await page.goto('/invoices');
		await page.waitForLoadState('networkidle');
		const results = await new AxeBuilder({ page }).analyze();
		expect(results.violations).toEqual([]);
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

// ---------------------------------------------------------------------------
// Story 5.3 — Téléchargement PDF QR Bill
// ---------------------------------------------------------------------------

async function createContactWithAddressViaApi(
	page: import('@playwright/test').Page,
	name: string,
): Promise<number> {
	const res = await page.request.post('/api/v1/contacts', {
		data: {
			contactType: 'Personne',
			name,
			isClient: true,
			isSupplier: false,
			address: 'Marktgasse 28\n9400 Rorschach',
			defaultPaymentTerms: '30 jours net',
		},
	});
	expect(res.ok(), `createContactWithAddress failed: ${res.status()}`).toBeTruthy();
	return (await res.json()).id as number;
}

async function createAndValidateInvoiceViaApi(
	page: import('@playwright/test').Page,
	contactId: number,
): Promise<number> {
	const today = new Date().toISOString().slice(0, 10);
	const createRes = await page.request.post('/api/v1/invoices', {
		data: {
			contactId,
			date: today,
			dueDate: today,
			paymentTerms: '30 jours net',
			lines: [
				{
					description: 'Conseil stratégique',
					quantity: '4.5',
					unitPrice: '200.00',
					vatRate: '7.70',
				},
			],
		},
	});
	expect(createRes.ok(), `create invoice failed: ${createRes.status()}`).toBeTruthy();
	const invoice = await createRes.json();
	const validateRes = await page.request.post(`/api/v1/invoices/${invoice.id}/validate`);
	expect(validateRes.ok(), `validate failed: ${validateRes.status()}`).toBeTruthy();
	return invoice.id as number;
}

test.describe('Factures — téléchargement PDF (Story 5.3)', () => {
	test('télécharge le PDF d\'une facture validée (golden path)', async ({ page, context }) => {
		await login(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Client'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();

		// Intercepte l'appel direct à l'endpoint PDF (plus robuste que window.open).
		const pdfRes = await page.request.get(`/api/v1/invoices/${invoiceId}/pdf`);
		expect(pdfRes.status()).toBe(200);
		expect(pdfRes.headers()['content-type']).toContain('application/pdf');
		const buf = await pdfRes.body();
		expect(buf.slice(0, 7).toString('utf8')).toMatch(/^%PDF-1\./);
	});

	test('bouton visible uniquement si status=validated', async ({ page }) => {
		await login(page);
		const contactName = uniq('PDF Draft');
		await createContactWithAddressViaApi(page, contactName);
		// Facture brouillon non validée
		await page.goto('/invoices/new');
		await page.getByRole('combobox').click();
		await page.getByRole('combobox').fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Item');
		const inputs = firstRow.locator('input[inputmode="decimal"]');
		await inputs.nth(0).fill('1');
		await inputs.nth(1).fill('50');
		await page.getByRole('button', { name: 'Créer la facture' }).click();
		await expect(page).toHaveURL('/invoices');

		// Ouvre la facture brouillon → pas de bouton PDF
		const row = page.locator('tbody tr', { hasText: contactName }).first();
		await row.getByRole('button').first().click();
		await expect(page.getByRole('button', { name: /Télécharger PDF/i })).toHaveCount(0);
	});

	test('erreur 400 INVOICE_NOT_PDF_READY affichée comme toast', async ({ page }) => {
		// AC17 : le cas d'erreur INVOICE_NOT_PDF_READY doit s'afficher sous
		// forme de toast côté UI. Le backend E2E (`invoice_pdf_e2e.rs`) couvre
		// déjà la détection backend ; ici on vérifie que le frontend affiche
		// correctement l'erreur en interceptant la réponse du serveur.
		await login(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Err'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		// Intercepte l'appel PDF pour renvoyer un 400 INVOICE_NOT_PDF_READY.
		await page.route(`**/api/v1/invoices/${invoiceId}/pdf`, async (route) => {
			await route.fulfill({
				status: 400,
				contentType: 'application/json',
				body: JSON.stringify({
					error: {
						code: 'INVOICE_NOT_PDF_READY',
						message: "Aucun compte bancaire principal n'est configuré pour cette company.",
					},
				}),
			});
		});

		await page.goto(`/invoices/${invoiceId}`);
		await page.getByRole('button', { name: /Télécharger PDF/i }).click();

		// Toast d'erreur affichant le message INVOICE_NOT_PDF_READY.
		await expect(
			page.getByText(/compte bancaire principal|primary bank|INVOICE_NOT_PDF_READY/i),
		).toBeVisible({ timeout: 5000 });
	});
});
