/**
 * Types TS miroir des DTOs API pour les exercices comptables (Story 3.7).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/fiscal_years.rs`
 * (serde `rename_all = "camelCase"`).
 */

export type FiscalYearStatus = 'Open' | 'Closed';

export interface FiscalYearResponse {
	id: number;
	companyId: number;
	name: string;
	/** Format ISO `YYYY-MM-DD`. */
	startDate: string;
	/** Format ISO `YYYY-MM-DD`. */
	endDate: string;
	status: FiscalYearStatus;
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

/** Les sept codes, dans l'ordre que rend le serveur. */
export type ClosingCheckCode =
	| 'EARLIER_FISCAL_YEAR_OPEN'
	| 'PENDING_BANK_TRANSACTIONS'
	| 'BANK_STATEMENT_AT_YEAR_END'
	| 'DRAFT_INVOICES'
	| 'IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE'
	| 'INCONSISTENT_ENTRIES'
	| 'NEXT_FISCAL_YEAR';

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
	/** K2 (par compte) et K3 (comptes en défaut). */
	accounts?: PendingByAccount[] | StatementIssue[];
	/** K3 — comptes examinés mais non vérifiables (sans effet sur le statut). */
	notCheckable?: NotCheckable[];
	/** K4. */
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
