import { test, expect } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Flux d'onboarding Chemin B (Story 2.3)
 *
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`. Le `beforeEach`
 * re-seed le preset `fresh` avant chaque test (onboarding mute le singleton
 * `onboarding_state` de façon irréversible dans le run).
 */

test.describe('Onboarding Path B', () => {
	test.beforeEach(async ({ page }) => {
		await seedTestState('fresh');
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL(/\/onboarding/);
	});

	test.afterEach(async ({ page }) => {
		// Clear localStorage after each test to prevent token bleed to next test
		await clearAuthStorage(page);
	});

	test('flux complet Path B : langue → mode → production → org → accounting → coords → bank', async ({ page }) => {
		// KF-032 (#124) : ce test attend `incomplete-config-banner` après skip-bank,
		// mais la bannière ne s'affiche qu'à stepCompleted===6 alors que skip-bank
		// finalise (step 7/8) → jamais affichée. Nudge « config incomplète » basé sur
		// l'absence de compte bancaire jamais implémenté. Décision produit requise.
		test.fixme(true, 'KF-032 (#124) — bannière config-incomplète jamais affichée après finalize');
		// Step 1: Language
		await page.click('button:has-text("Français")');

		// Step 2: Mode
		await page.click('button:has-text("Guidé")');

		// Step 3: Production path
		await page.click('button:has-text("Configurer pour la production")');

		// Step 4: Org type — testid scopé sur le bouton (titre h2 contient aussi "Type d'organisation")
		await expect(page.locator('[data-testid="onboarding-org-type-independant"]')).toBeVisible();
		await page.locator('[data-testid="onboarding-org-type-pme"]').click();

		// Step 5: Accounting language
		await expect(page.getByRole('heading', { name: 'Langue comptable' })).toBeVisible();
		await page.click('button:has-text("Français")');

		// Step 6: Coordinates
		await expect(page.getByRole('heading', { name: 'Coordonnées' })).toBeVisible();
		await page.fill('#coord-name', 'Ma Société SA');
		await page.fill('#coord-address', 'Rue du Test 1, 1000 Lausanne');
		await page.click('button:has-text("Continuer")');

		// Step 7: Bank (skip)
		await expect(page.getByRole('heading', { name: 'Compte bancaire' })).toBeVisible();
		await page.click('button:has-text("Configurer plus tard")');

		// Should be in app with blue banner
		await expect(page).toHaveURL('/');
		await expect(page.locator('[data-testid="incomplete-config-banner"]')).toBeVisible();
	});

	test('flux Path B avec banque configurée → pas de bannière bleue', async ({ page }) => {
		// Steps 1-6 same as above
		await page.click('button:has-text("Français")');
		await page.click('button:has-text("Expert")');
		await page.click('button:has-text("Configurer pour la production")');
		await page.locator('[data-testid="onboarding-org-type-association"]').click();
		await page.click('button:has-text("Français")');
		await page.fill('#coord-name', 'Mon Association');
		await page.fill('#coord-address', 'Rue 1');
		await page.click('button:has-text("Continuer")');

		// Step 7: Bank (fill)
		await page.fill('#bank-name', 'UBS');
		await page.fill('#bank-iban', 'CH93 0076 2011 6238 5295 7');
		// Le bouton submit de l'étape banque réutilise la clé i18n `onboarding-next`
		// (valeur FR « Continuer », pas le fallback « Enregistrer »). Cf. KF-032 (#124).
		await page.click('button:has-text("Continuer")');

		// Should be in app WITHOUT blue banner
		await expect(page).toHaveURL('/');
		await expect(page.locator('[data-testid="incomplete-config-banner"]')).not.toBeVisible();
	});

	test('fresh-install : bannière stub visible au départ, disparaît après coordonnées (#120)', async ({
		page,
	}) => {
		// Le preset `fresh` seede une company stub (is_stub=TRUE) — comme le bootstrap
		// sur DB vide. La bannière de nudge doit être visible dès le début du wizard.
		await expect(page.locator('[data-testid="onboarding-stub-notice"]')).toBeVisible();

		// Wizard production jusqu'aux coordonnées.
		await page.click('button:has-text("Français")');
		await page.click('button:has-text("Guidé")');
		await page.click('button:has-text("Configurer pour la production")');
		await page.locator('[data-testid="onboarding-org-type-pme"]').click();
		await page.click('button:has-text("Français")');
		await page.fill('#coord-name', 'Vraie Société SA');
		await page.fill('#coord-address', 'Rue du Test 1, 1000 Lausanne');
		await page.click('button:has-text("Continuer")');

		// À l'étape banque (step 7), is_stub est repassé FALSE → bannière disparue.
		await expect(page.getByRole('heading', { name: 'Compte bancaire' })).toBeVisible();
		await expect(page.locator('[data-testid="onboarding-stub-notice"]')).not.toBeVisible();

		// Skip bank → app accessible.
		await page.click('button:has-text("Configurer plus tard")');
		await expect(page).toHaveURL('/');
	});
});
