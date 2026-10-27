// Story 32-8c (refs #723) — client de l'API des envois du serveur MCP.
//
// Routes (Story 32-8b, `routes/mcp_envois.rs`), dans le bloc `admin_routes` :
// session d'Administrateur seulement — un Comptable reçoit 403, une clé d'API
// 403, une clé du serveur MCP 401. L'agent ne peut pas valider son propre envoi.
//
// ⛔ Ce qui part est ce qui a été vu (§ *La propriété centrale* de la fiche
// 32-8) : « tel quel » n'envoie PAS le texte — le serveur valide la valeur
// stockée à la version lue ; « modifié » envoie la valeur de la `textarea`
// telle quelle, sans `trim` ni normalisation. Les trois fabriques de corps
// ci-dessous sont les seules à le construire.

import { apiClient } from '$lib/shared/utils/api-client';
import type { McpSendDecisionBody, McpSendDetail, McpSendSummary } from './mcp-sends.types';

export type {
	McpSendDecisionBody,
	McpSendDetail,
	McpSendState,
	McpSendSummary,
} from './mcp-sends.types';

const BASE = '/api/v1/admin/mcp/sends';

/** `GET /api/v1/admin/mcp/sends` — les envois de la société, en attente d'abord. */
export async function listMcpSends(): Promise<McpSendSummary[]> {
	return apiClient.get<McpSendSummary[]>(BASE);
}

/** `GET /api/v1/admin/mcp/sends/{id}` — le détail ; 404 hors de la société. */
export async function getMcpSend(id: number): Promise<McpSendDetail> {
	return apiClient.get<McpSendDetail>(`${BASE}/${id}`);
}

/** Valider tel quel : **aucun** champ `content`. */
export function corpsTelQuel(version: number): McpSendDecisionBody {
	return { version, decision: 'approve' };
}

/**
 * Valider une version modifiée : `content` est la valeur reçue, **exacte** —
 * le serveur la refuse (400) s'il y trouve un caractère invisible ou si elle
 * dépasse la taille d'un envoi ; il ne la corrige jamais, l'écran non plus.
 */
export function corpsModifie(version: number, content: string): McpSendDecisionBody {
	return { version, decision: 'approve', content };
}

/** Refuser : aucun contenu (le serveur refuse un refus qui en porterait un). */
export function corpsRefus(version: number): McpSendDecisionBody {
	return { version, decision: 'refuse' };
}

/**
 * `POST /api/v1/admin/mcp/sends/{id}/decision` — rend le détail relu après la
 * décision. 404 introuvable ; 409 déjà décidé, échu ou version périmée ; 400
 * contenu refusé. Un `POST` n'est jamais rejoué par le client.
 */
export async function decideMcpSend(id: number, corps: McpSendDecisionBody): Promise<McpSendDetail> {
	return apiClient.post<McpSendDetail>(`${BASE}/${id}/decision`, corps);
}
