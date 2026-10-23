import { test, expect } from '@playwright/test';
import {
	seedTestState,
	authedApiContext,
	disposeContextSafe,
	clearAuthStorage,
} from './helpers/test-state';

/**
 * Smoke E2E — Story 31-2 (#577) : le HARNAIS est-il vivant ?
 *
 * Cette spec ne teste aucune fonctionnalité, et c'est délibéré. Elle vérifie
 * que la chaîne dont dépend toute la suite tient debout : le backend sert la
 * SPA, la connexion par l'interface pose les cookies de session, et un
 * contexte API cloné par `authedApiContext` les joint bien. C'est exactement ce
 * qui avait cassé en silence avec l'issue #285 : un contexte API sans cookie
 * rendait 401 partout, et rien ne signalait une suite qui cessait de tourner.
 *
 * Lancée seule par le job `e2e-smoke` de `ci.yml`, contre l'image Docker de la
 * PR ; et, comme toute spec, par la suite complète en local.
 *
 * Le second test est la sonde NÉGATIVE : sans cookie, la même route rend 401.
 * Sans lui, le 200 du premier pourrait venir d'une route ouverte — et un smoke
 * qui ne peut pas rougir ne prouve rien.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test('connexion par l’interface, puis appel API authentifié par cookie → 200', async ({ page }) => {
	try {
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');

		const api = await authedApiContext(page);
		try {
			const res = await api.get('/api/v1/auth/me');
			expect(res.status(), 'GET /api/v1/auth/me avec les cookies de la session').toBe(200);
			const body = await res.json();
			expect(body.username).toBe('admin');
		} finally {
			await disposeContextSafe(api);
		}
	} finally {
		// Ici plutôt qu'en `afterEach` : la sonde négative n'ouvre pas de page.
		await clearAuthStorage(page);
	}
});

test('sonde négative : la même route sans cookie → 401', async ({ request }) => {
	// `request` est un contexte API neuf, sans cookie, sur le `baseURL` de la
	// configuration (`KESH_BACKEND_URL`).
	const res = await request.get('/api/v1/auth/me');
	expect(res.status(), 'GET /api/v1/auth/me sans cookie').toBe(401);
});
