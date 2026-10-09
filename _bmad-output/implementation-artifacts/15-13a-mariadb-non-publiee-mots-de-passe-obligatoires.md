# Story 15.13a : MariaDB n'est plus publiée, et ses mots de passe sont obligatoires

Status: review

<!-- Née le 2026-10-09 du découpage de la 15-13 (`15-13-mariadb-et-sauvegarde.md`, désormais fiche index)
     après la validation P3, décision de l'orchestrateur (signal D5 de recyclage levé deux passes de suite) —
     choix C-15-13-16. Le contenu vient de la fiche unique au commit 8a9bcd27, réparti sans perte (recompte
     aux deux bornes : fiche index, § « Découpage ») ; la remédiation de la validation P3 y est appliquée.
     **La numérotation de la fiche unique est conservée** (AC, tests, mutations, tâches) pour la
     traçabilité : les numéros absents vivent dans la 15-13b. Choix : C-15-13-1, -2, -5, -10, -11 (remplacé
     par -13), -12, -13, -15, -16, -17, -18, -20, -22 et -23 du registre `epic-15-choix-autonomes.md`,
     chacun cité dans le corps (R4-6 de la validation P4). Statut `ready-for-dev` : convention du registre
     pour une fiche en cours de validation. -->

**Issues** : **ferme #551**. La PR porte `closes #551` (mot-clé dans le titre ou le corps : le dépôt merge en
squash) ; les commits intermédiaires portent `refs #551`. **`refs #577`** : la story corrige le décompte
« 4 jobs » de `docs/ci.md` et y renvoie à #577 (job `e2e` décrit mais absent de `ci.yml`, smoke E2E annoncé
par le `CLAUDE.md`), **sans** la fermer. **#575** (section Synology du manuel) reste hors story : citée, pas
fermée. **#554** (mot de passe admin de `docker-compose.dev.yml`) : citée. L'angle mort « mot de passe root
resté au gabarit » est tracé par l'issue que l'orchestrateur ouvre (§ *Angles morts* ; numéro à reporter
avant le développement).

**Dépendances** : après la **15-11a** et la **15-11b** (mergées dans `de285ea8`). **Indépendante de la 15-13b
en code** : aucune des deux n'appelle ce que l'autre ajoute. Elles touchent en revanche les **mêmes
fichiers**, à des endroits distincts : `docker-compose.yml`, `.env.example`, `crates/kesh-api/src/config.rs`,
`crates/kesh-api/tests/configuration_transmise.rs`, `docs/manual/fr/admin-manual.tex` (§ *Passer à la
0.13.0*), `DOCKER_START.md`, `CHANGELOG.md` `[0.13.0]` — et les **PDF versionnés** `admin-manual.pdf` et
`marketing-brochure.pdf`, que les deux régénèrent. La seconde mergée rebase et **recompte les gestes**
du paragraphe « Pour qui garde son fichier compose » (AC 11 f ; sa fiche dit les nombres) ; un conflit sur
un PDF ne se rebase pas : on fusionne les `.tex`, puis on **régénère** les PDF (`make fr`) et on rejoue les
contrôles aplatis des **deux** fiches (R4-3 de la validation P4). **Ordre suggéré :
15-13a d'abord** — #551 est P2 (sécurité), #552 et #576 P3 ; et le helper `lancer_binaire` (T3) ne concerne
qu'elle. **Coordination** avec la **15-7b3** (non mergée), qui corrige les `docker compose logs kesh` du
manuel (§ *Dépannage*) : cette story ne touche pas ces lignes (AC 11 j). Release visée : **v0.13.0** — la
section du manuel « Passer à la 0.13.0 » et l'entrée `[0.13.0]` du CHANGELOG sont **complétées**, pas
doublées.

**Si la 15-13a est mergée seule** (la 15-13b non encore mergée au tag) : le CHANGELOG `[0.13.0]` porte
l'entrée **Sécurité** #551 (AC 13 a), la rubrique **Retiré** `init-demo.sh` (AC 13 b), et `CHANGELOG.md:46`
a ses quatre phrases MariaDB réécrites (AC 13 c) ; la phrase « la sauvegarde pré-import reste dans `/tmp` »
**reste vraie** et n'est pas touchée. Le manuel porte les parties MariaDB ; ses mentions « défaut `/tmp` »
(`:757`, `:1664`, `:1699`) restent vraies. Le paragraphe « Pour qui garde son fichier compose » compte
**quatre** gestes pour `docker-compose.yml` et **deux** pour `docker-compose.prod.yml` (AC 11 f).

## Story

En tant qu'**exploitant d'une installation Kesh** (serveur Docker avec `docker-compose.yml`, ou NAS
Synology),
je veux que **la base comptable ne soit pas joignable depuis le réseau, et qu'elle ne soit pas protégée par
des mots de passe publiés dans le dépôt**,
afin de **ne pas exposer mes livres à toute machine du réseau local**.

## Le défaut, établi au sol le 2026-10-09

Faits revérifiés au code (`grep -nF`, worktree `kesh-15-13`, `HEAD` = `de285ea8`) :

### #551 — MariaDB publiée sur toutes les interfaces, avec des mots de passe publiés

| Site | Contenu | Effet |
|---|---|---|
| `docker-compose.yml:13` | `- "3306:3306"` sous `services.mariadb.ports` | MariaDB écoute sur **toutes** les interfaces de l'hôte |
| `docker-compose.yml:8` | `MARIADB_ROOT_PASSWORD: ${MARIADB_ROOT_PASSWORD:-kesh_dev_root}` | mot de passe root par défaut, publié |
| `docker-compose.yml:11` | `MARIADB_PASSWORD: ${MARIADB_PASSWORD:-kesh_dev}` | mot de passe applicatif par défaut, publié |
| `docker-compose.yml:53` | `DATABASE_URL: mysql://${MARIADB_USER:-kesh}:${MARIADB_PASSWORD:-kesh_dev}@mariadb:3306/…` | Kesh se connecte avec ce même défaut |
| `.env.example:336`, `:339` | `MARIADB_ROOT_PASSWORD=kesh_dev_root`, `MARIADB_PASSWORD=kesh_dev` — **lignes actives** | même un `.env` copié du gabarit, comme le prescrit le manuel (`admin-manual.tex:228-231`), pose les défauts publiés |
| `.env.example:328-334` | bloc intitulé « Service MariaDB bundled (**DEV ONLY** — docker-compose.yml + docker-compose.dev.yml) » | faux deux fois : `docker-compose.yml` est la **méthode d'installation recommandée** du manuel (`admin-manual.tex:204-224`), et `docker-compose.dev.yml` ne lit pas ces variables (mots de passe écrits en dur, `docker-compose.dev.yml:56-59`) |

Le service `kesh-api` atteint MariaDB par le réseau interne de Compose (`kesh-network`, hôte `mariadb`) :
**la publication du port n'a aucun rôle dans le fonctionnement**. Aucune commande du manuel n'utilise le
port publié : toutes passent par `docker compose exec` (`admin-manual.tex:1460`, `:1511`, `:1578`,
`:2216` ; `DOCKER_START.md:102`).

⚠️ **Un port publié par Docker contourne le pare-feu de l'hôte.** Docker insère ses propres règles
`iptables` (chaîne `DOCKER`) avant celles d'UFW : le pare-feu que recommande le manuel
(`admin-manual.tex:1998-2009`, `ufw default deny incoming`) **ne protège pas** un port publié en
`"3306:3306"` (documentation Docker, *Packet filtering and firewalls*, § « Docker and ufw »). L'exploitant
qui a suivi le manuel à la lettre croit sa base fermée ; elle ne l'est pas.

`docker-compose.prod.yml` (installation Synology du manuel) **n'a pas de service MariaDB** — la base est
fournie par l'exploitant (`DATABASE_URL` de `.env`) : #551 ne le concerne pas, hormis la consigne de
génération du mot de passe (ci-dessous).

**Défaut voisin, même sujet, trouvé en spécifiant** : le manuel fait générer le mot de passe MariaDB de
l'installation Synology par `openssl rand -base64 32` (`admin-manual.tex:546`), « repris dans
`DATABASE_URL` ». Une sortie base64 contient `/` avec une probabilité d'environ 1 − (63/64)^43 ≈ 49 % ;
dans une URL, `/` termine l'autorité — `mysql://kesh:ab/cd@hote:3306/kesh` ne se lit plus comme
l'identifiant `ab/cd`. Une installation sur deux suivant le manuel obtient une `DATABASE_URL`
inexploitable. **Traité ici** (AC 11 c, C-15-13-5) : `openssl rand -hex 32`.

**Défaut voisin, trouvé en validation P3** (F-P3-3) : le manuel fait recopier à l'étape 3
`MARIADB_PASSWORD=<mot de passe utilisateur applicatif>` (`admin-manual.tex:254`) et, pour Synology,
`DATABASE_URL=mysql://kesh:<MARIADB_PASSWORD>@mariadb:3306/kesh` (`:259`). Recopiées telles quelles, ces
chaînes deviennent le mot de passe applicatif — **imprimé dans le PDF distribué**, donc publié au même titre
que `kesh_dev`. Kesh le voit (il lit `DATABASE_URL`) et dispose déjà du prédicat qui le reconnaît
(`is_template_placeholder`, `config.rs:1405`, écrit par la 15-11a pour les gabarits du manuel). **Traité ici**
(AC 5 a-bis, C-15-13-17).

## Acceptance Criteria

*(Numérotation de la fiche unique 15-13 conservée ; les AC 7, 8, 9, 15 et 16 sont dans la 15-13b, et les
sous-points des AC 10 à 14 qui portent sur la sauvegarde aussi.)*

### Volet A — MariaDB (#551)

1. **Le port de MariaDB n'est plus publié par `docker-compose.yml`.**
   (a) Le service `mariadb` de `docker-compose.yml` ne porte **aucune** clé `ports:` — ni `"3306:3306"`, ni
   `"127.0.0.1:3306:3306"` (C-15-13-1).
   (b) Un commentaire YAML à l'endroit de l'ancienne clé dit : pourquoi le port n'est pas publié (Kesh
   passe par le réseau interne ; un port publié contourne UFW) ; comment accéder à la base
   (`docker compose exec mariadb mariadb -u kesh -p`) ; et, **pour un dépannage ponctuel seulement**, la
   forme à décommenter, liée au **loopback** (`- "127.0.0.1:3306:3306"`), jamais la forme sans adresse.
   (c) `kesh-api` continue d'atteindre MariaDB par `mariadb:3306` sur `kesh-network` (inchangé).

2. **Les deux mots de passe MariaDB n'ont plus de valeur par défaut dans `docker-compose.yml` : leur
   absence fait refuser le démarrage par Compose, avec un message qui nomme la variable.**
   (a) `services.mariadb.environment` : `MARIADB_ROOT_PASSWORD: ${MARIADB_ROOT_PASSWORD:?<message>}` et
   `MARIADB_PASSWORD: ${MARIADB_PASSWORD:?<message>}`.
   (b) La `DATABASE_URL` composée de `kesh-api` (`docker-compose.yml:53`) interpole
   `${MARIADB_PASSWORD:?<message>}` — **même message** qu'en (a) ; `MARIADB_USER` et `MARIADB_DATABASE`
   gardent `:-kesh` (ce ne sont pas des secrets).
   (c) (C-15-13-12.) Le `<message>` ne contient ni `}` ni `$` (la syntaxe de Compose s'arrête au premier `}`), et il
   doit laisser le **YAML** lisible : les valeurs de `docker-compose.yml:8`, `:11` et `:53` sont des
   scalaires **non cités**, où `": "` termine la clé (« mapping values are not allowed here ») et `" #"`
   ouvre un commentaire. Deux formes admises, **tranchées au T0** et écrites au Dev Agent Record :
   (c1) message **sans** `": "` ni `" #"`, valeur non citée ; (c2) valeur entière entre **guillemets
   doubles** (`"${MARIADB_PASSWORD:?…}"`), message sans `"` ni `\` — les guillemets simples sont exclus
   par l'apostrophe de « d'administration ». Le message dit d'abord le cas **existant**, puis le cas neuf
   (l'ordre inverse inviterait à écrire une valeur neuve sur une base existante) : « à poser dans .env.
   Installation existante, reprendre la valeur avec laquelle la base a été créée, jamais une neuve
   (manuel d'administration, Passer à la 0.13.0) ». Ses accents sont conservés si le T0 montre que
   Compose les restitue tels quels ; sinon, message sans accents, et la mesure est écrite au Dev Agent
   Record. Compose ne nomme qu'**une** variable manquante à la fois (la première qu'il rencontre) : le
   manuel le dit (AC 11 j).
   (d) **Aucun jeton `kesh_dev`** (donc ni `kesh_dev` ni `kesh_dev_root`) dans `docker-compose.yml`,
   `docker-compose.prod.yml` ni `.env.example` — commentaires compris.

3. **`.env.example` cesse de poser les mots de passe publiés.**
   (a) Le bloc `Service MariaDB bundled (DEV ONLY …)` (`.env.example:328-339`) est réécrit : il sert
   `docker-compose.yml` — l'installation autonome, MariaDB comprise —, pas « le développement » ;
   `docker-compose.prod.yml` et `docker-compose.dev.yml` ne le lisent pas (dire pourquoi en une ligne
   chacun).
   (b) `MARIADB_ROOT_PASSWORD` et `MARIADB_PASSWORD` y sont des lignes **commentées** (`#MARIADB_…=`),
   **sans valeur** — un `.env` copié tel quel fait donc refuser le démarrage par Compose (AC 2), au lieu
   de créer une base aux identifiants publiés. `MARIADB_DATABASE=kesh` et `MARIADB_USER=kesh` restent
   des lignes actives.
   (c) Le commentaire du bloc dit, **dans cet ordre** : d'abord, **installation existante** — reprendre
   la valeur avec laquelle la base a été créée, jamais une neuve (renvoi au manuel, § *Passer à la
   0.13.0*) ; **ces deux valeurs ne sont lues par MariaDB qu'à la création de la base** — les changer
   dans `.env` ensuite ne change pas le mot de passe enregistré et fait échouer la connexion de Kesh
   (renvoi au manuel, procédure de changement) ; ensuite seulement, **installation neuve** — générer
   chaque mot de passe par `openssl rand -hex 32` (hexadécimal : un mot de passe base64 peut contenir
   `/`, qui casse `DATABASE_URL`). Décommenter une ligne **sans** y écrire de valeur fait refuser Compose
   (forme `:?`, AC 10 a) : le dire.
   (d) Le gabarit `DATABASE_URL=<EDIT: …>` (`.env.example:47`, pour `docker-compose.prod.yml`) n'est pas
   modifié, mais son commentaire dit de prendre un mot de passe hexadécimal (même raison).

4. **Le refus de Compose est exercé en CI, dans les deux sens.** L'étape « Validate compose files » du job
   `docker-build` (`.github/workflows/ci.yml:242-245`) :
   (a) valide `docker-compose.yml` **avec** des valeurs factices
   (`MARIADB_ROOT_PASSWORD=ci MARIADB_PASSWORD=ci docker compose -f docker-compose.yml config -q`) — sans
   elles, l'étape actuelle échouerait désormais ;
   (b) exige que la même commande **sans** ces variables (`env -u MARIADB_ROOT_PASSWORD -u MARIADB_PASSWORD`)
   **échoue**, et que sa sortie d'erreur nomme `MARIADB_ROOT_PASSWORD` ou `MARIADB_PASSWORD` ;
   (c) valide `docker-compose.prod.yml` comme aujourd'hui ;
   (d) (C-15-13-15, rectifié par C-15-13-20.) Son commentaire d'en-tête est mis à jour (il dit aujourd'hui « les variables obligatoires non
   posées ne font qu'avertir », faux pour `docker-compose.yml`), et `docs/ci.md` (`:12`, `:117`) dit les
   deux sens du contrôle ; son décompte « 4 jobs » (`:9-10`), faux, devient « 3 jobs » (T4).

5. **Kesh avertit fortement au démarrage quand le mot de passe de `DATABASE_URL` est un mot de passe
   publié.** (C-15-13-2 : avertir, pas refuser ; C-15-13-17 pour le gabarit.)
   (a) `Config::from_env` extrait le mot de passe de `DATABASE_URL` (URL analysée par la crate `url`,
   dont `Url::password()` rend la forme **encodée** ; **décodé** par `percent_encoding::percent_decode_str`
   puis **`decode_utf8_lossy()`** — pas par `form_urlencoded`, qui décode `+` en espace ; pas par
   `decode_utf8()`, dont l'`Err` sur des octets non UTF-8 (`%FF`) inviterait un `?` ou un `unwrap` qui
   ferait échouer ou paniquer le démarrage, contraire au (b) : une valeur non UTF-8 n'est ni publiée ni
   gabarit, et sa forme décodée avec `U+FFFD` ne déclenche rien, C-15-13-22) et, s'il figure dans la liste fermée
   `MOTS_DE_PASSE_PUBLIES = ["kesh_dev", "kesh_dev_root"]` (comparaison exacte, sensible à la casse),
   émet **un** `tracing::warn!` qui dit : le mot de passe est publié dans le dépôt Kesh ; acceptable pour
   le développement seulement ; renvoi au manuel (§ *Passer à la 0.13.0*, changement du mot de passe).
   Le message **ne contient pas** le mot de passe ni l'URL (mutation M43).
   (a-bis) **Gabarit du manuel** (F-P3-3, C-15-13-17) : le même contrôle émet **un** `tracing::warn!` quand
   le mot de passe **décodé** satisfait `is_template_placeholder` (`config.rs:1405` — réemploi, pas de
   second prédicat : DRY). Message **distinct** de (a), pour que l'exploitant sache quoi faire : le mot
   de passe de `DATABASE_URL` est un **placeholder de gabarit** (par exemple une valeur entre chevrons `<…>`
   recopiée du manuel — le prédicat reconnaît aussi `generate_me`, `config.rs:1391`, R4-5) ; il
   protège la base comme un mot de passe publié ; le changer par la procédure du manuel (§ *Changer un
   mot de passe MariaDB*, AC 11 g — `ALTER USER` d'abord, `.env` ensuite). Même règle : ni le mot de
   passe, ni l'URL (M43). Un même mot de passe ne peut satisfaire (a) et (a-bis) à la fois (aucun élément de la
   liste n'est entre chevrons) : un seul avertissement au plus.
   (b) Kesh **démarre quand même** : aucun `ConfigError` neuf. Une URL illisible, sans mot de passe ou
   au mot de passe hors liste et non gabarit n'émet rien de ce chef. Une URL qui ne s'analyse pas (mot de
   passe contenant `/`, `?` ou `#` non encodés : l'autorité est tronquée) n'émet rien non plus — c'est le
   cas de l'ancien `openssl rand -base64 32` du manuel, que l'AC 11 c remplace.
   (c) La lecture de `DATABASE_URL` reste celle d'aujourd'hui (`env_nonempty`) : **aucune** lecture
   d'environnement neuve (le test (L) de `configuration_transmise.rs` reste vert sans nouvelle entrée
   d'`EMPLACEMENTS_AUTORISES`).
   (d) Le doc-comment d'`is_template_placeholder` (`config.rs:1403-1404`, « Sert aux contrôles de
   `KESH_JWT_SECRET` et de `KESH_ADMIN_PASSWORD` ») nomme le troisième appelant.

6. **Un refus d'authentification de MariaDB au démarrage est expliqué, pas seulement rapporté.**
   (a) Quand la connexion initiale (`main.rs:71-89`) échoue sur l'erreur MariaDB **1045** (*Access
   denied*), le message d'erreur journalisé ajoute un **indice** : « MariaDB refuse l'identifiant de
   DATABASE_URL. Avec docker-compose.yml : MARIADB_PASSWORD n'est lu qu'à la création de la base —
   changer sa valeur dans .env ne change pas le mot de passe enregistré (manuel d'administration, Passer à
   la 0.13.0). » L'erreur d'origine reste dans le message ; le processus sort toujours en code 1. Ni le
   message ni l'indice ne citent `database_url` ni son mot de passe (mutation M44, test 10).
   (b) L'indice vient de **deux** fonctions publiques de `kesh-api` (`config.rs`) — `main.rs` ne fait
   que les composer : l'**extraction** `numero_erreur_mariadb(&sqlx::Error) -> Option<u16>`
   (`as_database_error()?.try_downcast_ref::<MySqlDatabaseError>()?.number()`, forme déjà employée
   `routes/reconciliation.rs:1136` et `kesh-db/src/retry.rs:106`), et la **décision**
   `indice_connexion(Option<u16>) -> Option<&'static str>`, pure. La séparation tient à un fait :
   `MySqlDatabaseError` n'a pas de constructeur public (`sqlx-mysql-0.8.6/src/error.rs:11`,
   `pub(super) ErrPacket`), si bien qu'un test unitaire ne peut pas fabriquer l'erreur d'entrée ;
   l'extraction est couverte par le test 10 (binaire réel), la décision par le test 9. Toute autre
   erreur (hôte injoignable, base absente : 1049, droits sur la base : 1044) ne donne pas d'indice.
   (c) **Mesure d'abord (T0)** : que `MySqlPoolOptions::connect` rende bien une `sqlx::Error::Database`
   dont le numéro MariaDB vaut 1045 pour un mauvais mot de passe. Lu à la source en validation P1 :
   `sqlx-core-0.8.6` ne rejoue que `ConnectionRefused` et les erreurs `is_transient_in_connect_phase()`
   (`pool/inner.rs:346-383`), que `sqlx-mysql` ne redéfinit pas — le 1045 doit donc sortir
   immédiatement en `Error::Database`. Si la mesure le contredit, la reconnaissance se fait sur ce qu'elle
   montre ; un repli **textuel** exige `(using password` et non `Access denied`, qui attraperait aussi le
   1044 (*Access denied for user … to database*). La mesure est écrite au Dev Agent Record.

### Volet C — le garde-fou et la documentation (parties MariaDB)

10. **`configuration_transmise.rs` garde ces décisions.** *(Le sous-point (c), montage de la sauvegarde, et
    la partie « trois montages » de (e) sont dans la 15-13b.)*
    (a) **MariaDB** : un contrôle neuf lit le service `mariadb` de `docker-compose.yml` (extraction
    généralisée de `service_kesh_api` en `service(source, nom)`, DRY ; le `Service` gagne la présence de la
    clé `ports:`) et rougit si : la clé `ports:` est présente ; `MARIADB_ROOT_PASSWORD` ou
    `MARIADB_PASSWORD` n'est pas de la forme `${NOM:?…}` sur **elle-même**. **La variante
    `Interpolation::Obligatoire` est scindée** (`configuration_transmise.rs:541-564`, et le bras de `match`
    de `controle_valeur`, `:626`, qui ne compile plus après la scission — R4-2) : `ObligatoireNonVide`
    (`:?`, refuse une variable absente **ou vide**) et `ObligatoireSiAbsente` (`?`, ne refuse qu'une
    variable absente). Le contrôle exige la première : avec `?`, la ligne `#MARIADB_PASSWORD=` du gabarit
    décommentée **sans valeur** — le geste le plus naturel — ferait démarrer Compose avec un mot de passe
    vide. **La `DATABASE_URL` de `kesh-api`** n'est **pas** contrôlée par ce test neuf : elle l'est par la
    famille (V) existante (`controle_valeur`, `configuration_transmise.rs:572-663`, test `valeurs`), en
    **resserrant** le motif de `VALEURS_COMPOSEES` (`:169-174`) de `${MARIADB_` à `${MARIADB_PASSWORD:?` —
    pas par un second contrôle sur la même valeur (DRY). La valeur est lue **après** l'analyse YAML : une
    valeur citée (forme c2 de l'AC 2 c) arrive sans ses guillemets.
    (b) **Mots de passe publiés** : aucun des trois fichiers distribués (`docker-compose.yml`,
    `docker-compose.prod.yml`, `.env.example`) ne contient `kesh_dev` (recherche de sous-chaîne, qui couvre
    `kesh_dev_root`) ; dans `.env.example`, toute ligne d'affectation de `MARIADB_ROOT_PASSWORD` ou de
    `MARIADB_PASSWORD` est commentée (même extracteur `lignes_affectation` que (E)).
    (d) **Auto-test (S)** : des sources synthétiques exercent le contrôle (a) — refusées : clé `ports:`
    présente ; `${MARIADB_PASSWORD:-x}` ; `${MARIADB_PASSWORD}` ; `${MARIADB_PASSWORD?m}` (forme `?`) ;
    acceptées : `${MARIADB_ROOT_PASSWORD:?m}` et la même valeur **citée** (`"${MARIADB_ROOT_PASSWORD:?m}"`).
    (e) L'en-tête du fichier (`//!`) dit ce que le test établit désormais du service `mariadb`, et
    l'angle mort qui reste : `docker-compose.dev.yml` non contraint (inchangé).

11. **Le manuel d'administration dit vrai, et le PDF aussi** (parties MariaDB). `docs/manual/fr/admin-manual.tex` :
    (a) **Ports réseau** (`:162-177`) et **Référence des ports** (`:2354-2364`) : MariaDB 3306 — « non
    publié par les compose fournis (réseau interne Docker) » ; l'avertissement dit qu'un port publié par
    Docker **contourne UFW**.
    (b) **Étape 3 — `.env`** (`:229-270`) : mots de passe MariaDB par `openssl rand -hex 32` ; sans eux,
    Compose refuse de démarrer en nommant la variable ; ils ne sont lus qu'à la **création** de la base.
    La phrase « pour les mots de passe MariaDB, rien ne la refuse — choisissez-les vous-même » est
    corrigée : Compose refuse l'absence ; une valeur gabarit `<…>` est acceptée par Compose et par
    MariaDB — **Kesh avertit au démarrage** quand le mot de passe **applicatif** l'est (AC 5 a-bis),
    **rien** ne signale un mot de passe **root** gabarit (Kesh ne le voit pas : angle mort) — le dire. Et,
    au même endroit : une base **déjà créée** avec la chaîne gabarit littérale (l'étape 3 actuelle,
    `admin-manual.tex:253-254`, fait recopier `MARIADB_ROOT_PASSWORD=<mot de passe fort>`) garde ce mot de
    passe ; on ne le remplace **pas** dans `.env` — ce serait un 1045 —, on le change par la procédure de
    l'AC 11 g (`ALTER USER` d'abord, `.env` ensuite). La relecture des valeurs entre chevrons que
    prescrit la procédure de mise à jour (`:1746`, « vérifier les placeholders de `.env` ») nomme ces deux
    lignes — c'est le seul signal pour le mot de passe root.
    (c) **Synology, étape 2** (`:546`) : `openssl rand -hex 32 # → mot de passe MariaDB` au lieu de
    `-base64 32` (raison : `/` dans `DATABASE_URL`).
    (d) **`sec:env-vars`** — note MariaDB (`:689-694`) complétée (obligatoires avec `docker-compose.yml`,
    lues à la création seulement). **Les dix tableaux de `sec:env-vars` restent sans `Overfull \hbox`**
    (acquis de la 15-11a). *(La ligne `KESH_ADMIN_BACKUP_DIR` et le paragraphe des montages : 15-13b.)*
    (f) **`sec:maj-0-13`** (`:1757-1841`) — le paragraphe **MariaDB** (pour `docker-compose.yml`) ; le
    paragraphe **sauvegarde** est dans la 15-13b. Détail en Dev Notes, § *La mise à jour d'une
    installation existante* : re-télécharger le compose suffit pour le port ; `.env` sans
    `MARIADB_ROOT_PASSWORD` / `MARIADB_PASSWORD` → refus nommé de Compose, et il faut alors y écrire **la
    valeur avec laquelle la base a été créée** (pour qui n'en avait jamais posé : les anciennes valeurs
    par défaut, `kesh_dev_root` et `kesh_dev`, écrites **au manuel seulement**), **jamais une valeur
    neuve** ; puis changer ces mots de passe par la procédure donnée (AC 11 g) ; **contrôle de fin de mise
    à jour sur la base, pas sur `.env`** (C-15-13-10) : une connexion avec les **anciens** mots de passe
    publiés doit être **refusée** (1045), une connexion avec ceux de `.env` doit réussir — pour `root`
    et pour `kesh`, en TCP pour que l'authentification par socket ne fausse pas la mesure, par exemple
    `sudo docker compose exec mariadb mariadb --protocol=TCP -h 127.0.0.1 -uroot -pkesh_dev_root -e 'SELECT CURRENT_USER()'`
    (refus attendu) et
    `sudo docker compose exec mariadb sh -c 'mariadb --protocol=TCP -h 127.0.0.1 -uroot -p"$MARIADB_ROOT_PASSWORD" -e "SELECT CURRENT_USER()"'`
    (succès attendu) — `SELECT CURRENT_USER()` plutôt que `SELECT 1` : le résultat dit **quel compte**
    a répondu (`root@localhost` ou `root@%`, selon que le serveur résout `127.0.0.1` — l'image pose
    peut-être `skip-name-resolve`, non vérifié) ; le manuel dit ce que le contrôle couvre, et l'AC 11 g
    traite de toute façon **chaque** compte `root` listé. Ces commandes dépassent 76 caractères : au
    manuel, elles sont coupées par `\` en fin de ligne (AC 11 k) — y compris **à l'intérieur** des
    apostrophes de `sh -c '…'`, où la barre suivie d'un saut de ligne est transmise à `sh`, qui la lit
    comme une continuation ; forme exacte, comportement de `root@localhost`, et compte rendu par
    `CURRENT_USER()` pour `-h 127.0.0.1` **et** pour `-h mariadb` **mesurés au T7** ; une
    vérification `docker compose config | grep kesh_dev` ne lit que `.env` et resterait muette alors que
    la base garde `kesh_dev_root` (F3 de la validation P1). Puis **lancer une fois à la main** le script
    de sauvegarde (§ *Sauvegarde*) après toute mise à jour ou tout changement de mot de passe : c'est
    le seul moyen de voir qu'un mot de passe root changé dans `.env` sans `ALTER USER` le fait échouer.
    **Pour qui garde son compose — le paragraphe `:1793`** (F-P3-1, C-15-13-18) : son titre « Pour qui garde son
    fichier compose : **deux gestes**. » et sa phrase d'ouverture « Sous `environment:` du service
    `kesh-api` : » deviennent faux — les gestes de cette story ne sont pas sous `environment:` de
    `kesh-api`. Le titre perd son décompte global ; le paragraphe donne le décompte **par fichier**,
    recompté à l'écriture sur l'état de la branche : avec la 15-13a seule, **quatre** gestes pour
    `docker-compose.yml` (les deux de la 15-11a ; retirer `ports:` de `mariadb` ; remplacer les fragments
    de mot de passe) et **deux** pour `docker-compose.prod.yml` (inchangé) ; si la 15-13b est déjà mergée,
    **cinq** et **trois** (elle ajoute le montage `./backup` aux deux). Les gestes neufs sont écrits
    **dans** ce paragraphe, chacun avec le fichier qu'il concerne : retirer `ports:` de `mariadb` ;
    **un remplacement de fragment** — `:-kesh_dev_root` et `:-kesh_dev` (trois occurrences :
    `MARIADB_ROOT_PASSWORD`, `MARIADB_PASSWORD`, `DATABASE_URL`) deviennent `:?` suivi d'un message ; le
    manuel recommande de **recopier** ces lignes depuis le compose re-téléchargé et ne reproduit **pas** la
    ligne `DATABASE_URL` entière ni le message long de l'AC 2 c, qui dépasseraient les 76 caractères de
    l'AC 11 k ; il montre le fragment, par exemple `${MARIADB_PASSWORD:?à poser dans .env}`. Contrôle
    écrit après les gestes : `docker compose config` ne montre **aucune** entrée `ports` sous le service
    `mariadb` (graphie de la forme longue relevée au T0). Puis avertissement « Kesh avertit au démarrage »
    (AC 5) et « indice 1045 » (AC 6). **Le refus de Compose frappe toute sous-commande**, pas seulement
    `pull` et `up -d` : Compose interpole le fichier entier au chargement, si bien que
    `docker compose ps`, `logs`, `exec`, `stop` et `down` refusent aussi tant que `.env` n'est pas
    complété — l'exploitant ne peut alors ni diagnostiquer ni arrêter proprement, et le script de
    sauvegarde lancé par `cron` échoue dès que `.env` perd une des deux lignes ; le manuel le dit (mesuré
    au T0 par `docker compose ps` sur la copie sans `.env`). **Le contrôle des placeholders**
    (`:1780-1786`, `sudo docker compose config | grep -iE …`, « la commande doit rester muette ») devient
    trompeur : quand Compose refuse, `grep` ne reçoit rien et reste muet — muet comme un succès. La
    procédure pose donc les deux mots de passe **avant** ce contrôle, et le fait précéder de
    `sudo docker compose config -q && echo 'compose lisible'`, dont la seconde moitié **doit**
    s'afficher : sans elle, le silence du `grep` ne prouve rien (même correction au CHANGELOG, AC 13 d).
    **Propagation dans l'encadré `:1761-1776`** : la liste « Refus de démarrer »
    (`:1764`, qui renvoie à `docker compose logs kesh-api`) ne reçoit **pas** le refus de Compose tel
    quel — il s'affiche **au terminal** de la commande lancée, et aucun conteneur ne démarre :
    il est écrit à part, comme un refus distinct de ceux de Kesh ; le paragraphe « Un `.env` copié du
    gabarit sans modification » (`:1775`) devient faux — un `.env` copié du **nouveau** gabarit fait
    refuser Compose (mots de passe commentés), un `.env` copié d'un **ancien** gabarit fait émettre
    l'avertissement de l'AC 5 —, et son « sauf » est complété. Le manuel fait partie du corpus du
    contrôle (F) de `configuration_transmise.rs` (`:899-920`) : ne nommer **aucun** jeton `KESH_…`
    inexistant, pas même pour dire qu'il n'existe pas.
    (g) **Changer un mot de passe MariaDB** (sous-section neuve, dans `sec:maj-0-13` ou juste après,
    `\label{sec:mariadb-mdp}`) : lister les comptes (`SELECT user, host FROM mysql.user;`), `ALTER USER`
    pour `kesh` et pour **chaque** compte `root` listé, **puis** `.env`, **puis**
    `docker compose up -d` ; la recette est **rejouée** sur un conteneur jetable (T7) avant d'être écrite.
    C'est le renvoi de l'avertissement de l'AC 5 a-bis. **Hygiène et fenêtre** (F7 de la validation P4,
    C-15-13-23) : le nouveau mot de passe n'apparaît **ni dans l'historique du shell** (il n'est jamais
    tapé en clair : engendré dans une variable, `NEW=$(openssl rand -hex 32)`) **ni dans la liste des
    processus de l'hôte** (l'`ALTER USER` atteint `mariadb` par l'**entrée standard** — document en ligne
    `<<SQL` —, jamais en argument de commande) ; il est reporté dans `.env` depuis la même variable, à
    l'éditeur (une forme `sed -i` le mettrait un instant dans les arguments de `sed` : écartée). La recette
    **commence par `docker compose stop kesh-api`** : entre l'`ALTER USER kesh` et le `up -d` final, les
    connexions neuves de Kesh échoueraient en 1045 ; Kesh arrêté, la fenêtre est bornée à la durée de la
    recette, sans erreur servie. Forme de référence : Dev Notes, § *Forme de la recette de changement*.
    (h) **`exec db` → `exec mariadb`** (`:1460`, `:1511`, `:2216`, nom de service faux : le service
    s'appelle `mariadb`) ; et le mot de passe root est lu **dans le conteneur**
    (`sh -c '… -p"$MARIADB_ROOT_PASSWORD"'`, apostrophes simples) au lieu d'une variable de shell que rien
    ne pose (`-p"${MARIADB_ROOT_PASSWORD}"`). Le script quotidien (`:1446-1479`), lancé par `cron`, ne
    s'exécute pas depuis le répertoire du compose : il gagne un `cd` vers ce répertoire avant
    `docker compose exec`, faute de quoi il échoue — et il échoue dans `/var/log/kesh-backup.log`
    (`:1485`), que personne ne lit : le manuel prescrit de le lancer une fois à la main (AC 11 f). La
    forme exacte du `mariadb-dump` multiligne (commande entière entre apostrophes pour `sh -c`) et de la
    restauration est écrite aux Dev Notes (§ *Forme du script de sauvegarde*) ; un dump **puis** une
    restauration sont rejoués au T7.
    (j) **Dépannage — Kesh ne démarre pas** (`:2192-2199`) : deux causes ajoutées à la liste — le refus de
    Compose (`required variable MARIADB_PASSWORD is missing a value`, affiché **au terminal**, pas dans
    les journaux — et `docker compose logs` refuse lui-même tant que la variable manque, AC 11 f ;
    Compose ne nomme qu'**une** variable à la fois : corriger la première, la seconde
    apparaît au lancement suivant — l'ordre réel est relevé au T0) et l'erreur 1045 avec son indice. Les
    lignes `docker compose logs kesh` (`:2189`) **ne sont pas** touchées (15-7b3).
    (k) Toute ligne de commande neuve dans un encadré `keshwarning` tient en **70 caractères** au plus, en
    `keshnote`/`lstlisting` ordinaire en **76** (mesure de la 15-11a).
    (l) **PDF régénéré** (`make fr` dans `docs/manual/`), contrôlé **aplati**
    (`pdftotext admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`, ligatures normalisées comme à la 15-11a).
    Contrôles **discriminants** — chacun rouge sur le PDF d'avant (relevé au T0, `exec -T mariadb` y
    figurant déjà, `admin-manual.tex:1578`, il ne prouve rien) : **positifs** —
    `openssl rand -hex 32 # → mot de passe MariaDB` (étape Synology) ; la commande de l'AC 11 h
    (`MARIADB_ROOT_PASSWORD"'`, témoin de la forme `sh -c`) ; `--protocol=TCP` (contrôle de fin de mise
    à jour) ; `CURRENT_USER()` (même contrôle) ; `compose lisible` (contrôle des placeholders) ; tous
    vérifiés **absents** du PDF aplati à `468ce610` (`target/gate-logs/15-13-p1-R-admin-plat.txt`) en
    remédiation P1 — `CURRENT_USER()` et `compose lisible` vérifiés absents en remédiation P2
    (`target/gate-logs/15-13-p2-F-admin-plat.txt`, même PDF, dernier commit `de285ea8`) ; **négatifs** —
    `grep -E 'exec (-T )?db\b'` vide (attrape aussi `exec db`, `:2216`) ; `deux gestes` absent (présent
    **une** fois au PDF d'avant : `grep -oF "deux gestes" target/gate-logs/15-13-p3-F-admin-manual-plat.txt
    | wc -l` → 1, relevé en validation P3 — discriminant). Au `.log`, aucun `Overfull \hbox` **nouveau**
    (comparé au `.log` d'avant, relevé au T0). ⚠️ **Rendu des motifs à apostrophe et à double tiret
    non établi** : aucun `lstlisting` du PDF d'avant ne montre d'apostrophe (le `'pa$word'` relevé vient
    d'un `\texttt`), et `--` sort droit en `lstlisting` mais en demi-cadratin en `\texttt`. Les motifs
    `MARIADB_ROOT_PASSWORD"'` et `--protocol=TCP` se **valident sur l'aplati du PDF neuf** avant d'être
    déclarés discriminants : s'ils n'y figurent pas sous cette graphie, le motif est ajusté sur la graphie
    réelle (apostrophe typographique, tiret) et l'ajustement est écrit au Dev Agent Record.
    (m) **Brochure** (`docs/manual/fr/marketing-brochure.tex:542`, et son PDF versionné) : « lancer une
    instance locale en moins de 5 minutes avec `docker compose up -d`, une fois `KESH_JWT_SECRET` généré
    dans `.env` » devient faux — Compose refuse désormais sans les deux mots de passe MariaDB. La phrase
    dit : « … une fois `KESH_JWT_SECRET` et les deux mots de passe MariaDB (`MARIADB_ROOT_PASSWORD`,
    `MARIADB_PASSWORD`) générés dans `.env` (voir le manuel d'administration) ». Le `make fr` du T5
    régénère `marketing-brochure.pdf` ; contrôle aplati : `MARIADB_ROOT_PASSWORD` présent (absent du PDF
    d'avant : `pdftotext marketing-brochure.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF MARIADB | wc -l`
    rend **0** en remédiation P2).

12. **`DOCKER_START.md` dit vrai, et `init-demo.sh` est retiré.** *(La ligne sur le dossier `./backup` :
    15-13b, AC 12 a'.)*
    (a) `DOCKER_START.md` : prérequis « Port 80 et 3306 disponibles » → port 80 seulement ; l'étape `.env`
    pose les deux mots de passe MariaDB (`openssl rand -hex 32`), et sa phrase d'ouverture
    (`DOCKER_START.md:12-13`, « Sans `.env`, Kesh refuse de démarrer : le secret JWT par défaut … est
    refusé à dessein ») est réécrite — sans `.env`, c'est **Compose** qui refuse d'abord, au terminal, en
    nommant `MARIADB_…` ; le refus du secret JWT par Kesh ne vient qu'ensuite (F-P3-8) ; « Accéder à la
    base de données » : `docker compose exec mariadb mariadb -u kesh -p -D kesh` (mot de passe demandé,
    plus de `-pkesh_dev`) ; « volume `mariadb_data` » (`:146`, nom faux) → le volume `kesh-mariadb-data`
    du compose, que `docker volume ls` affiche `<projet>_kesh-mariadb-data` (préfixe du projet Compose).
    (a-bis) **« Base de données ne s'initialise pas »** (`DOCKER_START.md:134-137`) prescrit aujourd'hui
    `docker compose down -v` puis `docker compose up --build`. Après la story, la cause la plus probable
    d'une base « qui ne démarre pas » sur une installation existante est un **1045** — mot de passe neuf
    écrit dans `.env` alors que MariaDB garde celui de la création (§ *La mise à jour d'une installation
    existante*) — et `down -v` le « résout » en **supprimant le volume**, c'est-à-dire les livres. Le
    paragraphe est réécrit : **diagnostic d'abord** (`docker compose logs kesh-api` et
    `docker compose logs mariadb` ; un 1045 ou l'indice de l'AC 6 = mot de passe → revenir à la valeur
    d'origine, renvoi au manuel § *Passer à la 0.13.0* ; un refus de Compose nommant une variable → la
    poser dans `.env`) ; `down -v` n'y figure plus que pour une installation **sans données à conserver**,
    avec l'avertissement qu'il supprime le volume de la base, **définitivement**. `DOCKER_START.md:78`
    (« Arrêter et supprimer les données », `down -v` explicite) reste tel quel : il dit ce qu'il fait.
    (b) **`init-demo.sh` est supprimé du dépôt** (C-15-13-13, qui remplace C-15-13-11). Il visait par
    défaut le conteneur `kesh-mariadb` de `docker-compose.yml` avec `kesh_dev` en dur (`:20`, `:23`,
    `:69`), et la correction de la P1 (identifiant lu dans le conteneur) le rendait **opérant** sur toute
    installation conforme : ses `DELETE` successifs (`:70-75`, commentaire `:70`), sans transaction ni
    confirmation, auraient vidé `users` et `onboarding_state` avant que `DELETE FROM companies` n'échoue
    sur les 25 clés étrangères `ON DELETE RESTRICT` d'une base peuplée (F2 de la validation P2). Il est
    **redondant** : la démonstration est semée par l'application elle-même
    (`POST /api/v1/onboarding/seed-demo`, `routes/onboarding.rs:177-190`, qui appelle
    `kesh_seed::seed_demo` — plan comptable complet par les repositories, exercice, `is_demo`,
    réinitialisation par `reset_demo`), alors que le script écrit en SQL brut dix comptes sans rôle ni
    exercice et un administrateur `admin`/`admin123`. **Personne ne l'appelle** : `git grep -n init-demo`
    hors `_bmad-output` ne rend que le fichier lui-même (aucun script, aucune CI, aucune documentation, ni
    `README.md`, ni `DOCKER_START.md`, ni manuel, ni site). Le retrait supprime le risque au lieu de le
    borner ; l'historique git le garde. **Vérifié après coup** par
    `git grep -n init-demo -- . ':(exclude)_bmad-output' ':(exclude)CHANGELOG.md'` → vide : le CHANGELOG
    est exclu parce que l'AC 13 b y **exige** la mention du retrait (R3-2) ; cette ligne entre à
    l'inventaire comme résolue, voulue.

13. **CHANGELOG `[0.13.0]`** (parties MariaDB).
    (a) **Sécurité** — entrée #551 : MariaDB n'est plus publiée ; plus de mots de passe par défaut ;
    `.env.example` ne les pose plus ; avertissement au démarrage (mot de passe publié **ou** gabarit du
    manuel recopié, AC 5) ; **action requise** (re-télécharger le compose ; sans les deux variables,
    Compose refuse de démarrer en les nommant ; y écrire les valeurs d'origine, jamais des neuves, puis
    les changer selon le manuel ; outils qui se connectaient au port 3306 de l'hôte →
    `docker compose exec mariadb`). Et la consigne hexadécimale pour la `DATABASE_URL` de Synology. Elle
    dit aussi la limite : Kesh ne voit pas le mot de passe **root** — un `kesh_dev_root` recopié d'un
    ancien `.env`, ou un root resté au gabarit `<…>`, ne déclenche aucun avertissement ; seuls le contrôle
    de fin de mise à jour du manuel (connexion refusée avec l'ancien mot de passe, AC 11 f) et la relecture
    des chevrons de `.env` le montrent.
    (b) Rubrique **Retiré** : `init-demo.sh` (démonstration semée par l'écran d'accueil ; AC 12 b). Aucune
    version du `CHANGELOG.md` ne porte encore cette rubrique (F7 de la validation P4) ; elle est légitime :
    *Removed* est l'une des six rubriques de Keep a Changelog 1.1.0, que l'en-tête du fichier déclare suivre
    (vérifié le 2026-10-09 sur `keepachangelog.com/fr/1.1.0/` : *Added*, *Changed*, *Deprecated*,
    *Removed*, *Fixed*, *Security* — la page française garde les intitulés anglais) ; le fichier en traduit
    les intitulés (Ajouté, Modifié, Corrigé, Sécurité), « Retiré » traduit *Removed*. Elle se place entre
    « Modifié » et « Corrigé », dans l'ordre de la convention. *(Les
    entrées #552, #576 et « dossiers montés ignorés » : 15-13b.)*
    (c) **Propagation** : `CHANGELOG.md:46` porte **cinq** phrases que la 15-13 rend fausses (F-P3-1 : la
    fiche unique en comptait quatre, « deux gestes » lui avait échappé) ; **quatre** relèvent de la
    15-13a, toutes réécrites : « le manuel d'administration décrit les **deux gestes** » (le décompte
    par fichier de l'AC 11 f — quatre pour `docker-compose.yml`, deux pour `docker-compose.prod.yml`, ou
    cinq et trois si la 15-13b est mergée — ou un renvoi sans nombre) ; « Kesh **refuse de démarrer**
    si … » (la liste omet le refus de **Compose**, qui s'affiche au terminal — le dire à part, comme au
    manuel) ; « Une variable que `.env` ne pose pas, ou laisse vide, prend son défaut sans
    avertissement » (faux pour `MARIADB_ROOT_PASSWORD` et `MARIADB_PASSWORD` avec `docker-compose.yml` ;
    la fin de la phrase, « — la sauvegarde pré-import reste dans `/tmp` », relève de la 15-13b et reste
    vraie tant qu'elle n'est pas mergée) ; « Un `.env` copié du gabarit sans modification ne change rien
    d'autre que le refus du placeholder du secret JWT » (un `.env` copié du **nouveau** gabarit fait
    refuser Compose ; d'un **ancien**, il fait émettre l'avertissement de l'AC 5).
    (d) **Propagation (F-P3-2)** : `CHANGELOG.md:52` (entrée #557, rubrique **Sécurité**) recopie le
    contrôle des placeholders `sudo docker compose config | grep -iE …` avec « doit rester muette » —
    le piège corrigé au manuel à l'AC 11 f (muet quand Compose refuse). Il est remplacé comme au manuel :
    les deux mots de passe MariaDB posés d'abord, puis `sudo docker compose config -q && echo 'compose
    lisible'`, dont la seconde moitié doit s'afficher, puis le `grep` — ou un simple renvoi au manuel
    (§ *Passer à la 0.13.0*) sans recopier la commande ; le choix est écrit au Dev Agent Record.

14. **Inventaire des sites non résolus tenu** (§ *Inventaire* des Dev Notes) : à la fin du développement,
    chaque site de l'inventaire est soit résolu par un AC, soit écrit comme angle mort, et les greps de
    contrôle (T9) ne trouvent **aucun** site hors de l'inventaire.

## Tasks / Subtasks

- [x] **T0 — Mesures avant d'écrire** (AC 2 c, 6 c, 11 f, 11 j, 11 l ; test 8)
  - [x] `docker compose -f docker-compose.yml config -q` sur une copie du compose portant le message de
        l'AC 2 c, **sous ses deux formes** (c1 non citée, c2 entre guillemets doubles), **avec** les deux
        variables posées (le fichier doit se lire : c'est ce que le YAML casserait) puis **sans** elles :
        relever le message exact (accents restitués ?), la forme retenue, et **laquelle des deux
        variables Compose nomme en premier** quand les deux manquent — version de Compose au Dev Agent
        Record. Sur la même copie sans les variables : `docker compose ps` (et `logs`) refusent-ils aussi
        (AC 11 f) ? Relever. Sur la copie **avec** les variables : graphie de `docker compose config` pour
        un service sans `ports` et pour un service qui en a (forme longue `published:`), pour le contrôle
        écrit de l'AC 11 f.
  - [x] Erreur `sqlx` d'un mauvais mot de passe sur la MariaDB de dev (base du worktree, **jamais** le
        conteneur `kesh-mariadb-dev` redémarré) : variante d'`sqlx::Error`, numéro 1045 joignable ou non.
  - [x] Durée de l'échec de connexion du test 8 : hôte `kesh-15-13.invalid` (échec de résolution, non
        rejoué par `sqlx`) contre `127.0.0.1:1` (`ConnectionRefused`, rejoué jusqu'à
        `db_connect_timeout` = 10 s, `config.rs:639`) ; retenir le premier s'il échoue vite **et** produit
        « Base de données indisponible », sinon le second, durée écrite.
  - [x] `make fr` sur l'état d'avant : garder le `.log` (référence des `Overfull`) et le PDF aplati
        (référence des contrôles de l'AC 11 l, chacun **rouge** dessus).
- [x] **T1 — Compose, gabarit** (AC 1, 2, 3)
  - [x] `docker-compose.yml` : `ports:` de `mariadb` retiré + commentaire ; trois `:?` (forme du T0).
  - [x] `.env.example` : bloc MariaDB réécrit (ordre de l'AC 3 c), lignes commentées ; commentaire
        `DATABASE_URL` (hex).
  - [x] `grep -rnF kesh_dev docker-compose.yml docker-compose.prod.yml .env.example` → vide.
- [x] **T2 — Code** (AC 5, 6)
  - [x] `config.rs` : `MOTS_DE_PASSE_PUBLIES` + avertissement (a) **et** avertissement gabarit (a-bis) par
        `is_template_placeholder` (doc-comment complété, AC 5 d) — crates `url` **et** `percent-encoding`
        en dépendances directes de `kesh-api` (`url` ne réexporte pas `percent_encoding` ; toutes deux
        déjà au `Cargo.lock`, `url` 2.5.8 et `percent-encoding` 2.3.2 : aucun paquet neuf) ;
        `numero_erreur_mariadb` et `indice_connexion` (AC 6 b).
  - [x] `main.rs:82-87` : le message d'erreur ajoute l'indice quand la composition des deux fonctions en
        rend un.
- [x] **T3 — Tests Rust** (AC 5, 6, 10) — voir § *Tests et mutations*. Dans
      `configuration_transmise.rs` : scission d'`Interpolation::Obligatoire`, `VALEURS_COMPOSEES`
      resserré (AC 10 a, test `valeurs`). Le lancement du binaire est factorisé (DRY) :
      `sortie_du_binaire_sans_configuration` (`configuration_transmise.rs:2232`, privée) devient un appel
      d'un helper `lancer_binaire(env: &[(&str, &str)]) -> std::process::Output` (répertoire temporaire
      courant, `env_clear`, `NO_COLOR=1`, puis les couples donnés) posé dans
      `crates/kesh-api/tests/common/binaire.rs` et inclus par `#[path = "common/binaire.rs"] mod binaire;`
      dans `configuration_transmise.rs` et `demarrage_mariadb.rs` — **pas** par `mod common;`, qui
      tirerait dans ces deux binaires l'amorce de base de `common/mod.rs` qu'ils n'utilisent pas.
      **Le helper ne fait aucune assertion** (F-P3-9) : chaque appelant pose la sienne —
      `sortie_du_binaire_sans_configuration` garde son `!sortie.status.success()`
      (`configuration_transmise.rs:2232-2254`), les tests 8 et 10 posent « code de sortie ≠ 0 ».
- [x] **T4 — CI** (AC 4) : `.github/workflows/ci.yml` et `docs/ci.md` — dont le « 4 jobs (`backend`,
      `frontend`, `e2e`, `docker-build`) » de `docs/ci.md:9-10` (ligne 9 : pull request ; ligne 10 :
      « mêmes 4 jobs » au push), faux (`ci.yml` n'en a que trois : `backend:` `:21`, `frontend:` `:189`,
      `docker-build:` `:233` ; aucun job `e2e`), corrigé en « 3 jobs (`backend`, `frontend`,
      `docker-build`) » dans le même patch que `:12` et `:117`. Le même symptôme, grepé par la valeur
      (`grep -n e2e docs/ci.md`), court ailleurs dans le fichier : `:23` (schéma), `:30` (tableau des
      jobs), `:131`, `:164`, `:186`, `:210` décrivent un job `e2e` absent de `ci.yml` (`:66`,
      `npm run test:e2e`, est une recette locale et reste juste) — défaut **antérieur** à la story, hors de
      son sujet : ces lignes ne sont pas réécrites ; une phrase sous `:10` dit que les sections qui
      décrivent `e2e` ne correspondent à aucun job de `ci.yml` et renvoie à **#577** (le `CLAUDE.md`
      affirme lui aussi un smoke E2E en CI que `ci.yml` ne porte pas — même issue, `CLAUDE.md` non modifié
      ici).
- [x] **T5 — Manuel et brochure** (AC 11) : sites (a)–(d), (f)–(h), (j) et (m), puis `make fr` (trois
      PDF), contrôles (k)–(m).
- [x] **T6 — `DOCKER_START.md`, retrait d'`init-demo.sh`, CHANGELOG** (AC 12, 13) — `git rm init-demo.sh`.
- [x] **T7 — Recette de changement de mot de passe rejouée** (AC 11 f-h) sur un **conteneur MariaDB
      jetable** (`docker run --rm -d --name kesh-15-13-recette mariadb:10.11`, sans port publié, réseau
      jetable, **jamais** `kesh-mariadb-dev`) : initialisé avec `kesh_dev`/`kesh_dev_root`, recette
      appliquée ; **contrôle de fin de mise à jour** de l'AC 11 f rejoué tel qu'il sera écrit (forme
      coupée par `\`) — connexion avec les anciens mots de passe **refusée**, avec ceux de
      l'environnement **acceptée**, pour `root` et `kesh` ; `SELECT CURRENT_USER()` relevé pour
      `-h 127.0.0.1` **et** pour `-h <nom ou IP du conteneur>` (quel compte `root` répond dans chaque
      cas : `root@localhost` ou `root@%`) ; comportement de `root@localhost` par socket relevé
      (l'authentification `unix_socket` laisserait-elle entrer `root` avec un mauvais mot de passe ? d'où
      `--protocol=TCP`) ; `healthcheck.sh --connect --innodb_initialized` toujours vert ; **dump puis
      restauration** dans la forme du § *Forme du script de sauvegarde* (une table témoin créée, dumpée,
      supprimée, restaurée, relue) ; effet de `up -d kesh-api` (forme nommée, `admin-manual.tex:1747-1748`)
      sur le service `mariadb` relevé ; conteneur supprimé. Commandes et sorties au Dev Agent Record.
- [x] **T8 — Mutations** : chacune appliquée, test rouge relevé, fichier restauré **et touché**
      (`touch`, mémoire « mutation restaurée, binaire périmé ») ; tableau au Dev Agent Record.
- [x] **T9 — Propagation et inventaire** (AC 14) — exclusions communes
      `--exclude-dir={target,node_modules,.git,_bmad-output,_bmad,.claude,.svelte-kit}` (les skills BMAD de
      `.claude/` et `_bmad/` portent des `docker-compose up` étrangers au dépôt) :
      (1) `grep -rnIE "3306|kesh_dev|exec (-T )?db\b|base64 32|init-demo|down -v|(compose|décrit les)[^.]{0,20}deux gestes|doit rester muet" …`
      — chaque résultat est dans l'inventaire (résolu ou angle mort), sinon il y entre. Le jeton du décompte
      des gestes est **restreint** (validation P4, R4-1/F1) : `deux gestes` seul rendait six homonymes sans
      rapport (`CLAUDE.md:760`, `crates/kesh-api/src/routes/users.rs:334`,
      `crates/kesh-api/tests/users_e2e.rs:560`, `crates/kesh-db/src/repositories/invoice_settlements.rs:395`,
      `docs/manual/fr/user-manual.tex:1309`, `frontend/tests/e2e/invoice-unvalidate.spec.ts:67`) ; la forme
      restreinte, rejouée à `c702b7d5`, ne rend que `CHANGELOG.md:46` et `admin-manual.tex:1793`, les deux
      sites traités. Le grep entier, rejoué au même commit, ne rend aucun site hors de l'inventaire
      ci-dessous ;
      (2) `grep -rn "MARIADB_" …` (même exclusions) ;
      (3) un grep **par la valeur** du geste de démarrage, que les jetons ci-dessus ne voient pas (c'est par
      là que la brochure avait échappé à l'inventaire), **sur tout le dépôt** et non plus sur une liste de
      fichiers (F-P3-6 : `CHANGELOG.md`, `.env.example` et les commentaires des compose y échappaient) :
      `grep -rnE "docker[ -]compose( -f [^ ]+)? up" …` (les `.tex` et le HTML ; les PDF par leur aplati) —
      chaque site qui démarre `docker-compose.yml` doit soit venir **après** la pose des deux mots de passe,
      soit les nommer, soit être trié comme assumé ;
      (4) `grep -rnE "config \| grep|config \\\\$" …` puis lecture de la ligne suivante : tout contrôle
      « doit rester muet » qui suit un `docker compose config` est précédé du `config -q && echo` (AC 11 f,
      13 d).
- [x] **T10 — Gates** : `scripts/test-fast.sh` complet (base du worktree remise à zéro avant —
      `DROP`/`CREATE` de **ses** bases, jamais un redémarrage du conteneur), frontend non touché (gate
      frontend tout de même, CLAUDE.md), **E2E complet au dernier commit de code** (D7) ; la story ne
      touche aucune migration (exception `kesh-db` sans objet).

## Tests et mutations

Périmètre : tests **neufs ou modifiés** par la 15-13a, de `de285ea8` au commit de développement.
Numérotation de la fiche unique conservée ; la ligne **3a** est neuve (validation P3, ci-dessous).

| # | Test (fichier) | Établit | Mutation qui le rend rouge |
|---|---|---|---|
| 1 | `mariadb` (nouveau `#[test]`, `configuration_transmise.rs`) | AC 10 a | **M1** remettre `ports: ["3306:3306"]` au service `mariadb` · **M2** remettre `"127.0.0.1:3306:3306"` actif · **M3** `MARIADB_PASSWORD: ${MARIADB_PASSWORD:-kesh_dev}` · **M4** `${MARIADB_ROOT_PASSWORD}` (sans `?`) · **M28** `MARIADB_PASSWORD: ${MARIADB_PASSWORD?m}` (forme `?`, qui laisse passer une ligne vide) |
| 2 | `mots_de_passe_publies` (nouveau, même fichier) | AC 10 b | **M6** ligne active `MARIADB_PASSWORD=kesh_dev` dans `.env.example` · **M7** `kesh_dev_root` dans un **commentaire** de `docker-compose.yml` · **M8** `MARIADB_ROOT_PASSWORD=x` décommentée (valeur quelconque) |
| 3a | `valeurs` (existant, `configuration_transmise.rs:1743`, famille (V) — `VALEURS_COMPOSEES` resserré à `${MARIADB_PASSWORD:?`) | AC 10 a (`DATABASE_URL`) | **M5** `DATABASE_URL` en `${MARIADB_PASSWORD:-x}` seulement |
| 4 | `s_service_mariadb` (nouveau, (S)) — sources refusées et acceptées de l'AC 10 d, dont la valeur **citée** | AC 10 d | **M12** le contrôle accepte `DefautSiVide` comme forme obligatoire · **M13** le contrôle ignore la clé `ports:` · **M29** l'analyseur range `?` avec `:?` (scission annulée) |
| 7 | `from_env_warns_on_published_database_password` (nouveau, `config.rs`, capture `from_env_with_logs`) — cas `kesh_dev`, `kesh_dev_root`, `kesh%5Fdev` (décodé) → avertissement (a) ; **gabarits** : `mysql://kesh:%3Cmot%20de%20passe%20utilisateur%20applicatif%3E@h/kesh` (encodé) **et** la même URL non encodée, telle que Compose la compose depuis `.env` (`mysql://kesh:<mot de passe utilisateur applicatif>@h/kesh`), et `mysql://kesh:%3CMARIADB_PASSWORD%3E@h/kesh` → avertissement (a-bis), **distinct** de (a) — la forme non encodée **s'analyse** : établi à la source en validation P4 (`url` 2.5.8, `parser.rs:880-952`, `parse_userinfo` encode l'espace, `<` et `>` par le jeu `USERINFO`, `:20-37` ; `Url::password()` rend `%3Cmot%20de%20passe…%3E`, que le décodage restitue), cas **positif ferme**, sans repli ; **témoins** : `mysql://kesh_dev:fort@…` (l'utilisateur, pas le mot de passe), `Kesh_Dev` (casse), `a%3Cb%3Ec` (chevrons **intérieurs**, admis par le prédicat), `%FF` (octet non UTF-8 : ni panique ni avertissement, AC 5 a), mot de passe fort, URL sans mot de passe → aucun avertissement ; **aucun message capturé ne contient le mot de passe ni l'URL** (pour chaque cas averti, sous ses formes encodée et décodée) | AC 5 a, a-bis, b | **M16** avertissement (a) retiré · **M17** comparaison sur l'utilisateur · **M18** sans décodage pour cent · **M19** comparaison insensible à la casse · **M42** appel d'`is_template_placeholder` retiré (seule la liste reste) · **M43** l'avertissement (a) ou (a-bis) interpole le mot de passe décodé (F6 de la validation P4) |
| 8 | `mot_de_passe_publie_avertit_au_demarrage` (nouveau, lance le binaire comme `rust_log_vide_vaut_info` : `DATABASE_URL=mysql://kesh:kesh_dev@<hôte du T0>/kesh` — `kesh-15-13.invalid` si l'échec est immédiat, sinon `127.0.0.1:1` et ses ≈ 10 s —, `KESH_JWT_SECRET` valide, `KESH_HOST=127.0.0.1` ; le binaire sort en échec de connexion) — fichier neuf `crates/kesh-api/tests/demarrage_mariadb.rs`, lancement par le helper `lancer_binaire` (T3) ; assertions : l'avertissement est dans la sortie, **code de sortie ≠ 0** | AC 5 (câblage) | **M20** l'avertissement émis hors de `from_env` (fonction non appelée) — assertion de montage : la sortie contient « Base de données indisponible » |
| 9 | `indice_connexion_refusee` (nouveau, unitaire, `config.rs`) — la décision pure `indice_connexion` : `Some(1045)` → indice ; `Some(1049)`, `Some(1044)`, `None` → rien | AC 6 b | **M21** 1044 au lieu de 1045 · **M22** indice pour toute erreur |
| 10 | `mauvais_mot_de_passe_donne_l_indice` (nouveau, `demarrage_mariadb.rs`) — lance le binaire avec **les mêmes variables que le test 8** (`KESH_JWT_SECRET` valide, `KESH_HOST=127.0.0.1` : `Config::from_env` exige le secret, `config.rs:646`, **avant** la connexion, `main.rs:63-69`) et la `DATABASE_URL` des tests (`DATABASE_URL` de l'environnement, **obligatoire** : panique explicite si absente, pas de saut silencieux) dont le mot de passe est remplacé par `mauvais-15-13` ; la sortie contient l'erreur d'origine **et** l'indice ; code de sortie ≠ 0 ; la sortie ne contient **pas** `mauvais-15-13` (ni l'indice ni le message ne citent l'URL ou son mot de passe) ; **assertion de montage** : la sortie ne contient **pas** « Erreur de configuration » (sinon le binaire n'a jamais tenté la connexion) — couvre l'**extraction** `numero_erreur_mariadb` | AC 6 a-b | **M23** `main.rs` n'appelle pas la fonction d'indice · **M44** le message d'échec de connexion interpole `config.database_url` |
| 13 | Étape CI « Validate compose files » (shell, `ci.yml`) | AC 4 | **M27** remettre `:-kesh_dev_root` → la vérification négative (b) échoue (rejouée en local au T8 avec le même script) |

**Décompte** (recompté sur le tableau) : 9 lignes — 8 côté Rust, qui portent **8 fonctions de test** (7
neuves : lignes 1, 2, 4, 7, 8, 9, 10 ; 1 existante modifiée : ligne 3a, `valeurs`), dans un fichier neuf
(`demarrage_mariadb.rs`) et deux existants (`configuration_transmise.rs`, `config.rs`) — plus 1 étape CI, et
un module d'aide de test neuf (`tests/common/binaire.rs`, aucun test ; `sortie_du_binaire_sans_configuration`
refactorée, non comptée : ce n'est pas un test) ; **24 mutations** (M1–M8, M12, M13, M16–M23, M27–M29,
M42–M44), toutes attendues rouges. **M43** et **M44** (validation P4, F6) gardent la seule propriété de
sécurité des AC 5 et 6 qu'aucune mutation ne gardait : ni mot de passe ni URL dans un journal. **Ligne 3a** (validation P3, constat du remédiateur) : la fiche unique
rangeait le resserrement de `VALEURS_COMPOSEES` sous le test `transmission` et M5 sous le test 1 ; or
`VALEURS_COMPOSEES` n'est lu que par `controle_valeur` (`configuration_transmise.rs:592-603`), appelé par
`controle_valeurs` (`:763`) et le test `valeurs` (`:1743`, famille (V)) — c'est `valeurs` que M5 rougit.

Le test **(L)** (`garde_lecture_du_code`) et **(E)** (`env_example`) doivent rester verts **sans
modification** de leurs listes : la story n'ajoute aucune lecture d'environnement ni aucune variable.

## Dev Notes

### Ce que la story change, et ce qu'elle préserve

- **Code** : deux modules de `kesh-api` — `config` (avertissements, fonctions d'indice, tests) et `main`
  (un appel dans un message d'erreur). Aucune migration ; `kesh-db` non touché ; frontend non touché.
- **Préservé** : le réseau `kesh-network` ; `docker-compose.dev.yml` (pile de dev, mots de passe en dur,
  port en loopback) ; `docker-compose.prod.yml` (aucune modification de cette story) ; la transmission des
  variables de la 15-11a.

### Pourquoi refuser par Compose et seulement avertir dans Kesh (C-15-13-2)

Kesh ne voit pas `MARIADB_ROOT_PASSWORD` (non transmise, et ne doit pas l'être : C71 interdit `env_file`).
Il ne voit que `DATABASE_URL`. Refuser `kesh_dev` dans Kesh casserait la pile de développement
(`docker-compose.dev.yml:15` compose `mysql://kesh:kesh_dev@mariadb:3306/kesh`), le montage E2E local et
la documentation de test (`docs/testing.md`, `crates/*/README.md`, `CLAUDE.md`) — tous sur `kesh_dev`.
`KESH_TEST_MODE` ne permet pas de distinguer : `docker-compose.dev.yml` lie `0.0.0.0` et
`TestModeWithPublicBind` l'interdit. D'où le partage :

- **le refus** est porté par Compose, là où vit le défaut : sans les variables, rien ne démarre, et le
  message nomme la variable ;
- **l'avertissement** est porté par Kesh, pour les `.env` existants qui recopient les anciennes valeurs du
  gabarit (`.env.example:336-339` jusqu'à la 0.12.1) — le port n'étant plus publié, le risque restant est
  celui d'un conteneur du même réseau ; l'avertissement le dit. Le mot de passe **root**, que Kesh ne voit
  pas, n'est couvert que par le contrôle de fin de mise à jour du manuel, qui se connecte **à la base**
  avec les anciens mots de passe (AC 11 f, C-15-13-10) — un `grep` sur `docker compose config` ne lirait
  que `.env`, et resterait muet quand `.env` porte une valeur neuve alors que la base garde l'ancienne.

**Valeur gabarit (`<…>`)** — *rectifié en validation P3 (F-P3-3, C-15-13-17)* : la fiche unique la disait
« hors de portée de Kesh pour la même raison ». C'était faux pour le mot de passe **applicatif** : Kesh le
lit dans `DATABASE_URL`, et le prédicat `is_template_placeholder` existe. Il est donc **averti** (AC 5 a-bis),
pas refusé — même raison qu'en (a) : un mot de passe faible n'empêche pas de démarrer, et l'avertissement
suffit à le faire voir. La raison « Kesh ne voit pas root » ne vaut que pour **root** : un root gabarit reste
un angle mort, tracé par l'issue **#578** (§ *Angles morts*).

### La mise à jour d'une installation existante — les cas, un par un

Pour `docker-compose.yml` (MariaDB comprise). MariaDB **n'applique `MARIADB_*_PASSWORD` qu'à
l'initialisation d'un dossier de données vide** : ensuite, ces variables sont ignorées et seule la base
fait foi. C'est le piège central. *(Les cas propres à la sauvegarde — compose gardé sans montage, image
ancienne, restauration après un import raté — sont dans la 15-13b.)*

| État de l'installation avant mise à jour | Après re-téléchargement du compose 0.13.0 | Ce que dit le manuel |
|---|---|---|
| `.env` copié d'un ancien gabarit : `MARIADB_PASSWORD=kesh_dev` actif | démarre ; port fermé ; **avertissement** Kesh au démarrage ; le contrôle de fin de mise à jour (connexion avec `kesh_dev` **acceptée**) le montre | changer les deux mots de passe (procédure AC 11 g) |
| `.env` sans les lignes (s'appuyait sur les défauts du compose) | **Compose refuse** : `required variable MARIADB_PASSWORD is missing a value: …` | écrire `MARIADB_ROOT_PASSWORD=kesh_dev_root` et `MARIADB_PASSWORD=kesh_dev` (les valeurs d'**origine**), démarrer, puis changer (AC 11 g) |
| `.env` avec des mots de passe propres posés **avant** la première initialisation | aucun changement | rien |
| base créée avec la chaîne gabarit **littérale** du manuel (`<mot de passe fort>`, `<mot de passe utilisateur applicatif>`, `admin-manual.tex:253-254`) | démarre ; **avertissement** Kesh (AC 5 a-bis) pour le mot de passe **applicatif** ; **aucun message** pour root (Kesh ne le voit pas) | **ne pas** toucher `.env` ; changer les deux mots de passe par la procédure (AC 11 b, 11 g) ; root gabarit : seule la relecture des chevrons de `.env` le montre |
| l'exploitant écrit une valeur **neuve** de `MARIADB_PASSWORD` dans `.env` sans `ALTER USER` | Kesh sort sur **1045** — avec l'**indice** de l'AC 6 | revenir à l'ancienne valeur, puis la procédure |
| l'exploitant écrit une valeur **neuve** de `MARIADB_ROOT_PASSWORD` dans `.env` sans `ALTER USER` (cas probable : Compose la nomme absente, et qui n'en avait jamais posé « en met une forte ») | **rien ne rougit** : Kesh n'utilise pas root, le healthcheck passe par le compte `healthcheck` de l'image (`docker-compose.yml:19`) ; root garde `kesh_dev_root` dans la base ; **la sauvegarde nocturne échoue** (le script lit `$MARIADB_ROOT_PASSWORD` dans le conteneur), dans un journal que personne ne lit | **détection** : le contrôle de fin de mise à jour (connexion root avec `kesh_dev_root` **acceptée**, avec la valeur de `.env` **refusée**) et le lancement du script à la main (AC 11 f) ; remède : revenir à l'ancienne valeur, puis la procédure |
| ligne `#MARIADB_PASSWORD=` du nouveau gabarit décommentée **sans valeur** | **Compose refuse** (forme `:?`, qui refuse le vide — d'où la scission de l'AC 10 a) | écrire la valeur d'origine |
| outil de l'hôte connecté à `127.0.0.1:3306` ou `IP:3306` (client SQL, script de sauvegarde) | refusé (port non publié) | `docker compose exec mariadb …` ; dépannage ponctuel : décommenter la ligne loopback |
| compose **gardé** (non re-téléchargé) | port toujours publié, défauts toujours là ; avertissement si `kesh_dev` ou gabarit | les gestes du paragraphe « Pour qui garde son fichier compose » (AC 11 f) |
| compose neuf, **image ancienne** (oubli du `pull`) | pas d'avertissement ni d'indice (image 0.12.x) | le manuel impose `docker compose pull` avant `up -d` (déjà écrit, 15-11a) |

**Deux cas ne se signalent par aucun message** et ne sont vus que par un geste prescrit : le mot de passe
root neuf écrit sans `ALTER USER` (sauvegarde nocturne en échec) et le mot de passe **root** resté sur une
chaîne gabarit. Le manuel fait voir le premier par le **contrôle de fin de mise à jour** — connexion à la
base avec les anciens mots de passe, refus attendu — et par le **lancement manuel du script de sauvegarde**
(AC 11 f) ; le second, par la relecture des valeurs entre chevrons de `.env` que la procédure de mise à
jour prescrit déjà (« vérifier les placeholders », `admin-manual.tex:1746`) et que l'AC 11 b étend aux deux
mots de passe MariaDB. Toutes les autres pannes ont un message (Compose ou Kesh), et toute exposition
résiduelle un avertissement ou ce contrôle — dont, depuis la validation P3, le mot de passe **applicatif**
gabarit (AC 5 a-bis). *(La version de la fiche unique avant la P1 affirmait qu'« aucun de ces cas ne
casse en silence » : le tableau était incomplet — F3 et R-4 de la validation P1. Celle d'avant la P3
rangeait le gabarit applicatif parmi les cas silencieux : F-P3-3.)*

`docker-compose.prod.yml` (Synology, MariaDB externe) : cette story n'y change rien (le montage `./backup`
est de la 15-13b). La MariaDB de Package Center relève de DSM ; le manuel peut rappeler de vérifier qu'elle
n'écoute pas sur le réseau sans nécessité, sans recette (angle mort).

### Le refus de Compose, mesuré en CI dans les deux sens (AC 4)

`docker compose config -q` sans `.env` **échouera** sur `docker-compose.yml` dès l'AC 2 : l'étape actuelle
(`ci.yml:242-245`) rougirait. La corriger en lui donnant des valeurs ne suffit pas — il faut aussi
**exiger l'échec** sans elles, sinon le retour d'un défaut (`:-kesh_dev`) ne se verrait nulle part hors du
test Rust. Esquisse :

```sh
MARIADB_ROOT_PASSWORD=ci MARIADB_PASSWORD=ci docker compose -f docker-compose.yml config -q
if env -u MARIADB_ROOT_PASSWORD -u MARIADB_PASSWORD \
     docker compose -f docker-compose.yml config -q 2> refus.txt; then
  echo "docker-compose.yml doit refuser l'absence des mots de passe MariaDB"; exit 1
fi
grep -qE 'MARIADB_(ROOT_)?PASSWORD' refus.txt
docker compose -f docker-compose.prod.yml config -q
```

Le runner n'a pas de `.env` à la racine (le dépôt n'en versionne pas : `git ls-files | grep '^\.env'` →
`.env.example` seul) ; vérifier en T0 qu'aucun `.env` local ne fausse la mesure locale (le déplacer le
temps du T8, ou lancer depuis une copie).

### Forme du script de sauvegarde (AC 11 h)

Le script quotidien (`admin-manual.tex:1446-1479`) passe la commande **entière** à `sh -c` entre
apostrophes, pour que `$MARIADB_ROOT_PASSWORD` soit lu **dans le conteneur** ; chaque ligne tient en 76
caractères (AC 11 k). Forme de référence, à rejouer au T7 avant d'être écrite :

```sh
COMPOSE_DIR="/chemin/du/compose"   # répertoire de docker-compose.yml
cd "${COMPOSE_DIR}"

docker compose exec -T mariadb sh -c \
    'mariadb-dump --single-transaction --routines --triggers \
       --events --add-drop-database --databases kesh \
       -u root -p"$MARIADB_ROOT_PASSWORD"' \
    | gzip > "${BACKUP_FILE}"
```

La barre oblique **à l'intérieur** des apostrophes n'est pas lue par le shell de l'hôte : elle est
transmise à `sh`, qui lit la barre suivie d'un saut de ligne comme une continuation. La restauration
(`:1511-1512`) prend la même forme, l'entrée standard traversant `sh` :
`zcat <fichier>.sql.gz | docker compose exec -T mariadb sh -c 'mariadb -u root -p"$MARIADB_ROOT_PASSWORD"'`
(coupée par `\` au manuel). `-T` est obligatoire dans les deux sens (pas de pseudo-terminal dans un tube).
Au T7, le conteneur jetable est lancé par `docker run` et non par Compose : la même commande intérieure
passe par `docker exec -i kesh-15-13-recette sh -c '…'` — c'est la commande **intérieure** qui est
validée, la forme `docker compose exec` restant celle du manuel.

### Forme de la recette de changement (AC 11 g)

Forme de référence, à rejouer au T7 (sur le conteneur jetable, par `docker exec -i`) avant d'être écrite —
la mesure fixe la forme finale ; les exigences, elles, sont celles de l'AC 11 g (C-15-13-23) :

```sh
cd /chemin/du/compose
sudo docker compose stop kesh-api
NEW=$(openssl rand -hex 32)          # jamais tapé : absent de l'historique
sudo docker compose exec -T mariadb \
    sh -c 'mariadb -uroot -p"$MARIADB_ROOT_PASSWORD"' <<SQL
ALTER USER 'kesh'@'%' IDENTIFIED BY '$NEW';
SQL
echo "$NEW"    # à recopier dans .env (MARIADB_PASSWORD), à l'éditeur
sudo docker compose up -d
```

Le document en ligne est développé par le shell de l'hôte (`$NEW`) et transmis par l'**entrée standard** :
le mot de passe ne figure dans aucun argument de processus. Le même schéma vaut pour chaque compte `root`
listé, avec `MARIADB_ROOT_PASSWORD` dans `.env` — et, pour root, la connexion du `sh -c` suivant utilise
encore l'**ancienne** valeur de l'environnement du conteneur jusqu'au `up -d` (recréation) : le T7 vérifie
cet ordre. L'`echo` affiche le secret au terminal de l'exploitant (comme l'`openssl rand` de l'étape 3) :
c'est le prix de ne pas l'écrire dans une ligne de commande ; une recette qui écrit `.env` sans
l'afficher est admise si le T7 la valide sans l'exposer à `ps`.

### Inventaire fermé des sites — parties MariaDB

Greps de constitution : ceux du T9 (rejoués et élargis en validation P3). Constitué à partir de
l'inventaire de la fiche unique (57 lignes, réparties entre les deux sous-fiches : § *Découpage* de la
fiche index), puis complété des sites trouvés en validation P3. **Résolu** = un AC le traite ; **assumé** =
angle mort écrit.

| Site | Sujet | Sort |
|---|---|---|
| `docker-compose.yml:8`, `:11`, `:13`, `:53` | défauts, port | résolu (AC 1, 2) |
| `docker-compose.dev.yml:15`, `:54`, `:56-59` | dev : `kesh_dev`, `127.0.0.1:3306` | **assumé** (pile de dev non distribuée, loopback) |
| `.env.example:47` (`DATABASE_URL=<EDIT: …>`), `:328-339` | gabarit | résolu (AC 3) |
| `.github/workflows/ci.yml:55`, `:57`, `:64`, `:93` ; `release.yml:26`, `:28`, `:70`, `:94` | MariaDB de service du runner (`3306:3306`, `kesh_dev`, `kesh_root`) | **assumé** (runner éphémère, non distribué) |
| `.github/workflows/ci.yml:242-245` | validation des compose | résolu (AC 4) |
| `docs/ci.md:9-10`, `:12`, `:117` | doc de la CI (dont « 4 jobs », faux) | résolu (AC 4 d, T4) |
| `docs/ci.md:23`, `:30`, `:131`, `:164`, `:186`, `:210` | job `e2e` décrit, absent de `ci.yml` | **assumé, NON résolu** — défaut antérieur, hors sujet ; une phrase de renvoi sous `:10` vers **#577** (T4) |
| `crates/kesh-api/src/config.rs` — 34 URL `mysql://test:test@localhost:3306/test`, dont `:1485` | URL factices des tests | **assumé** (n'émettent pas l'avertissement : mot de passe `test`, ni publié ni gabarit) |
| `crates/kesh-api/src/config.rs:1398-1405` (`is_template_placeholder` et son doc-comment) | prédicat réemployé | résolu (AC 5 a-bis, d) |
| `crates/kesh-api/src/main.rs:82-87` | message d'échec de connexion | résolu (AC 6) |
| `crates/kesh-api/src/routes/reconciliation.rs:1129`, `:1136` ; `crates/kesh-db/src/retry.rs:57`, `:106` | constantes `MARIADB_SAVEPOINT_DOES_NOT_EXIST`, `MARIADB_DEADLOCK_ERROR_CODE` (grep `MARIADB_`, rejoué en remédiation P3 : absentes de l'inventaire jusque-là) | **assumé** — homonymes (codes d'erreur), forme citée par l'AC 6 b |
| `crates/kesh-api/tests/*_e2e.rs` (58 lignes `mysql://test:test@localhost:3306/test`, 2 `mysql://stub:stub@127.0.0.1:3306/stub`), `tests/health_endpoint.rs:36`, `tests/spa_resilience.rs:46` | URL de test | **assumé** (base de test, mot de passe non publié) |
| `crates/kesh-api/tests/configuration_transmise.rs:169-174` | `VALEURS_COMPOSEES` | résolu (AC 10 a, motif resserré, test `valeurs`) |
| `crates/kesh-api/tests/configuration_transmise.rs:190-193`, `:456` | `MARIADB` (variables d'hôte du service MariaDB), message de (E) | inchangé, voulu |
| `crates/kesh-api/tests/configuration_transmise.rs:541-564`, `:626`, `:2232-2254` | `Interpolation::Obligatoire` (définition, analyse, bras de `controle_valeur`) ; `sortie_du_binaire_sans_configuration` | résolu (AC 10 a ; T3) |
| `crates/kesh-api/src/middleware/auth.rs:234`, `:242` | URL factice `mysql://stub:stub@127.0.0.1:3306/stub` d'un test | **assumé** (test, `connect_lazy`) |
| `crates/kesh-api/README.md:169`, `:172`, `crates/kesh-db/README.md:34`, `:46`, `:53`, `:64`, `docs/testing.md:27`, `:29`, `:43`, `:53`, `:99`, `:100`, `:161`, `:171`, `:284`, `:302`, `scripts/test-fast.sh:33`, `scripts/regen-test-schema.sh:72`, `scripts/seed-dev-db.sql:8`, `:10`, `scripts/mariadb-init/01-dev-grants.sql:10`, `CLAUDE.md:179`, `:504`, `README.md:180`, `frontend/DEBUGGING-KF007.md:16` | recettes de dev et de test sur `kesh_dev` / `kesh_root` / `127.0.0.1:3306`, et démarrage de la pile de dev (`docker compose -f docker-compose.dev.yml up -d mariadb` : `crates/kesh-db/README.md:34`, `docs/testing.md:99`, `:161`) | **assumé** (base de dev jetable `docker-compose.dev.yml`, ou service MariaDB de la CI) |
| `init-demo.sh` (fichier entier, dont `:20`, `:23`, `:59`, `:69`, `:70-75`) | script de démonstration sur `kesh-mariadb` + `kesh_dev`, `DELETE` sans transaction | résolu **par retrait** (AC 12 b, C-15-13-13) |
| `.svelte-kit/ambient.d.ts:42-45`, `:240-243` (versionné) | noms `MARIADB_*` d'un environnement de build, sans valeur | **assumé** (fichier engendré, aucun secret) |
| `DOCKER_START.md:6`, `:12-13`, `:102`, `:146` | prérequis, phrase « Kesh refuse » sans `.env`, accès, volume | résolu (AC 12 a) |
| `DOCKER_START.md:134-137` | « Base de données ne s'initialise pas » → `down -v`, `up --build` | résolu (AC 12 a-bis) |
| `DOCKER_START.md:78` | « Arrêter et supprimer les données » (`down -v` explicite) | **assumé** — dit ce qu'il fait |
| `DOCKER_START.md:27`, `:37`, `:41`, `:86`, `:96`, `:132` | `docker compose up` (`:41` : `up -d` tire l'image publiée) | résolu par l'étape `.env` de l'AC 12 a (les deux mots de passe posés avant) |
| `docs/manual/fr/marketing-brochure.tex:542` + `marketing-brochure.pdf` | « `docker compose up -d`, une fois `KESH_JWT_SECRET` généré » | résolu (AC 11 m) |
| `docs/manual/fr/admin-manual.tex` — 21 lignes du `.tex` portent `docker compose up` (`grep -cE "docker[ -]compose( -f [^ ]+)? up"` → 21 : `:227`, `:275`, `:578`, … `:2247`), 21 occurrences aussi au PDF aplati | démarrage | **assumé** — `:227` est de la prose de l'étape 2 (« `docker compose up -d` la télécharge »), sans geste de démarrage ; les vingt autres sont postérieures à l'étape `.env` (`:229-270`), qui pose les mots de passe (AC 11 b), ou propres à Synology (`docker-compose.prod.yml`, sans MariaDB) — F2 de la validation P4 |
| `README.md:77`, `website/index.html:189` | `docker compose -f docker-compose.dev.yml up -d` | **assumé** — pile de dev ; aucun autre `docker compose up` sur `website/` (`grep -rniE "docker[ -]compose" website` : trois mentions sans commande) |
| `.env.example:19`, `:297` | commentaires « `docker compose up -d` après une modification » ; avertissement de Compose sur `$` | **assumé** — le gabarit pose les mots de passe dans son bloc MariaDB (AC 3), qu'un `up` suit |
| `docker-compose.yml:31-35`, `:49` (commentaires : l'image publiée est tirée ; `up -d` après une modification de `.env`) ; `docker-compose.prod.yml:12`, `:34`, `:53`, `:69` | `docker compose up` en commentaire | **assumé** — `docker-compose.yml` : le refus nommé de Compose dit ce qui manque ; `docker-compose.prod.yml` : sans MariaDB |
| `docs/kesh-specifications.txt:579`, `:1603`, `:1638`, `:1972`, `:2002`, `:2334` (**NFR-DEPLOY-1**, « une seule commande `docker-compose up` »), `:2629`, `:3153` | spécification historique | **assumé** — document de spécification, non réécrit ; NFR-DEPLOY-1 n'est plus vrai depuis la 15-11a (`.env` obligatoire) et l'est encore moins ici : signalé à l'orchestrateur |
| `admin-manual.tex:171`, `:176`, `:2362` | ports | résolu (AC 11 a) |
| `admin-manual.tex:240-259`, `:1746` | étape `.env`, placeholders | résolu (AC 11 b) |
| `admin-manual.tex:546` | `base64` → `hex` | résolu (AC 11 c) |
| `admin-manual.tex:689-694` | note MariaDB de `sec:env-vars` | résolu (AC 11 d) |
| `admin-manual.tex:1780-1786` | contrôle des placeholders muet quand Compose refuse | résolu (AC 11 f) |
| `admin-manual.tex:1460`, `:1469`, `:1511-1512`, `:2216` | `exec db`, `-p"${…}"` | résolu (AC 11 h) |
| `admin-manual.tex:1757-1841`, dont l'encadré `:1761-1776` (hors `:1773`, 15-13b) | mise à jour 0.13.0 — parties MariaDB | résolu (AC 11 f, g) |
| `admin-manual.tex:1793` (« Pour qui garde son fichier compose : deux gestes. » et « Sous `environment:` du service `kesh-api` : ») | décompte des gestes faux | résolu (AC 11 f, F-P3-1) |
| `admin-manual.tex:1747-1748` (`pull kesh-api`, `up -d kesh-api` de la procédure standard) | le service `mariadb` est-il recréé par un `up -d` nommant `kesh-api` ? | **assumé** — le § 0.13.0 prescrit `docker compose up -d` sans nom de service (déjà écrit) ; l'effet de la forme nommée est relevé au T7 |
| `admin-manual.tex:2192-2199` | dépannage | résolu (AC 11 j) ; `:2189` (`logs kesh`) laissé à la 15-7b3 |
| `admin-manual.tex:1578-1581` (pré-script Hyper Backup : `exec -T mariadb` et `grep MARIADB_ROOT_PASSWORD .env`) et `:573` (aperçu Container Manager listant `mariadb`) | **section Synology, dont le compose (`docker-compose.prod.yml`) n'a pas de service `mariadb`** | **assumé, NON résolu** — défaut antérieur à la story, **#575** (P3). Aggravé par l'AC 3 b : avec `#MARIADB_ROOT_PASSWORD=` commentée **et** une ligne active, `grep … \| cut -d= -f2` rend deux lignes — à joindre à #575 |
| `admin-manual.tex:1744` (« volume `kesh_db_data` ») et `:1422` (`/var/lib/docker/volumes/kesh_db_data`) | nom de volume faux (réel : `<projet>_kesh-mariadb-data`) | **assumé, NON résolu** — **#575** |
| `CHANGELOG.md:46` — quatre phrases (« deux gestes », « Kesh refuse de démarrer », « défaut sans avertissement », « `.env` copié du gabarit ») | rendues fausses | résolu (AC 13 c) |
| `CHANGELOG.md:48` | « après `git pull`, `docker compose up -d` tire … » (clone) | **assumé** — même section `[0.13.0]` que l'entrée #551, qui dit le refus de Compose et les mots de passe à poser (AC 13 a) |
| `CHANGELOG.md:52` | contrôle des placeholders « doit rester muette » | résolu (AC 13 d, F-P3-2) |
| `CHANGELOG.md` — rubrique **Retiré** de `[0.13.0]`, ligne `init-demo.sh` (créée par l'AC 13 b) | mention du retrait | résolu, **voulue** (exclue du grep de l'AC 12 b) |
| `CHANGELOG.md:653`, `:655` | entrées de versions antérieures (port 3000, `cargo run`) | **assumé** — historique, ne se réécrit pas |
| `docker-compose.yml` `kesh-api` `ports: "80:80"` (toutes interfaces) | port de l'application | **assumé** — hors #551 (accès LAN voulu sans reverse proxy) ; non traité |
| `admin-manual.tex:959` (`CREATE USER 'kesh'@'%' IDENTIFIED BY 'mot_de_passe_fort';`, initialisation manuelle hors Docker) | littéral imprimé dans le PDF distribué (F3 de la validation P4) | **assumé** — installation hors Docker, hors #551 ; non couvert par la liste fermée (§ *Angles morts*) |

**Décompte** : 48 lignes (recompté par `awk` sur le tableau — `grep -c '^| '` rend 49, en-tête compris ;
le séparateur commence par `|---` et n'est pas compté). 47 à la validation P4, plus la ligne `:959` (F3) ; les
sites de F1 (`docs/testing.md:171`, `:284`, `:302`, `CLAUDE.md:179`) enrichissent une ligne existante.

### Angles morts assumés (écrits, non traités)

- **Mot de passe root resté au gabarit** (`MARIADB_ROOT_PASSWORD=<mot de passe fort>` recopié de l'étape 3)
  : accepté par Compose et par MariaDB ; Kesh ne voit pas root (C71) — aucun avertissement possible. Seule
  la relecture des chevrons de `.env` prescrite par la procédure de mise à jour (AC 11 b) le montre. Le
  mot de passe **applicatif** gabarit, lui, est averti (AC 5 a-bis, C-15-13-17). Hérité de la 15-11a
  (F4-5), dont le propriétaire était « #551 » : #551 traite le **défaut publié** ; le root gabarit est
  tracé par l'issue **#578** (ouverte le 2026-10-09 ; § *Issue Tracking Rule* : pas de suivi local).
- **Mot de passe root** — `kesh_dev_root` recopié d'un ancien `.env`, ou valeur neuve écrite dans `.env`
  sans `ALTER USER` : invisible de Kesh et du healthcheck ; seuls le contrôle de fin de mise à jour du
  manuel (connexion à la base) et le lancement manuel du script de sauvegarde le montrent (AC 11 f,
  C-15-13-10). Le CHANGELOG le dit (AC 13 a).
- **Compose de Synology Container Manager** : `${VAR:?msg}` non mesurable depuis le poste ; la recette de
  la v0.13.0 sur le NAS le confirmera — mais le NAS de Guy suit l'installation Synology
  (`docker-compose.prod.yml`, sans MariaDB) : cette story ne le concerne pas.
- **`docker-compose.dev.yml`** : non contraint par le test (inchangé depuis la 15-11a) ; #554 ouverte pour
  son mot de passe admin.
- **Portée de la liste fermée `MOTS_DE_PASSE_PUBLIES`** (F3 de la validation P4) : elle ne couvre que les
  valeurs que Kesh a lui-même **distribuées comme défaut** d'une installation (`docker-compose.yml`,
  `.env.example`). D'autres littéraux du dépôt sont publiés au même titre sans y entrer : `kesh_root`
  (MariaDB de service de la CI, `ci.yml:64`, `:93`, `release.yml:70`, `:94` ; `frontend/DEBUGGING-KF007.md:16`)
  — runner éphémère, jamais un défaut d'installation ; `mot_de_passe_fort` (`admin-manual.tex:959`,
  initialisation hors Docker) — exemple d'une procédure manuelle, que Kesh n'a jamais posé de lui-même.
  Choix écrit, non traité.

### Règle de découpage

**Un crate, deux modules de code** — `kesh-api::config` et `kesh-api::main` —, plus des tests
(`configuration_transmise.rs` étendu, `demarrage_mariadb.rs` neuf, module d'aide `tests/common/binaire.rs`)
et des fichiers de configuration, de CI et de documentation (`docker-compose.yml`, `.env.example`,
`ci.yml`, `docs/ci.md`, `init-demo.sh` — **supprimé** —, manuel et brochure + PDF, `DOCKER_START.md`,
`CHANGELOG.md`). Convention de décompte (celle de la fiche unique) : le seuil compte les **modules de
code** au sens du `CLAUDE.md` ; tests, configuration, CI, scripts et documentation n'y entrent pas. Seuil
(« plus de 5 ») loin d'être atteint.

### Références

- Fiche index `15-13-mariadb-et-sauvegarde.md` (découpage, recompte, Change Log des validations P1 à P3) ;
  fiche unique complète au commit **8a9bcd27**.
- Issues #551, #577 (renvoi de `docs/ci.md`), #575 (section Synology, hors story), #554 ; #578 (root
  gabarit).
- Rapports de validation (non versionnés) : `target/gate-logs/15-13-p{1,2,3}-{R,F}.md` ; prompts
  `15-13-validate-prompt-p{1,2,3}.md`.
- `_bmad-output/implementation-artifacts/15-11a-compose-transmet-la-configuration.md` — § *Angles morts*
  (`#551`, F4-5), AC 12 (conventions du manuel : 76/70 caractères, `make fr`, contrôle aplati,
  `Overfull` des dix tableaux), AC 16 (`is_template_placeholder`, #557).
- `crates/kesh-api/tests/configuration_transmise.rs` — `VALEURS_COMPOSEES` (`:169-174`), `controle_valeur`
  (`:572-663`), test `valeurs` (`:1743`), `sortie_du_binaire_sans_configuration` (`:2232`).
- Documentation Docker : *Packet filtering and firewalls* — « Docker and ufw » (ports publiés hors UFW) ;
  image officielle `mariadb` — variables `MARIADB_*` lues à l'initialisation d'un datadir vide seulement.
- CLAUDE.md : § *Test Locally First* (E2E au dernier commit de code, D7), § *Propagation post-patch*, §
  *Recompter ses propres comptes rendus*, § *Le prompt d'une passe doit NOMMER le manuel*, § *Issue
  Tracking Rule* (mots-clés sur la PR).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, en autonomie — Epic 15), worktree `kesh-15-13a`, branche
`story/15-13a-mariadb-non-publiee`, base `200f5e79` + planification `08816686`.

### Debug Log References

Journaux non versionnés sous `target/gate-logs/` du worktree : `15-13a-t0/` (référence `make` d'avant :
`.log`, aplatis), `15-13a-t5/` (aplatis du neuf), `15-13a-mut-M*.txt` (23 mutations Rust),
`15-13a-gate-backend.txt`, `15-13a-front-*.txt`, `15-13a-e2e.txt`, `15-13a-e2e-backend.log`.
Mesures T0/T7 : journaux du T7 dans le répertoire de travail de la session (non versionnés), résumés
ci-dessous.

### Completion Notes List

**T0 — mesures** (détail au Change Log, entrée « T0 du développement »).
- Docker Compose **2.40.3**. Message `:?` : formes c1 et c2 lisibles, accents restitués → **c1 retenue**
  (valeurs non citées). Sans les variables : `config`, `pull`, `up -d`, `up -d kesh-api` refusent (code 1) ;
  `ps`, `logs`, `exec`, `stop`, `down` passent (code 0) — **écart à l'AC 11 f**, écrit selon la mesure
  (C-15-13a-1). Variable nommée **non déterministe** (12 lancements : `MARIADB_ROOT_PASSWORD` 5,
  `MARIADB_PASSWORD` 3, `DATABASE_URL` 4) — **écart à l'AC 11 j**, écrit « une à la fois, pas toujours la
  même ».
- `sqlx` : mauvais mot de passe → `Error::Database`, numéro **1045** joignable par
  `try_downcast_ref::<MySqlDatabaseError>()`, immédiat ; base sans droit → 1044. Pas de repli textuel.
- Test 8 : `kesh-15-13.invalid` échoue en 16 ms (retenu) ; `127.0.0.1:1` en 10 000 ms.
- `docker compose config` : forme développée des ports (`- mode: ingress` / `host_ip` / `target` /
  `published: "…"`) ; aucune entrée sous `mariadb` après la story.

**T7 — recette rejouée** (MariaDB 10.11.16 jetable ; d'abord sur un projet Compose jetable
`kesh1513a-t0`, puis — le démon Docker ayant figé ce conteneur lors d'une recréation sous une charge de 35,
`inspect`, `exec` et `kill` bloqués, aucun autre conteneur touché — sur des conteneurs `docker run`
jetables `kesh1513a-t7a/b`, réseau et volume propres ; tout supprimé à la fin, `kesh-mariadb-dev` jamais
touché) :
- comptes de l'image : `root@localhost`, `root@%`, `kesh@%`, `healthcheck@{127.0.0.1,::1,localhost}`,
  `mariadb.sys@localhost` ; `@@skip_name_resolve = 1` ;
- `CURRENT_USER()` : socket → `root@localhost` ; `--protocol=TCP -h 127.0.0.1` → `root@%` ;
  `-h mariadb` (alias réseau) → `root@%` ; `kesh` en TCP → `kesh@%` ;
- **socket avec un mauvais mot de passe root → 1045** : pas d'authentification `unix_socket`, d'où deux
  contrôles de root au manuel (socket et TCP) ;
- recette : `ALTER USER IF EXISTS` pour les deux root et `ALTER USER kesh`, en **un** document en ligne par
  `sh -c 'mariadb -uroot -p"$MARIADB_ROOT_PASSWORD"'` → code 0 ; `ps -eo args` de l'hôte pendant
  l'exécution : **aucune** ligne avec les nouveaux mots de passe hors le `grep` de contrôle ; après
  l'`ALTER`, avant recréation, la connexion par l'environnement du conteneur (ancien root) → **1045**
  (d'où un seul lot) ; `healthcheck.sh --connect --innodb_initialized` → 0 ;
- après recréation avec les nouvelles valeurs : contrôle de fin **dans la forme exacte du manuel**
  (coupée par `\`, y compris dans `sh -c '…'`) — trois anciens → `ERROR 1045`, trois de l'environnement →
  `root@localhost`, `root@%`, `kesh@%` ;
- sauvegarde puis restauration dans la forme du manuel (`sh -c`, `-T`) : table témoin créée, dumpée
  (présente dans le dump), supprimée, restaurée, relue (`1 avant-dump`) ; dump avec un mot de passe faux →
  code 2 sous `pipefail` (le script s'arrête) ;
- `up -d` après changement de `.env` : Compose décide **Recreate** pour `mariadb` ; `up -d kesh-api
  --dry-run` aussi (la forme nommée recrée la dépendance dont l'environnement a changé) ;
- `down` sans `.env` sur un projet en marche : code 0.

**Choix tranchés au fil du développement.**
- Forme du message : c1 (non citée). CHANGELOG `:52` (AC 13 d) : commande `config -q && echo 'compose
  lisible'` **recopiée** avant le `grep`, plutôt qu'un simple renvoi (l'entrée #557 recopiait déjà le
  `grep` : le renvoi seul l'aurait laissé piégeux). CHANGELOG `:46` : « les gestes, fichier par fichier »
  (**renvoi sans nombre**, insensible à l'ordre de merge 15-13a/15-13b).
- Étape CI renforcée (C-15-13a-2) : M27 tel qu'écrit **survivait** à l'esquisse de la fiche.
- Manuel : le contrôle de fin couvre `root@localhost` (socket) **et** `root@%` (TCP), mesure T7 ; la
  recette change les trois comptes en un lot (après l'`ALTER` de root, une seconde connexion par
  l'ancienne valeur échoue) ; `ALTER USER IF EXISTS` pour root (comptes listés d'abord).
- Script de sauvegarde : `COMPOSE_DIR="/opt/kesh"` (répertoire d'installation de l'étape 1 du manuel) ;
  encadré « lancer une fois à la main » ajouté sous la crontab.
- `DOCKER_START.md:12-13` réécrit (Compose d'abord), `:146` → `kesh-mariadb-data` /
  `<projet>_kesh-mariadb-data`.
- Contrôles de l'AC 11 l validés sur l'aplati neuf : apostrophe et double tiret sortent **droits** en
  `lstlisting` (`MARIADB_ROOT_PASSWORD"'` ×3, `--protocol=TCP` ×5 dont un `\texttt{-{}-…}` de prose) ;
  aucun ajustement de motif.

**Mutations (T8)** — 24 jouées, **24 rouges**, chacune sur le test visé et pour le motif voulu (aucun
échec de compilation) ; fichier restauré par `git checkout` puis `touch` (banc scripté) :

| Mut. | Test rouge |
|---|---|
| M1, M2, M3, M4, M28 | `configuration_transmise::mariadb` |
| M5 | `configuration_transmise::valeurs` |
| M6, M7, M8 | `configuration_transmise::mots_de_passe_publies` |
| M12, M13, M29 | `configuration_transmise::s_service_mariadb` |
| M16, M17, M18, M19, M42, M43 | `config::mot_de_passe_base_tests::from_env_warns_on_published_database_password` (M43 : « le journal cite « kesh_dev » ») |
| M20 | `demarrage_mariadb::mot_de_passe_publie_avertit_au_demarrage` |
| M21, M22 | `config::mot_de_passe_base_tests::indice_connexion_refusee` |
| M23, M44 | `demarrage_mariadb::mauvais_mot_de_passe_donne_l_indice` (M44 : « ne doit pas citer le mot de passe ») |
| M27 | étape CI « Validate compose files », rejouée sous `bash -eo pipefail` sur une copie : rouge (après renforcement, C-15-13a-2) ; le retour du seul défaut `MARIADB_PASSWORD` du service reste vert à dessein (la `DATABASE_URL` l'exige encore ; test Rust `mariadb` rouge) |

**Gates (T10)**, au commit `e38d96cf` (dernier commit de code : `38428924`, les deux suivants sont de la
documentation), bases `kesh_1513a` / `kesh_e2e_1513a` remises à zéro (`DROP`/`CREATE`, migrations, seed)
avant chacun :
- `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest) : **2980 / 2980**, 4 ignorés (2973 sur
  `main` + 7 fonctions neuves) ;
- frontend : `npm run check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership` vert,
  Vitest **1095 / 1095**, `npm run build` vert (frontend non touché) ;
- E2E complet (backend `target/debug/kesh-api` du worktree sur le port 3017, base `kesh_e2e_1513a`,
  `KESH_TEST_MODE=true` des deux côtés, `KESH_COOKIE_SECURE=false`, SMTP factices, répertoires
  `target/e2e/{inbox,documents}` ; `/health` → `smtpConfigured:true` ; lancé à 05:52 UTC) : **245 passés,
  9 échoués**, 19 ignorés, 10,5 min — les 9 sont la liste attendue de `docs/testing.md` : KF-029 ×7
  (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et KF-045 ×2
  (`invoices:415`, `:439`, avant midi UTC) ; aucun hors liste, aucune pollution. Le journal de ce backend
  porte une fois l'avertissement « mot de passe publié » (`DATABASE_URL` sur `kesh_dev`) : AC 5 observé en
  conditions réelles. Backend arrêté par son PID ;
- manuels : `make fr` ; `Overfull \hbox` du manuel d'administration **55 → 55**, aucun nouveau (comparés
  par texte de ligne source au `.log` d'avant ; 7 nouveaux au premier passage, résorbés par `sloppypar`) ;
  brochure 4 → 4 ; aucun renvoi indéfini ; `user-manual.pdf` restauré (non touché, seuls des numéros de
  page de la table différaient).
- Contrôles aplatis (AC 11 l, m) — avant → après : `openssl rand -hex 32 # → mot de passe MariaDB` 0 → 1 ;
  `MARIADB_ROOT_PASSWORD"'` 0 → 3 ; `--protocol=TCP` 0 → 5 ; `CURRENT_USER()` 0 → 6 ; `compose lisible`
  0 → 2 ; `exec (-T )?db` 3 → **0** ; `deux gestes` 1 → **0** ; brochure `MARIADB_ROOT_PASSWORD` 0 → 1.
- Étape CI : rejouée localement (nominal vert, M27 rouge) ; le job `docker-build` lui-même n'a pas tourné.

**T9 — propagation et inventaire** : greps (1) à (4) rejoués à `e38d96cf`. Aucun site hors inventaire.
Sites **neufs** créés par la story, voulus : constantes et témoins de test (`config.rs`
`MOTS_DE_PASSE_PUBLIES` et test 7, `configuration_transmise.rs` (M), `demarrage_mariadb.rs:25`), les
commentaires `3306` de `docker-compose.yml` (forme loopback à décommenter), `docs/ci.md` (`MARIADB_` de la
validation), les mentions du manuel et du CHANGELOG (anciens défauts écrits **au manuel seulement**,
contrôle de fin). `admin-manual.tex` porte désormais 23 lignes `docker compose up` (21 avant) : les deux
neuves sont dans le § 0.13.0 et le § *Changer un mot de passe MariaDB*, après la pose des mots de passe.
`git grep -n init-demo -- . ':(exclude)_bmad-output' ':(exclude)CHANGELOG.md'` → vide.

**À l'orchestrateur.**
- Conteneur figé du T7 : supprimé ensuite (`docker rm -f` a fini par aboutir) ; plus aucune ressource
  `kesh1513a` (conteneur, volume, réseau). Le démon a été lent tout le long (création d'un conteneur : 49 s).
- `docs/kesh-specifications.txt` (NFR-DEPLOY-1, « une seule commande `docker-compose up` ») : faux depuis la
  15-11a, davantage ici — signalé, non réécrit (inventaire).
- `CLAUDE.md` § E2E annonce encore un smoke E2E en CI : #577.

### File List

- `docker-compose.yml` — port de `mariadb` retiré (commentaire, forme loopback), trois `:?`.
- `.env.example` — bloc MariaDB réécrit, mots de passe commentés ; commentaire `DATABASE_URL` (hex).
- `crates/kesh-api/Cargo.toml` — `url`, `percent-encoding` en dépendances directes (déjà au `Cargo.lock`).
- `crates/kesh-api/src/config.rs` — `MOTS_DE_PASSE_PUBLIES`, avertissements, `numero_erreur_mariadb`,
  `indice_connexion`, doc d'`is_template_placeholder` ; tests 7 et 9.
- `crates/kesh-api/src/main.rs` — indice dans le message d'échec de connexion.
- `crates/kesh-api/tests/configuration_transmise.rs` — `service(source, nom)`, `ports`, scission
  d'`Interpolation::Obligatoire`, `VALEURS_COMPOSEES` resserré, contrôle (M), tests 1, 2, 4, en-tête ;
  `sortie_du_binaire_sans_configuration` sur `lancer_binaire`.
- `crates/kesh-api/tests/common/binaire.rs` (neuf) — `lancer_binaire`, `texte`.
- `crates/kesh-api/tests/demarrage_mariadb.rs` (neuf) — tests 8 et 10.
- `.github/workflows/ci.yml` — étape « Validate compose files ».
- `docs/ci.md` — 3 jobs, renvoi à #577, deux sens du contrôle.
- `docs/manual/fr/admin-manual.tex` + `.pdf` — AC 11 a–d, f–h, j, k.
- `docs/manual/fr/marketing-brochure.tex` + `.pdf` — AC 11 m.
- `DOCKER_START.md` — AC 12 a, a-bis.
- `init-demo.sh` — **supprimé** (AC 12 b).
- `CHANGELOG.md` — `[0.13.0]` : Sécurité #551, Retiré, propagation `:46` et `:52`.
- `_bmad-output/implementation-artifacts/sprint-status.yaml`, `epic-15-choix-autonomes.md` (C-15-13a-1, -2),
  cette fiche.

## Change Log

- 2026-10-09 — **Création par découpage de la 15-13, et remédiation de la validation P3** (agent remédiateur,
  Opus 5.5, en autonomie). Historique de la fiche unique (spécification, validations P1 et P2) : Change Log
  de la fiche index. **Trend** de la fiche unique : P1 (Opus 5.5 ×2) 8 MEDIUM distincts → P2 (Sonnet ×2) 6
  → P3 (Opus 5.5 ×2 : R 2 MEDIUM / 8 LOW, F 3 MEDIUM / 8 LOW, aucun doublon) **5 MEDIUM distincts** ; 0
  CRITICAL/HIGH aux trois passes. **Signal D5 de recyclage** levé deux passes de suite (P2 : deux MEDIUM nés
  de la P1 ; P3 : R3-1, R3-2 et F-P3-2 nés de la P2) → **découpage** décidé par l'orchestrateur
  (C-15-13-16) selon la couture #551 / #552-#576.
  - **Reçu de la 15-13** : AC 1 à 6, 10 a/b/d/e (partie MariaDB), 11 a/b/c/d (note MariaDB)/f (paragraphe
    MariaDB)/g/h/j/k/l (contrôles MariaDB)/m, 12 a/a-bis/b, 13 a/b (Retiré)/c (phrases MariaDB), 14 ; T0
    (mesures Compose, `sqlx`, hôte du test 8, `make fr`), T1 (compose, gabarit), T2 (`config`, `main`), T3,
    T4, T5, T6 (`DOCKER_START.md`, `init-demo.sh`, CHANGELOG), T7, T8, T9, T10 ; tests 1, 2, 4, 7, 8, 9,
    10, 13 ; mutations M1–M8, M12, M13, M16–M23, M27–M29 ; dix lignes du tableau des cas (deux scindées) ;
    quatre angles morts.
  - **Remédiation P3 appliquée ici** : **R3-2** (AC 12 b : grep qui exclut `CHANGELOG.md`, ligne de
    CHANGELOG à l'inventaire) ; **F-P3-1** (AC 11 f et 13 c : « deux gestes » → décompte par fichier, cinq
    phrases à `CHANGELOG.md:46` dont quatre ici ; contrôle négatif au PDF) ; **F-P3-2** (AC 13 d,
    `CHANGELOG.md:52`, inventaire, grep (4) du T9) ; **F-P3-3** (décision de l'orchestrateur, C-15-13-17 :
    AC 5 a-bis et d, test 7 étendu, **M42**, AC 11 b, 13 a, tableau des cas, Dev Notes rectifiées, angle
    mort root gabarit tracé par issue) ; LOW **R3-3/F-P3-4** (#577 au T4, à l'inventaire, aux Issues),
    **R3-4** (`docs/ci.md:9-10`), **R3-5** (`:210`), **R3-6** (sites du grep par la valeur triés :
    `docs/kesh-specifications.txt`, `docs/testing.md:99`, `:161`, `DOCKER_START.md:41`, `:137`),
    **R3-7** (`init-demo.sh:70-75`), **R3-8** (test 10 : `KESH_JWT_SECRET`, assertion « Erreur de
    configuration »), **F-P3-6** (grep par la valeur sur tout le dépôt : `CHANGELOG.md`, `.env.example`,
    commentaires des compose triés), **F-P3-8** (`DOCKER_START.md:12-13`), **F-P3-9** (helper sans
    assertion, code ≠ 0 aux tests 8 et 10).
  - **Constats du remédiateur** : M5 et le resserrement de `VALEURS_COMPOSEES` relevaient du test
    `valeurs` (famille (V)), non de `transmission` ni du test 1 — **ligne 3a** neuve, AC 10 a précisé ;
    grep `MARIADB_` (axe non exercé par les deux lentilles P3) rejoué : `retry.rs:57`, `:106` et
    `reconciliation.rs:1129`, `:1136` ajoutés (homonymes) ; `crates/kesh-db/README.md:34` ajouté.
  - **Recompte** (depuis cette fiche) : **11 AC** (numéros 1–6, 10–14 de la fiche unique), **11 tâches**
    (T0–T10), **9 lignes de test** portant **8 fonctions Rust** (7 neuves, 1 existante modifiée) + 1 étape
    CI ; **22 mutations** (M1–M8, M12, M13, M16–M23, M27–M29, M42) ; **10 lignes** au tableau des cas ;
    **47 lignes** d'inventaire ; **4 angles morts** ; **2 modules de code**.
- 2026-10-09 — **Remédiation de la validation P4** (agent remédiateur, Opus 5.5, en autonomie). Passe P4 :
  deux lentilles **Sonnet** en contexte frais (prompt `15-13a-validate-prompt-p4.md`) — R (regression
  hunter) **0 CRITICAL / 0 HIGH / 1 MEDIUM / 7 LOW**, F (adversaire plein périmètre) **0 / 0 / 0 / 7 LOW**
  (dont une vérification positive, F5) ; R4-1 = F1 (même défaut, vu MEDIUM par R, LOW par F) — soit
  **1 MEDIUM distinct**. **Trend** (fiche unique puis 15-13a) : P1 (Opus 5.5 ×2) 8 MEDIUM distincts → P2
  (Sonnet ×2) 6 → P3 (Opus 5.5 ×2) 5 → découpage → P4 (Sonnet ×2) **1** ; 0 CRITICAL/HIGH aux quatre
  passes. Le MEDIUM a deux moitiés : un trou d'inventaire d'**origine** (les quatre recettes de dev
  rendues par `3306|kesh_dev`, jetons présents dès la fiche unique) et un jeton trop large **né de la
  remédiation P3** (`deux gestes`, ajouté au grep au commit `a847f369` pour F-P3-1 — vérifié par
  `git log -S`). Sévérité en baisse (5 → 1), aucun défaut de code : la boucle converge ; le résidu né de la
  P3 est signalé à l'orchestrateur.
  - **MEDIUM R4-1/F1** : inventaire complété (`docs/testing.md:171`, `:284`, `:302`, `CLAUDE.md:179`, dans
    la ligne des recettes de dev, assumées) ; jeton `deux gestes` du T9 (1) restreint à
    `(compose|décrit les)[^.]{0,20}deux gestes`, les six homonymes nommés ; grep (1) rejoué entier à
    `c702b7d5` : aucun site hors inventaire.
  - **LOW** : **F2** (`admin-manual.tex:227` est de la prose ; métrique « 21 » précisée : lignes du `.tex`,
    autant au PDF aplati) ; **F3** (ligne d'inventaire `:959` ; angle mort « portée de la liste fermée »,
    `kesh_root` trié) ; **F4** (`decode_utf8_lossy()` tranché, témoin `%FF` au test 7, C-15-13-22) ;
    **R4-4/F5** (forme non encodée établie à la source : cas positif ferme, repli retiré) ; **F6** (**M43**
    au test 7, **M44** au test 10 avec l'assertion « pas de `mauvais-15-13` ») ; **F7** (recette de
    changement : nouveau mot de passe ni dans l'historique ni dans `ps`, `stop kesh-api` d'abord, forme de
    référence en Dev Notes, C-15-13-23 ; rubrique « Retiré » justifiée par Keep a Changelog 1.1.0,
    vérifié en ligne) ; **R4-2** (`configuration_transmise.rs:626` nommé) ; **R4-3** (PDF versionnés :
    régénérer après fusion, pas rebaser) ; **R4-5** (message « placeholder de gabarit », `generate_me`) ;
    **R4-6** (en-tête des choix aligné sur le corps, citations ajoutées aux AC 2 c, 4 d, 11 f) ; **R4-7**
    (fiche index : ventilation de l'inventaire retirée, totaux seuls).
  - **Règle changée** : aucune règle métier. Précisions de contrat : décodage `lossy` (AC 5 a), propriété
    « ni mot de passe ni URL » désormais gardée par mutation, exigences d'hygiène de la recette (AC 11 g).
  - **Recompte** (depuis cette fiche, `grep`) : **11 AC**, **11 tâches**, **9 lignes de test** portant
    **8 fonctions Rust** (7 neuves, 1 existante modifiée) + 1 étape CI ; **24 mutations** (M1–M8, M12, M13,
    M16–M23, M27–M29, M42–M44 ; `sort -u` → 24) ; **10 lignes** au tableau des cas ; **48 lignes**
    d'inventaire ; **5 angles morts** ; **2 modules de code** (inchangé).

- **2026-10-09 — validation P5 ciblée (Haiku, prompt `15-13a-validate-prompt-p5-ciblee.md`)** : rapport
  `target/gate-logs/15-13a-p5-ciblee.md`, **0 finding**, cinq axes déclarés exercés. Axe repris par
  l'orchestrateur : le grep restreint du T9 (1) rejoué ne rend que `CHANGELOG.md:46` et
  `admin-manual.tex:1793`, conformément à la fiche ; la recette `ALTER USER` passe bien par
  `docker compose exec -T mariadb sh -c`. **Validation close.** Trend (15-13 puis 15-13a) : P1 8 MEDIUM →
  P2 6 → P3 5 (découpage, C-15-13-16) → P4 1 → P5 ciblée 0. Modèles : Opus ×2, Sonnet ×2, Opus ×2,
  Sonnet ×2, Haiku (ciblée). Signal D5 déclaré en P2 et P3 (recyclage), suivi d'un découpage en P3.

- **2026-10-09 — T0 du développement** (agent de développement, Opus 5.5, worktree `kesh-15-13a`, base
  `200f5e79` + planification `08816686`). Fiche relue contre le code : les numéros de `config.rs`, `main.rs`,
  `configuration_transmise.rs`, `docker-compose.yml`, `.env.example` et `ci.yml` sont en place ; ceux du
  manuel sont décalés de **+3** après la 15-7a2 à partir du script de sauvegarde (`exec -T db` `:1463`,
  `:1514` ; `exec db` `:2219` ; « deux gestes » `:1796` ; référence des ports `:2365`) — relocalisés par le
  texte. `sprint-status.yaml` : 15-13 (`split`), 15-13a (`in-progress`), 15-13b (`ready-for-dev`) ajoutées
  (absentes de `main`). Mesures (détail au Dev Agent Record) :
  - **Message `:?`** (Docker Compose **2.40.3**) : les deux formes c1 (non citée) et c2 (citée) se lisent
    avec les variables posées (`config -q` → 0) ; sans elles, code 1 et message restitué **avec ses
    accents**. **Forme retenue : c1** (valeurs non citées, comme les lignes voisines).
  - **Écart à la fiche — ordre** : la variable nommée **n'est pas stable** : sur 12 lancements,
    `MARIADB_ROOT_PASSWORD` 5, `MARIADB_PASSWORD` 3, `DATABASE_URL` de `kesh-api` 4 (forme c1). L'AC 11 j
    (« corriger la première, la seconde apparaît au lancement suivant ») est écrit comme : une variable à
    la fois, pas toujours la même — poser les deux (C-15-13a-1).
  - **Écart à la fiche — sous-commandes** : sur un projet jetable en marche, sans les variables,
    `config`, `pull`, `up -d` refusent (code 1) ; `ps`, `logs`, `exec -T mariadb true`, `stop` **passent**
    (code 0). L'AC 11 f (« le refus frappe toute sous-commande », « le script de sauvegarde lancé par
    `cron` échoue ») est donc écrit selon la mesure (C-15-13a-1). Aucune règle ni AC changé sur le fond :
    le manuel dit vrai, ce que l'AC demandait.
  - **Graphie de `docker compose config`** : un service avec `ports` donne la forme longue
    (`ports:` / `- mode: ingress` / `host_ip:` / `target:` / `published: "…"`) ; le service `mariadb`
    sans `ports` n'en montre aucune (`grep -c ports` sur son bloc → 0).
  - **`sqlx`, mauvais mot de passe** (MariaDB 10.11.16 jetable) : `connect` rend immédiatement (0 ms)
    `Database(MySqlDatabaseError { code: Some("28000"), number: 1045, … "(using password: YES)" })` ;
    l'extraction `as_database_error()?.try_downcast_ref::<MySqlDatabaseError>()?.number()` rend
    `Some(1045)`. Base absente pour un compte sans droit : **1044**. Pas de repli textuel nécessaire.
  - **Hôte du test 8** : `kesh-15-13.invalid` échoue en **16 ms** (`Io`, résolution) ; `127.0.0.1:1`
    attend **10 000 ms** (`PoolTimedOut`). Retenu : `kesh-15-13.invalid`.
  - **`make admin brochure` d'avant** : référence `target/gate-logs/15-13a-t0/` (`.log` : 55 lignes
    `Overfull` au manuel d'administration ; aplatis). Contrôles de l'AC 11 l sur l'aplati d'avant :
    `openssl rand -hex 32 # → mot de passe MariaDB` 0, `MARIADB_ROOT_PASSWORD"'` 0, `--protocol=TCP` 0,
    `CURRENT_USER()` 0, `compose lisible` 0 ; `deux gestes` 1 ; `exec (-T )?db` 3 ; brochure `MARIADB` 0.
  - **`.env` local** : absent du worktree (aucune mesure faussée).
