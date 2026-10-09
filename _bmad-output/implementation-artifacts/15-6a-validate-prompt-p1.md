# Prompt — validation P1 de la spec, Story 15-6a

*Versionné le 2026-10-08. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-6`, branche `story/15-6-creance-juste-avoir-reglement`. **Fiche** : `_bmad-output/implementation-artifacts/15-6a-avoir-credite-la-creance-de-la-vente.md`. C'est la première passe : la fiche n'a pas encore été remédiée.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : 473, 523.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Fiche d'index 15-6, sœurs 15-6b et 15-6c. ⚠️ L'issue #523 (l'avoir relit aussi le compte d'arrondi dans les réglages) a été ouverte APRÈS l'écriture de la fiche, qui l'avait laissée en angle mort (C-15-6-2) : l'orchestrateur décide qu'elle entre dans la 15-6a — juge comment la traiter sans casser #486, et signale ce qu'il faut changer. Le dépôt principal (autre branche) contient des stories sœurs 15-5a/b/c qui touchent des fichiers voisins : `/home/gcorbaz/devel/kesh/_bmad-output/implementation-artifacts/15-5*.md` (lecture seulement).
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
Rapport compact.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
