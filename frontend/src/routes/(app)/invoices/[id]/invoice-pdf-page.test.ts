/**
 * La fiche facture, côté PDF émis — Story 25-6-b (#387).
 *
 * Ce que l'écran doit tenir : le bouton PDF d'une facture **annulée** n'existe
 * que si son document a été figé ; la mention « Document figé le » ; et, sur un
 * 410 `INVOICE_PDF_GONE`, le refigeage réservé à l'administrateur, qui
 * **confirme** avant d'envoyer.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `invoice-settlements-page.test.ts` — mocks hoistés AVANT l'import du composant.
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

const gone = {
  code: "INVOICE_PDF_GONE",
  message: "Le fichier abc123.pdf manque dans le répertoire des documents.",
  status: 410,
};

beforeEach(() => {
  auth.currentUser.role = "Admin";
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

describe("fiche facture — bouton PDF d'une facture annulée", () => {
  it("présent si le document a été figé (mutation : condition `validated` seule)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ status: "cancelled", pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    const { findByTestId } = render(Page);
    expect(await findByTestId("invoice-download-pdf")).toBeTruthy();
  });

  it("absent si elle n'a jamais été figée (mutation : `cancelled` sans condition de gel)", async () => {
    getInvoiceMock.mockResolvedValue(invoice({ status: "cancelled" }));
    const { findByTestId, queryByTestId } = render(Page);
    await findByTestId("invoice-detail");
    expect(queryByTestId("invoice-download-pdf")).toBeNull();
  });
});

describe("fiche facture — mention du gel", () => {
  it("« Document figé le » avec la date, quand le PDF est figé", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    const { findByTestId } = render(Page);
    expect((await findByTestId("invoice-pdf-frozen-at")).textContent).toBe(
      "2026-03-02",
    );
  });

  it("aucune mention quand il ne l'est pas", async () => {
    getInvoiceMock.mockResolvedValue(invoice());
    const { findByTestId, queryByTestId } = render(Page);
    await findByTestId("invoice-detail");
    expect(queryByTestId("invoice-pdf-frozen-at")).toBeNull();
  });
});

describe("fiche facture — refiger après un 410", () => {
  it("un admin voit le bouton après le 410, confirme, et la fiche se met à jour (mutation : envoi sans confirmation)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    getBlobMock.mockRejectedValue(gone);
    // La réponse du refigeage porte une AUTRE date que la relecture : si la
    // fiche l'affichait, le test le verrait (mutation : pas de relecture).
    refreezeMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-09-09T08:00:00.000" }),
    );
    // La fiche se RELIT après le refigeage (revue P2, F-L2) : c'est cette
    // lecture, et non la réponse du refigeage, qui doit s'afficher.
    getInvoiceMock
      .mockResolvedValueOnce(invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }))
      .mockResolvedValueOnce(
        invoice({
          pdfFrozenAt: "2026-04-01T08:00:00.000",
          amountSettled: "40.00",
          amountDue: "60.00",
        }),
      );
    const { findByTestId, getByTestId, queryByTestId } = render(Page);

    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await fireEvent.click(await findByTestId("invoice-pdf-refreeze-button"));
    // Rien n'est envoyé avant la confirmation, qui conseille de restaurer.
    expect(refreezeMock).not.toHaveBeenCalled();
    expect(
      getByTestId("invoice-pdf-refreeze-dialog").textContent,
    ).toContain("Restaurez d'abord");
    await fireEvent.click(getByTestId("invoice-pdf-refreeze-confirm"));

    await waitFor(() => expect(refreezeMock).toHaveBeenCalledWith(5));
    await waitFor(() => expect(getInvoiceMock).toHaveBeenCalledTimes(2));
    await waitFor(() =>
      expect(getByTestId("invoice-pdf-frozen-at").textContent).toBe(
        "2026-04-01",
      ),
    );
    await waitFor(() =>
      expect(queryByTestId("invoice-pdf-refreeze-button")).toBeNull(),
    );
  });

  it("pas de bouton avant le 410 (mutation : bouton toujours affiché à l'admin)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    const { findByTestId, queryByTestId } = render(Page);
    await findByTestId("invoice-download-pdf");
    expect(queryByTestId("invoice-pdf-refreeze-button")).toBeNull();
  });

  it("un Comptable ne le voit pas, même après le 410 (mutation : garde `isAdmin` retirée)", async () => {
    auth.currentUser.role = "Comptable";
    getInvoiceMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    getBlobMock.mockRejectedValue(gone);
    const { findByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await waitFor(() => expect(getBlobMock).toHaveBeenCalled());
    // Laisser le rejet se propager avant de conclure à l'absence.
    await new Promise((r) => setTimeout(r, 0));
    expect(queryByTestId("invoice-pdf-refreeze-button")).toBeNull();
  });

  it("un autre échec que le 410 n'ouvre pas le refigeage (mutation : tout échec pose `pdfGone`)", async () => {
    getInvoiceMock.mockResolvedValue(invoice());
    getBlobMock.mockRejectedValue({
      code: "INVOICE_NOT_PDF_READY",
      message: "Adresse incomplète.",
      status: 400,
    });
    const { findByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await waitFor(() => expect(getBlobMock).toHaveBeenCalled());
    await new Promise((r) => setTimeout(r, 0));
    expect(queryByTestId("invoice-pdf-refreeze-button")).toBeNull();
  });

  it("le message du 410 nomme le fichier manquant (mutation : message tiré du catalogue, `{ $sha256 }` brut)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    getBlobMock.mockRejectedValue(gone);
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await waitFor(() => expect(notifyErrorMock).toHaveBeenCalled());
    const msg = notifyErrorMock.mock.calls[0][0] as string;
    expect(msg).toContain("abc123.pdf");
    expect(msg).not.toContain("$sha256");
  });

  it("pas de refigeage pour une facture annulée, même après le 410 (mutation : condition `validated` retirée)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ status: "cancelled", pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    getBlobMock.mockRejectedValue(gone);
    const { findByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await waitFor(() => expect(notifyErrorMock).toHaveBeenCalled());
    await new Promise((r) => setTimeout(r, 0));
    expect(queryByTestId("invoice-pdf-refreeze-button")).toBeNull();
  });

  it("un téléchargement réussi après le 410 retire le bouton (mutation : `pdfGone` jamais remis à faux)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }),
    );
    getBlobMock
      .mockRejectedValueOnce(gone)
      .mockResolvedValueOnce({ blob: async () => new Blob(["%PDF-1.7"]) });
    const createObjectURL = vi
      .spyOn(URL, "createObjectURL")
      .mockReturnValue("blob:x");
    vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});
    const { findByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await findByTestId("invoice-pdf-refreeze-button");
    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await waitFor(() => expect(createObjectURL).toHaveBeenCalled());
    await waitFor(() =>
      expect(queryByTestId("invoice-pdf-refreeze-button")).toBeNull(),
    );
  });

  it("le refigeage réussi mais la relecture échoue : le dialogue se ferme, pas de faux échec (mutation : relecture dans le même try)", async () => {
    getInvoiceMock
      .mockResolvedValueOnce(invoice({ pdfFrozenAt: "2026-03-02T10:00:00.000" }))
      .mockRejectedValueOnce(new Error("réseau"));
    getBlobMock.mockRejectedValue(gone);
    refreezeMock.mockResolvedValue(invoice());
    const { findByTestId, getByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("invoice-download-pdf"));
    await fireEvent.click(await findByTestId("invoice-pdf-refreeze-button"));
    await fireEvent.click(getByTestId("invoice-pdf-refreeze-confirm"));

    await waitFor(() => expect(refreezeMock).toHaveBeenCalledWith(5));
    await waitFor(() =>
      expect(queryByTestId("invoice-pdf-refreeze-dialog")).toBeNull(),
    );
    expect(queryByTestId("invoice-pdf-refreeze-button")).toBeNull();
    expect(
      notifyErrorMock.mock.calls.some((c) => String(c[0]).includes("rechargez")),
    ).toBe(true);
  });
});
