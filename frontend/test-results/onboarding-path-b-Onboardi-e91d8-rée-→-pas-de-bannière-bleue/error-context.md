# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: onboarding-path-b.spec.ts >> Onboarding Path B >> flux Path B avec banque configurée → pas de bannière bleue
- Location: tests/e2e/onboarding-path-b.spec.ts:60:2

# Error details

```
Test timeout of 30000ms exceeded.
```

```
Error: page.click: Test timeout of 30000ms exceeded.
Call log:
  - waiting for locator('button:has-text("Enregistrer")')

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - generic [ref=e3]:
    - link "Kesh" [ref=e5] [cursor=pointer]:
      - /url: /
    - generic [ref=e6]:
      - heading "Compte bancaire principal" [level=2] [ref=e7]
      - generic [ref=e8]:
        - generic [ref=e9]:
          - generic [ref=e10]: Nom de la banque *
          - textbox "Nom de la banque *" [ref=e11]: UBS
        - generic [ref=e12]:
          - generic [ref=e13]: IBAN *
          - textbox "IBAN *" [active] [ref=e14]:
            - /placeholder: CH93 0076 2011 6238 5295 7
            - text: CH93 0076 2011 6238 5295 7
        - generic [ref=e15]:
          - generic [ref=e16]: QR-IBAN (optionnel)
          - textbox "QR-IBAN (optionnel)" [ref=e17]:
            - /placeholder: CH44 3199 9123 0008 8901 2
        - generic [ref=e18]:
          - button "Configurer plus tard" [ref=e19]
          - button "Continuer" [ref=e20]
    - contentinfo [ref=e21]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T"
  - generic [ref=e22]: Connexion - Kesh
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
  13 | 	test.beforeEach(async ({ page }) => {
  14 | 		await seedTestState('fresh');
  15 | 		await page.goto('/login');
  16 | 		await page.fill('#username', 'changeme');
  17 | 		await page.fill('#password', 'changeme');
  18 | 		await page.click('button[type="submit"]');
  19 | 		await expect(page).toHaveURL(/\/onboarding/);
  20 | 	});
  21 | 
  22 | 	test.afterEach(async ({ page }) => {
  23 | 		// Clear localStorage after each test to prevent token bleed to next test
  24 | 		await clearAuthStorage(page);
  25 | 	});
  26 | 
  27 | 	test('flux complet Path B : langue → mode → production → org → accounting → coords → bank', async ({ page }) => {
  28 | 		// Step 1: Language
  29 | 		await page.click('button:has-text("Français")');
  30 | 
  31 | 		// Step 2: Mode
  32 | 		await page.click('button:has-text("Guidé")');
  33 | 
  34 | 		// Step 3: Production path
  35 | 		await page.click('button:has-text("Configurer pour la production")');
  36 | 
  37 | 		// Step 4: Org type
  38 | 		await expect(page.getByText('Indépendant')).toBeVisible();
  39 | 		await page.click('button:has-text("PME")');
  40 | 
  41 | 		// Step 5: Accounting language
  42 | 		await expect(page.getByText('Langue comptable')).toBeVisible();
  43 | 		await page.click('button:has-text("Français")');
  44 | 
  45 | 		// Step 6: Coordinates
  46 | 		await expect(page.getByText('Coordonnées')).toBeVisible();
  47 | 		await page.fill('#coord-name', 'Ma Société SA');
  48 | 		await page.fill('#coord-address', 'Rue du Test 1, 1000 Lausanne');
  49 | 		await page.click('button:has-text("Continuer")');
  50 | 
  51 | 		// Step 7: Bank (skip)
  52 | 		await expect(page.getByText('Compte bancaire')).toBeVisible();
  53 | 		await page.click('button:has-text("Configurer plus tard")');
  54 | 
  55 | 		// Should be in app with blue banner
  56 | 		await expect(page).toHaveURL('/');
  57 | 		await expect(page.getByText('Configuration incomplète')).toBeVisible();
  58 | 	});
  59 | 
  60 | 	test('flux Path B avec banque configurée → pas de bannière bleue', async ({ page }) => {
  61 | 		// Steps 1-6 same as above
  62 | 		await page.click('button:has-text("Français")');
  63 | 		await page.click('button:has-text("Expert")');
  64 | 		await page.click('button:has-text("Configurer pour la production")');
  65 | 		await page.click('button:has-text("Association")');
  66 | 		await page.click('button:has-text("Français")');
  67 | 		await page.fill('#coord-name', 'Mon Association');
  68 | 		await page.fill('#coord-address', 'Rue 1');
  69 | 		await page.click('button:has-text("Continuer")');
  70 | 
  71 | 		// Step 7: Bank (fill)
  72 | 		await page.fill('#bank-name', 'UBS');
  73 | 		await page.fill('#bank-iban', 'CH93 0076 2011 6238 5295 7');
> 74 | 		await page.click('button:has-text("Enregistrer")');
     |              ^ Error: page.click: Test timeout of 30000ms exceeded.
  75 | 
  76 | 		// Should be in app WITHOUT blue banner
  77 | 		await expect(page).toHaveURL('/');
  78 | 		await expect(page.getByText('Configuration incomplète')).not.toBeVisible();
  79 | 	});
  80 | });
  81 | 
```