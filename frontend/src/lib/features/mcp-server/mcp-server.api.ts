// Story 32-7 (refs #723) — client de l'API des clés et des réglages du serveur MCP.
//
// Routes de la Story 32-5c (`crates/kesh-api/src/lib.rs`, bloc `admin_routes`) :
// Administrateur seulement ; un PAT est refusé, une clé MCP n'est pas reconnue.
// Le serveur reste l'autorité ; l'écran n'envoie que des corps bien formés.
//
// ⛔ Le secret d'une clé (`key`, réponse 201) est SÉPARÉ de la clé ici même :
// `createMcpKey` construit la clé champ par champ, jamais par décomposition de la
// réponse — le reste de l'écran ne voit jamais un objet qui porte les deux
// (AC-3 (a), C-32-7-5).

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	McpKey,
	McpKeyCreee,
	McpSendValidation,
	McpServerSettings,
	NewMcpKeyPayload,
	UpdateMcpServerSettingsPayload,
} from './mcp-server.types';

export type {
	McpKey,
	McpKeyCreee,
	McpScope,
	McpSendValidation,
	McpServerSettings,
	NewMcpKeyPayload,
	UpdateMcpServerSettingsPayload,
} from './mcp-server.types';

const CLES = '/api/v1/admin/mcp/keys';
const REGLAGES = '/api/v1/admin/mcp/settings';

/** La réponse brute d'une écriture de clé, telle que le client HTTP la rend. */
type ReponseCle = McpKey & { key?: unknown };

/**
 * La clé, champ par champ : les onze champs de `McpKeyResponse`, rien d'autre —
 * un champ inconnu de la réponse (dont `key`) n'est jamais recopié.
 */
function cleDepuis(r: ReponseCle): McpKey {
	return {
		id: r.id,
		name: r.name,
		scopes: r.scopes,
		unknownScopes: r.unknownScopes,
		sendValidation: r.sendValidation,
		createdByUserId: r.createdByUserId,
		createdAt: r.createdAt,
		lastUsedAt: r.lastUsedAt,
		revokedAt: r.revokedAt,
		expiresAt: r.expiresAt,
		version: r.version,
	};
}

/** `GET /api/v1/admin/mcp/keys` — toutes les clés de la société, révoquées comprises. */
export async function listMcpKeys(): Promise<McpKey[]> {
	return apiClient.get<McpKey[]>(CLES);
}

/**
 * `POST /api/v1/admin/mcp/keys` — crée une clé. Rend la clé (sans secret) et le
 * secret `kesh_mcp_…`, à part : il n'est affiché qu'une fois.
 */
export async function createMcpKey(payload: NewMcpKeyPayload): Promise<McpKeyCreee> {
	const corps: NewMcpKeyPayload = {
		name: payload.name,
		scopes: payload.scopes,
		sendValidation: payload.sendValidation,
	};
	if (payload.expiresAt !== undefined) corps.expiresAt = payload.expiresAt;
	const r = await apiClient.post<ReponseCle>(CLES, corps);
	// Un secret absent ou d'une autre forme ne s'affiche jamais (« undefined » copié comme
	// une clé) : la création se dit en échec, et l'erreur ne porte pas la valeur reçue
	// (revue de code P1, B2).
	if (typeof r.key !== 'string' || !r.key.startsWith('kesh_mcp_')) {
		throw new Error('réponse de création de clé MCP sans secret lisible');
	}
	return { cle: cleDepuis(r), secret: r.key };
}

/** `DELETE /api/v1/admin/mcp/keys/{id}` — 204 ; 404 absente ; 409 périmée ou déjà révoquée. */
export async function revokeMcpKey(id: number, version: number): Promise<void> {
	await apiClient.delete<void>(`${CLES}/${id}`, { version });
}

/**
 * `PUT /api/v1/admin/mcp/keys/{id}/send-validation` — rend la clé relue ;
 * 409 si révoquée ou version périmée, 404 absente.
 */
export async function setMcpKeySendValidation(
	id: number,
	sendValidation: McpSendValidation,
	version: number,
): Promise<McpKey> {
	const r = await apiClient.put<ReponseCle>(`${CLES}/${id}/send-validation`, { sendValidation, version });
	return cleDepuis(r);
}

/** `GET /api/v1/admin/mcp/settings` — les réglages, ou les défauts à `version: 0`. */
export async function getMcpServerSettings(): Promise<McpServerSettings> {
	return apiClient.get<McpServerSettings>(REGLAGES);
}

/** `PUT /api/v1/admin/mcp/settings` — remplacement complet ; rend les réglages relus. */
export async function updateMcpServerSettings(
	payload: UpdateMcpServerSettingsPayload,
): Promise<McpServerSettings> {
	const corps: UpdateMcpServerSettingsPayload = {
		enabled: payload.enabled,
		addressFilterEnabled: payload.addressFilterEnabled,
		allowedNetworks: payload.allowedNetworks,
		trustedProxies: payload.trustedProxies,
		allowedOrigins: payload.allowedOrigins,
		version: payload.version,
	};
	return apiClient.put<McpServerSettings>(REGLAGES, corps);
}
