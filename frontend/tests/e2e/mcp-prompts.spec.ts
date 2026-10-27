// Story 32-10b (refs #723) — E2E Playwright : l'écran des prompts du serveur MCP,
// de bout en bout (AC-16).
//
// Préparation par l'API d'administration (session d'Administrateur) : serveur MCP
// activé (`PUT /api/v1/admin/mcp/settings`), une clé `books:read` + `prompts:write`
// (`POST /api/v1/admin/mcp/keys`). Le « client MCP » est un contexte de requête
// NEUF, sans cookie, porteur de la seule clé (`Authorization: Bearer kesh_mcp_…`).
//
// ⛔ La propriété éprouvée : le texte LU DANS LA PAGE avant le geste (approbation,
// enregistrement, retour au standard) est, octet pour octet, celui que `prompts/get`
// sert ensuite. C'est le seul test qui voit la valeur traverser la frontière HTTP :
// Vitest fige l'affichage sur des réponses simulées, les tests Rust la composition,
// et ni l'un ni l'autre ne voit les deux bouts.
//
// Noms de prompt uniques (`e2e-3210b-<horodatage>-…`) : la base E2E est partagée
// entre runs et specs ; `kesh-assistant` aussi, d'où des comptes RELATIFS.
//
// Pré-requis : backend `KESH_TEST_MODE=true` + MariaDB + seed CI ; sur Ubuntu
// 26.04+, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`. Aucune donnée réelle.

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
const ADMIN = { username: 'admin', password: 'admin123' };

/** Le début de la marque d'un prompt d'agent (`services/prompts_mcp.rs`, `marque_agent`). */
const DEBUT_DE_MARQUE = '[Prompt écrit par un agent';

async function login(page: Page, username = ADMIN.username, password = ADMIN.password) {
	await page.goto('/login');
	await page.fill('#username', username);
	await page.fill('#password', password);
	await page.click('button[type="submit"]');
	await expect(page).toHaveURL('/');
}

/** Active le serveur MCP de la société et crée une clé `books:read` + `prompts:write`. */
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
			data: { name: `E2E 3210b ${Date.now()}`, scopes: ['books:read', 'prompts:write'] },
		});
		expect(cree.status(), `POST clé : ${await cree.text()}`).toBe(201);
		const cle = await cree.json();
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

/** Une requête JSON-RPC ; rend le corps entier (`result` ou `error`). */
async function rpc(mcp: APIRequestContext, method: string, params: Record<string, unknown>) {
	idJsonRpc += 1;
	const res = await mcp.post('/mcp', { data: { jsonrpc: '2.0', id: idJsonRpc, method, params } });
	expect(res.status(), `POST /mcp ${method} : ${await res.text()}`).toBe(200);
	const corps = await res.json();
	expect(corps.id).toBe(idJsonRpc);
	return corps;
}

/** `proposer_prompt` : une proposition d'agent, à valider. */
async function proposer(mcp: APIRequestContext, nom: string, description: string, texte: string) {
	const corps = await rpc(mcp, 'tools/call', {
		name: 'proposer_prompt',
		arguments: { nom, description, texte },
	});
	expect(corps.result?.isError, JSON.stringify(corps)).toBe(false);
}

/** L'identifiant d'un prompt de la société, par son nom (API de session). */
async function idDuPrompt(page: Page, nom: string): Promise<number> {
	const admin = await authedApiContext(page);
	try {
		const res = await admin.get('/api/v1/admin/mcp/prompts');
		expect(res.status()).toBe(200);
		const liste = (await res.json()) as { id: number; nom: string }[];
		const p = liste.find((x) => x.nom === nom);
		expect(p, `prompt ${nom}`).toBeDefined();
		return p!.id;
	} finally {
		await disposeContextSafe(admin);
	}
}

/** Le détail d'un prompt, lu en session. */
async function detailDuPrompt(page: Page, id: number) {
	const admin = await authedApiContext(page);
	try {
		const res = await admin.get(`/api/v1/admin/mcp/prompts/${id}`);
		expect(res.status()).toBe(200);
		return await res.json();
	} finally {
		await disposeContextSafe(admin);
	}
}

async function ouvrir(page: Page, id: number) {
	await page.goto(`/admin/mcp/prompts/${id}`);
	await expect(page.getByTestId('mcp-prompt-title')).toBeVisible();
}

test.beforeAll(async () => {
	await seedTestState('with-company');
});

test.afterEach(async ({ page }) => {
	await clearAuthStorage(page);
});

test('approbation, refus, version d’administrateur : le texte vu est le texte servi', async ({ page }) => {
	await login(page);
	const cle = await preparer(page);
	const mcp = await clientMcp(cle);
	const horodatage = Date.now();
	const nomA = `e2e-3210b-${horodatage}-a`;
	const nomB = `e2e-3210b-${horodatage}-b`;
	try {
		// (a) Approbation — la valeur servie est la valeur vue.
		await proposer(
			mcp,
			nomA,
			'Résumé des écritures de la période',
			'Premier paragraphe : lisez le grand livre, sans rien écrire.\n\nSecond paragraphe : résumez les écarts, en français.',
		);
		await page.goto('/admin/mcp/prompts');
		const ligne = page.locator(`tr[data-nom="${nomA}"]`);
		await expect(ligne).toBeVisible();
		await expect(ligne.locator('[data-testid^="mcp-prompts-pending-"]')).toHaveText('1 à valider');
		await expect(ligne.locator('[data-testid^="mcp-prompts-not-served-"]')).toBeVisible();
		const idA = await idDuPrompt(page, nomA);
		await ouvrir(page, idA);
		const preA = page.locator('[data-testid^="a-valider-mcp-prompt-served-text-"]');
		await expect(preA).toHaveCount(1);
		const n = Number((await preA.getAttribute('data-testid'))!.split('-').pop());
		const T = (await preA.textContent())!;
		const D = (await page.getByTestId(`a-valider-mcp-prompt-served-description-${n}`).textContent())!;
		expect(T.startsWith(DEBUT_DE_MARQUE), T).toBe(true);
		const lu = await detailDuPrompt(page, idA);
		const vA = lu.versions.find((v: { numero: number }) => v.numero === n);
		expect(T).toBe(vA.texteServi.visible);
		expect(D).toBe(vA.descriptionServie.visible);
		expect(vA.texteServi.visible).toBe(vA.texteServi.exact);
		await page.getByTestId(`mcp-prompt-approve-${n}`).click();
		await expect(page.getByTestId('mcp-prompt-approve-dialog')).toContainText(
			`version n° ${n}, proposée par la clé n°`,
		);
		await page.getByTestId('mcp-prompt-approve-confirm').click();
		await expect(page.getByTestId(`mcp-prompt-in-force-badge-${n}`)).toBeVisible();
		const servi = await rpc(mcp, 'prompts/get', { name: nomA });
		expect(servi.result.messages[0].content.text).toBe(vA.texteServi.exact);
		expect(servi.result.messages[0].content.text).toBe(T);
		expect(servi.result.description).toBe(vA.descriptionServie.exact);
		expect(servi.result.description).toBe(D);
		const liste = await rpc(mcp, 'prompts/list', {});
		expect(liste.result.prompts).toContainEqual({ name: nomA, description: D, arguments: [] });

		// (b) Refus — sur un nom unique neuf.
		await proposer(mcp, nomB, 'Proposition à refuser', 'Texte que l’administrateur refusera.');
		const idB = await idDuPrompt(page, nomB);
		await ouvrir(page, idB);
		const preB = page.locator('[data-testid^="a-valider-mcp-prompt-served-text-"]');
		const nB = Number((await preB.getAttribute('data-testid'))!.split('-').pop());
		await page.getByTestId(`mcp-prompt-refuse-${nB}`).click();
		await page.getByTestId('mcp-prompt-refuse-confirm').click();
		await expect(page.getByTestId(`mcp-prompt-history-state-${nB}`)).toHaveText('Refusée');
		const refuse = await rpc(mcp, 'prompts/get', { name: nomB });
		const inconnu = await rpc(mcp, 'prompts/get', { name: `e2e-3210b-${horodatage}-inconnu` });
		expect(refuse.error).toBeDefined();
		expect(refuse.error.code).toBe(-32602);
		expect(refuse.error).toEqual(inconnu.error);
		const liste2 = await rpc(mcp, 'prompts/list', {});
		expect(liste2.result.prompts.map((p: { name: string }) => p.name)).not.toContain(nomB);

		// (c) Nouvelle version d'administrateur, reprise de la version d'agent.
		await ouvrir(page, idA);
		await page.getByTestId(`mcp-prompt-history-reuse-${n}`).click();
		await expect(page.getByTestId('mcp-prompt-editor-agent-origin')).toBeVisible();
		const zone = page.getByTestId('mcp-prompt-editor-text');
		await zone.click();
		await page.keyboard.press('ControlOrMeta+End');
		await zone.pressSequentially('\nRelu par l’administrateur — é ✓');
		await page.getByTestId('mcp-prompt-editor-save').click();
		const preC = page.locator('[data-testid^="servi-mcp-prompt-served-text-"]');
		await expect(preC).toHaveAttribute('data-testid', `servi-mcp-prompt-served-text-${n + 1}`);
		const T2 = (await preC.textContent())!;
		expect(T2.startsWith(DEBUT_DE_MARQUE)).toBe(false);
		expect(T2.endsWith('Relu par l’administrateur — é ✓')).toBe(true);
		const servi2 = await rpc(mcp, 'prompts/get', { name: nomA });
		expect(servi2.result.messages[0].content.text).toBe(T2);
	} finally {
		await disposeContextSafe(mcp);
	}
});

test('retour au standard : le texte du code montré avant le geste est celui qui est servi', async ({
	page,
}) => {
	await login(page);
	const cle = await preparer(page);
	const mcp = await clientMcp(cle);
	try {
		const id = await idDuPrompt(page, 'kesh-assistant');
		await ouvrir(page, id);
		const lignes = page.locator('[data-testid^="mcp-prompt-history-row-"]');
		const k = await lignes.count();
		expect(k).toBeGreaterThan(0);
		// Une version d'administrateur.
		const zone = page.getByTestId('mcp-prompt-editor-text');
		await zone.click();
		await page.keyboard.press('ControlOrMeta+End');
		await zone.pressSequentially(`\nPersonnalisé par l’E2E 3210b ${Date.now()}.`);
		await page.getByTestId('mcp-prompt-editor-save').click();
		await expect(lignes).toHaveCount(k + 1);
		// Le retour au standard : le texte du code lu DANS LE DIALOGUE, avant la confirmation.
		await page.getByTestId('mcp-prompt-restore-standard').click();
		const T3 = (await page.getByTestId('mcp-prompt-standard-text').textContent())!;
		expect(T3.length).toBeGreaterThan(0);
		await page.getByTestId('mcp-prompt-standard-confirm').click();
		await expect(lignes).toHaveCount(k + 2);
		await expect(page.getByTestId(`mcp-prompt-history-origin-${k + 2}`)).toContainText('Standard');
		await expect(page.getByTestId(`mcp-prompt-history-origin-${k + 1}`)).toContainText('Administrateur');
		await expect(page.getByTestId(`mcp-prompt-in-force-badge-${k + 2}`)).toBeVisible();
		const servi = await rpc(mcp, 'prompts/get', { name: 'kesh-assistant' });
		expect(servi.result.messages[0].content.text).toBe(T3);
	} finally {
		await disposeContextSafe(mcp);
	}
});

test('Comptable : ni l’entrée de menu, ni le contenu', async ({ page }) => {
	await login(page);
	const ctx = await authedApiContext(page);
	const comptable = `comptable-3210b-${Date.now()}`;
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
	await expect(page.locator('nav a[href="/admin/mcp/prompts"]')).toHaveCount(0);
	await page.goto('/admin/mcp/prompts');
	await expect(page.getByTestId('mcp-prompts-error')).toHaveText('Réservé à l’Administrateur.');
	await expect(page.getByTestId('mcp-prompts-table')).toHaveCount(0);
});
