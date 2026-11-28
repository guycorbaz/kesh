import { expect, test } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';
import { createUser, USER_PASSWORD } from './helpers/api-fixtures';

/**
 * Tests E2E — Story 64-1 (#1712) : le mode d'interface (Guidé/Expert) est un réglage de
 * l'installation, réservé à l'Administrateur. Le menu utilisateur ne propose la bascule
 * qu'à lui, et sa bascule change `html[data-mode]` ET s'enregistre côté serveur (`204`).
 *
 * `mode-expert.spec.ts` n'en tient pas lieu : ses tests sont des échecs attendus (KF-029) et
 * son assertion de bascule est vraie par construction.
 */

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

async function login(page: import('@playwright/test').Page, user: string, pwd: string) {
	await page.goto('/login');
	await page.fill('#username', user);
	await page.fill('#password', pwd);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

async function openUserMenu(page: import('@playwright/test').Page) {
	await page.getByTestId('user-menu-trigger').click();
	// Sonde positive : le menu est bien ouvert (même menu que l'entrée « Mode »).
	await expect(page.getByTestId('user-menu-logout')).toBeVisible();
}

test.describe('Menu utilisateur — bascule du mode (64-1)', () => {
	for (const role of ['Consultation', 'Comptable']) {
		test(`${role} : pas d'entrée « Mode » dans le menu`, async ({ page }) => {
			const username = await createUser(page, role);
			await login(page, username, USER_PASSWORD);
			await openUserMenu(page);
			await expect(page.getByTestId('user-menu-mode')).toHaveCount(0);
		});
	}

	test('Admin : l\'entrée est là, la bascule change data-mode et le PUT répond 204', async ({
		page,
	}) => {
		await login(page, 'admin', 'admin123');
		await openUserMenu(page);
		const entry = page.getByTestId('user-menu-mode');
		await expect(entry).toBeVisible();
		const before = await page.locator('html').getAttribute('data-mode');
		expect(['guided', 'expert']).toContain(before);
		const [resp] = await Promise.all([
			page.waitForResponse(
				(r) => r.url().includes('/api/v1/profile/mode') && r.request().method() === 'PUT',
			),
			entry.click(),
		]);
		expect(resp.status()).toBe(204);
		const expected = before === 'guided' ? 'expert' : 'guided';
		await expect(page.locator('html')).toHaveAttribute('data-mode', expected);
	});
});
