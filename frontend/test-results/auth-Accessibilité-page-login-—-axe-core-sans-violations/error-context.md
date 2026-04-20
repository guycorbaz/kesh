# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: auth.spec.ts >> Accessibilité >> page login — axe-core sans violations
- Location: tests/e2e/auth.spec.ts:90:2

# Error details

```
Error: expect(received).toEqual(expected) // deep equality

- Expected  -   1
+ Received  + 109

- Array []
+ Array [
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
+         "html": "<html lang=\"fr\">",
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
+     "description": "Ensure the document has a main landmark",
+     "help": "Document should have one main landmark",
+     "helpUrl": "https://dequeuniversity.com/rules/axe/4.11/landmark-one-main?application=playwright",
+     "id": "landmark-one-main",
+     "impact": "moderate",
+     "nodes": Array [
+       Object {
+         "all": Array [
+           Object {
+             "data": null,
+             "id": "page-has-main",
+             "impact": "moderate",
+             "message": "Document does not have a main landmark",
+             "relatedNodes": Array [],
+           },
+         ],
+         "any": Array [],
+         "failureSummary": "Fix all of the following:
+   Document does not have a main landmark",
+         "html": "<html lang=\"fr\">",
+         "impact": "moderate",
+         "none": Array [],
+         "target": Array [
+           "html",
+         ],
+       },
+     ],
+     "tags": Array [
+       "cat.semantics",
+       "best-practice",
+     ],
+   },
+   Object {
+     "description": "Ensure that the page, or at least one of its frames contains a level-one heading",
+     "help": "Page should contain a level-one heading",
+     "helpUrl": "https://dequeuniversity.com/rules/axe/4.11/page-has-heading-one?application=playwright",
+     "id": "page-has-heading-one",
+     "impact": "moderate",
+     "nodes": Array [
+       Object {
+         "all": Array [
+           Object {
+             "data": null,
+             "id": "page-has-heading-one",
+             "impact": "moderate",
+             "message": "Page must have a level-one heading",
+             "relatedNodes": Array [],
+           },
+         ],
+         "any": Array [],
+         "failureSummary": "Fix all of the following:
+   Page must have a level-one heading",
+         "html": "<html lang=\"fr\">",
+         "impact": "moderate",
+         "none": Array [],
+         "target": Array [
+           "html",
+         ],
+       },
+     ],
+     "tags": Array [
+       "cat.semantics",
+       "best-practice",
+     ],
+   },
+ ]
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
  1   | import { test, expect } from '@playwright/test';
  2   | import AxeBuilder from '@axe-core/playwright';
  3   | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  4   | 
  5   | /**
  6   |  * Tests E2E — Authentification & Accessibilité (Story 1.11)
  7   |  *
  8   |  * Ces tests nécessitent un backend Kesh fonctionnel sur localhost:3000
  9   |  * avec `KESH_TEST_MODE=true` (cf. Story 6.4).
  10  |  */
  11  | 
  12  | test.beforeAll(async () => {
  13  | 	await seedTestState('with-company');
  14  | });
  15  | 
  16  | test.afterEach(async ({ page }) => {
  17  | 	// Clear localStorage after each test to prevent token bleed to next test
  18  | 	await clearAuthStorage(page);
  19  | });
  20  | 
  21  | test.describe('Login', () => {
  22  | 	test('login réussi → redirection accueil, affichage header/sidebar', async ({ page }) => {
  23  | 		await page.goto('/login');
  24  | 
  25  | 		await page.fill('#username', 'admin');
  26  | 		await page.fill('#password', 'admin123');
  27  | 		await page.click('button[type="submit"]');
  28  | 
  29  | 		// Redirection vers l'accueil
  30  | 		await expect(page).toHaveURL('/');
  31  | 
  32  | 		// Header visible avec logo Kesh
  33  | 		await expect(page.locator('header')).toBeVisible();
  34  | 		await expect(page.locator('header').getByText('Kesh')).toBeVisible();
  35  | 
  36  | 		// Sidebar navigation visible
  37  | 		await expect(page.locator('nav[aria-label="Navigation principale"]')).toBeVisible();
  38  | 	});
  39  | 
  40  | 	test('login échoué → message d\'erreur affiché', async ({ page }) => {
  41  | 		await page.goto('/login');
  42  | 
  43  | 		await page.fill('#username', 'wrong');
  44  | 		await page.fill('#password', 'wrong');
  45  | 		await page.click('button[type="submit"]');
  46  | 
  47  | 		// Rester sur la page login
  48  | 		await expect(page).toHaveURL(/\/login/);
  49  | 
  50  | 		// Message d'erreur visible
  51  | 		await expect(page.locator('#login-error')).toContainText('Identifiant ou mot de passe incorrect');
  52  | 	});
  53  | 
  54  | 	test('accès page protégée sans auth → redirect login', async ({ page }) => {
  55  | 		// Tenter d'accéder à l'accueil sans être connecté
  56  | 		await page.goto('/');
  57  | 
  58  | 		// Doit être redirigé vers /login
  59  | 		await expect(page).toHaveURL(/\/login/);
  60  | 	});
  61  | 
  62  | 	test('raccourci Ctrl+S déclenche l\'événement kesh:save', async ({ page }) => {
  63  | 		// Se connecter d'abord
  64  | 		await page.goto('/login');
  65  | 		await page.fill('#username', 'admin');
  66  | 		await page.fill('#password', 'admin123');
  67  | 		await page.click('button[type="submit"]');
  68  | 		await expect(page).toHaveURL('/');
  69  | 
  70  | 		// Attacher le listener AVANT de presser la touche (évite la race condition)
  71  | 		await page.evaluate(() => {
  72  | 			(window as unknown as Record<string, boolean>).__keshSaveFired = false;
  73  | 			window.addEventListener('kesh:save', () => {
  74  | 				(window as unknown as Record<string, boolean>).__keshSaveFired = true;
  75  | 			}, { once: true });
  76  | 		});
  77  | 
  78  | 		// Presser Ctrl+S
  79  | 		await page.keyboard.press('Control+s');
  80  | 
  81  | 		// Vérifier que l'événement a été déclenché
  82  | 		const eventFired = await page.evaluate(
  83  | 			() => (window as unknown as Record<string, boolean>).__keshSaveFired,
  84  | 		);
  85  | 		expect(eventFired).toBe(true);
  86  | 	});
  87  | });
  88  | 
  89  | test.describe('Accessibilité', () => {
  90  | 	test('page login — axe-core sans violations', async ({ page }) => {
  91  | 		await page.goto('/login');
  92  | 
  93  | 		const results = await new AxeBuilder({ page }).analyze();
  94  | 
> 95  | 		expect(results.violations).toEqual([]);
      |                              ^ Error: expect(received).toEqual(expected) // deep equality
  96  | 	});
  97  | 
  98  | 	test('layout principal — axe-core sans violations', async ({ page }) => {
  99  | 		// Se connecter d'abord
  100 | 		await page.goto('/login');
  101 | 		await page.fill('#username', 'admin');
  102 | 		await page.fill('#password', 'admin123');
  103 | 		await page.click('button[type="submit"]');
  104 | 		await expect(page).toHaveURL('/');
  105 | 
  106 | 		const results = await new AxeBuilder({ page }).analyze();
  107 | 
  108 | 		expect(results.violations).toEqual([]);
  109 | 	});
  110 | });
  111 | 
```