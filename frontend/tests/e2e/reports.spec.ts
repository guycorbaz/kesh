/**
 * Story 9-1 — Tests E2E Playwright pour la page `/reports`.
 *
 * Scénarios :
 *   1. AC #27 + #33 : page chargée + 4 onglets visibles
 *   2. AC #28 : génération bilan via UI
 *   3. AC #34 : axe a11y scan zero violations
 *
 * Pré-requis :
 *   - MariaDB up + KESH_TEST_MODE=true.
 *   - Seed `with-company` (1000-4000 comptes + fiscal_year).
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

test('reports page has zero axe a11y violations', async ({ page }) => {
	await login(page);
	await page.goto('/reports');
	await page.waitForLoadState('networkidle');

	const results = await new AxeBuilder({ page })
		.withTags(['wcag2a', 'wcag2aa'])
		.analyze();
	expect(results.violations).toEqual([]);
});
