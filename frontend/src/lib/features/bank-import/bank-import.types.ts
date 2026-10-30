// Story 8-1b — Types DTOs côté frontend (miroir des structs Rust kesh-api/routes/bank_imports.rs).

export interface PreviewStatement {
	statementId: string | null;
	accountIban: string;
	currency: string;
	periodFrom: string;
	periodTo: string;
	openingBalance: string | null;
	closingBalance: string | null;
}

export interface IgnoredStatement {
	statementId: string | null;
	accountIban: string;
}

export interface PreviewTransaction {
	bookingDate: string;
	valueDate: string | null;
	amount: string;
	currency: string;
	reference: string | null;
	details: string;
	counterpartyIban: string | null;
	counterpartyName: string | null;
}

export interface BankImportPreviewResponse {
	fileHash: string;
	filename: string;
	sourceFormat: string;
	selectedStatement: PreviewStatement;
	ignoredStatements: IgnoredStatement[];
	/// Story 8-3 — warnings non-bloquants structurés.
	warnings: PreviewWarnings;
	transactions: PreviewTransaction[];
	/// Story 8-3 KF #70 — métadonnées CSV profile (absent pour CAMT).
	csvProfileMatch?: CsvProfileMatch;
	/// Story 27-3 (#626) — empreinte des entrées de l'aperçu (64 hex), à renvoyer
	/// à la création en `previewFingerprint`.
	previewFingerprint: string;
}

export interface PreviewWarnings {
	balanceMismatch?: BalanceMismatchPayload | null;
	unsupportedCurrency?: { currency: string } | null;
	encodingMismatch?: { profile: string; detected: string } | null;
	duplicateFile?: DuplicateFilePayload | null;
	duplicateLines: DuplicateLineWarning[];
	invalidLines?: InvalidLinesPayload | null;
	/**
	 * ⚠️ **Optionnel, parce que le backend l'OMET quand la liste est vide** :
	 * `#[serde(skip_serializing_if = "Vec::is_empty")]` sur
	 * `bank_imports.rs`. C'est le cas normal de tout import CAMT.053 — le
	 * champ ne sert qu'aux profils CSV. Le déclarer obligatoire a fait
	 * planter le rendu du preview sur `undefined.length`, rendant l'écran
	 * d'import inutilisable dès qu'on y déposait un fichier.
	 */
	informational?: string[];
}

export interface BalanceMismatchPayload {
	opening: string;
	closing: string;
	sum: string;
	diff: string;
}

export interface DuplicateFilePayload {
	existingImportId: number;
	existingFilename: string;
	existingImportedAt: string;
}

export interface DuplicateLineWarning {
	newIndex: number;
	existingTransactionId: number;
	key: string;
}

export interface InvalidLinesPayload {
	lines: { line: number; code: string; value: string | null; messageI18nKey: string }[];
	totalErrors: number;
	truncated: boolean;
}

export interface CsvProfileMatch {
	profileId: number;
	profileName: string;
	autoMatched: boolean;
}

export interface BankImportResponse {
	id: number;
	bankAccountId: number;
	filename: string;
	fileHash: string;
	sourceFormat: string;
	statementId: string | null;
	periodFrom: string;
	periodTo: string;
	openingBalance: string | null;
	closingBalance: string | null;
	transactionCount: number;
	importedAt: string;
}

export interface BankTransactionDto {
	id: number;
	bookingDate: string;
	valueDate: string | null;
	amount: string;
	currency: string;
	reference: string | null;
	details: string;
	endToEndId: string | null;
	transactionId: string | null;
	counterpartyIban: string | null;
	counterpartyName: string | null;
	status: string;
	/** L'écriture liée par le rapprochement, `null` si la transaction n'est pas rapprochée (Story 25-3-b). */
	matchedEntryId: number | null;
	/**
	 * Date du rejet de la proposition de réconciliation (ISO 8601 UTC), `null`
	 * si la transaction n'est pas rejetée (Story 28-5, #526). Une rejetée quitte
	 * la liste de la réconciliation ; son rejet s'annule depuis le détail de son
	 * import (cf. `isRejected`, `reconciliation-unreject.ts`).
	 */
	autoMatchRejectedAt: string | null;
}

export interface BankImportDetailResponse extends BankImportResponse {
	transactions: BankTransactionDto[];
}

export interface BankImportListResponse {
	items: BankImportResponse[];
	total: number;
	offset: number;
	limit: number;
}

// Story v016-2b (#816) — relevé saisi à la main (`POST /api/v1/bank-imports/manual`
// et `/manual/preview`, contrat de la fiche parente `v016-2-releve-manuel.md`).

/** Une ligne du corps : montant positif et sens ; textes facultatifs à `null`. */
export interface ManualStatementLineBody {
	bookingDate: string;
	valueDate: string | null;
	details: string;
	counterpartyName: string | null;
	reference: string | null;
	amount: string | null;
	direction: 'debit' | 'credit' | null;
}

/** Corps de l'aperçu et de la création — montants et soldes en chaînes décimales. */
export interface ManualStatementBody {
	bankAccountId: number;
	periodFrom: string;
	periodTo: string;
	statementNumber: string | null;
	openingBalance: string;
	closingBalance: string;
	lines: ManualStatementLineBody[];
	confirmDuplicateFile: boolean;
	confirmDuplicateLines: boolean;
	confirmOpeningBalanceGap: boolean;
}

/** Doublon signalé par l'aperçu : `line` est le rang 1-based dans le tableau envoyé. */
export interface ManualDuplicateLineWarning {
	line: number;
	existingTransactionId: number;
	key: string;
}

export interface ManualOpeningBalanceGap {
	expected: string;
	entered: string;
	previousImportId: number;
	previousPeriodTo: string;
}

export interface ManualOverlappingStatement {
	importId: number;
	periodFrom: string;
	periodTo: string;
	sourceFormat: string;
}

/** Réponse de l'aperçu (`200`, aucune écriture). */
export interface ManualStatementPreviewResponse {
	totalCredits: string;
	totalDebits: string;
	computedClosingBalance: string;
	transactionCount: number;
	warnings: {
		duplicateFile: DuplicateFilePayload | null;
		duplicateLines: ManualDuplicateLineWarning[];
		openingBalanceGap: ManualOpeningBalanceGap | null;
		overlappingStatements: ManualOverlappingStatement[];
	};
}
