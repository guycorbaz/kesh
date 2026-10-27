// Story 32-7 (refs #723) — E2E Playwright : l'écran « Serveur MCP » (AC-16).
//
// Parcours, en série : un Administrateur active le serveur depuis l'écran (a), crée
// une clé et la voit une fois (b), la présente à `/mcp` (c), la passe en dispensée
// (d), la révoque (e) ; un Comptable ne voit ni l'entrée ni la page (f).
//
// `seedTestState` appelle `truncate_all`, qui vide `mcp_keys` et `mcp_server_settings`
// (`TABLES_TO_TRUNCATE`, `kesh-db/src/backup.rs`) : la spec repart d'un serveur
// désactivé et sans clé — aucun `afterAll` de remise en état.
//
// Sélecteurs par `data-testid` seulement. Aucune donnée réelle ; la clé créée est
// révoquée par la spec elle-même.
//
// Pré-requis : backend `KESH_TEST_MODE=true` + MariaDB + seed CI ; sur Ubuntu 26.04+,
// `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`.

import { test, expect, request as playwrightRequest, type APIRequestContext, type Page } from '@playwright/test';
import { seedTestState, clearAuthStorage, authedApiContext, disposeContextSafe } from './helpers/test-state';

const BACKEND_URL = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1';

/** Le 401 fixe de la 32-6a (`crates/kesh-api/src/mcp/textes.rs`, `TEXTE_NON_AUTHENTIFIE`). */
const NON_AUTHENTIFIE = { jsonrpc: '2.0', error: { code: -32001, message: 'Non authentifié.' } };

async function login(page: Page, username = 'admin', password = 'admin123'): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Un `initialize` JSON-RPC présenté à `/mcp` par un contexte neuf, sans cookie, la clé seule. */
async function initialiser(cle: string): Promise<{ status: number; corps: unknown }> {
	const client: APIRequestContext = await playwrightRequest.newContext({
		baseURL: BACKEND_URL,
		extraHTTPHeaders: { Authorization: `Bearer ${cle}`, 'Content-Type': 'application/json' },
	});
	try {
		const r = await client.post('/mcp', {
			data: {
				jsonrpc: '2.0',
				id: 1,
				method: 'initialize',
				params: {
					protocolVersion: '2025-11-25',
					capabilities: {},
					clientInfo: { name: 'e2e-327', version: '0' },
				},
			},
		});
		const texte = await r.text();
		let corps: unknown = texte;
		try {
			corps = JSON.parse(texte);
		} catch {
			// corps non JSON : rendu tel quel
		}
		return { status: r.status(), corps };
	} finally {
		await disposeContextSafe(client);
	}
}

test.describe.configure({ mode: 'serial' });

let secret = '';

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('(a) activer le serveur depuis l’écran', async ({ page }) => {
	await login(page);
	await page.getByTestId('nav-group-administration').click();
	await page.getByTestId('nav-link-admin-mcp').click();
	await expect(page).toHaveURL('/admin/mcp');
	const actif = page.getByTestId('mcp-server-settings-enabled');
	await expect(page.getByTestId('mcp-server-settings-save')).toBeEnabled();
	await expect(actif).not.toBeChecked();
	await actif.check();
	await page.getByTestId('mcp-server-settings-save').click();
	await expect(page.getByTestId('mcp-server-settings-saved')).toBeVisible();
	const ctx = await authedApiContext(page);
	try {
		const r = await ctx.get('/api/v1/admin/mcp/settings');
		expect(r.status()).toBe(200);
		expect((await r.json()).enabled).toBe(true);
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('(b) créer une clé et ne la voir qu’une fois ; (c) elle ouvre /mcp', async ({ page }) => {
	await login(page);
	await page.goto('/admin/mcp');
	await page.getByTestId('mcp-server-keys-new').click();
	await page.getByTestId('mcp-server-form-name').fill('agent E2E 327');
	await page.getByTestId('mcp-server-form-scope-books-read').check();
	await expect(page.getByTestId('mcp-server-form-policy-required')).toBeChecked();
	await page.getByTestId('mcp-server-form-submit').click();
	const valeur = page.getByTestId('mcp-server-secret-value');
	await expect(valeur).toBeVisible();
	secret = ((await valeur.textContent()) ?? '').trim();
	expect(secret).toMatch(/^kesh_mcp_[0-9A-Za-z]{27}$/);

	await page.getByTestId('mcp-server-secret-close').click();
	await expect(page.getByTestId('mcp-server-secret-notice')).toHaveCount(0);
	expect(await page.content()).not.toContain(secret);
	await page.reload();
	await expect(page.getByTestId('mcp-server-keys-table')).toBeVisible();
	expect(await page.content()).not.toContain(secret);
	expect(page.url()).not.toContain(secret);
	const stockage = await page.evaluate(() => {
		const tout: string[] = [];
		for (const s of [localStorage, sessionStorage]) {
			for (let i = 0; i < s.length; i++) tout.push(s.getItem(s.key(i) ?? '') ?? '');
		}
		return tout.join('\n');
	});
	expect(stockage).not.toContain(secret);

	// (c) la clé montrée est la vraie
	const r = await initialiser(secret);
	expect(r.status, JSON.stringify(r.corps)).toBe(200);
});

test('(d) passer la clé en dispensée', async ({ page }) => {
	await login(page);
	await page.goto('/admin/mcp');
	const ligne = page.locator('[data-testid^="mcp-server-key-row-"]').first();
	const id = ((await ligne.getAttribute('data-testid')) ?? '').replace('mcp-server-key-row-', '');
	await page.getByTestId(`mcp-server-key-policy-toggle-${id}`).click();
	await page.getByTestId(`mcp-server-key-exempt-confirm-${id}`).click();
	await expect(page.getByTestId(`mcp-server-key-policy-${id}`)).toHaveText('Dispensée');
	const ctx = await authedApiContext(page);
	try {
		const cles = await (await ctx.get('/api/v1/admin/mcp/keys')).json();
		expect(cles.find((k: { id: number }) => String(k.id) === id).sendValidation).toBe('exempt');
	} finally {
		await disposeContextSafe(ctx);
	}
});

test('(e) révoquer : la clé n’ouvre plus /mcp (401 fixe)', async ({ page }) => {
	await login(page);
	await page.goto('/admin/mcp');
	const ligne = page.locator('[data-testid^="mcp-server-key-row-"]').first();
	const id = ((await ligne.getAttribute('data-testid')) ?? '').replace('mcp-server-key-row-', '');
	await page.getByTestId(`mcp-server-key-revoke-${id}`).click();
	await page.getByTestId(`mcp-server-key-revoke-confirm-${id}`).click();
	await expect(page.getByTestId(`mcp-server-key-status-${id}`)).toContainText('Révoquée');
	const r = await initialiser(secret);
	expect(r.status).toBe(401);
	expect(r.corps).toEqual(NON_AUTHENTIFIE);
});

test('(f) un Comptable : ni l’entrée de menu, ni la page', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const comptable = `comptable-327-${Date.now()}`;
	try {
		const cree = await ctx.post('/api/v1/users', {
			data: { username: comptable, password: 'MotDePasse12345', role: 'Comptable' },
		});
		expect(cree.ok(), `création du comptable : ${cree.status()}`).toBeTruthy();
	} finally {
		await disposeContextSafe(ctx);
	}
	await clearAuthStorage(page);
	await login(page, comptable, 'MotDePasse12345');
	await expect(page.getByTestId('nav-link-admin-mcp')).toHaveCount(0);
	await page.goto('/admin/mcp');
	await expect(page).toHaveURL('/');
});
