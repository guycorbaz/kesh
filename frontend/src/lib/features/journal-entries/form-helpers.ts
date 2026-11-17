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
import { foreignFromResponse, type LineForeign } from './fx-lines';

export interface LineDraft {
	accountId: number | null;
	/**
	 * Montant au débit en devise principale (CHF). Pour une ligne en devise,
	 * **le CHF dérivé seul** (Story 37-3b, contrat de `fx-lines.ts`).
	 */
	debit: string;
	credit: string;
	/** Projet analytique de la ligne (Epic 19). `null` = non taguée. */
	projectId: number | null;
	/**
	 * Story 37-3b (AC-2) — partie en devise ; absente = ligne CHF. Sa devise est
	 * celle du compte, jamais saisie.
	 */
	foreign?: LineForeign;
	/**
	 * Story 37-3b (AC-2, S5) — ligne CHF que l'assistant TVA a calculée au cours
	 * d'une ligne en devise (le cours de l'insertion) ; effacée par toute retouche
	 * du montant. Comptée dans le seuil « arrondi » de la ligne d'écart.
	 */
	chfFromForeign?: { rate: string } | null;
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
	const draft: LineDraft = {
		accountId: line.accountId,
		debit: amountToFieldValue(line.debit),
		credit: amountToFieldValue(line.credit),
		projectId: line.projectId ?? null
	};
	// Story 37-3b (AC-3) — la partie en devise relue : devise de la réponse
	// (celle du compte), provenance relue gardée tant que rien n'est retouché.
	const foreign = foreignFromResponse(line);
	if (foreign) draft.foreign = foreign;
	return draft;
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
		case 'ENTRY_LETTERED':
		case 'OPTIMISTIC_LOCK_CONFLICT':
		// Story 37-3b (AC-2) — l'écriture est devenue une réévaluation : la fiche
		// se recharge et dit le motif.
		case 'ENTRY_IS_FX_REVALUATION':
			return 'stale';
		case 'PERIOD_LOCKED':
		case 'DATE_OUTSIDE_FISCAL_YEAR':
		case 'ENTRY_UNBALANCED':
		case 'INACTIVE_OR_INVALID_ACCOUNTS':
		case 'ACCOUNT_NOT_POSTABLE':
		case 'VALIDATION_ERROR':
			return 'stay';
		default:
			// Story 37-3b (AC-2) — les refus propres aux devises : la saisie est en
			// cause, le formulaire reste ouvert.
			return isFxRefusalCode(code) ? 'stay' : 'other';
	}
}

/**
 * Story 37-3b (AC-3) — « Supprimer » est-il offert sur la fiche ? Quand
 * l'écriture se modifie ; et pour une écriture de **réévaluation**
 * (`ENTRY_IS_FX_REVALUATION`, qui ne se modifie pas) hors période verrouillée et
 * sans ligne lettrée — ce motif se rend **avant** `PERIOD_LOCKED` et le
 * lettrage, que la suppression contrôle encore : sans ces deux termes, le bouton
 * serait offert pour un refus certain. `booksLockedThrough` inconnu
 * (`undefined`, lecture échouée) : bouton absent pour une réévaluation.
 */
export function canDeleteEntry(
	entry: {
		modifiable: boolean;
		modificationBlockedBy: string | null;
		entryDate: string;
		lines: { letteringKey: number | null }[];
	},
	booksLockedThrough: string | null | undefined
): boolean {
	if (entry.modifiable) return true;
	if (entry.modificationBlockedBy !== 'ENTRY_IS_FX_REVALUATION') return false;
	if (booksLockedThrough === undefined) return false;
	const periodLocked = booksLockedThrough !== null && entry.entryDate <= booksLockedThrough;
	const lettered = entry.lines.some((l) => l.letteringKey !== null);
	return !periodLocked && !lettered;
}

/**
 * Story 37-3b (AC-2) — les codes de refus propres aux lignes en devise :
 * `RATE_UNAVAILABLE`, tout `FX_*` (`FX_RATE_NOT_AFC`, `FX_AMOUNT_MISMATCH`,
 * `FX_CURRENCY_MISMATCH`, `FX_ACCOUNT_IN_CHF`, `FX_MULTIPLE_CURRENCIES`,
 * `FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO`, `FX_ACCOUNT_INVALID`,
 * `FX_CONVERSION_TRANSIT_UNBALANCED`), `ACCOUNT_REQUIRES_CURRENCY` et
 * `MULTI_CURRENCY_DISABLED` (exercice mono-devise).
 */
export function isFxRefusalCode(code: string): boolean {
	return (
		code.startsWith('FX_') ||
		code === 'RATE_UNAVAILABLE' ||
		code === 'ACCOUNT_REQUIRES_CURRENCY' ||
		code === 'MULTI_CURRENCY_DISABLED'
	);
}
