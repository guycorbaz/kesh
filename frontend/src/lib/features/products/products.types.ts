/**
 * Types TS miroir des DTOs API pour le catalogue produits (Story 4.2).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/products.rs`
 * (serde `rename_all = "camelCase"`). Les montants décimaux (`unitPrice`,
 * `vatRate`) sont transportés en **string** grâce à la feature `serde-str`
 * de `rust_decimal` — ne pas convertir en `number` côté frontend (perte
 * de précision au-delà de `Number.MAX_SAFE_INTEGER`).
 */

export type ProductSortBy = 'Name' | 'UnitPrice' | 'VatRate' | 'CreatedAt';

export type SortDirection = 'Asc' | 'Desc';

export interface ProductResponse {
	id: number;
	companyId: number;
	name: string;
	description: string | null;
	/** Montant en string décimal (ex: `"1500.0000"`). Formater via `formatPrice`. */
	unitPrice: string;
	/** Pourcentage en string décimal (ex: `"8.10"`). */
	vatRate: string;
	active: boolean;
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface CreateProductRequest {
	name: string;
	description?: string | null;
	/** String décimal. Le backend parse via `Decimal::from_str`. */
	unitPrice: string;
	vatRate: string;
}

export interface UpdateProductRequest {
	name: string;
	description?: string | null;
	unitPrice: string;
	vatRate: string;
	version: number;
}

export interface ArchiveProductRequest {
	version: number;
}

export interface ListProductsQuery {
	search?: string;
	includeArchived?: boolean;
	sortBy?: ProductSortBy;
	sortDirection?: SortDirection;
	limit?: number;
	offset?: number;
}

/** Enveloppe paginée générique (miroir backend `ListResponse<T>`). */
export interface ListResponse<T> {
	items: T[];
	total: number;
	limit: number;
	offset: number;
}
