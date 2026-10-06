# Prompt — revue de code P2, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Deux lentilles (Opus), contexte frais chacune. Rotation (D6) : P1 Sonnet ×3 → P2 Opus ×2.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. Fiche :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` (AC, Dev Notes, Completion Notes, Change Log — entrée
« Revue de code P1 »). Règles : `CLAUDE.md`. **La remédiation de P1** : `git diff 281afc54 face538e`. **Le code complet
de la story** : `git diff 85b51f5f face538e`.

## Lentilles

- **R — Regression hunter** : la remédiation de P1, ligne à ligne, puis son effet sur le reste.
  - `owned_account_ids` lit les comptes de la société **hors** de la transaction : cela casse-t-il une garantie
    (instantané, sérialisation, un compte qui changerait de société ou serait créé entre-temps, des identifiants en
    double, une liste vide) ? Le test qui tient le compte étranger prouve-t-il ce qu'il dit ?
  - Le rechargement après un refus (`+page.svelte`, `handleComplete`) : course avec un autre `load()` (jeton
    `loadGen`), message conservé ou effacé à tort, rechargement sur un refus de forme, état de `completing`.
  - `formatExact`, le plafond des montants, le libellé `#id` d'un compte étranger, les messages réécrits × 4 locales et
    leur repli Rust : cohérents entre eux et avec le manuel ?
  - Le Playwright réécrit (balance par l'API) : la forme JSON lue existe-t-elle (`rows`, `accountNumber`,
    `closingBalance`) ? `grep` le DTO de la balance.
- **F — Full-scope adversary** : le code complet de la story contre la fiche. Ce que P1 n'a pas regardé : les cinq
  entrelacements de `crates/kesh-db/tests/opening_complement_repository.rs` (chacun attend-il là où il le dit ; son
  assertion aurait-elle échoué sans le verrou qu'il nomme ?) ; la cohérence status / POST (une raison que le status
  rend `READY` alors que le POST refuserait, ou l'inverse) ; les catalogues de-CH, en-CH, it-CH relus pour le sens ; le
  **manuel** (`docs/manual/fr/user-manual.tex` **et le PDF aplati**, `pdftotext docs/manual/fr/user-manual.pdf - | tr
  '\n' ' ' | tr -s ' '` vers `target/gate-logs/`) contre le code livré ; le CHANGELOG ; la sécurité (RBAC, clés d'API
  `read`, isolation).

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
