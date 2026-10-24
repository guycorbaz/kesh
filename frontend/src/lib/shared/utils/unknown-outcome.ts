/**
 * Issue inconnue d'une opération longue — module partagé.
 *
 * Extrait d'`AdminRestorePanel.svelte` (Story 31-1b, AC18b) par la Story 32-1b
 * (#711 E-7, AC-b2), qui l'emploie aussi pour « Sauvegarder maintenant » : une
 * seule règle pour dire « la réponse n'est pas parvenue » (règle DRY).
 *
 * Importe `isApiError` de `./api-error`, **non** d'`./api-client` : dix-huit
 * fichiers de test simulent `$lib/shared/utils/api-client` par une fabrique qui
 * ne fournit presque jamais `isApiError` (cf. l'en-tête d'`api-error.ts`).
 */

import { isApiError } from './api-error';

/**
 * Vrai si l'échec laisse l'issue de l'opération **inconnue** : la réponse n'est
 * pas arrivée (délai du client dépassé, `TIMEOUT` ; connexion coupée,
 * `NETWORK_ERROR` — tous deux `status: 0`) ou un proxy inverse a abandonné
 * l'attente (502, 504 — Kesh n'émet ni l'un ni l'autre). Le serveur, lui, peut
 * être allé au bout de l'opération.
 *
 * Faux pour toute erreur qui n'est pas une `ApiError` : à l'appelant de dire
 * ce qu'il en fait (la sauvegarde la traite aussi en issue inconnue, l'import
 * non). Le 503 n'en est pas : Kesh l'émet quand la base est injoignable.
 */
export function isUnknownOutcome(err: unknown): boolean {
	if (!isApiError(err)) return false;
	return (
		err.code === 'TIMEOUT' ||
		err.code === 'NETWORK_ERROR' ||
		err.status === 502 ||
		err.status === 504
	);
}
