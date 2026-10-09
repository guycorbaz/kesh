# Prompt — validation P1 de la spec, Story 15-13

*Versionné le 2026-10-09. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-13`, branche `story/15-13-mariadb-et-sauvegarde`. **Fiche** : `_bmad-output/implementation-artifacts/15-13-mariadb-et-sauvegarde.md`. C'est la première passe : la fiche n'a pas encore été remédiée.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #551, #552 (et #558 pour le contexte).
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Code de référence : la branche elle-même (`de285ea8` + la fiche). Choix C-15-13-1 à 7 ; ce que la 15-11a (compose, `${KESH_X:-}`, refus des gabarits) et la 15-11b (`config::env_nonempty`, test lexical) ont livré. Issues voisines ouvertes à la spécification : #575, #576. **Axes prioritaires** : (1) la procédure de mise à jour et le tableau des cas des Dev Notes — aucun cas ne casse en silence, le texte ne pousse jamais à écrire un mot de passe NEUF dans `.env` ; (2) la syntaxe `${VAR:?message}` (message sans `}` ni `$`, accents) et l'étape CI dont la vérification négative doit réellement échouer ; (3) la faisabilité de l'indice sur l'erreur 1045 (ce que rend `MySqlPoolOptions::connect` en sqlx 0.8 — lis `~/.cargo/registry/src/*/sqlx-mysql-0.8*`) ; (4) le test 0600 et l'umask, la mutation M11 ; (5) l'inventaire des sites non résolus, à rejouer avec le grep du T9 ; (6) le manuel et son PDF aplati, largeurs de lignes.
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase des issues couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; implémentable sans deviner (fonctions, fichiers, codes d'erreur, clés i18n 4 locales, replis Svelte) ; faits cités revérifiés au code (`grep -nF`) ; recompte des AC et tâches.
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-13-p1-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
