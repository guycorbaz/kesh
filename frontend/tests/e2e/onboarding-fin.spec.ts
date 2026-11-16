import { test, expect, type Page, type Route } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Story 36-17 (#1187) — l'assistant de configuration ne laisse plus de page blanche à la fin.
 *
 * Avant la story, la page de l'assistant n'avait aucun écran aux étapes 7 et 8 : après
 * l'étape du compte bancaire, la finalisation tournait sur une page vide (le seul logo),
 * et l'utilisateur devait recharger pour entrer dans Kesh. Ici, l'assistant est joué À
 * L'INTERFACE jusqu'à l'étape bancaire, et **aucun `page.reload()` ni `page.goto()`**
 * n'intervient entre le clic bancaire et la page « Configuration terminée » (E1, E2).
 *
 * Sélecteurs : `id` et `data-testid` seulement — jamais une sous-chaîne d'un libellé
 * (« Configuration incomplète » commence aussi un message du serveur et un titre du
 * formulaire de facture, V3-6 de la validation).
 *
 * La finalisation est retenue par `page.route` sur une promesse, sans délai fixe : le test
 * attend l'écran « en cours » VISIBLE avant de la libérer. Le client ne rejoue pas seul un
 * `POST` (`api-client.ts`, retry réservé aux méthodes idempotentes) : une requête
 * interrompue par `route.abort()` n'a jamais atteint le serveur, l'état reste 7.
 *
 * Données synthétiques : société fictive, IBAN de démonstration de la suite.
 */

const FINALIZE = '**/api/v1/onboarding/finalize';

async function connexion(page: Page) {
	await seedTestState('fresh');
	await page.goto('/login');
	await page.fill('#username', 'changeme');
	await page.fill('#password', 'changeme');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL(/\/onboarding/);
}

/** L'assistant joué à l'interface, de la langue aux coordonnées (étape 6 atteinte). */
async function jouerJusquAuCompteBancaire(page: Page) {
	await page.getByTestId('onboarding-language-FR').click();
	await page.getByTestId('onboarding-mode-guided').click();
	await page.getByTestId('onboarding-path-production').click();
	await page.getByTestId('onboarding-org-type-pme').click();
	await page.getByTestId('onboarding-accounting-lang-FR').click();
	await expect(page.getByTestId('onboarding-screen-5')).toBeVisible();
	await page.fill('#coord-name', 'Exemple Conseil Sàrl');
	await page.fill('#coord-street', 'Rue du Lac');
	await page.fill('#coord-npa', '1003');
	await page.fill('#coord-city', 'Lausanne');
	await page.getByTestId('onboarding-coordinates-submit').click();
	await expect(page.getByTestId('onboarding-screen-6')).toBeVisible();
	// La liste des comptes liables chargée : l'envoi est permis (Story 34-1).
	await expect(page.getByTestId('onboarding-bank-save')).toBeEnabled();
}

async function etapeServeur(page: Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.get('/api/v1/onboarding/state');
		expect(res.status()).toBe(200);
		return ((await res.json()) as { stepCompleted: number }).stepCompleted;
	} finally {
		await disposeContextSafe(ctx);
	}
}

test.describe('Assistant — la fin de la configuration (#1187)', () => {
	test.afterEach(async ({ page }) => {
		await clearAuthStorage(page);
	});

	test('E1 — compte saisi : page terminée sans rechargement, puis Commencer', async ({ page }) => {
		await connexion(page);
		await jouerJusquAuCompteBancaire(page);
		await page.fill('#bank-name', 'Exemple Banque SA');
		await page.fill('#bank-iban', 'CH93 0076 2011 6238 5295 7');
		await page.fill('#bank-description', 'Compte courant');
		await page.getByTestId('onboarding-bank-save').click();

		await expect(page.getByTestId('onboarding-screen-done')).toBeVisible();
		await expect(page).toHaveURL(/\/onboarding$/);
		const recap = page.getByTestId('onboarding-done-summary');
		await expect(recap).toContainText('Exemple Conseil Sàrl');
		await expect(recap).toContainText('Exemple Banque SA');
		await expect(recap).toContainText('Compte courant');
		await expect(recap).not.toContainText('CH93');
		await expect(page.getByTestId('onboarding-done-todo-bank')).toHaveCount(0);
		expect(await etapeServeur(page)).toBe(8);

		await page.getByTestId('onboarding-done-start').click();
		await expect(page).toHaveURL(/\/$/);
		await expect(page.getByTestId('nav-group-quotidien')).toBeVisible();
		await expect(page.getByTestId('incomplete-config-banner')).toHaveCount(0);
	});

	test('E2 — compte passé : finalisation retenue, interrompue, puis Réessayer', async ({ page }) => {
		await connexion(page);
		await jouerJusquAuCompteBancaire(page);

		let liberer: () => void = () => {};
		const retenue = new Promise<void>((r) => {
			liberer = r;
		});
		await page.route(
			FINALIZE,
			async (route: Route) => {
				await retenue;
				await route.abort();
			},
			{ times: 1 }
		);
		await page.getByTestId('onboarding-skip-bank').click();
		await expect(page.getByTestId('onboarding-screen-finalizing')).toBeVisible();
		liberer();
		await expect(page.getByTestId('onboarding-screen-finalize-error')).toBeVisible();
		await expect(page.getByTestId('onboarding-finalize-motif')).not.toBeEmpty();
		await expect(page.getByTestId('onboarding-finalize-remedy-link')).toHaveCount(0);
		expect(await etapeServeur(page)).toBe(7);

		await page.unroute(FINALIZE);
		await page.getByTestId('onboarding-finalize-retry').click();
		await expect(page.getByTestId('onboarding-screen-done')).toBeVisible();
		await expect(page).toHaveURL(/\/onboarding$/);
		await expect(page.getByTestId('onboarding-done-todo-bank')).toBeVisible();
		expect(await etapeServeur(page)).toBe(8);
	});

	test('E3 — reprise : étape 7 au rechargement, Terminer la configuration', async ({ page }) => {
		await connexion(page);
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
						name: 'Exemple Conseil Sàrl',
						address: { street: 'Rue du Lac', building: '12', postalCode: '1003', city: 'Lausanne', country: 'CH' },
						ideNumber: null,
					},
				],
				['/api/v1/onboarding/skip-bank', undefined],
			];
			for (const [chemin, data] of etapes) {
				const res = await ctx.post(chemin, data ? { data } : {});
				expect(res.status(), chemin).toBe(200);
			}
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.goto('/onboarding');
		await expect(page).toHaveURL(/\/onboarding$/);
		await expect(page.getByTestId('onboarding-screen-resume')).toBeVisible();
		expect(await etapeServeur(page)).toBe(7);

		await page.getByTestId('onboarding-resume-finish').click();
		await expect(page.getByTestId('onboarding-screen-done')).toBeVisible();
		await page.reload();
		await expect(page.getByTestId('onboarding-screen-done')).toBeVisible();
		await expect(page.getByTestId('onboarding-done-summary')).toContainText('Exemple Conseil Sàrl');
	});

	test('E4 — démo : indicateur pendant le chargement, puis Kesh', async ({ page }) => {
		await connexion(page);
		await page.getByTestId('onboarding-language-FR').click();
		await page.getByTestId('onboarding-mode-guided').click();

		let liberer: () => void = () => {};
		const retenue = new Promise<void>((r) => {
			liberer = r;
		});
		await page.route(
			'**/api/v1/onboarding/seed-demo',
			async (route: Route) => {
				await retenue;
				await route.continue();
			},
			{ times: 1 }
		);
		await page.getByTestId('onboarding-path-demo').click();
		await expect(page.getByTestId('onboarding-screen-demo-loading')).toBeVisible();
		liberer();
		await expect(page).toHaveURL(/\/$/);
		await expect(page.getByTestId('nav-group-quotidien')).toBeVisible();
	});
});
