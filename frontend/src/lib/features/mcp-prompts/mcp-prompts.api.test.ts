/**
 * Story 32-10b (refs #723) — le client de l'API des prompts du serveur MCP (AC-2).
 *
 * ⛔ Ce qui part de l'écran se fige ici **au corps près** : chaque fonction
 * appelle le chemin et la méthode attendus, avec un corps égal ; les valeurs
 * partent EXACTES — ni `trim`, ni normalisation des fins de ligne, ni
 * substitution (le serveur normalise et valide, AC-ii2 de la 32-10a).
 *
 * Mutation tenue ici : M9 (`corpsTexte` applique `trim()` au texte).
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	addMcpPromptVersion,
	approveMcpPromptVersion,
	corpsDecision,
	corpsRetrait,
	corpsStandard,
	corpsTexte,
	createMcpPrompt,
	getMcpPrompt,
	listMcpPrompts,
	refuseMcpPromptVersion,
	restoreMcpPromptStandard,
} from './mcp-prompts.api';

function reponse(corps: unknown, status = 200): Response {
	return {
		ok: status < 400,
		status,
		json: () => Promise.resolve(corps),
		headers: new Headers(),
	} as Response;
}

describe('mcp-prompts.api', () => {
	let mockFetch: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		authState.clearSession();
		authState.login({ userId: '1', username: 'admin', role: 'Admin', expiresIn: 900 });
		mockFetch = vi.fn();
		vi.stubGlobal('fetch', mockFetch);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		authState.clearSession();
	});

	/** Le premier appel : chemin, méthode et corps décodé. */
	function appel(): { url: string; method: string; corps: unknown } {
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		return {
			url,
			method: init.method ?? '',
			corps: init.body === undefined ? undefined : JSON.parse(init.body as string),
		};
	}

	it('listMcpPrompts : GET /api/v1/admin/mcp/prompts', async () => {
		mockFetch.mockResolvedValueOnce(reponse([]));
		await listMcpPrompts();
		expect(appel()).toEqual({ url: '/api/v1/admin/mcp/prompts', method: 'GET', corps: undefined });
	});

	it('getMcpPrompt : GET /api/v1/admin/mcp/prompts/{id}', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ id: 7 }));
		await getMcpPrompt(7);
		expect(appel()).toEqual({ url: '/api/v1/admin/mcp/prompts/7', method: 'GET', corps: undefined });
	});

	it('createMcpPrompt : POST, corps égal', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ promptId: 9 }, 201));
		await createMcpPrompt({ nom: 'mon-prompt', description: ' d ', texte: 't\r\n' });
		expect(appel()).toEqual({
			url: '/api/v1/admin/mcp/prompts',
			method: 'POST',
			corps: { nom: 'mon-prompt', description: ' d ', texte: 't\r\n' },
		});
	});

	it('addMcpPromptVersion : POST …/versions, corps égal', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ promptId: 7 }, 201));
		await addMcpPromptVersion(7, corpsTexte(4, 'd', 't'));
		expect(appel()).toEqual({
			url: '/api/v1/admin/mcp/prompts/7/versions',
			method: 'POST',
			corps: { version: 4, nature: 'text', description: 'd', texte: 't' },
		});
	});

	it('approveMcpPromptVersion : POST …/versions/{numero}/approve, { version }', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ promptId: 7 }));
		await approveMcpPromptVersion(7, 3, 12);
		expect(appel()).toEqual({
			url: '/api/v1/admin/mcp/prompts/7/versions/3/approve',
			method: 'POST',
			corps: { version: 12 },
		});
	});

	it('refuseMcpPromptVersion : POST …/versions/{numero}/refuse, { version }', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ promptId: 7 }));
		await refuseMcpPromptVersion(7, 3, 12);
		expect(appel()).toEqual({
			url: '/api/v1/admin/mcp/prompts/7/versions/3/refuse',
			method: 'POST',
			corps: { version: 12 },
		});
	});

	it('restoreMcpPromptStandard : POST …/standard, { version }', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ promptId: 7 }, 201));
		await restoreMcpPromptStandard(7, 5);
		expect(appel()).toEqual({
			url: '/api/v1/admin/mcp/prompts/7/standard',
			method: 'POST',
			corps: { version: 5 },
		});
	});

	// M9 : `corpsTexte` applique `trim()` au texte.
	it('corps_texte_exact — espace de tête, tabulation, ligne vide finale, \\r\\n, U+202E littéral', async () => {
		const description = ' description à espace de tête';
		const texte = 'colonne\tvaleur\r\n\\u{202E} écrit en toutes lettres\n‮caractère réel\n\n';
		expect(corpsTexte(4, description, texte)).toEqual({
			version: 4,
			nature: 'text',
			description,
			texte,
		});
		mockFetch.mockResolvedValueOnce(reponse({ promptId: 7 }, 201));
		await addMcpPromptVersion(7, corpsTexte(4, description, texte));
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(init.body).toBe(JSON.stringify({ version: 4, nature: 'text', description, texte }));
		const corps = JSON.parse(init.body as string);
		expect(corps.texte).toBe(texte);
		expect(corps.description).toBe(description);
	});

	it('corps_retrait_sans_texte — aucun champ de texte', () => {
		const corps = corpsRetrait(6);
		expect(corps).toEqual({ version: 6, nature: 'withdrawal' });
		expect(Object.keys(corps).sort()).toEqual(['nature', 'version']);
	});

	it('corpsDecision et corpsStandard : la seule version', () => {
		expect(corpsDecision(3)).toEqual({ version: 3 });
		expect(corpsStandard(8)).toEqual({ version: 8 });
	});
});
