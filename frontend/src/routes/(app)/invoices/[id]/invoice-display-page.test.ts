/**
 * La fiche facture cliente, côté affichage — Story v0171 (#866, #879).
 *
 * La quantité d'une ligne s'affiche lisible (`1.0000` → `1`, `readableAmount`) ; les dates
 * de la fiche (date, échéance, payée le, document figé le, envoyée le) en forme suisse
 * (`02.03.2026`, `formatSwissDate`). Mutations M4/M5/M7 : retirer le helper d'un site
 * rougit le seul test qui le nomme. Mocks recopiés de `invoice-pdf-page.test.ts`.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import type { InvoiceResponse } from "$lib/features/invoices/invoices.types";

vi.mock("$app/environment", () => ({ browser: true }));
vi.mock("$app/navigation", () => ({ goto: vi.fn() }));
vi.mock("$app/state", () => ({
  page: { params: { id: "5" }, url: new URL("http://localhost/invoices/5") },
}));
// ⚠️ Le mock honore un CATALOGUE, comme le vrai `i18nMsg` : une clé connue
// l'emporte sur le repli, et le catalogue est résolu sans arguments (revue P1,
// M1 — un mock qui rendait toujours le repli cachait un `{ $sha256 }` affiché).
const CATALOGUE: Record<string, string> = {
  "error-invoice-pdf-gone":
    "Le fichier \u2068{ $sha256 }\u2069.pdf manque dans le répertoire des documents.",
};
vi.mock("$lib/shared/utils/i18n.svelte", () => ({
  i18nMsg: (
    k: string,
    fallback: string,
    args?: Record<string, string | number>,
  ) => {
    const raw = CATALOGUE[k] ?? fallback;
    return args
      ? raw.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ""))
      : raw;
  },
}));
const notifyErrorMock = vi.hoisted(() => vi.fn());
vi.mock("$lib/shared/utils/notify", () => ({
  notifyError: (m: string) => notifyErrorMock(m),
  notifySuccess: vi.fn(),
  notifyWarning: vi.fn(),
  notifyMissingFiscalYearOrFallback: vi.fn(() => false),
}));

const auth = vi.hoisted(() => ({
  currentUser: { role: "Admin", username: "a", userId: "1" },
}));
vi.mock("$lib/app/stores/auth.svelte", () => ({ authState: auth }));

const getBlobMock = vi.hoisted(() => vi.fn());
vi.mock("$lib/shared/utils/api-client", async (importOriginal) => {
  const real =
    await importOriginal<typeof import("$lib/shared/utils/api-client")>();
  return {
    ...real,
    apiClient: { ...real.apiClient, getBlob: (url: string) => getBlobMock(url) },
  };
});

const getInvoiceMock = vi.fn();
const refreezeMock = vi.fn();
vi.mock("$lib/features/invoices/invoices.api", () => ({
  getInvoice: (id: number) => getInvoiceMock(id),
  refreezeInvoicePdf: (id: number) => refreezeMock(id),
  listInvoiceSettlements: vi.fn(async () => []),
  cancelInvoiceSettlement: vi.fn(),
  deleteInvoice: vi.fn(),
  validateInvoice: vi.fn(),
  unvalidateInvoice: vi.fn(),
  settleInvoice: vi.fn(),
  getInvoiceEmailPreview: vi.fn(),
  sendInvoiceEmail: vi.fn(),
  getInvoiceSettings: vi.fn(async () => ({})),
}));
vi.mock("$lib/features/accounts/accounts.api", () => ({
  fetchAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/bank-accounts/bank-accounts.api", () => ({
  listBankAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/reminders/reminders.api", () => ({
  listReminderHistory: vi.fn(async () => []),
  pauseDunning: vi.fn(),
  resumeDunning: vi.fn(),
}));
// Story 36-16a (revue P1, E-2) — la liste des comptes analytiques peut échouer.
const listProjectsMock = vi.hoisted(() => vi.fn(async () => [] as unknown[]));
vi.mock("$lib/features/projects/projects.api", () => ({
  listProjects: () => listProjectsMock(),
}));
const createCreditNoteMock = vi.hoisted(() => vi.fn());
vi.mock("$lib/features/credit-notes/credit-notes.api", () => ({
  createCreditNote: (req: unknown) => createCreditNoteMock(req),
}));

import Page from "./+page.svelte";
import { pinClock } from "$lib/shared/utils/pin-clock.test-helper";

function invoice(partial: Partial<InvoiceResponse> = {}): InvoiceResponse {
  return {
    id: 5,
    companyId: 1,
    contactId: 1,
    invoiceNumber: "F-2026-005",
    status: "validated",
    date: "2026-03-01",
    dueDate: "2026-03-31",
    paymentTerms: null,
    totalAmount: "100.00",
    totalTtc: "100.00",
    vatBreakdown: [],
    roundingAmount: "0",
    roundingIsPreview: false,
    journalEntryId: 9,
    paidAt: null,
    emailedAt: null,
    emailedTo: null,
    projectId: null,
    dunningPausedAt: null,
    dunningPausedNote: null,
    pdfFrozenAt: null,
    pdfLanguage: null,
    isOverdue: false,
    amountSettled: "0.00",
    amountDue: "100.00",
    version: 3,
    createdAt: "2026-03-01T00:00:00",
    updatedAt: "2026-03-05T00:00:00",
    lines: [],
    ...partial,
  };
}

beforeEach(() => {
  auth.currentUser.role = "Admin";
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function ligne(id: number, quantity: string) {
  return {
    id,
    invoiceId: 5,
    position: id,
    description: `Ligne ${id}`,
    quantity,
    unitPrice: "10.00",
    vatRate: "8.10",
    lineTotal: "10.00",
    revenueAccountId: null,
  };
}

describe("fiche facture — quantité (v0171, #866)", () => {
  it("`1.0000` ⇒ `1`, `2.5000` ⇒ `2.5`, `33.3333` gardé, `1250.0000` ⇒ `1250` (mutation M4)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({
        lines: [
          ligne(1, "1.0000"),
          ligne(2, "2.5000"),
          ligne(3, "33.3333"),
          ligne(4, "1250.0000"),
        ],
      } as unknown as Partial<InvoiceResponse>),
    );
    const { findAllByTestId, container } = render(Page);
    const cells = await findAllByTestId("invoice-line-quantity");
    expect(cells.map((c) => c.textContent?.trim())).toEqual([
      "1",
      "2.5",
      "33.3333",
      "1250",
    ]);
    expect(container.textContent).not.toContain("1.0000");
  });
});

describe("fiche facture — dates (v0171, #879)", () => {
  it("date `01.03.2026` (mutation M5 : site `invoice.date`)", async () => {
    getInvoiceMock.mockResolvedValue(invoice());
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("invoice-detail-date")).textContent?.trim()).toBe(
      "01.03.2026",
    );
    expect(container.textContent).not.toContain("2026-03-01");
  });

  it("échéance `31.03.2026` (mutation M5 : site `invoice.dueDate`)", async () => {
    getInvoiceMock.mockResolvedValue(invoice());
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("invoice-detail-due")).textContent?.trim()).toBe(
      "31.03.2026",
    );
    expect(container.textContent).not.toContain("2026-03-31");
  });

  it("échéance absente ⇒ `—`", async () => {
    getInvoiceMock.mockResolvedValue(invoice({ dueDate: null }));
    const { findByTestId } = render(Page);
    expect((await findByTestId("invoice-detail-due")).textContent?.trim()).toBe(
      "—",
    );
  });

  it("« Payée le » `10.04.2026` (mutation M5 : site `paidAt`)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ status: "paid" as never, paidAt: "2026-04-10T09:30:00" }),
    );
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("invoice-detail-paid-at")).textContent?.trim()).toBe(
      "10.04.2026",
    );
    expect(container.textContent).not.toContain("2026-04-10");
  });

  it("« Document figé le » `02.03.2026` (mutation M5 : site `pdfFrozenAt`)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("invoice-pdf-frozen-at")).textContent?.trim()).toBe(
      "02.03.2026",
    );
    expect(container.textContent).not.toContain("2026-03-02");
  });

  it("« Envoyée le » `03.03.2026` (mutation M7 : site `emailedAt`)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({
        emailedAt: "2026-03-03T11:00:00",
        emailedTo: "client@example.invalid",
      }),
    );
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("invoice-emailed-at")).textContent?.trim()).toBe(
      "03.03.2026",
    );
    expect(container.textContent).not.toContain("2026-03-03");
  });
});

// Story v0174-correctifs-3 (#889) — à 00 h 30 le 5 novembre à Zurich (23 h 30 UTC la
// veille) : l'avoir est daté du 5 (site 9) ; un instant UTC du 4 à 23 h 30 s'affiche à sa
// date suisse, le 5 (affichages b et c) ; une date de règlement (minuit) reste la sienne (a).
describe("fiche facture — la date du jour et les instants à l'heure suisse (#889)", () => {
  pinClock();

  it("l'avoir créé est daté du 5 novembre (site 9)", async () => {
    getInvoiceMock.mockResolvedValue(invoice());
    createCreditNoteMock.mockResolvedValue({ id: 77 });
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("invoice-credit-note-button"));
    await fireEvent.click(await findByTestId("invoice-credit-note-confirm"));
    await waitFor(() => expect(createCreditNoteMock).toHaveBeenCalled());
    expect(createCreditNoteMock.mock.calls[0][0]).toEqual({
      invoiceId: 5,
      date: "2026-11-05",
    });
  });

  it("« Document figé le » d'un instant à 23 h 30 UTC : `05.11.2026` (b)", async () => {
    getInvoiceMock.mockResolvedValue(invoice({ pdfFrozenAt: "2026-11-04T23:30:00" }));
    const { findByTestId } = render(Page);
    expect((await findByTestId("invoice-pdf-frozen-at")).textContent?.trim()).toBe(
      "05.11.2026",
    );
  });

  it("« Envoyée le » d'un instant à 23 h 30 UTC : `05.11.2026` (c)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ emailedAt: "2026-11-04T23:30:00", emailedTo: "client@example.invalid" }),
    );
    const { findByTestId } = render(Page);
    expect((await findByTestId("invoice-emailed-at")).textContent?.trim()).toBe(
      "05.11.2026",
    );
  });

  it("« Payée le » d'une date à minuit reste `04.11.2026` (a, conforme)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ status: "paid" as never, paidAt: "2026-11-04T00:00:00" }),
    );
    const { findByTestId } = render(Page);
    expect((await findByTestId("invoice-detail-paid-at")).textContent?.trim()).toBe(
      "04.11.2026",
    );
  });
});

// Story 36-16a (#1167, AC12, D13) — « Changer le compte analytique » : Comptable et
// Administrateur, hors brouillon (validée, payée, annulée) ; ni le rôle Lecture, ni le
// brouillon (qui a son formulaire). Mutation : bouton montré au rôle Lecture.
describe("fiche facture — bouton « Changer le compte analytique » (36-16a)", () => {
  const bouton = "invoice-reassign-project-button";

  it("validée, payée, annulée : présent pour l'Administrateur et le Comptable", async () => {
    for (const role of ["Admin", "Comptable"]) {
      for (const partial of [
        {},
        { paidAt: "2026-03-10T00:00:00" },
        { status: "cancelled" as const },
      ]) {
        auth.currentUser.role = role;
        getInvoiceMock.mockResolvedValue(invoice(partial as Partial<InvoiceResponse>));
        const r = render(Page);
        expect(await r.findByTestId(bouton)).toBeTruthy();
        cleanup();
      }
    }
  });

  /**
   * La fiche rendue ET la liste des comptes analytiques réglée (chargée ou en échec) : le bouton
   * dépend de cette liste (revue P1, E-2), si bien qu'une absence constatée avant elle ne prouverait
   * rien — c'est ainsi que la mutation F4 (bouton au rôle Lecture) a survécu une fois.
   */
  async function ficheReglee(r: ReturnType<typeof render>) {
    await r.findByTestId("invoice-detail-date");
    await waitFor(() => expect(listProjectsMock).toHaveBeenCalled());
    await new Promise((resolve) => setTimeout(resolve, 0));
  }

  it("rôle Lecture : pas de bouton", async () => {
    auth.currentUser.role = "Consultation";
    getInvoiceMock.mockResolvedValue(invoice());
    const r = render(Page);
    await ficheReglee(r);
    expect(r.queryByTestId(bouton)).toBeNull();
  });

  // Revue P1, E-2 — mutation : bouton montré sans la liste (le sélecteur dirait « — Aucun »).
  it("comptes analytiques non chargés : pas de bouton", async () => {
    listProjectsMock.mockRejectedValueOnce(new Error("réseau"));
    getInvoiceMock.mockResolvedValue(invoice({ projectId: 3 }));
    const r = render(Page);
    await ficheReglee(r);
    expect(r.queryByTestId(bouton)).toBeNull();
  });

  it("brouillon : pas de bouton", async () => {
    getInvoiceMock.mockResolvedValue(invoice({ status: "draft", journalEntryId: null }));
    const r = render(Page);
    await ficheReglee(r);
    expect(r.queryByTestId(bouton)).toBeNull();
  });
});
