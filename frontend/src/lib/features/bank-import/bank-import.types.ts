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
	warnings: string[];
	transactions: PreviewTransaction[];
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
