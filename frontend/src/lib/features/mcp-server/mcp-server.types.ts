// Story 32-7 (refs #723) — types de l'écran « Serveur MCP » (Administration).
//
// Formes rendues par les routes de la Story 32-5c :
// - `McpKey` = `McpKeyResponse` (`crates/kesh-api/src/routes/mcp_keys.rs:56-73`),
//   onze champs ; les quatre dates sont des `NaiveDateTime` sérialisées SANS
//   fuseau, en UTC (#738 B7 = E6) — elles se lisent par `depuisUtcNaif`.
// - `McpServerSettings` = `McpServerSettingsResponse`
//   (`routes/mcp_server_settings.rs:63-102`), `updatedAt` AVEC son `Z`.
//
// ⛔ `McpKey` n'a PAS de champ `key` : le secret de la création est séparé de la
// clé à la frontière de l'API (`createMcpKey`, AC-3 (a), C-32-7-5).

/** Les cinq portées d'une clé MCP. */
export type McpScope = 'books:read' | 'invoices:read' | 'contacts:read' | 'documents:read' | 'prompts:write';

/**
 * Les portées dans l'ordre canonique de `McpScope::ALL`
 * (`crates/kesh-db/src/entities/mcp_key.rs`) — tableau UNIQUE de l'écran : les
 * cases du formulaire suivent cet ordre (AC-10).
 */
export const PORTEES_MCP: readonly McpScope[] = [
	'books:read',
	'invoices:read',
	'contacts:read',
	'documents:read',
	'prompts:write',
];

/** La politique de validation avant envoi d'une clé (Q12). */
export type McpSendValidation = 'required' | 'exempt';

/** Une clé MCP telle que la liste la rend — jamais le secret ni son condensat. */
export interface McpKey {
	id: number;
	name: string;
	scopes: McpScope[];
	/** Portées stockées inconnues de cette version : jamais accordées (C-32-5-37). */
	unknownScopes: string[];
	sendValidation: McpSendValidation;
	createdByUserId: number;
	createdAt: string;
	lastUsedAt: string | null;
	revokedAt: string | null;
	expiresAt: string | null;
	version: number;
}

/** Corps de création. `sendValidation` est toujours présent (C-32-7-7). */
export interface NewMcpKeyPayload {
	name: string;
	scopes: McpScope[];
	sendValidation: McpSendValidation;
	/** RFC 3339 avec fuseau (`toISOString()`), facultatif. */
	expiresAt?: string;
}

/** Ce que rend la création : la clé, sans secret, et le secret, à part. */
export interface McpKeyCreee {
	cle: McpKey;
	secret: string;
}

/** Les réglages du serveur MCP de la société. */
export interface McpServerSettings {
	enabled: boolean;
	addressFilterEnabled: boolean;
	allowedNetworks: string[];
	trustedProxies: string[];
	allowedOrigins: string[];
	/** `0` : aucune ligne, défauts du code. */
	version: number;
	/** UTC avec `Z` ; `null` pour les défauts. */
	updatedAt: string | null;
}

/** Corps de `PUT /api/v1/admin/mcp/settings` : les six champs, rien d'autre. */
export interface UpdateMcpServerSettingsPayload {
	enabled: boolean;
	addressFilterEnabled: boolean;
	allowedNetworks: string[];
	trustedProxies: string[];
	allowedOrigins: string[];
	version: number;
}
