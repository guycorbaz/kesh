# Prompt — validation P5 de la spec, Story 15-7b2

*Versionné le 2026-10-08. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-7`, branche `story/15-7-trace-onboarding`. **Fiche** : `_bmad-output/implementation-artifacts/15-7b2-remise-a-zero.md`. La dernière remédiation est le commit `9dd214f1` (`git show 9dd214f1`) — lis le Change Log de la fiche.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #528, #542, #434, #279, #540, #538, #546, #534.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. La P4 a remplacé attach_all_principals_in_tx par reattach_orphan_principals_in_tx (clés orphelines révoquées puis repointées, C-15-7-45), déplacé la réparation de #542 en 15-7b3 (C-15-7-46) et réécrit la sortie de la démonstration au manuel (KESH_PRODUCTION_RESET, C-15-7-48). Relire en priorité : AC 4 (verrou de la branche language à zéro société, NOT EXISTS sous FK=1, repointage sans toucher version), manuels :177-189, :691, :1314 et le tableau :683-693 du PDF.
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la dernière remédiation (`git show 9dd214f1`) a-t-elle introduit des défauts ? Contradictions entre AC, tâches, Dev Notes et choix du registre ; frontière avec les fiches sœurs ; faits recopiés sans vérification (numéros de ligne, fonctions, codes, clés i18n) — revérifie-les au code (`grep -nF`) ; recompte des AC et tâches ; implémentable sans deviner ?
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-7b2-p5-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext`
vers `target/gate-logs/`.
