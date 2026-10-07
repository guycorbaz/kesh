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

/**
 * Raison du mode « compléter » (Story 25-7, AC 3). `READY` seul autorise le
 * complément ; les autres disent pourquoi il est impossible.
 */
export type OpeningComplementReason =
	| 'READY'
	| 'NO_ENTRIES'
	| 'NO_OPEN_FISCAL_YEAR'
	| 'DATE_LOCKED'
	| 'NO_RETAINED_EARNINGS'
	| 'RETAINED_EARNINGS_NOT_POSTABLE'
	| 'NO_COMPLETABLE_ACCOUNT';

/** Compte proposé au complément : bilan, actif, imputable, jamais mouvementé. */
export interface CompletableAccount {
	id: number;
	number: string;
	name: string;
	accountType: 'Asset' | 'Liability';
}

/** Compte de report (rôle `RetainedEarnings`). */
export interface RetainedEarningsAccount {
	id: number;
	number: string;
	name: string;
}

/** Réponse de `GET /api/v1/opening-balances/status` (D6, étendue Story 25-7). */
export interface OpeningBalancesStatus {
	fiscalYear: OpeningBalancesFiscalYear | null;
	/** `true` ssi premier exercice existe + `Open` + company vierge. */
	canEnter: boolean;
	reason: OpeningBalancesReason;
	/** Vrai seulement si `completeReason = READY`. */
	canComplete: boolean;
	completeReason: OpeningComplementReason;
	/** Calculée par le serveur — jamais recalculée côté client. */
	completableAccounts: CompletableAccount[];
	/** Date prévue, **indicative** : le POST la recalcule sous verrou. */
	complementDate: string | null;
	/** `OPENING_DAY` (premier jour de l'ouverture) ou `TODAY` (régularisation). */
	complementDateKind: 'OPENING_DAY' | 'TODAY' | null;
	complementFiscalYear: { id: number; name: string } | null;
	retainedEarningsAccount: RetainedEarningsAccount | null;
}

/** Ligne de complément : une seule colonne non nulle. */
export interface OpeningComplementLineRequest {
	accountId: number;
	debit: string;
	credit: string;
}

/** Body de `POST /api/v1/opening-balances/complete` — sans la contrepartie. */
export interface OpeningComplementRequest {
	lines: OpeningComplementLineRequest[];
}
