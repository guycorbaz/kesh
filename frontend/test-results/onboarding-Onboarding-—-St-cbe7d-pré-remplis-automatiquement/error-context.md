# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: onboarding.spec.ts >> Onboarding — Story 2.6: Invoice Settings Pre-fill (AC 5-6) >> AC 5: Path A (démo) — comptes de facturation pré-remplis automatiquement
- Location: tests/e2e/onboarding.spec.ts:119:2

# Error details

```
Error: expect(locator).not.toBeVisible() failed

Locator:  locator('form').locator('text=Configuration incomplète')
Expected: not visible
Received: visible
Timeout:  5000ms

Call log:
  - Expect "not toBeVisible" with timeout 5000ms
  - waiting for locator('form').locator('text=Configuration incomplète')
    9 × locator resolved to <div class="rounded-md border border-warning bg-warning/10 px-3 py-2 text-sm text-warning">…</div>
      - unexpected value "visible"

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
    - status [ref=e14]:
      - generic [ref=e15]: Instance de démonstration — données fictives
      - button "Réinitialiser pour la production" [ref=e16]
    - generic [ref=e17]:
      - navigation "Navigation principale" [ref=e18]:
        - generic [ref=e19]: Quotidien
        - list [ref=e20]:
          - listitem [ref=e21]:
            - link "Accueil" [ref=e22] [cursor=pointer]:
              - /url: /
          - listitem [ref=e23]:
            - link "Carnet d'adresses" [ref=e24] [cursor=pointer]:
              - /url: /contacts
          - listitem [ref=e25]:
            - link "Catalogue" [ref=e26] [cursor=pointer]:
              - /url: /products
          - listitem [ref=e27]:
            - link "Facturer" [ref=e28] [cursor=pointer]:
              - /url: /invoices
          - listitem [ref=e29]:
            - link "Échéancier" [ref=e30] [cursor=pointer]:
              - /url: /invoices/due-dates
          - listitem [ref=e31]:
            - link "Payer" [ref=e32] [cursor=pointer]:
              - /url: /bank-accounts
          - listitem [ref=e33]:
            - link "Importer" [ref=e34] [cursor=pointer]:
              - /url: /bank-import
        - separator [ref=e35]
        - generic [ref=e36]: Mensuel
        - list [ref=e37]:
          - listitem [ref=e38]:
            - link "Écritures" [ref=e39] [cursor=pointer]:
              - /url: /journal-entries
          - listitem [ref=e40]:
            - link "Réconciliation" [ref=e41] [cursor=pointer]:
              - /url: /reconciliation
          - listitem [ref=e42]:
            - link "Rapports" [ref=e43] [cursor=pointer]:
              - /url: /reports
        - separator [ref=e44]
        - list [ref=e45]:
          - listitem [ref=e46]:
            - link "Paramètres" [ref=e47] [cursor=pointer]:
              - /url: /settings
        - separator [ref=e48]
        - generic [ref=e49]: Administration
        - list [ref=e50]:
          - listitem [ref=e51]:
            - link "Utilisateurs" [ref=e52] [cursor=pointer]:
              - /url: /users
          - listitem [ref=e53]:
            - link "Facturation" [ref=e54] [cursor=pointer]:
              - /url: /settings/invoicing
      - main [ref=e55]:
        - heading "Nouvelle facture" [level=1] [ref=e56]
        - generic [ref=e57]:
          - generic [ref=e58]:
            - text: Configuration incomplète —
            - link "Configurez les comptes de facturation" [ref=e59] [cursor=pointer]:
              - /url: /settings/invoicing
          - generic [ref=e60]:
            - generic [ref=e61]:
              - generic [ref=e62]: Contact
              - combobox "Rechercher un contact…" [ref=e64]
            - generic [ref=e65]:
              - generic [ref=e66]: Date
              - textbox "Date" [ref=e67]: 2026-04-23
            - generic [ref=e68]:
              - generic [ref=e69]: Échéance
              - textbox "Échéance" [ref=e70]
            - generic [ref=e71]:
              - generic [ref=e72]: Conditions de paiement
              - textbox "Conditions de paiement" [ref=e73]:
                - /placeholder: "ex: 30 jours net"
          - generic [ref=e74]:
            - generic [ref=e75]:
              - heading "Lignes" [level=3] [ref=e76]
              - generic [ref=e77]:
                - button "Ligne libre" [ref=e78]:
                  - img
                  - text: Ligne libre
                - button "Depuis catalogue" [ref=e79]:
                  - img
                  - text: Depuis catalogue
            - table [ref=e80]:
              - rowgroup [ref=e81]:
                - row "Description Quantité Prix unitaire TVA % Total" [ref=e82]:
                  - columnheader "Description" [ref=e83]
                  - columnheader "Quantité" [ref=e84]
                  - columnheader "Prix unitaire" [ref=e85]
                  - columnheader "TVA %" [ref=e86]
                  - columnheader "Total" [ref=e87]
                  - columnheader [ref=e88]
              - rowgroup [ref=e89]:
                - row "1 0.00 8.10% 0.00 Supprimer la ligne" [ref=e90]:
                  - cell [ref=e91]:
                    - textbox [ref=e92]
                  - cell "1" [ref=e93]:
                    - textbox [ref=e94]: "1"
                  - cell "0.00" [ref=e95]:
                    - textbox [ref=e96]: "0.00"
                  - cell "8.10%" [ref=e97]:
                    - combobox [ref=e98]:
                      - option "0.00%"
                      - option "2.60%"
                      - option "3.80%"
                      - option "8.10%" [selected]
                  - cell "0.00" [ref=e99]
                  - cell "Supprimer la ligne" [ref=e100]:
                    - button "Supprimer la ligne" [disabled]:
                      - img
              - rowgroup [ref=e101]:
                - row "Total 0.00" [ref=e102]:
                  - cell "Total" [ref=e103]
                  - cell "0.00" [ref=e104]
                  - cell [ref=e105]
          - generic [ref=e106]:
            - button "Annuler" [ref=e107]
            - button "Créer la facture" [disabled]
    - contentinfo [ref=e108]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
  - region "Notifications alt+T"
```

# Test source

```ts
  50  | 		await page.click('button:has-text("Explorer avec des données de démo")');
  51  | 
  52  | 		// Redirect vers / avec bannière démo
  53  | 		await expect(page).toHaveURL('/');
  54  | 		await expect(page.getByText('Instance de démonstration')).toBeVisible();
  55  | 	});
  56  | 
  57  | 	test('bannière démo — reset redirige vers onboarding', async ({ page }) => {
  58  | 		// Complete onboarding first
  59  | 		await page.click('button:has-text("Français")');
  60  | 		await page.click('button:has-text("Guidé")');
  61  | 		await page.click('button:has-text("Explorer avec des données de démo")');
  62  | 		await expect(page).toHaveURL('/');
  63  | 
  64  | 		// Click reset
  65  | 		await page.click('button:has-text("Réinitialiser pour la production")');
  66  | 
  67  | 		// Confirm dialog
  68  | 		await expect(page.getByText('Toutes les données de démonstration')).toBeVisible();
  69  | 		await page.click('button:has-text("Confirmer")');
  70  | 
  71  | 		// Should redirect to onboarding
  72  | 		await expect(page).toHaveURL(/\/onboarding/);
  73  | 	});
  74  | });
  75  | 
  76  | test.describe('Onboarding — Reprise après interruption', () => {
  77  | 	test('F5 à étape 2 reprend à étape 2', async ({ page }) => {
  78  | 		// Login
  79  | 		await page.goto('/login');
  80  | 		await page.fill('#username', 'admin');
  81  | 		await page.fill('#password', 'changeme');
  82  | 		await page.click('button[type="submit"]');
  83  | 		await expect(page).toHaveURL(/\/onboarding/);
  84  | 
  85  | 		// Complete step 1
  86  | 		await page.click('button:has-text("Français")');
  87  | 
  88  | 		// Should be at step 2 now
  89  | 		await expect(page.getByText('Guidé')).toBeVisible();
  90  | 
  91  | 		// Simulate refresh (F5)
  92  | 		await page.reload();
  93  | 
  94  | 		// Should still be at step 2
  95  | 		await expect(page.getByText('Guidé')).toBeVisible();
  96  | 	});
  97  | });
  98  | 
  99  | test.describe('Onboarding — Story 2.6: Invoice Settings Pre-fill (AC 5-6)', () => {
  100 | 	test.beforeEach(async ({ page }) => {
  101 | 		// Reset DB + user `changeme` seul (preset fresh)
  102 | 		await seedTestState('fresh');
  103 | 
  104 | 		// Login en tant que changeme/changeme
  105 | 		await page.goto('/login');
  106 | 		await page.fill('#username', 'changeme');
  107 | 		await page.fill('#password', 'changeme');
  108 | 		await page.click('button[type="submit"]');
  109 | 
  110 | 		// Le guard onboarding devrait rediriger vers /onboarding
  111 | 		await expect(page).toHaveURL(/\/onboarding/);
  112 | 	});
  113 | 
  114 | 	test.afterEach(async ({ page }) => {
  115 | 		// Clear localStorage after each test
  116 | 		await clearAuthStorage(page);
  117 | 	});
  118 | 
  119 | 	test('AC 5: Path A (démo) — comptes de facturation pré-remplis automatiquement', async ({ page }) => {
  120 | 		// Step 1: Choisir français
  121 | 		await page.click('button:has-text("Français")');
  122 | 
  123 | 		// Step 2: Mode guidé
  124 | 		await page.click('button:has-text("Guidé")');
  125 | 
  126 | 		// Step 3: Chemin démo
  127 | 		await page.click('button:has-text("Explorer avec des données de démo")');
  128 | 
  129 | 		// Attendre la redirection vers /
  130 | 		await expect(page).toHaveURL('/');
  131 | 
  132 | 		// Vérifier que la bannière de démo est visible
  133 | 		await expect(page.getByText('Instance de démonstration')).toBeVisible();
  134 | 
  135 | 		// Naviger vers creation de facture
  136 | 		await page.goto('/invoices/new');
  137 | 
  138 | 		// Vérifier que le formulaire de création est accessible
  139 | 		await expect(page.locator('label:has-text("Contact")')).toBeVisible();
  140 | 
  141 | 		// Vérifier que la bannière d'avertissement n'est PAS visible
  142 | 		// (car les comptes sont pré-remplis en mode démo)
  143 | 		const warningBanner = page.locator('text=Configuration incomplète');
  144 | 		// NOTE: Si la bannière est visible, cela signifie que la pré-remplissage a échoué
  145 | 		// Nous ne pouvons pas utiliser toBeVisible() car le selecteur pourrait trouver d'autres textes
  146 | 		const count = await warningBanner.count();
  147 | 		if (count > 0) {
  148 | 			// Si la bannière existe, elle ne doit pas être dans le formulaire de création
  149 | 			const formWarning = page.locator('form >> text=Configuration incomplète');
> 150 | 			await expect(formWarning).not.toBeVisible();
      |                                  ^ Error: expect(locator).not.toBeVisible() failed
  151 | 		}
  152 | 
  153 | 		// Vérifier que le bouton Créer la facture est activé
  154 | 		const createBtn = page.locator('button:has-text("Créer la facture")');
  155 | 		await expect(createBtn).toBeEnabled();
  156 | 	});
  157 | 
  158 | 	test('AC 6: Path B (production) — comptes de facturation pré-remplis après onboarding', async ({ page }) => {
  159 | 		// Step 1: Language
  160 | 		await page.click('button:has-text("Français")');
  161 | 
  162 | 		// Step 2: Mode
  163 | 		await page.click('button:has-text("Guidé")');
  164 | 
  165 | 		// Step 3: Production path
  166 | 		await page.click('button:has-text("Configurer pour la production")');
  167 | 
  168 | 		// Step 4: Org type — Indépendant
  169 | 		await page.click('button:has-text("Indépendant")');
  170 | 
  171 | 		// Step 5: Accounting language
  172 | 		await page.click('button:has-text("Français")');
  173 | 
  174 | 		// Step 6: Coordinates
  175 | 		await page.fill('#coord-name', 'Mon Business Indépendant');
  176 | 		await page.fill('#coord-address', 'Rue des Alpes 1, 1200 Genève');
  177 | 		await page.click('button:has-text("Continuer")');
  178 | 
  179 | 		// Step 7: Bank account (skip for now)
  180 | 		await page.click('button:has-text("Configurer plus tard")');
  181 | 
  182 | 		// Attendre la redirection vers /
  183 | 		await expect(page).toHaveURL('/');
  184 | 
  185 | 		// Naviguer vers creation de facture
  186 | 		// Per AC 6: comptes de facturation pré-remplis → pas besoin de config supplémentaire
  187 | 		await page.goto('/invoices/new');
  188 | 
  189 | 		// Vérifier que le formulaire de création est accessible
  190 | 		await expect(page.locator('label:has-text("Contact")')).toBeVisible();
  191 | 
  192 | 		// Vérifier que le bouton Créer la facture est activé
  193 | 		// (car les comptes de facturation ont été pré-remplis)
  194 | 		const createBtn = page.locator('button:has-text("Créer la facture")');
  195 | 		await expect(createBtn).toBeEnabled();
  196 | 
  197 | 		// Vérifier que la bannière de configuration des comptes
  198 | 		// ne s'affiche PAS (car ils sont pré-remplis)
  199 | 		// Note: Nous recherchons spécifiquement dans le formulaire
  200 | 		const formWarning = page.locator('form >> div:has-text("Configurez les comptes de facturation")');
  201 | 		await expect(formWarning).not.toBeVisible();
  202 | 	});
  203 | });
  204 | 
```