# Story 33-10 : l'outillage de release : le gate en mode CI, et la montée et le retour de version dans un script du dépôt (#925 ; B1, B3 de la rétrospective 0.17.4)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`, numéros de
ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`, C-e334-21 à 25, 30 et 31. Commits en
`refs` ; à la PR : `closes #925`. **Le script de montée servira à la 0.17.5, à migration (33-1a), première à passer par #847 en production.** La rétrospective citée,
`release-0.17.4-retro-2026-11-06.md`, n'est pas sur cette branche : elle se lit, en lecture seule, dans `/home/gcorbaz/devel/kesh/_bmad-output/implementation-artifacts/` (branche `docs/retro-0174`).*

## Story

En tant qu'orchestrateur d'une release, je veux lancer d'une seule commande les tests **tels que la CI les lance**, et la recette de montée et de retour **telle que la release
l'exige** ; pour ne plus découvrir en CI un défaut invisible au gate local, ni réécrire à chaque release, hors dépôt, un script que rien ne garde.

## Ce que le code dit (prémisses établies)

- **L'écart est réel** : `scripts/test-fast.sh:70` lance `cargo nextest` (un processus par test) ; `.github/workflows/ci.yml:195-199`, `cargo test --workspace -j1 -- --test-threads=1` (un processus par binaire). Incident : `release-0.17.4-retro-2026-11-06.md`, § 4.1, 17 tests rouges à la seule CI.
- **La CI** : `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3306/kesh` (`ci.yml:76`), `kesh` hors `root` (`:71-75`), migrations appliquées une à une par `mariadb … < fichier`, **sans `_sqlx_migrations`**, et seed en `root` (`:178-193`). ⚠️ **Le profil du shell local exporte `DATABASE_URL=…/kesh`, une base vide** : le script force la sienne. Dernier relevé : `rel0174/ci-cargo-test.log`, 204 binaires, 4820 passés, 16 ignorés. **La durée n'est dans aucun journal** : à mesurer et à écrire à `docs/ci.md`.
- **Le reste de #925** : `config.rs:1683-1714`, `EnvGuard` ne restitue que `DATABASE_URL`, alors que `reset_env` en efface une trentaine (`:1717`…) : **court, se corrige ici** : le garde capture `env::vars_os()` et le restitue entier au relâchement. Reste, écrit au doc-comment : un `cargo test` **parallèle** local peut lire l'environnement pendant qu'un test le tient ; ni la CI ni nextest ne sont concernés.
- **Les trois fois où le script a été réécrit** : `kesh-gate-logs/rel0173/boot-{lib,phase1,phase2}.sh`, `rel0174/boot-{lib,phase1,phase2,phase3}.sh`, `reset-db.sh`. Constante à chaque fois : phase 1 (image précédente, base vide, seed, API, empreinte, `.keshbackup`, dump) ; phase 2 (binaire neuf, migrations, empreinte) ; phase 3 (refus de l'image précédente au journal, dump restauré, image relancée, empreinte identique). Rétrospective § 5, A3 ; `git grep "previously applied" v0.17.4 -- scripts` vide.
- ⚠️ **Ce qui varie d'une release à l'autre** ne doit pas aller dans le script : 0.17.4 regardait les jetons et l'alias CAMT ; 0.17.5 regardera la numérotation des factures fournisseur et le libellé des écritures passées. D'où un **crochet** (C-e334-23).
- **Le patron de test** : `scripts/tests/check-dev-grants.test.sh` (`docker` factice, `KESH_DOCKER`, `scripts/check-dev-grants.sh:23,83`), lancé par `crates/kesh-db/tests/check_dev_grants_script.rs`, donc au gate et en CI.
- **Le relevé CAMT synthétique** de `rel0174/camt/` a été lu : IBAN d'exemple, noms et montants inventés.
- **`--network host`, `KESH_TEST_MODE=true`** (`_test/seed`) : repris ; `config.rs:97` refuse le mode test hors boucle locale. `verrous()` de `boot-lib.sh` est propre à la station : reste à l'appelant (C-e334-24).
- ⚠️ **Ce qui peut atteindre un binaire lancé par un script** (M-R10-1, lu) : `dotenvy::dotenv()` (`crates/kesh-api/src/main.rs:60`) cherche un `.env` **dans le répertoire courant puis ses parents**, et complète ce que l'environnement n'a pas ; le shell exporte `DATABASE_URL` ; les scripts `boot-*` travaillaient sur `kesh-mariadb-dev`. Un refus de chemins ne ferme rien de cela : d'où un environnement fermé, un conteneur de base et un dossier créés par le script lui-même (C-e334-30).

## Conception

### `scripts/gate-ci.sh <base> [journal]` (C-e334-21, 31)

Un script neuf, **`scripts/test-fast.sh` n'est pas touché**. **Base** : liste blanche `^kesh_ci_[a-z0-9_]+$`, qui écarte `kesh`, `kesh_gate*` et `kesh_e2e*` (L-R10-5) ; les droits de `kesh` la couvrent déjà (`kesh%`, `scripts/mariadb-init/01-dev-grants.sql`), ⛔ **aucun `GRANT`**. Recrée la base (`DROP`/`CREATE`, en `root` sur `kesh-mariadb-dev`, jamais un redémarrage). **Comme la CI** (L-R10-3) : boucle `mariadb … < fichier` sur les migrations, seed `scripts/seed-dev-db.sql`, **pas de `sqlx migrate run`** ; seul écart, déclaré : le nom de la base. **Environnement** : refus d'emblée si un `.env` se trouve à la racine du dépôt ou dans l'un de ses parents, la CI n'en ayant aucun ; `env -i` avec `PATH`, `HOME`, `CARGO_HOME`, `RUSTUP_HOME`, `CARGO_TARGET_DIR` et `DATABASE_URL` **forcée**. `cargo test --workspace -j1 -- --test-threads=1` ; sortie en `tee` au journal, code de sortie propagé ; dernière ligne : `gate-ci : N passés, M échoués, K ignorés` **recomptée depuis `test result:`**, et non recopiée. Sous `mem-guard` s'il n'y est pas déjà. Injections de test : `KESH_DOCKER`, `KESH_CARGO`.

**Quand** : le gate sert **avant la PR de release** (B1). La question d'en faire une règle plus large, « avant chaque push, ou seulement avant la PR de release », est la **Q3 de la rétrospective, sans réponse de Guy** : aucune règle neuve ne s'écrit ici. Au `CLAUDE.md`, une phrase factuelle seulement (*Documentation*).

### `scripts/recette-montee.sh` (C-e334-22, 23, 24, 30)

```sh
scripts/recette-montee.sh --image gcorbaz/kesh:0.17.4 --binaire target/debug/kesh-api --static frontend/build \
  --port-api 3190 --port-base 3391 --migrations-attendues 1 --diff-attendu supplier_invoices,migrations \
  [--apres ../kesh-gate-logs/rel0175/verifs-0175.sh] [--import]
```

**Une liste blanche, non une liste noire : le script ne connaît aucun accès de production, et ne nomme donc aucun chemin à refuser.** Tout ce qui suit se contrôle **avant toute action** :

- **Environnement.** Dès sa première ligne, le script se relance par `env -i`, avec `PATH=/usr/bin:/bin`, `HOME` et ses seules injections de test (`KESH_DOCKER`, `KESH_CURL`) : rien de l'environnement de l'appelant, `DATABASE_URL` comprise, ne l'atteint, ni son crochet.
- **Dossier jetable.** `D=$(mktemp -d /tmp/kesh-recette.XXXXXX)`, créé par le script, avec `documents/ inbox/ backup/ cwd/ home/ journal/`. **Le script n'écrit que là**, et aucune option ne désigne un lieu d'écriture ; il ne l'efface pas (données synthétiques), en donne le chemin à sa dernière ligne, et l'appelant en copie ce qu'il garde (`cp -a`). Seule variante, pour le test : `RECETTE_RACINE_JETABLE`, dont le `realpath` doit rester sous `/tmp`.
- **Aucun `.env` atteignable.** Le binaire tourne depuis `$D/cwd`. Si `$D/cwd`, `$D`, l'un de leurs parents jusqu'à `/` porte un `.env`, le script refuse : c'est ce qui rend `dotenvy` sans objet.
- **Base.** Le script crée **son propre conteneur** : `docker run --rm -d --name kesh-recette-db-$$ --label kesh.recette=$JETON --memory=2g --tmpfs /var/lib/mysql -p 127.0.0.1:$PORT_BASE:3306`, `-e` `MARIADB_ROOT_PASSWORD`, `MARIADB_DATABASE=kesh_recette`, `MARIADB_USER=kesh`, `MARIADB_PASSWORD` (littéraux factices), **`mariadb:10.11`**, image égale à `docker-compose.dev.yml:56`, ce qu'une garde du test vérifie. `DATABASE_URL` se construit à **un seul endroit**, `mysql://kesh:…@127.0.0.1:$PORT_BASE/kesh_recette`, après `docker inspect` : le conteneur qui répond porte ce nom et ce jeton, sinon refus. **Aucune option ne désigne un hôte, une base ou une URL** : `--base`, `--host`, `--database-url`, `--journal` sont des options inconnues, donc refusées, comme toute option inconnue. Pas de `GRANT` : `MARIADB_USER` a ses droits sur sa base ; la phase 5 prend un second conteneur jetable, `--port-base + 1`.
- **Binaire neuf.** `env -i`, depuis `$D/cwd`, avec cette liste exacte : `PATH`, `HOME=$D/home`, `DATABASE_URL`, `KESH_HOST=127.0.0.1`, `KESH_PORT`, `KESH_COOKIE_SECURE=false`, `KESH_STATIC_DIR`, `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR`, `KESH_ADMIN_BACKUP_DIR` (les trois sous `$D`), `KESH_ADMIN_USERNAME`, `KESH_ADMIN_PASSWORD`, `KESH_JWT_SECRET` (valeurs factices de `docs/testing.md`), `KESH_SMTP_HOST=smtp.invalid`, `KESH_SMTP_USER`, `KESH_SMTP_PASSWORD`, `KESH_SMTP_FROM`.
- **Image précédente.** `--network host`, `-e NOM=VALEUR` seulement, pour la même liste plus `KESH_TEST_MODE=true`, les dossiers étant **internes au conteneur** : jamais `-e NOM` nu (qui hérite), `--env-file`, `-v`, `--volume`, `--mount`. Motif d'image exact `^gcorbaz/kesh:[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9.]+)?$` : `latest` et tout autre dépôt refusés.
- **Lecture seule.** Trois entrées seulement viennent du dehors, chacune passée à `realpath -e` et comparée à `<racine>/` : `--binaire` et `--static` sous la racine du dépôt du script ; `--apres` sous le voisin `<racine>/../kesh-gate-logs/`, puis copié dans `$D` avant toute phase. `..` et lien symbolique ne passent donc pas. Hors de là : refus.
- **Ports.** Entiers de 1024 à 65535, libres à `ss -ltn`, API et base. Tout appel `curl` vise `http://127.0.0.1:$PORT_API`, `--noproxy '*'`.
- **Ne fait jamais** : nommer `kesh-mariadb-dev` ; `GRANT` ; `pkill`. Arrêt par PID et par le nom de ses propres conteneurs, `trap` de sortie, `ss -ltnp` final. Données : `Synthetique`, relevé versé dans `scripts/fixtures/recette/`.

**Phases, chacune vérifiée, code de sortie distinct** :

1. **Avant** : conteneur de base neuf ; image précédente en premier plan **par un `docker run --rm` lancé en tâche de fond, sortie vers un fichier** (le code de retour et le journal viennent du processus, et aucun `docker rm` n'est jamais nécessaire : C-e334-22) ; `/health` ; `_test/seed` `with-company` ; peuplement API, repris de `boot-phase1.sh` : factures clientes validée, payée, brouillon ; fournisseur validée, payée, annulée, brouillon ; compte bancaire et relevé CAMT synthétique ; une session ouverte ; empreinte **avant** (tables, `_sqlx_migrations`, `min_required`, API) ; `.keshbackup` ; dump ; image arrêtée (par son nom propre).
2. **Montée** : binaire neuf ; migrations `avant + N`, `N` égal à `--migrations-attendues` ; si `N > 0`, **un fichier dans `$D/backup/pre-upgrade/`** (#847), sinon aucun ; aucune erreur ni panique au journal ; empreinte **après**. Seuls les fichiers de `--diff-attendu` peuvent différer ; **tout autre écart échoue**. Crochet `--apres` : la copie, sourcée, disposant de `SQL`, `api`, `L`, pour les vérifications propres à la release (C-e334-23).
3. **Refus** (si `N > 0`) : l'image précédente sur la base migrée **sort sans répondre**, code non nul, et son journal porte `previously applied but is missing`. Si `N = 0` : ce refus n'existe pas ; le script le **dit**, et saute.
4. **Retour** : dump restauré, dans le même conteneur jetable ; image précédente relancée ; empreinte **retour** **identique à avant**, octet à octet (md5). **Limite assumée, écrite à `docs/testing.md`** : le retour joué ici passe par le dump ; la procédure du manuel, par le `.keshbackup` de `pre-upgrade/` dans une installation neuve, c'est **#920** (B4), qui reste à Guy et à une recette distincte.
5. **Import** (option `--import`) : `.keshbackup` de phase 1 dans le binaire neuf, sur le second conteneur, empreinte comparée sous `--diff-attendu`.

**Verrous** : à l'appelant, comme le reste des gates : `flock /tmp/station-heavy.lock flock <logs>/.gate.lock scripts/mem-guard.sh scripts/recette-montee.sh …`.

## Acceptance Criteria

1. **AC-1 (B1, #925)** : `scripts/gate-ci.sh` rejoue la commande exacte de `ci.yml` : la garde de T2 compare la ligne `cargo test` du script à celle du workflow et rougit si l'une bouge ; elle vérifie aussi que `ci.yml` applique toujours les migrations par sa boucle, faute de quoi le script ne l'imite plus.
2. **AC-2 — il voit ce que nextest ne voit pas**, avant T3, sans test jetable (la variable fuitée ne fait rien sans lecteur, #925) : le correctif `f5d40d7e` défait à la main (`git show f5d40d7e -- crates/kesh-api/src/config.rs | git apply -R`), `gate-ci.sh` rougit sur `kesh-api --lib` et `test-fast.sh --no-lint` non ; `git checkout -- crates/kesh-api/src/config.rs`, `git status` vide, `touch` du fichier. Point de comparaison : les 17 échecs de `rel0174/ci-repro-lib-avant.log` ; le nombre obtenu est cité au Change Log. Si le correctif ne se défait plus proprement, le dire, et demander un worktree à l'orchestrateur.
3. **AC-3** : `gate-ci.test.sh` (`docker` et `cargo` factices) : refus de base (`kesh`, `kesh_gate`, `kesh_e2e`, `root`, hors motif) et refus d'un `.env` à la racine du mini-dépôt ou dans un parent, chacun avec zéro appel ; `DATABASE_URL` et `KESH_ADMIN_BACKUP_DIR` exportés : `cargo` factice n'en reçoit aucun, sauf `DATABASE_URL` forcée, et son environnement est exactement la liste ; migrations par la boucle, aucun `sqlx` ; code de sortie de `cargo` propagé.
4. **AC-4 — `EnvGuard`** : tout l'environnement est restitué ; un test y pose et efface une `KESH_*` et la relit après le relâchement du garde.
5. **AC-5** : `recette-montee.test.sh` (`docker`, `curl`, binaire factices, mini-dépôt sous `mktemp -d`), lancé par `crates/kesh-db/tests/recette_montee_script.rs`, donc au gate **et** en CI :
   - **refus avec zéro appel `docker` et `curl`** : options `--base`, `--host`, `--database-url`, `--journal` ; image `latest`, sans version, d'un autre dépôt ; `--binaire` hors du dépôt, par chemin direct, par `..` et par lien symbolique ; `--static` et `--apres` hors de leur place ; port pris ; `.env` dans un parent de la racine jetable ; `RECETTE_RACINE_JETABLE` hors de `/tmp` ;
   - **environnement fermé** : `DATABASE_URL`, `KESH_ADMIN_BACKUP_DIR`, `KESH_SMTP_PASSWORD` exportés, et un `.env` factice dans le répertoire de l'appelant, son parent et la racine du mini-dépôt : le binaire factice consigne `env` et `pwd` : la liste est **exacte**, tous les chemins sous `$D`, `pwd` égal à `$D/cwd`, `DATABASE_URL` à `127.0.0.1:$PORT_BASE`, aucune valeur de la sonde ;
   - **conteneurs** : au journal du `docker` factice, le conteneur de base porte jeton, `--tmpfs`, `-p 127.0.0.1:` ; aucun `--env-file`, `-v`, `--volume`, `--mount`, ni `-e` sans `=` ; `kesh-mariadb-dev` n'apparaît jamais ; un `inspect` qui rend un autre jeton fait refuser ; `mariadb:10.11` égale `docker-compose.dev.yml` ;
   - **enchaînement** : phases dans l'ordre ; un écart d'empreinte hors `--diff-attendu` fait échouer ; une image qui **démarre** sur la base migrée fait échouer ; un journal sans `previously applied` fait échouer ; `N = 0` saute la phase 3 en le disant ; absence de fichier `pre-upgrade/` à `N > 0` fait échouer.
6. **AC-6 — recette réelle du script** : lancé depuis un dossier temporaire qui porte un `.env` sonde (`KESH_ADMIN_BACKUP_DIR=/inexistant/sonde`), `DATABASE_URL` exportée vers une base factice : sur cette branche (89 migrations, `N = 1` par rapport à une image de 88), `--image gcorbaz/kesh:0.17.2` (elle n'a pas la migration 89 : `rel0173` l'a joué) : les cinq phases passent, le fichier de `pre-upgrade/` est sous `$D/backup`. Puis `--image gcorbaz/kesh:0.17.4`, `--migrations-attendues 0` : le saut est dit. **`docker inspect -f '{{.State.StartedAt}}' kesh-mariadb-dev` identique avant et après**, et aucun conteneur `kesh-recette-*` ne reste. Journaux cités au Change Log : **aucune image ni base du NAS, aucune donnée réelle**. Fixtures incompatibles avec la 0.17.2 : le dire, et conditionner cette partie.
7. **AC-7** : décidable, et ne dépend d'aucune liste de refus, puisque les scripts n'en portent pas : sur `scripts/recette-montee.sh`, `scripts/gate-ci.sh`, `scripts/fixtures/recette/`, `/usr/bin/grep -rniE '/home/|\$\{?HOME\}?/|~/|/volume1|gvfs|kesh-test|192\.168\.|\.local\b|\bnas\b'` ne rend **rien**, et `/usr/bin/grep -rn gcorbaz` ne rend que `gcorbaz/kesh:`. Le test l'exécute, avec une sonde sur une ligne factice ; relevé CAMT relu.
8. **AC-8** : documentation, ci-dessous, et **une ligne que l'orchestrateur reprendra** au prompt d'intégration de la 0.17.5 : « avant la PR de release, `scripts/gate-ci.sh kesh_ci_rel0175` et, la release portant une migration, `scripts/recette-montee.sh`, chacun sous les verrous ».

## Tâches

- [ ] **T1 — rouges** : `scripts/tests/gate-ci.test.sh`, `scripts/tests/recette-montee.test.sh`, enveloppes Rust, sur le modèle cité (mini-dépôt sous `mktemp -d`, nettoyé par son `trap`, comme le test existant).
- [ ] **T2** : `gate-ci.sh` ; garde `ci.yml` ⇄ script ; AC-2, **avant** T3.
- [ ] **T3** : `EnvGuard`, `config.rs` (`#[cfg(test)]`) ; test.
- [ ] **T4** : `recette-montee.sh`, `scripts/fixtures/recette/` (relevés A et B, relus).
- [ ] **T5** : AC-6 ; journaux dans `kesh-gate-logs/e3310/`.
- [ ] **T6** : documentation ; CHANGELOG `[0.17.5]` : **uniquement** une ligne *Interne* si la section la tolère ; sinon rien : aucun effet utilisateur (C-e334-25).

### Mutations nommées

P3 : pas d'exigence, mais les gardes se prouvent. **M1** : élargir le motif à `^kesh_[a-z0-9_]+$` → rouge AC-3. **M2** : ignorer `--diff-attendu` (tout accepter) → rouge AC-5. **M3** : ne plus exiger `previously applied` → rouge AC-5. **M4** : changer `-j1` en `-j2` dans le script → rouge AC-1. **M5** : `EnvGuard` limité à `DATABASE_URL` → rouge AC-4. **M6** : retirer `env -i` du lancement du binaire → rouge AC-5 (environnement). **M7** : ne plus comparer `realpath` (lien symbolique admis) → rouge AC-5. **M8** : accepter `--base` → rouge AC-5. **M9** : ajouter `-v` au `docker run` de l'image → rouge AC-5. **M10** : `sqlx migrate run` à la place de la boucle → rouge AC-3.

## Gates

`scripts/test-fast.sh` (enveloppes comprises), `scripts/gate-ci.sh kesh_ci_e3310` lui-même, une fois au moins, en grandeur : c'est sa recette. `shellcheck` si disponible : à déclarer. E2E : non touché par cette story, mais le dernier commit de code passe par la règle : à lancer si le frontend ou le backend change hors `#[cfg(test)]`, à déclarer sinon.

## Documentation

- `CLAUDE.md`, **des phrases factuelles, aucune règle** (L-F10-1, Q3 sans réponse) : § *Test Locally First*, `:122` : nextest n'est pas le même mode d'exécution que la CI, et `scripts/gate-ci.sh` rejoue la commande de la CI ; liste pré-release : `scripts/recette-montee.sh` joue la montée et le retour.
- `docs/ci.md:90` : le script, durée mesurée. `docs/testing.md` : les deux scripts et la limite de la phase 4 (autre section que 33-9). `docs/agents-orchestration.md` § verrous : l'exemple.
- `docs/manual/fr/admin-manual.tex` : **aucun changement** ; c'est un outil de développement. À contrôler : il ne le cite pas.

## Recoupements avec les autres fiches

**33-1a** doit savoir que `--diff-attendu` et `--apres` existent : ses vérifications (numérotation par date, libellés passés intacts) vont au crochet de la release, non au script. `CLAUDE.md`, `docs/testing.md` avec **33-9** (paragraphes distincts). Aucun code de production partagé.

## Hors périmètre

#920 (procédure de retour du manuel, jouée telle qu'écrite : B4, hôte de Guy) ; passer la CI à nextest (écarté : la CI est la référence, et le sériel est ce qui attrape l'état global) ; les autres actions B ; automatiser l'E2E ou la recette visuelle.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Deux MEDIUM, six LOW, prémisses lues au code, aucun réfuté. **M-R10-1** (arbitrage de l'orchestrateur ; lu : `main.rs:60`, `boot-lib.sh`) : `env -i` et liste exacte, dossier et conteneur de base créés par le script, liste blanche, options `--base` et `--journal` retirées ; `kesh-mariadb-dev` n'est plus employé par la recette. **Conception changée.** Lecture de l'arbitrage à contrôler en passe 2 : « tout chemin hors de son dossier jetable » vise ce que Kesh et le script écrivent ou reçoivent en configuration ; les trois entrées en lecture seule (`--binaire`, `--static`, `--apres`) restent permises, à leur place : copier un binaire de plusieurs centaines de Mo dans `/tmp`, un tmpfs ici, coûterait de la mémoire pour rien. **M-R10-2** : AC-7 décidable, la liste de refus n'existant plus. **L-R10-3** : boucle de la CI (lu : `ci.yml:178-193`). **L-R10-4** : AC-2 sans test jetable, par le retrait de `f5d40d7e` (`git show --stat` : `config.rs` seul). **L-R10-5** : préfixe `kesh_ci_`, couvert par `kesh%` (lu). **L-F10-1** : aucune règle au `CLAUDE.md`. **L-F10-2** : mention de B2 retirée (B2 est corrigée au fichier d'epic). **L-F10-3** : lieu de la rétrospective dit. Aucune compilation ni exécution : passe 2 requise.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
