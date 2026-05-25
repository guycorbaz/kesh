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
 * Story 10-5 (T12.3.a) : helper obsolète post-Story 10-5 — le token JWT
 * n'est plus en localStorage, il est en cookie HttpOnly inaccessible JS.
 * Conservé pour rétro-compat documentaire, mais retourne `null` pour signaler
 * aux callers que cette voie ne fonctionne plus.
 *
 * @deprecated Use `authedApiContext(page)` qui clone le storageState
 * (incluant les cookies HttpOnly) du browser context.
 */
async function readAccessTokenFromStorage(page: Page): Promise<string | null> {
	// Defensive : check localStorage au cas où un test pré-Story 10-5 setrait
	// encore manuellement le token. Toujours retourne null en flow normal post-Story.
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
	// Story 10-5 (T12.3.a Pass 3 F-PLAYWRIGHT-COOKIE-CROSS-CONTEXT-P3-2 D4 acté
	// Option (a-ii)) : cloner le `storageState` du browser context (qui inclut
	// les cookies HttpOnly `kesh_access_token` et `kesh_refresh_token` set par
	// `login(page)`) dans un APIRequestContext isolé. Plus de lecture
	// localStorage ni d'injection Authorization Bearer header — les cookies
	// HttpOnly du browser context sont propagés automatiquement.
	//
	// Sans clone storageState, `playwrightRequest.newContext()` créerait un
	// context HTTP isolé sans aucun cookie → tous les appels API
	// authentifiés recevraient 401 (régression). Le `dispose()` reste safe
	// sur ce context cloné (pas d'impact sur le browser context partagé).
	const storageState = await page.context().storageState();
	return playwrightRequest.newContext({
		baseURL: resolveBackendUrl(),
		storageState,
	});
}

/**
 * Story 10-5 (T12.3.a) : nettoie les cookies HttpOnly d'auth entre tests
 * pour garantir l'isolation de session E2E.
 *
 * Pré-Story 10-5 : effaçait les clés `kesh:auth:*` de localStorage (les
 * tokens y étaient stockés). Post-Story 10-5 : les tokens sont en cookies
 * HttpOnly → `page.context().clearCookies()` est la bonne API. Le clean
 * localStorage est conservé en defensive cleanup pour purger les sessions
 * pre-Story 10-5 d'un browser context qui aurait persisté ces clés.
 *
 * @throws {Error} if page context is invalid (logged, not rethrown)
 */
export async function clearAuthStorage(page: Page): Promise<void> {
	try {
		// Story 10-5 — clear cookies (où sont les tokens HttpOnly post-Story).
		await page.context().clearCookies();
	} catch (error) {
		console.warn(
			'[test] clearAuthStorage clearCookies failed:',
			error instanceof Error ? error.message : String(error),
		);
	}
	try {
		// Defensive cleanup localStorage (purge sessions pre-Story 10-5).
		await page.evaluate((keys) => {
			for (const key of keys) {
				localStorage.removeItem(key);
			}
		}, [STORAGE_KEY_ACCESS_TOKEN, STORAGE_KEY_REFRESH_TOKEN, STORAGE_KEY_EXPIRES_IN]);
	} catch (error) {
		// If Playwright context is destroyed, log but don't fail test teardown
		console.warn(
			'[test] clearAuthStorage localStorage cleanup failed (page context may be destroyed):',
			error instanceof Error ? error.message : String(error),
		);
	}
}
