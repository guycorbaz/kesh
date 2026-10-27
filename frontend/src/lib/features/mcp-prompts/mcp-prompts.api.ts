// Story 32-10b (refs #723) — client de l'API des prompts du serveur MCP.
//
// Routes (Story 32-10a-iii, `routes/mcp_prompts.rs`), dans le bloc
// `admin_routes` : session d'Administrateur seulement — un Comptable reçoit
// 403, une clé d'API 403, une clé du serveur MCP 401. L'agent ne peut jamais
// approuver (AC-iii7 de la 32-10a).
//
// ⛔ Ce qu'écrit l'Administrateur part EXACT : ni `trim`, ni normalisation des
// fins de ligne, ni substitution — le serveur normalise (`\r\n` → `\n`) et
// valide (`valider_contenu`). Les quatre fabriques de corps ci-dessous sont les
// seules à construire un corps (AC-2 de la fiche 32-10b).
//
// ⛔ Une décision porte la `version` LUE AVEC L'AFFICHAGE : ces fonctions la
// reçoivent de l'appelant, aucune ne relit le prompt avant d'écrire (AC-7).

import { apiClient } from '$lib/shared/utils/api-client';
import type {
	CorpsCreation,
	CorpsVersion,
	CorpsVersionAttendue,
	McpPromptDetail,
	McpPromptSummary,
	VersionEcrite,
} from './mcp-prompts.types';

export type {
	CorpsCreation,
	CorpsVersion,
	CorpsVersionAttendue,
	McpPromptDetail,
	McpPromptSummary,
	McpPromptVersion,
	VersionEcrite,
} from './mcp-prompts.types';

const BASE = '/api/v1/admin/mcp/prompts';

/** `GET /api/v1/admin/mcp/prompts` — la synthèse de la société, sans texte, par nom. */
export async function listMcpPrompts(): Promise<McpPromptSummary[]> {
	return apiClient.get<McpPromptSummary[]>(BASE);
}

/** `GET /api/v1/admin/mcp/prompts/{id}` — le prompt et tout son historique ; 404 hors de la société. */
export async function getMcpPrompt(id: number): Promise<McpPromptDetail> {
	return apiClient.get<McpPromptDetail>(`${BASE}/${id}`);
}

/** Une version de texte : description et texte, **exacts**. */
export function corpsTexte(version: number, description: string, texte: string): CorpsVersion {
	return { version, nature: 'text', description, texte };
}

/** Un retrait : **aucun** champ de texte (le serveur refuse un retrait qui en porterait). */
export function corpsRetrait(version: number): CorpsVersion {
	return { version, nature: 'withdrawal' };
}

/** Une décision (approbation, refus) : la seule `version` lue avec l'affichage. */
export function corpsDecision(version: number): CorpsVersionAttendue {
	return { version };
}

/** Le retour au standard : la seule `version` lue avec l'affichage. */
export function corpsStandard(version: number): CorpsVersionAttendue {
	return { version };
}

/** `POST /api/v1/admin/mcp/prompts` — un prompt d'administrateur neuf → 201 ; 400 nom ou contenu. */
export async function createMcpPrompt(corps: CorpsCreation): Promise<VersionEcrite> {
	return apiClient.post<VersionEcrite>(BASE, {
		nom: corps.nom,
		description: corps.description,
		texte: corps.texte,
	});
}

/** `POST …/{id}/versions` — une version d'administrateur, texte ou retrait → 201. */
export async function addMcpPromptVersion(id: number, corps: CorpsVersion): Promise<VersionEcrite> {
	return apiClient.post<VersionEcrite>(`${BASE}/${id}/versions`, corps);
}

/** `POST …/{id}/versions/{numero}/approve` — 200 ; 409 périmée, non la dernière, décidée ; 400 non servable. */
export async function approveMcpPromptVersion(
	id: number,
	numero: number,
	version: number,
): Promise<VersionEcrite> {
	return apiClient.post<VersionEcrite>(
		`${BASE}/${id}/versions/${numero}/approve`,
		corpsDecision(version),
	);
}

/** `POST …/{id}/versions/{numero}/refuse` — 200 ; 409 périmée, déjà décidée. */
export async function refuseMcpPromptVersion(
	id: number,
	numero: number,
	version: number,
): Promise<VersionEcrite> {
	return apiClient.post<VersionEcrite>(
		`${BASE}/${id}/versions/${numero}/refuse`,
		corpsDecision(version),
	);
}

/** `POST …/{id}/standard` — le retour au texte du code → 201 ; 409 périmée, non standard, inconnu du code. */
export async function restoreMcpPromptStandard(id: number, version: number): Promise<VersionEcrite> {
	return apiClient.post<VersionEcrite>(`${BASE}/${id}/standard`, corpsStandard(version));
}
