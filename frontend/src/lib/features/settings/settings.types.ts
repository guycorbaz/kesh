export interface CompanyJson {
	id: number;
	name: string;
	address: string;
	ideNumber: string | null;
	orgType: string;
	accountingLanguage: string;
	instanceLanguage: string;
}

export interface BankAccountJson {
	id: number;
	bankName: string;
	iban: string;
	qrIban: string | null;
	isPrimary: boolean;
}

export interface CompanyCurrentResponse {
	company: CompanyJson;
	bankAccounts: BankAccountJson[];
}
