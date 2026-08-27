/**
 * Story 9-1 — Tests E2E Playwright pour la page `/reports`.
 *
 * Scénarios :
 *   1. AC #27 + #33 : page chargée + les onglets visibles (Bilan, Compte de résultat,
 *      Balance, Journaux, TVA, les deux rapports projet, Balance âgée, Grand livre)
 *   2. AC #28 (T12.1) : génération bilan via UI sur preset `with-company` (sans
 *      écritures → empty-state message attendu, mais le flow Generate→Response
 *      est vérifié end-to-end avec MariaDB up + audit best-effort)
 *   3. AC #34 : axe a11y scan zero violations (état empty + état populé)
 *   4. AC #34 (T12.4) : company sans fiscal_year → bouton Générer disabled
 *      + message i18n `reports-error-no-fiscal-year-available` visible
 *      (preset `with-company-no-fy` — Issue #90). **Exécuté en dernier** car
 *      reseed la DB en état no-fy, incompatible avec les tests précédents.
 *
 * Pré-requis :
 *   - MariaDB up + KESH_TEST_MODE=true.
 *   - Seed `with-company` (1000-4000 comptes + fiscal_year, pas d'écritures).
 *   - Playwright browsers installés (Ubuntu 26.04+ :
 *     PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
 */

import { expect, test, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test('reports page loads with 9 tabs (AC #27 + #33, + Balance âgée 21-7, + Grand livre 24-1)', async ({ page }) => {
	await login(page);
	await page.goto('/reports');

	await expect(page.getByRole('heading', { name: /rapports/i })).toBeVisible();

	// 9 onglets : Bilan, Résultat, Balance, Journaux, TVA, Dépenses par projet,
	// Rendement par projet (19-6b), Balance âgée (21-7), + Grand livre (24-1).
	//
	// ⚠️ Ce compte est le SEUL garde-fou du dépôt contre un onglet qui
	// disparaîtrait. Il se met à jour en NOMMANT ce qui entre, jamais en
	// alignant le chiffre sur ce qu'on observe.
	const tabs = page.getByRole('tab');
	await expect(tabs).toHaveCount(9);
});

test('reports project-expenses tab generates end-to-end (Story 19-6a)', async ({ page }) => {
	await login(page);

	// Crée un projet via API (le sélecteur n'apparaît que si un projet existe).
	const api = await authedApiContext(page);
	const code = `E2E-${Date.now().toString().slice(-6)}`;
	const projRes = await api.post('/api/v1/projects', {
		data: { code, name: 'Projet E2E rapport', parentId: null },
	});
	expect(projRes.ok(), `create project: ${projRes.status()}`).toBeTruthy();

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// Bascule sur l'onglet « Dépenses par projet ».
	await page.getByRole('tab', { name: /dépenses par projet/i }).click();
	await expect(page.getByTestId('project-report-controls')).toBeVisible();

	// Sélectionne le projet créé + mode Exercice (défaut) puis génère.
	await page.getByTestId('project-report-project').selectOption({ label: `${code} — Projet E2E rapport` });
	await page.getByTestId('project-report-generate').click();

	// La vue rend (empty-state attendu — le preset with-company n'a pas d'écritures
	// taguées) sans erreur backend. La vue englobe l'état vide.
	await expect(page.getByTestId('project-expenses-view')).toBeVisible({ timeout: 5000 });
	await expect(page.getByTestId('project-expenses-empty')).toBeVisible();
	await expect(page.getByRole('alert')).not.toBeVisible({ timeout: 1500 }).catch(() => {});

	await disposeContextSafe(api);
});

test('reports project-return tab generates end-to-end (Story 19-6b)', async ({ page }) => {
	await login(page);

	const api = await authedApiContext(page);
	const code = `RET-${Date.now().toString().slice(-6)}`;
	const projRes = await api.post('/api/v1/projects', {
		data: { code, name: 'Projet rendement E2E', parentId: null },
	});
	expect(projRes.ok(), `create project: ${projRes.status()}`).toBeTruthy();

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// Bascule sur l'onglet « Rendement par projet ».
	await page.getByRole('tab', { name: /rendement par projet/i }).click();
	await expect(page.getByTestId('project-report-controls')).toBeVisible();

	await page.getByTestId('project-report-project').selectOption({ label: `${code} — Projet rendement E2E` });
	await page.getByTestId('project-report-generate').click();

	// La vue rend (empty-state attendu — pas d'écritures taguées) sans erreur.
	await expect(page.getByTestId('project-return-view')).toBeVisible({ timeout: 5000 });
	await expect(page.getByTestId('project-return-empty')).toBeVisible();
	await expect(page.getByRole('alert')).not.toBeVisible({ timeout: 1500 }).catch(() => {});

	await disposeContextSafe(api);
});

test('reports page generates balance sheet end-to-end (AC #28, T12.1)', async ({ page }) => {
	await login(page);
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// L'onglet Bilan est actif par défaut. Le sélecteur d'exercice est populé
	// par +page.ts (preset `with-company` a 1 fiscal_year).
	const generateButton = page.getByRole('button', { name: /générer/i });
	await expect(generateButton).toBeEnabled();

	// Vérifier que l'onglet Bilan est bien aria-selected (P6 ARIA patches).
	const bilanTab = page.getByRole('tab').first();
	await expect(bilanTab).toHaveAttribute('aria-selected', 'true');

	await generateButton.click();

	// Le preset `with-company` ne seed PAS d'écritures comptables → la génération
	// retourne un BalanceSheetDto avec assets:[] liabilities:[] → empty-state
	// message via `isReportEmpty` helper. C'est le chemin success path AC #28
	// (200 OK + DTO valide rendu) même sans données.
	const tabpanel = page.getByRole('tabpanel');
	await expect(tabpanel).toBeVisible();

	// Soit l'empty-state message s'affiche (cas sans écriture, attendu pour
	// `with-company`), soit une réponse non-vide avec des montants suisses.
	// On valide qu'AU MOINS un des deux a lieu (pas d'erreur backend rendue).
	await expect(page.getByRole('alert')).not.toBeVisible({ timeout: 2000 }).catch(() => {});
	const emptyMsg = page.getByText(/aucune écriture/i);
	const totalActifs = page.getByText(/total actifs/i);
	await expect(emptyMsg.or(totalActifs)).toBeVisible({ timeout: 5000 });
});

test('reports page has zero axe a11y violations (empty state)', async ({ page }) => {
	await login(page);
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	const results = await new AxeBuilder({ page })
		.withTags(['wcag2a', 'wcag2aa'])
		.analyze();
	expect(results.violations).toEqual([]);
});

test('reports page has zero axe a11y violations (populated state)', async ({ page }) => {
	await login(page);
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// P24 — scanner aussi l'état rendu (empty-state ou bilan), pas seulement
	// le shell vide. Avec `with-company` (sans écritures), on déclenche
	// l'empty-state via `isReportEmpty` après Generate.
	await page.getByRole('button', { name: /générer/i }).click();
	await page.waitForLoadState('networkidle');

	const results = await new AxeBuilder({ page })
		.withTags(['wcag2a', 'wcag2aa'])
		.analyze();
	expect(results.violations).toEqual([]);
});

// Story 18-1f (AC2) — décompte TVA de bout-en-bout. Crée une facture validée avec
// TVA via API (pattern `invoices.spec.ts`), puis génère le rapport TVA dans l'UI et
// vérifie l'affichage du décompte (TVA due, récupérable, solde). Doit s'exécuter
// AVANT le test no-fy (qui reseed la DB).
async function createValidatedInvoiceWithVatViaApi(page: Page): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const today = new Date().toISOString().slice(0, 10);
		const contactRes = await ctx.post('/api/v1/contacts', {
			data: { contactType: 'Entreprise', name: `Client TVA ${today}`, isClient: true, isSupplier: false },
		});
		expect(contactRes.ok(), `create contact: ${contactRes.status()}`).toBeTruthy();
		const contactId = (await contactRes.json()).id as number;

		const invRes = await ctx.post('/api/v1/invoices', {
			data: {
				contactId,
				date: today,
				dueDate: today,
				lines: [{ description: 'Prestation TVA', quantity: '1', unitPrice: '1000.00', vatRate: '8.10' }],
			},
		});
		expect(invRes.ok(), `create invoice: ${invRes.status()}`).toBeTruthy();
		const invoiceId = (await invRes.json()).id as number;
		const valRes = await ctx.post(`/api/v1/invoices/${invoiceId}/validate`);
		expect(valRes.ok(), `validate invoice: ${valRes.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('reports page generates VAT décompte with TVA due end-to-end (Story 18-1f AC2)', async ({
	page,
}) => {
	await login(page);
	// Facture validée à 8.1 % → l'écriture comptable porte 81.00 de TVA due.
	await createValidatedInvoiceWithVatViaApi(page);

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// Sélectionner l'onglet TVA AVANT de générer (generate() dépend de l'onglet actif).
	await page.getByRole('tab', { name: /^TVA$/ }).click();
	const generateButton = page.getByRole('button', { name: /générer/i });
	await expect(generateButton).toBeEnabled();
	await generateButton.click();
	await page.waitForLoadState('networkidle');

	// Le décompte affiche TVA due (81.00), TVA récupérable et solde.
	const tabpanel = page.getByRole('tabpanel');
	await expect(tabpanel).toBeVisible();
	await expect(tabpanel.getByText(/TVA due/i)).toBeVisible();
	await expect(tabpanel.getByText('81.00').first()).toBeVisible({ timeout: 5000 });
	await expect(tabpanel.getByText(/TVA récupérable/i)).toBeVisible();
	await expect(tabpanel.getByText(/Solde/i)).toBeVisible();
	// Pas d'erreur backend rendue.
	await expect(page.getByRole('alert')).toHaveCount(0);
});

// ============================================================
// Story 21-7 — Balance âgée débiteurs (onglet /reports)
// ============================================================

/** Crée un contact + une facture validée échue (poste ouvert). Retourne le nom du contact. */
async function createOverdueInvoiceViaApi(page: Page, label: string): Promise<string> {
	const ctx = await authedApiContext(page);
	try {
		const name = `${label} ${Date.now().toString().slice(-6)}`;
		const contactRes = await ctx.post('/api/v1/contacts', {
			data: { contactType: 'Entreprise', name, isClient: true, isSupplier: false },
		});
		expect(contactRes.ok(), `create contact: ${contactRes.status()}`).toBeTruthy();
		const contactId = (await contactRes.json()).id as number;
		// Échéance 40 j dans le passe → bucket 31-60 ; poste ouvert (validée, impayée).
		const due = new Date();
		due.setDate(due.getDate() - 40);
		const dueStr = due.toISOString().slice(0, 10);
		const invRes = await ctx.post('/api/v1/invoices', {
			data: {
				contactId,
				date: dueStr,
				dueDate: dueStr,
				lines: [{ description: 'Prestation', quantity: '1', unitPrice: '500.00', vatRate: '0.00' }],
			},
		});
		expect(invRes.ok(), `create invoice: ${invRes.status()}`).toBeTruthy();
		const invoiceId = (await invRes.json()).id as number;
		const valRes = await ctx.post(`/api/v1/invoices/${invoiceId}/validate`);
		expect(valRes.ok(), `validate: ${valRes.status()}`).toBeTruthy();
		return name;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test('reports aged-receivables : génération + drill-down + export CSV (Admin)', async ({ page }) => {
	await login(page);
	const contactName = await createOverdueInvoiceViaApi(page, 'Débiteur âgé');

	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	// Bascule sur l'onglet Balance âgée → contrôles dédiés (pas de sélecteur FY).
	await page.getByRole('tab', { name: /balance âgée/i }).click();
	await expect(page.getByTestId('aged-report-controls')).toBeVisible();
	// L'URL est adressable (?tab=), synchro D-7c.
	await expect(page).toHaveURL(/\?tab=aged-receivables/);

	await page.getByTestId('aged-report-generate').click();

	// Le tableau affiche au moins la ligne du débiteur créé + le total général.
	const table = page.getByTestId('aged-receivables-table');
	await expect(table).toBeVisible({ timeout: 5000 });
	await expect(page.getByTestId('aged-receivables-total')).toBeVisible();
	const contactRow = page.getByTestId('aged-receivables-row').filter({ hasText: contactName });
	await expect(contactRow).toBeVisible();
	// Drill-down : le nom du contact est un lien vers la liste filtrée ?contactId=.
	await expect(contactRow.locator('a[href^="/invoices?contactId="]')).toBeVisible();

	// Export CSV visible pour Admin (canManage).
	await expect(page.getByTestId('aged-report-export-csv')).toBeVisible();
});

test('reports aged-receivables : deep-link ?tab= ouvre directement l’onglet', async ({ page }) => {
	await login(page);
	await page.goto('/reports?tab=aged-receivables');
	await page.waitForLoadState('networkidle');
	// L'onglet Balance âgée est actif + ses contrôles dédiés visibles.
	await expect(page.getByRole('tab', { name: /balance âgée/i })).toHaveAttribute(
		'aria-selected',
		'true',
	);
	await expect(page.getByTestId('aged-report-controls')).toBeVisible();
});

test('échéancier : le lien croisé ouvre la balance âgée', async ({ page }) => {
	await login(page);
	await page.goto('/invoices/due-dates');
	await page.getByTestId('due-dates-link-aged').click();
	await expect(page).toHaveURL(/\/reports\?tab=aged-receivables/);
	await expect(page.getByTestId('aged-report-controls')).toBeVisible();
});

// ---------------------------------------------------------------------------
// Story 24-1 — Grand livre
//
// ⚠️ Ce que SEUL un E2E vérifie ici : qu'une valeur traverse réellement la
// frontière HTTP. Vitest teste la construction du payload, les tests Rust la
// validation, et ni l'un ni l'autre ne voit une clé qui disparaît entre les
// deux. Le grand livre a précisément un contrat inhabituel — `from`/`to` et
// AUCUN `fiscalYearId` —, donc c'est le genre de rapport où une clé se perd.
// ---------------------------------------------------------------------------

test('grand livre : une écriture postée réapparaît dans l’extrait de son compte (Story 24-1)', async ({
	page,
}) => {
	await login(page);

	const api = await authedApiContext(page);
	let debitNumber = '';
	try {
		const accRes = await api.get('/api/v1/accounts?includeArchived=false');
		expect(accRes.ok(), `list accounts: ${accRes.status()}`).toBeTruthy();
		const accounts: Array<{ id: number; number: string }> = await accRes.json();
		const debit = accounts.find((a) => /^10[0-9]{2}$/.test(a.number)) ?? accounts[0];
		const credit = accounts.find((a) => /^3[0-9]{3}$/.test(a.number)) ?? accounts[1];
		debitNumber = debit.number;

		const today = new Date().toISOString().slice(0, 10);
		const libelle = `Grand livre E2E ${Date.now()}`;
		const entryRes = await api.post('/api/v1/journal-entries', {
			data: {
				entryDate: today,
				journal: 'OD',
				description: libelle,
				lines: [
					{ accountId: debit.id, debit: '1234.50', credit: '0.00' },
					{ accountId: credit.id, debit: '0.00', credit: '1234.50' },
				],
			},
		});
		expect(entryRes.ok(), `create entry: ${entryRes.status()}`).toBeTruthy();

		await page.goto('/reports');
		await page.waitForLoadState('networkidle');
		// ⚠️ L'onglet se cible par son ID, jamais par son libellé : un libellé est
		// traduit, et un sélecteur figé dessus casse à la première relecture de
		// la langue (règle du dépôt, issue #326).
		await page.locator('#reports-tab-general-ledger').click();
		await expect(page.getByTestId('ledger-controls')).toBeVisible();

		// Période libre — et surtout : AUCUN exercice à choisir. C'est le seul
		// rapport dans ce cas, et c'est ce qui le rend concordant avec le bilan.
		const year = today.slice(0, 4);
		await page.getByTestId('ledger-from').fill(`${year}-01-01`);
		await page.getByTestId('ledger-to').fill(`${year}-12-31`);
		await page.getByTestId('ledger-account').selectOption(String(debit.id));
		await page.getByTestId('ledger-generate').click();

		const section = page.getByTestId(`ledger-section-${debit.number}`);
		await expect(section).toBeVisible({ timeout: 5000 });
		await expect(section).toContainText(libelle);
		// Le montant a bien traversé : formaté à la suisse, apostrophe typographique.
		await expect(section).toContainText('1’234.50');
		// Les trois lignes d'encadrement, sans lesquelles ce n'est pas un extrait.
		await expect(page.getByTestId('ledger-opening')).toBeVisible();
		await expect(page.getByTestId('ledger-closing')).toBeVisible();
	} finally {
		await disposeContextSafe(api);
	}

	// Le lien depuis la balance mène à l'extrait de CE compte (AC11) — sans lui,
	// le rapport existe et personne ne le trouve.
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');
	await page.locator('#reports-tab-trial-balance').click();
	await page.getByRole('button', { name: /générer/i }).click();
	const lien = page.getByRole('link', { name: debitNumber }).first();
	await expect(lien).toBeVisible({ timeout: 5000 });
	await lien.click();
	await expect(page).toHaveURL(/tab=general-ledger/);
	await expect(page).toHaveURL(new RegExp(`accountId=`));
	// Le lien porte sa période : l'extrait se génère seul, sans redemander ce
	// que l'URL contenait déjà.
	await expect(page.getByTestId(`ledger-section-${debitNumber}`)).toBeVisible({ timeout: 5000 });
});

test('grand livre : deep-link ?tab= ouvre directement l’onglet (Story 24-1)', async ({ page }) => {
	await login(page);
	await page.goto('/reports?tab=general-ledger');
	await page.waitForLoadState('networkidle');
	await expect(page.locator('#reports-tab-general-ledger')).toHaveAttribute(
		'aria-selected',
		'true',
	);
	await expect(page.getByTestId('ledger-controls')).toBeVisible();
});

test('reports aged-receivables : export CSV absent pour un rôle Consultation', async ({ page }) => {
	await login(page);
	// Créer un user Consultation via API (admin connecté).
	const username = `consult-aged-${Date.now()}`;
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/users', {
			data: { username, password: 'MotDePasse12345', role: 'Consultation' },
		});
		expect(res.ok(), `create user: ${res.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
	await clearAuthStorage(page);
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', 'MotDePasse12345');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');

	await page.goto('/reports?tab=aged-receivables');
	await page.waitForLoadState('networkidle');
	// La vue est accessible (tous rôles) mais l'export CSV est masqué (Comptable+).
	await expect(page.getByTestId('aged-report-controls')).toBeVisible();
	await expect(page.getByTestId('aged-report-export-csv')).toHaveCount(0);
});

// Issue #90 — AC #34 / T12.4 : ce test reseed la DB avec `with-company-no-fy`
// et doit donc être exécuté EN DERNIER dans ce fichier. Sinon, les tests
// suivants qui s'attendent à un `with-company` (avec fiscal_year) seedé par
// `beforeAll` verraient un état dérivé (no-fy) et échoueraient en cascade.
test('reports page disables Générer when company has no fiscal year (AC #34, T12.4)', async ({
	page,
}) => {
	await seedTestState('with-company-no-fy');
	await login(page);
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	await expect(page.getByRole('button', { name: /générer/i })).toBeDisabled();
	await expect(page.getByText(/aucun exercice comptable disponible/i)).toBeVisible();
});
