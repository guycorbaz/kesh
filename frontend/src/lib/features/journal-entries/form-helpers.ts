/**
 * Helpers de transformation pour le formulaire d'écriture — extraction
 * et reconstitution du state LineDraft depuis/vers la forme API.
 */

import Big from 'big.js';
import type { JournalEntryLineResponse, JournalEntryResponse } from './journal-entries.types';

export interface LineDraft {
	accountId: number | null;
	debit: string;
	credit: string;
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
		credit: amountToFieldValue(line.credit)
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
