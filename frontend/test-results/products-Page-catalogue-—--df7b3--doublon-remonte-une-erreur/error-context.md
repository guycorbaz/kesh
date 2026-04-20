# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: products.spec.ts >> Page catalogue — validation & erreurs >> création d'un nom en doublon remonte une erreur
- Location: tests/e2e/products.spec.ts:166:2

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByText(/existe déjà|already exists/i)
Expected: visible
Error: strict mode violation: getByText(/existe déjà|already exists/i) resolved to 2 elements:
    1) <div data-title="">…</div> aka getByLabel('Notifications alt+T').getByText('Un produit avec ce nom existe')
    2) <p class="text-sm text-destructive">Un produit avec ce nom existe déjà</p> aka getByLabel('Nouveau produit').getByText('Un produit avec ce nom existe')

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByText(/existe déjà|already exists/i)

```

# Page snapshot

```yaml
- generic [active]:
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
            - generic:
              - heading "Catalogue produits/services" [level=1]
              - button "Nouveau produit":
                - img
                - text: Nouveau produit
            - generic:
              - generic:
                - generic: Rechercher par nom ou description…
                - generic:
                  - img
                  - searchbox "Rechercher par nom ou description…"
              - generic:
                - checkbox "Inclure archivés"
                - text: Inclure archivés
              - button "Réinitialiser"
            - generic:
              - table:
                - rowgroup:
                  - row "Nom ↑ Description Prix TVA Actions":
                    - columnheader "Nom ↑":
                      - button "Nom ↑"
                    - columnheader "Description"
                    - columnheader "Prix":
                      - button "Prix"
                    - columnheader "TVA":
                      - button "TVA"
                    - columnheader "Actions"
                - rowgroup:
                  - row "TestProduct Dup 1776697839327-546551 5.00 8.10% Modifier Archiver":
                    - cell "TestProduct Dup 1776697839327-546551"
                    - cell
                    - cell "5.00"
                    - cell "8.10%"
                    - cell "Modifier Archiver":
                      - button "Modifier":
                        - img
                      - button "Archiver":
                        - img
            - generic:
              - generic: 1-1 sur 1
              - generic:
                - button "Précédent" [disabled]
                - button "Suivant" [disabled]
      - contentinfo: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
    - region "Notifications alt+T":
      - list:
        - listitem:
          - generic:
            - img
          - generic:
            - generic: Un produit avec ce nom existe déjà
        - listitem:
          - generic:
            - img
          - generic:
            - generic: Produit créé
    - generic: untitled page
  - dialog "Nouveau produit" [ref=e2]:
    - heading "Nouveau produit" [level=2] [ref=e4]
    - generic [ref=e5]:
      - generic [ref=e6]:
        - text: Nom *
        - textbox "Nom *" [ref=e7]: TestProduct Dup 1776697839327-546551
      - generic [ref=e8]:
        - text: Description
        - textbox "Description" [ref=e9]
      - generic [ref=e10]:
        - text: Prix unitaire *
        - textbox "Prix unitaire *" [ref=e11]:
          - /placeholder: "0.00"
          - text: "7.00"
      - generic [ref=e12]:
        - text: Taux TVA *
        - combobox "Taux TVA *" [ref=e13]:
          - option "8.10 % — Taux normal" [selected]
          - option "3.80 % — Hébergement"
          - option "2.60 % — Taux réduit"
          - option "0.00 % — Exonéré"
        - paragraph [ref=e14]: Taux suisses en vigueur depuis le 01.01.2024
      - paragraph [ref=e15]: Un produit avec ce nom existe déjà
      - generic [ref=e16]:
        - button "Annuler" [ref=e17]
        - button "Créer" [ref=e18]
    - button "Close" [ref=e19]:
      - img
      - generic [ref=e20]: Close
```

# Test source

```ts
  75  | });
  76  | 
  77  | test.describe('Page catalogue — accessibilité', () => {
  78  | 	test('axe-core sans violations sur la liste produits', async ({ page }) => {
  79  | 		await goToProducts(page);
  80  | 		await page.waitForLoadState('networkidle');
  81  | 		const results = await new AxeBuilder({ page }).analyze();
  82  | 		expect(results.violations).toEqual([]);
  83  | 	});
  84  | });
  85  | 
  86  | test.describe('Page catalogue — CRUD', () => {
  87  | 	test('création nominale d\'un produit', async ({ page }) => {
  88  | 		await goToProducts(page);
  89  | 
  90  | 		const uniqueName = `TestProduct E2E ${Date.now()}`;
  91  | 
  92  | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  93  | 		await expect(page.getByRole('heading', { name: /Nouveau produit/ })).toBeVisible();
  94  | 
  95  | 		await page.fill('#form-name', uniqueName);
  96  | 		await page.fill('#form-price', '1500.00');
  97  | 		// Taux TVA 8.10 % est sélectionné par défaut.
  98  | 		await page.getByRole('button', { name: 'Créer' }).click();
  99  | 
  100 | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  101 | 
  102 | 		// Cleanup : archiver pour ne pas polluer les tests suivants.
  103 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  104 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  105 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  106 | 		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
  107 | 	});
  108 | 
  109 | 	test('archivage avec confirmation et disparition de la liste', async ({ page }) => {
  110 | 		await goToProducts(page);
  111 | 
  112 | 		const uniqueName = `TestProduct Arch ${Date.now()}`;
  113 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  114 | 		await page.fill('#form-name', uniqueName);
  115 | 		await page.fill('#form-price', '42.00');
  116 | 		await page.getByRole('button', { name: 'Créer' }).click();
  117 | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  118 | 
  119 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  120 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  121 | 
  122 | 		await expect(page.getByRole('dialog').getByText(/Archiver le produit/)).toBeVisible();
  123 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  124 | 
  125 | 		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
  126 | 	});
  127 | 
  128 | 	test('filtre recherche reflété dans URL et résultats', async ({ page }) => {
  129 | 		await goToProducts(page);
  130 | 
  131 | 		const uniqueName = `TestProduct Search ${Date.now()}`;
  132 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  133 | 		await page.fill('#form-name', uniqueName);
  134 | 		await page.fill('#form-price', '10.00');
  135 | 		await page.getByRole('button', { name: 'Créer' }).click();
  136 | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  137 | 
  138 | 		// Rechercher par nom unique → URL contient `search=` (attente event-driven
  139 | 		// plutôt que `waitForTimeout`, pour éviter les flakes CI sur machines lentes).
  140 | 		await page.fill('#filter-search', uniqueName);
  141 | 		await page.waitForURL(/search=/, { timeout: 2000 });
  142 | 		await expect(page.locator('tbody').getByText(uniqueName)).toBeVisible();
  143 | 
  144 | 		// Cleanup.
  145 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  146 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  147 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  148 | 	});
  149 | });
  150 | 
  151 | test.describe('Page catalogue — validation & erreurs', () => {
  152 | 	test('format prix invalide affiche un message inline et désactive Créer', async ({ page }) => {
  153 | 		await goToProducts(page);
  154 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  155 | 		await page.fill('#form-name', uniq('TestProduct Invalid'));
  156 | 		await page.fill('#form-price', '10.123456'); // > 4 décimales
  157 | 		// Feedback inline visible sans avoir cliqué sur Créer.
  158 | 		await expect(page.getByText(/prix invalide/i)).toBeVisible();
  159 | 		await expect(page.getByRole('button', { name: 'Créer' })).toBeDisabled();
  160 | 		// Correction → activation.
  161 | 		await page.fill('#form-price', '10.50');
  162 | 		await expect(page.getByRole('button', { name: 'Créer' })).toBeEnabled();
  163 | 		await page.getByRole('button', { name: 'Annuler' }).click();
  164 | 	});
  165 | 
  166 | 	test('création d\'un nom en doublon remonte une erreur', async ({ page }) => {
  167 | 		await goToProducts(page);
  168 | 		const name = uniq('TestProduct Dup');
  169 | 		await createProduct(page, name, '5.00');
  170 | 
  171 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  172 | 		await page.fill('#form-name', name);
  173 | 		await page.fill('#form-price', '7.00');
  174 | 		await page.getByRole('button', { name: 'Créer' }).click();
> 175 | 		await expect(page.getByText(/existe déjà|already exists/i)).toBeVisible({ timeout: 5000 });
      |                                                               ^ Error: expect(locator).toBeVisible() failed
  176 | 		await page.getByRole('button', { name: 'Annuler' }).click();
  177 | 
  178 | 		await archiveRow(page, name);
  179 | 	});
  180 | });
  181 | 
  182 | test.describe('Page catalogue — filtres, tri & pagination', () => {
  183 | 	test('toggle "Inclure archivés" réaffiche un produit archivé', async ({ page }) => {
  184 | 		await goToProducts(page);
  185 | 		const name = uniq('TestProduct Arch Toggle');
  186 | 		await createProduct(page, name, '12.00');
  187 | 
  188 | 		// Archiver.
  189 | 		const row = page.locator('tr', { hasText: name }).first();
  190 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  191 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  192 | 		await expect(page.getByText(name)).toHaveCount(0, { timeout: 5000 });
  193 | 
  194 | 		// Activer toggle → produit ré-apparaît (opacité réduite).
  195 | 		await page.getByLabel(/Inclure archivés|Include archived|Archivierte|archiviati/i).check();
  196 | 		await expect(page.getByText(name)).toBeVisible({ timeout: 5000 });
  197 | 		await expect(page).toHaveURL(/includeArchived=true/);
  198 | 	});
  199 | 
  200 | 	test('tri par nom : clic sur en-tête bascule Asc/Desc et URL', async ({ page }) => {
  201 | 		await goToProducts(page);
  202 | 		const header = page.getByRole('button', { name: /^Nom|^Name|^Nome$/ }).first();
  203 | 		await header.click();
  204 | 		await expect(page).toHaveURL(/sortDirection=Desc/);
  205 | 		await header.click();
  206 | 		await expect(page).not.toHaveURL(/sortDirection=Desc/);
  207 | 	});
  208 | 
  209 | 	test('AC #9 : filtres/tri/pagination restaurés depuis l\'URL après reload', async ({
  210 | 		page
  211 | 	}) => {
  212 | 		await goToProducts(page);
  213 | 		const name = uniq('TestProduct URLState');
  214 | 		await createProduct(page, name, '10.00');
  215 | 
  216 | 		// Applique un filtre recherche + tri Desc → URL doit porter les deux.
  217 | 		await page.fill('#filter-search', name);
  218 | 		await page.waitForURL(/search=/, { timeout: 2000 });
  219 | 		const header = page.getByRole('button', { name: /^Nom|^Name|^Nome$/ }).first();
  220 | 		await header.click();
  221 | 		await page.waitForURL(/sortDirection=Desc/, { timeout: 2000 });
  222 | 
  223 | 		const urlBefore = page.url();
  224 | 
  225 | 		// Reload : l'état doit être reconstruit depuis les query params.
  226 | 		await page.reload();
  227 | 		await expect(page).toHaveURL(urlBefore);
  228 | 		await expect(page.locator('#filter-search')).toHaveValue(name);
  229 | 		await expect(page.locator('tbody').getByText(name)).toBeVisible({ timeout: 5000 });
  230 | 
  231 | 		await archiveRow(page, name);
  232 | 	});
  233 | 
  234 | 	test('sélection taux TVA 2.60 % persiste après édition', async ({ page }) => {
  235 | 		await goToProducts(page);
  236 | 		const name = uniq('TestProduct VAT');
  237 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  238 | 		await page.fill('#form-name', name);
  239 | 		await page.fill('#form-price', '100.00');
  240 | 		await page.locator('#form-vat-rate').selectOption('2.60');
  241 | 		await page.getByRole('button', { name: 'Créer' }).click();
  242 | 		await expect(page.getByText(name)).toBeVisible({ timeout: 5000 });
  243 | 
  244 | 		const row = page.locator('tr', { hasText: name }).first();
  245 | 		await row.getByRole('button', { name: /Modifier/ }).click();
  246 | 		await expect(page.locator('#form-vat-rate')).toHaveValue('2.60');
  247 | 		await page.getByRole('button', { name: 'Annuler' }).click();
  248 | 
  249 | 		await archiveRow(page, name);
  250 | 	});
  251 | });
  252 | 
  253 | test.describe('Contact — conditions de paiement (Story 4.2 T1)', () => {
  254 | 	test('le champ conditions de paiement persiste après création+édition', async ({ page }) => {
  255 | 		await login(page);
  256 | 		await page.goto('/contacts');
  257 | 		await expect(page).toHaveURL(/\/contacts/);
  258 | 
  259 | 		const uniqueName = `TestContact PT ${Date.now()}`;
  260 | 
  261 | 		await page.getByRole('button', { name: /Nouveau contact/ }).click();
  262 | 		await page.fill('#form-name', uniqueName);
  263 | 		await page.fill('#form-payment-terms', '30 jours net');
  264 | 		await page.getByRole('button', { name: 'Créer' }).click();
  265 | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  266 | 
  267 | 		// Rouvrir en édition → la valeur doit être restaurée.
  268 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  269 | 		await row.getByRole('button', { name: /Modifier/ }).click();
  270 | 		await expect(page.locator('#form-payment-terms')).toHaveValue('30 jours net');
  271 | 		await page.getByRole('button', { name: 'Annuler' }).click();
  272 | 
  273 | 		// Cleanup.
  274 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  275 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
```