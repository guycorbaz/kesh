# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: journal-entries.spec.ts >> Page écritures — saisie >> saisie nominale d'une écriture équilibrée
- Location: tests/e2e/journal-entries.spec.ts:83:2

# Error details

```
Error: expect(received).toBeTruthy()

Received: false
```

# Test source

```ts
  1   | import { expect, test } from '@playwright/test';
  2   | import { seedTestState, clearAuthStorage } from './helpers/test-state';
  3   | 
  4   | test.beforeAll(async () => {
  5   | 	await seedTestState('with-company');
  6   | });
  7   | 
  8   | test.afterEach(async ({ page }) => {
  9   | 	// Clear localStorage after each test to prevent token bleed to next test
  10  | 	await clearAuthStorage(page);
  11  | });
  12  | 
  13  | /**
  14  |  * Tests E2E — Saisie d'écritures en partie double (Story 3.2)
  15  |  *
  16  |  * Ces tests nécessitent :
  17  |  * - un backend Kesh fonctionnel sur localhost:3000
  18  |  * - un admin bootstrap (admin / admin123)
  19  |  * - un seed démo effectué (plan comptable PME + exercice ouvert de
  20  |  *   l'année courante créés par `kesh_seed::seed_demo`)
  21  |  */
  22  | 
  23  | async function login(page: import('@playwright/test').Page) {
  24  | 	await page.goto('/login');
  25  | 	await page.fill('#username', 'admin');
  26  | 	await page.fill('#password', 'admin123');
  27  | 	await page.click('button[type="submit"]');
  28  | 	await expect(page).toHaveURL('/');
  29  | }
  30  | 
  31  | async function goToJournalEntries(page: import('@playwright/test').Page) {
  32  | 	await login(page);
  33  | 	await page.goto('/journal-entries');
  34  | 	await expect(page).toHaveURL('/journal-entries');
  35  | }
  36  | 
  37  | /**
  38  |  * Récupère deux comptes actifs via l'API pour injecter leur number/nom
  39  |  * dans l'autocomplétion. Les IDs ne sont pas stables entre resets, donc
  40  |  * on cherche par numéro (1020 Banque, 3000 Ventes, etc.).
  41  |  */
  42  | async function getSeedAccountNumbers(
  43  | 	page: import('@playwright/test').Page
  44  | ): Promise<{ debitNumber: string; creditNumber: string }> {
  45  | 	const resp = await page.request.get('/api/v1/accounts?includeArchived=false');
> 46  | 	expect(resp.ok()).toBeTruthy();
      |                    ^ Error: expect(received).toBeTruthy()
  47  | 	const accounts: Array<{ number: string; name: string }> = await resp.json();
  48  | 
  49  | 	// On prend un compte d'actif (1xxx) et un compte de produit/passif (3xxx ou 2xxx).
  50  | 	const asset = accounts.find((a) => /^10[0-9]{2}$/.test(a.number)) ?? accounts[0];
  51  | 	const revenue = accounts.find((a) => /^3[0-9]{3}$/.test(a.number)) ??
  52  | 		accounts.find((a) => /^2[0-9]{3}$/.test(a.number)) ??
  53  | 		accounts[1];
  54  | 
  55  | 	return { debitNumber: asset.number, creditNumber: revenue.number };
  56  | }
  57  | 
  58  | test.describe('Page écritures — affichage', () => {
  59  | 	test('affiche le titre et le bouton Nouvelle écriture', async ({ page }) => {
  60  | 		await goToJournalEntries(page);
  61  | 		await expect(page.getByRole('heading', { name: /Écritures/ })).toBeVisible();
  62  | 		await expect(page.getByRole('button', { name: /Nouvelle écriture/ })).toBeVisible();
  63  | 	});
  64  | 
  65  | 	test('affiche un message si liste vide', async ({ page }) => {
  66  | 		await goToJournalEntries(page);
  67  | 		// Après seed_demo, aucune écriture n'est créée — l'état initial
  68  | 		// peut montrer le message vide OU des écritures de tests précédents.
  69  | 		// On vérifie simplement que la page charge.
  70  | 		const hasEmpty = await page
  71  | 			.getByText(/Aucune écriture/)
  72  | 			.isVisible()
  73  | 			.catch(() => false);
  74  | 		const hasTable = await page
  75  | 			.getByRole('table')
  76  | 			.isVisible()
  77  | 			.catch(() => false);
  78  | 		expect(hasEmpty || hasTable).toBeTruthy();
  79  | 	});
  80  | });
  81  | 
  82  | test.describe('Page écritures — saisie', () => {
  83  | 	test('saisie nominale d\'une écriture équilibrée', async ({ page }) => {
  84  | 		await goToJournalEntries(page);
  85  | 		const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);
  86  | 
  87  | 		await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
  88  | 		await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();
  89  | 
  90  | 		// Libellé
  91  | 		await page.fill('#entry-description', 'Test E2E saisie nominale');
  92  | 
  93  | 		// Ligne 1 : débit
  94  | 		const accountInputs = page.locator('input[aria-autocomplete="list"]');
  95  | 		await accountInputs.nth(0).fill(debitNumber);
  96  | 		// Attendre que l'option apparaisse et la sélectionner.
  97  | 		await page.getByRole('option').first().click();
  98  | 		await page.locator('input[inputmode="decimal"]').nth(0).fill('100.00');
  99  | 
  100 | 		// Ligne 2 : crédit
  101 | 		await accountInputs.nth(1).fill(creditNumber);
  102 | 		await page.getByRole('option').first().click();
  103 | 		await page.locator('input[inputmode="decimal"]').nth(3).fill('100.00');
  104 | 
  105 | 		// L'indicateur doit être équilibré.
  106 | 		await expect(page.getByText(/✓ Équilibré/)).toBeVisible();
  107 | 
  108 | 		// Valider
  109 | 		await page.getByRole('button', { name: 'Valider' }).click();
  110 | 
  111 | 		// Retour à la liste + écriture visible.
  112 | 		await expect(page.getByText(/Test E2E saisie nominale/)).toBeVisible({ timeout: 5000 });
  113 | 	});
  114 | 
  115 | 	test('indicateur de déséquilibre et bouton Valider désactivé', async ({ page }) => {
  116 | 		await goToJournalEntries(page);
  117 | 		const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);
  118 | 
  119 | 		await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
  120 | 		await page.fill('#entry-description', 'Test déséquilibre');
  121 | 
  122 | 		const accountInputs = page.locator('input[aria-autocomplete="list"]');
  123 | 		await accountInputs.nth(0).fill(debitNumber);
  124 | 		await page.getByRole('option').first().click();
  125 | 		await page.locator('input[inputmode="decimal"]').nth(0).fill('100');
  126 | 
  127 | 		await accountInputs.nth(1).fill(creditNumber);
  128 | 		await page.getByRole('option').first().click();
  129 | 		await page.locator('input[inputmode="decimal"]').nth(3).fill('50');
  130 | 
  131 | 		// L'indicateur doit être rouge (déséquilibré).
  132 | 		await expect(page.getByText(/✗ Déséquilibré/)).toBeVisible();
  133 | 
  134 | 		// Le bouton Valider est désactivé.
  135 | 		const submitBtn = page.getByRole('button', { name: 'Valider' });
  136 | 		await expect(submitBtn).toBeDisabled();
  137 | 	});
  138 | 
  139 | 	test('rejet client d\'un montant avec plus de 4 décimales', async ({ page }) => {
  140 | 		await goToJournalEntries(page);
  141 | 		const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);
  142 | 
  143 | 		await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
  144 | 		await page.fill('#entry-description', 'Test > 4 décimales');
  145 | 
  146 | 		const accountInputs = page.locator('input[aria-autocomplete="list"]');
```