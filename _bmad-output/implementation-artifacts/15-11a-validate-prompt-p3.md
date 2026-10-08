# Prompt — validation P3 de la spec, Story 15-11a

*Versionné le 2026-10-08. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/15-5-gardes-postabilite-serveur`. **Fiche** : `_bmad-output/implementation-artifacts/15-11a-compose-transmet-la-configuration.md`. La dernière remédiation est le commit `b31d76fb` (`git show b31d76fb`) — lis le Change Log de la fiche.
Issues, par `gh api repos/guycorbaz/kesh/issues/N` (et `/comments`) — `gh issue view` échoue ici : #550, #557, #534, #551, #552, #554.
Choix autonomes : `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. La P2 a réécrit l'AC16 (placeholder GENERATE_ME refusé pour KESH_JWT_SECRET ET KESH_ADMIN_PASSWORD, contrôle avant la longueur, M24/M25), la recette de déplacement des dossiers (AC12 f, rejouée sur dix .env piégés), ajouté l'AC12 (j) (mise en page des dix tableaux de sec:env-vars, reprise de la 15-7b2) et propagé le placeholder (README, compose, manuel). Axes imposés : rejoue la recette de l'AC12 f TELLE QU'ÉCRITE dans la fiche, au scratchpad, sur des .env piégés que tu construis toi-même (guillemets, \r, espaces, doublons, commentaires, chemin relatif, dossier absent) — tu peux exécuter des commandes shell dans le scratchpad, jamais dans le dépôt ; cohérence AC16 / M24 / M25 / T3 / AC14 ; AC12 (j) face à la 15-7b2 (worktree /home/gcorbaz/devel/kesh-15-7) et à la 15-11b (:662) ; grep du T8 rejoué ; recomptes (16 AC, 10 tâches, 25 mutations).
Aucun code de la story n'est encore écrit.

## Lentilles

- **R — Regression hunter** : la dernière remédiation (`git show b31d76fb`) a-t-elle introduit des défauts ? Contradictions entre AC, tâches, Dev Notes et choix du registre ; frontière avec les fiches sœurs ; faits recopiés sans vérification (numéros de ligne, fonctions, codes, clés i18n) — revérifie-les au code (`grep -nF`) ; recompte des AC et tâches ; implémentable sans deviner ?
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
⛔ **Écris ton rapport complet** dans `target/gate-logs/15-11a-p3-<lentille>.md` (R ou F ; autorisé), et rends
comme dernier message **seulement** : ce chemin, le bilan par sévérité, et une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une
base : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`,
`sqlx`, `cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`, aucune requête SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, commandes shell DANS `/tmp/claude-1000/-home-gcorbaz-devel-kesh/be9e770b-9545-4b04-bcdd-e93abb73499c/scratchpad/` seulement (essais de la recette), `pdftotext`
vers `target/gate-logs/`.
