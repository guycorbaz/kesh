# Prompt — validation P3 de la spec, Story 31-2

*Versionné le 2026-10-23. Deux lentilles (R et F), modèle **Opus**, contexte frais chacune, en lecture seule stricte.
Rotation : passes complètes Sonnet ↔ Opus ; la P1 était Opus, la P2 Sonnet, celle-ci est donc Opus. Haiku n'est employé que
pour une passe ciblée de fin de boucle. Une boucle ne se clôt que sur une passe à 0 MEDIUM+ (`CLAUDE.md` § *La passe ciblée*).*

Dépôt `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2` (ne change pas de branche). **Fiche** :
`_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md`. Registres : `epic-31-choix-autonomes.md`
(C-31-0-2, C-31-0-3) et `epic-31-choix-autonomes-312.md` (C-31-2-1 à C-31-2-14, dont les **amendements du 2026-10-23**, P1 puis
P2, en fin de fichier). Règles : `CLAUDE.md`, `docs/agents-orchestration.md`. Issues : #577, #698 (intégrée), #699 (ouverte) —
`gh api repos/guycorbaz/kesh/issues/<n>` en lecture. Rapports des passes précédentes :
`/home/gcorbaz/devel/kesh-gate-logs/312-validate-p{1,2}-{R,F}.md` ; leurs corrections sont dans le Change Log de la fiche
(« Validation P1 » et « Validation P2 »).

**La fiche a été remédiée après la P2.** Lis le Change Log, puis **cherche d'abord ce que la remédiation a introduit** (la
sévérité se déplace vers ce qu'on vient d'écrire), avant de rouvrir ce que la P1 et la P2 ont déjà tranché :
- l'**enveloppe Rust** `crates/kesh-db/tests/check_dev_grants_script.rs` (C-31-2-13) et ses conséquences : `kesh-db` touché,
  gate complet même en boucle, test qui ne doit jamais lire le conteneur réel, garde anti-test-muet, `docker` factice ;
- la distinction **« absent » / « en erreur »** du garde (trois cas qui rendent 0, tout le reste non nul), la comparaison du
  port de `DATABASE_URL` au port publié (`docker port`), le refus d'un attendu à zéro `GRANT`, `MYSQL_PWD` exporté puis
  transmis sans valeur, la normalisation « chaîne du condensé seule » ; sont-ils implémentables sans deviner, et les cas
  d'AC3 les couvrent-ils tous ? Le jeu des **mutations M1 à M12** attrape-t-il chacun, et chaque mutation a-t-elle son cas ?
- le **câblage hors du bloc lint** de `test-fast.sh` : l'assertion par numéros de ligne (`command -v cargo-nextest` < garde <
  `if [ "$RUN_LINT" = 1 ]`) est-elle exacte contre le fichier réel ?
- l'**ordre de livraison 4, 3, 1, 2** et le report en T5 du service, de la connexion et de `setup-buildx` d'`e2e-smoke` :
  chaque commit atteint-il les AC qu'on lui attribue ? AC13 en deux temps, comptes ancrés ;
- la connexion Docker Hub **avant** `setup-buildx-action` dans `docker-build`, `e2e-smoke` et `release.yml` (remontée), et
  l'AC14 nuancé (hypothèse sur le builder, constatée au journal) ;
- l'inventaire d'AC5 (forme `':(exclude)…'`, neuf sites, `docker-compose.dev.yml:107-108` et `README.md:183` ajoutés — C-31-2-14)
  et celui d'AC12 (motif `\be2e\b` retiré) : **ré-exécute-les en lecture** (`git grep`) et compare à la sortie reportée ;
- la conduite quand une branche modifie `01-dev-grants.sql`, la section « Gate » corrigée, le décompte des modules déclaré ;
- les recomptes depuis la source : **18 AC**, **8 tâches**, **12 mutations** ; rattachement AC -> tâche (AC13 : T2 puis T5).
Un patch dont le symptôme n'a pas été grepé sur tout le dépôt n'est pas terminé : greper les **valeurs** (« M1–M6 »,
« 17 AC », « trois jobs », « indépendants », « faute de second filet », « Aucun test automatique », « kesh-db non touché »,
`':!`), non les formulations.

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de l'issue #577,
  de l'epic § 31-2 et de C-31-2-10 à C-31-2-14 couverte par un AC testable, chaque AC par une tâche ; implémentable sans
  deviner ; **faits cités revérifiés** (`grep -nF`, `sed -n`) dans `ci.yml`, `release.yml`, `docs/ci.md`, `CLAUDE.md`,
  `docs/testing.md`, `docs/agents-orchestration.md`, `scripts/test-fast.sh`, `scripts/tests/prepare-release.test.sh`,
  `crates/kesh-db/tests/prepare_release_script.rs`, `docker-compose*.yml`, `scripts/mariadb-init/01-dev-grants.sql`,
  `README.md` ; cohérence interne AC <-> conception <-> mutations (M1-M12) <-> tâches <-> tableau « local / CI uniquement »
  <-> section « Gate » ; sections du gabarit BMAD (« Story », « Dev Agent Record ») ; plus aucune phrase « hors périmètre »
  ou « à ouvrir » pour #698/#699 ; recomptes (18 AC, 8 tâches, 12 mutations) depuis la source.
- **F — Full-scope adversary** : la conception remédiée. (1) le **garde** : acquisition `-N -B -r`, normalisation minimale,
  `_`, guillemets, ordre, `kesh@%` vs `kesh@localhost`, `kesh%` couvrant `kesh`, rôle `PUBLIC`, `WITH GRANT OPTION` ;
  « absent » vs « en erreur » : reste-t-il un chemin qui rend 0 sur un conteneur défaillant, ou non nul sur une machine sans
  conteneur de dev (CI comprise, où l'enveloppe tourne) ? le `cmp` du montage et la branche qui modifie le fichier ;
  `docker` factice : le test peut-il, malgré lui, atteindre le vrai conteneur ou le vrai `docker` ? mot de passe et ligne
  `USAGE` jamais affichés ni dans l'argv ; (2) le placement dans `test-fast.sh` et l'assertion de câblage (`--no-lint`) ;
  (3) **CI en `kesh`** : étape de droits (ordre, client `mariadb`, `-h`), `DATABASE_URL` de niveau job, tests qui exigeraient
  `root` ou `PROCESS` ; l'enveloppe Rust en CI (le job `backend` tourne-t-il sous `kesh` quand elle s'exécute ?) ;
  (4) **`release.yml`** : `DATABASE_URL` en `kesh` ; remontée de la connexion avant `setup-buildx` ; (5) `e2e-smoke` et
  `docker/login-action` (forks, secrets absents, `needs`, cache `gha`, ordre connexion/buildx) ; (6) **`.svelte-kit/`** ;
  (7) le **manuel** : `docs/manual/` (`.tex` et PDF aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) mentionne-t-il le
  smoke, la CI, les droits MariaDB de dev, Docker Hub anonyme ? ; (8) **textes à réécrire** : ré-exécute les inventaires
  d'AC5 et d'AC12 et cherche les sites que ni l'un ni l'autre ne nomment ; (9) le découpage et le périmètre minimal
  (le décompte déclaré par la fiche est-il juste, `kesh-db` touché change-t-il le calcul ?).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code
relu), correction proposée. Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas, et un point qualifié de
« robuste » doit avoir été exécuté (lecture) et non supposé.
⛔ **Écris ton rapport complet** dans `/home/gcorbaz/devel/kesh-gate-logs/312-validate-p3-<lentille>.md` (R ou F) — c'est
le seul fichier autorisé — et rends comme dernier message **seulement** : ce chemin, le bilan par sévérité, une ligne
par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. Aucun `git checkout`/`switch`/`stash`/`reset`/`add`/`commit`, sous aucune forme. Aucune commande
qui écrit, compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/check-dev-grants.sh` et
`scripts/tests/*.test.sh`), `make`, `latexmk`, `cargo` (toutes sous-commandes), `sqlx`, `npm`, `npx`, `node`, `python3`,
`docker` (sauf `docker inspect` et `docker exec … mariadb -e "SHOW GRANTS…"`/`SELECT` en lecture), `gh issue
create|comment|edit|close`, toute requête SQL d'écriture, tout `docker restart|stop|compose`, tout `GRANT`/`REVOKE`.
Aucun fichier temporaire : comparer par pipe. Autorisés : `grep`, `sed -n`, `cat`, `ls`, `git log|show|diff|ls-files|grep`,
`gh api` en lecture, `pdftotext … -` vers la sortie standard.
