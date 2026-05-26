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

	test("Scénario (c) — cookie HttpOnly flag visible via Playwright context.cookies() (defense-in-depth)", async ({
		page,
		context,
	}) => {
		// CR Pass 1 M3 — scénario (c) reformulé : la version initiale testait
		// `credentials: 'omit'` (qui exclut le cookie côté JS) puis 401 — mais
		// ça ne testait PAS la protection XSS (un attaquant XSS peut tout à
		// fait utiliser `credentials: 'include'` pour pivoter via le cookie ;
		// la vraie défense est SameSite=Strict + CSP qui empêchent l'exfiltration
		// cross-site). Le test est donc reformulé pour valider l'invariant
		// concret : Playwright voit bien le flag httpOnly sur le cookie côté
		// browser context — preuve directe que le navigateur cache le token
		// du JS via la spec HTML5 Cookie API.
		//
		// Note v0.1 : un attaquant XSS in-page peut effectivement appeler
		// fetch('/api/v1/auth/me', {credentials:'include'}) et faire des
		// requêtes authentifiées. C'est ACCEPTÉ — `HttpOnly` protège contre
		// l'exfiltration du token via document.cookie (scénario a), pas
		// contre le pivot. La défense en profondeur passe par SameSite=Strict
		// (anti-CSRF), CSP `'self'` (anti-injection script externe) et
		// audit-trail backend (log des actions sensibles).
		await page.goto('/login');
		await page.fill('#username', 'admin');
		await page.fill('#password', 'admin123');
		await page.click('button[type="submit"]');
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');

		// Inspecte les cookies du browser context (Playwright API expose
		// les cookies HttpOnly via context.cookies() — c'est cette API qui
		// permet aux tests E2E de les manipuler, pas le JS de la page).
		const cookies = await context.cookies();
		const accessCookie = cookies.find((c) => c.name === 'kesh_access_token');
		const refreshCookie = cookies.find((c) => c.name === 'kesh_refresh_token');

		// CR Pass 3 AA3-M1 — asserter les 4 flags simultanément exigés par
		// l'AC #14(c) : `httpOnly === true` + `sameSite === 'Strict'` +
		// `secure === true` (hors test_mode) + `path` scoped. Le test passait
		// précédemment à tort si `Path=/` était changé en `Path=/api/v1/...`
		// sur l'access cookie ou si `Secure` était omis en prod.
		const inTestMode = process.env.KESH_TEST_MODE === 'true';

		expect(accessCookie).toBeDefined();
		expect(accessCookie?.httpOnly).toBe(true);
		expect(accessCookie?.sameSite).toBe('Strict');
		expect(accessCookie?.path).toBe('/');
		// `Secure` doit être présent en prod HTTPS (test_mode désactive cette
		// exigence pour les tests E2E sur HTTP loopback — cf. CHANGELOG §Sécurité
		// durcie ligne « Note pour les administrateurs »).
		expect(accessCookie?.secure).toBe(!inTestMode);

		expect(refreshCookie).toBeDefined();
		expect(refreshCookie?.httpOnly).toBe(true);
		expect(refreshCookie?.sameSite).toBe('Strict');
		// Vérifie le scope path-restricted du refresh cookie (defense-in-depth).
		expect(refreshCookie?.path).toBe('/api/v1/auth');
		expect(refreshCookie?.secure).toBe(!inTestMode);
	});
});
