import { test, expect } from '@playwright/test';
import { seedTestState, clearAuthStorage } from './helpers/test-state';

/**
 * Tests E2E — Onboarding self-service (Story v011-5).
 *
 * Couvre le flow `/setup` complet : preset `setup-required` (DB vidée +
 * 1 company stub seule) → navigation root redirige vers `/setup` (via 423
 * Locked intercepté par api-client.ts) → submit form → cookies HttpOnly
 * set → landing sur `/onboarding`.
 */

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test.describe('Setup self-service flow', () => {
	test('preset setup-required → navigation racine redirige vers /setup', async ({ page }) => {
		await seedTestState('setup-required');
		await page.goto('/');
		// Le hydrate /me retourne 423 → store set _setupRequired = true → +layout.ts redirect /setup
		await expect(page).toHaveURL(/\/setup/);

		// Écran de bienvenue avec champs username, password, password-confirm.
		await expect(page.getByTestId('setup-username')).toBeVisible();
		await expect(page.getByTestId('setup-password')).toBeVisible();
		await expect(page.getByTestId('setup-password-confirm')).toBeVisible();
		await expect(page.getByTestId('setup-submit')).toBeVisible();
	});

	test('validation client : password mismatch désactive le bouton submit', async ({ page }) => {
		await seedTestState('setup-required');
		await page.goto('/setup');

		await page.getByTestId('setup-username').fill('admin');
		await page.getByTestId('setup-password').fill('valid-password-12-chars');
		await page.getByTestId('setup-password-confirm').fill('DIFFERENT-password-12');

		// Mismatch message visible.
		await expect(page.getByTestId('setup-password-mismatch')).toBeVisible();

		// Submit désactivé.
		await expect(page.getByTestId('setup-submit')).toBeDisabled();
	});

	test('happy path → submit valide → redirect /onboarding + cookies HttpOnly', async ({
		page,
		context,
	}) => {
		await seedTestState('setup-required');
		await page.goto('/setup');

		const password = 'admin-setup-pw-12chars';
		await page.getByTestId('setup-username').fill('admin');
		await page.getByTestId('setup-password').fill(password);
		await page.getByTestId('setup-password-confirm').fill(password);

		await expect(page.getByTestId('setup-submit')).toBeEnabled();
		await page.getByTestId('setup-submit').click();

		// Redirige vers /onboarding (wizard).
		await page.waitForURL(/\/onboarding/, { timeout: 10_000 });

		// Cookies HttpOnly set (vérifie via context cookies — HttpOnly invisibles à JS,
		// mais Playwright peut les lire via context).
		const cookies = await context.cookies();
		const accessCookie = cookies.find((c) => c.name === 'kesh_access_token');
		const refreshCookie = cookies.find((c) => c.name === 'kesh_refresh_token');
		expect(accessCookie, 'access cookie should be set').toBeDefined();
		expect(refreshCookie, 'refresh cookie should be set').toBeDefined();
		expect(accessCookie?.httpOnly, 'access cookie must be HttpOnly').toBe(true);
		expect(refreshCookie?.httpOnly, 'refresh cookie must be HttpOnly').toBe(true);
	});

	test('reload de /setup post-création → redirige /login (410 SETUP_ALREADY_COMPLETE)', async ({
		page,
	}) => {
		await seedTestState('setup-required');
		await page.goto('/setup');

		const password = 'second-pw-12-chars-here';
		await page.getByTestId('setup-username').fill('admin');
		await page.getByTestId('setup-password').fill(password);
		await page.getByTestId('setup-password-confirm').fill(password);
		await page.getByTestId('setup-submit').click();

		// Attendre la redirection /onboarding.
		await page.waitForURL(/\/onboarding/, { timeout: 10_000 });

		// Logout pour ne pas être authentifié quand on revisite /setup.
		await page.evaluate(async () => {
			await fetch('/api/v1/auth/logout', { method: 'POST', credentials: 'include' });
		});

		// Tenter de revisiter /setup → soumettre devrait recevoir 410.
		await page.goto('/setup');
		const newPassword = 'attacker-tries-late-12';
		await page.getByTestId('setup-username').fill('attacker');
		await page.getByTestId('setup-password').fill(newPassword);
		await page.getByTestId('setup-password-confirm').fill(newPassword);
		await page.getByTestId('setup-submit').click();

		// Message d'erreur 410 visible.
		await expect(page.getByTestId('setup-error-message')).toContainText(
			/déjà été créé|already been created|bereits erstellt|già stato creato/,
		);
	});
});
