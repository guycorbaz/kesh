/**
 * L'échéancier porte le reste dû — Story 25-4-b1 (#416).
 *
 * Ce que ni le test Rust ni l'E2E ne prouvent seuls : que la page **affiche**
 * le reste dû de la ligne, rend le statut « partiellement payée », et **passe**
 * ce reste dû au dialogue de règlement, qui le pré-remplit.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `invoices/[id]/invoice-settlements-page.test.ts` — mocks hoistés AVANT
 * l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import type {
  DueDateItem,
  DueDatesResponse,
} from "$lib/features/invoices/invoices.types";

vi.mock("$app/environment", () => ({ browser: true }));
vi.mock("$app/navigation", () => ({ goto: vi.fn() }));
vi.mock("$app/state", () => ({
  page: {
    params: {},
    url: new URL("http://localhost/invoices/due-dates"),
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
}));
vi.mock("$lib/app/stores/auth.svelte", () => ({
  authState: { currentUser: { role: "Comptable", username: "c", userId: "1" } },
}));

const listDueDatesMock = vi.fn();
vi.mock("$lib/features/invoices/invoices.api", () => ({
  listDueDates: (q: unknown) => listDueDatesMock(q),
  settleInvoice: vi.fn(),
  exportDueDatesCsv: vi.fn(),
}));
vi.mock("$lib/features/accounts/accounts.api", () => ({
  fetchAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/bank-accounts/bank-accounts.api", () => ({
  listBankAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/contacts/contacts.api", () => ({
  getContact: vi.fn(),
  listContacts: vi.fn(async () => ({ items: [], total: 0, offset: 0, limit: 20 })),
}));

import Page from "./+page.svelte";

function item(partial: Partial<DueDateItem> = {}): DueDateItem {
  return {
    id: 5,
    companyId: 1,
    contactId: 1,
    contactName: "Client SA",
    invoiceNumber: "F-2026-005",
    status: "validated",
    date: "2026-03-01",
    dueDate: "2026-03-31",
    paymentTerms: null,
    totalAmount: "100.00",
    totalTtc: "108.10",
    amountSettled: "40.00",
    amountDue: "68.10",
    paidAt: null,
    dunningPausedAt: null,
    dunningPausedNote: null,
    isOverdue: false,
    version: 3,
    createdAt: "2026-03-01T00:00:00",
    updatedAt: "2026-03-05T00:00:00",
    ...partial,
  };
}

function response(items: DueDateItem[]): DueDatesResponse {
  return {
    items,
    total: items.length,
    offset: 0,
    limit: 20,
    summary: {
      unpaidCount: items.length,
      unpaidTotal: "68.10",
      overdueCount: 0,
      overdueTotal: "0.00",
    },
  };
}

beforeEach(() => {
  listDueDatesMock.mockResolvedValue(response([item()]));
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("échéancier — le reste dû (Story 25-4-b1)", () => {
  it("la ligne affiche le reste dû, pas le seul TTC (mutation : colonne retirée)", async () => {
    const { findByTestId } = render(Page);
    const cell = await findByTestId("due-dates-amount-due");
    expect(cell.textContent).toContain("68.10");
  });

  it("un reste à fraction de centime s'affiche aux quatre décimales, pas « 0.00 » (Story 25-4-d2b ; mutation : `formatInvoiceTotal`)", async () => {
    listDueDatesMock.mockResolvedValue(
      response([item({ amountSettled: "9.9960", amountDue: "0.0040" })]),
    );
    const { findByTestId } = render(Page);
    expect((await findByTestId("due-dates-amount-due")).textContent).toContain("0.0040");
  });

  it("une facture réglée en partie est « partiellement payée » (mutation : `partial` retiré du statut)", async () => {
    const { findByText } = render(Page);
    expect(await findByText("Partiellement payée")).toBeTruthy();
  });

  it("une facture sans règlement reste « impayée »", async () => {
    listDueDatesMock.mockResolvedValue(
      response([item({ amountSettled: "0.00", amountDue: "108.10" })]),
    );
    const { findByText, queryByText } = render(Page);
    expect(await findByText("Impayée")).toBeTruthy();
    expect(queryByText("Partiellement payée")).toBeNull();
  });

  it("le dialogue de règlement est pré-rempli au reste dû (mutation : `amountDue={null}` remis)", async () => {
    const { findByText, container } = render(Page);
    await fireEvent.click(await findByText("Régler"));
    await waitFor(() => {
      const input = document.getElementById("settle-amount") as HTMLInputElement | null;
      expect(input?.value).toBe("68.10");
    });
    expect(container).toBeTruthy();
  });
});
