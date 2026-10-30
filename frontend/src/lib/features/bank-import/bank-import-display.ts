/**
 * Affichage d'un import bancaire selon son origine — Story v016-2b (#816, AC-23).
 *
 * Un relevé saisi à la main est un import de format `MANUAL` dont le `filename`
 * (`saisie-manuelle-<du>-<au>`) n'est pas un fichier : la liste, le détail et la
 * boîte de suppression l'affichent « Saisie manuelle » (et son numéro, porté par
 * `statementId`, s'il en a un), et son format « Saisie manuelle » au lieu du code.
 * Source unique de cette règle côté frontend (DRY).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/** Le `sourceFormat` d'un relevé saisi à la main (`BankImportSourceFormat::Manual`). */
export const MANUAL_SOURCE_FORMAT = 'MANUAL';

/** Vrai pour un relevé saisi à la main. */
export function isManualImport(imp: { sourceFormat: string }): boolean {
	return imp.sourceFormat === MANUAL_SOURCE_FORMAT;
}

/** Le nom d'un import : son fichier, ou « Saisie manuelle » (n° …) pour un relevé saisi. */
export function importDisplayName(imp: { sourceFormat: string; filename: string; statementId: string | null }): string {
	if (!isManualImport(imp)) return imp.filename;
	if (imp.statementId) {
		return i18nMsg('bank-import-detail-manual-name-numbered', 'Saisie manuelle n° { $number }', {
			number: imp.statementId,
		});
	}
	return i18nMsg('bank-import-detail-manual-name', 'Saisie manuelle');
}

/** Le format d'un import : « Saisie manuelle » pour un relevé saisi, sinon son code. */
export function sourceFormatLabel(sourceFormat: string): string {
	if (sourceFormat === MANUAL_SOURCE_FORMAT) return i18nMsg('bank-import-detail-manual-name', 'Saisie manuelle');
	return sourceFormat;
}
