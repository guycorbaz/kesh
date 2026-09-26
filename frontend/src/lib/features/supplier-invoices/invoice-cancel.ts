/**
 * Les motifs qui empêchent d'annuler une **facture** fournisseur — Story 25-3-c (#454).
 *
 * ⛔ **Une famille de textes PROPRE**, et non la queue partagée
 * `settlement-cancel-blocked-*` : celle-ci dit « ce règlement », faux pour une
 * facture (même raison que la famille `reconciliation-cancel-blocked-*` de la
 * 25-3-b). Les codes sont ceux du serveur
 * (`supplier_invoices::supplier_invoice_cancel_blocker`), dans son ordre de
 * précédence ; l'union ne porte que ce qu'il **peut** rendre — le rang
 * `MATCHED_BANK_TRANSACTION`, inatteignable pour une écriture d'achat, y est
 * parce que le serveur ne le suppose pas.
 *
 * ⚠️ Les replis en dur disent **mot pour mot** le FTL fr-CH — le serveur rend
 * les mêmes clés quand il refuse (`crates/kesh-api/src/errors.rs`).
 */
import { i18nMsg } from "$lib/shared/utils/i18n.svelte";

export type SupplierInvoiceCancelCode =
  | "SUPPLIER_INVOICE_CANCELLED"
  | "FISCAL_YEAR_CLOSED"
  | "MATCHED_BANK_TRANSACTION"
  | "ACCOUNT_ARCHIVED"
  | "FISCAL_YEAR_INVALID"
  | "SUPPLIER_INVOICE_IN_PAYMENT_BATCH";

/**
 * Le texte affiché à la place du bouton « Annuler la facture ». `label`
 * porte le **numéro du compte** archivé.
 */
export function supplierInvoiceCancelMessage(
  code: SupplierInvoiceCancelCode,
  label: string | null,
): string {
  switch (code) {
    case "SUPPLIER_INVOICE_CANCELLED":
      return i18nMsg(
        "supplier-invoices-cancel-blocked-cancelled",
        "Cette facture fournisseur est déjà annulée.",
      );
    case "FISCAL_YEAR_CLOSED":
      return i18nMsg(
        "supplier-invoices-cancel-blocked-fiscal-year-closed",
        "Cette facture appartient à un exercice clôturé : un administrateur doit rouvrir l'exercice pour pouvoir l'annuler.",
      );
    case "MATCHED_BANK_TRANSACTION":
      return i18nMsg(
        "supplier-invoices-cancel-blocked-bank-match",
        "L'écriture d'achat de cette facture est rapprochée d'une transaction bancaire : annulez d'abord ce rapprochement.",
      );
    case "ACCOUNT_ARCHIVED": {
      const base = i18nMsg(
        "supplier-invoices-cancel-blocked-account-archived",
        "Un compte de l'écriture d'achat de cette facture a été archivé : réactivez-le pour pouvoir annuler la facture.",
      );
      return label ? `${base} (${label})` : base;
    }
    case "FISCAL_YEAR_INVALID":
      return i18nMsg(
        "supplier-invoices-cancel-blocked-no-fiscal-year",
        "Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler cette facture.",
      );
    case "SUPPLIER_INVOICE_IN_PAYMENT_BATCH":
      return i18nMsg(
        "supplier-invoices-cancel-blocked-in-payment-batch",
        "Cette facture figure dans un lot de paiement en cours : annulez d'abord le lot.",
      );
    default: {
      const exhaustive: never = code;
      return exhaustive;
    }
  }
}
