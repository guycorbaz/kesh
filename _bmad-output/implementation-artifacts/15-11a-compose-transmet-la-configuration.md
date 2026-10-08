# Story 15.11a : Les compose de production transmettent toute la configuration écrite dans `.env` — et un test qui compare les compose à la liste des variables lues

Status: review

<!-- Créée le 2026-10-08 par l'agent de découpage, en autonomie (consignes de l'Epic 15), par découpage
     de la Story 15-11 après sa validation P3 (signal D5 levé deux fois, par recyclage — choix C77).
     Elle reprend le volet « compose, `.env.example`, documentation, CI » de la 15-11 et la
     remédiation P3 qui y tombe (R3-1 = F-1, R3-2, R3-4, R3-5, R3-6, R3-8 = F-7, R3-9, F-5, F-6, F-8,
     F-10). Choix applicables : C71 (liste explicite, pas d'`env_file`), C72 (test Rust, `yaml-rust2`),
     C73 (périmètre), C75 (forme `${KESH_X:-}` ; sa partie « fonction de lecture unique » est à la
     15-11b), C76 (`KESH_ADMIN_PASSWORD` sans défaut, liste « relisez » fermée), C77 (découpage).
     Version complète de la 15-11 avant découpage : commit d69fdcca.
     Validation P1 de la 15-11a remédiée le 2026-10-08 (choix C79) : troisième source de la liste
     « relisez » (chemins d'hôte de P), garde du placeholder `GENERATE_ME` du secret JWT (#557, AC16).
     Validation P2 de la 15-11a remédiée le 2026-10-08 (choix C81) : AC16 étendue à `KESH_ADMIN_PASSWORD`,
     recette de déplacement réécrite et rejouée sur cas piégés, tableaux rognés de `sec:env-vars` repris
     de la 15-7b2, propagation complète du placeholder du secret.
     Validation P3 de la 15-11a remédiée le 2026-10-08 (choix C83, décision de l'orchestrateur sur
     signal D5) : AC4 ABANDONNÉE — `docker-compose.prod.yml` garde ses montages fixes `./documents`,
     `./inbox`, `./log` ; la recette de déplacement et son encadré sont retirés ; version configurable
     renvoyée à l'issue #558. AC16 étendue aux valeurs de la forme `<…>`.
     Validation P4 de la 15-11a (Sonnet ×2) : 0 au-dessus de LOW — VALIDATION CLOSE ; les 23 LOW
     appliqués le 2026-10-08 (choix C84 : `$` entre apostrophes simples, motif de vérification élargi
     aux espaces de tête, coordination avec la 15-7b3).
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation.
     Numérotation des findings : « R3-n », « F-n », « Fn », « R2-n », « R5 » sans autre précision désignent
     les validations de la **15-11** (rapports `15-11-p{1,2,3}-{R,F}.md`) ; « R1-n », « F1-n » ceux de la
     validation P1 de la **15-11a** (`15-11a-p1-{R,F}.md`). -->

**Issues** : **ferme #550 et #557**. La PR porte `closes #550` **et** `closes #557` (mots-clés **dans la
PR** : le dépôt merge en squash). **#557** (sécurité) : le placeholder du gabarit `<GENERATE_ME: …>`,
recopié tel quel comme `KESH_JWT_SECRET` — ou comme `KESH_ADMIN_PASSWORD`, ligne décommentée sans
modification —, est aujourd'hui **accepté** — corrigé par l'AC16 pour les deux variables (décision de
l'orchestrateur en P2 ; l'orchestrateur étend le texte de #557 en conséquence). **`refs #534`** : la sortie de la démonstration exige `KESH_PRODUCTION_RESET`, que cette story
rend transmissible ; le libellé du dialogue et le message 403 (objet de #534) ne sont **pas** traités ici.
**`refs #551`, `refs #552`** sans mot-clé de fermeture (hors périmètre, story 15-12). **`refs #558`**
sans mot-clé de fermeture : version configurable des montages de `docker-compose.prod.yml` (AC4
abandonnée, C83), story future.

**Story sœur : 15-11b** (`15-11b-lecture-unique-des-variables.md`, `refs #550`) — la fonction de lecture
unique `config::env_nonempty` (vide = absent, trim), la migration des 36 sites de lecture, et le test
qui lit le code avec `syn`. **Ordre** : la 15-11a d'abord ; la 15-11b se rebase sur elle et remplace la
liste écrite en dur du test (`LUES`, AC8) par la lecture du code. La 15-11a peut se merger **seule** —
sans la 15-11b — **sans refus de démarrer nouveau** dû aux variables ajoutées : aucune ne fait refuser
le démarrage quand elle arrive vide (§ *Le vide avant la 15-11b*, vérifié au code). Ce qu'elle change
seule, et qui est voulu, est écrit (R2-7) : le refus d'un `KESH_JWT_SECRET` ou d'un `KESH_ADMIN_PASSWORD`
resté à un placeholder (`GENERATE_ME`, ou gabarit entre chevrons `<…>` — AC16, #557), et les effets
annoncés par l'avertissement « relisez » (AC12 f) : les lignes de `.env` jusqu'ici sans effet. **Les
montages de `docker-compose.prod.yml` ne changent pas** (AC4 abandonnée en P3, C83 ; #558) : aucune
donnée n'a à être déplacée.
**Rebase de la 15-11b** : l'AC16 modifie les contrôles du mot de passe admin et du secret JWT dans
`Config::from_env` (`config.rs:633-649` et `:657-671`), voisins des lectures `env::var` de ces deux
variables (`:619`, `:654`) que la 15-11b migre — hunks voisins, à relocaliser par le texte. La 15-11b
touche aussi la ligne `admin-manual.tex:662` (`KESH_JWT_SECRET`, trim), que la 15-11a modifie (AC16 d)
et dont elle refait la mise en page (AC12 j) : **zone partagée**, la 15-11b relocalise par le texte. **La 15-11b est attendue avant le tag v0.13.0** (non bloquant pour la
sûreté). *(Revue de code P1, C-15-11a-6 : les avertissements « invalide » et la sauvegarde pré-import dans
`/app` que la 15-11a seule produisait sont corrigés **dans la 15-11a** pour les sept variables concernées
— § *Le vide avant la 15-11b*, « Mise à jour ».)*

**C'est d'elle que dépend la 15-7b2** (worktree `/home/gcorbaz/devel/kesh-15-7`, choix C-15-7-48,
C-15-7-51) : sa recette — « poser `KESH_PRODUCTION_RESET`, `docker compose up -d`, réinitialiser,
retirer » — ne peut réussir sur une installation standard que si la variable atteint le conteneur.
**Ordre de merge imposé : la 15-11a AVANT la 15-7b2.** Aucune dépendance de code ; des hunks voisins du
manuel (§ *Coordination avec la 15-7b2*).

**Écart déclaré avec l'« Attendu » de #550** : l'issue demande de **retirer** de `.env.example` les
variables fixées par l'image (`KESH_STATIC_DIR`, `KESH_LOCALES_DIR`). La fiche les **conserve** avec une
note (AC6 c) : elles servent à un `cargo run` hors Docker. Écart assumé, à rappeler dans la PR.

## Story

En tant qu'**exploitant d'une installation Kesh sous Docker Compose**,
je veux que **chaque variable que Kesh lit et que `.env.example` me propose de régler atteigne
réellement le conteneur**, et qu'un test empêche qu'un compose en oublie une,
afin de **pouvoir activer l'envoi d'e-mails, sortir de la démonstration, choisir la langue et régler
les plafonds** — au lieu de modifier `.env` sans aucun effet ni aucun message.

## Le défaut, établi au sol le 2026-10-08

Les deux compose distribués (`docker-compose.yml`, `docker-compose.prod.yml`) n'ont pas d'`env_file` :
le service `kesh-api` ne reçoit **que** les variables listées une à une sous `environment:`. Docker
Compose lit bien `.env`, mais **seulement pour interpoler** les `${…}` du fichier compose. Une variable
posée dans `.env` sans ligne correspondante sous `environment:` **n'atteint pas le conteneur**, et
rien ne le signale. Le manuel dit le contraire (`admin-manual.tex:648` : « Le fichier `.env` est chargé
automatiquement par Docker Compose »).

Vérifié par `docker compose config` dans un répertoire d'essai (2026-10-08, Docker Compose 2.40.3 ;
rejoué par la lentille R de la P3 pour les lignes 3 à 5 et la forme `${X-d}`) :

| forme sous `environment:` | `.env` | reçu par le conteneur |
|---|---|---|
| `FOO:` (clé sans valeur) | `FOO=depuis_env` | `FOO=depuis_env` |
| `BAR:` (clé sans valeur) | absente | **rien** — la variable n'existe pas dans le conteneur ; `config` la **montre** à `null` |
| `BAZ: ${BAZ:-}` | absente | `BAZ=` (**chaîne vide**) |
| `QUX: ${QUX:-12}` | `QUX=` (vide) | `QUX=12` (`:-` traite le vide comme absent) |
| `FOO:` (clé sans valeur) | `FOO=` (vide) | `FOO=` (**chaîne vide**) — la clé sans valeur ne protège **pas** du vide |
| `P1: ${P1-/def}` | `P1=` (vide) | `P1=` (vide) — sans deux-points, seul l'**absent** prend le défaut |
| `P: ${P:-}` | `P=pa$word` (ou `P="pa$word"`) | `P=pa` — **tronqué** : Compose interpole aussi les valeurs de `.env`, `$word` y est une variable (avertissement « The "word" variable is not set ») |
| `P: ${P:-}` | `P=pa$$word`, `P='pa$word'` ou `P="pa$$word"` | `P=pa$word` — `$$` échappe le `$` ; les apostrophes simples le gardent littéral |

*(Les deux dernières lignes : mesurées le 2026-10-08 par l'agent de remédiation P1, Compose 2.40.3,
`env -i … docker compose config --format json` — F1-4. Elles concernent toute valeur choisie par
l'exploitant, d'abord `KESH_SMTP_PASSWORD`, que la story rend transmise : un mot de passe d'application
contenant `$` arriverait tronqué. Écrit au gabarit et au manuel, AC6 h, AC12 a.)*

⚠️ **Deux précisions mesurées en P4 (R4-13, choix C84)** — scratchpad, Compose 2.40.3 et `dotenvy` 0.15.7
(la version du `Cargo.lock`, lue par `main.rs:44` pour `cargo run`) :
- **`docker compose config` réaffiche tout `$` littéral en `$$`** : `P='pa$word'` y apparaît
  `pa$$word` (YAML comme JSON). La valeur reçue par le conteneur est `pa$word` ; qui rejoue la mesure
  doit décoder ce réaffichage, sans quoi les lignes 7 et 8 paraissent fausses.
- **`$$` ne vaut que pour Compose.** `dotenvy` (`parse.rs`, l. 165-220 : `$` ouvre une substitution, un
  second `$` la clôt sur un nom vide et en rouvre une autre) lit `pa$$word` comme **`pa`**, comme
  `pa$word` ; `"pa$word"` et `"pa$$word"` y sont même des **erreurs d'analyse**. Seules formes
  littérales dans **les deux** lecteurs : **les apostrophes simples** `'pa$word'` (« strong quote »), et,
  pour une valeur qui contient elle-même une apostrophe, les guillemets doubles avec `\$`
  (`"it's pa\$word"` → `it's pa$word` dans les deux, mesuré). D'où la consigne de l'AC6 h et de
  l'AC12 a : **apostrophes simples d'abord**.

**Forme retenue (C75, révise C71)** : `KESH_X: ${KESH_X:-}` pour **chacun des 16 ajouts, sans exception**
(R2-9 ; la forme avec défaut reste celle des **entrées existantes** qui portent un défaut de déploiement
voulu, elle n'est pas admise pour un ajout). L'interpolation depuis `.env` est le mécanisme de base de
Compose, documenté depuis Compose v1 ; la variable est **toujours** présente dans le conteneur, vide quand
`.env` ne la pose pas. Une ligne `KESH_X=` vide dans `.env` atteint de toute façon le conteneur, quelle
que soit la forme (ligne 5) : la règle « vide = absent » relève du **code**, et c'est la 15-11b qui la
pose pour tous les lecteurs. La clé sans valeur est interdite (test (V)) : son effet dépend de la version
de Compose et elle ne protège pas du vide.

### Le vide avant la 15-11b — ce que fait aujourd'hui le code des 16 ajouts quand ils arrivent vides

Relevé au code le 2026-10-08 (à refaire au T0) — c'est ce qui rend la 15-11a sûre **seule** :

| variable (ajoutée dans) | valeur vide → aujourd'hui | refus de démarrer ? |
|---|---|---|
| `KESH_COOKIE_SECURE` (P) | `true` (`config.rs:1034`, vide explicitement accepté) | non |
| `KESH_SMTP_HOST`, `_USER`, `_PASSWORD`, `_FROM`, `KESH_PUBLIC_BASE_URL` | `None` (`opt_trimmed_env`, trim + vide = absent) | non |
| `KESH_SMTP_TLS`, `KESH_FEATURE_FORGOT_PASSWORD` | défaut (`parse_strict_bool` : `"" => Ok(default)`) | non |
| `KESH_PRODUCTION_RESET` | `false` (`env_flag_enabled`) | non |
| `KESH_SMTP_PORT` | avertissement « `KESH_SMTP_PORT=''` invalide », puis 587 | non |
| `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB` (Y) ; `KESH_ADMIN_EXPORT_INMEM_MB`, `KESH_ADMIN_IMPORT_MAX_MB` | avertissement « invalide », puis défaut (12, 10, 50, 512) | non |
| `KESH_LANG` (Y) | `Locale::from("")` → avertissement « Locale '' non reconnue », puis fr-CH (= défaut `fr`) | non |
| `KESH_ADMIN_BACKUP_DIR` | chaîne vide → la sauvegarde pré-import s'écrit dans le **répertoire courant** du conteneur (`/app`, `WORKDIR` du `Dockerfile`) au lieu de `/tmp` (`routes/admin.rs:471-490` ; `create_dir_all("")` rend `Ok` — **à mesurer au T0**). Les deux sont éphémères ; #552 (15-12) traite la persistance | non |

**Mise à jour — revue de code P1 (C-15-11a-6)** : le tableau ci-dessus décrit le code **avant** la revue.
Les trois dernières lignes ne valent plus : `KESH_ADMIN_BACKUP_DIR` et `KESH_LANG` sont lues par
`opt_trimmed_env` (vide ou blanc = absente → `/tmp`, `fr`, valeur non blanche trimée) ; les cinq
numériques `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB`, `KESH_ADMIN_EXPORT_INMEM_MB`,
`KESH_ADMIN_IMPORT_MAX_MB`, `KESH_SMTP_PORT` traitent une valeur vide ou blanche comme absente (défaut,
**sans** avertissement) ; une valeur **non vide** invalide garde son comportement (avertissement
« invalide », puis défaut). Tests : `from_env_empty_or_blank_vars_take_code_default_silently` et son
témoin `from_env_non_empty_invalid_values_still_warn` (`config.rs`) ; sept mutations, toutes rouges.

**Frontière avec la 15-11b.** La 15-11a règle le vide **des seules variables que ses compose
transmettent en `${NOM:-}`** (dix-sept noms ; les dix autres l'acceptaient déjà) — c'est-à-dire ce
qu'elle rend observable. La 15-11b règle le vide (et le trim) **de toutes** les variables lues, par une
fonction unique : `DATABASE_URL`, `KESH_PORT`, `KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR`, les variables JWT,
de session, de limitation et de journal, et un `cargo run` hors Docker ; elle trime aussi les valeurs non
blanches des cinq numériques (`" 12 "` → 12), ce que la 15-11a ne fait pas. Ce qu'il faut reporter dans
la fiche 15-11b est listé au Change Log (revue P1).

**Conséquence, écrite (avant la revue P1)** : la 15-11a seule produit jusqu'à six avertissements au démarrage et déplace la
sauvegarde pré-import de `/tmp` à `/app` ; aucune des variables **ajoutées** ne fait refuser le démarrage
d'une installation (les seuls refus nouveaux, voulus, sont ceux des placeholders — `GENERATE_ME` ou
forme `<…>` —, comme secret JWT ou comme mot de passe admin — AC16).
La 15-11b les fait disparaître (vide = absent partout). Le même état vaut pour un compose téléchargé
depuis `main` et employé avec l'image 0.12.1, entre le merge et la publication de la 0.13.0 (F-5 de la P3) :
le CHANGELOG et le manuel disent « le compose de la 0.13.0 s'emploie avec l'image 0.13.0 —
`docker compose pull` avant `up -d` » (AC12 f, AC13).

**Second défaut, trouvé en établissant le premier** : `docker-compose.yml` n'a **pas d'`image:`**, seulement
`build: context: .` — or le manuel (`admin-manual.tex:218-224`, § *Installation, Étape 2*) fait
télécharger ce seul fichier dans un répertoire vide. Vérifié : `docker compose build kesh-api` y échoue
sur `failed to read dockerfile: open Dockerfile: no such file or directory`. Traité ici (AC5, C73).

**Troisième fait, écrit pour qu'on ne le prenne pas pour un oubli (F-10 de la P3, corrigé par R1-2)** :
`docker-compose.yml` garde `KESH_JWT_SECRET: ${KESH_JWT_SECRET:-change-me-32-bytes-…}`. Ce défaut est
**refusé** par le code (`ConfigError::InsecureJwtSecret`, `config.rs:35-39`, contrôle `:669`) : une
installation **sans `.env`** — ou dont `.env` ne pose **aucun** secret — **ne démarre pas du tout**. C'est
voulu : le secret JWT est **obligatoire** et n'a pas de flux de remplacement (contrairement au mot de passe
admin, que l'onboarding `/setup` remplace — C76). Le défaut `change-me…` sert de **garde** contre
l'**absence** du secret ; il **n'est pas retiré**.

⛔ **Mais cette garde ne couvre pas l'oubli le plus probable** (R1-2, #557) : la ligne **active** du
gabarit, `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>` (`.env.example:88`), recopiée sans
modification, fait **35** caractères (≥ 32) et ne contient pas `change-me` : `Config::from_env` l'**accepte**
aujourd'hui (`config.rs:657-671`), et Kesh signe ses jetons avec un secret **public**, connu de tout lecteur
du dépôt. Le doc-comment `config.rs:35-36` (« `change-me` (placeholder du `.env.example`) ») est périmé : le
placeholder du gabarit n'est plus `change-me`. **Corrigé par l'AC16** : refus au démarrage de tout secret
qui contient `GENERATE_ME` **ou** qui est de la forme `<…>` (premier caractère `<`, dernier `>`, après
trim — F3-4 de la P3 : les gabarits du manuel `admin-manual.tex:240`, `:244`, `:988`, `:1230` sont de
cette forme et ne contiennent pas `GENERATE_ME`). Le manuel et `.env.example` disent « `KESH_JWT_SECRET` est obligatoire —
générez-le (`openssl rand -hex 32`) » (AC6 d, AC12 e).

⛔ **Même défaut pour le mot de passe admin** (R2-3 = F2-1, décision de l'orchestrateur en P2) :
`#KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>` (`.env.example:82`, ligne commentée) fait
**38** caractères et n'est pas `changeme` : décommentée sans modification — ce que le manuel demande à
l'étape « Générer les secrets obligatoires » (`admin-manual.tex:537-539`) —, elle est **acceptée**
(`config.rs:633-646`), et sur une base vide `auth/bootstrap.rs` crée un administrateur dont le mot de
passe est lisible dans le dépôt, l'écran `/setup` n'étant plus proposé. **Corrigé par l'AC16** (même
garde, variante `InsecureAdminPassword`), et le manuel cesse de dire « obligatoire » ce mot de passe :
il est **optionnel** (onboarding `/setup` sinon — AC12 e). ⛔ **Même défaut sous une autre graphie (F3-4
de la P3)** : les exemples du manuel `KESH_ADMIN_PASSWORD=<mot de passe fort, min. 12 caracteres>`
(`admin-manual.tex:244`, ligne **active** de l'*Étape 3*, 39 caractères), `<mot-de-passe-fort-12-chars>`
(`:988`, 28) et `<nouveau-mdp-12+>` (`:1230`, break-glass, 17) passent aujourd'hui `config.rs:633-646`
(ni `changeme`, ni moins de 12 caractères) : recopiés tels quels, ils créent ou réinitialisent un
administrateur au mot de passe publié. **Corrigé par l'AC16** : toute valeur de la forme `<…>` est
refusée, pour les deux variables.

## Inventaire fermé — refait depuis le code le 2026-10-08

**Méthode** (à refaire au T0 ; la table est l'**attendu** du test, AC8, qui la porte en dur jusqu'à la
15-11b) : toute lecture de l'environnement dans le code de production du workspace. Commande qui reproduit
la liste — elle rend **40** noms, plus `RUST_LOG` (lu par `logging.rs:130` via
`EnvFilter::DEFAULT_ENV`, constante de `tracing-subscriber`) :

```sh
grep -rhoE '(env::var|opt_trimmed_env|parse_strict_bool|env_flag_enabled)\("[A-Z][A-Z0-9_]*"' crates/*/src \
  | grep -oE '"[A-Z][A-Z0-9_]*"' | tr -d '"' | sort -u          # 40 noms ; + RUST_LOG = 41
```

Elle ne distingue pas le code de test : les seules lectures littérales de `kesh-db/src` (`DATABASE_URL`,
dans des `mod tests`) nomment une variable déjà lue en production, et aucune autre n'apparaît — vérifié le
2026-10-08. L'analyse exacte (règle `cfg(test)`, indirections, macros) est celle de la 15-11b.
`std::env::temp_dir()` (`routes/admin.rs:80`, production) lit `TMPDIR` : angle mort assumé (R3-4 : le
site `document_storage.rs:174` cité jusqu'ici est dans un `mod tests`, il n'est pas de production). Le
frontend est un SPA statique, sans lecture d'environnement à l'exécution.

**Résultat : 41 variables lues** (39 `KESH_*`, plus `DATABASE_URL` et `RUST_LOG`). Croisées avec
`.env.example` et les blocs `environment:` des trois compose (Y = `docker-compose.yml`,
P = `docker-compose.prod.yml`, D = `docker-compose.dev.yml`) :

| classe | variables | Y | P | D |
|---|---|---|---|---|
| **transmise** par les deux (22) | `DATABASE_URL`, `RUST_LOG`, `KESH_PORT`, `KESH_HOST`, `KESH_ADMIN_USERNAME`, `KESH_ADMIN_PASSWORD`, `KESH_JWT_SECRET`, `KESH_JWT_EXPIRY_MINUTES`, `KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS`, `KESH_REFRESH_INACTIVITY_MINUTES`, `KESH_RATE_LIMIT_WINDOW_MINUTES`, `KESH_RATE_LIMIT_MAX_ATTEMPTS`, `KESH_RATE_LIMIT_BLOCK_MINUTES`, `KESH_LOG_FILE_PATH`, `KESH_LOG_FILE_ROTATION`, `KESH_LOG_FILE_MAX_FILES`, `KESH_LOG_FILE_FORMAT`, `KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR`, `KESH_INBOX_MAX_FILE_BYTES`, `KESH_INBOX_MAX_FILES_PER_RUN`, `KESH_INBOX_MAX_PDF_PAGES` | ✓ | ✓ | 11 sur 22 |
| **à transmettre — par un seul** (4) | `KESH_COOKIE_SECURE` | ✓ | **✗** | ✗ |
| | `KESH_LANG`, `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB` | **✗** | ✓ | ✗ |
| **à transmettre — par aucun** (12) | `KESH_SMTP_HOST`, `KESH_SMTP_PORT`, `KESH_SMTP_USER`, `KESH_SMTP_PASSWORD`, `KESH_SMTP_FROM`, `KESH_SMTP_TLS`, `KESH_PUBLIC_BASE_URL`, `KESH_FEATURE_FORGOT_PASSWORD`, `KESH_PRODUCTION_RESET`, `KESH_ADMIN_BACKUP_DIR`, `KESH_ADMIN_EXPORT_INMEM_MB`, `KESH_ADMIN_IMPORT_MAX_MB` | ✗ | ✗ | `KESH_PRODUCTION_RESET` seule |
| **fixée par l'image** (2) | `KESH_STATIC_DIR` (`/app/static`), `KESH_LOCALES_DIR` (`/app/locales`) — `Dockerfile` `ENV`, l. 45-46 ; les transmettre écraserait le chemin de l'image par une valeur d'hôte et casserait le service du SPA ou le chargement des traductions | ✗ | ✗ | ✗ |
| **interdite en production** (1) | `KESH_TEST_MODE` — expose `/api/v1/_test/*` ; ne doit **jamais** être transmissible par un compose distribué | ✗ | ✗ | ✗ |

Total : 22 + 4 + 12 + 2 + 1 = **41**. **À ajouter : 16 variables**, soit **28 lignes** (12 × 2 + 1 dans P +
3 dans Y) : **15 lignes** dans Y, **13** dans P.

⚠️ **« Transmise » dit « nommée sous `environment:` », pas « effective ».** Deux entrées de la classe
« transmise » ne font pas aujourd'hui ce que leur documentation promet, et sont corrigées ici :
- **`KESH_LOG_FILE_PATH`** : l'opt-out documenté (`KESH_LOG_FILE_PATH=` vide) est **sans effet** —
  `${KESH_LOG_FILE_PATH:-…}` remplace le vide par le défaut (ligne `QUX`). Corrigé (AC2 ii).
- **`KESH_ADMIN_PASSWORD` dans Y**, écrit `${KESH_ADMIN_PASSWORD:-changeme}` : absent ou vide dans `.env`, il
  vaut `changeme`, que `Config::from_env` refuse (`config.rs:636-637`, `InsecureAdminPassword`) — **sur Y,
  Kesh refuse de démarrer** dès que `.env` ne pose pas de mot de passe. La procédure **recommandée** du
  manuel (`admin-manual.tex:972`, onboarding `/setup`) et `.env.example:66-68` (« vars NON renseignées →
  onboarding ») échouent sur Y, et l'étape 5 du break-glass (`admin-manual.tex:1238`) fait refuser le
  démarrage suivant. **Corrigé** (AC2 i, C76) : `${KESH_ADMIN_PASSWORD:-}` — absent ou vide → `None`
  (`config.rs:619-629`, trim et vide = absent, déjà le cas) → `has_admin_env` faux
  (`auth/bootstrap.rs:62-69`, couple exigé) → cas « setup-required », aucun refus. P porte
  `${KESH_ADMIN_PASSWORD}` : même valeur (vide si absente), mais **Compose avertit** à chaque `up`
  (`The "KESH_ADMIN_PASSWORD" variable is not set`) sur le flux **recommandé** `/setup` — la forme dit
  « obligatoire » quand le commentaire dira « optionnelle » (R1-5). **Passe aussi à `${KESH_ADMIN_PASSWORD:-}`**
  (AC2 i, C79) : valeur inchangée, avertissement supprimé, une seule forme pour les deux compose. `KESH_ADMIN_USERNAME` garde `${…:-admin}` (un nom
  seul ne déclenche rien). **Risque écrit comme comportement existant** : sur base vide, l'assistant
  `/setup` est ouvert à qui l'atteint le premier (`admin-manual.tex:980`) et Y publie `80:80` ; c'est le
  flux recommandé depuis la v0.1.2 et celui de P depuis toujours — la correction le rend atteignable sur Y,
  elle ne le crée pas. *(Rappel : sur Y, le secret JWT reste obligatoire — § ci-dessus, F-10.)*

**Variables de `.env.example` que le code ne lit pas** (7) — légitimes, et le test les contrôle :

| classe | variables | usage vérifié |
|---|---|---|
| **hôte seulement** (3) | `KESH_INBOX_HOST_DIR`, `KESH_DOCUMENTS_HOST_DIR`, `KESH_LOG_HOST_DIR` | `volumes:` de `docker-compose.yml`. ⚠️ **`docker-compose.prod.yml` les ignore** (montages fixes `./log`, `./inbox`, `./documents`) **et continue de les ignorer** : l'AC4 (montages configurables) a été **abandonnée en P3** (C83) — la rendre configurable exige une procédure de déplacement des données existantes, dont trois passes de validation ont montré qu'elle ne s'écrit pas sûrement en quelques lignes de shell ; version configurable renvoyée à l'issue **#558**. Le gabarit et le manuel, qui conseillent de les poser sur Synology (`.env.example:172-177`, `:199-204` ; `admin-manual.tex:782-792`), disent qu'elles sont **sans effet** avec `docker-compose.prod.yml` (AC6 i, AC12 c) |
| **service MariaDB** (4) | `MARIADB_ROOT_PASSWORD`, `MARIADB_DATABASE`, `MARIADB_USER`, `MARIADB_PASSWORD` | service `mariadb` et `DATABASE_URL` de `docker-compose.yml` ; sans objet pour P |

**Variable fantôme** (1) : `KESH_ADMIN_RESET` — **lue nulle part**, mais nommée comme recours
« break-glass » dans deux messages **émis à l'exécution** (`config.rs:158`, refus de démarrage
`IncompleteSmtpConfig`, dans un `write!` ; `main.rs:337`, journal d'information, dans un `tracing::info!`),
dans la **prose** de `.env.example:235` (pas une ligne d'affectation) et dans deux commentaires
(`config.rs:319`, `lib.rs:1040`). Le vrai recours est le couple `KESH_ADMIN_USERNAME` /
`KESH_ADMIN_PASSWORD`. **Retiré** (AC7).

**Documentée dans `.env.example` en prose seulement** (1) : `KESH_TEST_MODE` (`:267-268`, « DO NOT SET
KESH_TEST_MODE IN PRODUCTION »). Exception nommée du test, **interdite** de ligne d'affectation (AC6 a).
`KESH_HOST` a sa ligne (`.env.example:28`, prose commentée qui commence par l'affectation).

**Hors périmètre du test, écrit** : `docker-compose.dev.yml` (pile de développement, non distribuée :
`README.md:77`) — inventorié ci-dessus, pas contraint.

## Acceptance Criteria

1. **Inventaire et liste fermée `LUES`.** La table § *Inventaire fermé* est refaite au T0 depuis `main` à
   jour par la commande `grep` de la section, et reproduite au Dev Agent Record : 41 lues, 38 transmises
   après la story (22 + 4 + 12), 2 fixées par l'image, 1 interdite ; 3 hôte seulement ; 4 MariaDB ;
   0 fantôme. Tout écart (variable ajoutée par une story mergée entre-temps) est **écrit** et porté dans
   la liste `LUES` du test, non corrigé en silence.

2. **Transmission.** Le service `kesh-api` de `docker-compose.yml` **et** celui de
   `docker-compose.prod.yml` portent, sous `environment:`, **chacune des 38 variables** des classes
   « transmise » et « à transmettre ». Les **16 variables ajoutées** (28 lignes : 15 dans Y, 13 dans P)
   le sont sous la forme **`KESH_X: ${KESH_X:-}`**, sans exception (C75, R2-9) : la valeur de `.env` (ou
   du shell) si elle y est, **une chaîne vide sinon** — que le code traite comme une absence ou un défaut
   avec avertissement (§ *Le vide avant la 15-11b*), puis comme une absence pure après la 15-11b. **La
   clé sans valeur (`KESH_X:`) est interdite** (test (V)). **La forme `${KESH_X:-}` des ajouts est
   contrôlée** (F1-8) : la liste fermée `AJOUTS` du test (V) — les 28 couples (variable, compose) de la
   § *Inventaire fermé* — exige `${NOM:-}` et **aucune autre** forme, si bien qu'un ajout écrit
   `${KESH_SMTP_PORT:-587}` (défaut du code recopié) **rougit** (M21). Les entrées existantes gardent leur
   forme (`${X:-v}` : défauts de déploiement voulus, ex. `KESH_PORT`, `KESH_JWT_SECRET` de Y — garde,
   § *Le défaut* ; `${X}` : obligatoires de P), **sauf deux variables** :
   (i) **`KESH_ADMIN_PASSWORD`** passe, dans `docker-compose.yml`, de `${KESH_ADMIN_PASSWORD:-changeme}` à
   **`${KESH_ADMIN_PASSWORD:-}`** (C76) et, dans `docker-compose.prod.yml`, de `${KESH_ADMIN_PASSWORD}` à
   **`${KESH_ADMIN_PASSWORD:-}`** (R1-5, C79 : même valeur, sans l'avertissement de Compose sur le flux
   `/setup`) ; elle entre dans la liste fermée `SANS_DEFAUT` du test (V), qui n'admet **que** `${NOM:-}` ; le
   commentaire « Admin initial » de Y (`:47`) gagne « vide → onboarding `/setup` » ; les commentaires de P
   qui la disent obligatoire (`docker-compose.prod.yml:27-29`, `:43`, `:67-69` — R3-9) sont corrigés :
   optionnelle, absente → onboarding `/setup` ; si posée, ≥ 12 caractères et ≠ `changeme` ;
   (ii) `KESH_LOG_FILE_PATH` passe, dans les deux compose, à **`${KESH_LOG_FILE_PATH-/var/log/kesh/kesh.log}`**
   (tiret **sans** deux-points) — le défaut s'applique quand la variable est **absente** de `.env`, et une
   ligne `KESH_LOG_FILE_PATH=` **vide** transmet le vide, qui désactive le journal fichier
   (`LogConfig::from_raw` filtre déjà le vide, test `config.rs:2315`). C'est la seule variable dont le vide
   a un sens distinct de l'absence côté compose (recensement : `grep -niE 'vide|empty' .env.example` et
   lecture des lecteurs).
   **Une même variable peut porter deux formes selon le compose** — `KESH_COOKIE_SECURE` (`${…:-true}` dans
   Y, `${…:-}` ajoutée dans P), `KESH_LANG`, `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB`
   (`${…:-défaut}` dans P, `${…:-}` ajoutées dans Y) : voulu ; le Dev Agent Record vérifie pour ces quatre
   que le défaut du compose = le défaut du code. **Aucun `env_file:`** sur `kesh-api` (C71). Chaque groupe
   ajouté porte un commentaire d'une ligne qui dit ce qu'il règle et renvoie au manuel — **sans nommer
   `KESH_PRODUCTION_RESET`** dans un commentaire (celui de son groupe dit « sortie de la démonstration ») :
   le T8 de la 15-7b2 attend de `grep -n "PRODUCTION_RESET" docker-compose.yml docker-compose.prod.yml`
   **une ligne par fichier** (R2-10). Un commentaire en tête du bloc `environment:` dit la règle :
   *« Seules les variables listées ici atteignent Kesh. Une variable ajoutée au code doit l'être ici —
   test `configuration_transmise`. Après une modification de `.env` : `docker compose up -d` (un
   `restart` ne relit pas `.env`). »*

3. **Exceptions.** `KESH_STATIC_DIR` et `KESH_LOCALES_DIR` ne sont transmises par **aucun** des deux
   compose, et le `Dockerfile` (étape finale) les fixe par `ENV`. `KESH_TEST_MODE` n'est transmise par
   **aucun** des deux compose. Ces trois exceptions sont les **seules** variables lues non transmises.

4. **Montages de `docker-compose.prod.yml` inchangés — AC abandonnée en P3 (C83, révise C73).** Les trois
   montages de P **restent fixes** : `./log:/var/log/kesh`, `./inbox:/data/inbox`,
   `./documents:/data/documents`. Ceux de `docker-compose.yml` restent `${KESH_LOG_HOST_DIR:-./log}`,
   `${KESH_INBOX_HOST_DIR:-./inbox}`, `${KESH_DOCUMENTS_HOST_DIR:-./documents}`. **Raison** : rendre les
   montages de P configurables déplace, sur une installation existante qui a posé ces variables comme le
   conseillent le gabarit et le manuel, le dossier monté sur `/data/documents` — justificatifs et PDF
   figés « disparus » ; la recette de déplacement écrite pour l'éviter a produit un défaut MEDIUM ou
   HIGH à chacune des trois passes (P1 R1-1 ; P2 R2-1, R2-2 ; P3 R3-1 = F3-1, F3-2, R3-2), par recyclage
   de la même machinerie (un analyseur de `.env` qui doit imiter Compose). **Version configurable : issue
   #558.** Ce que la story fait à la place : (a) le test garde l'état des deux compose (AC8 (T),
   structure ; M14, M28) — un montage de P rendu configurable sans la procédure de #558 **rougit** ;
   (b) le gabarit, le manuel et un commentaire de P disent que les `KESH_*_HOST_DIR` sont **sans effet**
   avec `docker-compose.prod.yml` (AC6 i, AC12 c, T2). Les commentaires de montage existants de P
   (`docker-compose.prod.yml:97-101`, `:107-108`, `:119-127` — « monté sur `./log` », « déposez vos
   factures dans `./inbox` », « bind mounts host co-localisés (scope Hyper Backup unique) ») **restent
   vrais** et ne changent pas (F3-9, vérifié). Numéro d'AC conservé pour ne pas décaler les renvois.

5. **`docker-compose.yml` s'installe seul.** Le service `kesh-api` reçoit
   `image: gcorbaz/kesh:latest`, **`build:` conservé** (avec les deux et sans `pull_policy`, Compose tire
   l'image et ne construit que si elle est introuvable — comportement rejoué par la lentille R de la P3 :
   « Pulling / Pulled », aucune construction). Preuve (T6) : dans un répertoire vide ne contenant que ce
   fichier et un `.env` d'essai, `docker compose --dry-run up -d` annonce le **tirage** de
   `gcorbaz/kesh:latest` et **aucune construction** ; le même essai **avant** la modification annonce une
   construction (ou échoue sur le `Dockerfile` absent) — les deux sorties au Dev Agent Record.
   **Précondition** : `gcorbaz/kesh:latest` absente localement (`docker image ls gcorbaz/kesh` avant
   l'essai ; sinon la supprimer d'abord, ou essayer sur un tag absent). **Repli** si le comportement
   diffère : `pull_policy: missing`, mesuré à nouveau, écrit au Dev Agent Record et au registre ; si le
   repli échoue aussi, retirer `image:` de la story, l'écrire et le signaler, sans bloquer le reste.
   **Effet sur les développeurs, à écrire** (commentaire du service et `DOCKER_START.md`) : un
   `docker compose up -d` lancé depuis le dépôt **tire l'image publiée** — « `--build` pour construire
   depuis les sources » ; et l'**effet inverse** (F11) : tout `up --build` — `DOCKER_START.md:13` et `:23`
   (démarrage rapide) comme `:70` (`up -d --build kesh-api`, R1-4) — **étiquette le build local
   `gcorbaz/kesh:latest`**, que les `up -d` suivants emploient sans tirer — remède :
   `docker compose pull kesh-api`.
   **`DOCKER_START.md`, deux affirmations fausses corrigées** (F1-2) : `:38-40` (« Admin initial : `admin` /
   `admin` ») — aucun compte `admin` n'est créé ; l'administrateur se crée à l'écran **`/setup`** au premier
   accès (ou par `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD`, ≥ 12 caractères, posées dans `.env`) ; `:13`,
   `:23` (démarrage rapide **sans `.env`**) — refusé par le code (`InsecureJwtSecret`, défaut `change-me…`
   de Y) : le démarrage rapide commence par « `cp .env.example .env`, puis y remplacer `KESH_JWT_SECRET` par
   la sortie de `openssl rand -hex 32` » (AC16 : le placeholder recopié est refusé).
   **`DOCKER_START.md` passe entièrement à `docker compose` (Compose v2)** (R2-6, F2-10, décision de
   l'orchestrateur) : ses douze commandes `docker-compose` (`:13, 23, 46, 47, 52, 57, 62, 70, 76, 89, 110,
   111` — `grep -nE 'docker-compose ' DOCKER_START.md`, à refaire au T8) deviennent `docker compose`, si
   bien que les lignes neuves de l'AC5 et de l'AC15 ne détonnent pas. Raison : tout ce que la fiche affirme
   de Compose (tirage plutôt que construction quand `image:` et `build:` coexistent, interpolation,
   `$$`) est **mesuré sous v2** (2.40.3) ; sous v1, un service constructible dont l'image manque est
   **construit** sans être tiré (comportement de mémoire, **non mesuré**), et le manuel emploie v2 partout.
   Le Dev Agent Record écrit « Compose v1 non mesuré ». Les **noms de fichiers** `docker-compose.yml`
   (trait d'union) ne changent pas — le motif de remplacement est la commande suivie d'une espace.

6. **`.env.example`.**
   (a) Chacune des 41 variables lues y figure en ligne d'affectation (commentée ou non), **sauf**
   `KESH_TEST_MODE`, qui n'y a **aucune** ligne d'affectation (prose seulement, `:267-268`, inchangée) ;
   `KESH_HOST` a la sienne (`:28`). Déjà le cas : l'AC fixe l'invariant que le test tient.
   (b) Aucune ligne d'affectation (au sens de l'AC8 (E) : `^#?\s*[A-Z][A-Z0-9_]*=`) n'y nomme une variable
   que ni le code, ni un `volumes:`, ni le service MariaDB n'utilise.
   (c) Le bloc « Chemins internes » (`:152-156`) dit que `KESH_STATIC_DIR` et `KESH_LOCALES_DIR` sont
   **fixées par l'image** (`/app/static`, `/app/locales`), **ignorées par les compose fournis**, et ne
   servent qu'à un `cargo run` hors Docker.
   (d) L'en-tête dit la règle de l'AC2 (seules les variables listées dans le compose atteignent Kesh ;
   `docker compose up -d` après modification ; `KESH_JWT_SECRET` obligatoire **et à générer**
   — `openssl rand -hex 32` ; le placeholder `<GENERATE_ME: …>` laissé tel quel **fait refuser le
   démarrage** depuis la 0.13.0, AC16). La phrase existante de l'en-tête (`:6-8`, « les defaults insecure
   type `changeme` ou `change-me-32-bytes-...` font fail-fast au boot ») nomme aussi `GENERATE_ME` (R2-4,
   F2-6). Le commentaire de `KESH_JWT_SECRET` (`:85-86`, « ne doit pas
   contenir "change-me" ») dit « ni `GENERATE_ME` ». Le commentaire de
   `KESH_LOG_FILE_PATH` (`:206-207`, « VIDE **ou absent** → logs fichier désactivés ») est **faux sur
   l'absence** sous Docker et se **réécrit** :
   *« Sous Docker : absente → journal fichier activé (défaut du compose, `/var/log/kesh/kesh.log`) ; ligne
   présente et vide → journal fichier désactivé, contrairement aux autres variables. Hors Docker
   (`cargo run`) : absente ou vide → désactivé. »* Le marqueur « contrairement aux autres variables » est
   celui que contrôle (V). *(La règle générale « une ligne vide vaut une ligne absente » s'écrit à
   l'en-tête par la 15-11b, qui la rend vraie pour tous les lecteurs.)*
   (e) La prose `KESH_ADMIN_RESET` (`:235`) est remplacée par le vrai recours (`KESH_ADMIN_USERNAME` /
   `KESH_ADMIN_PASSWORD`, section « Compte admin initial »).
   (f) ⛔ Le bloc `KESH_PRODUCTION_RESET` (`:272-279`) **n'est pas réécrit** ici : la 15-7b2 en est
   propriétaire (son AC 10). Si la 15-7b2 est déjà mergée au développement, vérifier seulement que son
   texte ne contredit pas la transmission.
   (g) Section « Compte admin initial » (`:58-82`) : inchangée sur le fond — elle dit déjà « vars NON
   renseignées → onboarding », que l'AC2 (i) rend vrai sur Y ; la ligne `#KESH_ADMIN_PASSWORD=` reste
   **commentée** (contrôle de `SANS_DEFAUT`, AC8 (V)) et garde son placeholder (le test de l'AC16 c le
   lit). Une ligne de commentaire s'y ajoute (AC16) : *« décommentée sans remplacer `<GENERATE_ME: …>`,
   elle fait refuser le démarrage depuis la 0.13.0 ; laissée commentée, l'administrateur se crée à l'écran
   `/setup` »*.
   (h) **Le `$` dans une valeur** (F1-4, mesuré, § *Le défaut* ; réécrit en P4, R4-13, C84) : l'en-tête et
   le commentaire de `KESH_SMTP_PASSWORD` (`:253`) disent qu'**une valeur qui contient `$` se met entre
   apostrophes simples** (`KESH_SMTP_PASSWORD='pa$word'`) — sinon Compose la tronque au `$`, avec un seul
   avertissement à `docker compose up`, et `cargo run` (`dotenvy`) aussi. **Pas de consigne `$$` dans
   `.env.example`** : `$$` échappe le `$` pour Compose mais `dotenvy` lit `pa$$word` comme `pa`, et le
   gabarit sert aux deux (AC6 c). Une valeur qui contient elle-même une apostrophe : guillemets doubles et
   `\$` (`"it's pa\$word"`, mesuré dans les deux lecteurs).
   (i) **Chemins d'hôte** (C83) : les blocs `:170-177` (`KESH_INBOX_HOST_DIR`, `KESH_DOCUMENTS_HOST_DIR`)
   et `:199-204` (`KESH_LOG_HOST_DIR`) **gardent leurs lignes d'affectation commentées** (les variables
   servent `docker-compose.yml` ; (E) les contrôle) et gagnent chacun une ligne : *« Sans effet avec
   `docker-compose.prod.yml` (installation Synology du manuel) : ses montages sont fixes — `./inbox`,
   `./documents`, `./log` dans le répertoire du compose (issue #558). »* La phrase `:172-174`
   (« Synology : créez un dossier partagé… puis : ») est conditionnée à `docker-compose.yml`.

7. **Le fantôme `KESH_ADMIN_RESET` disparaît du code et du gabarit.** `config.rs:158` (texte de
   `ConfigError::IncompleteSmtpConfig`), `main.rs:337` (journal), `config.rs:319` et `lib.rs:1040`
   (commentaires) nomment le vrai recours : *break-glass `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD`* ;
   `.env.example:235` (AC6 e). Grep de contrôle à zéro, collé au Dev Agent Record :
   `grep -rn 'KESH_ADMIN_RESET' --exclude-dir={target,node_modules,_bmad-output,.git} --exclude=CHANGELOG.md .`.
   Le test neuf n'écrit pas le nom en un seul jeton (« `KESH_ADMIN_` + `RESET` »), si bien que le grep
   reste à zéro **sans exclusion** du test (R2-12). *(Les sites du code ne sont pas vus par le (F) de la
   15-11a, qui ne lit pas le code Rust : ce grep est leur contrôle ; la 15-11b étend (F) aux littéraux du
   code et des macros.)*

8. **Le test garde-fou** — `crates/kesh-api/tests/configuration_transmise.rs`, sans base de données,
   exécuté par `cargo test --workspace` (CI) et `scripts/test-fast.sh` (local). Il lit les fichiers du
   dépôt (racine = `CARGO_MANIFEST_DIR/../..`). **Il ne lit pas le code Rust.** Ses listes écrites à la
   main sont fermées, chacune avec sa raison en commentaire et, pour les exceptions, un **contrôle de
   cette raison** — `LUES`, `EXCEPTIONS`, `VIDE_SIGNIFIANT`, `SANS_DEFAUT`, `AJOUTS`, et les deux listes
   de (E) **`HOTE`** (`KESH_INBOX_HOST_DIR`, `KESH_DOCUMENTS_HOST_DIR`, `KESH_LOG_HOST_DIR`) et
   **`MARIADB`** (les quatre `MARIADB_*`), écrites en dur comme la § *Inventaire fermé* (R4-11, C84 ;
   ni dérivées de `volumes:`, ni d'un préfixe) :
   - **`LUES`** — la liste des **41 variables lues**, écrite en dur, triée, avec en commentaire la commande
     `grep` qui la reproduit (§ *Inventaire fermé*) et le renvoi à la 15-11b, qui la **remplace** par la
     lecture du code. **Garde contre le test muet** : `LUES` compte **au moins 41** noms et contient
     `DATABASE_URL`, `KESH_JWT_SECRET`, `KESH_SMTP_HOST`, `KESH_PRODUCTION_RESET` et `RUST_LOG`.
     **Limite écrite** : une variable ajoutée au code **sans** être ajoutée à `LUES` n'est pas vue — angle
     mort jusqu'à la 15-11b (§ *Angles morts*), propriétaire : 15-11b.
   - **(T) Transmission** — `docker-compose.yml` et `docker-compose.prod.yml` analysés par un vrai
     analyseur YAML (`yaml-rust2`, dépendance de test, C72) : service `kesh-api`, clé `environment`, forme
     dictionnaire **ou** liste (`KEY=VAL`, et `KEY` sans `=`, qui équivaut à la clé sans valeur et
     **rougit** en (V) — F13). Pour **chaque** variable de `LUES` : présente dans les deux, **ou** dans
     `EXCEPTIONS` — `FixeeParImage` (contrôle : le `Dockerfile` porte `ENV <nom>=` après le dernier
     `FROM`, **et** la variable est absente des deux compose) ou `InterditeEnProduction` (contrôle : absente
     des deux compose). Toute clé transmise est dans `LUES` (une faute de frappe rougit). `env_file` sur
     `kesh-api` **rougit**, avec un message qui renvoie à C71. **Structure gardée** (R5) : le service
     `kesh-api` de `docker-compose.yml` porte `image:` commençant par `gcorbaz/kesh:` (AC5) ; dans
     `docker-compose.yml`, les trois montages de `kesh-api` vers `/var/log/kesh`, `/data/inbox` et
     `/data/documents` ont pour source respectivement `${KESH_LOG_HOST_DIR:-`, `${KESH_INBOX_HOST_DIR:-`,
     `${KESH_DOCUMENTS_HOST_DIR:-` ; dans `docker-compose.prod.yml`, ces trois montages ont pour source
     **exactement** `./log`, `./inbox`, `./documents` — toute autre source (interpolation comprise)
     **rougit**, avec un message qui renvoie à #558 et à C83 : rendre configurable un montage de P exige
     la procédure de déplacement des données de #558 (AC4). **Extraction de la source** (R4-12 = F4-8,
     C84) : en syntaxe courte, la cible est l'une des trois chaînes `/var/log/kesh`, `/data/inbox`,
     `/data/documents`, cherchée comme `:<cible>` suivie de la **fin** de l'entrée ou de `:ro`/`:rw` ; la
     source est tout ce qui **précède** ce `:<cible>` — jamais un découpage sur le premier `:`, qui
     tomberait dans le `:-` de `${KESH_LOG_HOST_DIR:-./log}`. Une entrée en **forme longue**
     (dictionnaire `type`/`source`/`target`) ou un montage de `kesh-api` dont la cible n'est pas trouvée
     **rougit** (« forme de montage non reconnue »), jamais ignoré ; chacune des trois cibles doit être
     trouvée **exactement une fois** par compose. (S) l'exerce : `${X:-./a}:/data/inbox` → source
     `${X:-./a}` ; `./a:/data/inbox:ro` → source `./a` ; forme longue → rouge.
   - **(V) Valeurs** — dans les deux compose, la valeur d'une variable transmise est l'une des formes :
     `${NOM:-}` (**forme des ajouts**), `${NOM:-défaut non vide}`, `${NOM}`, `${NOM:?…}`, un littéral
     scalaire non vide (chaîne, entier ou booléen YAML : `KESH_HOST: 0.0.0.0`, `KESH_PORT: 80` admis), où
     **`NOM` est la clé elle-même** — sauf `DATABASE_URL` de `docker-compose.yml` (composée de `MARIADB_*` :
     exception nommée). **Rougissent** : la **clé sans valeur** (`null` en dictionnaire, `- NOM` sans `=`
     en liste — message : « forme `${NOM:-}` ») ; une chaîne vide littérale (`''`) ; `${AUTRE…}` sous la
     clé `NOM`. **Vide signifiant** : la liste fermée `VIDE_SIGNIFIANT` (aujourd'hui
     `KESH_LOG_FILE_PATH`) porte, dans **les deux** compose, la forme **`${NOM-défaut}`** et **elle seule** ;
     `${NOM:-…}` pour une variable de la liste **rougit**, `${NOM-…}` pour une variable hors liste
     **rougit**. Contrôle de la liste : dans `.env.example`, pour **chaque** ligne d'affectation de `NOM`
     (R1-7 = F1-7 : la définition (E) peut en rendre plusieurs, dont une prose), les **cinq** lignes qui la
     précèdent sont **jointes en un texte** — `#` de tête et espaces de tête retirés sur
     chaque ligne, lignes jointes par une espace, espaces multiples réduits à une, comparaison sans casse
     (R3-8 = F-7 : le marqueur peut être coupé par un retour à la ligne du commentaire) — et ce texte
     contient « contrairement aux autres variables » **et ne contient pas** « ou absent » — chaque ligne
     d'affectation de `NOM` doit satisfaire le contrôle. **Sans défaut**
     (C76, C79) : la liste fermée `SANS_DEFAUT` (aujourd'hui `KESH_ADMIN_PASSWORD`) porte, dans **les deux**
     compose, la forme `${NOM:-}` et **aucune autre** (`${NOM}` rougit : R1-5). Contrôle de la raison : la
     variable est dans `LUES`, et **chacune** de ses lignes d'affectation dans `.env.example` est
     **commentée** (`^#\s*NOM=`). **Ajouts** (F1-8, C79) : la liste fermée `AJOUTS` — les **28** couples
     (variable, compose) de la § *Inventaire fermé* (15 pour Y, 13 pour P), écrite en dur avec en
     commentaire le renvoi à #550 — porte, **quand la clé est présente** dans ce compose, la forme `${NOM:-}`
     et **aucune autre** (son absence est le rouge de (T), pas de (V) : un seul rouge par défaut).
     Contrôle de la raison : chaque couple nomme une variable de `LUES` et l'un des deux compose ; deux
     couples identiques rougissent. *(La liste est historique, non dérivée : une story future qui veut un défaut de déploiement
     pour l'une de ces variables retire le couple de `AJOUTS`, et l'écrit.)*
   - **(E) `.env.example`** — une **ligne d'affectation** est une ligne qui satisfait
     `^#?\s*[A-Z][A-Z0-9_]*=` (F-1 : le NOM est en **majuscules** ; avec `[A-Za-z_]`, la prose
     `.env.example:275`, `# is_demo=true. NE PAS définir…`, propriété de la 15-7b2, deviendrait une
     affectation d'une variable inconnue et (E) resterait rouge en permanence). Chaque ligne d'affectation
     nomme une variable de `LUES`, ou de `HOTE` (contrôle de la raison, pour chaque entrée : `${NOM` apparaît
     dans un `volumes:` de `docker-compose.yml` — **seul** compose qui les honore, AC4 abandonnée, C83), ou
     de `MARIADB` (contrôle, pour chaque entrée : `${NOM` apparaît dans `docker-compose.yml`) ; une entrée
     de `HOTE` ou de `MARIADB` qui n'a aucune ligne d'affectation dans `.env.example` **rougit** (entrée
     inutilisée). Chaque variable de `LUES`
     y a une ligne d'affectation, sauf `KESH_TEST_MODE` — **exception unique**, contrôlée dans les deux
     sens : `.env.example` contient `DO NOT SET KESH_TEST_MODE`, **et** n'a **aucune** ligne
     `^#?\s*KESH_TEST_MODE=`.
   - **(F) Fantômes, corpus texte** — tout jeton `KESH_[A-Z0-9_]*[A-Z0-9]` de `.env.example`, des deux
     compose, des `messages.ftl` des **4 locales** et des `docs/manual/fr/*.tex` (après `\_` → `_`) est une
     variable de `LUES` ou de `HOTE`. Un jeton **immédiatement suivi de `_`** (`KESH_SMTP_*`,
     `KESH\_ADMIN\_*`) est un **préfixe** : il doit préfixer au moins une variable de `LUES` ou de `HOTE`.
     Pas de liste `PREFIXES_HORS_VARIABLES` : aucun préfixe hors variables n'est produit par ces corpus
     (vérifié par la lentille F de la P3) ; si un corpus en produit un jour, la liste se crée alors, avec
     contrôle de sa raison, et toute entrée inutilisée rougit. Le code Rust n'est **pas** un corpus de la
     15-11a (AC7, 15-11b).
   - **(S) Auto-test des extracteurs** sur sources synthétiques : YAML en forme liste et dictionnaire →
     mêmes clés ; clé sans valeur → rouge, en dictionnaire (`A:`) **et** en liste (`- A`) ; `${A:-}` sous
     `A` → vert ; `${B}` sous la clé `A` → rouge ; `${A-x}` pour `A` hors `VIDE_SIGNIFIANT` → rouge ;
     `${A:-x}` pour `A` dans `SANS_DEFAUT` → rouge ; `${A}` pour `A` dans `SANS_DEFAUT` → rouge ;
     `${A:-x}` pour un couple de `AJOUTS` → rouge ; `.env.example` synthétique où une variable de
     `VIDE_SIGNIFIANT` a **deux** lignes d'affectation (une prose sans marqueur, une ligne réelle avec) →
     rouge (R1-7) ; entier YAML `80` → vert ; `.env.example` synthétique
     dont le commentaire dit « VIDE ou absent → désactivé » au-dessus d'une variable de `VIDE_SIGNIFIANT`
     → rouge ; marqueur « contrairement aux \n# autres variables » coupé sur deux lignes → **trouvé** ;
     `# is_demo=true` → **pas** une ligne d'affectation ; `# blabla KESH_X à la fin` → pas une
     affectation, mais jeton vu par (F) ; `KESH\_FANTOME` dans un `.tex` synthétique → rouge (F).
   - **Limites de l'analyseur YAML, écrites** (`yaml-rust2`) : pas de clés de fusion `<<: *ancre` — une
     forme non reconnue fait **rougir**, jamais passer ; scalaires typés acceptés comme littéraux ;
     l'étape `docker compose config -q` de la CI (AC11) couvre la validité au sens de Compose.

9. **Mutations** (T5) — chacune appliquée, test exécuté, **rouge constaté avec le message attendu**,
   puis restaurée (`git diff --stat` vide) ; résultat au Dev Agent Record :
   M1 retirer `KESH_SMTP_HOST` de `docker-compose.prod.yml` (T) · M2 le retirer de `docker-compose.yml`
   seul (T) · M3 ajouter `KESH_ESSAI_MUTATION` à `LUES` — simule une variable ajoutée au code (rouge
   **trois fois** : (T) non transmise par Y, (T) par P, (E) sans ligne d'affectation — R3-6) ·
   M4 transmettre `KESH_STATIC_DIR` dans `docker-compose.prod.yml` (T) · M5 retirer `ENV KESH_LOCALES_DIR`
   du `Dockerfile` (T, contrôle de `FixeeParImage`) · M6 ajouter `KESH_TEST_MODE: ${KESH_TEST_MODE:-}` à
   `docker-compose.yml` (T) · M7 ajouter `#KESH_OBSOLETE=1` à `.env.example` (rouge **deux fois** : (E) et
   (F)) · M8 écrire dans la prose qui remplace l'ancienne `.env.example:235` (AC6 e, R1-8) le fantôme `KESH_ADMIN_` + `RESET` en un jeton (F)
   · M9 `KESH_SMTP_PORT:` (clé sans valeur, V) · M10 `KESH_SMTP_HOST: ${KESH_SMTP_HSOT:-}` (V) ·
   M11 `env_file: .env` sur `kesh-api` (T) · M12 retirer `KESH_LANG=fr` de `.env.example` (E) ·
   M13 retirer `image:` du service `kesh-api` de `docker-compose.yml` (T) · M14 remplacer,
   dans `docker-compose.yml`, `${KESH_LOG_HOST_DIR:-./log}:/var/log/kesh` par `./log:/var/log/kesh`
   (rouge **deux fois** : (T) structure, et (E) — `${KESH_LOG_HOST_DIR` absent des volumes de Y ; réécrite
   en P3, C83) · M15 écrire
   `KESH_LOG_FILE_PATH: ${KESH_LOG_FILE_PATH:-/var/log/kesh/kesh.log}` (deux-points) dans
   `docker-compose.yml` (V, vide signifiant) · M16 remettre `KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD:-changeme}`
   dans `docker-compose.yml` (V, `SANS_DEFAUT`) · M17 remettre au-dessus de `KESH_LOG_FILE_PATH` le
   commentaire « VIDE ou absent → logs fichier désactivés » (V, contrôle de `VIDE_SIGNIFIANT`) ·
   M18 écrire `KESH\_ESSAI\_FANTOME` dans un paragraphe de `admin-manual.tex` (F, corpus `.tex`) ·
   M19 décommenter `KESH_ADMIN_PASSWORD=` dans `.env.example` (rouge **deux fois**, dans deux binaires :
   (V), contrôle de la raison de `SANS_DEFAUT` ; et `config_rejects_admin_password_left_at_template_placeholder`
   de l'AC16 c, dont l'aide ne trouve plus **aucune** ligne `#KESH_ADMIN_PASSWORD=` en colonne 0 —
   R4-6) ·
   **M20** retirer de `Config::from_env` l'appel à `is_template_placeholder` du contrôle de
   `KESH_JWT_SECRET` (au moins `config_rejects_jwt_secret_left_at_template_placeholder` et
   `…_jwt_secret_generate_me_case_insensitive` rouges : le secret du gabarit est accepté ; aussi
   `config_rejects_jwt_secret_and_admin_password_in_angle_brackets`, dont le cas `<x>` rend
   `WeakJwtSecret` — F4-7 ; le Dev Agent Record nomme les tests réellement rouges) · **M21** écrire `KESH_SMTP_PORT: ${KESH_SMTP_PORT:-587}` dans
   `docker-compose.prod.yml` (V, `AJOUTS`) · **M22** remettre `KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD}`
   dans `docker-compose.prod.yml` (V, `SANS_DEFAUT`) · **M23** remplacer, à `.env.example:88`, le placeholder
   par `REMPLACER_MOI: openssl rand -hex 32` (35 caractères, ni `GENERATE_ME`, ni `change-me`, ni forme
   `<…>` — R4-5, C84 : l'ancien texte de mutation, `<A_GENERER: …>`, est de la forme `<…>` et serait
   **refusé** depuis F3-4, si bien que seule l'assertion de montage l'aurait vu). Rouge de
   `config_rejects_jwt_secret_left_at_template_placeholder` : l'assertion « la valeur extraite contient
   `GENERATE_ME` » de l'AC16 c rougit **la première**, avec un message qui nomme le gabarit ; sans elle, le
   refus attendu manquerait aussi (la valeur est acceptée) — preuve que le test suit le gabarit au lieu
   d'une copie · **M24** (R2-3 = F2-1) retirer l'appel à `is_template_placeholder` du
   contrôle de `KESH_ADMIN_PASSWORD` (tests `config_rejects_admin_password_left_at_template_placeholder`
   et `…_admin_password_generate_me_case_insensitive` rouges : le placeholder est accepté) · **M25**
   (F2-8) redescendre le contrôle des placeholders du secret JWT **après** le contrôle de longueur (test
   `…_jwt_secret_generate_me_case_insensitive` rouge sur le secret court `GENERATE_ME` :
   `WeakJwtSecret` au lieu d'`InsecureJwtSecret` ; et le secret court `xchange-mex` : `WeakJwtSecret` au
   lieu d'`InsecureJwtSecret` — le contrôle `change-me` remonte avec celui des placeholders, F3-8) ·
   **M26** (F3-4) retirer de la fonction d'aide `is_template_placeholder` la branche « forme `<…>` » (test
   `config_rejects_jwt_secret_and_admin_password_in_angle_brackets` rouge : les gabarits du manuel sont
   acceptés, pour les deux variables) · **M27** (R3-7 = F3-8) redescendre le contrôle des placeholders du
   mot de passe admin **après** le contrôle `< 12` (test `…_admin_password_generate_me_case_insensitive`
   rouge sur `generate_me` seul : `WeakAdminPassword` au lieu d'`InsecureAdminPassword`) · **M28** (C83)
   écrire `${KESH_DOCUMENTS_HOST_DIR:-./documents}:/data/documents` dans `docker-compose.prod.yml` ((T)
   structure : montage fixe exigé, message qui renvoie à #558).
   **28 mutations, toutes rouges.** M1-M19, M21, M22, M23 et M28 ne touchent aucun `.rs` de production
   (le test `configuration_transmise` ne lit pas le code ; M19 et M23 touchent `.env.example`, que les
   tests de l'AC16 lisent par `include_str!` — F4-7) ; **M20, M24, M25, M26 et M27 touchent `config.rs`**
   (seul changement de comportement de la story, AC16) ; M3 touche le test lui-même, qui se recompile —
   restauration suivie d'un `touch` du fichier de test avant le run suivant ; M19, M20, M23, M24 à M27 :
   restauration suivie d'un `touch` de `config.rs` (la lecture du gabarit par `include_str!` est
   recompilée, mais le `touch` lève le doute — mémoire « mutation restaurée, binaire périmé »).

10. **Vérification par `docker compose config`, sans démarrer** (T6). Dans un répertoire d'essai du
    scratchpad, `env -i PATH="$PATH" HOME="$HOME" docker compose -f <fichier> config --format json` pour
    les deux compose (`env -i` : Compose fait passer le **shell** avant `.env`, R2-7), quatre `.env` :
    (i) **complet** — les **38 noms** transmis, chacun à une valeur reconnaissable (`ESSAI_<NOM>`, ou une
    valeur valide quand Compose l'exige) : le service `kesh-api` porte les 38 clés avec la valeur posée,
    **sauf** `KESH_HOST` dans P (littéral `0.0.0.0`) et `DATABASE_URL` dans Y (composée des `MARIADB_*`) ;
    (ii) **absent** : les 38 clés sont **présentes** dans les deux ; dans Y, **22** valeurs non vides (les
    23 existantes, toutes à défaut écrit, moins `KESH_ADMIN_PASSWORD`, AC2 i) et **16** chaînes vides (les
    15 ajouts + `KESH_ADMIN_PASSWORD`) ; dans P, **22** non vides (25 existantes moins `DATABASE_URL` et
    `KESH_JWT_SECRET`, en `${X}` sans défaut, que Compose rend vides avec un avertissement, et
    `KESH_ADMIN_PASSWORD`, en `${X:-}` depuis l'AC2 i, vide **sans** avertissement) et **16** vides
    (13 ajouts + ces 3) — **aucune valeur `null`** ; les avertissements de Compose sont **comptés** : deux
    sur P (`DATABASE_URL`, `KESH_JWT_SECRET`), aucun sur Y ; (iii) **lignes vides**
    — `.env` portant `KESH_SMTP_PORT=` et `KESH_LOG_FILE_PATH=` : `KESH_SMTP_PORT` vaut `""` dans les deux
    (le code avertit puis applique 587 ; après la 15-11b, 587 sans avertissement), `KESH_LOG_FILE_PATH`
    vaut `""` dans les deux (journal fichier désactivé) ; sans la ligne, `KESH_LOG_FILE_PATH` vaut
    `/var/log/kesh/kesh.log` ; (iv) **chemins d'hôte** (réécrit en P3, C83) — `.env` portant
    `KESH_DOCUMENTS_HOST_DIR=/essai/documents` : le montage de `/data/documents` a pour source
    `/essai/documents` dans Y et **reste** `<répertoire d'essai>/documents` dans P (montage fixe, AC4) ;
    sans la ligne, `./documents` résolu dans les deux. Comptes **recomptés** depuis la sortie JSON
    (commande au Dev Agent Record).
    `docker-compose.prod.yml` exige le réseau externe `frontend` au démarrage, pas à `config`. **Aucun
    `up`, aucun conteneur** (sauf le `--dry-run` de l'AC5, qui n'en crée pas).

11. **CI.** Le job `docker-build` de `.github/workflows/ci.yml` gagne une étape
    `docker compose -f docker-compose.yml config -q` et `docker compose -f docker-compose.prod.yml config -q`
    (validation par Compose lui-même : clé inconnue, interpolation mal formée, clé en double). Variables
    obligatoires non posées : avertissement, pas d'échec — à vérifier en local (T6) ; si `config -q` échoue
    sur une variable absente, poser un `.env` d'essai minimal dans l'étape. **`docs/ci.md`** (F-8) : `:12`
    et `:117`, qui décrivent `docker-build` comme une simple sanité du `Dockerfile`, disent aussi la
    validation des deux compose.

12. **Manuel d'administration** (`docs/manual/fr/admin-manual.tex` + PDF régénéré) :
    (a) `:648` — la phrase « Le fichier `.env` est chargé automatiquement par Docker Compose » devient
    la règle exacte : Compose lit `.env` pour remplir le compose ; **seules les variables listées sous
    `environment:` atteignent Kesh** ; les compose fournis les listent toutes, sauf `KESH_STATIC_DIR` et
    `KESH_LOCALES_DIR` (fixées par l'image) et `KESH_TEST_MODE` (jamais en production) ; une modification
    de `.env` s'applique par `docker compose up -d` (**pas** `docker compose restart`) ; `KESH_LOG_FILE_PATH`
    laissée vide désactive le journal fichier ; **une valeur qui contient `$` se met entre apostrophes
    simples** (`KESH_SMTP_PASSWORD='pa$word'`), notamment pour `KESH_SMTP_PASSWORD` — sinon Compose la
    tronque (F1-4, § *Le défaut*) ; `$$` n'est cité qu'en second, comme échappement **propre à Compose**
    (un `cargo run` hors Docker lit `pa$$word` comme `pa` — R4-13, C84), avec l'avertissement que
    `docker compose config` **réaffiche** tout `$` en `$$` (ce n'est pas la valeur reçue). *(La phrase générale « une ligne vide vaut une ligne
    absente » est ajoutée par la 15-11b.)*
    (b) `:678-679` — défauts de `KESH_STATIC_DIR` / `KESH_LOCALES_DIR` : la précision « fixé par l'image
    Docker (`/app/static`, `/app/locales`) ; `frontend/build` / dossier du dépôt hors Docker » va dans la
    colonne **Description** (`X`), **pas** dans « Défaut » (`l`, qui déborde déjà — F6) ; contrôle : le
    texte de la cellule se lit **entier** dans le PDF aplati — contrôle tenable une fois la mise en page
    des tableaux corrigée (j), et pas avant (F2-3).
    (c) `:783-784` — la **phrase d'introduction** du tableau des chemins d'hôte dit qu'il vaut pour
    `docker-compose.yml` **seul** : avec `docker-compose.prod.yml` (installation Synology du manuel), les
    montages sont fixes — `./inbox`, `./documents`, `./log` dans le répertoire du compose — et ces trois
    variables sont **sans effet** (C83 ; une version configurable est suivie par l'issue #558). Le
    **texte** du `\paragraph{…}` `:782` n'est pas modifié (R2-8) ; sa mise en page l'est, avec celle des
    neuf autres (j). **Propagation** (sites qui nomment `KESH_DOCUMENTS_HOST_DIR` comme source du dossier
    des documents) : `:1384` (« monté depuis `KESH_DOCUMENTS_HOST_DIR` ») et `:1518` (Hyper Backup,
    « y compris le répertoire des documents (`KESH_DOCUMENTS_HOST_DIR`) ») disent « `./documents` avec
    `docker-compose.prod.yml` ; avec `docker-compose.yml`, le dossier de `KESH_DOCUMENTS_HOST_DIR` s'il est
    posé — à ajouter alors à la tâche de sauvegarde ». *(F3-3 — PDF figés sortis de la sauvegarde par le
    déplacement — **disparaît** avec l'AC4 : aucun dossier ne change de place. Ce qui reste est la précision
    pour `docker-compose.yml`, situation antérieure à la story.)*
    (d) `:218-224` (*Étape 2*) — le fichier téléchargé tire l'image publiée ; une phrase.
    (e) `:236-254` (*Étape 3*) — avec `docker-compose.yml`, `DATABASE_URL` est **composée** des
    `MARIADB_*` par le compose : la ligne `DATABASE_URL` de `.env` y est ignorée (elle sert
    `docker-compose.prod.yml`). Et (F-10, R1-2) : **`KESH_JWT_SECRET` est obligatoire et se génère** —
    *« générez le secret (`openssl rand -hex 32`) et collez-le à la place du placeholder ; vérifiez :
    `grep KESH_JWT_SECRET .env` ne doit montrer ni `GENERATE_ME`, ni `change-me`, ni valeur entre chevrons
    `<…>` »* — **relecture à l'œil** de ce que l'exploitant vient d'écrire, non une vérification de l'état
    effectif, qui est `docker compose config` (AC12 f) ; la fiche le dit pour qu'un relecteur n'« harmonise »
    pas les deux dans un sens ou dans l'autre (F4-9) ; sans secret, Kesh refuse
    de démarrer, y compris avec `docker-compose.yml`, dont le défaut `change-me…` est refusé à dessein, et
    le placeholder `<GENERATE_ME: …>` laissé tel quel est refusé depuis la 0.13.0 (AC16) ; `KESH_ADMIN_*`
    restent optionnelles (onboarding `/setup`). Trois phrases et la commande de vérification ; l'exemple
    garde ses lignes, et sa ligne `:240` (`<secret aleatoire, min. 32 octets, sans "change-me">`) dit
    « sans `change-me` ni `GENERATE_ME` ». **Gabarits entre chevrons** (F3-4, AC16) : les lignes `:243-244`
    (`KESH_ADMIN_USERNAME`, `KESH_ADMIN_PASSWORD`) passent **en commentaire**, sous « optionnel — sans
    elles, l'administrateur se crée à l'écran `/setup` » (le texte d'introduction « au minimum les
    variables suivantes » cesse de les compter) ; avant l'exemple, une phrase : *« Remplacez chaque valeur
    entre chevrons `<…>` : recopiée telle quelle, elle fait refuser le démarrage (depuis la 0.13.0) pour
    `KESH_JWT_SECRET` et `KESH_ADMIN_PASSWORD` ; pour les mots de passe MariaDB, rien ne la refuse —
    choisissez-les vous-même. »* (F4-5 : `MARIADB_ROOT_PASSWORD=<mot de passe fort>` et
    `MARIADB_PASSWORD=<…>` de l'exemple, MariaDB les accepte ; angle mort écrit, propriétaire #551.) La même consigne (« remplacez la valeur entre
    chevrons ») accompagne `:988` (`KESH\_ADMIN\_PASSWORD=<mot-de-passe-fort-12-chars>`, création
    déclarative) et `:1230` (`<nouveau-mdp-12+>`, break-glass). Les quatre exemples **restent des
    gabarits manifestes** (inventaire fermé de la classe :
    `grep -nE 'KESH(\\)?_(JWT(\\)?_SECRET|ADMIN(\\)?_PASSWORD)=' docs/manual/fr/*.tex` rend `:240`, `:244`,
    `:988`, `:1230` — à refaire au T8, tout site neuf lu).
    **Installation Synology** (`:530-548`, R2-3 = F2-1, R2-4, F2-6) : le commentaire `:537` (« Générer les
    secrets obligatoires (DOIVENT remplacer les placeholders dans .env) ») est **faux pour le mot de
    passe admin** — il devient « secrets **obligatoires** : `KESH_JWT_SECRET`, `DATABASE_URL` (et les
    mots de passe de votre MariaDB) » (R3-11 : cette installation emploie `docker-compose.prod.yml`, pour
    lequel les `MARIADB_*` sont sans objet, et la variable obligatoire est `DATABASE_URL`) ; la ligne
    `:539` (`openssl rand -base64 24 # → KESH_ADMIN_PASSWORD`) passe sous un commentaire « **optionnel** :
    sans `KESH_ADMIN_PASSWORD`, l'administrateur se crée à l'écran `/setup` ; si vous décommentez la ligne,
    remplacez son placeholder » ; l'encadré `keshwarning` `:546-548` (« ne jamais laisser `changeme` ou
    `change-me-32-bytes-...` ») nomme aussi `<GENERATE_ME: …>` et toute valeur entre chevrons, pour les
    deux variables.
    **Tableaux de `sec:env-vars`** (F2-3, R2-4) : la ligne `KESH_JWT_SECRET` (`:662`, « Rejeté s'il
    contient `change-me` ») devient « Rejeté s'il contient `change-me` ou `GENERATE_ME` (sans égard à la
    casse), ou s'il est un gabarit entre chevrons `<…>` » ; la ligne `KESH_ADMIN_PASSWORD` (`:690`, « refusé
    si `changeme` ») devient « refusé s'il vaut `changeme`, contient `GENERATE_ME` ou est un gabarit entre
    chevrons `<…>` ». Zone partagée avec la 15-11b (trim de `:662`, relocalisation
    par le texte).
    (f) `:1704-1714` (*Procédure de mise à jour standard*), point 3 — remplacer « si nécessaire » par
    une consigne datée. *(Réécrite en P3, C83 : l'encadré « déplacez d'abord vos dossiers » et la recette
    de déplacement sont **retirés** avec l'AC4 — les montages de `docker-compose.prod.yml` ne changent
    pas, aucune donnée n'a à être déplacée. Les findings de la P3 qui portaient sur cette recette
    **disparaissent** avec elle : R3-1 = F3-1 (lecture de `.env` qui n'imite pas Compose), F3-2 (`diff -rq`
    et destination non vide), R3-2 (largeur des lignes dans l'encadré), R3-8, R3-9 = F3-6, R3-10, F3-7 ;
    F3-3 (PDF figés sortis de la sauvegarde) aussi, faute de déplacement ; de même la nuance sur le message
    `error-invoice-pdf-gone` (F2-7), qui n'avait de sens qu'après un montage déplacé.)*
    **Pour passer à 0.13.0, re-télécharger le compose** (`docker-compose.yml` ou, sur Synology,
    `docker-compose.prod.yml` enregistré sous `docker-compose.yml`, `:534`) **et y reporter ses seules
    adaptations locales** (port `8080:80`, réseau, chemins des montages) — **voie recommandée**, présentée
    en premier ; le manuel dit la conséquence d'un oubli sur les montages (F4-6) : un montage modifié à la
    main et non reporté revient à `./documents` (ou `./inbox`, `./log`), et justificatifs et PDF
    paraissent disparus — ils sont toujours dans l'ancien dossier ; les commandes s'exécutent depuis le répertoire du compose (`/volume1/docker/kesh` sur
    Synology ; avec un autre nom de fichier, `-f <son fichier>` à chaque commande `docker compose` — R2-5) ;
    puis **`docker compose pull`** avant `up -d` : le compose de la 0.13.0 s'emploie avec l'image 0.13.0
    (F-5) ; et **la vérification des placeholders** (AC16, #557), **demandée à Compose lui-même**, le
    compose final en place — Compose lit `.env` à sa façon (`export`, espaces autour de `=`, guillemets,
    `\r`), qu'aucune expression régulière écrite sur `.env` n'imite sûrement (R3-13 = F3-11 ; c'est la
    leçon de R3-1 = F3-1) :
    ```sh
    sudo docker compose config \
      | grep -iE 'KESH_(JWT_SECRET|ADMIN_PASSWORD): .? *(<|.*generate_me)'
    ```
    doit être **muet** — s'il montre `KESH_JWT_SECRET`, générer le secret (`openssl rand -hex 32`) **avant**
    le `up -d` (les sessions ouvertes sont alors perdues, chacun se reconnecte) ; s'il montre
    `KESH_ADMIN_PASSWORD`, choisir un vrai mot de passe (≥ 12 caractères) ou recommenter la ligne
    (onboarding `/setup` sur base vide, sans effet sur une base qui a déjà un administrateur) — faute de quoi
    Kesh refuse de démarrer. Le manuel dit que la commande affiche la ligne trouvée (dans le cas qu'elle
    détecte, un placeholder, non un secret) et qu'elle est un **sur-ensemble** du refus de l'AC16 (une
    valeur qui commence par `<` sans finir par `>` y apparaît, de même qu'un caractère puis des espaces
    puis `<` ; Kesh les accepte). **Motif corrigé en P4** (R4-14, C84) : `.? *` au lieu de `.?` — sur
    `KESH_ADMIN_PASSWORD="   <x>  "`, Compose rend `'   <x>  '` et l'ancien motif, qui ne tolérait qu'un
    caractère avant `<`, restait **muet** alors que Kesh (trim) refuse. Rejoué au scratchpad le 2026-10-08
    (Compose 2.40.3, `env -i`) sur dix-sept `.env` : quatorze cas piégés **montrés** — ceux de la lentille
    R de la P4 (`export `, `KEY = v`, guillemets doubles, apostrophes, espace de tête, commentaire de fin
    de ligne, `\r\n`, `<a>`, exemples du manuel, `generate_me-et-12-caracteres`), plus
    `KESH_ADMIN_PASSWORD="   <x>  "`, `KESH_JWT_SECRET='   <x>  '` et `"  <GENERATE_ME: x>"` — ; trois
    témoins (64 hexadécimaux, `pw<valide>12chars`, secret vide) **muets**. Deux lignes de moins de
    76 caractères (seconde ligne : 70) (le style `kesh` replie les lignes longues, `breaklines = true`, `kesh-style.sty:224`).
    **Rejouée au T6** contre Compose, sur `.env` piégés. **Pour qui garde son fichier**, **deux gestes
    distincts** (le troisième — remplacer les montages de P — est retiré avec l'AC4, C83), un jeu par
    compose, recopiés **du compose final**, dans une sous-section, `lstlisting` sans retour de ligne coupé
    (R2-4) :
    1. **ajouter** sous `environment:` du service `kesh-api` les lignes **nouvelles** — les 15 de
       `docker-compose.yml`, les 13 de `docker-compose.prod.yml` — et elles seules ;
    2. **remplacer** les lignes **existantes** qui changent de forme : `KESH_LOG_FILE_PATH` (les deux
       compose) et `KESH_ADMIN_PASSWORD` (les deux compose — R1-5) — ligne remplacée, jamais dupliquée.
    Puis **`docker compose config -q`** (refuse une clé en double), contrôle que `docker compose config`
    montre `KESH_SMTP_HOST` et `KESH_PRODUCTION_RESET`, et `docker compose up -d`. **Avertissement, en
    encadré, avant le `up -d`** : *« Relisez votre `.env` avant de mettre à jour : des lignes jusqu'ici sans
    effet le deviennent. »* La liste est **fermée** ; elle est tirée de **trois** sources, relevées de
    nouveau au T0 et dont l'écart s'écrit : (1) les variantes de `ConfigError` que
    `Config::from_env` peut rendre sur une variable **nouvellement transmise**, ou que la story **ajoute ou
    étend** (`config.rs:15-100` ; AC16 étend `InsecureJwtSecret` et `InsecureAdminPassword`) ; (2) **les
    `std::process::exit` de `main.rs` atteignables par une variable nouvellement transmise** (R3-2) ;
    (3) **les interpolations du compose nouvellement prises en compte ou dont la forme change** (R1-1 =
    F1-1) — inventaire fermé, tiré du diff des deux compose (`git diff main -- docker-compose.yml
    docker-compose.prod.yml`, relu au T8 contre cette liste) : `KESH_LOG_FILE_PATH` (AC2 ii) et
    `KESH_ADMIN_PASSWORD` (AC2 i, les deux compose) — **plus aucun montage** (AC4 abandonnée, C83 : le
    diff ne touche aucune ligne de montage — seul le commentaire du T2 s'ajoute au bloc `volumes:` de P —,
    ce que le T8 vérifie ; R4-10) ; les 28 ajouts relèvent des sources (1) et (2)
    et des effets ci-dessous :
    - **refus de démarrer** — le message nomme la variable, à lire par `docker compose logs kesh-api` :
      (i) `KESH_SMTP_TLS` ou `KESH_FEATURE_FORGOT_PASSWORD` d'une valeur autre que `true`/`false`/`1`/`0`
      (espaces de tête et de fin ignorés) — `ConfigError::InvalidBoolValue`, sur les **deux** compose,
      **même sans** le mot de passe oublié pour `KESH_SMTP_TLS` (lu inconditionnellement) ;
      (ii) `KESH_COOKIE_SECURE` d'une valeur autre que `true`/`false`/`1`/`0`, sur
      `docker-compose.prod.yml` où il était ignoré — `InvalidCookieSecureValue` ; (iii)
      `KESH_FEATURE_FORGOT_PASSWORD=true` avec l'une des cinq `KESH_SMTP_HOST`/`_USER`/`_PASSWORD`/`_FROM`,
      `KESH_PUBLIC_BASE_URL` absente — **ou** un `KESH_SMTP_FROM` mal formé (display-name « Nom <email> »)
      — **ou** un `KESH_SMTP_HOST` au format `hôte:port` — `IncompleteSmtpConfig` ; (iv) **R3-2** :
      `KESH_FEATURE_FORGOT_PASSWORD=true` avec une configuration SMTP complète, qui passe
      `Config::from_env`, mais dont le **mailer ne se construit pas** — adresse `KESH_SMTP_FROM` refusée
      par `lettre` (`mail/smtp.rs:56`), échec de l'initialisation STARTTLS de l'hôte (`:73-74`) : Kesh
      s'arrête (`main.rs:316-323`, `process::exit(1)`) sur le message « `KESH_FEATURE_FORGOT_PASSWORD=true`
      mais le build du mailer SMTP a échoué (…) — abandon ». *(Sans le mot de passe oublié, le même échec
      dégrade seulement : envoi de factures indisponible, démarrage maintenu — `main.rs:324` et suiv.)* ;
      (v) **AC16** : `KESH_JWT_SECRET` resté au placeholder `<GENERATE_ME: …>` du gabarit ou à un gabarit
      entre chevrons `<…>` — `InsecureJwtSecret` — ou `KESH_ADMIN_PASSWORD` décommenté sans remplacer le
      sien, ou recopié d'un exemple du manuel (`<mot de passe fort, …>`, `<nouveau-mdp-12+>`) —
      `InsecureAdminPassword` ;
    - **effets sans refus** : `KESH_PRODUCTION_RESET` posé d'une ancienne démonstration → **sur une
      installation restée en démonstration** (F1-5 : `routes/onboarding.rs:265-279` refuse de toute façon
      la réinitialisation d'une installation de production passée l'étape 2), la réinitialisation, qui vide
      toutes les données de la société, redevient permise ; les **quatre**
      `KESH_SMTP_HOST`/`_USER`/`_PASSWORD`/`_FROM` posées « pour plus tard » → **l'envoi de factures par
      e-mail s'active** (`smtpConfigured: true`) ; **R3-2** : la même configuration avec
      `KESH_FEATURE_FORGOT_PASSWORD=true` (et `KESH_PUBLIC_BASE_URL`) → **la réinitialisation du mot de
      passe par e-mail s'active** (`main.rs:296-299`, « recovery email ACTIVÉ ») — fonction sensible :
      quiconque connaît l'adresse d'un utilisateur peut demander un lien ; `KESH_COOKIE_SECURE=false` sur
      `docker-compose.prod.yml` → cookies sans `Secure` ; `KESH_ADMIN_BACKUP_DIR` (un chemin non monté y
      fait écrire dans le conteneur) ; `KESH_LANG` ; `KESH_PASSWORD_MIN_LENGTH` sur `docker-compose.yml`
      → s'applique côté serveur, alors que **trois** écrans gardent 12 en dur (F-6 : `SetupForm.svelte:39`,
      `ResetPasswordForm.svelte:28`, `routes/(app)/users/+page.svelte:16` — création et réinitialisation
      d'un utilisateur) : **au-dessous de 12, les écrans refusent quand même** (ils bloquent avant d'appeler
      le serveur — F1-3 : `SetupForm.svelte:47`, `ResetPasswordForm.svelte:43`, `users/+page.svelte:105,
      239`) ; **au-dessus de 12, l'écran accepte et le serveur refuse** (400) ; **source 3** : une ligne
      `KESH_LOG_FILE_PATH=` **vide** (l'opt-out documenté, jusqu'ici sans effet) **désactive désormais** le
      journal fichier ; et plus généralement toute ligne décommentée des 16 variables ;
    - **constat rassurant, écrit** : un `.env` copié du gabarit **sans modification** ne change pas de
      comportement — les seules lignes actives du gabarit parmi les 16 valent leur défaut
      (`KESH_LANG=fr`, `KESH_PASSWORD_MIN_LENGTH=12`, `KESH_BANK_IMPORT_MAX_MB=10`, et, depuis v0.1.3,
      `KESH_COOKIE_SECURE=true` — `git log -p -- .env.example`, à refaire au T0) — **sauf**, voulu, le
      secret JWT laissé au placeholder, désormais refusé (refus (v)) ; source 3 : `KESH_ADMIN_PASSWORD`
      absente ou vide ne change rien sur P (vide avant, vide après) et cesse de faire refuser Y (AC13).
    Vérification fonctionnelle : `curl …/health` → `smtpConfigured: true` si le SMTP est posé.
    (g) PDF régénéré (`make fr` dans `docs/manual/`), contrôlé **aplati** — et, pour **tout** contrôle
    sur PDF aplati de cette fiche (b, g, h, j), **ligatures normalisées** après l'aplatissement (F4-4,
    C84 : le PDF actuel porte des `ﬀ`, U+FB00, qu'un `grep` sur « sans effet », « affiche » ou
    « différent » ne verrait pas) :
    `pdftotext admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬃ/ffi/g; s/ﬄ/ffl/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'`
    (même normalisation au `pdftotext -layout` non aplati) contient les phrases nouvelles de (a) (dont
    « apostrophes simples » et `$$`), (e) (dont `grep KESH_JWT_SECRET .env`, « Rejeté s'il contient » suivi de `change-me` et de
    `GENERATE_ME`, « gabarit entre chevrons », « remplacez chaque valeur entre chevrons », « optionnel »
    pour `KESH_ADMIN_PASSWORD` à l'installation Synology) et (f) (dont « re-télécharger le compose », « deux gestes »
    et « chemins des montages »), « sans effet » pour les `KESH_*_HOST_DIR` (c), et ne contient plus `docker compose restart kesh-api` (AC15) ; les **deux lignes** de
    la commande de vérification des placeholders (f) se retrouvent **chacune sur une ligne** de
    `pdftotext -layout admin-manual.pdf -` **non aplati** (espaces de tête retirés et espaces multiples
    réduits **des deux côtés** avant la comparaison) — l'aplatissement restaure en espace un repli de
    `breaklines` et ne le verrait pas (leçon de R3-2, F3-7) ; dans le `.log`, **zéro** `Overfull \hbox`
    dans les **dix tableaux** de `sec:env-vars` (bornes de (j)), et aucun `Overfull \hbox` **nouveau**
    ailleurs (comparer au `.log` d'avant, relevé au T8 sur un `make fr` de l'état d'avant — le `.log` est
    **local, non versionné** (`.gitignore:32`, R3-5) et peut dater d'avant le dernier `.tex`).
    (h) `user-manual.tex` : aucune modification attendue — **à vérifier** par grep (`SMTP`, `KESH\\_`,
    `démonstration`) et à écrire au Dev Agent Record. **Brochure** (F2-11) : `marketing-brochure.tex:542`
    (« lancer une instance locale en moins de 5 minutes avec `docker compose up` ») est **inexacte** —
    un clone sans `.env` ne démarre pas (secret JWT obligatoire, défaut de Y refusé, F-10) et, depuis
    l'AC5, `up` tire l'image publiée ; elle devient « … avec `docker compose up -d`, une fois
    `KESH_JWT_SECRET` généré dans `.env` (voir le manuel d'administration) » ; PDF régénéré par le même
    `make fr`, contrôlé aplati.
    (i) **Synology Container Manager** (méthode GUI, `:552-566`) : l'action de l'interface qui recrée le
    conteneur et relit `.env` n'est **ni nommée ni mesurée** (F12) — le manuel renvoie, pour appliquer une
    modification de `.env` ou du compose, à `sudo docker compose up -d` par SSH (comme `:639`) ; au Dev
    Agent Record : « non mesuré ».
    (j) **Mise en page des dix tableaux de `sec:env-vars`** (F2-3 ; décision de l'orchestrateur en P2 :
    **reprise de la 15-7b2**, T7 et C-15-7-52 de sa fiche, worktree `kesh-15-7` — **la 15-7b2 n'a plus à le
    faire**, l'orchestrateur ajuste sa fiche). **Le défaut** : chacun des dix tableaux suit un
    `\paragraph{…}` **en ligne** (`\titleformat{\paragraph}[runin]`, `docs/manual/shared/kesh-style.sty:141`),
    qui pousse le `tabularx` de toute sa largeur vers la droite : le défaut est le titre en ligne, non le
    tableau. Relevé de la 15-7b2 (base `b74c3dac`), retrouvé ici au `.log` **local** (non versionné,
    `.gitignore:32` ; 2026-10-07 — R3-5) —
    `Overfull \hbox` aux lignes `650--655`, `664--670`, `682--683`, `693--694`, `709--710`, `722--723`,
    `734--735`, `747--748`, `750--772`, `780--781`, `783--794` — et au PDF aplati
    (`pdftotext -layout`, F2-3) : la ligne `KESH_JWT_SECRET` s'arrête à « `openssl rand -hex 32 .`
    Rejeté », `grep -c "Rejeté s'il contient"` rend **0**, `DATABASE_URL` s'arrête à « mys ». **Les dix
    tableaux sont rognés** (R3-4, relevé P3 au `pdftotext -layout` : le constat « huit rognés, deux
    décalés mais entiers », recopié de la 15-7b2, était faux) — dont « Serveur \& réseau », qui perd
    `build` de `frontend/build`, et « Chemins host des bind mounts », qui perd toute sa colonne « Défaut
    host ». **La correction** : les dix
    par **un même geste**, titre hors ligne — `\paragraph{…}\mbox{}\\` ou `\subsubsection*{…}`, l'un
    des deux retenu **après essai** (celui qui supprime l'`Overfull` sans changer la table des matières ni
    la numérotation ; le choix et sa raison au Dev Agent Record) —, **dans la section seule** : le
    `\titleformat{\paragraph}` de `kesh-style.sty` **n'est pas modifié** (le manuel d'administration et
    le manuel utilisateur emploient `\paragraph` ailleurs, en ligne, sans tableau — 38 et 10 occurrences ;
    la brochure aucune, R3-12). Le **texte** des dix titres ne change pas.
    **Contrôle** : au `.log`, zéro `Overfull \hbox` dont les lignes tombent entre `\label{sec:env-vars}`
    et `\subsubsection{Import de factures depuis un dossier}` (`sec:inbox-import`) — c'est-à-dire dans les
    dix tableaux et leurs titres (R3-3 = F3-5 : la borne `sec:logs-fichier` englobait le
    `\paragraph{Note Synology (chemins symboliques).}` de `sec:inbox-import`, `Overfull` des lignes
    `821--825`, paragraphe de texte que le geste ne touche pas ; **tranché** : il reste hors du périmètre
    de (j), l'abandon de l'AC4 ne le touchant pas non plus — défaut **antérieur**, couvert par « aucun
    `Overfull` nouveau ailleurs » de (g), et écrit au Dev Agent Record comme préexistant) ; au PDF aplati, **chaque** tableau y figure en
    entier, colonne Défaut comprise — au moins `grep -c "Rejeté s'il contient"` ≥ 1, la ligne
    `DATABASE_URL` jusqu'à `PORT/BASE`, la colonne Défaut de `KESH_STATIC_DIR` entière, et la précision
    de (b) entière. Les numéros de ligne ci-dessus se relocalisent par le texte (le `.tex` a bougé
    depuis le `.log` local).

13. **CHANGELOG `[0.13.0]`**, section **Corrigé** : une entrée « **La configuration posée dans `.env`
    atteint enfin Kesh (#550)** » — ce qui était sans effet (e-mails, mot de passe oublié, sortie de la
    démonstration, langue, longueur des mots de passe, plafonds d'import et de sauvegarde, cookies
    `Secure` sur `docker-compose.prod.yml`), le message de démarrage qui renvoyait à une variable
    inexistante, l'installation par `docker-compose.yml` seul ; **inchangé, écrit** (C83) : avec
    `docker-compose.prod.yml`, les montages restent `./documents`, `./inbox`, `./log` et les
    `KESH_*_HOST_DIR` y sont sans effet (issue #558) ; puis **⚠️ Action requise sur une installation
    existante** *(l'ouverture en gras « déplacez d'abord vos dossiers » et le renvoi à la recette de
    déplacement sont retirés avec l'AC4, C83)* :
    re-télécharger le compose et y reporter ses seules adaptations locales (port, réseau, **chemins des
    montages** — un montage modifié à la main et non reporté revient à `./documents`, et justificatifs et
    PDF paraissent disparus ; F4-6),
    **`docker compose pull` puis `docker compose up -d`** —
    **la nouvelle image seule ne corrige rien**, le défaut est dans le fichier compose que l'exploitant
    détient, et le compose de la 0.13.0 s'emploie avec l'image 0.13.0 (F-5) ; pour qui garde son fichier,
    renvoi aux **deux gestes** du manuel (AC12 f) ; et l'avertissement, en toutes lettres : *« relisez
    votre `.env` avant de mettre à jour »* — **refus de démarrer** si `KESH_SMTP_TLS` ou
    `KESH_FEATURE_FORGOT_PASSWORD` portent une valeur autre que `true`/`false`/`1`/`0`, si
    `KESH_COOKIE_SECURE` en porte une sur `docker-compose.prod.yml`, si le mot de passe oublié est activé
    avec un SMTP incomplet **ou mal formé**, ou si, activé, le mailer SMTP ne se construit pas (R3-2), ou
    si `KESH_JWT_SECRET` est resté au placeholder du gabarit ou à une valeur entre chevrons `<…>`, ou si
    `KESH_ADMIN_PASSWORD` a été décommenté sans remplacer le sien ou recopié d'un exemple entre chevrons du
    manuel (AC16) — le
    message nomme la variable (`docker compose logs kesh-api`) ; `KESH_PRODUCTION_RESET` réautorise la
    réinitialisation **d'une installation restée en démonstration** (F1-5) ; les quatre `KESH_SMTP_*`
    posées activent l'envoi de factures, et avec
    `KESH_FEATURE_FORGOT_PASSWORD=true` la réinitialisation du mot de passe par e-mail ;
    `KESH_COOKIE_SECURE` agit enfin sur `docker-compose.prod.yml`, `KESH_ADMIN_BACKUP_DIR`, `KESH_LANG`… ;
    une ligne `KESH_LOG_FILE_PATH=` vide désactive désormais le journal fichier ; une valeur qui contient `$`
    se met entre apostrophes simples (F1-4, R4-13) ; un `.env` copié du gabarit sans modification ne change rien d'autre que le refus
    du placeholder JWT. **Exploitant de `docker-compose.yml` qui construit depuis un clone** (R2-11) : après
    `git pull`, `docker compose up -d` **tire** `gcorbaz/kesh:latest` au lieu de construire ses sources
    (AC5) — `docker compose up -d --build` pour construire, et `docker compose pull kesh-api` pour revenir à
    l'image publiée après un build local qui l'a étiquetée `latest`. Dans la même entrée : un `.env` sans `KESH_ADMIN_PASSWORD` ne fait plus refuser le
    démarrage avec `docker-compose.yml` ni avertir Compose avec `docker-compose.prod.yml` —
    l'onboarding `/setup` recommandé par le manuel fonctionne, et le retrait des variables après un
    break-glass aussi (AC2 i) ; `KESH_JWT_SECRET` reste obligatoire (F-10). *(L'entrée **Modifié** — vide =
    absent, trim — est celle de la 15-11b.)*
    Section **Sécurité** (#557, AC16) : une entrée « **Le secret JWT et le mot de passe admin laissés au
    placeholder du gabarit, ou à un gabarit entre chevrons du manuel, sont refusés au démarrage** » —
    jusqu'ici, `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>` recopié tel quel était **accepté**, et
    les jetons de session signés avec un secret publié dans le dépôt ; de même
    `KESH_ADMIN_PASSWORD=<GENERATE_ME: openssl rand -base64 24>` décommenté tel quel, ou
    `KESH_ADMIN_PASSWORD=<mot de passe fort, min. 12 caracteres>` recopié du manuel, qui créait sur une base
    vide un administrateur au mot de passe publié (F3-4) ; **action pour une installation existante** : la
    vérification par Compose de l'AC12 f (`sudo docker compose config | grep -iE …`, recopiée **entière**
    du manuel, ou renvoi à elle — jamais un `grep` sur `.env`, R3-13 = F3-11) ; pour
    `KESH_JWT_SECRET`, remplacer la valeur par la sortie de `openssl rand -hex 32` **avant** la mise à jour
    (toutes les sessions ouvertes sont invalidées, chacun se reconnecte) ; pour `KESH_ADMIN_PASSWORD`,
    choisir un vrai mot de passe ou recommenter la ligne — **et**, si l'administrateur a été créé avec ce
    placeholder, **changer son mot de passe** dans Kesh (le refus au démarrage ne change pas un compte déjà
    créé) ; puis `docker compose up -d`. Sans cette action, Kesh 0.13.0 refuse de démarrer, avec un message
    qui nomme la variable.

14. **Gates.** `kesh-api` touché (messages, commentaires, **deux contrôles de `Config::from_env`** — AC16 —,
    tests neufs, dépendance de test) : gate ciblé
    (`cargo nextest run -E 'binary(configuration_transmise) | (package(kesh-api) & (test(jwt_secret) | test(admin_password)))'` +
    `fmt` + `clippy` workspace) entre les
    passes ; **gate complet** au dernier commit de code (`scripts/test-fast.sh`, base remise à zéro
    avant — `DROP`/`CREATE` de ses seules bases, jamais de redémarrage du conteneur MariaDB) ; frontend non
    touché (`npm run check` et `test:unit` non requis — l'écrire) ; **E2E complet au dernier commit de
    code** (D7), jugé fichier par fichier contre `docs/testing.md` § *Les échecs attendus* — le montage E2E
    local pose déjà un `KESH_JWT_SECRET` qui ne contient pas `GENERATE_ME` (à vérifier au T9 : `grep -rn
    GENERATE_ME frontend/ scripts/ .github/` doit rester vide hors gabarit) ni ne prend la forme `<…>`
    (`grep -rnE "KESH(\\\\)?_(JWT(\\\\)?_SECRET|ADMIN(\\\\)?_PASSWORD)[=:] *['\"]?<" --exclude-dir={target,node_modules,.git,_bmad-output} .`
    — la forme du T8, qui voit aussi la graphie `\_` du `.tex` (R4-9 = F4-3) ; relevé P4, sept lignes :
    `.env.example:82`, `:88`, `admin-manual.tex:240`, `:244`, `:988`, `:1230` (AC12 e) et
    **`CLAUDE.md:180`**, la recette E2E du dépôt, qui écrit `KESH_ADMIN_PASSWORD='<12+ caractères>'` comme
    gabarit : recopiée telle quelle, elle est refusée après l'AC16 (elle l'était déjà pour le secret,
    12 caractères) — **le `CLAUDE.md` n'est pas modifié par la story**, signalé au Project Lead). Ne
    déclarer que ce qui a tourné.

15. **Propagation : `up -d`, jamais `restart`, pour appliquer `.env` ou le compose.** Toute consigne qui
    fait suivre une modification de `.env` (ou du compose) d'un `docker compose restart` — ou d'un
    « redémarrer » sans commande — devient **`docker compose up -d`** (avec le service, si la consigne le
    nommait). Sites relevés : `admin-manual.tex:648` (AC12 a), **`:1239`** (« Redémarrer une dernière
    fois »), **`:1289`** (procédure HTTP-only `KESH_COOKIE_SECURE=false`) et **`:991`** (F1-6 :
    « Retirer les variables `KESH_ADMIN_*` de `.env` après la première connexion réussie (sinon, chaque
    redémarrage logge un warning) » — un `restart` garde la variable retirée dans le conteneur et
    l'avertissement persiste : la phrase gagne « puis `docker compose up -d` ») ; `DOCKER_START.md:62`
    (§ *Redémarrer* : gagne « après une modification de `.env` ou du compose : `docker compose up -d` ») et
    `:106` (« Et redémarrez », après une modification du mapping de port) ; les sites déjà justes
    (`admin-manual.tex:1232`, `:1711`, `:2111-2114`) sont listés au Dev Agent Record comme vérifiés. Grep
    de contrôle, à coller au Dev Agent Record :
    `grep -rnE 'compose restart|docker-compose restart|redémarr' docs/manual/fr/*.tex *.md docs/*.md .env.example docker-compose*.yml website`
    — chaque site lu, traité ou écrit hors sujet. **La recette de la 15-7b2** (`user-manual.tex:177-189`,
    `admin-manual.tex:691`, `:1314`) **n'est pas touchée ici** : la 15-7b2, rebasée, écrit
    `docker compose up -d` (report par l'orchestrateur — F3-12 de la P3 : dans le worktree `kesh-15-7`,
    la cellule `:1314` de la fiche 15-7b2 dit encore « redémarrer Kesh … puis retirer la variable et
    redémarrer » ; `up -d` n'est que dans son Change Log : report dans la cellule **et** dans son T7). `DOCKER_START.md` passe entièrement à
    `docker compose` (v2) — AC5 (R2-6, F2-10). **Synology Container Manager** : AC12 (i).
    **`admin-manual.tex:1238`** (R2-8 : « un `WARN bootstrap: retirer les vars de .env` apparaît à chaque
    redémarrage », même formulation que `:991`) : vrai tant que les variables restent dans le conteneur ;
    le geste qui les en retire est le `:1239` (« Redémarrer une dernière fois »), qui devient
    `docker compose up -d` — `:1238` est donc **lu**, corrigé par ricochet, et inscrit tel quel au Dev
    Agent Record parmi les sites du grep.

16. **Le placeholder du gabarit — et tout gabarit entre chevrons — est refusé comme secret JWT et comme
    mot de passe admin (#557 ; R1-2, C79 ; R2-3 = F2-1, F2-8, C81 ; F3-4, C83).** Une **constante
    commune** de `config.rs`, `const TEMPLATE_PLACEHOLDERS: &[&str] = &["generate_me"];` (sous-chaînes en
    minuscules, comparées au texte mis en minuscules par `to_ascii_lowercase`), et une **fonction d'aide
    commune** `fn is_template_placeholder(value: &str) -> bool` (doc-commentée) qui rend vrai si la valeur
    contient une entrée de `TEMPLATE_PLACEHOLDERS` **ou** si, **après trim**, elle **commence par `<` et
    finit par `>`** (F3-4, décision de l'orchestrateur en P3 : la règle couvre `<GENERATE_ME: …>`, les
    quatre gabarits du manuel — `<secret aleatoire, …>`, `<mot de passe fort, …>`,
    `<mot-de-passe-fort-12-chars>`, `<nouveau-mdp-12+>` — et ceux à venir ; un secret engendré par
    `openssl rand -hex`/`-base64` n'a jamais cette forme ; un `<` ou un `>` **à l'intérieur** d'une valeur
    reste admis). Le trim est local au contrôle : le secret JWT n'est pas trimé à la lecture avant la
    15-11b, et `" <x> "` doit être refusé quand même. L'aide sert aux deux contrôles de
    `Config::from_env` :
    - **`KESH_JWT_SECRET`** (`config.rs:657-671`) : refusé s'il contient `change-me` (inchangé — garde du
      défaut de `docker-compose.yml`, § *Le défaut*) **ou** si `is_template_placeholder` → 
      `ConfigError::InsecureJwtSecret` ;
    - **`KESH_ADMIN_PASSWORD`** (`config.rs:633-646`, dans `if let Some(ref p)`, valeur déjà trimée) :
      refusé s'il vaut `changeme` (inchangé, `eq_ignore_ascii_case`) **ou** si `is_template_placeholder`
      → **`ConfigError::InsecureAdminPassword`**, variante **existante** réutilisée (décision de
      l'orchestrateur) ;
    - **ordre (F2-8)** : pour les deux variables, le contrôle du placeholder vient **avant** celui de la
      longueur — un `KESH_JWT_SECRET=GENERATE_ME` (11 caractères) rend `InsecureJwtSecret`, qui nomme le
      gabarit, et non « trop court : 11 octets » ; le mot de passe admin l'était déjà (`changeme` avant
      `< 12`). Le contrôle `change-me` du secret JWT **remonte** donc avant `len() < 32`. Effet sur les
      tests existants, relu : `config_rejects_weak_jwt_secret` (`"too-short"`, sans placeholder) reste
      `WeakJwtSecret` ; le doc-comment de `config_rejects_jwt_secret_containing_change_me`
      (`config.rs:1800-1803`, « ligne `len() < 32` AVANT ligne `contains("change-me")` ») et le
      commentaire `:663-667` (« `eq_ignore_ascii_case` ligne 408 ») deviennent faux : **corrigés**
      (propagation), **de même que** le commentaire de test `config.rs:1811` (« 34 chars, ≥ 32 → passe
      WeakJwtSecret, contient "change-me" → fail » — R3-6 : faux une fois `change-me` contrôlé avant la
      longueur) ; leurs secrets de 34 caractères restent valables. Le contrôle `change-me` et le contrôle
      du placeholder du secret forment **une seule condition** (`contains("change-me") ||
      is_template_placeholder(…)`), placée avant `len() < 32` — M25 la redescend tout entière.
    **Pourquoi les variantes existantes** (C79, étendu par C81) : c'est le même défaut — un placeholder du
    gabarit non remplacé —, la même action (générer ou choisir la valeur), et une variante neuve
    dupliquerait message et traitement pour aucune différence d'action. **Aucun appelant ne distingue ces
    variantes hors de `config.rs`** (R2-4 : `grep -rn 'InsecureJwtSecret\|InsecureAdminPassword' crates
    --include=*.rs` ne rend que `config.rs` ; `main.rs:63-68` logge le `Display` de l'erreur, quelle
    qu'elle soit). Les variantes restent **unitaires** (ni la valeur ni la sous-chaîne trouvée :
    *Display ne loggue jamais la valeur*).
    (a) **Messages** (`config.rs:111-123`) : `InsecureJwtSecret` — « `KESH_JWT_SECRET` contient un
    placeholder (`change-me`, `GENERATE_ME`, ou valeur entre chevrons `<…>`). Générer un vrai secret via :
    openssl rand -hex 32 » ; `InsecureAdminPassword` — « `KESH_ADMIN_PASSWORD` est un placeholder
    (`changeme`, contient `GENERATE_ME` — sans égard à la casse —, ou valeur entre chevrons `<…>`).
    Choisir un vrai mot de passe (≥ 12 caractères), ou
    retirer la ligne de `.env` pour créer l'administrateur à l'écran `/setup` ». Aucun test n'asserte le
    texte actuel de ces messages (`grep -rnF "contient le placeholder" crates` et `grep -rnF "valeur par
    défaut 'changeme'" crates` : `config.rs` seul, dans le `write!`).
    (b) **Doc-comments** `config.rs:35-45` : `InsecureJwtSecret` — « contient `change-me` (défaut de garde
    de `docker-compose.yml`), `GENERATE_ME` (placeholder `<GENERATE_ME: …>` de `.env.example`), ou est de la
    forme `<…>` (gabarits du manuel) » ; `InsecureAdminPassword` — « vaut `changeme`, contient
    `GENERATE_ME` (placeholder de la ligne commentée `#KESH_ADMIN_PASSWORD=` de `.env.example`) ou est de la
    forme `<…>` » ; et le doc-comment de `:184` qui énumère les
    refus admin.
    (c) **Tests unitaires** (module `tests` de `config.rs`, patron `config_rejects_jwt_secret_containing_change_me`,
    `env_lock()` + `reset_env()`). **Lecture du gabarit** (R2-9) : `include_str!("../../../.env.example")`,
    dans le `mod tests` — jamais compilé hors test, l'image Docker n'en dépend pas ; une fonction d'aide
    du `mod tests` rend la valeur de **la seule ligne qui commence, en colonne 0, par** le préfixe demandé
    (`KESH_JWT_SECRET=`, non commenté ; `#KESH_ADMIN_PASSWORD=`, commenté — le `#` n'est retiré que par le
    test), et **asserte qu'il y en a exactement une** (`assert_eq!(nombre, 1)`, assertion de montage : la
    prose de l'en-tête que l'AC6 d ajoute ne commence pas par l'affectation, et un doublon rougirait) ;
    elle asserte aussi que la valeur extraite contient `GENERATE_ME` (sans quoi le test passerait à vide
    sur un gabarit sans placeholder — et M23 le prouve dans l'autre sens).
    - **`config_rejects_jwt_secret_left_at_template_placeholder`** : la valeur de la ligne
      `KESH_JWT_SECRET=` du gabarit → `InsecureJwtSecret` (l'ancienne assertion de montage « ≥ 32
      caractères » tombe : le placeholder est contrôlé avant la longueur, F2-8) ;
    - **`config_rejects_admin_password_left_at_template_placeholder`** (neuf, R2-3 = F2-1) : la valeur
      de la ligne `#KESH_ADMIN_PASSWORD=` du gabarit, avec un `KESH_JWT_SECRET` valide → 
      `InsecureAdminPassword` ;
    - **`config_rejects_jwt_secret_generate_me_case_insensitive`** : `generate_me`, `Generate_Me`,
      `GENERATE_ME` dans un secret de 40 caractères → `InsecureJwtSecret` ; **et** le secret court
      `GENERATE_ME` (11 caractères) → `InsecureJwtSecret`, non `WeakJwtSecret` (ordre, F2-8) ; **et** le
      secret court `xchange-mex` (11 caractères) → `InsecureJwtSecret`, non `WeakJwtSecret` (F3-8 : la
      remontée de `change-me` avant la longueur, qu'aucun test n'exerçait — les secrets existants font
      34 caractères) ;
    - **`config_rejects_admin_password_generate_me_case_insensitive`** (neuf) : les mêmes trois casses
      dans un mot de passe de 20 caractères, et `generate_me` seul (11 caractères) →
      `InsecureAdminPassword`, non `WeakAdminPassword` ;
    - **`config_rejects_jwt_secret_and_admin_password_in_angle_brackets`** (neuf, F3-4 ; le nom contient
      `jwt_secret`, si bien que le filtre de l'AC14 le sélectionne) : pour `KESH_JWT_SECRET`, la valeur
      `<un-gabarit-de-plus-de-32-caracteres-ici>` (≥ 32, sans `change-me` ni `GENERATE_ME`) →
      `InsecureJwtSecret`, et `<x>` (3 caractères) → `InsecureJwtSecret`, non `WeakJwtSecret` (ordre) ;
      pour `KESH_ADMIN_PASSWORD` (avec un secret valide), les trois exemples du manuel
      `<mot de passe fort, min. 12 caracteres>`, `<mot-de-passe-fort-12-chars>`, `<nouveau-mdp-12+>` →
      `InsecureAdminPassword`, et `<x>` → `InsecureAdminPassword`, non `WeakAdminPassword` ; `" <x> "`
      (espaces autour) → refusé pour les deux ; **témoins** acceptés : un secret de 40 caractères qui
      **contient** `<` et `>` à l'intérieur (`abc<def>` + 32 caractères), et un mot de passe
      `pw<valide>12chars` — la règle porte sur la forme, pas sur la présence d'un chevron ;
    - **`config_accepts_generated_jwt_secret`** : 64 caractères hexadécimaux → `Ok` (témoin que la garde
      ne refuse pas un vrai secret — s'il existe déjà un test équivalent, le nommer au Dev Agent Record au
      lieu d'en écrire un) ; le témoin admin existe déjà (un mot de passe valide est posé par la plupart
      des tests, ex. `"valid-test-pw-12chars"`) : le nommer au Dev Agent Record.
    Mutations **M20**, **M23** à **M27** (AC9).
    (d) **Documentation** — propagation complète du symptôme « placeholder du secret » (R2-4, F2-5, F2-6,
    F2-4 ; décision de l'orchestrateur en P2), chaque site traité :
    - `.env.example` : en-tête `:6-8` et commentaire `:85-86` (AC6 d), section admin `:78-82` (AC6 g) ;
    - `docker-compose.prod.yml:27-29` (« doivent être définis… defaults insecure type `changeme` ou
      `change-me-32-bytes-...` ») — `KESH_ADMIN_PASSWORD` **optionnel** (déjà prévu par l'AC2 i) et
      `GENERATE_ME` nommé ; `:74` (« NE PAS contenir "change-me" ») — « ni `GENERATE_ME` » ;
    - manuel : `admin-manual.tex:240`, `:236-254` (AC12 e), `:243-244`, `:988`, `:1230` (gabarits entre
      chevrons, F3-4, AC12 e), `:537-539` et `:546-548` (AC12 e,
      installation Synology), `:662` et `:690` (AC12 e, tableaux), la vérification de mise à jour (AC12 f) ;
    - **`crates/kesh-api/README.md`** (F2-5, hors du grep du T8 jusqu'ici) : `:55-57` (« une valeur factice
      `change-me-32-bytes-…` qui est explicitement rejetée au démarrage via un warning ») est **faux** —
      le gabarit porte `<GENERATE_ME: …>` et le refus est une **erreur fatale** — : réécrit ; la liste
      des erreurs fatales `:62-65` gagne `InsecureJwtSecret` et, pour `KESH_ADMIN_PASSWORD` posé,
      `InsecureAdminPassword` et `WeakAdminPassword` ; la ligne du tableau `:38` (`KESH_ADMIN_PASSWORD`,
      défaut `changeme`, « logué en warning s'il vaut `changeme` ») est **fausse** — défaut vide, refus au
      démarrage — : corrigée. Le reste de ce README (inventaire de variables partiel `:30-47`, exemple
      de requête `:80`) est **hors périmètre, écrit** : ce n'est pas un document d'exploitation ;
    - **`README.md:79-81`** (F2-4 : « `cp .env.example .env` — Adapter les valeurs ») : « Adapter les
      valeurs, **dont `KESH_JWT_SECRET`** (`openssl rand -hex 32`) — le placeholder est refusé au
      démarrage ». **Effet sur la pile de développement**, à écrire au Dev Agent Record : un développeur
      qui a copié le gabarit sans remplacer le secret voit `cargo run` (qui lit `.env`, `main.rs:44`) et
      `docker-compose.dev.yml` (dont `${KESH_JWT_SECRET:-dev-secret…}` cède à `.env`) refuser de démarrer
      — voulu, message clair ; le contrôle de l'AC14 (`grep -rn GENERATE_ME frontend/ scripts/ .github/`)
      ne voit pas cet effet, qui passe par une copie ;
    - AC13 (**Sécurité**), `DOCKER_START.md` (AC5).

## Tasks / Subtasks

- [x] **T0 — Relevé** (AC1) : refaire l'inventaire depuis `main` à jour (commande de la § *Inventaire
  fermé*), comparer à la table, écrire l'écart. Refaire la table § *Le vide avant la 15-11b* (dont
  `create_dir_all("")` et l'emplacement réel de la sauvegarde pré-import, mesurés). Refaire la liste fermée
  « relisez » (AC12 f) depuis ses **trois** sources : `ConfigError` (dont les deux variantes étendues par l'AC16),
  les `process::exit` de `main.rs` (R3-2), et **les interpolations du compose nouvellement prises en
  compte ou dont la forme change** (`KESH_LOG_FILE_PATH`, `KESH_ADMIN_PASSWORD` ; plus de montage
  `KESH_*_HOST_DIR` de P depuis l'abandon de l'AC4, C83) — cette troisième relevée ici depuis l'AC2, puis
  **revérifiée au T8**
  contre `git diff main -- docker-compose.yml docker-compose.prod.yml` : toute interpolation changée qui
  n'est pas dans la liste est un écart à écrire. **Compose** :
  `docker compose version` au Dev Agent Record, et une mesure `docker compose config` de la forme retenue
  (`${X:-}` absente, posée, vide ; `${X-d}` absente, vide ; valeur `.env` contenant `$`, `$$`, `'…$…'` ; et les mêmes valeurs lues par `dotenvy` — `cargo run`,
  R4-13 —, en décodant le réaffichage `$` → `$$` de `config`).
  Version de Synology Container Manager non mesurable depuis le poste — le noter.
- [x] **T1 — Test d'abord, rouge** (AC8) : écrire `configuration_transmise.rs` (`LUES`, `AJOUTS`, T, V, E,
  F, S) ; dépendance de test `yaml-rust2` (C72). Constater le **rouge attendu** sur l'état actuel —
  **exactement** ceci, et rien d'autre :
  - **(T)** : les **16** variables manquantes, soit **28** couples (variable, compose) — les 12 « par
    aucun » dans les deux, `KESH_COOKIE_SECURE` dans P, `KESH_LANG`, `KESH_PASSWORD_MIN_LENGTH`,
    `KESH_BANK_IMPORT_MAX_MB` dans Y — ; l'`image:` absente de Y ; **aucun** rouge de montage (les
    montages fixes de P et interpolés de Y sont déjà l'état exigé, AC4 abandonnée, C83) ; **aucune**
    « faute de frappe » ;
  - **(V)** : `KESH_LOG_FILE_PATH` dans les deux (`:-` au lieu de `-`) ; `KESH_ADMIN_PASSWORD` dans Y
    (`:-changeme`) **et dans P** (`${KESH_ADMIN_PASSWORD}`, R1-5) — `SANS_DEFAUT` ; le contrôle de
    `VIDE_SIGNIFIANT` sur `.env.example` (marqueur absent, « ou absent » présent) ; **aucun** rouge
    `AJOUTS` (les 28 clés sont absentes : c'est (T) qui les signale) ;
  - **(E)** : **aucun** rouge (réécrit en P3, C83 — le rouge « trois variables d'hôte » venait des
    montages figés de P, que l'abandon de l'AC4 garde et que (E) ne consulte plus : `${NOM` est cherché
    dans les `volumes:` de `docker-compose.yml`, où il est déjà). **Pas** `KESH_ADMIN_RESET` :
    `.env.example:235` est de la prose, pas une ligne d'affectation. (E) est exercé par les mutations M3,
    M7, M12 et M14 ;
  - **(F)** : `KESH_ADMIN_RESET` dans `.env.example:235` seulement (le code n'est pas un corpus ; les
    `.tex`, les `.ftl` et les compose n'en portent pas) ;
  - **garde `LUES`** : verte ; **(S)** : vert.
  Tout rouge hors de cette liste, ou tout élément de la liste resté vert, est un défaut du test à corriger
  avant le T2. Sortie au Dev Agent Record — c'est la preuve que le test voit le défaut de #550.
- [x] **T2 — Compose** (AC2, AC3, AC4, AC5) : les 28 lignes en `${KESH_X:-}`, `KESH_LOG_FILE_PATH` en
  `${…-…}` dans les deux, `KESH_ADMIN_PASSWORD` en `${…:-}` dans les deux, les commentaires (dont ceux de P
  sur le mot de passe admin, aucun qui nomme `KESH_PRODUCTION_RESET`, « `--build` pour construire depuis
  les sources » dans Y), `image:` dans Y ; **montages de P inchangés** (AC4 abandonnée, C83), une ligne
  de commentaire au-dessus d'eux : *« Montages fixes : les `KESH_*_HOST_DIR` de `.env` sont sans effet
  ici (issue #558). »* — les commentaires existants `:97-101`, `:107-108`, `:119-127` restent vrais (F3-9,
  vérifié).
- [x] **T3 — `.env.example`** (AC6) **, fantôme** (AC7) **et garde des placeholders** (AC16) : bloc
  « Chemins internes », en-tête (règle de transmission, `up -d`, `KESH_JWT_SECRET` obligatoire et à
  générer, valeur contenant `$` entre apostrophes simples — AC6 h), commentaires de `KESH_LOG_FILE_PATH`, `KESH_JWT_SECRET` et `KESH_SMTP_PASSWORD` ;
  les cinq sites `KESH_ADMIN_RESET` (`.env.example:235`, `config.rs:158`, `:319`, `main.rs:337`,
  `lib.rs:1040`), la ligne de commentaire de la section admin (AC6 g) ; grep de l'AC7 à zéro. **AC16,
  test d'abord** : écrire les **six** tests unitaires (dont l'aide de lecture du gabarit et ses deux
  assertions de montage), constater que les **cinq** tests de refus sont **rouges** (placeholder ou
  gabarit entre chevrons accepté ; pour les secrets courts `GENERATE_ME`, `xchange-mex` et `<x>`,
  `WeakJwtSecret` ; pour `generate_me` seul et `<x>` en mot de passe admin, `WeakAdminPassword`) et le
  témoin vert ; puis la constante `TEMPLATE_PLACEHOLDERS`, la fonction `is_template_placeholder`, les deux
  contrôles dans l'ordre de l'AC16 (placeholder avant longueur), les messages et les doc-comments (dont
  `:184`, `:663-667`, `:1800-1803`, `:1811`) ; les six verts, et les tests existants du secret et du mot
  de passe admin toujours verts. Les blocs d'hôte de `.env.example` (AC6 i).
- [x] **T4 — Vert** : le test passe ; gate ciblé.
- [x] **T5 — Mutations M1-M28** (AC9), une à une, restauration vérifiée par `git diff --stat` vide.
- [x] **T6 — Docker sans démarrer** (AC5, AC10, AC11) : `config --format json` ×2 avec les quatre `.env` de
  l'AC10 ; `--dry-run up -d` avant/après dans un répertoire vide ; `config -q` tel que l'étape CI le
  lancera. Répertoires d'essai dans le scratchpad, supprimés ensuite ; aucun réseau, volume ni conteneur
  créé (`docker ps -a`, `docker network ls` avant/après). *(La recette de déplacement n'existe plus,
  C83 : rien à rejouer.)* **Vérification des placeholders** de l'AC12 f, **recopiée du `.tex` final**
  (`sudo` retiré), rejouée dans un répertoire d'essai portant le compose final (P **et** Y) et un `.env`
  piégé à chaque fois : `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>` ; la même avec `export ` ;
  `KESH_JWT_SECRET = <GENERATE_ME: …>` (espaces autour de `=`) ; entre guillemets doubles ; avec `\r\n` ;
  `KESH_ADMIN_PASSWORD=<nouveau-mdp-12+>` ; `KESH_ADMIN_PASSWORD=<mot de passe fort, min. 12 caracteres>`
  (espaces dans la valeur) ; `KESH_ADMIN_PASSWORD=generate_me-et-12-caracteres` ;
  `KESH_ADMIN_PASSWORD="   <x>  "` (plusieurs espaces avant `<`, R4-14) ; et deux témoins **muets**
  (secret de 64 caractères hexadécimaux ; mot de passe `pw<valide>12chars`) — pour chacun, la sortie de
  `docker compose config` pour la clé **et** la sortie de la commande : la commande montre la ligne dans
  chaque cas piégé, reste muette pour les témoins ; au Dev Agent Record, avec `docker compose version`. Si
  le rendu YAML de Compose met la valeur sous une forme que le motif ne voit pas, **corriger le motif au
  manuel et dans la fiche**, ne pas l'écrire comme limite.
- [x] **T7 — CI** (AC11) et `docs/ci.md`.
- [x] **T8 — Manuel** (AC12, dont la mise en page des dix tableaux de `sec:env-vars` — AC12 j —, AC15),
  **brochure** (AC12 h), **`DOCKER_START.md`** (AC5, AC15, passage à `docker compose`), **`README.md`** et
  **`crates/kesh-api/README.md`** (AC16 d) puis **CHANGELOG** (AC13 : **Corrigé** et **Sécurité**).
  Revérification de la troisième source de la liste « relisez » (T0). `make fr` de l'état **d'avant**
  (relevé des `Overfull \hbox`), puis de l'état d'après. Propagation :
  `grep -rnE 'chargé automatiquement|env_file|KESH_ADMIN_RESET|compose restart|docker-compose restart|docker-compose |redémarr|clé sans valeur|ou absent|changeme|change-me|GENERATE_ME|secrets obligatoires|MIN_PASSWORD|Mot de passe: .admin.|up (-d )?--build|HOST(\\)?_DIR' docs *.md crates/*/README.md website .env.example docker-compose*.yml crates/kesh-api/src frontend/src`
  — chaque site lu, traité ou écrit comme hors sujet (R2-4, F2-5 : `crates/*/README.md` n'était pas
  couvert, le motif `*.md` ne vaut qu'à la racine — et `README.md`, **à la racine**, est déjà couvert par
  `*.md` : ne pas le nommer une seconde fois, R3-12 ; `HOST(\\)?_DIR` voit aussi la graphie `\_` du
  `.tex` ; `rsync -a` et `refiger` retirés avec la recette, C83) ; chaque site `HOST_DIR` dit, pour
  `docker-compose.prod.yml`, « sans effet, montage fixe » (AC6 i, AC12 c) ; inventaire fermé des
  gabarits entre chevrons (AC12 e, AC14) :
  `grep -rnE "KESH(\\\\)?_(JWT(\\\\)?_SECRET|ADMIN(\\\\)?_PASSWORD)[=:] *['\"]?<" --exclude-dir={target,node_modules,.git,_bmad-output} .`
  — tout site neuf lu ; **`git diff main -- docker-compose.prod.yml`** ne touche **aucune** ligne sous
  `volumes:` hors le commentaire du T2 (C83) ; rappel au Dev Agent Record : la 15-7b2 rebasée écrit
  `docker compose up -d` dans sa recette (AC15).
- [x] **T9 — Gates** (AC14) et Dev Agent Record (décomptes recomptés depuis la source, avec leur périmètre).

## Dev Notes

### Pourquoi une liste explicite et non `env_file: .env` (C71, maintenu par C75)

`env_file: .env` transmettrait **tout** `.env` au conteneur : les quatre `MARIADB_*` — dont
`MARIADB_ROOT_PASSWORD`, qui n'a rien à faire dans l'environnement de l'application (visible par
`docker inspect`, `/proc/1/environ`) —, toute variable étrangère, et les lignes décommentées par erreur :
`KESH_STATIC_DIR=frontend/build` décommenté écraserait le chemin de l'image et casserait le SPA. Il
rendrait aussi le test (T) **vide de sens**. La liste explicite garde un contrat lisible, les valeurs
forcées (`KESH_HOST: 0.0.0.0` dans P) et les défauts de déploiement ; son défaut — redériver quand le code
gagne une variable — est exactement ce que le test ferme (avec la liste `LUES` ici, avec la lecture du code
après la 15-11b). La forme **`${KESH_X:-}`** évite de recopier dans le compose les défauts du code (DRY).

### Pourquoi le test ne lit pas encore le code (C77)

La 15-11 complète lisait le code par `syn` (règles L et F, indirections, `cfg(test)`, macros). Ses trois
passes de validation ont montré que les défauts se **recyclaient dans cette machinerie** — chaque remédiation
y faisait naître le défaut suivant (P2 : R2-1 ; P3 : F-2, F-3). Cette story-ci livre la correction de #550,
dont dépend la 15-7b2, avec un test **simple** : une liste fermée `LUES`, écrite en dur et reproductible par
un `grep` documenté. La 15-11b remplace cette liste par la lecture du code, en même temps qu'elle pose la
fonction de lecture unique — les deux se tiennent.

### Dépendances de test (C72)

- **Analyseur YAML** : aucun au `Cargo.lock`. Retenu : **`yaml-rust2`** (pur Rust, maintenu, sans
  `serde`). `cargo add --dev yaml-rust2 -p kesh-api`, version au Dev Agent Record (0.13.0 au 2026-10-08,
  `MIT OR Apache-2.0`, `rust-version` 1.85, présente au registre local — vérifié par la lentille R de la
  P3). Elle fait entrer une seconde version de `hashlink` (le lock porte 0.10.0) : à écrire, sans
  conséquence (dépendance de test). Pas de `deny.toml` dans le dépôt ; si la CI refuse la dépendance, revenir
  à cette fiche plutôt que de basculer sur un analyseur maison.
- **Pas de `syn`** dans cette story : c'est la 15-11b (et la 15-5e1) qui l'ajoutent. Aucune collision de
  `Cargo.toml` avec la 15-5e1 sur ce point ; si la 15-5e1 est mergée avant, rebase ordinaire (une ligne de
  dev-dépendance de plus de chaque côté), `Cargo.lock` repris de `main` puis régénéré par
  `cargo check -p kesh-api --tests`, jamais fusionné à la main.

### Le `Dockerfile` n'est pas modifié

`ENV KESH_STATIC_DIR=/app/static` et `ENV KESH_LOCALES_DIR=/app/locales` (l. 45-46) sont la raison des
deux exceptions, et le test les contrôle (M5).

### Appliquer une modification de `.env` : `up -d`, pas `restart`

`docker compose restart` redémarre le conteneur **avec son environnement d'origine** ; seul
`docker compose up -d` recrée le conteneur quand sa configuration a changé. C'est la précision qui manque
au manuel (AC15), et elle concerne directement la recette de la 15-7b2.

### Coordination avec la 15-7b2

**Ordre de merge : la 15-11a d'abord** (C-15-7-51 ; la 15-7b2 l'impose dans son en-tête et son T8, qui
contrôle `grep PRODUCTION_RESET` sur les deux compose et `KESH_PRODUCTION_RESET=true docker compose config`
— contrôle que la forme `${KESH_PRODUCTION_RESET:-}` satisfait).

Zones partagées, écrites :

- `.env.example:272-279` (bloc `KESH_PRODUCTION_RESET`, dont la prose `# is_demo=true` de `:275`) :
  **propriété de la 15-7b2**, non touché ici (AC6 f) ; la ligne d'affectation reste, (E) la contrôle ; la
  classe `[A-Z]` du NOM de (E) rend la prose `is_demo=` invisible (F-1).
- `admin-manual.tex`, section `sec:env-vars` : **la mise en page des dix tableaux** (titres
  `\paragraph{…}` hors ligne), prévue par la 15-7b2 (T7, C-15-7-52), **passe à la 15-11a** (AC12 j,
  décision de l'orchestrateur en P2, F2-3) — la 15-11a merge la première et ses propres contrôles PDF
  (AC12 b, `:662`) en dépendent ; **la 15-7b2 n'a plus à le faire**, l'orchestrateur ajuste sa fiche
  (AC 11 ligne `:691`, T7). La 15-11a modifie aussi les **cellules** `:662`, `:678-679`, `:690` et la
  phrase d'introduction `:783-784`. La 15-7b2 garde la réécriture de la ligne `:691`
  (`KESH_PRODUCTION_RESET`), voisine de `:690` — hunks voisins. Règle : **la 15-11a ne touche ni le texte
  des titres `\paragraph{…}`, ni `:691`, ni `:1314`** ; la 15-7b2 relocalise ses lignes **par le
  texte**. La 15-11b touche `:662` (trim) : relocalisation par le texte.
- `admin-manual.tex` hors section : la 15-11a touche `:218-254`, `:530-548`, `:552-566`, `:648`, `:1239`,
  `:1289`, `:988`, `:991`, `:1230`, `:1384`, `:1518`, `:1704-1714` (les quatre sites `:988`, `:1230`,
  `:1384`, `:1518` ajoutés en P3, C83) ; la 15-7b2 `:967`, `:1250-1259`, `:1314`. **Fiche 15-7b3 relue
  en P4** (worktree `kesh-15-7`, R4-1 = F4-2, C84) : elle ne touche ni `:988`, ni `:991`, ni `:1230`, ni
  `:1384`, ni `:1518` ; **mais elle écrit dans la même sous-section** que l'AC12 f — son « texte
  principal » de la réparation au premier démarrage va dans `admin-manual.tex:1704-1717` (§ *Procédure
  de mise à jour standard*, après l'étape 6 « Vérifier les logs »), que la 15-11a réécrit au point 3
  (consigne datée, deux `lstlisting`, encadré « relisez ») — et elle ajoute une phrase au § *Rollback en
  cas d'échec* (`:1734-1749`, après l'étape 3), vingt lignes plus bas : hunks voisins, que l'insertion
  longue de la 15-11a décale. **Ordre : la 15-11a d'abord** (la 15-7b3 vient après la 15-7b2, qui vient
  après la 15-11a) ; la 15-7b3, rebasée, **relocalise par le texte** — le titre
  `\subsection{Procédure de mise à jour standard}`, l'item « \textbf{Vérifier} les logs » et le
  `\subsection{Rollback en cas d'échec}`, jamais les numéros de ligne — et place son texte **après**
  l'énumération et l'encadré « relisez » de la 15-11a, sans les réécrire ; le PDF est régénéré par celle
  qui merge en second. La 15-7b3 touche aussi `:1637-1650`, `:1793`, `:1904-1906`, `:2054-2063` : hors
  des zones de la 15-11a. Report dans la fiche 15-7b3 : **par l'orchestrateur**. *(Le site `:1306`, voisin de `:1314`, est passé à
  la 15-11b avec le trim.)*
- `user-manual.tex:177-189` : 15-7b2 seule.
- PDF : régénéré par les deux ; celui qui merge second **régénère** après rebase (`make fr`).
- **Mentions nouvelles de `KESH_PRODUCTION_RESET`** (F1-5) : la 15-11a en ajoute au manuel — `lstlisting`
  des gestes de mise à jour (AC12 f), liste « relisez », contrôle « `docker compose config` montre
  `KESH_PRODUCTION_RESET` » — et au CHANGELOG (AC13). Le motif de contrôle de la 15-7b2
  (fiche `15-7b2-remise-a-zero.md`, worktree `kesh-15-7`, liste qui suit le motif
  `KESH.{1,2}PRODUCTION.{1,2}RESET` — à retrouver par ce texte, les numéros de ligne de cette fiche
  bougent ; R2-10) énumère les sites attendus sans eux :
  **report par l'orchestrateur** dans la fiche 15-7b2 (sites relocalisés par le texte au rebase). Leur
  formulation dit « **sur une installation restée en démonstration** » (portée réelle,
  `routes/onboarding.rs:265-279`), cohérente avec la réécriture de `:691` par la 15-7b2.
- CHANGELOG `[0.13.0]` : entrées distinctes, relocaliser par le texte.

### Angles morts assumés (écrits, non traités)

- **Une variable ajoutée au code sans être ajoutée à `LUES`** n'est pas vue par le test de la 15-11a.
  **Propriétaire : 15-11b**, qui remplace `LUES` par la lecture du code (`syn`). Entre les deux merges, la
  revue de code de toute story qui ajoute une lecture d'environnement doit vérifier `LUES` à la main.
- **Fantômes dans le code Rust** : non vus par le (F) de la 15-11a ; le grep de l'AC7 tient le seul
  fantôme connu ; la 15-11b étend (F) au code.
- Lectures d'environnement par des **dépendances** (`sqlx`, `lettre`, `reqwest`, proxys…) : hors inventaire.
- `docker-compose.dev.yml` : inventorié, non contraint ; il ne démarre pas sans `.env`
  (`KESH_ADMIN_PASSWORD: ${…:-admin}`, 5 caractères → `WeakAdminPassword`) — issue à ouvrir par
  l'orchestrateur (F14 de la P2).
- **Mots de passe MariaDB laissés à leur gabarit** (F4-5) : `MARIADB_ROOT_PASSWORD=<mot de passe fort>`,
  `MARIADB_PASSWORD=<…>` (exemple de l'*Étape 3*) et `DATABASE_URL=<EDIT: …>` (`.env.example:21`) recopiés
  tels quels sont acceptés par MariaDB — Kesh ne peut pas les refuser. Le manuel le dit (« rien ne la
  refuse — choisissez-les vous-même », AC12 e) ; risque aggravé par le port publié de #551. **Propriétaire :
  #551** (15-12).
- **Hors périmètre, avec leurs issues** (15-12) : **#552** — `KESH_ADMIN_BACKUP_DIR` vaut `/tmp` par défaut,
  perdu à la recréation du conteneur, sans volume où le pointer ; **#551** — MariaDB publiée sur `3306:3306`
  avec des mots de passe de développement par défaut ; le montage E2E local ne pose pas
  `KESH_PRODUCTION_RESET` (#97).
- **Synology Container Manager** : version de Compose non mesurable depuis le poste ; la recette de la
  v0.13.0 sur le NAS le confirmera.
- **`KESH_ADMIN_USERNAME`** : « vide → onboarding » au manuel (`:689`) alors que les compose posent
  `:-admin` — sans effet observable une fois `KESH_ADMIN_PASSWORD` corrigé (couple exigé) ; non traité.
- **L'assistant `/setup` exposé** sur base vide : comportement existant et documenté
  (`admin-manual.tex:980`) ; non traité.
- **`TMPDIR`** : `std::env::temp_dir()` (`routes/admin.rs:80`) ; sous Docker, `/tmp` du conteneur.
- **`KESH_PASSWORD_MIN_LENGTH`** : trois écrans à 12 en dur (AC12 f, F-6) ; frontend non touché.
- **Fenêtre compose neuf / image ancienne** (F-5) : avec l'image 0.12.1, avertissements « invalide » et
  sauvegarde pré-import dans `/app` (§ *Le vide avant la 15-11b* ; l'image 0.13.0 ne les a plus, revue
  P1) ; dit au CHANGELOG et au manuel (`docker compose pull`).
- **Gabarits non refusés hors des deux secrets de l'AC16** (E-4 = A-L4 de la revue de code P1) : les
  gabarits `<EDIT: …>` de `.env.example` — `KESH_SMTP_USER`, `KESH_SMTP_PASSWORD`, et les autres
  `KESH_SMTP_*`, `KESH_PUBLIC_BASE_URL`, `DATABASE_URL` — décommentés tels quels sont **acceptés** par
  Kesh (`DATABASE_URL` échoue ensuite à la connexion, les `KESH_SMTP_*` à l'envoi). Et pour
  `KESH_ADMIN_PASSWORD`, parmi les variantes « change-me », seul `changeme` **exact** (sans égard à la
  casse) est refusé comme placeholder : la garde de l'AC16 ne couvre **pas** `change-me-…` — vérifié au
  code (`config.rs`, `p.eq_ignore_ascii_case("changeme") || is_template_placeholder(p)` ; la sous-chaîne
  `change-me` n'est cherchée que dans `KESH_JWT_SECRET`) : `change-me` seul (9 caractères) tombe sur la
  longueur (`WeakAdminPassword`), `change-me-please` (16) est accepté ; entre chevrons, `<change-me…>`
  est refusé par la forme `<…>`. Asymétrie antérieure à la story, non traitée.
- **`docker-compose.yml` porte `image:` et `build:`** : effet développeur écrit (AC5), non empêché.
- *(Retiré en P2 : `KESH_ADMIN_PASSWORD=<GENERATE_ME: …>` décommenté — traité par l'AC16, C81.)*
- *(Retirés en P3, C83 : le message `error-invoice-pdf-gone` face à un montage déplacé, et la recette de
  déplacement non mesurée sur DSM — sans objet, aucun montage ne change.)*
- **`KESH_*_HOST_DIR` sans effet avec `docker-compose.prod.yml`** (AC4 abandonnée, C83) : écrit au
  gabarit, au manuel, au CHANGELOG et dans un commentaire de P ; un exploitant Synology ne peut pas
  monter ses dossiers sur un dossier partagé DSM sans modifier lui-même son compose (ce qu'il fait à ses
  risques, « adaptations locales »). **Propriétaire : issue #558** (version configurable, avec sa
  procédure de déplacement des données).
- **Démarrage rapide du site** (`website/index.html:189`, F3-10) :
  `docker compose -f docker-compose.dev.yml up -d` juste après `git clone`, sans `.env` — refusé
  (`WeakAdminPassword`, défaut `admin` du compose de dev), même classe que la brochure (F2-11) et que
  `docker-compose.dev.yml` ci-dessus : **à ajouter par l'orchestrateur à l'issue de F14**, non traité ici
  (pile de développement non distribuée).
- **`CLAUDE.md:180`** (recette E2E du dépôt) écrit `KESH_ADMIN_PASSWORD='<12+ caractères>'` : gabarit
  entre chevrons, refusé par l'AC16 s'il est recopié tel quel — le `CLAUDE.md` n'est pas modifié par la
  story (AC14), **signalé au Project Lead**.
- **Le `\paragraph{Note Synology (chemins symboliques).}`** de `sec:inbox-import` (`Overfull` des lignes
  `821--825`, R3-3 = F3-5) : défaut de mise en page **antérieur**, hors des dix tableaux de l'AC12 j —
  non traité, écrit au Dev Agent Record.
- **Un administrateur déjà créé avec le placeholder** : l'AC16 refuse le démarrage tant que la ligne
  reste dans `.env`, mais ne change pas un compte existant ; le CHANGELOG dit de changer son mot de passe
  (AC13). Non détectable par le code sans lire le hachage — non traité.
- **`$` dans une valeur de `.env`** : écrit (AC6 h, AC12 a), non contrôlé — aucun test ne peut lire le
  `.env` de l'exploitant ; Compose avertit à l'`up`.

### Fichiers touchés

`docker-compose.yml`, `docker-compose.prod.yml`, `.env.example`, `crates/kesh-api/src/config.rs` (trois
messages — `IncompleteSmtpConfig`, `InsecureJwtSecret`, `InsecureAdminPassword` —, commentaires et
doc-comments, la constante `TEMPLATE_PLACEHOLDERS`, la fonction `is_template_placeholder`, **deux
contrôles** et six tests unitaires — AC16),
`crates/kesh-api/src/main.rs` (un message), `crates/kesh-api/src/lib.rs` (un commentaire),
`crates/kesh-api/Cargo.toml`, `Cargo.lock`, `crates/kesh-api/tests/configuration_transmise.rs` (neuf),
`crates/kesh-api/README.md` (AC16 d), `README.md` (AC16 d), `.github/workflows/ci.yml`, `docs/ci.md`,
`docs/manual/fr/admin-manual.tex` + `.pdf`, `docs/manual/fr/marketing-brochure.tex` + `.pdf` (AC12 h),
`DOCKER_START.md`, `CHANGELOG.md`. `docs/manual/shared/kesh-style.sty` **non touché** (AC12 j). Aucune
migration ; `kesh-db` non touché. **Un seul comportement du code modifié** : le refus du placeholder du
gabarit (`GENERATE_ME` ou forme `<…>`) comme secret JWT et comme mot de passe admin, contrôlé avant la
longueur (AC16, #557) ; le reste du code, textes seulement. **`docker-compose.prod.yml` : environnement
seulement**, `volumes:` intacts hors un commentaire (AC4 abandonnée, C83).

**Règle de découpage** (CLAUDE.md, § *Règle de splitting préventif*) : un crate ; dans `kesh-api`, **trois**
modules de code (`config`, `main`, `lib`) — `config` pour des textes **et** le contrôle de l'AC16 (une
condition de plus dans une fonction existante), `main` et `lib` pour des textes —, plus le test neuf et
des fichiers de configuration et de documentation. Recompté après la remédiation P2 : l'AC16 étendue
n'ajoute **aucun** module (les deux contrôles, la constante et les tests vivent dans `config.rs` ; les
deux README, la brochure et `DOCKER_START.md` sont de la documentation). Seuil (« plus de 5 modules ») **non
franchi**. Cette fiche est elle-même le produit d'un découpage (C77). Signal D5 : la P1 de la 15-11a
remonte un HIGH (R1-1 = F1-1) après une P3 de la 15-11 à MEDIUM, mais c'est un défaut **d'origine**,
distinct, qui ne vient d'aucune remédiation (la troisième source manquait depuis la conception de la
liste) — pas de recyclage, pas de découpage (C79). **P3 (C83)** : sévérité MEDIUM égale à la P2, et
**recyclage** constaté — la recette de déplacement a produit un défaut à trois passes de suite (P1 HIGH
R1-1, P2 MEDIUM R2-1/R2-2, P3 MEDIUM R3-1 = F3-1 et F3-2), par la même machinerie (un analyseur de `.env`
qui doit imiter Compose). **Sortie du recyclage par retrait, non par découpage** (décision de
l'orchestrateur) : l'AC4 qui l'exigeait est abandonnée, la machinerie disparaît ; la version
configurable part à #558. Modules de code recomptés : toujours **3**.

### Project Structure Notes

Le test vit avec les tests d'intégration de `kesh-api` (patron `audit_route_registry.rs`,
`audit_label_registry.rs` : tests qui lisent les fichiers du dépôt). Il ne monte ni base ni serveur.

### References

- Issue #550 ; #534 ; #551, #552 (hors périmètre) ; fiche `15-11-configuration-transmise.md` (index du
  découpage ; version complète au commit d69fdcca) ; fiche sœur `15-11b-lecture-unique-des-variables.md` ;
  fiche `15-7b2-remise-a-zero.md` (worktree `kesh-15-7`), choix C-15-7-48, C-15-7-51, C-15-7-52.
- Rapports de validation : `target/gate-logs/15-11-p{1,2,3}-{R,F}.md`.
- `DOCKER_START.md:13, 23, 38-40, 60-63, 70, 106` ; `.env.example:26-28, 58-82, 152-156, 176-177, 203-208, 235, 267-268, 272-279` ;
  `auth/bootstrap.rs:62-69` ; `routes/admin.rs:471-490` ; `mail/smtp.rs:56, 73-74`.
- `crates/kesh-api/src/config.rs:35-39, 111-116, 158, 319, 619-637, 657-671, 1031-1036`, tests `:1800-1820`,
  `:1915-1937` ; `routes/onboarding.rs:265-279` ; `routes/issued_invoice_pdf.rs:389` ; `main.rs:296-299, 316-323, 337` ;
  `lib.rs:1040` ; `frontend/src/routes/(app)/users/+page.svelte:16`.
- `Dockerfile:41-48` ; `docker-compose.yml` ; `docker-compose.prod.yml` ; `docker-compose.dev.yml` ; `docs/ci.md:12, 117`.
- `admin-manual.tex:218-254, 552-566, 646-793, 991, 1239, 1289, 1704-1714`.
- Rapports de la validation P1 de la 15-11a : `target/gate-logs/15-11a-p1-{R,F}.md` ; issue #557.
- Rapports de la validation P3 de la 15-11a : `target/gate-logs/15-11a-p3-{R,F}.md` ; issue #558
  (montages configurables de `docker-compose.prod.yml`, version future). Sites ajoutés en P3 :
  `config.rs:1811` ; `admin-manual.tex:243-244, 988, 1230, 1384, 1518, 821-825` ;
  `docker-compose.prod.yml:97-101, 107-108, 119-127` ; `.env.example:170-177, 199-204` ;
  `website/index.html:189` ; `CLAUDE.md:180`.
- Rapports de la validation P4 de la 15-11a : `target/gate-logs/15-11a-p4-{R,F}.md` ; prompt versionné
  `15-11a-validate-prompt-p4.md` (commit `f80cf585`). Sources ajoutées en P4 :
  `dotenvy-0.15.7/src/parse.rs:165-220` ; `admin-manual.tex:534, 1704-1717, 1734-1749` ; fiche
  `15-7b3-reparation-des-installations-atteintes.md` (worktree `kesh-15-7`).
- Rapports de la validation P2 de la 15-11a : `target/gate-logs/15-11a-p2-{R,F}.md`. Sites ajoutés en P2 :
  `config.rs:35-45, 111-123, 184, 619-649, 663-667, 1800-1803` ; `main.rs:44, 63-68` ;
  `.env.example:6-8, 78-82, 85-86` ; `docker-compose.prod.yml:27-29, 74` ;
  `admin-manual.tex:530-548, 656-794 (dix tableaux), 662, 690, 1238` ; `kesh-style.sty:141, 224` ;
  `marketing-brochure.tex:542` ; `crates/kesh-api/README.md:38, 55-57, 62-65` ; `README.md:79-81` ;
  `messages.ftl` (`error-invoice-pdf-gone`) ; `DOCKER_START.md` (douze commandes `docker-compose`).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement, autonomie — consignes de l'Epic 15). Worktree
`/home/gcorbaz/devel/kesh-15-11a`, **cible cargo propre** `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-15-11a/target`
(compilation à froid au début de la session) ; bases dédiées `kesh_1511a` (gate) et `kesh_e2e_1511a` (E2E).

### Debug Log References

Sorties brutes dans le scratchpad de la session (non versionnées) : `t1-rouge.log` (T1), `mut1511a/M*.log`
(T5), `t6-1511a/ac10.out` (AC10), `gate-complet.log`, `vitest.log`, `e2e-1511a/e2e.log`.

### Completion Notes List

**T0 — relevé, refait le 2026-10-08 sur la base `ef39dd54` (`origin/main`).**
- Inventaire : la commande de la § *Inventaire fermé* rend **40** noms, plus `RUST_LOG` (`logging.rs:130`,
  `EnvFilter::DEFAULT_ENV`) = **41** — identiques à la table. Classes : 22 transmises par les deux, 4 par un
  seul, 12 par aucun (38 à transmettre), 2 fixées par l'image, 1 interdite ; 3 hôte, 4 MariaDB ; **0 fantôme**
  après la story. **Aucun écart**, aucune variable ajoutée par une story mergée entre-temps.
- § *Le vide avant la 15-11b* : table confirmée au code (`config.rs:809-829`, `:831-833`, `:863-937`,
  `:1031-1036`, `:1070-1083`, `:1367`, `:1395-1410` ; `onboarding.rs:45-53`). **Mesuré** :
  `std::fs::create_dir_all("")` rend `Ok(())` et `Path::new("").join("f.keshbackup")` rend `"f.keshbackup"`
  (relatif) — la sauvegarde pré-import d'un `KESH_ADMIN_BACKUP_DIR` vide s'écrit donc dans le répertoire
  courant, `/app` (`WORKDIR`, `Dockerfile:41`).
- Liste « relisez », trois sources : (1) `ConfigError` atteignables par une variable nouvellement transmise —
  `InvalidBoolValue` (`KESH_SMTP_TLS`, `KESH_FEATURE_FORGOT_PASSWORD`), `InvalidCookieSecureValue` (P),
  `IncompleteSmtpConfig` ; étendues par l'AC16 : `InsecureJwtSecret`, `InsecureAdminPassword` ; (2)
  `process::exit` de `main.rs` atteignables : `:67` (erreur de `Config`) et `:322` (mailer non construit avec
  le mot de passe oublié) — les neuf autres (`:87`, `:126`, `:133`, `:140`, `:161`, `:167`, `:185`, `:239`,
  `:373`) dépendent de la base, des migrations, de l'i18n ou du bind, non d'une variable ajoutée ; (3)
  interpolations dont la forme change : `KESH_LOG_FILE_PATH` et `KESH_ADMIN_PASSWORD` (les deux compose).
  **Revérifiée au T8** contre `git diff ef39dd54 -- docker-compose.yml docker-compose.prod.yml` : seules ces
  deux interpolations changent, plus la ligne `image:` de Y et les 28 ajouts ; **aucune ligne de montage**
  (le bloc `volumes:` de P ne gagne que le commentaire du T2). Aucun écart.
- Constat rassurant revérifié : les seules lignes **actives** de `.env.example` parmi les 16 sont
  `KESH_COOKIE_SECURE=true`, `KESH_LANG=fr`, `KESH_PASSWORD_MIN_LENGTH=12`, `KESH_BANK_IMPORT_MAX_MB=10`.
- Compose : `Docker Compose version 2.40.3+ds1-0ubuntu1`. Mesures (`env -i … docker compose config --format
  json`, scratchpad `t0-1511a`) : `${A:-}` absente → `''` ; posée → la valeur ; `C=` → `''` ; `${D-/def}`
  absente → `/def` ; `E=` vide → `''` ; `P1=pa$word` → `pa` (quatre avertissements « The "word" variable is
  not set ») ; `pa$$word`, `'pa$word'`, `"pa$$word"` → réaffichés `pa$$word` (valeur reçue `pa$word`) ;
  `"it's pa\$word"` → `it's pa$$word` (reçue `it's pa$word`). `dotenvy` 0.15.7 (petit programme du
  scratchpad, `from_path_iter`) : `pa$word` → `pa`, `pa$$word` → `pa`, `'pa$word'` → `pa$word`, `"pa$$word"`
  et `"pa$word"` → **erreur d'analyse**, `"it's pa\$word"` → `it's pa$word`. Le tableau de la fiche et R4-13
  sont confirmés.
- Synology Container Manager : version non mesurable depuis le poste — non mesuré.

**T1 — rouge constaté, exactement l'attendu** (`cargo nextest run -p kesh-api --test configuration_transmise`) :
(T) **29** écarts — les 28 couples manquants (12 « par aucun » × 2, `KESH_COOKIE_SECURE` dans P, `KESH_LANG`,
`KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB` dans Y) et l'`image:` absente de Y ; aucun rouge de
montage, aucune faute de frappe. (V) **5** — `KESH_ADMIN_PASSWORD` dans Y (`:-changeme`) et dans P (`${…}`),
`KESH_LOG_FILE_PATH` dans les deux, le contrôle `VIDE_SIGNIFIANT` de `.env.example:208` ; aucun rouge
`AJOUTS`. (E) **0**. (F) **1** — `KESH_ADMIN_RESET` dans `.env.example`. Garde `LUES` et (S) verts — après
une correction du test lui-même : `LUES` était triée selon la locale (`FILE_BYTES` avant `FILES_PER_RUN`) et
la garde de tri par octets l'a refusée (C-15-11a-1).

**T2-T4.** 28 lignes ajoutées (15 dans Y, 13 dans P — recompté : 16 et 14 occurrences de `${KESH_…:-}`,
`KESH_ADMIN_PASSWORD` comprise) ; `KESH_PRODUCTION_RESET` : **une ligne par fichier** (`grep -n`). Défauts des
quatre variables à double forme = défauts du code : `KESH_COOKIE_SECURE` (Y `true`, `config.rs:1031-1036`
`true`), `KESH_LANG` (P `fr`, code `"fr"`), `KESH_PASSWORD_MIN_LENGTH` (P `12`, code 12),
`KESH_BANK_IMPORT_MAX_MB` (P `10`, code 10). AC16 : constante `TEMPLATE_PLACEHOLDERS`, fonction
`is_template_placeholder`, deux contrôles avant la longueur ; tests écrits d'abord — **5 rouges** (placeholder
du gabarit accepté pour les deux variables, `generate_me` accepté, `<…>` accepté), le témoin
`config_accepts_generated_jwt_secret` vert —, puis verts. Témoin admin existant : le mot de passe
`"valid-test-pw-12chars"` de `set_minimum_required()` (`config_from_env_with_database_url` et la plupart des
tests). Aucun test équivalent à `config_accepts_generated_jwt_secret` n'existait (`TEST_JWT_SECRET` n'est pas
hexadécimal) : il est écrit. Gate ciblé de l'AC14 (`binary(configuration_transmise) | (package(kesh-api) &
(test(jwt_secret) | test(admin_password)))`) : **28 passés** ; `cargo fmt --check` et `cargo clippy
--workspace --all-targets -D warnings` verts.

**T5 — 28 mutations, toutes rouges**, chacune restaurée par `git checkout` puis `touch`, `git diff --stat`
vide après chacune (script du scratchpad `mut1511a/mutations.py`). Rouges observés (test : message) :
M1 (T, P) · M2 (T, Y) · M3 (T ×2 : Y et P ; E : sans ligne d'affectation) · M4 (T : exception transmise) ·
M5 (T : `FixeeParImage` sans `ENV`) · M6 (T : exception transmise) · M7 (E et F) · M8 (F : `KESH_ADMIN_` +
`RESET`) · M9 (V : clé sans valeur) · M10 (V : autre variable interpolée ; **et F** : jeton `KESH_SMTP_HSOT`,
non annoncé par la fiche) · M11 (T : `env_file`, renvoi à C71) · M12 (E) · M13 (T : `image:`) · M14 (T :
source du montage ; E : `HOTE`) · M15 (V : `VIDE_SIGNIFIANT`) · M16 (V : `SANS_DEFAUT`) · M17 (V : « ou
absent ») · M18 (F : `KESH_ESSAI_FANTOME` dans `admin-manual.tex`) · M19 (V : `SANS_DEFAUT` non commentée ;
**et** `config_rejects_admin_password_left_at_template_placeholder` : aucune ligne `#KESH_ADMIN_PASSWORD=`) ·
M20 (`config_rejects_jwt_secret_left_at_template_placeholder`, `…_jwt_secret_generate_me_case_insensitive`,
`…_jwt_secret_and_admin_password_in_angle_brackets` — ce dernier sur
`<un-gabarit-de-plus-de-32-caracteres-ici>` accepté) · M21 (V : `AJOUTS`) · M22 (V : `SANS_DEFAUT`) · M23
(`config_rejects_jwt_secret_left_at_template_placeholder` : l'assertion de montage « ne porte plus le
placeholder `GENERATE_ME` » rougit la première) · M24 (`…_admin_password_left_at_template_placeholder`,
`…_admin_password_generate_me_case_insensitive`, `…_in_angle_brackets`) · M25 (`…_in_angle_brackets` : `<x>` →
`WeakJwtSecret { 3 }` ; `…_jwt_secret_generate_me_case_insensitive` : `GENERATE_ME` **et** `xchange-mex` →
`WeakJwtSecret { 11 }` — après C-15-11a-2) · M26 (`…_in_angle_brackets`) · M27
(`…_admin_password_generate_me_case_insensitive` : `generate_me` → `WeakAdminPassword { 11 }` ;
`…_in_angle_brackets` : `<x>` → `WeakAdminPassword { 3 }`) · M28 (T : montage fixe exigé, renvoi à #558 et
C83). M20 et M25 rejouées après le commit `80231b62`.

**T6 — Docker, sans démarrer** (scratchpad `t6-1511a`, `t6-dry`, `t6-pieges` ; `docker ps -a`, `docker network
ls`, `docker volume ls` identiques avant/après : aucun conteneur, réseau ni volume créé).
- AC10 (compose finaux, `env -i`, sortie JSON recomptée par script) : (i) complet — **38/38** clés dans les
  deux, valeur posée partout sauf `DATABASE_URL` de Y (`mysql://kesh:kesh_dev@mariadb:3306/kesh`, composée) et
  `KESH_HOST` de P (`0.0.0.0`) ; 0 avertissement. (ii) absent — 38 clés présentes, **aucun `null`** ; Y : **22**
  non vides, **16** vides (15 ajouts + `KESH_ADMIN_PASSWORD`), **0** avertissement ; P : **22** non vides, **16**
  vides (13 ajouts + `DATABASE_URL`, `KESH_JWT_SECRET`, `KESH_ADMIN_PASSWORD`), **2** avertissements
  (`DATABASE_URL`, `KESH_JWT_SECRET`) ; `KESH_LOG_FILE_PATH` = `/var/log/kesh/kesh.log` dans les deux. (iii)
  `KESH_SMTP_PORT=` et `KESH_LOG_FILE_PATH=` → `''` dans les deux. (iv) `KESH_DOCUMENTS_HOST_DIR=/essai/documents`
  → source `/essai/documents` dans Y, `<répertoire d'essai>/documents` dans P ; sans la ligne,
  `<répertoire d'essai>/documents` dans les deux.
- AC5 : `gcorbaz/kesh:latest` **absente** localement avant l'essai (`docker image ls gcorbaz/kesh` : seuls
  `0.1.2-rc1`, `0.1.3-rc1`, `v011-5-test`) ; dans un répertoire ne contenant que le compose et un `.env`
  d'essai, `docker compose --dry-run up -d` : **avant** (compose de `ef39dd54`) → « kesh-api ==> naming to
  avant-kesh-api … Built » (construction) ; **après** → « kesh-api Pulling / Pulled », **zéro** ligne `build`.
  Pas de repli `pull_policy` nécessaire.
- AC11 : `docker compose -f … config -q` **sans `.env`** : exit 0 pour les deux (P : deux avertissements, pas
  d'échec) — l'étape CI n'a pas besoin de `.env` d'essai. Vérifié en plus : une clé dupliquée fait échouer
  `config -q` (« mapping key "KESH_SMTP_HOST" already defined », exit 1).
- Vérification des placeholders de l'AC12 f, **copiée du `.tex` final** (`sudo` retiré ; lignes de 28 et 70
  caractères), rejouée contre Y **et** P sur onze `.env` : neuf piégés **montrés** (`<GENERATE_ME: …>` nu,
  avec `export `, avec espaces autour de `=`, entre guillemets, avec `\r` ; `<nouveau-mdp-12+>` ; `<mot de
  passe fort, min. 12 caracteres>` ; `generate_me-et-12-caracteres` ; `"   <x>  "`, rendu `'   <x>  '` par
  Compose) ; deux témoins **muets** (64 hexadécimaux, `pw<valide>12chars`). Le motif voit la forme rendue par
  Compose dans tous les cas : aucune correction nécessaire.

**T7.** Étape `Validate compose files` dans `docker-build` (avant le build) ; `docs/ci.md:12` et `:117`.

**T8 — documentation.**
- Manuel : sites de la fiche traités (AC12 a-j, AC15), plus quatre de la propagation (C-15-11a-4).
  Inventaire des gabarits entre chevrons (grep du T8) : les sept lignes du relevé P4, relocalisées
  (`.env.example:103`, `:109` ; `admin-manual.tex:246`, `:250` — devenue commentaire —, `:1018` (ex-`:988`),
  `:1260` (ex-`:1230`) ; `CLAUDE.md:180`), **plus un site neuf, lu** : `CHANGELOG.md:37`, l'entrée **Sécurité**,
  qui cite `KESH_JWT_SECRET=<GENERATE_ME: …>` comme exemple de la valeur désormais refusée — description du
  défaut, non gabarit à recopier ; laissé. Mise en page des dix tableaux : C-15-11a-3.
- PDF régénérés (`make fr`), contrôlés aplatis avec ligatures **et apostrophes typographiques** normalisées
  (`’` → `'` : sans elle, « Rejeté s'il contient » rend 0 — le PDF porte `s’il`) : présents « Seules les
  variables listées sous environment: », « apostrophes simples », `pa$$word`, « réaffiche tout », `grep
  KESH_JWT_SECRET .env`, « Rejeté s'il contient » (1), « GENERATE_ME (sans égard à la casse) », « gabarit
  entre chevrons » (2), « Remplacez chaque valeur entre chevrons », « Optionnel : sans KESH_ADMIN_PASSWORD »,
  « re-télécharger le compose », « deux gestes », « chemins des montages », « sans effet » (4),
  `HÔTE:PORT/BASE`, « Relisez votre .env » ; **absent** `docker compose restart kesh-api` (0). Les deux lignes
  de la commande de vérification se retrouvent **chacune sur une ligne** de `pdftotext -layout` (espaces
  normalisés des deux côtés) ; les 30 lignes `KESH_…: ${…}` des `lstlisting` des deux gestes sont entières.
  Cellules `KESH_STATIC_DIR` / `KESH_LOCALES_DIR` entières au `-layout` (« Fixé par l'image Docker
  (/app/static) ; frontend/build hors Docker », « … (/app/locales) ; dossier du dé-pôt hors Docker »), colonne
  « Défaut host » des chemins d'hôte entière.
- `.log` : état d'avant relevé par un `make admin` sur `1bd55491` (**67** `Overfull \hbox`) ; après : **52**,
  **0** entre `\label{sec:env-vars}` et `\subsubsection{Import de factures depuis un dossier}`, **aucun nouveau**
  ailleurs (comparaison des boîtes par contenu). Le `\paragraph{Note Synology (chemins symboliques).}` de
  `sec:inbox-import` déborde toujours (défaut antérieur, hors périmètre).
- `user-manual.tex` : grep `SMTP`, `KESH\\_`, `démonstration` — `:295`, `:1049` (`KESH\_SMTP\_*`), `:2090`,
  `:2099` : rien à modifier ; son PDF, régénéré sans changement de source, a été **restauré**.
- Brochure : `:542` corrigée, PDF régénéré, phrase contrôlée aplatie.
- `DOCKER_START.md` : douze commandes `docker-compose` → `docker compose` (`grep -nE 'docker-compose '` rend
  0) ; « Compose v1 non mesuré ». `README.md`, `crates/kesh-api/README.md` (AC16 d ; l'exemple de requête
  `"password": "changeme"` et l'inventaire partiel restent hors périmètre). **Effet sur la pile de dev**, écrit :
  un `.env` copié du gabarit sans remplacer le secret fait refuser `cargo run` (`main.rs:44` lit `.env`) et
  `docker-compose.dev.yml` (`${KESH_JWT_SECRET:-dev-secret…}` cède à `.env`) — voulu, message clair ; le
  contrôle `grep -rn GENERATE_ME frontend/ scripts/ .github/` (vide) ne voit pas cet effet, qui passe par une
  copie.
- Grep de l'AC7 : `grep -rn 'KESH_ADMIN_RESET' --exclude-dir={target,node_modules,_bmad-output,.git}
  --exclude=CHANGELOG.md .` → **vide**.
- Grep de l'AC15 (`compose restart|docker-compose restart|redémarr` sur manuels, `*.md`, `docs/*.md`,
  `.env.example`, compose, `website`) : traités `admin-manual.tex` § « Comment .env atteint Kesh » (règle
  écrite), `:1021` (ex-`:991`), `:1269` (ex-`:1239`, « (Optionnel) » retiré), `:1319` (ex-`:1289`), `:2238` ;
  `:1268` (ex-`:1238`) lu, corrigé par ricochet ; déjà justes : `:1262` (ex-`:1232`), `:1741` (procédure de
  mise à jour, `up -d kesh-api`), `:2231` (break-glass, `up -d`) ; `DOCKER_START.md` § *Redémarrer* (consigne `up -d`
  ajoutée) et ex-`:106` (« Puis appliquez : `docker compose up -d` ») ; hors sujet : `admin-manual.tex:649`
  (auto-restart DSM), `:847`, `:863`, `:1121`, `:1583`, `:2057`, `:2274` ; `.env.example:170`, `:185` ;
  `docs/testing.md` (conteneur MariaDB de dev) ; `CHANGELOG.md:622`, `:630`, `:742`, `:749` (versions
  publiées, historiques). La recette de la 15-7b2 n'est pas touchée : **rappel**, la 15-7b2 rebasée écrit
  `docker compose up -d` dans sa recette.
- CHANGELOG `[0.13.0]` : entrée **Corrigé** (#550) et section **Sécurité** (#557).


**T9 — gates réellement exécutés**, cible cargo `/home/gcorbaz/devel/kesh-15-11a/target` :
- **Gate backend complet** au commit `edd45a6c` (dernier commit de code Rust : `80231b62` ; `edd45a6c`
  porte le manuel, le CHANGELOG, `DOCKER_START.md`, les READMEs, la brochure **et l'étape `Validate compose
  files` de `.github/workflows/ci.yml`** — comportement de CI, **non exécuté localement** : seules ses deux
  commandes `docker compose config -q` ont été rejouées à la main en T6 ; correction A-L2 de la revue P1), base `kesh_1511a` remise à zéro juste avant (`DROP`/`CREATE`, migrations, seed ; aucun
  redémarrage du conteneur), après `wait-kesh.sh` : `scripts/test-fast.sh` (`fmt --check`, `clippy --workspace
  --all-targets -D warnings`, nextest) — **2827 passés, 4 ignorés, 0 échec**.
- **Frontend** (non touché par la story, exécuté quand même) : `npm run check` 0 erreur (27 avertissements
  préexistants), `lint-i18n-ownership` PASS, `test:unit` **111 fichiers / 1086 tests passés**, `build` vert.
- **E2E complet** au même commit, base `kesh_e2e_1511a` reconstruite, backend `target/debug/kesh-api` sur le
  port **3004**, `KESH_COOKIE_SECURE=false`, `KESH_STATIC_DIR` du worktree, `KESH_JWT_SECRET` et
  `KESH_ADMIN_PASSWORD` générés (C-15-11a-5) : **244 passés, 7 échoués, 19 ignorés (10,1 min)**. Les sept
  échecs sont **exactement** les sept KF-029 (#97) de `docs/testing.md` § *Les échecs attendus* :
  `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
  `:150`. Aucun huitième variable ce run ; run de l'après-midi (KF-045 hors fenêtre). Backend arrêté.
- Commit postérieur aux gates : documentation seule (`admin-manual.tex` — ligne `openssl rand -base64 32` du
  bloc Synology rétablie, C-15-11a-4 —, son PDF, la fiche, le registre). Après lui : `make admin` refait, 52
  `Overfull`, aucun nouveau, 0 dans `sec:env-vars`.
- Contrôles de l'AC14 : `grep -rn GENERATE_ME frontend/ scripts/ .github/` → **vide** ; le montage E2E local
  n'emploie ni `GENERATE_ME` ni forme `<…>`.

**À signaler (hors fiche, pour l'orchestrateur)** : `CLAUDE.md:180` (recette E2E) écrit
`KESH_ADMIN_PASSWORD='<12+ caractères>'`, désormais refusé s'il est recopié tel quel (le `CLAUDE.md` n'est pas
modifié) ; `website/index.html:189` et `docker-compose.dev.yml` (F14, F3-10) restent à verser à leur issue ;
fiche 15-7b2 / 15-7b3 : le report des zones partagées est à faire (§ *Coordination*) ; #557 et #558 : mises à
jour de texte demandées par C83/C84.

### File List

- `docker-compose.yml`, `docker-compose.prod.yml`, `.env.example`
- `crates/kesh-api/src/config.rs`, `crates/kesh-api/src/main.rs`, `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/tests/configuration_transmise.rs` (neuf)
- `crates/kesh-api/Cargo.toml`, `Cargo.lock` (`yaml-rust2` 0.13.0, dépendance de test ; entrent `hashlink`
  0.12.2 — à côté de 0.10.0, sans conséquence — et `foldhash` 0.2.0)
- `crates/kesh-api/README.md`, `README.md`, `DOCKER_START.md`, `CHANGELOG.md`
- `.github/workflows/ci.yml`, `docs/ci.md`
- `docs/manual/fr/admin-manual.tex` + `.pdf`, `docs/manual/fr/marketing-brochure.tex` + `.pdf`
- `_bmad-output/implementation-artifacts/15-11a-compose-transmet-la-configuration.md`,
  `_bmad-output/implementation-artifacts/sprint-status.yaml` (lignes 15-11 / 15-11a / 15-11b reportées),
  `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md` (C-15-11a-1 à 5)
- Non touchés, comme prévu : `Dockerfile`, `docs/manual/shared/kesh-style.sty`, `user-manual.tex` (+ PDF),
  `kesh-db`, frontend.

## Change Log

- 2026-10-08 — **Création par découpage de la 15-11** (agent de découpage, autonomie ; choix **C77**).
  Reprend de la 15-11 (version d69fdcca) les AC 1 à 14 et 16 (renuméroté 15), adaptés : le test ne lit
  plus le code (liste fermée `LUES`, écrite en dur, reproduite par un `grep` documenté) ; la règle
  générale « vide = absent », le trim et leurs textes (AC6 g et AC12 i **de la 15-11**, version d69fdcca : `.env.example:109-111` et `admin-manual.tex:1306` ; entrée CHANGELOG **Modifié**) passent
  à la 15-11b. Ajout : § *Le vide avant la 15-11b* (vérifié au code : aucun refus de démarrer, d'où un
  merge seul sans danger). **Remédiation de la validation P3** de la 15-11 qui tombe ici :
  - **R3-1 = F-1 (MEDIUM)** : rouge exact du T1 corrigé — (E) rougit sur les trois `KESH_*_HOST_DIR`, pas
    sur `KESH_ADMIN_RESET` (prose, non une affectation) ; NOM de (E) = `[A-Z][A-Z0-9_]*` (la prose
    `# is_demo=true` n'est pas une affectation) ; (S) l'exerce.
  - **R3-2 (MEDIUM)** : la liste « relisez » ajoute le refus de `main.rs:316-323` (mailer non construit avec
    le mot de passe oublié) et l'activation de la réinitialisation par e-mail ; le T0 relève les
    `process::exit` de `main.rs`.
  - **F-10 (LOW)** : le défaut `change-me…` du secret JWT est une garde, conservée ; `KESH_JWT_SECRET`
    obligatoire écrit (§ *Le défaut*, AC6 d, AC12 e, AC13).
  - **LOW** : `document_storage.rs:174` n'est pas de production (R3-4) ; M3, M7, M14 rougissent plusieurs
    fois, écrit (R3-6) ; jointure des cinq lignes du contrôle `VIDE_SIGNIFIANT` (R3-8 = F-7) ;
    `docker-compose.prod.yml:67-69` (R3-9) ; fenêtre compose neuf / image ancienne, `docker compose pull`
    (F-5) ; troisième écran à 12 en dur (F-6) ; `docs/ci.md` (F-8). R3-5 (registre) : traité dans C77.
  - Comptes **recomptés depuis cette fiche** : AC **15**, tâches **10** (T0-T9), mutations **19** (toutes
    rouges ; M3 trois rouges, M7 et M14 deux), modules de code **3** (`config`, `main`, `lib`, textes
    seulement) — seuil de découpage non franchi.
- 2026-10-08 — **Remédiation de la validation P1 de la 15-11a** (agent de remédiation, autonomie ;
  décisions de l'orchestrateur et choix **C79**). Passe P1 : **deux lentilles Opus 5.5** en contexte frais
  — R (regression hunter) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW ; F (full-scope adversary) : 0 CRITICAL,
  1 HIGH, 1 MEDIUM, 6 LOW ; R1-1 = F1-1, R1-7 = F1-7 — soit **14 findings distincts** (1 HIGH, 2 MEDIUM,
  11 LOW), tous traités :
  - **R1-1 = F1-1 (HIGH)** : la liste « relisez » gagne une **troisième source**, les interpolations du
    compose nouvellement prises en compte ou dont la forme change (montages `KESH_*_HOST_DIR` de P,
    `KESH_LOG_FILE_PATH`, `KESH_ADMIN_PASSWORD`) — T0, AC12 f, AC13 ; avertissement **en tête et en gras**
    du CHANGELOG et de la procédure de mise à jour, avec la recette de déplacement (`grep`, `sed`, `rsync`,
    `diff -rq`, rejouée au scratchpad) et la consigne de ne refiger aucun PDF « disparu » ; AC10 (iv) ;
    revérification au T8 contre le diff des compose.
  - **R1-2 (MEDIUM) + #557** : **AC16** neuf — `Config::from_env` refuse un `KESH_JWT_SECRET` contenant
    `GENERATE_ME` (sans égard à la casse), même variante `InsecureJwtSecret` (C79), message et doc-comment
    `config.rs:35-38` corrigés, trois tests unitaires dont un qui lit la ligne réelle du gabarit, mutations
    M20 et M23 ; F-10 réécrit au plus juste (§ *Le défaut*) ; AC6 d, AC12 e/f, AC13 (**Sécurité**),
    `DOCKER_START.md`. La fiche passe à `closes #550`, `closes #557`.
  - **F1-2 (MEDIUM)** : `DOCKER_START.md:38-40` (pas de compte `admin`/`admin` : `/setup` ou variables) et
    `:13`, `:23` (démarrage sans `.env` refusé : générer `KESH_JWT_SECRET` d'abord) — AC5 ; motifs ajoutés
    au grep du T8 (`Mot de passe: .admin.`, `up (-d )?--build`, `GENERATE_ME`, `_HOST_DIR`).
  - **LOW** : F1-3 (au-dessous de 12, les écrans refusent quand même) ; F1-4 (`$` → `$$` ou apostrophes
    simples, **mesuré** Compose 2.40.3, table § *Le défaut*, AC6 h, AC12 a) ; F1-5 (mentions nouvelles de
    `KESH_PRODUCTION_RESET` signalées à la 15-7b2, « installation restée en démonstration ») ; F1-6
    (`admin-manual.tex:991` dans l'AC15) ; R1-7 = F1-7 (« **chaque** ligne d'affectation », cas (S)) ;
    F1-8 (liste fermée `AJOUTS`, M21) ; R1-3 (renvois « AC6 g, AC12 i » qualifiés « de la 15-11 » ;
    numérotation des findings expliquée en tête) ; R1-4 (`DOCKER_START.md:70`, et `:13`, `:23` dans
    l'effet inverse) ; R1-5 (`KESH_ADMIN_PASSWORD` de P en `${…:-}`, `SANS_DEFAUT` n'admet plus que
    `${NOM:-}`, M22) ; R1-6 (fiche 15-7b2, l. 540-541 : hors de ce dépôt de travail, **signalé** à
    l'orchestrateur — *périmé en P2, R2-10 : les commits `80bc16df` et `a23f832b` du worktree `kesh-15-7`
    avaient déjà fait passer ces renvois à « 15-11a » ; seul reste le motif de contrôle, § Coordination*) ; R1-8 (M8 visé par le texte, non par le numéro de ligne).
  - **Signal D5** : HIGH en P1 de la 15-11a, mais défaut d'origine distinct, non issu d'une remédiation —
    pas de découpage (§ *Règle de découpage*).
  - Comptes **recomptés depuis cette fiche** : AC **16** (AC16 neuf), tâches **10** (T0-T9), mutations
    **23** (M1-M23, toutes rouges ; M3 trois rouges, M7 et M14 deux), modules de code **3** (`config`,
    `main`, `lib`) — seuil de découpage non franchi.
- 2026-10-08 — **Remédiation de la validation P2 de la 15-11a** (agent de remédiation, autonomie ;
  décisions de l'orchestrateur et choix **C81**). Passe P2 : **deux lentilles Sonnet 5.5** en contexte
  frais — R (regression hunter) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 7 LOW ; F (full-scope adversary) : 0
  CRITICAL, 0 HIGH, 3 MEDIUM et, **recompté depuis le rapport**, 8 LOW (F2-4 à F2-11 ; le bilan du
  rapport en annonce 7). Recoupements : R2-3 = F2-1, R2-2 = F2-2 (+ F2-9), R2-4 ≈ F2-5 + F2-6 — soit
  **17 findings distincts** (5 MEDIUM, 12 LOW), tous traités :
  - **R2-3 = F2-1 (MEDIUM)** : l'AC16 refuse aussi `KESH_ADMIN_PASSWORD` contenant `GENERATE_ME` —
    constante commune `TEMPLATE_PLACEHOLDERS`, variante existante `InsecureAdminPassword`, deux tests
    neufs calqués sur ceux du secret (dont la lecture de la ligne `#KESH_ADMIN_PASSWORD=` du gabarit),
    **M24** ; AC12 e (installation Synology : le mot de passe admin n'est **pas** obligatoire), AC13
    (**Sécurité**, dont « changer le mot de passe d'un administrateur déjà créé avec le placeholder »),
    § *Le défaut* ; l'ancienne AC16 e (« hors périmètre ») et son angle mort sont retirés. **F2-8** :
    contrôle du placeholder **avant** la longueur pour les deux variables — **M25** ; doc-comments
    devenus faux (`config.rs:663-667`, `:1800-1803`) nommés.
  - **R2-1 + R2-2 = F2-2 + F2-9 (MEDIUM)** : recette de déplacement **réécrite et rejouée** — boucle sur
    les trois dossiers, dernière affectation non commentée (`tail -n 1`), `\r`, guillemets et
    commentaire de fin de ligne retirés, refus d'un chemin non absolu ou d'un dossier inexistant (plus de
    `mkdir -p`), `rsync -rt --no-perms --no-owner --no-group` (plus de `-a`), `diff -rq`, `stat`
    avant/après, conteneur arrêté, sous-shell (un « ARRET » ne ferme pas la session), `sudo` réservé à
    `docker compose` et, sur « Permission denied », à la copie ; lignes de 76 caractères au plus
    (`breaklines`) ; rejouée sur dix `.env` piégés sous `dash`, `bash` et BusyBox, **non mesurée sur
    DSM** ; le CHANGELOG renvoie à la recette au lieu d'en recopier une ligne.
  - **R2-4 + F2-5 + F2-6 (MEDIUM, LOW)** : propagation complète du placeholder — `.env.example:6-8`,
    `:85-86`, `docker-compose.prod.yml:27-29`, `:74`, `admin-manual.tex:537-539`, `:546-548`, `:662`,
    `:690`, `crates/kesh-api/README.md:38, 55-57, 62-65` ; affirmation fausse sur `main.rs` retirée
    (aucun appelant ne distingue les variantes) ; grep du T8 étendu à `README.md`, `crates/*/README.md`
    et aux motifs `secrets obligatoires`, `docker-compose `, `rsync -a`, `refiger`.
  - **F2-3 (MEDIUM)** : **AC12 j** — la 15-11a reprend de la 15-7b2 (T7, C-15-7-52) la mise en page des
    dix tableaux de `sec:env-vars` (titre `\paragraph` hors ligne, dans la section seule,
    `kesh-style.sty` non touché ; contrôle au `.log` — zéro `Overfull \hbox` dans la section — et au PDF
    aplati) ; **la 15-7b2 n'a plus à le faire** (ajustement de sa fiche par l'orchestrateur) ; § *Coordination*.
  - **LOW** : R2-5 (nom du fichier compose dit avant la recette, `-f` sinon ; « non mesuré sur DSM ») ;
    R2-6 + F2-10 (`DOCKER_START.md` passe entièrement à `docker compose`, v1 non mesuré) ; R2-7 (« se
    merge seule sans danger » borné) ; R2-8 (`admin-manual.tex:1238` lu, corrigé par ricochet) ; R2-9
    (extraction des lignes du gabarit : colonne 0, exactement une ligne, valeur contenant `GENERATE_ME`) ;
    R2-10 (renvoi à la fiche 15-7b2 par le texte ; mention R1-6 du Change Log P1 annotée) ; R2-11
    (exploitant de Y qui construit depuis un clone, au CHANGELOG) ; F2-4 (`README.md:79-81`, effet sur
    la pile de dev au Dev Agent Record) ; F2-7 (nuance « ne refigez pas » sur le message
    `error-invoice-pdf-gone` au manuel et au CHANGELOG, **clé i18n non modifiée**) ; F2-11 (brochure
    `marketing-brochure.tex:542` corrigée, PDF régénéré).
  - **Signal D5** : sévérité maximale **HIGH en P1 → MEDIUM en P2** — le critère « égale ou supérieure »
    n'est pas atteint. Décompte des **5 MEDIUM** distincts : **3 nés de la remédiation P1** (R2-1 et
    R2-2 = F2-2 : la recette de déplacement écrite en P1 ; R2-4 : la propagation incomplète de l'AC16
    ajoutée en P1) et **2 d'origine** (R2-3 = F2-1 : défaut du code antérieur à la story, laissé hors
    périmètre en P1 ; F2-3 : défaut de mise en page du manuel antérieur à la story). Le motif « la
    remédiation introduit le défaut suivant » est donc présent (3 sur 5), sur une seule zone — la
    procédure de mise à jour du manuel ; pas de dispersion (modules de code : 3). **Déclaré à
    l'orchestrateur** ; pas de découpage. La P3 doit viser d'abord ce que cette remédiation écrit.
  - Comptes **recomptés depuis cette fiche** : AC **16** (AC12 gagne (j), AC16 réécrite), tâches **10**
    (T0-T9), mutations **25** (M1-M25, toutes rouges ; M3 trois rouges, M7 et M14 deux), tests unitaires
    de l'AC16 **5** (trois pour le secret JWT dont le témoin, deux pour le mot de passe admin), modules
    de code **3** (`config`, `main`, `lib`) — seuil de découpage non franchi.
- 2026-10-08 — **Remédiation de la validation P3 de la 15-11a** (agent de remédiation, autonomie ;
  décision de l'orchestrateur sur signal D5 et choix **C83**, qui révise C73, C77, C79 et C81). Passe P3 :
  **deux lentilles Opus 5.5** en contexte frais — R (regression hunter) : 0 CRITICAL, 0 HIGH, 3 MEDIUM,
  10 LOW ; F (full-scope adversary) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 8 LOW. Recoupements : R3-1 = F3-1,
  R3-3 = F3-5 (MEDIUM chez R, LOW chez F : compté MEDIUM), R3-9 = F3-6, R3-7 = F3-8 (F3-8 l'étend au
  contrôle `change-me`), R3-13 = F3-11 — soit **20 findings distincts** (6 MEDIUM, 14 LOW), tous traités :
  - **Décision de l'orchestrateur — AC4 abandonnée** : `docker-compose.prod.yml` **garde ses montages
    fixes** `./documents`, `./inbox`, `./log` ; la recette de déplacement, son encadré, sa ligne dans la
    liste « relisez » (source 3 réduite à `KESH_LOG_FILE_PATH` et `KESH_ADMIN_PASSWORD`), le geste 3 de la
    mise à jour (« deux gestes » désormais), l'AC10 (iv) et le rouge (E) du T1 sont retirés ou réécrits ;
    le test garde l'état (T structure : P fixe, Y interpolé ; (E) cherche `${NOM` dans les `volumes:` de
    `docker-compose.yml` seul) ; M14 réécrite sur Y, **M28** neuve sur P ; les `KESH_*_HOST_DIR` restent
    au gabarit et au manuel (elles servent `docker-compose.yml`), marquées **sans effet avec
    `docker-compose.prod.yml`** (AC6 i, AC12 c, T2, AC13) ; version configurable : **issue #558**.
  - **R3-1 = F3-1 (MEDIUM)**, **F3-2 (MEDIUM)**, **R3-2 (MEDIUM)** — lecture de `.env` qui n'imite pas
    Compose, `diff -rq` sur destination non vide, largeur des lignes dans l'encadré : **disparaissent**
    avec la recette. Leurs leçons servent ailleurs : la vérification des placeholders est **demandée à
    Compose** (`sudo docker compose config | grep -iE …`, deux lignes ≤ 76 caractères), rejouée au T6 sur
    `.env` piégés ; le contrôle PDF de ces deux lignes se fait au `pdftotext -layout` non aplati.
  - **F3-3 (MEDIUM)** — PDF figés sortis de la sauvegarde : **disparaît**, aucun dossier ne change de
    place ; reste une précision pour `docker-compose.yml` (`admin-manual.tex:1384`, `:1518`, AC12 c).
  - **F3-4 (MEDIUM)** — décision de l'orchestrateur : l'AC16 refuse, pour `KESH_JWT_SECRET` et
    `KESH_ADMIN_PASSWORD`, toute valeur de la forme `<…>` après trim (fonction commune
    `is_template_placeholder`), avant la longueur ; test neuf
    `config_rejects_jwt_secret_and_admin_password_in_angle_brackets` (gabarits du manuel, `<x>` court,
    espaces, témoins à chevron intérieur), **M26** ; manuel : « remplacez la valeur entre chevrons »
    (`:240`, `:244`, `:988`, `:1230`), `:243-244` en commentaire (optionnels) ; messages, doc-comments,
    tableaux `:662`, `:690`, CHANGELOG **Sécurité**.
  - **R3-3 = F3-5** : contrôle `Overfull` de l'AC12 j **borné aux dix tableaux**
    (`\label{sec:env-vars}` … `\subsubsection{Import de factures depuis un dossier}`) ; le
    `\paragraph{Note Synology…}` (`821--825`), que l'abandon de l'AC4 ne touche pas, reste hors périmètre,
    écrit comme préexistant.
  - **LOW** : R3-4 (les dix tableaux sont rognés, constat corrigé) ; R3-5 (`.log` local, non versionné) ;
    R3-6 (`config.rs:1811` nommé) ; R3-7 = F3-8 (**M27**, ordre du contrôle admin ; cas `xchange-mex`
    pour la remontée de `change-me`) ; R3-11 (secrets obligatoires de l'installation Synology :
    `KESH_JWT_SECRET`, `DATABASE_URL`) ; R3-12 (doublon `README.md` retiré du grep du T8 ; « les trois
    manuels emploient `\paragraph` » corrigé) ; R3-13 = F3-11 (Compose comme oracle, ci-dessus) ; F3-9
    (commentaires de montage de P vérifiés vrais, une ligne ajoutée) ; R3-8, R3-9 = F3-6, R3-10, F3-7 :
    **disparaissent** avec la recette ; **F3-10** (`website/index.html:189`) et **F3-12** (cellule `:1314`
    de la 15-7b2 encore en « redémarrer ») : **signalés à l'orchestrateur**, hors de cette fiche.
  - Trouvé à la remédiation, signalé : `CLAUDE.md:180` (recette E2E) écrit `KESH_ADMIN_PASSWORD='<12+
    caractères>'`, que l'AC16 refusera s'il est recopié tel quel — `CLAUDE.md` non modifié.
  - **Signal D5 — trend et sortie** : sévérité maximale **P1 1 HIGH → P2 5 MEDIUM → P3 7 MEDIUM bruts
    (6 distincts)**, dont **2 recyclés** au sens de l'amendement D5 (R3-1 = F3-1 et F3-2 : la même
    machinerie que R2-1/R2-2, un analyseur de `.env` qui doit imiter Compose, en défaut pour la troisième
    passe de suite) ; R3-2 et R3-3 naissent aussi de la remédiation P2 ; F3-3 et F3-4 sont d'origine.
    **Sortie du recyclage par retrait de la machinerie, non par découpage** (décision de
    l'orchestrateur) : l'AC4 qui exigeait la recette est abandonnée, sa version configurable part à #558.
    Une seule zone touchée par le recyclage ; modules de code inchangés.
  - Comptes **recomptés depuis cette fiche** : AC **16** (AC4 conservée, réécrite en « montages
    inchangés »), tâches **10** (T0-T9), mutations **28** (M1-M28, toutes rouges ; M3 trois rouges, M7 et
    M14 deux), tests unitaires de l'AC16 **6** (trois pour le secret JWT dont le témoin, deux pour le mot
    de passe admin, un commun aux deux pour la forme `<…>`), lignes ajoutées aux compose **28** (15 dans
    Y, 13 dans P — inchangé, l'AC4 ne portait que sur `volumes:`), modules de code **3** (`config`,
    `main`, `lib`) — seuil de découpage non franchi.
- 2026-10-08 — **Validation P4 (Sonnet ×2) : 0 au-dessus de LOW — VALIDATION CLOSE** (agent de clôture,
  autonomie ; choix **C84**). Passe P4 : **deux lentilles Sonnet 5.5** en contexte frais (prompt versionné
  `15-11a-validate-prompt-p4.md`) — R (regression hunter) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, **14 LOW** ;
  F (full-scope adversary) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, **9 LOW**. Recoupements : R4-1 ≈ F4-2,
  R4-9 = F4-3, R4-12 = F4-8 — soit **20 LOW distincts** (23 bruts), tous appliqués :
  - **R4-13** : la consigne « un `$` s'écrit `$$` » était fausse pour `cargo run` — **mesuré** au
    scratchpad : `dotenvy` 0.15.7 (`parse.rs:165-220`) lit `pa$$word` et `pa$word` comme `pa`, et refuse
    `"pa$word"` ; Compose 2.40.3 lit `'pa$word'` littéralement. Consigne qui vaut partout : **apostrophes
    simples** (`KESH_SMTP_PASSWORD='pa$word'`), guillemets doubles et `\$` si la valeur contient une
    apostrophe (mesuré dans les deux lecteurs) ; `$$` réservé à Compose ; `docker compose config`
    **réaffiche** tout `$` en `$$` (§ *Le défaut*, AC6 h, AC12 a, AC12 g, AC13, T0, T3).
  - **R4-14** : motif de vérification de l'AC12 f élargi (`.? *` au lieu de `.?`) — l'ancien était muet sur
    `KESH_ADMIN_PASSWORD="   <x>  "` ; nouveau motif rejoué sur dix-sept `.env` (quatorze piégés montrés,
    trois témoins muets) ; sur-ensemble redit ; cas ajouté au T6. Seconde ligne : 70 caractères.
  - **R4-1 = F4-2** : fiche 15-7b3 relue — elle écrit dans `admin-manual.tex:1704-1717` (même
    sous-section que l'AC12 f) et `:1734-1749` (voisin) ; ordre 15-11a d'abord, relocalisation par le
    texte, PDF régénéré par la seconde (§ *Coordination*) ; ferme l'action (f) de C83.
  - **R4-2** : fiche index `15-11-configuration-transmise.md` mise à jour (16 AC, 28 mutations, `closes #550
    #557`, `refs #558`, AC4 abandonnée). **R4-3** : `refs #558` à l'en-tête « Issues ». **R4-4** : la
    fiche dit déjà « sans effet avec le compose de prod » ; le texte de **#558** (« les retire ») est à
    corriger **par l'orchestrateur**, ainsi que « 70 caractères » → 76 dans son cahier des charges.
  - **LOW** : R4-5 (M23 : texte de mutation `REMPLACER_MOI: …`, ni `<…>` ni `GENERATE_ME`, rationnel
    réécrit) ; R4-6 (M19 rouge deux fois, `touch config.rs`) ; R4-7 (39 caractères) ; R4-8 (`:534`) ;
    R4-9 = F4-3 (AC14 à la forme du T8, `:988` ajouté, sept lignes rejouées) ; R4-10 (« aucune ligne de
    montage ») ; R4-11 (listes `HOTE` et `MARIADB` nommées à l'AC8, entrée inutilisée rouge) ;
    R4-12 = F4-8 (extraction de la source d'un montage : `:<cible>` final, `:ro`/`:rw`, forme longue rouge,
    cas (S)) ; F4-1 (C73, C77, C79, C81 annotés « révisés par C83 » par la **nouvelle** entrée C84, sans
    réécrire les anciennes) ; F4-4 (ligatures `ﬀ`/`ﬁ`/`ﬂ`/`ﬃ`/`ﬄ` normalisées dans tout contrôle sur PDF
    aplati) ; F4-5 (mots de passe MariaDB au gabarit : phrase du manuel, angle mort, propriétaire #551) ;
    F4-6 (« chemins des montages » à l'AC13 et à l'AC12 f, conséquence d'un oubli) ; F4-7 (M23 dans la
    liste des mutations sans `.rs` ; M20 « au moins ») ; F4-9 (`grep … .env` de l'AC12 e : relecture à
    l'œil, non vérification).
  - **Trend complet de la validation de la 15-11a** (findings distincts) : **P1** (Opus ×2) 1 HIGH, 2
    MEDIUM, 11 LOW → **P2** (Sonnet ×2) 5 MEDIUM, 12 LOW → **P3** (Opus ×2) 6 MEDIUM, 14 LOW → **P4**
    (Sonnet ×2) **0 au-dessus de LOW**, 20 LOW. Modèles : rotation Opus ↔ Sonnet pour les passes complètes
    (D6 : aucune passe complète confiée à Haiku).
  - **Signaux D5** : P1 HIGH d'origine, non recyclé — pas de découpage (C79) ; P2 HIGH → MEDIUM, 3 des 5
    MEDIUM nés de la remédiation P1, déclaré (C81) ; P3 MEDIUM égal à la P2 **par recyclage** de la
    recette de déplacement — sortie par retrait (AC4 abandonnée, #558, C83) ; **P4 : aucun signal**
    (sévérité tombée à LOW ; les LOW sont des résidus de propagation et des précisions, aucun ne touche
    la conception).
  - Comptes **recomptés depuis cette fiche** : AC **16**, tâches **10** (T0-T9), mutations **28** (M1-M28,
    toutes rouges ; M3 trois rouges, M7, M14 et M19 deux), tests unitaires de l'AC16 **6**, lignes
    ajoutées aux compose **28** (15 dans Y, 13 dans P), modules de code **3** — seuil de découpage non
    franchi.
- 2026-10-08 — **Développement (dev-story, autonomie ; choix C-15-11a-1 à 5).** T0-T9 faits : 28 lignes de
  compose, AC16 (`is_template_placeholder`, deux contrôles avant la longueur, 6 tests), test
  `configuration_transmise` (12 tests : 5 sur le dépôt, 7 d'auto-test), 28 mutations toutes rouges, AC10/AC5
  mesurés sans conteneur, manuel (dix tableaux remis en page, sous-section « Passer à la 0.13.0 »), brochure,
  `DOCKER_START.md`, READMEs, CHANGELOG (Corrigé + Sécurité), étape CI. Gates : backend 2827/2827 (4 ignorés),
  frontend vert, E2E 244 passés / 7 échecs attendus (KF-029). Tests unitaires de `config.rs` : 72 → 78
  (`ef39dd54` → `HEAD`). Statut → `review`.

