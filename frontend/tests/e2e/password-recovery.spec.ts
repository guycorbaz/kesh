/**
 * Story 17-4e (AC24) — E2E Playwright du recovery de mot de passe self-service.
 *
 * **Recette backend (DE-6)** — le backend test-mode doit tourner avec le
 * feature recovery ACTIVÉ (sinon les tests sont skip avec message) :
 *
 * ```bash
 * KESH_TEST_MODE=true \
 *   KESH_HOST=127.0.0.1 \
 *   KESH_FEATURE_FORGOT_PASSWORD=true \
 *   KESH_PUBLIC_BASE_URL=http://127.0.0.1 \
 *   KESH_SMTP_HOST=127.0.0.1 KESH_SMTP_PORT=2525 \
 *   KESH_SMTP_USER=e2e KESH_SMTP_PASSWORD=e2e \
 *   KESH_SMTP_FROM=kesh@example.invalid \
 *   DATABASE_URL="mysql://..." KESH_JWT_SECRET="..." \
 *   cargo run -p kesh-api
 * ```
 *
 * Les valeurs SMTP sont FACTICES (fail-fast boot exige une config complète
 * quand le feature est on) : les envois réels échouent en tâche détachée
 * (loggés serveur, jamais propagés — DC4). Les tests n'en dépendent pas :
 * le token est INJECTÉ via `POST /api/v1/_test/password-reset-token` (DE-1).
 *
 * Rate-limit recovery (5 req / 15 min / IP, partagé forgot+reset) : purgé par
 * chaque `seedTestState` (DE-2) — et le budget de cette spec reste ≤ 3 POSTs.
 */

import { test, expect, request as playwrightRequest } from '@playwright/test';
import { seedTestState, clearAuthStorage, disposeContextSafe } from './helpers/test-state';

const NEW_PASSWORD = 'nouveau-mdp-e2e-12chars';

function backendUrl(): string {
	return (process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1').trim();
}

/**
 * Lit le flag DC9 exposé par /health (présent même en 503).
 *
 * Pass 1 BH-F1 — distingue « feature off » (skip légitime) de « backend
 * injoignable » (erreur d'infrastructure) : un backend down doit faire
 * ÉCHOUER la suite avec un message clair, pas la skipper en faux vert.
 */
async function recoveryFeatureEnabled(): Promise<boolean> {
	const ctx = await playwrightRequest.newContext({ baseURL: backendUrl() });
	try {
		let res;
		try {
			res = await ctx.get('/health');
		} catch (err) {
			throw new Error(
				`Backend injoignable sur ${backendUrl()} (/health) — démarrer le backend test-mode ` +
					`(recette DE-6 en tête de spec). Cause: ${err instanceof Error ? err.message : String(err)}`,
			);
		}
		const body = (await res.json().catch(() => ({}))) as { forgotPasswordEnabled?: unknown };
		return body.forgotPasswordEnabled === true;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Injecte un token de reset valide pour `username` via l'endpoint test-mode
 * (DE-1) et retourne le clair.
 */
async function injectResetToken(username: string): Promise<string> {
	const ctx = await playwrightRequest.newContext({ baseURL: backendUrl() });
	try {
		const res = await ctx.post('/api/v1/_test/password-reset-token', { data: { username } });
		if (!res.ok()) {
			const body = await res.text().catch(() => '<no body>');
			throw new Error(
				`injectResetToken(${username}) failed: ${res.status()} — ${body} — ` +
					`KESH_TEST_MODE doit être actif sur ${backendUrl()}`,
			);
		}
		const body = (await res.json()) as { token: string };
		return body.token;
	} finally {
		await disposeContextSafe(ctx);
	}
}

let featureOn = false;

test.describe('Recovery mot de passe (Story 17-4e)', () => {
	test.beforeAll(async () => {
		featureOn = await recoveryFeatureEnabled();
	});

	test.beforeEach(async () => {
		// Re-seed par test : état DB déterministe + purge des rate-limiters (DE-2).
		await seedTestState('with-company');
	});

	test.afterEach(async ({ page }) => {
		await clearAuthStorage(page);
	});

	test('login affiche le lien « Mot de passe oublié ? » quand le flag est actif', async ({
		page,
	}) => {
		test.skip(!featureOn, 'KESH_FEATURE_FORGOT_PASSWORD désactivé — cf. recette DE-6 en tête de spec');

		await page.goto('/login');
		const link = page.getByTestId('forgot-password-link');
		await expect(link).toBeVisible();
		await link.click();
		await expect(page).toHaveURL(/\/forgot-password/);
		await expect(page.getByTestId('forgot-identifier')).toBeVisible();
	});

	test('forgot-password : message générique de succès, même pour un inconnu (anti-énum)', async ({
		page,
	}) => {
		test.skip(!featureOn, 'KESH_FEATURE_FORGOT_PASSWORD désactivé — cf. recette DE-6 en tête de spec');

		await page.goto('/forgot-password');
		await page.getByTestId('forgot-identifier').fill('utilisateur-inconnu');
		await page.getByTestId('forgot-submit').click();

		// DC4 : message générique identique que le compte existe ou non —
		// et il ne doit PAS écho-er l'identifiant saisi.
		const success = page.getByTestId('forgot-success');
		await expect(success).toBeVisible();
		await expect(success).not.toContainText('utilisateur-inconnu');
	});

	test('reset-password happy path : token injecté → nouveau mdp → login OK', async ({ page }) => {
		test.skip(!featureOn, 'KESH_FEATURE_FORGOT_PASSWORD désactivé — cf. recette DE-6 en tête de spec');

		// Note (Pass 1 ECH) : le user `changeme` du preset n'a PAS d'email — le
		// token est injecté HORS du flux forgot-password (DE-1). Ce scénario
		// valide la mécanique reset+login UI ; le flux email complet (forgot →
		// mail → reset) est couvert par les tests d'intégration AC23-a.
		const token = await injectResetToken('changeme');
		await page.goto(`/reset-password?token=${token}`);

		// PD1 (17-4d) : le token est retiré de la barre d'adresse après capture.
		await expect(page).toHaveURL(/\/reset-password$/);

		await page.getByTestId('reset-password').fill(NEW_PASSWORD);
		await page.getByTestId('reset-password-confirm').fill(NEW_PASSWORD);
		await page.getByTestId('reset-submit').click();

		await expect(page.getByTestId('reset-success')).toBeVisible();
		await page.getByTestId('reset-login-cta').click();
		await expect(page).toHaveURL(/\/login/);

		// Login avec le NOUVEAU mot de passe (preset with-company : user changeme).
		await page.locator('#username').fill('changeme');
		await page.locator('#password').fill(NEW_PASSWORD);
		await page.getByRole('button', { name: 'Se connecter' }).click();
		await page.waitForURL('/', { timeout: 10_000 });
	});

	test('reset-password sans token → état lien invalide direct (aucun appel API)', async ({
		page,
	}) => {
		test.skip(!featureOn, 'KESH_FEATURE_FORGOT_PASSWORD désactivé — cf. recette DE-6 en tête de spec');

		await page.goto('/reset-password');
		await expect(page.getByTestId('reset-invalid-link')).toBeVisible();
		await expect(page.getByTestId('reset-request-new-link')).toBeVisible();
		// Le formulaire n'est pas proposé.
		await expect(page.getByTestId('reset-submit')).toHaveCount(0);
	});

	test('reset-password avec token bidon → lien invalide après submit + CTA refaire une demande', async ({
		page,
	}) => {
		test.skip(!featureOn, 'KESH_FEATURE_FORGOT_PASSWORD désactivé — cf. recette DE-6 en tête de spec');

		await page.goto('/reset-password?token=token-bidon-inexistant');
		await page.getByTestId('reset-password').fill(NEW_PASSWORD);
		await page.getByTestId('reset-password-confirm').fill(NEW_PASSWORD);
		await page.getByTestId('reset-submit').click();

		// 400 INVALID_OR_EXPIRED_TOKEN générique → bascule en état lien invalide.
		await expect(page.getByTestId('reset-invalid-link')).toBeVisible();
		const cta = page.getByTestId('reset-request-new-link');
		await expect(cta).toBeVisible();
		await cta.click();
		await expect(page).toHaveURL(/\/forgot-password/);
	});
});
