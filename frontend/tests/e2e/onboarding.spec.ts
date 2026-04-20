import { test, expect } from '@playwright/test';
import { seedTestState } from './helpers/test-state';

/**
 * Tests E2E — Flux d'onboarding Chemin A (Story 2.2)
 *
 * Prérequis backend (Story 6.4) : `KESH_TEST_MODE=true`. Le `beforeEach`
 * seed le preset `fresh` (uniquement user `changeme/changeme`, aucune
 * company, aucun onboarding_state) → chaque test démarre d'un onboarding
 * vierge, évitant la mutation irréversible du singleton `onboarding_state`.
 */

test.describe('Onboarding Wizard', () => {
	test.beforeEach(async ({ page }) => {
		// Clear localStorage to isolate each test and prevent token bleed from previous tests
		await page.context().clearCookies();

		// Reset DB + user `changeme` seul (preset fresh, cf. AC #7).
		await seedTestState('fresh');

		// Login en tant que changeme/changeme (le seul user du preset fresh).
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');

		// Le guard onboarding devrait rediriger vers /onboarding
		await expect(page).toHaveURL(/\/onboarding/);
	});

	test('étape 1 — choix de langue affiche 4 options', async ({ page }) => {
		// 4 boutons de langue
		const langButtons = page.locator('button:has-text("Français"), button:has-text("Deutsch"), button:has-text("Italiano"), button:has-text("English")');
		await expect(langButtons).toHaveCount(4);
	});

	test('flux complet : langue → mode → démo → bannière visible', async ({ page }) => {
		// Étape 1 : Choisir français
		await page.click('button:has-text("Français")');

		// Étape 2 : Choisir mode guidé
		await expect(page.getByText('Guidé')).toBeVisible();
		await expect(page.getByText('Expert')).toBeVisible();
		await page.click('button:has-text("Guidé")');

		// Étape 3 : Choisir démo
		await expect(page.getByText('Explorer avec des données de démo')).toBeVisible();
		await page.click('button:has-text("Explorer avec des données de démo")');

		// Redirect vers / avec bannière démo
		await expect(page).toHaveURL('/');
		await expect(page.getByText('Instance de démonstration')).toBeVisible();
	});

	test('bannière démo — reset redirige vers onboarding', async ({ page }) => {
		// Complete onboarding first
		await page.click('button:has-text("Français")');
		await page.click('button:has-text("Guidé")');
		await page.click('button:has-text("Explorer avec des données de démo")');
		await expect(page).toHaveURL('/');

		// Click reset
		await page.click('button:has-text("Réinitialiser pour la production")');

		// Confirm dialog
		await expect(page.getByText('Toutes les données de démonstration')).toBeVisible();
		await page.click('button:has-text("Confirmer")');

		// Should redirect to onboarding
		await expect(page).toHaveURL(/\/onboarding/);
	});
});

test.describe('Onboarding — Reprise après interruption', () => {
	test('F5 à étape 2 reprend à étape 2', async ({ page }) => {
		// Login
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL(/\/onboarding/);

		// Complete step 1
		await page.click('button:has-text("Français")');

		// Should be at step 2 now
		await expect(page.getByText('Guidé')).toBeVisible();

		// Simulate refresh (F5)
		await page.reload();

		// Should still be at step 2
		await expect(page.getByText('Guidé')).toBeVisible();
	});
});
