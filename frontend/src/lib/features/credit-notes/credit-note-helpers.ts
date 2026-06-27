/**
 * Helpers purs pour les avoirs (notes de crédit) — Story 12.1.
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import type { CreditNoteStatus } from './credit-notes.types';

/**
 * Formate un montant d'avoir (string décimal 4 décimales) au format suisse :
 * `"1000.0000"` → `"1’000.00"`. Retourne `''` si vide/invalide.
 */
export function formatCreditNoteTotal(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return formatSwissAmount(new Big(d));
	} catch {
		return '';
	}
}

/** Libellé FR du statut d'un avoir (fallback i18n côté composant). */
export function creditNoteStatusLabel(status: CreditNoteStatus): string {
	switch (status) {
		case 'draft':
			return 'Brouillon';
		case 'issued':
			return 'Émis';
		case 'cancelled':
			return 'Annulé';
		default:
			return status;
	}
}
