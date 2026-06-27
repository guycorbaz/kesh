// Story 9-1 — DTOs miroir des structures Rust kesh-report sérialisées en JSON camelCase.
// Tous les montants en `string` (rust_decimal serde-str), dates en `string` ISO 8601.

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
}

export interface BalanceSheetDto {
	period: ReportPeriod;
	assets: AccountBalance[];
	liabilities: AccountBalance[];
	totalAssets: string;
	totalLiabilities: string;
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

/** Query params communs (camelCase API). */
export interface ReportQuery {
	fiscalYearId: number;
	periodStart?: string; // YYYY-MM-DD
	periodEnd?: string;
}

export interface JournalReportQuery extends ReportQuery {
	journal?: Journal;
}
