# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: products.spec.ts >> Contact — conditions de paiement (Story 4.2 T1) >> le champ conditions de paiement persiste après création+édition
- Location: tests/e2e/products.spec.ts:254:2

# Error details

```
Error: expect(locator).toHaveValue(expected) failed

Locator:  locator('#form-payment-terms')
Expected: "30 jours net"
Received: ""
Timeout:  5000ms

Call log:
  - Expect "toHaveValue" with timeout 5000ms
  - waiting for locator('#form-payment-terms')
    9 × locator resolved to <input type="text" maxlength="100" data-slot="input" id="form-payment-terms" placeholder="ex: 30 jours net" class="dark:bg-input/30 border-input focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 disabled:bg-input/50 dark:disabled:bg-input/80 h-8 rounded-lg border bg-transparent px-2.5 py-1 text-base transition-colors file:h-6 file:text-sm file:font-medium f…/>
      - unexpected value ""

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
            - generic:
              - heading "Carnet d'adresses" [level=1]
              - button "Nouveau contact":
                - img
                - text: Nouveau contact
            - generic:
              - generic:
                - generic: Rechercher par nom ou email…
                - generic:
                  - img
                  - searchbox "Rechercher par nom ou email…"
              - generic:
                - generic: Type
                - combobox "Type":
                  - option "Tous les types" [selected]
                  - option "Personne"
                  - option "Entreprise"
              - generic:
                - generic: Client
                - combobox "Client":
                  - option "—" [selected]
                  - option "Client"
                  - option "Non"
              - generic:
                - generic: Fournisseur
                - combobox "Fournisseur":
                  - option "—" [selected]
                  - option "Fournisseur"
                  - option "Non"
              - generic:
                - checkbox "Inclure archivés"
                - text: Inclure archivés
              - button "Réinitialiser"
            - generic:
              - table:
                - rowgroup:
                  - row "Nom tri ascendant Type Rôles IDE Email Actions":
                    - columnheader "Nom tri ascendant":
                      - button "Nom tri ascendant": Nom ↑
                    - columnheader "Type"
                    - columnheader "Rôles"
                    - columnheader "IDE"
                    - columnheader "Email"
                    - columnheader "Actions"
                - rowgroup:
                  - row "TestContact PT 177669784662730 jours net Entreprise Client Modifier Archiver":
                    - cell "TestContact PT 177669784662730 jours net"
                    - cell "Entreprise"
                    - cell "Client":
                      - generic:
                        - generic: Client
                    - cell
                    - cell
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
    - region "Notifications alt+T"
    - generic: untitled page
  - dialog "Modifier le contact" [ref=e2]:
    - heading "Modifier le contact" [level=2] [ref=e4]
    - generic [ref=e5]:
      - generic [ref=e6]:
        - text: Nom / Raison sociale *
        - textbox "Nom / Raison sociale *" [active] [ref=e7]: TestContact PT 177669784662730 jours net
      - generic [ref=e8]:
        - text: Type
        - combobox "Type" [ref=e9]:
          - option "Personne"
          - option "Entreprise" [selected]
      - generic [ref=e10]:
        - generic [ref=e11]:
          - checkbox "Client" [checked] [ref=e12]
          - text: Client
        - generic [ref=e13]:
          - checkbox "Fournisseur" [ref=e14]
          - text: Fournisseur
      - generic [ref=e15]:
        - text: Email
        - textbox "Email" [ref=e16]
      - generic [ref=e17]:
        - text: Téléphone
        - textbox "Téléphone" [ref=e18]
      - generic [ref=e19]:
        - text: Adresse
        - textbox "Adresse" [ref=e20]
      - generic [ref=e21]:
        - text: Numéro IDE (CHE)
        - textbox "Numéro IDE (CHE)" [ref=e22]:
          - /placeholder: CHE-123.456.789
        - paragraph [ref=e23]: "Format : CHE-123.456.789"
      - generic [ref=e24]:
        - text: Conditions de paiement
        - textbox "Conditions de paiement" [ref=e25]:
          - /placeholder: "ex: 30 jours net"
      - generic [ref=e26]:
        - button "Annuler" [ref=e27]
        - button "Enregistrer" [ref=e28]
    - button "Close" [ref=e29]:
      - img
      - generic [ref=e30]: Close
```

# Test source

```ts
  170 | 
  171 | 		await page.getByRole('button', { name: /Nouveau produit/ }).click();
  172 | 		await page.fill('#form-name', name);
  173 | 		await page.fill('#form-price', '7.00');
  174 | 		await page.getByRole('button', { name: 'Créer' }).click();
  175 | 		await expect(page.getByText(/existe déjà|already exists/i)).toBeVisible({ timeout: 5000 });
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
> 270 | 		await expect(page.locator('#form-payment-terms')).toHaveValue('30 jours net');
      |                                                     ^ Error: expect(locator).toHaveValue(expected) failed
  271 | 		await page.getByRole('button', { name: 'Annuler' }).click();
  272 | 
  273 | 		// Cleanup.
  274 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  275 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  276 | 	});
  277 | });
  278 | 
```