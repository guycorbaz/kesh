import { expect, test, type Page } from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

/**
 * Tests E2E — une erreur d'API s'affiche par son message (Story 28-2, #520).
 *
 * # Ce que ces tests prouvent, et que rien d'autre ne prouve
 *
 * Les Vitest de la story simulent le client : ils prouvent qu'un objet
 * `ApiError` donné au composant s'affiche par son message. Ici, la réponse
 * **traverse réellement** le client HTTP (`parseErrorResponse`,
 * `toFetchApiError`) avant d'atteindre l'écran — c'est le seul endroit où une
 * `ApiError` objet simple, et non une `Error`, arrive au `catch`.
 *
 * Aucune écriture dans la base partagée : les réponses sont **interceptées**
 * (`page.route`).
 *
 * # Ce que le scénario 2 n'établit pas (C-28-2-23)
 *
 * L'instance E2E tourne en `fr-CH`, où le texte servi d'`error-network` égale
 * son repli : l'assertion ne distingue pas un message traduit d'un littéral
 * français conservé. Elle prouve la traversée (aucun « [object Object] ») ; la
 * traduction n'est prouvée que par les tests unitaires au catalogue allemand.
 *
 * # Recette visuelle (action B2 de la rétrospective de l'Epic 27)
 *
 * Si `KESH_E2E_CAPTURES_DIR` est posé, le scénario 1 capture la section du
 * verrou en thème clair et sombre, nommée d'après `KESH_E2E_CAPTURES_TAG`
 * (`avant` / `apres`). Les captures vont **hors dépôt** (C-28-2-30) ; sans la
 * variable, rien n'est écrit.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: Page, username = 'admin', password = 'admin123') {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

const REFUS = 'Message de refus de test (28-2).';

test.describe('Erreurs d’API affichées par leur message (Story 28-2)', () => {
	test('scénario 1 — un refus du verrou de période affiche le message du serveur', async ({ page }) => {
		await login(page);
		await page.route('**/api/v1/companies/current/books-lock', (route) =>
			route.fulfill({
				status: 400,
				contentType: 'application/json',
				body: JSON.stringify({ error: { code: 'VALIDATION_ERROR', message: REFUS } }),
			}),
		);
		await page.goto('/settings');
		const section = page.getByTestId('settings-books-lock');
		await expect(section).toBeVisible();
		await page.fill('#books-lock-through', '2020-01-31');
		await page.getByTestId('books-lock-submit').click();

		const erreur = page.getByTestId('books-lock-error');
		await expect(erreur).toBeVisible();

		const dossier = process.env.KESH_E2E_CAPTURES_DIR;
		if (dossier) {
			const tag = process.env.KESH_E2E_CAPTURES_TAG ?? 'capture';
			for (const colorScheme of ['light', 'dark'] as const) {
				await page.emulateMedia({ colorScheme });
				await section.screenshot({ path: `${dossier}/books-lock-${tag}-${colorScheme}.png` });
			}
			await page.emulateMedia({ colorScheme: 'light' });
		}

		await expect(erreur).toHaveText(REFUS);
		await expect(page.locator('body')).not.toContainText('[object Object]');
	});

	test('scénario 2 — une panne réseau des profils bancaires affiche le message réseau, jamais [object Object]', async ({
		page,
	}) => {
		await login(page);
		const ctx = await authedApiContext(page);
		let attendu: string;
		try {
			const res = await ctx.get('/api/v1/i18n/messages');
			expect(res.ok()).toBeTruthy();
			const { messages } = (await res.json()) as { messages: Record<string, string> };
			attendu = messages['error-network'];
			expect(attendu, 'la clé error-network doit être servie par le catalogue').toBeTruthy();
		} finally {
			await disposeContextSafe(ctx);
		}

		await page.addInitScript(() => {
			(window as unknown as { __KESH_RETRY_DELAYS: number[] }).__KESH_RETRY_DELAYS = [10, 10, 10, 10];
		});
		await page.route('**/api/v1/bank-profiles*', (route) => route.abort('failed'));
		await page.goto('/bank-import/profiles');

		const liste = page.getByTestId('bank-import-profile-list');
		await expect(liste).toContainText(attendu, { timeout: 15000 });
		await expect(page.locator('body')).not.toContainText('[object Object]');
	});
});
