# Prompt — validation P3 de la spec, Story 15-12

*Versionné le 2026-10-09. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** : `_bmad-output/implementation-artifacts/15-12-cloture-dans-l-ordre.md`. La dernière remédiation est le commit `9b403aaf` (`git show 9b403aaf`) — lis le Change Log de la fiche.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #543.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. **La story vient d'être découpée** : l'index ci-dessus renvoie à `15-12a-cloture-dans-l-ordre.md` (l'ordre) et `15-12b-filet-sous-un-bilan-clos.md` (le filet) — relis LES DEUX fiches ensemble. Choix C100, C107 à C112. Axes prioritaires : rien n'est perdu au découpage (recompte AC/tâches/tests/mutations aux deux bornes contre `git show dae3a618:_bmad-output/implementation-artifacts/15-12-cloture-dans-l-ordre.md`) ; aucune dépendance résiduelle de A vers B (A livrable et testable seule) ; le lot de rapprochement à trois voies (`accept_one_rule`) ; `ORDER BY start_date` dans `OPEN_COVERING_DATE_SQL` et les verrous de clé suivante en REPEATABLE READ (étape (b)) ; la preuve HTTP du rejeu ; le texte du refus et ses 5 sites ; le manuel, PDF aplati compris.
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la dernière remédiation (`git show 9b403aaf`) a-t-elle introduit des défauts ? Contradictions entre AC, tâches, Dev Notes et choix du registre ; frontière avec les fiches sœurs ; faits recopiés sans vérification (numéros de ligne, fonctions, codes, clés i18n) — revérifie-les au code (`grep -nF`) ; recompte des AC et tâches ; implémentable sans deviner ?
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-12-p3-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
