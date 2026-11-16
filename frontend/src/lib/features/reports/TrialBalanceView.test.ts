// Story 25-5-b (#385) — tests Vitest pour TrialBalanceView.svelte : ouverture,
// mouvements, clôture, la ligne calculée du résultat reporté et le contrôle
// d'ouverture. i18nMsg est mocké pour renvoyer le fallback.

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";

vi.mock("$lib/shared/utils/i18n.svelte", () => ({
  // Le repli, paramètres substitués (Story 37-4).
  i18nMsg: (
    _key: string,
    fallback: string,
    args?: Record<string, string | number>,
  ) =>
    fallback.replace(/\{ \$(\w+) \}/g, (_: string, k: string) =>
      String(args?.[k] ?? ""),
    ),
}));

import TrialBalanceView from "./TrialBalanceView.svelte";
import type { TrialBalanceDto } from "./reports.types";

function makeDto(overrides: Partial<TrialBalanceDto> = {}): TrialBalanceDto {
  return {
    period: { fiscalYearId: 1, startDate: "2026-01-01", endDate: "2026-12-31" },
    rows: [
      {
        accountId: 1,
        accountNumber: "1100",
        accountName: "Banque",
        accountType: "Asset",
        active: true,
        openingBalance: "1000",
        totalDebit: "500",
        totalCredit: "0",
        balance: "500",
        closingBalance: "1500",
      },
    ],
    totalDebit: "500",
    totalCredit: "500",
    balanced: true,
    retainedEarnings: "600",
    openingBalanced: true,
    ...overrides,
  };
}

describe("TrialBalanceView — Story 25-5-b", () => {
  it("affiche les colonnes Ouverture et Clôture, sans « Solde » ni la note retirée", () => {
    render(TrialBalanceView, { dto: makeDto() });
    const headers = screen
      .getAllByRole("columnheader")
      .map((h) => h.textContent?.trim());
    expect(headers).toEqual([
      "N°",
      "Intitulé",
      "Ouverture",
      "Débit",
      "Crédit",
      "Clôture",
    ]);
    expect(screen.queryByText(/mouvement de la période/)).toBeNull();
  });

  it("porte l’ouverture et la clôture de chaque compte, dans l’ordre des colonnes", () => {
    render(TrialBalanceView, { dto: makeDto() });
    const cells = Array.from(
      screen.getByTestId("tb-row").querySelectorAll("td"),
    ).map((c) => c.textContent?.replace(/\s+/g, " ").trim());
    expect(cells[2]).toMatch(/1.?000\.00/);
    expect(cells[5]).toMatch(/1.?500\.00/);
  });

  it("rend la ligne calculée du résultat reporté (ouverture = clôture)", () => {
    render(TrialBalanceView, { dto: makeDto() });
    const row = screen.getByTestId("tb-retained");
    expect(row.textContent).toContain("Résultat reporté (calculé)");
    const cells = Array.from(row.querySelectorAll("td")).map((c) =>
      c.textContent?.trim(),
    );
    expect(cells[2]).toBe(cells[5]);
    expect(cells[3]).toBe("");
  });

  it("bascule sur « Perte reportée » quand le résultat reporté est négatif", () => {
    render(TrialBalanceView, { dto: makeDto({ retainedEarnings: "-800" }) });
    expect(screen.getByTestId("tb-retained").textContent).toContain(
      "Perte reportée",
    );
  });

  it("signale une ouverture déséquilibrée, indépendamment des mouvements", () => {
    render(TrialBalanceView, { dto: makeDto({ openingBalanced: false }) });
    expect(screen.getByTestId("tb-opening-check").textContent?.trim()).toBe(
      "⚠️",
    );
    expect(screen.getByTestId("tb-movements-check").textContent?.trim()).toBe(
      "✓",
    );
  });

  it("affiche ✓ quand l’ouverture s’équilibre", () => {
    render(TrialBalanceView, { dto: makeDto() });
    expect(screen.getByTestId("tb-opening-check").textContent?.trim()).toBe(
      "✓",
    );
  });
});

describe("TrialBalanceView — Story 37-4, compte en devise", () => {
  it("met une seconde ligne en devise sous un compte en devise, sans total en devise", () => {
    const base = makeDto();
    const usd = {
      ...base.rows[0],
      accountId: 2,
      accountNumber: "1020",
      accountName: "Banque USD",
      currency: "USD",
      openingCurrency: "1300.0000",
      totalDebitCurrency: "100.0000",
      totalCreditCurrency: "91.0000",
      balanceCurrency: "9.0000",
      closingCurrency: "1309.0000",
    };
    const { container } = render(TrialBalanceView, {
      dto: makeDto({ rows: [base.rows[0], usd] }),
    });
    expect(
      screen.getByTestId("tb-fx-row-1020").textContent?.replace(/\s+/g, " ").trim(),
    ).toBe(
      "USD ouverture 1’300.00 · débit 100.00 · crédit 91.00 · clôture 1’309.00",
    );
    expect(screen.queryByTestId("tb-fx-row-1100")).toBeNull();
    // Le pied reste en devise principale.
    expect(container.querySelector("tfoot")?.textContent).not.toContain("USD");
  });

  it("mono-devise : aucune ligne en devise", () => {
    const { container } = render(TrialBalanceView, { dto: makeDto() });
    expect(container.innerHTML).not.toContain("tb-fx-row");
    expect(screen.getAllByTestId("tb-row")).toHaveLength(1);
  });
});
