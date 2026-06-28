/**
 * Helpers purs pour les factures fournisseurs — Story 12.2 (#191).
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import type { SupplierInvoiceStatus } from './supplier-invoices.types';

/** Formate un montant (string décimal) au format suisse `1'081.00`. `''` si vide/invalide. */
export function formatSupplierInvoiceTotal(d: string | null | undefined): string {
	if (!d) return '';
	try {
		return formatSwissAmount(new Big(d));
	} catch {
		return '';
	}
}

/** Libellé FR du statut (fallback i18n côté composant). */
export function supplierInvoiceStatusLabel(status: SupplierInvoiceStatus): string {
	switch (status) {
		case 'open':
			return 'Ouverte';
		case 'paid':
			return 'Payée';
		case 'cancelled':
			return 'Annulée';
		default:
			return status;
	}
}
