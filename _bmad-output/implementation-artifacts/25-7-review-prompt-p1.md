# Prompt — revue de code P1, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. **Le diff à revoir** :
`git diff 85b51f5f 281afc54` (le commit de développement ; la fiche validée est à `85b51f5f`). Fiche :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` — AC 1 à 9, Dev Notes, Limites, et les **Completion
Notes**, qui déclarent deux écarts assumés. Règles : `CLAUDE.md`. Issue : `gh issue view 445`.

## Lentilles

- **B — Bugs et concurrence (`kesh-db`, `kesh-api`)** : `crates/kesh-db/src/repositories/opening_complement.rs` contre
  l'AC 4. L'ordre réel des requêtes respecte-t-il la règle (verrous d'abord, une ligne par requête ; lectures ordinaires
  ensuite ; refus en dernier) ? Une lecture ordinaire s'intercale-t-elle avant un verrou ? La requête des comptes
  verrouille-t-elle des lignes d'une autre société ou plus que les comptes saisis ? La contrepartie (signe, montant,
  écart nul), `decide_date` (bords : premier exercice ouvert dont le début est **après** `today` ; `today` avant tout
  exercice ; candidat clos), l'ordre des refus par compte. Le handler (`routes/opening_balances.rs`) : contrôles de forme
  (montants, échelle, une colonne, doublons, plafond), `retry_with` (la closure refait-elle tout ?), le status
  (`complement_status` est-il cohérent avec le POST ?). Le mapping des erreurs (`crates/kesh-api/src/errors.rs`) :
  statut et code par cause selon la table de l'AC 5 ; un refus peut-il finir en 500 ?
- **E — Cas limites, écran, i18n** : `frontend/src/routes/(app)/settings/opening-balances/+page.svelte`,
  `opening-balances-totals.ts` et leurs tests. Les formules de l'AC 2 (signe d'un compte contre-nature, compte de
  report exclu), l'avertissement de l'AC 1 (trois variantes, seulement sur saisie équilibrée, non bloquant), la grille de
  complément (seulement sous `ALREADY_HAS_ENTRIES`, `data-testid` distincts, contrepartie en direct, payload envoyé,
  rechargement, double clic, montant invalide). Les 4 catalogues `crates/kesh-i18n/locales/*/messages.ftl` : chaque clé
  utilisée existe dans les 4, les variables (`{ $number }`, `{ $account }`…) correspondent à celles passées par le code
  (Rust et Svelte), les replis Svelte disent la même chose que le fr-CH.
- **A — Conformité à la fiche et au manuel** : chaque AC est-il satisfait, et chaque test de l'AC 7 existe-t-il et
  prouve-t-il ce qu'il nomme (aurait-il échoué avant le patch ?) ? Les deux écarts déclarés sont-ils justifiés ? Le
  **manuel** : `docs/manual/fr/user-manual.tex` **et le PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr
  '\n' ' ' | tr -s ' '` vers `target/gate-logs/`) disent-ils vrai au regard du code livré (date, contrepartie, comptes
  complétables, numérotation, limites) ? Le CHANGELOG et le README promettent-ils plus que le code ? Les décomptes du
  Dev Agent Record (tests, clés, `sitesTotal`) se recomptent-ils depuis la source ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu),
correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh` et `25-7-validate-p6-lockprobe.sh`), `make`, `latexmk`, `git commit`/
`add`/`checkout`/`stash`, `sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `docker`, `gh issue create`/
`comment`/`edit`, aucune requête SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers `target/gate-logs/`.
