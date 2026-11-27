/**
 * Types TS miroir des DTOs API pour les exercices comptables (Story 3.7).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/fiscal_years.rs`
 * (serde `rename_all = "camelCase"`).
 */

export type FiscalYearStatus = 'Open' | 'Closed';

/**
 * Story 37-1 (AC-1, AC-2) — genre du cours d'ouverture ou de clôture d'un
 * exercice : cours du jour ou cours mensuel. Miroir de l'énumération serveur
 * (`day` / `month`, valeurs du HTTP).
 */
export type RateKind = 'day' | 'month';

export interface FiscalYearResponse {
	id: number;
	companyId: number;
	name: string;
	/** Format ISO `YYYY-MM-DD`. */
	startDate: string;
	/** Format ISO `YYYY-MM-DD`. */
	endDate: string;
	status: FiscalYearStatus;
	/**
	 * Story 37-1 (AC-3 bis) — exercice multi-devises. Posé à la création (réglage
	 * de la société) ou à l'activation du multi-devises (exercices ouverts),
	 * jamais changé ensuite. Toujours présent, mono-devise compris (AC-2).
	 */
	multiCurrency: boolean;
	/** Story 37-1 (AC-1) — genre du cours d'ouverture ; toujours présent. */
	openingRateKind: RateKind;
	/** Story 37-1 (AC-1) — genre du cours de clôture ; toujours présent. */
	closingRateKind: RateKind;
	createdAt: string;
	updatedAt: string;
}

export interface CreateFiscalYearRequest {
	name: string;
	/** Format ISO `YYYY-MM-DD`. */
	startDate: string;
	/** Format ISO `YYYY-MM-DD`. */
	endDate: string;
}

export interface UpdateFiscalYearRequest {
	name: string;
}

/** Story 14-2 — réouverture d'un exercice clôturé (motif obligatoire, tracé en audit). */
export interface ReopenFiscalYearRequest {
	motif: string;
}

// ---------------------------------------------------------------------------
// Story 29-1a (#232) — contrôles préalables à la clôture
// (`GET /api/v1/fiscal-years/{id}/closing-checks`, miroir de
// `crates/kesh-db/src/repositories/closing_checks.rs`). Montants en chaînes
// décimales, jamais convertis en `Number`.
// ---------------------------------------------------------------------------

/**
 * Les neuf codes, dans l'ordre que rend le serveur. Story v017-4b (#849, AC-26) : K8
 * `DRAFT_SUPPLIER_INVOICES` (AC-16 de la 4a) ; Story 37-5 (AC-9) : K9
 * `FOREIGN_ACCOUNTS_NOT_REVALUED`, bloquant, rendu en dernier.
 */
export type ClosingCheckCode =
	| 'EARLIER_FISCAL_YEAR_OPEN'
	| 'PENDING_BANK_TRANSACTIONS'
	| 'BANK_STATEMENT_AT_YEAR_END'
	| 'DRAFT_INVOICES'
	| 'IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE'
	| 'INCONSISTENT_ENTRIES'
	| 'NEXT_FISCAL_YEAR'
	| 'DRAFT_SUPPLIER_INVOICES'
	| 'FOREIGN_ACCOUNTS_NOT_REVALUED';

/** K9 (Story 37-5) — un compte en devise non réévalué au cours de clôture. */
export interface NotRevaluedAccount {
	accountId: number;
	number: string;
	name: string;
	currency: string;
	/** Écart en débit net (chaîne décimale) ; `null` sans cours ou hors bornes. */
	delta: string | null;
	reason: 'NOT_REVALUED' | 'RATE_UNAVAILABLE' | 'NOT_POSTABLE';
}

export type ClosingCheckSeverity = 'blocking' | 'warning' | 'info' | 'anomaly';
export type ClosingCheckStatus = 'ok' | 'attention';

export interface ClosingFiscalYearRef {
	id: number;
	name: string;
}

export interface PendingByAccount {
	bankAccountId: number;
	name: string;
	count: number;
	rejectedCount: number;
	/**
	 * Story 38-11d (revue P1, M1) — prestataire dont le compte est le solde (`'paypal'`, `'stripe'`, `'wise'`),
	 * `null` ou absent pour un compte ordinaire : ses transactions se rapprochent par la page
	 * du prestataire, non par la réconciliation générique.
	 */
	provider?: string | null;
}

export type StatementReason = 'MISSING' | 'ENDS_BEFORE_YEAR_END' | 'GAP';
export type NotCheckableReason = 'NOT_COMPARABLE' | 'NO_BALANCE_STATEMENT';

export interface StatementIssue {
	bankAccountId: number;
	name: string;
	reason: StatementReason;
	statementDate: string | null;
	statementClosingBalance: string | null;
	ledgerBalanceAtStatement: string | null;
	gap: string | null;
}

export interface NotCheckable {
	bankAccountId: number;
	name: string;
	reason: NotCheckableReason;
}

export interface DraftInvoiceRef {
	id: number;
	label: string;
	date: string;
}

export type EntryReason =
	| 'UNBALANCED'
	| 'TOO_FEW_LINES'
	| 'DATE_OUTSIDE_ITS_YEAR'
	| 'OTHER_YEAR_DATED_INSIDE';

export interface InconsistentEntry {
	id: number;
	entryNumber: number;
	entryDate: string;
	fiscalYearName: string;
	reasons: EntryReason[];
}

export interface NextFiscalYearRef {
	id: number;
	name: string;
	startDate: string;
	status: FiscalYearStatus;
}

/** Détails, selon le code (la forme est fixée par le code). */
export interface ClosingCheckDetails {
	/** K1 (antérieur ouvert) et K7 (exercice suivant). */
	fiscalYear?: ClosingFiscalYearRef | NextFiscalYearRef | null;
	/** K2 (par compte), K3 (comptes en défaut) et K9 (comptes non réévalués). */
	accounts?: PendingByAccount[] | StatementIssue[] | NotRevaluedAccount[];
	/** K3 — comptes examinés mais non vérifiables (sans effet sur le statut). */
	notCheckable?: NotCheckable[];
	/** K4, et K8 (même forme : brouillons fournisseur). */
	invoices?: DraftInvoiceRef[];
	/** K6. */
	countByReason?: Record<EntryReason, number>;
	entries?: InconsistentEntry[];
}

export interface ClosingCheck {
	code: ClosingCheckCode;
	severity: ClosingCheckSeverity;
	status: ClosingCheckStatus;
	/** Nombre de points relevés (K1 et K7 : 1 en attention, 0 sinon). */
	count: number;
	details: ClosingCheckDetails;
}

export interface ClosingChecksResponse {
	fiscalYearId: number;
	fiscalYearName: string;
	startDate: string;
	endDate: string;
	checks: ClosingCheck[];
}

// ---------------------------------------------------------------------------
// Story 37-5 — réévaluation de fin d'exercice des comptes en devise
// (`POST /api/v1/fiscal-years/{id}/fx-revaluation[/preview]`, miroir de
// `crates/kesh-api/src/routes/fx_revaluation.rs`). Montants et cours en chaînes
// décimales, jamais convertis en `Number`.
// ---------------------------------------------------------------------------

/** Genre d'un cours retenu : défaut journalier, défaut mensuel, ou reçu différent. */
export type RevaluationRateKind = 'day' | 'month' | 'request';

/** Corps des deux routes : seuls les cours modifiés à l'aperçu, tous les cours affichés à la passation. */
export interface RevaluationRequest {
	rates?: Record<string, string>;
	accountIds?: number[];
}

/** Le cours d'une devise dans l'aperçu (AC-1, U1). */
export interface RevaluationRate {
	currency: string;
	rate: string | null;
	rateKind: RevaluationRateKind | null;
	/** Date du cours par défaut (jour, ou 1er du mois). */
	rateDate: string | null;
	/** Genre du cours par défaut (`day` ou `month`) ; `null` sans défaut. */
	defaultKind: RateKind | null;
	/** Cours retenu par la société de la résolution par défaut. */
	defaultRate: string | null;
	/** Sa source, valeur brute : `afc`, `manual` ou autre (A30). */
	defaultSource: string | null;
	overridden: boolean;
	/** Date de publication d'un cours journalier `afc`. */
	sourceDate: string | null;
}

/** Une ligne de l'aperçu : un compte et l'écriture prévue (R9, A39). */
export interface RevaluationLine {
	accountId: number;
	accountNumber: string;
	accountName: string;
	accountType: 'Asset' | 'Liability';
	currency: string;
	archived: boolean;
	selectable: boolean;
	selected: boolean;
	reason: 'ACCOUNT_ARCHIVED' | 'ACCOUNT_NOT_POSTABLE' | 'RATE_UNAVAILABLE' | null;
	/** Débit net en devise (sens brut). */
	deviseNet: string;
	/** Débit net en CHF (« CHF actuel »). */
	chfNet: string;
	rate: string | null;
	/** CHF réévalué (« CHF cible »). */
	targetChf: string | null;
	/** Écart en débit net. */
	delta: string | null;
	counterpartAccountId: number | null;
	counterpartAccountNumber: string | null;
	counterpartAccountName: string | null;
	fxAccountsOrigin: 'own' | 'company' | null;
	label: string | null;
	previousRevaluationTotal: string;
	previousEntryIds: number[];
	revaluationTotalAfter: string | null;
}

/** Ce que la passation refuserait (code, message du serveur, détails). */
export interface RevaluationBlocker {
	code: string;
	message: string;
	details: Record<string, unknown>;
}

export interface RevaluationPreview {
	fiscalYearId: number;
	startDate: string;
	endDate: string;
	closingRateKind: RateKind;
	rates: RevaluationRate[];
	lines: RevaluationLine[];
	totals: { gains: string; losses: string };
	preselectionTruncated: boolean;
	blockers: RevaluationBlocker[];
}

export interface RevaluationPostResponse {
	entryIds: number[];
}
