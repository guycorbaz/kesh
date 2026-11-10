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
 * non). Le 503 n'en est pas là où un rejeu est sans danger : la décision sur un
 * envoi MCP porte une version (un rejeu rend 409), une sauvegarde en double ne fait
 * aucun tort. Pour une écriture qu'un rejeu doublerait, voir
 * {@link isSettlementOutcomeUnknown}.
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

/**
 * Vrai si l'échec d'un **règlement de facture** ne prouve pas qu'il n'a pas été écrit
 * (revue P2 de la 33-6, P2-2) : tout ce que dit {@link isUnknownOutcome}, tout 5xx, et
 * toute erreur qui n'est pas une `ApiError`.
 *
 * Une panne pendant le `COMMIT` remonte en 503 alors que la base a pu le valider. Depuis
 * la Story 34-5 (#951), le POST porte la version de la facture et le serveur refuse en
 * 409 un second envoi : un second clic n'écrit plus de second règlement. La relecture
 * reste due, pour tenir une version fraîche (sans elle, le clic suivant rendrait 409) et
 * montrer ce qui a été écrit (C-e343-5). Seul un 4xx, refus certain, laisse le dialogue
 * ouvert — sauf le 409 de version, que l'écran traite à part.
 *
 * Une exception levée après un succès (P2-7) se lit donc aussi « issue inconnue » :
 * c'est sans danger, la relecture montre l'écriture.
 */
export function isSettlementOutcomeUnknown(err: unknown): boolean {
	if (!isApiError(err)) return true;
	return isUnknownOutcome(err) || err.status >= 500;
}
