# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: users.spec.ts >> Page utilisateurs — CRUD >> création d'un utilisateur
- Location: tests/e2e/users.spec.ts:57:2

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByText('test-1776697856062')
Expected: visible
Error: strict mode violation: getByText('test-1776697856062') resolved to 2 elements:
    1) <td data-slot="table-cell" class="p-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0">…</td> aka getByRole('cell', { name: 'test-1776697856062', exact: true })
    2) <div data-title="">…</div> aka getByText('Utilisateur « test-')

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByText('test-1776697856062')

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
          - heading "Utilisateurs" [level=1] [ref=e54]
          - button "Nouvel utilisateur" [active] [ref=e55]:
            - img
            - text: Nouvel utilisateur
        - table [ref=e57]:
          - rowgroup [ref=e58]:
            - row "Nom d'utilisateur Rôle Statut Créé le Actions" [ref=e59]:
              - columnheader "Nom d'utilisateur" [ref=e60]
              - columnheader "Rôle" [ref=e61]
              - columnheader "Statut" [ref=e62]
              - columnheader "Créé le" [ref=e63]
              - columnheader "Actions" [ref=e64]
          - rowgroup [ref=e65]:
            - row "admin Vous Admin Actif 20.04.2026 Modifier admin Réinitialiser le mot de passe de admin" [ref=e66]:
              - cell "admin Vous" [ref=e67]:
                - text: admin
                - generic [ref=e68]: Vous
              - cell "Admin" [ref=e69]
              - cell "Actif" [ref=e70]
              - cell "20.04.2026" [ref=e71]
              - cell "Modifier admin Réinitialiser le mot de passe de admin" [ref=e72]:
                - generic [ref=e73]:
                  - button "Modifier admin" [ref=e74]:
                    - img
                  - button "Réinitialiser le mot de passe de admin" [ref=e75]:
                    - img
            - row "changeme Admin Actif 20.04.2026 Modifier changeme Réinitialiser le mot de passe de changeme Désactiver changeme" [ref=e76]:
              - cell "changeme" [ref=e77]
              - cell "Admin" [ref=e78]
              - cell "Actif" [ref=e79]
              - cell "20.04.2026" [ref=e80]
              - cell "Modifier changeme Réinitialiser le mot de passe de changeme Désactiver changeme" [ref=e81]:
                - generic [ref=e82]:
                  - button "Modifier changeme" [ref=e83]:
                    - img
                  - button "Réinitialiser le mot de passe de changeme" [ref=e84]:
                    - img
                  - button "Désactiver changeme" [ref=e85]:
                    - img
            - row "test-1776697856062 Comptable Actif 20.04.2026 Modifier test-1776697856062 Réinitialiser le mot de passe de test-1776697856062 Désactiver test-1776697856062" [ref=e86]:
              - cell "test-1776697856062" [ref=e87]
              - cell "Comptable" [ref=e88]
              - cell "Actif" [ref=e89]
              - cell "20.04.2026" [ref=e90]
              - cell "Modifier test-1776697856062 Réinitialiser le mot de passe de test-1776697856062 Désactiver test-1776697856062" [ref=e91]:
                - generic [ref=e92]:
                  - button "Modifier test-1776697856062" [ref=e93]:
                    - img
                  - button "Réinitialiser le mot de passe de test-1776697856062" [ref=e94]:
                    - img
                  - button "Désactiver test-1776697856062" [ref=e95]:
                    - img
    - contentinfo [ref=e96]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T":
    - list:
      - listitem [ref=e97]:
        - img [ref=e99]
        - generic [ref=e102]: Utilisateur « test-1776697856062 » créé.
```

# Test source

```ts
  1   | import { test, expect } from '@playwright/test';
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
  15  |  * Tests E2E — Gestion des utilisateurs (Story 1.12)
  16  |  *
  17  |  * Ces tests nécessitent un backend Kesh fonctionnel sur localhost:3000
  18  |  * avec un admin bootstrap (admin / admin123).
  19  |  */
  20  | 
  21  | /** Helper : login as admin and navigate to /users. */
  22  | async function loginAndGoToUsers(page: import('@playwright/test').Page) {
  23  | 	await page.goto('/login');
  24  | 	await page.fill('#username', 'admin');
  25  | 	await page.fill('#password', 'admin123');
  26  | 	await page.click('button[type="submit"]');
  27  | 	await expect(page).toHaveURL('/');
  28  | 	await page.goto('/users');
  29  | 	await expect(page).toHaveURL('/users');
  30  | }
  31  | 
  32  | test.describe('Page utilisateurs — CRUD', () => {
  33  | 	test('admin voit le lien Utilisateurs dans le sidebar', async ({ page }) => {
  34  | 		await page.goto('/login');
  35  | 		await page.fill('#username', 'admin');
  36  | 		await page.fill('#password', 'admin123');
  37  | 		await page.click('button[type="submit"]');
  38  | 		await expect(page).toHaveURL('/');
  39  | 
  40  | 		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
  41  | 		await expect(sidebar.getByText('Utilisateurs')).toBeVisible();
  42  | 	});
  43  | 
  44  | 	test('liste des utilisateurs affichée avec tableau', async ({ page }) => {
  45  | 		await loginAndGoToUsers(page);
  46  | 
  47  | 		// Le tableau doit être visible
  48  | 		await expect(page.locator('table')).toBeVisible();
  49  | 
  50  | 		// Au moins l'admin bootstrap doit apparaître
  51  | 		await expect(page.getByText('admin')).toBeVisible();
  52  | 
  53  | 		// Badge "Vous" sur l'admin connecté
  54  | 		await expect(page.getByText('Vous')).toBeVisible();
  55  | 	});
  56  | 
  57  | 	test('création d\'un utilisateur', async ({ page }) => {
  58  | 		await loginAndGoToUsers(page);
  59  | 
  60  | 		// Ouvrir le dialog de création
  61  | 		await page.getByText('Nouvel utilisateur').click();
  62  | 		await expect(page.getByText('Créez un nouveau compte')).toBeVisible();
  63  | 
  64  | 		// Remplir le formulaire
  65  | 		const testUser = `test-${Date.now()}`;
  66  | 		await page.fill('#create-username', testUser);
  67  | 		await page.fill('#create-password', 'MotDePasse12345');
  68  | 		await page.fill('#create-confirm', 'MotDePasse12345');
  69  | 
  70  | 		// Soumettre
  71  | 		await page.getByRole('button', { name: 'Créer' }).click();
  72  | 
  73  | 		// L'utilisateur doit apparaître dans le tableau
> 74  | 		await expect(page.getByText(testUser)).toBeVisible({ timeout: 5000 });
      |                                          ^ Error: expect(locator).toBeVisible() failed
  75  | 	});
  76  | 
  77  | 	test('validation mot de passe trop court', async ({ page }) => {
  78  | 		await loginAndGoToUsers(page);
  79  | 
  80  | 		await page.getByText('Nouvel utilisateur').click();
  81  | 		await page.fill('#create-username', 'test-short-pw');
  82  | 		await page.fill('#create-password', 'short');
  83  | 		await page.fill('#create-confirm', 'short');
  84  | 
  85  | 		await page.getByRole('button', { name: 'Créer' }).click();
  86  | 
  87  | 		// Message d'erreur de validation
  88  | 		await expect(page.getByText('au moins 12 caractères')).toBeVisible();
  89  | 	});
  90  | 
  91  | 	test('validation mots de passe non identiques', async ({ page }) => {
  92  | 		await loginAndGoToUsers(page);
  93  | 
  94  | 		await page.getByText('Nouvel utilisateur').click();
  95  | 		await page.fill('#create-username', 'test-mismatch');
  96  | 		await page.fill('#create-password', 'MotDePasse12345');
  97  | 		await page.fill('#create-confirm', 'Différent12345!');
  98  | 
  99  | 		await page.getByRole('button', { name: 'Créer' }).click();
  100 | 
  101 | 		await expect(page.getByText('ne correspondent pas')).toBeVisible();
  102 | 	});
  103 | });
  104 | 
  105 | test.describe('Page utilisateurs — Erreurs', () => {
  106 | 	test('le bouton désactiver est absent pour soi-même', async ({ page }) => {
  107 | 		await loginAndGoToUsers(page);
  108 | 
  109 | 		// La ligne de l'admin connecté (avec badge "Vous") ne doit pas avoir de bouton désactiver
  110 | 		const adminRow = page.locator('tr', { has: page.getByText('Vous') });
  111 | 		await expect(adminRow.getByLabel(/Désactiver/)).not.toBeVisible();
  112 | 	});
  113 | });
  114 | 
  115 | test.describe('Page utilisateurs — Accessibilité', () => {
  116 | 	test('axe-core : pas de violations critiques', async ({ page }) => {
  117 | 		await loginAndGoToUsers(page);
  118 | 
  119 | 		const results = await new AxeBuilder({ page })
  120 | 			.disableRules(['color-contrast']) // tokens custom, vérifiés manuellement
  121 | 			.analyze();
  122 | 
  123 | 		expect(results.violations.filter((v) => v.impact === 'critical')).toHaveLength(0);
  124 | 	});
  125 | });
  126 | 
```