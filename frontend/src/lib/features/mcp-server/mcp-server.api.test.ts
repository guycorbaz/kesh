/**
 * Story 32-7 (refs #723) — le client de l'API des clés et des réglages du serveur MCP.
 *
 * AC-3 (a) : `createMcpKey` sépare le secret de la clé à la frontière de l'API —
 * la clé est construite champ par champ ; ni `key` ni un champ inconnu n'y passe.
 * AC-3 (b) : les corps envoyés, comparés EN ENTIER.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

const { getMock, postMock, putMock, deleteMock } = vi.hoisted(() => ({
	getMock: vi.fn(),
	postMock: vi.fn(),
	putMock: vi.fn(),
	deleteMock: vi.fn(),
}));

vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: { get: getMock, post: postMock, put: putMock, delete: deleteMock },
}));

import {
	createMcpKey,
	getMcpServerSettings,
	listMcpKeys,
	revokeMcpKey,
	setMcpKeySendValidation,
	updateMcpServerSettings,
} from './mcp-server.api';

// Factice, hors du format d'un secret réel (revue de code P1, A-7 ; C-32-7-38).
const SECRET = 'kesh_mcp_SECRET_factice_0123456789';

const REPONSE_CLE = {
	id: 4,
	name: 'agent',
	scopes: ['books:read'],
	unknownScopes: [],
	sendValidation: 'required',
	createdByUserId: 1,
	createdAt: '2026-10-25T10:00:00.123',
	lastUsedAt: null,
	revokedAt: null,
	expiresAt: null,
	version: 1,
};

/** Toutes les valeurs d'un objet, récursivement. */
function valeursRecursives(v: unknown): unknown[] {
	if (v === null || typeof v !== 'object') return [v];
	return Object.values(v as Record<string, unknown>).flatMap(valeursRecursives);
}

describe('mcp-server.api', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('createMcpKey_separe_le_secret', async () => {
		postMock.mockResolvedValue({ ...REPONSE_CLE, key: SECRET, zzz: 'en trop' });
		const { cle, secret } = await createMcpKey({
			name: 'agent',
			scopes: ['books:read'],
			sendValidation: 'required',
		});
		expect('key' in cle).toBe(false);
		expect('zzz' in cle).toBe(false);
		expect(valeursRecursives(cle)).not.toContain(SECRET);
		expect(JSON.stringify(cle)).not.toContain(SECRET);
		expect(secret).toBe(SECRET);
		expect(cle).toEqual(REPONSE_CLE);
	});

	it('création : le corps envoyé, sendValidation toujours présent, expiresAt seulement s’il est donné', async () => {
		postMock.mockResolvedValue({ ...REPONSE_CLE, key: SECRET });
		await createMcpKey({ name: 'a', scopes: ['books:read', 'prompts:write'], sendValidation: 'required' });
		expect(postMock).toHaveBeenCalledWith('/api/v1/admin/mcp/keys', {
			name: 'a',
			scopes: ['books:read', 'prompts:write'],
			sendValidation: 'required',
		});
		await createMcpKey({
			name: 'b',
			scopes: ['contacts:read'],
			sendValidation: 'exempt',
			expiresAt: '2026-12-31T22:59:59.999Z',
		});
		expect(postMock.mock.calls[1][1]).toEqual({
			name: 'b',
			scopes: ['contacts:read'],
			sendValidation: 'exempt',
			expiresAt: '2026-12-31T22:59:59.999Z',
		});
	});

	it('une réponse 201 sans secret lisible lève une erreur, jamais « undefined » (revue P1, B2)', async () => {
		for (const sans of [{ ...REPONSE_CLE }, { ...REPONSE_CLE, key: 42 }, { ...REPONSE_CLE, key: 'autre' }]) {
			postMock.mockResolvedValueOnce(sans);
			await expect(
				createMcpKey({ name: 'a', scopes: ['books:read'], sendValidation: 'required' }),
			).rejects.toThrow();
		}
	});

	it('révocation : DELETE avec { version }', async () => {
		deleteMock.mockResolvedValue(undefined);
		await revokeMcpKey(4, 7);
		expect(deleteMock).toHaveBeenCalledWith('/api/v1/admin/mcp/keys/4', { version: 7 });
	});

	it('politique : PUT { sendValidation, version }, la clé relue sans champ en trop', async () => {
		putMock.mockResolvedValue({ ...REPONSE_CLE, sendValidation: 'exempt', version: 2, key: 'x' });
		const cle = await setMcpKeySendValidation(4, 'exempt', 1);
		expect(putMock).toHaveBeenCalledWith('/api/v1/admin/mcp/keys/4/send-validation', {
			sendValidation: 'exempt',
			version: 1,
		});
		expect('key' in cle).toBe(false);
		expect(cle.sendValidation).toBe('exempt');
	});

	it('réglages : GET, puis PUT des six champs exactement', async () => {
		getMock.mockResolvedValue({ enabled: false });
		await getMcpServerSettings();
		expect(getMock).toHaveBeenCalledWith('/api/v1/admin/mcp/settings');
		putMock.mockResolvedValue({});
		const payload = {
			enabled: true,
			addressFilterEnabled: true,
			allowedNetworks: ['10.0.0.0/8'],
			trustedProxies: [],
			allowedOrigins: ['http://x.local'],
			version: 0,
			// un champ en trop, que le corps ne doit pas porter
			updatedAt: null,
		};
		await updateMcpServerSettings(payload);
		expect(putMock).toHaveBeenCalledWith('/api/v1/admin/mcp/settings', {
			enabled: true,
			addressFilterEnabled: true,
			allowedNetworks: ['10.0.0.0/8'],
			trustedProxies: [],
			allowedOrigins: ['http://x.local'],
			version: 0,
		});
	});

	it('liste : GET des clés', async () => {
		getMock.mockResolvedValue([REPONSE_CLE]);
		expect(await listMcpKeys()).toEqual([REPONSE_CLE]);
		expect(getMock).toHaveBeenCalledWith('/api/v1/admin/mcp/keys');
	});
});
