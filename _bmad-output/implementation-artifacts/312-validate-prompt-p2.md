# Prompt — validation P2 de la spec, Story 31-2

*Versionné le 2026-10-23. Deux lentilles (R et F), modèle **Sonnet**, contexte frais chacune, en lecture seule stricte.
Rotation : passes complètes Sonnet ↔ Opus ; la passe 1 était Opus, celle-ci est donc Sonnet. Haiku n'est employé que pour
une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2` (ne change pas de branche). **Fiche** :
`_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md`. Registres : `epic-31-choix-autonomes.md`
(C-31-0-2, C-31-0-3) et `epic-31-choix-autonomes-312.md` (C-31-2-1 à C-31-2-12, dont les **amendements du 2026-10-23**
en fin de fichier). Règles : `CLAUDE.md`, `docs/agents-orchestration.md`. Issues : #577, #698 (intégrée), #699 (ouverte) —
`gh api repos/guycorbaz/kesh/issues/<n>` en lecture. Rapports de la passe 1 : `/home/gcorbaz/devel/kesh-gate-logs/
312-validate-p1-{R,F}.md` ; leurs corrections sont dans le Change Log de la fiche (« Validation P1 »).

**La fiche a été remédiée après la P1.** Lis le Change Log, puis **cherche d'abord ce que la remédiation a introduit**
(la sévérité se déplace vers ce qu'on vient d'écrire) : l'AC18 et la tâche T4 (`release.yml`), les sources injectables du
garde (`--grants-from-stdin`, `--mount-source`, `cmp`), le placement du garde avant `cargo clippy`, `docker/login-action`
sur `docker-build` et `e2e-smoke` (C-31-2-11), la prémisse datée et l'état du conteneur (recréé le 2026-10-23 vers 18:13Z),
les inventaires de textes « clos par la forme », l'interdit de `GRANT` manuel et la règle « depuis le dépôt principal »
(AC5), les recomptes (18 AC, 8 tâches). Un patch dont le symptôme n'a pas été grepé sur tout le dépôt n'est pas terminé :
greper les **valeurs** (« 17 AC », `C-31-2-9`, « trois jobs », « hors périmètre », « demander »), non les formulations.

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de l'issue #577,
  de l'epic § 31-2 et de C-31-2-10/11/12 couverte par un AC testable, chaque AC par une tâche ; implémentable sans
  deviner ; **faits cités revérifiés** (`grep -nF`, `sed -n`) dans `ci.yml`, `release.yml`, `docs/ci.md`, `CLAUDE.md`,
  `docs/testing.md`, `docs/agents-orchestration.md`, `scripts/test-fast.sh`, `docker-compose*.yml`,
  `scripts/mariadb-init/01-dev-grants.sql` ; cohérence interne AC ↔ conception ↔ mutations (M1–M6) ↔ tâches ↔ tableau
  « local / CI uniquement » ; plus aucune phrase « hors périmètre » ou « à ouvrir » pour #698/#699 ; recompte des AC (18)
  et des tâches (8) depuis la source ; rattachement de chaque AC à une tâche.
- **F — Full-scope adversary** : la conception remédiée. (1) le **garde** : acquisition `-N -B -r` et normalisation
  minimale — un faux vert ou faux rouge reste-t-il possible (`_`, guillemets, ordre, compte `kesh@%` vs `kesh@localhost`,
  `kesh%` couvrant `kesh`, rôle `PUBLIC`) ? le montage vérifié par `cmp` avec le worktree : cas d'une branche qui
  modifie le fichier ; `docker` absent ; mot de passe et ligne `USAGE` jamais affichés ; `MYSQL_PWD` ; (2) le placement
  dans `test-fast.sh` (sous `--no-lint`, avant `cargo clippy`) et la mutation M3 ; (3) **CI en `kesh`** : étape de droits
  (ordre des étapes, client `mariadb`, `-h`), `DATABASE_URL` de niveau job, tests qui exigeraient `root` ou `PROCESS` ;
  (4) **`release.yml`** : `DATABASE_URL` en `kesh` suffit-il (droits de l'utilisateur créé par le service, création des
  tables par les migrations de l'image) ? risque de bloquer la release v0.15.0 sans second filet ; (5) le job
  `e2e-smoke` et `docker/login-action` (forks, secrets absents, `needs`, cache `gha`) ; (6) **`.svelte-kit/`** : inventaire
  clos ; (7) le **manuel** : `docs/manual/` (`.tex` et PDF aplati) mentionne-t-il le smoke, la CI, les droits MariaDB de
  dev, Docker Hub anonyme ? ; (8) **textes à réécrire** : inventaire par la forme (`git grep -nE "job .?e2e|Smoke test /api|
  \b(3|4|trois|quatre)\b.*jobs?|\be2e\b|droits globaux|mysql://root|GRANT ALL.*\*\.\*"`), sites oubliés ; (9) le
  découpage (règle de splitting préventif : cinq modules ?) et le périmètre minimal après l'intégration de #698.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code
relu), correction proposée. Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas, et un point qualifié de
« robuste » doit avoir été exécuté (lecture) et non supposé.
⛔ **Écris ton rapport complet** dans `/home/gcorbaz/devel/kesh-gate-logs/312-validate-p2-<lentille>.md` (R ou F) — c'est
le seul fichier autorisé — et rends comme dernier message **seulement** : ce chemin, le bilan par sévérité, une ligne
par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. Aucun `git checkout`/`switch`/`stash`/`reset`/`add`/`commit`, sous aucune forme. Aucune commande
qui écrit, compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh` et `scripts/check-dev-grants.sh`),
`make`, `latexmk`, `cargo` (toutes sous-commandes), `sqlx`, `npm`, `npx`, `node`, `python3`, `docker` (sauf `docker
inspect` et `docker exec … mariadb -e "SHOW GRANTS…"`/`SELECT` en lecture), `gh issue create|comment|edit|close`,
toute requête SQL d'écriture, tout `docker restart|stop|compose`, tout `GRANT`/`REVOKE`. Aucun fichier temporaire :
comparer par pipe. Autorisés : `grep`, `sed -n`, `cat`, `ls`, `git log|show|diff|ls-files`, `gh api` en lecture,
`pdftotext … -` vers la sortie standard.
