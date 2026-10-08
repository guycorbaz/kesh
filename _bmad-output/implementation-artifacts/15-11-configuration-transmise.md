# Story 15.11 : La configuration écrite dans `.env` atteint Kesh — compose de production complets, et un test qui empêche l'écart de se reformer

Status: ready-for-dev

<!-- Créée le 2026-10-08 par l'agent de spécification, en autonomie (consignes de l'Epic 15), sur
     l'issue #550 — relevée par la validation P5 de la Story 15-7b2 (finding F5-1), élargie par
     l'orchestrateur. Choix applicables : C71, C72, C73 (`epic-15-choix-autonomes.md`).
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation — il
     n'existe pas de statut « en validation » dans `sprint-status.yaml`. Le développement attend la
     clôture de la boucle de validation. -->

**Issues** : **ferme #550** (les compose de production ne transmettent pas au conteneur une partie de
la configuration que `.env.example` documente et que le code lit). La PR porte `closes #550` (mot-clé
**dans la PR** : le dépôt merge en squash). **`refs #534`** : la sortie de la démonstration exige
`KESH_PRODUCTION_RESET`, que cette story rend enfin transmissible ; le libellé du dialogue et le
message 403 qui accuse le rôle (objet de #534) **ne sont pas traités ici**.

**Liée à la 15-7b2** (worktree `/home/gcorbaz/devel/kesh-15-7`, fiche `15-7b2-remise-a-zero.md`, choix
C-15-7-48) : sa recette — « poser `KESH_PRODUCTION_RESET`, redémarrer, réinitialiser, retirer » — **ne
peut pas réussir** sur une installation standard tant que cette story n'est pas livrée : la variable
posée dans `.env` n'atteint pas le conteneur. **Aucune dépendance de code** entre les deux stories ;
deux zones de texte partagées (voir § *Coordination avec la 15-7b2*).

## Story

En tant qu'**exploitant d'une installation Kesh sous Docker Compose**,
je veux que **chaque variable que Kesh lit et que `.env.example` me propose de régler atteigne
réellement le conteneur**, et qu'un test empêche qu'une variable nouvelle soit oubliée,
afin de **pouvoir activer l'envoi d'e-mails, sortir de la démonstration, choisir la langue et régler
les plafonds** — au lieu de modifier `.env` sans aucun effet ni aucun message.

## Le défaut, établi au sol le 2026-10-08

Les deux compose distribués (`docker-compose.yml`, `docker-compose.prod.yml`) n'ont pas d'`env_file` :
le service `kesh-api` ne reçoit **que** les variables listées une à une sous `environment:`. Docker
Compose lit bien `.env`, mais **seulement pour interpoler** les `${…}` du fichier compose. Une variable
posée dans `.env` sans ligne correspondante sous `environment:` **n'atteint pas le conteneur**, et
rien ne le signale. Le manuel dit le contraire (`admin-manual.tex:648` : « Le fichier `.env` est chargé
automatiquement par Docker Compose »).

Vérifié par `docker compose config` dans un répertoire d'essai (2026-10-08, Docker Compose 2.40.3) —
et c'est ce qui fonde le choix C71 :

| forme sous `environment:` | `.env` | reçu par le conteneur |
|---|---|---|
| `FOO:` (clé sans valeur) | `FOO=depuis_env` | `FOO=depuis_env` |
| `BAR:` (clé sans valeur) | absente | **rien** — la variable n'existe pas dans le conteneur (`env` vérifié par `docker compose run --rm … env`) |
| `BAZ: ${BAZ:-}` | absente | `BAZ=` (**chaîne vide**) |
| `QUX: ${QUX:-12}` | `QUX=` (vide) | `QUX=12` (`:-` traite le vide comme absent) |

**Second défaut, trouvé en établissant le premier** : `docker-compose.yml` n'a **pas d'`image:`**, seulement
`build: context: .` — or le manuel (`admin-manual.tex:218-224`, § *Installation, Étape 2*) fait
télécharger ce seul fichier dans un répertoire vide. Vérifié : `docker compose build kesh-api` y échoue
sur `failed to read dockerfile: open Dockerfile: no such file or directory`. **La procédure d'installation
générique du manuel ne fonctionne pas telle qu'écrite.** Traité ici (AC5, choix C73) : c'est le fichier
que l'exploitant doit justement re-télécharger pour recevoir la correction.

## Inventaire fermé — refait depuis le code le 2026-10-08

**Méthode** (à refaire par le test, AC8 — la table ci-dessous n'est **pas** la source du test, elle en
est l'attendu au jour de la spécification) : toute lecture de l'environnement **dans le code de
production** du workspace (`crates/*/src/**/*.rs`, hors items `#[cfg(test)]`). Relevé :
`env::var("…")` littéral (`config.rs`, `main.rs`), et quatre **indirections** à argument non littéral,
chacune résolue par ses appels littéraux :

| site non résolu | résolu par |
|---|---|
| `config.rs` `opt_trimmed_env(var)` (`:1367`) | ses appels `opt_trimmed_env("KESH_…")` |
| `config.rs` `parse_strict_bool(var, …)` (`:1395`) | ses appels `parse_strict_bool("KESH_…", …)` |
| `routes/onboarding.rs` `env_flag_enabled(name)` (`:45`) | son appel `env_flag_enabled("KESH_PRODUCTION_RESET")` (`:276`) |
| `logging.rs:130` `std::env::var(EnvFilter::DEFAULT_ENV)` | constante de `tracing-subscriber` : `RUST_LOG` |

Aucun `env::vars()`, `var_os`, `envy`, `option_env!` de configuration ; `dotenvy::dotenv()` (`main.rs:44`)
charge un `.env` du répertoire courant — **absent de l'image** (`/app`), sans effet en conteneur. Hors
`src/` : `kesh-db` lit `DATABASE_URL` dans ses tests seulement ; le frontend est un SPA statique, sans
lecture d'environnement à l'exécution (`grep` de `$env/`, `import.meta.env`, `process.env` sur
`frontend/src` : 0).

**Résultat : 41 variables lues** (39 `KESH_*`, plus `DATABASE_URL` et `RUST_LOG`). Croisées avec
`.env.example` et les blocs `environment:` des trois compose (Y = `docker-compose.yml`,
P = `docker-compose.prod.yml`, D = `docker-compose.dev.yml`) :

| classe | variables | Y | P | D |
|---|---|---|---|---|
| **transmise** par les deux (22) | `DATABASE_URL`, `RUST_LOG`, `KESH_PORT`, `KESH_HOST`, `KESH_ADMIN_USERNAME`, `KESH_ADMIN_PASSWORD`, `KESH_JWT_SECRET`, `KESH_JWT_EXPIRY_MINUTES`, `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS`, `KESH_REFRESH_INACTIVITY_MINUTES`, `KESH_RATE_LIMIT_WINDOW_MINUTES`, `KESH_RATE_LIMIT_MAX_ATTEMPTS`, `KESH_RATE_LIMIT_BLOCK_MINUTES`, `KESH_LOG_FILE_PATH`, `KESH_LOG_FILE_ROTATION`, `KESH_LOG_FILE_MAX_FILES`, `KESH_LOG_FILE_FORMAT`, `KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR`, `KESH_INBOX_MAX_FILE_BYTES`, `KESH_INBOX_MAX_FILES_PER_RUN`, `KESH_INBOX_MAX_PDF_PAGES` | ✓ | ✓ | 11 sur 22 |
| **à transmettre — par un seul** (4) | `KESH_COOKIE_SECURE` | ✓ | **✗** | ✗ |
| | `KESH_LANG`, `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB` | **✗** | ✓ | ✗ |
| **à transmettre — par aucun** (12) | `KESH_SMTP_HOST`, `KESH_SMTP_PORT`, `KESH_SMTP_USER`, `KESH_SMTP_PASSWORD`, `KESH_SMTP_FROM`, `KESH_SMTP_TLS`, `KESH_PUBLIC_BASE_URL`, `KESH_FEATURE_FORGOT_PASSWORD`, `KESH_PRODUCTION_RESET`, `KESH_ADMIN_BACKUP_DIR`, `KESH_ADMIN_EXPORT_INMEM_MB`, `KESH_ADMIN_IMPORT_MAX_MB` | ✗ | ✗ | `KESH_PRODUCTION_RESET` seule |
| **fixée par l'image** (2) | `KESH_STATIC_DIR` (`/app/static`), `KESH_LOCALES_DIR` (`/app/locales`) — `Dockerfile` `ENV`, l. 45-46 ; les transmettre écraserait le chemin de l'image par une valeur d'hôte (`frontend/build`, `locales`) et casserait le service du SPA ou le chargement des traductions | ✗ | ✗ | ✗ |
| **interdite en production** (1) | `KESH_TEST_MODE` — expose `/api/v1/_test/*` (remise à zéro et amorçage) ; refusée au démarrage avec un bind non loopback, mais ne doit **jamais** être transmissible par un compose distribué | ✗ | ✗ | ✗ |

Total : 22 + 4 + 12 + 2 + 1 = **41**. **À ajouter : 16 variables**, soit **28 lignes** (12 × 2 + 1 dans P +
3 dans Y).

**Variables de `.env.example` que le code ne lit pas** (7) — légitimes, et le test les contrôle :

| classe | variables | usage vérifié |
|---|---|---|
| **hôte seulement** (3) | `KESH_INBOX_HOST_DIR`, `KESH_DOCUMENTS_HOST_DIR`, `KESH_LOG_HOST_DIR` | `volumes:` de `docker-compose.yml` (`${KESH_LOG_HOST_DIR:-./log}` …). ⚠️ **`docker-compose.prod.yml` les ignore** : chemins figés `./log`, `./inbox`, `./documents` — même classe de défaut (réglage documenté, sans effet), traité ici (AC4, C73) |
| **service MariaDB** (4) | `MARIADB_ROOT_PASSWORD`, `MARIADB_DATABASE`, `MARIADB_USER`, `MARIADB_PASSWORD` | service `mariadb` et `DATABASE_URL` de `docker-compose.yml` ; sans objet pour P (MariaDB fournie par l'exploitant) |

**Variable fantôme** (1) : `KESH_ADMIN_RESET` — **lue nulle part**, mais nommée comme recours
« break-glass » dans deux messages **émis à l'exécution** (`config.rs:158`, message de refus de démarrage
`IncompleteSmtpConfig` ; `main.rs:337`, journal d'information au démarrage), dans `.env.example:235` et
dans deux commentaires (`config.rs:319`, `lib.rs:1040`). Le vrai recours est le couple
`KESH_ADMIN_USERNAME` / `KESH_ADMIN_PASSWORD` (break-glass, `config.rs:177`). Un exploitant qui suit le
message pose une variable sans effet. **Obsolète à retirer** (AC7).

**Documentées dans `.env.example` en prose seulement, sans ligne d'affectation** (2) : `KESH_HOST`
(« volontairement non défini ici — le compose prod le force à 0.0.0.0 ») et `KESH_TEST_MODE`
(« volontairement absent de ce template »). Conservées telles quelles ; exceptions nommées du test.

**Hors périmètre du test, écrit** : `docker-compose.dev.yml` (pile d'intégration de développement, non
distribuée aux exploitants : `README.md:77`) — il est inventorié ci-dessus, pas contraint.

## Acceptance Criteria

1. **Inventaire.** La table § *Inventaire fermé* est reproduite au Dev Agent Record **telle que le test
   la recalcule** (AC8) au dernier commit de code : 41 lues, 22 + 4 + 12 transmises après la story
   (38), 2 fixées par l'image, 1 interdite ; 3 hôte seulement ; 4 MariaDB ; 0 fantôme. Tout écart avec
   la table de cette fiche (variable ajoutée par une story mergée entre-temps) est **écrit**, non
   corrigé en silence.

2. **Transmission.** Le service `kesh-api` de `docker-compose.yml` **et** celui de
   `docker-compose.prod.yml` portent, sous `environment:`, **chacune des 38 variables** des classes
   « transmise » et « à transmettre ». Les **16 ajoutées** le sont sous la forme **clé sans valeur**
   (`KESH_SMTP_HOST:`), qui reprend la valeur de `.env` (ou de l'environnement du shell) si elle y est,
   et **n'existe pas** dans le conteneur sinon — le défaut du code s'applique alors, sans être recopié
   dans le compose (choix C71). Les entrées existantes ne changent pas de forme (leurs défauts
   `${X:-v}` sont des choix de déploiement, ex. `KESH_LOG_FILE_PATH`, `KESH_PORT`). **Aucun
   `env_file:`** sur `kesh-api` (C71). Chaque groupe ajouté porte un commentaire d'une ligne qui dit ce
   qu'il règle et renvoie au manuel ; un commentaire en tête du bloc `environment:` dit la règle :
   *« Seules les variables listées ici atteignent Kesh. Une variable ajoutée au code doit l'être ici —
   test `configuration_transmise`. Après une modification de `.env` : `docker compose up -d` (un
   `restart` ne relit pas `.env`). »*

3. **Exceptions.** `KESH_STATIC_DIR` et `KESH_LOCALES_DIR` ne sont transmises par **aucun** des deux
   compose, et le `Dockerfile` (étape finale) les fixe par `ENV`. `KESH_TEST_MODE` n'est transmise par
   **aucun** des deux compose. Ces trois exceptions sont les **seules** variables lues non transmises.

4. **Chemins d'hôte honorés par les deux compose.** Dans `docker-compose.prod.yml`, les trois montages
   deviennent `${KESH_LOG_HOST_DIR:-./log}:/var/log/kesh`, `${KESH_INBOX_HOST_DIR:-./inbox}:/data/inbox`,
   `${KESH_DOCUMENTS_HOST_DIR:-./documents}:/data/documents` — **défauts identiques** aux chemins figés
   actuels : une installation dont `.env` ne pose pas ces variables ne voit aucun changement (C73).

5. **`docker-compose.yml` s'installe seul.** Le service `kesh-api` reçoit
   `image: gcorbaz/kesh:latest`, **`build:` conservé** (la règle de la spécification Compose : avec les
   deux et sans `pull_policy`, Compose tire l'image et ne construit que si elle est introuvable). Preuve
   (T6) : dans un répertoire vide ne contenant que ce fichier et un `.env` d'essai,
   `docker compose --dry-run up -d` annonce le **tirage** de `gcorbaz/kesh:latest` et **aucune
   construction** ; le même essai **avant** la modification annonce une construction (ou échoue sur le
   `Dockerfile` absent) — les deux sorties au Dev Agent Record. Si le comportement observé diffère de
   la règle citée, **s'arrêter et l'écrire** (ne pas ajouter de `pull_policy` sans mesure).

6. **`.env.example`.**
   (a) Chacune des 41 variables lues y figure en ligne d'affectation (commentée ou non), **sauf**
   `KESH_HOST` et `KESH_TEST_MODE` (prose, inchangée) ; aujourd'hui c'est déjà le cas — l'AC fixe
   l'invariant que le test tient.
   (b) Aucune ligne d'affectation n'y nomme une variable que ni le code, ni un `volumes:`, ni le service
   MariaDB n'utilise.
   (c) Le bloc « Chemins internes » (`:152-156`) dit que `KESH_STATIC_DIR` et `KESH_LOCALES_DIR` sont
   **fixées par l'image** (`/app/static`, `/app/locales`), **ignorées par les compose fournis**, et ne
   servent qu'à un `cargo run` hors Docker.
   (d) L'en-tête dit la règle de l'AC2 (seules les variables listées dans le compose atteignent Kesh ;
   `docker compose up -d` après modification).
   (e) `KESH_ADMIN_RESET` (`:235`) est remplacé par le vrai recours (`KESH_ADMIN_USERNAME` /
   `KESH_ADMIN_PASSWORD`, section « Compte admin initial »).
   (f) ⛔ Le bloc `KESH_PRODUCTION_RESET` (`:272-279`) **n'est pas réécrit** ici : la 15-7b2 en est
   propriétaire (son AC 10). Seule exception : si la 15-7b2 est déjà mergée au moment du développement,
   vérifier que son texte ne contredit pas la transmission ; sinon, rien.

7. **Le fantôme `KESH_ADMIN_RESET` disparaît du code.** `config.rs:158` (texte de
   `ConfigError::IncompleteSmtpConfig`), `main.rs:337` (journal), `config.rs:319` et `lib.rs:1040`
   (commentaires) nomment le vrai recours : *break-glass `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD`*.
   Grep de contrôle à zéro sur tout le dépôt hors `_bmad-output/` et hors `CHANGELOG.md` (historique) :
   `grep -rn 'KESH_ADMIN_RESET' --exclude-dir={target,node_modules,_bmad-output,.git} .`.

8. **Le test garde-fou** — `crates/kesh-api/tests/configuration_transmise.rs`, sans base de données,
   exécuté par `cargo test --workspace` (CI) et `scripts/test-fast.sh` (local). Il **recalcule** tout
   depuis les fichiers du dépôt (racine = `CARGO_MANIFEST_DIR/../..`) ; ses seules listes écrites à la
   main sont les **exceptions justifiées**, chacune avec sa raison en commentaire et un **contrôle de
   cette raison**. Assertions :
   - **(L) Lectures** — analyse par `syn` 2 (`parse_file`, `visit`) de chaque `crates/*/src/**/*.rs`,
     items `#[cfg(test)]` exclus. Est un **site** tout appel dont le chemin se termine par `env::var` ou
     `env::var_os`, et tout appel à `env::vars`/`vars_os`. Argument littéral → nom lu. Sinon → **site non
     résolu**, qui doit figurer dans la liste fermée `INDIRECTIONS` (fichier + fonction englobante →
     résolution : « appels littéraux de la fonction `f` » ou « nom fixe `RUST_LOG` ») ; un site non
     résolu absent de la liste **rougit** ; une entrée de la liste qui ne correspond plus à aucun site
     **rougit** (liste périmée) ; un appel non littéral à une fonction d'indirection **rougit**. Toute
     **macro** (`syn::Macro`) du code de production dont le flux de jetons contient `env :: var`
     **rougit** (`syn` ne voit pas l'intérieur des macros : faux rouge possible, faux vert jamais —
     limite écrite). Garde contre le test muet : l'ensemble lu contient au moins `DATABASE_URL`,
     `KESH_JWT_SECRET`, `KESH_SMTP_HOST` et `KESH_PRODUCTION_RESET`, et compte **au moins 41** noms.
   - **(T) Transmission** — `docker-compose.yml` et `docker-compose.prod.yml` analysés par un vrai
     analyseur YAML (dépendance de test, C72) : service `kesh-api`, clé `environment`, forme
     dictionnaire **ou** liste (`KEY=VAL`, `KEY`). Pour **chaque** variable lue : présente dans les deux,
     **ou** dans `EXCEPTIONS` — `FixeeParImage` (contrôle : le `Dockerfile` porte `ENV <nom>=` après le
     dernier `FROM`, **et** la variable est absente des deux compose) ou `InterditeEnProduction`
     (contrôle : absente des deux compose). Toute clé transmise est une variable lue (une faute de
     frappe rougit). `env_file` sur `kesh-api` **rougit**, avec un message qui renvoie à C71.
   - **(V) Valeurs** — dans les deux compose, la valeur d'une variable transmise est : absente (clé
     sans valeur), un littéral non vide, ou une interpolation `${NOM}`, `${NOM:?…}`, `${NOM:-défaut
     non vide}` dont **`NOM` est la clé elle-même** — sauf `DATABASE_URL` de `docker-compose.yml`
     (composée de `MARIADB_*` : exception nommée). `${NOM:-}` (défaut vide) **rougit** : il transmet
     une chaîne vide, que plusieurs lecteurs ne traitent pas comme une absence (`KESH_ADMIN_BACKUP_DIR`
     → chemin vide ; `KESH_SMTP_PORT` → avertissement « invalide » à chaque démarrage).
   - **(E) `.env.example`** — chaque ligne d'affectation (`^#?\s*NOM=`) nomme une variable lue, ou
     d'hôte (contrôle : `${NOM` apparaît dans un `volumes:` des **deux** compose), ou MariaDB (contrôle :
     `${NOM` apparaît dans `docker-compose.yml`). Chaque variable lue y a une ligne d'affectation, sauf
     `KESH_HOST` et `KESH_TEST_MODE` (exceptions nommées, contrôle : présentes dans la prose).
   - **(F) Fantômes** — tout jeton `KESH_[A-Z0-9_]*[A-Z0-9]` de `.env.example`, des deux compose, des
     `messages.ftl` des **4 locales**, des chaînes littérales du code de production (`syn::LitStr`, ce
     qui inclut les doc-comments), et des `docs/manual/fr/*.tex` (après `\_` → `_`), est une variable
     lue ou d'hôte. Un jeton **immédiatement suivi de `_`** (`KESH_SMTP_*`, `KESH\_ADMIN\_*`,
     `KESH_PAT_`) est un **préfixe** : il doit préfixer au moins une variable lue ou d'hôte, ou figurer
     dans `PREFIXES_HORS_VARIABLES` (aujourd'hui `KESH_PAT_`, préfixe des clés d'API — contrôle : le
     code de production contient la chaîne `KESH_PAT_` ou `kesh_pat`).
   - **(S) Auto-test des extracteurs** sur sources synthétiques : lecture littérale trouvée ; nom en
     commentaire, en chaîne d'un autre appel, dans un `#[cfg(test)] mod` → non lu ; appel dans une macro
     → rouge ; YAML en forme liste et dictionnaire → mêmes clés ; `${A:-}` → rouge ; `${B}` sous la clé
     `A` → rouge.

9. **Mutations** (T5) — chacune appliquée, test exécuté, **rouge constaté avec le message attendu**,
   puis restaurée ; résultat au Dev Agent Record :
   M1 retirer `KESH_SMTP_HOST` de `docker-compose.prod.yml` · M2 le retirer de `docker-compose.yml`
   seul · M3 ajouter `env::var("KESH_ESSAI_MUTATION")` dans une fonction de production de `config.rs` ·
   M4 ajouter `fn lire(n: &str) -> Option<String> { std::env::var(n).ok() }` hors test · M5 ajouter
   `opt_trimmed_env("KESH_ESSAI_MUTATION")` · M6 ajouter `tracing::info!("{:?}", std::env::var("X"))` ·
   M7 transmettre `KESH_STATIC_DIR` dans `docker-compose.prod.yml` · M8 retirer `ENV KESH_LOCALES_DIR`
   du `Dockerfile` · M9 ajouter `KESH_TEST_MODE:` à `docker-compose.yml` · M10 ajouter
   `#KESH_OBSOLETE=1` à `.env.example` · M11 remettre `KESH_ADMIN_RESET` dans le message de
   `main.rs:337` · M12 `KESH_SMTP_PORT: ${KESH_SMTP_PORT:-}` · M13 `KESH_SMTP_HOST: ${KESH_SMTP_HSOT}` ·
   M14 `env_file: .env` sur `kesh-api` · M15 retirer `KESH_LANG=fr` de `.env.example` ·
   **M16 (vert attendu)** ajouter `std::env::var("KESH_FAUX")` dans un `#[cfg(test)] mod tests` de
   production → le test **reste vert**. Les sources `.rs` sont lues **à l'exécution** du test, pas
   compilées dedans : une mutation restaurée ne laisse pas de binaire périmé, mais M3-M6 et M11
   recompilent `kesh-api` (le fichier muté appartient au crate) — vérifier que la restauration est
   suivie d'un `touch` avant le run suivant.

10. **Vérification par `docker compose config`, sans démarrer** (T6). Dans un répertoire d'essai du
    scratchpad, avec un `.env` qui pose **toutes** les variables d'`.env.example` à une valeur
    reconnaissable : `docker compose -f <fichier> config --format json` pour les deux compose ; le
    service `kesh-api` y porte les 38 variables avec la valeur posée ; avec un `.env` **vide** (hors
    variables obligatoires), les variables ajoutées **n'apparaissent pas** (clé sans valeur → absente). Attendu, à recompter : 38 / 38 avec
    `.env` complet ; sans `.env`, **23** dans `docker-compose.yml` (22 + `KESH_COOKIE_SECURE`, qui y est déjà)
    et **25** dans `docker-compose.prod.yml` (22 + `KESH_LANG`, `KESH_PASSWORD_MIN_LENGTH`,
    `KESH_BANK_IMPORT_MAX_MB`) — 15 lignes ajoutées à l'un, 13 à l'autre. Comptes au Dev Agent Record.
    `docker-compose.prod.yml` exige le réseau externe `frontend` au démarrage, pas à `config` :
    aucune création de réseau n'est nécessaire. **Aucun `up`, aucun conteneur** (sauf le `--dry-run`
    de l'AC5, qui n'en crée pas).

11. **CI.** Le job `docker-build` de `.github/workflows/ci.yml` gagne une étape
    `docker compose -f docker-compose.yml config -q` et `docker compose -f docker-compose.prod.yml config -q`
    (validation syntaxique par Compose lui-même, que l'analyseur YAML du test ne remplace pas : clé
    inconnue, interpolation mal formée). Variables obligatoires non posées : avertissement de Compose,
    pas d'échec — à vérifier en local avant le push (T6) ; si `config -q` échoue sur une variable
    absente, poser un `.env` d'essai minimal dans l'étape.

12. **Manuel d'administration** (`docs/manual/fr/admin-manual.tex` + PDF régénéré) :
    (a) `:648` — la phrase « Le fichier `.env` est chargé automatiquement par Docker Compose » devient
    la règle exacte : Compose lit `.env` pour remplir le compose ; **seules les variables listées sous
    `environment:` atteignent Kesh** ; les compose fournis les listent toutes, sauf `KESH_STATIC_DIR` et
    `KESH_LOCALES_DIR` (fixées par l'image) et `KESH_TEST_MODE` (jamais en production) ; une
    modification de `.env` s'applique par `docker compose up -d` (**pas** `docker compose restart`).
    (b) `:678-679` — défauts de `KESH_STATIC_DIR` / `KESH_LOCALES_DIR` : préciser « fixé par l'image
    Docker (`/app/static`, `/app/locales`) ; `frontend/build` / dossier du dépôt hors Docker ».
    (c) `:782` — le tableau des chemins d'hôte vaut désormais pour **les deux** compose.
    (d) `:218-224` (*Étape 2*) — le fichier téléchargé tire l'image publiée ; une phrase.
    (e) `:236-254` (*Étape 3*) — avec `docker-compose.yml`, `DATABASE_URL` est **composée** des
    `MARIADB_*` par le compose : la ligne `DATABASE_URL` de `.env` y est ignorée (elle sert
    `docker-compose.prod.yml`). Une phrase ; l'exemple garde sa ligne, cohérente.
    (f) `:1704-1714` (*Procédure de mise à jour standard*), point 3 — remplacer « si nécessaire » par
    une consigne datée : **pour passer à 0.13.0, re-télécharger le compose** (`docker-compose.yml` ou,
    sur Synology, `docker-compose.prod.yml` enregistré sous `docker-compose.yml`, comme à l'installation)
    **et y reporter ses adaptations locales** (mapping de port `8080:80` recommandé par le manuel,
    réseau) — ou, à défaut, ajouter à la main les lignes manquantes ; vérifier par
    `docker compose config` ; puis `docker compose up -d`. Vérification fonctionnelle :
    `curl …/health` → `smtpConfigured: true` si le SMTP est posé.
    (g) PDF régénéré (`make fr` dans `docs/manual/`), contrôlé **aplati** :
    `pdftotext admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` contient les phrases nouvelles de (a) et
    (f) ; aucun `Overfull \hbox` **nouveau** dans le `.log` (comparer au `.log` d'avant).
    (h) `user-manual.tex` : aucune modification attendue — **à vérifier** par grep (`SMTP`,
    `KESH\\_`, `démonstration`) et à écrire au Dev Agent Record.

13. **CHANGELOG `[0.13.0]`**, section **Corrigé** : une entrée « **La configuration posée dans `.env`
    atteint enfin Kesh (#550)** » — ce qui était sans effet (e-mails, mot de passe oublié, sortie de la
    démonstration, langue, longueur des mots de passe, plafonds d'import et de sauvegarde, cookies
    `Secure` sur `docker-compose.prod.yml`, chemins d'hôte sur `docker-compose.prod.yml`), le message
    de démarrage qui renvoyait à une variable inexistante, l'installation par `docker-compose.yml` seul ;
    puis **⚠️ Action requise sur une installation existante** : re-télécharger le compose et y reporter
    ses adaptations, `docker compose up -d` — **la nouvelle image seule ne corrige rien**, le défaut est
    dans le fichier compose que l'exploitant détient.

14. **Gates.** `kesh-api` touché (messages, test neuf) : gate ciblé
    (`cargo nextest run -E 'binary(configuration_transmise)'` + `fmt` + `clippy` workspace) entre les
    passes ; **gate complet** au dernier commit de code (`scripts/test-fast.sh`, base remise à zéro
    avant — `DROP`/`CREATE` de ses seules bases, jamais de redémarrage du conteneur MariaDB) ;
    frontend non touché (`npm run check` et `test:unit` non requis — l'écrire) ; **E2E complet au
    dernier commit de code** (D7), jugé fichier par fichier contre `docs/testing.md` § *Les échecs
    attendus*. Ne déclarer que ce qui a tourné.

## Tasks / Subtasks

- [ ] **T0 — Relevé** (AC1) : refaire l'inventaire à la main depuis `main` à jour (les greps de la § *Inventaire*),
  comparer à la table de cette fiche, écrire l'écart. Vérifier que la 15-5e1 n'a pas déjà ajouté `syn` aux
  `[dev-dependencies]` de `kesh-api` (C70) ; si oui, réutiliser sa ligne.
- [ ] **T1 — Test d'abord, rouge** (AC8) : écrire `configuration_transmise.rs` (L, T, V, E, F, S) ; dépendances de
  test `syn = { version = "2", features = ["full", "visit"] }` et l'analyseur YAML retenu (C72). Constater le
  **rouge attendu** sur l'état actuel : (T) nomme les 16 variables manquantes, (E)/(F) nomment `KESH_ADMIN_RESET`,
  (T) nomme `KESH_COOKIE_SECURE` pour P et les trois de Y. Sortie au Dev Agent Record — c'est la preuve que le
  test voit le défaut de #550.
- [ ] **T2 — Compose** (AC2, AC3, AC4, AC5) : les 28 lignes, les commentaires, `image:` dans Y, les trois montages de P.
- [ ] **T3 — `.env.example`** (AC6) et **code** (AC7) : 4 sites `KESH_ADMIN_RESET`, bloc « Chemins internes », en-tête.
- [ ] **T4 — Vert** : le test passe ; gate ciblé.
- [ ] **T5 — Mutations M1-M16** (AC9), une à une, restauration vérifiée par `git diff --stat` vide.
- [ ] **T6 — Docker sans démarrer** (AC5, AC10, AC11) : `config --format json` ×2 avec `.env` complet et vide ;
  `--dry-run up -d` avant/après dans un répertoire vide ; `config -q` tel que l'étape CI le lancera. Répertoires
  d'essai dans le scratchpad, supprimés ensuite ; aucun réseau, volume ni conteneur créé (`docker ps -a`,
  `docker network ls` avant/après).
- [ ] **T7 — CI** (AC11).
- [ ] **T8 — Manuel** (AC12) puis **CHANGELOG** (AC13). Propagation : `grep -rnE 'chargé automatiquement|env_file|KESH_ADMIN_RESET|docker compose restart' docs README.md website .env.example docker-compose*.yml`
  — chaque site lu, traité ou écrit comme hors sujet.
- [ ] **T9 — Gates** (AC14) et Dev Agent Record (décomptes recomptés depuis la source, avec leur périmètre).

## Dev Notes

### Pourquoi une liste explicite et non `env_file: .env` (C71)

`env_file: .env` transmettrait **tout** `.env` au conteneur : les quatre `MARIADB_*` — dont
`MARIADB_ROOT_PASSWORD`, qui n'a rien à faire dans l'environnement de l'application (visible par
`docker inspect`, `/proc/1/environ`) —, toute variable que l'exploitant y aurait ajoutée pour un autre
usage, et les lignes décommentées par erreur : `KESH_STATIC_DIR=frontend/build` décommenté écraserait
le chemin de l'image et casserait le SPA. Il rendrait aussi le test (T) **vide de sens** : tout
passerait, y compris ce que le compose doit forcer. La liste explicite garde un contrat lisible
(« voici ce que Kesh reçoit »), les valeurs forcées (`KESH_HOST: 0.0.0.0` dans P) et les défauts de
déploiement ; son défaut — redériver quand le code gagne une variable — est exactement ce que le test
ferme. La **clé sans valeur** évite de recopier dans le compose les défauts du code (DRY) et le piège de
la chaîne vide (`${X:-}`), mesuré ci-dessus.

### Ce que lit chaque lecteur quand la valeur est vide — pourquoi (V) interdit `${X:-}`

`opt_trimmed_env` (SMTP hôte/utilisateur/mot de passe/expéditeur, `KESH_PUBLIC_BASE_URL`) et
`parse_strict_bool` (`KESH_SMTP_TLS`, `KESH_FEATURE_FORGOT_PASSWORD`) traitent le vide comme absent ;
`env_flag_enabled` (`KESH_PRODUCTION_RESET`) aussi. Mais `KESH_SMTP_PORT` (`config.rs:1070`) avertit
« invalide » puis prend 587 ; `KESH_ADMIN_BACKUP_DIR` (`:944`) prendrait un chemin **vide** ; `KESH_LANG`
(`:831`) passerait `""` à `Locale::from`. La clé sans valeur supprime la question.

### Dépendances de test (C72)

- **`syn` 2** (`full`, `visit`) : déjà au `Cargo.lock` (2.0.118, transitif) ; la 15-5e1 l'ajoute aussi
  aux `[dev-dependencies]` de `kesh-api` (C70) — conflit de fusion trivial (même ligne) selon l'ordre de
  merge.
- **Analyseur YAML** : aucun au `Cargo.lock`. Retenu : **`yaml-rust2`** (pur Rust, maintenu, sans
  `serde` — le test parcourt des nœuds, il ne désérialise pas de structure). `serde_yaml` est archivé
  par son auteur ; `serde_yml` a eu des alertes de qualité. Vérifier la dernière version au
  développement (`cargo add --dev yaml-rust2 -p kesh-api`) et l'écrire au Dev Agent Record ; si
  `cargo deny`/la CI refuse la licence ou la dépendance, revenir à cette fiche plutôt que de basculer
  sur un analyseur maison.

### Le `Dockerfile` n'est pas modifié

`ENV KESH_STATIC_DIR=/app/static` et `ENV KESH_LOCALES_DIR=/app/locales` (l. 45-46) sont la raison des
deux exceptions, et le test les contrôle (M8). Rien d'autre à y changer.

### Appliquer une modification de `.env` : `up -d`, pas `restart`

`docker compose restart` redémarre le conteneur **avec son environnement d'origine** ; seul
`docker compose up -d` recrée le conteneur quand sa configuration a changé. C'est la précision qui
manque aujourd'hui au manuel, et elle concerne directement la recette de la 15-7b2 (« poser la
variable, **redémarrer** ») : signalé à l'orchestrateur.

### Coordination avec la 15-7b2

Zones partagées : `.env.example:272-279` (bloc `KESH_PRODUCTION_RESET`, **propriété de la 15-7b2**,
non touché ici — AC6 f) et `admin-manual.tex` (la 15-7b2 réécrit la ligne `:691` et `:1314` ; cette
story touche `:648`, `:678-679`, `:782`, `:218-254`, `:1704-1714` — pas de chevauchement de lignes, mais
le PDF est régénéré par les deux : celui qui merge second **régénère** après rebase, sans résoudre le
conflit du PDF à la main). CHANGELOG : deux entrées distinctes de `[0.13.0]`, relocaliser par le texte.

### Angles morts assumés (écrits, non traités)

- Lectures d'environnement par des **dépendances** (`sqlx`, `lettre`, `reqwest`, `tracing-subscriber`
  hors `DEFAULT_ENV`, variables de proxy `HTTP_PROXY`…) : hors inventaire. Aucune n'est documentée par
  `.env.example`.
- Un `env::var` à l'intérieur d'une **macro** : rouge par (L), donc jamais muet.
- `docker-compose.dev.yml` : inventorié, non contraint.
- **Hors périmètre, à signaler à l'orchestrateur pour issue** : (i) `KESH_ADMIN_BACKUP_DIR` vaut `/tmp`
  par défaut, perdu au redémarrage du conteneur, et **aucun compose ne monte de volume** où le pointer —
  la transmission le rend réglable, pas encore utile ; (ii) `docker-compose.yml` publie MariaDB sur
  `3306:3306` (toutes interfaces) avec des mots de passe de développement par défaut ; (iii) le montage
  E2E local ne pose pas `KESH_PRODUCTION_RESET` (relevé par la 15-7b2, déjà signalé sur #97).

### Fichiers touchés

`docker-compose.yml`, `docker-compose.prod.yml`, `.env.example`, `crates/kesh-api/src/config.rs`,
`crates/kesh-api/src/main.rs`, `crates/kesh-api/src/lib.rs`, `crates/kesh-api/Cargo.toml`, `Cargo.lock`,
`crates/kesh-api/tests/configuration_transmise.rs` (neuf), `.github/workflows/ci.yml`,
`docs/manual/fr/admin-manual.tex` + `.pdf`, `CHANGELOG.md`. Aucune migration ; `kesh-db` non touché.

### Project Structure Notes

Le test vit avec les tests d'intégration de `kesh-api` (patron `audit_route_registry.rs`,
`audit_label_registry.rs` : tests qui lisent les sources du dépôt). Il ne monte ni base ni serveur.

### References

- Issue #550 ; #534 ; fiche `15-7b2-remise-a-zero.md` (worktree `kesh-15-7`), choix C-15-7-48.
- `crates/kesh-api/src/config.rs:566-1150` (`from_env`), `:1335-1343` (journal fichier), `:1367-1415`
  (helpers) ; `main.rs:44, 220, 247, 337` ; `routes/onboarding.rs:45-53, 276` ; `logging.rs:130`.
- `Dockerfile:18-48` ; `docker-compose.yml` ; `docker-compose.prod.yml` ; `docker-compose.dev.yml`.
- `admin-manual.tex:218-254, 528-545, 646-793, 1704-1714`.
- CLAUDE.md § *Review Iteration Rule* (inventorier les sites non résolus), § *Synchroniser TOUTES les
  docs*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — Création (agent de spécification, autonomie). Inventaire refait depuis le code : 41
  variables lues, 16 à transmettre (28 lignes), 2 fixées par l'image, 1 interdite, 3 d'hôte (ignorées
  par `docker-compose.prod.yml`), 4 MariaDB, 1 fantôme (`KESH_ADMIN_RESET`). Défaut connexe établi :
  `docker-compose.yml` sans `image:` (installation du manuel en échec, vérifié). Choix C71, C72, C73.
