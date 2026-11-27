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
import type { TrialBalanceDto, TrialBalanceGroupedRow } from "./reports.types";

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

  it("revue P2 : un résidu en devise sous le demi-centime ne s'affiche pas « -0.00 », et s'arrondit", () => {
    const base = makeDto();
    const usd = {
      ...base.rows[0],
      accountId: 2,
      accountNumber: "1020",
      accountName: "Banque USD",
      currency: "USD",
      openingCurrency: "-0.0040",
      totalDebitCurrency: "100.0050",
      totalCreditCurrency: "91.0000",
      balanceCurrency: "9.0000",
      closingCurrency: "1309.0000",
    };
    render(TrialBalanceView, { dto: makeDto({ rows: [base.rows[0], usd] }) });
    const t = screen
      .getByTestId("tb-fx-row-1020")
      .textContent?.replace(/\s+/g, " ")
      .trim();
    expect(t).toContain("ouverture 0.00");
    expect(t).not.toContain("-0.00");
    expect(t).toContain("débit 100.01");
  });

  it("mono-devise : aucune ligne en devise", () => {
    const { container } = render(TrialBalanceView, { dto: makeDto() });
    expect(container.innerHTML).not.toContain("tb-fx-row");
    expect(screen.getAllByTestId("tb-row")).toHaveLength(1);
  });
});

// Story 46-6b — la balance par groupes : le composant partagé du bilan, à quatre
// colonnes ; résultat reporté après les groupes ; totaux et contrôles inchangés.
describe("TrialBalanceView — Story 46-6b, vue groupée", () => {
  const ligne = (
    kind: TrialBalanceGroupedRow["kind"],
    depth: number,
    accountId: number,
    accountNumber: string,
    accountName: string,
    m: [string, string, string, string],
    extra: Partial<TrialBalanceGroupedRow> = {},
  ): TrialBalanceGroupedRow => ({
    kind,
    depth,
    accountId,
    accountNumber,
    accountName,
    openingBalance: m[0],
    totalDebit: m[1],
    totalCredit: m[2],
    balance: "999",
    closingBalance: m[3],
    ...extra,
  });

  function groupee(
    rows: TrialBalanceGroupedRow[],
    position: "header" | "footer" = "header",
  ): TrialBalanceDto {
    return makeDto({
      grouping: {
        detailLevel: "all",
        subtotalPosition: position,
        maxDepth: 2,
        open: [],
        closed: [],
      },
      groupedRows: rows,
    });
  }

  const g10 = ligne("group", 1, 90, "10", "Actifs circulants", ["1000", "500", "0", "1500"], { open: true });
  const banque = ligne("account", 2, 1, "1100", "Banque", ["1000", "500", "0", "1500"], { active: true });

  const cellules = (el: HTMLElement) =>
    Array.from(el.querySelectorAll("td")).map((c) =>
      c.textContent?.replace(/\s+/g, " ").trim(),
    );

  it("rend les lignes groupées, quatre montants arrondis, en-tête et pied d'avant", () => {
    const { container } = render(TrialBalanceView, {
      dto: groupee([
        { ...g10, openingBalance: "1000.005" },
        banque,
      ]),
    });
    expect(screen.queryByTestId("tb-row")).toBeNull();
    const titre = cellules(screen.getByTestId("grouped-group-10"));
    expect(titre[1]).toBe("Actifs circulants");
    expect(titre.slice(2)).toEqual(["1’000.01", "500.00", "0.00", "1’500.00"]);
    expect(cellules(screen.getByTestId("grouped-account-1100")).slice(2)).toEqual([
      "1’000.00",
      "500.00",
      "0.00",
      "1’500.00",
    ]);
    // Le net des mouvements (`balance`) ne paraît nulle part.
    expect(container.textContent).not.toContain("999");
    expect(screen.getAllByRole("columnheader")).toHaveLength(6);
    expect(screen.getByTestId("tb-opening-check").textContent?.trim()).toBe("✓");
    expect(screen.getByTestId("tb-movements-check").textContent?.trim()).toBe("✓");
  });

  it("place le résultat reporté APRÈS les groupes, hors d'eux, « Perte reportée » si négatif", () => {
    render(TrialBalanceView, {
      dto: { ...groupee([g10, banque]), retainedEarnings: "-50" },
    });
    const lignes = Array.from(document.querySelectorAll("tbody tr")).map(
      (tr) => tr.getAttribute("data-testid"),
    );
    expect(lignes).toEqual(["grouped-group-10", "grouped-account-1100", "tb-retained"]);
    expect(screen.getByTestId("tb-retained").textContent).toContain("Perte reportée");
  });

  it("sous-totaux en pied : « Total <nom> », titre sans montant", () => {
    render(TrialBalanceView, {
      dto: groupee(
        [g10, banque, ligne("subtotal", 1, 90, "10", "Actifs circulants", ["1000", "500", "0", "1500"])],
        "footer",
      ),
    });
    expect(cellules(screen.getByTestId("grouped-group-10")).slice(2)).toEqual(["", "", "", ""]);
    const pied = cellules(screen.getByTestId("grouped-subtotal-10"));
    expect(pied[1]).toBe("Total Actifs circulants");
    expect(pied[5]).toBe("1’500.00");
  });

  it("seconde ligne en devise d'un compte et d'un groupe en devise unique", () => {
    const fx = {
      currency: "USD",
      openingCurrency: "10",
      totalDebitCurrency: "5",
      totalCreditCurrency: "0",
      balanceCurrency: "5",
      closingCurrency: "15",
    };
    render(TrialBalanceView, {
      dto: groupee([
        { ...g10, ...fx },
        { ...banque, ...fx },
      ]),
    });
    const attendu = "USD ouverture 10.00 · débit 5.00 · crédit 0.00 · clôture 15.00";
    expect(
      screen.getByTestId("tb-group-fx-row-group-10").textContent?.replace(/\s+/g, " ").trim(),
    ).toBe(attendu);
    expect(
      screen.getByTestId("tb-fx-row-1100").textContent?.replace(/\s+/g, " ").trim(),
    ).toBe(attendu);
    // Pas de solde en devise à côté du libellé, comme au bilan : la seconde ligne le porte.
    expect(screen.queryByTestId("group-fx-10")).toBeNull();
    expect(screen.queryByTestId("account-fx-1100")).toBeNull();
  });

  it("un compte archivé garde son badge", () => {
    render(TrialBalanceView, {
      dto: groupee([g10, { ...banque, active: false }]),
    });
    expect(screen.getByTestId("grouped-account-1100").textContent).toContain("archivé");
  });

  it("l'accordéon appelle onToggleGroup avec la ligne du groupe", async () => {
    const onToggleGroup = vi.fn();
    render(TrialBalanceView, { dto: groupee([g10, banque]), onToggleGroup });
    screen.getByTestId("group-toggle-10").click();
    expect(onToggleGroup).toHaveBeenCalledWith(expect.objectContaining({ accountId: 90 }));
  });
});

// Story 55-2 (AC-1, AC-16) — filet de non-régression du lien inchangé (passé par `AccountLink`).
describe("TrialBalanceView — Story 55-2, lien de compte", () => {
  it("lien exact, titre, classe LINK_CLASS", async () => {
    const { LINK_CLASS } = await import("$lib/shared/navigation/link-class");
    render(TrialBalanceView, { dto: makeDto() });
    const a = screen.getByRole("link", { name: "1100" });
    expect(a.getAttribute("href")).toBe(
      "/reports?tab=general-ledger&accountId=1&from=2026-01-01&to=2026-12-31",
    );
    expect(a.getAttribute("title")).toBe("Voir le détail dans le grand livre");
    expect(a.getAttribute("class")).toBe(LINK_CLASS);
  });
});
