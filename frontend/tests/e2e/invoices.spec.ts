import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';
import {
	createContactWithAddressViaApi,
	ensurePrimaryBankAccountViaApi,
	createAndValidateInvoiceViaApi,
} from './helpers/api-fixtures';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	// Clear localStorage after each test to prevent token bleed to next test
	await clearAuthStorage(page);
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
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
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
	} finally {
		await disposeContextSafe(ctx);
	}
}

async function createProductViaApi(
	page: import('@playwright/test').Page,
	name: string,
	unitPrice: string,
	vatRate: string,
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/products', {
			data: { name, unitPrice, vatRate },
		});
		expect(res.ok(), `createProductViaApi failed: ${res.status()}`).toBeTruthy();
		const json = await res.json();
		return json.id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
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
		// Strict-mode fix (KF #57 cascade-cleared) : `getByRole('combobox')` matche
		// 2 éléments (contact picker + VAT rate select `data-testid=invoice-line-vat-rate`).
		// Discriminer par placeholder du picker contact (cf. ContactPicker component).
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Ligne libre par défaut : remplir description, quantité, prix
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Conseil stratégique');
		const numericInputs = firstRow.locator('input[inputmode="decimal"]');
		await numericInputs.nth(0).fill('4.5');
		await numericInputs.nth(1).fill('200.00');

		await page.getByRole('button', { name: 'Créer la facture' }).click();
		// Depuis Story 5.x (review P5), la création redirige vers la vue DÉTAIL.
		await expect(page).toHaveURL(/\/invoices\/\d+$/);
		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();
		// Persistance : la facture apparaît dans la liste (nom du contact).
		await page.goto('/invoices');
		await expect(page.locator('tbody').getByText(contactName)).toBeVisible({ timeout: 5000 });
	});

	// Story 21-1 (#245) — pré-remplissage échéance + libellé depuis le délai
	// de paiement structuré du contact, à la sélection dans le formulaire.
	test('pré-remplit échéance et conditions depuis le délai du contact (#245)', async ({
		page
	}) => {
		await login(page);
		const contactName = uniq('Client PTD');
		// Contact avec délai structuré 30 jours (helper étendu Story 21-1).
		await createContactWithAddressViaApi(page, contactName, undefined, undefined, 30);

		await page.goto('/invoices/new');
		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();

		// Avant sélection : échéance et conditions vides.
		await expect(page.locator('#invoice-due-date')).toHaveValue('');
		await expect(page.locator('#invoice-payment-terms')).toHaveValue('');

		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		// Échéance = date du jour + 30 (calcul UTC identique à addDaysIso).
		const today = new Date().toISOString().slice(0, 10);
		const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(today)!;
		const expectedDue = new Date(Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3]) + 30))
			.toISOString()
			.slice(0, 10);
		await expect(page.locator('#invoice-due-date')).toHaveValue(expectedDue);
		// Libellé serveur (langue contact héritée = FR sur le seed E2E).
		await expect(page.locator('#invoice-payment-terms')).toHaveValue('Payable à 30 jours net');

		// Une valeur déjà saisie n'est PAS écrasée par une re-sélection.
		await page.locator('#invoice-due-date').fill('2030-01-15');
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
		await expect(page.locator('#invoice-due-date')).toHaveValue('2030-01-15');
	});

	test('crée une facture avec projet analytique (Story 19-4)', async ({ page }) => {
		await login(page);
		const contactName = uniq('Client Projet');
		await createContactViaApi(page, contactName);

		// Projet actif via l'API (code unique par run).
		const code = `E2E194-${Date.now() % 1000000}`;
		const setupCtx = await authedApiContext(page);
		let projectId: number;
		try {
			const resp = await setupCtx.post('/api/v1/projects', {
				data: { parentId: null, code, name: 'Projet E2E 19-4', description: null, startDate: null, endDate: null }
			});
			expect(resp.ok()).toBeTruthy();
			projectId = (await resp.json()).id;
		} finally {
			await disposeContextSafe(setupCtx);
		}

		await page.goto('/invoices/new');
		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();

		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();

		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Prestation projet');
		const numericInputs = firstRow.locator('input[inputmode="decimal"]');
		await numericInputs.nth(0).fill('1');
		await numericInputs.nth(1).fill('500.00');

		// Sélecteur projet document-level (arbre 2 niveaux).
		await page.getByTestId('invoice-project').selectOption({
			label: `${code} — Projet E2E 19-4`
		});

		// Capturer la réponse du POST pour vérifier la persistance ground-truth.
		const [resp] = await Promise.all([
			page.waitForResponse(
				(r) => r.url().includes('/api/v1/invoices') && r.request().method() === 'POST'
			),
			page.getByRole('button', { name: 'Créer la facture' }).click()
		]);
		expect(resp.ok()).toBeTruthy();
		const created: { id: number; projectId: number | null } = await resp.json();
		expect(created.projectId).toBe(projectId);

		// La vue détail affiche le libellé du projet.
		await page.goto(`/invoices/${created.id}`);
		await expect(page.getByTestId('invoice-project')).toHaveText(`${code} — Projet E2E 19-4`);
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
		// Strict-mode fix (KF #57 cascade-cleared) : `getByRole('combobox')` matche
		// 2 éléments (contact picker + VAT rate select `data-testid=invoice-line-vat-rate`).
		// Discriminer par placeholder du picker contact (cf. ContactPicker component).
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
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
		await expect(secondRowNumerics.nth(1)).toHaveValue('150.0000'); // prix catalogue à 4 décimales

		// Soumettre
		await page.getByRole('button', { name: 'Créer la facture' }).click();
		// Depuis Story 5.x (review P5), la création redirige vers la vue DÉTAIL.
		await expect(page).toHaveURL(/\/invoices\/\d+$/);

		// La persistance des lignes se vérifie sur la page d'ÉDITION (le
		// testid invoice-lines-table appartient au formulaire InvoiceForm).
		const detailUrl = page.url();
		await page.goto(`${detailUrl}/edit`);

		// Reload dur — l'état doit être identique. Scope à la table de lignes via testid
		// (un autre <tbody> peut exister sur la page : tableau récap, picker produits, etc.).
		await page.reload();
		const linesTable = page.locator('[data-testid="invoice-lines-table"]');
		await expect(linesTable).toBeVisible();
		// Les descriptions du formulaire vivent dans des <input> — asserter la
		// VALUE (toContainText ne voit pas les valeurs d'inputs).
		const rows = linesTable.locator('tbody tr');
		await expect(rows).toHaveCount(2);
		await expect(rows.nth(0).locator('input[type="text"]').first()).toHaveValue(
			'Prestation libre'
		);
		await expect(rows.nth(1).locator('input[type="text"]').first()).toHaveValue(productName);
	});
});

// ---------------------------------------------------------------------------
// Story 5.3 — Téléchargement PDF QR Bill
// ---------------------------------------------------------------------------

// Story 20-4 : helpers API extraits vers helpers/api-fixtures.ts (DRY —
// partagés avec invoice-send-email.spec.ts).
test.describe('Factures — téléchargement PDF (Story 5.3)', () => {
	test('télécharge le PDF d\'une facture validée (golden path)', async ({ page, context }) => {
		await login(page);
		await ensurePrimaryBankAccountViaApi(page);
		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Client'));
		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);

		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();

		// Intercepte l'appel direct à l'endpoint PDF (plus robuste que window.open).
		// Use authedApiContext (KF #54) — page.request.* n'a pas le Bearer header.
		const pdfCtx = await authedApiContext(page);
		try {
			const pdfRes = await pdfCtx.get(`/api/v1/invoices/${invoiceId}/pdf`);
			expect(pdfRes.status()).toBe(200);
			expect(pdfRes.headers()['content-type']).toContain('application/pdf');
			const buf = await pdfRes.body();
			expect(buf.slice(0, 7).toString('utf8')).toMatch(/^%PDF-1\./);
		} finally {
			await disposeContextSafe(pdfCtx);
		}
	});

	test('bouton visible uniquement si status=validated', async ({ page }) => {
		await login(page);
		const contactName = uniq('PDF Draft');
		await createContactWithAddressViaApi(page, contactName);
		// Facture brouillon non validée
		await page.goto('/invoices/new');
		// Strict-mode fix (KF #57 cascade-cleared) : `getByRole('combobox')` matche
		// 2 éléments (contact picker + VAT rate select `data-testid=invoice-line-vat-rate`).
		// Discriminer par placeholder du picker contact (cf. ContactPicker component).
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).click();
		await page.getByRole('combobox', { name: /Rechercher un contact/ }).fill(contactName);
		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
		const firstRow = page.locator('tbody tr').first();
		await firstRow.locator('input[type="text"]').first().fill('Item');
		const inputs = firstRow.locator('input[inputmode="decimal"]');
		await inputs.nth(0).fill('1');
		await inputs.nth(1).fill('50');
		await page.getByRole('button', { name: 'Créer la facture' }).click();
		await expect(page).toHaveURL(/\/invoices\/\d+$/); // redirection détail (Story 5.x P5)

		// On est déjà sur le détail du brouillon → pas de bouton PDF.
		// NB : l'accessible name du bouton vient de son aria-label
		// (« Télécharger la facture N au format PDF »).
		await expect(page.getByRole('button', { name: /Télécharger.*PDF/i })).toHaveCount(0);
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
		await page.getByRole('button', { name: /Télécharger.*PDF/i }).click();

		// Toast d'erreur affichant le message INVOICE_NOT_PDF_READY.
		await expect(
			// La clé FTL invoice-pdf-error-invoice-not-pdf-ready traduit le code —
			// le message backend intercepté n'est pas affiché tel quel.
			page.getByText(/pas prête pour la génération PDF|compte bancaire principal|INVOICE_NOT_PDF_READY/i),
		).toBeVisible({ timeout: 5000 });
	});
});
