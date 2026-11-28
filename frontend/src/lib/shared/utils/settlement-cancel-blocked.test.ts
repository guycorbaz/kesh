/**
 * Les textes de la queue commune des motifs d'annulation — Story 25-3-a-1 (#414).
 *
 * ⚠️ Chaque test nomme la mutation qu'il attrape. Les replis sont comparés
 * **mot pour mot** au FTL fr-CH, qui est aussi le texte que le serveur rend au
 * refus : les deux ne doivent pas diverger.
 */
import { describe, it, expect, vi } from "vitest";

// Story 63-1 : les variables `{ $x }` sont interpolées, pour que la borne du
// verrou soit vérifiée en date suisse (patron de `reconciliation-cancel.test.ts`).
vi.mock("./i18n.svelte", () => ({
  i18nMsg: (
    _k: string,
    fallback: string,
    args?: Record<string, string | number>,
  ) =>
    args
      ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k]))
      : fallback,
}));

import {
  cancelRefusalMessage,
  settlementCancelTailMessage,
} from "./settlement-cancel-blocked";
import { invoiceSettlementCancelMessage } from "$lib/features/invoices/settlement-cancel";
import { supplierSettlementCancelMessage } from "$lib/features/supplier-invoices/settlement-cancel";

describe("motifs d’annulation d’un règlement", () => {
  it("T-a10 (Story 30-1a, AC-a6) — « rapproché » : un texte vrai sur la fiche fournisseur, qui n’a pas le dialogue, comme sur la fiche client (mutation : « depuis cette fiche » rétabli)", () => {
    const attendu =
      "Ce règlement est rapproché d'une transaction bancaire : annulez d'abord le rapprochement, depuis le détail de l'import bancaire — ou, pour une facture client, depuis sa fiche.";
    expect(
      supplierSettlementCancelMessage("MATCHED_BANK_TRANSACTION", null),
    ).toBe(attendu);
    expect(invoiceSettlementCancelMessage("MATCHED_BANK_TRANSACTION", null)).toBe(
      attendu,
    );
    expect(attendu).not.toContain("cette fiche");
  });

  it("chaque code de la queue a son propre texte (mutation : deux codes confondus)", () => {
    const textes = new Set(
      (
        [
          "FISCAL_YEAR_CLOSED",
          "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
          "MATCHED_BANK_TRANSACTION",
          "ACCOUNT_ARCHIVED",
          "PERIOD_LOCKED",
          "FISCAL_YEAR_INVALID",
        ] as const
      ).map((c) => settlementCancelTailMessage(c, null)),
    );
    expect(textes.size).toBe(6);
  });

  // Story 63-1 (#1752, #1753, AC-13, D5) — l'annulation depuis la fiche est
  // datée du règlement : sous la borne du verrou, la fiche l'annonce.
  it("période verrouillée : son texte, mot pour mot le FTL fr-CH, et la borne en date suisse dans une phrase à part (mutation M8 : clé ou texte échangé, borne ignorée ou accolée entre parenthèses)", () => {
    const base =
      "Cette écriture est dans une période verrouillée : pour pouvoir l'annuler, un administrateur doit faire reculer le verrou avant sa date.";
    expect(settlementCancelTailMessage("PERIOD_LOCKED", null)).toBe(base);
    expect(settlementCancelTailMessage("PERIOD_LOCKED", "2025-12-20")).toBe(
      `${base} La période est verrouillée jusqu'au 20.12.2025 inclus.`,
    );
    expect(settlementCancelTailMessage("PERIOD_LOCKED", "2025-12-20")).not.toContain(
      "(20.12.2025)",
    );
    // Partagé par les deux fiches, sans jumeau.
    expect(supplierSettlementCancelMessage("PERIOD_LOCKED", "2025-12-20")).toBe(
      settlementCancelTailMessage("PERIOD_LOCKED", "2025-12-20"),
    );
    expect(invoiceSettlementCancelMessage("PERIOD_LOCKED", "2025-12-20")).toBe(
      settlementCancelTailMessage("PERIOD_LOCKED", "2025-12-20"),
    );
  });

  // Revue P1 de la 63-1 (L4) — le refus au clic.
  it("cancelRefusalMessage : `PERIOD_LOCKED` dit le motif de la fiche avec la borne ; tout autre refus garde son message (mutation : `err.message` toujours rendu)", () => {
    const motif = (t: string | null) => settlementCancelTailMessage("PERIOD_LOCKED", t);
    const verrou = {
      code: "PERIOD_LOCKED",
      message: "Les écritures sont verrouillées jusqu'au 2025-12-31 ; …",
      details: { lockedThrough: "2025-12-31" },
    };
    expect(cancelRefusalMessage(verrou, motif)).toBe(motif("2025-12-31"));
    expect(cancelRefusalMessage(verrou, motif)).toContain("31.12.2025");
    // Sans borne jointe : le motif seul, jamais un texte à dates ISO.
    expect(cancelRefusalMessage({ ...verrou, details: undefined }, motif)).toBe(motif(null));
    expect(
      cancelRefusalMessage({ code: "FISCAL_YEAR_CLOSED", message: "tel quel" }, motif),
    ).toBe("tel quel");
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
