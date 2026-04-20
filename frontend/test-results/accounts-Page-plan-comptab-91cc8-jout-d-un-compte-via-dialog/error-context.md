# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: accounts.spec.ts >> Page plan comptable — CRUD >> ajout d'un compte via dialog
- Location: tests/e2e/accounts.spec.ts:61:2

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByText('9999')
Expected: visible
Error: strict mode violation: getByText('9999') resolved to 2 elements:
    1) <span class="font-mono text-sm text-text-muted whitespace-nowrap">9999</span> aka getByText('9999', { exact: true })
    2) <div data-title="">…</div> aka getByText('Compte 9999 créé')

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByText('9999')

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
          - heading "Plan comptable" [level=1] [ref=e54]
          - generic [ref=e55]:
            - generic [ref=e56]:
              - checkbox "Afficher les archivés" [ref=e57]
              - text: Afficher les archivés
            - button "Nouveau compte" [active] [ref=e58]:
              - img
              - text: Nouveau compte
        - generic [ref=e59]:
          - generic [ref=e60]:
            - generic [ref=e61]:
              - generic [ref=e62]: "1000"
              - generic [ref=e63]: Caisse CI
              - generic [ref=e64]: Actif
            - generic [ref=e65]:
              - button "Modifier 1000" [ref=e66]:
                - img
              - button "Archiver 1000" [ref=e67]:
                - img
          - generic [ref=e68]:
            - generic [ref=e69]:
              - generic [ref=e70]: "1100"
              - generic [ref=e71]: Banque CI
              - generic [ref=e72]: Actif
            - generic [ref=e73]:
              - button "Modifier 1100" [ref=e74]:
                - img
              - button "Archiver 1100" [ref=e75]:
                - img
          - generic [ref=e76]:
            - generic [ref=e77]:
              - generic [ref=e78]: "2000"
              - generic [ref=e79]: Capital CI
              - generic [ref=e80]: Passif
            - generic [ref=e81]:
              - button "Modifier 2000" [ref=e82]:
                - img
              - button "Archiver 2000" [ref=e83]:
                - img
          - generic [ref=e84]:
            - generic [ref=e85]:
              - generic [ref=e86]: "3000"
              - generic [ref=e87]: Ventes CI
              - generic [ref=e88]: Produit
            - generic [ref=e89]:
              - button "Modifier 3000" [ref=e90]:
                - img
              - button "Archiver 3000" [ref=e91]:
                - img
          - generic [ref=e92]:
            - generic [ref=e93]:
              - generic [ref=e94]: "4000"
              - generic [ref=e95]: Charges CI
              - generic [ref=e96]: Charge
            - generic [ref=e97]:
              - button "Modifier 4000" [ref=e98]:
                - img
              - button "Archiver 4000" [ref=e99]:
                - img
          - generic [ref=e100]:
            - generic [ref=e101]:
              - generic [ref=e102]: "9999"
              - generic [ref=e103]: Compte de test E2E
              - generic [ref=e104]: Actif
            - generic [ref=e105]:
              - button "Modifier 9999" [ref=e106]:
                - img
              - button "Archiver 9999" [ref=e107]:
                - img
        - paragraph [ref=e108]: 6 comptes
    - contentinfo [ref=e109]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T":
    - list:
      - listitem [ref=e110]:
        - img [ref=e112]
        - generic [ref=e115]: Compte 9999 créé.
```

# Test source

```ts
  1   | import { test, expect } from '@playwright/test';
  2   | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  3   | 
  4   | /**
  5   |  * Tests E2E — Plan comptable (Story 3.1)
  6   |  *
  7   |  * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true` + `KESH_HOST=127.0.0.1`.
  8   |  * Le `beforeAll` truncate la DB et re-seed via l'endpoint `/api/v1/_test/seed`
  9   |  * → état déterministe indépendant de l'ordre des specs.
  10  |  */
  11  | 
  12  | test.beforeAll(async () => {
  13  | 	await seedTestState('with-company');
  14  | });
  15  | 
  16  | test.afterEach(async ({ page }) => {
  17  | 	// Clear auth tokens after each test to prevent token bleed to next test
  18  | 	await clearAuthStorage(page);
  19  | });
  20  | 
  21  | /** Helper : login as admin and navigate to /accounts. */
  22  | async function loginAndGoToAccounts(page: import('@playwright/test').Page) {
  23  | 	await page.goto('/login');
  24  | 	await page.fill('#username', 'admin');
  25  | 	await page.fill('#password', 'admin123');
  26  | 	await page.click('button[type="submit"]');
  27  | 	await expect(page).toHaveURL('/');
  28  | 	await page.goto('/accounts');
  29  | 	await expect(page).toHaveURL('/accounts');
  30  | }
  31  | 
  32  | test.describe('Page plan comptable — affichage', () => {
  33  | 	test('affiche le titre Plan comptable', async ({ page }) => {
  34  | 		await loginAndGoToAccounts(page);
  35  | 		await expect(page.getByText('Plan comptable')).toBeVisible();
  36  | 	});
  37  | 
  38  | 	test('affiche l\'arborescence des comptes avec numeros', async ({ page }) => {
  39  | 		await loginAndGoToAccounts(page);
  40  | 
  41  | 		// Les comptes du plan PME doivent etre visibles
  42  | 		await expect(page.getByText('1000')).toBeVisible();
  43  | 		await expect(page.getByText('2000')).toBeVisible();
  44  | 	});
  45  | 
  46  | 	test('affiche le type de compte (badge)', async ({ page }) => {
  47  | 		await loginAndGoToAccounts(page);
  48  | 
  49  | 		// Les badges de type doivent etre presents
  50  | 		await expect(page.getByText('Actif').first()).toBeVisible();
  51  | 		await expect(page.getByText('Passif').first()).toBeVisible();
  52  | 	});
  53  | 
  54  | 	test('affiche le compteur de comptes', async ({ page }) => {
  55  | 		await loginAndGoToAccounts(page);
  56  | 		await expect(page.getByText(/\d+ comptes/)).toBeVisible();
  57  | 	});
  58  | });
  59  | 
  60  | test.describe('Page plan comptable — CRUD', () => {
  61  | 	test('ajout d\'un compte via dialog', async ({ page }) => {
  62  | 		await loginAndGoToAccounts(page);
  63  | 
  64  | 		// Ouvrir le dialog de creation
  65  | 		await page.getByText('Nouveau compte').click();
  66  | 		await expect(page.getByText('Ajoutez un compte')).toBeVisible();
  67  | 
  68  | 		// Remplir le formulaire
  69  | 		const testNumber = `9999`;
  70  | 		await page.fill('#create-number', testNumber);
  71  | 		await page.fill('#create-name', 'Compte de test E2E');
  72  | 
  73  | 		// Soumettre
  74  | 		await page.getByRole('button', { name: 'Créer' }).click();
  75  | 
  76  | 		// Le toast de succes doit apparaitre
  77  | 		await expect(page.getByText(`Compte ${testNumber} créé`)).toBeVisible();
  78  | 
  79  | 		// Le compte doit apparaitre dans la liste
> 80  | 		await expect(page.getByText(testNumber)).toBeVisible();
      |                                            ^ Error: expect(locator).toBeVisible() failed
  81  | 		await expect(page.getByText('Compte de test E2E')).toBeVisible();
  82  | 	});
  83  | 
  84  | 	test('modification d\'un compte via dialog', async ({ page }) => {
  85  | 		await loginAndGoToAccounts(page);
  86  | 
  87  | 		// Cliquer sur le bouton modifier du premier compte visible
  88  | 		const editButton = page.getByLabel(/Modifier/).first();
  89  | 		await editButton.click();
  90  | 
  91  | 		// Le dialog de modification doit s'ouvrir
  92  | 		await expect(page.getByText('Le numéro n\'est pas modifiable')).toBeVisible();
  93  | 
  94  | 		// Le champ numero doit etre desactive
  95  | 		const numberField = page.locator('#edit-number');
  96  | 		await expect(numberField).toBeDisabled();
  97  | 
  98  | 		// Fermer sans modifier
  99  | 		await page.getByRole('button', { name: 'Annuler' }).click();
  100 | 	});
  101 | 
  102 | 	test('toggle afficher les archives', async ({ page }) => {
  103 | 		await loginAndGoToAccounts(page);
  104 | 
  105 | 		// La checkbox "Afficher les archives" doit exister
  106 | 		await expect(page.getByText('Afficher les archivés')).toBeVisible();
  107 | 	});
  108 | });
  109 | 
```