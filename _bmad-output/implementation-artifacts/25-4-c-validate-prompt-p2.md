# Prompt — validation P2, Story 25-4-c (le résiduel au rapprochement)

*Versionné le 2026-09-29. **Une lentille** (Haiku), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c-residuel-au-rapprochement`. Fiche à valider :
`_bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md` — **lis le fichier dans
son état actuel**, ne raisonne pas sur un diff. Ce que la passe 1 a changé : `git show 36f9ce31 --
_bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md` (à lire comme contexte ;
les numéros de ligne font foi dans le fichier courant, pas dans le diff). Règles : `CLAUDE.md`.

Les arbitrages Q1–Q3 sont **retenus** : ne pas les contester, en contester la mise en œuvre.

## Axes — tous obligatoires

1. **La remédiation de la passe 1** — c'est là que la sévérité se déplace :
   - AC 5 : le helper « sur `Money::round_to_centimes()` » est-il réalisable depuis `kesh-db`
     (`crates/kesh-db/Cargo.toml`, API de `Money` dans `crates/kesh-core/src/types/money.rs`) ?
     Rebrancher `reminder_amount_due` (`crates/kesh-api/src/routes/invoice_pdf_service.rs:112`)
     change-t-il son comportement (refus ≤ 0, tests `:892-900`) ?
   - AC 5, dialogue : comment le frontend obtient-il le reste arrondi (`amountDue` est une chaîne à
     4 décimales) ? `Big` arrondit-il dans le même sens que `MidpointAwayFromZero` (mode par défaut
     de big.js) ? Le message d'erreur et ses clés i18n restent-ils justes ?
   - AC 6 : l'annulation laissant 0.0040 — ce scénario est-il **constructible** (règlements de
     quels montants, sur quelle facture) ? Sinon l'AC promet un test impossible.
   - AC 5-bis : lis `accept_one_invoice` (`crates/kesh-api/src/routes/reconciliation.rs`, env.
     `:1056-1600`), `settle_invoice` (`invoice_settlements_write.rs:40-260`) et
     `reconciliation_cancel.rs:250-330`. **Quel ordre de verrous prennent-ils réellement**
     (bank_transactions, invoices, fiscal_years, journal) ? L'AC le laisse au dev : faut-il le
     trancher dans la fiche ? Un test de concurrence est-il faisable avec `#[sqlx::test]` (deux
     transactions, pool) — existe-t-il un précédent dans le dépôt (`grep -rn "tokio::join\|join!" crates/*/tests`) ?
2. **Sites encore oubliés** : `grep -rn "amountDue\|amount_due\|due_after\|due_before" crates/ frontend/src`
   — toute comparaison du reste dû à zéro ou à un montant, hors des sept sites de l'AC 5, est-elle
   légitime ou oubliée ?
3. **Faisabilité des tests (AC 7)** — axe non exercé en P1 : l'E2E Playwright peut-il monter une
   facture partiellement réglée puis un import bancaire ? Cherche les helpers existants
   (`frontend/tests/e2e/`, fixtures CAMT, `authedApiContext`). Les mutations annoncées sont-elles
   tuables par les tests prévus ?
4. **Cohérence interne** : numéros d'AC référencés par les tâches, section « Arbitrages », Change
   Log, modules (recompte), aucune phrase contradictoire entre l'inventaire et les AC.

## Ce que tu rends

- **Findings** : sévérité, endroit exact, **preuve** (commande exécutée et sa sortie, ou code lu cité),
  correction proposée. Pour toute affirmation qu'un élément est **absent**, cite le `grep -nF` qui le
  prouve.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un rapport sans elle
  ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`cargo check`.
