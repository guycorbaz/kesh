/**
 * Mapping `errorCode` (`FailedFile` d'un rapport d'import) → libellé i18n.
 *
 * ⚠️ **Ce module existe parce que la carte, tant qu'elle vivait dans le composant,
 * pouvait GRANDIR sans qu'aucune garde ne rougisse.** `i18n-keys.test.ts` énumère les
 * suffixes attendus et en vérifie la cardinalité — mais les deux valeurs étaient
 * internes au test, si bien qu'une onzième entrée ajoutée à la production passait tous
 * les gates au vert et servait du français à toutes les locales. Mesuré en passe 4 de
 * revue de la story 23-3 : mutation appliquée, **111 tests verts**.
 *
 * Le sens décroissant, lui, était déjà couvert : un code retiré fait rougir la garde
 * d'existence des clés. C'est la **croissance** qui était aveugle — et c'est le cas réel,
 * l'Epic 12 devant ajouter des codes d'erreur d'import.
 *
 * Les codes proviennent des constantes `ERR_*` de `crates/kesh-api/src/inbox_import.rs`.
 * Même patron que `$lib/features/reminders/reminder-error-label.ts` (story 21-6b).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/**
 * `errorCode` backend → `[suffixe de clé, repli français]`.
 *
 * ⚠️ **Exportée pour être LUE par `i18n-keys.test.ts`.** Toute entrée ajoutée ici doit
 * l'être aussi dans `MOTIFS_DYNAMIQUES`, et la clé correspondante dans les quatre
 * catalogues — la garde le vérifie désormais depuis cette carte, plus depuis une copie.
 */
export const IMPORT_ERROR_LABELS: Record<string, [string, string]> = {
	UNSUPPORTED_FILE_TYPE: ['unsupported-file-type', 'Type de fichier non supporté'],
	FILE_TOO_LARGE: ['file-too-large', 'Fichier trop volumineux'],
	SYMLINK_REJECTED: ['symlink-rejected', 'Lien symbolique rejeté'],
	DUPLICATE: ['duplicate', 'Déjà importé (doublon)'],
	NO_QR_CODE_FOUND: ['no-qr-code-found', 'Aucune QR-facture détectée'],
	INVALID_SPC_PAYLOAD: ['invalid-spc-payload', 'QR illisible (format non SPC)'],
	INVALID_IBAN: ['invalid-iban', 'IBAN créancier invalide'],
	PDF_RENDER_ERROR: ['pdf-render-error', 'PDF illisible'],
	FILE_READ_ERROR: ['file-read-error', 'Lecture du fichier impossible'],
	FIELD_TOO_LONG: ['field-too-long', 'Un champ du QR dépasse la longueur autorisée'],
};

/** Libellé traduit d'un `errorCode` d'import ; repli sur le code brut si inconnu. */
export function importErrorLabel(code: string): string {
	const entry = IMPORT_ERROR_LABELS[code];
	if (entry) return i18nMsg(`imported-supplier-invoices-error-${entry[0]}`, entry[1]);
	return i18nMsg('imported-supplier-invoices-error-unknown', 'Échec de l’import ({$code})', {
		code,
	});
}
