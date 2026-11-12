import { expect, test } from "@playwright/test";
import {
  seedTestState,
  clearAuthStorage,
  authedApiContext,
  disposeContextSafe,
} from "./helpers/test-state";
import { invoiceVersionViaApi } from "./helpers/api-fixtures";

/**
 * Story 34-5 (#951) — un second envoi du même règlement sans relecture n'écrit pas un
 * second règlement. `POST /invoices/{id}/settlements` porte la version de la facture ; une
 * version périmée rend `409 OPTIMISTIC_LOCK_CONFLICT`, sans rien écrire, et l'écran
 * ferme le dialogue, relit, et dit que le règlement n'a pas été enregistré.
 *
 * Seul un E2E voit la version traverser la frontière HTTP : Vitest teste le corps
 * construit, les tests Rust le contrôle du serveur.
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

/** Une facture validée de 100.00 HT (108.10 TTC), sans règlement ; rend son id. */
async function validatedInvoice(page: import("@playwright/test").Page): Promise<number> {
  const ctx = await authedApiContext(page);
  try {
    const contact = await ctx.post("/api/v1/contacts", {
      data: {
        contactType: "Entreprise",
        name: `Version règlement ${Date.now()}-${Math.floor(Math.random() * 1e6)}`,
        isClient: true,
        isSupplier: false,
      },
    });
    expect(contact.ok(), `contact: ${contact.status()}`).toBeTruthy();
    const created = await ctx.post("/api/v1/invoices", {
      data: {
        contactId: (await contact.json()).id,
        date: today(),
        dueDate: today(),
        paymentTerms: null,
        lines: [
          { description: "Prestation", quantity: "1", unitPrice: "100.00", vatRate: "8.10" },
        ],
      },
    });
    expect(created.ok(), `facture: ${created.status()}`).toBeTruthy();
    const id = (await created.json()).id as number;
    const validated = await ctx.post(`/api/v1/invoices/${id}/validate`);
    expect(validated.ok(), `validation: ${validated.status()}`).toBeTruthy();
    return id;
  } finally {
    await disposeContextSafe(ctx);
  }
}

/** Le compte de caisse 1000 du seed. */
async function caisse(ctx: import("@playwright/test").APIRequestContext): Promise<number> {
  const accounts = (await (await ctx.get("/api/v1/accounts")).json()) as {
    id: number;
    number: string;
  }[];
  const found = accounts.find((a) => a.number === "1000");
  expect(found, "compte 1000 du seed").toBeTruthy();
  return found!.id;
}

/** `POST` d'un règlement de `amount` par la caisse, à la `version` donnée. */
async function settleByApi(
  ctx: import("@playwright/test").APIRequestContext,
  id: number,
  amount: string,
  version: number,
) {
  return ctx.post(`/api/v1/invoices/${id}/settlements`, {
    data: {
      settlementType: "internal_account",
      accountId: await caisse(ctx),
      amount,
      settledOn: today(),
      version,
    },
  });
}

async function settlementCount(page: import("@playwright/test").Page, id: number) {
  const ctx = await authedApiContext(page);
  try {
    const res = await ctx.get(`/api/v1/invoices/${id}/settlements`);
    expect(res.ok()).toBeTruthy();
    return ((await res.json()) as unknown[]).length;
  } finally {
    await disposeContextSafe(ctx);
  }
}

async function fillAndConfirm(page: import("@playwright/test").Page, amount: string) {
  await page.locator("#settle-amount").fill(amount);
  await page.getByTestId("settle-type").selectOption("internal_account");
  await page.getByTestId("settle-account").selectOption({ index: 1 });
  await page.getByTestId("settle-confirm").click();
}

test("AC-2 : le même règlement renvoyé à la version d'avant est refusé, un seul règlement écrit", async ({
  page,
}) => {
  await login(page);
  const id = await validatedInvoice(page);
  const ctx = await authedApiContext(page);
  try {
    const before = await invoiceVersionViaApi(ctx, id);
    await page.goto(`/invoices/${id}`);
    await page.getByTestId("settle-open").click();
    await fillAndConfirm(page, "8.10");
    await expect(page.getByTestId("settle-confirm")).toHaveCount(0);
    await expect(page.getByTestId("invoice-amount-settled")).toContainText("8.10");
    expect(await invoiceVersionViaApi(ctx, id)).toBe(before + 1);

    // Le second envoi du même règlement, à la version que l'écran avait lue.
    const second = await settleByApi(ctx, id, "8.10", before);
    expect(second.status(), await second.text()).toBe(409);
    expect(((await second.json()) as { error: { code: string } }).error.code).toBe(
      "OPTIMISTIC_LOCK_CONFLICT",
    );
  } finally {
    await disposeContextSafe(ctx);
  }
  expect(await settlementCount(page, id)).toBe(1);
  await page.reload();
  await expect(page.getByTestId("invoice-amount-settled")).toContainText("8.10");
});

test("fiche facture : un règlement passé ailleurs pendant la saisie rend l'avis de conflit", async ({
  page,
}) => {
  await login(page);
  const id = await validatedInvoice(page);
  await page.goto(`/invoices/${id}`);
  await page.getByTestId("settle-open").click();

  // Pendant que le dialogue est ouvert, un règlement passe par l'API.
  const ctx = await authedApiContext(page);
  try {
    const res = await settleByApi(ctx, id, "50.00", await invoiceVersionViaApi(ctx, id));
    expect(res.ok(), `règlement concurrent: ${res.status()}`).toBeTruthy();
  } finally {
    await disposeContextSafe(ctx);
  }

  await fillAndConfirm(page, "8.10");
  const avis = page.getByTestId("settle-conflict");
  await expect(avis).toBeVisible();
  await expect(avis).toContainText("n’a pas été enregistré");
  await expect(page.getByTestId("settle-unknown-outcome")).toHaveCount(0);
  await expect(page.getByTestId("settle-confirm")).toHaveCount(0);
  // Relue : la fiche montre le règlement passé ailleurs, et le geste est rouvert.
  await expect(page.getByTestId("invoice-amount-settled")).toContainText("50.00");
  await expect(page.getByTestId("settle-open")).toBeEnabled();
  expect(await settlementCount(page, id)).toBe(1);
});

test("échéancier : un règlement passé ailleurs pendant la saisie rend l'avis de conflit, qui nomme la facture", async ({
  page,
}) => {
  await login(page);
  const id = await validatedInvoice(page);
  await page.goto("/invoices/due-dates");
  await page.getByTestId(`due-dates-settle-${id}`).click();

  const ctx = await authedApiContext(page);
  let number = "";
  try {
    const res = await settleByApi(ctx, id, "50.00", await invoiceVersionViaApi(ctx, id));
    expect(res.ok(), `règlement concurrent: ${res.status()}`).toBeTruthy();
    number = ((await res.json()) as { invoice: { invoiceNumber: string } }).invoice.invoiceNumber;
  } finally {
    await disposeContextSafe(ctx);
  }

  await fillAndConfirm(page, "8.10");
  const avis = page.getByTestId("due-dates-settle-conflict");
  await expect(avis).toBeVisible();
  await expect(avis).toContainText("n’a pas été enregistré");
  await expect(avis).toContainText(number);
  await expect(page.getByTestId("settle-confirm")).toHaveCount(0);
  await expect(page.getByTestId(`due-dates-settle-${id}`)).toBeEnabled();
  expect(await settlementCount(page, id)).toBe(1);
});
