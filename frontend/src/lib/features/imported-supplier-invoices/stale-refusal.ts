/**
 * Refus qui disent que l'écran est **périmé** — Story 30-2c, revue P1 (B2 = E5).
 *
 * Joindre ou détacher le document d'une facture fournisseur peut être refusé parce
 * qu'un autre onglet, un autre utilisateur ou un import a changé le document ou la
 * facture entre-temps : `404` (document ou facture disparu, document déjà détaché)
 * ou `409` (document déjà traité ou retiré, facture qui a déjà un document). L'écran
 * qui a reçu un tel refus relit alors ce qu'il montre — la liste du sélecteur, la
 * facture —, faute de quoi il proposerait encore le geste qui vient d'échouer.
 *
 * Un autre refus (réseau, `5xx`, `403`) ne dit rien de l'état des données : rien
 * n'est relu.
 */

import type { ApiError } from '$lib/shared/types/api';
import { isApiError } from '$lib/shared/utils/api-error';

/** Vrai si `err` est un refus `404` ou `409` du serveur. */
export function isStaleDocumentRefusal(err: unknown): err is ApiError {
	return isApiError(err) && (err.status === 404 || err.status === 409);
}
