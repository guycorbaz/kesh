# Prompt — revue de code P1, Story 25-5-b (la balance porte un solde d'ouverture)

*Versionné le 2026-10-03. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-5-b-balance-solde-ouverture`. **Diff à revoir : `git diff 42623926 46edd970`**
— un seul commit d'implémentation. Fiche : `_bmad-output/implementation-artifacts/25-5-b-balance-solde-ouverture.md`
(AC 1–10, Dev Notes, Dev Agent Record). Issue : `gh issue view 385`. Règles : `CLAUDE.md`.

## Lentilles

- **A — Blind hunter** : le diff **seul**, sans la fiche. SQL de `trial_balance.rs` (liaisons dans le bon ordre —
  compte-les une à une contre les `?` ; `SUM(CASE … END)` sans `ELSE` → `NULL` couvert par `COALESCE` ? ; `CAST … AS
  SIGNED` lu en `i64` ; la jointure et le filtre de société), la requête d'ouverture réécrite du grand livre, `Decimal`,
  rendus CSV/PDF, réactivité Svelte 5 et `big.js`.
- **B — Edge-case hunter** : le diff et le code environnant.
  - Le grand livre : sa nouvelle requête d'ouverture rend-elle **exactement** l'ancien résultat dans tous les cas — compte
    de bilan, compte de résultat, `from` hors de tout exercice, `from` = premier jour d'un exercice, exercices avec trous ?
  - La balance : `fy_start` lu par `find_by_id_in_company` — exercice d'une autre société ? Une écriture d'une **autre
    société** sur un compte de cette société (impossible ?) ; une écriture datée hors de son exercice ; un compte retypé.
  - La règle d'inclusion : un compte archivé dont l'ouverture est nulle mais qui a des lignes **de montant net nul** dans
    la période ; `period_lines` compte-t-il les bonnes lignes ?
  - `opening_balanced` : faux positif possible sur des livres exacts (arrondi `Decimal`, échelle 4 décimales) ?
  - L'écran : `retainedEarnings` absent ou illisible, rapport vide, ligne calculée quand il n'y a aucun compte.
  - La performance : la sous-requête lit toutes les lignes jusqu'à `end` — index utilisés (`idx_jel_account`) ?
- **C — Acceptance auditor** : chaque AC tenu ? Recompte depuis la source (`sitesTotal`, relevé des libellés en dur, clés
  dans les 4 locales, décomptes de tests du Dev Agent Record aux deux bornes `42623926` / `46edd970`). **Pars du
  symptôme** : `grep -rn "\.balance\b\|totalDebit\|reports-column-balance" frontend/src crates/kesh-report crates/kesh-api/src`
  — un consommateur de la balance qui suppose encore l'ancienne forme ? Les tests prouvent-ils ce qu'ils disent (chaque
  mutation déclarée est-elle tuée par l'assertion écrite) ? Le **manuel** (`docs/manual/fr/user-manual.tex` et **PDF
  aplati**, `pdftotext … | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`) dit-il ce que fait le code — § *Balance des
  comptes*, § *Grand livre*, glossaire, § *Reprise de comptabilité* ? Le CHANGELOG ? Le Dev Agent Record n'affirme-t-il que
  ce qui a tourné ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu),
scénario d'échec, correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`.
⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
