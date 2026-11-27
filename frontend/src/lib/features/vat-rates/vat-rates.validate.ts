/**
 * Story 64-1 (#1178, D1/D2) — conversion et validation du taux saisi dans le
 * formulaire des taux de TVA. Fonctions pures.
 *
 * Svelte 5 lie un `<input type="number">` à un **nombre** (`null` si le champ est
 * vidé), non à une chaîne : appeler `.trim()` sur la valeur liée levait une
 * `TypeError`. Le taux part au serveur en chaîne décimale (`rate: string`).
 *
 * Les attributs `min`/`max`/`step` du champ ne protègent rien (le bouton n'est pas
 * l'envoi d'un `<form>`) : ce validateur arrête, avant toute écriture, les cas
 * évidents que `validate_rate` du serveur refuserait (`vat.rs`).
 */

/** Valeur liée au champ : nombre, chaîne (champ jamais touché) ou `null` (champ vidé). */
export type RateInput = string | number | null | undefined;

/** Cause d'un refus ; la page la traduit en message i18n. */
export type RateError = 'required' | 'invalid' | 'range' | 'decimals';

/** Le seul point de conversion : la valeur liée devient une chaîne décimale. */
export function decimalOf(value: RateInput): string {
	if (value === null || value === undefined) return '';
	// `-0 + 0` vaut `0` : `-0` s'écrit « 0 ».
	if (typeof value === 'number') return String(value + 0);
	return value.trim();
}

/** `null` si le taux est admis ; sinon la cause du refus. */
export function validateRateInput(value: RateInput): RateError | null {
	const text = decimalOf(value);
	if (text === '') return 'required';
	const n = Number(text);
	if (!Number.isFinite(n)) return 'invalid';
	if (n < 0 || n > 100) return 'range';
	// Au plus deux décimales (miroir de `validate_rate`) ; la tolérance absorbe l'erreur
	// binaire (8.1 × 100 = 810.0000000000001).
	const centi = n * 100;
	if (Math.abs(centi - Math.round(centi)) > 1e-9) return 'decimals';
	return null;
}

/** `null` si la date est saisie ; sinon `'required'`. */
export function validateDateInput(value: string | null | undefined): 'required' | null {
	return value ? null : 'required';
}
