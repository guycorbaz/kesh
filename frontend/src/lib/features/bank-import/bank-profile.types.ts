// Story 8-2 — Types frontend pour les profils CSV bancaires.

export type ColumnMapping = {
	date: number;
	amount?: number;
	debit_credit_split?: [number, number];
	reference?: number;
	details?: number;
	counterparty?: number;
};

export type BankProfile = {
	id: number;
	bankName: string;
	filenamePattern: string | null;
	columnMapping: ColumnMapping;
	dateFormat: string;
	decimalSeparator: string;
	fieldSeparator: string;
	encoding: string | null;
	headerRowCount: number;
	createdAt: string;
	updatedAt: string;
};

export type CreateOrUpdateBankProfileRequest = Omit<
	BankProfile,
	'id' | 'createdAt' | 'updatedAt'
>;

export type BankProfileListResponse = {
	items: BankProfile[];
	total: number;
	limit: number;
	offset: number;
};
