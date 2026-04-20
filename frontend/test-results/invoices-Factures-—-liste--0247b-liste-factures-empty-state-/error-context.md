# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: invoices.spec.ts >> Factures — liste >> axe-core sans violations sur la liste factures (empty state)
- Location: tests/e2e/invoices.spec.ts:77:2

# Error details

```
Error: expect(received).toEqual(expected) // deep equality

- Expected  -   1
+ Received  + 105

- Array []
+ Array [
+   Object {
+     "description": "Ensure the contrast between foreground and background colors meets WCAG 2 AA minimum contrast ratio thresholds",
+     "help": "Elements must meet minimum color contrast ratio thresholds",
+     "helpUrl": "https://dequeuniversity.com/rules/axe/4.11/color-contrast?application=playwright",
+     "id": "color-contrast",
+     "impact": "serious",
+     "nodes": Array [
+       Object {
+         "all": Array [],
+         "any": Array [
+           Object {
+             "data": Object {
+               "bgColor": "#1e40af",
+               "contrastRatio": 1.67,
+               "expectedContrastRatio": "4.5:1",
+               "fgColor": "#1e293b",
+               "fontSize": "10.5pt (14px)",
+               "fontWeight": "normal",
+               "messageKey": null,
+             },
+             "id": "color-contrast",
+             "impact": "serious",
+             "message": "Element has insufficient color contrast of 1.67 (foreground color: #1e293b, background color: #1e40af, font size: 10.5pt (14px), font weight: normal). Expected contrast ratio of 4.5:1",
+             "relatedNodes": Array [
+               Object {
+                 "html": "<button data-slot=\"button\" class=\"focus-visible:border...\" type=\"button\">",
+                 "target": Array [
+                   ".inline-flex",
+                 ],
+               },
+             ],
+           },
+         ],
+         "failureSummary": "Fix any of the following:
+   Element has insufficient color contrast of 1.67 (foreground color: #1e293b, background color: #1e40af, font size: 10.5pt (14px), font weight: normal). Expected contrast ratio of 4.5:1",
+         "html": "<button data-slot=\"button\" class=\"focus-visible:border...\" type=\"button\">",
+         "impact": "serious",
+         "none": Array [],
+         "target": Array [
+           ".inline-flex",
+         ],
+       },
+     ],
+     "tags": Array [
+       "cat.color",
+       "wcag2aa",
+       "wcag143",
+       "TTv5",
+       "TT13.c",
+       "EN-301-549",
+       "EN-9.1.4.3",
+       "ACT",
+       "RGAAv4",
+       "RGAA-3.2.1",
+     ],
+   },
+   Object {
+     "description": "Ensure interactive controls are not nested as they are not always announced by screen readers or can cause focus problems for assistive technologies",
+     "help": "Interactive controls must not be nested",
+     "helpUrl": "https://dequeuniversity.com/rules/axe/4.11/nested-interactive?application=playwright",
+     "id": "nested-interactive",
+     "impact": "serious",
+     "nodes": Array [
+       Object {
+         "all": Array [],
+         "any": Array [
+           Object {
+             "data": null,
+             "id": "no-focusable-content",
+             "impact": "serious",
+             "message": "Element has focusable descendants",
+             "relatedNodes": Array [
+               Object {
+                 "html": "<button data-slot=\"button\" class=\"focus-visible:border...\" type=\"button\">",
+                 "target": Array [
+                   ".hover\\:bg-muted",
+                 ],
+               },
+             ],
+           },
+         ],
+         "failureSummary": "Fix any of the following:
+   Element has focusable descendants",
+         "html": "<button data-slot=\"dropdown-menu-trigger\" id=\"bits-c1\" aria-haspopup=\"menu\" aria-expanded=\"false\" data-state=\"closed\" data-dropdown-menu-trigger=\"\" type=\"button\">",
+         "impact": "serious",
+         "none": Array [],
+         "target": Array [
+           "#bits-c1",
+         ],
+       },
+     ],
+     "tags": Array [
+       "cat.keyboard",
+       "wcag2a",
+       "wcag412",
+       "TTv5",
+       "TT6.a",
+       "EN-301-549",
+       "EN-9.4.1.2",
+       "RGAAv4",
+       "RGAA-7.1.1",
+     ],
+   },
+ ]
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
        - generic [ref=e53]:
          - heading "Factures" [level=1] [ref=e54]
          - button "Nouvelle facture" [ref=e55]:
            - img
            - text: Nouvelle facture
        - generic [ref=e56]:
          - generic [ref=e57]:
            - generic [ref=e58]: Recherche
            - searchbox "Recherche" [ref=e59]
          - generic [ref=e60]:
            - generic [ref=e61]: Statut
            - combobox "Statut" [ref=e62]:
              - option "Tous les statuts" [selected]
              - option "Brouillon"
              - option "Validée"
              - option "Annulée"
          - generic [ref=e63]:
            - generic [ref=e64]: Contact
            - combobox "Tous les contacts" [ref=e66]
          - generic [ref=e67]:
            - generic [ref=e68]: Depuis
            - textbox "Depuis" [ref=e69]
          - generic [ref=e70]:
            - generic [ref=e71]: Jusqu'à
            - textbox "Jusqu'à" [ref=e72]
        - paragraph [ref=e73]: Aucune facture.
    - contentinfo [ref=e74]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T"
  - generic [ref=e75]: Factures — Kesh
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
  46  | 	expect(res.ok(), `createContactViaApi failed: ${res.status()}`).toBeTruthy();
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
> 82  | 		expect(results.violations).toEqual([]);
      |                              ^ Error: expect(received).toEqual(expected) // deep equality
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
  147 | 		await expect(page.locator('tbody tr')).toHaveCount(2);
  148 | 		const secondRow = page.locator('tbody tr').nth(1);
  149 | 		await expect(secondRow.locator('input[type="text"]').first()).toHaveValue(productName);
  150 | 		const secondRowNumerics = secondRow.locator('input[inputmode="decimal"]');
  151 | 		await expect(secondRowNumerics.nth(1)).toHaveValue('150.00');
  152 | 
  153 | 		// Soumettre
  154 | 		await page.getByRole('button', { name: 'Créer la facture' }).click();
  155 | 		await expect(page).toHaveURL('/invoices');
  156 | 
  157 | 		// Ouvrir la facture créée pour vérifier la persistance après reload
  158 | 		const row = page.locator('tbody tr', { hasText: contactName }).first();
  159 | 		await row.getByRole('button').first().click();
  160 | 		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();
  161 | 
  162 | 		// Reload dur — l'état doit être identique
  163 | 		await page.reload();
  164 | 		await expect(page.getByText('Prestation libre')).toBeVisible();
  165 | 		await expect(page.getByText(productName)).toBeVisible();
  166 | 	});
  167 | });
  168 | 
  169 | // ---------------------------------------------------------------------------
  170 | // Story 5.3 — Téléchargement PDF QR Bill
  171 | // ---------------------------------------------------------------------------
  172 | 
  173 | async function createContactWithAddressViaApi(
  174 | 	page: import('@playwright/test').Page,
  175 | 	name: string,
  176 | ): Promise<number> {
  177 | 	const res = await page.request.post('/api/v1/contacts', {
  178 | 		data: {
  179 | 			contactType: 'Personne',
  180 | 			name,
  181 | 			isClient: true,
  182 | 			isSupplier: false,
```