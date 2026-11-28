/**
 * Ce qui empêche d'annuler un règlement — la **queue commune** (Story 25-3-a-1, #414).
 *
 * ⛔ **Partagé entre le client et le fournisseur.** Ces cinq motifs tiennent
 * à l'**écriture** de règlement (son exercice, le lettrage de pièce figé par la
 * période — rang 2 bis, Story 15-1a2-0 —, son rapprochement, ses comptes,
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
 *
 * Story 63-1 (#1752, #1753, #1754) : les annulations depuis la fiche sont datées
 * de l'écriture d'origine ; la queue annonce donc la **période verrouillée**
 * (`PERIOD_LOCKED`, borne du verrou en étiquette), et le rang « aucun exercice
 * ouvert au jour » (`FISCAL_YEAR_INVALID`) n'est plus produit par la fiche — son
 * texte reste, la variante aussi (issue #1759).
 */
// ⚠️ Le seul formateur de date suisse du frontend vit dans `lib/shared/utils/swiss-date.ts`
// (`reports.api` le ré-exporte) : la couche partagée ne remonte pas vers `features/`.
import { formatSwissDate } from "./swiss-date";
import { i18nMsg } from "./i18n.svelte";

/** Les codes de la queue commune, dans l'ordre de précédence du serveur. */
export type SettlementCancelTailCode =
  | "FISCAL_YEAR_CLOSED"
  // Story 15-1a2-0 (#518) — rang 2 bis : le lettrage de pièce figé par la période.
  | "LETTERING_ALL_LINES_IN_CLOSED_PERIODS"
  | "MATCHED_BANK_TRANSACTION"
  | "ACCOUNT_ARCHIVED"
  // Story 63-1 — l'écriture est sous la borne du verrou de période.
  | "PERIOD_LOCKED"
  // Inatteignable depuis la fiche depuis la Story 63-1 ; gardé (issue #1759).
  | "FISCAL_YEAR_INVALID";

/**
 * La phrase qui nomme la **borne** du verrou de période (`AAAA-MM-JJ`), en date
 * suisse — ajoutée à part au motif `PERIOD_LOCKED`, qui ne porte pas de variable
 * (le serveur rend la clé telle quelle). Accolée entre parenthèses après « avant
 * sa date. », elle se lirait comme la date de l'écriture (leçon de la revue P1
 * de la 53-6a, L1). Partagée par la queue des règlements et par l'annulation
 * d'une facture fournisseur (`features/supplier-invoices/invoice-cancel.ts`).
 */
export function periodLockedThrough(base: string, label: string | null): string {
  if (!label) return base;
  const through = i18nMsg(
    "settlement-cancel-blocked-period-locked-through",
    "La période est verrouillée jusqu'au { $date } inclus.",
    { date: formatSwissDate(label) },
  );
  return `${base} ${through}`;
}

/**
 * Le texte d'un refus **au clic** d'une annulation (revue P1 de la 63-1, L4). Sur un
 * `400 PERIOD_LOCKED` — le verrou posé entre la lecture de la fiche et le clic —, le
 * serveur rend le texte générique des écritures (`journal-entries-period-locked`, dates
 * ISO, sans remède) : l'écran dit à la place **le motif de la fiche**, avec la borne
 * (`details.lockedThrough`) en date suisse. Tout autre refus garde son message.
 * `motif` donne le texte de la fiche pour la borne (`null` si le serveur ne l'a pas jointe).
 */
export function cancelRefusalMessage(
  err: { code: string; message: string; details?: Record<string, unknown> },
  motif: (lockedThrough: string | null) => string,
): string {
  if (err.code !== "PERIOD_LOCKED") return err.message;
  const through = err.details?.lockedThrough;
  return motif(typeof through === "string" ? through : null);
}

/**
 * Le texte d'un motif de la queue commune. `label` porte le **numéro du
 * compte** archivé — sans lui, « réactivez-le » ne dirait pas lequel —, ou la
 * **borne** du verrou de période (`PERIOD_LOCKED`), ajoutée en date suisse.
 */
export function settlementCancelTailMessage(
  code: SettlementCancelTailCode,
  label: string | null,
): string {
  switch (code) {
    case "FISCAL_YEAR_CLOSED":
      return i18nMsg(
        "settlement-cancel-blocked-fiscal-year-closed",
        "Ce règlement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.",
      );
    case "LETTERING_ALL_LINES_IN_CLOSED_PERIODS":
      return i18nMsg(
        "settlement-cancel-blocked-lettering-closed",
        "Ce règlement est lettré avec sa facture, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du dernier règlement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.",
      );
    // Story 30-1a (#686, AC-a6) — un texte vrai sur les DEUX fiches : la fiche
    // fournisseur n'a pas le dialogue « Annuler le rapprochement ».
    case "MATCHED_BANK_TRANSACTION":
      return i18nMsg(
        "settlement-cancel-blocked-bank-match",
        "Ce règlement est rapproché d'une transaction bancaire : annulez d'abord le rapprochement, depuis le détail de l'import bancaire — ou, pour une facture client, depuis sa fiche.",
      );
    case "ACCOUNT_ARCHIVED": {
      const base = i18nMsg(
        "settlement-cancel-blocked-account-archived",
        "Un compte de ce règlement a été archivé : réactivez-le pour pouvoir annuler le règlement.",
      );
      return label ? `${base} (${label})` : base;
    }
    case "PERIOD_LOCKED":
      return periodLockedThrough(
        i18nMsg(
          "settlement-cancel-blocked-period-locked",
          "Cette écriture est dans une période verrouillée : pour pouvoir l'annuler, un administrateur doit faire reculer le verrou avant sa date.",
        ),
        label,
      );
    // Inatteignable depuis la fiche depuis la Story 63-1 (datée de l'origine,
    // dont l'exercice, ouvert, couvre la date) ; clé et repli gardés (#1759).
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
