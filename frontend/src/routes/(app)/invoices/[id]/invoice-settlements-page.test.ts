/**
 * La fiche facture, côté annulation d'un règlement — Story 25-3-a-1 (#414) —
 * et côté avoir sur une facture réglée — Story 25-4-a (#456).
 *
 * Ce que le composant de liste ne peut pas prouver seul : que la fiche
 * **confirme** avant d'envoyer, **relit** facture et règlements après succès, et
 * affiche un refus **dans le dialogue** sans effacer la fiche.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron : `contacts-page.test.ts`
 * — mocks hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import type {
  InvoiceResponse,
  InvoiceSettlementResponse,
} from "$lib/features/invoices/invoices.types";

vi.mock("$app/environment", () => ({ browser: true }));
vi.mock("$app/navigation", () => ({ goto: vi.fn() }));
vi.mock("$app/state", () => ({
  page: { params: { id: "5" }, url: new URL("http://localhost/invoices/5") },
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
  notifyMissingFiscalYearOrFallback: vi.fn(() => false),
}));
vi.mock("$lib/app/stores/auth.svelte", () => ({
  authState: { currentUser: { role: "Comptable", username: "c", userId: "1" } },
}));

const getInvoiceMock = vi.fn();
const listSettlementsMock = vi.fn();
const cancelSettlementMock = vi.fn();
vi.mock("$lib/features/invoices/invoices.api", () => ({
  getInvoice: (id: number) => getInvoiceMock(id),
  listInvoiceSettlements: (id: number) => listSettlementsMock(id),
  cancelInvoiceSettlement: (id: number, sid: number) =>
    cancelSettlementMock(id, sid),
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
const createCreditNoteMock = vi.fn();
vi.mock("$lib/features/credit-notes/credit-notes.api", () => ({
  createCreditNote: (req: unknown) => createCreditNoteMock(req),
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
    paidAt: "2026-03-05T00:00:00",
    emailedAt: null,
    emailedTo: null,
    projectId: null,
    dunningPausedAt: null,
    dunningPausedNote: null,
    isOverdue: false,
    amountSettled: "100.00",
    amountDue: "0.00",
    version: 3,
    createdAt: "2026-03-01T00:00:00",
    updatedAt: "2026-03-05T00:00:00",
    lines: [],
    ...partial,
  };
}

const reglement: InvoiceSettlementResponse = {
  id: 7,
  journalEntryId: 11,
  amount: "100.00",
  settledOn: "2026-03-05",
  settlementType: "internal_account",
  cancellable: true,
  cancelBlockedBy: null,
  cancelBlockedLabel: null,
  cancelBlockedDocumentId: null,
};

beforeEach(() => {
  getInvoiceMock.mockResolvedValue(invoice());
  listSettlementsMock.mockResolvedValueOnce([reglement]).mockResolvedValue([]);
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  listSettlementsMock.mockReset();
});

describe("fiche facture — annuler un règlement", () => {
  it("confirme, annule LE règlement nommé, puis relit facture et liste (mutation : pas de relecture)", async () => {
    cancelSettlementMock.mockResolvedValue({
      invoice: invoice({
        paidAt: null,
        amountSettled: "0.00",
        amountDue: "100.00",
        version: 4,
      }),
      reversalJournalEntryId: 12,
    });
    const { findByTestId, getByTestId, queryByTestId } = render(Page);

    await fireEvent.click(await findByTestId("invoice-settlement-cancel"));
    // Rien n'est envoyé avant la confirmation.
    expect(cancelSettlementMock).not.toHaveBeenCalled();
    await fireEvent.click(getByTestId("invoice-settlement-cancel-confirm"));

    await waitFor(() =>
      expect(cancelSettlementMock).toHaveBeenCalledWith(5, 7),
    );
    await waitFor(() => expect(listSettlementsMock).toHaveBeenCalledTimes(2));
    // La facture relue n'est plus soldée : le bouton de règlement revient.
    await waitFor(() => expect(queryByTestId("settle-open")).not.toBeNull());
    await waitFor(() =>
      expect(queryByTestId("invoice-settlements")).toBeNull(),
    );
  });

  it("un refus s’affiche DANS le dialogue, la fiche reste (mutation : message dans `errorMsg`)", async () => {
    cancelSettlementMock.mockRejectedValue({
      code: "FISCAL_YEAR_CLOSED",
      message:
        "Ce règlement appartient à un exercice clôturé : un administrateur doit rouvrir l'exercice pour pouvoir l'annuler.",
      status: 409,
    });
    const { findByTestId, getByTestId, findByText } = render(Page);

    await fireEvent.click(await findByTestId("invoice-settlement-cancel"));
    await fireEvent.click(getByTestId("invoice-settlement-cancel-confirm"));

    const erreur = await findByTestId("invoice-settlement-cancel-error");
    expect(erreur.textContent).toContain("rouvrir l");
    // La fiche est toujours là.
    expect(await findByText("F-2026-005", { exact: false })).toBeTruthy();
  });
});

describe("fiche facture — créer un avoir sur une facture réglée (Story 25-4-a)", () => {
  it("le bouton s'affiche sur une facture ENTIÈREMENT payée (mutation m6 : `!invoice.paidAt` remis)", async () => {
    const { findByTestId } = render(Page);
    expect(await findByTestId("invoice-credit-note-button")).toBeTruthy();
  });

  it("le bouton s'affiche sur une facture réglée EN PARTIE, sans `paidAt`", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ paidAt: null, amountSettled: "40.00", amountDue: "60.00" }),
    );
    const { findByTestId } = render(Page);
    expect(await findByTestId("invoice-credit-note-button")).toBeTruthy();
  });

  it("le refus du serveur s'affiche DANS le dialogue, avec son motif (le serveur tranche, l'écran le dit)", async () => {
    createCreditNoteMock.mockRejectedValue({
      code: "CREDIT_NOTE_INVOICE_SETTLED",
      message:
        "Cette facture porte un règlement, même partiel : annulez-le d'abord pour pouvoir émettre un avoir. (F-2026-005)",
      status: 409,
    });
    const { findByTestId, getByTestId } = render(Page);

    await fireEvent.click(await findByTestId("invoice-credit-note-button"));
    await fireEvent.click(getByTestId("invoice-credit-note-confirm"));

    await waitFor(() => expect(createCreditNoteMock).toHaveBeenCalledTimes(1));
    const erreur = await findByTestId("invoice-credit-note-error");
    expect(erreur.textContent).toContain("annulez-le d'abord");
  });

  // Story 25-4-c4-b (#494) — la ligne « Arrondi » du récapitulatif.
  it("une facture émise arrondie montre la ligne « Arrondi », signée (mutation : ligne retirée)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ roundingAmount: "0.01", roundingIsPreview: false, totalTtc: "123.45" }),
    );
    const { findByTestId } = render(Page);
    const row = await findByTestId("invoice-detail-rounding");
    expect(row.textContent).toContain("Arrondi");
    expect(row.textContent).not.toContain("estimé");
    expect(row.textContent).toContain("+0.01");
  });

  it("un brouillon montre l'aperçu « estimé » et un total qui l'inclut (mutation : total sans l'aperçu)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({
        status: "draft",
        roundingAmount: "-0.02",
        roundingIsPreview: true,
        totalTtc: "234.52",
      }),
    );
    const { findByTestId, container } = render(Page);
    const row = await findByTestId("invoice-detail-rounding");
    expect(row.textContent).toContain("Arrondi (estimé)");
    expect(row.textContent).toContain("-0.02");
    expect(container.textContent).toContain("Total TTC (estimé)");
    expect(container.textContent).toContain("234.50");
  });

  it("un écart infra-centime ne montre pas « +0.00 » (mutation : décision sur la valeur brute)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ roundingAmount: "0.0004", roundingIsPreview: false, totalTtc: "123.4500" }),
    );
    const { findByText, queryByTestId } = render(Page);
    expect(await findByText("F-2026-005", { exact: false })).toBeTruthy();
    expect(queryByTestId("invoice-detail-rounding")).toBeNull();
  });

  it("sans arrondi, aucune ligne « Arrondi »", async () => {
    const { findByText, queryByTestId } = render(Page);
    expect(await findByText("F-2026-005", { exact: false })).toBeTruthy();
    expect(queryByTestId("invoice-detail-rounding")).toBeNull();
  });
});
