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
	/** Story 37-4 — devise du compte (ISO 4217), **absente** pour un compte en devise
	 *  principale. Les totaux du rapport restent en devise principale. */
	currency?: string;
	/** Story 37-4 — solde en devise, signé comme `balance` ; absent hors compte en devise. */
	balanceCurrency?: string;
}

// --- Story 46-2 — bilan et compte de résultat par groupes (contrat de la 46-1) ---

/** Niveau de détail : `1` à `20`, ou `all` (tous les groupes ouverts). */
export type DetailLevel = number | 'all';

/** Place du sous-total d'un groupe ouvert : sur la ligne du titre, ou en pied. */
export type SubtotalPosition = 'header' | 'footer';

/**
 * Paramètres de groupement d'une demande de bilan, de compte de résultat ou de balance
 * (46-1 § *Le contrat*, 46-6a) ; les mêmes vont à l'export (46-3, 46-6b).
 */
export interface GroupingParams {
	detailLevel: DetailLevel;
	subtotalPosition: SubtotalPosition;
	/** Groupes ouverts à la main (identifiants de comptes de groupe). */
	open: number[];
	/** Groupes fermés à la main. */
	closed: number[];
}

/**
 * Écho `grouping` de la réponse : les choix **retenus** (identifiants ignorés
 * retirés, listes triées) et `maxDepth`, la profondeur de l'arbre affiché,
 * indépendante de l'ouverture (`0` rapport vide, `1` plan sans groupe).
 */
export interface GroupingEcho extends GroupingParams {
	maxDepth: number;
}

/** Genre d'une ligne groupée. */
export type GroupedRowKind = 'group' | 'account' | 'own' | 'subtotal';

/**
 * Ce que toute ligne groupée porte, quel que soit le rapport (Story 46-6b) :
 * genre, profondeur, compte, devise unique éventuelle et solde dans cette
 * devise, état d'un groupe (`open`, sur `group` seulement) et compte actif
 * (`active`, sur `account` et `own`). Les autres montants sont propres à chaque
 * rapport.
 */
export interface GroupedRowBase {
	kind: GroupedRowKind;
	depth: number;
	accountId: number;
	accountNumber: string;
	accountName: string;
	currency?: string;
	/**
	 * Le solde en devise, présent avec `currency` : au bilan et au compte de
	 * résultat, le solde de la ligne ; à la balance, son net des mouvements.
	 */
	balanceCurrency?: string;
	open?: boolean;
	active?: boolean;
}

/**
 * Une ligne de `groupedRows` du bilan ou du compte de résultat, déjà dans
 * l'ordre d'affichage. `balance` est le sous-total d'un groupe.
 */
export interface GroupedRow extends GroupedRowBase {
	balance: string;
	role?: AccountRole | null;
}

/**
 * Une ligne de `groupedRows` de la balance (Story 46-6a ; un tableau, la
 * balance n'a pas de sections) : les montants de {@link TrialBalanceRow}, sous-
 * totaux exacts d'un groupe. Les montants en devise ne sont présents qu'avec
 * `currency` (toute la descendance non nulle dans cette devise).
 */
export interface TrialBalanceGroupedRow extends GroupedRowBase {
	openingBalance: string;
	totalDebit: string;
	totalCredit: string;
	/** Net des mouvements : rendu par l'API, montré nulle part (25-5-b). */
	balance: string;
	closingBalance: string;
	openingCurrency?: string;
	totalDebitCurrency?: string;
	totalCreditCurrency?: string;
	balanceCurrency?: string;
	closingCurrency?: string;
}

export interface BalanceSheetGroupedRows {
	assets: GroupedRow[];
	liabilities: GroupedRow[];
	equity: GroupedRow[];
}

export interface IncomeStatementGroupedRows {
	revenues: GroupedRow[];
	expenses: GroupedRow[];
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
	/** Story 46-2 — présents seulement si la demande portait `detailLevel`. */
	grouping?: GroupingEcho;
	groupedRows?: BalanceSheetGroupedRows;
}

export interface IncomeStatementDto {
	period: ReportPeriod;
	revenues: AccountBalance[];
	expenses: AccountBalance[];
	totalRevenues: string;
	totalExpenses: string;
	netResult: string;
	/** Story 46-2 — présents seulement si la demande portait `detailLevel`. */
	grouping?: GroupingEcho;
	groupedRows?: IncomeStatementGroupedRows;
}

export interface TrialBalanceRow {
	accountId: number;
	accountNumber: string;
	accountName: string;
	accountType: AccountType;
	active: boolean;
	/** Story 25-5-b — solde au début de la période, signé selon la nature du compte. */
	openingBalance: string;
	totalDebit: string;
	totalCredit: string;
	/** Net des mouvements de la période — pas un solde de compte. */
	balance: string;
	/** Story 25-5-b — ouverture + mouvements : pour un compte de bilan, son solde au bilan. */
	closingBalance: string;
	/** Story 37-4 — devise du compte ; absente (avec les champs suivants) pour un compte
	 *  en devise principale. Aucun total en devise (devises mêlées). */
	currency?: string;
	openingCurrency?: string;
	totalDebitCurrency?: string;
	totalCreditCurrency?: string;
	balanceCurrency?: string;
	closingCurrency?: string;
}

export interface TrialBalanceDto {
	period: ReportPeriod;
	rows: TrialBalanceRow[];
	totalDebit: string;
	totalCredit: string;
	balanced: boolean;
	/** Story 25-5-b — résultat reporté calculé (exercices antérieurs), crédit-positif. */
	retainedEarnings: string;
	/** Story 25-5-b — Σ des ouvertures en sens débit − résultat reporté = 0. */
	openingBalanced: boolean;
	/** Story 46-6b — présents seulement si la demande portait `detailLevel`. */
	grouping?: GroupingEcho;
	groupedRows?: TrialBalanceGroupedRow[];
}

export type Journal = 'Achats' | 'Ventes' | 'Banque' | 'Caisse' | 'OD';

export interface JournalEntryLineRow {
	accountId: number;
	accountNumber: string;
	accountName: string;
	debit: string;
	credit: string;
	lineOrder: number;
	/** Story 37-4 — devise du compte, seulement si la ligne porte un montant en devise. */
	currency?: string;
	/** Montant en devise, sans signe : son côté est celui du montant en devise principale.
	 *  0 pour une réévaluation. */
	amountCurrency?: string;
	/** Cours par unité de devise. */
	exchangeRate?: string;
	/** Provenance du cours. */
	rateSource?: RateSource;
}

/** Story 37-4 — provenance du cours d'une ligne en devise. */
export type RateSource = 'afc' | 'manual' | 'revaluation';

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

/** Story 25-4-d2c — une diminution de contre-prestation (TVA des soldes), par taux. */
export interface VatWriteOffRow {
	rate: string; // Decimal string, %
	baseHt: string; // Decimal string
	vat: string; // Decimal string
}

export interface VatReportDto {
	period: ReportPeriod;
	rows: VatReportRow[];
	totalBaseHt: string;
	totalVatDue: string;
	/** Story 25-4-d2c — la TVA des soldes (escompte, perte) datés dans la période. */
	writeOffRows: VatWriteOffRow[];
	totalVatWriteOff: string;
	/** TVA due nette = TVA facturée − TVA des soldes ; le solde (`vatBalance`) en part. */
	totalVatDueNet: string;
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
	/**
	 * Date de la première ligne du nouvel exercice ; pour une rupture qu'aucune
	 * ligne ne suit (fin d'exercice franchie après la dernière), premier jour de
	 * l'exercice en cours à la fin de la période.
	 */
	date: string;
	closingFiscalYearId: number;
	closingBalance: string;
	/** Story 37-4 — solde en devise de l'exercice clos (compte en devise seulement). */
	closingBalanceCurrency?: string;
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
	/**
	 * Story 15-1b (AC6) — le code du groupe de lettrage de la ligne, `null` si
	 * elle est ouverte. Montré à l'écran (15-1c-ii) ; **absent** des exports CSV
	 * et PDF (C-15-1b-6).
	 */
	letteringCode: string | null;
	/** Story 37-4 — montant en devise au débit ; présent sur toute ligne d'une section en
	 *  devise (0 du côté opposé), absent ailleurs. */
	debitCurrency?: string;
	/** Montant en devise au crédit (même règle). */
	creditCurrency?: string;
	/** Cours par unité de devise. */
	exchangeRate?: string;
	/** Provenance du cours ; `revaluation` se lit « Réévaluation au cours X ». */
	rateSource?: RateSource;
	/** Solde progressif en devise. */
	runningBalanceCurrency?: string;
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
	/**
	 * Solde de clôture. Compte de résultat dont la période franchit une fin
	 * d'exercice : le solde de l'exercice en cours à la fin de la période — le
	 * dernier solde progressif, ruptures comprises (#778) —, et non ouverture
	 * plus mouvements.
	 */
	closing: string;
	/** Solde du côté opposé à la nature du compte : l'anomalie à voir. */
	unnaturalBalance: boolean;
	fiscalYearBreaks: FiscalYearBreak[];
	/** Nombre de lignes sur la période, **avant pagination**. */
	lineCount: number;
	/** Story 37-4 — devise du compte ; absente (avec les champs suivants) pour un compte
	 *  en devise principale. */
	currency?: string;
	openingCurrency?: string;
	totalDebitCurrency?: string;
	totalCreditCurrency?: string;
	closingCurrency?: string;
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

/**
 * Story 55-2 (AC-6) — un compte désigné, tel que la page le passe à une vue : son identifiant (le lien) et
 * son numéro (le texte).
 */
export interface AccountRef {
	id: number;
	number: string;
}
