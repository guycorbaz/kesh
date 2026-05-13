/**
 * Story 8-5b T7 — Tests E2E Playwright pour `/reconciliation/rules`.
 *
 * 2 scénarios :
 *   1. Créer une rule via UI + vérifier qu'elle apparaît dans la liste (AC #101 + UI).
 *   2. Accessibility — axe scan zero violations sur le RuleFormModal (AC #124).
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true + seed `with-company`.
 * Playwright browsers : PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64.
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

test('reconciliation-rules create rule end-to-end', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/rules');
	await expect(page.locator('h1')).toContainText('Règles');

	// Ouvrir le form.
	await page.getByTestId('rules-create-button').click();
	await expect(page.getByTestId('rule-form-modal')).toBeVisible();

	// Remplir.
	await page.getByTestId('rule-form-label').fill('Swisscom auto');
	await page
		.getByTestId('rule-form-match-type')
		.selectOption({ value: 'counterparty_contains' });
	await page.getByTestId('rule-form-match-value').fill('Swisscom');

	// Sélectionner le premier compte Expense disponible.
	const counterpartySelect = page.getByTestId('rule-form-counterparty');
	const options = await counterpartySelect.locator('option').allTextContents();
	const firstExpense = options.find((o) => o && o !== '—');
	expect(firstExpense, 'Au moins un compte de contrepartie doit être dispo').toBeTruthy();
	await counterpartySelect.selectOption({ label: firstExpense! });

	// Submit.
	await page.getByTestId('rule-form-submit').click();

	// La rule apparaît dans la liste.
	await expect(page.getByTestId('rules-list')).toBeVisible();
	await expect(
		page.locator('[data-testid^="rule-row-"]').filter({ hasText: 'Swisscom auto' }),
	).toBeVisible();
});

test('accessibility — RuleFormModal axe scan zero violations', async ({ page }) => {
	await login(page);
	await page.goto('/reconciliation/rules');
	await expect(page.locator('h1')).toContainText('Règles');
	await page.getByTestId('rules-create-button').click();
	await expect(page.getByTestId('rule-form-modal')).toBeVisible();

	const results = await new AxeBuilder({ page })
		.include('[data-testid="rule-form-modal"]')
		.analyze();
	expect(
		results.violations,
		`axe violations: ${JSON.stringify(results.violations, null, 2)}`,
	).toEqual([]);
});
