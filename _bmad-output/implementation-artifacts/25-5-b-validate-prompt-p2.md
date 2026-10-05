# Prompt — validation P2, Story 25-5-b (la balance porte un solde d'ouverture)

*Versionné le 2026-10-03. Une passe (Opus), contexte frais, les trois lentilles. Trend : P1 1H/4M (Sonnet ×3).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-5-b-balance-solde-ouverture`. **Fiche :**
`_bmad-output/implementation-artifacts/25-5-b-balance-solde-ouverture.md` (tête `b6c1d5f4`). Remédiation de P1 :
`git diff 7e529131 b6c1d5f4`. Issue : `gh issue view 385`. Règles : `CLAUDE.md`. La fiche décrit du travail **à faire** :
un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Ce que tu vérifies

1. **La remédiation de P1** (le diff) : chaque ajout est-il juste ? En particulier :
   - la formule `debit_sense` et la vérification chiffrée de l'AC 5 — refais le calcul ;
   - le test du contrôle à `false` : une ligne déséquilibrée insérée en SQL brut dans l'exercice 1 casse-t-elle
     **réellement** les deux égalités (ouverture ET clôture) sans faire échouer le rapport ? `fetch_retained_earnings`
     lit-il cette ligne (selon le type du compte choisi, elle tombe dans le résultat reporté ou dans une ouverture —
     l'écart survit-il ?) ; `TrialBalanceUnbalanced` (mouvements de la période) reste-t-il muet ?
   - les positions des six colonnes du PDF (`pdf.rs`, police Helvetica 10 pt, `MARGIN_LEFT_MM`, `PAGE_WIDTH_MM`,
     `truncate_with_ellipsis`) : chevauchement ? la colonne de clôture déborde-t-elle de la page ?
   - les scénarios ajoutés à l'AC 9 contredisent-ils un autre AC ?
2. **Ce que P1 n'a pas regardé** — cohérence interne de la fiche (AC, tâches, Dev Notes, *Modules*) ; la règle d'ouverture
   pour une période **dont le début n'est pas le début de l'exercice** combinée à la règle d'inclusion (AC 6) ; un compte de
   **résultat** archivé à ouverture non nulle en cours d'exercice ; la ligne calculée au CSV (libellé fixe fr — cohérent
   avec les autres lignes CSV de `csv.rs` ?) ; la doc de l'API publique (`docs/` — `grep -rn "trial-balance" docs`) et le
   manuel admin s'ils décrivent la réponse ; le test de concordance existant `general_ledger.rs:150-205` : étendu à
   `opening`/`closing`, son scénario (exercice entier) prouve-t-il quelque chose sur l'ouverture ?
3. **Le manuel** — `docs/manual/fr/user-manual.tex` (§ *Balance des comptes*, § *Grand livre*, glossaire) **et le PDF
   aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) : la fiche nomme-t-elle
   tous les sites que le patch rendra faux ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute
présence ou absence, calcul chiffré), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`,
`gh issue view`, `pdftotext` vers le scratchpad.
