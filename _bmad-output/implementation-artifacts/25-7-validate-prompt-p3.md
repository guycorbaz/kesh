# Prompt — validation P3 de la spec, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Deux lentilles (Sonnet), contexte frais chacune. Rotation (D6) : P1 Sonnet ×3 → P2 Opus ×2
→ P3 Sonnet ×2.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. **Fiche** :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` — lire son **Change Log** (entrées P1 et P2). La
remédiation de P2 : `git diff bede9278 e25e8bb4 -- _bmad-output/implementation-artifacts/25-7-soldes-de-depart.md`.
Issue : `gh issue view 445`. Règles : `CLAUDE.md`. Aucun code n'est encore écrit.

**Constat qui motive la passe** : en P2, les 4 HIGH venaient de la remédiation de P1. La remédiation de P2 réécrit
l'**ordre des verrous** de l'AC 4 et les **entrelacements** de l'AC 7.

## Lentilles

- **R — Regression hunter** : le diff ci-dessus, puis la fiche entière.
  - **L'ordre des verrous de l'AC 4** (exercices → société `FOR SHARE` → comptes `FOR UPDATE` → compte de report
    `FOR UPDATE` → lignes `FOR SHARE` → `create_in_tx`) : refais toi-même, au code, l'ordre de fait d'une écriture
    ordinaire (`crates/kesh-db/src/repositories/journal_entries.rs`, `create_in_tx_inner` : exercice, compteur
    `journal_entry_number_sequences`, en-tête, lignes), de `create_opening_entry` (génération), de `lock_books`
    (`crates/kesh-api/src/routes/companies.rs` et son dépôt), de l'archivage d'un compte (`accounts.rs`), de
    `invoices::unvalidate`. Reste-t-il un **cycle** que la fiche ne dit pas ? Le verrou `FOR SHARE` sur `companies`
    est-il compatible avec ce que prennent les autres (partagé / exclusif) ? La variante de `find_first_by_company`
    `FOR UPDATE`, puis `find_open_covering_date` `FOR UPDATE` : deux exercices verrouillés dans un ordre qui pourrait
    croiser une écriture ordinaire de l'autre exercice ?
  - **Les entrelacements (1) à (4) de l'AC 7** : chacun est-il **réalisable** (le complément attend-il vraiment là où
    la fiche le dit ?) et chaque mutation déclarée tuable l'est-elle par le test qui la nomme ?
  - Contradictions nées de la réécriture entre arbitrages, AC, tâches, Dev Notes, Change Log.
- **C — Complétude et cohérence** : la fiche est-elle implémentable sans deviner ?
  - Chaque code de la table de l'AC 5 a-t-il son test (AC 7) et sa clé dans les 4 locales ? Chaque `completeReason`
    de l'AC 3 et sa priorité correspondent-ils à un refus du POST ?
  - L'AC 2 (montant à porter, limite) et l'AC 1 (avertissement) sont-ils cohérents avec la contrepartie (AC 4 étape 6) ?
  - L'AC 6 (bandeau gardé, grille de complément, `data-testid`) et les tests existants (Vitest `:206-216`,
    `:360-384`, `:400-419` ; Playwright `opening-balances.spec.ts:107-112`) : restent-ils verts tels que la fiche le
    prédit ?
  - Les sites rendus faux (AC 8) : le **manuel** (`.tex` **et PDF aplati**, `pdftotext docs/manual/fr/user-manual.pdf -
    | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`), les 4 catalogues, les replis, `README.md`, `website/`.
  - Les faits cités (`fichier:ligne`) introduits par la remédiation de P2 : revérifie-les (`grep -nF`).

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et sortie, ou
code cité relu), correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un
`grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL d'écriture.
Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
