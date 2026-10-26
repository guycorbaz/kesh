// Story 32-8c (refs #723) — E2E Playwright : la validation avant envoi du
// serveur MCP, de bout en bout (AC-C7).
//
// Préparation par l'API d'administration de la 32-5c (session d'Administrateur) :
// serveur MCP activé (`PUT /api/v1/admin/mcp/settings`), une clé `books:read`
// dont la validation est requise (le défaut, `POST /api/v1/admin/mcp/keys`).
// Le « client MCP » est un contexte de requête NEUF, sans cookie, porteur de la
// seule clé (`Authorization: Bearer kesh_mcp_…`).
//
// ⛔ La propriété éprouvée : la valeur LUE DANS LA PAGE (`inputValue()`) juste
// avant le clic est, octet pour octet, le second bloc que le client MCP relit
// par `kesh_lire_envoi`. C'est le seul test qui voit la valeur traverser la
// frontière HTTP depuis la `textarea` : Vitest fige le corps, les tests Rust la
// validation, et ni l'un ni l'autre ne voit l'écran.
//
// Pré-requis : backend `KESH_TEST_MODE=true` + MariaDB + seed CI ; sur Ubuntu
// 26.04+, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`. Aucune donnée
// réelle : la seule donnée lue est la version de Kesh (`kesh_version`).

import {
	test,
	expect,
	request as playwrightRequest,
	type APIRequestContext,
	type Page,
} from '@playwright/test';
import {
	seedTestState,
	clearAuthStorage,
	authedApiContext,
	disposeContextSafe,
} from './helpers/test-state';

const BACKEND_URL = process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1';

async function login(page: Page): Promise<void> {
	await page.goto('/login');
	await page.fill('#username', 'admin');
	await page.fill('#password', 'admin123');
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Active le serveur MCP de la société et crée une clé `books:read`, validation requise. */
async function preparer(page: Page): Promise<string> {
	const admin = await authedApiContext(page);
	try {
		const lus = await admin.get('/api/v1/admin/mcp/settings');
		expect(lus.status(), 'GET réglages').toBe(200);
		const r = await lus.json();
		const ecrits = await admin.put('/api/v1/admin/mcp/settings', {
			data: {
				enabled: true,
				addressFilterEnabled: r.addressFilterEnabled,
				allowedNetworks: r.allowedNetworks,
				trustedProxies: r.trustedProxies,
				allowedOrigins: r.allowedOrigins,
				version: r.version,
			},
		});
		expect(ecrits.status(), `PUT réglages : ${await ecrits.text()}`).toBe(200);

		const cree = await admin.post('/api/v1/admin/mcp/keys', {
			data: { name: `E2E 328c ${Date.now()}`, scopes: ['books:read'] },
		});
		expect(cree.status(), `POST clé : ${await cree.text()}`).toBe(201);
		const cle = await cree.json();
		expect(cle.sendValidation).toBe('required');
		expect(cle.key).toMatch(/^kesh_mcp_/);
		return cle.key as string;
	} finally {
		await disposeContextSafe(admin);
	}
}

/** Le client MCP : un contexte neuf, sans cookie, la clé seule. */
async function clientMcp(cle: string): Promise<APIRequestContext> {
	return playwrightRequest.newContext({
		baseURL: BACKEND_URL,
		extraHTTPHeaders: {
			Authorization: `Bearer ${cle}`,
			'MCP-Protocol-Version': '2025-11-25',
		},
	});
}

let idJsonRpc = 0;

/** Un `tools/call` ; rend le `result` JSON-RPC. */
async function appeler(mcp: APIRequestContext, nom: string, args: Record<string, unknown>) {
	idJsonRpc += 1;
	const res = await mcp.post('/mcp', {
		data: { jsonrpc: '2.0', id: idJsonRpc, method: 'tools/call', params: { name: nom, arguments: args } },
	});
	expect(res.status(), `POST /mcp ${nom} : ${await res.text()}`).toBe(200);
	const corps = await res.json();
	expect(corps.id).toBe(idJsonRpc);
	return corps.result;
}

/** Dépose un envoi (`kesh_version` sous une clé requise) ; rend son numéro. */
async function deposer(mcp: APIRequestContext): Promise<number> {
	const r = await appeler(mcp, 'kesh_version', {});
	expect(r.isError).toBe(false);
	expect(r.structuredContent.etat).toBe('en_attente');
	expect(Number.isInteger(r.structuredContent.envoi)).toBe(true);
	return r.structuredContent.envoi as number;
}

/** Ouvre le détail d'un envoi et attend sa zone de texte. */
async function ouvrir(page: Page, n: number) {
	await page.goto(`/admin/mcp/sends/${n}`);
	await expect(page.getByTestId('mcp-send-title')).toBeVisible();
	const zone = page.getByTestId('mcp-send-content');
	await expect(zone).toBeVisible();
	return zone;
}

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('la valeur lue dans la page est celle que le client MCP reçoit — modifiée, telle quelle, refusée', async ({
	page,
}) => {
	await login(page);
	const cle = await preparer(page);
	const mcp = await clientMcp(cle);
	try {
		// (1) Modifiée — la première ligne retirée, une ligne ajoutée au clavier.
		const x = await deposer(mcp);
		const zone = await ouvrir(page, x);
		const v1 = await zone.inputValue();
		expect(v1.length).toBeGreaterThan(0);
		const lignes = v1.split('\n');
		const voulu = [...lignes.slice(1), 'E2E_328 — é ✓'].join('\n');
		await page.getByTestId('mcp-send-modify').click();
		await zone.click();
		await page.keyboard.press('ControlOrMeta+a');
		await page.keyboard.press('Delete');
		await zone.pressSequentially(voulu);
		const v2 = await zone.inputValue();
		expect(v2).toBe(voulu);
		await page.getByTestId('mcp-send-approve-modified').click();
		await expect(page.getByTestId('mcp-send-approved-how')).toBeVisible();
		const relu = await appeler(mcp, 'kesh_lire_envoi', { envoi: x });
		expect(relu.isError).toBe(false);
		expect(relu.structuredContent).toEqual({ envoi: x, etat: 'valide', modifie: true });
		expect(relu.content).toHaveLength(2);
		expect(relu.content[1]).toEqual({ type: 'text', text: v2 });

		// (2) Telle quelle.
		const y = await deposer(mcp);
		const zoneY = await ouvrir(page, y);
		const v1y = await zoneY.inputValue();
		await page.getByTestId('mcp-send-approve').click();
		await expect(page.getByTestId('mcp-send-approved-how')).toBeVisible();
		const reluY = await appeler(mcp, 'kesh_lire_envoi', { envoi: y });
		expect(reluY.isError).toBe(false);
		expect(reluY.structuredContent).toEqual({ envoi: y, etat: 'valide', modifie: false });
		expect(reluY.content[1]).toEqual({ type: 'text', text: v1y });

		// (3) Refusé, après confirmation.
		const z = await deposer(mcp);
		await ouvrir(page, z);
		await page.getByTestId('mcp-send-refuse').click();
		await page.getByTestId('mcp-send-refuse-confirm').click();
		await expect(page.getByTestId('mcp-send-erased')).toBeVisible();
		const reluZ = await appeler(mcp, 'kesh_lire_envoi', { envoi: z });
		expect(reluZ.isError).toBe(true);
		expect(reluZ.structuredContent).toBeUndefined();

		// La liste montre les trois, sans contenu.
		await page.goto('/admin/mcp/sends');
		await expect(page.getByTestId(`mcp-sends-row-${x}`)).toBeVisible();
		await expect(page.getByTestId(`mcp-sends-row-${z}`)).toBeVisible();
		await expect(page.getByTestId('mcp-sends-table')).not.toContainText('E2E_328');
	} finally {
		await disposeContextSafe(mcp);
	}
});

test('une modification portant un caractère invisible est refusée par le serveur, expliquée à l’écran', async ({
	page,
}) => {
	await login(page);
	const cle = await preparer(page);
	const mcp = await clientMcp(cle);
	try {
		const n = await deposer(mcp);
		const zone = await ouvrir(page, n);
		await page.getByTestId('mcp-send-modify').click();
		await zone.fill('avertissement ⚠️');
		await page.getByTestId('mcp-send-approve-modified').click();
		await expect(page.getByTestId('mcp-send-error')).toBeVisible();
		await expect(page.getByTestId('mcp-send-error-help')).toBeVisible();
		const relu = await appeler(mcp, 'kesh_lire_envoi', { envoi: n });
		expect(relu.structuredContent).toEqual({ envoi: n, etat: 'en_attente' });
	} finally {
		await disposeContextSafe(mcp);
	}
});
