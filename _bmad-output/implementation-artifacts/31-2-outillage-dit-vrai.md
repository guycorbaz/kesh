# Story 31-2 — L'outillage dit vrai : droits de dev, smoke E2E en CI, Docker authentifié, `.svelte-kit/`

Status: ready-for-dev

Issue : [#577](https://github.com/guycorbaz/kesh/issues/577) (bug, P3, documentation) ; actions C3 (rétrospective 29),
A2, A3 (rétrospective 15), A-C7 et A-C9 (rétrospective 15). Branche `story/31-2`. Choix : `epic-31-choix-autonomes.md`
(C-31-0-2, C-31-0-3) et `epic-31-choix-autonomes-312.md` (C-31-2-1 à C-31-2-9). Aucune migration, aucun code de
production, aucun texte visible de l'application : ni i18n, ni manuel, ni capture.

Quatre volets indépendants (C-31-0-3), revus en une boucle. Le dév les livre **dans cet ordre** : 4, 3, 1, 2 — du plus
mécanique au plus coûteux en gate. Un commit par volet.

## Ce que cette story établit d'abord (prémisses vérifiées le 2026-10-23, lecture seule)

| Prémisse | Constat | Verdict |
|---|---|---|
| « Le conteneur de dev a les droits du fichier versionné » (C3, rétro 29 § 4.2 : il aurait `GRANT ALL *.*`) | `SHOW GRANTS FOR 'kesh'@'%'` rend `GRANT USAGE ON *.*` et `GRANT ALL PRIVILEGES ON \`kesh\`.*` — **moins** que le fichier, pas plus | **Fausse dans les deux sens.** |
| « Le fichier `scripts/mariadb-init/01-dev-grants.sql` est monté dans le conteneur » (`docs/testing.md` § *Base de dev jetable*, point 1) | `docker inspect kesh-mariadb-dev` : le montage a pour source `/home/gcorbaz/devel/kesh-chore-tmpfs/scripts/mariadb-init`, un répertoire **vide** (créé par Docker, propriétaire `root`, 2026-10-11) : le worktree `kesh-chore-tmpfs` n'existe plus, ce n'est même plus un dépôt git | **Cause établie.** |
| « Le conteneur est celui du compose de dev du dépôt principal » | `com.docker.compose.project.working_dir` = `/home/gcorbaz/devel/kesh-chore-tmpfs` ; démarré le 2026-10-23 07:55 | Le conteneur a été (re)créé depuis un worktree, par `docker compose -f docker-compose.dev.yml up` lancé *dans* ce worktree. |
| « Les suites emploient l'utilisateur `kesh` » | `scripts/test-fast.sh:33` : `DATABASE_URL=mysql://kesh:kesh_dev@…/kesh` par défaut ; `docs/testing.md` § Prérequis idem ; l'environnement de la session porte la même valeur | Vrai pour le dev. **La CI emploie `root`** (`ci.yml:64`, `root:kesh_root`) ; `docs/testing.md:102` et `:308` aussi, pour deux recettes ponctuelles. |
| « La CI a les droits de la production » | La CI est `root` sur son service MariaDB ; la production a `kesh`.* seulement (`MARIADB_USER`, `docker-compose.yml:14`) | **Fausse : la CI a bien plus que la production.** Un test qui exige un privilège passe en CI *et* en dev si le conteneur est trop large — c'est le mécanisme des Stories 29-4 et 29-8, côté CI. |
| « L'écart est corrigé quand le conteneur est conforme au fichier » | Le conteneur actuel, tel quel, **ne peut plus créer une base éphémère** (`CREATE DATABASE` hors `kesh`.* refusé) : tout `#[sqlx::test]` rougit d'emblée. Non exécuté ici (écriture) ; déduit des droits relevés | À confirmer par le dév au premier gate — **signalé à l'orchestrateur : tout gate lancé maintenant sur la station échoue pour cette raison, et non pour la branche.** |

**Prémisse de l'issue #577 vérifiée** : `ci.yml` n'a que trois jobs (`backend`, `frontend`, `docker-build`), aucun ne lance
Playwright ; `CLAUDE.md` (§ *E2E*, lignes ~220–226) et `docs/ci.md` (lignes 12, 23–32, 133, 166, 188–192, 212–216)
décrivent un job `e2e` qui n'existe pas. L'issue reste vraie telle qu'écrite.

**Prémisse de l'epic : « les six fichiers `.svelte-kit/` versionnés à la racine »** — vérifiée : `git ls-files | grep
svelte-kit` en rend six (`.svelte-kit/ambient.d.ts`, `generated/client/{app,matchers,nodes/0,nodes/1}.js`,
`tsconfig.json`), tous ajoutés par les commits `046efa51` (23-3) et `08e20353` (24-4b) ; `.gitignore:8` n'ignore que
`frontend/.svelte-kit/`. Aucun outil ne les lit : `grep -rn svelte-kit` (hors `node_modules`, `target`, documents de
planning) ne rend que `frontend/package.json` (`svelte-kit sync`, qui écrit dans `frontend/.svelte-kit/`),
`frontend/tsconfig.json` (`extends ./.svelte-kit/tsconfig.json`, **relatif à `frontend/`**), `.gitignore` et
`.dockerignore`. Les fichiers de la racine sont des résidus d'un `svelte-kit sync` lancé depuis la racine.

## Volet 1 — Les droits du conteneur de dev (C3)

### Conception

**1a. La cause se traite à sa source : un montage qui dépend d'un worktree jetable.** Le compose de dev monte
`./scripts/mariadb-init` ; relatif au répertoire où l'on lance `docker compose`. Lancé depuis un worktree, le conteneur
hérite d'un chemin que la suppression du worktree vide. Ce n'est pas corrigeable dans le compose (un chemin absolu
écrit en dur casserait les autres postes ; une variable de plus ajoute un réglage à oublier). **Retenu (C-31-2-1)** : la
règle « le conteneur de dev se crée **depuis le dépôt principal** seulement » est écrite (`docs/testing.md` § *Base de
dev jetable*, `docs/agents-orchestration.md` § 4), **et** un garde la rend visible (1b) — parce que, comme tout ce dépôt
le montre, une règle sans garde est une règle contournée.

**1b. Un garde qui lit les droits réels : `scripts/check-dev-grants.sh`.** Lancé par `scripts/test-fast.sh` avant
`nextest` (et utilisable seul). Il sépare deux gestes :

- l'**acquisition** : `SHOW GRANTS FOR CURRENT_USER` de l'utilisateur de `DATABASE_URL`, par `docker exec
  kesh-mariadb-dev mariadb …` (aucun client MariaDB requis sur l'hôte) ; et, du même conteneur, la **source du montage**
  `/docker-entrypoint-initdb.d` par `docker inspect` — répertoire absent, vide, ou sans `01-dev-grants.sql` ;
- la **comparaison** (fonction pure, lisible sur l'entrée standard avec `--from-stdin`, pour être testée sans
  conteneur) : l'ensemble attendu est **dérivé du fichier versionné** (ses lignes `GRANT … TO 'kesh'@'%'`) **plus** le
  droit de `MARIADB_USER`, `GRANT ALL PRIVILEGES ON \`kesh\`.*`, plus `USAGE` ; tout droit **en moins** (ce que montre
  le conteneur d'aujourd'hui) **et tout droit en plus** (un `*.*` ou `PROCESS` posé à la main, ce que la rétrospective 29
  décrivait) échoue. L'égalité d'ensembles est le critère : c'est ce qui fait que « retirer les droits que la
  production n'a pas » (décision de Guy, 2026-10-23) est contrôlé, non constaté une fois.
- sortie en cas d'échec : le droit manquant ou superflu nommé, la cause probable (montage vide : chemin lu et
  répertoire), et **la remise en état** — recréer le conteneur depuis le dépôt principal. Cette remise en état efface
  les bases de **tous** les agents (tmpfs) : le script **n'y procède jamais**, il la nomme (`docs/agents-orchestration.md`
  § 2).
- hors conteneur de dev (`docker` ou le conteneur absent, `DATABASE_URL` vers un autre hôte) : le garde **dit** qu'il ne
  contrôle rien et rend 0 — un MariaDB non conteneurisé reste possible, mais le silence est exclu (« un détecteur qui se
  tait »).

**1c. Aligné sur la production — et sur la CI.** Table à écrire dans `docs/testing.md` :

| | Droits de `kesh` |
|---|---|
| Production (`docker-compose.yml`) | `ALL` sur `kesh`.* (celui de `MARIADB_DATABASE`) |
| Dev (`01-dev-grants.sql`) | idem **+** `ALL` sur `\_sqlx\_test%`.* et `kesh%`.* — nécessaires à `#[sqlx::test]` et à `kesh_e2e` ; **aucun droit global** |
| CI, aujourd'hui | `root` — **plus large que la production** |
| CI, après la story | `kesh` avec le **même fichier** appliqué en `root` au démarrage du job |

Dans `ci.yml`, job `backend` : une étape après « Wait for MariaDB » exécute `mariadb -uroot -pkesh_root <
scripts/mariadb-init/01-dev-grants.sql`, et `DATABASE_URL` devient `mysql://kesh:kesh_dev@127.0.0.1:3306/kesh`.
Les étapes « Apply migrations » et « Seed » (qui emploient `root` sur le client `mariadb`) restent en `root` : c'est
de l'outillage, pas le chemin de l'application. **Retenu (C-31-2-4)** parce qu'une CI en `root` fait exactement ce que
fait un conteneur de dev trop large : elle ne peut rien dire d'un privilège que la production n'a pas. ⚠️ **Ne se
vérifie qu'en CI** : un test qui dépendrait de `root` n'a jamais pu rougir en dev, où tout tourne en `kesh`. Si la CI
rougit pour cette raison, le dév **n'annule pas silencieusement** : chaque test concerné est nommé dans la fiche et
une issue est demandée (« on trace toujours »).

**1d. Pas de CI : `release.yml`.** Son smoke Docker emploie `root` (`DATABASE_URL=mysql://root:kesh_root@…`) pour lancer
l'image publiée : le backend crée ses tables avec `root`, ce que la production ne fait pas (`kesh`). Le passer en
`kesh` serait cohérent mais touche le workflow de **publication** : **hors périmètre, demandé en issue** (voir
« Issues à ouvrir »), pas modifié ici.

### Critères d'acceptation

- **AC1** — `scripts/check-dev-grants.sh` existe, exécutable ; `--from-stdin` lit un `SHOW GRANTS` et rend 0 si
  l'ensemble égale celui attendu (dérivé du fichier + `MARIADB_USER`), non nul sinon, en nommant chaque écart.
- **AC2** — Le garde échoue, message à l'appui, dans chacun des cas : droit manquant (`kesh`.* seul — l'état du
  2026-10-23), droit global en trop (`ALL … ON *.*`, `PROCESS`), montage vide ou absent, `01-dev-grants.sql` introuvable
  dans la source du montage. Il rend 0 et le dit quand il ne peut rien contrôler.
- **AC3** — `scripts/tests/check-dev-grants.test.sh` (même forme que `scripts/tests/prepare-release.test.sh`) exerce
  la comparaison sur cinq entrées figées : conforme, `kesh`.* seul, `*.*` en trop, `PROCESS` en trop, un droit du
  fichier retiré. **Éprouvé par mutation** : une comparaison qui ne vérifie que « les attendus sont présents » (et non
  l'égalité) fait rougir le cas « en trop ».
- **AC4** — `scripts/test-fast.sh` appelle le garde avant `nextest`, et s'arrête (code non nul) s'il échoue, **avant**
  de compiler quoi que ce soit de lourd.
- **AC5** — `docs/testing.md` § *Base de dev jetable* (point 1) dit la vérité : le montage dépend du répertoire d'où
  le compose est lancé ; la règle « depuis le dépôt principal » ; le garde et son message ; la table de 1c.
  `docs/agents-orchestration.md` § 6.1 renvoie au garde (la phrase « écart corrigé par l'Epic 31 » devient vraie
  ou est ajustée à l'état réel). `crates/kesh-db/README.md` § Prérequis 2 renvoie au même endroit.
- **AC6 (CI uniquement)** — Le job `backend` applique `01-dev-grants.sql` en `root` puis emploie `kesh` ; il passe vert
  sur la PR.
- **AC7 (local)** — Après remise en état du conteneur **par l'orchestrateur** (voir ci-dessous), `SHOW GRANTS` est
  conforme, `scripts/check-dev-grants.sh` rend 0, et **un gate complet** (`scripts/test-fast.sh`) est lancé : tout rouge
  est trié (cause : privilège que la production n'a pas, ou autre) et consigné ; un test qui exigeait un privilège
  absent de la production est **corrigé** s'il emploie `INNODB_TRX`/`INNODB_LOCKS` (sondes de § 6.1), sinon
  une issue est demandée.

### Ce que le dév ne fait pas

Aucun `docker restart|stop|compose down|up --force-recreate` : la remise en état du conteneur **efface la base de
tous les agents**. Le dév la **demande** à l'orchestrateur, qui choisit un moment sans gate en cours : depuis
`/home/gcorbaz/devel/kesh` (dépôt principal), `docker compose -f docker-compose.dev.yml up -d --force-recreate mariadb`,
puis migrations et seed (`docs/testing.md`), puis `scripts/check-dev-grants.sh`. Le garde et ses tests se développent
et s'éprouvent sans conteneur modifié (`--from-stdin` et l'état actuel du conteneur, qui est justement l'état défaillant).

## Volet 2 — Le smoke E2E en CI (#577, C-31-0-2)

### Conception

**Un job `e2e-smoke` dans `ci.yml`, une spec, un navigateur.** Il vérifie que **le harnais est vivant** (login par
l'interface, un appel API authentifié par `authedApiContext`) — précisément le mode d'échec de l'issue #285 : un
contexte API sans cookie rend 401 partout, et rien ne signale une suite qui cesse de tourner.

**La spec : `frontend/tests/e2e/smoke.spec.ts` (nouvelle).** Aucune des specs existantes ne convient telle quelle :
`auth.spec.ts` (login) n'appelle aucune API authentifiée, et ses autres tests ajoutent des dépendances (axe-core,
déconnexion). La spec : `beforeAll(seedTestState('with-company'))` ; un test qui se connecte par l'interface
(`/login`, `#username`/`#password`, `admin`/`admin123`, redirection `/`), puis `authedApiContext(page)` et
`GET /api/v1/auth/me` → `200` avec le nom d'utilisateur `admin` (route protégée : `lib.rs:916`). Un second test, hors
connexion, confirme que `GET /api/v1/auth/me` **sans** cookie rend `401` — ce qui prouve que le `200` du premier est
dû au cookie et non à une route ouverte (un smoke qui ne peut pas rougir ne prouve rien). Elle réutilise
`seedTestState`, `authedApiContext`, `disposeContextSafe`, `clearAuthStorage` (aucune réécriture).

**Le montage CI : l'image Docker de la PR, pas un `cargo run`.** Retenu (C-31-2-2) : le job dépend de `docker-build`,
reconstruit l'image avec `cache-from: type=gha` (les couches de `docker-build` sont déjà là : quelques secondes) et
`load: true`, la lance en `--network=host` avec `KESH_TEST_MODE=true`, `KESH_HOST=127.0.0.1` (le mode test refuse un
bind non-loopback, `ConfigError::TestModeWithPublicBind`), `KESH_COOKIE_SECURE=false` (HTTP, cf. `CLAUDE.md` § E2E),
`KESH_PORT=3000`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` (le **service MariaDB** du job, utilisateur
`kesh` : l'application ne reçoit pas plus que la production), `KESH_JWT_SECRET` et `KESH_ADMIN_*` factices valides.
Avantage : on teste ce qui sera publié (binaire de production, frontend buildé tel que servi), sans second
`cargo build`. Les migrations s'appliquent au démarrage du backend (comme en production) ; **le seed est celui du
backend** (`POST /api/v1/_test/seed`, par `seedTestState`), pas un SQL : aucune étape « Seed » dans ce job.
Ni `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR` ni `KESH_SMTP_*` : ils ne servent qu'à des specs que le smoke ne lance pas.
Puis `actions/setup-node@v4` (Node 22, cache npm), `npm ci`, `npx playwright install --with-deps chromium`, et
`KESH_TEST_MODE=true KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test tests/e2e/smoke.spec.ts`.
Attente du backend par `/health` en boucle avec délai (consigne § 4.4), avec le journal du conteneur affiché si elle
échoue. `timeout-minutes: 25`. Le rapport/trace Playwright est téléversé **en cas d'échec** (`actions/upload-artifact`,
rétention 3 jours).

**Écartée** : un job qui compile `kesh-api` (`cargo build`) + télécharge l'artefact `frontend-build` — un second
`cargo build` complet (plus de dix minutes à froid), pour tester un binaire de débogage et un dossier de build qui ne
sont pas ce qu'on publie. Repli si le cache `gha` ne tient pas : revenir à ce montage (réversible).

⚠️ **Le smoke ne teste aucune fonctionnalité, et c'est délibéré** (`CLAUDE.md` § E2E, repris tel quel). Il reste
**facultatif pour la fusion** tant qu'il n'est pas apparu stable : la règle de branche protégée n'est pas modifiée par
cette story — décision de Guy à demander à l'orchestrateur (voir « Issues à ouvrir »).

### Critères d'acceptation

- **AC8 (local)** — `smoke.spec.ts` passe contre la recette E2E de `docs/testing.md` (backend du worktree, base
  `kesh_e2e_312`, port attribué) : `2 passed`.
- **AC9 (mutation locale, exigée)** — Backend lancé **sans** `KESH_COOKIE_SECURE=false` : la spec rougit en `401` sur
  l'appel authentifié (c'est l'échec #285/`docs/testing.md`) — preuve que le smoke détecte un harnais mort. Backend
  sans `KESH_TEST_MODE` : rouge dès le `beforeAll`. Les deux mutations sont décrites au Dev Agent Record **avec leur
  sortie**.
- **AC10 (CI uniquement)** — Le job `e2e-smoke` figure dans `ci.yml`, passe vert sur la PR, et sa durée est relevée
  (objectif : quelques minutes ; si > 10 min, le noter et envisager le cache des navigateurs, `docs/ci.md` D-6-1-A).
- **AC11** — `smoke.spec.ts` ne dépend d'aucun autre `*.spec.ts` ni d'aucune variable absente de la recette de
  `docs/testing.md` ; `npx playwright test --list tests/e2e/smoke.spec.ts` rend exactement 2 tests.
- **AC12 (texte)** — `CLAUDE.md` § *E2E* : le paragraphe « ⛔ Constat du 2026-10-10 : ce smoke N'EXISTE PAS » est
  **remplacé** par une description de ce que fait le job (nom, spec, durée mesurée), et les phrases qui supposent le
  smoke (« **La CI n'exécute PAS la suite complète — mais elle exécute un SMOKE** ») restent exactes ; la mention
  « issue #577 » devient l'historique. `docs/ci.md` est **réécrit pour dire le dépôt** : quatre jobs
  (`backend`, `frontend`, `docker-build`, `e2e-smoke`) — schéma d'exécution, tableau des jobs, timeouts, sections
  « migrations » et « mode test » (lignes 12, 23–32, 133, 166, 188–192), le bandeau ⚠️ de la ligne 12 retiré, les lignes
  D-6-1-* qui décrivent un job complet (212–216) annotées « non retenu ». Inventaire clos, recompté par
  `grep -nE "\be2e\b|4 jobs|trois|quatre" docs/ci.md` : chaque occurrence soit réécrite, soit assumée par écrit.

## Volet 3 — Docker Hub authentifié (A2)

### Conception

`services.mariadb.credentials` (`username: ${{ secrets.DOCKERHUB_USERNAME }}`, `password: ${{ secrets.DOCKERHUB_TOKEN }}`)
dans les **trois** services MariaDB : `ci.yml` job `backend`, `ci.yml` job `e2e-smoke` (nouveau), `release.yml` job
`docker`. Les secrets existent déjà pour `release.yml` (étape `docker/login-action`) ; elles n'y couvraient pas le
service, tiré **avant** toute étape.

**Les PR issues de forks et de Dependabot n'ont pas accès aux secrets.** Tranché (C-31-2-3) : **pas de contournement
conditionnel.** Le dépôt est tenu par un seul développeur, sans fork actif et sans `dependabot.yml`
(`ls .github` : aucun) ; une expression conditionnelle sur `credentials` (un objet nul ou `fromJSON(format(…))`) est
invérifiable hors d'un runner et ajoute une fragilité pour un cas qui n'existe pas. Conséquence **assumée et écrite**
dans `docs/ci.md` : une PR de fork échouera au démarrage du service (échec bruyant, jamais un faux vert). Si le dépôt
accepte un jour des PR externes, le repli est `image: mirror.gcr.io/library/mariadb:10.11` (sans secret) — noté, non
adopté (la provenance de l'image n'est plus celle du Docker Hub officiel).

### Critères d'acceptation

- **AC13 (CI uniquement)** — Les trois services portent `credentials` ; `grep -c 'credentials:' .github/workflows/ci.yml`
  rend 2 et `release.yml` 1 ; les runs de la PR passent (le tirage est authentifié : l'en-tête du service dans le journal
  du job le montre).
- **AC14** — Le paragraphe « ⚠️ **La CI dépend de Docker Hub.** » (`CLAUDE.md`, § Test Locally First, fin) est
  retiré, avec la mention A-C9 ; `docs/ci.md` dit à sa place les secrets requis et le comportement des forks. Aucun
  autre site : `grep -rn "429\|anonymement" CLAUDE.md docs .github` — inventaire recompté et traité.
- **AC15** — `actionlint` (ou à défaut `python -c "import yaml; yaml.safe_load(...)"` sur les deux fichiers)
  confirme que les workflows restent syntaxiquement valides **localement** ; la validité d'exécution ne se voit qu'en
  CI (AC13).

## Volet 4 — `.svelte-kit/` à la racine (A3)

### Conception et critères

- **AC16** — `git rm -r --cached .svelte-kit` puis suppression du répertoire (`rm -r`, sans danger : régénérable et
  ignoré ; **aucune** de ses six pièces n'est lue, cf. prémisses) ; `.gitignore` gagne `/.svelte-kit/` (ancré :
  `frontend/.svelte-kit/` y est déjà) ; `.dockerignore` gagne la même ligne (aujourd'hui `frontend/.svelte-kit/`
  seulement) — mécanique, même symptôme. `git ls-files | grep -c svelte-kit` rend **0**.
- **AC17** — `cd frontend && npm run check && npm run build` verts après le retrait (preuve qu'aucun outil ne lisait les
  fichiers de la racine) ; `git check-ignore -v .svelte-kit/tsconfig.json` rend la règle `/.svelte-kit/`.

## Mutations à prévoir

| # | Mutation | Attendu |
|---|---|---|
| M1 | Comparaison du garde réduite à « attendus ⊆ réels » | AC3 : le cas `*.*` en trop rougit |
| M2 | Garde sans lecture de la source du montage | AC2/AC3 : le cas « montage vide » rougit |
| M3 | Appel du garde retiré de `test-fast.sh` | AC4 : `scripts/tests/check-dev-grants.test.sh` porte une assertion de câblage (`grep -nF 'check-dev-grants.sh' scripts/test-fast.sh` non vide, et placée avant la ligne `cargo nextest run`) ; elle rougit |
| M4 | Backend du smoke sans `KESH_COOKIE_SECURE=false` | AC9 : 401 |
| M5 | `smoke.spec.ts` : le test « sans cookie » attend 200 | Rouge — prouve que la sonde négative sonde |

Restauration de chaque mutation : `git checkout -- <fichier>` **puis** `touch <fichier>` (`docs/agents-orchestration.md`
§ 7).

## Ce qui se vérifie en local et ce qui ne se vérifie qu'en CI

| Local | CI seulement |
|---|---|
| Garde et son test (AC1–AC4), mutations M1–M3 | AC6 (job `backend` en `kesh`), AC10 (job `e2e-smoke`), AC13 (tirage authentifié) |
| `smoke.spec.ts` contre la recette locale (AC8, AC9, AC11) | La durée réelle du smoke ; le cache `gha` du `docker-build` ; la validité des secrets |
| `.svelte-kit` (AC16, AC17), YAML syntaxique (AC15), textes (AC5, AC12, AC14) | Tout test qui dépendrait encore de `root` en CI |

Gate : `scripts/test-fast.sh` complet (kesh-db non touché, mais le volet 1 change le chemin d'exécution du gate ;
sous le verrou commun), `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run
build`, **E2E complet au dernier commit de code** (le smoke y est ajouté : la baseline devient 7 à 10 échecs attendus
sur N+2 tests, à recompter), contrôle `scripts/tests/check-dev-grants.test.sh` et `scripts/tests/prepare-release.test.sh`.

## Issues à ouvrir ou à commenter (demandées à l'orchestrateur)

1. **Commenter #577** : prémisse confirmée, plus `docs/ci.md` entièrement concerné (cinq sites), pas seulement le
   `CLAUDE.md` ; la 31-2 la ferme (**mot-clé de fermeture sur la PR seulement**).
2. **Ouvrir (P3, `bug`)** — `release.yml` : le smoke de l'image publiée emploie `root` pour `DATABASE_URL` ; la
   production emploie `kesh` (même défaut de classe que le volet 1c, côté publication). Hors périmètre ici.
3. **Ouvrir (P3)** — le conteneur de dev peut être créé depuis un worktree : `docker-compose.dev.yml` monte un chemin
   relatif. Le garde le *signale*, il ne l'*empêche* pas ; une vraie prévention (script `scripts/dev-db.sh` qui refuse de
   démarrer hors du dépôt principal, ou montage par un volume nommé alimenté une fois) est une story, pas un correctif.
4. **Décision de Guy à demander** — le job `e2e-smoke` doit-il être une **vérification obligatoire** de la branche
   protégée ? Cette story ne modifie aucun réglage de dépôt.
5. **Rien à ouvrir** pour C3 : le constat (conteneur *moins* doté que le fichier) et la cause (montage vide) sont dans
   cette fiche ; la rétrospective 29 § 4.2 décrivait un état ancien du conteneur (`GRANT ALL *.*` d'avant la recréation) —
   à noter à la rétrospective, non à rouvrir.

## Tâches

- [ ] **T1 (volet 4)** `.svelte-kit/` retiré, `/.svelte-kit/` aux deux ignore ; AC16, AC17.
- [ ] **T2 (volet 3)** `credentials` sur les trois services ; note A-C9 retirée ; AC13–AC15.
- [ ] **T3 (volet 1)** `scripts/check-dev-grants.sh` + `scripts/tests/check-dev-grants.test.sh` (rouge d'abord :
  M1–M3) ; câblage dans `test-fast.sh` ; textes `docs/testing.md`, `docs/agents-orchestration.md`, `crates/kesh-db/README.md` ;
  AC1–AC5.
- [ ] **T4 (volet 1c)** `ci.yml` job `backend` : droits appliqués en `root`, `DATABASE_URL` en `kesh` ; AC6.
- [ ] **T5 (volet 2)** `smoke.spec.ts` (rouge d'abord : M5), job `e2e-smoke`, mutations M4 ; AC8–AC11.
- [ ] **T6 (volet 2, textes)** `CLAUDE.md` § E2E et `docs/ci.md` réécrits ; AC12.
- [ ] **T7** Demander à l'orchestrateur la remise en état du conteneur ; gate complet ; trier les rouges (AC7).
- [ ] **T8** E2E complet au dernier commit de code ; Dev Agent Record **sans affirmer ce qui n'a pas tourné** (les
  AC « CI uniquement » y figurent comme *non vérifiés localement*).

## Dev Notes

- **Périmètre minimal.** Aucun code de production ; `release.yml` n'est modifié que pour `credentials` (pas pour `root`).
- **Le symétrique existe peut-être déjà** (`docs/agents-orchestration.md` § 3) : `scripts/tests/prepare-release.test.sh`
  est le modèle de forme des tests de scripts ; `seedTestState`/`authedApiContext` sont réutilisés, non réécrits ;
  la fonction qui lit `KESH_BACKEND_URL` est `resolveBackendUrl` (`test-state.ts`).
- **Un test ne dépend d'aucun privilège que la production n'a pas** (§ 6.1) : c'est le critère que le volet 1 rend
  contrôlé.
- **Données réelles** : aucune ; mots de passe de dev/CI factices déjà versionnés (`kesh_dev`, `kesh_root`).
- **Le garde lit un mot de passe de dev** (`DATABASE_URL`) : ne l'affiche jamais dans ses messages.

## Change Log

- 2026-10-23 — Fiche écrite (agent de fiche 31-2, Sonnet 5.5). Prémisses C3 et #577 établies au code et à la lecture
  du conteneur (lecture seule). Validation P1 à lancer (`312-validate-prompt-p1.md`, Opus, lentilles R et F).
