# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: invoices.spec.ts >> Factures — création brouillon >> crée une facture avec une ligne libre et la persiste
- Location: tests/e2e/invoices.spec.ts:87:2

# Error details

```
Error: createContactViaApi failed: 401

expect(received).toBeTruthy()

Received: false
```

# Page snapshot

```yaml
- generic [ref=e2]:
  - generic [ref=e3]:
    - banner [ref=e4]:
      - link "Kesh" [ref=e5] [cursor=pointer]:
        - /url: /
        - generic [ref=e6]: Kesh
      - generic [ref=e7]:
        - img [ref=e8]
        - searchbox "Rechercher..." [ref=e9]
      - button "Admin" [ref=e11]:
        - button "Admin" [ref=e12]:
          - img
          - generic [ref=e13]: Admin
          - img
    - generic [ref=e14]:
      - navigation "Navigation principale" [ref=e15]:
        - generic [ref=e16]: Quotidien
        - list [ref=e17]:
          - listitem [ref=e18]:
            - link "Accueil" [ref=e19] [cursor=pointer]:
              - /url: /
          - listitem [ref=e20]:
            - link "Carnet d'adresses" [ref=e21] [cursor=pointer]:
              - /url: /contacts
          - listitem [ref=e22]:
            - link "Catalogue" [ref=e23] [cursor=pointer]:
              - /url: /products
          - listitem [ref=e24]:
            - link "Facturer" [ref=e25] [cursor=pointer]:
              - /url: /invoices
          - listitem [ref=e26]:
            - link "Échéancier" [ref=e27] [cursor=pointer]:
              - /url: /invoices/due-dates
          - listitem [ref=e28]:
            - link "Payer" [ref=e29] [cursor=pointer]:
              - /url: /bank-accounts
          - listitem [ref=e30]:
            - link "Importer" [ref=e31] [cursor=pointer]:
              - /url: /bank-import
        - separator [ref=e32]
        - generic [ref=e33]: Mensuel
        - list [ref=e34]:
          - listitem [ref=e35]:
            - link "Écritures" [ref=e36] [cursor=pointer]:
              - /url: /journal-entries
          - listitem [ref=e37]:
            - link "Réconciliation" [ref=e38] [cursor=pointer]:
              - /url: /reconciliation
          - listitem [ref=e39]:
            - link "Rapports" [ref=e40] [cursor=pointer]:
              - /url: /reports
        - separator [ref=e41]
        - list [ref=e42]:
          - listitem [ref=e43]:
            - link "Paramètres" [ref=e44] [cursor=pointer]:
              - /url: /settings
        - separator [ref=e45]
        - generic [ref=e46]: Administration
        - list [ref=e47]:
          - listitem [ref=e48]:
            - link "Utilisateurs" [ref=e49] [cursor=pointer]:
              - /url: /users
          - listitem [ref=e50]:
            - link "Facturation" [ref=e51] [cursor=pointer]:
              - /url: /settings/invoicing
      - main [ref=e52]:
        - heading "Tableau de bord" [level=1] [ref=e53]
        - generic [ref=e54]:
          - generic [ref=e55]:
            - heading "Dernières écritures" [level=3] [ref=e56]
            - paragraph [ref=e57]: Aucune écriture pour le moment. Commencez par saisir votre première écriture comptable.
            - link "Saisir une écriture" [ref=e58] [cursor=pointer]:
              - /url: /journal-entries
          - generic [ref=e59]:
            - heading "Factures ouvertes" [level=3] [ref=e60]
            - paragraph [ref=e61]: Aucune facture ouverte. Créez votre première facture pour facturer vos clients.
            - link "Créer une facture" [ref=e62] [cursor=pointer]:
              - /url: /invoices
          - generic [ref=e63]:
            - heading "Comptes bancaires" [level=3] [ref=e64]
            - paragraph [ref=e65]: Aucun compte bancaire configuré. Ajoutez votre compte pour importer vos relevés.
            - link "Configurer" [ref=e66] [cursor=pointer]:
              - /url: /settings
    - contentinfo [ref=e67]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T"
  - generic [ref=e68]: Accueil - Kesh
```

# Test source

```ts
  1   | import { expect, test } from '@playwright/test';
  2   | import AxeBuilder from '@axe-core/playwright';
  3   | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  4   | 
  5   | test.beforeAll(async () => {
  6   | 	await seedTestState('with-company');
  7   | });
  8   | 
  9   | test.afterEach(async ({ page }) => {
  10  | 	// Clear localStorage after each test to prevent token bleed to next test
  11  | 	await clearAuthStorage(page);
  12  | });
  13  | 
  14  | /**
  15  |  * Tests E2E — Factures brouillon (Story 5.1)
  16  |  *
  17  |  * Prérequis seed DB :
  18  |  * - admin bootstrap (admin / admin123)
  19  |  * - une `companies` active
  20  |  *
  21  |  * Les tests créent leurs propres contacts et factures avec suffixes uniques.
  22  |  */
  23  | 
  24  | async function login(page: import('@playwright/test').Page) {
  25  | 	await page.goto('/login');
  26  | 	await page.fill('#username', 'admin');
  27  | 	await page.fill('#password', 'admin123');
  28  | 	await page.click('button[type="submit"]');
  29  | 	await expect(page).toHaveURL('/');
  30  | }
  31  | 
  32  | function uniq(prefix: string): string {
  33  | 	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
  34  | }
  35  | 
  36  | async function createContactViaApi(page: import('@playwright/test').Page, name: string): Promise<number> {
  37  | 	const res = await page.request.post('/api/v1/contacts', {
  38  | 		data: {
  39  | 			contactType: 'Entreprise',
  40  | 			name,
  41  | 			isClient: true,
  42  | 			isSupplier: false,
  43  | 			defaultPaymentTerms: '30 jours net',
  44  | 		},
  45  | 	});
> 46  | 	expect(res.ok(), `createContactViaApi failed: ${res.status()}`).toBeTruthy();
      |                                                                  ^ Error: createContactViaApi failed: 401
  47  | 	const json = await res.json();
  48  | 	return json.id as number;
  49  | }
  50  | 
  51  | async function createProductViaApi(
  52  | 	page: import('@playwright/test').Page,
  53  | 	name: string,
  54  | 	unitPrice: string,
  55  | 	vatRate: string,
  56  | ): Promise<number> {
  57  | 	const res = await page.request.post('/api/v1/products', {
  58  | 		data: { name, unitPrice, vatRate },
  59  | 	});
  60  | 	expect(res.ok(), `createProductViaApi failed: ${res.status()}`).toBeTruthy();
  61  | 	const json = await res.json();
  62  | 	return json.id as number;
  63  | }
  64  | 
  65  | test.describe('Factures — liste', () => {
  66  | 	test('affiche le titre et le bouton Nouvelle facture', async ({ page }) => {
  67  | 		await login(page);
  68  | 		await page.goto('/invoices');
  69  | 		await expect(page.getByRole('heading', { name: 'Factures' })).toBeVisible();
  70  | 		await expect(page.getByRole('button', { name: /Nouvelle facture/ })).toBeVisible();
  71  | 	});
  72  | 
  73  | 	// Note (D-6-1-D) : avec le seed E2E actuel (bootstrap admin seul), la liste
  74  | 	// /invoices est vide → ce test axe valide uniquement l'empty state. Une fois
  75  | 	// Story 6-4 (`seed_accounting_company`) en place, étendre pour couvrir l'état
  76  | 	// peuplé (badges statut, contraste lignes de tableau, etc.).
  77  | 	test('axe-core sans violations sur la liste factures (empty state)', async ({ page }) => {
  78  | 		await login(page);
  79  | 		await page.goto('/invoices');
  80  | 		await page.waitForLoadState('networkidle');
  81  | 		const results = await new AxeBuilder({ page }).analyze();
  82  | 		expect(results.violations).toEqual([]);
  83  | 	});
  84  | });
  85  | 
  86  | test.describe('Factures — création brouillon', () => {
  87  | 	test('crée une facture avec une ligne libre et la persiste', async ({ page }) => {
  88  | 		await login(page);
  89  | 		const contactName = uniq('Client');
  90  | 		await createContactViaApi(page, contactName);
  91  | 
  92  | 		await page.goto('/invoices/new');
  93  | 		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();
  94  | 
  95  | 		// Sélection du contact via le combobox
  96  | 		await page.getByRole('combobox').click();
  97  | 		await page.getByRole('combobox').fill(contactName);
  98  | 		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
  99  | 
  100 | 		// Ligne libre par défaut : remplir description, quantité, prix
  101 | 		const firstRow = page.locator('tbody tr').first();
  102 | 		await firstRow.locator('input[type="text"]').first().fill('Conseil stratégique');
  103 | 		const numericInputs = firstRow.locator('input[inputmode="decimal"]');
  104 | 		await numericInputs.nth(0).fill('4.5');
  105 | 		await numericInputs.nth(1).fill('200.00');
  106 | 
  107 | 		await page.getByRole('button', { name: 'Créer la facture' }).click();
  108 | 		await expect(page).toHaveURL('/invoices');
  109 | 		await expect(page.locator('tbody').getByText(contactName)).toBeVisible({ timeout: 5000 });
  110 | 	});
  111 | 
  112 | 	test('crée une facture avec 1 ligne libre + 1 ligne catalogue et persiste après reload (AC #1, #2)', async ({
  113 | 		page,
  114 | 	}) => {
  115 | 		await login(page);
  116 | 		const contactName = uniq('ClientCombo');
  117 | 		const productName = uniq('Prod');
  118 | 		await createContactViaApi(page, contactName);
  119 | 		await createProductViaApi(page, productName, '150.00', '8.10');
  120 | 
  121 | 		await page.goto('/invoices/new');
  122 | 
  123 | 		// Contact
  124 | 		await page.getByRole('combobox').click();
  125 | 		await page.getByRole('combobox').fill(contactName);
  126 | 		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
  127 | 
  128 | 		// Ligne libre (celle par défaut)
  129 | 		const firstRow = page.locator('tbody tr').first();
  130 | 		await firstRow.locator('input[type="text"]').first().fill('Prestation libre');
  131 | 		const firstRowNumerics = firstRow.locator('input[inputmode="decimal"]');
  132 | 		await firstRowNumerics.nth(0).fill('2');
  133 | 		await firstRowNumerics.nth(1).fill('100.00');
  134 | 
  135 | 		// Ligne depuis catalogue
  136 | 		await page.getByRole('button', { name: /Depuis catalogue/ }).click();
  137 | 		await expect(page.getByRole('dialog')).toBeVisible();
  138 | 		await page.getByPlaceholder(/Rechercher un produit/).fill(productName);
  139 | 		await page
  140 | 			.getByRole('dialog')
  141 | 			.getByRole('button')
  142 | 			.filter({ hasText: productName })
  143 | 			.first()
  144 | 			.click();
  145 | 
  146 | 		// Le formulaire doit maintenant contenir 2 lignes, la 2e pré-remplie
```