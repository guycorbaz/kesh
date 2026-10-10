/**
 * Appels HTTP de l'écran des postes ouverts (Story 15-1c-i, #518).
 *
 * Aucune règle n'est recopiée ici : la lettrabilité, la possession par une
 * pièce, la période et la prévision du délettrage viennent du serveur.
 */
import { apiClient } from '$lib/shared/utils/api-client';
import type {
	LetteringCreated,
	LetteringDetail,
	OpenItemsResponse,
	ProposalsResponse,
} from './open-items.types';

/** Taille de page des postes ouverts (défaut du serveur). */
export const OPEN_ITEMS_PAGE_SIZE = 50;

/**
 * Les postes ouverts d'un compte à `asOf`. ⚠️ `asOf` est **toujours** envoyé
 * (C-15-1c-5) : l'omettre rendrait la date UTC du serveur, en décalage la nuit.
 */
export async function fetchOpenItems(
	accountId: number,
	asOf: string,
	offset = 0,
	limit = OPEN_ITEMS_PAGE_SIZE,
): Promise<OpenItemsResponse> {
	const q = new URLSearchParams({
		asOf,
		limit: String(limit),
		offset: String(offset),
	});
	return apiClient.get<OpenItemsResponse>(`/api/v1/accounts/${accountId}/open-items?${q}`);
}

/** Les rapprochements proposés — calculés **aujourd'hui**, sans `asOf` ni `offset`. */
export async function fetchProposals(accountId: number): Promise<ProposalsResponse> {
	return apiClient.get<ProposalsResponse>(`/api/v1/accounts/${accountId}/lettering-proposals`);
}

/** Lettre les lignes données (201) — un `POST` par geste de l'utilisateur. */
export async function createLettering(lineIds: number[]): Promise<LetteringCreated> {
	return apiClient.post<LetteringCreated>('/api/v1/letterings', { lineIds });
}

/** Lit un groupe en détail, par son code ou sa clé. */
export async function fetchLettering(code: string): Promise<LetteringDetail> {
	return apiClient.get<LetteringDetail>(`/api/v1/letterings/${encodeURIComponent(code)}`);
}

/** Délettre un groupe (204). */
export async function deleteLettering(code: string): Promise<void> {
	await apiClient.delete<void>(`/api/v1/letterings/${encodeURIComponent(code)}`);
}
