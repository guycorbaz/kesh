# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: homepage-settings.spec.ts >> Settings >> page Paramètres affiche 4 sections
- Location: tests/e2e/homepage-settings.spec.ts:43:2

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByText('Organisation')
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByText('Organisation')

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - main [ref=e3]:
    - generic [ref=e4]:
      - heading "Kesh" [level=1] [ref=e5]
      - alert
      - generic [ref=e6]:
        - generic [ref=e7]:
          - generic [ref=e8]: Identifiant
          - textbox "Identifiant" [ref=e9]:
            - /placeholder: Votre identifiant
        - generic [ref=e10]:
          - generic [ref=e11]: Mot de passe
          - textbox "Mot de passe" [ref=e12]:
            - /placeholder: Votre mot de passe
        - button "Se connecter" [ref=e13]
  - region "Notifications alt+T"
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
> 45 | 		await expect(page.getByText('Organisation')).toBeVisible();
     |                                                ^ Error: expect(locator).toBeVisible() failed
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
  64 | 		expect(results.violations).toEqual([]);
  65 | 	});
  66 | });
  67 | 
```