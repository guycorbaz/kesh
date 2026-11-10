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
const settleMock = vi.fn();
vi.mock("$lib/features/invoices/invoices.api", () => ({
  getInvoice: (id: number) => getInvoiceMock(id),
  listInvoiceSettlements: (id: number) => listSettlementsMock(id),
  cancelInvoiceSettlement: (id: number, sid: number) =>
    cancelSettlementMock(id, sid),
  deleteInvoice: vi.fn(),
  validateInvoice: vi.fn(),
  unvalidateInvoice: vi.fn(),
  settleInvoice: (id: number, payload: unknown) => settleMock(id, payload),
  getInvoiceEmailPreview: vi.fn(),
  sendInvoiceEmail: vi.fn(),
  getInvoiceSettings: vi.fn(async () => ({})),
  writeOffInvoice: vi.fn(),
}));
vi.mock("$lib/features/accounts/accounts.api", () => ({
  fetchAccounts: vi.fn(async () => [
    { id: 1, number: "1000", name: "Caisse", accountType: "Asset", active: true, postable: true, role: null },
  ]),
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
    pdfFrozenAt: null,
    pdfLanguage: null,
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
  writeOffNature: null,
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
        "Ce règlement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.",
      status: 409,
    });
    const { findByTestId, getByTestId, findByText } = render(Page);

    await fireEvent.click(await findByTestId("invoice-settlement-cancel"));
    await fireEvent.click(getByTestId("invoice-settlement-cancel-confirm"));

    const erreur = await findByTestId("invoice-settlement-cancel-error");
    expect(erreur.textContent).toContain("en commençant par le plus récent");
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

// Revue P1 de la 33-6 (E-4 = A-4) : sans réponse, le règlement a pu être écrit. Depuis la
// Story 34-5 (#951), le serveur refuse un second envoi (le POST porte la version) ; l'écran
// relit encore, pour montrer ce qui est écrit et tenir une version fraîche (C-e343-5).
describe("fiche facture — règlement à l'issue inconnue", () => {
  const aRegler = () => invoice({ paidAt: null, amountSettled: "0.00", amountDue: "100.00" });

  async function regler() {
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("settle-open"));
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    return r;
  }

  beforeEach(() => {
    getInvoiceMock.mockReset();
    getInvoiceMock.mockResolvedValue(aRegler());
    listSettlementsMock.mockReset();
    listSettlementsMock.mockResolvedValue([]);
    settleMock.mockReset();
  });

  for (const [nom, erreur] of [
    ["coupure réseau", { code: "NETWORK_ERROR", status: 0, message: "Serveur injoignable." }],
    ["délai dépassé", { code: "TIMEOUT", status: 0, message: "Pas de réponse." }],
    ["502 du proxy", { code: "BAD_GATEWAY", status: 502, message: "Passerelle." }],
    ["504 du proxy", { code: "GATEWAY_TIMEOUT", status: 504, message: "Passerelle." }],
    ["500 du serveur", { code: "INTERNAL_ERROR", status: 500, message: "Erreur interne." }],
    ["503 au commit", { code: "SERVICE_UNAVAILABLE", status: 503, message: "Base injoignable." }],
    ["réponse illisible", new TypeError("réponse vide")],
  ] as const) {
    it(`${nom} : plus de geste identique, facture et règlements relus, issue dite`, async () => {
      settleMock.mockRejectedValue(erreur);
      const r = await regler();
      const avis = await r.findByTestId("settle-unknown-outcome");
      expect(avis.getAttribute("role")).toBe("alert");
      await waitFor(() => expect(r.queryByTestId("settle-confirm")).toBeNull());
      expect(settleMock).toHaveBeenCalledTimes(1);
      await waitFor(() => expect(getInvoiceMock).toHaveBeenCalledTimes(2));
      await waitFor(() => expect(listSettlementsMock).toHaveBeenCalledTimes(2));
      await waitFor(() =>
        expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(false),
      );
    });
  }

  it("relecture en échec : bouton indisponible jusqu'à une relecture réussie", async () => {
    settleMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("settle-open"));
    getInvoiceMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    await r.findByTestId("settle-unknown-reread");
    expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(true);

    getInvoiceMock.mockRejectedValueOnce({ code: "NETWORK_ERROR", status: 0, message: "x" });
    await fireEvent.click(r.getByTestId("settle-unknown-reread"));
    await waitFor(() => expect(getInvoiceMock.mock.calls.length).toBeGreaterThan(2));
    expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(true);

    getInvoiceMock.mockReset();
    getInvoiceMock.mockResolvedValue(aRegler());
    await fireEvent.click(await r.findByTestId("settle-unknown-reread"));
    await waitFor(() =>
      expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(false),
    );
    expect(r.queryByTestId("settle-unknown-reread")).toBeNull();
    expect(r.getByTestId("settle-unknown-outcome")).toBeTruthy();
  });

  it("une liste de règlements illisible n'est pas une relecture réussie", async () => {
    settleMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("settle-open"));
    listSettlementsMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    await r.findByTestId("settle-unknown-reread");
    expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(true);
  });

  it("un refus du serveur reste un refus : dialogue ouvert, message, aucune relecture", async () => {
    settleMock.mockRejectedValue({ code: "VALIDATION_ERROR", status: 400, message: "Trop-perçu." });
    const r = await regler();
    await r.findByText("Trop-perçu.");
    expect(r.getByTestId("settle-confirm")).toBeTruthy();
    expect(r.queryByTestId("settle-unknown-outcome")).toBeNull();
    expect(getInvoiceMock).toHaveBeenCalledTimes(1);
  });

  it("un règlement réussi ensuite retire l'avis", async () => {
    settleMock.mockRejectedValueOnce({ code: "TIMEOUT", status: 0, message: "x" });
    const r = await regler();
    await r.findByTestId("settle-unknown-outcome");
    await waitFor(() =>
      expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(false),
    );
    settleMock.mockResolvedValueOnce({ invoice: aRegler(), fullySettled: false });
    await fireEvent.click(r.getByTestId("settle-open"));
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    await waitFor(() => expect(r.queryByTestId("settle-unknown-outcome")).toBeNull());
  });
});

// Story 34-5 (#951) — le règlement porte la version de la facture ; un 409 de version
// ferme le dialogue, relit, et dit que rien n'a été enregistré (C-e343-4).
describe("fiche facture — règlement à une version périmée", () => {
  const aRegler = (partial: Partial<InvoiceResponse> = {}) =>
    invoice({ paidAt: null, amountSettled: "0.00", amountDue: "100.00", ...partial });
  const conflit = {
    code: "OPTIMISTIC_LOCK_CONFLICT",
    status: 409,
    message: "Conflit de version.",
  };

  async function ouvrirEtRegler(r: ReturnType<typeof render>) {
    await fireEvent.click(await r.findByTestId("settle-open"));
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
  }

  beforeEach(() => {
    getInvoiceMock.mockReset();
    getInvoiceMock.mockResolvedValue(aRegler());
    listSettlementsMock.mockReset();
    listSettlementsMock.mockResolvedValue([]);
    settleMock.mockReset();
  });

  it("le règlement part avec la version que la fiche a lue", async () => {
    settleMock.mockResolvedValue({ invoice: aRegler({ version: 4 }), fullySettled: false });
    const r = render(Page);
    await ouvrirEtRegler(r);
    await waitFor(() => expect(settleMock).toHaveBeenCalledTimes(1));
    expect(settleMock.mock.calls[0][1]).toMatchObject({
      settlementType: "internal_account",
      accountId: 1,
      version: 3,
    });
  });

  it("deux acomptes successifs : le second part avec la version RENDUE par le premier (M3)", async () => {
    settleMock
      .mockResolvedValueOnce({
        invoice: aRegler({ version: 4, amountSettled: "40.00", amountDue: "60.00" }),
        fullySettled: false,
      })
      .mockResolvedValueOnce({
        invoice: aRegler({ version: 5, amountSettled: "100.00", amountDue: "0.00" }),
        fullySettled: true,
      });
    const r = render(Page);
    await ouvrirEtRegler(r);
    await waitFor(() => expect(r.queryByTestId("settle-confirm")).toBeNull());
    await ouvrirEtRegler(r);
    await waitFor(() => expect(settleMock).toHaveBeenCalledTimes(2));
    expect((settleMock.mock.calls[0][1] as { version: number }).version).toBe(3);
    expect((settleMock.mock.calls[1][1] as { version: number }).version).toBe(4);
    // Aucune relecture de la facture : la réponse suffit.
    expect(getInvoiceMock).toHaveBeenCalledTimes(1);
  });

  it("409 : dialogue fermé, facture et règlements relus, avis « non enregistré » (M4)", async () => {
    settleMock.mockRejectedValue(conflit);
    const r = render(Page);
    getInvoiceMock.mockResolvedValue(aRegler({ version: 4 }));
    await ouvrirEtRegler(r);
    const avis = await r.findByTestId("settle-conflict");
    expect(avis.getAttribute("role")).toBe("alert");
    expect(avis.textContent).toContain("n’a pas été enregistré");
    expect(avis.textContent).toContain("liste des règlements");
    expect(avis.textContent).not.toContain("n’est pas connue");
    expect(r.queryByTestId("settle-unknown-outcome")).toBeNull();
    await waitFor(() => expect(r.queryByTestId("settle-confirm")).toBeNull());
    await waitFor(() => expect(getInvoiceMock).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(listSettlementsMock).toHaveBeenCalledTimes(2));
    await waitFor(() =>
      expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(false),
    );
    expect(r.queryByTestId("settle-conflict-reread")).toBeNull();

    // Le clic suivant part avec la version relue, non la périmée.
    settleMock.mockReset();
    settleMock.mockResolvedValue({ invoice: aRegler({ version: 5 }), fullySettled: false });
    await ouvrirEtRegler(r);
    await waitFor(() => expect(settleMock).toHaveBeenCalledTimes(1));
    expect((settleMock.mock.calls[0][1] as { version: number }).version).toBe(4);
    await waitFor(() => expect(r.queryByTestId("settle-conflict")).toBeNull());
  });

  it("409 puis relecture en échec : `conflict-blocked`, puis « Relire » réussi mène au conflit, non à l'issue inconnue (M6)", async () => {
    settleMock.mockRejectedValue(conflit);
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("settle-open"));
    getInvoiceMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));

    await r.findByTestId("settle-conflict-reread");
    const bloque = r.getByTestId("settle-conflict");
    expect(bloque.textContent).toContain("n’a pas été enregistré");
    expect(bloque.textContent).toContain("n’a pas pu être relue");
    expect(bloque.textContent).not.toContain("n’est pas connue");
    expect(r.queryByTestId("settle-unknown-outcome")).toBeNull();
    expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(true);

    getInvoiceMock.mockReset();
    getInvoiceMock.mockResolvedValue(aRegler({ version: 4 }));
    await fireEvent.click(r.getByTestId("settle-conflict-reread"));
    await waitFor(() =>
      expect((r.getByTestId("settle-open") as HTMLButtonElement).disabled).toBe(false),
    );
    const relu = r.getByTestId("settle-conflict");
    expect(relu.textContent).toContain("Elle a été relue");
    expect(relu.textContent).not.toContain("n’est pas connue");
    expect(r.queryByTestId("settle-unknown-outcome")).toBeNull();
    expect(r.queryByTestId("settle-conflict-reread")).toBeNull();
  });

  it("un autre 4xx garde le dialogue ouvert, sans relecture ni avis de conflit", async () => {
    settleMock.mockRejectedValue({ code: "VALIDATION_ERROR", status: 400, message: "Trop-perçu." });
    const r = render(Page);
    await ouvrirEtRegler(r);
    await r.findByText("Trop-perçu.");
    expect(r.getByTestId("settle-confirm")).toBeTruthy();
    expect(r.queryByTestId("settle-conflict")).toBeNull();
    expect(getInvoiceMock).toHaveBeenCalledTimes(1);
  });
});
