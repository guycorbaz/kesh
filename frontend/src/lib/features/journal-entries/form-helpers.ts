/**
 * Helpers du formulaire d'écriture — extraction et reconstitution du state
 * `LineDraft` depuis/vers la forme API, et règles du **mode édition**.
 *
 * Story 15-8a (#532) : le mode édition est rétabli (supprimé par la 24-4b), non
 * plus depuis la liste mais depuis la **fiche** de l'écriture. Ces fonctions,
 * pures, portent ce qui se teste sans monter le composant.
 */

import Big from 'big.js';
import type { JournalEntryLineResponse, JournalEntryResponse } from './journal-entries.types';

export interface LineDraft {
	accountId: number | null;
	debit: string;
	credit: string;
	/** Projet analytique de la ligne (Epic 19). `null` = non taguée. */
	projectId: number | null;
}

/**
 * Règle : si le montant reçu depuis l'API vaut 0, on laisse la string
 * vide dans le state du formulaire — cela permet de distinguer visuellement
 * une ligne débit d'une ligne crédit et correspond à la sémantique
 * « ligne incomplète » du composant.
 */
function amountToFieldValue(raw: string): string {
	if (!raw || raw === '') return '';
	try {
		return new Big(raw).eq(0) ? '' : raw;
	} catch {
		return raw;
	}
}

/**
 * Convertit une ligne persistée en draft de formulaire.
 */
export function lineResponseToDraft(line: JournalEntryLineResponse): LineDraft {
	return {
		accountId: line.accountId,
		debit: amountToFieldValue(line.debit),
		credit: amountToFieldValue(line.credit),
		projectId: line.projectId ?? null
	};
}

/**
 * Convertit une écriture existante en `LineDraft[]` prêt à injecter
 * dans l'état initial de `JournalEntryForm`.
 */
export function fromJournalEntryResponse(entry: JournalEntryResponse): LineDraft[] {
	return entry.lines
		.slice()
		.sort((a, b) => a.lineOrder - b.lineOrder)
		.map(lineResponseToDraft);
}

/** Le lendemain d'une date `AAAA-MM-JJ`. */
function nextDay(iso: string): string {
	const d = new Date(`${iso}T00:00:00Z`);
	d.setUTCDate(d.getUTCDate() + 1);
	return d.toISOString().slice(0, 10);
}

/**
 * Bornes du champ date (Story 15-8a, D8).
 *
 * - `min` : le plus tardif du début de l'exercice **de l'écriture** et du
 *   lendemain de la borne du verrou de période (la borne est INCLUSIVE) ;
 * - `max` : la fin de l'exercice de l'écriture.
 *
 * En création (`entryFiscalYear` nul), seul le verrou borne le `min`, comme
 * depuis la 24-4c. ⚠️ **Confort de saisie seulement** : le refus qui fait
 * autorité est le 400 du serveur (`DATE_OUTSIDE_FISCAL_YEAR`, `PERIOD_LOCKED`).
 */
export function entryDateBounds(
	entryFiscalYear: { startDate: string; endDate: string } | null,
	booksLockedThrough: string | null
): { min: string | undefined; max: string | undefined } {
	const afterLock = booksLockedThrough ? nextDay(booksLockedThrough) : undefined;
	const start = entryFiscalYear?.startDate;
	let min: string | undefined;
	if (start && afterLock) {
		min = start > afterLock ? start : afterLock;
	} else {
		min = start ?? afterLock;
	}
	return { min, max: entryFiscalYear?.endDate };
}

/**
 * Ce que le formulaire fait d'un refus du serveur **en mode édition** (Story
 * 15-8a, D8) :
 *
 * - `stale` : l'état de l'écriture a changé sous l'utilisateur (exercice clos,
 *   exercice postérieur clos, pièce ou contre-passation apparue entre-temps,
 *   version périmée) — toast, puis la fiche se recharge (`onStale`) ; **aucune
 *   modale** ;
 * - `stay` : la saisie est en cause (période, date, équilibre, comptes, forme) —
 *   toast, le formulaire reste ouvert pour corriger ;
 * - `other` : tout le reste, traité comme à la création.
 *
 * ⚠️ `FISCAL_YEAR_CLOSED` est `stale` en édition : c'est l'exercice **de
 * l'écriture** qui a été clôturé — le conseil de
 * `notifyMissingFiscalYearOrFallback` (« vérifiez la date saisie ») y serait faux.
 */
export function editRefusalOutcome(code: string): 'stale' | 'stay' | 'other' {
	switch (code) {
		case 'FISCAL_YEAR_CLOSED':
		case 'LATER_FISCAL_YEAR_CLOSED':
		case 'ENTRY_IS_REVERSED':
		case 'IS_A_REVERSAL':
		case 'OWNED_BY_INVOICE':
		case 'OWNED_BY_CREDIT_NOTE':
		case 'OWNED_BY_SUPPLIER_INVOICE':
		case 'OWNED_BY_SETTLEMENT':
		case 'MATCHED_BANK_TRANSACTION':
		case 'DETACHED_SUPPLIER_SETTLEMENT':
		case 'OPTIMISTIC_LOCK_CONFLICT':
			return 'stale';
		case 'PERIOD_LOCKED':
		case 'DATE_OUTSIDE_FISCAL_YEAR':
		case 'ENTRY_UNBALANCED':
		case 'INACTIVE_OR_INVALID_ACCOUNTS':
		case 'ACCOUNT_NOT_POSTABLE':
		case 'VALIDATION_ERROR':
			return 'stay';
		default:
			return 'other';
	}
}
