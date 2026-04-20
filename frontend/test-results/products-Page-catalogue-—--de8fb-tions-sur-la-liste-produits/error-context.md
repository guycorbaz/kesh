# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: products.spec.ts >> Page catalogue — accessibilité >> axe-core sans violations sur la liste produits
- Location: tests/e2e/products.spec.ts:78:2

# Error details

```
Error: expect(received).toEqual(expected) // deep equality

- Expected  -   1
+ Received  + 146

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
+                   ".bg-primary",
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
+           ".bg-primary",
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
+     "description": "Ensure each HTML document contains a non-empty <title> element",
+     "help": "Documents must have <title> element to aid in navigation",
+     "helpUrl": "https://dequeuniversity.com/rules/axe/4.11/document-title?application=playwright",
+     "id": "document-title",
+     "impact": "serious",
+     "nodes": Array [
+       Object {
+         "all": Array [],
+         "any": Array [
+           Object {
+             "data": null,
+             "id": "doc-has-title",
+             "impact": "serious",
+             "message": "Document does not have a non-empty <title> element",
+             "relatedNodes": Array [],
+           },
+         ],
+         "failureSummary": "Fix any of the following:
+   Document does not have a non-empty <title> element",
+         "html": "<html lang=\"fr\" data-mode=\"guided\">",
+         "impact": "serious",
+         "none": Array [],
+         "target": Array [
+           "html",
+         ],
+       },
+     ],
+     "tags": Array [
+       "cat.text-alternatives",
+       "wcag2a",
+       "wcag242",
+       "TTv5",
+       "TT12.a",
+       "EN-301-549",
+       "EN-9.2.4.2",
+       "ACT",
+       "RGAAv4",
+       "RGAA-8.5.1",
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
+                   ".dark\\:hover\\:bg-muted\\/50",
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
          - generic [ref=e54]:
            - heading "Catalogue produits/services" [level=1] [ref=e55]
            - button "Nouveau produit" [ref=e56]:
              - img
              - text: Nouveau produit
          - generic [ref=e57]:
            - generic [ref=e58]:
              - generic [ref=e59]: Rechercher par nom ou description…
              - generic [ref=e60]:
                - img [ref=e61]
                - searchbox "Rechercher par nom ou description…" [ref=e62]
            - generic [ref=e63]:
              - checkbox "Inclure archivés" [ref=e64]
              - text: Inclure archivés
            - button "Réinitialiser" [ref=e65]
          - table [ref=e67]:
            - rowgroup [ref=e68]:
              - row "Nom ↑ Description Prix TVA Actions" [ref=e69]:
                - columnheader "Nom ↑" [ref=e70]:
                  - button "Nom ↑" [ref=e71]
                - columnheader "Description" [ref=e72]
                - columnheader "Prix" [ref=e73]:
                  - button "Prix" [ref=e74]
                - columnheader "TVA" [ref=e75]:
                  - button "TVA" [ref=e76]
                - columnheader "Actions" [ref=e77]
            - rowgroup [ref=e78]:
              - row "Aucun produit. Créez votre premier produit avec le bouton « Nouveau produit »." [ref=e79]:
                - cell "Aucun produit. Créez votre premier produit avec le bouton « Nouveau produit »." [ref=e80]
          - generic [ref=e81]:
            - generic [ref=e82]: 0-0 sur 0
            - generic [ref=e83]:
              - button "Précédent" [disabled]
              - button "Suivant" [disabled]
    - contentinfo [ref=e84]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T"
  - generic [ref=e85]: untitled page
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
  15  |  * Tests E2E — Catalogue produits (Story 4.2)
  16  |  *
  17  |  * Prérequis seed DB identiques à contacts.spec.ts :
  18  |  * - admin bootstrap (admin / admin123)
  19  |  * - une `companies` active
  20  |  */
  21  | 
  22  | async function login(page: import('@playwright/test').Page) {
  23  | 	await page.goto('/login');
  24  | 	await page.fill('#username', 'admin');
  25  | 	await page.fill('#password', 'admin123');
  26  | 	await page.click('button[type="submit"]');
  27  | 	await expect(page).toHaveURL('/');
  28  | }
  29  | 
  30  | async function goToProducts(page: import('@playwright/test').Page) {
  31  | 	await login(page);
  32  | 	await page.goto('/products');
  33  | 	await expect(page).toHaveURL(/\/products/);
  34  | }
  35  | 
  36  | function uniq(prefix: string): string {
  37  | 	// Suffixe unique tolérant aux exécutions parallèles (ms + pid + random).
  38  | 	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
  39  | }
  40  | 
  41  | async function createProduct(
  42  | 	page: import('@playwright/test').Page,
  43  | 	name: string,
  44  | 	price: string,
  45  | 	vatValue?: '0.00' | '2.60' | '3.80' | '8.10'
  46  | ) {
  47  | 	await page.getByRole('button', { name: /Nouveau produit/ }).click();
  48  | 	await page.fill('#form-name', name);
  49  | 	await page.fill('#form-price', price);
  50  | 	if (vatValue) {
  51  | 		// `selectOption` avec `{ value }` est déterministe ; `{ label }` ne matche
  52  | 		// pas les regex et dépend de la locale courante — les valeurs TVA sont
  53  | 		// stables côté backend et constituent un point d'ancrage sûr.
  54  | 		await page.locator('#form-vat-rate').selectOption(vatValue);
  55  | 	}
  56  | 	await page.getByRole('button', { name: 'Créer' }).click();
  57  | 	await expect(page.locator('tbody').getByText(name)).toBeVisible({ timeout: 5000 });
  58  | }
  59  | 
  60  | async function archiveRow(page: import('@playwright/test').Page, name: string) {
  61  | 	const row = page.locator('tr', { hasText: name }).first();
  62  | 	await row.getByRole('button', { name: /Archiver/ }).click();
  63  | 	await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  64  | 	// Scopé à `tbody` pour éviter les faux positifs avec le toast de succès
  65  | 	// qui peut répéter brièvement le nom du produit.
  66  | 	await expect(page.locator('tbody').getByText(name)).toHaveCount(0, { timeout: 5000 });
  67  | }
  68  | 
  69  | test.describe('Page catalogue — affichage', () => {
  70  | 	test('affiche le titre et le bouton Nouveau produit', async ({ page }) => {
  71  | 		await goToProducts(page);
  72  | 		await expect(page.getByRole('heading', { name: /Catalogue/ })).toBeVisible();
  73  | 		await expect(page.getByRole('button', { name: /Nouveau produit/ })).toBeVisible();
  74  | 	});
  75  | });
  76  | 
  77  | test.describe('Page catalogue — accessibilité', () => {
  78  | 	test('axe-core sans violations sur la liste produits', async ({ page }) => {
  79  | 		await goToProducts(page);
  80  | 		await page.waitForLoadState('networkidle');
  81  | 		const results = await new AxeBuilder({ page }).analyze();
> 82  | 		expect(results.violations).toEqual([]);
      |                              ^ Error: expect(received).toEqual(expected) // deep equality
  83  | 	});
  84  | });
  85  | 
  86  | test.describe('Page catalogue — CRUD', () => {
  87  | 	test('création nominale d\'un produit', async ({ page }) => {
  88  | 		await goToProducts(page);
  89  | 
  90  | 		const uniqueName = `TestProduct E2E ${Date.now()}`;
  91  | 
  92  | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  93  | 		await expect(page.getByRole('heading', { name: /Nouveau produit/ })).toBeVisible();
  94  | 
  95  | 		await page.fill('#form-name', uniqueName);
  96  | 		await page.fill('#form-price', '1500.00');
  97  | 		// Taux TVA 8.10 % est sélectionné par défaut.
  98  | 		await page.getByRole('button', { name: 'Créer' }).click();
  99  | 
  100 | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  101 | 
  102 | 		// Cleanup : archiver pour ne pas polluer les tests suivants.
  103 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  104 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  105 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  106 | 		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
  107 | 	});
  108 | 
  109 | 	test('archivage avec confirmation et disparition de la liste', async ({ page }) => {
  110 | 		await goToProducts(page);
  111 | 
  112 | 		const uniqueName = `TestProduct Arch ${Date.now()}`;
  113 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  114 | 		await page.fill('#form-name', uniqueName);
  115 | 		await page.fill('#form-price', '42.00');
  116 | 		await page.getByRole('button', { name: 'Créer' }).click();
  117 | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  118 | 
  119 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  120 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  121 | 
  122 | 		await expect(page.getByRole('dialog').getByText(/Archiver le produit/)).toBeVisible();
  123 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  124 | 
  125 | 		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
  126 | 	});
  127 | 
  128 | 	test('filtre recherche reflété dans URL et résultats', async ({ page }) => {
  129 | 		await goToProducts(page);
  130 | 
  131 | 		const uniqueName = `TestProduct Search ${Date.now()}`;
  132 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  133 | 		await page.fill('#form-name', uniqueName);
  134 | 		await page.fill('#form-price', '10.00');
  135 | 		await page.getByRole('button', { name: 'Créer' }).click();
  136 | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  137 | 
  138 | 		// Rechercher par nom unique → URL contient `search=` (attente event-driven
  139 | 		// plutôt que `waitForTimeout`, pour éviter les flakes CI sur machines lentes).
  140 | 		await page.fill('#filter-search', uniqueName);
  141 | 		await page.waitForURL(/search=/, { timeout: 2000 });
  142 | 		await expect(page.locator('tbody').getByText(uniqueName)).toBeVisible();
  143 | 
  144 | 		// Cleanup.
  145 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  146 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  147 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  148 | 	});
  149 | });
  150 | 
  151 | test.describe('Page catalogue — validation & erreurs', () => {
  152 | 	test('format prix invalide affiche un message inline et désactive Créer', async ({ page }) => {
  153 | 		await goToProducts(page);
  154 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  155 | 		await page.fill('#form-name', uniq('TestProduct Invalid'));
  156 | 		await page.fill('#form-price', '10.123456'); // > 4 décimales
  157 | 		// Feedback inline visible sans avoir cliqué sur Créer.
  158 | 		await expect(page.getByText(/prix invalide/i)).toBeVisible();
  159 | 		await expect(page.getByRole('button', { name: 'Créer' })).toBeDisabled();
  160 | 		// Correction → activation.
  161 | 		await page.fill('#form-price', '10.50');
  162 | 		await expect(page.getByRole('button', { name: 'Créer' })).toBeEnabled();
  163 | 		await page.getByRole('button', { name: 'Annuler' }).click();
  164 | 	});
  165 | 
  166 | 	test('création d\'un nom en doublon remonte une erreur', async ({ page }) => {
  167 | 		await goToProducts(page);
  168 | 		const name = uniq('TestProduct Dup');
  169 | 		await createProduct(page, name, '5.00');
  170 | 
  171 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  172 | 		await page.fill('#form-name', name);
  173 | 		await page.fill('#form-price', '7.00');
  174 | 		await page.getByRole('button', { name: 'Créer' }).click();
  175 | 		await expect(page.getByText(/existe déjà|already exists/i)).toBeVisible({ timeout: 5000 });
  176 | 		await page.getByRole('button', { name: 'Annuler' }).click();
  177 | 
  178 | 		await archiveRow(page, name);
  179 | 	});
  180 | });
  181 | 
  182 | test.describe('Page catalogue — filtres, tri & pagination', () => {
```