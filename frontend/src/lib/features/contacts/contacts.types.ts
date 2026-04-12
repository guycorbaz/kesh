/**
 * Types TS miroir des DTOs API pour les contacts (Story 4.1).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/contacts.rs`
 * (serde `rename_all = "camelCase"`).
 */

export type ContactType = 'Personne' | 'Entreprise';

export type ContactSortBy = 'Name' | 'CreatedAt' | 'UpdatedAt';

export type SortDirection = 'Asc' | 'Desc';

export interface ContactResponse {
	id: number;
	companyId: number;
	contactType: ContactType;
	name: string;
	isClient: boolean;
	isSupplier: boolean;
	address: string | null;
	email: string | null;
	phone: string | null;
	/** Forme normalisée `CHE109322551` (12 chars) ou null. Formatée à l'affichage. */
	ideNumber: string | null;
	defaultPaymentTerms: string | null;
	active: boolean;
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateContactRequest {
	contactType: ContactType;
	name: string;
	isClient: boolean;
	isSupplier: boolean;
	address?: string | null;
	email?: string | null;
	phone?: string | null;
	ideNumber?: string | null;
	defaultPaymentTerms?: string | null;
}

export interface UpdateContactRequest {
	contactType: ContactType;
	name: string;
	isClient: boolean;
	isSupplier: boolean;
	address?: string | null;
	email?: string | null;
	phone?: string | null;
	ideNumber?: string | null;
	defaultPaymentTerms?: string | null;
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
