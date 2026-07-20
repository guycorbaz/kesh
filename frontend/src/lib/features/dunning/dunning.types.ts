/**
 * Types de la feature `dunning` (rappels débiteurs — Story 21-4, #231).
 *
 * camelCase (le backend sérialise en camelCase). Les montants décimaux sont des
 * `string` (rust_decimal `serde-str`, comme `VatRateResponse.rate`).
 */

/** Un niveau de rappel configuré (`dunning_levels`). */
export interface DunningLevelResponse {
	id: number;
	/** Numéro de niveau 1-based, contigu. */
	levelNumber: number;
	/** Délai en jours depuis l'étape précédente (échéance + grâce pour le 1er). */
	delayDays: number;
	/** Frais de rappel en CHF (décimale sérialisée en string). */
	feeAmount: string;
	/** Verrou optimiste. */
	version: number;
}

/** Payload de création d'un niveau (le backend calcule `levelNumber` = MAX+1). */
export interface CreateDunningLevelPayload {
	delayDays: number;
	feeAmount: string;
}

/** Payload de modification (délai + frais ; `levelNumber` immutable). */
export interface UpdateDunningLevelPayload {
	delayDays: number;
	feeAmount: string;
	version: number;
}

/** Réglages de rappel de l'entreprise (`company_dunning_settings`, singleton). */
export interface DunningSettingsResponse {
	/** Jours de grâce après l'échéance avant le 1er rappel. */
	gracePeriodDays: number;
	/** Horodatage du seed initial (null = jamais seedé). */
	seededAt: string | null;
	/** Verrou optimiste. */
	version: number;
}

/** Payload de modification des réglages (période de grâce). */
export interface UpdateDunningSettingsRequest {
	gracePeriodDays: number;
	version: number;
}
