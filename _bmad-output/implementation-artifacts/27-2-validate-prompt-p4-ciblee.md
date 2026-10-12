# Prompt — validation P4 **ciblée** de la spec, Story 27-2 (la devise d'un relevé CSV est lue)

*Versionné le 2026-10-12. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P3** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Trend des MEDIUM distincts : P1 3 → P2 3 → P3 1, 0 HIGH, 0
CRITICAL.*

Dépôt `/home/gcorbaz/devel/kesh-27-2`, branche `story/27-2-devise-import-csv`.
**Périmètre : `git show eb3f4932`** (remédiation P3), soit un **diff unique** — ne lis pas la séquence des commits
antérieurs. Fichiers touchés : la fiche `_bmad-output/implementation-artifacts/27-2-devise-import-csv.md` et le
registre `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md`. Rapports P3 que ce commit remédie :
`/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p3-{R,F}.md`.

⚠️ La fiche est une **spec** : le code qu'elle prescrit n'est pas écrit. Les numéros de ligne qu'elle cite sont ceux du
code au sol (`a2743fd4` et la branche), qui n'a pas bougé.

## La lentille — chasseur de régressions de la remédiation P3

Pour **chaque hunk** du commit, vérifie contre le code au sol :

1. **Garde de devise de la réconciliation** (le MEDIUM de P3, C-27-2-20) : la phrase neuve — propositions automatiques
   et acceptation d'une règle filtrent la devise ; rapprochement manuel, éclatement et acceptation d'une facture ne la
   filtrent pas — est-elle vraie **sur chacun** des chemins nommés ? Refais :
   `grep -n 'currency != "CHF"\|currency == "CHF"' crates/kesh-api/src/routes/reconciliation.rs` et situe chaque ligne
   dans sa fonction (`grep -n "^async fn\|^pub async fn" crates/kesh-api/src/routes/reconciliation.rs`) ; lis
   `crates/kesh-api/tests/reconciliation_e2e.rs:2241-2262`. Existe-t-il un **cinquième** chemin de rapprochement que le
   texte ne nomme pas (doc-module de `crates/kesh-api/src/routes/reconciliation_cancel.rs`, « cinq chemins ») ? La
   phrase a-t-elle été propagée **partout** où l'ancienne apparaissait — refais
   `grep -rn "rapprochable\|écarte\|l'écart" _bmad-output/implementation-artifacts/27-2-devise-import-csv.md _bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md`
   et classe chaque sortie (texte courant à corriger, ou mention historique légitime) ?
2. **AC12, CHANGELOG restructuré** (R8, C-27-2-24) : le *texte à publier* ne porte-t-il plus **aucun** renvoi de
   traçabilité (`validation P`, `C-27-2-`, `AC1…`, `git show`) ? Chaque puce est-elle dans une rubrique du patron de la
   0.13.1 (compare à `awk '/^## \[0.13.1\]/{f=1} /^## \[0.13.0\]/{exit} f' CHANGELOG.md | grep -n "^#\|^⚠️"`) ? Le
   lien d'issue en ligne est-il bien formé ? Le sens des puces d'avant la P3 est-il conservé (rien de perdu en
   déplaçant le texte — compare au même passage dans `git show eb3f4932^:_bmad-output/implementation-artifacts/27-2-devise-import-csv.md`) ?
3. **Export global de la 0.13.1** (F2, C-27-2-22) : `git show v0.13.1:crates/kesh-api/src/exports/csv_tables.rs`
   (l. 1041-1075) et `git show v0.13.1:crates/kesh-api/src/exports/global.rs` (l. 320) — l'export recopie-t-il bien
   `column_mapping_json` **sans** désérialiser le profil, et porte-t-il `id` ? Un autre maillon de l'export de la 0.13.1
   désérialiserait-il les profils (et rendrait 500 comme le `GET`) ?
4. **AC15-1 et AC15-2** (R5, C-27-2-25) : contre `frontend/tests/e2e/bank-csv-import.spec.ts` (l. 28, 70-85, 260-356)
   et `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv` (synthétique) — le décalage `2026-` → `2025-` sur les
   seules lignes de données change-t-il l'empreinte de **chaque** ligne (`crates/kesh-api/src/routes/bank_imports.rs:1532-1539`) ?
   Le motif `^ubs_synthetique_devise\.csv$` évite-t-il le profil voisin ? Le `toHaveLength(11)` est-il faisable par la
   route que le test voisin emploie ?
5. **AC19, entrée 3 et M25** (F3, C-27-2-27) : contre `crates/kesh-import/src/camt053/mod.rs` (l. 168-183, 372, 427,
   603, 618) — sous le code d'avant, (1-bis) rougit-il ? sous la correction prescrite (`path.last()`), passe-t-il ? sous
   M25 (« première devise gagne »), rougit-il, et (1) reste-t-il vert ? L'affirmation « forme dominante au dépôt » est-elle
   vraie (`grep -c "<TxDtls>"` et lecture des `<TxDtls>` des fixtures de `crates/kesh-import/tests/fixtures/camt053/`) ?
   AC14-8 (« trois » transactions) est-il cohérent avec la fixture prescrite ?
6. **Mutants équivalents et doc-comment** (F6 = R3, C-27-2-26) : contre
   `frontend/src/lib/features/bank-import/BankProfileForm.svelte:56-69` — les quatre clés déclarées équivalentes
   (`date`, `reference`, `details`, `currency`) sont-elles bien écrites dans **les deux** modes ? Le doc-comment neuf
   dit-il vrai ? L'`import type` existant est-il aux lignes citées (15-19) ?
7. **Liste fermée des « v0.1 »** (R2, C-27-2-21) : refais
   `grep -rnF "v0.1" crates --include=*.rs | grep -i "devise\|currency\|chf"` — la sortie, **une fois retirés les
   sites qu'AC10 prescrit de réécrire**, égale-t-elle exactement la liste fermée écrite à AC10 ? La parenthèse de
   `reconciliation_e2e.rs:2253-2254` est-elle citée exactement ?
8. **Ordre des refus et lignes citées** (F4, F5, R4, R6, R7, R9) : la place du `409 BANK_IMPORT_PREVIEW_STALE`
   contre la fiche de la 27-3 (`git show story/27-3-profil-apres-apercu:_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md`,
   AC8, l. 170-190) ; `bank_imports.rs:1141` et `:433` ; `parser.rs:191-195` ; `grep -c "<TxDtls>"` de
   `v04_ntry_sign_fallback.xml` (3) et de `v04_minimal.xml` (1) ; `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` ;
   apostrophes des clés voisines (`grep -n "^bank-import-csv-errors-" crates/kesh-i18n/locales/fr-CH/messages.ftl`) ;
   `remove_typescript_nodes.js:86`, `:89` (sous `node_modules/svelte/src/compiler/phases/1-parse/`, dans ce worktree
   ou dans `/home/gcorbaz/devel/kesh/frontend/`).
9. **Recomptes** écrits par le commit — recompte depuis les fichiers, ne relis pas les nombres : 1 MEDIUM et 13 LOW
   distincts en P3 (bruts : rapports R et F) ; 22 AC ; 25 mutations (T1 : 15, T2 : 5, T4 : 5) ; 4 items AC19 ;
   C-27-2-1 à 28 (`grep -c '^### C-27-2-' …registre`) ; trend des LOW distincts 13 → 15 → 13.

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — la fiche est une spec, le code n'est pas encore écrit (« `ubs_synthetique_devise.csv` n'existe
pas », « la fixture `v04_amtdtls_foreign_currency.xml` est absente », « M25 n'est pas jouée », « la parenthèse de
`reconciliation_e2e.rs` n'est pas récrite » sont **attendus**). Un finding ne porte que sur ce que **le texte du commit
affirme du code existant**, ou sur ce qu'il **prescrit de faux ou d'infaisable**.

⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = commande `grep -nF` (ou `grep -n`,
`grep -c`, `sed -n`) **et sa sortie copiée** dans le rapport. Un CRITICAL ou HIGH qui affirme qu'un patch « n'a pas été
appliqué » se vérifie au `grep -nF` sur le fichier **final** (`HEAD`), jamais sur les numéros de ligne d'un hunk.

## Ce que tu rends

Findings numérotés `P4-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (`[régression P3]` si le défaut naît du commit `eb3f4932`, sinon `fiche d'origine`),
correction proposée. ⛔ **La liste des axes exercés ET non exercés** (les neuf ci-dessus, un par un, avec ce qui a été
lu) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p4-ciblee.md` ; dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` (le rapport seul). Aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, `docker`, **tout interpréteur** (`python`,
`python3`, `perl`, `ruby`, `node`, `deno`, `bash` ou `sh` sur un script, y compris un script écrit à la volée), `gh` en
écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé réel de Guy) : ni lu, ni cité, ni compté. Autorisés :
lecture (`Read`, `cat`, `sed -n`, `head`, `tail`), `grep`, `awk` en lecture vers la sortie standard, `wc`, `ls`,
`git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
