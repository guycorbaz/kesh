// Story 17-3d — API frontend pour l'import complet d'installation (.keshbackup).
//
// Consomme `POST /api/v1/admin/full-import` (Story 17-3c : Admin strict +
// anti-PAT, multipart champ `file`). Opération **destructrice** : remplace
// toutes les données de l'installation puis invalide la session
// (`sessionInvalidated: true` ⇒ l'UI redirige vers /login).

import { apiClient } from '$lib/shared/utils/api-client';

/** Réponse `200` de `POST /api/v1/admin/full-import` (miroir camelCase 17-3c). */
export interface FullImportResponse {
	backupCreated: boolean;
	tablesRestored: number;
	rowsRestored: number;
	sourceVersion: string;
	sessionInvalidated: boolean;
}

/**
 * Téléverse un `.keshbackup` et déclenche le restore complet d'installation.
 *
 * `apiClient.postFormData` jette une `ApiError` sur HTTP non-2xx (409 version,
 * 400 structure/schéma, 413 taille, 500 échec) — l'exception remonte au
 * composant appelant (`AdminRestorePanel`) qui mappe le code en message lisible.
 */
export async function uploadFullImport(file: File): Promise<FullImportResponse> {
	const form = new FormData();
	form.append('file', file);
	return apiClient.postFormData<FullImportResponse>('/api/v1/admin/full-import', form);
}
