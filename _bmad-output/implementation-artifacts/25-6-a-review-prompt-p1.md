# Prompt — revue de code P1, Story 25-6-a (le tableau de bord dit vrai)

*Versionné le 2026-10-03. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-a-tableau-de-bord`. **Diff à revoir : `git diff 91ca244f 9e17de0c`**
(non-régression `8713c9de`, backend `297003c2`, frontend et docs `9e17de0c`). Fiche :
`_bmad-output/implementation-artifacts/25-6-a-tableau-de-bord.md` (AC 1–8, limites, Dev Agent Record). Issues :
`gh issue view 388`, `389`. Règles : `CLAUDE.md`.

## Lentilles

- **A — Blind hunter** : le diff **seul**, sans la fiche. SQL de `list_by_company_with_balances` (liaisons dans l'ordre —
  compte-les contre les `?` ; sous-requêtes corrélées ; `ROW_NUMBER()` ; `CAST … AS SIGNED` ; filtres de société dans
  CHAQUE sous-requête) ; `Decimal` ; Svelte 5 (`$props`, `$derived`, `{@const}`) ; `big.js` (aucun `Number` sur un
  montant non arrondi) ; gestion d'erreur des quatre chargements de la page.
- **B — Edge-case hunter** : le diff et le code environnant.
  - Le calcul, refait chiffré : écriture de rapprochement à **plusieurs lignes** sur le compte lié ; rapprochement par
    **éclatement** (split) ; relevé dont `period_to` est antérieur à toute écriture ; `include_archived` vrai/faux et le
    partage ; un compte bancaire sans relevé mais lié ; deux sociétés.
  - La page : un rôle Consultation (403 évité ?), un appel qui échoue pendant que les autres réussissent, le mode guidé,
    un montant négatif, `lastTransactionDate` désormais formaté (`formatSwissDate`) — d'autres lecteurs l'attendaient-ils
    brut ? `formatChfBalance` : ses appelants passent-ils tous une chaîne (`grep -rn formatChfBalance frontend/src`) ?
  - Les E2E neufs : déterministes ? la facture créée par un test est-elle la seule ouverte (le compteur peut-il valoir
    autre chose) ? l'axe sans relevé couvre-t-il ce qu'il prétend ?
- **C — Acceptance auditor** : chaque AC tenu ? Recompte depuis la source : `sitesTotal`, `relais`, `sitesNonResolus`
  (`frontend/src/lib/shared/i18n-keys.test.ts`), parité des clés `homepage-*` dans les 4 locales, décomptes de tests du
  Dev Agent Record aux deux bornes `91ca244f` / `9e17de0c`. **Pars du symptôme** :
  `grep -rn "liquidit\|currentBalance\|Number(" frontend/src/lib/features/bank-accounts frontend/src/lib/features/homepage "frontend/src/routes/(app)"`.
  Les tests prouvent-ils ce qu'ils disent (chaque mutation déclarée tuée l'est-elle par l'assertion écrite) ? Le
  **manuel** (`docs/manual/fr/user-manual.tex` et **PDF aplati**, `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
  `target/gate-logs/`) dit-il ce que fait le code — § *Tableau de bord*, *Solde comptable, relevé et écart*, *Import
  bancaire* ? Le CHANGELOG ? Les deux écarts déclarés au Dev Agent Record (testid remplacé, relevé non couvert en E2E)
  sont-ils acceptables ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu),
scénario d'échec, correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`.
⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
