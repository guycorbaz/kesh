/**
 * Les motifs qui empêchent d'annuler un règlement **client** (Story 25-3-a-1, #414).
 *
 * La tête client (`INVOICE_CREDITED`) est ici ; la queue commune vit dans
 * `$lib/shared/utils/settlement-cancel-blocked`, partagée avec le fournisseur.
 * L'union ne porte que les codes que le serveur peut rendre pour un règlement
 * client : aucun cas mort.
 */
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import {
	settlementCancelTailMessage,
	type SettlementCancelTailCode,
} from '$lib/shared/utils/settlement-cancel-blocked';

export type InvoiceSettlementCancelCode =
	| 'INVOICE_CREDITED'
	// Story 25-4-d2a (#384) : un solde existe — annuler d'abord le solde.
	| 'INVOICE_WRITTEN_OFF'
	| SettlementCancelTailCode;

/** Le texte affiché à la place du bouton « Annuler le règlement ». */
export function invoiceSettlementCancelMessage(
	code: InvoiceSettlementCancelCode,
	label: string | null,
): string {
	if (code === 'INVOICE_CREDITED') {
		return i18nMsg(
			'invoices-settlement-cancel-blocked-credited',
			'Cette facture a été créditée par un avoir : ce règlement est un paiement à lettrer, il ne s\'annule pas.',
		);
	}
	if (code === 'INVOICE_WRITTEN_OFF') {
		return i18nMsg(
			'invoices-settlement-cancel-blocked-written-off',
			"Le reste de cette facture a été soldé : annulez d'abord le solde.",
		);
	}
	return settlementCancelTailMessage(code, label);
}
