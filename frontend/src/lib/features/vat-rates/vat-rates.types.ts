/**
 * Types pour la feature `vat-rates` (Story 7.2 — KF-003 closure).
 *
 * `rate` est sérialisé en string décimale par le backend (feature `serde-str`
 * de `rust_decimal` activée par défaut), cohérent avec `vatRate` côté
 * produits et factures.
 */

export type VatRateResponse = {
	id: number;
	/**
	 * Clé i18n (ex. `'product-vat-normal'`) — résolue côté frontend via
	 * `i18nMsg(label, fallback)`. Pas de texte traduit en DB.
	 */
	label: string;
	/** Taux en string décimale (ex. `'8.10'`). */
	rate: string;
	/** Date ISO `YYYY-MM-DD` (inclusive). */
	validFrom: string;
	/** Date ISO `YYYY-MM-DD` (exclusive) ou `null` si pas d'expiration. */
	validTo: string | null;
	active: boolean;
};
