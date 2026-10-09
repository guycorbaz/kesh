# Prompt — validation P3 de la spec, Story 15-1a

*Versionné le 2026-10-09. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** : `_bmad-output/implementation-artifacts/15-1a-socle-lettrage.md`. La dernière remédiation est le commit `58e9dac0` (`git show 58e9dac0`) — lis le Change Log de la fiche.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #518, #543.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Choix C90, C94, C101 à C106, C113 à C118 ; fiches sœurs 15-1a2, 15-1b, 15-1c, index 15-1-lettrage.md ; prérequis 15-12a (`15-12a-cloture-dans-l-ordre.md`). **Axes prioritaires** : la règle « ligne en période ouverte » de C113 (exercice ouvert, aucun postérieur clos, date après le verrou de période) contre le code des verrous de période et la 15-12a, et la garde de l'état hérité (modèle 15-8a) ; la séquence de verrous de C114 (min de `start_date` lu sans verrou, puis `start_date >= ? ORDER BY start_date ASC FOR UPDATE`) : ordre réel, verrous de clé suivante en REPEATABLE READ, course entre la lecture non verrouillée et le verrouillage, cohérence avec la 15-12a ; le motif du bump et l'en-tête de migration arrêtés mot pour mot (C115, règle P8) ; la relecture du corps 201 de `reverse` (C116) ; la précédence dans `delete_in_tx` (C117) ; le déclencheur de découpage (C118) ; le manuel, PDF aplati compris. Si un défaut naît d'un correctif de P2 sur une règle métier, signale-le explicitement (C118 prévoit alors le découpage).
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la dernière remédiation (`git show 58e9dac0`) a-t-elle introduit des défauts ? Contradictions entre AC, tâches, Dev Notes et choix du registre ; frontière avec les fiches sœurs ; faits recopiés sans vérification (numéros de ligne, fonctions, codes, clés i18n) — revérifie-les au code (`grep -nF`) ; recompte des AC et tâches ; implémentable sans deviner ?
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-1a-p3-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
