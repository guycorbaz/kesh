# Prompt — validation P4 de la spec, Story 15-11a

*Versionné le 2026-10-08. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** : `_bmad-output/implementation-artifacts/15-11a-compose-transmet-la-configuration.md`. La dernière remédiation est le commit `9f9de55b` (`git show 9f9de55b`) — lis le Change Log de la fiche.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #550, #557, #558, #554.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. La P3 a ABANDONNÉ l'AC4 (montages fixes ./documents, ./inbox, ./log conservés dans docker-compose.prod.yml ; plus de recette de déplacement ; C83) et étendu l'AC16 (toute valeur <…> refusée pour KESH_JWT_SECRET et KESH_ADMIN_PASSWORD, is_template_placeholder, M26–M27). Axes imposés : (1) AC16 réécrite (fonction, trim local, ordre avant la longueur, 6 tests, M20 M24–M27, filtre nextest de l'AC14) ; (2) (T)/(E) réécrites, M14, M28, rouge exact du T1 ; (3) AC12 f : la commande 'docker compose config | grep -iE …' face au rendu YAML réel de Compose — tu peux l'essayer dans le scratchpad avec des .env piégés (guillemets, export, KEY = v) si docker est disponible, sinon dis-le ; (4) propagation de l'abandon de l'AC4 : cherche tout résidu de « recette », « trois gestes », « déplacez », « montages de P » dans la fiche, le registre et la fiche 15-7b3 (worktree /home/gcorbaz/devel/kesh-15-7, lignes admin-manual :1384, :1518) ; (5) AC12 e ; (6) recomptes.
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la dernière remédiation (`git show 9f9de55b`) a-t-elle introduit des défauts ? Contradictions entre AC, tâches, Dev Notes et choix du registre ; frontière avec les fiches sœurs ; faits recopiés sans vérification (numéros de ligne, fonctions, codes, clés i18n) — revérifie-les au code (`grep -nF`) ; recompte des AC et tâches ; implémentable sans deviner ?
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-11a-p4-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, commandes shell et `docker compose config` DANS `/tmp/claude-1000/-home-gcorbaz-devel-kesh/be9e770b-9545-4b04-bcdd-e93abb73499c/scratchpad/` seulement, `pdftotext`
vers `target/gate-logs/`.
