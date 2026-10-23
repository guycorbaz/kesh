# Story 31-2 — L'outillage dit vrai : droits de dev, smoke E2E en CI, Docker authentifié, `.svelte-kit/`

Status: review

Issue : [#577](https://github.com/guycorbaz/kesh/issues/577) (bug, P3, documentation) ; actions C3 (rétrospective 29),
A2, A3 (rétrospective 15), A-C7 et A-C9 (rétrospective 15). Branche `story/31-2`. Choix : `epic-31-choix-autonomes.md`
(C-31-0-2, C-31-0-3) et `epic-31-choix-autonomes-312.md` (C-31-2-1 à C-31-2-15). Issues liées : #698 (smoke de release
en `root`, **intégrée**, fermée par la PR) et #699 (montage relatif, **reste ouverte**). Aucune migration, aucun code de
production, aucun texte visible de l'application : ni i18n, ni manuel, ni capture.

Quatre volets (C-31-0-3), revus en une boucle, un commit par volet. Le dév les livre **dans cet ordre** : 4, 3, 1, 2 — du
plus mécanique au plus coûteux en gate. **Un seul couplage** : le job `e2e-smoke` n'existe qu'au volet 2 (T5) ; son
service MariaDB (`credentials`), sa connexion Docker Hub (`docker/login-action`) et son `setup-buildx-action` sont donc
écrits **en T5**, non en T2 — AC13 se constate en deux temps (voir AC13).

**Découpage (règle de splitting préventif, déclaré).** Modules au sens de la règle (crates, packages npm, modules métier) :
**trois** — `kesh-db` (l'enveloppe Rust du test du garde, le commentaire de `refresh_tokens_repository.rs`, le README),
`kesh-api` (le seul doc-comment de `reconciliation_e2e.rs`, AC5) et le package `frontend` (une spec, `DEBUGGING-KF007.md`).
Le reste est de l'outillage et du texte (`scripts/`, `.github/workflows/`, `docs/`, `CLAUDE.md`, `README.md`, `.gitignore` de
la racine, `docker-compose.dev.yml` en commentaire) : aucun code de production, quatre volets à un commit chacun, sans cycle de
dépendance. La règle ne joue pas ; le décompte est écrit ici pour qu'on ne le refasse pas.

## Story

En tant que mainteneur de Kesh, je veux que les droits du conteneur de dev, le smoke E2E, l'authentification Docker Hub et
l'arborescence `.svelte-kit/` disent exactement ce que le dépôt fait, afin qu'un texte de `CLAUDE.md`, de `docs/ci.md` ou de
`docs/testing.md` ne décrive plus un job, un droit ou un garde qui n'existent pas.

## Ce que cette story établit d'abord (prémisses vérifiées le 2026-10-23 **au matin**, lecture seule — état du conteneur AVANT sa recréation de 18:13Z, voir la dernière ligne)

| Prémisse | Constat | Verdict |
|---|---|---|
| « Le conteneur de dev a les droits du fichier versionné » (C3, rétro 29 § 4.2 : il aurait `GRANT ALL *.*`) | `SHOW GRANTS FOR 'kesh'@'%'` rend `GRANT USAGE ON *.*` et `GRANT ALL PRIVILEGES ON \`kesh\`.*` — **moins** que le fichier, pas plus | **Fausse dans les deux sens.** Provenance du `GRANT ALL *.*` que décrivait la rétrospective 29 : le fichier versionné ne l'a porté que dans un commit intermédiaire de la 22-5, jamais sur `main` (`git log --all -S'ON *.*' -- scripts/mariadb-init/01-dev-grants.sql` : `31aaa20a`, hors de `main`, porte la ligne `GRANT ALL PRIVILEGES ON *.*` ; `94b5643e`, sur `main`, ne la porte que dans un **commentaire** qui la dit retirée) ; le montage est vide depuis le 2026-10-11, donc `kesh` n'avait que `kesh`.* dès ce démarrage ; le `*.*` observé le 2026-10-14 (29-4) est un **`GRANT` posé à la main par un agent** (journal de l'Epic 15, 2026-10-10 : « L'agent a fait GRANT ALL ON *.* TO kesh sur le dev ») — pratique qui a masqué la panne du montage et rendu vertes les sondes `INNODB_TRX` des 29-4 et 29-8. |
| « Le fichier `scripts/mariadb-init/01-dev-grants.sql` est monté dans le conteneur » (`docs/testing.md` § *Base de dev jetable*, point 1) | `docker inspect kesh-mariadb-dev` : le montage a pour source `/home/gcorbaz/devel/kesh-chore-tmpfs/scripts/mariadb-init`, un répertoire **vide** (créé par Docker, propriétaire `root`, 2026-10-11) : le worktree `kesh-chore-tmpfs` n'existe plus, ce n'est même plus un dépôt git | **Cause établie.** |
| « Le conteneur est celui du compose de dev du dépôt principal » | `com.docker.compose.project.working_dir` = `/home/gcorbaz/devel/kesh-chore-tmpfs` ; démarré le 2026-10-23 07:55 | Le conteneur a été (re)créé depuis un worktree, par `docker compose -f docker-compose.dev.yml up` lancé *dans* ce worktree. |
| « Les suites emploient l'utilisateur `kesh` » | `scripts/test-fast.sh:33` : `DATABASE_URL=mysql://kesh:kesh_dev@…/kesh` par défaut ; `docs/testing.md` § Prérequis idem ; l'environnement de la session porte la même valeur | Vrai pour le dev. **La CI emploie `root`** (`ci.yml:64`, `root:kesh_root`) ; `docs/testing.md:102` et `:308` aussi, pour deux recettes ponctuelles. |
| « La CI a les droits de la production » | La CI est `root` sur son service MariaDB ; la production a `kesh`.* seulement (`MARIADB_USER`, `docker-compose.yml:14`) | **Fausse : la CI a bien plus que la production.** Un test qui exige un privilège passe en CI *et* en dev si le conteneur est trop large — c'est le mécanisme des Stories 29-4 et 29-8, côté CI. |
| « L'écart est corrigé quand le conteneur est conforme au fichier » | Le conteneur actuel, tel quel, **ne peut plus créer une base éphémère** (`CREATE DATABASE` hors `kesh`.* refusé) : tout `#[sqlx::test]` rougit d'emblée. Non exécuté ici (écriture) ; déduit des droits relevés | Relevé du matin ; **sans objet depuis 18:13Z** (ligne suivante). |
| « Le conteneur a été remis en état » | **Recréé par l'orchestrateur le 2026-10-23 vers 18:13Z** depuis `/home/gcorbaz/devel/kesh` : `docker inspect` rend le montage `/home/gcorbaz/devel/kesh/scripts/mariadb-init`, `StartedAt` 2026-10-23T18:13:03Z ; `SHOW GRANTS` (lu en `-N -B -r`) : `USAGE` sur `*.*`, `ALL` sur `kesh`.*, `\_sqlx\_test%`.*, `kesh%`.* — exactement le fichier plus `MARIADB_USER`. | **Conforme.** La remise en état est faite ; le dév n'en demande aucune et ne dispose plus d'un conteneur défaillant pour éprouver le garde (d'où la source injectable, 1b). |

**Prémisse de l'issue #577 vérifiée** : `ci.yml` n'a que trois jobs (`backend`, `frontend`, `docker-build`), aucun ne lance
Playwright ; `CLAUDE.md` (§ *E2E*, lignes ~220–226) et `docs/ci.md` (lignes 12, 23–32, 133, 166, 188–192, 212–216)
décrivent un job `e2e` qui n'existe pas. L'issue reste vraie telle qu'écrite. Le même symptôme est écrit ailleurs : `docs/ci.md:9-10` (« 3 jobs »), `CLAUDE.md:267` (« les **trois** jobs ») ; `:269` est un relevé daté, annoté et non réécrit (AC12), `docs/testing.md:590` (« job `e2e`, step `Smoke test /api/v1/_test/seed` »).

**Prémisse de l'epic : « les six fichiers `.svelte-kit/` versionnés à la racine »** — vérifiée : `git ls-files | grep
svelte-kit` en rend six (`.svelte-kit/ambient.d.ts`, `generated/client/{app,matchers,nodes/0,nodes/1}.js`,
`tsconfig.json`), tous ajoutés par le commit `046efa51` (23-3) — `08e20353` (24-4b) n'a fait que retoucher `ambient.d.ts` ; `.gitignore:8` n'ignore que
`frontend/.svelte-kit/`. Aucun outil ne les lit : `grep -rn svelte-kit` (hors `node_modules`, `target`, documents de
planning) ne rend que `frontend/package.json` (`svelte-kit sync`, qui écrit dans `frontend/.svelte-kit/`),
`frontend/tsconfig.json` (`extends ./.svelte-kit/tsconfig.json`, **relatif à `frontend/`**), `.gitignore`, `frontend/.gitignore:8` (`/.svelte-kit`), `.dockerignore`, et `crates/kesh-db/tests/test_schema_guard.rs:847` (`SKIPPED_DIRS`, un lecteur qui *saute* le répertoire). Les fichiers de la racine sont des résidus d'un `svelte-kit sync` lancé depuis la racine.

## Volet 1 — Les droits du conteneur de dev (C3)

### Conception

**1a. La cause se traite à sa source : un montage qui dépend d'un worktree jetable.** Le compose de dev monte
`./scripts/mariadb-init` ; relatif au répertoire où l'on lance `docker compose`. Lancé depuis un worktree, le conteneur
hérite d'un chemin que la suppression du worktree vide. Ce n'est pas corrigeable dans le compose (un chemin absolu
écrit en dur casserait les autres postes ; une variable de plus ajoute un réglage à oublier). **Retenu (C-31-2-1)** : la
règle « le conteneur de dev se crée **depuis le dépôt principal** seulement » est écrite (`docs/testing.md` § *Base de
dev jetable*, `docs/agents-orchestration.md` § 2 — interdits — et rappelée au § 4), **et** un garde en signale la **conséquence** (1b : des droits qui ne sont plus ceux du fichier) — non la règle elle-même, qu'un
conteneur monté depuis un worktree encore présent satisfait (C-31-2-15) ; la prévention est #699. Le même § 2 reçoit **trois interdits** (AC5) : créer le conteneur ailleurs que depuis le dépôt principal, toute recréation du conteneur par un agent, et **tout `GRANT`/`REVOKE` manuel sur le MariaDB de dev partagé** — un droit manquant se signale à l'orchestrateur (c'est ce geste, répété sans trace, qui a masqué la panne du montage douze jours). La règle est portée par un AC (AC5), non par la seule prose.

**1b. Un garde qui lit les droits réels : `scripts/check-dev-grants.sh`.** Lancé par `scripts/test-fast.sh` **hors
du bloc `if [ "$RUN_LINT" = 1 ]`**, donc avant toute compilation (`cargo fmt`, `cargo clippy`) et sous `--no-lint` comme sans,
juste après le contrôle de `cargo-nextest` (et utilisable seul). **Il ne fait qu'une chose — ce qui aurait détecté l'incident
du 2026-10-23 : comparer l'ensemble des droits de `kesh` sur le conteneur à celui du fichier versionné** (C-31-2-15 : le
périmètre a été réduit en validation P3, après que les contrôles ajoutés en P1 et P2 — comparaison de la source du montage,
distinction fine par le texte des erreurs de `docker`, options d'injection propres au garde — se furent révélés chacun défaillants).

- **Sources.** Le fichier de droits et le compose sont lus **relativement à l'emplacement du script**
  (`$(dirname "$0")/..`) ; `MARIADB_USER`, `MARIADB_PASSWORD` et `MARIADB_DATABASE` sont **trois clés lues** dans
  `docker-compose.dev.yml` (:63-65) — le mot de passe ne vient donc pas de `DATABASE_URL`, que le garde ne lit pas. La commande
  `docker` s'injecte par la variable `KESH_DOCKER` (défaut `docker`) : c'est la seule injection, et c'est elle qui rend tous
  les cas de test de bout en bout.
- **Déroulé.** (1) `KESH_DOCKER` introuvable : « garde sauté : docker introuvable », code 0. (2) `docker ps --filter
  name=^kesh-mariadb-dev$ --filter status=running -q` ; sortie vide : « garde sauté : conteneur non en marche », code 0
  (message distinct) ; `ps` lui-même en échec (socket, groupe `docker`, **démon arrêté**) : non nul, cause nommée — un démon
  absent n'est pas un conteneur absent, et `--no-lint` ne l'y excuse pas. (3) Acquisition : `docker exec
  -e MYSQL_PWD kesh-mariadb-dev mariadb -N -B -r -u <MARIADB_USER> -e 'SHOW GRANTS FOR CURRENT_USER'`, le mot de passe **exporté**
  (`export MYSQL_PWD`) puis transmis **sans valeur** — ni dans l'argv de `docker` ni dans celle du client, donc absent de `ps`
  (la forme `-e MYSQL_PWD=<valeur>` l'y mettrait : M13) ; `exec` en échec (authentification refusée, client absent) : non nul,
  cause nommée. Aucun client MariaDB n'est requis sur l'hôte. **`-N -B -r` est imposé** : sans `-r`, le client double les
  antislashs (`\\_sqlx\\_test%`) ; avec, il rend `\_sqlx\_test%`, forme du fichier. (4) Comparaison d'**ensembles**.
- **Attendu.** Dérivé des seules lignes de `01-dev-grants.sql` qui, **une fois les commentaires retirés** (`-- …` jusqu'en
  fin de ligne, `# …` en début de ligne, `/* … */` y compris sur plusieurs lignes) et le fichier découpé en instructions par
  `;`, **commencent par `GRANT`, sans égard à la casse et sur une ou plusieurs lignes** ; une instruction qui contient `GRANT`
  sans commencer par lui est refusée (non nul : l'extraction ne comprend pas le fichier, elle ne l'ignore pas en silence).
  **Au moins une** instruction `GRANT`, sinon le garde échoue (un fichier absent, vide ou tout commenté ferait attendre
  `{USAGE, ALL sur kesh.*}`, exactement l'état défaillant du matin du 2026-10-23) ; l'attendu est ces `GRANT` **plus**
  ``GRANT ALL PRIVILEGES ON `kesh`.*`` et `USAGE`. `MARIADB_USER` est la **seule source** du nom de compte : `-u`, la
  normalisation (a) et l'attendu en dérivent ; une clé absente, vide ou interpolée (`${…}`) est refusée (non nul, clé nommée).
  **Normalisation explicite et minimale** (et rien d'autre) : (a) le compte `'kesh'@'%'` du fichier est réécrit en
  `` `kesh`@`%` `` (forme du serveur) ; (b) le `;` final des instructions du fichier est retiré ; (c) **sur toute ligne
  `ON *.*`** (critère : ` ON \*\.\* TO `), **réelle ou attendue**, la clause entière ` IDENTIFIED BY PASSWORD '<condensé>'`
  — espace qui la précède comprise — est **retirée** (aucun jeton de remplacement ; la ligne `USAGE` attendue, écrite par le
  garde, s'écrit donc sans cette clause : `` GRANT USAGE ON *.* TO `kesh`@`%` ``) ; privilèges, compte et `WITH GRANT OPTION`
  sont **conservés**. **Sur MariaDB le
  condensé figure sur l'unique ligne de niveau global, quelle que soit sa liste de privilèges** : un `GRANT PROCESS ON *.*`
  ou `GRANT ALL ON *.*` posé à la main **remplace** la ligne `USAGE` au lieu de s'y ajouter — c'est la forme réelle de
  l'incident, et celle des fixtures d'AC3. **Les antislashs sont conservés** — les retirer rendrait `_sqlx_test%` non échappé
  (le défaut que le fichier documente, lignes 33–39 : un `_` joker ouvre `Xsqlx_testZZZ`) indiscernable du bon droit (M6).
  La comparaison **et l'affichage** portent sur les **mêmes** lignes ainsi normalisées : la sortie du garde **ne contient
  jamais** les mots `IDENTIFIED BY PASSWORD`, ni le condensé. Tout droit **en moins** et tout droit **en plus** — au niveau global
  (la ligne `*.*` diffère : « droits globaux : attendu <ligne>, relevé <ligne> », privilège en trop et compte fautif
  visibles) comme **hors niveau global** (`ALL ON autre.*`, `USAGE` intact : « droit en plus : <ligne> ») — échoue, de même
  qu'un compte autre que `kesh@%` ou un `WITH GRANT OPTION`.
- **Sortie en cas d'échec** : le droit manquant ou superflu nommé (affichage sans condensé, ci-dessus), et **la remise en
  état, en texte arrêté**, que le garde affiche et n'exécute jamais : « Si le conteneur vient de démarrer, attendre l'état
  `healthy` (`docker ps`) et relancer. Sinon, **depuis le dépôt principal** (jamais un worktree) :
  `docker compose -f docker-compose.dev.yml up -d --force-recreate mariadb`, puis `sqlx migrate run --source
  crates/kesh-db/migrations` et `scripts/seed-dev-db.sql`. ⚠️ Cela **efface les bases de tous les agents** (tmpfs) : à demander
  à l'orchestrateur, jamais d'initiative ; un simple `restart` ne répare pas un montage périmé. » Le même texte est celui de
  `docs/testing.md` (AC5) ; `docs/agents-orchestration.md` § 2 reste la règle.
  **Démarrage à froid** : après un `restart`, le conteneur est « en marche » alors que `/docker-entrypoint-initdb.d` s'exécute
  encore ; des droits partiels n'y sont pas un écart durable. Le garde ne l'attend pas (extension écartée) : le message
  ci-dessus ouvre sur « attendre `healthy` », et `docs/testing.md` l'écrit.
- **Angles morts écrits, non cachés.** (i) Les droits du rôle `PUBLIC` n'apparaissent pas dans `SHOW GRANTS` d'un utilisateur.
  (ii) L'acquisition passe par le socket du conteneur (`kesh@localhost`), le gate par TCP (`kesh@%`) : un compte `'kesh'@'172.%'`
  posé à la main resterait invisible. (iii) **Le garde ne vérifie pas la règle « depuis le dépôt principal »** : un conteneur
  monté depuis un worktree encore présent, au fichier identique, passe ; il rougit au moment où l'égalité des droits est
  rompue (typiquement après la suppression du worktree et un redémarrage), non à celui de la violation. La prévention est
  #699, qui reste ouverte. **Ces trois angles morts sont écrits à trois endroits** — l'en-tête de `check-dev-grants.sh`, `docs/testing.md` et la ligne de `docs/agents-orchestration.md` § 6.1 — pour que « aucun droit global » (table 1c) et « écart corrigé » ne se lisent pas comme une garantie plus forte que celle du garde (AC5).
- **Une branche qui modifie `01-dev-grants.sql`** fait échouer l'égalité (droits réels du conteneur ≠ fichier de la branche) :
  c'est voulu et rare, et **cela ne fait rougir que le garde local** (AC7, `test-fast.sh`) : le test du garde est hermétique
  (AC3) et ne lit pas le fichier réel, la CI ne rougit donc pas. Conduite écrite (AC5) : l'agent **ne contourne pas**
  (aucune variable d'esquive, qui serait un silence), il signale à l'orchestrateur, seul à pouvoir recréer le conteneur
  (efface les bases de tous les agents, § 2).

**1c. Aligné sur la production — et sur la CI.** Table à écrire dans `docs/testing.md` :

| | Droits de `kesh` |
|---|---|
| Production (`docker-compose.yml` ; le manuel, `admin-manual.tex:995`, `GRANT ALL PRIVILEGES ON kesh.*`) | `ALL` sur `kesh`.* (celui de `MARIADB_DATABASE`) |
| Dev (`01-dev-grants.sql`) | idem **+** `ALL` sur `\_sqlx\_test%`.* et `kesh%`.* — nécessaires à `#[sqlx::test]` et à `kesh_e2e` ; **aucun droit global** |
| CI, aujourd'hui | `root` — **plus large que la production** |
| CI, après la story | `kesh` avec le **même fichier** appliqué en `root` au démarrage du job |

Dans `ci.yml`, job `backend` : une étape placée **après « Install mariadb-client »** (le client `mariadb` n'est installé qu'à cet endroit ; « Wait for MariaDB » n'emploie que `mysqladmin`) et avant « Apply migrations » exécute `mariadb -h 127.0.0.1 -P 3306 -uroot -pkesh_root <
scripts/mariadb-init/01-dev-grants.sql` (**sans `-h`, le client prendrait le socket Unix local, alors que MariaDB est un service conteneurisé**), et `DATABASE_URL` — défini au niveau `env:` du **job**, donc aussi vu par `cargo build` et `clippy` — devient `mysql://kesh:kesh_dev@127.0.0.1:3306/kesh`.
Les étapes « Apply migrations » et « Seed » (qui emploient `root` sur le client `mariadb`) restent en `root` : c'est
de l'outillage, pas le chemin de l'application. **Retenu (C-31-2-4)** parce qu'une CI en `root` fait exactement ce que
fait un conteneur de dev trop large : elle ne peut rien dire d'un privilège que la production n'a pas. ⚠️ **Ne se
vérifie qu'en CI** : un test qui dépendrait de `root` n'a jamais pu rougir en dev, où tout tourne en `kesh`. Si la CI
rougit pour cette raison, le dév **n'annule pas silencieusement** : chaque test concerné est nommé dans la fiche et
une issue est demandée (« on trace toujours »).

**1d. Le smoke de `release.yml` (#698, intégrée — C-31-2-10).** Son étape de smoke Docker emploie `root`
(`DATABASE_URL=mysql://root:kesh_root@…`, `release.yml:94`) pour lancer l'image publiée : le backend crée ses tables avec
`root`, ce que la production ne fait pas. Le service MariaDB du job crée **déjà** `kesh` (`MARIADB_USER: kesh`,
`MARIADB_PASSWORD: kesh_dev`, `MARIADB_DATABASE: kesh`, `release.yml:22-26`) : cet utilisateur a exactement `ALL` sur
`kesh`.*, c'est-à-dire **les droits de la production**. **Retenu : la seule modification est `DATABASE_URL` →
`mysql://kesh:kesh_dev@127.0.0.1:3306/kesh`** ; aucun fichier de droits, aucun `GRANT` (appliquer `01-dev-grants.sql`
ajouterait `\_sqlx\_test%`.* et `kesh%`.*, que la production n'a pas — l'inverse du but). ⚠️ **Le smoke de release ne se vérifie qu'à la
publication** (tag v0.15.0) : un échec y bloque la release ; la procédure de release le dit (`docs/ci.md`). Le risque est
borné, non nul : le job `e2e-smoke` (volet 2) lance, **sur chaque PR**, la même image contre le même service en
utilisateur `kesh`, avec les migrations au démarrage — il couvre le chemin « image + `kesh` + migrations » ; le smoke de
release n'ajoute que son montage propre (`KESH_PORT=80`, `0.0.0.0`, image taguée). La PR porte `closes #698` (sur la PR,
non sur les commits).

### Critères d'acceptation

- **AC1** — `scripts/check-dev-grants.sh` existe, exécutable ; il compare l'**ensemble** des droits de `kesh` sur le conteneur
  (acquis en `-N -B -r`) à celui attendu — dérivé des lignes `^GRANT` du fichier décommenté + `MARIADB_USER`, avec la
  normalisation minimale de 1b —, rend 0 si les ensembles sont égaux, non nul sinon, en nommant chaque écart **par un libellé qui lui est propre** (privilège
  et compte visibles) et qui ne contient jamais ni le condensé, ni les mots `IDENTIFIED BY
  PASSWORD`, ni le mot de passe (l'absence de fuite est **assertée**, AC3, M11, M13, M14) ; un attendu à zéro `GRANT` extrait
  est refusé, de même qu'une instruction qui contient `GRANT` sans commencer par lui. Sa seule injection est `KESH_DOCKER` (la
  commande `docker`) : aucune option propre au garde. Son en-tête de script dit ses **trois angles morts** (rôle `PUBLIC`, socket
  contre TCP, règle « depuis le dépôt principal » non vérifiée) et la remise en état (1b).
- **AC2** — Le garde échoue, message à l'appui, dans chacun des cas : droit manquant (`kesh`.* seul), droit global en trop
  (`ALL … ON *.*`, `PROCESS`), droit superflu **hors niveau global** (`ALL ON autre.*`, `USAGE` intact), `_` non échappé,
  compte autre que `kesh@%`, `WITH GRANT OPTION`, `docker ps` ou `docker exec` en échec (cause nommée), attendu à zéro `GRANT`,
  `MARIADB_USER` absent ou interpolé. **Le message nomme l'écart et donne la remise en état en texte arrêté** (1b) ; le
  privilège en trop (`PROCESS`, `ALL PRIVILEGES`) et le compte fautif (`` `kesh`@`localhost` ``) y figurent, le condensé jamais. Il rend 0 **et le dit** (`garde sauté : …`) dans **deux cas seulement** :
  `docker` introuvable, conteneur `kesh-mariadb-dev` non en marche — chacun avec son message propre. Il ne lit **aucun message
  d'erreur de `docker`** : l'état se lit par le code de retour et par la sortie de `docker ps`.
- **AC3** — `scripts/tests/check-dev-grants.test.sh` (même forme que `scripts/tests/prepare-release.test.sh`) exerce le garde
  et **est lancé par une enveloppe Rust**, `crates/kesh-db/tests/check_dev_grants_script.rs`, calquée sur
  `crates/kesh-db/tests/prepare_release_script.rs` (garde anti-test-muet incluse : le résumé « aucune assertion en échec »
  est exigé) — donc par nextest, le gate et la CI ; sans enveloppe, l'assertion de câblage (M3) ne tournerait jamais. **Tous
  les cas sont de bout en bout et hermétiques** : le test pointe `KESH_DOCKER` vers un `docker` factice qui **consigne son
  argv et la valeur de `$MYSQL_PWD`** dans un fichier et **échoue bruyamment** (code et message propres) sur toute
  sous-commande non prévue (`ps`, `exec`) — il n'atteint jamais le vrai `docker` ni le vrai conteneur, sur le poste comme en
  CI. **Chaque cas bâtit un mini-dépôt sous `mktemp -d`** (copie du script, d'un `01-dev-grants.sql` de référence **écrit par
  le test** — contenu figé —, et d'un compose de référence portant un mot de passe **distinctif**) : le test ne lit jamais
  `scripts/mariadb-init/01-dev-grants.sql` réel, si bien qu'une branche qui le modifie légitimement ne le fait pas rougir
  (conduite, AC5). **Seule l'assertion de câblage (AC4) lit le vrai `scripts/test-fast.sh`.** Fixtures : celle des droits égaux
  est **capturée sur le serveur réel** (sortie `-N -B -r`, condensé remplacé par une chaîne distinctive), et **les cas
  d'écart suivent la forme que MariaDB produit** (la ligne globale *remplace* `USAGE`, elle ne s'y ajoute pas ; fixtures à
  confirmer par une capture du dév). Cas : droits égaux → 0 ; droit en moins (`kesh`.* seul) → non nul ; droit global en trop
  (`ALL PRIVILEGES … ON *.*`, puis `PROCESS`, ligne remplaçante) → non nul, la sortie **contient** `ALL PRIVILEGES`, puis
  `PROCESS` ; droit superflu **hors niveau global** (`ALL ON autre.*`, `USAGE` intact) → non nul, la sortie contient `autre` ;
  `` `_sqlx_test%` `` non échappé → non nul ; compte `kesh@localhost` (ligne remplaçante) → non nul, la sortie contient
  `` `kesh`@`localhost` `` ; `USAGE … WITH GRANT OPTION` → non nul ; `docker` introuvable (`KESH_DOCKER` vers un chemin
  inexistant) → « garde sauté », 0 ; conteneur non en marche (`ps` vide) → « garde sauté », 0, message **distinct** du
  précédent ; `ps` en échec → non nul ; `exec` en échec → non nul ; attendu à zéro `GRANT` → non nul ; instruction contenant
  `GRANT` sans commencer par lui → non nul ; `MARIADB_USER` interpolé → non nul. **Assertions transversales** : sur chaque cas
  d'échec, la sortie ne contient ni `IDENTIFIED BY PASSWORD`, ni le **faux condensé de la
  fixture**, ni le mot de passe du compose ; sur tous les cas qui atteignent `exec`, l'argv consigné contient `-N -B -r`, le
  nom `kesh-mariadb-dev`, et `-e MYSQL_PWD` **suivi d'une espace ou de la fin de ligne** (regex `-e MYSQL_PWD( |$)`), ne
  contient pas le mot de passe, et la valeur consignée de `$MYSQL_PWD` **égale** celle du compose du mini-dépôt. **Éprouvé par
  mutation** : M1, M2, M3, M6 à M20 (tableau ci-dessous).
- **AC4** — `scripts/test-fast.sh` appelle le garde **avant la première commande `cargo` de compilation** (la ligne
  `cargo fmt`, à l'intérieur du bloc `if [ "$RUN_LINT" = 1 ]`), **après** le contrôle `cargo-nextest` et **avant** la ligne
  `if [ "$RUN_LINT" = 1 ]` — donc **hors du bloc lint**, sous `--no-lint` comme sans —, et s'arrête (code non nul) s'il
  échoue. L'assertion de câblage porte sur l'**unique ligne d'invocation non commentée** :
  `grep -nE '^[[:space:]]*[^#[:space:]].*check-dev-grants\.sh' scripts/test-fast.sh` rend **exactement une** ligne, dont le
  numéro est compris entre celui de `command -v cargo-nextest` et celui de `if [ "$RUN_LINT" = 1 ]`. Les commentaires
  (l'en-tête d'usage compris) ne comptent pas : M3 et M7 laissent un commentaire voisin en place. **La ligne est nue** :
  elle satisfait `^[[:space:]]*(bash )?(\./)?scripts/check-dev-grants\.sh[[:space:]]*$` (ni `||`, ni `;`, ni `&`, ni `if` : M15),
  et aucun `echo` du script ne nomme le fichier (le message d'étape dit « garde des droits »). Un appel inséré dans le
  bloc, « avant `cargo clippy` », ne tournerait pas sous `--no-lint` (M7).
- **AC5** — `docs/testing.md` § *Base de dev jetable* (point 1) dit la vérité : le montage dépend du répertoire d'où
  le compose est lancé ; la phrase « il rend à `kesh` les droits globaux » (`:22`) est **réécrite** (aucun droit global) ;
  la règle « depuis le dépôt principal » (écrite, **non vérifiée par le garde**) ; le garde, son message et la remise en état en texte arrêté (1b, avec « attendre `healthy` après un `restart` ») ; **les trois angles morts du garde** (rôle `PUBLIC`, socket contre TCP, règle non vérifiée) ; **la conduite quand le garde rougit parce que la
  branche modifie `01-dev-grants.sql`** (signaler à l'orchestrateur, ne pas contourner ; le test du garde, hermétique, ne rougit pas) ; la table de 1c ; les recettes en
  `root` (`:102`, `:308`) passent en `kesh` ou portent leur justification écrite. **`docs/agents-orchestration.md` § 2** (et la ligne de § 6.1, qui nomme les angles morts)
  reçoit trois interdits — créer le conteneur de dev ailleurs que depuis le dépôt principal (l'orchestrateur seul le recrée, ci-après) ; **toute recréation
  du conteneur par un agent** (`docker compose … up -d --force-recreate mariadb` compris, même depuis le dépôt principal :
  elle efface les bases de tous les agents, seul l'orchestrateur la fait) ; et tout `GRANT`/`REVOKE` manuel sur le MariaDB
  de dev partagé — rappelés au § 4 ; § 6.1 renvoie au garde (« écart corrigé par
  l'Epic 31 » devient vrai ou est ajusté). `crates/kesh-db/README.md` § Prérequis 2 renvoie au même endroit ;
  `crates/kesh-db/tests/refresh_tokens_repository.rs:5` (« nécessite `GRANT ALL PRIVILEGES ON *.*` ») est corrigé ;
  `frontend/DEBUGGING-KF007.md:16` (`root`) est corrigé ou assumé par écrit ; **`docker-compose.dev.yml:107-108`** (commentaire :
  « les droits globaux dont `#[sqlx::test]` a besoin ») est réécrit — un commentaire, qui ne recrée rien —,
  **`README.md:183`** (`mysql://root:…`) passe en `kesh`, **`README.md:193`** (« droits `CREATE/DROP` sur `*.*` (en local, utiliser `root`) », que le motif ci-dessous ne capte pas) est réécrit, et le doc-comment de **`crates/kesh-api/tests/reconciliation_e2e.rs:8249-8250`** (« la CI tourne en `root` », faux dès T4) est ajusté en T4 — `kesh-api` s'ajoute au décompte des modules. **Les recettes qui créent ou redémarrent le conteneur** portent la règle « depuis le dépôt principal » ou y renvoient : `docs/testing.md:41`, `:52` (prose du point 3, qui renvoie à la règle), `:63`, `:101` (commentaire `up -d mariadb`, ajusté avec la recette `:102`), `:163`, `CLAUDE.md:527`, `README.md:80`, `crates/kesh-db/README.md:34`, `website/index.html:189` (`git grep -nE "compose -f docker-compose\.dev\.yml"` ; les autres sorties — `.env.example`, `CHANGELOG.md`, `docs/FRONTEND-TENANT-AUDIT.md`, `docs/kesh-specifications.txt`, `configuration_transmise.rs` — sont des lecteurs de fichier ou du texte historique, sans recette) — au moins `docs/testing.md:163` et `CLAUDE.md:527`, lieux que lisent les agents ; `website/index.html`, guide public, est assumé par écrit s'il n'est pas modifié. **Inventaire clos par la forme**, exécuté le 2026-10-23 (la forme
  `':!_bmad-output'` échoue : `fatal: Spécificateur magique '_' non implémenté`, d'où `':(exclude)…'`) :
  `git grep -nE "GRANT ALL.*\*\.\*|droits globaux|mysql://root" -- . ':(exclude)_bmad-output' ':(exclude)docs/known-failures.md' ':(exclude)*.pdf' ':(exclude)docs/manual'`
  rend **9 sites** hors étapes de CI : `.github/workflows/ci.yml:64` et `release.yml:94` (T4), `README.md:183`,
  `crates/kesh-db/tests/refresh_tokens_repository.rs:5`, `docker-compose.dev.yml:107`, `docs/testing.md:22`, `:102`, `:308`,
  `frontend/DEBUGGING-KF007.md:16` (T3). Le dév **ré-exécute** la commande ; chaque occurrence est réécrite ou assumée par
  écrit, et la sortie finale ne porte que des occurrences assumées (le `-uroot` des recettes de seed, qui ne passe pas par
  ce motif, reste de l'outillage de conteneur).
- **AC6 (CI uniquement)** — Le job `backend` applique `01-dev-grants.sql` en `root` (étape après « Install
  mariadb-client », `-h 127.0.0.1`) puis emploie `kesh` ; il passe vert sur la PR.
- **AC7 (local)** — Le conteneur a été **recréé conforme par l'orchestrateur le 2026-10-23 vers 18:13Z** : le dév ne
  demande aucune recréation. Il vérifie `SHOW GRANTS`, `scripts/check-dev-grants.sh` rend 0 contre le conteneur actuel, et
  **un gate complet** (`scripts/test-fast.sh`) est lancé : tout rouge est trié (cause : privilège que la production n'a
  pas, ou autre) et consigné ; un test qui exigeait un privilège absent de la production est **corrigé** s'il emploie
  `INNODB_TRX`/`INNODB_LOCKS` (sondes de § 6.1), sinon une issue est demandée. L'état défaillant ne s'éprouve que par le
  `docker` factice (AC3, `KESH_DOCKER`).
- **AC18 (release uniquement, #698)** — `release.yml` démarre l'image du smoke avec `DATABASE_URL=mysql://kesh:kesh_dev@
  127.0.0.1:3306/kesh` : `grep -nF 'mysql://root' .github/workflows/release.yml` rend vide (local) ; la réussite du smoke
  de **release** ne se voit qu'au tag v0.15.0 (**non vérifié localement**, dit au Dev Agent Record et à `docs/ci.md`). Le
  chemin « image + `kesh` + migrations » est exercé plus tôt, sur chaque PR, par `e2e-smoke` (AC10).

### Ce que le dév ne fait pas

Aucun `docker restart|stop|compose down|up --force-recreate`, aucun `GRANT`/`REVOKE` à la main. Le conteneur a déjà été
remis en état (2026-10-23 vers 18:13Z, depuis le dépôt principal) : le dév n'en demande **pas** de nouvelle — elle
effacerait les bases de tous les agents. Le garde et ses tests s'éprouvent sans toucher au conteneur
(`docker` factice injecté par `KESH_DOCKER`).

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
se connecte à Docker Hub (`docker/login-action@v3`, **avant** `docker/setup-buildx-action@v3` : le builder tire
`moby/buildkit` depuis Docker Hub), monte ce builder (indispensable au cache `gha`), reconstruit l'image avec `cache-from:
type=gha` (les couches de `docker-build` sont déjà là : quelques secondes) et `load: true`, la lance en `--network=host` avec `KESH_TEST_MODE=true`, `KESH_HOST=127.0.0.1` (le mode test refuse un
bind non-loopback, `ConfigError::TestModeWithPublicBind`), `KESH_COOKIE_SECURE=false` (HTTP, cf. `CLAUDE.md` § E2E),
`KESH_PORT=3000`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` (le **service MariaDB** du job, utilisateur
`kesh` : l'application ne reçoit pas plus que la production), `KESH_JWT_SECRET` et `KESH_ADMIN_*` factices valides.
Avantage : on teste ce qui sera publié (binaire de production, frontend buildé tel que servi), sans second
`cargo build`. Les migrations s'appliquent au démarrage du backend (comme en production) ; **le seed est celui du
backend** (`POST /api/v1/_test/seed`, par `seedTestState`), pas un SQL : aucune étape « Seed » dans ce job.
Ni `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR` ni `KESH_SMTP_*` : ils ne servent qu'à des specs que le smoke ne lance pas.
Puis `actions/setup-node@v4` (Node 22, cache npm), `npm ci`, `npx playwright install --with-deps chromium`, et
`KESH_TEST_MODE=true KESH_BACKEND_URL=http://127.0.0.1:3000 npx playwright test tests/e2e/smoke.spec.ts`.
Attente du backend par `/health` en boucle avec délai (consigne § 4, point 4), avec le journal du conteneur affiché si elle
échoue. `timeout-minutes: 25`. Le rapport/trace Playwright (`frontend/test-results/`, `trace: 'retain-on-failure'`) est téléversé **en cas d'échec** (`actions/upload-artifact`,
rétention 3 jours). Le service MariaDB du job porte `MARIADB_USER: kesh`, `MARIADB_PASSWORD: kesh_dev`, `MARIADB_DATABASE: kesh` (c'est ce qui crée `kesh`) et `credentials` — **écrits en T5**, avec le job.

**Écartée** : un job qui compile `kesh-api` (`cargo build`) + télécharge l'artefact `frontend-build` — un second
`cargo build` complet (plus de dix minutes à froid), pour tester un binaire de débogage et un dossier de build qui ne
sont pas ce qu'on publie. Repli si le cache `gha` ne tient pas : revenir à ce montage (réversible).

⚠️ **Le smoke ne teste aucune fonctionnalité, et c'est délibéré** (`CLAUDE.md` § E2E, repris tel quel). Il reste
**facultatif pour la fusion** tant qu'il n'est pas apparu stable : la règle de branche protégée n'est pas modifiée par
cette story — décision de Guy à demander à l'orchestrateur (point 4 de « Issues liées, à commenter ou à décider »).

### Critères d'acceptation

- **AC8 (local)** — `smoke.spec.ts` passe contre la recette E2E de `docs/testing.md` (backend du worktree, base
  `kesh_e2e_312`, port attribué) : `2 passed`.
- **AC9 (mutation locale, exigée)** — Backend lancé **sans** `KESH_COOKIE_SECURE=false` : la spec rougit en `401` sur
  l'appel authentifié (c'est l'échec #285/`docs/testing.md`) — preuve que le smoke détecte un harnais mort. Backend
  sans `KESH_TEST_MODE` : rouge dès le `globalSetup` (`global-setup.ts` appelle `seedTestState('with-company')`). Les deux mutations sont décrites au Dev Agent Record **avec leur
  sortie**.
- **AC10 (CI uniquement)** — Le job `e2e-smoke` figure dans `ci.yml`, passe vert sur la PR, et sa durée est relevée ; il exerce le chemin « image + `kesh` + migrations au démarrage » que le smoke de release reprendra
  (objectif : quelques minutes ; si > 10 min, le noter et envisager le cache des navigateurs, `docs/ci.md` D-6-1-A).
- **AC11** — `smoke.spec.ts` ne dépend d'aucun autre `*.spec.ts` ni d'aucune variable absente de la recette de
  `docs/testing.md` ; `npx playwright test --list tests/e2e/smoke.spec.ts` rend exactement 2 tests.
- **AC12 (texte)** — `CLAUDE.md` § *E2E* : le paragraphe « ⛔ Constat du 2026-10-10 : ce smoke N'EXISTE PAS » est
  **remplacé** par une description de ce que fait le job (nom, spec, durée mesurée), (la mention « 2-3 minutes » de `CLAUDE.md:222` est **remplacée** par cette durée mesurée), et les phrases qui supposent le
  smoke (« **La CI n'exécute PAS la suite complète — mais elle exécute un SMOKE** ») restent exactes ; la mention
  « issue #577 » devient l'historique ; `CLAUDE.md:267` (« les **trois** jobs ») est corrigée ; `:269` est un relevé daté du 2026-08-30 (« les deux autres jobs étant
  verts ») : il reçoit une annotation datée, non une réécriture. `docs/ci.md` est **réécrit pour dire le dépôt** : quatre jobs (`backend`, `frontend`, `docker-build`,
  `e2e-smoke`) — lignes 9-10 (« 3 jobs »), schéma d'exécution, tableau des jobs et **timeouts** (dont `backend` : « 30 min »
  contre `timeout-minutes: 120` de `ci.yml`), section « migrations » (`:165` décrit `cargo sqlx migrate run`, alors que
  `ci.yml` boucle `mariadb <`) et « mode test », le bandeau ⚠️ de la ligne 12 retiré, les lignes D-6-1-* qui décrivent un
  job complet annotées « non retenu ». `docs/testing.md:590` (« job `e2e`, step `Smoke test /api/v1/_test/seed` ») est
  réécrit. **Inventaire clos par la forme, non par la valeur** : relecture ligne à ligne de `docs/ci.md` contre `ci.yml`
  (tableaux « jobs » et « timeouts » compris), complétée par `git grep -nE "job .?e2e|Smoke test /api|\b(3|4|trois|quatre)\b.*jobs?|jobs?\b.*\b(trois|quatre|3|4)\b" -- docs CLAUDE.md README.md website ':(exclude)docs/known-failures.md'`
  (archivé, laissé ; le motif `\be2e\b` est **retiré** : il rend des dizaines de `tests/e2e/`, `/tmp/kesh-e2e/` sans rapport,
  donc un inventaire que personne ne ferme). Sortie réelle du 2026-10-23 : `CLAUDE.md:267`, `docs/ci.md:9`, `:10`, `:12`,
  `:166`, `:172`, `:188`, `docs/testing.md:590` — plus quatre faux positifs de `CLAUDE.md` (`jobs` de cargo : `:300`, `:320`,
  `:326`, `:346`) ; `docs/ci.md:172` (« job `backend` », juste) est un faux positif de plus. Chaque occurrence restante est
  réécrite ou assumée par écrit ; les lignes `docs/ci.md:25`, `:32`, `:133`, `:212`, qui décrivent le job `e2e` sans le
  motif, sont couvertes par la **relecture ligne à ligne** de `docs/ci.md`.

## Volet 3 — Docker Hub authentifié (A2)

### Conception

`services.mariadb.credentials` (`username: ${{ secrets.DOCKERHUB_USERNAME }}`, `password: ${{ secrets.DOCKERHUB_TOKEN }}`)
dans les **trois** services MariaDB : `ci.yml` job `backend`, `ci.yml` job `e2e-smoke` (nouveau, **T5**), `release.yml` job
`docker` — **et** `docker/login-action@v3` (mêmes secrets) dans les jobs `docker-build` et `e2e-smoke` (**T5** pour ce
dernier) : leur `Dockerfile` tire `rust`, `node` et `debian` depuis Docker Hub, anonymement sinon (C-31-2-11). **La
connexion se place AVANT `docker/setup-buildx-action@v3`** dans les trois jobs qui construisent : le builder
`docker-container` tire `moby/buildkit` depuis Docker Hub à cette étape. `release.yml` se connectait déjà, mais **après**
`setup-buildx` (l. 41 puis 44) : T2 remonte la connexion avant l'étape de builder. Les secrets existent déjà pour `release.yml` (étape `docker/login-action`) ; elles n'y couvraient pas le
service, tiré **avant** toute étape.

**Les PR issues de forks et de Dependabot n'ont pas accès aux secrets.** Tranché (C-31-2-3) : **pas de contournement
conditionnel.** Le dépôt est tenu par un seul développeur, sans fork actif et sans `dependabot.yml`
(`ls .github` : aucun) ; une expression conditionnelle sur `credentials` (un objet nul ou `fromJSON(format(…))`) est
invérifiable hors d'un runner et ajoute une fragilité pour un cas qui n'existe pas. Conséquence **assumée et écrite**
dans `docs/ci.md` : une PR de fork échouera au démarrage du service **ou à l'étape de connexion** (échec bruyant, jamais un faux vert). Si le dépôt
accepte un jour des PR externes, le repli est `image: mirror.gcr.io/library/mariadb:10.11` (sans secret) — noté, non
adopté (la provenance de l'image n'est plus celle du Docker Hub officiel).

### Critères d'acceptation

- **AC13 (CI uniquement ; se constate en deux temps)** — Comptes **ancrés** (un commentaire ne doit pas les fausser) :
  `grep -cE '^\s+credentials:\s*$'` et `grep -cE '^\s*(-\s+)?uses: docker/login-action@'` (la forme `- uses:` compte). **Au commit de T2** : `ci.yml` rend 1
  et 1 (`backend`, `docker-build`), `release.yml` 1 et 1. **Au commit de T5** : `ci.yml` rend 2 et 2 (`e2e-smoke` ajoute son
  service et sa connexion). Dans chacun des trois jobs qui construisent (`docker-build`, `e2e-smoke`, `release.yml`),
  `grep -nE 'docker/login-action|docker/setup-buildx-action'` rend la connexion **avant** `setup-buildx`. Les runs de la PR
  passent (le tirage est authentifié : l'en-tête du service dans le journal du job le montre).
- **AC14** — Le paragraphe « ⚠️ **La CI dépend de Docker Hub.** » (`CLAUDE.md`, § Test Locally First, fin) est
  retiré, avec la mention A-C9, **parce que les images de service et de base sont authentifiées** (AC13) ; **l'image du
  builder** (`moby/buildkit`) l'est si la connexion précède bien `setup-buildx`, ce qui est une hypothèse sur le
  comportement de l'action : elle se **constate au journal du premier run** (tirage authentifié ou non), et `docs/ci.md`
  l'écrit comme telle. `docs/ci.md` dit à la place de l'avertissement les secrets requis, le comportement des forks, et que
  les vérifications requises de la branche protégée sont **trois** (`gh api repos/guycorbaz/kesh/branches/main/protection/required_status_checks` : `Backend (Rust)`, `Docker build (sanity)`, `Frontend (Svelte)`) ; `Backend (Rust)` (service `credentials`) et `Docker build (sanity)` (connexion) dépendent du jeton, dont l'invalidité bloquerait donc les fusions. Aucun autre site : `git grep -nI "429\|anonymement" -- 'docs/*.md' CLAUDE.md README.md '.github/*.yml'` rend trois lignes — `CLAUDE.md:275` (le site à réécrire), `README.md:225` et `docs/testing.md:314` (sans rapport : contenu de version, piège SMTP) ; `_bmad-output/`, `_bmad/` et `.claude/` (artefacts et cadre, hors documentation du dépôt) sont exclus du périmètre, ainsi que `CHANGELOG.md` (hors des quatre globs ; ses occurrences de « 429 » sont l'issue #429, sans rapport) — inventaire recompté et traité.
- **AC15** — `actionlint` (**absent de la station** : `command -v actionlint` vide ; le repli est donc le chemin réel) ou, à
  défaut, `python3 -c "import yaml; yaml.safe_load(...)"` sur les deux fichiers, **refait au dernier commit qui touche du YAML** (T2, T4, T5) confirme que les workflows restent
  syntaxiquement valides **localement** — syntaxe YAML seulement, **non le schéma GitHub Actions**
  (`services.<id>.credentials`) ; la validité d'exécution ne se voit qu'en CI (AC13).

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
| M1 | Comparaison du garde réduite à « attendus ⊆ réels » | AC3 : le cas « droit superflu hors niveau global » (`ALL ON autre.*`, `USAGE` intact) rougit — **non** le cas `*.*`, que la ligne remplaçante fait rougir par l'absence de `USAGE` |
| M2 | Garde qui ne compare pas la sortie de `docker exec` (rend 0 après une acquisition réussie) | AC3 : les cas « droit en moins » et « droit en trop » rougissent |
| M3 | Appel du garde retiré de `test-fast.sh`, **commentaire voisin laissé** | AC4 : l'assertion de câblage (une seule ligne d'invocation non commentée, comprise entre `command -v cargo-nextest` et `if [ "$RUN_LINT" = 1 ]`) rougit ; portée par `check-dev-grants.test.sh` |
| M4 | Backend du smoke sans `KESH_COOKIE_SECURE=false` | AC9 : 401 |
| M5 | `smoke.spec.ts` : le test « sans cookie » attend 200 | Rouge — prouve que la sonde négative sonde |
| M6 | Comparaison du garde qui retire les antislashs des deux côtés | AC3 : le cas `_` non échappé rougit |
| M7 | Appel du garde déplacé **dans** le bloc `if [ "$RUN_LINT" = 1 ]`, avant `cargo clippy`, **commentaire voisin laissé** | AC4 : l'assertion de câblage (borne `if RUN_LINT`) rougit |
| M8 | Garde qui rend 1 au lieu de 0 quand le conteneur n'est pas en marche (`docker ps` vide) | AC3 : le cas « conteneur non en marche » rougit |
| M9 | Garde qui rend 1 au lieu de 0 quand `docker` est introuvable | AC3 : le cas `KESH_DOCKER` vers un chemin inexistant rougit |
| M10 | Garde qui traite l'échec de `docker exec` comme « garde sauté » (rend 0) | AC3 : le cas `exec` en échec rougit |
| M11 | Message d'échec qui recopie la ligne `ON *.*` brute (avec son condensé) | AC3 : l'assertion de non-fuite rougit |
| M12 | Garde qui accepte un fichier de droits sans aucun `GRANT` | AC3 : le cas « zéro `GRANT` » rougit |
| M13 | Garde qui transmet le mot de passe **en valeur** dans l'argv (`-e MYSQL_PWD=<valeur>`) | AC3 : la seconde moitié de l'assertion (absence de la valeur) rougit |
| M14 | Retrait de la clause du condensé limité à la ligne `USAGE` (les autres lignes `ON *.*` gardent leur clause) | AC3 : les cas `*.*` et `PROCESS` (ligne remplaçante) rougissent sur l'assertion de non-fuite du faux condensé |
| M15 | Appel du garde conservé dans `test-fast.sh` mais suffixé `|| true` | AC4 : l'assertion de ligne nue rougit |
| M16 | Garde qui traite l'échec de `docker ps` comme « conteneur non en marche » (rend 0) | AC3 : le cas `ps` en échec rougit |
| M17 | Garde qui émet le même message pour « docker introuvable » et « conteneur non en marche » | AC3 : l'assertion de messages distincts rougit |
| M18 | Garde sans `-r` dans l'acquisition | AC3 : l'assertion sur `-N -B -r` rougit |
| M19 | Garde dont `MYSQL_PWD` n'est pas exporté (ou est codé en dur) | AC3 : la valeur consignée de `$MYSQL_PWD` diffère de celle du compose du mini-dépôt |
| M20 | Garde qui appelle `docker` en dur au lieu de `"$KESH_DOCKER"` | AC3 : l'argv consigné reste vide, ou le cas sans `docker` factice rend autre chose que « sauté » |

Restauration de chaque mutation : `git checkout -- <fichier>` **puis** `touch <fichier>` (`docs/agents-orchestration.md`
§ 7).

## Ce qui se vérifie en local et ce qui ne se vérifie qu'en CI

| Local | CI seulement |
|---|---|
| Garde et son test (AC1–AC4), mutations M1–M3, M6–M20 (le test tourne aussi en CI par son enveloppe Rust) ; **AC7** (le garde contre le conteneur actuel, puis le gate complet) | AC6 (job `backend` en `kesh`), AC10 (job `e2e-smoke`), AC13 (tirage authentifié) |
| `smoke.spec.ts` contre la recette locale (AC8, AC9, AC11) | La durée réelle du smoke ; le cache `gha` du `docker-build` ; la validité des secrets ; **AC18 : le smoke de `release.yml` en `kesh`, effectif au seul tag v0.15.0** (échec = release bloquée) |
| `.svelte-kit` (AC16, AC17), YAML syntaxique (AC15), textes (AC5, AC12, AC14) | Tout test qui dépendrait encore de `root` en CI |

Gate : `scripts/test-fast.sh` complet, sous le verrou commun. **`kesh-db` est touché** (l'enveloppe
`check_dev_grants_script.rs`, le commentaire de `refresh_tokens_repository.rs`, le README), sans que `CLAUDE.md` § *Exception
`kesh-db`* (migrations, `post_restore.rs`, repositories) l'**impose** : le gate complet est requis ici par AC7, par le
`push` et parce que l'enveloppe nouvelle doit tourner sous nextest — et, par prudence, sans ciblage en cours de boucle. Le volet 1 change en outre le chemin
d'exécution du gate (le garde passe avant nextest). Puis, depuis `frontend/` : `npm run check && npm run lint-i18n-ownership
&& npm run test:unit && npm run build`, et l'**E2E complet au dernier commit de code** (le smoke y est ajouté : la baseline
devient 7 à 10 échecs attendus sur N+2 tests, à recompter). Les deux tests de scripts (`check-dev-grants.test.sh`,
`prepare-release.test.sh`) tournent par leurs enveloppes Rust dans le gate complet.

## Issues liées, à commenter ou à décider (demandées à l'orchestrateur)

1. **Commenter #577** : prémisse confirmée, plus `docs/ci.md` entièrement concerné, `CLAUDE.md:267` et
   `docs/testing.md:590` ; la 31-2 la ferme (**mot-clé de fermeture sur la PR seulement**).
2. **#698 (ouverte, intégrée — C-31-2-10)** : fermée par la PR (`closes #698` sur la PR). Son corps dit encore « Hors
   périmètre de la 31-2 » : l'orchestrateur y ajoute un commentaire renvoyant à C-31-2-10.
3. **#699 (ouverte, reste ouverte)** — le conteneur de dev peut être créé depuis un worktree : `docker-compose.dev.yml`
   monte un chemin relatif. Le garde le *signale*, il ne l'*empêche* pas ; une vraie prévention (script
   `scripts/dev-db.sh`, volume nommé alimenté une fois) est une story, pas un correctif.
4. **Décision de Guy à demander** — le job `e2e-smoke` doit-il être une **vérification obligatoire** de la branche
   protégée ? Cette story ne modifie aucun réglage de dépôt.
5. **Écart à la lettre de l'epic, à porter à la rétrospective 31** : l'epic § 31-2 écrit que la fiche « rend le montage fiable » ;
   elle le **signale** après coup (le garde) et renvoie la prévention à #699 (C-31-2-1, C-31-2-15). Aucune issue à ouvrir.
6. **Rien à ouvrir** pour C3 : le constat (conteneur *moins* doté que le fichier), la cause (montage vide depuis le
   2026-10-11) et la provenance du `GRANT ALL *.*` de la rétrospective 29 (posé à la main par un agent, jamais dans le
   fichier versionné de `main`) sont dans cette fiche ; à noter à la rétrospective 31, non à rouvrir. À vérifier par
   l'orchestrateur : si `docker compose … restart` lancé depuis un worktree cible bien le conteneur (projet
   `kesh`) — hypothèse non exécutée ici — avant de borner la règle « depuis le dépôt principal » à `up`.

## Tâches

- [x] **T1 (volet 4)** `.svelte-kit/` retiré, `/.svelte-kit/` aux deux ignore ; AC16, AC17.
- [x] **T2 (volet 3)** `credentials` sur les services de `ci.yml` (`backend`) et de `release.yml`, `docker/login-action`
  sur `docker-build` et dans `release.yml`, **placés avant `setup-buildx-action`** (celui de `release.yml` est remonté) ;
  note A-C9 retirée ; AC13 (premier temps), AC14, AC15 (refait à T4 et T5). Le service et la connexion d'`e2e-smoke` sont écrits en T5.
- [x] **T3 (volet 1)** `scripts/check-dev-grants.sh` (égalité d'ensembles de droits, `docker` injectable par `KESH_DOCKER`,
  deux « garde sauté », tout autre échec non nul) + `scripts/tests/check-dev-grants.test.sh` + son enveloppe
  `crates/kesh-db/tests/check_dev_grants_script.rs` (rouge d'abord : M1–M3, M6–M20) ; câblage dans `test-fast.sh` hors du bloc
  lint ; textes `docs/testing.md` (dont la règle aux recettes `up`/`restart`), `docs/agents-orchestration.md`, `CLAUDE.md:527`,
  `crates/kesh-db/README.md`, `refresh_tokens_repository.rs:5`, `DEBUGGING-KF007.md`, `docker-compose.dev.yml:107-108`,
  `README.md:183` et `:193` ; AC1–AC5.
- [x] **T4 (volet 1c et 1d)** `ci.yml` job `backend` : droits appliqués en `root` après « Install mariadb-client »,
  `DATABASE_URL` en `kesh` ; `release.yml` : `DATABASE_URL` du smoke en `kesh` (#698) ; doc-comment de `reconciliation_e2e.rs:8249-8250` ajusté ; AC6, AC18.
- [x] **T5 (volet 2)** `smoke.spec.ts` (rouge d'abord : M5), job `e2e-smoke` **avec son service `credentials`, sa
  connexion Docker Hub puis `setup-buildx-action`**, mutations M4 ; AC8–AC11 ; AC13 (second temps).
- [x] **T6 (volet 2, textes)** `CLAUDE.md` § E2E et `docs/ci.md`, `docs/testing.md:590` réécrits ; AC12.
- [x] **T7** Vérifier le conteneur (déjà conforme, aucune recréation à demander) ; gate complet ; trier les rouges (AC7).
- [x] **T8** E2E complet au dernier commit de code ; Dev Agent Record **sans affirmer ce qui n'a pas tourné** (les
  AC « CI uniquement » et AC18 y figurent comme *non vérifiés localement*).

## Dev Notes

- **Périmètre minimal.** Aucun code de production ; `release.yml` est modifié pour `credentials` et pour le `DATABASE_URL` du smoke (#698, C-31-2-10) — rien d'autre.
- **Le symétrique existe peut-être déjà** (`docs/agents-orchestration.md` § 3) : `scripts/tests/prepare-release.test.sh`
  est le modèle de forme des tests de scripts ; `seedTestState`/`authedApiContext` sont réutilisés, non réécrits ;
  la fonction qui lit `KESH_BACKEND_URL` est `resolveBackendUrl` (`test-state.ts`).
- **Un test ne dépend d'aucun privilège que la production n'a pas** (§ 6.1) : c'est le critère que le volet 1 rend
  contrôlé.
- **Données réelles** : aucune ; mots de passe de dev/CI factices déjà versionnés (`kesh_dev`, `kesh_root`).
- **Le garde lit un mot de passe de dev** (`MARIADB_PASSWORD` du compose) : ne l'affiche jamais dans ses messages, ni le condensé d'une ligne `ON *.*`. Il l'exporte en `MYSQL_PWD` puis le transmet sans valeur (`docker exec -e MYSQL_PWD`) : jamais dans l'argv, ce que le `docker` factice consigne et que le test assère (AC3, M13).
- **Les tests de scripts sont lancés par une enveloppe Rust** : `crates/kesh-db/tests/prepare_release_script.rs` lance
  `scripts/tests/prepare-release.test.sh` (gate nextest et CI) et refuse un test muet ; `check-dev-grants.test.sh` reçoit
  la sienne (AC3). Conséquence : `kesh-db` est touché, gate complet (section « Gate » : requis par AC7 et l'enveloppe, non par la règle des migrations).

## Change Log

- 2026-10-23 — Fiche écrite (agent de fiche 31-2, Sonnet 5.5). Prémisses C3 et #577 établies au code et à la lecture
  du conteneur (lecture seule). Validation P1 à lancer (`312-validate-prompt-p1.md`, Opus, lentilles R et F).
- 2026-10-23 — **Validation P1** (Opus, deux lentilles ; R : 0 CRITICAL / 1 HIGH / 7 MEDIUM / 8 LOW ; F : 0 / 0 / 8 / 8).
  Rapports `312-validate-p1-{R,F}.md`. Vérifiés au `grep -nF`/`sed -n`/`docker inspect` (lecture seule) puis corrigés : R-1/F5
  (#698 non reportée dans la fiche : AC18, 1d, T4, tableaux, issues), R-2/F6 (la prescription « fichier de droits ou
  `GRANT` » donnait plus que la production : `release.yml` crée déjà `kesh`, seul `DATABASE_URL` change), R-3/F4 (étape CI
  après « Install mariadb-client », `-h 127.0.0.1` ; `DATABASE_URL` au niveau du job), R-4/F1 (conteneur déjà recréé à
  18:13Z : prémisse datée, AC7/T7/« Ce que le dév ne fait pas » réécrits), R-5/F1 (montage et droits à sources
  injectables `--mount-source`/`--grants-from-stdin`, cas supplémentaires d'AC3, M2 recâblée), R-6/L1 (garde avant
  `cargo clippy`, `--no-lint` compris ; M3), R-7/F8 (inventaire de textes : `CLAUDE.md:267-269`, `docs/ci.md:9-10`, `:30`,
  `:131`, `:165`, `docs/testing.md:590`, clos par la forme), R-8 (`refresh_tokens_repository.rs:5`, `testing.md:22`/`:102`/`:308`,
  DEBUGGING-KF007, règle au § 2 et § 4 portée par AC5), F2 (provenance du `GRANT ALL *.*`, interdit de `GRANT` manuel au
  § 2), F3 (`-N -B -r`, normalisation minimale qui garde les antislashs, cas `_` non échappé, fixture capturée, M6),
  F7 (`docker/login-action` sur `docker-build` et `e2e-smoke`, C-31-2-11 ; sources injectables du garde, C-31-2-12), LOW : R-L1, R-L2/L3, R-L3/L2, R-L4/L4 (compte
  `kesh@%`, `MYSQL_PWD`, ligne `USAGE`, rôle `PUBLIC`), R-L5 (tests de scripts non automatisés, dit), R-L6 (commentaire
  #698 demandé), R-L7 (actionlint absent), R-L8 (artefact, service), F-L5 (`cmp` du fichier du montage), F-L6, F-L8, F-L7
  (hypothèse signalée, non exécutée). **Réfuté : aucun** (les 33 constats se retrouvent au dépôt). Recomptes : **18 AC**
  (AC1 à AC18, +AC18), **8 tâches**. Validation P2 à lancer (`312-validate-prompt-p2.md`, Sonnet, lentilles R et F).
- 2026-10-23 — **Validation P2** (Sonnet, deux lentilles ; R : 0 CRITICAL / 0 HIGH / 3 MEDIUM / 8 LOW ; F : 0 / 0 / 5 / 9).
  Rapports `312-validate-p2-{R,F}.md`. Tous vérifiés au `grep -nF`/`sed -n`/`git grep` avant correction ; **réfuté : aucun**
  (R2-1/F2-1 : `prepare_release_script.rs` lance bien le test shell ; R2-2/F2-3 : la commande d'inventaire d'AC5 échoue
  réellement en `fatal: Spécificateur magique '_'` et la forme `':(exclude)…'` rend 9 sites ; F2-2 : `test-fast.sh:53-58`
  place `cargo clippy` dans le bloc `if` ; F2-4/R2-5 : `release.yml:41` précède `:44` ; `docker-compose.dev.yml:107` et
  `README.md:183` existent). Corrigés : R2-1/F2-1 (enveloppe Rust `check_dev_grants_script.rs`, Dev Note fausse remplacée,
  section « Gate » : `kesh-db` touché ⇒ gate complet même en boucle), F2-2 (câblage comparé à la ligne `if RUN_LINT`, M7),
  R2-2/F2-3 (AC5 : forme exécutée, sortie réelle, `docker-compose.dev.yml:107-108` et `README.md:183` ajoutés à AC5 et T3),
  R2-3 (T2/T5 : service, connexion et buildx d'`e2e-smoke` écrits en T5 ; AC13 en deux temps ; « indépendants » corrigé),
  F2-4/R2-5 (connexion avant `setup-buildx`, `release.yml` remonté ; AC14 nuancé, constaté au journal du premier run),
  F2-5 (« absent » ≠ « en erreur », port de `DATABASE_URL` comparé au port publié, attendu à zéro `GRANT` refusé, cas AC3,
  M9, M10, M12) ; LOW : F2-6 (`MYSQL_PWD` exporté, transmis sans valeur), F2-7 (chaîne du condensé seule retirée,
  `WITH GRANT OPTION`, `MARIADB_USER` et `MARIADB_DATABASE`), R2-4/F2-8 (« faute de second filet » réécrit : `e2e-smoke`
  couvre « image + `kesh` + migrations »), F2-9 (conduite si la branche modifie le fichier), R2-9/F2-10 (renvoi),
  F2-11 (`setup-buildx` sur `e2e-smoke`), F2-12 (comptes ancrés), F2-13 (décompte des modules déclaré), R2-6 (motif `\be2e\b`
  retiré d'AC12, sortie réelle reportée), R2-7 (non-fuite assertée, M11), R2-8 (registre, amendement C-31-2-4), R2-10
  (M8, M9), R2-11 (sections « Story » et « Dev Agent Record »). F2-14 : informations, aucune correction. Recomptes depuis
  la source : **18 AC** (inchangé), **8 tâches** (inchangé), **12 mutations** (M1 à M12, +6). Validation P3 à lancer
  (`312-validate-prompt-p3.md`, Opus, lentilles R et F).
- 2026-10-24 — **Validation P3** (Opus, deux lentilles ; R : 0 CRITICAL / 0 HIGH / 5 MEDIUM / 10 LOW ; F : 0 / 0 / 6 / 7).
  Rapports `312-validate-p3-{R,F}.md`. **Signal de recyclage déclaré** (`CLAUDE.md` § *Règle de splitting préventif*,
  amendement D5) : sévérité égale à la P2 (3 et 5 MEDIUM), tous sur le garde tel que les remédiations P1 et P2 l'avaient
  étendu. **Sortie par retrait** (C-31-2-15, choix de l'orchestrateur) : le garde se réduit à l'égalité des droits ; la
  source du montage, le port, les options `--grants-from-stdin` et `--mount-source` et la lecture du texte des erreurs de
  `docker` disparaissent. Vérifiés au `grep -nF`/`sed -n` avant correction ; **réfuté : aucun**. *Sans objet après retrait* :
  R3-4 et F-2 (comparaison du port), F-1 (casse de « No such object »), R3-L5 (source de montage inexistante), et, dans R3-3,
  le contrôle de port toujours « autre serveur » et les options d'injection. *Corrigés* : R3-1/F-4 (câblage assis sur
  l'unique ligne d'invocation non commentée ; M3 et M7 laissent un commentaire voisin), R3-2/F-L3 (`KESH_DOCKER` pointé vers
  un chemin inexistant, jamais un `PATH` amputé ; deux messages « garde sauté » distincts et assertés), R3-3 et F-3 (tous les
  cas de bout en bout avec un `docker` factice qui consigne son argv et échoue bruyamment sur une sous-commande imprévue ;
  mini-dépôt sous `mktemp -d`, R3-L6 ; aucun `DATABASE_URL` hérité), F-5 (argv sans valeur du mot de passe asserté, M13 ;
  M2 et M8 recomposées), R3-5 (le garde signale la conséquence, non la règle : angle mort écrit, #699), F-6 (`README.md:193`,
  `reconciliation_e2e.rs:8249-8250` à AC5 et T4), LOW : R3-L1/F-L5 (forme `- uses:` à AC13), R3-L2 (trois vérifications
  requises à AC14), R3-L3 (`python3`, AC15 refait au dernier commit YAML), R3-L4 (prémisse : `94b5643e` est sur `main`, sa
  seule occurrence est un commentaire), R3-L7/F-L4 (mot de passe tiré de `MARIADB_PASSWORD` du compose), R3-L8 (AC7 au tableau
  local/CI, phrase de la section « Gate » réécrite), R3-L9 (recettes `up`/`restart` à l'inventaire de la règle), R3-L10
  (table 1c), F-L1 (normalisation complétée : `;`, ligne `USAGE` attendue, écart nommé sans condensé), F-L6 (`CLAUDE.md:269`
  annoté et non réécrit), F-L7 (décompte à trois modules ; amendement de C-31-0-3 au registre 312). F-L2 : l'angle mort de
  l'hôte (socket contre TCP) est écrit ; le contrôle de `SHOW GRANTS FOR PUBLIC` n'est **pas** adopté (extension du garde,
  contraire à C-31-2-15) — `PUBLIC` reste un angle mort déclaré. Recomptes depuis la source (fiche, après remédiation) :
  **18 AC** (AC1 à AC18), **8 tâches**, **13 mutations** (M1 à M13, +1). Validation P4 à lancer (`312-validate-prompt-p4.md`,
  Sonnet, lentilles R et F).
- 2026-10-24 — **Validation P4** (Sonnet, deux lentilles ; R : 0 CRITICAL / 0 HIGH / 1 MEDIUM / 9 LOW ; F : 0 / 0 / 2 / 6).
  Rapports `312-validate-p4-{R,F}.md`. Vérifiés au `grep -nF`/`sed -n` avant correction ; **réfuté : aucun**. Les trois MEDIUM
  (R4-1, F4-1, F4-2) sont un seul défaut : sur MariaDB le condensé figure sur l'unique ligne `ON *.*`, qu'un droit global posé à
  la main *remplace* (elle ne s'ajoute pas à `USAGE`). *Corrigés* : masquage du condensé par un jeton sur **toute** ligne
  `ON *.*`, réelle ou attendue ; affichage des écarts par un libellé sans `IDENTIFIED BY PASSWORD` ; fixtures « remplaçantes » ;
  assertions **positives** (le privilège et le compte en trop figurent) et négatives sur le faux condensé ; cas « droit superflu
  hors niveau global » rattaché à M1 ; M14 (masquage limité à `USAGE`). LOW : R4-2 (espace du condensé retirée avec lui),
  R4-3/F4-7 (test hermétique : `01-dev-grants.sql` de référence écrit par le test ; seul le câblage lit le vrai `test-fast.sh` ;
  conduite si une branche modifie le fichier), R4-4 (angles morts écrits en en-tête du script, `docs/testing.md`, § 6.1),
  R4-5/F4-4 (ligne d'invocation nue, M15 `|| true`, M16 échec de `ps`, M17 messages identiques, M20 `docker` en dur ; démon
  arrêté = non nul, dit), R4-6 (texte exact de la remise en état), R4-7/F4-6 (`docs/testing.md:52` et `:101` à l'inventaire ;
  AC14 en `git grep -nI … -- '*.md' '*.yml'`), F4-3 (assertions `-N -B -r`, nom du conteneur, valeur de `$MYSQL_PWD` : M18, M19,
  regex `-e MYSQL_PWD( |$)`), F4-5 (démarrage à froid : « attendre `healthy` » dans le message, extension écartée), F4-8
  (extraction écrite : commentaires, casse, multilignes, instruction contenant `GRANT` sans commencer par lui refusée), R4-8
  (citation de l'« Exception `kesh-db` » corrigée : le gate complet est requis par AC7 et l'enveloppe, non par cette règle),
  R4-9 (écart à la lettre de l'epic aux « Issues liées »), R4-10 (« 2-3 minutes » remplacé en AC12 ; `MARIADB_USER` source
  unique, clé absente ou interpolée refusée). *Non repris* : R4-10 sections « Project Structure Notes » et « References » du
  gabarit (sans conséquence). **Écart de procédure déclaré par la lentille F** : deux commandes `docker` en lecture hors liste
  (`docker ps --format …`, et un `docker exec … SHOW GRANTS` avec le mot de passe de dev en valeur dans l'argv), sans écriture.
  Recomptes depuis la source (fiche, après remédiation) : **18 AC**, **8 tâches**, **20 mutations** (M1 à M20, +7 ; périmètre :
  fiche seule). Passe ciblée P5 à lancer (`312-validate-prompt-p5-ciblee.md`, Haiku, une lentille, `git diff HEAD~1..HEAD` du
  commit de remédiation).

### Validation P5 ciblée (2026-10-24) — Haiku, lentille T sur `12a23dbf` : 0/0/3/2 ; remédiée par l'orchestrateur

Les trois MEDIUM, vérifiés au `sed -n` par l'orchestrateur (règle des MEDIUM de Haiku), sont **réels** :
- **M-1** — forme du jeton ambiguë (remplacement « espace comprise » contre affichage qui retire la clause « avec espace » ;
  ligne `USAGE` attendue sans chaîne quotée). **Retrait** : plus de jeton ; la clause ` IDENTIFIED BY PASSWORD '…'` est
  **retirée** des lignes `ON *.*`, comparaison et affichage portent sur les mêmes lignes normalisées ; la ligne `USAGE`
  attendue s'écrit sans clause. AC1, AC3 et M14 alignés.
- **M-2** — la remise en état prescrit `up -d --force-recreate`, qu'aucun interdit d'agent ne nommait : AC5 fait recevoir
  à `docs/agents-orchestration.md` § 2 un **troisième** interdit, toute recréation du conteneur par un agent.
- **M-3** — la commande d'inventaire de l'AC14 était trop large (346 lignes) : bornée à `docs/*.md`, `CLAUDE.md`,
  `README.md`, `.github/*.yml`, sortie réelle reportée (trois lignes, une seule à traiter).
LOW laissés : L-1 (sorties de l'inventaire du compose), L-2 (littéral `kesh` du fichier de droits — échec bruyant).
**Écart de la lentille** : deux fichiers temporaires écrits dans `kesh-gate-logs/`, supprimés avant le rapport.
Passe suivante : ciblée P6 (Sonnet) sur le commit de cette remédiation.

- 2026-10-23 — **Validation P6 ciblée** (Sonnet, lentille T sur `b37ce17d`, rapport `312-validate-p6-T.md`) :
  0 CRITICAL / 0 HIGH / 0 MEDIUM / 4 LOW. Appliqués par l'agent de développement : L-1 (conception 1a : les trois
  interdits du § 2 nommés), L-2 (AC5 : premier interdit reformulé, « créer […] ailleurs que depuis le dépôt principal
  (l'orchestrateur seul le recrée) »), L-3 (AC14 : `CHANGELOG.md` nommé dans l'exclusion), L-4 (l'entrée P5, placée après
  la File List, déplacée sous `## Change Log` sans changement de texte).
- 2026-10-23 — **Validation terminée** (0 MEDIUM+ à la P6). Trend : P1 Opus R 0/1/7/8, F 0/0/8/8 → P2 Sonnet R 0/0/3/8,
  F 0/0/5/9 → P3 Opus R 0/0/5/10, F 0/0/6/7 (signal de recyclage, retrait C-31-2-15) → P4 Sonnet R 0/0/1/9, F 0/0/2/6 →
  P5 ciblée Haiku 0/0/3/2 → P6 ciblée Sonnet 0/0/0/4. Fait constaté par l'orchestrateur le 2026-10-23 (recréation du
  conteneur) : `SHOW GRANTS FOR 'kesh'@'%'` rend `` GRANT USAGE ON *.* TO `kesh`@`%` IDENTIFIED BY PASSWORD '*…' `` — la
  capture réelle de la fixture du dév le confirmera. Statut `in-progress`.
- 2026-10-23 — **Développement** (Claude Opus 5.5). Volets livrés dans l'ordre 4, 3, 1, 2 (`a5233131`, `7ceee92e`,
  `883293b3`, `7495144d`). Garde des droits et son test hermétique (20 cas, 105 assertions, gawk et mawk), enveloppe
  Rust ; 18 mutations du garde sur 18 tuées ; smoke 2 passed, M4, M5 et la variante sans `KESH_TEST_MODE` rouges.
  Gate complet vert (nextest 3847/3847, 6 ignorés ; Vitest 1820 ; check, i18n, build) ; E2E complet : 7 échecs KF-029
  après rejeu isolé de deux échecs de montage. AC6, AC10, AC13 et AC18 ne se constatent qu'en CI. Fixture capturée :
  la forme `IDENTIFIED BY PASSWORD '*…'` est confirmée. Choix C-31-2-16 à C-31-2-22. Revue de code P1 à lancer
  (`312-review-prompt-p1.md`, Sonnet, lentilles B, E, A). Statut `review`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement de l'Epic 31), le 2026-10-23.

### Debug Log References

Journaux hors dépôt (`/home/gcorbaz/devel/kesh-gate-logs/`) : `312-gate-complet.log` (gate backend),
`312-gate-frontend.log`, `312-e2e-complet.log`.

**Tests rouges d'abord (sortie copiée).** Le test shell seul, avant l'écriture du garde :

```
⛔ garde introuvable : /home/gcorbaz/devel/kesh-31-2/scripts/check-dev-grants.sh
⛔ 1 assertion(s) en échec sur 0 cas.
rc=1
```

L'enveloppe Rust par nextest (`-E 'binary(check_dev_grants_script)'`) :

```
thread 'check_dev_grants_script_compares_real_grants_to_versioned_file' panicked at crates/kesh-db/tests/check_dev_grants_script.rs:43:5:
le test de check-dev-grants.sh a échoué (exit status: 1):
⛔ garde introuvable : /home/gcorbaz/devel/kesh-31-2/scripts/check-dev-grants.sh
Summary [   0.066s] 1 test run: 0 passed, 1 failed, 0 skipped
```

Puis, garde écrit et avant son câblage dans `test-fast.sh` : 19 cas verts, le cas 20 (câblage) rouge —
`✗ 0 ligne(s) d'invocation non commentée(s)` — ; vert après câblage.

**Fixture capturée** (AC3 ; confirme le fait relevé par l'orchestrateur) : `docker exec -e MYSQL_PWD kesh-mariadb-dev
mariadb -N -B -r -u kesh -e 'SHOW GRANTS FOR CURRENT_USER'`, mot de passe exporté et absent de l'argv, condensé masqué à
l'affichage :

```
GRANT USAGE ON *.* TO `kesh`@`%` IDENTIFIED BY PASSWORD '<condensé>'
GRANT ALL PRIVILEGES ON `kesh`.* TO `kesh`@`%`
GRANT ALL PRIVILEGES ON `\_sqlx\_test%`.* TO `kesh`@`%`
GRANT ALL PRIVILEGES ON `kesh%`.* TO `kesh`@`%`
```

Sans `-r`, la troisième ligne sort en `` `\\_sqlx\\_test%` `` (vérifié). `SELECT CURRENT_USER()` par le socket rend
`kesh@%`.

### Completion Notes List

**Ordre de livraison** 4, 3, 1, 2 (C-31-2-6), un commit par volet : `a5233131` (volet 4), `7ceee92e` (volet 3),
`883293b3` (volet 1 : T3 et T4), `7495144d` (volet 2 : T5 et T6). **Dernier commit de code : `7495144d`.**

**Garde et test (AC1–AC4).** Test hermétique `scripts/tests/check-dev-grants.test.sh` : **20 cas, 105 assertions**
(`grep -c '^    ✓'` sur sa sortie), vert sous gawk **et** sous `mawk 1.3.4` (`awk` → `mawk` en tête du `PATH`) —
`ubuntu-latest` a mawk. Contre le conteneur réel (AC7) : `garde des droits : droits de kesh conformes à
01-dev-grants.sql`, code 0 ; il passe en tête du gate (`312-gate-complet.log`, ligne 3).

**Mutations du garde (M1–M3, M6–M20), sous verrou**, chacune par motif vérifié unique, test shell rejoué, restauration
par `git checkout -- <fichier>` puis `touch` (`git status` propre à l'issue) — **18 sur 18 tuées** :

| # | Résultat | Première assertion qui rougit |
|---|---|---|
| M1 | tuée | « droit superflu hors niveau global » : code 0, sortie sans `` `autre` `` |
| M2 | tuée | 19 assertions, dont « droit en moins » (code 0) |
| M3 | tuée | câblage : 0 ligne d'invocation non commentée |
| M6 | tuée | `_` non échappé : code 0 |
| M7 | tuée | câblage : ligne 64 hors de ]49 ; 61[ |
| M8 | tuée | « conteneur non en marche » : code 1, attendu 0 |
| M9 | tuée | « docker introuvable » : code 1, attendu 0 |
| M10 | tuée | « exec en échec » : code 0, sortie sans « docker exec a échoué » |
| M11 | tuée | non-fuite : la sortie contient `IDENTIFIED BY PASSWORD` et le faux condensé |
| M12 | tuée | « zéro GRANT » : sortie sans « aucune instruction GRANT » |
| M13 | tuée | argv d'exec sans `-e MYSQL_PWD` nu ; mot de passe dans un argv |
| M14 | tuée | non-fuite sur les cas `ALL PRIVILEGES ON *.*` et `PROCESS` |
| M15 | tuée | invocation non nue (`… \|\| true`) |
| M16 | tuée | « ps en échec » : code 0 |
| M17 | tuée | message « docker introuvable » absent ; messages identiques |
| M18 | tuée | argv d'exec sans `-N -B -r` |
| M19 | tuée | `MYSQL_PWD` vu par exec : `<absente>` |
| M20 | tuée | « docker introuvable » : code 1 ; aucun `exec` consigné |

⚠️ **M20 a atteint le vrai `docker`** — c'est l'objet de la mutation : un `docker ps` et un `docker exec … SHOW GRANTS`
en lecture, avec le mot de passe factice du mini-dépôt (authentification refusée). Aucune écriture.

**Smoke (AC8, AC9, AC11), contre le backend du worktree** (port 3312, base `kesh_e2e_312`, recette de
`docs/testing.md`) : `npx playwright test --list tests/e2e/smoke.spec.ts` → `Total: 2 tests in 1 file` ; nominal →
**`2 passed`** ; **M4** (backend sans `KESH_COOKIE_SECURE=false`) → `Expected: 200 / Received: 401` à
`smoke.spec.ts:42`, 1 failed, 1 passed (la sonde négative) ; **sans `KESH_TEST_MODE`** → rouge au `globalSetup` :
`seedTestState(with-company) failed: 405 Method Not Allowed — … KESH_TEST_MODE may not be enabled` ; **M5** (sonde
négative attendant 200) → rouge à `smoke.spec.ts:58`, 1 failed, 1 passed. ⚠️ La spec n'étant pas encore suivie par git,
la restauration de M5 par `git checkout` n'a rien restauré : remise à `toBe(401)` par l'outil d'édition, vérifiée au
`grep -n "toBe("`.

**Gates réellement exécutés, au dernier commit de code `7495144d`** :

- `scripts/test-fast.sh` complet sous verrou, base `kesh_312` remise à zéro (supprimée, recréée, migrée, seedée — sans
  toucher au conteneur) : garde vert, fmt, clippy `-D warnings`, nextest **3847 exécutés, 3847 passés, 6 ignorés**
  (162 s ; 3846 à la v0.14.0 + l'enveloppe neuve). **Aucun rouge** : aucun test ne dépend localement d'un droit absent
  du fichier (AC7).
- Frontend sous verrou : `npm run check` **0 erreur** (29 avertissements préexistants), `lint-i18n-ownership` PASS,
  Vitest **156 fichiers, 1820 tests** verts, `npm run build` vert.
- **E2E complet** (base `kesh_e2e_312` neuve, **333 tests dans 80 fichiers**, `--list`) : **297 passed, 9 failed,
  17 skipped, 10 did not run** (12,0 min). Jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* :
  sept **KF-029 (#97)** — `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150` — ; deux échecs **de mon montage** — `inbox-import.spec.ts:59` et
  `supplier-documents.spec.ts:78`, dans `assertInboxDirsUsable` : mes dossiers étaient `/tmp/kesh-e2e-312/…`, non passés
  au runner ; les 10 « did not run » sont les tests suivants de ces deux specs. **Rejouées seules** avec `KESH_INBOX_DIR`
  et `KESH_DOCUMENTS_DIR` passés au runner : **12 passed**. Compte effectif : **7 échecs, la borne basse de la
  baseline** ; ni KF-052 ni pollution sur ce run.
- AC15 : `yaml.safe_load` sur `ci.yml` et `release.yml` après T2, T4 et T5 (« yaml ok » ; quatre jobs lus) — syntaxe YAML
  seulement, non le schéma GitHub Actions (`actionlint` absent de la station).
- AC16/AC17 : `git ls-files | grep -c svelte-kit` → 0 ; `npm run check` (0 erreur) et `npm run build` verts après le
  retrait ; `git check-ignore -v .svelte-kit/tsconfig.json` → `.gitignore:11:/.svelte-kit/`.

**Inventaires clos ré-exécutés.** AC5 (`git grep -nE "GRANT ALL.*\*\.\*|droits globaux|mysql://root" …`) : **vide**
après T4. Recettes `compose -f docker-compose.dev.yml` : toutes portent la règle ou y renvoient, sauf
`website/index.html:189`, assumé (C-31-2-21). AC12 : ne restent que des occurrences justes (quatre jobs), des
annotations historiques et les faux positifs `jobs` de cargo (`CLAUDE.md:298`, `:318`, `:324`, `:344`). AC13 : `ci.yml`
2 et 2, `release.yml` 1 et 1 ; la connexion précède `setup-buildx` dans `docker-build`, `e2e-smoke` et `release.yml`.
AC14 : l'avertissement A-C9 est retiré ; restent `README.md:226` et `docs/testing.md:336` (sans rapport ; `:225` et
`:314` avant les ajouts de la story), et deux lignes neuves qui racontent l'historique (`ci.yml:52`, `docs/ci.md:173`). AC18 : `grep -nF 'mysql://root'
.github/workflows/release.yml` → vide. **Manuel** : `grep` sur `docs/manual/fr/*.tex` ne rend que des droits de
production (`GRANT ALL PRIVILEGES ON kesh.*`, `GRANT SELECT … kesh_backup`), conformes à la table de `docs/testing.md` ;
aucun PDF à régénérer (la fiche n'en prescrit pas).

**Non vérifié localement — ne se constate qu'en CI** : AC6 (le job `backend` en `kesh` après « Apply dev grants ») ;
AC10 (le job `e2e-smoke` vert, et **sa durée**, que `CLAUDE.md` et `docs/ci.md` annoncent « à relever au premier run ») ;
AC13 (tirages authentifiés, dont `moby/buildkit` — hypothèse sur l'action) ; la validité des secrets ; le cache `gha`
partagé entre `docker-build` et `e2e-smoke`. **AC18** : le smoke de `release.yml` en `kesh` ne se voit qu'au tag
v0.15.0. Si la CI rougit faute d'un droit que la production n'a pas, chaque test se nomme ici et une issue se demande.

**Écarts à la fiche** : aucun sur le périmètre. Ajouts consignés au registre (C-31-2-16 à C-31-2-22) : ligne `USAGE`
attendue conditionnelle, clé de compose en double refusée, awk POSIX, message de réussite du garde, sonde négative par
la fixture `request`, `website/index.html` assumé, CHANGELOG sous « Interne ».

### File List

Développement (`git diff --name-status b34314ae 7495144d`, hors `_bmad-output/`) — **26 fichiers** : 4 ajoutés,
16 modifiés, 6 supprimés.

- Ajoutés : `scripts/check-dev-grants.sh`, `scripts/tests/check-dev-grants.test.sh`,
  `crates/kesh-db/tests/check_dev_grants_script.rs`, `frontend/tests/e2e/smoke.spec.ts`.
- Modifiés : `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.gitignore`, `.dockerignore`,
  `CHANGELOG.md`, `CLAUDE.md`, `README.md`, `crates/kesh-api/tests/reconciliation_e2e.rs`, `crates/kesh-db/README.md`,
  `crates/kesh-db/tests/refresh_tokens_repository.rs`, `docker-compose.dev.yml`, `docs/agents-orchestration.md`,
  `docs/ci.md`, `docs/testing.md`, `frontend/DEBUGGING-KF007.md`, `scripts/test-fast.sh`.
- Supprimés : les six `.svelte-kit/…` de la racine.
- Artefacts : cette fiche, `epic-31-choix-autonomes-312.md`, `sprint-status.yaml`, `312-review-prompt-p1.md`.
