# Prompt — validation P5 de la spec, Story 15-13b

*Versionné le 2026-10-09. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-13`, branche `story/15-13-mariadb-et-sauvegarde`. **Fiche** : `_bmad-output/implementation-artifacts/15-13b-sauvegarde-avant-import-persistante.md`. La dernière remédiation est le commit `3ebedca9` (`git show 3ebedca9`) — lis le Change Log de la fiche.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #552, #576.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Code de référence : la branche elle-même (`de285ea8` + les fiches). Index `15-13-mariadb-et-sauvegarde.md`, sœur `15-13a-…`. Choix C-15-13-1 à 26. **Signal D5** : un MEDIUM de P4 (R4-2/F-P4-1) était né de la remédiation P3 ; l'orchestrateur n'a pas redécoupé (défaut isolé dans un texte, fiche déjà issue d'un découpage) — si cette passe trouve un défaut né de la remédiation P4, dis-le explicitement. **Axes prioritaires** : l'écriture par `.partial` puis `sync_all` puis `rename` (C-15-13-21) — atomicité, refus d'écraser (vérification du nom final avant `rename`, fenêtre écrite en angle mort), le `.partial` laissé en cas d'échec, test `write_backup_file_passe_par_un_partiel` et M46/M26 ; le texte de #576 (C-15-13-25) dans les 4 locales et le repli Rust ; la liaison de `DEFAULT_ADMIN_BACKUP_DIR` au littéral et au montage (tests 3 et 6 b, M15) ; `source_conforme` et M14 ; le contrôle `target: /data/backup` ; l'inventaire (grep (4) par la valeur) ; le manuel et son PDF aplati.
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la dernière remédiation (`git show 3ebedca9`) a-t-elle introduit des défauts ? Contradictions entre AC, tâches, Dev Notes et choix du registre ; frontière avec les fiches sœurs ; faits recopiés sans vérification (numéros de ligne, fonctions, codes, clés i18n) — revérifie-les au code (`grep -nF`) ; recompte des AC et tâches ; implémentable sans deviner ?
- **F — Full-scope adversary** : la conception elle-même. Les chemins de code qui écrivent ou lisent ce que la story
  change (inventorier les sites NON résolus, pas énumérer ceux qui marchent) ; transactions, verrous, ordre des
  contrôles ; tests existants qui changeront de sens ; frontend qui lit les codes ou champs touchés (`grep -rn` dans
  `frontend/src`) ; i18n 4 locales et replis ; le **manuel** (`docs/manual/fr/*.tex` **et le PDF aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`) ;
  `docs/api-external.md` ; CHANGELOG ; la règle de découpage (§ « Règle de splitting préventif », amendement D5).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et
sortie, ou code cité relu), correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la
sortie d'un `grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-13b-p5-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
