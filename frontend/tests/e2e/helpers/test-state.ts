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

import { request as playwrightRequest, type APIRequestContext } from '@playwright/test';

// `@types/node` n'est pas installé côté frontend — Playwright tourne pourtant
// bien sous Node, donc `process` existe à l'exécution. Déclaration ambient
// minimale pour que `svelte-check` (TypeScript) ne s'étouffe pas.
declare const process: { env: { readonly [key: string]: string | undefined } };

export type Preset = 'fresh' | 'post-onboarding' | 'with-company' | 'with-data';

const BACKEND_URL = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000';

/**
 * Truncate la DB backend puis seed l'état correspondant au preset.
 *
 * @throws {Error} si l'endpoint répond != 200 (KESH_TEST_MODE désactivé,
 *                 backend éteint, ou preset invalide).
 */
export async function seedTestState(preset: Preset): Promise<void> {
	const ctx: APIRequestContext = await playwrightRequest.newContext({ baseURL: BACKEND_URL });
	try {
		const res = await ctx.post('/api/v1/_test/seed', { data: { preset } });
		if (!res.ok()) {
			const body = await res.text().catch(() => '<no body>');
			throw new Error(
				`seedTestState(${preset}) failed: ${res.status()} ${res.statusText()} — ` +
					`body: ${body} — KESH_TEST_MODE may not be enabled on backend ${BACKEND_URL}`,
			);
		}
	} finally {
		await ctx.dispose();
	}
}
