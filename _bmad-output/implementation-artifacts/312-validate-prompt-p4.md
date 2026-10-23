# Prompt — validation P4 de la spec, Story 31-2

*Versionné le 2026-10-24. Deux lentilles (R et F), modèle **Sonnet**, contexte frais chacune, en lecture seule stricte.
Rotation : passes complètes Sonnet ↔ Opus ; la P3 était Opus, celle-ci est donc Sonnet. Haiku n'est employé que pour une passe
ciblée de fin de boucle. Une boucle ne se clôt que sur une passe à 0 MEDIUM+ (`CLAUDE.md` § *La passe ciblée*).*

Dépôt `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2` (ne change pas de branche). **Fiche** :
`_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md`. Registres : `epic-31-choix-autonomes.md`
(C-31-0-2, C-31-0-3) et `epic-31-choix-autonomes-312.md` (C-31-2-1 à C-31-2-15, dont les **amendements du 2026-10-23** et **du
2026-10-24** en fin de fichier). Règles : `CLAUDE.md`, `docs/agents-orchestration.md`. Issues : #577, #698 (intégrée), #699
(ouverte) — `gh api repos/guycorbaz/kesh/issues/<n>` en lecture. Rapports des passes précédentes :
`/home/gcorbaz/devel/kesh-gate-logs/312-validate-p{1,2,3}-{R,F}.md` ; leurs corrections sont dans le Change Log de la fiche
(« Validation P1 », « P2 », « P3 »).

**La fiche a été remédiée après la P3, et le volet 1 a été RÉÉCRIT.** La P3 a levé un **signal de recyclage** (sévérité égale à
la P2, tous les MEDIUM sur le garde) ; la sortie retenue est un **retrait** (C-31-2-15) : le garde `check-dev-grants.sh` ne fait
plus que comparer l'ensemble des droits de `kesh` sur le conteneur à celui du fichier versionné. **ATTAQUE D'ABORD CETTE
RÉÉCRITURE** (1a, 1b, AC1 à AC4, mutations, tâche T3, Dev Notes), avant de rouvrir le reste : la sévérité se déplace vers ce
qu'on vient d'écrire. Points à éprouver, sans te limiter à eux :
- la **conception réduite** est-elle implémentable sans deviner, et sans contradiction interne ? `KESH_DOCKER` comme seule
  injection ; les **deux** « garde sauté » (docker introuvable, conteneur non en marche — `docker ps --filter … -q` vide) et
  leurs messages distincts ; `ps` ou `exec` en échec → non nul ; aucune lecture du texte d'erreur de `docker` ; reste-t-il un
  chemin qui rend 0 sur un conteneur défaillant, ou non nul sur une machine sans conteneur de dev (CI, où l'enveloppe Rust
  tourne) ?
- le mot de passe tiré de **`MARIADB_PASSWORD`** du compose (non de `DATABASE_URL`), exporté puis transmis par le nom
  (`-e MYSQL_PWD`) : l'assertion sur l'argv consigné par le `docker` factice (AC3, M13) peut-elle réellement rougir ?
- les **lectures relatives à l'emplacement du script** et le **mini-dépôt** sous `mktemp -d` : suffisent-ils à faire varier
  l'attendu (cas « zéro `GRANT` ») sans lire le dépôt réel ? le `docker` factice qui échoue bruyamment sur toute sous-commande
  imprévue empêche-t-il vraiment d'atteindre le vrai `docker` ?
- la **normalisation** (compte, `;`, chaîne du condensé seule, ligne `USAGE` attendue, écart « ligne USAGE : … » sans condensé)
  est-elle complète et cohérente avec « la sortie ne contient ni `IDENTIFIED BY PASSWORD` ni le mot de passe » ?
- l'**assertion de câblage** sur l'unique ligne d'invocation non commentée (AC4, M3, M7 avec commentaire laissé) : exacte contre
  `scripts/test-fast.sh` réel ? (`command -v cargo-nextest` < garde < `if [ "$RUN_LINT" = 1 ]`)
- le jeu des **mutations M1 à M13** : chacune a-t-elle son cas dans AC3, chaque cas sa mutation ? M2 et M8 ont été
  **recomposées** (ancien contenu : montage, `cmp`) ; **propagation** : aucun résidu hors Change Log et entrées datées de
  « port », « mount-source », « grants-from-stdin », « cmp », « No such object », ni de « fichier du montage » vivant.
- les **angles morts écrits** (rôle `PUBLIC`, socket contre TCP, règle « depuis le dépôt principal » non vérifiée, #699) :
  sont-ils honnêtes, et la fiche ne promet-elle plus rien que le garde ne tienne ? (AC5, Dev Notes, tableau 1c, registre)
Puis le reste de la remédiation P3 : `README.md:193` et `reconciliation_e2e.rs:8249-8250` à l'inventaire d'AC5 (ré-exécute la
commande d'inventaire en lecture et cherche les sites que ni elle ni les ajouts ne nomment) ; recettes `up`/`restart` portant
la règle ; AC12 (`CLAUDE.md:269` annoté, non réécrit) ; AC13 (forme `- uses:`) ; AC14 (trois vérifications requises, à
revérifier par `gh api` en lecture) ; AC15 (`python3`, refait au dernier commit YAML) ; décompte à trois modules ; tableau
« local / CI » et section « Gate » ; table 1c ; recomptes depuis la source : **18 AC**, **8 tâches**, **13 mutations** ;
rattachement AC -> tâche.
Un patch dont le symptôme n'a pas été grepé sur tout le dépôt n'est pas terminé : greper les **valeurs** (« M1–M12 »,
« 12 mutations », « C-31-2-14 » comme borne haute, « trois modules »/« deux modules », « `--grants-from-stdin` »,
« `--mount-source` », `DATABASE_URL` à propos du mot de passe du garde), non les formulations.

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de l'issue #577,
  de l'epic § 31-2 et de C-31-2-10 à C-31-2-15 couverte par un AC testable, chaque AC par une tâche ; implémentable sans
  deviner ; **faits cités revérifiés** (`grep -nF`, `sed -n`) dans `ci.yml`, `release.yml`, `docs/ci.md`, `CLAUDE.md`,
  `docs/testing.md`, `docs/agents-orchestration.md`, `scripts/test-fast.sh`, `scripts/tests/prepare-release.test.sh`,
  `crates/kesh-db/tests/prepare_release_script.rs`, `docker-compose*.yml` (dont `MARIADB_PASSWORD`, :65),
  `scripts/mariadb-init/01-dev-grants.sql`, `README.md` ; cohérence interne AC <-> conception <-> mutations (M1-M13) <->
  tâches <-> tableau « local / CI uniquement » <-> section « Gate » ; sections du gabarit BMAD (« Story », « Dev Agent
  Record ») ; recomptes (18 AC, 8 tâches, 13 mutations) depuis la source.
- **F — Full-scope adversary** : la conception remédiée. (1) le **garde réduit** : acquisition `-N -B -r`, normalisation,
  `_`, antislashs, `kesh@%` contre `kesh@localhost`, `kesh%` couvrant `kesh`, `WITH GRANT OPTION`, ordre des contrôles ;
  `docker` factice (peut-il, malgré lui, atteindre le vrai `docker` ou le vrai conteneur, sur le poste comme en CI ?) ;
  mot de passe et ligne `USAGE` jamais affichés ni dans l'argv ; (2) le placement dans `test-fast.sh` et l'assertion de
  câblage (`--no-lint`) ; (3) **CI en `kesh`** : étape de droits (ordre, client `mariadb`, `-h`), `DATABASE_URL` de niveau job,
  tests qui exigeraient `root` ou `PROCESS` ; l'enveloppe Rust en CI ; (4) **`release.yml`** : `DATABASE_URL` en `kesh` ;
  connexion avant `setup-buildx` ; (5) `e2e-smoke` et `docker/login-action` (forks, secrets absents, `needs`, cache `gha`) ;
  (6) **`.svelte-kit/`** ; (7) le **manuel** : `docs/manual/` (`.tex` et PDF aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s
  ' '`) mentionne-t-il le smoke, la CI, les droits MariaDB de dev, Docker Hub anonyme ? ; (8) **textes à réécrire** :
  ré-exécute les inventaires d'AC5 et d'AC12 et cherche les sites que ni l'un ni l'autre ne nomment ; (9) le découpage et le
  périmètre minimal (le décompte à trois modules est-il juste ?).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code
relu), correction proposée. Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas, et un point qualifié de
« robuste » doit avoir été exécuté (lecture) et non supposé. Un « 0 » se vérifie comme un finding : l'orchestrateur reprendra
les axes que tu déclares non exercés.
⛔ **Écris ton rapport complet** dans `/home/gcorbaz/devel/kesh-gate-logs/312-validate-p4-<lentille>.md` (R ou F) — c'est
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
