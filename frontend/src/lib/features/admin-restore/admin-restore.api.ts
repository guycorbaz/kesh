// Story 17-3d — API frontend pour l'import complet d'installation (.keshbackup).
//
// Consomme `POST /api/v1/admin/full-import` (Story 17-3c : Admin strict +
// anti-PAT, multipart champ `file`). Opération **destructrice** : remplace
// toutes les données de l'installation puis invalide la session
// (`sessionInvalidated: true` ⇒ l'UI propose de se reconnecter, Story 31-1b).

import { apiClient, LONG_OPERATION_TIMEOUT_MS } from '$lib/shared/utils/api-client';

/** Réponse `200` de `POST /api/v1/admin/full-import` (miroir camelCase 17-3c). */
export interface FullImportResponse {
	backupCreated: boolean;
	tablesRestored: number;
	rowsRestored: number;
	sourceVersion: string;
	sessionInvalidated: boolean;
	/**
	 * Story 31-1a (AC9) — pièces écrites dans le dossier des documents. Optionnel :
	 * absent d'un serveur antérieur à la 0.15.0.
	 */
	filesRestored?: number;
	/**
	 * Story 31-1a (AC9) — pièces que la base restaurée référence mais que le
	 * dossier des documents ne porte pas (ou altérées) : elles seront introuvables
	 * (« Le document n'a pas été restauré. » pour un document fournisseur, « Le PDF
	 * émis de cette facture est introuvable… » pour un PDF figé — refus 410). Optionnel, comme
	 * `filesRestored`.
	 */
	filesMissing?: number;
}

/**
 * Story 36-5 (#931) — marge du maintien de session pendant l'import (ms) : le
 * renouvellement et la relecture de `/me` entre les deux envois possibles.
 * Hypothèse, non borne : ni l'un ni l'autre n'a de délai d'abandon.
 */
export const RESTORE_HOLD_MARGIN_MS = 60_000;

/**
 * Story 36-5 (#931) — durée du maintien posé à la confirmation de l'import (ms).
 * L'import tient en au plus deux envois (un 401 renouvelé fait un rejeu, avec un
 * délai neuf de `LONG_OPERATION_TIMEOUT_MS`) et un renouvellement ; jamais moins
 * que la durée de lecture `readingMs` (la durée d'inactivité réglée). Seule
 * écriture de ce maximum.
 */
export function importHoldMs(readingMs: number): number {
	return Math.max(readingMs, 2 * LONG_OPERATION_TIMEOUT_MS + RESTORE_HOLD_MARGIN_MS);
}

/**
 * Téléverse un `.keshbackup` et déclenche le restore complet d'installation.
 *
 * `apiClient.postFormData` jette une `ApiError` sur HTTP non-2xx (409 version,
 * 400 structure/schéma, 413 taille, 500 échec) — l'exception remonte au
 * composant appelant (`AdminRestorePanel`) qui mappe le code en message lisible.
 *
 * Story 31-1b (AC18b) : l'import d'une installation avec ses pièces dure des
 * minutes — l'appel échappe au délai de 30 s du client (une heure) et n'est
 * jamais rejoué.
 */
export async function uploadFullImport(file: File): Promise<FullImportResponse> {
	const form = new FormData();
	form.append('file', file);
	return apiClient.postFormData<FullImportResponse>('/api/v1/admin/full-import', form, {
		timeoutMs: LONG_OPERATION_TIMEOUT_MS,
		retry: false,
	});
}
