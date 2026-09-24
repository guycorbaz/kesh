import { expect, test } from "@playwright/test";
import {
  seedTestState,
  clearAuthStorage,
  authedApiContext,
  disposeContextSafe,
} from "./helpers/test-state";

/**
 * Tests E2E — Annuler un règlement client, Story 25-3-a-1 (#414).
 *
 * De bout en bout : une facture validée, réglée par l'API, puis annulée depuis
 * la fiche — la liste des règlements, la confirmation, et la facture qui
 * redevient « à régler ». ⛔ C'est le seul test qui voit la valeur traverser la
 * frontière HTTP dans les deux sens : vitest teste l'écran sur des réponses
 * simulées, les tests Rust le serveur sans écran.
 *
 * ⚠️ Les contrôles se ciblent par `data-testid`, jamais par leur libellé
 * traduit (règle du dépôt, #326).
 */

test.beforeAll(async () => {
  await seedTestState("with-data");
});

test.afterEach(async ({ page }) => {
  await clearAuthStorage(page);
});

async function login(page: import("@playwright/test").Page) {
  await page.goto("/login");
  await page.fill("#username", "admin");
  await page.fill("#password", "admin123");
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/");
}

function today(): string {
  return new Date().toISOString().slice(0, 10);
}

/** Facture validée de 100.00 HT (108.10 TTC), réglée en entier ; rend son id. */
async function settledInvoice(
  page: import("@playwright/test").Page,
): Promise<number> {
  const ctx = await authedApiContext(page);
  try {
    const contact = await ctx.post("/api/v1/contacts", {
      data: {
        contactType: "Entreprise",
        name: `Annulation ${Date.now()}-${Math.floor(Math.random() * 1e6)}`,
        isClient: true,
        isSupplier: false,
      },
    });
    expect(contact.ok(), `contact: ${contact.status()}`).toBeTruthy();
    const contactId = (await contact.json()).id as number;

    const created = await ctx.post("/api/v1/invoices", {
      data: {
        contactId,
        date: today(),
        dueDate: today(),
        paymentTerms: null,
        lines: [
          {
            description: "Prestation",
            quantity: "1",
            unitPrice: "100.00",
            vatRate: "8.10",
          },
        ],
      },
    });
    expect(created.ok(), `facture: ${created.status()}`).toBeTruthy();
    const id = (await created.json()).id as number;
    const validated = await ctx.post(`/api/v1/invoices/${id}/validate`);
    expect(validated.ok(), `validation: ${validated.status()}`).toBeTruthy();

    // Un compte de LIQUIDITÉS (classe 10), actif et imputable : c'est ce que
    // l'encaissement exige de sa contrepartie. Le numéro exact dépend du plan
    // comptable du préréglage — ne pas le figer.
    const accounts = await ctx.get("/api/v1/accounts?includeArchived=false");
    expect(accounts.ok(), `comptes: ${accounts.status()}`).toBeTruthy();
    const cash = (
      (await accounts.json()) as {
        id: number;
        number: string;
        accountType: string;
        active: boolean;
        postable: boolean;
      }[]
    ).find(
      (a) =>
        a.active &&
        a.postable &&
        a.accountType === "Asset" &&
        a.number.startsWith("10"),
    );
    expect(cash, "un compte de liquidités imputable").toBeTruthy();
    const settled = await ctx.post(`/api/v1/invoices/${id}/settlements`, {
      data: {
        settlementType: "internal_account",
        accountId: cash!.id,
        amount: "108.10",
        settledOn: today(),
      },
    });
    expect(settled.ok(), `règlement: ${settled.status()}`).toBeTruthy();
    return id;
  } finally {
    await disposeContextSafe(ctx);
  }
}

test.describe("Annuler un règlement client — Story 25-3-a-1", () => {
  test("régler puis annuler : la facture redevient à régler", async ({
    page,
  }) => {
    await login(page);
    const id = await settledInvoice(page);

    await page.goto(`/invoices/${id}`);
    // Soldée : pas de bouton de règlement, mais la ligne du règlement et son
    // bouton d'annulation.
    await expect(page.getByTestId("settle-open")).toHaveCount(0);
    await expect(page.getByTestId("invoice-settlement-row")).toHaveCount(1);

    await page.getByTestId("invoice-settlement-cancel").click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.getByTestId("invoice-settlement-cancel-confirm").click();

    // La facture est RELUE : le bouton de règlement revient, la liste se vide.
    await expect(page.getByTestId("settle-open")).toBeVisible({
      timeout: 5000,
    });
    await expect(page.getByTestId("invoice-settlements")).toHaveCount(0);

    // Et le serveur le confirme — le reste dû est entier.
    const ctx = await authedApiContext(page);
    try {
      const inv = await (await ctx.get(`/api/v1/invoices/${id}`)).json();
      expect(inv.paidAt).toBeNull();
      expect(Number(inv.amountDue)).toBeCloseTo(108.1, 2);
    } finally {
      await disposeContextSafe(ctx);
    }
  });
});
