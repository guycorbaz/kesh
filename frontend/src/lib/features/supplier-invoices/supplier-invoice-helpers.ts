/**
 * Helpers purs pour les factures fournisseurs — Story 12.2 (#191).
 */

import Big from 'big.js';
import { formatSwissAmount } from '$lib/features/journal-entries/balance';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
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

/**
 * Libellé traduit du statut d'une facture fournisseur.
 *
 * ⚠️ **Ces trois valeurs étaient en dur, et rien ne pouvait le signaler.** Elles
 * n'appelaient pas `i18nMsg`, donc ni le moissonneur, ni l'allowlist, ni les deux
 * gardes i18n ne les voyaient — c'est l'angle mort #255. Le défaut était visible à
 * l'écran et nulle part ailleurs : la story 23-3 a traduit l'en-tête de la colonne
 * (`supplier-invoices-col-status`), si bien qu'un germanophone lisait « Status » au
 * -dessus de « Payée », sur chaque ligne de la liste et en badge sur chaque fiche.
 * Trouvé en passe 4 de revue, par deux lentilles indépendantes.
 *
 * Les trois termes n'ont demandé aucun arbitrage : `ouverte` et `payée` sont en
 * partie A du glossaire, et `annulée` est attesté par `invoice-status-cancelled`.
 */
export function supplierInvoiceStatusLabel(status: SupplierInvoiceStatus): string {
	switch (status) {
		case 'open':
			return i18nMsg('supplier-invoices-status-open', 'Ouverte');
		case 'paid':
			return i18nMsg('supplier-invoices-status-paid', 'Payée');
		case 'cancelled':
			return i18nMsg('supplier-invoices-status-cancelled', 'Annulée');
		default:
			return status;
	}
}
