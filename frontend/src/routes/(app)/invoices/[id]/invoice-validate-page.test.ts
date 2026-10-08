/**
 * La fiche facture, côté « Valider » — Story 15-5d (#429, choix C41, C46).
 *
 * Le refus de la garde à l'usage (400 `ACCOUNT_NOT_POSTABLE` : un compte
 * DÉSIGNÉ dans les réglages de facturation est devenu non imputable) s'affiche
 * **tel que le serveur l'a rédigé** — il dit déjà où agir et qui peut le
 * faire —, pour un Comptable comme pour un Admin : il n'emprunte ni le suffixe
 * « Configurez les comptes par défaut » ni le « Demandez à votre
 * administrateur » de la branche `CONFIGURATION_REQUIRED`. Et le dialogue se
 * FERME : réessayer rendrait le même refus.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `invoice-write-off-page.test.ts` — mocks hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import type { InvoiceResponse } from "$lib/features/invoices/invoices.types";

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
const validateInvoiceMock = vi.fn();
vi.mock("$lib/features/invoices/invoices.api", () => ({
  getInvoice: (id: number) => getInvoiceMock(id),
  listInvoiceSettlements: (id: number) => listSettlementsMock(id),
  cancelInvoiceSettlement: (id: number, sid: number) =>
    cancelSettlementMock(id, sid),
  deleteInvoice: vi.fn(),
  validateInvoice: (id: number) => validateInvoiceMock(id),
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
import { notifyError } from "$lib/shared/utils/notify";

const MESSAGE =
  "Le compte 1100, désigné dans Paramètres → Facturation, n’est pas imputable (compte de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à sa place un compte imputable.";

function brouillon(): InvoiceResponse {
  return {
    id: 5,
    companyId: 1,
    contactId: 1,
    invoiceNumber: null,
    status: "draft",
    date: "2026-03-01",
    dueDate: "2026-03-31",
    paymentTerms: null,
    totalAmount: "100.00",
    totalTtc: "108.10",
    vatBreakdown: [],
    roundingAmount: "0",
    roundingIsPreview: true,
    journalEntryId: null,
    paidAt: null,
    emailedAt: null,
    emailedTo: null,
    projectId: null,
    dunningPausedAt: null,
    dunningPausedNote: null,
    pdfFrozenAt: null,
    pdfLanguage: null,
    isOverdue: false,
    amountSettled: null,
    amountDue: null,
    version: 1,
    createdAt: "2026-03-01T00:00:00",
    updatedAt: "2026-03-01T00:00:00",
    lines: [],
  } as unknown as InvoiceResponse;
}

beforeEach(() => {
  getInvoiceMock.mockResolvedValue(brouillon());
  listSettlementsMock.mockResolvedValue([]);
  validateInvoiceMock.mockRejectedValue({
    code: "ACCOUNT_NOT_POSTABLE",
    status: 400,
    message: MESSAGE,
  });
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  getInvoiceMock.mockReset();
  listSettlementsMock.mockReset();
  validateInvoiceMock.mockReset();
});

describe("fiche facture — validation refusée par la garde des comptes désignés", () => {
  for (const role of ["Comptable", "Admin"] as const) {
    it(`${role} : le message du serveur est affiché tel quel et le dialogue se ferme (mutations : branche CONFIGURATION_REQUIRED empruntée ; code retiré de la liste de fermeture)`, async () => {
      auth.currentUser = { role, username: "u", userId: "1" };
      const r = render(Page);
      await fireEvent.click(await r.findByTestId("invoice-validate-button"));
      await r.findByTestId("invoice-validate-dialog");
      await fireEvent.click(r.getByTestId("invoice-validate-confirm"));
      await waitFor(() => expect(validateInvoiceMock).toHaveBeenCalledTimes(1));
      await waitFor(() => expect(notifyError).toHaveBeenCalledWith(MESSAGE));
      const affiche = vi.mocked(notifyError).mock.calls.map((c) => String(c[0])).join(" ");
      expect(affiche).not.toContain("Configurez les comptes par défaut");
      expect(affiche).not.toContain("Demandez à votre administrateur");
      await waitFor(() => expect(r.queryByTestId("invoice-validate-dialog")).toBeNull());
    });
  }
});
