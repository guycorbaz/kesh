# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: invoices_echeancier.spec.ts >> Échéancier factures — Story 5.4 >> golden path : création → échéancier → marquer payée → dé-marquer → export CSV
- Location: tests/e2e/invoices_echeancier.spec.ts:90:2

# Error details

```
Error: createContact failed: 401

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
  1   | import { expect, test } from '@playwright/test';
  2   | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  3   | 
  4   | test.beforeAll(async () => {
  5   | 	// Story 6.4 : preset `with-data` (= with-company + 1 contact + 1 product,
  6   | 	// PAS de facture pré-seedée — les tests ci-dessous créent les leurs via
  7   | 	// `daysFromToday` pour des dates déterministes).
  8   | 	await seedTestState('with-data');
  9   | });
  10  | 
  11  | test.afterEach(async ({ page }) => {
  12  | 	// Clear localStorage after each test to prevent token bleed to next test
  13  | 	await clearAuthStorage(page);
  14  | });
  15  | 
  16  | /**
  17  |  * Tests E2E — Échéancier factures (Story 5.4)
  18  |  *
  19  |  * Golden path : créer + valider 2 factures (une en retard, une future) →
  20  |  * naviguer /invoices/due-dates → marquer la passée payée → vérifier qu'elle
  21  |  * disparaît de « Impayées » → basculer sur « Payées » → vérifier le badge →
  22  |  * dé-marquer depuis la page détail → export CSV.
  23  |  *
  24  |  * Prérequis seed (identique aux autres `invoices.spec.ts`) : admin bootstrap,
  25  |  * une company, fiscal_year couvrant aujourd'hui, company_invoice_settings
  26  |  * avec comptes par défaut configurés.
  27  |  */
  28  | 
  29  | async function login(page: import('@playwright/test').Page) {
  30  | 	await page.goto('/login');
  31  | 	await page.fill('#username', 'admin');
  32  | 	await page.fill('#password', 'admin123');
  33  | 	await page.click('button[type="submit"]');
  34  | 	await expect(page).toHaveURL('/');
  35  | }
  36  | 
  37  | function uniq(prefix: string): string {
  38  | 	return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 1e6)}`;
  39  | }
  40  | 
  41  | async function createContactViaApi(
  42  | 	page: import('@playwright/test').Page,
  43  | 	name: string,
  44  | ): Promise<number> {
  45  | 	const res = await page.request.post('/api/v1/contacts', {
  46  | 		data: {
  47  | 			contactType: 'Entreprise',
  48  | 			name,
  49  | 			isClient: true,
  50  | 			isSupplier: false,
  51  | 			defaultPaymentTerms: '30 jours net',
  52  | 		},
  53  | 	});
> 54  | 	expect(res.ok(), `createContact failed: ${res.status()}`).toBeTruthy();
      |                                                            ^ Error: createContact failed: 401
  55  | 	return (await res.json()).id as number;
  56  | }
  57  | 
  58  | async function createAndValidateInvoice(
  59  | 	page: import('@playwright/test').Page,
  60  | 	contactId: number,
  61  | 	date: string,
  62  | 	dueDate: string,
  63  | 	amount: string,
  64  | ): Promise<number> {
  65  | 	const createRes = await page.request.post('/api/v1/invoices', {
  66  | 		data: {
  67  | 			contactId,
  68  | 			date,
  69  | 			dueDate,
  70  | 			paymentTerms: null,
  71  | 			lines: [
  72  | 				{ description: 'Prestation', quantity: '1', unitPrice: amount, vatRate: '8.10' },
  73  | 			],
  74  | 		},
  75  | 	});
  76  | 	expect(createRes.ok(), `create invoice failed: ${createRes.status()}`).toBeTruthy();
  77  | 	const inv = await createRes.json();
  78  | 	const validateRes = await page.request.post(`/api/v1/invoices/${inv.id}/validate`);
  79  | 	expect(validateRes.ok(), `validate failed: ${validateRes.status()}`).toBeTruthy();
  80  | 	return inv.id as number;
  81  | }
  82  | 
  83  | function daysFromToday(offset: number): string {
  84  | 	const d = new Date();
  85  | 	d.setDate(d.getDate() + offset);
  86  | 	return d.toISOString().slice(0, 10);
  87  | }
  88  | 
  89  | test.describe('Échéancier factures — Story 5.4', () => {
  90  | 	test('golden path : création → échéancier → marquer payée → dé-marquer → export CSV', async ({
  91  | 		page,
  92  | 	}) => {
  93  | 		await login(page);
  94  | 
  95  | 		const contactName = uniq('EchContact');
  96  | 		const contactId = await createContactViaApi(page, contactName);
  97  | 
  98  | 		// Facture en retard (due_date = hier) + facture future.
  99  | 		const overdueId = await createAndValidateInvoice(
  100 | 			page,
  101 | 			contactId,
  102 | 			daysFromToday(-30),
  103 | 			daysFromToday(-1),
  104 | 			'100.00',
  105 | 		);
  106 | 		const futureId = await createAndValidateInvoice(
  107 | 			page,
  108 | 			contactId,
  109 | 			daysFromToday(0),
  110 | 			daysFromToday(30),
  111 | 			'250.00',
  112 | 		);
  113 | 
  114 | 		await page.goto('/invoices/due-dates');
  115 | 		await expect(page.getByRole('heading', { name: /Échéancier/i })).toBeVisible();
  116 | 
  117 | 		// Les 2 doivent apparaître (filtre défaut = unpaid).
  118 | 		const overdueRow = page.locator('tbody tr', { hasText: contactName }).filter({
  119 | 			hasText: daysFromToday(-1),
  120 | 		});
  121 | 		await expect(overdueRow).toBeVisible({ timeout: 5000 });
  122 | 		// Badge "En retard" présent au moins une fois dans le tableau.
  123 | 		await expect(page.locator('tbody').getByText(/En retard|Overdue|In ritardo|Überfällig/i).first()).toBeVisible();
  124 | 
  125 | 		// Cliquer "Marquer payée" sur la facture en retard.
  126 | 		await overdueRow.getByRole('button', { name: /Marquer payée|Mark as paid|Segna|Als bezahlt/i }).click();
  127 | 		// Dialog s'ouvre → confirmer (date défaut = today).
  128 | 		await expect(page.getByRole('dialog')).toBeVisible();
  129 | 		await page.getByRole('button', { name: /Confirmer|Confirm|Conferma|bestätigen/i }).click();
  130 | 
  131 | 		// Après reload, la facture en retard disparaît du filtre "Impayées".
  132 | 		await expect(overdueRow).toHaveCount(0, { timeout: 5000 });
  133 | 
  134 | 		// Basculer sur "Payées" → la facture réapparaît avec le badge Payée.
  135 | 		await page.getByRole('tab', { name: /Payées|Paid|Pagate|Bezahlt/i }).click();
  136 | 		const paidRow = page.locator('tbody tr', { hasText: contactName }).first();
  137 | 		await expect(paidRow).toBeVisible({ timeout: 5000 });
  138 | 		await expect(paidRow.getByText(/Payée|Paid|Pagata|Bezahlt/i).first()).toBeVisible();
  139 | 
  140 | 		// Naviguer vers la page détail → dé-marquer.
  141 | 		await page.goto(`/invoices/${overdueId}`);
  142 | 		await page.getByRole('button', { name: /Dé-marquer|Unmark|Annulla|rückgängig/i }).click();
  143 | 		await page.getByRole('dialog').getByRole('button', { name: /Dé-marquer|Unmark|Annulla|rückgängig/i }).click();
  144 | 
  145 | 		// Retour échéancier : la facture réapparaît en Impayées/En retard.
  146 | 		await page.goto('/invoices/due-dates');
  147 | 		const backRow = page.locator('tbody tr', { hasText: contactName }).filter({
  148 | 			hasText: daysFromToday(-1),
  149 | 		});
  150 | 		await expect(backRow).toBeVisible({ timeout: 5000 });
  151 | 
  152 | 		// Export CSV : intercepter le téléchargement.
  153 | 		const [download] = await Promise.all([
  154 | 			page.waitForEvent('download'),
```