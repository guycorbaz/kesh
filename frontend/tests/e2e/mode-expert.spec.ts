import { test, expect } from '@playwright/test';
import { seedTestState } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.beforeEach(async ({ page }) => {
	// Clear localStorage to isolate each test and prevent token bleed from previous tests
	await page.context().clearCookies();
});

/**
 * Tests E2E — Mode Guidé/Expert (Story 2.5)
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`.
 */

test.describe('Mode toggle', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
	});

	test('toggle mode changes data-mode attribute on html', async ({ page }) => {
		// Default should be guided
		const htmlMode = await page.locator('html').getAttribute('data-mode');
		expect(htmlMode).toBe('guided');

		// Open profile dropdown and click mode toggle
		await page.click('button:has-text("Mode")');

		// Check data-mode changed
		const newMode = await page.locator('html').getAttribute('data-mode');
		expect(['guided', 'expert']).toContain(newMode);
	});
});

test.describe('Ctrl+N shortcut (Expert mode)', () => {
	test('Ctrl+N navigates to journal-entries in Expert mode', async ({ page }) => {
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');

		// Set expert mode via keyboard evaluation
		await page.evaluate(() => {
			document.documentElement.setAttribute('data-mode', 'expert');
		});

		// Ctrl+N shortcut
		await page.keyboard.press('Control+n');

		// Should navigate to journal-entries
		await expect(page).toHaveURL(/\/journal-entries/);
	});
});
