/**
 * Ce qui empêche d'annuler un règlement — la **queue commune** (Story 25-3-a-1, #414).
 *
 * ⛔ **Partagé entre le client et le fournisseur.** Ces quatre motifs tiennent
 * à l'**écriture** de règlement (son exercice, son rapprochement, ses comptes,
 * l'exercice du jour), pas à la pièce qui la possède : la fiche facture client
 * et la fiche facture fournisseur (Story 25-3-a-2) les affichent par ce seul
 * module. Chaque fiche ajoute sa **tête** propre (`INVOICE_CREDITED` côté
 * client) dans son dossier de fonctionnalité.
 *
 * ⚠️ Vit dans `lib/shared/` et non dans un dossier `features/` : le lint
 * d'appartenance des clés (`lint-i18n-ownership`) ne balaie que `features/`,
 * et ces clés servent deux fonctionnalités.
 *
 * ⚠️ Les replis en dur disent **mot pour mot** le FTL fr-CH — le serveur rend
 * les mêmes clés quand il refuse (`crates/kesh-api/src/errors.rs`).
 */
import { i18nMsg } from "./i18n.svelte";

/** Les codes de la queue commune, dans l'ordre de précédence du serveur. */
export type SettlementCancelTailCode =
  | "FISCAL_YEAR_CLOSED"
  | "MATCHED_BANK_TRANSACTION"
  | "ACCOUNT_ARCHIVED"
  | "FISCAL_YEAR_INVALID";

/**
 * Le texte d'un motif de la queue commune. `label` porte le **numéro du
 * compte** archivé — sans lui, « réactivez-le » ne dirait pas lequel.
 */
export function settlementCancelTailMessage(
  code: SettlementCancelTailCode,
  label: string | null,
): string {
  switch (code) {
    case "FISCAL_YEAR_CLOSED":
      return i18nMsg(
        "settlement-cancel-blocked-fiscal-year-closed",
        "Ce règlement appartient à un exercice clôturé : un administrateur doit rouvrir l'exercice pour pouvoir l'annuler.",
      );
    case "MATCHED_BANK_TRANSACTION":
      return i18nMsg(
        "settlement-cancel-blocked-bank-match",
        "Ce règlement est rapproché d'une transaction bancaire : annulez d'abord le rapprochement.",
      );
    case "ACCOUNT_ARCHIVED": {
      const base = i18nMsg(
        "settlement-cancel-blocked-account-archived",
        "Un compte de ce règlement a été archivé : réactivez-le pour pouvoir annuler le règlement.",
      );
      return label ? `${base} (${label})` : base;
    }
    case "FISCAL_YEAR_INVALID":
      return i18nMsg(
        "settlement-cancel-blocked-no-fiscal-year",
        "Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler ce règlement.",
      );
    default: {
      const exhaustive: never = code;
      return exhaustive;
    }
  }
}
