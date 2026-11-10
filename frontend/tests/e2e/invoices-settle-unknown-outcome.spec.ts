import { expect, test } from "@playwright/test";
import {
  seedTestState,
  clearAuthStorage,
  authedApiContext,
  disposeContextSafe,
} from "./helpers/test-state";

/**
 * Revue P1 de la Story 33-6 (E-4 = A-4) : la réponse d'un règlement coupée APRÈS
 * l'écriture. Le serveur écrit, le navigateur ne le sait pas : l'écran ne doit pas
 * laisser le même geste à portée de clic. Depuis la Story 34-5 (#951), le POST porte
 * la version et le serveur refuserait le second envoi (409) ; l'écran relit encore,
 * pour montrer ce qui est écrit (`invoices-settle-version.spec.ts`).
 *
 * La coupure est réelle : `route.fetch()` envoie la requête au serveur, puis
 * `route.abort()` jette sa réponse.
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
        name: `Issue inconnue ${Date.now()}-${Math.floor(Math.random() * 1e6)}`,
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

/** Laisse le serveur écrire le règlement, puis jette sa réponse. */
async function cutResponseAfterWrite(page: import("@playwright/test").Page, id: number) {
  await page.route(`**/api/v1/invoices/${id}/settlements`, async (route) => {
    if (route.request().method() !== "POST") return route.fallback();
    const response = await route.fetch();
    expect(response.status(), "le serveur a bien écrit").toBe(200);
    await route.abort("connectionreset");
  });
}

async function fillAndConfirm(page: import("@playwright/test").Page) {
  await page.getByTestId("settle-type").selectOption("internal_account");
  await page.getByTestId("settle-account").selectOption({ index: 1 });
  await page.getByTestId("settle-confirm").click();
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

test("fiche facture : réponse coupée après l'écriture, aucun second geste à portée de clic", async ({
  page,
}) => {
  await login(page);
  const id = await validatedInvoice(page);
  await cutResponseAfterWrite(page, id);
  await page.goto(`/invoices/${id}`);
  await page.getByTestId("settle-open").click();
  await fillAndConfirm(page);

  await expect(page.getByTestId("settle-unknown-outcome")).toBeVisible();
  await expect(page.getByTestId("settle-confirm")).toHaveCount(0);
  // Le règlement était écrit : la relecture le montre, et la facture est soldée.
  await expect(page.getByTestId("invoice-amount-settled")).toContainText("108");
  await expect(page.getByTestId("settle-open")).toHaveCount(0);
  expect(await settlementCount(page, id)).toBe(1);
});

test("fiche facture : règlement partiel coupé, le bouton rouvert propose le reste relu", async ({
  page,
}) => {
  await login(page);
  const id = await validatedInvoice(page);
  await cutResponseAfterWrite(page, id);
  await page.goto(`/invoices/${id}`);
  await page.getByTestId("settle-open").click();
  await page.locator("#settle-amount").fill("8.10");
  await fillAndConfirm(page);

  await expect(page.getByTestId("settle-unknown-outcome")).toBeVisible();
  await expect(page.getByTestId("settle-confirm")).toHaveCount(0);
  // Partiel : la facture reste à régler, c'est l'état `verify` qui rouvre le geste,
  // et le montant proposé est le reste relu, non celui d'avant l'envoi.
  await expect(page.getByTestId("settle-open")).toBeEnabled();
  await page.getByTestId("settle-open").click();
  await expect(page.locator("#settle-amount")).toHaveValue("100.00");
  await expect(page.getByTestId("settle-unknown-outcome")).toBeVisible();
  expect(await settlementCount(page, id)).toBe(1);
});

test("échéancier : réponse coupée après l'écriture, aucun second geste à portée de clic", async ({
  page,
}) => {
  await login(page);
  const id = await validatedInvoice(page);
  await cutResponseAfterWrite(page, id);
  await page.goto("/invoices/due-dates");
  await page.getByTestId(`due-dates-settle-${id}`).click();
  await fillAndConfirm(page);

  await expect(page.getByTestId("due-dates-settle-unknown")).toBeVisible();
  await expect(page.getByTestId("settle-confirm")).toHaveCount(0);
  // Soldée par le règlement écrit : la liste relue n'en propose plus le geste.
  await expect(page.getByTestId(`due-dates-settle-${id}`)).toHaveCount(0);
  expect(await settlementCount(page, id)).toBe(1);
});
