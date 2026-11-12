import { test, expect } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Story 34-1 (#966) — l'étape bancaire de l'assistant pose la description et le compte
 * comptable lié du compte bancaire.
 *
 * Le compte bancaire de l'assistant naissait toujours sans lien : la première
 * réconciliation tombait sur `BANK_ACCOUNT_NOT_CONFIGURED`. Cette spec joue l'étape par
 * l'écran — le 1020 présélectionné, une description saisie — puis relit le compte par
 * `GET /api/v1/bank-accounts` : la description et le compte lié doivent **traverser la
 * frontière HTTP**, ce qu'aucun test Vitest ni Rust ne voit à lui seul.
 *
 * L'état est amené à l'étape 6 **par l'API** (patron `onboarding-motif.spec.ts`) ; l'écran
 * se joue par sélecteurs d'`id` (aucun sélecteur de texte). `onboarding-path-b.spec.ts`,
 * rouge à l'étape bancaire depuis avril (KF-029, #97), n'est pas un appui.
 *
 * Données synthétiques : IBAN de démonstration de la suite, société fictive.
 */

test.describe('Assistant — compte bancaire lié dès l’installation (#966)', () => {
	test.beforeEach(async ({ page }) => {
		await seedTestState('fresh');
		await page.goto('/login');
		await page.fill('#username', 'changeme');
		await page.fill('#password', 'changeme');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL(/\/onboarding/);

		const ctx = await authedApiContext(page);
		try {
			const etapes: [string, Record<string, unknown> | undefined][] = [
				['/api/v1/onboarding/language', { language: 'FR' }],
				['/api/v1/onboarding/mode', { mode: 'guided' }],
				['/api/v1/onboarding/start-production', undefined],
				['/api/v1/onboarding/org-type', { orgType: 'Pme' }],
				['/api/v1/onboarding/accounting-language', { language: 'FR' }],
				[
					'/api/v1/onboarding/coordinates',
					{
						name: 'Exemple Banque SA',
						address: {
							street: 'Rue du Lac',
							building: '12',
							postalCode: '1003',
							city: 'Lausanne',
							country: 'CH',
						},
						ideNumber: null,
					},
				],
			];
			for (const [chemin, data] of etapes) {
				const res = await ctx.post(chemin, data ? { data } : {});
				expect(res.status(), chemin).toBe(200);
			}
		} finally {
			await disposeContextSafe(ctx);
		}
		// Rechargement complet : l'état se relit, étape 6 (compte bancaire).
		await page.goto('/onboarding');
		await expect(page.locator('#bank-name')).toBeVisible();
	});

	test.afterEach(async ({ page }) => {
		await clearAuthStorage(page);
	});

	test('1020 présélectionné, description saisie : le compte créé les porte', async ({ page }) => {
		// La présélection est un geste d'écran : on attend qu'elle soit faite.
		await expect(page.locator('#bank-journal-account option:checked')).toContainText('1020');
		await expect(page.getByTestId('onboarding-bank-editable-later')).toBeVisible();

		await page.fill('#bank-name', 'Banque Exemple');
		await page.fill('#bank-iban', 'CH93 0076 2011 6238 5295 7');
		await page.fill('#bank-description', 'Compte courant');
		await page.locator('form button[type="submit"]').click();
		await expect(page).toHaveURL(/\/$/);

		const ctx = await authedApiContext(page);
		try {
			const comptes = await ctx.get('/api/v1/accounts?includeArchived=false');
			expect(comptes.status()).toBe(200);
			const plan = (await comptes.json()) as { id: number; number: string }[];
			const banque = plan.find((a) => a.number === '1020');
			expect(banque, 'le plan PME porte le 1020').toBeDefined();

			const res = await ctx.get('/api/v1/bank-accounts');
			expect(res.status()).toBe(200);
			const liste = (await res.json()) as {
				description: string | null;
				journalAccountId: number | null;
				isPrimary: boolean;
			}[];
			expect(liste).toHaveLength(1);
			expect(liste[0].description).toBe('Compte courant');
			expect(liste[0].journalAccountId).toBe(banque!.id);
			expect(liste[0].isPrimary).toBe(true);
		} finally {
			await disposeContextSafe(ctx);
		}
	});
});
