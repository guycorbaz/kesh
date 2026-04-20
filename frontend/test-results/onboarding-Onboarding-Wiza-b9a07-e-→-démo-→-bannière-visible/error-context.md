# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: onboarding.spec.ts >> Onboarding Wizard >> flux complet : langue → mode → démo → bannière visible
- Location: tests/e2e/onboarding.spec.ts:37:2

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
  5  |  * Tests E2E — Flux d'onboarding Chemin A (Story 2.2)
  6  |  *
  7  |  * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`. Le `beforeEach`
  8  |  * seed le preset `fresh` (uniquement user `changeme/changeme`, aucune
  9  |  * company, aucun onboarding_state) → chaque test démarre d'un onboarding
  10 |  * vierge, évitant la mutation irréversible du singleton `onboarding_state`.
  11 |  */
  12 | 
  13 | test.describe('Onboarding Wizard', () => {
  14 | 	test.afterEach(async ({ page }) => {
  15 | 		// Clear localStorage after each test to prevent token bleed to next test
  16 | 		await clearAuthStorage(page);
  17 | 
  18 | 		// Reset DB + user `changeme` seul (preset fresh, cf. AC #7).
  19 | 		await seedTestState('fresh');
  20 | 
  21 | 		// Login en tant que changeme/changeme (le seul user du preset fresh).
  22 | 		await page.goto('/login');
  23 | 		await page.fill('#username', 'changeme');
  24 | 		await page.fill('#password', 'changeme');
  25 | 		await page.click('button[type="submit"]');
  26 | 
  27 | 		// Le guard onboarding devrait rediriger vers /onboarding
  28 | 		await expect(page).toHaveURL(/\/onboarding/);
  29 | 	});
  30 | 
  31 | 	test('étape 1 — choix de langue affiche 4 options', async ({ page }) => {
  32 | 		// 4 boutons de langue
  33 | 		const langButtons = page.locator('button:has-text("Français"), button:has-text("Deutsch"), button:has-text("Italiano"), button:has-text("English")');
  34 | 		await expect(langButtons).toHaveCount(4);
  35 | 	});
  36 | 
  37 | 	test('flux complet : langue → mode → démo → bannière visible', async ({ page }) => {
  38 | 		// Étape 1 : Choisir français
> 39 | 		await page.click('button:has-text("Français")');
     |              ^ Error: page.click: Test timeout of 30000ms exceeded.
  40 | 
  41 | 		// Étape 2 : Choisir mode guidé
  42 | 		await expect(page.getByText('Guidé')).toBeVisible();
  43 | 		await expect(page.getByText('Expert')).toBeVisible();
  44 | 		await page.click('button:has-text("Guidé")');
  45 | 
  46 | 		// Étape 3 : Choisir démo
  47 | 		await expect(page.getByText('Explorer avec des données de démo')).toBeVisible();
  48 | 		await page.click('button:has-text("Explorer avec des données de démo")');
  49 | 
  50 | 		// Redirect vers / avec bannière démo
  51 | 		await expect(page).toHaveURL('/');
  52 | 		await expect(page.getByText('Instance de démonstration')).toBeVisible();
  53 | 	});
  54 | 
  55 | 	test('bannière démo — reset redirige vers onboarding', async ({ page }) => {
  56 | 		// Complete onboarding first
  57 | 		await page.click('button:has-text("Français")');
  58 | 		await page.click('button:has-text("Guidé")');
  59 | 		await page.click('button:has-text("Explorer avec des données de démo")');
  60 | 		await expect(page).toHaveURL('/');
  61 | 
  62 | 		// Click reset
  63 | 		await page.click('button:has-text("Réinitialiser pour la production")');
  64 | 
  65 | 		// Confirm dialog
  66 | 		await expect(page.getByText('Toutes les données de démonstration')).toBeVisible();
  67 | 		await page.click('button:has-text("Confirmer")');
  68 | 
  69 | 		// Should redirect to onboarding
  70 | 		await expect(page).toHaveURL(/\/onboarding/);
  71 | 	});
  72 | });
  73 | 
  74 | test.describe('Onboarding — Reprise après interruption', () => {
  75 | 	test('F5 à étape 2 reprend à étape 2', async ({ page }) => {
  76 | 		// Login
  77 | 		await page.goto('/login');
  78 | 		await page.fill('#username', 'admin');
  79 | 		await page.fill('#password', 'changeme');
  80 | 		await page.click('button[type="submit"]');
  81 | 		await expect(page).toHaveURL(/\/onboarding/);
  82 | 
  83 | 		// Complete step 1
  84 | 		await page.click('button:has-text("Français")');
  85 | 
  86 | 		// Should be at step 2 now
  87 | 		await expect(page.getByText('Guidé')).toBeVisible();
  88 | 
  89 | 		// Simulate refresh (F5)
  90 | 		await page.reload();
  91 | 
  92 | 		// Should still be at step 2
  93 | 		await expect(page.getByText('Guidé')).toBeVisible();
  94 | 	});
  95 | });
  96 | 
```