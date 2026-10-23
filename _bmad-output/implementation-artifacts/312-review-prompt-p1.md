# Prompt — revue de code P1 de la story 31-2 (trois lentilles B, E, A ; modèle Sonnet)

Tu relis l'implémentation de la story **31-2 « L'outillage dit vrai »** de Kesh (comptabilité suisse) : garde des
droits du MariaDB de dev, smoke E2E en CI (#577), tirages Docker Hub authentifiés, `.svelte-kit/` de la racine retirés,
smoke de release en utilisateur `kesh` (#698). Worktree `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2`.
Langue du rapport : français. **Une lentille par agent**, contexte frais ; ta lentille t'est donnée par l'orchestrateur.

**Le diff à relire** — le développement, hors artefacts de planification :

```sh
git -C /home/gcorbaz/devel/kesh-31-2 diff b34314ae HEAD -- . ':(exclude)_bmad-output'
```

(`b34314ae` est le commit qui clôt la validation de la fiche ; les commits suivants sont ceux du développement.)
La spécification : `_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md` (AC1 à AC18, tableau des mutations
M1–M20, « Ce qui se vérifie en local et ce qui ne se vérifie qu'en CI ») ; les choix : `epic-31-choix-autonomes-312.md`
(C-31-2-1 à C-31-2-22) ; le Dev Agent Record de la fiche dit ce qui a tourné.

## Interdits absolus

- **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier — pas même temporaire —, hormis ton rapport.
  Compare par des commandes enchaînées (pipe) ; lis les PDF vers la sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`).
- **N'exécute rien qui écrit, compile ou lance du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/check-dev-grants.sh`, ni `scripts/tests/*.test.sh`, ni `scripts/mem-guard.sh`, ni `cargo`, ni `npm`,
  ni `npx`, ni `make`, ni `latexmk`, ni **`python3`** (même « en lecture »), ni **`sed -i`**, ni `perl`, ni `bash -c` d'un
  script du dépôt. **Aucune commande `docker`** (le conteneur `kesh-mariadb-dev` est partagé par plusieurs agents).
  Aucune commande `gh` qui écrit ; `gh issue view` et `gh api … --jq` en lecture sont permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (lecture), `cat`, `wc`, `sort`,
  `comm`, `diff` entre deux sorties de commande, `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention. Cherche ce qui est faux, fragile ou trompeur :

1. `scripts/check-dev-grants.sh` : `set -euo pipefail` et substitutions de commande (`x=$(…) || rc=$?`, assignations,
   fonctions qui font `exit` dans un sous-shell) — un échec peut-il être avalé ou rendre 0 à tort ? L'awk d'extraction
   est-il **POSIX** (la CI a mawk) ? Commentaires `--`, `#`, `/* */`, instructions sur plusieurs lignes, `;` dans une
   chaîne ou un nom entre backticks ? `sort`/`comm` sous `LC_ALL=C` ? Le mot de passe ou le condensé peuvent-ils sortir
   par un autre chemin (stderr de `docker`, message d'extraction, `set -x` hérité) ?
2. `scripts/tests/check-dev-grants.test.sh` : chaque assertion peut-elle rougir ? Un cas peut-il passer à vide (factice
   non appelé, mauvais `$R`, état du factice hérité du cas précédent) ? L'assertion de câblage lit-elle le vrai
   `scripts/test-fast.sh` et sa borne est-elle juste ?
3. `crates/kesh-db/tests/check_dev_grants_script.rs` : garde anti-test-muet, variables d'environnement retirées.
4. `.github/workflows/ci.yml` et `release.yml` : ordre des étapes, `needs:`, `working-directory`, `if: failure()`,
   variables, guillemets ; le job `e2e-smoke` peut-il passer vert sans avoir joué la spec ? L'étape « Apply dev grants »
   vient-elle après l'installation du client et avant les migrations ?
5. `frontend/tests/e2e/smoke.spec.ts` : la sonde négative sonde-t-elle vraiment (contexte sans cookie, même route) ?
   Un `try/finally` peut-il masquer un échec ?
6. Les textes (`CLAUDE.md`, `docs/ci.md`, `docs/testing.md`, `docs/agents-orchestration.md`, `README.md`,
   `crates/kesh-db/README.md`, `CHANGELOG.md`, commentaires) : affirment-ils ce que le code fait, ni plus ni moins ?

## Lentille E — Edge cases (chaque branche, chaque bord)

Prends `scripts/check-dev-grants.sh` branche par branche et, pour chacune, dis quel cas d'AC3 l'exerce (ou qu'aucun ne
l'exerce) : `docker` introuvable ; `ps` en échec, vide, non vide ; clé de compose absente, vide, interpolée, entre
guillemets, suivie d'un commentaire, présente deux fois ; fichier de droits absent, vide, tout commenté, avec une
instruction `GRANT` en minuscules, sur plusieurs lignes, contenant `GRANT` sans commencer par lui, portant lui-même un
droit `ON *.*` (C-31-2-16) ; `exec` en échec ; sortie de `SHOW GRANTS` vide, avec lignes vides, avec `\r`, avec
`WITH GRANT OPTION`, avec un compte autre que `kesh@%`, avec un condensé sur une ligne qui n'est pas `ON *.*` ;
démarrage à froid. Pour le smoke : backend lent à démarrer, `KESH_BACKEND_URL` absent, `globalSetup` en échec. Pour la
CI : PR de fork (secrets absents), cache `gha` vide, `moby/buildkit` tiré avant la connexion.
Croise avec le tableau des mutations M1–M20 de la fiche et les résultats du Dev Agent Record : une mutation déclarée
« tuée » l'est-elle par l'assertion que la fiche annonce, ou par une autre, plus faible ?

## Lentille A — Acceptance (la fiche contre le livré)

Pour **chaque** AC (AC1 à AC18) : satisfait / partiel / non satisfait / « CI seulement, déclaré comme tel », avec la
preuve (`grep -nF` ou `sed -n` copié). En particulier :

- AC4 : rejoue à la lecture la commande d'assertion de câblage sur `scripts/test-fast.sh` (sans l'exécuter : `grep -nE`
  est permis) et la borne `command -v cargo-nextest` / `if [ "$RUN_LINT" = 1 ]`.
- AC5 et AC12 : **ré-exécute les inventaires clos** que la fiche prescrit (`git grep -nE "GRANT ALL.*\*\.\*|droits
  globaux|mysql://root" -- . ':(exclude)_bmad-output' ':(exclude)docs/known-failures.md' ':(exclude)*.pdf'
  ':(exclude)docs/manual'`, `git grep -nE "compose -f docker-compose\.dev\.yml"`, celui d'AC12) et juge chaque sortie.
  Relis `docs/ci.md` **ligne à ligne** contre `.github/workflows/ci.yml` (jobs, timeouts, dépendances, migrations, mode
  test, secrets).
- AC13 : les comptes ancrés `grep -cE '^\s+credentials:\s*$'` et `grep -cE '^\s*(-\s+)?uses: docker/login-action@'`
  sur les deux workflows (attendu : `ci.yml` 2 et 2, `release.yml` 1 et 1), et l'ordre connexion → `setup-buildx` dans
  les trois jobs qui construisent.
- AC14 : `git grep -nI "429\|anonymement" -- 'docs/*.md' CLAUDE.md README.md '.github/*.yml'`.
- AC16/AC17 : `git ls-files | grep -c svelte-kit` (attendu 0) ; `.gitignore` et `.dockerignore`.
- AC18 : `grep -nF 'mysql://root' .github/workflows/release.yml` (attendu vide).
- **Le manuel** : la fiche dit « aucun texte visible de l'application : ni i18n, ni manuel ». Vérifie que c'est vrai
  — aucune page du manuel (`docs/manual/fr/*.tex`) ne décrit les droits MariaDB de dev, la CI ou un `mysql://root` que
  la story aurait dû corriger (`grep -rn "mysql://root\|GRANT\|droits" docs/manual/fr/*.tex`), et contrôle le **PDF**
  aplati du manuel d'administration (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o
  '.\{80\}GRANT.\{80\}'`) : ce qu'il dit des droits de `kesh` en production concorde-t-il avec la table de
  `docs/testing.md` ?
- Le **Dev Agent Record** : chaque chiffre (cas, assertions, tests nextest, Vitest, E2E, mutations) est-il adossé à une
  sortie, et ce qui ne se vérifie qu'en CI est-il déclaré comme tel ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/312-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : `CRITICAL n / HIGH n / MEDIUM n / LOW n` ;
- findings numérotés (`B1…`, `E1…`, `A1…`), chacun avec sévérité, fichier:ligne, **preuve copiée** (`grep -nF`,
  `sed -n`) pour toute affirmation de présence ou d'absence, et correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été**, avec la raison — un « 0 finding » sans cette
  liste ne compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
