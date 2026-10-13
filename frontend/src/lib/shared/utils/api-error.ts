/**
 * Garde-type des erreurs d'API — module sans dépendance.
 *
 * Extrait d'`api-client.ts` (Story 28-2, C-28-2-2), qui le **réexporte** : ses
 * importeurs existants ne changent pas. Il vit à part pour que
 * `error-message.ts` puisse l'importer sans tirer le client HTTP — dix-huit
 * fichiers de test simulent `$lib/shared/utils/api-client` par une fabrique
 * qui ne fournit presque jamais `isApiError`.
 */

import type { ApiError } from '$lib/shared/types/api';

/**
 * Vrai si `err` est une `ApiError` : un **objet simple** `{ code, status, … }`,
 * tel que le lève le client — et non une instance d'`Error`.
 *
 * ⚠️ Ne vérifie que `code` (chaîne) et `status` (nombre) : le `message` n'est
 * pas contrôlé (cf. `errorMessageOf`, qui le contrôle, C-28-2-24).
 */
export function isApiError(err: unknown): err is ApiError {
	return (
		typeof err === 'object' &&
		err !== null &&
		'code' in err &&
		'status' in err &&
		typeof (err as ApiError).code === 'string' &&
		typeof (err as ApiError).status === 'number'
	);
}
