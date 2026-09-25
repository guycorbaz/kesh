/**
 * La tête fournisseur des motifs d'annulation — Story 25-3-a-2 (#414).
 */
import { describe, it, expect, vi } from "vitest";

vi.mock("$lib/shared/utils/i18n.svelte", () => ({
  i18nMsg: (_k: string, fallback: string) => fallback,
}));

import { supplierSettlementCancelMessage } from "./settlement-cancel";
import { settlementCancelTailMessage } from "$lib/shared/utils/settlement-cancel-blocked";

describe("motifs d’annulation d’un règlement fournisseur", () => {
  it("la tête : facture non payée (mutation : renvoi à la queue)", () => {
    expect(
      supplierSettlementCancelMessage("SUPPLIER_INVOICE_NOT_PAID", null),
    ).toBe(
      "Cette facture fournisseur n'est pas payée : il n'y a pas de règlement à annuler.",
    );
  });

  it("la queue est RÉUTILISÉE, sans jumeau (mutation : texte dupliqué qui diverge)", () => {
    for (const c of [
      "FISCAL_YEAR_CLOSED",
      "MATCHED_BANK_TRANSACTION",
      "ACCOUNT_ARCHIVED",
      "FISCAL_YEAR_INVALID",
    ] as const) {
      expect(supplierSettlementCancelMessage(c, "1000")).toBe(
        settlementCancelTailMessage(c, "1000"),
      );
    }
  });
});
