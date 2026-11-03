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
vi.mock("$lib/features/projects/projects.api", () => ({
  listProjects: vi.fn(async () => []),
}));
vi.mock("$lib/features/credit-notes/credit-notes.api", () => ({
  createCreditNote: vi.fn(),
}));

import Page from "./+page.svelte";

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
