# Prompt — validation P1 de la spec, Story 31-2

*Versionné le 2026-10-23. Deux lentilles (R et F), modèle **Opus**, contexte frais chacune, en lecture seule stricte.
Rotation : passes complètes Sonnet ↔ Opus ; la fiche a été écrite par Sonnet, cette première passe est donc Opus.*

Dépôt `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2` (ne change pas de branche). **Fiche** :
`_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md`. Registres : `epic-31-choix-autonomes.md`
(C-31-0-2, C-31-0-3) et `epic-31-choix-autonomes-312.md`. Règles : `CLAUDE.md`, `docs/agents-orchestration.md`.
Première passe : la fiche n'est pas remédiée. Aucun code de la story n'est écrit. Issue #577 : `gh api
repos/guycorbaz/kesh/issues/577` (lecture) — `gh issue view` peut échouer.

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de
  l'issue #577 et de l'epic § 31-2 couverte par un AC testable, chaque AC par une tâche ; implémentable sans deviner ;
  **faits cités revérifiés** (`grep -nF`, `sed -n`) : les lignes de `ci.yml`, `release.yml`, `docs/ci.md`, `CLAUDE.md`,
  `docs/testing.md`, `scripts/test-fast.sh` ; la séparation AC « local » / « CI uniquement » est-elle honnête ?
  Recompte des AC (17) et des tâches (8).
- **F — Full-scope adversary** : la conception. Attaquer en priorité : (1) la **prémisse C3** — le conteneur *moins* doté
  que le fichier, montage vide : relis `docker-compose.dev.yml`, `scripts/mariadb-init/01-dev-grants.sql`, et cherche un
  autre chemin qui l'expliquerait ; (2) le **garde** — égalité d'ensembles, dérivation depuis le fichier (ordre, échappement
  des `_`, `kesh%` couvrant `kesh`), cas où `docker` est absent, mot de passe jamais affiché, faux vert possible ;
  (3) **CI en `kesh`** (C-31-2-4) : quels tests (`grep -rn "root\|CREATE DATABASE\|GRANT\|INNODB_TRX\|FLUSH\|LOCK TABLES"`
  sous `crates/*/tests` et `src`) exigent un privilège que `kesh` du fichier n'a pas ; (4) le **job `e2e-smoke`** : l'image
  supporte-t-elle `KESH_TEST_MODE` (`config.rs`), `needs: docker-build` et le cache `gha`, port, attente `/health`,
  Playwright sur `ubuntu-latest`, fuite de secrets, collision des ports 3306/3000 ; la spec : `/auth/me` est-elle bien
  protégée (`lib.rs`), `admin`/`admin123` est-il bien le seed `with-company` ; (5) **credentials** : forks, Dependabot,
  syntaxe exacte de `services.<id>.credentials` ; (6) **`.svelte-kit/`** : inventaire clos des lecteurs possibles
  (`grep -rn "svelte-kit"` sur tout l'arbre suivi, scripts, workflows, Dockerfile, `.cargo`, `.config`) ;
  (7) le **manuel** : `docs/manual/` mentionne-t-il le smoke, la CI, les droits MariaDB de dev ? (`grep -rn` dans les
  `.tex` ; contrôler le PDF aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers la
  sortie standard seulement) ; (8) les **textes à réécrire** : inventaire des occurrences de « e2e », « smoke »,
  « Docker Hub », « anonymement », « A-C9 » dans `CLAUDE.md`, `docs/`, `website/`, `README.md` ; (9) le découpage
  (règle de splitting préventif) et le périmètre minimal (`release.yml`).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code
relu), correction proposée. Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.
⛔ **Écris ton rapport complet** dans `/home/gcorbaz/devel/kesh-gate-logs/312-validate-p1-<lentille>.md` (R ou F) — c'est
le seul fichier autorisé — et rends comme dernier message **seulement** : ce chemin, le bilan par sévérité, une ligne
par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. Aucun `git checkout`/`switch`/`stash`/`reset`/`add`/`commit`, sous aucune forme. Aucune commande
qui écrit, compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh` et `scripts/check-dev-grants.sh`),
`make`, `latexmk`, `cargo` (toutes sous-commandes), `sqlx`, `npm`, `npx`, `node`, `python3`, `docker` (sauf `docker
inspect` et `docker exec … mariadb -e "SHOW GRANTS…"`/`SELECT` en lecture), `gh issue create|comment|edit|close`,
toute requête SQL d'écriture, tout `docker restart|stop|compose`. Aucun fichier temporaire : comparer par pipe.
Autorisés : `grep`, `sed -n`, `cat`, `ls`, `git log|show|diff|ls-files`, `gh api` en lecture, `pdftotext … -` vers la
sortie standard.
