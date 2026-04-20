# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: onboarding-path-b.spec.ts >> Onboarding Path B >> flux complet Path B : langue → mode → production → org → accounting → coords → bank
- Location: tests/e2e/onboarding-path-b.spec.ts:25:2

# Error details

```
Test timeout of 30000ms exceeded.
```

```
Error: page.evaluate: SecurityError: Failed to read the 'localStorage' property from 'Window': Access is denied for this document.
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

```
Error: page.click: Test timeout of 30000ms exceeded.
Call log:
  - waiting for locator('button:has-text("Français")')

```

# Test source

```ts
  1  | import { test, expect } from '@playwright/test';
  2  | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  3  | 
  4  | /**
  5  |  * Tests E2E — Flux d'onboarding Chemin B (Story 2.3)
  6  |  *
  7  |  * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`. Le `beforeEach`
  8  |  * re-seed le preset `fresh` avant chaque test (onboarding mute le singleton
  9  |  * `onboarding_state` de façon irréversible dans le run).
  10 |  */
  11 | 
  12 | test.describe('Onboarding Path B', () => {
  13 | 	test.afterEach(async ({ page }) => {
  14 | 		// Clear localStorage after each test to prevent token bleed to next test
  15 | 		await clearAuthStorage(page);
  16 | 
  17 | 		await seedTestState('fresh');
  18 | 		await page.goto('/login');
  19 | 		await page.fill('#username', 'changeme');
  20 | 		await page.fill('#password', 'changeme');
  21 | 		await page.click('button[type="submit"]');
  22 | 		await expect(page).toHaveURL(/\/onboarding/);
  23 | 	});
  24 | 
  25 | 	test('flux complet Path B : langue → mode → production → org → accounting → coords → bank', async ({ page }) => {
  26 | 		// Step 1: Language
> 27 | 		await page.click('button:has-text("Français")');
     |              ^ Error: page.click: Test timeout of 30000ms exceeded.
  28 | 
  29 | 		// Step 2: Mode
  30 | 		await page.click('button:has-text("Guidé")');
  31 | 
  32 | 		// Step 3: Production path
  33 | 		await page.click('button:has-text("Configurer pour la production")');
  34 | 
  35 | 		// Step 4: Org type
  36 | 		await expect(page.getByText('Indépendant')).toBeVisible();
  37 | 		await page.click('button:has-text("PME")');
  38 | 
  39 | 		// Step 5: Accounting language
  40 | 		await expect(page.getByText('Langue comptable')).toBeVisible();
  41 | 		await page.click('button:has-text("Français")');
  42 | 
  43 | 		// Step 6: Coordinates
  44 | 		await expect(page.getByText('Coordonnées')).toBeVisible();
  45 | 		await page.fill('#coord-name', 'Ma Société SA');
  46 | 		await page.fill('#coord-address', 'Rue du Test 1, 1000 Lausanne');
  47 | 		await page.click('button:has-text("Continuer")');
  48 | 
  49 | 		// Step 7: Bank (skip)
  50 | 		await expect(page.getByText('Compte bancaire')).toBeVisible();
  51 | 		await page.click('button:has-text("Configurer plus tard")');
  52 | 
  53 | 		// Should be in app with blue banner
  54 | 		await expect(page).toHaveURL('/');
  55 | 		await expect(page.getByText('Configuration incomplète')).toBeVisible();
  56 | 	});
  57 | 
  58 | 	test('flux Path B avec banque configurée → pas de bannière bleue', async ({ page }) => {
  59 | 		// Steps 1-6 same as above
  60 | 		await page.click('button:has-text("Français")');
  61 | 		await page.click('button:has-text("Expert")');
  62 | 		await page.click('button:has-text("Configurer pour la production")');
  63 | 		await page.click('button:has-text("Association")');
  64 | 		await page.click('button:has-text("Français")');
  65 | 		await page.fill('#coord-name', 'Mon Association');
  66 | 		await page.fill('#coord-address', 'Rue 1');
  67 | 		await page.click('button:has-text("Continuer")');
  68 | 
  69 | 		// Step 7: Bank (fill)
  70 | 		await page.fill('#bank-name', 'UBS');
  71 | 		await page.fill('#bank-iban', 'CH93 0076 2011 6238 5295 7');
  72 | 		await page.click('button:has-text("Enregistrer")');
  73 | 
  74 | 		// Should be in app WITHOUT blue banner
  75 | 		await expect(page).toHaveURL('/');
  76 | 		await expect(page.getByText('Configuration incomplète')).not.toBeVisible();
  77 | 	});
  78 | });
  79 | 
```