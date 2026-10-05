# Prompt — validation P2, Story 25-6-a (le tableau de bord dit vrai)

*Versionné le 2026-10-03. Une passe (Opus), contexte frais, les trois lentilles. Trend : P1 4H/7M (Sonnet ×3, dont un HIGH
réfuté).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-a-tableau-de-bord`. **Fiche :**
`_bmad-output/implementation-artifacts/25-6-a-tableau-de-bord.md` (tête `89511559`). Remédiation de P1 :
`git diff 06c9a685 89511559`. Issues : `gh issue view 388`, `389`. Règles : `CLAUDE.md`. La fiche décrit du travail **à
faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Ce que tu vérifies

1. **La remédiation de P1** :
   - **La réfutation du HIGH A1 tient-elle ?** Vérifie le signe du solde CAMT (`crates/kesh-import/src/camt053/`) et
     celui d'un compte de passif lié (`débit − crédit`), sur un découvert et une ligne de crédit tirée. Calcul chiffré.
   - **Le contrat de la requête (AC 6) est-il réalisable** dans `bank_accounts.rs:620-690` sans changer
     `currentBalance` ni `lastTransactionDate` — y compris quand deux comptes bancaires partagent un compte de grand livre
     (la requête passe de « par compte de grand livre » à « par compte bancaire ») ?
   - **Le dédoublonnage du total (AC 3)** : par `journalAccountId` — quel solde garder, et que devient un compte sans
     compte lié ? La note, en quels termes ?
   - **`formatChfBalance` sur `Big`** : `new Big(v).toFixed(2)` arrondit-il comme le reste du dépôt (mode d'arrondi de
     `big.js` par défaut) ? Ses autres appelants (`grep -rn formatChfBalance frontend/src`) restent-ils compilables ?
   - **L'extraction en composants dans `features/homepage/`** : la garde `lint-i18n-ownership`
     (`frontend/scripts/lint-i18n-ownership.js`, `keyBelongsToFeature`) admet-elle bien les clés `homepage-*` dans ce
     dossier ? D'autres clés que la page emploie aujourd'hui (`grep -o "msg('[a-z-]*'" "frontend/src/routes/(app)/+page.svelte"`)
     seraient-elles refusées une fois déplacées ?
2. **Cohérence interne** : la fiche dit-elle la même chose partout (AC, tâches, Dev Notes, *Modules*, limites) ?
3. **Ce que P1 n'a pas regardé** : l'accessibilité des tuiles (titres, liens, axe — un E2E axe existe-t-il pour
   l'accueil ?) ; le rendu de l'écart (couleur seule ?) ; une clé API sur ces endpoints ; la taille du `limit=5` face au
   N+1 réel de la liste (`journal_entries.rs`, lecture des lignes) ; le mode expert/guidé et les textes de
   `homepage-*-empty-guided`.
4. **Le manuel** — `docs/manual/fr/user-manual.tex` § *Tableau de bord* et l'import bancaire, **PDF aplati** vers
   `target/gate-logs/`.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute
présence ou absence, calcul chiffré), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`, `node -e` sans effet de bord.
