# Prompt — revue de code P1, Story 25-4-c3-b (le reste dû au centime, l'écart en écriture)

*Versionné le 2026-09-30. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c3-b-arrondi-au-centime`. **Diff à revoir :
`git diff 262100d2 e4a9419b`** (un seul commit d'implémentation, aplati). Fiche :
`_bmad-output/implementation-artifacts/25-4-c3-b-arrondi-au-centime.md` (AC 1–8, Dev Notes, Change Log des
quatre passes de validation). Règles : `CLAUDE.md` (§ *Pattern batch*, § *Review Iteration Rule*,
§ *Le prompt d'une passe doit NOMMER le manuel*).

## Lentilles

- **A — Blind hunter** : lis le diff seul, sans la fiche, et cherche ce qui est faux : arithmétique
  décimale, signe de l'écart, lignes d'écriture, erreurs avalées, verrous, ordre des gardes.
- **B — Edge-case hunter** : diff + code environnant. Restes négatifs ou nuls, reste sous le demi-centime
  (#490), second paiement après un partiel, transaction bancaire à deux décimales, compte d'arrondi d'une
  AUTRE société, réglage absent vs ligne `company_invoice_settings` absente, compte de type Revenue, écart
  quand le compte d'arrondi est aussi le compte de contrepartie, annulation après archivage, la
  concurrence (verrou optimiste de la 25-4-c2, `FOR UPDATE` du compte d'arrondi : risque de deadlock ou de
  désarmement du verrou ?), `DATABASE_ERROR` vs code dédié.
- **C — Acceptance auditor** : diff + fiche. Chaque AC tenu ? Chaque site des huit touché — et **pars du
  symptôme** : `grep -rn "amount_due\|amountDue\|due_before\|due_after" crates/ frontend/src` — reste-t-il
  une comparaison au brut ? Les tests de l'AC 7 prouvent-ils ce qu'ils disent (rouges avant le patch, ou
  déclarés gardes de mutation) ? **Le manuel** : `docs/manual/fr/admin-manual.tex` (*Compte de
  différences d'arrondi*) et `user-manual.tex` (règlement, rapprochement) disent-ils vrai contre le code —
  **PDF aplati** (`pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' '`, attention aux ligatures
  ﬀ/ﬁ) vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`.
  CHANGELOG, i18n (4 locales), compteur `sitesTotal`, exemption `lint-i18n-ownership`.

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande exécutée et sa
  sortie, ou code cité relu), scénario d'échec concret, correction proposée. Pour tout finding affirmant
  qu'un code est absent ou présent : la sortie d'un `grep -nF`.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » sans elle
  ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/regen-test-schema.sh`),
`make`, `latexmk`, `git commit`/`add`/`stash`/`reset`/`checkout`/`switch`/`rebase`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx playwright`/`vitest`, `gh issue create`/`comment`/`edit`.
Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `cargo check`, `pdftotext` et
expériences (`node -e`) dans le scratchpad.
