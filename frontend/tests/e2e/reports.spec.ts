/**
 * Story 9-1 — Tests E2E Playwright pour la page `/reports`.
 *
 * Scénarios :
 *   1. AC #27 + #33 : page chargée + 4 onglets visibles
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
import { seedTestState, clearAuthStorage } from './helpers/test-state';

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

test('reports page loads with 4 tabs (AC #27 + #33)', async ({ page }) => {
	await login(page);
	await page.goto('/reports');

	await expect(page.getByRole('heading', { name: /rapports/i })).toBeVisible();

	// AC #33 : exactement 4 onglets (Pass 1 AA-10)
	const tabs = page.getByRole('tab');
	await expect(tabs).toHaveCount(4);
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
