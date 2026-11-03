import { test, expect } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Story v017-2 (#809) — l'assistant d'installation affiche le motif d'un refus du serveur.
 *
 * Le 2026-10-28, à l'installation neuve de la v0.16.0, « Suisse » saisi au champ Pays n'a
 * produit que « Erreur lors de la sauvegarde des coordonnées » : le serveur disait
 * pourtant « le pays doit être un code ISO à 2 lettres (ex. CH) ». Cette spec vérifie que
 * le motif traverse la frontière HTTP jusqu'au toast, puis qu'une saisie corrigée passe.
 *
 * L'état est amené à l'étape 5 **par l'API** ; seul l'écran des coordonnées se joue par
 * l'interface, par sélecteurs d'`id` (aucun sélecteur de texte : la garde
 * `e2e-selecteurs-traduits.test.ts`).
 */

test.describe('Assistant — le motif du refus (#809)', () => {
	test.beforeEach(async ({ page }) => {
		await seedTestState('fresh');
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL(/\/onboarding/);

		const ctx = await authedApiContext(page);
		try {
			const etapes: [string, Record<string, string> | undefined][] = [
				['/api/v1/onboarding/language', { language: 'FR' }],
				['/api/v1/onboarding/mode', { mode: 'guided' }],
				['/api/v1/onboarding/start-production', undefined],
				['/api/v1/onboarding/org-type', { orgType: 'Independant' }],
				['/api/v1/onboarding/accounting-language', { language: 'FR' }],
			];
			for (const [chemin, data] of etapes) {
				const res = await ctx.post(chemin, data ? { data } : {});
				expect(res.status(), chemin).toBe(200);
			}
		} finally {
			await disposeContextSafe(ctx);
		}
		// Rechargement complet : l'état se relit, étape 5 (coordonnées).
		await page.goto('/onboarding');
		await expect(page.locator('#coord-country')).toBeVisible();
	});

	test.afterEach(async ({ page }) => {
		await clearAuthStorage(page);
	});

	test('pays en toutes lettres : le motif du serveur s’affiche, puis CH passe', async ({ page }) => {
		await page.fill('#coord-name', 'Exemple Sàrl');
		await page.fill('#coord-npa', '1000');
		await page.fill('#coord-city', 'Lausanne');
		await page.fill('#coord-country', 'Suisse');
		await page.locator('form button[type="submit"]').click();

		await expect(page.locator('[data-sonner-toast]')).toContainText('code ISO à 2 lettres');
		await expect(page.locator('#coord-country')).toBeVisible();

		await page.fill('#coord-country', 'CH');
		await page.locator('form button[type="submit"]').click();
		await expect(page.locator('#bank-name')).toBeVisible();
	});
});
