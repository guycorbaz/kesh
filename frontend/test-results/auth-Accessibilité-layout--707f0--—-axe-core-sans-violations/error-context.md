# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: auth.spec.ts >> Accessibilité >> layout principal — axe-core sans violations
- Location: tests/e2e/auth.spec.ts:98:2

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
  95  | 		expect(results.violations).toEqual([]);
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
> 108 | 		expect(results.violations).toEqual([]);
      |                              ^ Error: expect(received).toEqual(expected) // deep equality
  109 | 	});
  110 | });
  111 | 
```