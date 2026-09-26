/**
 * Les options des listes « Type d'entité » et « Action » — Story 25-1c-b1 (#378).
 *
 * ⚠️ La route rend le vocabulaire dans l'ordre des **codes** : les options se
 * rangent ici par **libellé**, avec `Intl.Collator` dans la locale de
 * l'interface — un tri d'octets rangerait « Écriture » après « Utilisateur ».
 *
 * ⛔ Un code de l'URL absent du vocabulaire (code historique) est AJOUTÉ comme
 * option, libellée par son code : sans lui, la liste afficherait « Tous » alors
 * que la page filtre sur ce code.
 */

import type { AuditLogVocabularyItem } from './audit-log.types';

/** Une option de `<select>`. */
export interface SelectOption {
	value: string;
	label: string;
}

/**
 * Les options, triées par libellé dans `locale`, plus le code `current` s'il
 * manque au vocabulaire.
 */
export function toSelectOptions(
	items: AuditLogVocabularyItem[],
	locale: string,
	current: string | undefined,
): SelectOption[] {
	const collator = new Intl.Collator(locale, { sensitivity: 'base' });
	const options = items.map((i) => ({ value: i.code, label: i.label }));
	if (current && !items.some((i) => i.code === current)) {
		options.push({ value: current, label: current });
	}
	return options.sort((a, b) => collator.compare(a.label, b.label));
}
