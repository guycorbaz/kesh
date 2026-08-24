import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';

/**
 * Tests E2E — Gestion des exercices comptables (Story 3.7).
 *
 * Pré-requis seed DB (preset `with-company`) :
 * - admin / admin123
 * - une company configurée + 1 fiscal_year ouvert (2020-2030).
 *
 * Pour les tests de fallback toast (AC #22), on utilise des dates **hors**
 * de la plage 2020-2030 du fiscal_year seedé, ce qui déclenche
 * `NO_FISCAL_YEAR` (journal_entries) ou `FISCAL_YEAR_INVALID` (validate_invoice).
 */

// Code Review Pass 1 F16 — reset DB state before each test pour isoler les
// tests entre eux, même si un test précédent a échoué après création d'un
// fiscal_year. `with-company` reset à un seul fiscal_year (Exercice CI
// 2020-2030), donc les tests qui créent leurs propres exercices ne risquent
// pas de tomber sur un overlap résiduel.
test.beforeEach(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: import('@playwright/test').Page) {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function goToFiscalYears(page: import('@playwright/test').Page) {
	await login(page);
	await page.goto('/settings/fiscal-years');
	await expect(page).toHaveURL(/\/settings\/fiscal-years/);
}

test.describe('Page exercices — affichage', () => {
	test('affiche le titre et le bouton Nouvel exercice', async ({ page }) => {
		await goToFiscalYears(page);
		await expect(page.getByRole('heading', { name: /Exercices comptables/ })).toBeVisible();
		await expect(page.getByTestId('fiscal-year-create-button')).toBeVisible();
	});

	test('affiche le fiscal_year seedé (2020-2030)', async ({ page }) => {
		await goToFiscalYears(page);
		// L'exercice seedé est nommé "Exercice CI" (2020-01-01 → 2030-12-31).
		await expect(page.getByTestId('fiscal-year-table')).toBeVisible();
		await expect(page.locator('tr', { hasText: '2020-01-01' })).toBeVisible();
	});
});

test.describe('Page exercices — création + clôture', () => {
	test('crée un exercice 2031, le renomme puis le clôture', async ({ page }) => {
		// Reset DB déjà appliqué via test.beforeEach (Code Review Pass 1 F16).
		await goToFiscalYears(page);

		await page.getByTestId('fiscal-year-create-button').click();

		// La modale est pré-remplie avec l'année courante. On force une année
		// distincte de l'exercice seedé pour éviter les overlaps.
		await page.fill('#fy-create-name', 'Exercice 2031 E2E');
		await page.fill('#fy-create-start', '2031-01-01');
		await page.fill('#fy-create-end', '2031-12-31');
		await page.getByRole('button', { name: 'Créer' }).click();

		// La nouvelle ligne apparaît en tête (DESC).
		const row2031 = page.locator('tr', { hasText: 'Exercice 2031 E2E' }).first();
		await expect(row2031).toBeVisible({ timeout: 5000 });

		// Renommer.
		await row2031.getByRole('button', { name: /Renommer/ }).click();
		await page.fill('#fy-rename-name', 'FY 2031');
		await page.getByRole('button', { name: 'Enregistrer' }).click();
		await expect(page.locator('tr', { hasText: 'FY 2031' })).toBeVisible({ timeout: 5000 });

		// Clôturer.
		const rowFy = page.locator('tr', { hasText: 'FY 2031' }).first();
		await rowFy.getByRole('button', { name: /Clôturer/ }).click();
		// KF-047 (#344) — ⚠️ ce sélecteur visait `/définitivement/`, un mot qui
		// n'existe PLUS dans ce dialogue : le libellé de confirmation dit « Clôturer »,
		// et le corps explique qu'un administrateur peut rouvrir l'exercice. Le texte
		// a donc été corrigé — **la clôture n'est pas définitive** — et le test est
		// resté sur l'ancien mot. C'est l'angle mort de KF-043 (#326) : un sélecteur
		// figé sur un libellé ne survit pas à sa correction. Visé par `data-testid`.
		await page.getByTestId('fiscal-year-close-confirm').click();

		// Le statut passe à Clôturé et le bouton Clôturer disparaît.
		await expect(rowFy.getByText(/Clôturé/)).toBeVisible({ timeout: 5000 });
		await expect(rowFy.getByRole('button', { name: /Clôturer/ })).toHaveCount(0);
	});
});

/**
 * Helpers Story 9.5-1d KF #47 — AC #22 fallback toast actionnable.
 *
 * Les 3 tests AC #22 exercent le helper `notifyMissingFiscalYearOrFallback`
 * (`frontend/src/lib/shared/utils/notify.ts:98`) via ses 2 call sites
 * instrumentés :
 *   - `validateInvoice` flow (`/(app)/invoices/[id]/+page.svelte:94-96`)
 *     → couvre `FISCAL_YEAR_INVALID`.
 *   - `JournalEntryForm.handleSubmit` (`features/journal-entries/JournalEntryForm.svelte:140-141`)
 *     → couvre `NO_FISCAL_YEAR` + `FISCAL_YEAR_CLOSED`.
 *
 * Routage critique (vérifié spec validate Pass 1 ground-truth) :
 *   - `validate_invoice` utilise `find_open_covering_date` (`invoices.rs:970`)
 *     → un FY clos retourne `FISCAL_YEAR_INVALID` (PAS `FISCAL_YEAR_CLOSED`).
 *   - `FISCAL_YEAR_CLOSED` n'est levé QUE par `journal_entries::create`
 *     (`journal_entries.rs:109`) + `journal_entries::update` (`:598 + :836`).
 *   - Test 3 utilise donc `JournalEntryForm`, pas `validateInvoice`.
 */

/** Récupère 2 numéros de comptes (1xxx asset + 3xxx revenue) du seed
 * pour remplir les lignes du formulaire d'écriture comptable. Pattern
 * réutilisé de `journal-entries.spec.ts:42-58`. */
async function getSeedAccountNumbers(
	page: import('@playwright/test').Page
): Promise<{ debitNumber: string; creditNumber: string }> {
	const ctx = await authedApiContext(page);
	try {
		const resp = await ctx.get('/api/v1/accounts?includeArchived=false');
		expect(resp.ok()).toBeTruthy();
		const accounts: Array<{ number: string; name: string }> = await resp.json();
		const asset = accounts.find((a) => /^10[0-9]{2}$/.test(a.number)) ?? accounts[0];
		const revenue =
			accounts.find((a) => /^3[0-9]{3}$/.test(a.number)) ??
			accounts.find((a) => /^2[0-9]{3}$/.test(a.number)) ??
			accounts[1];
		return { debitNumber: asset.number, creditNumber: revenue.number };
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Crée un contact via API et retourne son id. */
async function createContactViaApi(page: import('@playwright/test').Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: `KF47 Test Contact ${Date.now()}`,
				isClient: true,
				isSupplier: false,
				defaultPaymentTerms: '30 jours net'
			}
		});
		expect(res.ok(), `createContact failed: ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Crée une facture brouillon avec une date donnée et retourne son id.
 * La date peut être hors plage du fiscal_year (l'API `create_invoice`
 * accepte n'importe quelle date — seule `validate_invoice` vérifie). */
async function createDraftInvoiceViaApi(
	page: import('@playwright/test').Page,
	contactId: number,
	date: string
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/invoices', {
			data: {
				contactId,
				date,
				dueDate: date,
				paymentTerms: '30 jours net',
				lines: [
					{
						description: 'KF47 test line',
						quantity: '1',
						unitPrice: '100.00',
						// VAT 8.10 = taux Suisse 2024+ (la DB ne reconnaît plus 7.70 hérité 2018-2023).
						vatRate: '8.10'
					}
				]
			}
		});
		expect(res.ok(), `create draft invoice failed: ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Clôture le fiscal_year ouvert seedé par `with-company` (2020-2030).
 * Endpoint : POST /api/v1/fiscal-years/{id}/close (`fiscal_years.rs:8`).
 * Retourne l'id du fiscal_year clôturé. */
async function closeSeededFiscalYearViaApi(
	page: import('@playwright/test').Page
): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const listRes = await ctx.get('/api/v1/fiscal-years');
		expect(listRes.ok()).toBeTruthy();
		const fys: Array<{ id: number; status: string }> = await listRes.json();
		// Assertion explicite : exactement 1 FY Open dans le seed `with-company`
		// (cohérent fixture `test_fixtures.rs:170-176` qui seed 1 unique FY).
		// Si un futur seed produit ≥ 2 Open, ce test deviendrait silencieusement
		// fragile (find pick le premier, le 2ᵉ reste ouvert → journal_entries::create
		// trouve toujours un FY couvrant 2025-06-15 → pas de FISCAL_YEAR_CLOSED).
		// Fail fast plutôt que timeout confus (Pass 1 code-review ECH-6 polish).
		const openFys = fys.filter((f) => f.status === 'Open');
		expect(openFys.length, 'expected exactly one Open fiscal_year in with-company seed').toBe(1);
		const open = openFys[0]!;
		const closeRes = await ctx.post(`/api/v1/fiscal-years/${open.id}/close`);
		expect(closeRes.ok(), `close FY failed: ${closeRes.status()}`).toBeTruthy();
		return open.id;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Remplit le formulaire JournalEntryForm jusqu'à activer le bouton Valider.
 * Pattern : 2 lignes équilibrées avec 2 comptes du seed.
 *
 * Indexation `input[inputmode="decimal"]` — vérifié ground-truth
 * `JournalEntryForm.svelte` : le formulaire initialise toujours exactement 2
 * lignes (`{ accountId, debit, credit } × 2`), rendant 4 inputs decimal stables
 * dans l'ordre DOM :
 *   - `nth(0)` = ligne 0 debit  ← rempli ici (100.00)
 *   - `nth(1)` = ligne 0 credit
 *   - `nth(2)` = ligne 1 debit
 *   - `nth(3)` = ligne 1 credit ← rempli ici (100.00 → équilibre)
 * Le skip `nth(1)/(2)` est intentionnel : on construit la paire debit ligne 0
 * + credit ligne 1 = 2 lignes équilibrées (Pass 1 code-review BH-4 polish). */
async function fillJournalEntryFormForSubmit(
	page: import('@playwright/test').Page,
	{ entryDate, debitNumber, creditNumber }: { entryDate: string; debitNumber: string; creditNumber: string }
): Promise<void> {
	await page.fill('#entry-date', entryDate);
	await page.fill('#entry-description', 'KF47 fallback toast test');
	const accountInputs = page.locator('input[aria-autocomplete="list"]');
	await accountInputs.nth(0).fill(debitNumber);
	await page.getByRole('option').first().click();
	await page.locator('input[inputmode="decimal"]').nth(0).fill('100.00');
	await accountInputs.nth(1).fill(creditNumber);
	await page.getByRole('option').first().click();
	await page.locator('input[inputmode="decimal"]').nth(3).fill('100.00');
	await expect(page.getByText(/✓ Équilibré/)).toBeVisible();
}

test.describe('AC #22 — fallback toast actionnable', () => {
	// 3 vrais tests Playwright qui exercent le helper `notifyMissingFiscalYearOrFallback`
	// via ses 2 call sites Svelte instrumentés (validateInvoice + JournalEntryForm).
	// Remplace l'ancien `test.skip(true, ...)` placeholder (Story 9-5-1d KF #47).

	test('FISCAL_YEAR_INVALID — validateInvoice avec date hors plage du fiscal_year', async ({
		page
	}) => {
		// Setup : `with-company` seedé (FY ouvert 2020-2030). Création contact +
		// facture brouillon avec date `1900-01-01` (hors plage).
		await login(page);
		const contactId = await createContactViaApi(page);
		const invoiceId = await createDraftInvoiceViaApi(page, contactId, '1900-01-01');

		// Naviguer sur la facture et déclencher le flow validate.
		await page.goto(`/invoices/${invoiceId}`);
		await expect(page.getByRole('heading', { name: /Facture/ })).toBeVisible({ timeout: 5000 });
		// Cliquer le bouton "Valider" principal (ouvre la modale de confirmation).
		await page.getByRole('button', { name: 'Valider' }).first().click();
		// Guard : attendre que la modale bits-ui `Dialog.Root` (title "Valider la
		// facture") soit rendue avant de cibler le bouton confirmation, sinon
		// `.last()` pourrait re-sélectionner le bouton page si la modale n'a pas
		// encore monté (Pass 1 code-review BH-3 — Playwright auto-wait empirique
		// suffit 6/6 mais ce guard rend l'intention explicite et le test robuste
		// à un futur ralentissement du portal mount).
		// ⚠️ Ciblé par `data-testid`, jamais par le libellé : ce titre est traduit depuis la
		// story 23-3b, et un sélecteur figé sur une chaîne française casse dès qu'une autre
		// locale est servie. (AC7 — un sélecteur se bascule, il ne se « répare » pas.)
		const validateDialog = page.getByTestId('invoice-validate-dialog');
		await expect(validateDialog).toBeVisible({ timeout: 5000 });
		// Cliquer "Valider" dans la modale → confirmValidate() → backend retourne
		// FISCAL_YEAR_INVALID (find_open_covering_date renvoie None pour 1900-01-01).
		// `confirmValidate` invoque `notifyMissingFiscalYearOrFallback(err)` qui rend le toast.
		await validateDialog.getByRole('button', { name: 'Valider' }).click();

		// Assertion 1 : toast actionnable visible avec message NO/INVALID (« Créez d'abord »).
		// svelte-sonner rend les toasts avec `data-sonner-toast=""` + `aria-live="polite"`
		// (PAS de `role="alert"` — vérifié `node_modules/svelte-sonner/dist/Toast.svelte:344-360`).
		// KF-047 (#344) — ⚠️ `d['’]abord` accepte les DEUX apostrophes, et ce n'est
		// pas une coquetterie : le repli JS de `notify.ts` écrit l'apostrophe DROITE,
		// le catalogue `fr-CH` la TYPOGRAPHIQUE. Le catalogue gagne — c'est lui qui
		// s'affiche — et la regex figée sur `d'abord` ne matchait plus rien. La
		// traduction a amélioré la typographie ; le sélecteur ne l'a pas suivie.
		const toast = page.locator('[data-sonner-toast]').filter({ hasText: /Créez d['’]abord un exercice/ });
		await expect(toast).toBeVisible({ timeout: 5000 });

		// Assertion 2 : clic sur le bouton action "Ouvrir Paramètres" → navigation.
		await toast.getByRole('button', { name: /Ouvrir Paramètres/i }).click();
		await expect(page).toHaveURL(/\/settings\/fiscal-years/);
	});

	test('NO_FISCAL_YEAR — JournalEntryForm sans aucun fiscal_year configuré', async ({
		page
	}) => {
		// Setup : override `with-company-no-fy` (company sans aucun FY). Re-login
		// obligatoire après seed override (truncate invalide les sessions JWT —
		// MEDIUM-02 Pass 1 spec validate).
		await seedTestState('with-company-no-fy');
		await login(page);

		// Le chart of accounts est aussi seedé par with-company-no-fy (1000, 3000, etc.).
		const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);

		// Naviguer + ouvrir le formulaire d'écriture.
		await page.goto('/journal-entries');
		await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
		await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();

		// Remplir le formulaire (date arbitraire — pas de FY donc toute date déclenchera NO_FISCAL_YEAR).
		await fillJournalEntryFormForSubmit(page, {
			entryDate: '2025-06-15',
			debitNumber,
			creditNumber
		});

		// Soumettre → backend `journal_entries::create` retourne NO_FISCAL_YEAR
		// (aucun FY couvrant cette date) → helper trigger le toast.
		await page.getByRole('button', { name: 'Valider' }).click();

		// Assertion 1 : toast actionnable « Créez d'abord » (selector svelte-sonner data-sonner-toast).
		const toast = page.locator('[data-sonner-toast]').filter({ hasText: /Créez d['’]abord un exercice/ });
		await expect(toast).toBeVisible({ timeout: 5000 });

		// Assertion 2 : clic action → navigation paramètres.
		await toast.getByRole('button', { name: /Ouvrir Paramètres/i }).click();
		await expect(page).toHaveURL(/\/settings\/fiscal-years/);
	});

	test('FISCAL_YEAR_CLOSED — JournalEntryForm sur FY clos avec date in-range', async ({
		page
	}) => {
		// Setup : `with-company` seedé (FY ouvert 2020-2030). On le clôture via API
		// puis on tente une écriture comptable avec date in-range (2025-06-15) —
		// le backend `journal_entries::create:109` rejette avec FISCAL_YEAR_CLOSED
		// (NB : `validate_invoice` aurait retourné FISCAL_YEAR_INVALID via
		// `find_open_covering_date` qui filtre status='Open' — d'où le choix
		// JournalEntryForm pour ce test, spec validate Pass 1 CRITICAL ground-truth).
		await login(page);
		await closeSeededFiscalYearViaApi(page);

		const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);

		await page.goto('/journal-entries');
		await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
		await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();

		// Date in-range du FY 2020-2030 (maintenant clôturé).
		await fillJournalEntryFormForSubmit(page, {
			entryDate: '2025-06-15',
			debitNumber,
			creditNumber
		});

		await page.getByRole('button', { name: 'Valider' }).click();

		// Assertion 1 : toast distinct « clôturé » (message i18n
		// `error-fiscal-year-closed-for-date`).
		const toast = page.locator('[data-sonner-toast]').filter({ hasText: /clôturé/ });
		await expect(toast).toBeVisible({ timeout: 5000 });

		// Assertion 2 : le toast NE doit PAS contenir « Créez d'abord » (différencier
		// du toast FY_INVALID/NO_FY pour confirmer le branching helper).
		await expect(
			page.locator('[data-sonner-toast]').filter({ hasText: /Créez d'abord/ })
		).toHaveCount(0);

		// Assertion 3 : clic action → navigation paramètres (même behavior helper).
		await toast.getByRole('button', { name: /Ouvrir Paramètres/i }).click();
		await expect(page).toHaveURL(/\/settings\/fiscal-years/);
	});
});
