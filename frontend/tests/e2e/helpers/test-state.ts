/**
 * Story 6.4 — helper Playwright pour seeder l'état DB déterministe via
 * l'endpoint gated `POST /api/v1/_test/seed` du backend kesh-api.
 *
 * Principe : chaque spec appelle `seedTestState('<preset>')` dans son
 * `beforeAll` (ou `beforeEach` pour les specs onboarding qui mutent le
 * singleton `onboarding_state` de façon irréversible). Le backend truncate
 * l'ensemble de la DB puis insère uniquement les rows du preset demandé —
 * les specs partent d'un état connu, indépendant de l'ordre d'exécution
 * Playwright.
 *
 * **Routing backend absolu** (cf. F1 review pass 1) : Playwright tourne
 * contre `:4173` (SvelteKit `preview`) et `:4173` ne proxy pas `/api/v1/*`
 * vers le backend `:3000`. Le helper crée donc son propre
 * `APIRequestContext` ciblant `http://127.0.0.1:3000` (override via
 * `KESH_BACKEND_URL`). On **ne dépend pas** de la `page`/`request`
 * injectés par Playwright.
 *
 * **Sécurité** : l'endpoint `/api/v1/_test/*` n'est monté que si
 * `KESH_TEST_MODE=true` dans l'env du backend ET le bind est loopback
 * (`KESH_HOST=127.0.0.1`). Sinon, le backend refuse de démarrer (garde-fou
 * `ConfigError::TestModeWithPublicBind`). Si l'endpoint est injoignable,
 * le helper throw avec un message explicite listant les vérifications.
 */

import { request as playwrightRequest, type APIRequestContext, type Page } from '@playwright/test';

export type Preset =
	| 'fresh'
	| 'post-onboarding'
	| 'with-company'
	| 'with-data'
	| 'with-company-no-fy';

// Storage key constants (must match auth.svelte.ts — if keys change there, update here too)
const STORAGE_KEY_ACCESS_TOKEN = 'kesh:auth:accessToken';
const STORAGE_KEY_REFRESH_TOKEN = 'kesh:auth:refreshToken';
const STORAGE_KEY_EXPIRES_IN = 'kesh:auth:expiresIn';

/**
 * Résout et valide l'URL backend (code review P9).
 *
 * Sans validation, `KESH_BACKEND_URL=""` ferait que `baseURL: ''` dans
 * `newContext` accepte des POST relatifs contre le webServer Playwright
 * (`:4173`) → erreurs opaques. On throw early avec un message clair.
 */
function resolveBackendUrl(): string {
	const raw = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000';
	const trimmed = raw.trim();
	if (trimmed === '') {
		throw new Error('KESH_BACKEND_URL est vide — attendu URL absolue (ex: http://127.0.0.1:3000).');
	}
	try {
		// Lance TypeError si l'URL est malformée.
		new URL(trimmed);
	} catch {
		throw new Error(
			`KESH_BACKEND_URL="${trimmed}" n'est pas une URL valide — attendu ex: http://127.0.0.1:3000.`,
		);
	}
	return trimmed;
}

/**
 * Truncate la DB backend puis seed l'état correspondant au preset.
 *
 * @throws {Error} si l'endpoint répond != 200 (KESH_TEST_MODE désactivé,
 *                 backend éteint, ou preset invalide), ou si
 *                 `KESH_BACKEND_URL` est malformée.
 */
export async function seedTestState(preset: Preset): Promise<void> {
	const backendUrl = resolveBackendUrl();
	const ctx: APIRequestContext = await playwrightRequest.newContext({ baseURL: backendUrl });
	try {
		const res = await ctx.post('/api/v1/_test/seed', { data: { preset } });
		if (!res.ok()) {
			const body = await res.text().catch(() => '<no body>');
			throw new Error(
				`seedTestState(${preset}) failed: ${res.status()} ${res.statusText()} — ` +
					`body: ${body} — KESH_TEST_MODE may not be enabled on backend ${backendUrl}`,
			);
		}
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Lit le token d'accès JWT depuis le `localStorage` côté `page`. Helper
 * interne pour `authedApiContext`.
 *
 * @returns Le token sous forme de string, ou `null` si la clé est absente
 *          (utilisateur non-loggé ou storage vidé par `clearAuthStorage`).
 */
async function readAccessTokenFromStorage(page: Page): Promise<string | null> {
	return page.evaluate((key) => localStorage.getItem(key), STORAGE_KEY_ACCESS_TOKEN);
}

/**
 * Dispose un `APIRequestContext` en swallowing les erreurs de teardown.
 *
 * Motivation (Pass 1 code-review ECH-M1) : si `ctx.dispose()` throw dans un
 * `finally`, l'erreur originelle du `try` (e.g. assertion `expect(res.ok())
 * .toBeTruthy()` qui révèle un 401 ou 400) est silencieusement écrasée par
 * l'erreur de teardown. La cause originelle est perdue → diagnostic E2E très
 * difficile. Le wrapper console.warn préserve l'erreur originelle tout en
 * loggant la défaillance de teardown (cohérent avec le pattern défensif de
 * `clearAuthStorage` lignes ~104-107).
 *
 * Utilisation typique :
 * ```
 * const ctx = await authedApiContext(page);
 * try { ... } finally { await disposeContextSafe(ctx); }
 * ```
 */
export async function disposeContextSafe(ctx: APIRequestContext): Promise<void> {
	await ctx.dispose().catch((err) => {
		console.warn(
			'[test] APIRequestContext dispose failed:',
			err instanceof Error ? err.message : String(err),
		);
	});
}

/**
 * Construit un `APIRequestContext` Playwright avec le Bearer token de la
 * session courante automatiquement injecté en header `Authorization`.
 *
 * Utilisation : remplace les appels directs `page.request.post/get(...)`
 * dans les helpers de seed de spec files E2E quand l'endpoint cible est
 * authentifié. `page.request.*` est un client HTTP raw Playwright qui ne
 * lit pas le `localStorage` Svelte — donc n'injecte pas le Bearer token
 * du store `authState`. Sans ce helper, les appels API authentifiés
 * reçoivent 401 (régression Story 6-5 KF-007 storage shift cookie →
 * localStorage — cf. KF #54 / KF-022).
 *
 * Pré-condition : `login(page)` doit avoir été appelé avant — sinon throw
 * (cf. AC #3 spec 9-5-1b — anti-pattern « 401 silencieux »).
 *
 * Le caller est responsable de `await ctx.dispose()` après usage (try/finally
 * cohérent avec le pattern `seedTestState` ci-dessus). Le context n'est PAS
 * cached entre appels — chaque invocation crée un nouveau context, ce qui
 * évite les fuites de tokens entre tests (cohérent avec `clearAuthStorage`).
 *
 * @param page Page Playwright avec session authentifiée (post-`login`).
 * @returns APIRequestContext avec baseURL backend + Bearer header injecté.
 * @throws Error si `localStorage.kesh:auth:accessToken` est absent / vide
 *               (token non présent en storage = `login(page)` non appelé).
 */
export async function authedApiContext(page: Page): Promise<APIRequestContext> {
	const token = await readAccessTokenFromStorage(page);
	if (!token) {
		throw new Error(
			'authedApiContext: no accessToken in localStorage — call login(page) before this helper',
		);
	}
	return playwrightRequest.newContext({
		baseURL: resolveBackendUrl(),
		extraHTTPHeaders: { Authorization: `Bearer ${token}` },
	});
}

/**
 * Nettoie les clés d'authentification du localStorage côté client.
 * Appelé dans afterEach() pour isoler les tokens entre tests.
 *
 * Note: `page.context().clearCookies()` n'efface QUE les cookies, pas localStorage.
 * Cette fonction efface explicitement les clés kesh:auth:* du localStorage.
 *
 * @throws {Error} if page context is invalid or localStorage access fails
 */
export async function clearAuthStorage(page: Page): Promise<void> {
	try {
		await page.evaluate((keys) => {
			for (const key of keys) {
				localStorage.removeItem(key);
			}
		}, [STORAGE_KEY_ACCESS_TOKEN, STORAGE_KEY_REFRESH_TOKEN, STORAGE_KEY_EXPIRES_IN]);
	} catch (error) {
		// If Playwright context is destroyed, log but don't fail test teardown
		console.warn('[test] clearAuthStorage failed (page context may be destroyed):', error instanceof Error ? error.message : String(error));
	}
}
