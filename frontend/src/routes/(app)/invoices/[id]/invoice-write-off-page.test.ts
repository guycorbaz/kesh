/**
 * La fiche facture, côté « Solder le reste » — Story 25-4-d2b (#490).
 *
 * Ce que le dialogue seul ne peut pas prouver : que la fiche envoie la
 * `version` de la facture AFFICHÉE, distingue les refus par leur code, relit la
 * facture, ferme ou garde le dialogue selon l'état relu, et sépare « Déjà
 * réglé » de « Soldé » dans le récapitulatif.
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
const auth = vi.hoisted(() => ({
  currentUser: { role: "Comptable", username: "c", userId: "1" },
}));
vi.mock("$lib/app/stores/auth.svelte", () => ({ authState: auth }));

const getInvoiceMock = vi.fn();
const listSettlementsMock = vi.fn();
const cancelSettlementMock = vi.fn();
const writeOffMock = vi.fn();
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
  writeOffInvoice: (id: number, req: unknown) => writeOffMock(id, req),
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

function settlement(
  partial: Partial<InvoiceSettlementResponse> & { id: number },
): InvoiceSettlementResponse {
  return {
    journalEntryId: 100 + partial.id,
    amount: "40.00",
    settledOn: "2026-03-05",
    settlementType: "internal_account",
    writeOffNature: null,
    cancellable: true,
    cancelBlockedBy: null,
    cancelBlockedLabel: null,
    cancelBlockedDocumentId: null,
    ...partial,
  };
}

/** Une facture réglée de 40.— sur 100.—, reste 60.—, version 3. */
const partielle = () =>
  invoice({ paidAt: null, amountSettled: "40.0000", amountDue: "60.0000", version: 3 });

beforeEach(() => {
  auth.currentUser = { role: "Comptable", username: "c", userId: "1" };
  getInvoiceMock.mockResolvedValue(partielle());
  listSettlementsMock.mockResolvedValue([settlement({ id: 7 })]);
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  getInvoiceMock.mockReset();
  listSettlementsMock.mockReset();
  writeOffMock.mockReset();
});

async function openAndChoose(
  r: { findByTestId: (id: string) => Promise<HTMLElement> },
  nature = "discount",
): Promise<void> {
  await fireEvent.click(await r.findByTestId("write-off-open"));
  await fireEvent.click(await r.findByTestId(`write-off-nature-${nature}`));
}

describe("fiche facture — le bouton « Solder le reste »", () => {
  it("visible du Comptable sur une facture validée non payée au reste positif", async () => {
    const r = render(Page);
    expect(await r.findByTestId("write-off-open")).toBeTruthy();
  });

  it("masqué pour un rôle en lecture seule (mutation : `canManage` retiré)", async () => {
    auth.currentUser = { role: "Consultation", username: "l", userId: "2" };
    const r = render(Page);
    await r.findByTestId("settle-open");
    expect(r.queryByTestId("write-off-open")).toBeNull();
  });

  it("masqué quand le reste dû n'est pas calculé (`amountDue: null`)", async () => {
    getInvoiceMock.mockResolvedValue(invoice({ paidAt: null, amountDue: null }));
    const r = render(Page);
    await r.findByTestId("settle-open");
    expect(r.queryByTestId("write-off-open")).toBeNull();
  });
});

describe("fiche facture — solder le reste", () => {
  it("envoie la nature, la date et la VERSION de la facture affichée, puis relit la liste (mutation : version absente)", async () => {
    writeOffMock.mockResolvedValue({
      invoice: invoice({ paidAt: "2026-03-10T00:00:00", amountSettled: "100.0000", amountDue: "0.0000", version: 4 }),
      journalEntryId: 50,
      amount: "60.0000",
    });
    const r = render(Page);
    await openAndChoose(r);
    await fireEvent.click(r.getByTestId("write-off-confirm"));
    await waitFor(() => expect(writeOffMock).toHaveBeenCalledTimes(1));
    const [id, req] = writeOffMock.mock.calls[0];
    expect(id).toBe(5);
    expect(req).toMatchObject({ nature: "discount", version: 3 });
    expect((req as { settledOn: string }).settledOn).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    await waitFor(() => expect(listSettlementsMock).toHaveBeenCalledTimes(2));
  });

  it("409 de version : message dans le dialogue, facture relue, nouvelle tentative avec la version RELUE (mutation : copie à l'ouverture)", async () => {
    writeOffMock.mockRejectedValueOnce({
      code: "OPTIMISTIC_LOCK_CONFLICT",
      message: "conflit",
      status: 409,
    });
    const r = render(Page);
    await openAndChoose(r);
    // La relecture rend une version plus récente.
    getInvoiceMock.mockResolvedValue({ ...partielle(), version: 9 });
    await fireEvent.click(r.getByTestId("write-off-confirm"));
    const err = await r.findByTestId("write-off-error");
    expect(err.textContent).toContain("a changé");
    // Le dialogue reste ouvert : on peut confirmer à nouveau.
    writeOffMock.mockResolvedValueOnce({
      invoice: invoice({ amountDue: "0.0000", version: 10 }),
      journalEntryId: 51,
      amount: "60.0000",
    });
    await fireEvent.click(r.getByTestId("write-off-confirm"));
    await waitFor(() => expect(writeOffMock).toHaveBeenCalledTimes(2));
    expect(writeOffMock.mock.calls[1][1]).toMatchObject({ version: 9 });
  });

  it("facture dévalidée entre-temps (`ILLEGAL_STATE_TRANSITION`, 409 aussi) : relue, et le dialogue se ferme (mutation : distinction sur le statut HTTP)", async () => {
    writeOffMock.mockRejectedValueOnce({
      code: "ILLEGAL_STATE_TRANSITION",
      message: "Transition d'état interdite",
      status: 409,
    });
    const r = render(Page);
    await openAndChoose(r);
    getInvoiceMock.mockResolvedValue({ ...partielle(), status: "draft" });
    await fireEvent.click(r.getByTestId("write-off-confirm"));
    await waitFor(() => expect(r.queryByTestId("write-off-confirm")).toBeNull());
    expect(getInvoiceMock.mock.calls.length).toBeGreaterThanOrEqual(2);
  });
});

describe("fiche facture — fermer pendant l'envoi (revue P1, B-H1)", () => {
  it("Échap pendant l'envoi ne ferme pas le dialogue, et le refus s'y affiche (mutation : fermeture acceptée)", async () => {
    let reject: (e: unknown) => void = () => {};
    writeOffMock.mockReturnValue(new Promise((_, rej) => (reject = rej)));
    const r = render(Page);
    await openAndChoose(r);
    await fireEvent.click(r.getByTestId("write-off-confirm"));
    await fireEvent.keyDown(document.activeElement ?? document.body, { key: "Escape" });
    expect(r.queryByTestId("write-off-confirm")).not.toBeNull();
    // Revue P2 (L4) : la croix est masquée pendant l'envoi.
    expect(document.querySelector('[data-slot="dialog-close"]')).toBeNull();
    reject({
      code: "WRITE_OFF_ACCOUNT_NOT_CONFIGURED",
      message: "Aucun compte d'escompte utilisable n'est désigné",
      status: 400,
    });
    expect((await r.findByTestId("write-off-error")).textContent).toContain("escompte");
  });
});

describe("fiche facture — le récapitulatif sépare « Déjà réglé » et « Soldé »", () => {
  it("« Déjà réglé » ne compte plus le solde (mutation : `amountSettled` affiché tel quel)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ paidAt: "2026-03-10T00:00:00", amountSettled: "100.0000", amountDue: "0.0000" }),
    );
    listSettlementsMock.mockResolvedValue([
      settlement({ id: 7, amount: "40.0000" }),
      settlement({ id: 8, amount: "60.0000", settlementType: "write_off", writeOffNature: "discount" }),
    ]);
    const r = render(Page);
    expect((await r.findByTestId("invoice-amount-settled")).textContent).toContain("40.00");
    expect(r.getByTestId("invoice-amount-written-off").textContent).toContain("60.00");
  });

  it("sans liste (échec de chargement), seul le « Reste dû » s'affiche (mutation : drapeau absent)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ paidAt: "2026-03-10T00:00:00", amountSettled: "100.0000", amountDue: "0.0000" }),
    );
    listSettlementsMock.mockRejectedValue(new Error("réseau"));
    const r = render(Page);
    expect(await r.findByTestId("invoice-amount-due")).toBeTruthy();
    // ⚠️ Attendre que le chargement de la liste ait ÉCHOUÉ : interrogée avant, la
    // page montre l'état « en chargement », et le test passerait à vide.
    await waitFor(() => expect(listSettlementsMock).toHaveBeenCalled());
    await new Promise((done) => setTimeout(done, 0));
    expect(r.queryByTestId("invoice-amount-settled")).toBeNull();
    expect(r.queryByTestId("invoice-amount-written-off")).toBeNull();
  });

  it("un reste à fraction de centime s'affiche aux quatre décimales, pas « 0.00 » (#490)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ paidAt: null, amountSettled: "9.9960", amountDue: "0.0040" }),
    );
    listSettlementsMock.mockResolvedValue([settlement({ id: 7, amount: "9.9960" })]);
    const r = render(Page);
    expect((await r.findByTestId("invoice-amount-due")).textContent).toContain("0.0040");
  });
});

describe("fiche facture — annuler un solde", () => {
  it("le dialogue de confirmation dit « solde » pour un solde (mutation : texte du règlement)", async () => {
    getInvoiceMock.mockResolvedValue(
      invoice({ paidAt: "2026-03-10T00:00:00", amountSettled: "100.0000", amountDue: "0.0000" }),
    );
    listSettlementsMock.mockResolvedValue([
      settlement({ id: 8, amount: "100.0000", settlementType: "write_off", writeOffNature: "bad_debt" }),
    ]);
    const r = render(Page);
    const bouton = await r.findByTestId("invoice-settlement-cancel");
    expect(bouton.textContent).toContain("Annuler le solde");
    await fireEvent.click(bouton);
    expect(r.getByTestId("invoice-settlement-cancel-confirm").textContent).toContain(
      "Annuler le solde",
    );
    expect(await r.findByText(/Annuler ce solde \?/)).toBeTruthy();
  });
});
