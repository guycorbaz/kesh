/**
 * Les motifs qui empêchent d'annuler un règlement **fournisseur** — Story 25-3-a-2 (#414).
 *
 * La tête fournisseur (`SUPPLIER_INVOICE_NOT_PAID`) est ici ; la queue commune
 * vit dans `$lib/shared/utils/settlement-cancel-blocked`, partagée avec le
 * client (Story 25-3-a-1), **sans jumeau**. L'union ne porte que les codes que
 * le serveur peut rendre pour un règlement fournisseur : aucun cas mort.
 */
import { i18nMsg } from "$lib/shared/utils/i18n.svelte";
import {
  settlementCancelTailMessage,
  type SettlementCancelTailCode,
} from "$lib/shared/utils/settlement-cancel-blocked";

export type SupplierSettlementCancelCode =
  "SUPPLIER_INVOICE_NOT_PAID" | SettlementCancelTailCode;

/** Le texte affiché à la place du bouton « Annuler le règlement ». */
export function supplierSettlementCancelMessage(
  code: SupplierSettlementCancelCode,
  label: string | null,
): string {
  if (code === "SUPPLIER_INVOICE_NOT_PAID") {
    return i18nMsg(
      "supplier-invoices-settlement-cancel-blocked-not-paid",
      "Cette facture fournisseur n'est pas payée : il n'y a pas de règlement à annuler.",
    );
  }
  return settlementCancelTailMessage(code, label);
}
