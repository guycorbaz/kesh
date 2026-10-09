/**
 * Les textes de la queue commune des motifs d'annulation — Story 25-3-a-1 (#414).
 *
 * ⚠️ Chaque test nomme la mutation qu'il attrape. Les replis sont comparés
 * **mot pour mot** au FTL fr-CH, qui est aussi le texte que le serveur rend au
 * refus : les deux ne doivent pas diverger.
 */
import { describe, it, expect, vi } from "vitest";

vi.mock("./i18n.svelte", () => ({
  i18nMsg: (_k: string, fallback: string) => fallback,
}));

import { settlementCancelTailMessage } from "./settlement-cancel-blocked";
import { invoiceSettlementCancelMessage } from "$lib/features/invoices/settlement-cancel";

describe("motifs d’annulation d’un règlement", () => {
  it("chaque code de la queue a son propre texte (mutation : deux codes confondus)", () => {
    const textes = new Set(
      (
        [
          "FISCAL_YEAR_CLOSED",
          "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
          "MATCHED_BANK_TRANSACTION",
          "ACCOUNT_ARCHIVED",
          "FISCAL_YEAR_INVALID",
        ] as const
      ).map((c) => settlementCancelTailMessage(c, null)),
    );
    expect(textes.size).toBe(5);
  });

  it("exercice clos : le chemin est la réouverture (mutation : texte générique)", () => {
    expect(settlementCancelTailMessage("FISCAL_YEAR_CLOSED", null)).toBe(
      "Ce règlement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.",
    );
  });

  it("lettrage figé par la période (Story 15-1a2-0) : son texte propre, et non celui de l'exercice clos (mutation : code renvoyé vers un autre texte)", () => {
    const texte = settlementCancelTailMessage(
      "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
      null,
    );
    expect(texte).toContain("toutes les lignes de ce lettrage");
    expect(texte).toContain("faire reculer le verrou avant elle");
    expect(texte).toContain("jusqu'à celui-ci, en commençant par le plus récent");
  });

  it("compte archivé : le NUMÉRO du compte accompagne le texte (mutation : libellé ignoré)", () => {
    expect(settlementCancelTailMessage("ACCOUNT_ARCHIVED", "1000")).toBe(
      "Un compte de ce règlement a été archivé : réactivez-le pour pouvoir annuler le règlement. (1000)",
    );
    expect(settlementCancelTailMessage("ACCOUNT_ARCHIVED", null)).not.toContain(
      "(",
    );
  });

  it("la tête client : facture créditée ⇒ règlement resté ouvert (mutation : renvoi à la queue)", () => {
    expect(invoiceSettlementCancelMessage("INVOICE_CREDITED", null)).toBe(
      "Cette facture a été créditée par un avoir : ce règlement reste ouvert au compte débiteurs, il ne s'annule pas.",
    );
  });

  it("la tête client délègue la queue sans la réécrire (mutation : texte dupliqué qui diverge)", () => {
    for (const c of [
      "FISCAL_YEAR_CLOSED",
      "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
      "MATCHED_BANK_TRANSACTION",
      "ACCOUNT_ARCHIVED",
      "FISCAL_YEAR_INVALID",
    ] as const) {
      expect(invoiceSettlementCancelMessage(c, "1000")).toBe(
        settlementCancelTailMessage(c, "1000"),
      );
    }
  });
});
