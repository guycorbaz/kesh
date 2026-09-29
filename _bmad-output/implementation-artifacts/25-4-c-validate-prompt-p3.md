# Prompt — validation P3, Story 25-4-c (le résiduel au rapprochement)

*Versionné le 2026-09-29. **Une lentille** (Opus), contexte frais. La P2 (Haiku) a rendu 0 finding
sans preuve ; l'orchestrateur en a repris deux MEDIUM. Cette passe porte donc sur la fiche entière,
avec un poids particulier sur ce qui a été écrit depuis la P1.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c-residuel-au-rapprochement`. Fiche :
`_bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md` (état courant). Ce que
P1 et P2 ont changé : `git diff 1db610a5 HEAD -- _bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md`.
Mère : `25-4-propager-le-residuel.md` ; sœurs `25-4-a`, `25-4-b1`, `25-4-b2` (même répertoire).
Issues : `gh issue view 420`, `gh issue view 476`. Règles : `CLAUDE.md` (§ *Pattern batch*,
§ *Review Iteration Rule*). Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages Q1–Q3 sont **retenus** : ne pas les contester, en contester la mise en œuvre.

## Axes — tous obligatoires

1. **L'AC 5-bis et son tableau d'ordre des verrous** — vérifie chaque ligne contre le code
   (`crates/kesh-api/src/routes/reconciliation.rs` autour de `accept_batch` et `accept_one_invoice`,
   `crates/kesh-reconciliation/src/mutex.rs`, `crates/kesh-db/src/repositories/invoice_settlements_write.rs`,
   `crates/kesh-db/src/repositories/reconciliation_cancel.rs`). L'interblocage décrit est-il le bon,
   en manque-t-il (insertion dans `invoice_settlements` sous REPEATABLE READ, verrous de trou, route
   de règlement manuel contre acceptation) ? L'affirmation « InnoDB annule toute la transaction, pas
   le point de sauvegarde » est-elle exacte pour MariaDB, et le code d'`accept_batch` la gère-t-il
   comme la fiche le dit (lis la boucle des savepoints) ? Le verrou au chargement change-t-il le
   comportement de la garde de statut ou du verrou optimiste (`version`, `:1455-1470`) ?
2. **L'arrondi (AC 5, 6)** : sept sites — relis-les tous. Le helper « sur `Money::round_to_centimes()` »
   et `reminder_amount_due` rebranché : même résultat qu'aujourd'hui pour le rappel ? Le filtre SQL
   des candidats compare-t-il au reste **arrondi** (en SQL ou en Rust) et la tolérance ± 0.05
   l'exige-t-elle encore ? Le dialogue : quelle valeur de comparaison, quel mode d'arrondi de big.js ?
   Le test de dépôt par insertion directe de l'AC 6 est-il cohérent avec les gardes-fous du dépôt
   (base partagée vs `#[sqlx::test]`, squash `test-schema`) ?
3. **Inventaire des sites** — pars du symptôme, pas de la liste :
   `grep -rn "total_ttc\|amount_due\|amountDue\|invoice_amount\|invoiceAmount\|due_after\|due_before\|OVERPAYMENT" crates/ frontend/src`.
   Tout site qui compare un montant à régler, ou l'affiche au comptable dans le flux du rapprochement,
   hors de la fiche : légitime ou oublié ?
4. **Tests (AC 7)** : chacun faisable, et **aurait-il échoué avant le patch** ? Le test de concurrence
   de l'AC 5-bis est-il déterministe (sans `sleep`) — propose un montage. L'E2E Playwright : quels
   helpers réels permettent règlement partiel + import bancaire (`frontend/tests/e2e/`) ?
5. **Le manuel (AC 8)** : lignes citées, et **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).
   Le manuel parle-t-il ailleurs du règlement manuel ou de l'annulation d'une façon que l'arrondi
   rendrait fausse ?
6. **Cohérence interne et périmètre** : AC ↔ tâches, modules recomptés, Change Log recompté
   (sévérités de P1 et P2), aucune phrase contradictoire. Le verrou (AC 5-bis) et l'arrondi du règlement
   manuel font-ils déborder la story au-delà de ce qu'une passe adversariale peut tenir (règle de
   splitting du `CLAUDE.md`) ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** (commande exécutée et
  sa sortie, ou code lu cité), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un rapport sans elle
  ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers le scratchpad, `cargo check`.
