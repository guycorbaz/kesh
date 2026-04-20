# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: contacts.spec.ts >> Page contacts — accessibilité >> axe-core sans violations sur la liste contacts
- Location: tests/e2e/contacts.spec.ts:150:2

# Error details

```
Error: expect(received).toEqual(expected) // deep equality

- Expected  -   1
+ Received  + 146

- Array []
+ Array [
+   Object {
+     "description": "Ensure the contrast between foreground and background colors meets WCAG 2 AA minimum contrast ratio thresholds",
+     "help": "Elements must meet minimum color contrast ratio thresholds",
+     "helpUrl": "https://dequeuniversity.com/rules/axe/4.11/color-contrast?application=playwright",
+     "id": "color-contrast",
+     "impact": "serious",
+     "nodes": Array [
+       Object {
+         "all": Array [],
+         "any": Array [
+           Object {
+             "data": Object {
+               "bgColor": "#1e40af",
+               "contrastRatio": 1.67,
+               "expectedContrastRatio": "4.5:1",
+               "fgColor": "#1e293b",
+               "fontSize": "10.5pt (14px)",
+               "fontWeight": "normal",
+               "messageKey": null,
+             },
+             "id": "color-contrast",
+             "impact": "serious",
+             "message": "Element has insufficient color contrast of 1.67 (foreground color: #1e293b, background color: #1e40af, font size: 10.5pt (14px), font weight: normal). Expected contrast ratio of 4.5:1",
+             "relatedNodes": Array [
+               Object {
+                 "html": "<button data-slot=\"button\" class=\"focus-visible:border...\" type=\"button\">",
+                 "target": Array [
+                   ".bg-primary",
+                 ],
+               },
+             ],
+           },
+         ],
+         "failureSummary": "Fix any of the following:
+   Element has insufficient color contrast of 1.67 (foreground color: #1e293b, background color: #1e40af, font size: 10.5pt (14px), font weight: normal). Expected contrast ratio of 4.5:1",
+         "html": "<button data-slot=\"button\" class=\"focus-visible:border...\" type=\"button\">",
+         "impact": "serious",
+         "none": Array [],
+         "target": Array [
+           ".bg-primary",
+         ],
+       },
+     ],
+     "tags": Array [
+       "cat.color",
+       "wcag2aa",
+       "wcag143",
+       "TTv5",
+       "TT13.c",
+       "EN-301-549",
+       "EN-9.1.4.3",
+       "ACT",
+       "RGAAv4",
+       "RGAA-3.2.1",
+     ],
+   },
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
+         "html": "<html lang=\"fr\" data-mode=\"guided\">",
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
+                   ".dark\\:hover\\:bg-muted\\/50",
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
        - generic [ref=e53]:
          - generic [ref=e54]:
            - heading "Carnet d'adresses" [level=1] [ref=e55]
            - button "Nouveau contact" [ref=e56]:
              - img
              - text: Nouveau contact
          - generic [ref=e57]:
            - generic [ref=e58]:
              - generic [ref=e59]: Rechercher par nom ou email…
              - generic [ref=e60]:
                - img [ref=e61]
                - searchbox "Rechercher par nom ou email…" [ref=e62]
            - generic [ref=e63]:
              - generic [ref=e64]: Type
              - combobox "Type" [ref=e65]:
                - option "Tous les types" [selected]
                - option "Personne"
                - option "Entreprise"
            - generic [ref=e66]:
              - generic [ref=e67]: Client
              - combobox "Client" [ref=e68]:
                - option "—" [selected]
                - option "Client"
                - option "Non"
            - generic [ref=e69]:
              - generic [ref=e70]: Fournisseur
              - combobox "Fournisseur" [ref=e71]:
                - option "—" [selected]
                - option "Fournisseur"
                - option "Non"
            - generic [ref=e72]:
              - checkbox "Inclure archivés" [ref=e73]
              - text: Inclure archivés
            - button "Réinitialiser" [ref=e74]
          - table [ref=e76]:
            - rowgroup [ref=e77]:
              - row "Nom tri ascendant Type Rôles IDE Email Actions" [ref=e78]:
                - columnheader "Nom tri ascendant" [ref=e79]:
                  - button "Nom tri ascendant" [ref=e80]: Nom ↑
                - columnheader "Type" [ref=e81]
                - columnheader "Rôles" [ref=e82]
                - columnheader "IDE" [ref=e83]
                - columnheader "Email" [ref=e84]
                - columnheader "Actions" [ref=e85]
            - rowgroup [ref=e86]:
              - row "Aucun contact. Créez votre premier contact avec le bouton « Nouveau contact »." [ref=e87]:
                - cell "Aucun contact. Créez votre premier contact avec le bouton « Nouveau contact »." [ref=e88]
          - generic [ref=e89]:
            - generic [ref=e90]: 0-0 sur 0
            - generic [ref=e91]:
              - button "Précédent" [disabled]
              - button "Suivant" [disabled]
    - contentinfo [ref=e92]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T"
  - generic [ref=e93]: untitled page
```

# Test source

```ts
  54  | 
  55  | 		await page.fill('#form-name', uniqueName);
  56  | 		await page.fill('#form-email', 'test@example.ch');
  57  | 		await page.getByRole('button', { name: 'Créer' }).click();
  58  | 
  59  | 		// Le nouveau contact apparaît dans la liste.
  60  | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  61  | 
  62  | 		// Cleanup : archiver le contact pour éviter de polluer les tests suivants.
  63  | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  64  | 		await row.getByRole('button', { name: /Archiver/ }).click();
  65  | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  66  | 		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
  67  | 	});
  68  | 
  69  | 	test("validation IDE invalide affiche un message d'erreur", async ({ page }) => {
  70  | 		await goToContacts(page);
  71  | 
  72  | 		await page.getByRole('button', { name: /Nouveau contact/ }).click();
  73  | 
  74  | 		const uniqueName = `TestContact IDE ${Date.now()}`;
  75  | 		await page.fill('#form-name', uniqueName);
  76  | 		// CHE-109.322.552 = checksum invalide (dernier chiffre décalé).
  77  | 		// CHE-000.000.000 est VALIDE (checksum 0 modulo 11) — ne PAS l'utiliser ici.
  78  | 		await page.fill('#form-ide', 'CHE-109.322.552');
  79  | 
  80  | 		// La validation client-side accepte le format, le bouton est actif.
  81  | 		const submitBtn = page.getByRole('button', { name: 'Créer' });
  82  | 		await submitBtn.click();
  83  | 
  84  | 		// Le backend rejette avec message d'erreur (toast ou inline).
  85  | 		// Le message peut venir de notifyError (toast) ou du formError inline.
  86  | 		await expect(
  87  | 			page.getByText(/IDE|invalid/i).first()
  88  | 		).toBeVisible({ timeout: 5000 });
  89  | 	});
  90  | 
  91  | 	test('archivage avec confirmation et disparition de la liste', async ({ page }) => {
  92  | 		await goToContacts(page);
  93  | 
  94  | 		// Créer un contact ad-hoc.
  95  | 		const uniqueName = `TestContact Archive ${Date.now()}`;
  96  | 		await page.getByRole('button', { name: /Nouveau contact/ }).click();
  97  | 		await page.fill('#form-name', uniqueName);
  98  | 		await page.getByRole('button', { name: 'Créer' }).click();
  99  | 		await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 5000 });
  100 | 
  101 | 		// Archiver.
  102 | 		const row = page.locator('tr', { hasText: uniqueName }).first();
  103 | 		await row.getByRole('button', { name: /Archiver/ }).click();
  104 | 
  105 | 		// Confirmation dialog.
  106 | 		await expect(page.getByRole('dialog').getByText(/Archiver le contact/)).toBeVisible();
  107 | 		await page.getByRole('dialog').getByRole('button', { name: 'Archiver' }).click();
  108 | 
  109 | 		// Le contact disparaît de la liste par défaut.
  110 | 		await expect(page.getByText(uniqueName)).toHaveCount(0, { timeout: 5000 });
  111 | 	});
  112 | 
  113 | 	test('filtre par type Entreprise', async ({ page }) => {
  114 | 		await goToContacts(page);
  115 | 
  116 | 		// Sélectionner le filtre type = Entreprise.
  117 | 		await page.locator('#filter-type').selectOption('Entreprise');
  118 | 
  119 | 		// Attendre le rechargement (debounce 300ms pour search, mais filtres type
  120 | 		// déclenchent immédiatement via $effect).
  121 | 		await page.waitForTimeout(500);
  122 | 
  123 | 		// URL reflète le filtre.
  124 | 		await expect(page).toHaveURL(/contactType=Entreprise/);
  125 | 	});
  126 | 
  127 | 	test('URL state préservé après reload', async ({ page }) => {
  128 | 		await goToContacts(page);
  129 | 
  130 | 		// Appliquer un filtre.
  131 | 		await page.locator('#filter-type').selectOption('Entreprise');
  132 | 		await page.waitForTimeout(500);
  133 | 		expect(page.url()).toContain('contactType=Entreprise');
  134 | 
  135 | 		// Reload.
  136 | 		await page.reload();
  137 | 		await page.waitForTimeout(500);
  138 | 
  139 | 		// Le filtre est restauré.
  140 | 		const selectedValue = await page.locator('#filter-type').inputValue();
  141 | 		expect(selectedValue).toBe('Entreprise');
  142 | 	});
  143 | 
  144 | 	// Reportés à Story 4.2 ou post-MVP.
  145 | 	test.skip('filtre combinés (type + client + search)', async () => {});
  146 | 	test.skip('pagination navigation précédent/suivant', async () => {});
  147 | });
  148 | 
  149 | test.describe('Page contacts — accessibilité', () => {
  150 | 	test('axe-core sans violations sur la liste contacts', async ({ page }) => {
  151 | 		await goToContacts(page);
  152 | 		await page.waitForLoadState('networkidle');
  153 | 		const results = await new AxeBuilder({ page }).analyze();
> 154 | 		expect(results.violations).toEqual([]);
      |                              ^ Error: expect(received).toEqual(expected) // deep equality
  155 | 	});
  156 | });
  157 | 
```