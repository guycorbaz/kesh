# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: accounts.spec.ts >> Page plan comptable — CRUD >> modification d'un compte via dialog
- Location: tests/e2e/accounts.spec.ts:84:2

# Error details

```
Error: locator.click: Error: strict mode violation: getByRole('button', { name: 'Annuler' }) resolved to 2 elements:
    1) <button tabindex="0" type="button" id="bits-c15" data-state="open" data-dialog-close="" data-slot="dialog-close">…</button> aka locator('#bits-c15')
    2) <button type="button" data-slot="button" class="focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 rounded-lg border bg-clip-padding text-sm font-medium focus-visible:ring-3 active:not-aria-[haspopup]:translate-y-px aria-invalid:ring-3 [&_svg:not([class*='size-'])]:size-4 group/button inline-flex shrink-0 items-center justify-center whitespace-nowrap transiti…>…</button> aka getByText('Annuler')

Call log:
  - waiting for getByRole('button', { name: 'Annuler' })

```

# Page snapshot

```yaml
- generic:
  - generic:
    - generic:
      - banner:
        - link "Kesh":
          - /url: /
          - generic: Kesh
        - generic:
          - img
          - searchbox "Rechercher..."
        - generic:
          - button "Admin":
            - button "Admin":
              - img
              - generic: Admin
              - img
      - generic:
        - navigation "Navigation principale":
          - generic: Quotidien
          - list:
            - listitem:
              - link "Accueil":
                - /url: /
            - listitem:
              - link "Carnet d'adresses":
                - /url: /contacts
            - listitem:
              - link "Catalogue":
                - /url: /products
            - listitem:
              - link "Facturer":
                - /url: /invoices
            - listitem:
              - link "Échéancier":
                - /url: /invoices/due-dates
            - listitem:
              - link "Payer":
                - /url: /bank-accounts
            - listitem:
              - link "Importer":
                - /url: /bank-import
          - separator
          - generic: Mensuel
          - list:
            - listitem:
              - link "Écritures":
                - /url: /journal-entries
            - listitem:
              - link "Réconciliation":
                - /url: /reconciliation
            - listitem:
              - link "Rapports":
                - /url: /reports
          - separator
          - list:
            - listitem:
              - link "Paramètres":
                - /url: /settings
          - separator
          - generic: Administration
          - list:
            - listitem:
              - link "Utilisateurs":
                - /url: /users
            - listitem:
              - link "Facturation":
                - /url: /settings/invoicing
        - main:
          - generic:
            - heading "Plan comptable" [level=1]
            - generic:
              - generic:
                - checkbox "Afficher les archivés"
                - text: Afficher les archivés
              - button "Nouveau compte":
                - img
                - text: Nouveau compte
          - generic:
            - generic:
              - generic:
                - generic: "1000"
                - generic: Caisse CI
                - generic: Actif
              - generic:
                - button "Modifier 1000":
                  - img
                - button "Archiver 1000":
                  - img
            - generic:
              - generic:
                - generic: "1100"
                - generic: Banque CI
                - generic: Actif
              - generic:
                - button "Modifier 1100":
                  - img
                - button "Archiver 1100":
                  - img
            - generic:
              - generic:
                - generic: "2000"
                - generic: Capital CI
                - generic: Passif
              - generic:
                - button "Modifier 2000":
                  - img
                - button "Archiver 2000":
                  - img
            - generic:
              - generic:
                - generic: "3000"
                - generic: Ventes CI
                - generic: Produit
              - generic:
                - button "Modifier 3000":
                  - img
                - button "Archiver 3000":
                  - img
            - generic:
              - generic:
                - generic: "4000"
                - generic: Charges CI
                - generic: Charge
              - generic:
                - button "Modifier 4000":
                  - img
                - button "Archiver 4000":
                  - img
          - paragraph: 5 comptes
      - contentinfo: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
    - region "Notifications alt+T"
  - dialog "Modifier le compte 1000" [ref=e2]:
    - generic [ref=e3]:
      - heading "Modifier le compte 1000" [level=2] [ref=e4]
      - generic [ref=e5]: Le numéro n'est pas modifiable après création.
    - generic [ref=e6]:
      - generic [ref=e7]:
        - text: Numéro
        - textbox "Numéro" [disabled]: "1000"
      - generic [ref=e8]:
        - text: Nom
        - textbox "Nom" [active] [ref=e9]: Caisse CI
      - generic [ref=e10]:
        - text: Type
        - button "Type" [ref=e11]:
          - text: Actif
          - img
      - generic [ref=e12]:
        - button "Annuler" [ref=e13]:
          - button "Annuler" [ref=e14]
        - button "Enregistrer" [ref=e15]
    - button "Close" [ref=e16]:
      - img
      - generic [ref=e17]: Close
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
  80  | 		await expect(page.getByText(testNumber)).toBeVisible();
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
> 99  | 		await page.getByRole('button', { name: 'Annuler' }).click();
      |                                                       ^ Error: locator.click: Error: strict mode violation: getByRole('button', { name: 'Annuler' }) resolved to 2 elements:
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