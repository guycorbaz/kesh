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
	/**
	 * Compte de produit de l'article (Story 16-2a, #144).
	 *
	 * `null` = l'article n'impose rien : la ligne de facture montée depuis lui
	 * reste à `null` et suit le compte de produit par défaut de la société.
	 * Toujours **présent** dans la réponse, jamais omis.
	 */
	defaultRevenueAccountId: number | null;
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
	/**
	 * ⚠️ **Non optionnel — pas de `?`**, contrairement à `description`.
	 *
	 * Le `PUT` est full-replace : omettre la clé **efface** le compte, sans
	 * erreur (décision D5 de 16-2a, CR #278). Rendre le champ obligatoire fait
	 * porter par **le compilateur** l'obligation « toujours envoyé, jamais
	 * omis » — plutôt que par la forme actuelle du code, où un littéral unique
	 * sert création et modification et masquerait un oubli.
	 */
	defaultRevenueAccountId: number | null;
}

export interface UpdateProductRequest {
	name: string;
	description?: string | null;
	unitPrice: string;
	vatRate: string;
	/** Cf. `CreateProductRequest` — non optionnel, et pour la même raison. */
	defaultRevenueAccountId: number | null;
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
