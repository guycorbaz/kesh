// Story 9-1 — DTOs miroir des structures Rust kesh-report sérialisées en JSON camelCase.
// Tous les montants en `string` (rust_decimal serde-str), dates en `string` ISO 8601.

import type { AccountRole } from '$lib/features/accounts/accounts.types';

export interface ReportPeriod {
	fiscalYearId: number;
	startDate: string; // ISO 8601 YYYY-MM-DD
	endDate: string;
}

export type AccountType = 'Asset' | 'Liability' | 'Revenue' | 'Expense';

export interface AccountBalance {
	accountId: number;
	accountNumber: string;
	accountName: string;
	accountType: AccountType;
	active: boolean;
	balance: string; // Decimal string
	/** Rôle métier explicite du compte (Story 14-3a), `null` si aucun. Porte le
	 *  regroupement par rôle de la section Capitaux propres au bilan (Story 14-3c). */
	role: AccountRole | null;
}

export interface BalanceSheetDto {
	period: ReportPeriod;
	assets: AccountBalance[];
	/** Dettes réelles seulement — les comptes de fonds propres sont dans `equity` (Story 14-3c). */
	liabilities: AccountBalance[];
	/** Comptes physiques de fonds propres, triés par rang de rôle en backend (Story 14-3c). */
	equity: AccountBalance[];
	totalAssets: string;
	totalLiabilities: string;
	/** Σ des comptes physiques de `equity` (hors lignes calculées virtuelles), Story 14-3c. */
	totalEquity: string;
	/** Résultat reporté = cumul P&L des exercices antérieurs (Story 14-1). Négatif = perte reportée.
	 *  Ligne **calculée** — disjointe d'un compte physique de rôle RetainedEarnings (collision D1). */
	retainedEarnings: string;
	equityResult: string;
	equationHolds: boolean;
}

export interface IncomeStatementDto {
	period: ReportPeriod;
	revenues: AccountBalance[];
	expenses: AccountBalance[];
	totalRevenues: string;
	totalExpenses: string;
	netResult: string;
}

export interface TrialBalanceRow {
	accountId: number;
	accountNumber: string;
	accountName: string;
	accountType: AccountType;
	active: boolean;
	totalDebit: string;
	totalCredit: string;
	balance: string;
}

export interface TrialBalanceDto {
	period: ReportPeriod;
	rows: TrialBalanceRow[];
	totalDebit: string;
	totalCredit: string;
	balanced: boolean;
}

export type Journal = 'Achats' | 'Ventes' | 'Banque' | 'Caisse' | 'OD';

export interface JournalEntryLineRow {
	accountId: number;
	accountNumber: string;
	accountName: string;
	debit: string;
	credit: string;
	lineOrder: number;
}

export interface JournalEntryRow {
	entryId: number;
	entryNumber: number;
	entryDate: string;
	description: string;
	lines: JournalEntryLineRow[];
}

export interface JournalSection {
	journal: Journal;
	entries: JournalEntryRow[];
	sectionTotalDebit: string;
	sectionTotalCredit: string;
}

export interface JournalReportDto {
	period: ReportPeriod;
	journals: JournalSection[];
	grandTotalDebit: string;
	grandTotalCredit: string;
}

// Story 11-2 — Rapport TVA (TVA due / vente).
export interface VatReportRow {
	rate: string; // Decimal string, % (ex. "8.10")
	category: string | null; // null en v0.2 (grouping par taux)
	baseHt: string; // Decimal string
	vatDue: string; // Decimal string
}

export interface VatReportDto {
	period: ReportPeriod;
	rows: VatReportRow[];
	totalBaseHt: string;
	totalVatDue: string;
	totalVatRecoverable: string; // solde compte impôt préalable (Story 18-1d)
	vatBalance: string;
	/** Écart de réconciliation TVA due (dérivée − solde grand livre périmètre ventes), Story 18-1e. */
	reconciliationDelta: string;
	/** "delta" si |écart| >= 0.01 (écriture validée modifiée à la main), sinon "ok". */
	reconciliationStatus: 'ok' | 'delta';
}

export type ReportType = 'balance-sheet' | 'income-statement' | 'trial-balance' | 'journals' | 'vat';

// --- Story 21-7 : Balance âgée débiteurs (montants string, rust_decimal serde-str) ---

/** Montants d'une tranche d'ancienneté (ligne de contact ou total général). */
export interface AgedBucketDto {
	notDue: string;
	days1To30: string;
	days31To60: string;
	days61To90: string;
	daysOver90: string;
	total: string;
}

/** Une ligne de la balance âgée : les créances ouvertes d'un contact, ventilées. */
export interface AgedReceivablesRowDto extends AgedBucketDto {
	contactId: number;
	contactName: string;
}

/** Balance âgée complète, arrêtée à `asOf` (YYYY-MM-DD). */
export interface AgedReceivablesDto {
	asOf: string;
	rows: AgedReceivablesRowDto[];
	totals: AgedBucketDto;
}

/** Query params communs (camelCase API). */
export interface ReportQuery {
	fiscalYearId: number;
	periodStart?: string; // YYYY-MM-DD
	periodEnd?: string;
}

export interface JournalReportQuery extends ReportQuery {
	journal?: Journal;
}

// ============================================================================
// Story 19-6a — Rapport « Dépenses par projet »
// ============================================================================

/** Identité minimale d'un projet (miroir kesh-report ProjectInfo). */
export interface ProjectInfoDto {
	id: number;
	code: string;
	name: string;
}

/** Écriture contributrice (drill-down). */
export interface ProjectEntryRefDto {
	entryId: number;
	entryNumber: number;
	entryDate: string; // YYYY-MM-DD
	description: string;
	amount: string; // Decimal string
}

/** Ligne compte d'une section (montant + écritures drill-down). */
export interface ExpenseAccountRowDto {
	accountId: number;
	accountNumber: string;
	accountName: string;
	amount: string;
	entries: ProjectEntryRefDto[];
}

/** Section = un projet du scope (racine ou sous-projet). */
export interface ProjectExpenseSectionDto {
	project: ProjectInfoDto;
	isRoot: boolean;
	rows: ExpenseAccountRowDto[];
	subtotal: string;
}

/** Rapport « Dépenses par projet » (miroir camelCase). */
export interface ProjectExpensesDto {
	reportType: string;
	project: ProjectInfoDto;
	mode: string;
	periodLabel: string;
	sections: ProjectExpenseSectionDto[];
	grandTotal: string;
}

/** Mode de période d'un rapport projet. */
export type ProjectReportMode = 'fiscal_year' | 'cumulative';

/** Query params des rapports par projet (JSON + export). */
export interface ProjectReportQuery {
	projectId: number;
	mode: ProjectReportMode;
	/** Requis si mode === 'fiscal_year'. */
	fiscalYearId?: number;
	periodStart?: string;
	periodEnd?: string;
}

/** Type de rapport par projet (path URL). */
export type ProjectReportType = 'project-expenses' | 'project-return';

// ---- Story 19-6b : Rendement par projet ----

/** Section « rendement » d'un projet du scope. */
export interface ProjectReturnSectionDto {
	project: ProjectInfoDto;
	isRoot: boolean;
	coutInvesti: string;
	revenus: string;
	resultatNet: string;
	/** `null` si coût investi = 0. */
	rendementPct: string | null;
}

export interface ProjectReturnTotalsDto {
	coutInvesti: string;
	revenus: string;
	resultatNet: string;
	rendementPct: string | null;
}

export interface ProjectReturnDto {
	reportType: string;
	project: ProjectInfoDto;
	mode: string;
	periodLabel: string;
	sections: ProjectReturnSectionDto[];
	totals: ProjectReturnTotalsDto;
}

// --- Grand livre (Story 24-1) ---

/**
 * Fenêtre du grand livre.
 *
 * ⚠️ **Sans exercice, et c'est délibéré** : le grand livre franchit la borne
 * d'exercice, sans quoi il ne concorderait pas avec le bilan, qui est cumulatif
 * depuis l'origine. C'est le seul rapport dans ce cas.
 */
export interface LedgerPeriod {
	from: string;
	to: string;
}

/** Fin d'exercice traversée : sur un compte de résultat, le solde y repart de zéro. */
export interface FiscalYearBreak {
	date: string;
	closingFiscalYearId: number;
	closingBalance: string;
}

export interface LedgerLine {
	lineId: number;
	entryId: number;
	entryDate: string;
	fiscalYearId: number;
	/** Nom de l'exercice — ce qui désambiguïse le numéro de pièce. */
	fiscalYearName: string;
	/** ⚠️ Unique **par exercice** seulement : il repart à 1. */
	entryNumber: number;
	journal: Journal;
	/** Libellé de l'écriture — les lignes n'en portent pas. */
	description: string;
	counterpart: string[];
	debit: string;
	credit: string;
	runningBalance: string;
}

export interface LedgerSection {
	accountId: number;
	accountNumber: string;
	accountName: string;
	accountType: AccountType;
	/** `false` pour un compte archivé — il reste rendu s'il porte un solde. */
	active: boolean;
	balanceSide: 'debit' | 'credit';
	opening: string;
	lines: LedgerLine[];
	totalDebit: string;
	totalCredit: string;
	closing: string;
	/** Solde du côté opposé à la nature du compte : l'anomalie à voir. */
	unnaturalBalance: boolean;
	fiscalYearBreaks: FiscalYearBreak[];
	/** Nombre de lignes sur la période, **avant pagination**. */
	lineCount: number;
}

export interface GeneralLedgerDto {
	period: LedgerPeriod;
	sections: LedgerSection[];
}

/** Paramètres du grand livre — ni `fiscalYearId`, ni `periodStart`/`periodEnd`. */
export interface LedgerQuery {
	from: string;
	to: string;
	accountIds?: number[];
	includeZero?: boolean;
	offset?: number;
	limit?: number;
}
