# Prompt — revue de code P3, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Deux lentilles (Sonnet), contexte frais chacune. Rotation (D6) : P1 Sonnet ×3 → P2 Opus ×2 → P3 Sonnet ×2.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. Fiche :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` (AC, Dev Notes, Completion Notes, Change Log — entrée
« Revue de code P2 »). Règles : `CLAUDE.md`. **La remédiation de P2** : `git diff b162abdb 7af648a3`. **Le code complet
de la story** : `git diff 85b51f5f 7af648a3`.

## Lentilles

- **R — Regression hunter** : la remédiation de P2, ligne à ligne, puis son effet sur le reste. Le test rendu
  indépendant de l'horloge (`complete_each_refusal_has_its_code`) : tient-il un 1ᵉʳ janvier, un 31 décembre, en 2026
  et après ? La borne de la contrepartie (`parse_complement_lines`) et son test. `SHAPE_REFUSALS` dans l'écran : la
  liste est-elle exactement celle des refus de forme du handler ? Le refus en tête de `create_opening_complement`
  (liste vide). Les messages réécrits (nom du rôle, « en service et imputables ») × 4 locales, replis Svelte et Rust :
  le nom du rôle cité est-il **exactement** celui que le plan comptable affiche (`account-role-retained-earnings`) ?
  Le manuel (`.tex` et PDF aplati vers `target/gate-logs/`) et le CHANGELOG réécrits disent-ils vrai au regard du code ?
- **F — Full-scope adversary** : le code complet de la story contre la fiche, en cherchant ce que P1 et P2 n'ont **pas**
  regardé : les tests Vitest (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`) relus ligne à ligne
  contre l'AC 7 et l'écran ; le Playwright ; la sérialisation JSON du status (noms camelCase, `null`) contre les types
  TypeScript ; l'accessibilité de l'écran (rôles, `aria-invalid`, libellés de champs, le `role="status"` de
  l'avertissement) ; le comportement de l'écran quand `fetchAccounts` échoue sous `ALREADY_HAS_ENTRIES` ; ce que dit
  l'écran après un complément qui vide la liste.

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
