/**
 * Tests E2E Playwright — Résilience frontend si DB inaccessible (Story 10.3).
 *
 * 3 scénarios couvrent le comportement de l'app face à une DB indisponible :
 * (1) DB down au load — SPA + banner + version footer affichés.
 * (2) DB down mid-navigation — banner apparaît après retry exponentiel.
 * (3) DB recovery — banner disparaît après que le ping `/health` détecte le retour.
 *
 * Pattern d'isolation : `page.route()` interception côté navigateur (pas de
 * manipulation Docker / `docker compose stop mariadb`) — déterministe, isolé
 * du runner CI, rapide (cf. Dev Notes §"Pattern E2E DB down").
 *
 * Pattern d'accélération retry : `page.addInitScript(() => { window.__KESH_RETRY_DELAYS = [10, 10, 10, 10]; })`
 * exécuté AVANT `page.goto()` — la fenêtre est patchée avant que `api-client.ts`
 * ne lise la valeur au 1er retry. Race-free vs `localStorage.setItem` (qui aurait
 * un round-trip storage non-déterministe).
 */

import { test, expect } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.describe('DB resilience (Story 10.3)', () => {
	test("Scénario 1 — DB down at load : SPA s'affiche, banner dégradé visible, version footer visible", async ({
		page,
	}) => {
		// Intercept TOUTES les routes API + le healthcheck → 503 simulé
		await page.route('/api/v1/**', (route) =>
			route.fulfill({
				status: 503,
				contentType: 'application/json',
				body: JSON.stringify({
					error: { code: 'SERVICE_UNAVAILABLE', message: 'DB temporairement indisponible' },
				}),
			}),
		);
		await page.route('/health', (route) =>
			route.fulfill({
				status: 503,
				contentType: 'application/json',
				body: JSON.stringify({ status: 'degraded', db: false, version: '0.1.0' }),
			}),
		);

		await page.goto('/login');

		// AC #20 : Le SPA s'affiche malgré la DB down (titre login visible)
		await expect(page.locator('h1', { hasText: 'Kesh' })).toBeVisible();

		// AC #20 : Le DegradedBanner s'affiche avec texte FR (chargé soit via i18n
		// API, soit via le fallback FR embarqué dans i18nMsg)
		const banner = page.locator('[data-testid=degraded-banner]');
		await expect(banner).toBeVisible({ timeout: 10000 });
		await expect(banner).toContainText(/Base de données temporairement indisponible/);

		// AC #20 : La version Kesh est visible en pied (preuve frontend servi malgré DB down)
		const version = page.locator('[data-testid=app-version]');
		await expect(version).toContainText(/v\d+\.\d+\.\d+/);
	});

	test("Scénario 2 — DB down mid-navigation : banner s'affiche après NETWORK_ERROR sur retry", async ({
		page,
	}) => {
		// Override des délais retry pour accélérer le test (40ms total au lieu de 14.3s)
		await page.addInitScript(() => {
			(window as unknown as { __KESH_RETRY_DELAYS: number[] }).__KESH_RETRY_DELAYS = [
				10, 10, 10, 10,
			];
		});

		// Login normal (intercept off)
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');

		// Activer l'intercept api/v1 → NETWORK_ERROR (route.abort)
		await page.route('/api/v1/**', (route) => route.abort('failed'));

		// Navigation déclenche fetch API → retry exponentiel → banner s'affiche
		await page.goto('/contacts');

		const banner = page.locator('[data-testid=degraded-banner]');
		await expect(banner).toBeVisible({ timeout: 5000 });
	});

	test("Scénario 3 — DB recovery : banner disparaît automatiquement après unroute health + api", async ({
		page,
	}) => {
		await page.addInitScript(() => {
			(window as unknown as { __KESH_RETRY_DELAYS: number[] }).__KESH_RETRY_DELAYS = [
				10, 10, 10, 10,
			];
		});

		// Setup état dégradé via intercept SUR /api/v1/** ET /health (les deux sont
		// obligatoires — sinon le ping /health resterait 503 et clearDegraded ne
		// serait jamais appelé, cf. AC #22).
		await page.route('/api/v1/**', (route) =>
			route.fulfill({
				status: 503,
				contentType: 'application/json',
				body: JSON.stringify({
					error: { code: 'SERVICE_UNAVAILABLE', message: 'DB temporairement indisponible' },
				}),
			}),
		);
		await page.route('/health', (route) =>
			route.fulfill({
				status: 503,
				contentType: 'application/json',
				body: JSON.stringify({ status: 'degraded', db: false, version: '0.1.0' }),
			}),
		);

		await page.goto('/login');

		const banner = page.locator('[data-testid=degraded-banner]');
		await expect(banner).toBeVisible({ timeout: 10000 });

		// Recovery : désactive les deux intercepts. Le ping pollHealth (5s tick)
		// va alors atteindre le vrai backend joignable et appeler clearDegraded().
		await page.unroute('/api/v1/**');
		await page.unroute('/health');

		// Le banner disparaît dans les 5-6s (1 tick pollHealth + marge). Pass 1
		// code review F9 : timeout porté à 10s (vs 7s) pour défense en
		// profondeur — en CI loaded (GitHub Actions shared runner, Docker
		// overhead), un backend `SELECT 1` peut dépasser 200ms sous pression
		// mémoire ; 1 tick pollHealth (5s) + jusqu'à 5s de margin couvre les
		// pires cas observés. Tampon vs flakiness, pas un timeout absolu.
		await expect(banner).toBeHidden({ timeout: 10000 });
	});
});
