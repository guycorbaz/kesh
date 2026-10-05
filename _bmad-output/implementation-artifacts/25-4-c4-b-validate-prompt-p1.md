# Prompt — validation P1, Story 25-4-c4-b (l'arrondi à 5 centimes, visible et réglable)

*Versionné le 2026-10-01. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c4-arrondi-facture-5-centimes` (la c4-a y est faite).
Fiche : `_bmad-output/implementation-artifacts/25-4-c4-b-arrondi-visible-et-reglage.md`. Sœur :
`25-4-c4-a-arrondi-fige-a-la-validation.md`. CR : `gh issue view 494`. Règles : `CLAUDE.md`. Checklist :
`.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages de Guy sont **retenus** : conteste la mise en œuvre, pas le principe. ⚠️ La fiche décrit du
travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Le PDF (AC 1)** : le gabarit `crates/kesh-qrbill/src/pdf.rs` (`draw_invoice_section`, `recap_reserve`,
   la garde `TooManyLines`) — l'ajout d'une ligne et d'un sous-total conditionnels est-il faisable sans casser la
   géométrie ? Qui remplit `InvoicePdfData` (`grep -rn "InvoicePdfData {" crates/`) — tous les remplisseurs sont-ils
   dans la fiche (facture, rappel, avoir, tests, `kesh-qrbill` lui-même) ? Où vivent les libellés du PDF
   (`invoice-pdf-total-ttc`) ?
3. **L'API (AC 2)** : `get_invoice` peut-il lire le réglage sans coût ni verrou indus ? Qui d'autre construit
   `InvoiceResponse` (`grep -rn "from_parts" crates/kesh-api`) et lirait un `roundingAmount` absent ou faux
   (listes, `settle_invoice_handler`, `cancel`, e-mail, validation) ? La fiche dit-elle ce qu'ils renvoient ?
4. **La fiche (AC 3)** : `frontend/src/routes/(app)/invoices/[id]/+page.svelte:988-1011` — les libellés y sont-ils
   déjà traduits (`i18nMsg`) ou codés en dur ? Le compteur `sitesTotal`
   (`frontend/src/lib/shared/i18n-keys.test.ts`) et la garde `e2e-selecteurs-traduits` ?
5. **Le réglage (AC 4)** : tous les sites qui énumèrent les champs de la mise à jour (`grep -rn
   "default_rounding_account_id" crates/ frontend/src`) — la fiche les couvre-t-elle (type frontend, `PUT` du
   frontend qui envoie le corps entier, Vitest et E2E existants de l'écran, `settings-rounding-account.spec.ts`) ?
6. **Le manuel (AC 5)** : `docs/manual/fr/user-manual.tex` — où décrit-il la facture, son total, le PDF ?
   `admin-manual.tex`, paragraphe *Compte de différences d'arrondi* (PDF aplati : `pdftotext … | tr '\n' ' ' | tr -s ' '`
   vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
7. **Faisabilité des tests (AC 6)** et **périmètre** (modules recomptés depuis les tâches).

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité), correction. ⛔ La liste des axes
exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`, `cargo check`, `pdftotext` vers le scratchpad.
