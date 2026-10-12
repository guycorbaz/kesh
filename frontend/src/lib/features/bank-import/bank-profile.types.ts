// Story 8-2 — Types frontend pour les profils CSV bancaires.

export type ColumnMapping = {
	date: number;
	amount?: number;
	debit_credit_split?: [number, number];
	reference?: number;
	details?: number;
	counterparty?: number;
	/** Index de la colonne Devise (facultatif) ; sans elle, le relevé est supposé
	 *  en CHF et l'aperçu le dit (Story 27-2, #624). */
	currency?: number;
};

/**
 * Sens des montants de la colonne Débit en mode débit/crédit séparés
 * (Story 26-1, #620) : `positive` = débits écrits sans signe, Kesh les rend
 * négatifs ; `negative` = débits déjà négatifs (UBS), Kesh les garde.
 */
export type DebitSign = 'positive' | 'negative';

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
	debitSign: DebitSign;
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
