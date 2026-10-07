# Prompt — revue de code P4, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Deux lentilles (Opus), contexte frais chacune. Rotation (D6) : P1 Sonnet ×3 → P2 Opus ×2 → P3 Sonnet ×2 → P4 Opus ×2.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. Fiche :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` (AC, Dev Notes, Completion Notes, Change Log — entrée
« Revue de code P3 »). Règles : `CLAUDE.md`. **La remédiation de P3** : `git diff 5faf1cc8 489026a2`. **Le code complet
de la story** : `git diff 85b51f5f 489026a2`.

## Lentilles

- **R — Regression hunter** : la remédiation de P3, ligne à ligne. La fusion de la saisie au rechargement
  (`+page.svelte`, `load()`) : que se passe-t-il au premier chargement, après un succès (la saisie d'un compte qui
  reste proposé doit-elle survivre à un complément **réussi** ?), après une génération, quand un compte change de nom ?
  Les `aria-label` (valeur exacte rendue, quatre locales, `{ $name }` qui contiendrait des guillemets) ; `scope` ; le
  message de dépassement × 4 et son repli Rust ; l'`afterEach` qui restaure `confirm` ; les tests ajoutés
  prouvent-ils ce qu'ils nomment ? Le manuel (`.tex` et PDF aplati vers `target/gate-logs/`) : plus aucun ancien nom
  du rôle, et rien de faux introduit.
- **F — Full-scope adversary** : le code complet de la story (`kesh-db`, `kesh-api`, écran, i18n, manuel), en
  cherchant un défaut **d'origine** que trois passes n'ont pas vu. Priorités : ce qui ferait perdre ou fausser une
  écriture comptable (montant, sens, compte, date, exercice) ; ce qui ferait mentir l'écran ou le manuel à
  l'utilisateur ; ce qui casserait sous une autre langue comptable que le français (libellé de l'écriture,
  `accounting_language`).

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
