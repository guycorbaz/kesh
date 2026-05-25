/**
 * Story 10-5 — Tests E2E Playwright XSS token protection (T10).
 *
 * 3 scénarios vérifient que les tokens d'authentification sont protégés
 * contre les attaques XSS, conformément aux ACs #14 et #2 de la story :
 *
 * (a) `document.cookie` exécuté en JavaScript ne retourne PAS les tokens —
 *     les cookies HttpOnly sont inaccessibles au script du navigateur.
 * (b) `localStorage.getItem('kesh:auth:accessToken')` retourne `null` —
 *     les tokens ne sont plus stockés en localStorage post-Story 10-5.
 * (c) Un fetch simulé avec `credentials: 'omit'` (équivalent script
 *     malveillant injecté qui essaie d'utiliser le cookie sans
 *     credentials) reçoit 401 → le cookie HttpOnly reste protégé.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true sur le backend, seedé
 * avec preset `with-company` (cohérent autres specs E2E).
 */

import { test, expect } from '@playwright/test';
import { seedTestState, clearAuthStorage } from '../helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.describe('XSS token protection (Story 10-5)', () => {
	test("Scénario (a) — document.cookie ne contient pas les tokens (HttpOnly inaccessible JS)", async ({
		page,
	}) => {
		// Login via UI.
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');

		// Lire document.cookie depuis le JavaScript navigateur (simule un
		// XSS qui voudrait exfiltrer les tokens via document.cookie).
		const cookieString = await page.evaluate(() => document.cookie);

		// Les cookies HttpOnly NE doivent PAS apparaître dans document.cookie.
		// Le navigateur les cache au JS conformément à la spec HTML5.
		expect(cookieString).not.toContain('kesh_access_token');
		expect(cookieString).not.toContain('kesh_refresh_token');
		// Plus généralement : pas de JWT pattern (xxxxx.xxxxx.xxxxx) attendu.
		expect(cookieString).not.toMatch(/[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/);
	});

	test("Scénario (b) — localStorage ne contient pas les tokens (Story 10-5 retrait localStorage)", async ({
		page,
	}) => {
		// Login via UI.
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');

		// Vérifier que les 3 keys legacy localStorage sont absentes
		// (T6.1 a retiré les setItem dans login()).
		const accessToken = await page.evaluate(() =>
			localStorage.getItem('kesh:auth:accessToken'),
		);
		const refreshToken = await page.evaluate(() =>
			localStorage.getItem('kesh:auth:refreshToken'),
		);
		const expiresIn = await page.evaluate(() =>
			localStorage.getItem('kesh:auth:expiresIn'),
		);

		expect(accessToken).toBeNull();
		expect(refreshToken).toBeNull();
		expect(expiresIn).toBeNull();
	});

	test("Scénario (c) — fetch avec credentials: 'omit' reçoit 401 (XSS-simulated, cookie protégé)", async ({
		page,
	}) => {
		// Login pour avoir une session active.
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');

		// Simule un script XSS qui voudrait utiliser le cookie HttpOnly
		// pour faire une requête authentifiée — sans `credentials: 'include'`,
		// le browser N'envoie PAS le cookie (defaut credentials = 'same-origin'
		// inclut les cookies, mais `'omit'` les exclut explicitement).
		const status = await page.evaluate(async () => {
			const res = await fetch('/api/v1/auth/me', { credentials: 'omit' });
			return res.status;
		});

		// Sans cookie + sans header Authorization → 401.
		expect(status).toBe(401);

		// Vérification croisée : avec `credentials: 'include'`, le cookie
		// est envoyé et la requête réussit (200).
		const statusWithCreds = await page.evaluate(async () => {
			const res = await fetch('/api/v1/auth/me', { credentials: 'include' });
			return res.status;
		});
		expect(statusWithCreds).toBe(200);
	});
});
