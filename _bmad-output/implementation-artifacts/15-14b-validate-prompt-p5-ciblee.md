# Prompt — validation P5 ciblée, Story 15-14b

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Sonnet plutôt que
Haiku : la remédiation réécrit une recette de sauvegarde et de restauration, où une erreur coûte une base perdue),
contexte frais, en lecture seule, braquée sur le seul commit `f76bc2e4`.*

Dépôt `/home/gcorbaz/devel/kesh-15-14`, branche `story/15-14-lot-documentation-libelles` (base `245b91ee`).
**Objet** : `git show f76bc2e4 -- _bmad-output/implementation-artifacts/15-14b-exploitation-et-multi-societe.md`
puis la fiche à `HEAD`. Trend : P1 8 MEDIUM → P2 9 → P3 4 → P4 3 (tous dans la 15-14b, nés de la remédiation P3) ;
rapports `/home/gcorbaz/devel/kesh-gate-logs/15-14-validate-p{1,2,3,4}-{R,F}.md`. Choix C-15-14-33 à 37
(`_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`). La 15-14a est close : ne la relis pas.

## Lentille unique — chasseur de régressions de la remédiation P4

1. **Deux comptes, deux recettes** (C-15-14-35, fiche `:93-130`, `:185-205`, `:231-246`) : le compte de restauration
   (celui de `DATABASE_URL`, `GRANT ALL ON kesh.*`) peut-il réellement recharger un dump produit avec
   `--add-drop-database` / `--databases` (`DROP DATABASE`, `CREATE DATABASE` exigent-ils un privilège global ?
   vérifie au manuel MariaDB et aux options exactes du listing) ? Le contrôle négatif (recharger avec `kesh-dump.cnf`
   doit être refusé) est-il décidable ? `kesh-api` doit-il être arrêté pendant le rechargement, et la recette le dit-elle ?
2. **Post-script** (`:157-165`) et **G16 (d)** : le texte cible et la garde se tiennent-ils ? Le motif `rm` attrape-t-il
   trop (le `.tmp`, `rm -f` dans une autre recette) ou trop peu ?
3. **G16 (e) liste fermée** de quatre fragments (`:1741`, `:1748`, `:2476` avec renvoi ; `:912` exempté) : applique-la
   au texte cible écrit dans la fiche, site par site. Une occurrence d'« Hyper Backup » du texte cible manque-t-elle
   à la liste ?
4. **Contrôle de cohérence** écrit à la fiche (`:365-`) : rejoue-le toi-même, garde par garde (G14–G18), contre les
   textes cibles ; bornes de section prises sur le source brut au `\subsection{` qui porte le `\label`.
5. **Partition 14 + 29 = 43** (`:257-270`) : rejoue la commande d'inventaire telle qu'écrite (bloc `E`,
   `LC_ALL=C.UTF-8`) et compare ligne à ligne.
6. **AC 3** (`user-manual.tex:79` et `:91`, C-15-14-37) : vrai au code (`LoginRequest`, `CreateUserRequest`, champ
   « Identifiant ») ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de commande copiée), correction
proposée. ⛔ **Liste des axes exercés ET non exercés.** Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/15-14b-validate-p5-ciblee.md` ; dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Autorisés : lecture, `git grep`, `grep`,
`sed -n`, `git log`/`show`/`diff`, `pdftotext` vers la sortie standard, `perl` sans `-i`. Interdits : `scripts/*`,
`make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`/`fetch`, `sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en
écriture, SQL.
