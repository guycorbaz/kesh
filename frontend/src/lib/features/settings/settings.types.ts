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
	/**
	 * Téléphone et site web (Story 16-3a, #151) — rendus sur le PDF de facture.
	 *
	 * ⚠️ Cette interface est un miroir **écrit à la main** du DTO Rust : rien ne
	 * la vérifie contre le backend, et `npm run check` la valide contre
	 * elle-même. Un champ oublié ici est stocké, rendu sur le PDF, et invisible
	 * dans cet écran — sans qu'aucun gate ne le signale.
	 */
	phone: string | null;
	website: string | null;
	/** Verrou optimiste — requis par les routes `PUT /companies/current/*`. */
	/** Borne **inclusive** du verrou de période (Story 24-4c, #380). `null` = aucun verrou. */
	booksLockedThrough: string | null;
	version: number;
}

/** Payload de `PUT /api/v1/companies/current/email` (Admin-only, Story 20-3b2). */
export interface UpdateCompanyEmailRequest {
	/** `null` = effacer (Reply-To omis à l'envoi). */
	email: string | null;
	version: number;
}

/** Payload de `PUT /companies/current/contact-details` (Story 16-3a, #151). */
export interface UpdateCompanyContactDetailsRequest {
	/** `null` ou vide = effacer (la ligne disparaît du PDF). */
	phone: string | null;
	website: string | null;
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
