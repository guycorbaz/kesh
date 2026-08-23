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
