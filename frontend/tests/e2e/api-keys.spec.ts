// Story 17-2b — E2E Playwright : page de gestion des clés API (PAT).
//
// Scénario AC9 : login → /settings/api-keys → créer une clé read-write →
// capturer le secret one-time → l'utiliser comme `Authorization: Bearer` réel
// sur GET (200) et POST (201) → révoquer via l'UI → réutiliser → 401.
// + cas clé `read` : POST → 403 API_KEY_READ_ONLY.
//
// Pré-requis : backend `KESH_TEST_MODE=true` + MariaDB + seed CI. Sur
// Ubuntu 26.04+, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`.

import { test, expect, request as playwrightRequest, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, disposeContextSafe } from './helpers/test-state';

const BACKEND_URL = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1';

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Crée une clé via l'UI et retourne le secret `kesh_pat_…` affiché une fois. */
async function createKeyViaUi(page: Page, scope: 'read' | 'read-write'): Promise<string> {
	await page.goto('/settings/api-keys');
	await expect(page.getByTestId('api-keys-page-title')).toBeVisible();
	await page.getByTestId('api-keys-create-button').click();
	await page.getByTestId('api-keys-name-input').fill(`E2E ${scope} ${Date.now()}`);
	await page.getByTestId('api-keys-scope-select').selectOption(scope);
	await page.getByTestId('api-keys-submit').click();

	const secretEl = page.getByTestId('api-keys-secret-value');
	await expect(secretEl).toBeVisible({ timeout: 5000 });
	const secret = (await secretEl.textContent())?.trim() ?? '';
	expect(secret).toMatch(/^kesh_pat_/);
	return secret;
}

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('clé read-write : créée via UI, utilisable en Bearer (GET 200 + POST 201), révoquée → 401', async ({
	page,
}) => {
	await login(page);
	const secret = await createKeyViaUi(page, 'read-write');

	// La clé apparaît dans la liste (active).
	await expect(page.getByTestId('api-keys-list')).toBeVisible();

	// Contexte API avec le PAT réel en Bearer (pas de cookie → pure auth PAT).
	const pat = await playwrightRequest.newContext({
		baseURL: BACKEND_URL,
		extraHTTPHeaders: { Authorization: `Bearer ${secret}` },
	});
	try {
		// GET autorisé.
		const getRes = await pat.get('/api/v1/accounts');
		expect(getRes.status(), 'GET /accounts via PAT read-write').toBe(200);

		// POST autorisé (scope read-write).
		const postRes = await pat.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: `PAT contact ${Date.now()}`,
				isClient: true,
				isSupplier: false,
				defaultPaymentTerms: '30 jours net',
			},
		});
		expect([200, 201], 'POST /contacts via PAT read-write').toContain(postRes.status());

		// Révocation via l'UI.
		await page.getByTestId('api-keys-list').getByRole('button', { name: 'Révoquer' }).click();
		await page.getByTestId('api-keys-revoke-confirm-button').click();
		await expect(page.getByTestId('api-keys-revoke-confirm')).toHaveCount(0);

		// Le token révoqué ne s'authentifie plus.
		const afterRevoke = await pat.get('/api/v1/accounts');
		expect(afterRevoke.status(), 'GET via PAT révoqué').toBe(401);
	} finally {
		await disposeContextSafe(pat);
	}
});

test('clé read : GET autorisé (200) mais POST refusé (403 API_KEY_READ_ONLY)', async ({ page }) => {
	await login(page);
	const secret = await createKeyViaUi(page, 'read');

	const pat = await playwrightRequest.newContext({
		baseURL: BACKEND_URL,
		extraHTTPHeaders: { Authorization: `Bearer ${secret}` },
	});
	try {
		const getRes = await pat.get('/api/v1/accounts');
		expect(getRes.status(), 'GET via PAT read').toBe(200);

		const postRes = await pat.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name: `read PAT contact ${Date.now()}`,
				isClient: true,
				isSupplier: false,
				defaultPaymentTerms: '30 jours net',
			},
		});
		expect(postRes.status(), 'POST via PAT read → refusé').toBe(403);
		const body = await postRes.json();
		expect(body?.error?.code).toBe('API_KEY_READ_ONLY');
	} finally {
		await disposeContextSafe(pat);
	}
});
