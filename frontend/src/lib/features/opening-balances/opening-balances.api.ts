/**
 * Client API typé pour le bilan d'ouverture (Story 14-4).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import type { JournalEntryResponse } from '$lib/features/journal-entries/journal-entries.types';
import type { OpeningBalancesRequest, OpeningBalancesStatus } from './opening-balances.types';

/** État de l'écran « Soldes de départ » (verrou vs grille, D6). */
export async function getOpeningBalancesStatus(): Promise<OpeningBalancesStatus> {
	return apiClient.get<OpeningBalancesStatus>('/api/v1/opening-balances/status');
}

/**
 * Génère l'écriture d'ouverture (une OD équilibrée datée au 1er jour du
 * premier exercice). 409 `ILLEGAL_STATE_TRANSITION` si la company contient
 * déjà des écritures — le serveur localise tous les messages, afficher
 * `err.message` tel quel (AC-E).
 */
export async function generateOpeningBalances(
	req: OpeningBalancesRequest
): Promise<JournalEntryResponse> {
	return apiClient.post<JournalEntryResponse>('/api/v1/opening-balances', req);
}
