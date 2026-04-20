import { test, expect } from '@playwright/test';
import { seedTestState } from './helpers/test-state';

/**
 * Tests E2E — Flux d'onboarding Chemin B (Story 2.3)
 *
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`. Le `beforeEach`
 * re-seed le preset `fresh` avant chaque test (onboarding mute le singleton
 * `onboarding_state` de façon irréversible dans le run).
 */

test.describe('Onboarding Path B', () => {
	test.beforeEach(async ({ page }) => {
		// Clear localStorage to isolate each test and prevent token bleed from previous tests
		await page.context().clearCookies();

		await seedTestState('fresh');
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL(/\/onboarding/);
	});

	test('flux complet Path B : langue → mode → production → org → accounting → coords → bank', async ({ page }) => {
		// Step 1: Language
		await page.click('button:has-text("Français")');

		// Step 2: Mode
		await page.click('button:has-text("Guidé")');

		// Step 3: Production path
		await page.click('button:has-text("Configurer pour la production")');

		// Step 4: Org type
		await expect(page.getByText('Indépendant')).toBeVisible();
		await page.click('button:has-text("PME")');

		// Step 5: Accounting language
		await expect(page.getByText('Langue comptable')).toBeVisible();
		await page.click('button:has-text("Français")');

		// Step 6: Coordinates
		await expect(page.getByText('Coordonnées')).toBeVisible();
		await page.fill('#coord-name', 'Ma Société SA');
		await page.fill('#coord-address', 'Rue du Test 1, 1000 Lausanne');
		await page.click('button:has-text("Continuer")');

		// Step 7: Bank (skip)
		await expect(page.getByText('Compte bancaire')).toBeVisible();
		await page.click('button:has-text("Configurer plus tard")');

		// Should be in app with blue banner
		await expect(page).toHaveURL('/');
		await expect(page.getByText('Configuration incomplète')).toBeVisible();
	});

	test('flux Path B avec banque configurée → pas de bannière bleue', async ({ page }) => {
		// Steps 1-6 same as above
		await page.click('button:has-text("Français")');
		await page.click('button:has-text("Expert")');
		await page.click('button:has-text("Configurer pour la production")');
		await page.click('button:has-text("Association")');
		await page.click('button:has-text("Français")');
		await page.fill('#coord-name', 'Mon Association');
		await page.fill('#coord-address', 'Rue 1');
		await page.click('button:has-text("Continuer")');

		// Step 7: Bank (fill)
		await page.fill('#bank-name', 'UBS');
		await page.fill('#bank-iban', 'CH93 0076 2011 6238 5295 7');
		await page.click('button:has-text("Enregistrer")');

		// Should be in app WITHOUT blue banner
		await expect(page).toHaveURL('/');
		await expect(page.getByText('Configuration incomplète')).not.toBeVisible();
	});
});
