/**
 * Helpers purs pour les lots de paiement — Story 12.3 (#191).
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import type { PaymentBatchStatus } from './payment-batches.types';

/** Formate un montant (string décimal) au format suisse. `''` si vide/invalide. */
export function formatBatchTotal(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return formatSwissAmount(new Big(d));
	} catch {
		return '';
	}
}

/** Libellé FR du statut d'un lot (fallback i18n côté composant). */
export function paymentBatchStatusLabel(status: PaymentBatchStatus): string {
	switch (status) {
		case 'generated':
			return 'Généré';
		case 'confirmed':
			return 'Confirmé';
		case 'cancelled':
			return 'Annulé';
		default:
			return status;
	}
}

/** Libellé FR d'un code d'échec per-facture (FailedProposal). */
export function failedItemLabel(code: string): string {
	switch (code) {
		case 'SUPPLIER_INVOICE_NOT_FOUND':
			return 'Facture introuvable';
		case 'SUPPLIER_INVOICE_NOT_OPEN':
			return 'Facture non ouverte';
		case 'NO_PAYMENT_COORDINATES':
			return 'Pas de coordonnées de paiement (IBAN/QR-IBAN)';
		case 'ALREADY_IN_GENERATED_BATCH':
			return 'Déjà dans un lot en cours';
		case 'INVALID_IBAN':
			return 'IBAN invalide';
		case 'INVALID_QR_IBAN':
			return 'QR-IBAN invalide';
		default:
			return code;
	}
}
