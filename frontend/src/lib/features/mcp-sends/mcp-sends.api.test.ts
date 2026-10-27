/**
 * Story 32-8c (refs #723) — le client de l'API des envois du serveur MCP.
 *
 * ⛔ Ce qui part de l'écran se fige ici **au corps près** (AC-C4, #768 point 4) :
 * le serveur lit `content` absent ou nul comme « tel quel » ; le corps d'une
 * version modifiée porte donc `content` en chaîne, exacte, et celui de « tel
 * quel » ne le porte pas du tout.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authState } from '$lib/app/stores/auth.svelte';
import {
	corpsModifie,
	corpsRefus,
	corpsTelQuel,
	decideMcpSend,
	getMcpSend,
	listMcpSends,
} from './mcp-sends.api';

function reponse(corps: unknown, status = 200): Response {
	return {
		ok: status < 400,
		status,
		json: () => Promise.resolve(corps),
		headers: new Headers(),
	} as Response;
}

describe('mcp-sends.api', () => {
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

	it('listMcpSends : GET /api/v1/admin/mcp/sends', async () => {
		mockFetch.mockResolvedValueOnce(reponse([]));
		await listMcpSends();
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/admin/mcp/sends');
		expect(init.method).toBe('GET');
	});

	it('getMcpSend : GET /api/v1/admin/mcp/sends/{id}', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ id: 7 }));
		await getMcpSend(7);
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/admin/mcp/sends/7');
		expect(init.method).toBe('GET');
	});

	// MC2 : « tel quel » envoie `content`.
	it('tel_quel_n_envoie_pas_de_contenu — corps exact, sans clé `content`', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ id: 7 }));
		await decideMcpSend(7, corpsTelQuel(3));
		const [url, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/admin/mcp/sends/7/decision');
		expect(init.method).toBe('POST');
		expect(init.body).toBe('{"version":3,"decision":"approve"}');
		expect(Object.keys(JSON.parse(init.body as string))).not.toContain('content');
	});

	// MC1 : `trim()` sur la valeur envoyée.
	it('envoie_la_valeur_exacte — espace de tête, tabulation, ligne vide finale, U+202E', async () => {
		const valeur = ' tête\tcolonne\n\\u{202E} code visible\n‮caractère réel\n\n';
		mockFetch.mockResolvedValueOnce(reponse({ id: 7 }));
		await decideMcpSend(7, corpsModifie(4, valeur));
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(init.body).toBe(JSON.stringify({ version: 4, decision: 'approve', content: valeur }));
		expect(JSON.parse(init.body as string).content).toBe(valeur);
	});

	it('une version modifiée vide part comme chaîne vide, jamais comme `null`', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ id: 7 }));
		await decideMcpSend(7, corpsModifie(4, ''));
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(init.body).toBe('{"version":4,"decision":"approve","content":""}');
	});

	it('refus : corps exact, sans contenu', async () => {
		mockFetch.mockResolvedValueOnce(reponse({ id: 7 }));
		await decideMcpSend(7, corpsRefus(2));
		const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
		expect(init.body).toBe('{"version":2,"decision":"refuse"}');
	});
});
