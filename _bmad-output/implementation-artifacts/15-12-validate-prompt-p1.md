# Prompt — validation P1 de la spec, Story 15-12

*Versionné le 2026-10-08. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** : `_bmad-output/implementation-artifacts/15-12-cloture-dans-l-ordre.md`. C'est la première passe : la fiche n'a pas encore été remédiée.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #543.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Fiche 029f023c (choix C89). La branche de planification est en RETARD sur main : lis le CODE sur origin/main (git show origin/main:<chemin>) ou dans le worktree /home/gcorbaz/devel/kesh-15-6a (à jour, lecture seule). Attaquer en priorité : (1) la preuve du filet SANS verrou de l'AC8 — cherche un écrivain qui ne tient pas le verrou de son exercice, ou une écriture de fiscal_years.status hors close/reopen (grep au code) ; (2) l'AC 13 b : la course création/clôture — l'interblocage annoncé est-il réel (ordre des verrous lu au code) ? ; (3) la table de l'AC12 : toute écriture SQL sur journal_entries hors tests passe-t-elle par create_in_tx_inner / update / delete_in_tx ? (inventaire des sites NON résolus) ; (4) le motif 'ORDER BY start_date ASC … FOR UPDATE' qui risque de confondre les tests de concurrence de la 15-8a ; (5) les 10 sites frontend qui réagissent à FISCAL_YEAR_CLOSED (AC18) ; (6) la dérogation au découpage (8 modules) ; (7) le bandeau pour les installations déjà fautives et le sens de la réparation (rouvrir le plus récent clos).
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-12-p1-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
