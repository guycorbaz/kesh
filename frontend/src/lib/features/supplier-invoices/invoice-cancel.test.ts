/**
 * Les motifs d'annulation d'une facture fournisseur — Story 15-1a2-0 (#518).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

// Story 63-1 : les variables `{ $x }` sont interpolées (la borne du verrou).
vi.mock("$lib/shared/utils/i18n.svelte", () => ({
  i18nMsg: (
    _k: string,
    fallback: string,
    args?: Record<string, string | number>,
  ) =>
    args
      ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k]))
      : fallback,
}));

import { supplierInvoiceCancelMessage } from "./invoice-cancel";

/** La valeur fr-CH d'une clé sur une seule ligne du catalogue. */
function valeurFr(cle: string): string | undefined {
  const texte = readFileSync(
    join("../crates/kesh-i18n/locales", "fr-CH", "messages.ftl"),
    "utf-8",
  );
  const prefixe = `${cle} = `;
  return texte
    .split("\n")
    .find((l) => l.startsWith(prefixe))
    ?.slice(prefixe.length);
}

describe("le lettrage figé par la période (rang 2 bis)", () => {
  it("le motif a son texte, mot pour mot le FTL fr-CH (mutation : repli divergent, ou code renvoyé vers le texte de l'exercice clos)", () => {
    const attendu = valeurFr("supplier-invoices-cancel-blocked-lettering-closed");
    expect(attendu).toBeDefined();
    expect(
      supplierInvoiceCancelMessage("LETTERING_ALL_LINES_IN_CLOSED_PERIODS", null),
    ).toBe(attendu);
    expect(
      supplierInvoiceCancelMessage("LETTERING_ALL_LINES_IN_CLOSED_PERIODS", null),
    ).not.toBe(supplierInvoiceCancelMessage("FISCAL_YEAR_CLOSED", null));
  });
});

// Story v017-4b (#849, AC-25) — le serveur rend `SUPPLIER_INVOICE_DRAFT` sur le GET d'un
// brouillon (AC-5 (7)) : l'union le porte, avec le texte posé par la 4a (AC-13, clé
// `supplier-invoices-cancel-blocked-draft`). Mutation : le `case` renvoyé vers un autre
// texte, ou repli divergent du catalogue.
describe("le brouillon (rang 1, Story v017-4a)", () => {
  it("le motif a le texte de la 4a, mot pour mot le FTL fr-CH", () => {
    const attendu = valeurFr("supplier-invoices-cancel-blocked-draft");
    expect(attendu).toBeDefined();
    expect(supplierInvoiceCancelMessage("SUPPLIER_INVOICE_DRAFT", null)).toBe(attendu);
  });
});

// Story 63-1 (#1754, AC-13, D5) — l'annulation d'une facture est datée de l'écriture
// d'achat : sous la borne du verrou, la fiche le dit à la place du bouton. Mutation M8 :
// clé échangée (texte de la queue du règlement, ou du rapprochement), borne ignorée.
describe("la période verrouillée (Story 63-1)", () => {
  it("le motif a son texte, mot pour mot le FTL fr-CH, qui dit « l'écriture d'achat » et non « ce règlement »", () => {
    const attendu = valeurFr("supplier-invoices-cancel-blocked-period-locked");
    expect(attendu).toBe(
      "L'écriture d'achat de cette facture est dans une période verrouillée : pour pouvoir l'annuler, un administrateur doit faire reculer le verrou avant sa date.",
    );
    expect(supplierInvoiceCancelMessage("PERIOD_LOCKED", null)).toBe(attendu);
  });

  it("la borne s'ajoute en date suisse, dans une phrase à part", () => {
    const texte = supplierInvoiceCancelMessage("PERIOD_LOCKED", "2025-12-15");
    expect(texte).toContain("L'écriture d'achat de cette facture est dans une période verrouillée");
    expect(texte).toContain("verrouillée jusqu'au 15.12.2025 inclus.");
    expect(texte).not.toContain("(15.12.2025)");
  });
});
