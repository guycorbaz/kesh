# Story 15.11b : Une seule fonction lit l'environnement — le vide vaut l'absence — et le test qui lit le code remplace la liste écrite à la main

Status: in-progress

<!-- Créée le 2026-10-08 par l'agent de découpage, en autonomie (consignes de l'Epic 15), par découpage
     de la Story 15-11 après sa validation P3 (signal D5 levé deux fois, par recyclage — choix C77).
     Elle reprend l'AC15 de la 15-11 (fonction de lecture unique), les règles (L) et le volet « code » de
     (F) de son test, et la remédiation P3 qui y tombe (F-2, F-3 = R3-3, R3-6, R3-7, F-9). Choix
     applicables : C72 (test Rust, `syn`), C75 (fonction de lecture unique, vide = absent, trim), C76
     (`dotenvy` et macros vus par le test), C77 (découpage), C78 (validation P1 : le test devient
     LEXICAL), C80 (validation P2 : appels qualifiés, API de `tracing-subscriber` surveillée).
     Version complète de la 15-11 avant découpage : commit d69fdcca.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation. -->

**Issues** : **`refs #550`** (sans mot-clé de fermeture : c'est la 15-11a qui ferme #550).

**Dépendances, ordre écrit** :
1. **15-11a mergée** (`15-11a-compose-transmet-la-configuration.md`) — elle pose le test
   `configuration_transmise.rs` (familles T, V, E, F sur les corpus texte, S) avec une liste fermée `LUES`
   écrite en dur ; la 15-11b **remplace** `LUES` par la lecture du code, et étend (F) au code. Elle retire
   aussi le fantôme `KESH_ADMIN_RESET` du code : le (F) étendu de la 15-11b part d'un code propre.
2. **15-5e1 mergée** (`15-5e1-socle-rejeu.md`) — elle ajoute `syn` aux `[dev-dependencies]` de `kesh-api`
   (C70). **Un seul `syn`** : la 15-11b réutilise cette ligne (union des features : `full`, `visit` au
   moins), et ajoute `proc-macro2` (feature `span-locations`). `Cargo.lock` repris de `main` puis régénéré
   par `cargo check -p kesh-api --tests`, jamais fusionné à la main.
3. **Avant le tag v0.13.0** (souhaité). Non bloquant pour la sûreté (aucun refus de démarrer). *(Alignement
   sur le livré, T0 : la revue de code P1 de la 15-11a — C-15-11a-6 — a déjà supprimé les avertissements
   « invalide » et la sauvegarde pré-import dans `/app` pour les sept variables qu'elle transmet en
   `${NOM:-}` ; ce motif d'urgence ne vaut plus. Reste : le vide et le trim de toutes les autres.)*

## Story

En tant que **mainteneur de Kesh**,
je veux qu'**une seule fonction lise l'environnement, avec une seule règle — une valeur vide ou faite
d'espaces vaut une absence, et la valeur lue est débarrassée de ses espaces —**, et que le test garde-fou
**lise le code** pour savoir quelles variables Kesh lit,
afin qu'**une ligne laissée vide dans `.env` produise toujours le défaut**, quel que soit le lecteur, et
qu'**une variable ajoutée au code sans être transmise par les compose fasse rougir le test** sans que
personne ait à tenir une liste à la main.

## Pourquoi (C75)

Les compose transmettent les ajouts sous la forme `${NOM:-}` (15-11a) : une variable absente de `.env`
arrive **vide**. Et une ligne laissée vide dans `.env` arrive vide **quelle que soit** la forme du compose
(mesuré, Compose 2.40.3). La question du vide ne se règle donc pas côté compose. Or chaque lecteur a
aujourd'hui sa règle : `opt_trimmed_env`, `parse_strict_bool`, `env_flag_enabled` et les variables
d'administration traitent le vide comme absent ; `KESH_SMTP_PORT` avertit « invalide » puis prend 587,
`KESH_ADMIN_BACKUP_DIR` prend un chemin **vide**, `KESH_LANG` passe `""` à `Locale::from`, `DATABASE_URL`
vide échoue plus tard à la connexion. *(T0 : depuis C-15-11a-6, `KESH_SMTP_PORT`, `KESH_ADMIN_BACKUP_DIR`,
`KESH_LANG` et les quatre autres numériques d'administration traitent déjà le vide comme absent ; restent
`KESH_HOST`, `KESH_PORT`, `DATABASE_URL`, `KESH_JWT_SECRET`, `KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR`, les
variables JWT, de session, de limitation, d'inbox et de journal, `KESH_STATIC_DIR`, `KESH_LOCALES_DIR`,
`RUST_LOG`.)* Une seule fonction, imposée par le test, règle la question une fois :
**vide = absent**, partout.

## Inventaire des sites de lecture — refait depuis le code le 2026-10-08

> **Recompte du T0 (2026-10-09, `HEAD` = `b2b09f34`, sur `8f9811d8` qui porte la 15-11a)** : la 15-11a a
> fait passer `KESH_ADMIN_BACKUP_DIR` et `KESH_LANG` de `env::var` à `opt_trimmed_env`. D'où : **34 sites**
> (et non 36) — `config.rs` **30** (**24** lectures littérales dans `Config::from_env`, 4 dans
> `LogConfig::from_env`, 2 des helpers), `main.rs` 2, `logging.rs` 1, `routes/onboarding.rs` 1 ; appels
> `opt_trimmed_env("…")` **7** (et non 5) ; identifiant `env` en production **38** (`config.rs` **33**) ;
> ensemble lu **41**, inchangé ; appels `env_nonempty` littéraux de `Config::from_env` après le T2 :
> 24 + 7 = **31**, inchangé. Les chiffres du paragraphe suivant sont ceux d'avant la 15-11a, conservés
> pour la trace.

**36 sites de lecture** en production (`grep -cE 'env::var(_os)?\(' ` par fichier, hors items
`#[cfg(test)]` — les huit de `kesh-db/src` sont tous dans des `mod tests`) : `config.rs` **32** (26 lectures
littérales dans `Config::from_env`, 4 dans `LogConfig::from_env` `:1340-1343`, les 2 des helpers
`opt_trimmed_env` `:1368` et `parse_strict_bool` `:1396`), `main.rs` **2** (`:220` `KESH_LOCALES_DIR`,
`:247` `KESH_STATIC_DIR`), `logging.rs` **1** (`:130`), `routes/onboarding.rs` **1** (`:46`). Quatre
**indirections** à argument non littéral, chacune résolue par ses appels littéraux :

| site non résolu | résolu par |
|---|---|
| `config.rs` `opt_trimmed_env(var)` (`:1367`) | ses appels `opt_trimmed_env("KESH_…")` (5 noms) |
| `config.rs` `parse_strict_bool(var, …)` (`:1395`) | ses appels `parse_strict_bool("KESH_…", …)` (2 noms) |
| `routes/onboarding.rs` `env_flag_enabled(name)` (`:45`) | son appel `env_flag_enabled("KESH_PRODUCTION_RESET")` (`:276`, dans `reset`) |
| `logging.rs:130` `std::env::var(EnvFilter::DEFAULT_ENV)` | constante de `tracing-subscriber` : `RUST_LOG` |

Ensemble lu : **41** noms = 32 arguments littéraux de sites directs (26 + 4 + 2) + 2 + 1 + 5 + `RUST_LOG` —
identique à la liste `LUES` de la 15-11a. **Après la story, un seul site** : celui de
`config::env_nonempty` ; les 35 autres passent par elle (`opt_trimmed_env` disparaît, absorbée).

**Vue lexicale** (celle du test, AC3 — relevé par `grep -w` hors commentaires et hors `mod tests`, à
refaire au T0 par le test lui-même) : l'identifiant `env` (hors `env!`) apparaît **40** fois en production
— `config.rs` **35** (les 32 lectures, `use std::env;` `:7`, `env::VarError::NotUnicode` et
`env::VarError::NotPresent` d'`opt_trimmed_env`), `main.rs` 2, `logging.rs` 1, `routes/onboarding.rs` 1,
`routes/admin.rs` **1** (`:80`, `std::env::temp_dir()` dans `stream_via_tempfile`, qui lit `TMPDIR`) ;
`dotenvy` **2** fois (`main.rs:44` dans `main`, `config.rs:566` dans `Config::from_env`, sous
`#[cfg(not(test))]`) ; aucune occurrence de `from_default_env` / `try_from_default_env`, `Builder`,
`with_env_var`, `from_env_lossy`, `try_from_env` ni `try_init` ; l'identifiant **`EnvFilter`** apparaît
**4** fois, toutes dans `logging.rs` (`use` `:23`, type de retour de `build_log_filter` `:104`,
`EnvFilter::new(raw)` `:105`, `EnvFilter::DEFAULT_ENV` `:130` — le `[`EnvFilter`]` du doc-comment `:96`
est un littéral, non un identifiant), et l'identifiant **`init`** **1** fois (`.init()` `logging.rs:162`,
dans `init_tracing` ; les autres « init » du code sont dans des chaînes). Les `env!(…)`
(version, `CARGO_MANIFEST_DIR`) sont des macros de compilation, non des lectures, et ne sont pas des
occurrences.

## Acceptance Criteria

1. **Inventaire.** Le § *Inventaire des sites* est refait au T0 depuis `main` à jour (15-11a et 15-5e1
   mergées) et reproduit au Dev Agent Record **tel que le test le recalcule** (AC3) au dernier commit de
   code : **1 site de lecture** (`config::env_nonempty`), la liste `EMPLACEMENTS_AUTORISES` (AC3) avec ses
   nombres, ensemble lu de **41** noms, **égal** à la liste `LUES` de la 15-11a au moment où elle est
   retirée (T1 : l'égalité est assertée, l'écart éventuel écrit avant que la liste disparaisse). Tout écart
   avec cette fiche est écrit, non corrigé en silence.

2. **Une fonction de lecture unique — le vide vaut l'absence** (C75). `crates/kesh-api/src/config.rs`
   expose `pub fn env_nonempty(name: &str) -> Option<String>` (documentée `///` : contrat, invariant
   `Some(s) ⟹ !s.is_empty()`, valeur **trimée** ; **aucun jeton `KESH_` fictif** dans ce doc-comment — un
   nom réel ou aucun : le (F) étendu de l'AC3 lit les `///`) : variable absente, vide ou faite d'espaces →
   `None` ; non-UTF-8 → `None` avec l'avertissement actuel d'`opt_trimmed_env` (qu'elle absorbe :
   `opt_trimmed_env` disparaît, ses appels deviennent `env_nonempty`). Lecture recommandée par
   `std::env::var_os(name)` puis `into_string()` (une seule occurrence de `env` dans son corps, sans
   `VarError`), et retrait du `use std::env;` de production de `config.rs` (le module de test, qui s'en
   sert, reçoit son propre `use std::env;`) ; la forme retenue fixe les fenêtres de l'AC3, relevées au T3.
   **Limite écrite** (R2-6, F7 de la P2) : cet avertissement est un `tracing::warn!` ; pour les lecteurs qui
   précèdent l'installation de l'abonné — `LogConfig::from_env` (`main.rs:56`, les quatre
   `KESH_LOG_FILE_*`) et la lecture de `RUST_LOG` dans `init_tracing` (`logging.rs:130`) —, il est
   **perdu** ; le contrat doc-commenté le dit (« avertissement émis par `tracing`, perdu s'il précède
   l'abonné ») ; le module journal, qui collecte ses avertissements pour les rejouer
   (`config.rs:1331-1334`), n'est **pas** étendu au non-UTF-8 (angle mort).
   **Tous les lecteurs de production passent par elle** — inventaire fermé : les **33** autres sites
   (`config.rs` 29 : 24 de `Config::from_env`, 4 de `LogConfig::from_env`, celui de `parse_strict_bool` ;
   `main.rs` 2 ; `logging.rs` 1 — `RUST_LOG`, par `EnvFilter::DEFAULT_ENV` — ; `routes/onboarding.rs` 1 —
   `env_flag_enabled`) —, **sans exception**. **Hors de `config.rs`, l'appel s'écrit par chemin qualifié,
   sans aucun `use` d'`env_nonempty`** (R-1 = F-2 de la P2) : `kesh_api::config::env_nonempty(…)` dans
   `main.rs` (crate binaire : il n'a pas de module `config`, il nomme la bibliothèque `kesh_api`),
   `crate::config::env_nonempty(…)` dans `logging.rs` et `routes/onboarding.rs`. Un
   `use crate::config::env_nonempty;` serait une occurrence du jeton à l'emplacement `<hors fonction>`,
   qu'aucune entrée de l'AC3 n'autorise : **rouge**, voulu. Le chemin qualifié laisse la fenêtre
   commencer à `env_nonempty` (AC3) : les entrées de la table n'en dépendent pas. Ainsi : le seul « vide signifiant », `KESH_LOG_FILE_PATH`, a dans le
   code le même sens que l'absence (pas de journal fichier, `LogConfig::from_raw` filtre déjà le vide — test
   `config.rs:2315`) ; sa distinction vit côté compose (15-11a, AC2 ii).
   **Changements de comportement, à écrire au Dev Agent Record lecteur par lecteur** (relevé au T0 —
   *pour `KESH_SMTP_PORT`, les quatre numériques d'administration, `KESH_ADMIN_BACKUP_DIR` et `KESH_LANG`,
   l'« avant » du vide est désormais « défaut sans avertissement » (C-15-11a-6) : la 15-11b n'y change plus
   que le trim d'une valeur non blanche des cinq numériques — `KESH_LANG` et `KESH_ADMIN_BACKUP_DIR` sont
   déjà trimées*) :
   pour chaque variable, ce que produisait une valeur vide (ou à bords blancs) avant et ce qu'elle produit
   après — la table nomme **au moins** : `RUST_LOG=""` (avant : erreurs seules — `EnvFilter::new("")` ne
   pose aucune directive et garde le défaut `ERROR` — `tracing-subscriber` 0.3.23 : `EnvFilter::new` vaut
   `with_default_directive(ERROR).parse_lossy(…)` (`filter/env/mod.rs:350-353`), et `parse_lossy`
   (`filter/env/builder.rs:146-158`) écarte les directives vides (`.filter(|s| !s.is_empty())`) —, plus `sqlx=warn` ajouté par `build_log_filter` `logging.rs:104-120` ; après :
   `info`), `KESH_HOST=""` (avant : adresse d'écoute `":80"`, échec au démarrage ; après : `127.0.0.1`),
   `KESH_PORT=""` (avant : avertissement puis 80 ; après : 80 sans avertissement), `KESH_STATIC_DIR=""` /
   `KESH_LOCALES_DIR=""` hors Docker (avant : chemin vide ; après : défaut), `KESH_JWT_SECRET=""` (avant :
   `WeakJwtSecret{0}` ; après : `MissingVar`), `KESH_SMTP_PORT` (avant : avertissement puis 587 ; après :
   587 sans avertissement), `KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB`,
   `KESH_ADMIN_EXPORT_INMEM_MB`, `KESH_ADMIN_IMPORT_MAX_MB` (idem), `KESH_ADMIN_BACKUP_DIR` (avant : chemin
   vide, sauvegarde dans le répertoire courant ; après : `/tmp`), `KESH_DOCUMENTS_DIR` / `KESH_INBOX_DIR`
   (avant : chemin vide, `config.rs:948-953` ; après : `/data/documents`, `/data/inbox`), `KESH_LANG`
   (avant : avertissement « Locale '' non reconnue » puis fr-CH ; après : `fr` sans avertissement),
   `KESH_COOKIE_SECURE="   "` et `KESH_TEST_MODE="   "` — **espaces seuls** (F-4 de la P2 ; avant : refus
   du démarrage, `InvalidCookieSecureValue` / `InvalidTestModeValue`, aucune branche n'y trime ; après :
   valeur par défaut — cookie `Secure`, mode test inactif, sens sûr dans les deux cas), `KESH_LOG_FILE_*`
   (F-4 de la P2, **rectifié au code** : le trim n'y change rien — `from_raw` trime le chemin, et
   `LogRotation::parse`, `LogFormat::parse` et `parse_max_files` triment déjà leur entrée, `config.rs:1211`,
   `:1236`, `:1253` ; l'exemple « `" daily"` invalide devient valide » des rapports est faux. Ce qui change
   est le **vide** : avant, `KESH_LOG_FILE_ROTATION=""`, `KESH_LOG_FILE_FORMAT=""` et
   `KESH_LOG_FILE_MAX_FILES=""` produisaient un avertissement « invalide » collecté puis le défaut ; après,
   le défaut sans avertissement — `KESH_LOG_FILE_PATH=""` reste « pas de journal fichier ») ; la **valeur non-UTF-8** (R-7 de la P2), pour tous les lecteurs (avant :
   `Err(NotUnicode)`, défaut silencieux ou `MissingVar` selon le lecteur ; après : le même résultat, précédé
   d'un avertissement), et en particulier `KESH_PRODUCTION_RESET` non-UTF-8 (avant : `false` silencieux ;
   après : `false` et un avertissement **à chaque** appel de `reset`, puisque `env_flag_enabled` est lue à
   chaque requête) ; et l'effet du **trim** :
   - **lecteurs stricts** : `KESH_COOKIE_SECURE` et `KESH_TEST_MODE` acceptent désormais `" true "` — le
     test `config.rs:2501`, qui range `"  true  "` parmi les valeurs refusées, est **mis à jour** (la valeur
     passe aux acceptées ; `"True "` reste refusé, trimé en `"True"`), et les commentaires `config.rs:838`,
     `:1028`, `:2195` qui citent `" true"` comme refusé sont corrigés ;
   - **lecteurs numériques** (F5) : `KESH_PORT`, `KESH_JWT_EXPIRY_MINUTES`, `KESH_RATE_LIMIT_*`,
     `KESH_INBOX_MAX_*`… font aujourd'hui `parse()` **sans** trim : `" 8080"` avertissait puis prenait le
     défaut ; elle est désormais lue (8080) ;
   - **`KESH_JWT_SECRET`** (R-10 = F4) : un espace de tête ou de fin cesse d'en faire partie, d'où **deux
     conséquences** : (i) un secret de 32 octets dont un bord est un espace en compte désormais 31 et le
     démarrage est **refusé** (`WeakJwtSecret`, `config.rs:657`) ; (ii) tout secret modifié par le trim
     **invalide les jetons émis** — les utilisateurs connectés doivent se reconnecter (le secret ne sert
     qu'aux JWT : `routes/auth.rs`, `setup.rs`, `middleware/auth.rs` ; aucune donnée persistée chiffrée).
   **Tests unitaires** d'`env_nonempty` (module de test de `config.rs`, sous le verrou `env_lock()`) :
   absente → `None` ; `""` → `None` ; `"   "` → `None` ; `" x "` → `Some("x")` ; non-UTF-8 (`OsString`
   Unix) → `None`.
   **Tests « valeur vide » de `from_env` — chacun constaté ROUGE avant le T2** (R-5 ; mémoire « tests qui
   prouvent moins » : un test vert avant le changement ne prouve pas le changement), sortie au Dev Agent
   Record :
   `KESH_HOST=""` → `127.0.0.1` (avant `""`) ; `KESH_JWT_SECRET=""` → `MissingVar("KESH_JWT_SECRET")`
   (avant `WeakJwtSecret{actual_bytes: 0}`) ; `DATABASE_URL=""` → `MissingVar("DATABASE_URL")` (avant :
   accepté vide) ; `KESH_PORT=" 8080 "` → 8080 (avant 80) ; `KESH_ADMIN_BACKUP_DIR=""` → `/tmp` (avant
   `""`) ; `KESH_DOCUMENTS_DIR=""` → `/data/documents` (avant `""`) ; `KESH_SMTP_PORT=""` → 587 **et zéro
   avertissement capté** (avant : un avertissement) ; `KESH_LANG=""` → `Locale::FrCh` **et zéro
   avertissement capté** (avant : l'avertissement de `kesh_i18n`, `Locale::from`). Les deux derniers ne
   discriminent **que par la capture** — la valeur est la même avant et après.
   **⚠️ Alignement sur le livré (T0)** : `KESH_ADMIN_BACKUP_DIR=""`, `KESH_SMTP_PORT=""` et `KESH_LANG=""`
   **ne discriminent plus** — la 15-11a (C-15-11a-6) les rend déjà au défaut sans avertissement, et son
   test `from_env_empty_or_blank_vars_take_code_default_silently` le vérifie (gardé tel quel, vert avant et
   après). Ils sont **remplacés** par des cas qui discriminent sur `HEAD` (relevés au code) :
   `KESH_INBOX_DIR=""` → `/data/inbox` (avant `""`) ; `KESH_PASSWORD_MIN_LENGTH=" 14 "` → 14 (avant :
   avertissement « invalide » puis 12 — le trim d'une valeur **non blanche**, que la 15-11a ne fait pas) ;
   `KESH_SMTP_PORT=" 2525 "` → 2525 **et zéro avertissement capté nommant `KESH_SMTP_PORT`** (avant :
   avertissement puis 587) ; `KESH_COOKIE_SECURE="   "` → `true` (avant : `InvalidCookieSecureValue`) ;
   `KESH_LOG_FILE_ROTATION=""` par `LogConfig::from_env` → aucun avertissement collecté (avant : un
   avertissement « invalide »). Restent : `KESH_HOST`, `KESH_JWT_SECRET`, `DATABASE_URL`, `KESH_PORT`
   (trim), `KESH_DOCUMENTS_DIR`.
   **Comment « sans avertissement » s'asserte** (F6) — *T0 : la 15-11a a déjà posé dans le module de
   test de `config.rs` une capture locale, `from_env_with_logs()` (abonné `fmt` écrivant dans un tampon,
   installé par `tracing::subscriber::with_default` pour l'appel), avec son témoin
   `from_env_non_empty_invalid_values_still_warn` (`KESH_SMTP_PORT="abc"` → un avertissement nommant la
   variable). Elle est **réutilisée** (DRY) au lieu de la couche décrite ci-dessous ; les assertions sont
   bornées au message nommant la variable (`KESH_SMTP_PORT`), et une assertion de montage vérifie que la
   capture voit la ligne « Locale instance » (C-15-11b-1)* : une **capture `tracing` locale** dans le module de
   test de `config.rs`, sur le modèle de `crates/kesh-api/tests/common/capture_rejeu.rs` (15-5e1 — non
   importable depuis un test unitaire, le motif est reproduit en une vingtaine de lignes) : une couche
   `tracing_subscriber::Layer` qui compte les événements de niveau `WARN` (toutes cibles), installée par
   `tracing::subscriber::set_default(tracing_subscriber::registry().with(couche))` pour le **fil courant**
   le temps de l'appel à `from_env` (un test unitaire s'exécute sur son propre fil, `from_env` aussi) ;
   aucune dépendance nouvelle (`tracing-subscriber` est une dépendance normale de `kesh-api`). La couche
   **enregistre la cible et le message** de chaque événement `WARN`, et les assertions sont **bornées à la
   cible attendue** (R-9 de la P2 — `from_env` peut émettre d'autres avertissements, sans rapport, avec le
   jeu de variables du test) : pour `KESH_SMTP_PORT`, les événements dont le message contient
   `KESH_SMTP_PORT` ; pour `KESH_LANG`, les événements de cible `kesh_i18n`. **Témoin obligatoire contre la
   capture muette** : `KESH_SMTP_PORT="abc"` → 587 **et exactement un** avertissement capté **dont le
   message contient `KESH_SMTP_PORT`** — vert avant comme après ; s'il est rouge, la capture ne voit rien et
   les deux assertions « zéro avertissement » ne prouvent rien.
   **`reset_env()`** (`config.rs:1515`) est **complété pour les variables des tests neufs** (F10 de la P2
   de la 15-11 ; R-2 de la P2 de la 15-11b) : il retire déjà les 22 noms que les tests existants posent
   (25 noms au total, vérifié le 2026-10-08) ; il n'y manque que **trois** variables, que les tests neufs
   posent ou dont ils assertent le défaut — `KESH_ADMIN_BACKUP_DIR`, `KESH_DOCUMENTS_DIR`,
   `KESH_INBOX_DIR`. *(T0 : la 15-11a l'a porté à **28** noms, dont `KESH_ADMIN_BACKUP_DIR` ; manquent
   `KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR`, et les quatre `KESH_LOG_FILE_*` que pose le test neuf de
   `LogConfig::from_env`.)* Contrôle, à refaire au T2 sur les tests écrits, qui voit aussi les `set_var(` écrits sur deux
   lignes (R-7 = F7 — huit `env::set_var(\n "KESH_JWT_SECRET", …)`) :
   `grep -Pzo 'set_var\(\s*"\K[A-Z_]+' crates/kesh-api/src/config.rs | tr '\0' '\n' | sort -u`, comparé à
   sa liste.

3. **Le test lit le code — lexicalement** (C78) — `crates/kesh-api/tests/configuration_transmise.rs`
   (posé par la 15-11a) gagne la famille (L) et l'extension de (F) au code ; la liste `LUES` est
   **retirée**, l'ensemble lu étant désormais **calculé** par (L). (T), (V), (E) et le (F) des corpus texte
   de la 15-11a jugent cet ensemble calculé, sans autre changement.
   **Ce que le test affirme, et rien de plus** : il ne reconnaît **aucune forme d'appel** ; il relève
   **chaque occurrence** d'un jeton surveillé dans le flux de jetons du code de production et exige qu'elle
   figure, à son emplacement et sous sa forme, dans une liste fermée. **Faux rouge possible** (une ligne à
   ajouter à la liste) ; **faux vert impossible pour le code du workspace** (F-6 de la P2) — toute lecture
   écrite dans `crates/*/src` passe par `std::env`, `dotenvy` ou une API de dépendance surveillée par un
   jeton (`EnvFilter`, `Builder`, `init`…). Une **dépendance qui lit l'environnement par son API interne**,
   sans qu'aucun jeton du workspace le trahisse, est un **angle mort écrit**, avec la liste connue et la
   façon dont elle a été établie (§ *Angles morts*).
   - **(L) Lectures** — pour chaque fichier `crates/*/src/**/*.rs` :
     - **Flux** : le contenu est lu à l'exécution et converti en `proc_macro2::TokenStream` (`str::parse`,
       feature `span-locations` pour les positions) ; le parcours **descend dans tous les groupes** —
       arguments, blocs, **corps de macros** (`tracing::info!(…)`, `format!`, `macro_rules!`) et
       **attributs** compris. Les commentaires `//` et `/* */` ne sont pas des jetons ; les doc-comments
       `///` sont des littéraux `#[doc = "…"]`, jamais des identifiants.
     - **Jetons surveillés** (identifiants, comparés **exactement** — `parse_strict_bool_multipart`,
       `environment` ne sont pas des occurrences) : `env` (sauf suivi de `!` : `env!(…)` est une macro de
       compilation) — ce qui couvre `std::env::var`, `var_os`, `vars`, `vars_os`, `temp_dir`, tout
       `use … env …` et tout renommage `as` de `std::env` ; `dotenvy` — ce qui couvre `dotenvy::var`,
       `dotenvy::dotenv_iter`, `use dotenvy …`, `extern crate dotenvy as …` ; `from_default_env` et
       `try_from_default_env` (lecture de `RUST_LOG` par `EnvFilter`, F8 c) ; **l'API de lecture de
       `tracing-subscriber`** (F-1 de la P2) — l'identifiant **`EnvFilter`** lui-même, ce qui couvre
       `EnvFilter::from_env("X")` (nom **arbitraire**), `try_from_env`, `builder()`, et tout alias
       (`use … EnvFilter as F;` porte le jeton : relevé, rouge ; `type F = EnvFilter;` aussi) ; **`Builder`**,
       `with_env_var`, `from_env_lossy`, `try_from_env`, car le constructeur s'atteint **sans** le jeton
       `EnvFilter` (`tracing_subscriber::filter::Builder::default().from_env()` — `filter` ré-exporte
       `env::Builder`) et que sa méthode `from_env` ne peut être surveillée (homonyme de `Config::from_env`) ;
       **`init`** et **`try_init`**, car `tracing_subscriber::fmt::init()` / `fmt::try_init()` lisent `RUST_LOG`
       par `EnvFilter::from_default_env()` interne (`fmt/mod.rs:1200-1204`) sans aucun autre jeton — l'unique
       `.init()` de production (`logging.rs:162`) a son entrée ; et les **indirections de
       lecture** `env_nonempty`, `parse_strict_bool`, `env_flag_enabled`, `init_tracing` — liste vérifiée
       au code le 2026-10-08 (§ *Inventaire*), à revérifier au T0 —, plus `opt_trimmed_env` **du T1 au T2
       seulement** (le T2 l'absorbe et retire le jeton). Un `use crate::config::env_nonempty as lire;`
       est une occurrence d'`env_nonempty` comme une autre : **rouge**, car aucune entrée ne l'autorise.
     - **Fenêtre** d'une occurrence : l'occurrence, puis les jetons qui la suivent **dans le même groupe**,
       jusqu'au premier groupe `( … )` **inclus**, ou jusqu'au premier `;`, `,` ou groupe `{ … }` / `[ … ]`
       **exclu** (sans quoi `-> EnvFilter { … }` s'étendrait jusqu'à la fonction suivante), ou jusqu'à la
       fin du groupe englobant ou du fichier. Elle se compare **jeton à jeton** (texte de chaque jeton, groupes aplatis
       avec leurs délimiteurs), et s'écrit dans la liste en jetons séparés par une espace. Exemples :
       `std::env::var_os(name)` → `env :: var_os ( name )` ; `dotenvy::dotenv().ok()` →
       `dotenvy :: dotenv ( )` ; `.map(env_nonempty)` → `env_nonempty` ; `use std::env;` → `env` ;
       `crate::config::env_nonempty(name)` → `env_nonempty ( name )` ; `fn f() -> EnvFilter { … }` →
       `EnvFilter` ; `use tracing_subscriber::{EnvFilter, Layer, fmt};` → `EnvFilter`.
     - **Emplacement** d'une occurrence : fichier, plus le chemin de l'élément englobant **le plus
       intérieur**, résolu par un visiteur `syn` (`parse_file`, `visit`) qui relève la plage de positions de
       chaque élément — **API nommée** (R-5 = F-3 de la P2) : `syn::spanned::Spanned::span()` de l'élément,
       puis `proc_macro2::Span::start()` / `end()` (`LineColumn`, feature `span-locations`) ; une occurrence
       du flux appartient à l'élément si sa position `start()` est comprise dans la plage, **comparée par
       (ligne, colonne)**, **jamais** par `byte_range()` : le flux (`str::parse`) et l'arbre
       (`syn::parse_file`) sont deux analyses du même texte, et dans le repli de `proc-macro2` chaque
       analyse reçoit un décalage d'octets global distinct, si bien que les `byte_range()` ne se comparent pas
       d'une analyse à l'autre, alors que (ligne, colonne) si. Faisabilité lue dans les sources
       (`proc-macro2` 1.0.106, `fallback.rs` : `Span::join` fonctionne à l'intérieur d'une analyse, donc
       `span()` d'un élément `syn` couvre l'élément entier) ; le cas (S) « dernière ligne d'une fonction »
       l'éprouve à l'exécution. Emplacements : modules en ligne, puis `Type::fonction` pour une fonction d'`impl` (dernier segment
       du type ; `Config::from_env` et `LogConfig::from_env` sont **deux** emplacements), `fonction` pour une
       fonction libre ou imbriquée (la plus intérieure), `<hors fonction>` sinon (`use`, `const`, `static`,
       `macro_rules!` de niveau module). Une fermeture appartient à sa fonction ; la signature d'une
       fonction, à la fonction.
     - **Liste fermée `EMPLACEMENTS_AUTORISES`** : entrées (fichier, emplacement, jeton, forme, nombre).
       **Forme** : soit `Exacte(fenêtre)`, soit `Littéral` — la fenêtre se termine par un groupe `( … )`
       dont le **premier jeton est un littéral de chaîne**, suivi de `,` ou de rien. Pour chaque entrée, le
       nombre d'occurrences trouvées sous cette forme à cet emplacement doit être **égal** au nombre écrit.
       **Toute occurrence qu'aucune entrée ne couvre → rouge** (message : fichier, emplacement, fenêtre,
       renvoi à `config::env_nonempty` et à C75, et la ligne à ajouter si l'occurrence est voulue) ;
       **toute entrée dont le nombre diffère, en plus ou en moins → rouge** (liste périmée).
       **Liste attendue après le T2** (fenêtres indicatives : figées au T3 depuis le code, tout écart écrit
       au Dev Agent Record) :

       | fichier | emplacement | jeton | forme | nombre |
       |---|---|---|---|---|
       | `kesh-api/src/config.rs` | `env_nonempty` | `env_nonempty` | `Exacte(env_nonempty ( name : & str ))` (définition) | 1 |
       | `kesh-api/src/config.rs` | `env_nonempty` | `env` | `Exacte(env :: var_os ( name ))` | 1 |
       | `kesh-api/src/config.rs` | `Config::from_env` | `env_nonempty` | `Littéral` | 31 |
       | `kesh-api/src/config.rs` | `Config::from_env` | `parse_strict_bool` | `Littéral` | 2 |
       | `kesh-api/src/config.rs` | `Config::from_env` | `dotenvy` | `Exacte(dotenvy :: dotenv ( ))` | 1 |
       | `kesh-api/src/config.rs` | `LogConfig::from_env` | `env_nonempty` | `Littéral` | 4 |
       | `kesh-api/src/config.rs` | `parse_strict_bool` | `parse_strict_bool` | `Exacte(parse_strict_bool ( var : & str , default : bool ))` (définition) | 1 |
       | `kesh-api/src/config.rs` | `parse_strict_bool` | `env_nonempty` | `Exacte(env_nonempty ( var ))` | 1 |
       | `kesh-api/src/main.rs` | `main` | `dotenvy` | `Exacte(dotenvy :: dotenv ( ))` | 1 |
       | `kesh-api/src/main.rs` | `main` | `env_nonempty` | `Littéral` | 2 |
       | `kesh-api/src/main.rs` | `main` | `init_tracing` | `Exacte(init_tracing ( & log_config ))` | 1 |
       | `kesh-api/src/logging.rs` | `init_tracing` | `init_tracing` | `Exacte(init_tracing ( cfg : & LogConfig ))` (définition) | 1 |
       | `kesh-api/src/logging.rs` | `init_tracing` | `env_nonempty` | `Exacte(env_nonempty ( EnvFilter :: DEFAULT_ENV ))` | 1 |
       | `kesh-api/src/logging.rs` | `init_tracing` | `EnvFilter` | `Exacte(EnvFilter :: DEFAULT_ENV)` | 1 |
       | `kesh-api/src/logging.rs` | `init_tracing` | `init` | `Exacte(init ( ))` | 1 |
       | `kesh-api/src/logging.rs` | `<hors fonction>` | `EnvFilter` | `Exacte(EnvFilter)` (`use tracing_subscriber::{EnvFilter, …}`) | 1 |
       | `kesh-api/src/logging.rs` | `build_log_filter` | `EnvFilter` | `Exacte(EnvFilter)` (type de retour) | 1 |
       | `kesh-api/src/logging.rs` | `build_log_filter` | `EnvFilter` | `Exacte(EnvFilter :: new ( raw ))` | 1 |
       | `kesh-api/src/routes/onboarding.rs` | `env_flag_enabled` | `env_flag_enabled` | `Exacte(env_flag_enabled ( name : & str ))` (définition) | 1 |
       | `kesh-api/src/routes/onboarding.rs` | `env_flag_enabled` | `env_nonempty` | `Exacte(env_nonempty ( name ))` | 1 |
       | `kesh-api/src/routes/onboarding.rs` | `reset` | `env_flag_enabled` | `Littéral` | 1 |
       | `kesh-api/src/routes/admin.rs` | `stream_via_tempfile` | `env` | `Exacte(env :: temp_dir ( ))` | 1 |

       **22 entrées** (17 de la P1, plus 5 pour `EnvFilter` et `init` — F-1 de la P2 ; recomptées sur
       `logging.rs` : `EnvFilter` 4 fois en production, `init` 1 fois). Aucune entrée `use` pour
       `env_nonempty` : les appels hors de `config.rs` sont qualifiés (AC2). `init_tracing` a ses emplacements propres (définition, appel de `main`, lecture de
       `RUST_LOG`) : son appel `init_tracing(&log_config)` est **autorisé**, non « non littéral hors
       indirection » (F1). Un appel non littéral d'`env_nonempty` **à l'intérieur** d'une indirection autre
       que celui de la table — `crate::config::env_nonempty(NIVEAU)` dans `init_tracing`, un second `env_nonempty(…)` dans
       `parse_strict_bool` — n'est couvert par aucune entrée : **rouge** (R-1). `temp_dir()` est autorisé
       et **inventorié** (il lit `TMPDIR`, nom non compté dans l'ensemble lu — angle mort assumé).
     - **Noms lus** (indépendants de la liste, pour que (T), (E) et (F) jugent ce qui est réellement lu,
       R2-1) : (1) pour **toute** occurrence relevée d'un jeton de lecture — tous les jetons surveillés sauf
       `init_tracing`, `init` et `try_init` : `env`, `dotenvy`, `EnvFilter`, `Builder`, `with_env_var`,
       `try_from_env`, `from_env_lossy`, les indirections, **y compris `opt_trimmed_env` au T1** (R-4 de la P2 : ses cinq
       littéraux sont dans les 41) —, autorisée **ou non**, dont la fenêtre se termine par un groupe `( … )` dont le
       premier jeton est un littéral de chaîne, ce littéral est un nom lu — `std::env::var("X")` hors
       liste **rougit** et lit `X` ; (2) une fenêtre dont le groupe d'arguments se termine par
       `EnvFilter :: DEFAULT_ENV` lit `RUST_LOG` — **contrôle** : le test lit
       `tracing_subscriber::EnvFilter::DEFAULT_ENV` (dépendance normale de `kesh-api`) et exige
       `"RUST_LOG"`.
     - **Règle `cfg(test)`** (exclusion, par position : toute occurrence située dans la plage d'un élément
       exclu est ignorée) : est exclu (1) tout **élément** — `mod`, `fn`, `impl`, élément d'`impl`,
       **élément de trait**, `use`, `const`, `static`, `macro_rules!` — portant `#[cfg(test)]` ou
       `#[cfg(all(test, …))]` (`test` conjoint **au premier niveau** de `all`), ou `#[test]` ; (2) toute
       **instruction** (`syn::Stmt`) portant l'un de ces attributs ; (3) le **fichier entier** s'il porte
       l'attribut interne `#![cfg(test)]` (aucun aujourd'hui) ; (4) tout ce que contient un élément exclu.
       **Ne sont pas exclus** : `cfg(any(test, …))`, `cfg(not(test))`, `cfg_attr(…)`, et le **fichier** d'un
       module hors ligne `#[cfg(test)] mod x;` (aucun aujourd'hui) — faux rouge possible, faux vert jamais.
     - **Garde contre le test muet** : la liste des fichiers lus contient `kesh-api/src/config.rs`,
       `main.rs`, `logging.rs`, `routes/onboarding.rs` ; au moins une plage exclue est trouvée dans
       `config.rs` (son `mod tests`) ; l'ensemble lu contient au moins `DATABASE_URL`, `KESH_JWT_SECRET`,
       `KESH_SMTP_HOST`, `KESH_PRODUCTION_RESET` et `RUST_LOG`, et compte **au moins 41** noms.
   - **(F) Fantômes, étendu au code** — aux corpus texte de la 15-11a s'ajoutent les **littéraux de chaîne
     du flux de jetons** du code de production, relevés par le même parcours lexical et la **même règle
     `cfg(test)`** que (L) (F-2) : tout `Literal` du flux converti par `syn::Lit::new`, retenu s'il est un
     `Lit::Str` — ce qui comprend les doc-comments (`#[doc = "…"]`), les chaînes des **macros**
     (`write!`, `tracing::info!`) et celles des **attributs** (`#[error("…")]`, `#[serde(rename = "…")]` —
     F8 a). Même expression et même règle de préfixe que le (F) texte de la 15-11a. Le code de test n'est
     pas un corpus — sans quoi la mutation verte M6 rougirait sur `"KESH_FAUX"`. Les commentaires `//`
     restent invisibles (ex. `KESH_PAT_`, `middleware/auth.rs:129`).
   - **(S) Auto-test**, ajouts aux cas de la 15-11a (sources synthétiques passées aux mêmes fonctions,
     avec une liste d'autorisations synthétique) : `env_nonempty("X")` à un emplacement autorisé → vert,
     `X` lu ; `std::env::var("X")` hors liste → rouge **et** `X` lu ; `dotenvy::var("X")` → rouge et `X`
     lu ; `dotenvy::dotenv()` à son emplacement autorisé → vert ; `use std::env;`, `use std::env as e;`,
     `use std::env::{self, var_os};`, `use std::{env, fs};`, `extern crate dotenvy as d;` → rouges ;
     `.map(std::env::var)`, `.map(env_nonempty)`, `use crate::config::env_nonempty as lire;` → rouges ;
     `tracing::info!("{:?}", std::env::var_os("X"))` → rouge et `X` lu ; `std::env::var("X")` dans le corps
     d'un `macro_rules!` → rouge ; `env_nonempty(n)` non littéral dans une fonction qui a d'autres
     entrées → rouge (R-1) ; `init_tracing(&cfg)` à son emplacement autorisé → vert, ailleurs → rouge (F1) ;
     `EnvFilter::from_default_env()` → rouge ; `EnvFilter::from_env("X")` et `EnvFilter::try_from_env("X")`
     → rouges **et** `X` lu ; `EnvFilter::builder().with_env_var("X")` → rouge et `X` lu ;
     `use tracing_subscriber::EnvFilter as F;` → rouge ;
     `tracing_subscriber::filter::Builder::default().from_env_lossy()` → rouge ;
     `tracing_subscriber::fmt::init()` → rouge ; `fn f() -> EnvFilter { … }` → fenêtre `EnvFilter` (F-1) ;
     `use crate::config::env_nonempty;` → rouge, et `crate::config::env_nonempty("X")` à un emplacement
     autorisé → vert (R-1 = F-2) ; une occurrence sur la **dernière ligne** d'une fonction de plusieurs
     lignes → attribuée à cette fonction, et la même dans une `#[cfg(test)] fn` de plusieurs lignes →
     exclue (R-5 = F-3 : une plage réduite à son premier jeton rougirait) ; `env!("CARGO_PKG_VERSION")`, un identifiant
     `parse_strict_bool_multipart` → aucune occurrence ; `env_nonempty(EnvFilter::DEFAULT_ENV)` → `RUST_LOG`
     lu ; deux fonctions `from_env` dans deux `impl` → deux emplacements distincts ; une entrée dont le
     nombre diffère (en plus, en moins) → rouge ; une entrée sans occurrence → rouge ; occurrence dans un
     `#[cfg(test)] mod`, un `#[cfg(all(test, unix))] fn`, une instruction `#[cfg(test)] let …`, un élément
     de trait `#[cfg(test)] fn`, un fichier `#![cfg(test)]` → exclue ; dans un
     `#[cfg(any(test, feature = "x"))] fn` ou une instruction `#[cfg(not(test))]` → **non** exclue ; nom
     en commentaire `//` → non lu ; jeton fantôme dans `tracing::info!("… KESH_FANTOME …")`, dans
     `write!(f, "… KESH_FANTOME …")`, dans un `/// … KESH_FANTOME …` et dans un
     `#[error("… KESH_FANTOME …")]` → rouge (F) ; le même jeton dans un `#[cfg(test)] mod` → **non vu**
     (F-2).
   - **Dépendances de test** : `syn` 2 (`full`, `visit` ; ligne de la 15-5e1, features unies) et
     **`proc-macro2 = { version = "1", features = ["span-locations"] }`** (F-3 = R3-3 : un test
     d'intégration ne peut nommer `proc_macro2::TokenStream`/`TokenTree`/`Group`/`Literal` qu'avec une
     dépendance directe ; `syn` ne la ré-exporte pas ; déjà au `Cargo.lock`, 1.0.106 — aucun paquet neuf ;
     la feature, côté cible seulement — résolveur 3 —, n'ajoute aucun paquet).

4. **Mutations** (T4) — chacune appliquée, test exécuté, rouge constaté **avec le message attendu** (toutes
   les familles listées, et elles seules), puis restaurée ; résultat au Dev Agent Record :
   M1 ajouter `std::env::var("KESH_ESSAI_MUTATION")` dans `Config::from_env` (rouge dans **trois
   familles** — (L) occurrence hors liste, (T) lue non transmise, (E) lue sans ligne d'affectation ; R3-6)
   · M2 ajouter `fn lire(n: &str) -> Option<String> { std::env::var(n).ok() }` hors test (L) ·
   M3 ajouter `env_nonempty("KESH_ESSAI_MUTATION")` dans `Config::from_env` ((L) nombre 32 ≠ 31, (T), (E))
   · M4 ajouter `tracing::info!("{:?}", std::env::var_os("KESH_ESSAI_MUTATION"))` dans
   `Config::from_env` ((L) lecture dans une macro, (T), (E)) · M5 remettre le fantôme `KESH_ADMIN_` +
   `RESET` dans le message `tracing::info!` de `main.rs` que la 15-11a a réécrit (ligne et texte relevés
   au T0 — R-12) (F, littéral de macro) · **M6 (vert attendu)** ajouter à la fin de `config.rs` un module de
   test **de nom neuf** (un second `mod tests` ne compilerait pas, E0428 — F-5 de la P2) :
   `#[cfg(test)] mod mutation_m6 { #[test] fn lit() { let _ = std::env::var("KESH_FAUX"); } }` → le test
   **reste vert** ((L) et (F) excluent le code de
   test — F-2) · M7 ajouter `use std::env::var;` et `let _ = var("KESH_LANG");` dans `main` (L, import) ·
   M8 remplacer, dans `parse_strict_bool`, l'expression `env_nonempty(var)` par `std::env::var(var).ok()`
   — même type `Option<String>`, le `match` qui suit compile inchangé (F-5 de la P2) — ((L) :
   occurrence hors liste **et** entrée `parse_strict_bool`/`env_nonempty` périmée) · M9 ajouter
   `let _ = dotenvy::var("KESH_LANG");` dans `main` (L : fenêtre ≠ `dotenvy :: dotenv ( )`) ·
   M10 ajouter `let _: Vec<_> = ["KESH_LANG"].into_iter().map(env_nonempty).collect();` dans
   `Config::from_env` (L, référence non appelée) · M11 ajouter
   `use crate::config::env_nonempty as lire;` et `let _ = lire("KESH_LANG");` dans `reset` de
   `routes/onboarding.rs` (L, renommage) · M12 ajouter, dans `init_tracing`, `const NIVEAU: &str =
   "KESH_ESSAI_MUTATION";` et `let _ = crate::config::env_nonempty(NIVEAU);` (chemin qualifié, comme l'appel autorisé — AC2) ((L)
   appel non littéral dans une
   indirection — R-1 ; et (F) : le littéral `KESH_ESSAI_MUTATION`, non lu, est un fantôme) · M13 retirer
   l'entrée (`routes/onboarding.rs`, `env_flag_enabled`, `env_nonempty`) d'`EMPLACEMENTS_AUTORISES` ((L)
   occurrence hors liste, **seule** famille : l'ensemble lu n'en dépend pas) · M14 ajouter à la liste une
   entrée (`kesh-api/src/main.rs`, `main`, `env`, `Exacte(env :: var ( "KESH_LANG" ))`, 1) sans
   occurrence ((L) liste périmée) · M15 ajouter `/// voir KESH_ESSAI_FANTOME` sur une fonction de
   production (F, doc-comment) · **M16** ajouter, dans `init_tracing`,
   `let _ = EnvFilter::from_env("KESH_ESSAI_MUTATION");` ((L) occurrence d'`EnvFilter` hors liste, (T) lue
   non transmise, (E) lue sans ligne d'affectation — F-1 de la P2) · **M17** ajouter, dans `init_tracing`,
   `let _ = tracing_subscriber::filter::Builder::default().from_env_lossy();` ((L) seule : occurrences de
   `Builder` et de `from_env_lossy` hors liste, aucun nom lu — le chemin qui évite le jeton `EnvFilter`).
   **17 mutations, 16 rouges et 1 verte.** Les sources `.rs` sont lues **à l'exécution** du test ;
   M1-M12 et M15-M17 recompilent `kesh-api` (le fichier muté appartient au crate), M13-M14 le test —
   restauration suivie d'un `touch` avant le run suivant (mutation restaurée, binaire périmé).

5. **Documentation de la règle.** ⚠️ **Aucun jeton `KESH_` fictif** dans les textes écrits ici (R-4 = F3) :
   `.env.example`, les deux compose et `docs/manual/fr/*.tex` sont des corpus du (F) de la 15-11a, et
   `.env.example` de son (E) — une ligne de commentaire qui commence par un nom en majuscules suivi de
   `=` (`# NOM=…`) est une ligne d'affectation. Les formulations ci-dessous n'emploient que des noms réels.
   (a) `.env.example`, en-tête : **une ligne laissée vide (le nom suivi de `=`, sans valeur) vaut une ligne
   absente** — le défaut s'applique —, **sauf** `KESH_LOG_FILE_PATH`, dont le vide désactive le journal
   fichier (renvoi au commentaire que la 15-11a a réécrit, marqueur « contrairement aux autres variables »,
   inchangé) ; et les valeurs sont débarrassées de leurs espaces de tête et de fin. La phrase est en prose
   (aucune ligne de la forme `# NOM=`).
   (b) `.env.example:109-111` (`KESH_COOKIE_SECURE`) : « Tout autre valeur (`True`, `yes`, `on`,
   **espaces**...) refuse le démarrage » devient : *« les espaces de tête et de fin sont ignorés ; toute
   autre valeur (`True`, `yes`, `on`…) refuse le démarrage avec `ConfigError::InvalidCookieSecureValue`. »*
   (c) Les numéros ci-dessous mélangent deux états (F-7 de la P2) : `:648-651` est l'état **annoncé** par la
   15-11a (sur `main` d'aujourd'hui, `:648` porte l'introduction « Les tableaux ci-dessous… »), `:662` et
   `:1306` l'état **actuel**, que la 15-11a décalera ; tous se relèvent **par le texte** au T0.
   `admin-manual.tex:648` (la règle de transmission écrite par la 15-11a) gagne la phrase : *« une ligne
   laissée vide vaut une ligne absente, sauf `KESH_LOG_FILE_PATH` (vide = pas de journal fichier) ; les
   espaces de tête et de fin sont ignorés »* ; la phrase voisine `:649-651` devient *« les variables
   « obligatoires » font échouer le démarrage si absentes ou vides »* (F11). `:662` (`KESH_JWT_SECRET`,
   tableau) gagne : *« les espaces de tête et de fin ne font pas partie du secret et ne comptent pas dans
   les 32 octets »*. `:1306` (§ *Valeurs acceptées et fail-fast* de `KESH_COOKIE_SECURE`) : « Toute autre
   valeur (`True`, `TRUE`, `yes`, `on`, **espaces avant/après**, etc.) refuse le démarrage » devient
   *« les espaces de tête et de fin sont ignorés ; toute autre valeur (`True`, `TRUE`, `yes`, `on`…) refuse
   le démarrage »* ; la phrase sur `KESH_TEST_MODE` qui suit reste juste. PDF régénéré (`make fr`), contrôlé
   aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`) : contient les phrases nouvelles, ne contient plus
   « espaces avant/après, etc.) refuse » ; aucun `Overfull \hbox` nouveau.
   ⚠️ `:1306` est voisin de `:1314` (15-7b2) : conflit textuel possible, résolu par le texte.
   (d) **CHANGELOG `[0.13.0]`**, section **Modifié** : une ligne vide de `.env` vaut désormais une ligne
   absente pour toutes les variables (sauf `KESH_LOG_FILE_PATH`), et les valeurs sont débarrassées de leurs
   espaces de tête et de fin (ex. `KESH_COOKIE_SECURE=" true "`, refusé jusqu'ici, est accepté ;
   `KESH_PORT=" 8080"` est lu au lieu de retomber sur 80). **`KESH_JWT_SECRET`** (R-10 = F4) : un espace de
   bord cesse de faire partie du secret — un secret de 32 octets dont un bord est un espace fait désormais
   **refuser le démarrage** (trop court), et un secret ainsi modifié **déconnecte** les utilisateurs (jetons
   émis invalidés). **Hors Docker seulement** (F-9 de la P3) : `RUST_LOG=""` ne laissait passer que les
   erreurs et vaut désormais `info` ; `KESH_HOST=""` faisait échouer le démarrage et vaut désormais
   `127.0.0.1` — **sous Docker, ces deux cas n'existaient pas** : les compose écrivent `${RUST_LOG:-info}`
   et `${KESH_HOST:-0.0.0.0}` (ou le littéral `0.0.0.0`), qui remplacent déjà le vide. L'entrée le dit, pour
   ne pas annoncer aux exploitants Docker un effet qu'ils ne peuvent pas observer.
   (e) Propagation : `grep -rnE 'opt_trimmed_env|espaces|" true"|VIDE|vide|KESH_X\b' .env.example docs/manual/fr/*.tex crates/kesh-api/src`
   — chaque site lu, traité ou écrit hors sujet.

6. **Gates.** `kesh-api` touché (comportement de lecture) : gate ciblé
   (`cargo nextest run -E 'binary(configuration_transmise)'`, les tests unitaires de `config.rs`, `fmt`,
   `clippy` workspace) entre les passes ; **gate complet** au dernier commit de code
   (`scripts/test-fast.sh`, base remise à zéro avant — `DROP`/`CREATE` de ses seules bases, jamais de
   redémarrage du conteneur MariaDB) ; frontend non touché (l'écrire) ; **E2E complet au dernier commit de
   code** (D7 ; le démarrage du backend E2E lit toute la configuration par la fonction neuve), jugé
   fichier par fichier contre `docs/testing.md` § *Les échecs attendus*. Ne déclarer que ce qui a tourné.

## Tasks / Subtasks

- [ ] **T0 — Relevé** (AC1, AC2) : vérifier que la 15-11a et la 15-5e1 sont mergées (sinon : attendre et le
  signaler) ; refaire les 36 sites, les indirections et la vue lexicale depuis `main` (dont : aucune autre
  fonction de production ne prend un nom de variable en argument — sinon, elle rejoint les jetons
  surveillés) ; pour chacun, ce que produit aujourd'hui une valeur vide (table « avant/après » de l'AC2) ;
  relever la ligne `syn` de `kesh-api` posée par la 15-5e1 et ses features, et la ligne/le texte du message
  de `main.rs` que vise M5. **Toutes les citations de numéro de ligne de cette fiche** (`config.rs:657`,
  `:838`, `:948-953`, `:1028`, `:1211`, `:1236`, `:1253`, `:1331-1343`, `:1348-1415`, `:1515`, `:2195`,
  `:2315`, `:2501`, `main.rs:44, 56, 220, 247`, `logging.rs:23, 96-130, 162`, `admin-manual.tex:648-651,
  662, 1306`…) sont celles du dépôt **avant** le merge de la 15-11a, qui modifie `config.rs` et le manuel :
  elles se **relocalisent par le texte** au T0, depuis `main` à jour, et ne se suivent pas aveuglément (R-3,
  F-7 de la P2).
- [ ] **T1 — Test d'abord, rouge** (AC3, AC2) : ajouter (L) lexical et le (F) du code, avec la liste
  `EMPLACEMENTS_AUTORISES` **cible** (celle d'après le T2) et le jeton transitoire `opt_trimmed_env` ;
  dépendances `syn` (union) et `proc-macro2` (`span-locations`). **Asserter l'égalité de l'ensemble lu
  calculé et de `LUES`** (41 noms ; écart écrit), puis **retirer `LUES`** du test. Constater le **rouge
  attendu** — **exactement** ceci, et rien d'autre :
  - **(L)** : les occurrences hors liste — les **37** `env` de lecture ou d'import (`config.rs` 33,
    `main.rs` 2, `logging.rs` 1, `routes/onboarding.rs` 1 ; le `temp_dir` d'`admin.rs` est couvert) et les
    **8** `opt_trimmed_env` (définition et sept appels — T0) ; les **8** entrées qui portent sur `env_nonempty`
    (ses deux entrées propres, ses trois entrées `Littéral`, et les trois corps d'indirection
    `parse_strict_bool`, `init_tracing`, `env_flag_enabled` — 2 + 3 + 3 = 8), périmées. Les entrées
    `dotenvy`, `temp_dir`, les définitions d'indirection, l'appel `init_tracing` de `main`, les appels
    `Littéral` de `parse_strict_bool` et `env_flag_enabled`, et les **cinq** entrées `EnvFilter`/`init` de
    `logging.rs` (au T1, `EnvFilter::DEFAULT_ENV` est encore dans `std::env::var(…)` : même fenêtre) sont
    **vertes**. La garde « au moins 41 » est
    **verte** ;
  - **(T), (V), (E), (F)** (corpus texte **et** code) : **verts** — la 15-11a a transmis les variables et
    retiré le fantôme du code ;
  - **(S)** : vert.
  Tout rouge hors de cette liste, ou tout élément de la liste resté vert, est un défaut du test à corriger
  avant le T2. Écrire aussi, dans `config.rs`, les **tests « valeur vide » de l'AC2** et la mise à jour du
  test `:2501`, et les constater **rouges** un par un (le témoin de capture est **vert**). Sorties au Dev
  Agent Record.
- [ ] **T2 — `env_nonempty` et bascule** (AC2) : la fonction (doc-comment sans jeton `KESH_` fictif), la
  bascule des 33 sites (T0 ; 35 avant la 15-11a), `opt_trimmed_env` retirée **et son jeton transitoire
  avec elle** — ~~le doc-comment qui précède `opt_trimmed_env` est celui d'`is_loopback_host` (R-8)~~
  **sans objet** (T0 : la 15-11a l'a déjà rattaché à sa fonction, C-15-11a-6) ; `use std::env;` de production
  retiré (le module de test reçoit le sien) ; commentaires `:838`, `:1028`, `:2195` mis à jour ;
  `reset_env()` complété des trois variables (AC2) ; appels qualifiés hors de `config.rs`, sans `use`
  (AC2) ; tests unitaires d'`env_nonempty`.
- [ ] **T3 — Vert** : le test passe ; les tests « valeur vide » passent ; liste `EMPLACEMENTS_AUTORISES`
  figée depuis le code (écarts aux fenêtres indicatives écrits) ; gate ciblé ; table « avant/après »
  remplie au Dev Agent Record.
- [ ] **T4 — Mutations M1-M17** (AC4), une à une, restauration vérifiée par `git diff --stat` vide.
- [ ] **T5 — Documentation** (AC5) : `.env.example`, manuel et PDF, CHANGELOG ; propagation (AC5 e) ; le
  test reste vert après le T5 (aucun jeton fictif introduit).
- [ ] **T6 — Gates** (AC6) et Dev Agent Record (décomptes recomptés depuis la source, avec leur périmètre).

## Dev Notes

### Le trim, et ce qu'il change

`env_nonempty` rend la valeur trimée — c'est déjà le cas des lecteurs d'`opt_trimmed_env`, de
`parse_strict_bool` et des variables d'administration, au motif écrit qu'un espace invisible dans un `.env`
édité à la main ne doit pas faire échouer le démarrage. Deux lecteurs stricts (`KESH_COOKIE_SECURE`,
`KESH_TEST_MODE`) refusaient au contraire `" true"` ; leur motif (« un opérateur croirait le drapeau actif
alors qu'il ne l'est pas ») ne vaut plus quand la valeur trimée **est** acceptée — l'intention de
l'opérateur est honorée, et `"True"`/`"yes"` restent refusés. Le seul effet défavorable est celui du
secret JWT (AC2, AC5 d) : bruyant (refus nommé ou reconnexion), jamais silencieux.

### Pourquoi le test lit le code plutôt qu'une liste

Une liste écrite à la main (`LUES`, 15-11a) ne voit pas une variable ajoutée au code par quelqu'un qui
oublie de la compléter — exactement le défaut de #550, déplacé d'un cran. Lire le code est la seule
assertion qu'un oubli ne contourne pas (CLAUDE.md, § *Inventorier les sites NON RÉSOLUS*).

### Pourquoi le test est lexical, et non sémantique (C78)

Les versions précédentes reconnaissaient des **formes** : appels de chemin, imports et renommages,
indirections « à un site », macros cherchées jeton par jeton. Sur **quatre passes** (P1 à P3 de la 15-11,
P1 de la 15-11b), chaque remédiation de cette machinerie a fait naître la forme suivante qu'elle ne voyait
pas : l'extraction déplacée vers la fonction unique (R2-1), les macros (P2), l'exclusion `cfg(test)` de (F)
et `proc-macro2` (P3), puis, en P1 de la 15-11b, l'appel non littéral **dans** une indirection (R-1, né de
R3-7), `var_os` et les indirections dans une macro (R-2), la référence non appelée et le renommage
d'`env_nonempty` (R-3 = F2), et l'appel autorisé d'`init_tracing` que la règle rougissait en permanence
(F1). Une énumération de formes est ouverte par nature.

Le test lexical renverse la charge : il ne se demande plus **comment** on lit, il relève **chaque
occurrence** d'un jeton par lequel toute lecture doit passer, et la confronte à une liste fermée
d'emplacements. Ce qui n'est pas dans la liste rougit, quelle qu'en soit la forme — macro, renommage,
référence non appelée, `use` groupé. Le prix est un faux rouge (une ligne à ajouter quand on ajoute
légitimement une lecture) ; c'est le prix voulu : ajouter une variable **doit** passer par la liste.

### Pourquoi cette machinerie est isolée dans sa propre story (C77)

Sur les trois passes de validation de la 15-11, les défauts se sont **recyclés** dans cette partie. L'isoler
permet de la valider à part, sans retarder la correction de #550 dont dépend la 15-7b2.

### Angles morts assumés (écrits, non traités)

- Lectures d'environnement **internes aux dépendances** et par **FFI** (`libc::getenv`) : hors champ — la
  promesse de l'AC3 vaut **pour le code du workspace** (F-6 de la P2). **Liste connue**, établie le
  2026-10-08 sur les **28 dépendances directes normales de `kesh-api`** hors `kesh-*`
  (`cargo tree -p kesh-api --depth 1 -e normal`), plus `sqlx-core` et `sqlx-mysql` 0.8.6, par
  `grep -rlE 'env::var|var_os\(|getenv'` dans le `src/` de chaque paquet du registre `~/.cargo/registry/src`,
  chaque site lu :
  - **API de lecture appelable par le code** : `dotenvy` (surveillé) ; `tracing-subscriber` — `EnvFilter`
    (`from_default_env`, `try_from_default_env`, `from_env`, `try_from_env`), `filter::Builder`
    (`with_env_var`, `from_env`, `try_from_env`, `from_env_lossy`, `builder.rs:132-216`), `fmt::init` /
    `fmt::try_init` — **tous surveillés** par un jeton (AC3). **Aucune autre** dépendance directe n'expose
    d'API qui lise une variable nommée par l'appelant.
  - **Lectures internes à nom fixe** (aucun jeton possible ; noms hors de l'ensemble lu) : `NO_COLOR`
    (`tracing-subscriber`, `fmt::Layer::default`, `fmt_layer.rs:743` — **lu en production**, `fmt::layer()`
    de `init_tracing`), `TOKIO_WORKER_THREADS` (`tokio`, création du runtime), `TZ` (`chrono`, `time`,
    fuseau local).
  - **Sans effet à l'exécution** : `tower` et `tokio` (exemples de doc-comments), `rust_decimal`
    (`MYSQL_URL`, sous `#[cfg(test)]`), `zip` (`build.rs`, compilation).
  - **Non examinées** : les dépendances **transitives** (hors `sqlx-core`/`sqlx-mysql`) et celles des autres
    crates du workspace — une dépendance nouvelle qui lirait l'environnement par son API ne laisserait
    aucun jeton (F-6) : angle mort, sans garde de classement des dépendances (option écartée, C80).
- Identifiants **synthétisés** par une macro procédurale (type `paste!`) : invisibles au flux de jetons ;
  aucune telle dépendance dans le workspace aujourd'hui.
- Littéraux de chaîne d'octets (`b"…"`) et caractères : non retenus par (F).
- **`include!`** et **`#[path]`** vers un fichier hors `crates/*/src` : non lus (aucun aujourd'hui).
- **Avertissement non-UTF-8 perdu** pour les lecteurs qui précèdent l'abonné `tracing` (AC2).
- **Module hors ligne `#[cfg(test)] mod x;`** : son fichier n'est pas exclu (faux rouge possible).
- **`TMPDIR`** (`std::env::temp_dir()`, `routes/admin.rs:80`) : occurrence **inventoriée et autorisée**,
  nom non compté dans l'ensemble lu.

### Fichiers touchés

`crates/kesh-api/src/config.rs`, `crates/kesh-api/src/main.rs`, `crates/kesh-api/src/logging.rs`,
`crates/kesh-api/src/routes/onboarding.rs`, `crates/kesh-api/Cargo.toml`, `Cargo.lock`,
`crates/kesh-api/tests/configuration_transmise.rs`, `.env.example`, `docs/manual/fr/admin-manual.tex` +
`.pdf`, `CHANGELOG.md`. Aucune migration ; `kesh-db` non touché ; compose non touchés ; `routes/admin.rs`
non modifié (seulement inventorié par le test).

**Règle de découpage** (CLAUDE.md, § *Règle de splitting préventif*) : un crate ; dans `kesh-api`,
**quatre** modules de code (`config`, `main`, `logging`, `routes/onboarding`), plus le test et des fichiers
de documentation. Seuil (« plus de 5 modules ») **non franchi**. Cette fiche est elle-même le produit d'un
découpage (C77).

### References

- Issue #550 ; fiche `15-11-configuration-transmise.md` (index du découpage ; version complète au commit
  d69fdcca) ; fiche sœur `15-11a-compose-transmet-la-configuration.md` ; `15-5e1-socle-rejeu.md` (`syn`, C70 ;
  capture `tracing` `tests/common/capture_rejeu.rs`, C74).
- Rapports de validation : `target/gate-logs/15-11-p{1,2,3}-{R,F}.md`, `target/gate-logs/15-11b-p{1,2}-{R,F}.md`.
- `crates/kesh-api/src/config.rs:7` (`use std::env;`), `:566-1150` (`Config::from_env`), `:657` (longueur
  du secret JWT), `:948-953`, `:1331-1343` (journal fichier), `:1348-1415` (helpers, doc
  d'`is_loopback_host`), `:1507-1546` (`env_lock`, `reset_env`), `:2315`, `:2501` ; `main.rs:44, 56, 57,
  220, 247` ; `routes/onboarding.rs:45-53, 276` ; `routes/admin.rs:80` ; `logging.rs:104-120, 130`.
- `.env.example:109-111` ; `admin-manual.tex:648-651, 662, 1306`.
- CLAUDE.md § *Inventorier les sites NON RÉSOLUS* ; mémoires « mutation restaurée, binaire périmé »,
  « tests qui prouvent moins ».

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — **Création par découpage de la 15-11** (agent de découpage, autonomie ; choix **C77**).
  Reprend de la 15-11 (version d69fdcca) l'AC15 (fonction de lecture unique), la famille (L) et le volet
  « code » de (F) de l'AC8, les mutations qui touchent le code Rust (anciennes M3-M6, M11, M16, M17, M21,
  M23), l'AC6 (g), l'AC12 (i) et l'entrée CHANGELOG **Modifié** de l'AC13. Le test ne part plus de rien :
  il étend celui de la 15-11a et remplace sa liste `LUES` par la lecture du code (égalité assertée au T1).
  **Remédiation de la validation P3** de la 15-11 qui tombe ici :
  - **F-2 (MEDIUM)** : (F) applique la règle `cfg(test)` de (L) ; (S) l'exerce ; la mutation verte M6 en
    dépend.
  - **F-3 = R3-3 (MEDIUM / LOW)** : `proc-macro2 = "1"` en dev-dépendance (déjà au lock).
  - **R3-6 (LOW)** : M1 rougit dans trois familles, M3 dans deux — écrit.
  - **R3-7 (LOW)** : « site » d'une entrée d'`INDIRECTIONS` = lecture directe **ou** appel de
    `env_nonempty` à argument non littéral ; (S) l'exerce.
  - **F-9 (LOW)** : l'entrée **Modifié** dit « hors Docker » pour `RUST_LOG=""` et `KESH_HOST=""`.
  - Nouvelle mutation **M10** (entrée d'`INDIRECTIONS` retirée) et **M11** (doc-comment fantôme), pour
    exercer la liste fermée et les `LitStr` de doc.
  - Comptes **recomptés depuis cette fiche** : AC **6**, tâches **7** (T0-T6), mutations **11** (10 rouges,
    1 verte), modules de code **4** (`config`, `main`, `logging`, `routes/onboarding`) — seuil de découpage
    non franchi.
- 2026-10-08 — **Validation P1** (Opus 5.5 ×2, contextes frais : lentille R `target/gate-logs/15-11b-p1-R.md`
  — 0 CRITICAL / 0 HIGH / 5 MEDIUM / 8 LOW ; lentille F `target/gate-logs/15-11b-p1-F.md` — 0 / 0 / 3 MEDIUM
  / 8 LOW). **6 MEDIUM distincts** après recoupement (F2 ≈ R-2 + R-3 ; F3 = R-4), 16 LOW, **tous traités**
  (agent de remédiation, choix **C78**).
  **Signal D5 levé, et c'est un recyclage** : R-1 naît de la remédiation R3-7 ; R-2, R-3 = F2 et F1 portent
  sur la même machinerie (formes d'appel, indirections, macros) que les passes P1-P3 de la 15-11. Ce
  n'est pas l'exception de l'amendement D5. **Sortie retenue par l'orchestrateur, au lieu d'un nouveau
  découpage : la machinerie sémantique est abandonnée et le test devient LEXICAL** (AC3 réécrite, § *Pourquoi
  le test est lexical*). Retirées de la fiche : la définition du « site » par forme d'appel, la règle
  « Imports » par `syn::ItemUse`, la liste `INDIRECTIONS` et sa définition du site d'entrée (R3-7), la
  règle « Macros » à motifs, la liste `EXCEPTIONS_LECTURE`. Remplacées par : jetons surveillés relevés
  dans le flux complet (macros et attributs compris), emplacement résolu par un visiteur `syn`, liste
  fermée `EMPLACEMENTS_AUTORISES` (17 entrées) à nombre exact.
  - **R-1 (MEDIUM)** : un appel non littéral d'`env_nonempty` dans une indirection n'a plus de statut
    particulier — seule la fenêtre exacte de la table est autorisée ; (S) et **M12**.
  - **R-2 + R-3 = F2 (MEDIUM)** : `var_os`/`vars`/`vars_os`, indirections dans une macro, référence non
    appelée, renommage, appel qualifié (F10) — tous couverts par le relevé lexical (fenêtre commençant à
    l'occurrence, quel que soit le préfixe) ; (S) et **M4, M10, M11**.
  - **F1 (MEDIUM)** : `init_tracing` a ses emplacements autorisés propres (définition, appel de `main`,
    lecture de `RUST_LOG`) ; plus de rouge permanent ; (S).
  - **R-4 = F3 (MEDIUM)** : aucun jeton `KESH_` fictif dans `.env.example`, le manuel ni les doc-comments
    (AC2, AC5 en tête et a) ; vérifié contre (E) et (F) de la 15-11a : la formulation en prose « le nom
    suivi de `=` » ne satisfait pas `^#?\s*[A-Z][A-Z0-9_]*=` et ne contient aucun jeton `KESH_` ; T5 exige le
    test vert après la documentation.
  - **R-5 (MEDIUM)** : les tests « valeur vide » discriminent — `KESH_HOST`, `KESH_JWT_SECRET`,
    `DATABASE_URL`, `KESH_PORT` (trim), `KESH_ADMIN_BACKUP_DIR`, `KESH_DOCUMENTS_DIR` changent de valeur ;
    `KESH_SMTP_PORT` et `KESH_LANG` (même valeur avant/après) discriminent par une **capture `tracing`**
    (motif de `capture_rejeu.rs`, 15-5e1) avec **témoin** positif ; tous constatés rouges avant le T2 (T1).
  - **LOW** : R-6 (`RUST_LOG=""` confirmé par la lentille R dans `tracing-subscriber` 0.3.23 — « à
    confirmer » retiré) ; R-7 = F7 (contrôle de `reset_env` multi-ligne, `grep -Pzo`, vérifié : 22 noms) ;
    R-8 (doc d'`is_loopback_host` replacé au T2) ; R-9 (C76 dit « entrée transitoire du T1 au T3 » en
    numérotation de la 15-11 ; c'est T1-T2 ici, et c'est désormais un **jeton** transitoire, non une
    entrée) ; R-10 = F4 (trim du secret JWT : refus possible et déconnexion, à la table, au CHANGELOG et au
    manuel `:662`) ; R-11 (`KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR`, `KESH_PORT` à la table) ; R-12 (M5
    relevée au T0) ; R-13 (`extern crate dotenvy as d` : jeton `dotenvy`) ; F5 (trim des lecteurs
    numériques) ; F6 (méthode de capture écrite) ; F8 (angles morts : `#[error]` désormais **lu** par (F),
    `dotenv_iter` couvert par `dotenvy`, `from_default_env` surveillé, éléments de trait et `#![cfg(test)]`
    ajoutés à la règle d'exclusion) ; F9 (M10 de l'ancienne numérotation remplacée par M13, à famille
    unique) ; F10 (appel qualifié) ; F11 (« si absentes ou vides », manuel `:649-651`).
  - **Fiche 15-11a non modifiée** : aucun finding ne l'exige (ses règles (E)/(F) ne changent pas ; seuls
    les textes que la 15-11b prescrit devaient les respecter).
  - Comptes **recomptés depuis cette fiche** : AC **6**, tâches **7** (T0-T6), mutations **15** (M1-M15 ;
    14 rouges, 1 verte — M6), entrées `EMPLACEMENTS_AUTORISES` **17**, modules de code **4** — seuil de
    découpage non franchi.
- 2026-10-08 — **Validation P2** (Sonnet ×2, contextes frais : lentille R `target/gate-logs/15-11b-p2-R.md`
  — 0 CRITICAL / 0 HIGH / 1 MEDIUM / 8 LOW ; lentille F `target/gate-logs/15-11b-p2-F.md` — 0 / 0 / 1 MEDIUM
  / 6 LOW). **2 MEDIUM distincts** après recoupement (R-1 = F-2, classé MEDIUM ; F-1), **11 LOW distincts**
  (R-2 à R-9, F-4, F-5, F-6 ; F-3 = R-5, F-7 = R-3 ; R-6 absorbé par F-1), **tous traités** (agent de remédiation, choix **C80**).
  **Trend** : P1 5 + 3 MEDIUM (6 distincts) → P2 **2 MEDIUM distincts, non recyclés** — ni l'un ni l'autre
  ne naît du patch de la P1 : R-1 est un trou d'origine de la consigne « sans `use` » (écrite pour
  `main.rs` seul), F-1 une API de dépendance jamais inventoriée. Signal D5 : sévérité égale (MEDIUM →
  MEDIUM) mais défauts distincts, non issus d'une remédiation — amendement D5, **pas de découpage**.
  - **R-1 = F-2 (MEDIUM)** : hors de `config.rs`, appel par **chemin qualifié, sans `use`** —
    `kesh_api::config::env_nonempty(…)` dans `main.rs` (crate binaire : `crate::config` n'y existe pas),
    `crate::config::env_nonempty(…)` dans `logging.rs` et `routes/onboarding.rs` ; un `use` d'`env_nonempty`
    rougit, voulu ; (S) l'exerce ; M12 réécrite avec le chemin. Les 17 entrées de la P1 restent exactes.
  - **F-1 (MEDIUM)** : l'API de lecture de `tracing-subscriber` est surveillée — `EnvFilter` (identifiant
    entier : `from_env`, `try_from_env`, `builder()`, alias), `Builder`, `with_env_var`, `from_env_lossy`,
    `try_from_env` (le constructeur s'atteint sans `EnvFilter`), `init`/`try_init` (`fmt::init()` lit
    `RUST_LOG`, trouvé à la remédiation en lisant les sources). **5 entrées** de plus (`EnvFilter` ×4,
    `init` ×1, recomptés sur `logging.rs`) : **22**. Fenêtre : un groupe `{ … }`/`[ … ]` la termine
    (exclu), sans quoi `-> EnvFilter { … }` déborderait. (S) ; **M16**, **M17**.
  - **LOW** : R-2 (`reset_env` : trois variables seulement) ; R-3 = F-7 (citations relocalisées par le
    texte au T0 ; manuel : deux états nommés) ; R-4 (`opt_trimmed_env` nommé dans « Noms lus (1) ») ;
    R-5 = F-3 (`Spanned::span()` + `Span::start()/end()`, (ligne, colonne), jamais `byte_range()` ; (S) de
    la dernière ligne d'une fonction) ; R-6 (couvert par F-1) ; R-7 + F-4 (table avant/après : non-UTF-8,
    espaces seuls de `KESH_COOKIE_SECURE`/`KESH_TEST_MODE`, `KESH_PRODUCTION_RESET` non-UTF-8 ;
    **`KESH_LOG_FILE_*` rectifié au code** : les quatre valeurs sont déjà trimées en aval, `config.rs:1211`,
    `:1236`, `:1253`, `from_raw` — l'exemple « `" daily"` devient valide » des deux rapports est faux ; seul
    le vide change, avertissement « invalide » → défaut silencieux) ; R-8 (`parse_lossy`,
    `builder.rs:146-158`) ; R-9 (capture bornée à la cible : message contenant `KESH_SMTP_PORT`, cible
    `kesh_i18n`) ; F-5 (M6 en module de nom neuf, M8 en `std::env::var(var).ok()`, toutes deux compilables) ;
    F-6 (promesse « pour le code du workspace » ; angles morts : liste connue établie sur les 28 dépendances
    directes de `kesh-api` + `sqlx-core`/`sqlx-mysql`, méthode écrite).
  - **Fiche 15-11a non modifiée.**
  - Comptes **recomptés depuis cette fiche** : AC **6**, tâches **7** (T0-T6), mutations **17** (M1-M17 ;
    16 rouges, 1 verte — M6), entrées `EMPLACEMENTS_AUTORISES` **22**, modules de code **4** — seuil de
    découpage non franchi.
- 2026-10-09 — **T0 : alignement sur le livré de la 15-11a** (agent de développement, autonomie ; choix
  **C-15-11b-1**). Relevé sur `HEAD` `b2b09f34` (`origin/main` `8f9811d8` + planification). 15-11a et
  15-5e1 mergées (`syn = { version = "2", features = ["full", "visit"] }` en dev-dépendance de `kesh-api`,
  `proc-macro2` 1.0.106 au `Cargo.lock`). Écarts, tous recomptés à la source (`grep -cE 'env::var(_os)?\('`
  par fichier, lignes < `#[cfg(test)]` de `config.rs:1499`) :
  - **Inventaire** : `config.rs` 32 → **30** sites (`Config::from_env` 26 → **24**), total 36 → **34**,
    « les 35 autres » → **33** (`config.rs` 31 → **29**), appels `opt_trimmed_env("…")` 5 → **7**
    (`KESH_LANG`, `KESH_ADMIN_BACKUP_DIR` en plus), identifiant `env` en production 40 → **38** (`config.rs`
    35 → **33**), ensemble lu **41** inchangé, appels `Littéral` d'`env_nonempty` attendus dans
    `Config::from_env` **31** inchangé (24 + 7). Table `EMPLACEMENTS_AUTORISES` : **22** entrées, recomptées.
  - **Rouge exact du T1** : **37** `env` hors liste (`config.rs` 33, `main.rs` 2, `logging.rs` 1,
    `routes/onboarding.rs` 1) et **8** `opt_trimmed_env` (définition + sept appels) ; **8** entrées périmées
    (inchangé).
  - **Tests « valeur vide »** : `KESH_ADMIN_BACKUP_DIR`, `KESH_SMTP_PORT`, `KESH_LANG` vides ne discriminent
    plus (test de la 15-11a `from_env_empty_or_blank_vars_take_code_default_silently`, gardé) ; remplacés par
    `KESH_INBOX_DIR=""`, `KESH_PASSWORD_MIN_LENGTH=" 14 "`, `KESH_SMTP_PORT=" 2525 "` (valeur et capture),
    `KESH_COOKIE_SECURE="   "`, `KESH_LOG_FILE_ROTATION=""` (avertissement collecté). Capture : celle de la
    15-11a (`from_env_with_logs`) est réutilisée, et son témoin `from_env_non_empty_invalid_values_still_warn`
    tient lieu de témoin obligatoire.
  - **`reset_env()`** : 28 noms (et non 25) ; manquent `KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR` et les quatre
    `KESH_LOG_FILE_*` (et non trois variables).
  - **R-8 sans objet** : le doc-comment d'`is_loopback_host` est déjà rattaché.
  - **Citations relocalisées par le texte** : `config.rs` — `use std::env;` `:7`, `Config::from_env`
    `:572-1222`, contrôle de longueur du secret `:683`, `KESH_DOCUMENTS_DIR`/`KESH_INBOX_DIR` `:982-987`,
    `LogConfig::from_env` `:1375-1382`, `opt_trimmed_env` `:1388`, `parse_strict_bool` `:1441`, commentaires
    `" true"` `:859`, `:1062`, `:2594`, test `"  true  "` `:2900`, `reset_env` `:1577` ; M5 : `main.rs:337`
    (« recovery break-glass KESH_ADMIN_USERNAME/KESH_ADMIN_PASSWORD ») ; manuel — règle de transmission
    `admin-manual.tex:664` (dit déjà, pour les compose fournis, « une variable facultative que `.env` ne pose
    pas, ou laisse vide, prend son défaut »), phrase « obligatoires » `:672-676`, ligne `KESH_JWT_SECRET`
    `:689`, `KESH_COOKIE_SECURE` `:1343` ; `.env.example:130-132` (`KESH_COOKIE_SECURE`).
  - Aucun écart ne change une règle ni un AC sur le fond (l'AC2 change de cas de test, non de règle) :
    développement enchaîné.
