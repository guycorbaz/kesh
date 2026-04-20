# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: invoices.spec.ts >> Factures — téléchargement PDF (Story 5.3) >> erreur 400 INVOICE_NOT_PDF_READY affichée comme toast
- Location: tests/e2e/invoices.spec.ts:259:2

# Error details

```
Error: createContactWithAddress failed: 401

expect(received).toBeTruthy()

Received: false
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
  87  | 	test('crée une facture avec une ligne libre et la persiste', async ({ page }) => {
  88  | 		await login(page);
  89  | 		const contactName = uniq('Client');
  90  | 		await createContactViaApi(page, contactName);
  91  | 
  92  | 		await page.goto('/invoices/new');
  93  | 		await expect(page.getByRole('heading', { name: 'Nouvelle facture' })).toBeVisible();
  94  | 
  95  | 		// Sélection du contact via le combobox
  96  | 		await page.getByRole('combobox').click();
  97  | 		await page.getByRole('combobox').fill(contactName);
  98  | 		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
  99  | 
  100 | 		// Ligne libre par défaut : remplir description, quantité, prix
  101 | 		const firstRow = page.locator('tbody tr').first();
  102 | 		await firstRow.locator('input[type="text"]').first().fill('Conseil stratégique');
  103 | 		const numericInputs = firstRow.locator('input[inputmode="decimal"]');
  104 | 		await numericInputs.nth(0).fill('4.5');
  105 | 		await numericInputs.nth(1).fill('200.00');
  106 | 
  107 | 		await page.getByRole('button', { name: 'Créer la facture' }).click();
  108 | 		await expect(page).toHaveURL('/invoices');
  109 | 		await expect(page.locator('tbody').getByText(contactName)).toBeVisible({ timeout: 5000 });
  110 | 	});
  111 | 
  112 | 	test('crée une facture avec 1 ligne libre + 1 ligne catalogue et persiste après reload (AC #1, #2)', async ({
  113 | 		page,
  114 | 	}) => {
  115 | 		await login(page);
  116 | 		const contactName = uniq('ClientCombo');
  117 | 		const productName = uniq('Prod');
  118 | 		await createContactViaApi(page, contactName);
  119 | 		await createProductViaApi(page, productName, '150.00', '8.10');
  120 | 
  121 | 		await page.goto('/invoices/new');
  122 | 
  123 | 		// Contact
  124 | 		await page.getByRole('combobox').click();
  125 | 		await page.getByRole('combobox').fill(contactName);
  126 | 		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
  127 | 
  128 | 		// Ligne libre (celle par défaut)
  129 | 		const firstRow = page.locator('tbody tr').first();
  130 | 		await firstRow.locator('input[type="text"]').first().fill('Prestation libre');
  131 | 		const firstRowNumerics = firstRow.locator('input[inputmode="decimal"]');
  132 | 		await firstRowNumerics.nth(0).fill('2');
  133 | 		await firstRowNumerics.nth(1).fill('100.00');
  134 | 
  135 | 		// Ligne depuis catalogue
  136 | 		await page.getByRole('button', { name: /Depuis catalogue/ }).click();
  137 | 		await expect(page.getByRole('dialog')).toBeVisible();
  138 | 		await page.getByPlaceholder(/Rechercher un produit/).fill(productName);
  139 | 		await page
  140 | 			.getByRole('dialog')
  141 | 			.getByRole('button')
  142 | 			.filter({ hasText: productName })
  143 | 			.first()
  144 | 			.click();
  145 | 
  146 | 		// Le formulaire doit maintenant contenir 2 lignes, la 2e pré-remplie
  147 | 		await expect(page.locator('tbody tr')).toHaveCount(2);
  148 | 		const secondRow = page.locator('tbody tr').nth(1);
  149 | 		await expect(secondRow.locator('input[type="text"]').first()).toHaveValue(productName);
  150 | 		const secondRowNumerics = secondRow.locator('input[inputmode="decimal"]');
  151 | 		await expect(secondRowNumerics.nth(1)).toHaveValue('150.00');
  152 | 
  153 | 		// Soumettre
  154 | 		await page.getByRole('button', { name: 'Créer la facture' }).click();
  155 | 		await expect(page).toHaveURL('/invoices');
  156 | 
  157 | 		// Ouvrir la facture créée pour vérifier la persistance après reload
  158 | 		const row = page.locator('tbody tr', { hasText: contactName }).first();
  159 | 		await row.getByRole('button').first().click();
  160 | 		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();
  161 | 
  162 | 		// Reload dur — l'état doit être identique
  163 | 		await page.reload();
  164 | 		await expect(page.getByText('Prestation libre')).toBeVisible();
  165 | 		await expect(page.getByText(productName)).toBeVisible();
  166 | 	});
  167 | });
  168 | 
  169 | // ---------------------------------------------------------------------------
  170 | // Story 5.3 — Téléchargement PDF QR Bill
  171 | // ---------------------------------------------------------------------------
  172 | 
  173 | async function createContactWithAddressViaApi(
  174 | 	page: import('@playwright/test').Page,
  175 | 	name: string,
  176 | ): Promise<number> {
  177 | 	const res = await page.request.post('/api/v1/contacts', {
  178 | 		data: {
  179 | 			contactType: 'Personne',
  180 | 			name,
  181 | 			isClient: true,
  182 | 			isSupplier: false,
  183 | 			address: 'Marktgasse 28\n9400 Rorschach',
  184 | 			defaultPaymentTerms: '30 jours net',
  185 | 		},
  186 | 	});
> 187 | 	expect(res.ok(), `createContactWithAddress failed: ${res.status()}`).toBeTruthy();
      |                                                                       ^ Error: createContactWithAddress failed: 401
  188 | 	return (await res.json()).id as number;
  189 | }
  190 | 
  191 | async function createAndValidateInvoiceViaApi(
  192 | 	page: import('@playwright/test').Page,
  193 | 	contactId: number,
  194 | ): Promise<number> {
  195 | 	const today = new Date().toISOString().slice(0, 10);
  196 | 	const createRes = await page.request.post('/api/v1/invoices', {
  197 | 		data: {
  198 | 			contactId,
  199 | 			date: today,
  200 | 			dueDate: today,
  201 | 			paymentTerms: '30 jours net',
  202 | 			lines: [
  203 | 				{
  204 | 					description: 'Conseil stratégique',
  205 | 					quantity: '4.5',
  206 | 					unitPrice: '200.00',
  207 | 					vatRate: '7.70',
  208 | 				},
  209 | 			],
  210 | 		},
  211 | 	});
  212 | 	expect(createRes.ok(), `create invoice failed: ${createRes.status()}`).toBeTruthy();
  213 | 	const invoice = await createRes.json();
  214 | 	const validateRes = await page.request.post(`/api/v1/invoices/${invoice.id}/validate`);
  215 | 	expect(validateRes.ok(), `validate failed: ${validateRes.status()}`).toBeTruthy();
  216 | 	return invoice.id as number;
  217 | }
  218 | 
  219 | test.describe('Factures — téléchargement PDF (Story 5.3)', () => {
  220 | 	test('télécharge le PDF d\'une facture validée (golden path)', async ({ page, context }) => {
  221 | 		await login(page);
  222 | 		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Client'));
  223 | 		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
  224 | 
  225 | 		await page.goto(`/invoices/${invoiceId}`);
  226 | 		await expect(page.getByRole('heading', { name: 'Facture' })).toBeVisible();
  227 | 
  228 | 		// Intercepte l'appel direct à l'endpoint PDF (plus robuste que window.open).
  229 | 		const pdfRes = await page.request.get(`/api/v1/invoices/${invoiceId}/pdf`);
  230 | 		expect(pdfRes.status()).toBe(200);
  231 | 		expect(pdfRes.headers()['content-type']).toContain('application/pdf');
  232 | 		const buf = await pdfRes.body();
  233 | 		expect(buf.slice(0, 7).toString('utf8')).toMatch(/^%PDF-1\./);
  234 | 	});
  235 | 
  236 | 	test('bouton visible uniquement si status=validated', async ({ page }) => {
  237 | 		await login(page);
  238 | 		const contactName = uniq('PDF Draft');
  239 | 		await createContactWithAddressViaApi(page, contactName);
  240 | 		// Facture brouillon non validée
  241 | 		await page.goto('/invoices/new');
  242 | 		await page.getByRole('combobox').click();
  243 | 		await page.getByRole('combobox').fill(contactName);
  244 | 		await page.getByRole('option', { name: new RegExp(contactName) }).first().click();
  245 | 		const firstRow = page.locator('tbody tr').first();
  246 | 		await firstRow.locator('input[type="text"]').first().fill('Item');
  247 | 		const inputs = firstRow.locator('input[inputmode="decimal"]');
  248 | 		await inputs.nth(0).fill('1');
  249 | 		await inputs.nth(1).fill('50');
  250 | 		await page.getByRole('button', { name: 'Créer la facture' }).click();
  251 | 		await expect(page).toHaveURL('/invoices');
  252 | 
  253 | 		// Ouvre la facture brouillon → pas de bouton PDF
  254 | 		const row = page.locator('tbody tr', { hasText: contactName }).first();
  255 | 		await row.getByRole('button').first().click();
  256 | 		await expect(page.getByRole('button', { name: /Télécharger PDF/i })).toHaveCount(0);
  257 | 	});
  258 | 
  259 | 	test('erreur 400 INVOICE_NOT_PDF_READY affichée comme toast', async ({ page }) => {
  260 | 		// AC17 : le cas d'erreur INVOICE_NOT_PDF_READY doit s'afficher sous
  261 | 		// forme de toast côté UI. Le backend E2E (`invoice_pdf_e2e.rs`) couvre
  262 | 		// déjà la détection backend ; ici on vérifie que le frontend affiche
  263 | 		// correctement l'erreur en interceptant la réponse du serveur.
  264 | 		await login(page);
  265 | 		const contactId = await createContactWithAddressViaApi(page, uniq('PDF Err'));
  266 | 		const invoiceId = await createAndValidateInvoiceViaApi(page, contactId);
  267 | 
  268 | 		// Intercepte l'appel PDF pour renvoyer un 400 INVOICE_NOT_PDF_READY.
  269 | 		await page.route(`**/api/v1/invoices/${invoiceId}/pdf`, async (route) => {
  270 | 			await route.fulfill({
  271 | 				status: 400,
  272 | 				contentType: 'application/json',
  273 | 				body: JSON.stringify({
  274 | 					error: {
  275 | 						code: 'INVOICE_NOT_PDF_READY',
  276 | 						message: "Aucun compte bancaire principal n'est configuré pour cette company.",
  277 | 					},
  278 | 				}),
  279 | 			});
  280 | 		});
  281 | 
  282 | 		await page.goto(`/invoices/${invoiceId}`);
  283 | 		await page.getByRole('button', { name: /Télécharger PDF/i }).click();
  284 | 
  285 | 		// Toast d'erreur affichant le message INVOICE_NOT_PDF_READY.
  286 | 		await expect(
  287 | 			page.getByText(/compte bancaire principal|primary bank|INVOICE_NOT_PDF_READY/i),
```