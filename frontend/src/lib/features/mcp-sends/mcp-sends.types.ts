// Story 32-8c (refs #723) — types de l'écran des envois du serveur MCP.
//
// Ils suivent, champ pour champ, les réponses de `routes/mcp_envois.rs`
// (`EnvoiResumeResponse`, `EnvoiDetailResponse`, `DecisionRequest`), en
// camelCase. Les dates sont des `NaiveDateTime` de MariaDB, à l'heure UTC
// (`time_zone = '+00:00'`, `kesh-db/src/pool.rs`), sans suffixe de fuseau.

/** L'état d'un envoi ; `expired` est dérivé de l'échéance par le serveur. */
export type McpSendState = 'pending' | 'approved' | 'refused' | 'expired';

/** Une ligne de la liste — **sans** contenu ni résumé (l'API ne les rend pas). */
export interface McpSendSummary {
	id: number;
	mcpKeyId: number;
	mcpKeyName: string;
	mcpKeyRevoked: boolean;
	toolName: string;
	state: McpSendState;
	/** Vrai si l'envoi a été validé dans une version modifiée. */
	modified: boolean;
	expiresAt: string;
	decidedAt: string | null;
	/** Le nom de l'utilisateur qui a décidé, s'il y a lieu. */
	decidedBy: string | null;
	createdAt: string;
}

/**
 * Le détail d'un envoi. `summary` et `content` sont nuls pour un envoi refusé
 * ou échu (contenu effacé) ; `content` est la chaîne **exacte** que l'agent
 * relira si l'envoi est validé tel quel.
 */
export interface McpSendDetail extends McpSendSummary {
	summary: string | null;
	content: string | null;
	contentBytes: number | null;
	version: number;
}

/**
 * Le corps d'une décision. ⚠️ Le serveur lit un `content` absent **ou nul**
 * comme « tel quel » (`#[serde(default)]`, issue #768 point 4) : l'écran
 * n'envoie donc `content` que pour une version modifiée, et toujours comme
 * chaîne — jamais `null`, jamais `undefined`.
 */
export type McpSendDecisionBody =
	| { version: number; decision: 'approve' }
	| { version: number; decision: 'approve'; content: string }
	| { version: number; decision: 'refuse' };
