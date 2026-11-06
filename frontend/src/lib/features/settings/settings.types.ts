/** Adresse structurée de la société, telle que le serveur la rend et la reçoit. */
export interface StructuredAddress {
	street: string;
	building: string;
	postalCode: string;
	city: string;
	country: string | null;
}

export interface CompanyJson {
	id: number;
	name: string;
	address: string;
	/** Composants de l'adresse (#213), source du formulaire de correction (Story 33-5, #909). */
	addressStructured: StructuredAddress;
	/**
	 * Faux si le pays enregistré n'est pas un code ISO connu (Story 33-5, #909) :
	 * la QR-facture ne se produit pas. Jugé par le serveur, l'écran ne recopie pas la liste.
	 */
	addressCountryKnown: boolean;
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
	/** Borne **inclusive** du verrou de période (Story 24-4c, #380). `null` = aucun verrou. */
	booksLockedThrough: string | null;
	/** Verrou optimiste — requis par les routes `PUT /companies/current/*`. */
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

/** Payload de `PUT /companies/current/address` (Admin-only, Story 33-5, #909). */
export interface UpdateCompanyAddressRequest {
	address: StructuredAddress;
	version: number;
}

export interface BankAccountJson {
	id: number;
	bankName: string;
	/**
	 * Story 26-2b (#622) — description optionnelle, `null` si absente.
	 * **Obligatoire** : `npm run check` refuse une réponse simulée qui l'omet.
	 */
	description: string | null;
	iban: string;
	qrIban: string | null;
	isPrimary: boolean;
}

export interface CompanyCurrentResponse {
	company: CompanyJson;
	bankAccounts: BankAccountJson[];
}

/**
 * Story v0172-session (0.17.3, CR #891) — la durée d'inactivité de la session,
 * `GET /api/v1/admin/session-settings` (Administrateur seul). `version` vaut 0
 * tant que rien n'est enregistré (défaut 60 minutes).
 */
export interface SessionSettings {
	sessionIdleMinutes: number;
	version: number;
	min: number;
	max: number;
}

/** Corps de `PUT /api/v1/admin/session-settings`. */
export interface UpdateSessionSettingsRequest {
	sessionIdleMinutes: number;
	version: number;
}
