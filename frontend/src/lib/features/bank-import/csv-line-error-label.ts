/**
 * Mapping `code` d'une erreur de ligne CSV (`CsvLineErrorCode` côté
 * `kesh-import`, graphie `SCREAMING_SNAKE_CASE`) → libellé i18n.
 *
 * Story 26-1 (AC3-bis, #620) : les panneaux d'import n'affichaient que le code
 * brut, et le refus de signe (`AMOUNT_SIGN_MISMATCH`) aurait été illisible. Le
 * champ `messageI18nKey` du fil n'est volontairement **pas** lu : un `i18nMsg(`
 * dont la clé n'est pas littérale est un site non résolu pour
 * `i18n-keys.test.ts`. On passe donc par cette table littérale, déclarée dans
 * `MOTIFS_DYNAMIQUES` (famille `bank-import-csv-errors-`) — patron de
 * `$lib/features/imported-supplier-invoices/error-label.ts`.
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/**
 * `code` backend → `[suffixe de clé, repli français]`.
 *
 * ⚠️ **Exportée pour être LUE par `i18n-keys.test.ts`** (cardinalité de la
 * famille) : toute entrée ajoutée ici doit l'être aussi dans
 * `MOTIFS_DYNAMIQUES`, et la clé correspondante dans les quatre catalogues.
 */
export const CSV_LINE_ERROR_LABELS: Record<string, [string, string]> = {
	INVALID_DATE: ['invalid-date', 'Date invalide dans une ligne CSV.'],
	INVALID_AMOUNT: ['invalid-amount', 'Montant invalide dans une ligne CSV.'],
	AMBIGUOUS_DEBIT_CREDIT: ['ambiguous-debit-credit', 'Débit et crédit tous deux non-vides sur la même ligne.'],
	EMPTY_MANDATORY_FIELD: ['empty-mandatory-field', 'Champ obligatoire vide.'],
	ROW_TOO_SHORT: ['row-too-short', 'Ligne trop courte (colonnes manquantes).'],
	AMOUNT_SIGN_MISMATCH: [
		'amount-sign-mismatch',
		'Montant de signe contraire à sa colonne — vérifier le réglage du sens des débits du profil.',
	],
	// Story 27-2 (#624) — cellule Devise illisible.
	INVALID_CURRENCY: [
		'invalid-currency',
		'Devise illisible dans une ligne CSV (trois lettres attendues, par exemple CHF).',
	],
};

/** Libellé traduit d'un code d'erreur de ligne CSV ; code brut si inconnu. */
export function csvLineErrorLabel(code: string): string {
	const entry = CSV_LINE_ERROR_LABELS[code];
	if (entry) return i18nMsg(`bank-import-csv-errors-${entry[0]}`, entry[1]);
	return code;
}
