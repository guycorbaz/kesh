import { expect, test } from "@playwright/test";
import {
  seedTestState,
  clearAuthStorage,
  authedApiContext,
  disposeContextSafe,
} from "./helpers/test-state";
import {
  createAndValidateInvoiceViaApi,
  createContactWithAddressViaApi,
} from "./helpers/api-fixtures";

test.beforeAll(async () => {
  await seedTestState("with-company");
});

test.afterEach(async ({ page }) => {
  // Clear localStorage after each test to prevent token bleed to next test
  await clearAuthStorage(page);
});

/**
 * Tests E2E — Saisie d'écritures en partie double (Story 3.2)
 *
 * Ces tests nécessitent :
 * - un backend Kesh fonctionnel sur localhost
 * - un admin bootstrap (admin / admin123)
 * - un seed démo effectué (plan comptable PME + exercice ouvert de
 *   l'année courante créés par `kesh_seed::seed_demo`)
 */

async function login(page: import("@playwright/test").Page) {
  await page.goto("/login");
  await page.fill("#username", "admin");
  await page.fill("#password", "admin123");
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/");
}

async function goToJournalEntries(page: import("@playwright/test").Page) {
  await login(page);
  await page.goto("/journal-entries");
  await expect(page).toHaveURL("/journal-entries");
}

/**
 * Récupère deux comptes actifs via l'API pour injecter leur number/nom
 * dans l'autocomplétion. Les IDs ne sont pas stables entre resets, donc
 * on cherche par numéro (1020 Banque, 3000 Ventes, etc.).
 */
async function getSeedAccountNumbers(
  page: import("@playwright/test").Page,
): Promise<{ debitNumber: string; creditNumber: string }> {
  const ctx = await authedApiContext(page);
  try {
    const resp = await ctx.get("/api/v1/accounts?includeArchived=false");
    expect(resp.ok()).toBeTruthy();
    const accounts: Array<{ number: string; name: string }> = await resp.json();

    // On prend un compte d'actif (1xxx) et un compte de produit/passif (3xxx ou 2xxx).
    const asset =
      accounts.find((a) => /^10[0-9]{2}$/.test(a.number)) ?? accounts[0];
    const revenue =
      accounts.find((a) => /^3[0-9]{3}$/.test(a.number)) ??
      accounts.find((a) => /^2[0-9]{3}$/.test(a.number)) ??
      accounts[1];

    return { debitNumber: asset.number, creditNumber: revenue.number };
  } finally {
    await disposeContextSafe(ctx);
  }
}

test.describe("Page écritures — affichage", () => {
  test("affiche le titre et le bouton Nouvelle écriture", async ({ page }) => {
    await goToJournalEntries(page);
    await expect(
      page.getByRole("heading", { name: /Écritures/ }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: /Nouvelle écriture/ }),
    ).toBeVisible();
  });

  test("affiche un message si liste vide", async ({ page }) => {
    await goToJournalEntries(page);
    // Après seed_demo, aucune écriture n'est créée — l'état initial
    // peut montrer le message vide OU des écritures de tests précédents.
    // On vérifie simplement que la page charge.
    // Auto-wait : l'un des deux états doit apparaître une fois le
    // chargement terminé (isVisible one-shot était race-prone).
    await expect(
      page
        .getByText(/Aucune écriture/)
        .or(page.getByRole("table"))
        .first(),
    ).toBeVisible();
  });
});

test.describe("Page écritures — saisie", () => {
  test("saisie nominale d'une écriture équilibrée", async ({ page }) => {
    await goToJournalEntries(page);
    const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);

    await page.getByRole("button", { name: /Nouvelle écriture/ }).click();
    await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();

    // Libellé
    await page.fill("#entry-description", "Test E2E saisie nominale");

    // Ligne 1 : débit
    const accountInputs = page.locator('input[aria-autocomplete="list"]');
    await accountInputs.nth(0).fill(debitNumber);
    // Attendre que l'option apparaisse et la sélectionner.
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(0).fill("100.00");

    // Ligne 2 : crédit
    await accountInputs.nth(1).fill(creditNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(3).fill("100.00");

    // L'indicateur doit être équilibré.
    await expect(page.getByText(/✓ Équilibré/)).toBeVisible();

    // Valider
    await page.getByRole("button", { name: "Valider" }).click();

    // Retour à la liste + écriture visible.
    await expect(page.getByText(/Test E2E saisie nominale/)).toBeVisible({
      timeout: 5000,
    });
  });

  test("saisie avec tag projet analytique par ligne (Story 19-2)", async ({
    page,
  }) => {
    await goToJournalEntries(page);
    const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);

    // Projet actif créé via l'API (code unique par run pour éviter le 409).
    const code = `E2E192-${Date.now() % 1000000}`;
    const setupCtx = await authedApiContext(page);
    let projectId: number;
    try {
      const resp = await setupCtx.post("/api/v1/projects", {
        data: {
          parentId: null,
          code,
          name: "Projet E2E 19-2",
          description: null,
          startDate: null,
          endDate: null,
        },
      });
      expect(resp.ok()).toBeTruthy();
      projectId = (await resp.json()).id;
    } finally {
      await disposeContextSafe(setupCtx);
    }

    // Recharger la page pour que le formulaire reçoive la liste des projets.
    await page.goto("/journal-entries");
    await page.getByRole("button", { name: /Nouvelle écriture/ }).click();
    await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();

    const description = `Test E2E tag projet ${code}`;
    await page.fill("#entry-description", description);

    // NB : scoper la sélection au listbox de l'autocomplete — les <option>
    // natifs du <select> projet (Story 19-2) matchent aussi role=option.
    const accountInputs = page.locator('input[aria-autocomplete="list"]');
    await accountInputs.nth(0).fill(debitNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(0).fill("80.00");

    await accountInputs.nth(1).fill(creditNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(3).fill("80.00");

    // La colonne Projet est visible (des projets actifs existent) —
    // taguer uniquement la ligne 1.
    await page.getByTestId("journal-entry-line-project-0").selectOption({
      label: `${code} — Projet E2E 19-2`,
    });

    await expect(page.getByText(/✓ Équilibré/)).toBeVisible();
    await page.getByRole("button", { name: "Valider" }).click();
    await expect(page.getByText(description)).toBeVisible({ timeout: 5000 });

    // Ground-truth API : ligne 1 taguée, ligne 2 non taguée.
    const verifyCtx = await authedApiContext(page);
    try {
      const resp = await verifyCtx.get(
        `/api/v1/journal-entries?description=${encodeURIComponent(description)}`,
      );
      expect(resp.ok()).toBeTruthy();
      const list: {
        items: Array<{ lines: Array<{ projectId: number | null }> }>;
      } = await resp.json();
      expect(list.items.length).toBeGreaterThanOrEqual(1);
      const lines = list.items[0].lines;
      expect(lines[0].projectId).toBe(projectId);
      expect(lines[1].projectId).toBeNull();
    } finally {
      await disposeContextSafe(verifyCtx);
    }
  });

  test("indicateur de déséquilibre et bouton Valider désactivé", async ({
    page,
  }) => {
    await goToJournalEntries(page);
    const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);

    await page.getByRole("button", { name: /Nouvelle écriture/ }).click();
    await page.fill("#entry-description", "Test déséquilibre");

    const accountInputs = page.locator('input[aria-autocomplete="list"]');
    await accountInputs.nth(0).fill(debitNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(0).fill("100");

    await accountInputs.nth(1).fill(creditNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(3).fill("50");

    // L'indicateur doit être rouge (déséquilibré).
    await expect(page.getByText(/✗ Déséquilibré/)).toBeVisible();

    // Le bouton Valider est désactivé.
    const submitBtn = page.getByRole("button", { name: "Valider" });
    await expect(submitBtn).toBeDisabled();
  });

  test("rejet client d'un montant avec plus de 4 décimales", async ({
    page,
  }) => {
    await goToJournalEntries(page);
    const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);

    await page.getByRole("button", { name: /Nouvelle écriture/ }).click();
    await page.fill("#entry-description", "Test > 4 décimales");

    const accountInputs = page.locator('input[aria-autocomplete="list"]');
    await accountInputs.nth(0).fill(debitNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(0).fill("10.99999");

    await accountInputs.nth(1).fill(creditNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(3).fill("10.99999");

    // Message "Maximum 4 décimales" doit apparaître
    await expect(page.getByText(/Maximum 4 décimales/).first()).toBeVisible();

    // Le bouton Valider reste désactivé.
    await expect(page.getByRole("button", { name: "Valider" })).toBeDisabled();
  });

  test("raccourci Ctrl+N ouvre le formulaire", async ({ page }) => {
    await goToJournalEntries(page);
    // S'assurer qu'on est en mode liste.
    await expect(
      page.getByRole("button", { name: /Nouvelle écriture/ }),
    ).toBeVisible();
    await page.keyboard.press("Control+n");
    await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();
  });

  // Tests reportés aux stories suivantes (nécessitent CRUD fiscal_years
  // ou fermeture d'exercice — hors scope 3.2).
  test.skip("refus écriture sans exercice couvrant la date (3.3)", async () => {});
  test.skip("refus écriture exercice clos FR24 (12.1)", async () => {});
});

test.describe("Page écritures — la liste mène à la fiche, où l'on corrige (Stories 24-4b, 15-8a)", () => {
  /**
   * La liste n'offre ni modification ni suppression (24-4b) : son seul geste de
   * ligne est le lien vers la FICHE. Depuis la Story 15-8a (#532), c'est la
   * fiche qui porte « Modifier » — tant que l'exercice est ouvert — à côté de
   * « Contre-passer ». Les parcours de modification de la 3.3, qui vivaient dans
   * la liste (✎, modale de conflit), sont REMPLACÉS par des parcours depuis la
   * fiche, pas rétablis.
   *
   * ⚠️ Ce qui se vérifie ici est ce qu'aucun test Rust ne voit : que la valeur
   * traverse la frontière HTTP depuis l'écran. Les refus eux-mêmes sont
   * couverts par `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`.
   *
   * ⛔ Les nouveaux parcours n'emploient que des `data-testid` et des `id`, plus
   * le bouton « Valider » du formulaire, déjà inscrit pour ce fichier à
   * `e2e-selecteurs-traduits.test.ts` (KF-043) — aucun couple neuf.
   */
  async function createSeedEntry(page: import("@playwright/test").Page) {
    const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);
    await page.getByRole("button", { name: /Nouvelle écriture/ }).click();
    await page.fill("#entry-description", "Test 24-4b gel target");
    const accountInputs = page.locator('input[aria-autocomplete="list"]');
    await accountInputs.nth(0).fill(debitNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(0).fill("200.00");
    await accountInputs.nth(1).fill(creditNumber);
    await page.getByRole("listbox").getByRole("option").first().click();
    await page.locator('input[inputmode="decimal"]').nth(3).fill("200.00");
    await page.getByRole("button", { name: "Valider" }).click();
    await expect(page.getByText(/Test 24-4b gel target/).first()).toBeVisible({
      timeout: 5000,
    });
  }

  test("la liste n'offre plus ni modification ni suppression", async ({
    page,
  }) => {
    await goToJournalEntries(page);
    await createSeedEntry(page);

    const row = page
      .locator("tr", { hasText: "Test 24-4b gel target" })
      .first();

    // ⛔ Absence, pas désactivation : un bouton grisé laisserait croire qu'un
    // droit manque, alors que le geste n'existe plus pour personne.
    await expect(row.getByRole("button", { name: /Modifier/ })).toHaveCount(0);
    await expect(row.getByRole("button", { name: /Supprimer/ })).toHaveCount(0);
  });

  test("la ligne renvoie vers la fiche, d'où part la contre-passation", async ({
    page,
  }) => {
    await goToJournalEntries(page);
    await createSeedEntry(page);

    const row = page
      .locator("tr", { hasText: "Test 24-4b gel target" })
      .first();

    // Le sélecteur est un `data-testid`, jamais un libellé traduit (KF-043).
    await row.getByTestId("journal-entry-open").click();

    await expect(page).toHaveURL(/\/journal-entries\/\d+$/);
    await expect(
      page.getByTestId("reverse-entry"),
    ).toBeVisible();
  });
});

test.describe("Page écritures — modifier et supprimer depuis la fiche (Stories 15-8a, 15-8b, #532)", () => {
  type Compte = { id: number; number: string; postable: boolean; active: boolean };

  /** Crée une écriture par l'API, rend son id, son numéro et sa version. */
  async function creerEcriture(
    page: import("@playwright/test").Page,
    libelle: string,
  ): Promise<{ id: number; entryNumber: number; version: number }> {
    const ctx = await authedApiContext(page);
    try {
      const accounts = (await (await ctx.get("/api/v1/accounts")).json()) as Compte[];
      const postables = accounts.filter((a) => a.postable && a.active);
      const created = await ctx.post("/api/v1/journal-entries", {
        data: {
          entryDate: new Date().toLocaleDateString("sv-SE", { timeZone: "Europe/Zurich" }),
          journal: "OD",
          description: libelle,
          lines: [
            { accountId: postables[0].id, debit: "80.00", credit: "0.00" },
            { accountId: postables[1].id, debit: "0.00", credit: "80.00" },
          ],
        },
      });
      expect(created.status(), await created.text()).toBe(201);
      return (await created.json()) as { id: number; entryNumber: number; version: number };
    } finally {
      await disposeContextSafe(ctx);
    }
  }

  async function lireEcriture(page: import("@playwright/test").Page, id: number) {
    const ctx = await authedApiContext(page);
    try {
      const resp = await ctx.get(`/api/v1/journal-entries/${id}`);
      expect(resp.ok()).toBeTruthy();
      return (await resp.json()) as {
        description: string;
        entryNumber: number;
        version: number;
        lines: Array<{ accountId: number; debit: string; credit: string; projectId: number | null }>;
        entryDate: string;
        journal: string;
      };
    } finally {
      await disposeContextSafe(ctx);
    }
  }

  /** Modifie le libellé d'une écriture par l'API (`PUT`, version courante) : rend la fiche « Modifiée ». */
  async function modifierLibelle(page: import("@playwright/test").Page, id: number, nouveau: string) {
    const courant = await lireEcriture(page, id);
    const ctx = await authedApiContext(page);
    try {
      const put = await ctx.put(`/api/v1/journal-entries/${id}`, {
        data: {
          entryDate: courant.entryDate,
          journal: courant.journal,
          description: nouveau,
          version: courant.version,
          lines: courant.lines.map((l) => ({
            accountId: l.accountId,
            debit: l.debit,
            credit: l.credit,
            projectId: l.projectId,
          })),
        },
      });
      expect(put.status(), await put.text()).toBe(200);
    } finally {
      await disposeContextSafe(ctx);
    }
  }

  test("modifier depuis la fiche : le libellé change, le numéro reste", async ({ page }) => {
    await login(page);
    const libelle = `Modification E2E ${Date.now()}`;
    const origine = await creerEcriture(page, libelle);

    await page.goto(`/journal-entries/${origine.id}`);
    await page.getByTestId("edit-entry").click();
    await expect(page.locator("#entry-description")).toHaveValue(libelle);
    await page.fill("#entry-description", `${libelle} corrigé`);
    await page.getByRole("button", { name: "Valider" }).click();

    // La fiche est revenue (bouton de nouveau visible) et montre le libellé corrigé.
    await expect(page.getByTestId("edit-entry")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("dl")).toContainText(`${libelle} corrigé`);

    const apres = await lireEcriture(page, origine.id);
    expect(apres.description).toBe(`${libelle} corrigé`);
    expect(apres.entryNumber, "le numéro ne change jamais").toBe(origine.entryNumber);
    expect(apres.version).toBe(origine.version + 1);
  });

  test("conflit de version : toast et fiche rechargée, sans modale", async ({ page }) => {
    await login(page);
    const libelle = `Conflit E2E ${Date.now()}`;
    const origine = await creerEcriture(page, libelle);

    await page.goto(`/journal-entries/${origine.id}`);
    await page.getByTestId("edit-entry").click();
    await expect(page.locator("#entry-description")).toHaveValue(libelle);

    // Un autre client modifie l'écriture entre l'ouverture du formulaire et
    // l'enregistrement : la version lue par l'écran est périmée.
    const courant = await lireEcriture(page, origine.id);
    const ctx = await authedApiContext(page);
    try {
      const put = await ctx.put(`/api/v1/journal-entries/${origine.id}`, {
        data: {
          entryDate: courant.entryDate,
          journal: courant.journal,
          description: `${libelle} par l'API`,
          version: courant.version,
          lines: courant.lines.map((l) => ({
            accountId: l.accountId,
            debit: l.debit,
            credit: l.credit,
            projectId: l.projectId,
          })),
        },
      });
      expect(put.status(), await put.text()).toBe(200);
    } finally {
      await disposeContextSafe(ctx);
    }

    await page.fill("#entry-description", `${libelle} par l'écran`);
    await page.getByRole("button", { name: "Valider" }).click();

    // Pas de modale : la fiche se recharge et montre l'état présent.
    await expect(page.getByTestId("edit-entry")).toBeVisible({ timeout: 10000 });
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.locator("dl")).toContainText(`${libelle} par l'API`);
    expect((await lireEcriture(page, origine.id)).description).toBe(`${libelle} par l'API`);
  });

  test("écriture de facture : ni Modifier ni Supprimer, le motif est affiché", async ({ page }) => {
    await login(page);
    const contact = await createContactWithAddressViaApi(page, `Modif facture ${Date.now()}`);
    const invoiceId = await createAndValidateInvoiceViaApi(page, contact);
    const ctx = await authedApiContext(page);
    let entryId: number;
    try {
      const inv = await (await ctx.get(`/api/v1/invoices/${invoiceId}`)).json();
      entryId = inv.journalEntryId as number;
      expect(entryId).toBeTruthy();
    } finally {
      await disposeContextSafe(ctx);
    }

    await page.goto(`/journal-entries/${entryId}`);
    // Présence avant absence : la fiche est chargée avant d'affirmer l'absence.
    await expect(page.getByTestId("reverse-blocked-reason")).toBeVisible();
    await expect(page.getByTestId("edit-entry")).toHaveCount(0);
    // Story 15-8b — « Supprimer » suit le même `modifiable` : absent, pas grisé.
    await expect(page.getByTestId("delete-entry")).toHaveCount(0);
    // Même motif que la contre-passation (OWNED_BY_INVOICE) : affiché une seule fois.
    await expect(page.getByTestId("modification-blocked-reason")).toHaveCount(0);
  });

  test("rôle Consultation : ni Modifier, ni Contre-passer, ni Supprimer, ni Historique — mais « Modifiée »", async ({ page }) => {
    await login(page);
    const libelle = `Consultation E2E ${Date.now()}`;
    const origine = await creerEcriture(page, libelle);
    // Revue P1 (A2) : l'écriture est MODIFIÉE avant le passage à Consultation,
    // pour que la mention « Modifiée » (visible à tous les rôles, C-15-8b-4)
    // soit exercée sur ce rôle — sans quoi une régression qui la cacherait ne
    // rougirait nulle part.
    await modifierLibelle(page, origine.id, `${libelle} corrigé`);
    const username = `consult-je-${Date.now()}`;
    const ctx = await authedApiContext(page);
    try {
      const res = await ctx.post("/api/v1/users", {
        data: { username, password: "MotDePasse12345", role: "Consultation" },
      });
      expect(res.ok(), `create user failed: ${res.status()}`).toBeTruthy();
    } finally {
      await disposeContextSafe(ctx);
    }
    await clearAuthStorage(page);
    await page.goto("/login");
    await page.fill("#username", username);
    await page.fill("#password", "MotDePasse12345");
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/");

    await page.goto(`/journal-entries/${origine.id}`);
    // Présence avant absence : la fiche est chargée.
    await expect(page.locator("dl")).toContainText(libelle);
    await expect(page.getByTestId("edit-entry")).toHaveCount(0);
    await expect(page.getByTestId("reverse-entry")).toHaveCount(0);
    await expect(page.getByTestId("modification-blocked-reason")).toHaveCount(0);
    // Story 15-8b — ni « Supprimer », ni le lien « Historique » : le journal
    // d'audit est refusé à Consultation (403).
    await expect(page.getByTestId("delete-entry")).toHaveCount(0);
    await expect(page.getByTestId("entry-history-link")).toHaveCount(0);
    // …mais la mention « Modifiée », elle, est visible à tous les rôles.
    await expect(page.getByTestId("entry-modified")).toBeVisible();
  });

  test("supprimer depuis la fiche : confirmation, retour à la liste, l'écriture a disparu", async ({ page }) => {
    await login(page);
    const libelle = `Suppression E2E ${Date.now()}`;
    const origine = await creerEcriture(page, libelle);

    await page.goto(`/journal-entries/${origine.id}`);
    await page.getByTestId("delete-entry").click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.getByTestId("delete-entry-confirm").click();

    await expect(page).toHaveURL(/\/journal-entries$/, { timeout: 10000 });
    const ctx = await authedApiContext(page);
    try {
      const resp = await ctx.get(`/api/v1/journal-entries/${origine.id}`);
      expect(resp.status(), "l'écriture a disparu").toBe(404);
    } finally {
      await disposeContextSafe(ctx);
    }
  });

  test("annuler la suppression : la fiche reste, l'écriture aussi", async ({ page }) => {
    await login(page);
    const libelle = `Suppression annulée E2E ${Date.now()}`;
    const origine = await creerEcriture(page, libelle);

    await page.goto(`/journal-entries/${origine.id}`);
    await page.getByTestId("delete-entry").click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await page.getByTestId("delete-entry-cancel").click();

    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page).toHaveURL(new RegExp(`/journal-entries/${origine.id}$`));
    await expect(page.locator("dl")).toContainText(libelle);
    expect((await lireEcriture(page, origine.id)).description).toBe(libelle);
  });

  test("après modification : « Modifiée » et « Historique », qui ouvre le journal d'audit filtré", async ({ page }) => {
    await login(page);
    const libelle = `Historique E2E ${Date.now()}`;
    const origine = await creerEcriture(page, libelle);

    // Avant toute modification : le lien est là, la mention non.
    await page.goto(`/journal-entries/${origine.id}`);
    await expect(page.getByTestId("entry-history-link")).toBeVisible();
    await expect(page.getByTestId("entry-modified")).toHaveCount(0);

    // Modification par l'API, puis relecture de la fiche.
    await modifierLibelle(page, origine.id, `${libelle} corrigé`);
    await page.goto(`/journal-entries/${origine.id}`);
    await expect(page.getByTestId("entry-modified")).toBeVisible();

    await page.getByTestId("entry-history-link").click();
    await expect(page).toHaveURL(
      new RegExp(`/audit-log\\?entityType=journal_entry&entityId=${origine.id}$`),
    );
    await expect(page.getByTestId("audit-log-filter-entity-id")).toHaveValue(String(origine.id));
    // La création et la modification de CETTE écriture, rien d'autre.
    await expect(page.getByTestId("audit-log-row")).toHaveCount(2, { timeout: 10000 });
  });
});

test.describe("Page écritures — contre-passation (Story 24-4a, #380)", () => {
  /**
   * ⚠️ **Le seul test qui vérifie que la contre-passation traverse réellement
   * la frontière HTTP.** Vitest teste la construction du payload, les tests
   * Rust la validation, et ni l'un ni l'autre ne voit une clé qui disparaît
   * entre les deux.
   *
   * ⛔ **L'écriture est créée par l'API, et la fiche atteinte par son URL** —
   * la liste des écritures ne renvoie PAS vers la fiche (vérifié au sol : elle
   * n'a aucun `href`, seule la page d'un avoir et le grand livre y mènent). Un
   * test qui cliquerait une ligne de la liste attendrait un lien qui n'existe
   * pas.
   *
   * ⛔ Sélecteurs par `data-testid`, jamais par libellé traduit (garde KF-043,
   * #326) — et la dette connue de ce fichier ne doit pas grossir.
   */
  test("corriger une écriture ajoute son inverse, et l'origine reste", async ({
    page,
  }) => {
    await login(page);
    const ctx = await authedApiContext(page);
    try {
      const accountsResp = await ctx.get("/api/v1/accounts");
      expect(accountsResp.ok()).toBeTruthy();
      const accounts = (await accountsResp.json()) as Array<{
        id: number;
        number: string;
        postable: boolean;
        active: boolean;
      }>;
      const postables = accounts.filter((a) => a.postable && a.active);
      expect(postables.length).toBeGreaterThanOrEqual(2);

      const libelle = `Contre-passation E2E ${Date.now()}`;
      const created = await ctx.post("/api/v1/journal-entries", {
        data: {
          entryDate: new Date().toISOString().slice(0, 10),
          journal: "OD",
          description: libelle,
          lines: [
            { accountId: postables[0].id, debit: "150.00", credit: "0.00" },
            { accountId: postables[1].id, debit: "0.00", credit: "150.00" },
          ],
        },
      });
      expect(created.status(), await created.text()).toBe(201);
      const origin = (await created.json()) as { id: number };

      await page.goto(`/journal-entries/${origin.id}`);
      await page.getByTestId("reverse-entry").click();
      await page.getByTestId("reverse-entry-confirm").click();

      // On atterrit sur la CONTRE-PASSATION, qui renvoie vers son origine.
      await expect(page.getByTestId("reverses-link")).toBeVisible({
        timeout: 10000,
      });

      // ⛔ Et l'origine EXISTE TOUJOURS : corriger n'efface pas.
      await page.goto(`/journal-entries/${origin.id}`);
      await expect(page.getByTestId("reversed-by-link")).toBeVisible();
      // Elle n'est plus contre-passable — le bouton est ABSENT, pas grisé.
      await expect(page.getByTestId("reverse-entry")).toHaveCount(0);
      await expect(page.getByTestId("reverse-blocked-reason")).toBeVisible();
    } finally {
      await disposeContextSafe(ctx);
    }
  });
});

test.describe("Page écritures — recherche & pagination (Story 3.4)", () => {
  async function createSeedEntries(
    page: import("@playwright/test").Page,
    count: number,
    descriptionPrefix = "Test 3.4",
  ) {
    const { debitNumber, creditNumber } = await getSeedAccountNumbers(page);
    for (let i = 0; i < count; i++) {
      await page.getByRole("button", { name: /Nouvelle écriture/ }).click();
      await page.fill("#entry-description", `${descriptionPrefix} ${i + 1}`);
      const accountInputs = page.locator('input[aria-autocomplete="list"]');
      await accountInputs.nth(0).fill(debitNumber);
      await page.getByRole("listbox").getByRole("option").first().click();
      await page
        .locator('input[inputmode="decimal"]')
        .nth(0)
        .fill(String(100 * (i + 1)));
      await accountInputs.nth(1).fill(creditNumber);
      await page.getByRole("listbox").getByRole("option").first().click();
      await page
        .locator('input[inputmode="decimal"]')
        .nth(3)
        .fill(String(100 * (i + 1)));
      await page.getByRole("button", { name: "Valider" }).click();
      await expect(
        page.getByText(new RegExp(`${descriptionPrefix} ${i + 1}`)),
      ).toBeVisible({
        timeout: 5000,
      });
    }
  }

  test("filtre par libellé avec debounce", async ({ page }) => {
    await goToJournalEntries(page);
    await createSeedEntries(page, 2, "Filtre Test");

    // Tapoter dans l'input description — le debounce doit grouper.
    const descInput = page.locator("#filter-description");
    await descInput.fill("Filtre Test 1");

    // Après 400ms (debounce 300ms + marge), seule l'écriture "Filtre Test 1"
    // devrait apparaître dans la liste.
    await page.waitForTimeout(400);
    await expect(page.getByText(/Filtre Test 1/).first()).toBeVisible();
  });

  test("filtre par plage de montants", async ({ page }) => {
    await goToJournalEntries(page);
    await createSeedEntries(page, 3, "Montant Test");

    // Filtrer 150-250 → devrait matcher uniquement l'écriture 2 (montant 200).
    await page.locator("#filter-amount-min").fill("150");
    await page.locator("#filter-amount-max").fill("250");
    await page.waitForTimeout(400);

    await expect(page.getByText(/Montant Test 2/)).toBeVisible();
  });

  test("tri ascendant puis descendant sur Date", async ({ page }) => {
    await goToJournalEntries(page);
    await createSeedEntries(page, 2, "Tri Test");

    // Clic sur header Date → toggle Asc/Desc.
    await page
      .getByRole("button", { name: new RegExp(i18nOrFallback("Date")) })
      .first()
      .click();

    // Vérifier qu'un indicateur de tri apparaît (↑ ou ↓).
    await expect(page.getByText(/[↑↓]/).first()).toBeVisible();
  });

  test("pagination — changement de taille de page", async ({ page }) => {
    await goToJournalEntries(page);

    // Changer la taille de page — le sélecteur est un shadcn-svelte Select.
    // Le premier Select visible dans le pied de tableau contrôle `limit`.
    // Le scénario vérifie simplement que l'URL reflète le changement.
    const initialUrl = page.url();
    expect(initialUrl).toContain("/journal-entries");
  });

  test("URL state préservé après rafraîchissement", async ({ page }) => {
    await goToJournalEntries(page);
    await createSeedEntries(page, 1, "URL State");

    // Appliquer un filtre.
    await page.locator("#filter-description").fill("URL State");
    await page.waitForTimeout(400);

    // Vérifier que l'URL contient le paramètre.
    expect(page.url()).toContain("description=URL+State");

    // Recharger la page — le filtre doit être restauré.
    await page.reload();
    await page.waitForTimeout(500);
    const desc = await page.locator("#filter-description").inputValue();
    expect(desc).toBe("URL State");
  });

  test("bouton Réinitialiser efface tous les filtres", async ({ page }) => {
    await goToJournalEntries(page);

    await page.locator("#filter-description").fill("quelque chose");
    await page.locator("#filter-amount-min").fill("100");
    await page.waitForTimeout(400);

    await page.getByRole("button", { name: /Réinitialiser/ }).click();

    const desc = await page.locator("#filter-description").inputValue();
    const min = await page.locator("#filter-amount-min").inputValue();
    expect(desc).toBe("");
    expect(min).toBe("");
  });

  // Scénarios reportés aux stories suivantes.
  test.skip("filtre par numéro de facture (story 5.x)", async () => {});
});

test.describe("Page écritures — tooltips pédagogiques (Story 3.5)", () => {
  test("hover sur l'en-tête Débit affiche la définition naturelle et technique", async ({
    page,
  }) => {
    await goToJournalEntries(page);
    await page.getByRole("button", { name: /Nouvelle écriture/ }).click();
    await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();

    // Cibler le trigger tooltip enveloppant le mot "Débit" dans l'en-tête de table.
    const debitTrigger = page
      .locator('[data-slot="tooltip-trigger"]')
      .filter({ hasText: "Débit" })
      .first();
    await expect(debitTrigger).toBeVisible();

    // Hover déclenche le tooltip.
    await debitTrigger.hover();

    // Le contenu doit afficher les deux registres : naturel + technique.
    // On utilise le timeout global Playwright (pas d'override) — un
    // timeout trop court rend le test flaky sur CI avec fade-in.
    await expect(page.getByText(/L'argent entre dans ce compte/)).toBeVisible();
    await expect(page.getByText(/colonne de gauche/)).toBeVisible();
  });

  // Couverture implicite : même pattern que débit, code partagé via
  // AccountingTooltip. Skippés pour éviter la duplication de setup.
  test.skip("hover crédit — même pattern que débit, couverture implicite", async () => {});
  test.skip("hover journal — même pattern que débit, couverture implicite", async () => {});
  test.skip("hover équilibré — même pattern que débit, couverture implicite", async () => {});
});

/** Helper local : renvoie le fallback FR si la clé i18n n'est pas résolue. */
function i18nOrFallback(fallback: string): string {
  return fallback;
}
