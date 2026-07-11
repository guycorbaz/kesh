export interface CompanyJson {
	id: number;
	name: string;
	address: string;
	ideNumber: string | null;
	orgType: string;
	accountingLanguage: string;
	instanceLanguage: string;
	/** E-mail de contact (Story 20-3b2) — Reply-To des factures envoyées. `null` = non renseigné. */
	email: string | null;
	/** Verrou optimiste — requis par `PUT /companies/current/email`. */
	version: number;
}

/** Payload de `PUT /api/v1/companies/current/email` (Admin-only, Story 20-3b2). */
export interface UpdateCompanyEmailRequest {
	/** `null` = effacer (Reply-To omis à l'envoi). */
	email: string | null;
	version: number;
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
