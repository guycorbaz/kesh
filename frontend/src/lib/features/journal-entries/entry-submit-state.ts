/**
 * Story 53-1a (#1355) — la saisie d'une écriture ne passe pas deux fois : les états
 * de l'envoi (table J de la fiche) et la lecture du refus de doublon du serveur.
 *
 * Module **pur**, écrit sans dépendance au formulaire : la conversion de devises
 * l'emploiera (Story 53-1b), qui l'étendra d'un champ facultatif pour le rang de
 * l'écriture (`details.entry`). Il suit celui de la 48-12b
 * (`supplier-invoice-form.ts`, table A), sans le refactorer.
 *
 * Importe `isApiError` d'`api-error`, **non** d'`api-client` : des fichiers de test
 * simulent `$lib/shared/utils/api-client` sans `isApiError` (cf. `unknown-outcome.ts`).
 */

import { isApiError } from '$lib/shared/utils/api-error';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

/**
 * Les états de l'envoi d'une écriture **à la création** (table J) :
 * - `J1` saisie ; `J2a` attente des recherches de cours en vol, corps non formé ;
 * - `J2b` requête en vol, corps formé et gardé ; `J3` enregistré ;
 * - `J5` doublon probable, confirmable — figé, deux boutons ;
 * - `J6` issue inconnue — figé, lien de vérification et bouton de reprise ;
 * - `J7` erreur ordinaire — saisie modifiable. (Pas de `J4` : le serveur ne rend
 *   jamais ce refus non confirmable.)
 */
export type EntrySubmitState = 'J1' | 'J2a' | 'J2b' | 'J3' | 'J5' | 'J6' | 'J7';

/** Tous les états, pour les tests et les gardes. */
export const ENTRY_SUBMIT_STATES: readonly EntrySubmitState[] = [
	'J1',
	'J2a',
	'J2b',
	'J3',
	'J5',
	'J6',
	'J7'
];

/**
 * Vrai si la saisie peut être **écrite** dans cet état : J1, J2a, J7.
 *
 * - En **J2a** le corps n'est pas encore formé : une recherche de cours en vol peut
 *   encore mettre à jour la ligne, et c'est voulu — l'envoi attend les recherches
 *   pour que le cours mis à jour parte avec lui (37-3b, AC-2 ; correctif de
 *   37-3b-CR-P1-5). L'interdire rouvrirait ce défaut.
 * - À partir de **J2b** le corps est formé et gardé (c'est lui que renvoie la
 *   confirmation de J5) : une écriture tardive ferait diverger l'écran de ce qui a été
 *   envoyé. Défense en profondeur : ces cas sont inatteignables par l'interface.
 */
export function canWriteEntry(state: EntrySubmitState): boolean {
	return state === 'J1' || state === 'J2a' || state === 'J7';
}

/** Vrai si un envoi **ordinaire** (« Valider », Ctrl+S) peut partir de cet état : J1, J7. */
export function canSendFrom(state: EntrySubmitState): boolean {
	return state === 'J1' || state === 'J7';
}

/** Vrai si la **confirmation** « Ce n'est pas un doublon » peut partir : J5 seulement. */
export function canConfirmFrom(state: EntrySubmitState): boolean {
	return state === 'J5';
}

/**
 * Vrai si les contrôles de la saisie sont **figés** (`<fieldset disabled>`) : tout état
 * sauf J1 et J7.
 */
export function isEntryFrozen(state: EntrySubmitState): boolean {
	return state !== 'J1' && state !== 'J7';
}

/** Le code du refus de doublon à la saisie (Story 53-1a, D7). */
export const ENTRY_DUPLICATE_CODE = 'JOURNAL_ENTRY_PROBABLE_DUPLICATE';

/**
 * Un refus de doublon **lisible** (`details` du contrat) : l'écriture existante citée,
 * et `confirmable`, qui seul décide d'un bouton de confirmation.
 */
export interface EntryDuplicateRefusal {
	journalEntryId: number;
	entryNumber: number;
	/** `AAAA-MM-JJ`. */
	entryDate: string;
	confirmable: boolean;
}

/**
 * Lit un refus de doublon : `null` si ce n'est pas ce code, **ou** si ses `details` ne
 * sont pas lisibles (identifiants entiers, date, `confirmable` booléen) — un tel refus
 * est une erreur ordinaire (J7), affichée avec le message du serveur.
 */
export function readEntryDuplicateRefusal(err: unknown): EntryDuplicateRefusal | null {
	if (!isApiError(err) || err.code !== ENTRY_DUPLICATE_CODE) return null;
	const d = err.details as Record<string, unknown> | null | undefined;
	if (!d || typeof d !== 'object') return null;
	const { journalEntryId, entryNumber, entryDate, confirmable } = d;
	if (
		typeof journalEntryId !== 'number' ||
		!Number.isInteger(journalEntryId) ||
		typeof entryNumber !== 'number' ||
		!Number.isInteger(entryNumber) ||
		typeof entryDate !== 'string' ||
		!/^\d{4}-\d{2}-\d{2}$/.test(entryDate) ||
		typeof confirmable !== 'boolean'
	) {
		return null;
	}
	return { journalEntryId, entryNumber, entryDate, confirmable };
}

/**
 * L'état qui suit un rejet de l'envoi (table J, sorties de J2b) : un refus de doublon
 * lisible **et confirmable** → J5 ; issue inconnue (`isSettlementOutcomeUnknown` : tout
 * 5xx, coupure, délai, erreur hors `ApiError`) → J6 ; tout le reste — le code de doublon
 * sans `details` lisibles compris — → J7.
 */
export function stateAfterRejection(
	err: unknown,
	isUnknown: (e: unknown) => boolean
): 'J5' | 'J6' | 'J7' {
	const dup = readEntryDuplicateRefusal(err);
	if (dup !== null && dup.confirmable) return 'J5';
	if (isUnknown(err)) return 'J6';
	return 'J7';
}

/** Le texte du refus de doublon, **le même que celui du serveur** : la clé du catalogue. */
export function entryDuplicateText(): string {
	return i18nMsg(
		'error-journal-entry-probable-duplicate',
		'Une écriture identique existe déjà à cette date (même journal, même libellé, mêmes comptes et montants). Si c’est la même, ne l’enregistrez pas une seconde fois ; sinon, confirmez que ce n’est pas un doublon.'
	);
}

/**
 * Le lien de vérification de J6 : le journal, filtré sur le jour de la saisie
 * (`/journal-entries?dateFrom=D&dateTo=D`).
 */
export function unknownOutcomeHref(entryDate: string): string {
	const params = new URLSearchParams({ dateFrom: entryDate, dateTo: entryDate });
	return `/journal-entries?${params.toString()}`;
}
