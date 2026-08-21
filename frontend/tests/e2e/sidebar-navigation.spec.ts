/**
 * Story v014-1 — Tests E2E Playwright pour la sidebar collapsible avec
 * persistence localStorage.
 *
 * AC#33 — couvre :
 *   1. Navigation vers les 5 entrées Administration (Plan comptable,
 *      Exercices, Comptes bancaires, Profils bancaires, Règles d'affectation)
 *      depuis la sidebar.
 *   2. Persistence localStorage : toggle Administration expand → reload
 *      → état préservé.
 *   3. Auto-expand du groupe contenant la route active (F12 Pass 3 Opus).
 */

import { expect, test, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function loginAsAdmin(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

test.describe('Sidebar navigation v014-1', () => {
	// ⚠️ Le groupe se cible par `data-testid`, jamais par `has-text("Administration")`. Le
	// libellé est traduit depuis la story 23-3b : identique en français, en allemand et en
	// anglais — mais `Amministrazione` en italien. Un sélecteur qui passe dans trois langues
	// sur quatre est plus traître qu'un sélecteur cassé, puisque rien ne le signale.
	test('groupe Administration fermé par défaut, ouverture au clic', async ({ page }) => {
		await loginAsAdmin(page);

		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		const adminDetails = sidebar.locator('[data-testid="nav-group-administration"]');

		// Défaut fermé.
		await expect(adminDetails).not.toHaveAttribute('open', /.*/);

		// Click pour ouvrir.
		await sidebar.locator('[data-testid="nav-group-administration"] summary').click();
		await expect(adminDetails).toHaveAttribute('open', '');
	});

	test('navigue vers les 5 entrées Administration', async ({ page }) => {
		await loginAsAdmin(page);
		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		await sidebar.locator('[data-testid="nav-group-administration"] summary').click();

		const routes = [
			{ testid: 'nav-link-accounts', url: '/accounts' },
			{ testid: 'nav-link-settings-fiscal-years', url: '/settings/fiscal-years' },
			{ testid: 'nav-link-bank-accounts', url: '/bank-accounts' },
			{ testid: 'nav-link-bank-import-profiles', url: '/bank-import/profiles' },
			{ testid: 'nav-link-reconciliation-rules', url: '/reconciliation/rules' },
		];

		for (const { testid, url } of routes) {
			await sidebar.locator(`[data-testid="${testid}"]`).click();
			await expect(page).toHaveURL(url);
			// Le groupe doit rester ouvert (auto-expand F12 Pass 3 Opus pour route active).
			await expect(
				sidebar.locator('[data-testid="nav-group-administration"]'),
			).toHaveAttribute('open', '');
		}
	});

	test('persistence localStorage : Administration ouvert puis reload reste ouvert', async ({ page }) => {
		await loginAsAdmin(page);
		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		const adminDetails = sidebar.locator('[data-testid="nav-group-administration"]');

		// Ouvrir le groupe (persiste via localStorage).
		await sidebar.locator('[data-testid="nav-group-administration"] summary').click();
		await expect(adminDetails).toHaveAttribute('open', '');

		// Vérifier la clé localStorage.
		const stored = await page.evaluate(() =>
			localStorage.getItem('kesh:sidebar:expanded:administration'),
		);
		expect(stored).toBe('true');

		// Reload — l'état doit persister.
		await page.reload();
		await expect(adminDetails).toHaveAttribute('open', '');
	});

	test('auto-expand groupe contenant la route active (F12 Pass 3 Opus)', async ({ page }) => {
		await loginAsAdmin(page);

		// Forcer Administration fermé en localStorage AVANT navigation.
		await page.evaluate(() => {
			localStorage.setItem('kesh:sidebar:expanded:administration', 'false');
		});

		// Naviguer directement vers /accounts (sous Administration).
		await page.goto('/accounts');

		// Le groupe Administration doit s'auto-expandre car contient la route active.
		const sidebar = page.locator('nav[aria-label="Navigation principale"]');
		await expect(
			sidebar.locator('[data-testid="nav-group-administration"]'),
		).toHaveAttribute('open', '');
	});
});
