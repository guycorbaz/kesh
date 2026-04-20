# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: mode-expert.spec.ts >> Ctrl+N shortcut (Expert mode) >> Ctrl+N navigates to journal-entries in Expert mode
- Location: tests/e2e/mode-expert.spec.ts:41:2

# Error details

```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /\/journal-entries/
Received string:  "http://127.0.0.1:3000/"
Timeout: 5000ms

Call log:
  - Expect "toHaveURL" with timeout 5000ms
    3 × unexpected value "http://127.0.0.1:3000/login"
    6 × unexpected value "http://127.0.0.1:3000/"

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
  2  | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  3  | 
  4  | test.beforeAll(async () => {
  5  | 	await seedTestState('with-company');
  6  | });
  7  | 
  8  | test.afterEach(async ({ page }) => {
  9  | 	// Clear localStorage after each test to prevent token bleed to next test
  10 | 	await clearAuthStorage(page);
  11 | });
  12 | 
  13 | /**
  14 |  * Tests E2E — Mode Guidé/Expert (Story 2.5)
  15 |  * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`.
  16 |  */
  17 | 
  18 | test.describe('Mode toggle', () => {
  19 | 	test.beforeEach(async ({ page }) => {
  20 | 		await page.goto('/login');
  21 | 		await page.fill('#username', 'changeme');
  22 | 		await page.fill('#password', 'changeme');
  23 | 		await page.click('button[type="submit"]');
  24 | 	});
  25 | 
  26 | 	test('toggle mode changes data-mode attribute on html', async ({ page }) => {
  27 | 		// Default should be guided
  28 | 		const htmlMode = await page.locator('html').getAttribute('data-mode');
  29 | 		expect(htmlMode).toBe('guided');
  30 | 
  31 | 		// Open profile dropdown and click mode toggle
  32 | 		await page.click('button:has-text("Mode")');
  33 | 
  34 | 		// Check data-mode changed
  35 | 		const newMode = await page.locator('html').getAttribute('data-mode');
  36 | 		expect(['guided', 'expert']).toContain(newMode);
  37 | 	});
  38 | });
  39 | 
  40 | test.describe('Ctrl+N shortcut (Expert mode)', () => {
  41 | 	test('Ctrl+N navigates to journal-entries in Expert mode', async ({ page }) => {
  42 | 		await page.goto('/login');
  43 | 		await page.fill('#username', 'changeme');
  44 | 		await page.fill('#password', 'changeme');
  45 | 		await page.click('button[type="submit"]');
  46 | 
  47 | 		// Set expert mode via keyboard evaluation
  48 | 		await page.evaluate(() => {
  49 | 			document.documentElement.setAttribute('data-mode', 'expert');
  50 | 		});
  51 | 
  52 | 		// Ctrl+N shortcut
  53 | 		await page.keyboard.press('Control+n');
  54 | 
  55 | 		// Should navigate to journal-entries
> 56 | 		await expect(page).toHaveURL(/\/journal-entries/);
     |                      ^ Error: expect(page).toHaveURL(expected) failed
  57 | 	});
  58 | });
  59 | 
```