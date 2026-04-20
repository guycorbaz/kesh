# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: homepage-settings.spec.ts >> Homepage — accessibilité >> axe-core sans violations sur la page d'accueil
- Location: tests/e2e/homepage-settings.spec.ts:60:2

# Error details

```
Error: expect(received).toEqual(expected) // deep equality

- Expected  -  1
+ Received  + 82

- Array []
+ Array [
+   Object {
+     "description": "Ensure the order of headings is semantically correct",
+     "help": "Heading levels should only increase by one",
+     "helpUrl": "https://dequeuniversity.com/rules/axe/4.11/heading-order?application=playwright",
+     "id": "heading-order",
+     "impact": "moderate",
+     "nodes": Array [
+       Object {
+         "all": Array [],
+         "any": Array [
+           Object {
+             "data": null,
+             "id": "heading-order",
+             "impact": "moderate",
+             "message": "Heading order invalid",
+             "relatedNodes": Array [],
+           },
+         ],
+         "failureSummary": "Fix any of the following:
+   Heading order invalid",
+         "html": "<h3 class=\"text-lg font-semibold text-text\">Dernières écritures</h3>",
+         "impact": "moderate",
+         "none": Array [],
+         "target": Array [
+           ".bg-white.p-6.shadow-sm:nth-child(1) > h3",
+         ],
+       },
+     ],
+     "tags": Array [
+       "cat.semantics",
+       "best-practice",
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
+                   ".border-transparent",
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
  1  | import { test, expect } from '@playwright/test';
  2  | import AxeBuilder from '@axe-core/playwright';
  3  | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  4  | 
  5  | /**
  6  |  * Tests E2E — Page d'accueil & Paramètres (Story 2.4)
  7  |  * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`.
  8  |  */
  9  | 
  10 | test.beforeAll(async () => {
  11 | 	await seedTestState('with-company');
  12 | });
  13 | 
  14 | test.afterEach(async ({ page }) => {
  15 | 	// Clear localStorage after each test to prevent token bleed to next test
  16 | 	await clearAuthStorage(page);
  17 | });
  18 | 
  19 | test.describe('Homepage', () => {
  20 | 	test.beforeEach(async ({ page }) => {
  21 | 		await page.goto('/login');
  22 | 		await page.fill('#username', 'changeme');
  23 | 		await page.fill('#password', 'changeme');
  24 | 		await page.click('button[type="submit"]');
  25 | 	});
  26 | 
  27 | 	test('affiche 3 widgets sur la page d\'accueil', async ({ page }) => {
  28 | 		await expect(page).toHaveURL('/');
  29 | 		await expect(page.getByText('Dernières écritures')).toBeVisible();
  30 | 		await expect(page.getByText('Factures ouvertes')).toBeVisible();
  31 | 		await expect(page.getByText('Comptes bancaires')).toBeVisible();
  32 | 	});
  33 | });
  34 | 
  35 | test.describe('Settings', () => {
  36 | 	test.beforeEach(async ({ page }) => {
  37 | 		await page.goto('/login');
  38 | 		await page.fill('#username', 'changeme');
  39 | 		await page.fill('#password', 'changeme');
  40 | 		await page.click('button[type="submit"]');
  41 | 	});
  42 | 
  43 | 	test('page Paramètres affiche 4 sections', async ({ page }) => {
  44 | 		await page.goto('/settings');
  45 | 		await expect(page.getByText('Organisation')).toBeVisible();
  46 | 		await expect(page.getByText('Comptabilité')).toBeVisible();
  47 | 		await expect(page.getByText('Comptes bancaires')).toBeVisible();
  48 | 		await expect(page.getByText('Utilisateurs')).toBeVisible();
  49 | 	});
  50 | });
  51 | 
  52 | test.describe('Homepage — accessibilité', () => {
  53 | 	test.beforeEach(async ({ page }) => {
  54 | 		await page.goto('/login');
  55 | 		await page.fill('#username', 'changeme');
  56 | 		await page.fill('#password', 'changeme');
  57 | 		await page.click('button[type="submit"]');
  58 | 	});
  59 | 
  60 | 	test('axe-core sans violations sur la page d\'accueil', async ({ page }) => {
  61 | 		await expect(page).toHaveURL('/');
  62 | 		await page.waitForLoadState('networkidle');
  63 | 		const results = await new AxeBuilder({ page }).analyze();
> 64 | 		expect(results.violations).toEqual([]);
     |                              ^ Error: expect(received).toEqual(expected) // deep equality
  65 | 	});
  66 | });
  67 | 
```