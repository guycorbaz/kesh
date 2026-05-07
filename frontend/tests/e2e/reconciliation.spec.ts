/**
 * Story 8-4 T8 — Tests E2E Playwright pour la réconciliation
 * automatique (FR44).
 *
 * 2 scénarios actifs + 1 a11y :
 *   1. reconciliation page renders empty state quand pas de tx pending
 *   2. reconciliation page accessibility — axe scan zero violations
 *
 * Note : les scénarios accept/reject end-to-end nécessitent un setup
 * de fixtures (invoice validée + tx CAMT.053 importée + matching candidate)
 * qui sort du scope d'un seul fichier. Couvert par les tests E2E HTTP
 * Rust (`crates/kesh-api/tests/reconciliation_e2e.rs` — différé, cf.
 * Completion Notes story 8-4 §dette-test-e2e-http).
 *
 * Pré-requis :
 *   - MariaDB up + KESH_TEST_MODE=true.
 *   - Onboarding avec au moins 1 bank_account.
 *   - Playwright browsers (Ubuntu 26.04 : PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64).
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

test('reconciliation page renders empty state when no pending transactions', async ({
	page,
}) => {
	await login(page);
	await page.goto('/reconciliation');
	// Le sélecteur de compte bancaire OU l'état "no account" doit apparaître.
	const accountSelect = page.getByTestId('bank-account-select');
	const noAccount = page.getByTestId('reconciliation-no-account');
	await expect(accountSelect.or(noAccount)).toBeVisible();
	if (await accountSelect.isVisible()) {
		// Empty state attendu : pas de transactions pending sur compte fraichement seedé.
		await expect(page.getByTestId('reconciliation-empty')).toBeVisible({
			timeout: 5000,
		});
	}
});

test('reconciliation page accessibility — axe scan zero violations', async ({
	page,
}) => {
	await login(page);
	await page.goto('/reconciliation');
	// Wait for page to be in a stable state.
	await page
		.getByTestId('bank-account-select')
		.or(page.getByTestId('reconciliation-no-account'))
		.first()
		.waitFor();
	const results = await new AxeBuilder({ page })
		.withTags(['wcag2a', 'wcag2aa'])
		.analyze();
	expect(results.violations).toEqual([]);
});
