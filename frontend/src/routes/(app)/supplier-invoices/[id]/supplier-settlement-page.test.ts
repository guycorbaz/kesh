/**
 * La fiche facture fournisseur, côté annulation du règlement — Story 25-3-a-2 (#414).
 *
 * Premier test de cette fiche. ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 * Patron : `invoices/[id]/invoice-settlements-page.test.ts` — mocks hoistés
 * AVANT l'import du composant.
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

let confirmSpy: ReturnType<typeof vi.spyOn>;
beforeEach(() => {
  role.value = "Comptable";
  confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  confirmSpy.mockRestore();
});

describe("fiche facture fournisseur — annuler le règlement", () => {
  it("payée et annulable ⇒ le bouton, qui annule puis rouvre la facture (mutation : réponse ignorée)", async () => {
    getMock.mockResolvedValue(inv());
    cancelSettlementMock.mockResolvedValue({
      invoice: inv({
        status: "open",
        settlementJournalEntryId: null,
        paidAt: null,
        settlementCancellable: false,
        settlementCancelBlockedBy: "SUPPLIER_INVOICE_NOT_PAID",
      }),
      reversalJournalEntryId: 12,
    });
    const { findByTestId, queryByTestId } = render(Page);
    expect(
      (await findByTestId("supplier-invoice-settlement-entry")).getAttribute(
        "href",
      ),
    ).toBe("/journal-entries/5");
    await fireEvent.click(
      await findByTestId("supplier-invoice-settlement-cancel"),
    );
    await waitFor(() => expect(cancelSettlementMock).toHaveBeenCalledWith(9));
    // Revenue « ouverte » : le formulaire de paiement revient, le motif « non
    // payée » ne s'affiche PAS à côté de lui.
    await waitFor(() =>
      expect(queryByTestId("supplier-invoice-pay")).not.toBeNull(),
    );
    expect(
      queryByTestId("supplier-invoice-settlement-cancel-blocked"),
    ).toBeNull();
  });

  it("non annulable ⇒ le MOTIF à la place du bouton (mutation : bouton sur le seul statut)", async () => {
    getMock.mockResolvedValue(
      inv({
        settlementCancellable: false,
        settlementCancelBlockedBy: "ACCOUNT_ARCHIVED",
        settlementCancelBlockedLabel: "1000",
      }),
    );
    const { findByTestId, queryByTestId } = render(Page);
    expect(
      (await findByTestId("supplier-invoice-settlement-cancel-blocked"))
        .textContent,
    ).toContain("(1000)");
    expect(queryByTestId("supplier-invoice-settlement-cancel")).toBeNull();
  });

  it("sans droit d’écriture ⇒ pas de bouton (mutation : rôle ignoré)", async () => {
    role.value = "Consultation";
    getMock.mockResolvedValue(inv());
    const { findByTestId, queryByTestId } = render(Page);
    await findByTestId("supplier-invoice-settlement");
    expect(queryByTestId("supplier-invoice-settlement-cancel")).toBeNull();
  });

  it("lot confirmé ⇒ la confirmation avertit du double paiement, sans dire « payée par ce lot » (mutation : avertissement omis)", async () => {
    getMock.mockResolvedValue(
      inv({
        lastConfirmedBatch: { id: 42, confirmedAt: "2026-07-02T10:00:00" },
      }),
    );
    cancelSettlementMock.mockResolvedValue({
      invoice: inv({ status: "open" }),
      reversalJournalEntryId: 1,
    });
    const { findByTestId } = render(Page);
    await fireEvent.click(
      await findByTestId("supplier-invoice-settlement-cancel"),
    );
    const texte = String(confirmSpy.mock.calls[0][0]);
    expect(texte).toContain("lot de paiement n° 42");
    expect(texte).toContain("2026-07-02");
    expect(texte).not.toContain("payée par");
  });

  it("refus au clic ⇒ affiché localement, la fiche reste (mutation : message dans `errorMsg`)", async () => {
    getMock.mockResolvedValue(inv());
    cancelSettlementMock.mockRejectedValue({
      code: "FISCAL_YEAR_CLOSED",
      message: "Ce règlement appartient à un exercice clôturé.",
      status: 409,
    });
    const { findByTestId, findByText } = render(Page);
    await fireEvent.click(
      await findByTestId("supplier-invoice-settlement-cancel"),
    );
    expect(
      (await findByTestId("supplier-invoice-settlement-cancel-error"))
        .textContent,
    ).toContain("exercice clôturé");
    expect(await findByText("FF-9")).toBeTruthy();
  });

  it("après `pay`, la réponse porte les champs : le bouton apparaît sans rechargement (mutation : `pay` sans champs)", async () => {
    getMock.mockResolvedValue(
      inv({
        status: "open",
        settlementJournalEntryId: null,
        paidAt: null,
        settlementCancellable: false,
        settlementCancelBlockedBy: "SUPPLIER_INVOICE_NOT_PAID",
      }),
    );
    payMock.mockResolvedValue(inv());
    const { findByTestId, getByTestId } = render(Page);
    await findByTestId("supplier-invoice-pay");
    const radios = document.querySelectorAll('input[type="radio"]');
    await fireEvent.click(radios[1]);
    const select = getByTestId("pay-internal-account") as HTMLSelectElement;
    select.value = select.options[1].value;
    await fireEvent.change(select);
    await fireEvent.click(getByTestId("supplier-invoice-pay-submit"));
    expect(
      await findByTestId("supplier-invoice-settlement-cancel"),
    ).toBeTruthy();
    expect(getMock).toHaveBeenCalledTimes(1);
  });
});

describe("fiche facture fournisseur — annuler la facture (Story 25-3-c)", () => {
  it("payée et annulable ⇒ le bouton, AUSSI sur une facture payée (mutation : bouton réservé à `open`)", async () => {
    getMock.mockResolvedValue(inv());
    const { findByTestId } = render(Page);
    expect(await findByTestId("supplier-invoice-cancel")).toBeTruthy();
  });

  it("Consultation ⇒ pas de bouton (mutation : rôle ignoré)", async () => {
    role.value = "Consultation";
    getMock.mockResolvedValue(inv({ status: "open", settlementJournalEntryId: null }));
    const { findByTestId, queryByTestId } = render(Page);
    await findByTestId("supplier-invoice-status");
    expect(queryByTestId("supplier-invoice-cancel")).toBeNull();
  });

  it("non annulable ⇒ le motif à la place du bouton, avec le numéro du compte (mutation : bouton sur le seul statut)", async () => {
    getMock.mockResolvedValue(
      inv({
        status: "open",
        cancellable: false,
        cancelBlockedBy: "ACCOUNT_ARCHIVED",
        cancelBlockedLabel: "4000",
      }),
    );
    const { findByTestId, queryByTestId } = render(Page);
    const motif = await findByTestId("supplier-invoice-cancel-blocked");
    expect(motif.textContent).toContain("archivé");
    expect(motif.textContent).toContain("4000");
    expect(queryByTestId("supplier-invoice-cancel")).toBeNull();
  });

  it("confirmation d'une facture payée ⇒ elle dit « paiement sans facture » (mutation : texte `open` seul)", async () => {
    getMock.mockResolvedValue(inv());
    cancelInvoiceMock.mockResolvedValue(
      inv({
        status: "cancelled",
        settlementJournalEntryId: null,
        paidAt: null,
        cancellable: false,
        cancelBlockedBy: "SUPPLIER_INVOICE_CANCELLED",
      }),
    );
    const { findByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-cancel"));
    await waitFor(() => expect(cancelInvoiceMock).toHaveBeenCalledWith(9));
    const message = String(confirmSpy.mock.calls[0][0]);
    expect(message).toContain("paiement sans facture");
    expect(message).toContain("annulez plutôt le règlement d'abord");
    // Annulée : l'état, et ni bouton, ni motif « déjà annulée », ni lien de règlement.
    await findByTestId("supplier-invoice-cancelled-info");
    expect(queryByTestId("supplier-invoice-cancel")).toBeNull();
    expect(queryByTestId("supplier-invoice-cancel-blocked")).toBeNull();
    expect(queryByTestId("supplier-invoice-settlement-entry")).toBeNull();
  });

  it("confirmation d'une facture ouverte ⇒ sans le paragraphe du paiement", async () => {
    getMock.mockResolvedValue(inv({ status: "open", settlementJournalEntryId: null }));
    cancelInvoiceMock.mockResolvedValue(inv({ status: "cancelled", cancellable: false }));
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-cancel"));
    await waitFor(() => expect(cancelInvoiceMock).toHaveBeenCalled());
    expect(String(confirmSpy.mock.calls[0][0])).not.toContain("paiement sans facture");
  });

  it("refus au clic ⇒ affiché LOCALEMENT, la fiche reste (mutation : refus dans `errorMsg`)", async () => {
    getMock.mockResolvedValue(inv({ status: "open", settlementJournalEntryId: null }));
    cancelInvoiceMock.mockRejectedValue({
      code: "FISCAL_YEAR_CLOSED",
      message: "Cette facture appartient à un exercice clôturé",
      status: 409,
    });
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-cancel"));
    expect((await findByTestId("supplier-invoice-cancel-error")).textContent).toContain(
      "exercice clôturé",
    );
    // La fiche est toujours là.
    expect(await findByTestId("supplier-invoice-status")).toBeTruthy();
  });
});

// Story 15-6b (#474, AC8 ; test 19) — le compte créanciers n'est pas une contrepartie.
describe("fiche facture fournisseur — la contrepartie n’est pas le compte créanciers", () => {
  const CAISSE = {
    id: 1,
    number: "1000",
    name: "Caisse",
    active: true,
    postable: true,
    role: null,
  };
  const CREANCIERS = {
    id: 2000,
    number: "2000",
    name: "Créanciers",
    active: true,
    postable: true,
    role: "Payable",
  };
  function banque(id: number, journalAccountId: number | null) {
    return {
      id,
      bankName: `Banque ${id}`,
      iban: "CH00",
      qrIban: null,
      isPrimary: id === 1,
      journalAccountId,
      version: 1,
      archived: false,
    };
  }
  const EMPTY =
    "Aucun compte bancaire utilisable : le seul compte lié est le compte créanciers de cette facture. Reliez un compte bancaire à son propre compte de banque, ou réglez par un compte interne.";
  const options = (testId: string) =>
    Array.from(
      (document.querySelector(`[data-testid="${testId}"]`) as HTMLSelectElement)
        .options,
    ).map((o) => o.value);

  async function monter(banks: unknown[], accounts: unknown[] = [CAISSE, CREANCIERS]) {
    const { listBankAccounts } = await import(
      "$lib/features/bank-accounts/bank-accounts.api"
    );
    const { fetchAccounts } = await import(
      "$lib/features/accounts/accounts.api"
    );
    vi.mocked(listBankAccounts).mockResolvedValueOnce(banks as never);
    vi.mocked(fetchAccounts).mockResolvedValueOnce(accounts as never);
    getMock.mockResolvedValue(
      inv({ status: "open", settlementJournalEntryId: null, paidAt: null }),
    );
    return render(Page);
  }

  it("la banque liée au 2000 est absente, l’autre reste (mutation : filtre retiré)", async () => {
    const { findByTestId, queryByTestId } = await monter([
      banque(1, 2000),
      banque(2, 1020),
    ]);
    await findByTestId("pay-bank-account");
    await waitFor(() => expect(options("pay-bank-account")).toContain("2"));
    expect(options("pay-bank-account")).not.toContain("1");
    expect(queryByTestId("pay-no-eligible-bank")).toBeNull();
  });

  it("2000 est absent du menu « Compte interne »", async () => {
    const { findByTestId, container } = await monter([banque(2, 1020)]);
    await findByTestId("supplier-invoice-pay");
    const radio = container.querySelector(
      'input[value="internal_account"]',
    ) as HTMLInputElement;
    await fireEvent.click(radio);
    await findByTestId("pay-internal-account");
    await waitFor(() => expect(options("pay-internal-account")).toContain("1"));
    expect(options("pay-internal-account")).not.toContain("2000");
  });

  it("seule banque liée au 2000 → le message de liste vide", async () => {
    const { findByText } = await monter([banque(1, 2000)]);
    expect(await findByText(EMPTY)).toBeTruthy();
  });

  // Revue de code P1 (finding L6) — les ids se calculent AVANT le filtre
  // `active && postable` : un 2000 devenu non imputable écarte toujours la
  // banque qui y est liée.
  it("2000 non imputable : la banque qui y est liée reste absente (mutation : ids calculés après le filtre `active && postable`)", async () => {
    const { findByTestId } = await monter(
      [banque(1, 2000), banque(2, 1020)],
      [CAISSE, { ...CREANCIERS, postable: false }],
    );
    await findByTestId("pay-bank-account");
    await waitFor(() => expect(options("pay-bank-account")).toContain("2"));
    expect(options("pay-bank-account")).not.toContain("1");
  });

  it("aucun compte bancaire lié → pas de message de liste vide", async () => {
    const { findByTestId, queryByTestId } = await monter([banque(3, null)]);
    await findByTestId("pay-bank-account");
    expect(queryByTestId("pay-no-eligible-bank")).toBeNull();
  });
});
