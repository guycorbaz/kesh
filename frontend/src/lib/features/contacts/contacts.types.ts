/**
 * Types TS miroir des DTOs API pour les contacts (Story 4.1).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/contacts.rs`
 * (serde `rename_all = "camelCase"`).
 */

export type ContactType = 'Personne' | 'Entreprise';

/** Langue de correspondance (Story 20-3b2). `null` = héritée de la langue d'instance. */
export type ContactLanguage = 'FR' | 'DE' | 'IT' | 'EN';

/** Civilité pour la formule d'appel des e-mails (Story 20-3b2). */
export type Salutation = 'Monsieur' | 'Madame' | 'Neutre';

export type ContactSortBy = 'Name' | 'CreatedAt' | 'UpdatedAt';

export type SortDirection = 'Asc' | 'Desc';

/** Adresse structurée (#213, QR-bill type S). */
export interface StructuredAddress {
	street: string;
	building: string;
	postalCode: string;
	city: string;
	country: string;
}

export function emptyAddress(): StructuredAddress {
	return { street: '', building: '', postalCode: '', city: '', country: 'CH' };
}

export interface ContactResponse {
	id: number;
	companyId: number;
	contactType: ContactType;
	name: string;
	/** Prénom / nom (#213) — renseignés pour les Personne. */
	firstName: string | null;
	lastName: string | null;
	isClient: boolean;
	isSupplier: boolean;
	/** Chaîne d'affichage dérivée (#213). */
	address: string | null;
	/** Adresse structurée (source de vérité éditable, #213). */
	addressStructured: StructuredAddress;
	email: string | null;
	phone: string | null;
	/** Forme normalisée `CHE109322551` (12 chars) ou null. Formatée à l'affichage. */
	ideNumber: string | null;
	defaultPaymentTerms: string | null;
	/** Délai de paiement en jours (#245). `null` = non renseigné. */
	defaultPaymentTermsDays: number | null;
	/**
	 * Libellé localisé des conditions (#245), généré côté serveur dans la
	 * langue du CONTACT (le i18n frontend ne connaît que la locale UI).
	 * `null` si `defaultPaymentTermsDays` absent.
	 */
	defaultPaymentTermsLabel: string | null;
	/** Langue de correspondance (Story 20-3b2). `null` = héritée instance. */
	language: ContactLanguage | null;
	/** Civilité (Story 20-3b2). Toujours renseignée (défaut `Neutre`). */
	salutation: Salutation;
	active: boolean;
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateContactRequest {
	contactType: ContactType;
	name: string;
	firstName?: string | null;
	lastName?: string | null;
	isClient: boolean;
	isSupplier: boolean;
	addressStructured: StructuredAddress;
	email?: string | null;
	phone?: string | null;
	ideNumber?: string | null;
	defaultPaymentTerms?: string | null;
	/** Délai de paiement en jours 0..365 (#245). Absent/`null` = non renseigné. */
	defaultPaymentTermsDays?: number | null;
	/** Absent/`null` = héritée instance (Story 20-3b2). */
	language?: ContactLanguage | null;
	/** Absent = `Neutre` (Story 20-3b2). */
	salutation?: Salutation;
}

export interface UpdateContactRequest {
	contactType: ContactType;
	name: string;
	firstName?: string | null;
	lastName?: string | null;
	isClient: boolean;
	isSupplier: boolean;
	addressStructured: StructuredAddress;
	email?: string | null;
	phone?: string | null;
	ideNumber?: string | null;
	defaultPaymentTerms?: string | null;
	/** Délai de paiement en jours 0..365 (#245). Absent/`null` = effacé (PUT full-payload). */
	defaultPaymentTermsDays?: number | null;
	/** Absent/`null` = héritée instance (Story 20-3b2). */
	language?: ContactLanguage | null;
	/** Absent = `Neutre` (Story 20-3b2). */
	salutation?: Salutation;
	version: number;
}

export interface ArchiveContactRequest {
	version: number;
}

export interface ListContactsQuery {
	search?: string;
	contactType?: ContactType;
	isClient?: boolean;
	isSupplier?: boolean;
	includeArchived?: boolean;
	sortBy?: ContactSortBy;
	sortDirection?: SortDirection;
	limit?: number;
	offset?: number;
}

/** Enveloppe paginée générique (miroir de `ListResponse<T>` backend). */
export interface ListResponse<T> {
	items: T[];
	total: number;
	limit: number;
	offset: number;
}
