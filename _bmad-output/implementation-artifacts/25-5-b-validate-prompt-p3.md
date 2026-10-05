# Prompt — validation P3 ciblée, Story 25-5-b

*Versionné le 2026-10-03. **Une lentille** (Haiku), contexte frais — passe ciblée sur la remédiation de P2. Trend :
P1 1H/4M (Sonnet ×3) → P2 4M/9L (Opus).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-5-b-balance-solde-ouverture`. **Diff à relire, aplati, un seul
commit : `git diff 6fc79cf5 013b2ae7`** (la fiche `_bmad-output/implementation-artifacts/25-5-b-balance-solde-ouverture.md`).
La fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Lentille — regression hunter

1. **Cohérence interne après le patch** : `closing_balanced` a été retiré — reste-t-il une phrase (AC 5, 7, 8, 9, tâches,
   Dev Notes) qui suppose deux indicateurs ou un ✓/⚠️ sous la colonne Clôture ? `grep -nF "closing" <fiche>` et lis chaque
   occurrence.
2. **L'AC 2 réécrit** : la forme prescrite (trois paires de sommes brutes, `opening_from` en Rust) est-elle compatible
   avec le grand livre tel qu'il est ? Lis `crates/kesh-report/src/general_ledger.rs:194-275` : sa requête d'ouverture
   peut-elle rendre les deux paires sans changer son résultat ? `fiscal_year_start_containing` y renvoie `None` quand
   aucun exercice ne couvre `from` — la forme prescrite conserve-t-elle ce cas ?
3. **Les libellés** : les clés `reports-column-opening` / `reports-column-closing` existent-elles **déjà** dans
   `crates/kesh-i18n/locales/fr-CH/messages.ftl` (collision) ? `grep -n "reports-column-" …`. Les champs `col_opening` /
   `col_closing` existent-ils déjà dans `SectionLabels` (`crates/kesh-report/src/pdf.rs`) ?
4. **Les références ajoutées** existent-elles et disent-elles ce que la fiche affirme : `balance_sheet.rs:237`,
   `BalanceSheetView.svelte:97-100`, `csv.rs:139-145`, `csv.rs:140-144`, `pdf.rs:596-599`, `general_ledger.rs:135-205`,
   `user-manual.tex:615`, `CHANGELOG.md` (section `## [0.12.1] — Non publié`) ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve obligatoire : la commande ET sa sortie** — un finding sans sortie de commande
sera rejeté, un « 0 finding » sans les sorties des quatre vérifications aussi. ⛔ La liste des axes exercés ET non
exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`.
