/**
 * Story 34-11b (#972, AC-5, AC-7) — tenir une carte de crédit : le relevé saisi à la main, les
 * achats passés par « Écriture manuelle », le prélèvement mensuel passé **une fois** côté banque
 * et **rattaché** côté carte, le solde de la carte égal au relevé.
 *
 * Parcours, montants synthétiques (dette `P` unique à chaque passage, la base étant partagée
 * entre specs ; achats 120.00 et 80.00) :
 * 1. par l'API : un passif neuf, une carte liée, une écriture d'ouverture qui porte la dette
 *    `P` (la Story 34-2 — solde d'ouverture à la création — n'est pas dans cette branche :
 *    choix C-e34b-6) ; un relevé de la banque, un débit de `P` ;
 * 2. à l'écran : le relevé de la carte saisi à la main — départ `-P`, achats au débit, paiement
 *    reçu de `P` au crédit, arrivée `-200.00` ;
 * 3. *Réconciliation*, banque : *Écriture manuelle* du débit sur le passif de la carte ;
 * 4. *Réconciliation*, carte : chaque achat par *Écriture manuelle* sur une charge ; la ligne du
 *    paiement porte l'avertissement (b′) ; *Rattacher à une écriture existante* liste l'écriture
 *    du prélèvement, rien n'est choisi d'office (AC-7) ; on la choisit ; la ligne part ;
 * 5. le solde de la carte vaut `-200.00`, sans écart au relevé (`statementGap`).
 *
 * ⚠️ La base E2E porte déjà des écritures : la portée `SingleBankLedger` de la 34-2 n'y est pas
 * exercée (fiche, AC-5). En `finally` : les rapprochements annulés (le côté carte retire son lien,
 * le côté banque contre-passe), l'écriture d'ouverture contre-passée, les relevés supprimés.
 *
 * Captures (`KESH_E2E_CAPTURES_DIR`, sinon la sortie du test) : le relevé saisi, la ligne du
 * paiement signalée, la fenêtre de rattachement, la liste des comptes. Schéma clair seulement.
 *
 * Pré-requis : MariaDB up + KESH_TEST_MODE=true (cf. docs/testing.md).
 */

import { expect, test, type Page, type TestInfo } from "@playwright/test";
import path from "path";
import {
  seedTestState,
  clearAuthStorage,
  authedApiContext,
  disposeContextSafe,
} from "./helpers/test-state";
import {
  createManualDebitStatementViaApi,
  todayInZurich,
  wiredBankAccountViaApi,
} from "./helpers/api-fixtures";

const TEST_IBAN = "CH4431999123000889012";

test.beforeAll(async () => {
  await seedTestState("with-company");
});

test.afterEach(async ({ page }) => {
  await clearAuthStorage(page);
});

async function login(page: Page): Promise<void> {
  await page.goto("/login");
  await page.fill("#username", "admin");
  await page.fill("#password", "admin123");
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/");
}

function capturePath(testInfo: TestInfo, name: string): string {
  const dossier = process.env.KESH_E2E_CAPTURES_DIR;
  return dossier
    ? path.join(dossier, `${name}.png`)
    : testInfo.outputPath(`${name}.png`);
}

type Account = {
  id: number;
  number: string;
  accountType: string;
  active: boolean;
  postable: boolean;
};

/** « Écriture manuelle » de la ligne `ligne`, sur le compte `numero`. */
async function ecritureManuelle(
  page: Page,
  ligne: ReturnType<Page["locator"]>,
  numero: string,
  libelle: string,
) {
  await ligne.getByTestId("manual-match-button").click();
  const modal = page.getByTestId("manual-match-modal");
  await expect(modal).toBeVisible();
  const champ = modal.locator('input[aria-autocomplete="list"]');
  await champ.focus();
  await champ.fill(numero);
  const option = modal.getByRole("option").filter({ hasText: numero });
  await expect(option).toHaveCount(1);
  await option.dispatchEvent("mousedown");
  await modal.getByTestId("manual-match-description-input").fill(libelle);
  await modal.getByTestId("manual-match-submit").click();
  await expect(modal).toBeHidden();
  await expect(ligne).toHaveCount(0);
}

test("une carte de crédit : relevé saisi, achats, prélèvement rattaché des deux côtés, solde égal au relevé (#972)", async ({
  page,
}, testInfo) => {
  await login(page);
  const ctx = await authedApiContext(page);
  const unique = `${Date.now()}${Math.floor(Math.random() * 1000)}`;
  const cents = 100000 + (Date.now() % 50000);
  const dette = (cents / 100).toFixed(2);
  const today = todayInZurich();
  const [y, m] = today.split("-");
  const lastDay = new Date(Date.UTC(Number(y), Number(m), 0)).getUTCDate();
  const periodFrom = `${y}-${m}-01`;
  const periodTo = `${y}-${m}-${String(lastDay).padStart(2, "0")}`;

  let ouverture: number | null = null;
  let banqueImport: number | null = null;
  let carteImport: number | null = null;
  const aAnnuler: number[] = [];
  try {
    const bankAccountId = await wiredBankAccountViaApi(page, TEST_IBAN);
    const accounts = (await (
      await ctx.get("/api/v1/accounts")
    ).json()) as Account[];
    const charge = accounts.find(
      (a) => a.accountType === "Expense" && a.active && a.postable,
    );
    expect(charge, "un compte de charge imputable").toBeTruthy();
    const numero = `2${unique.slice(-6)}`;
    const passifRes = await ctx.post("/api/v1/accounts", {
      data: {
        number: numero,
        name: `Carte de crédit ${unique}`,
        accountType: "Liability",
      },
    });
    expect(passifRes.ok(), `passif : ${await passifRes.text()}`).toBeTruthy();
    const passifId = ((await passifRes.json()) as { id: number }).id;
    const carteRes = await ctx.post("/api/v1/bank-accounts", {
      data: {
        accountKind: "credit_card",
        bankName: `Visa ${unique}`,
        cardLast4: unique.slice(-4),
        journalAccountId: passifId,
      },
    });
    expect(carteRes.status(), await carteRes.text()).toBe(201);
    const carteId = ((await carteRes.json()) as { id: number }).id;

    // La dette d'ouverture P sur le passif de la carte.
    const ouv = await ctx.post("/api/v1/journal-entries", {
      data: {
        entryDate: periodFrom,
        journal: "OD",
        description: `Dette d'ouverture de la carte ${unique}`,
        lines: [
          { accountId: charge!.id, debit: dette, credit: "0" },
          { accountId: passifId, debit: "0", credit: dette },
        ],
      },
    });
    expect(ouv.status(), await ouv.text()).toBe(201);
    ouverture = ((await ouv.json()) as { id: number }).id;

    // Le relevé de la banque : le prélèvement de P.
    banqueImport = await createManualDebitStatementViaApi(page, {
      bankAccountId,
      bookingDate: today,
      amount: dette,
      counterpartyName: `Émetteur ${unique}`,
      details: `Prélèvement carte ${unique}`,
    });
    const debitBanque = (
      (await (
        await ctx.get(`/api/v1/bank-imports/${banqueImport}`)
      ).json()) as {
        transactions: Array<{ id: number }>;
      }
    ).transactions[0].id;

    // 2. Le relevé de la carte, saisi à la main.
    await page.goto("/bank-import");
    await page.getByTestId("bank-import-manual-link").click();
    await page
      .getByTestId("bank-import-manual-account")
      .selectOption(carteId.toString());
    await page.getByTestId("bank-import-manual-period-from").fill(periodFrom);
    await page.getByTestId("bank-import-manual-period-to").fill(periodTo);
    await page
      .getByTestId("bank-import-manual-statement-number")
      .fill(`VISA-${unique}`);
    await page
      .getByTestId("bank-import-manual-opening-balance")
      .fill(`-${dette}`);
    await page
      .getByTestId("bank-import-manual-closing-balance")
      .fill("-200.00");
    const lignes = [
      { details: `Librairie ${unique}`, debit: "120.00" },
      { details: `Restaurant ${unique}`, debit: "80.00" },
      { details: `Paiement reçu ${unique}`, credit: dette },
    ];
    for (let i = 0; i < lignes.length; i++) {
      if (i > 0) await page.getByTestId("bank-import-manual-add-line").click();
      const l = page.getByTestId("bank-import-manual-line").nth(i);
      await l.getByTestId("bank-import-manual-line-booking-date").fill(today);
      await l
        .getByTestId("bank-import-manual-line-details")
        .fill(lignes[i].details);
      if (lignes[i].debit)
        await l
          .getByTestId("bank-import-manual-line-debit")
          .fill(lignes[i].debit!);
      if (lignes[i].credit)
        await l
          .getByTestId("bank-import-manual-line-credit")
          .fill(lignes[i].credit!);
    }
    await expect(page.getByTestId("bank-import-manual-gap")).toHaveText(
      "0.00 CHF",
    );
    await page.getByTestId("bank-import-manual-verify").click();
    const ecartOuverture = page.getByTestId(
      "bank-import-manual-confirm-opening-gap",
    );
    await expect(
      page.getByTestId("bank-import-manual-preview-ok").or(ecartOuverture),
    ).toBeVisible();
    if (await ecartOuverture.isVisible()) await ecartOuverture.check();
    await page
      .getByTestId("bank-import-manual-form")
      .screenshot({ path: capturePath(testInfo, "34-11b-releve-saisi") });
    await expect(page.getByTestId("bank-import-manual-save")).toBeEnabled();
    await page.getByTestId("bank-import-manual-save").click();
    await expect(page).toHaveURL(/\/bank-import$/);
    const premiere = page.getByTestId("bank-import-list-row").first();
    carteImport = Number(await premiere.getAttribute("data-import-id"));
    const txCarte = (
      (await (await ctx.get(`/api/v1/bank-imports/${carteImport}`)).json()) as {
        transactions: Array<{ id: number; amount: string; details: string }>;
      }
    ).transactions;
    expect(txCarte.map((t) => Number(t.amount)).sort((a, b) => a - b)).toEqual(
      [-120, -80, Number(dette)].sort((a, b) => a - b),
    );
    const paiement = txCarte.find((t) => Number(t.amount) > 0)!.id;

    // 3. Côté banque : Écriture manuelle sur le passif de la carte.
    await page.goto("/reconciliation");
    await page
      .getByTestId("bank-account-select")
      .selectOption(bankAccountId.toString());
    const ligneBanque = page.locator(
      `[data-testid="reconciliation-row"][data-tx-id="${debitBanque}"]`,
    );
    await expect(ligneBanque).toBeVisible({ timeout: 10000 });
    await ecritureManuelle(
      page,
      ligneBanque,
      numero,
      `Prélèvement carte ${unique}`,
    );
    aAnnuler.push(debitBanque);
    const virement = (
      (await (
        await ctx.get(`/api/v1/bank-imports/${banqueImport}`)
      ).json()) as {
        transactions: Array<{ id: number; matchedEntryId: number | null }>;
      }
    ).transactions[0].matchedEntryId;
    expect(virement, "le débit de la banque est rapproché").toBeTruthy();

    // 4. Côté carte : les achats, puis le paiement reçu.
    await page
      .getByTestId("bank-account-select")
      .selectOption(carteId.toString());
    for (const achat of txCarte.filter((t) => Number(t.amount) < 0)) {
      const l = page.locator(
        `[data-testid="reconciliation-row"][data-tx-id="${achat.id}"]`,
      );
      await expect(l).toBeVisible({ timeout: 10000 });
      await ecritureManuelle(page, l, charge!.number, achat.details);
      aAnnuler.push(achat.id);
    }
    const lignePaiement = page.locator(
      `[data-testid="reconciliation-row"][data-tx-id="${paiement}"]`,
    );
    await expect(lignePaiement).toBeVisible();
    await expect(
      page.locator(
        `[data-testid="already-booked-warning"][data-tx-id="${paiement}"]`,
      ),
    ).toHaveText(
      "Cette transaction est probablement l’autre côté d’un virement entre vos comptes : l’écriture de ce virement est déjà rapprochée d’une transaction d’un autre de vos comptes bancaires. Rattachez-la à cette écriture (« Rattacher à une écriture existante ») plutôt que de passer une écriture : le virement serait compté deux fois.",
    );
    await page
      .locator("tbody")
      .screenshot({ path: capturePath(testInfo, "34-11b-paiement-signale") });

    await lignePaiement.getByTestId("link-settlement-button").click();
    const modal = page.getByTestId("link-settlement-modal");
    const choix = modal.locator(
      `[data-testid="entry-candidate-radio"][data-entry-id="${virement}"]`,
    );
    await expect(choix).toBeVisible({ timeout: 10000 });
    await expect(choix).not.toBeChecked();
    await expect(modal.getByTestId("link-submit")).toBeDisabled();
    await choix.check();
    await expect(modal.getByTestId("entry-cancel-unlink-only")).toBeVisible();
    await expect(modal.getByTestId("entry-cancel-blocked-ack")).toHaveCount(0);
    await modal.screenshot({
      path: capturePath(testInfo, "34-11b-rattachement-prelevement"),
    });
    await modal.getByTestId("link-submit").click();
    await expect(modal).toBeHidden();
    await expect(lignePaiement).toHaveCount(0);
    aAnnuler.unshift(paiement);

    // 5. Le solde de la carte égale le relevé.
    const comptes = (await (
      await ctx.get("/api/v1/bank-accounts")
    ).json()) as Array<{
      id: number;
      currentBalance: string | null;
      statementGap: string | null;
    }>;
    const carte = comptes.find((c) => c.id === carteId)!;
    expect(Number(carte.currentBalance)).toBe(-200);
    expect(
      carte.statementGap === null || Number(carte.statementGap) === 0,
      `écart : ${carte.statementGap}`,
    ).toBe(true);
    await page.goto("/bank-accounts");
    await expect(page.getByTestId(`balance-cell-${carteId}`)).toBeVisible();
    await page.screenshot({
      path: capturePath(testInfo, "34-11b-liste-des-comptes"),
      fullPage: true,
    });
  } finally {
    for (const tx of aAnnuler) {
      await ctx.post(`/api/v1/reconciliation/transactions/${tx}/cancel`, {
        data: {},
      });
    }
    if (ouverture !== null)
      await ctx.post(`/api/v1/journal-entries/${ouverture}/reverse`);
    if (carteImport !== null)
      await ctx.delete(`/api/v1/bank-imports/${carteImport}`);
    if (banqueImport !== null)
      await ctx.delete(`/api/v1/bank-imports/${banqueImport}`);
    await disposeContextSafe(ctx);
  }
});
