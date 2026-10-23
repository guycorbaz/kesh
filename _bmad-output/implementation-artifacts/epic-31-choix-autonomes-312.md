# Epic 31 — choix autonomes de la story 31-2 (registre propre, à verser par union)

### C-31-2-1 — Le montage de dev dépend du répertoire de lancement : un garde, pas un correctif du compose

- **Contexte** : le conteneur `kesh-mariadb-dev` monte `/home/gcorbaz/devel/kesh-chore-tmpfs/scripts/mariadb-init`, un
  répertoire vide (le worktree n'existe plus) ; `kesh` n'a donc que `kesh`.*. Le compose monte `./scripts/mariadb-init`,
  relatif au répertoire de lancement.
- **Retenu** : règle « créé depuis le dépôt principal » écrite + garde `scripts/check-dev-grants.sh` branché sur
  `test-fast.sh` (égalité d'ensembles de droits, dérivée du fichier versionné).
- **Écartées** : chemin absolu en dur (casse les autres postes) ; variable d'environnement de plus (un réglage à
  oublier) ; script `dev-db.sh` qui refuse hors dépôt principal (une story, issue demandée).
- **Réversible** : oui.

### C-31-2-2 — Le smoke tourne contre l'image Docker de la PR

- **Contexte** : #577 demande un job minimal ; deux montages possibles.
- **Retenu** : job `e2e-smoke`, `needs: docker-build`, image reconstruite par le cache `gha`, lancée en
  `--network=host` avec `KESH_TEST_MODE`, `KESH_HOST=127.0.0.1`, `KESH_COOKIE_SECURE=false`, base `kesh` sur le service
  MariaDB en utilisateur `kesh`. Une spec neuve `smoke.spec.ts` (login UI + `GET /auth/me` authentifié + sonde négative
  sans cookie).
- **Écartée** : `cargo build` + artefact `frontend-build` (second build complet, binaire de débogage, pas ce qu'on
  publie). Repli si le cache ne tient pas.
- **Réversible** : oui.

### C-31-2-3 — Docker Hub : pas de contournement conditionnel pour les forks

- **Contexte** : les PR de forks et de Dependabot n'ont pas les secrets.
- **Retenu** : `credentials` inconditionnels sur les trois services ; limite écrite dans `docs/ci.md` ; repli
  `mirror.gcr.io` noté, non adopté. Dépôt à un seul développeur, ni fork ni `dependabot.yml`.
- **Écartée** : expression conditionnelle sur `credentials` (invérifiable hors runner, fragile pour un cas inexistant).
- **Réversible** : oui.

### C-31-2-4 — La CI emploie `kesh` avec les droits du fichier de dev, plus `root`

- **Contexte** : la CI est `root` (plus large que la production) ; un test à privilège excessif y passe.
- **Retenu** : le job `backend` applique `01-dev-grants.sql` en `root` après l'attente de MariaDB, `DATABASE_URL` en
  `kesh` ; migrations et seed restent en `root` (outillage). Non vérifiable hors CI : tout rouge est trié, nommé et
  tracé, jamais annulé en silence.
- **Écartée** : laisser `root` (conserve l'asymétrie que C3 dénonce) ; modifier `release.yml` (publication, issue demandée).
- **Réversible** : oui.

### C-31-2-5 — `.dockerignore` reçoit aussi `/.svelte-kit/`

- **Contexte** : mécanique, même symptôme que le `.gitignore` racine. **Retenu** : ajouté. **Réversible** : oui.

### C-31-2-6 — Ordre de livraison 4, 3, 1, 2 et un commit par volet

- **Retenu** : du plus mécanique au plus coûteux ; un volet défaillant n'arrête pas les autres. **Réversible** : oui.

### C-31-2-7 — Le smoke n'est pas rendu obligatoire par la story

- **Retenu** : aucun réglage de branche protégée modifié ; la décision est demandée à Guy via l'orchestrateur.
- **Réversible** : oui.

### C-31-2-8 — Le garde ne remet jamais le conteneur en état

- **Retenu** : il nomme la remise en état (recréation depuis le dépôt principal) sans l'exécuter : elle efface les
  bases de tous les agents (`docs/agents-orchestration.md` § 2). L'orchestrateur la fait, hors gate. **Réversible** : oui.

### C-31-2-9 — Prémisse corrigée : le conteneur a MOINS de droits que le fichier

- **Contexte** : la rétrospective 29 § 4.2 et l'epic le disaient plus doté (`GRANT ALL *.*`). Constat 2026-10-23 :
  `kesh`.* seul, montage vide. **Retenu** : la fiche corrige ; à noter à la rétrospective 31, aucune issue ne rouvre C3.

### C-31-2-10 — #698 intégrée à la 31-2 (choix de l'orchestrateur)

- **Contexte** : la fiche demandait d'ouvrir en P3 le fait que le smoke de `release.yml` interroge l'image publiée avec
  l'utilisateur `root` ; ouverte (#698). La règle de release fixe le plafond au tag v0.15.0 à moins de 136 issues
  ouvertes ; le compte est à 138.
- **Retenu** : intégrer #698 au volet 1 de la 31-2 — le smoke de release applique `scripts/mariadb-init/01-dev-grants.sql`
  (ou un `GRANT` aux droits de la production) puis démarre l'image avec l'utilisateur `kesh`, comme le job `backend` de
  `ci.yml` dans la même story. La PR porte `closes #698`. Ne se vérifie qu'en CI de release : le dire.
- **Écartée** : laisser #698 ouverte — même mécanisme que le volet 1, même correction.
- **Réversible** : oui.
- **Rappel** : #699 (montage relatif) reste ouverte ; le garde du volet 1 la signale sans la prévenir.

### Amendements du 2026-10-23 (validation P1 ; les entrées ci-dessus ne sont pas réécrites)

- **Amende C-31-2-10** (R-2, F6) : la prescription « applique `01-dev-grants.sql` (ou un `GRANT`…) » est **retirée**. Le
  service de `release.yml` crée déjà `kesh` (`MARIADB_USER: kesh`, `release.yml:22-26`) avec `ALL` sur `kesh`.*, soit les
  droits de la production ; la seule modification est `DATABASE_URL` (`release.yml:94`) → `mysql://kesh:kesh_dev@
  127.0.0.1:3306/kesh`, sans fichier de droits ni `GRANT` (le fichier de dev ajouterait `\_sqlx\_test%`.* et `kesh%`.*,
  que la production n'a pas). Porté par l'AC18 et la tâche T4 de la fiche. Ne se vérifie qu'au tag v0.15.0.
- **Amende C-31-2-4** (R-1) : l'« écartée » « modifier `release.yml` » vaut pour `credentials` et le reste de la
  publication, **non** pour le `DATABASE_URL` du smoke, repris par C-31-2-10.
- **Amende C-31-2-9** (F2) : provenance du `GRANT ALL *.*` de la rétrospective 29 — le fichier versionné de `main` n'a
  jamais porté de droit global (seulement deux commits intermédiaires de la 22-5, corrigés en passe 1) ; le montage est
  vide depuis le 2026-10-11 ; le `*.*` a été **posé à la main par un agent** (journal de l'Epic 15, 2026-10-10). D'où
  l'interdit de tout `GRANT`/`REVOKE` manuel sur le MariaDB de dev partagé, ajouté à `docs/agents-orchestration.md` § 2 par la
  fiche (AC5). Le conteneur a été recréé par l'orchestrateur le 2026-10-23 vers 18:13Z depuis le dépôt principal : plus de
  recréation à demander (AC7, T7).

### C-31-2-11 — Docker Hub authentifié aussi pour le build des images (F7)

- **Contexte** : `credentials` ne couvre que les services MariaDB ; les jobs `docker-build` et `e2e-smoke` tirent `rust`,
  `node` et `debian` (Dockerfile) anonymement. Retirer l'avertissement A-C9 aurait laissé un texte qui dit moins que le
  dépôt.
- **Retenu** : `docker/login-action@v3` (mêmes secrets) avant le build de ces deux jobs (`release.yml` le fait déjà) ;
  AC13 compte les étapes de connexion en plus des `credentials`. Les forks échouent bruyamment, comme pour C-31-2-3.
- **Écartée** : réécrire l'avertissement au lieu de le retirer (garde la dépendance anonyme, pour un coût de deux étapes
  YAML). **Réversible** : oui.

### C-31-2-12 — Le garde a des sources injectables (R-5, F1, F3)

- **Contexte** : depuis la recréation du conteneur, aucun état défaillant réel n'existe pour éprouver le garde ; AC3 ne
  portait que sur des `SHOW GRANTS`.
- **Retenu** : acquisition et comparaison séparées pour les droits (`--grants-from-stdin`, lu en `-N -B -r`,
  normalisation minimale qui garde les antislashs, fixture capturée) et pour le montage (`--mount-source <répertoire>`,
  `cmp` avec le fichier du worktree) ; `PATH` sans `docker` testé. Mot de passe par `MYSQL_PWD`.
- **Écartée** : normaliser en retirant les antislashs (rend `_sqlx_test%` non échappé indiscernable du bon droit).
  **Réversible** : oui.

### Amendements du 2026-10-23 (validation P2 ; les entrées ci-dessus ne sont pas réécrites)

- **Amende C-31-2-4** (R2-8) : le « Retenu » dit « après l'attente de MariaDB » ; l'étape de droits emploie le client
  `mariadb`, installé par « Install mariadb-client » (`ci.yml:156-157`), **après** « Wait for MariaDB ». Elle se place donc
  après « Install mariadb-client » et avant « Apply migrations » (déjà dans la fiche, 1c).
- **Amende C-31-2-11** (F2-4, R2-5, F2-11) : la connexion `docker/login-action` précède `docker/setup-buildx-action` dans
  les trois jobs qui construisent (`docker-build`, `e2e-smoke`, `release.yml` — remontée de `:44` avant `:41`), le builder
  tirant `moby/buildkit` depuis Docker Hub ; `e2e-smoke` porte lui-même `setup-buildx` (cache `gha`). Que l'image du
  builder soit authentifiée est une hypothèse sur le comportement de l'action, constatée au journal du premier run.
- **Amende C-31-2-12** (F2-5, F2-6, F2-7) : le mot de passe est exporté puis transmis sans valeur (`docker exec -e
  MYSQL_PWD`) ; seule la chaîne du condensé est retirée de la ligne `USAGE` ; un attendu à zéro `GRANT` est refusé ; le
  garde distingue « absent » (trois cas, rend 0 en le disant) de « en erreur » (non nul, cause nommée).

### C-31-2-13 — Le test du garde est lancé par une enveloppe Rust dans `kesh-db` (R2-1, F2-1)

- **Contexte** : la fiche affirmait qu'aucun test automatique ne lance `scripts/tests/*.test.sh` ; faux, car
  `crates/kesh-db/tests/prepare_release_script.rs` lance `prepare-release.test.sh` (nextest, CI) et refuse un test muet.
- **Retenu** : `crates/kesh-db/tests/check_dev_grants_script.rs`, calquée sur ce précédent, lance
  `check-dev-grants.test.sh` ; toutes ses acquisitions sont injectées (jamais le conteneur réel). Conséquence assumée :
  `kesh-db` est touché, donc gate complet même en cours de boucle (`CLAUDE.md`).
- **Écartée** : un test local jamais lancé (test muet : M3 ne rougirait jamais en CI) ; une étape YAML d'une ligne dans
  `ci.yml` (ne tourne pas au gate local).
- **Réversible** : oui.

### C-31-2-14 — `docker-compose.dev.yml:107-108` et `README.md:183` rejoignent la story (R2-2, F2-3)

- **Contexte** : l'inventaire d'AC5, enfin exécutable, rend deux sites que la fiche ne nommait pas : un commentaire du
  compose de dev (« les droits globaux dont `#[sqlx::test]` a besoin », faux) et un `mysql://root:…` de `README.md`.
- **Retenu** : les deux sont réécrits (mécanique, même symptôme) ; le commentaire n'exige aucune recréation du conteneur.
- **Réversible** : oui.

### C-31-2-15 — Signal de recyclage en validation P3 : le garde des droits se réduit à ce qu'il doit voir (choix de l'orchestrateur, 2026-10-24)

- **Contexte** : la validation P3 (Opus) rend 5 MEDIUM (R) et 6 MEDIUM (F), après 3 et 5 en P2 — sévérité égale, et
  **tous** portent sur le garde `check-dev-grants.sh` tel que les remédiations P1 et P2 l'ont étendu (source du montage
  comparée par `cmp`, port de `DATABASE_URL`, distinction fine « absent / en erreur » par le texte des messages de
  `docker`, options `--grants-from-stdin` et `--mount-source`). C'est le **recyclage** du `CLAUDE.md` (§ *Règle de
  splitting préventif*, amendement D5) : chaque correction ajoutait une surface que la passe suivante trouvait fautive.
  La sortie que le `CLAUDE.md` désigne comme la meilleure est un **retrait**.
- **Retenu** : le garde ne fait plus que ce qui aurait détecté l'incident du 2026-10-23 — **comparer l'ensemble des
  droits de `kesh` sur le conteneur à celui du fichier versionné**. Conception :
  1. `docker` s'injecte par la variable `KESH_DOCKER` (défaut `docker`) : les tests passent un `docker` factice qui
     **consigne son argv**, et tous les cas sont de bout en bout, sans option d'injection propre au garde.
  2. Si `docker` est introuvable, ou si le conteneur n'est pas **en marche** (`docker ps --filter name=^<nom>$ --filter
     status=running -q` vide) : message « garde sauté : … » et code 0. Aucune lecture de message d'erreur de `docker`.
  3. Sinon : `SHOW GRANTS` par `docker exec`, le mot de passe transmis **par le nom** de la variable
     (`-e MYSQL_PWD`, valeur héritée de l'environnement), jamais par sa valeur dans l'argv ; échec d'`exec` → code non
     nul avec la cause ; ensembles différents → code non nul, différence affichée et commande de remise en état
     **nommée** (jamais exécutée).
  4. **Retirés** : la comparaison du port de `DATABASE_URL`, la comparaison de la source du montage, les options
     `--grants-from-stdin` et `--mount-source`. La règle « conteneur créé depuis le dépôt principal » reste une règle
     **écrite** (`docs/testing.md`, `docs/agents-orchestration.md`) ; le garde ne prétend pas la vérifier, et #699
     reste ouverte pour une prévention.
  5. Le câblage s'asserte sur **l'unique** ligne d'invocation **non commentée** de `test-fast.sh`.
- **Écartées** : redécouper la 31-2 (le recyclage est confiné au garde, les trois autres volets ont convergé) ;
  poursuivre l'extension (chaque passe en a trouvé le défaut suivant).
- **Réversible** : oui — une vérification retirée pourra revenir avec sa propre story si un incident la motive.

### Amendements du 2026-10-24 (validation P3 ; les entrées ci-dessus ne sont pas réécrites)

- **Amende C-31-2-1** (R3-5) : « un garde la rend visible » est inexact. Le garde ne signale que la **conséquence** (des
  droits qui ne sont plus ceux du fichier) ; un conteneur monté depuis un worktree encore présent, au fichier identique, passe.
  La fiche l'écrit comme angle mort (1b) et renvoie la prévention à #699.
- **Remplace C-31-2-12** (C-31-2-15, R3-2, R3-3) : les options `--grants-from-stdin` et `--mount-source`, la comparaison de la
  source du montage et celle du port ne sont plus dans la fiche. La seule injection est `KESH_DOCKER` ; tous les cas de test
  sont de bout en bout avec un `docker` factice qui consigne son argv. Ce qui reste de C-31-2-12 : `-N -B -r`, la
  normalisation minimale qui garde les antislashs, la fixture capturée, `MYSQL_PWD` sans valeur dans l'argv.
- **Amende C-31-2-12 / C-31-2-15** (F-L4, R3-L7, F-3) : le mot de passe vient de la clé `MARIADB_PASSWORD` du compose
  (`docker-compose.dev.yml:65`, avec `MARIADB_USER` et `MARIADB_DATABASE`), non de `DATABASE_URL` que le garde ne lit plus ;
  le garde lit ses fichiers relativement à son emplacement, ce qui permet au test de bâtir un mini-dépôt (`mktemp -d`) pour
  changer l'attendu (R3-L6). Aucun cas ne dépend d'une variable d'environnement héritée.
- **Amende C-31-2-14** (F-6) : l'inventaire d'AC5 gagne `README.md:193` et le doc-comment de
  `crates/kesh-api/tests/reconciliation_e2e.rs:8249-8250` (ajusté en T4) ; le décompte des modules passe à trois
  (`kesh-db`, `kesh-api`, `frontend`) — la règle de splitting ne joue toujours pas. La règle « depuis le dépôt principal »
  est portée aussi par les recettes `up`/`restart` (R3-L9).
- **Amende C-31-0-3** (F-L7) : « petits, indépendants » et « pour du YAML et un fichier SQL » ne décrivent plus la 31-2 : un
  couplage (le job `e2e-smoke` n'existe qu'au volet 2) et `kesh-db` touché pour l'enveloppe du test du garde (C-31-2-13).
  Le registre commun n'est pas réécrit ici.

### Amendements du 2026-10-24 (validation P4 ; ne réétendent pas C-31-2-15)

- **Précise C-31-2-15** (R4-1, F4-1, F4-2) : le condensé de `SHOW GRANTS` figure sur l'unique ligne `ON *.*`, que tout droit
  global posé à la main remplace. Le masquage vaut donc pour toute ligne `ON *.*` (jeton fixe), l'affichage d'un écart n'emploie
  jamais les mots `IDENTIFIED BY PASSWORD`. Ce n'est pas un contrôle de plus : c'est la comparaison existante, correctement définie.
- **Précise C-31-2-13** (R4-3, F4-7) : le test du garde est hermétique (mini-dépôt, `01-dev-grants.sql` de référence écrit par le
  test) ; seule l'assertion de câblage lit le vrai `test-fast.sh`. Une branche qui modifie le vrai fichier de droits ne fait donc
  rougir que le garde local, jamais la CI.
- **Écart à l'epic** (R4-9) : « rend le montage fiable » devient « le signale » (C-31-2-1, C-31-2-15) ; à porter à la rétrospective 31.


### Choix du développement (2026-10-23 ; agent de développement, ne réécrivent aucune entrée ci-dessus)

### C-31-2-16 — La ligne `USAGE` attendue n'est ajoutée que si le fichier ne porte aucun droit `ON *.*`

- **Contexte** : sur MariaDB, un droit global *remplace* la ligne `USAGE ON *.*` au lieu de s'y ajouter (fait établi en
  validation P4). Un fichier de droits qui accorderait un jour un droit global ferait attendre deux lignes `*.*` là où le
  serveur n'en rend qu'une : le garde rougirait pour toujours, même sur un conteneur conforme.
- **Retenu** : l'attendu reçoit `GRANT USAGE ON *.* TO \`<user>\`@\`%\`` seulement si aucune de ses lignes ne porte
  ` ON *.* TO `. C'est la définition correcte de l'attendu, non un contrôle de plus (C-31-2-15 tient).
- **Écartée** : ajouter `USAGE` inconditionnellement (faux rouge permanent dans un cas que le fichier interdit
  aujourd'hui, mais qu'il n'empêche pas). **Réversible** : oui.

### C-31-2-17 — Une clé `MARIADB_*` présente plusieurs fois dans le compose de dev est refusée

- **Contexte** : le garde lit `MARIADB_USER`, `MARIADB_PASSWORD` et `MARIADB_DATABASE` par `grep` ; un second service qui
  porterait la même clé rendrait la lecture ambiguë.
- **Retenu** : une seule occurrence admise, sinon non nul, clé et nombre d'occurrences nommés — même famille que « clé
  absente, vide ou interpolée » (fiche 1b). **Écartée** : lire le seul bloc `mariadb:` (analyse YAML en shell, fragile).
  **Réversible** : oui.

### C-31-2-18 — L'extraction des `GRANT` est en awk POSIX, éprouvée sous gawk et mawk

- **Contexte** : le poste a gawk, `ubuntu-latest` a mawk par défaut ; le test du garde tourne en CI par son enveloppe.
- **Retenu** : aucune extension gawk ; le test a été rejoué avec `awk` → `mawk 1.3.4` (105 assertions vertes).
  **Réversible** : oui.

### C-31-2-19 — Le garde dit aussi sa réussite

- **Retenu** : « garde des droits : droits de kesh conformes à 01-dev-grants.sql » (code 0) ; sans lui, `test-fast.sh`
  n'affichait rien entre l'en-tête de l'étape et la suivante, et l'AC7 ne se constatait qu'au code de retour.
  **Réversible** : oui.

### C-31-2-20 — Smoke : sonde négative par la fixture `request`, nettoyage dans le premier test

- **Contexte** : `resolveBackendUrl` n'est pas exporté ; un `afterEach({ page })` ouvre une page vide pour la sonde
  négative, et `clearAuthStorage` y journalise un `SecurityError` sur `localStorage`.
- **Retenu** : la sonde négative emploie la fixture `request` de Playwright (contexte neuf, sans cookie, sur le `baseURL`
  de la configuration, qui lit le même `KESH_BACKEND_URL`) ; `clearAuthStorage` est appelé dans le `finally` du premier
  test. Aucun helper réécrit ni exporté. **Réversible** : oui.

### C-31-2-21 — `website/index.html:189` n'est pas modifié (assumé par écrit, AC5)

- **Retenu** : la commande `docker compose -f docker-compose.dev.yml up -d` du guide public suit un `git clone` dans la
  même recette : elle se lance, par construction, depuis le dépôt principal. La page est en anglais et publique ; y
  ajouter la règle des worktrees parlerait à un lecteur qui n'en a pas. **Réversible** : oui.

### C-31-2-22 — Le CHANGELOG range la 31-2 sous « Interne, invisible de l'utilisateur »

- **Retenu** : sous-section déjà employée par le CHANGELOG ; aucune des quatre entrées ne change ce que voit
  l'utilisateur de Kesh. Section `## [0.15.0] — Non publié` créée par la 31-2 (la 31-3 l'écrit aussi : union au rebase).
  **Réversible** : oui.
