/**
 * La fiche facture fournisseur, côté document — Story 30-2c (#685, AC18, AC20, AC24,
 * AC25 ; QR-IBAN, C-30-2-47).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `supplier-settlement-page.test.ts` — mocks hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@testing-library/svelte";
import type { SupplierInvoiceResponse } from "$lib/features/supplier-invoices/supplier-invoices.types";
import type { ImportedSupplierInvoice } from "$lib/features/imported-supplier-invoices/imported-supplier-invoices.types";

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
vi.mock("$lib/features/supplier-invoices/supplier-invoices.api", () => ({
  getSupplierInvoice: (id: number) => getMock(id),
  paySupplierInvoice: vi.fn(),
  cancelSupplierInvoice: vi.fn(),
  cancelSupplierInvoiceSettlement: vi.fn(),
}));
vi.mock("$lib/features/bank-accounts/bank-accounts.api", () => ({
  listBankAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/accounts/accounts.api", () => ({
  fetchAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/projects/projects.api", () => ({
  listProjects: vi.fn(async () => []),
}));
const fetchDocMock = vi.fn();
const attachMock = vi.fn();
const detachMock = vi.fn();
const listImportedMock = vi.fn();
vi.mock(
  "$lib/features/imported-supplier-invoices/imported-supplier-invoices.api",
  () => ({
    downloadSupplierInvoiceSourceDocument: vi.fn(),
    fetchSupplierInvoiceSourceDocument: (id: number) => fetchDocMock(id),
    attachSourceDocument: (id: number, doc: number) => attachMock(id, doc),
    detachSourceDocument: (id: number) => detachMock(id),
    listImported: (status: string) => listImportedMock(status),
    fetchImportedSourceDocument: vi.fn(
      async () => new Blob(["x"], { type: "image/png" }),
    ),
  }),
);

import Page from "./+page.svelte";

const DOC = {
  originalFilename: "facture-orage.png",
  mimeType: "image/png",
  byteSize: 1234,
};

function inv(
  partial: Partial<SupplierInvoiceResponse> = {},
): SupplierInvoiceResponse {
  return {
    id: 9,
    contactId: 1,
    supplierInvoiceNumber: "FF-9",
    status: "open",
    invoiceDate: "2026-06-15",
    dueDate: null,
    totalAmount: "100.00",
    creditorIban: null,
    creditorQrIban: null,
    paymentReference: null,
    expectedPaymentAmount: null,
    projectId: null,
    purchaseJournalEntryId: 4,
    settlementType: null,
    settlementBankAccountId: null,
    settlementAccountId: null,
    settlementJournalEntryId: null,
    paidAt: null,
    version: 1,
    createdAt: "2026-06-15T00:00:00",
    lines: [],
    settlementCancellable: false,
    settlementCancelBlockedBy: null,
    settlementCancelBlockedLabel: null,
    lastConfirmedBatch: null,
    cancellable: true,
    cancelBlockedBy: null,
    cancelBlockedLabel: null,
    sourceDocument: null,
    ...partial,
  };
}

function staged(
  partial: Partial<ImportedSupplierInvoice> = {},
): ImportedSupplierInvoice {
  return {
    id: 31,
    companyId: 1,
    status: "to_complete",
    supplierInvoiceId: null,
    fileHash: "h",
    storagePath: "h.png",
    originalFilename: "qr-orage.png",
    mimeType: "image/png",
    byteSize: 10,
    creditorIban: "CH4431999123000889012",
    isQrIban: true,
    creditorAddressType: "S",
    creditorName: "Orage SA",
    creditorLine1: null,
    creditorLine2: null,
    creditorPostalCode: null,
    creditorTown: null,
    creditorCountry: "CH",
    referenceType: "QRR",
    referenceValue: "210000000003139471430009017",
    amount: "1949.75",
    currency: "CHF",
    unstructuredMessage: null,
    billingInformation: null,
    version: 1,
    createdAt: "2026-06-16T08:00:00",
    updatedAt: "2026-06-16T08:00:00",
    ...partial,
  };
}

let confirmSpy: ReturnType<typeof vi.spyOn>;
beforeEach(() => {
  role.value = "Comptable";
  confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
  vi.spyOn(URL, "createObjectURL").mockImplementation(() => "blob:doc");
  vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});
});
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

describe("fiche facture fournisseur — le document (AC18)", () => {
  it("avec document ⇒ son nom, « Voir le document » ouvre l'aperçu ; ni « Joindre » (mutation M-c8 inverse)", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: DOC }));
    fetchDocMock.mockResolvedValue(new Blob(["x"], { type: "image/png" }));
    const { findByTestId, queryByTestId, getByTestId } = render(Page);
    expect(
      (await findByTestId("supplier-invoice-document-name")).textContent,
    ).toBe("facture-orage.png");
    expect(queryByTestId("supplier-invoice-attach")).toBeNull();
    await fireEvent.click(getByTestId("supplier-invoice-source-document"));
    expect(await findByTestId("document-preview-image")).toBeTruthy();
    expect(fetchDocMock).toHaveBeenCalledWith(9);
  });

  it("sans document (`null`) ⇒ aucun « Voir le document », « Joindre un document » (mutation M-c8 : bouton toujours rendu)", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: null }));
    const { findByTestId, queryByTestId } = render(Page);
    expect(await findByTestId("supplier-invoice-attach")).toBeTruthy();
    expect(queryByTestId("supplier-invoice-source-document")).toBeNull();
    expect(queryByTestId("supplier-invoice-detach")).toBeNull();
  });

  it("non calculé (`undefined`) ⇒ ni « Voir » ni « Joindre » : non calculé n'est pas « sans »", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: undefined }));
    const { findByTestId, queryByTestId } = render(Page);
    await findByTestId("supplier-invoice-status");
    expect(queryByTestId("supplier-invoice-source-document")).toBeNull();
    expect(queryByTestId("supplier-invoice-attach")).toBeNull();
  });

  it("rôle Consultation ⇒ voit le document, ni « Joindre » ni « Détacher »", async () => {
    role.value = "Consultation";
    getMock.mockResolvedValue(inv({ sourceDocument: DOC }));
    const { findByTestId, queryByTestId } = render(Page);
    expect(await findByTestId("supplier-invoice-source-document")).toBeTruthy();
    expect(queryByTestId("supplier-invoice-detach")).toBeNull();
    cleanup();
    getMock.mockResolvedValue(inv({ sourceDocument: null }));
    const second = render(Page);
    await second.findByTestId("supplier-invoice-status");
    expect(second.queryByTestId("supplier-invoice-attach")).toBeNull();
  });
});

describe("fiche facture fournisseur — joindre (AC20)", () => {
  it("le sélecteur montre le montant du QR à côté du total de la facture, puis joint après confirmation", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: null }));
    listImportedMock.mockResolvedValue([staged()]);
    attachMock.mockResolvedValue(inv({ sourceDocument: DOC }));
    const { findByTestId, getByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-attach"));
    expect(listImportedMock).toHaveBeenCalledWith("to_complete");
    const amounts = await findByTestId("attach-document-amounts");
    expect(getByTestId("attach-document-qr-amount").textContent).toContain(
      "1’949.75",
    );
    expect(getByTestId("attach-document-qr-amount").textContent).toContain(
      "CHF",
    );
    expect(amounts.textContent).toContain("Total de la facture : 100.00");
    // Aperçu du document choisi.
    await fireEvent.click(getByTestId("attach-document-preview"));
    expect(await findByTestId("attach-document-preview-panel")).toBeTruthy();
    await fireEvent.click(getByTestId("attach-document-submit"));
    expect(confirmSpy).toHaveBeenCalledWith(
      "Joindre le document « qr-orage.png » à cette facture ?",
    );
    await waitFor(() => expect(attachMock).toHaveBeenCalledWith(9, 31));
    expect(
      (await findByTestId("supplier-invoice-document-name")).textContent,
    ).toBe("facture-orage.png");
    expect(queryByTestId("attach-document-picker")).toBeNull();
  });

  it("un document sans QR : aucun montant de QR, le total de la facture seul", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: null }));
    listImportedMock.mockResolvedValue([
      staged({
        creditorIban: null,
        isQrIban: null,
        creditorAddressType: null,
        creditorName: null,
        creditorCountry: null,
        referenceType: null,
        referenceValue: null,
        amount: null,
        currency: null,
      }),
    ]);
    const { findByTestId, queryByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-attach"));
    expect(
      (await findByTestId("attach-document-invoice-total")).textContent,
    ).toContain("100.00");
    expect(queryByTestId("attach-document-qr-amount")).toBeNull();
  });

  it("confirmation refusée ⇒ rien n'est joint", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: null }));
    listImportedMock.mockResolvedValue([staged()]);
    confirmSpy.mockReturnValue(false);
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-attach"));
    await fireEvent.click(await findByTestId("attach-document-submit"));
    expect(attachMock).not.toHaveBeenCalled();
  });

  it("liste vide ⇒ le message qui dit où déposer le document", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: null }));
    listImportedMock.mockResolvedValue([]);
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-attach"));
    expect((await findByTestId("attach-document-empty")).textContent).toContain(
      "Aucun document à traiter — déposez-le dans le dossier d’import puis importez-le.",
    );
  });

  it("un refus du serveur s'affiche dans le sélecteur, qui reste ouvert", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: null }));
    listImportedMock.mockResolvedValue([staged()]);
    attachMock.mockRejectedValue(
      Object.assign(new Error("Cette facture a déjà un document."), {
        name: "ApiError",
        code: "SOURCE_DOCUMENT_ALREADY_ATTACHED",
        status: 409,
      }),
    );
    const { findByTestId, getByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-attach"));
    await fireEvent.click(await findByTestId("attach-document-submit"));
    expect(await findByTestId("attach-document-error")).toBeTruthy();
    expect(getByTestId("attach-document-picker")).toBeTruthy();
  });
});

describe("fiche facture fournisseur — détacher (AC24)", () => {
  it("confirme avec le texte prescrit, détache, puis propose de nouveau « Joindre un document »", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: DOC }));
    detachMock.mockResolvedValue(inv({ sourceDocument: null }));
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-detach"));
    expect(confirmSpy).toHaveBeenCalledWith(
      "Détacher ce document ? Il reviendra dans les documents à traiter ; la facture et son écriture ne changent pas.",
    );
    await waitFor(() => expect(detachMock).toHaveBeenCalledWith(9));
    expect(await findByTestId("supplier-invoice-attach")).toBeTruthy();
  });

  it("confirmation refusée ⇒ rien n'est détaché", async () => {
    getMock.mockResolvedValue(inv({ sourceDocument: DOC }));
    confirmSpy.mockReturnValue(false);
    const { findByTestId } = render(Page);
    await fireEvent.click(await findByTestId("supplier-invoice-detach"));
    expect(detachMock).not.toHaveBeenCalled();
  });
});

describe("fiche facture fournisseur — date de règlement (AC25) et QR-IBAN", () => {
  it("`paidAt` à minuit ⇒ la date au format suisse, jamais la chaîne brute (mutation M-c12)", async () => {
    getMock.mockResolvedValue(
      inv({
        status: "paid",
        paidAt: "2026-06-20T00:00:00",
        settlementJournalEntryId: 5,
      }),
    );
    const { findByTestId } = render(Page);
    const info = (await findByTestId("supplier-invoice-paid-info")).textContent ?? "";
    expect(info).toContain("Facture réglée le 20.06.2026.");
    expect(info).not.toContain("T00:00:00");
    expect(info).not.toContain("2026-06-20");
  });

  it("`paidAt` absent ⇒ la phrase sans date", async () => {
    getMock.mockResolvedValue(inv({ status: "paid", paidAt: null }));
    const { findByTestId } = render(Page);
    expect(
      (await findByTestId("supplier-invoice-paid-info")).textContent?.trim(),
    ).toBe("Facture réglée.");
  });

  it("une facture à QR-IBAN montre son QR-IBAN (C-30-2-47)", async () => {
    getMock.mockResolvedValue(
      inv({ creditorQrIban: "CH4431999123000889012" }),
    );
    const { findByTestId } = render(Page);
    expect((await findByTestId("supplier-invoice-qr-iban")).textContent).toBe(
      "CH4431999123000889012",
    );
  });
});
