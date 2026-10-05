# Prompt — validation P1, Story 25-4-d1 (les comptes de solde)

*Versionné le 2026-10-01. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d1-comptes-de-solde`. Fiche :
`_bmad-output/implementation-artifacts/25-4-d1-comptes-de-solde.md`. Issue : `gh issue view 384`. Patron : la
Story 25-4-c3-a2 (`25-4-c3-a2-compte-arrondi-plans.md`, marqueur `roundingDifference`). Règles : `CLAUDE.md`
(§ *Migration breaking policy*). Checklist : `.claude/skills/bmad-create-story/checklist.md`.

Les arbitrages sont **retenus** : conteste la mise en œuvre, pas le principe. ⚠️ La fiche décrit du travail **à
faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Le marqueur (AC 1)** : factoriser la validation avec celle de `roundingDifference` est-il faisable sans changer
   son comportement ? Les tests existants du marqueur d'arrondi (`chart_of_accounts/mod.rs`) en dépendent-ils ? Un
   compte peut-il porter deux marqueurs (6940 arrondi + une nature) — la fiche le dit-elle ?
3. **Les plans (AC 2)** : `grep -rn "load_chart\|85\|82\|entries.len()" crates/*/tests crates/*/src | grep -i chart` —
   quels tests comptent les entrées ou comparent un plan semé à un état attendu (`accounts_role_backfill`,
   `closing_accounts_backfill`, onboarding, seed de démo `kesh-seed`) ? Le 3805 sous le parent `30` est-il correct
   (groupe des ventes) pour un compte de déduction ? Les traductions sont-elles justes (DE, IT, EN) ?
4. **La désignation (AC 4)** : `rounding_account_from_chart` peut-il être généralisé sans toucher son comportement ?
   Une société dont le plan est importé, ou dont la forme juridique a changé, reçoit-elle un mauvais compte ?
5. **Le réglage (AC 5)** : l'inventaire des sites est-il complet — `grep -rn "default_rounding_account_id\|defaultRoundingAccountId" crates/ frontend/src` ?
   Trois colonnes multiplient les sites par trois : la fiche le mesure-t-elle (taille de la story) ?
6. **La migration (AC 3)**, **le manuel (AC 6)** (PDF aplati : `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`), **les tests (AC 7)**,
   **le périmètre** (modules recomptés).

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, ou code cité), correction. ⛔ La liste des axes
exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`, `cargo check`, `pdftotext` vers le scratchpad.
