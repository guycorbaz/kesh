/**
 * Motifs traduits de la fiche d'une écriture — ce qui empêche de la
 * **contre-passer** (Story 24-4a) et de la **modifier** (Story 15-8a, #532).
 *
 * ⚠️ Le serveur rend un **code**, jamais une phrase : c'est ici que la
 * traduction se fait. Des `switch` exhaustifs plutôt qu'une table indexée, pour
 * qu'un code neuf fasse rougir le type-check au lieu d'afficher du vide.
 *
 * ⛔ **Une seule source pour les codes communs** : les sept motifs de pièce ou
 * de contre-passation que la modification partage avec la contre-passation
 * rendent les mêmes messages (mêmes clés), parce que le serveur les refuse sous
 * les mêmes codes — DRY, extrait de la fiche par la Story 15-8a.
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { ModificationBlocker, ReversalBlocker } from './journal-entries.types';

/** Motif de contre-passation, traduit (Story 24-4a). */
export function reversalBlockerLabel(code: ReversalBlocker): string {
	switch (code) {
		case 'IS_A_REVERSAL':
			return i18nMsg(
				'journal-entries-reverse-blocked-is-a-reversal',
				'Cette écriture est elle-même une contre-passation.'
			);
		case 'ALREADY_REVERSED':
			return i18nMsg(
				'journal-entries-reverse-blocked-already-reversed',
				'Cette écriture a déjà été contre-passée.'
			);
		case 'OWNED_BY_INVOICE':
			return i18nMsg(
				'journal-entries-reverse-blocked-invoice',
				'Cette écriture appartient à une facture client : dévalidez la facture, ou corrigez-la par un avoir.'
			);
		case 'OWNED_BY_CREDIT_NOTE':
			return i18nMsg(
				'journal-entries-reverse-blocked-credit-note',
				"Cette écriture est celle d'un avoir, qui est déjà une contre-passation."
			);
		case 'OWNED_BY_SUPPLIER_INVOICE':
			return i18nMsg(
				'journal-entries-reverse-blocked-supplier-invoice',
				'Cette écriture appartient à une facture fournisseur : elle se corrige depuis la fiche de la facture, qui indique ce qui est possible.'
			);
		case 'OWNED_BY_SETTLEMENT':
			return i18nMsg(
				'journal-entries-reverse-blocked-settlement',
				"Cette écriture est un règlement de facture : annulez le règlement depuis la fiche de la facture, qui indique si c'est possible."
			);
		case 'MATCHED_BANK_TRANSACTION':
			return i18nMsg(
				'journal-entries-reverse-blocked-bank-match',
				"Cette écriture est rapprochée d'une transaction bancaire : annulez le rapprochement depuis le détail de l'import bancaire."
			);
		case 'ACCOUNT_ARCHIVED':
			return i18nMsg(
				'journal-entries-reverse-blocked-account-archived',
				'Un compte de cette écriture a été archivé : réactivez-le pour pouvoir la contre-passer.'
			);
		default: {
			// ⛔ **C'est l'affectation à `never` qui fait rougir**, pas le
			// `default` : ajouter un code au type sans l'ajouter ici casse le
			// type-check. *(Passe 2 de revue de la 24-4a.)*
			const _exhaustif: never = code;
			void _exhaustif;
			// ⚠️ **Et l'on rend une chaîne vide, PAS `_exhaustif`.** À l'exécution,
			// `never` n'est qu'une fiction de compilation : un navigateur au bundle
			// périmé face à un serveur plus récent afficherait le code en clair.
			// *(Passe 3 de revue de la 24-4a.)*
			return '';
		}
	}
}

/**
 * Motif de modification, traduit (Story 15-8a, D8) — `label` est l'étiquette
 * que le serveur rend avec le motif : le nom de l'exercice postérieur clos, la
 * borne du verrou, ou le numéro de la pièce (suffixé par la fiche).
 */
export function modificationBlockerLabel(
	code: ModificationBlocker,
	label: string | null
): string {
	switch (code) {
		case 'FISCAL_YEAR_CLOSED':
			return i18nMsg(
				'journal-entries-modify-blocked-fiscal-year-closed',
				'L’exercice de cette écriture est clôturé : elle est figée. Corrigez-la par une contre-passation.'
			);
		case 'LATER_FISCAL_YEAR_CLOSED':
			return i18nMsg(
				'journal-entries-modify-blocked-later-fiscal-year-closed',
				'L’exercice postérieur { $name } est clôturé, et son bilan reprend cette écriture : elle reste figée tant qu’il l’est. Corrigez-la par une contre-passation ; sinon, un administrateur rouvre les exercices postérieurs clôturés, en commençant par le plus récent.',
				{ name: label ?? '' }
			);
		case 'PERIOD_LOCKED':
			return i18nMsg(
				'journal-entries-modify-blocked-period-locked',
				'La période est verrouillée jusqu’au { $date } : cette écriture, datée dans la période, reste figée. Corrigez-la par une contre-passation.',
				{ date: label ?? '' }
			);
		case 'DETACHED_SUPPLIER_SETTLEMENT':
			return i18nMsg(
				'journal-entries-modify-blocked-detached-settlement',
				'Ce paiement appartient à une facture fournisseur annulée : l’argent est sorti, il reste figé. Corrigez-le par une contre-passation.'
			);
		case 'IS_A_REVERSAL':
		case 'ALREADY_REVERSED':
		case 'OWNED_BY_INVOICE':
		case 'OWNED_BY_CREDIT_NOTE':
		case 'OWNED_BY_SUPPLIER_INVOICE':
		case 'OWNED_BY_SETTLEMENT':
		case 'MATCHED_BANK_TRANSACTION':
			return reversalBlockerLabel(code);
		default: {
			const _exhaustif: never = code;
			void _exhaustif;
			return '';
		}
	}
}

/**
 * Codes de modification dont l'étiquette est déjà DANS le message (nom de
 * l'exercice, borne) — la fiche ne la suffixe pas une seconde fois.
 */
export const MODIFICATION_LABEL_IN_MESSAGE: ReadonlySet<ModificationBlocker> = new Set([
	'LATER_FISCAL_YEAR_CLOSED',
	'PERIOD_LOCKED'
]);
