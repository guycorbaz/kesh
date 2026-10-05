# Prompt — revue de code P2 ciblée, Story 25-5-b

*Versionné le 2026-10-03. **Une lentille** (Haiku), contexte frais — passe ciblée sur la remédiation de P1 (P1 Sonnet ×3 :
1 MED, LOW).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-5-b-balance-solde-ouverture`. **Diff à relire, un seul commit, aplati :
`git show 84505eed`.** Fiche : `_bmad-output/implementation-artifacts/25-5-b-balance-solde-ouverture.md`.

## Lentille — regression hunter

1. `crates/kesh-report/src/pdf.rs` : le libellé `opening_unbalanced` est-il ajouté à `SectionLabels` **et** à ses défauts,
   sans autre construction littérale de `SectionLabels` ailleurs (`grep -rn "SectionLabels {" crates`) ? L'avertissement
   est-il dessiné après `ensure_space_for_row` ? Tient-il sur une ligne (Helvetica-Bold 10 pt, ~0,55 em par caractère,
   largeur utile 180 mm) ? Le test l'exerce-t-il dans les deux sens ?
2. `crates/kesh-report/tests/trial_balance_opening.rs`, test `un_compte_archive_aux_mouvements_compenses_figure` : les deux
   écritures tombent-elles dans l'exercice et la période ? L'assertion distingue-t-elle « présent à zéro » d'« absent » ?
3. Manuel et CHANGELOG : la phrase corrigée dit-elle juste ? Lis `crates/kesh-report/src/income_statement.rs` (filtre des
   mouvements) et vérifie : sur une période qui ne commence pas au premier jour de l'exercice, compte de résultat ≠ clôture
   de la balance ; sur l'exercice entier, égalité. Reste-t-il **ailleurs** la promesse fausse ? `grep -rn "montant au compte
   de résultat" docs CHANGELOG.md _bmad-output/implementation-artifacts/25-5-b-*.md crates`.
4. La doc de module de `trial_balance.rs` : `DateOutsideFiscalYear` existe-t-il bien dans `journal_entries::create_in_tx`
   (`grep -n "DateOutsideFiscalYear" crates/kesh-db/src/repositories/journal_entries.rs`) ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve obligatoire : la commande ET sa sortie** — un finding sans sortie sera rejeté,
un « 0 finding » sans les sorties des quatre vérifications aussi. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`.
