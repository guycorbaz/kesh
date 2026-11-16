import { test, expect } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Story 36-19 (#1200) — la finalisation de l'assistant complète les réglages de
 * facturation créés par un écran ouvert avant elle.
 *
 * Le journal (`/journal-entries`) lit les réglages de facturation dès son ouverture
 * (`getInvoiceSettings()`), ce qui crée une ligne de réglages sans aucun compte. La
 * finalisation refusait ensuite (« 1100, 3000 manquants ») alors que les comptes
 * existent. Cette spec joue le geste par le navigateur — journal ouvert à l'étape du
 * compte bancaire, retour à l'assistant, *Configurer plus tard* —, puis relit le
 * résultat **par l'API** : l'étape et les comptes désignés doivent traverser la
 * frontière HTTP.
 *
 * ⚠️ Aucune assertion sur l'écran d'après la finalisation, que la Story 36-17 change
 * (C-36-19-6). L'état est amené à l'étape 6 par l'API (patron
 * `onboarding-bank-account.spec.ts`).
 *
 * Données synthétiques : société fictive, plan PME livré.
 */

test.describe('Assistant — réglages de facturation ouverts avant la fin (#1200)', () => {
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
						name: 'Exemple Réglages SA',
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
	});

	test.afterEach(async ({ page }) => {
		await clearAuthStorage(page);
	});

	test('journal ouvert à l’étape 6, puis « Configurer plus tard » : comptes désignés', async ({
		page,
	}) => {
		// Le journal lit les réglages : c'est ce GET qui crée la ligne vide.
		const reglages = page.waitForResponse(
			(r) =>
				r.url().includes('/api/v1/company/invoice-settings') && r.request().method() === 'GET'
		);
		await page.goto('/journal-entries');
		expect((await reglages).status()).toBe(200);

		await page.goto('/onboarding');
		await expect(page.locator('#bank-name')).toBeVisible();
		await page.locator('form button[type="button"]').click();

		const ctx = await authedApiContext(page);
		try {
			await expect
				.poll(async () => {
					const res = await ctx.get('/api/v1/onboarding/state');
					return ((await res.json()) as { stepCompleted: number }).stepCompleted;
				})
				.toBe(8);

			const comptes = await ctx.get('/api/v1/accounts?includeArchived=false');
			expect(comptes.status()).toBe(200);
			const plan = (await comptes.json()) as { id: number; role: string | null }[];
			const debiteurs = plan.find((a) => a.role === 'Receivable');
			const produit = plan.find((a) => a.role === 'DefaultRevenue');
			expect(debiteurs, 'le plan PME porte le rôle Créances clients').toBeDefined();
			expect(produit, 'le plan PME porte le rôle Produit par défaut').toBeDefined();

			const res = await ctx.get('/api/v1/company/invoice-settings');
			expect(res.status()).toBe(200);
			const s = (await res.json()) as {
				defaultReceivableAccountId: number | null;
				defaultRevenueAccountId: number | null;
			};
			expect(s.defaultReceivableAccountId).toBe(debiteurs!.id);
			expect(s.defaultRevenueAccountId).toBe(produit!.id);
		} finally {
			await disposeContextSafe(ctx);
		}
	});
});
