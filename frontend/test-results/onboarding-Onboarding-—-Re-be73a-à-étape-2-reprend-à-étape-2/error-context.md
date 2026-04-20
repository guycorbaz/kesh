# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: onboarding.spec.ts >> Onboarding — Reprise après interruption >> F5 à étape 2 reprend à étape 2
- Location: tests/e2e/onboarding.spec.ts:77:2

# Error details

```
Error: expect(page).toHaveURL(expected) failed

Expected pattern: /\/onboarding/
Received string:  "http://127.0.0.1:3000/login"
Timeout: 5000ms

Call log:
  - Expect "toHaveURL" with timeout 5000ms
    9 × unexpected value "http://127.0.0.1:3000/login"

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - main [ref=e3]:
    - generic [ref=e4]:
      - heading "Kesh" [level=1] [ref=e5]
      - alert [ref=e6]:
        - img [ref=e7]
        - generic [ref=e8]: Identifiant ou mot de passe incorrect
      - generic [ref=e9]:
        - generic [ref=e10]:
          - generic [ref=e11]: Identifiant
          - textbox "Identifiant" [ref=e12]:
            - /placeholder: Votre identifiant
            - text: admin
        - generic [ref=e13]:
          - generic [ref=e14]: Mot de passe
          - textbox "Mot de passe" [ref=e15]:
            - /placeholder: Votre mot de passe
            - text: changeme
        - button "Se connecter" [ref=e16]
  - region "Notifications alt+T"
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
  14 | 	test.beforeEach(async ({ page }) => {
  15 | 		// Reset DB + user `changeme` seul (preset fresh, cf. AC #7).
  16 | 		await seedTestState('fresh');
  17 | 
  18 | 		// Login en tant que changeme/changeme (le seul user du preset fresh).
  19 | 		await page.goto('/login');
  20 | 		await page.fill('#username', 'changeme');
  21 | 		await page.fill('#password', 'changeme');
  22 | 		await page.click('button[type="submit"]');
  23 | 
  24 | 		// Le guard onboarding devrait rediriger vers /onboarding
  25 | 		await expect(page).toHaveURL(/\/onboarding/);
  26 | 	});
  27 | 
  28 | 	test.afterEach(async ({ page }) => {
  29 | 		// Clear localStorage after each test to prevent token bleed to next test
  30 | 		await clearAuthStorage(page);
  31 | 	});
  32 | 
  33 | 	test('étape 1 — choix de langue affiche 4 options', async ({ page }) => {
  34 | 		// 4 boutons de langue
  35 | 		const langButtons = page.locator('button:has-text("Français"), button:has-text("Deutsch"), button:has-text("Italiano"), button:has-text("English")');
  36 | 		await expect(langButtons).toHaveCount(4);
  37 | 	});
  38 | 
  39 | 	test('flux complet : langue → mode → démo → bannière visible', async ({ page }) => {
  40 | 		// Étape 1 : Choisir français
  41 | 		await page.click('button:has-text("Français")');
  42 | 
  43 | 		// Étape 2 : Choisir mode guidé
  44 | 		await expect(page.getByText('Guidé')).toBeVisible();
  45 | 		await expect(page.getByText('Expert')).toBeVisible();
  46 | 		await page.click('button:has-text("Guidé")');
  47 | 
  48 | 		// Étape 3 : Choisir démo
  49 | 		await expect(page.getByText('Explorer avec des données de démo')).toBeVisible();
  50 | 		await page.click('button:has-text("Explorer avec des données de démo")');
  51 | 
  52 | 		// Redirect vers / avec bannière démo
  53 | 		await expect(page).toHaveURL('/');
  54 | 		await expect(page.getByText('Instance de démonstration')).toBeVisible();
  55 | 	});
  56 | 
  57 | 	test('bannière démo — reset redirige vers onboarding', async ({ page }) => {
  58 | 		// Complete onboarding first
  59 | 		await page.click('button:has-text("Français")');
  60 | 		await page.click('button:has-text("Guidé")');
  61 | 		await page.click('button:has-text("Explorer avec des données de démo")');
  62 | 		await expect(page).toHaveURL('/');
  63 | 
  64 | 		// Click reset
  65 | 		await page.click('button:has-text("Réinitialiser pour la production")');
  66 | 
  67 | 		// Confirm dialog
  68 | 		await expect(page.getByText('Toutes les données de démonstration')).toBeVisible();
  69 | 		await page.click('button:has-text("Confirmer")');
  70 | 
  71 | 		// Should redirect to onboarding
  72 | 		await expect(page).toHaveURL(/\/onboarding/);
  73 | 	});
  74 | });
  75 | 
  76 | test.describe('Onboarding — Reprise après interruption', () => {
  77 | 	test('F5 à étape 2 reprend à étape 2', async ({ page }) => {
  78 | 		// Login
  79 | 		await page.goto('/login');
  80 | 		await page.fill('#username', 'admin');
  81 | 		await page.fill('#password', 'changeme');
  82 | 		await page.click('button[type="submit"]');
> 83 | 		await expect(page).toHaveURL(/\/onboarding/);
     |                      ^ Error: expect(page).toHaveURL(expected) failed
  84 | 
  85 | 		// Complete step 1
  86 | 		await page.click('button:has-text("Français")');
  87 | 
  88 | 		// Should be at step 2 now
  89 | 		await expect(page.getByText('Guidé')).toBeVisible();
  90 | 
  91 | 		// Simulate refresh (F5)
  92 | 		await page.reload();
  93 | 
  94 | 		// Should still be at step 2
  95 | 		await expect(page.getByText('Guidé')).toBeVisible();
  96 | 	});
  97 | });
  98 | 
```