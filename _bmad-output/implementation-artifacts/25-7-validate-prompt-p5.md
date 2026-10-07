# Prompt — validation P5 de la spec, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Deux lentilles (Sonnet), contexte frais chacune. Rotation (D6) : P1 Sonnet ×3 → P2 Opus ×2
→ P3 Sonnet ×2 → P4 Opus ×2 → P5 Sonnet ×2.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. **Fiche** :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` — lire son **Change Log** (P1 à P4). La remédiation de
P4 : `git diff 4e22b14f 0a8e2ae6 -- _bmad-output/implementation-artifacts/25-7-soldes-de-depart.md`. Issue :
`gh issue view 445`. Règles : `CLAUDE.md`. Aucun code n'est encore écrit.

**Constat qui motive la passe** : P2, P3 et P4 n'ont trouvé de défauts graves que dans la remédiation précédente,
sur l'**ordre des verrous**. P4 change de méthode : l'AC 4 ne prétend plus à un ordre sans cycle ; il suit une règle
(tous les verrous d'abord, une ligne par requête ; les lectures ordinaires ensuite ; les refus en dernier) et garantit
trois propriétés indépendantes de l'ordre, les cycles résiduels passant tous par un compte visé. **Ce sont ces
affirmations-là qu'il faut casser.**

## Lentilles

- **R — Regression hunter (l'AC 4 et ce qui en dépend)** : refais au code l'ordre de fait de **chaque** transaction
  qui écrit une écriture ou verrouille `companies`, `fiscal_years`, `accounts` ou `journal_entries` :
  `create_in_tx_inner` (avec et sans projet), `create_opening_entry`, `reverse_in_tx_inner`, `delete_in_tx` et
  `invoices::unvalidate`, les flux facture / fournisseur / rapprochement / paiement, `lock_books`, l'archivage et la
  modification d'un compte, la clôture d'exercice. Pour chacune, contre l'ordre de l'AC 4 : un **cycle à deux**
  existe-t-il que la section « Cycles » ne dit pas ? Les affirmations « aucun cycle à deux » et « préexistant »
  tiennent-elles ? `create_in_tx` lit `companies` sans verrou, puis l'exercice : y a-t-il un problème quand le
  complément tient déjà les deux ? L'instantané, ouvert par `NO_ENTRIES` : une lecture **ordinaire** postérieure
  (dans `create_in_tx` ou `validate_lines_accounts_in_tx`) lit-elle un état qui contredit un verrou pris plus tôt ?
  Les entrelacements (1) à (4) et les mutations déclarées tuées : réalisables, et tuées par le test nommé ?
- **F — Full-scope adversary** : la fiche entière, implémentable sans deviner ? Contradictions entre arbitrages, AC,
  tâches, Dev Notes, Limites, Change Log nées de la réécriture ; un code d'erreur de l'AC 5 sans test ni clé × 4
  locales ; un `completeReason` sans refus correspondant ; l'AC 2 et l'AC 1 contre l'étape 6 ; l'AC 6 contre les tests
  existants ; le **manuel** (`.tex` **et PDF aplati**, `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
  tr -s ' '` vers `target/gate-logs/`) et les autres sites de l'AC 8 ; les faits `fichier:ligne` introduits par P3
  (`grep -nF`).

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et sortie, ou
code cité relu), correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un
`grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL d'écriture.
Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
