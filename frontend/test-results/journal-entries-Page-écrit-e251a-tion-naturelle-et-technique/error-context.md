# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: journal-entries.spec.ts >> Page écritures — tooltips pédagogiques (Story 3.5) >> hover sur l'en-tête Débit affiche la définition naturelle et technique
- Location: tests/e2e/journal-entries.spec.ts:404:2

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: getByText(/L'argent entre dans ce compte/)
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for getByText(/L'argent entre dans ce compte/)

```

# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - generic [ref=e2]:
    - generic [ref=e3]:
      - banner [ref=e4]:
        - link "Kesh" [ref=e5] [cursor=pointer]:
          - /url: /
          - generic [ref=e6]: Kesh
        - generic [ref=e7]:
          - img [ref=e8]
          - searchbox "Rechercher..." [ref=e9]
        - button "Admin" [ref=e11]:
          - button "Admin" [ref=e12]:
            - img
            - generic [ref=e13]: Admin
            - img
      - generic [ref=e14]:
        - navigation "Navigation principale" [ref=e15]:
          - generic [ref=e16]: Quotidien
          - list [ref=e17]:
            - listitem [ref=e18]:
              - link "Accueil" [ref=e19] [cursor=pointer]:
                - /url: /
            - listitem [ref=e20]:
              - link "Carnet d'adresses" [ref=e21] [cursor=pointer]:
                - /url: /contacts
            - listitem [ref=e22]:
              - link "Catalogue" [ref=e23] [cursor=pointer]:
                - /url: /products
            - listitem [ref=e24]:
              - link "Facturer" [ref=e25] [cursor=pointer]:
                - /url: /invoices
            - listitem [ref=e26]:
              - link "Échéancier" [ref=e27] [cursor=pointer]:
                - /url: /invoices/due-dates
            - listitem [ref=e28]:
              - link "Payer" [ref=e29] [cursor=pointer]:
                - /url: /bank-accounts
            - listitem [ref=e30]:
              - link "Importer" [ref=e31] [cursor=pointer]:
                - /url: /bank-import
          - separator [ref=e32]
          - generic [ref=e33]: Mensuel
          - list [ref=e34]:
            - listitem [ref=e35]:
              - link "Écritures" [ref=e36] [cursor=pointer]:
                - /url: /journal-entries
            - listitem [ref=e37]:
              - link "Réconciliation" [ref=e38] [cursor=pointer]:
                - /url: /reconciliation
            - listitem [ref=e39]:
              - link "Rapports" [ref=e40] [cursor=pointer]:
                - /url: /reports
          - separator [ref=e41]
          - list [ref=e42]:
            - listitem [ref=e43]:
              - link "Paramètres" [ref=e44] [cursor=pointer]:
                - /url: /settings
          - separator [ref=e45]
          - generic [ref=e46]: Administration
          - list [ref=e47]:
            - listitem [ref=e48]:
              - link "Utilisateurs" [ref=e49] [cursor=pointer]:
                - /url: /users
            - listitem [ref=e50]:
              - link "Facturation" [ref=e51] [cursor=pointer]:
                - /url: /settings/invoicing
        - main [ref=e52]:
          - generic [ref=e53]:
            - heading "Écritures comptables" [level=1] [ref=e55]
            - generic [ref=e56]:
              - heading "Saisie d'écriture" [level=2] [ref=e57]
              - generic [ref=e58]:
                - generic [ref=e59]:
                  - generic [ref=e60]: Date
                  - textbox "Date" [ref=e61]: 2026-04-20
                - generic [ref=e62]:
                  - button "Journal" [ref=e64]
                  - button "Journal" [ref=e65]:
                    - text: Achats
                    - img
                - generic [ref=e66]:
                  - generic [ref=e67]: Libellé
                  - textbox "Libellé" [ref=e68]
              - table [ref=e69]:
                - rowgroup [ref=e70]:
                  - row "Compte Débit Crédit" [ref=e71]:
                    - columnheader "Compte" [ref=e72]
                    - columnheader "Débit" [ref=e73]:
                      - button "Débit" [ref=e74]
                    - columnheader "Crédit" [ref=e75]:
                      - button "Crédit" [ref=e76]
                    - columnheader [ref=e77]
                - rowgroup [ref=e78]:
                  - row [ref=e79]:
                    - cell [ref=e80]:
                      - textbox "Compte" [ref=e82]
                    - cell [ref=e83]:
                      - textbox "0.00" [ref=e84]
                    - cell [ref=e85]:
                      - textbox "0.00" [ref=e86]
                    - cell [ref=e87]
                  - row [ref=e88]:
                    - cell [ref=e89]:
                      - textbox "Compte" [ref=e91]
                    - cell [ref=e92]:
                      - textbox "0.00" [ref=e93]
                    - cell [ref=e94]:
                      - textbox "0.00" [ref=e95]
                    - cell [ref=e96]
              - button "+ Ajouter une ligne" [ref=e97]:
                - img
                - text: + Ajouter une ligne
              - generic [ref=e98]:
                - generic [ref=e99]:
                  - generic [ref=e100]:
                    - strong [ref=e101]: "Total débits :"
                    - text: "0.00"
                  - generic [ref=e102]:
                    - strong [ref=e103]: "Total crédits :"
                    - text: "0.00"
                  - generic [ref=e104]:
                    - strong [ref=e105]: "Différence :"
                    - text: "0.00"
                - generic [ref=e106]: —
              - generic [ref=e107]:
                - button "Annuler" [ref=e108]
                - button "Valider" [disabled]
      - contentinfo [ref=e109]: Kesh v0.1.0 — Logiciel libre (EUPL 1.2). Les données ne remplacent pas un fiduciaire.
    - region "Notifications alt+T"
    - generic [ref=e110]: Écritures comptables - Kesh
  - generic [ref=e112]:
    - paragraph [ref=e113]: debit
    - paragraph [ref=e114]: debit
```

# Test source

```ts
  324 | 		// Après 400ms (debounce 300ms + marge), seule l'écriture "Filtre Test 1"
  325 | 		// devrait apparaître dans la liste.
  326 | 		await page.waitForTimeout(400);
  327 | 		await expect(page.getByText(/Filtre Test 1/).first()).toBeVisible();
  328 | 	});
  329 | 
  330 | 	test('filtre par plage de montants', async ({ page }) => {
  331 | 		await goToJournalEntries(page);
  332 | 		await createSeedEntries(page, 3, 'Montant Test');
  333 | 
  334 | 		// Filtrer 150-250 → devrait matcher uniquement l'écriture 2 (montant 200).
  335 | 		await page.locator('#filter-amount-min').fill('150');
  336 | 		await page.locator('#filter-amount-max').fill('250');
  337 | 		await page.waitForTimeout(400);
  338 | 
  339 | 		await expect(page.getByText(/Montant Test 2/)).toBeVisible();
  340 | 	});
  341 | 
  342 | 	test('tri ascendant puis descendant sur Date', async ({ page }) => {
  343 | 		await goToJournalEntries(page);
  344 | 		await createSeedEntries(page, 2, 'Tri Test');
  345 | 
  346 | 		// Clic sur header Date → toggle Asc/Desc.
  347 | 		await page
  348 | 			.getByRole('button', { name: new RegExp(i18nOrFallback('Date')) })
  349 | 			.first()
  350 | 			.click();
  351 | 
  352 | 		// Vérifier qu'un indicateur de tri apparaît (↑ ou ↓).
  353 | 		await expect(page.getByText(/[↑↓]/).first()).toBeVisible();
  354 | 	});
  355 | 
  356 | 	test('pagination — changement de taille de page', async ({ page }) => {
  357 | 		await goToJournalEntries(page);
  358 | 
  359 | 		// Changer la taille de page — le sélecteur est un shadcn-svelte Select.
  360 | 		// Le premier Select visible dans le pied de tableau contrôle `limit`.
  361 | 		// Le scénario vérifie simplement que l'URL reflète le changement.
  362 | 		const initialUrl = page.url();
  363 | 		expect(initialUrl).toContain('/journal-entries');
  364 | 	});
  365 | 
  366 | 	test('URL state préservé après rafraîchissement', async ({ page }) => {
  367 | 		await goToJournalEntries(page);
  368 | 		await createSeedEntries(page, 1, 'URL State');
  369 | 
  370 | 		// Appliquer un filtre.
  371 | 		await page.locator('#filter-description').fill('URL State');
  372 | 		await page.waitForTimeout(400);
  373 | 
  374 | 		// Vérifier que l'URL contient le paramètre.
  375 | 		expect(page.url()).toContain('description=URL+State');
  376 | 
  377 | 		// Recharger la page — le filtre doit être restauré.
  378 | 		await page.reload();
  379 | 		await page.waitForTimeout(500);
  380 | 		const desc = await page.locator('#filter-description').inputValue();
  381 | 		expect(desc).toBe('URL State');
  382 | 	});
  383 | 
  384 | 	test('bouton Réinitialiser efface tous les filtres', async ({ page }) => {
  385 | 		await goToJournalEntries(page);
  386 | 
  387 | 		await page.locator('#filter-description').fill('quelque chose');
  388 | 		await page.locator('#filter-amount-min').fill('100');
  389 | 		await page.waitForTimeout(400);
  390 | 
  391 | 		await page.getByRole('button', { name: /Réinitialiser/ }).click();
  392 | 
  393 | 		const desc = await page.locator('#filter-description').inputValue();
  394 | 		const min = await page.locator('#filter-amount-min').inputValue();
  395 | 		expect(desc).toBe('');
  396 | 		expect(min).toBe('');
  397 | 	});
  398 | 
  399 | 	// Scénarios reportés aux stories suivantes.
  400 | 	test.skip('filtre par numéro de facture (story 5.x)', async () => {});
  401 | });
  402 | 
  403 | test.describe('Page écritures — tooltips pédagogiques (Story 3.5)', () => {
  404 | 	test('hover sur l\'en-tête Débit affiche la définition naturelle et technique', async ({
  405 | 		page
  406 | 	}) => {
  407 | 		await goToJournalEntries(page);
  408 | 		await page.getByRole('button', { name: /Nouvelle écriture/ }).click();
  409 | 		await expect(page.getByText(/Saisie d'écriture/)).toBeVisible();
  410 | 
  411 | 		// Cibler le trigger tooltip enveloppant le mot "Débit" dans l'en-tête de table.
  412 | 		const debitTrigger = page
  413 | 			.locator('[data-slot="tooltip-trigger"]')
  414 | 			.filter({ hasText: 'Débit' })
  415 | 			.first();
  416 | 		await expect(debitTrigger).toBeVisible();
  417 | 
  418 | 		// Hover déclenche le tooltip.
  419 | 		await debitTrigger.hover();
  420 | 
  421 | 		// Le contenu doit afficher les deux registres : naturel + technique.
  422 | 		// On utilise le timeout global Playwright (pas d'override) — un
  423 | 		// timeout trop court rend le test flaky sur CI avec fade-in.
> 424 | 		await expect(page.getByText(/L'argent entre dans ce compte/)).toBeVisible();
      |                                                                 ^ Error: expect(locator).toBeVisible() failed
  425 | 		await expect(page.getByText(/colonne de gauche/)).toBeVisible();
  426 | 	});
  427 | 
  428 | 	// Couverture implicite : même pattern que débit, code partagé via
  429 | 	// AccountingTooltip. Skippés pour éviter la duplication de setup.
  430 | 	test.skip('hover crédit — même pattern que débit, couverture implicite', async () => {});
  431 | 	test.skip('hover journal — même pattern que débit, couverture implicite', async () => {});
  432 | 	test.skip('hover équilibré — même pattern que débit, couverture implicite', async () => {});
  433 | });
  434 | 
  435 | /** Helper local : renvoie le fallback FR si la clé i18n n'est pas résolue. */
  436 | function i18nOrFallback(fallback: string): string {
  437 | 	return fallback;
  438 | }
  439 | 
```