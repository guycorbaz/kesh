/**
 * La fiche facture fournisseur, côté affichage — Story v0171 (#866, #871).
 *
 * La quantité d'une ligne s'affiche lisible (`1.0000` → `1`, `readableAmount`) et les
 * dates de la fiche en forme suisse (`29.01.2026`, `formatSwissDate`). Mutations : retirer
 * le helper d'un site rougit le test qui le nomme. Mocks recopiés de
 * `supplier-settlement-page.test.ts`.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import type { SupplierInvoiceResponse } from "$lib/features/supplier-invoices/supplier-invoices.types";

vi.mock("$app/navigation", () => ({ goto: vi.fn() }));
vi.mock("$app/state", () => ({
  page: {
    params: { id: "9" },
    url: new URL("http://localhost/supplier-invoices/9"),
  },
}));
vi.mock("$lib/shared/utils/i18n.svelte", () => ({
  i18nMsg: (
    _k: string,
    fallback: string,
    args?: Record<string, string | number>,
  ) =>
    args
      ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ""))
      : fallback,
}));
vi.mock("$lib/shared/utils/notify", () => ({
  notifyError: vi.fn(),
  notifySuccess: vi.fn(),
  notifyWarning: vi.fn(),
  notifyInfo: vi.fn(),
}));
const role = { value: "Comptable" };
vi.mock("$lib/app/stores/auth.svelte", () => ({
  authState: {
    get currentUser() {
      return { role: role.value, username: "c", userId: "1" };
    },
  },
}));

const getMock = vi.fn();
const payMock = vi.fn();
const cancelSettlementMock = vi.fn();
const cancelInvoiceMock = vi.fn();
vi.mock("$lib/features/supplier-invoices/supplier-invoices.api", () => ({
  getSupplierInvoice: (id: number) => getMock(id),
  paySupplierInvoice: (id: number, req: unknown) => payMock(id, req),
  cancelSupplierInvoice: (id: number) => cancelInvoiceMock(id),
  cancelSupplierInvoiceSettlement: (id: number) => cancelSettlementMock(id),
}));
vi.mock("$lib/features/bank-accounts/bank-accounts.api", () => ({
  listBankAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/accounts/accounts.api", () => ({
  fetchAccounts: vi.fn(async () => [
    { id: 1, number: "1000", name: "Caisse", active: true, postable: true },
  ]),
}));
vi.mock("$lib/features/projects/projects.api", () => ({
  listProjects: vi.fn(async () => []),
}));
vi.mock(
  "$lib/features/imported-supplier-invoices/imported-supplier-invoices.api",
  () => ({
    downloadSupplierInvoiceSourceDocument: vi.fn(),
    // Story 30-2c — la fiche joint, détache et prévisualise le document ; ces
    // tests-ci ne touchent pas au document (`sourceDocument` absent : rien
    // n'est rendu, rien n'est appelé).
    fetchSupplierInvoiceSourceDocument: vi.fn(),
    attachSourceDocument: vi.fn(),
    detachSourceDocument: vi.fn(),
    listImported: vi.fn(async () => []),
    fetchImportedSourceDocument: vi.fn(),
  }),
);

import Page from "./+page.svelte";

function inv(
  partial: Partial<SupplierInvoiceResponse> = {},
): SupplierInvoiceResponse {
  return {
    id: 9,
    contactId: 1,
    supplierInvoiceNumber: "FF-9",
    internalNumber: null,
    status: "paid",
    invoiceDate: "2026-06-15",
    dueDate: null,
    totalAmount: "100.00",
    creditorIban: null,
    creditorQrIban: null,
    paymentReference: null,
    expectedPaymentAmount: null,
    projectId: null,
    purchaseJournalEntryId: 4,
    settlementType: "internal_account",
    settlementBankAccountId: null,
    settlementAccountId: 1,
    settlementJournalEntryId: 5,
    paidAt: "2026-06-20T00:00:00",
    version: 2,
    createdAt: "2026-06-15T00:00:00",
    lines: [],
    settlementCancellable: true,
    settlementCancelBlockedBy: null,
    settlementCancelBlockedLabel: null,
    lastConfirmedBatch: null,
    cancellable: true,
    cancelBlockedBy: null,
    cancelBlockedLabel: null,
    ...partial,
  };
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function line(quantity: string, position = 1) {
  return {
    position,
    description: `Ligne ${position}`,
    quantity,
    unitPrice: "10.00",
    vatRate: "8.10",
    lineTotal: "10.00",
    expenseAccountId: 1,
  };
}

describe("fiche facture fournisseur — affichage (v0171)", () => {
  it("quantité `1.0000` ⇒ `1` ; `2.5000` ⇒ `2.5` ; `33.3333` gardé ; `1250.0000` ⇒ `1250` (mutation M4 : readableAmount retiré)", async () => {
    getMock.mockResolvedValue(
      inv({
        status: "open",
        lines: [
          line("1.0000", 1),
          line("2.5000", 2),
          line("33.3333", 3),
          line("1250.0000", 4),
        ],
      }),
    );
    const { findAllByTestId, container } = render(Page);
    const cells = await findAllByTestId("supplier-invoice-detail-line-quantity");
    expect(cells.map((c) => c.textContent?.trim())).toEqual([
      "1",
      "2.5",
      "33.3333",
      "1250",
    ]);
    expect(container.textContent).not.toContain("1.0000");
    expect(container.textContent).not.toContain("2.5000");
  });

  it("date de facture et échéance en forme suisse (mutation M5 : formatSwissDate retiré de `invoiceDate`)", async () => {
    getMock.mockResolvedValue(
      inv({ status: "open", invoiceDate: "2026-01-29", dueDate: "2026-02-28" }),
    );
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("supplier-invoice-detail-date")).textContent?.trim()).toBe(
      "29.01.2026",
    );
    expect(container.textContent).not.toContain("2026-01-29");
  });

  it("échéance en forme suisse (mutation M5 : formatSwissDate retiré de `dueDate`)", async () => {
    getMock.mockResolvedValue(
      inv({ status: "open", invoiceDate: "2026-01-29", dueDate: "2026-02-28" }),
    );
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("supplier-invoice-detail-due")).textContent?.trim()).toBe(
      "28.02.2026",
    );
    expect(container.textContent).not.toContain("2026-02-28");
  });

  it("échéance absente ⇒ `—`", async () => {
    getMock.mockResolvedValue(inv({ status: "open", dueDate: null }));
    const { findByTestId } = render(Page);
    expect((await findByTestId("supplier-invoice-detail-due")).textContent?.trim()).toBe("—");
  });
});
