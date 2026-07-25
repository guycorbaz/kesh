/**
 * Types du bilan d'ouverture — saisie des soldes de départ (Story 14-4).
 *
 * Montants en **string décimale** (jamais `number` — CO art. 957-964).
 * `journal` et `entryDate` sont absents du request : forcés serveur
 * (`OD` + `startDate` du premier exercice, D5).
 */

/** Ligne de soldes de départ envoyée au POST. */
export interface OpeningBalanceLineRequest {
	accountId: number;
	debit: string;
	credit: string;
}

/** Body de `POST /api/v1/opening-balances`. */
export interface OpeningBalancesRequest {
	lines: OpeningBalanceLineRequest[];
}

/** Raison pilotant l'état verrou/grille de l'écran (D6). */
export type OpeningBalancesReason =
	| 'READY'
	| 'NO_FISCAL_YEAR'
	| 'FIRST_YEAR_CLOSED'
	| 'ALREADY_HAS_ENTRIES';

/** Résumé du premier exercice retourné par `GET /status`. */
export interface OpeningBalancesFiscalYear {
	id: number;
	name: string;
	startDate: string;
	status: 'Open' | 'Closed';
}

/** Réponse de `GET /api/v1/opening-balances/status` (D6). */
export interface OpeningBalancesStatus {
	fiscalYear: OpeningBalancesFiscalYear | null;
	/** `true` ssi premier exercice existe + `Open` + company vierge. */
	canEnter: boolean;
	reason: OpeningBalancesReason;
}
