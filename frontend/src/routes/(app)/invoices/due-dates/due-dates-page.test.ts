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
const settleMock = vi.fn();
const exportDueDatesCsvMock = vi.hoisted(() => vi.fn());
vi.mock("$lib/features/invoices/invoices.api", () => ({
  listDueDates: (q: unknown) => listDueDatesMock(q),
  settleInvoice: (id: number, payload: unknown) => settleMock(id, payload),
  exportDueDatesCsv: (q: unknown) => exportDueDatesCsvMock(q),
}));
vi.mock("$lib/features/accounts/accounts.api", () => ({
  fetchAccounts: vi.fn(async () => [
    { id: 1, number: "1000", name: "Caisse", accountType: "Asset", active: true, postable: true, role: null },
  ]),
}));
vi.mock("$lib/features/bank-accounts/bank-accounts.api", () => ({
  listBankAccounts: vi.fn(async () => []),
}));
vi.mock("$lib/features/contacts/contacts.api", () => ({
  getContact: vi.fn(),
  listContacts: vi.fn(async () => ({ items: [], total: 0, offset: 0, limit: 20 })),
}));

import Page from "./+page.svelte";
import { notifyError } from "$lib/shared/utils/notify";
import { pinClock } from "$lib/shared/utils/pin-clock.test-helper";

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

// Story v0171 (#879) — les dates de l'échéancier en forme suisse. Mutation M5 : retirer
// `formatSwissDate` d'un site rougit le test qui le nomme.
describe("échéancier — dates (v0171)", () => {
  it("date de facture `01.03.2026` (mutation M5 : site `inv.date`)", async () => {
    const { findByTestId, container } = render(Page);
    expect((await findByTestId("due-dates-date")).textContent?.trim()).toBe(
      "01.03.2026",
    );
    expect(container.textContent).not.toContain("2026-03-01");
  });

  it("échéance `31.03.2026`, absente ⇒ `—` (mutation M5 : site `inv.dueDate`)", async () => {
    listDueDatesMock.mockResolvedValue(
      response([item(), item({ id: 6, dueDate: null })]),
    );
    const { findAllByTestId, container } = render(Page);
    const dues = await findAllByTestId("due-dates-due");
    expect(dues.map((d) => d.textContent?.trim())).toEqual(["31.03.2026", "—"]);
    expect(container.textContent).not.toContain("2026-03-31");
  });

  it("« Payée le » `10.04.2026`, non payée ⇒ `—` (mutation M5 : site `inv.paidAt`)", async () => {
    listDueDatesMock.mockResolvedValue(
      response([
        item({ paidAt: "2026-04-10T09:30:00", amountDue: "0.00" }),
        item({ id: 6 }),
      ]),
    );
    const { findAllByTestId, container } = render(Page);
    const paid = await findAllByTestId("due-dates-paid-at");
    expect(paid.map((d) => d.textContent?.trim())).toEqual(["10.04.2026", "—"]);
    expect(container.textContent).not.toContain("2026-04-10");
  });
});

// Story v0174-correctifs-3 (#889, AC-3.6, site 10 ; AC-3.8, f) — à 00 h 30 le 5 novembre à
// Zurich : le CSV exporté porte la date du 5 ; une date de règlement (minuit) reste la sienne.
describe("échéancier — date du jour locale (#889)", () => {
  pinClock();

  it("le fichier exporté s'appelle echeancier-2026-11-05.csv (site 10)", async () => {
    exportDueDatesCsvMock.mockResolvedValue(new Blob(["a;b\r\n"], { type: "text/csv" }));
    vi.stubGlobal("URL", {
      createObjectURL: vi.fn().mockReturnValue("blob:http://localhost/x"),
      revokeObjectURL: vi.fn(),
    });
    const names: string[] = [];
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(function (this: HTMLAnchorElement) {
        names.push(this.download);
      });
    const { findByText } = render(Page);
    await fireEvent.click(await findByText("Exporter CSV"));
    await waitFor(() => expect(names).toEqual(["echeancier-2026-11-05.csv"]));
    click.mockRestore();
    // `URL` reste remplacé : la page révoque l'URL objet 5 s plus tard (minuterie réelle).
  });

  it("« Payée le » d'une date à minuit reste `04.11.2026` (f, conforme)", async () => {
    listDueDatesMock.mockResolvedValue(
      response([item({ paidAt: "2026-11-04T00:00:00", amountDue: "0.00" })]),
    );
    const { findAllByTestId } = render(Page);
    const paid = await findAllByTestId("due-dates-paid-at");
    expect(paid.map((d) => d.textContent?.trim())).toEqual(["04.11.2026"]);
  });
});

describe("échéancier — une date en cours de frappe (Story 33-4)", () => {
  it("l'année tapée n'interroge pas, ne notifie rien, ne vide ni la liste ni les totaux (mutation : valeur appliquée sans contrôle)", async () => {
    const { findAllByTestId, container } = render(Page);
    await waitFor(() => expect(listDueDatesMock).toHaveBeenCalled());
    await waitFor(() =>
      expect(container.querySelector("#dd-due-before")).not.toBeNull(),
    );
    const input = container.querySelector("#dd-due-before") as HTMLInputElement;
    const avant = listDueDatesMock.mock.calls.length;
    for (const v of ["0002-12-31", "0020-12-31", "0202-12-31"]) {
      await fireEvent.input(input, { target: { value: v } });
      expect(input.value).toBe(v);
    }
    await new Promise((r) => setTimeout(r, 0));
    expect(listDueDatesMock.mock.calls.length).toBe(avant);
    expect(vi.mocked(notifyError)).not.toHaveBeenCalled();
    expect((await findAllByTestId("due-dates-date")).length).toBeGreaterThan(0);

    await fireEvent.input(input, { target: { value: "2026-12-31" } });
    await waitFor(() =>
      expect(listDueDatesMock).toHaveBeenLastCalledWith(
        expect.objectContaining({ dueBefore: "2026-12-31" }),
      ),
    );
    await fireEvent.input(input, { target: { value: "0202-12-31" } });
    await fireEvent.blur(input);
    expect(input.value).toBe("2026-12-31");
  });
});

// Revue P1 de la 33-6 (E-4 = A-4) : le même geste, depuis l'échéancier.
describe("échéancier — règlement à l'issue inconnue", () => {
  async function regler() {
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("due-dates-settle-5"));
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    return r;
  }

  beforeEach(() => {
    settleMock.mockReset();
  });

  it("sans réponse : le dialogue se ferme, la liste est relue, l'issue est dite et nomme la facture", async () => {
    settleMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    const r = await regler();
    const avis = await r.findByTestId("due-dates-settle-unknown");
    expect(avis.textContent).toContain("F-2026-005");
    await waitFor(() => expect(r.queryByTestId("settle-confirm")).toBeNull());
    await waitFor(() => expect(listDueDatesMock.mock.calls.length).toBeGreaterThan(1));
    await waitFor(() =>
      expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false),
    );
  });

  it("un 503 à l'instant du commit vaut issue inconnue : dialogue fermé, liste relue (P2-2)", async () => {
    settleMock.mockRejectedValue({ code: "SERVICE_UNAVAILABLE", status: 503, message: "Base." });
    const r = await regler();
    await r.findByTestId("due-dates-settle-unknown");
    await waitFor(() => expect(r.queryByTestId("settle-confirm")).toBeNull());
    await waitFor(() => expect(listDueDatesMock.mock.calls.length).toBeGreaterThan(1));
  });

  it("l'avis nomme la facture réglée, même si une autre a été ouverte pendant l'envoi (P2-3)", async () => {
    listDueDatesMock.mockResolvedValue(
      response([item(), item({ id: 6, invoiceNumber: "F-2026-006" })]),
    );
    let rejeter: (err: unknown) => void = () => {};
    settleMock.mockReturnValue(new Promise((_, reject) => (rejeter = reject)));
    const r = await regler();
    await fireEvent.click(await r.findByTestId("due-dates-settle-6"));
    rejeter({ code: "NETWORK_ERROR", status: 0, message: "x" });
    const avis = await r.findByTestId("due-dates-settle-unknown");
    expect(avis.textContent).toContain("F-2026-005");
    expect(avis.textContent).not.toContain("F-2026-006");
  });

  it("relecture en échec : la liste vidée n'offre aucun geste, Relire la rétablit", async () => {
    settleMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("due-dates-settle-5"));
    listDueDatesMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    await r.findByTestId("due-dates-settle-unknown-reread");

    listDueDatesMock.mockResolvedValue(response([item()]));
    await fireEvent.click(r.getByTestId("due-dates-settle-unknown-reread"));
    await waitFor(() =>
      expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false),
    );
    expect(r.queryByTestId("due-dates-settle-unknown-reread")).toBeNull();
    expect(r.getByTestId("due-dates-settle-unknown")).toBeTruthy();
  });

  it("un refus du serveur garde le dialogue ouvert, sans relecture", async () => {
    settleMock.mockRejectedValue({ code: "VALIDATION_ERROR", status: 400, message: "Trop-perçu." });
    const r = await regler();
    await r.findByText("Trop-perçu.");
    expect(r.getByTestId("settle-confirm")).toBeTruthy();
    expect(r.queryByTestId("due-dates-settle-unknown")).toBeNull();
  });
});

// Story 34-5 (#951) — la version de la ligne part avec le règlement ; un 409 de version
// ferme le dialogue, relit la liste, et dit que rien n'a été enregistré (C-e343-4).
describe("échéancier — règlement à une version périmée", () => {
  const conflit = { code: "OPTIMISTIC_LOCK_CONFLICT", status: 409, message: "Conflit." };

  async function regler() {
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("due-dates-settle-5"));
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    return r;
  }

  beforeEach(() => {
    settleMock.mockReset();
  });

  it("le règlement part avec la version de la ligne", async () => {
    settleMock.mockResolvedValue({ invoice: {}, fullySettled: false });
    await regler();
    await waitFor(() => expect(settleMock).toHaveBeenCalledTimes(1));
    expect(settleMock.mock.calls[0][0]).toBe(5);
    expect(settleMock.mock.calls[0][1]).toMatchObject({ version: 3 });
  });

  it("409 : dialogue fermé, liste relue, l'avis nomme la facture et dit « non enregistré » (M4, M5)", async () => {
    settleMock.mockRejectedValue(conflit);
    const r = await regler();
    const avis = await r.findByTestId("due-dates-settle-conflict");
    expect(avis.getAttribute("role")).toBe("alert");
    expect(avis.textContent).toContain("F-2026-005");
    expect(avis.textContent).toContain("n’a pas été enregistré");
    expect(avis.textContent).not.toContain("n’est pas connue");
    expect(r.queryByTestId("due-dates-settle-unknown")).toBeNull();
    await waitFor(() => expect(r.queryByTestId("settle-confirm")).toBeNull());
    await waitFor(() => expect(listDueDatesMock.mock.calls.length).toBeGreaterThan(1));
    await waitFor(() =>
      expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false),
    );
    expect(r.queryByTestId("due-dates-settle-conflict-reread")).toBeNull();
  });

  it("revue P1 (E1-7) : après un conflit relu, le règlement suivant part avec la version de la ligne relue", async () => {
    listDueDatesMock
      .mockResolvedValueOnce(response([item()]))
      .mockResolvedValue(response([item({ version: 4 })]));
    settleMock.mockRejectedValueOnce(conflit);
    const r = await regler();
    await r.findByTestId("due-dates-settle-conflict");
    await waitFor(() =>
      expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false),
    );
    settleMock.mockResolvedValueOnce({ invoice: {}, fullySettled: false });
    await fireEvent.click(r.getByTestId("due-dates-settle-5"));
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    await waitFor(() => expect(settleMock).toHaveBeenCalledTimes(2));
    expect(settleMock.mock.calls[0][1]).toMatchObject({ version: 3 });
    expect(settleMock.mock.calls[1][1]).toMatchObject({ version: 4 });
  });

  // Revue P1 (A1-2). Sur l'échéancier, `conflict-blocked` ne coexiste jamais avec une ligne
  // affichée : la liste est masquée pendant la relecture (`{#if loading}`), vidée si elle
  // échoue, et un chargement réussi lève l'état (C-e345-10). La garantie qui compte — aucun
  // « Régler » sur une liste qui n'a pas été relue — se prouve donc par l'absence du geste ;
  // le `disabled` de `settleBlocked` n'y est pas observable (mutation M7, équivalente).
  it("revue P1 (A1-2) et P2 (B2-1) : pendant la relecture d'après un 409, aucun « Régler », et l'avis dit « en cours », non « échoué »", async () => {
    let rendre: (v: unknown) => void = () => {};
    listDueDatesMock
      .mockResolvedValueOnce(response([item()]))
      .mockReturnValue(new Promise((resolve) => (rendre = resolve)));
    settleMock.mockRejectedValue(conflit);
    const r = await regler();
    await waitFor(() =>
      expect(r.getByTestId("due-dates-settle-conflict").textContent).toContain(
        "Relecture de la liste en cours",
      ),
    );
    const enCours = r.getByTestId("due-dates-settle-conflict").textContent ?? "";
    expect(enCours).toContain("F-2026-005");
    expect(enCours).not.toContain("n’a pas pu être relue");
    expect(r.queryByTestId("due-dates-settle-conflict-reread")).toBeNull();
    expect(r.queryByTestId("due-dates-settle-5")).toBeNull();
    rendre(response([item({ version: 4 })]));
    await waitFor(() =>
      expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false),
    );
    expect(r.getByTestId("due-dates-settle-conflict").textContent).toContain(
      "La liste a été relue",
    );
  });

  it("revue P1 (E1-5) : une relecture supplantée par un filtre réussi lève l'état bloquant", async () => {
    let rendreRelecture: (v: unknown) => void = () => {};
    listDueDatesMock
      .mockResolvedValueOnce(response([item()]))
      .mockReturnValueOnce(new Promise((resolve) => (rendreRelecture = resolve)))
      .mockResolvedValue(response([item({ version: 4 })]));
    settleMock.mockRejectedValue(conflit);
    const r = await regler();
    await r.findByTestId("due-dates-settle-conflict");
    // Pendant la relecture, un autre filtre part et réussit : il la supplante.
    await fireEvent.click(r.getByRole("tab", { name: "Toutes" }));
    await waitFor(() =>
      expect(r.getByTestId("due-dates-settle-conflict").textContent).toContain(
        "La liste a été relue",
      ),
    );
    expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false);
    // La relecture supplantée qui rend enfin sa réponse ne change plus rien.
    rendreRelecture(response([item({ version: 3 })]));
    await waitFor(() =>
      expect(r.getByTestId("due-dates-settle-conflict").textContent).toContain(
        "La liste a été relue",
      ),
    );
  });

  it("revue P3 (C3-1) : une relecture supplantée reste « en cours » jusqu'à l'issue du chargement qui l'emporte", async () => {
    let rendreRelecture: (v: unknown) => void = () => {};
    let rejeterFiltre: (e: unknown) => void = () => {};
    listDueDatesMock
      .mockResolvedValueOnce(response([item()]))
      .mockReturnValueOnce(new Promise((resolve) => (rendreRelecture = resolve)))
      .mockReturnValueOnce(new Promise((_, reject) => (rejeterFiltre = reject)));
    settleMock.mockRejectedValue(conflit);
    const r = await regler();
    await r.findByTestId("due-dates-settle-conflict");
    // Pendant la relecture, un filtre part : il la supplante, et reste en attente.
    await fireEvent.click(r.getByRole("tab", { name: "Toutes" }));
    await waitFor(() => expect(listDueDatesMock).toHaveBeenCalledTimes(3));
    // La relecture supplantée rend sa réponse : la relecture n'est pas finie pour autant.
    rendreRelecture(response([item({ version: 3 })]));
    await new Promise((resolve) => setTimeout(resolve, 0));
    const pendant = r.getByTestId("due-dates-settle-conflict").textContent ?? "";
    expect(pendant).toContain("Relecture de la liste en cours");
    expect(pendant).not.toContain("n’a pas pu être relue");
    expect(r.queryByTestId("due-dates-settle-conflict-reread")).toBeNull();
    // Le chargement qui l'emporte échoue : alors seulement, l'avis d'échec et *Relire*.
    rejeterFiltre({ code: "NETWORK_ERROR", status: 0, message: "x" });
    await r.findByTestId("due-dates-settle-conflict-reread");
    expect(r.getByTestId("due-dates-settle-conflict").textContent).toContain(
      "la liste n’a pas pu être relue",
    );
  });

  it("revue P4 : une relecture supplantée par un filtre qui réussit reste « en cours », puis passe à « relue »", async () => {
    let rendreRelecture: (v: unknown) => void = () => {};
    let rendreFiltre: (v: unknown) => void = () => {};
    listDueDatesMock
      .mockResolvedValueOnce(response([item()]))
      .mockReturnValueOnce(new Promise((resolve) => (rendreRelecture = resolve)))
      .mockReturnValueOnce(new Promise((resolve) => (rendreFiltre = resolve)));
    settleMock.mockRejectedValue(conflit);
    const r = await regler();
    await r.findByTestId("due-dates-settle-conflict");
    await fireEvent.click(r.getByRole("tab", { name: "Toutes" }));
    await waitFor(() => expect(listDueDatesMock).toHaveBeenCalledTimes(3));
    rendreRelecture(response([item({ version: 3 })]));
    await new Promise((resolve) => setTimeout(resolve, 0));
    const pendant = r.getByTestId("due-dates-settle-conflict").textContent ?? "";
    expect(pendant).toContain("Relecture de la liste en cours");
    expect(r.queryByTestId("due-dates-settle-conflict-reread")).toBeNull();
    // Le chargement qui l'emporte réussit : l'état se lève, la ligne relue est offerte.
    rendreFiltre(response([item({ version: 4 })]));
    await waitFor(() =>
      expect(r.getByTestId("due-dates-settle-conflict").textContent).toContain(
        "La liste a été relue",
      ),
    );
    expect(r.queryByTestId("due-dates-settle-conflict-reread")).toBeNull();
    expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false);
  });

  it("409 puis liste illisible : `conflict-blocked`, puis « Relire » mène au conflit, non à l'issue inconnue", async () => {
    settleMock.mockRejectedValue(conflit);
    const r = render(Page);
    await fireEvent.click(await r.findByTestId("due-dates-settle-5"));
    listDueDatesMock.mockRejectedValue({ code: "NETWORK_ERROR", status: 0, message: "x" });
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));

    await r.findByTestId("due-dates-settle-conflict-reread");
    const bloque = r.getByTestId("due-dates-settle-conflict");
    expect(bloque.textContent).toContain("F-2026-005");
    expect(bloque.textContent).toContain("la liste n’a pas pu être relue");
    expect(bloque.textContent).not.toContain("n’est pas connue");
    expect(r.queryByTestId("due-dates-settle-unknown")).toBeNull();

    listDueDatesMock.mockResolvedValue(response([item({ version: 4 })]));
    await fireEvent.click(r.getByTestId("due-dates-settle-conflict-reread"));
    await waitFor(() =>
      expect((r.getByTestId("due-dates-settle-5") as HTMLButtonElement).disabled).toBe(false),
    );
    const relu = r.getByTestId("due-dates-settle-conflict");
    expect(relu.textContent).toContain("La liste a été relue");
    expect(relu.textContent).not.toContain("n’est pas connue");
    expect(r.queryByTestId("due-dates-settle-conflict-reread")).toBeNull();

    // Le règlement suivant part avec la version relue.
    settleMock.mockReset();
    settleMock.mockResolvedValue({ invoice: {}, fullySettled: false });
    await fireEvent.click(r.getByTestId("due-dates-settle-5"));
    await fireEvent.change(await r.findByTestId("settle-type"), {
      target: { value: "internal_account" },
    });
    await fireEvent.change(await r.findByTestId("settle-account"), { target: { value: "1" } });
    await fireEvent.click(r.getByTestId("settle-confirm"));
    await waitFor(() => expect(settleMock).toHaveBeenCalledTimes(1));
    expect(settleMock.mock.calls[0][1]).toMatchObject({ version: 4 });
  });
});
