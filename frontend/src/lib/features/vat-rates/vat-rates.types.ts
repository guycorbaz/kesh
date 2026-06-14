/**
 * Types pour la feature `vat-rates` (Story 7.2 — KF-003 ; CRUD Story 11-1).
 *
 * `rate` est sérialisé en string décimale par le backend (feature `serde-str`
 * de `rust_decimal`), cohérent avec `vatRate` côté produits et factures.
 */

export type VatRateResponse = {
	id: number;
	/**
	 * Catégorie métier (clé extensible : `normal`/`reduced`/`special`/`exempt`/
	 * `custom` ou une catégorie officielle future). Affichée via la clé i18n
	 * `vat-category-{category}` (fallback : clé brute).
	 */
	category: string;
	/**
	 * Libellé d'affichage (clé i18n `product-vat-*` pour les taux seedés, ou
	 * texte libre) — résolu via `i18nMsg(label, fallback)`.
	 */
	label: string;
	/** Taux en string décimale (ex. `'8.10'`). */
	rate: string;
	/** Date ISO `YYYY-MM-DD` (inclusive). */
	validFrom: string;
	/** Date ISO `YYYY-MM-DD` (exclusive) ou `null` si pas d'expiration. */
	validTo: string | null;
	active: boolean;
	/** Verrou optimiste — à renvoyer au `PUT`/`DELETE`. */
	version: number;
};

/** Corps de `POST /api/v1/vat-rates`. */
export type NewVatRatePayload = {
	category: string;
	/** Optionnel — vide → le backend défaut à la catégorie. */
	label?: string;
	rate: string;
	validFrom: string;
	validTo?: string | null;
};

/** Corps de `PUT /api/v1/vat-rates/{id}`. `rate`/`category` non modifiables. */
export type UpdateVatRatePayload = {
	label?: string;
	validTo?: string | null;
	active: boolean;
	version: number;
};
