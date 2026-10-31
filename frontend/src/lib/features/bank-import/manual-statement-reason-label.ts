/**
 * Mapping `reason` d'un refus `BANK_STATEMENT_INVALID` (relevé saisi à la main,
 * ensemble fermé `ManualStatementReason` de `kesh-core`) → libellé i18n.
 * Story v016-2b (#816, AC-24) — patron de `csv-line-error-label.ts`.
 *
 * ⚠️ **Forme imposée (validation P4 de la fiche, P2) : un test Rust lit ce fichier**
 * (`crates/kesh-api/tests/textes_coherents.rs`) et égale ses clés, dans les deux
 * sens, à `ManualStatementReason::ALL` mappé par `code()`. Un seul objet
 * `export const MANUAL_STATEMENT_REASON_LABELS = {` … `};`, **une entrée par
 * ligne**, clé littérale en début de ligne, repli compris, et **aucun commentaire
 * dans le bloc**.
 *
 * Exportée aussi pour `i18n-keys.test.ts` (cardinalité de la famille
 * `bank-import-manual-reason-` de `MOTIFS_DYNAMIQUES`) : toute entrée ajoutée ici
 * l'est aussi dans `MOTIFS_DYNAMIQUES` et dans les quatre catalogues.
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/** `reason` serveur → `[suffixe de clé, repli français]`. */
export const MANUAL_STATEMENT_REASON_LABELS: Record<string, [string, string]> = {
	required: ['required', 'valeur requise'],
	invalid_date: ['invalid-date', 'date illisible (AAAA-MM-JJ attendu)'],
	invalid_number: ['invalid-number', 'nombre illisible'],
	period_inverted: ['period-inverted', 'la fin de la période précède son début'],
	too_many_lines: ['too-many-lines', 'plus de 500 lignes'],
	out_of_period: ['out-of-period', 'date hors de la période du relevé'],
	too_long: ['too-long', 'texte trop long'],
	not_positive: ['not-positive', 'montant nul ou négatif'],
	too_many_decimals: ['too-many-decimals', 'plus de deux décimales'],
	too_many_digits: ['too-many-digits', 'plus de treize chiffres avant la virgule'],
	unknown_direction: ['unknown-direction', 'sens inconnu (débit ou crédit attendu)'],
};

/** Libellé traduit d'un `reason` ; code brut si inconnu. */
export function manualStatementReasonLabel(reason: string): string {
	const entry = MANUAL_STATEMENT_REASON_LABELS[reason];
	if (entry) return i18nMsg(`bank-import-manual-reason-${entry[0]}`, entry[1]);
	return reason;
}
