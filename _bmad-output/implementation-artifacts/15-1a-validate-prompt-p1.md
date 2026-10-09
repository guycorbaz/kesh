# Prompt — validation P1 de la spec, Story 15-1a

*Versionné le 2026-10-08. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** : `_bmad-output/implementation-artifacts/15-1a-socle-lettrage.md`. C'est la première passe : la fiche n'a pas encore été remédiée.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #518.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Fiche réécrite (reprise du 2026-10-08, commit db532ebe, choix C90 à C99 ; index 15-1-lettrage.md ; sœurs 15-1a2, 15-1b, 15-1c). La branche de planification est en RETARD sur main : lis le CODE sur origin/main (git show origin/main:<chemin>) ou dans le worktree /home/gcorbaz/devel/kesh-15-6a (à jour, lecture seule). Attaquer en priorité : (1) C90 : aucune relève de kesh_version_min_required alors qu'un binaire antérieur relancé sur une base lettrée pourrait modifier une écriture lettrée et casser un groupe sans signal — la politique de migration (P1–P8 du CLAUDE.md) l'impose-t-elle ? quelle est la bonne décision ? ; (2) C90 : l'inventaire de tables strictement identique à l'import d'installation (vérifie l'argument au code de l'import) et ce que deviennent deux colonnes neuves à la restauration d'une sauvegarde antérieure (post_restore, backfill) ; (3) l'ordre des verrous R7 (en-têtes par id, lignes, exercices partagés) contre update_in_tx, la contre-passation, delete_in_tx, et le rejeu livré (15-5e1/15-5e2) ; (4) la garde ENTRY_LETTERED sur modification d'en-tête seul et sur suppression, intégrée à reversal_blockers / modification_guard (15-8a/15-8b) ; (5) C94 délettrage à cheval sur deux exercices ; (6) comptes lettrables (C96) ; (7) la règle de découpage.
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-1a-p1-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
