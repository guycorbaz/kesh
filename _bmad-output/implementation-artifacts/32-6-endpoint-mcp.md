# Story 32.6 : l'endpoint `/mcp` — protocole, garde, lecture seule, filtre, journal

Status: ready-for-dev

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-6`, branche `story/32-6` partie de `main`
`b723e3d9` (32-5a fusionnée). Registre : `epic-32-choix-autonomes-326.md` (C-32-6-1 à C-32-6-30). Clé courte : `326`.
Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs` seulement. Les numéros de ligne cités sont ceux
de `b723e3d9`.*

⛔ **Développement APRÈS la fusion des 32-5b, 32-5d et 32-5c** (ordre a → b → d → c de la 32-5, C-32-5-27). La 32-6
appelle `services::cles_mcp::authentifier` (32-5b), compte sur `Portee` = `McpScope` à cinq variantes (32-5b, AC-b2),
sur `mcp_keys` conservée à la remise à zéro (32-5d), sur la dépendance directe `ipnet` et le type d'entité d'audit
`mcp_key` (32-5c). **Tout compte de cette fiche qui dépend de ces parties se recompte à l'étape 0 du développement**
(§ *À recontrôler à l'étape 0*), ligne lue citée au Change Log.

⚠️ **Une fiche, deux parties** (C-32-6-1, découpage préventif) : **32-6a** — le point d'entrée et toutes ses gardes
(protocole, clé, règle « servi », filtre d'adresse, `Origin`, lecture seule, politique de validation) ; **32-6b** — le
journal d'audit des appels. Ordre **imposé** : a → b. Chaque partie a ses AC, ses tâches, ses mutations, ses gates et sa
boucle de revue (**boucle complète, Sonnet ↔ Opus** : c'est la story de sécurité de l'epic). Clés au registre de
sprint : `32-6a-endpoint-mcp-garde`, `32-6b-journal-appels-mcp` ; la clé `32-6-endpoint-mcp` passe à `split`.

## Story

En tant que Guy, qui va brancher un agent (Claude Code, Hermes) sur ses livres réels, je veux un endpoint `/mcp` dans
le processus de Kesh qui **n'existe pas** tant que je ne l'ai pas activé, qui ne répond qu'à une **clé MCP** et dans
les **portées** de cette clé, qui ne sert **rien** à une adresse hors de mon réseau ni à une page web étrangère, dont
les outils **lisent sur une connexion en lecture seule**, qui **refuse d'envoyer des données** à une clé soumise à la
validation tant que la validation n'existe pas, et dont **chaque appel d'outil est journalisé** sans ses arguments ni
ses données — afin qu'ouvrir ce canal ne mette en danger ni l'intégrité ni la confidentialité de mes livres.

## Origine

- **Kickoff** (`planning-artifacts/epic-32-serveur-mcp.md` § *32-6*, l. 371-403) : `POST /mcp` en Streamable HTTP,
  JSON-RPC 2.0 ; `initialize`, `ping`, `tools/list`, `tools/call`, `prompts/list`, `prompts/get` ; traits `McpTool`
  (méthode `portee()` **sans défaut**) et `McpPrompt` ; registre ; outil de démonstration `kesh_version`
  (`books:read`) ; garde propre (clé MCP seulement) ; `tools/list` filtré ; hors portée = inconnu ; lecture seule ;
  désactivé → 404 indiscernable d'une route absente, aucun écouteur dédié ; filtre d'adresse sur le pair TCP,
  `X-Forwarded-For` des seuls mandataires déclarés ; `Origin` ; journal (outil, clé, issue, durée — sans arguments ni
  données). **Reports de la 32-5** (l. 395-403) : refus d'un PAT **à l'endpoint** (401, `last_used_at` du PAT
  inchangée) ; **la règle du 404 face à des réglages par société** ; `authentifier` appelé depuis la garde ; politique
  `CleMcpAuthentifiee::validation` au répartiteur (refus d'une clé `required` avant la 32-8) ; réglages lus sur la
  `Lecture`, sans écriture.
- **Spike** (`planning-artifacts/epic-32-spike-mcp.md`, lue en entier) : révision **`2025-11-25`**, `2025-06-18`
  acceptée, sans session, JSON (§ 1.3) ; `Origin` présent et étranger → 403, absent admis, jamais déduit de `Host`
  (§ 1.3) ; outil inconnu ou hors portée → `-32602` identique ; méthode non servie → `-32601` ; `readOnlyHint: true`
  (§ 1.3) ; exigences **E1 (côté répartiteur), E2, E3, E5**, mutations **1, 2, 3, 5 d'E6** (§ 5.3.1) ; points remis à
  la 32-6 (§ 5.3.3) ; conséquences pour la 32-6 (§ 6, l. 932-944) ; six LOW de la relecture P4 (§ 6.2).
- **Réponses de Guy du 2026-10-24 au soir** (spike § 7 *Réponses*, l. 1057-1069 ; epic § *Réponses*) :
  - **Q11** — lecture seule **applicative** : la poignée `Lecture` **plus** le mode de session `READ ONLY` sur la
    connexion du MCP ; **ni second utilisateur MariaDB, ni variable de plus au `.env`** ; **les mots « par
    construction » sont tombés** — la garantie repose sur le code de Kesh et ses tests. Cette fiche ne les emploie que
    pour une propriété que le **compilateur** tient (une visibilité, un type), jamais pour la lecture seule.
  - **Q10 (c)** — la poignée ; **Q12** — validation **par clé**, `requise` par défaut, `dispensée` pour une clé que
    l'administrateur désigne, chaque envoi journalisé. La 32-6 applique la politique **au répartiteur seulement** ; la
    poignée, sa table et son écran sont la 32-8 (frontière : § *La frontière avec la 32-8*).
- **Décisions réseau** (kickoff § *Décisions*, l. 32-35) : local seulement, la sécurité relève de l'infrastructure ;
  filtre d'adresse **de défense en profondeur, désactivable** ; `X-Forwarded-For` lu seulement d'un mandataire de
  confiance déclaré ; une partie des conteneurs de Guy derrière Traefik.
- **Hypothèses de travail** héritées (questions sans réponse) : **Q3** (clés créées par l'Administrateur, C-32-5-7),
  **Q5** (cinq portées), **Q7** (réglages restaurés tels quels, C-32-5-11).

## Ce que le code dit (relevé au code le 2026-10-25, `b723e3d9`)

Chaque fait a été lu ; la ligne est citée.

1. **Une route absente ne répond PAS 404.** `build_router` termine par `.fallback_service(fallback)` puis
   `.layer(csp_html)` (`lib.rs:1225-1227`), le repli étant `ServeDir::new(&static_dir).fallback(ServeFile::new(
   "{static_dir}/index.html"))` (`lib.rs:213-214`). Dans `tower-http 0.6.11` (`Cargo.lock:4400-4401`), `ServeDir`
   rend, pour **toute méthode autre que `GET`/`HEAD`**, `405 Method Not Allowed` avec `Allow: GET,HEAD` et un corps
   vide (`serve_dir/mod.rs:317-327`, `serve_dir/future.rs:172-176`, `response_with_status` l. 185-190), sauf
   `call_fallback_on_method_not_allowed`, non posé ici. Un `GET` d'un chemin absent rend la page de l'application
   (`index.html`, avec l'en-tête CSP posé par `csp_html`, `middleware/csp.rs:52-60`). Les tests du dépôt le savent :
   `password_recovery_e2e.rs:690-703` et `test_endpoints_e2e.rs:97-104` admettent « 404 ou 405 » pour une route non
   montée. ⇒ **« désactivé → 404 » et « indiscernable d'une route absente » se contredisent à la lettre** : un `POST`
   vers un chemin absent rend **405**. C-32-6-2 retient l'indiscernabilité ; **soumis à Guy** (point 1).
2. **Les routes protégées passent par `require_auth`** (`lib.rs:1165-1172`) ; les routes publiques sont montées sur
   `main_router` (`lib.rs:1174-1183`), le bloc protégé fusionné ensuite (`lib.rs:1204`). `require_auth` lit **d'abord**
   le cookie `kesh_access_token`, puis l'en-tête `Authorization: Bearer` (`middleware/auth.rs:83-100`), et route le
   préfixe `kesh_pat_` vers `validate_pat` (`:134`), le reste vers `jwt::decode`. Sa garde « clé `read` = méthode sûre »
   (`:141`, `is_safe_method` `:172`) ne protège rien sur un `POST` JSON-RPC (prémisse 2 du kickoff). `/mcp` ne doit
   donc **pas** passer par `require_auth` : il a sa propre garde.
3. **L'adresse du pair TCP** : `axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())`
   (`main.rs:397-400`) ; soixante-dix fichiers de test montent de même (`grep -rln into_make_service_with_connect_info
   crates/kesh-api/tests | wc -l` → 70), tous sur `127.0.0.1`. L'extracteur `ConnectInfo` d'Axum 0.8.9 se rabat sur
   l'extension `MockConnectInfo` (`axum-0.8.9/src/extract/connect_info.rs:163-164`) : c'est par elle qu'un test simule
   un pair public. Aucun code ne lit `X-Forwarded-For` (`grep -rn "x-forwarded-for\|X-Forwarded-For" crates` → rien).
4. **Le pool de production** se construit dans `main.rs` par `MySqlPoolOptions::new().max_connections(5)
   .acquire_timeout(…)` **sans** `after_connect` (`main.rs:74-78`) ; `kesh_db::create_pool`, qui pose `sql_mode` et
   `time_zone` (`kesh-db/src/pool.rs:28-49`), n'est pas employé en production (#669). Dans `sqlx-core 0.8.6` :
   `Pool::connect_options()` (`pool/mod.rs:545`), `Pool::options()` (`:567`), `PoolOptions::after_connect`
   (`pool/options.rs:380`), `before_acquire` (`:435`), `connect_lazy_with` (`:579`). La `Lecture` de test en lecture
   seule copie déjà les options du pool du test et passe `SET SESSION TRANSACTION READ ONLY` en `after_connect`
   (`tests/common/lecture_seule.rs:17-32`), **sans** `before_acquire`.
5. **Les poignées.** `Lecture(MySqlPool)` et `Trace(MySqlPool)`, champ privé, `pool()` en `pub(in crate::services)`
   (`services/poignees.rs:21-33`, `:38-52`) ; fabriquées en production par `AppState::lecture()` et `AppState::trace()`,
   **toutes deux sur `self.pool`** (`lib.rs:153-155`, `:162-164`). La garde lexicale (v-2) de
   `tests/services_motif.rs` exige que `Lecture::new(`/`Trace::new(` n'apparaissent qu'à ces **deux** lignes
   (`FABRIQUES_V2`, `:1888-1898` ; test `:2539-2546`) ; la garde (v-1) refuse tout `.pool(`/`::pool` hors de
   `services/` sauf `EXCEPTIONS_V1`, **vide** (`:1886`) ; (vi) compte **dix** fichiers dans `services/` (`:2559-2563`).
   `forbid(unsafe_code)` hors tests sur la lib (`lib.rs:9`) et sur `services` (`services/mod.rs:207`).
6. **Le contrôle d'accès des services** : `ContexteAcces::mcp(company_id, user_id, portees)` (`acces.rs:137-143`),
   `exiger` rend `AppError::Forbidden` si la portée manque ou si l'opération n'est pas servie au MCP (`:166-176`) ;
   C-32-3-5 (fiche 32-3, l. 747) : « la 32-6 le traduira en “outil inconnu” côté MCP (aucun oracle) ». `Portee` n'a
   encore que trois variantes (`acces.rs:76-86`) : la 32-5b la remplace par `McpScope` (AC-b2).
7. **Les réglages MCP** (32-5a) : `McpServerSettings` (`kesh-db/src/entities/mcp_server_settings.rs:44-55`), défauts
   **désactivé**, filtre **activé**, sept plages privées, aucun mandataire, aucune origine (`:18-40`, `:59-70`) ;
   `repositories::mcp_server_settings::get` et `get_or_defaults`, sans écriture (`repositories/mcp_server_settings.rs:
   69-79`, `:83-90`) ; **aucune fonction ne liste les sociétés activées** — la règle « servi » de la 32-6 en a besoin
   (C-32-6-3). Les listes sont des chaînes canoniques, validées à l'écriture par la 32-5c (C-32-5-19).
8. **Le journal d'audit** : `audit_log::insert_in_tx` dérive `actor_label` du nom d'utilisateur et `company_id` de
   `users` (`repositories/audit_log.rs:94-95`) ; `actor_type` est un `ENUM('user', 'api_key')`
   (`migrations/20260605000002_audit_log_actor.sql:23`), décodé par `ActorType::from_str`, qui **échoue** sur toute
   autre valeur (`entities/audit_log.rs:53-62`). Toute action de production a son libellé (`audit_labels.rs`, garde
   `tests/audit_label_registry.rs`).
9. **Le registre des routes mutantes** lit `lib.rs` par `include_str!` (`audit_route_registry.rs:529`) et n'extrait que
   la forme `post(routes::module::handler)` ; toute autre forme est un « site non résolu » qui fait rougir le test
   (`:540-567`) ; les appels à `routes::…` qui ne sont pas un handler d'un constructeur `post(`/`put(`/`delete(`/
   `patch(`/`get(` sont inventoriés à part (`:668-700`). Inventaire à **124** routes, registre à **127** (`:768`,
   `:1129`) — **128 et 131 après la 32-5c** (AC-c2 de la 32-5) : à recompter.
10. **La configuration** : `port` et `host` (`config.rs:180-181`), `public_base_url` facultatif, sans barre finale
    (`config.rs:334`, `:1089-1091`). « L'installation ne porte qu'une société » (`user-manual.tex:190`) : la règle par
    société est surtout théorique, mais le schéma en permet plusieurs (`mcp_server_settings.company_id` clé primaire).
11. **Garde lexicale de lecture seule, état de départ** : `grep -rniE "read write|tx_read_only|transaction_read" crates`
    ne rend **aucune ligne** (le `SET SESSION TRANSACTION READ ONLY` de `tests/common/lecture_seule.rs:23` n'est pas
    visé) — état **vierge** des formes qui défont le mode de session (spike § 4.1, quatre formes).
12. **Le manuel** : `docs/api-external.md:183` dit « Il n'existe pas de serveur MCP “Kesh-natif” **en v0.2** » (vrai,
    borné) ; `admin-manual.tex:2925` renvoie à ce guide ; ces textes changent avec la 32-11 (prémisse 10 du kickoff).
    `admin-manual.tex:2230`, `:2237` et `user-manual.tex:3028-3030` nomment déjà les clés et réglages MCP (32-5a).
    Aucune ligne des manuels ne décrit `/mcp`.

## Le découpage (C-32-6-1)

Convention de compte (celle de la 32-5) : un crate pour `kesh-db` et `kesh-i18n` ; un module de second niveau pour
`kesh-api` ; `lib.rs` compte avec `routes` ; un fichier de test compte avec le module qu'il éprouve ; `tests/common/`
avec le module dont il sert les tests.

| Partie | Objet | Modules | 🗄️ |
|---|---|---|---|
| **32-6a** | transport et protocole, traits et registre, `kesh_version`, garde de clé, règle « servi », filtre d'adresse et `X-Forwarded-For`, `Origin`, pool en lecture seule, politique de validation au répartiteur | `kesh-api/mcp` (neuf, et `tests/mcp_*`) ; `kesh-api/routes` (`routes/mcp.rs`, `lib.rs`, `tests/audit_route_registry.rs`) ; `kesh-api/services` (`serveur_mcp.rs`, `tests/services_motif.rs`, `tests/common/lecture_seule.rs`) ; `kesh-db` (`pool.rs`, `repositories/mcp_server_settings.rs`, leurs tests) — **4** | oui (un dépôt : gate complet en boucle ; **aucune migration**) |
| **32-6b** | le journal d'audit de chaque `tools/call` | `kesh-api/mcp` (répartiteur) ; `kesh-api/services` (`serveur_mcp.rs`, doc de `poignees.rs`) ; `kesh-api/audit_labels` ; `kesh-i18n` (quatre locales) — **4** | non |

**Sans découpage : six modules distincts** (`kesh-db`, `mcp`, `routes`, `services`, `audit_labels`, `kesh-i18n`), **huit
en somme** — au-delà du plafond de cinq. Le journal se détache sans laisser d'accès ouvert : **après la 32-6a, aucun
outil ne rend de donnée des livres** — `Registre::standard()` ne porte que `kesh_version`, qui rend la version du
binaire et l'état de la session (AC-a12, figé par un test d'inventaire), et une clé `required` ne peut appeler aucun
outil (AC-a14). Les deux parties se fusionnent **avant toute release** ; aucune n'est publiable seule (la 32-11 précède
la release).

## Critères d'acceptation

Conventions des tests : fichiers neufs `crates/kesh-api/tests/mcp_transport_e2e.rs` (protocole, clé, politique, par
HTTP réel, motif `spawn_app` des fichiers du fait 3), `crates/kesh-api/tests/mcp_reseau_e2e.rs` (filtre, mandataires,
`Origin`, par `tower::ServiceExt::oneshot` et `MockConnectInfo`, fait 3), `crates/kesh-api/tests/mcp_gardes_lexicales.rs`,
`crates/kesh-db/tests/pool_lecture_seule.rs`, `crates/kesh-api/tests/mcp_journal_e2e.rs` (32-6b). Chaque AC dit sa
commande ; « vert » s'entend `cargo nextest run -E 'binary(<fichier>)'` puis le gate complet (§ *Gates*). Bases
`kesh_326`/`kesh_e2e_326` propres au worktree (`docs/agents-orchestration.md` § 2), utilisateur `kesh` du fichier
versionné, **aucun `GRANT` posé à la main**.

### Partie A — 32-6a : le point d'entrée et ses gardes

**A.1 — Le montage et la règle « servi »**

1. **AC-a1 — un seul point d'entrée, sur le même port.** `lib.rs` monte, sur `main_router` **hors** du bloc
   `protected` et **avant** `.fallback_service(…)`/`.layer(csp_html)` (fait 1, fait 2) :
   `.route("/mcp", post(routes::mcp::recevoir).fallback(routes::mcp::autre_methode))` (forme lisible par l'extracteur
   du fait 9). `routes/mcp.rs` est mince : il extrait `State`, `Option<ConnectInfo<SocketAddr>>`, les en-têtes et le
   corps, et délègue à `crate::mcp`. Aucun `TcpListener::bind` neuf : `grep -rn "TcpListener::bind" crates/kesh-api/src`
   rend la seule ligne de `main.rs` (garde lexicale `un_seul_ecouteur`, AC-a30). Le registre des routes mutantes gagne
   `("post", "mcp::recevoir", Traced, SansEcritureAuJournal)` (statut d'audit `Traced` : chaque `tools/call` l'est, en
   32-6b — jusque-là la justification le dit) ; l'inventaire et le registre gagnent un (recomptés à l'étape 0 : 129 et
   132 si la 32-5c a laissé 128 et 131), et la liste des appels non-handlers (`audit_route_registry.rs:668-700`)
   accueille `routes::mcp::autre_methode` (appelé par `.fallback(`), écrit en connaissance de cause.
2. **AC-a2 — l'ensemble des réglages qui « servent » ce pair.** Pour chaque requête vers `/mcp`, avant toute autre
   chose, le service `services::serveur_mcp::reglages_actifs(&Lecture)` lit sur la **`Lecture` du MCP** les lignes
   `enabled = TRUE` (dépôt neuf `mcp_server_settings::list_enabled`, **sans écriture**, testé : aucune, une, deux
   sociétés ; une ligne `enabled = FALSE` absente). L'ensemble **S** retient celles dont le filtre admet le client
   (A.2). **S vide → réponse de route absente** (AC-a3). Une erreur de la base pendant cette décision → réponse de route
   absente, et `tracing::error!` (fail closed, C-32-6-3) — test : dans la base éphémère du test, `ALTER TABLE
   mcp_server_settings RENAME TO mcp_server_settings_cachee` par le pool ordinaire → réponse identique à celle d'un
   chemin absent (l'utilisateur `kesh` a tous les droits sur sa base de test, aucun `GRANT` à poser).
3. **AC-a3 — « désactivé » est indiscernable d'une route absente (C-32-6-2).** Quand `/mcp` n'est pas servi, le
   gestionnaire **délègue la requête au même service de fichiers statiques** que le repli du routeur (une seule
   fonction `service_statique(static_dir)` dans `lib.rs`, appelée pour le repli et pour `/mcp` — DRY), et sa réponse
   traverse la même couche `csp_html`. Test `desactive_indiscernable_d_une_route_absente` : pour `POST`, `GET`, `HEAD`,
   `DELETE`, `OPTIONS`, avec et sans `Authorization` (clé valide comprise), avec et sans corps, la réponse de `/mcp`
   est **égale** à celle de `/mcp-route-absente-326` — statut, **tous** les en-têtes hors `date`, octets du corps —,
   dans trois états : aucune ligne de réglages, ligne `enabled = FALSE`, ligne `enabled = TRUE` dont le filtre refuse
   le pair. Le test compare aussi à un **`static_dir` réel** (un `index.html` de test) pour que le `GET` ne passe pas à
   vide sur deux 404 identiques. Témoin : `enabled = TRUE` et pair admis → `POST` sans clé rend **401** (la règle
   n'est pas « toujours absent »).
4. **AC-a4 — la société de la clé doit elle-même servir.** Après l'authentification (A.3), la société de la clé
   (`ContexteAcces::company_id`) doit avoir sa ligne dans **S** ; sinon, réponse de route absente (même comparaison que
   l'AC-a3). Tests à deux sociétés : (i) société A activée, société B non, clé de B → route absente ; (ii) A et B
   activées, le filtre de B refuse le pair, celui de A l'admet, clé de B → route absente ; (iii) clé de A → servi. Le
   basculement est immédiat : désactiver la ligne entre deux requêtes de la même application change la réponse (aucun
   cache).
5. **AC-a5 — méthodes HTTP quand `/mcp` est servi.** `POST` seul ; `GET`, `HEAD`, `DELETE`, `PUT`, `PATCH`, `OPTIONS`
   → **405** avec `Allow: POST`, corps vide, **sans** lire l'authentification (spike § 1.3 : `GET`/`DELETE` → 405 ;
   `OPTIONS` → 405 ferme toute pré-vérification CORS). Aucun `Mcp-Session-Id` émis ni lu (sans session).

**A.2 — Le filtre d'adresse et les mandataires (C-32-6-4 à C-32-6-6)**

6. **AC-a6 — le client se déduit du pair TCP.** Le pair est `ConnectInfo<SocketAddr>` (fait 3), **normalisé** par
   `IpAddr::to_canonical` (une adresse IPv4 mappée `::ffff:a.b.c.d` devient IPv4). **`ConnectInfo` absent → réponse de
   route absente** (fail closed ; test `oneshot` sans `MockConnectInfo`). Pour un réglage `s` :
   - pair **hors** de `s.trusted_proxies` → client = pair ; **`X-Forwarded-For` ignoré** ;
   - pair **dans** `s.trusted_proxies` → les valeurs de **toutes** les lignes `X-Forwarded-For`, dans l'ordre, coupées
     aux virgules et rognées, sont lues **de droite à gauche** ; les entrées qui sont des mandataires de confiance sont
     sautées ; la **première qui ne l'est pas** est le client ; en-tête **absent** → client = pair ; **toutes** les
     entrées de confiance → client = la plus à gauche ; une entrée qui n'est pas une adresse IP nue (port, crochets,
     `unknown`, vide) → **aucun client** (refus) ;
   - `Forwarded` (RFC 7239), `X-Real-IP` et tout autre en-tête sont **toujours** ignorés.
7. **AC-a7 — l'admission.** `s` admet le client si `s.address_filter_enabled = FALSE`, ou si le client appartient à une
   entrée de `s.allowed_networks` (`ipnet::IpNet::contains`). Filtre activé et liste vide → **personne**. Une entrée
   stockée qui ne s'analyse pas (`IpNet`/`IpAddr`) est **ignorée** — jamais élargissante — et journalisée
   (`tracing::warn!`, sans autre donnée que la colonne et l'id de société).
8. **AC-a8 — les tests réseau** (`mcp_reseau_e2e.rs`, réglages posés par le dépôt de la 32-5a, `enabled = TRUE`) ;
   « refusé » s'entend **réponse de route absente** (AC-a3), « admis » s'entend **401 sans clé** (puis 200 avec clé) :
   - (a) pair `8.8.8.8` → refusé ; (b) pair `192.168.1.10` → admis ; (c) pair `::ffff:8.8.8.8` → refusé et
     `::ffff:192.168.1.10` → admis (normalisation) ; (d) filtre désactivé, pair `8.8.8.8` → admis ;
   - (e) pair `8.8.8.8` non déclaré, `X-Forwarded-For: 192.168.1.10` **forgé** → refusé (en-tête ignoré) ; (f) pair
     `192.168.1.10` non déclaré, `X-Forwarded-For: 8.8.8.8` → admis (en-tête ignoré, le pair compte) ;
   - (g) pair `172.18.0.2` déclaré mandataire, `X-Forwarded-For: 192.168.1.20` → admis ; (h) même mandataire,
     `X-Forwarded-For: 8.8.8.8` → refusé ; (i) même mandataire, `X-Forwarded-For: 192.168.1.20, 8.8.8.8` (le client
     forge la partie gauche, le mandataire ajoute la vraie à droite) → **refusé** ; (j) deux mandataires en chaîne
     déclarés, `X-Forwarded-For: 192.168.1.20, 172.18.0.3` → admis ; (k) mandataire déclaré, en-tête absent → client =
     pair, refusé si le mandataire n'est pas dans les plages admises ; (l) mandataire déclaré, entrée `unknown` ou
     `8.8.8.8:443` → refusé ; (m) `X-Real-IP: 192.168.1.10` et `Forwarded: for=192.168.1.10` d'un pair public → refusé ;
   - (n) entrée stockée invalide (`"pas-un-réseau"`, écrite par SQL dans le test) à côté de `192.168.0.0/16` →
     `192.168.1.10` admis, `8.8.8.8` refusé ; (o) filtre activé, liste vide → `127.0.0.1` refusé.

**A.3 — L'`Origin` (C-32-6-7)**

9. **AC-a9 — validation de l'en-tête `Origin`**, après la règle « servi » et la méthode, **avant** l'authentification
   (spike § 1.1 : « validate the `Origin` header on all incoming connections »). Absent → admis. Présent → normalisé
   (schéma et hôte en minuscules, port par défaut retiré, aucun chemin admis) et admis s'il égale une **origine propre**
   — `config.public_base_url` réduit à son origine, `http://127.0.0.1:<port>`, `http://localhost:<port>`,
   `http://[::1]:<port>` (`config.port`, fait 10) — ou une entrée de `allowed_origins` d'un réglage de **S** ; après
   l'authentification, la même règle se rejoue avec les seules `allowed_origins` de la société de la clé. Sinon →
   **403**, `Content-Type: application/json`, corps JSON-RPC **sans `id`** à texte fixe. **Jamais déduite de `Host`**.
   Tests : `Origin` absent → 401 sans clé ; `http://localhost:<port>` → admis ; `https://evil.example` → 403 ;
   `Origin: null` → 403 ; `Origin` égale à `Host` forgé (`Host: evil.example`, `Origin: http://evil.example`) → 403 ;
   origine listée dans la société A, clé de B → 403 après authentification ; casse et port par défaut
   (`HTTP://LOCALHOST:<port>`, `https://kesh.example:443` quand `public_base_url = https://kesh.example`) → admis.

**A.4 — Le transport (C-32-6-8, C-32-6-18)**

10. **AC-a10 — l'enveloppe HTTP**, dans cet ordre (tests à chaque échelon, chacun avec le corps fixe attendu) :
    `Content-Type` qui n'est pas `application/json` (paramètres admis, casse ignorée) → **415** ; en-tête
    `MCP-Protocol-Version` présent et ni `2025-11-25` ni `2025-06-18` (dont `2026-07-28`) → **400**, `text/plain`,
    texte fixe, **jamais** un code `-32022` (spike § 1.3) ; absent → admis ; authentification (A.5) ; corps lu **après**
    l'authentification, borné à **1 Mio** (`MCP_CORPS_MAX`) → **413** au-delà ; JSON invalide → **400** et `-32700`
    sans `id` ; **tableau** (lot) → **400** et `-32600` sans `id` ; `jsonrpc` ≠ `"2.0"`, `id` nul ou ni chaîne ni
    nombre → `-32600` ; **message de type réponse** (`result` ou `error`, sans `method`) → **400** (Kesh n'envoie
    aucune requête au client : rien à accepter, spike § 5.3.3 L-4) ; **notification** (pas d'`id`) → **202**, corps
    vide, **jamais traitée par un gestionnaire de requête** — `{"method":"tools/call"}` sans `id` n'exécute aucun
    outil (test au compteur, AC-a20) ; requête → **200**, `application/json`, `Cache-Control: no-store`. L'en-tête
    `Accept` n'est pas contrôlé (C-32-6-18). Aucune réponse en `text/event-stream` (Q10 (c) : JSON partout).
11. **AC-a11 — les méthodes servies (E2), liste close.** Le répartiteur associe chaque `method` à une entrée d'une
    **énumération fermée** ; tout le reste → **`-32601`** à texte fixe. Servies : `initialize`, `ping`, `tools/list`,
    `tools/call`, `prompts/list`, `prompts/get` ; notifications acceptées : toute `notifications/*` (202). Test
    d'inventaire `methodes_servies_liste_close` : l'ensemble des méthodes acceptées égale la liste **écrite en dur dans
    le test** ; chaque méthode client du schéma `2025-11-25` qui n'y est pas rend `-32601` — liste tirée du schéma à
    l'étape 0 (T-a0) et citée avec sa source, comprenant au moins `resources/list`, `resources/read`,
    `resources/templates/list`, `resources/subscribe`, `resources/unsubscribe`, `completion/complete`,
    `logging/setLevel`, `tasks/get`, `tasks/list`, `tasks/result`, `tasks/cancel`, et un nom inventé.
12. **AC-a12 — `initialize`, `ping`, et l'inventaire des outils.** `initialize` répond `protocolVersion` = la version
    demandée si c'est `2025-11-25` ou `2025-06-18`, sinon `2025-11-25` (spike § 1.3) ; `serverInfo` `{name: "kesh",
    version: CARGO_PKG_VERSION}` ; `capabilities` = `tools` et `prompts` (`listChanged: false`) **seulement** —
    **aucune** `resources`, `logging`, `completions`, `tasks` (E3) ; `instructions` est une **constante** du binaire.
    `ping` → `{}`. Le cycle de vie n'est pas imposé (sans état, C-32-6-25) : `tools/list` sans `initialize` préalable
    est servi. Test `registre_standard_inventaire` : `Registre::standard()` porte **exactement** `["kesh_version"]` —
    la 32-9 l'amendera ; jusque-là, aucun outil de données ne peut s'y glisser sans faire rougir ce test.

**A.5 — La clé (reports de la 32-5)**

13. **AC-a13 — clé MCP seulement.** Le jeton se lit **uniquement** dans `Authorization: Bearer <jeton>` (schéma
    insensible à la casse, RFC 7235) ; **le cookie `kesh_access_token` est ignoré** ; il est passé à
    `services::cles_mcp::authentifier(&Lecture du MCP, &Trace, jeton)` (32-5b). **Même réponse 401** — statut, en-têtes
    hors `date`, octets du corps (fixe, JSON-RPC sans `id`, **aucun `WWW-Authenticate`**, C-32-6-27) — pour : en-tête
    absent, schéma autre que `Bearer`, jeton vide, **PAT actif `read-write`**, JWT valide en `Bearer`, cookie de session
    valide sans en-tête, clé inconnue, révoquée, expirée, créateur inactif ou rétrogradé, jeton d'un PAT derrière le
    préfixe `kesh_mcp_`. Tests : `pat_refuse_a_l_endpoint` (et `api_keys.last_used_at` du PAT **inchangée**, report
    (1) de la 32-5), `session_refusee`, `refus_indiscernables`. Une erreur de la base pendant l'authentification n'est
    pas un 401 : **500** à corps fixe (test : réglages actifs, puis `ALTER TABLE mcp_keys RENAME TO mcp_keys_cachee` → 500,
    et ni `mcp_keys_cachee` ni le texte de l'erreur `sqlx` dans le corps). Le jeton, l'en-tête et le corps ne sont **jamais**
    journalisés (`grep -rnE "jeton|token|authorization|corps|body" crates/kesh-api/src/mcp` relu ligne à ligne, cité au
    Dev Agent Record).
14. **AC-a14 — la politique de validation au répartiteur (Q12 ; frontière avec la 32-8).** `CleMcpAuthentifiee::
    validation` est lue par le répartiteur **et par lui seul** ; un outil ne la voit pas. Pour un `tools/call` d'un
    `McpTool` trouvé dans les portées : `Required` → **l'outil n'est pas exécuté**, réponse `result` avec
    `isError: true` et le texte fixe `TEXTE_VALIDATION_REQUISE` (« la validation avant envoi n'est pas encore
    disponible ; cette clé l'exige ») ; `Exempt` → l'outil s'exécute et son résultat part. Tests :
    `cle_requise_refusee_sans_execution` (outil de test à compteur : 0 exécution, aucune lecture), `cle_dispensee_servie`.
    Les méthodes de métadonnées (`initialize`, `ping`, `tools/list`, `prompts/*`) sont servies quelle que soit la
    politique (E2 : textes du binaire).

**A.6 — Les outils, les portées et l'absence d'oracle**

15. **AC-a15 — le trait `McpTool`** (`mcp/outil.rs`) : `nom() -> &'static str`, `description() -> &'static str`,
    `schema_entree() -> &'static str` (JSON **texte** du binaire : aucun canal par lequel une valeur lue entrerait dans
    `tools/list`, spike § 5.3.3 L-5), `portee() -> Portee` **sans implémentation par défaut**, `executer(&self, &ContexteMcp,
    serde_json::Value) -> Pin<Box<dyn Future<Output = Result<Donnees, ErreurOutil>> + Send + '_>>`. `ContexteMcp`
    porte `&ContexteAcces`, `&Lecture` (celle du MCP) et **rien d'autre** — ni `Trace`, ni `AppState`, ni pool —, et
    ne se construit que dans `mcp/repartiteur.rs` (champs privés, constructeur privé au module). Test
    `registre_coherent` : chaque outil du registre a un nom `[a-z0-9_]{1,64}` unique, un schéma qui s'analyse, de type
    `object`, avec `additionalProperties: false`, et une portée **différente de `PromptsWrite`**.
16. **AC-a16 — `tools/list` filtré.** Rend les seuls outils dont la portée appartient aux portées de la clé, dans
    l'ordre du registre, chacun avec `name`, `description`, `inputSchema` et `annotations: {readOnlyHint: true}` —
    l'annotation est posée **par le répartiteur**, non par l'outil. `cursor` fourni → `-32602` (Kesh n'émet aucun
    curseur). Tests : clé `books:read` → `kesh_version` listé et annoté ; clé `contacts:read` seule → liste vide.
17. **AC-a17 — hors portée = inconnu, sans oracle.** `tools/call` d'un outil inconnu, hors portée de la clé, ou dont le
    **service** refuse l'accès (`AppError::Forbidden`, fait 6, C-32-3-5) → **la même** erreur JSON-RPC `-32602` à texte
    fixe (`TEXTE_OUTIL_INCONNU`) ; test `hors_portee_indiscernable` : les trois corps sont égaux octet pour octet, `id`
    mis à part ; l'outil hors portée **n'est pas exécuté** (compteur) ; le cas « refus du service » passe par un outil
    de test de portée `BooksRead` qui appelle un service exigeant `InvoicesRead` (p. ex. `factures::lister`).
18. **AC-a18 — les arguments.** `params.arguments` absent vaut `{}` ; chaque outil lit ses arguments par une aide
    commune `arguments::<T: DeserializeOwned>()` (`deny_unknown_fields` exigé par la revue) qui rend
    `ErreurOutil::ArgumentsInvalides { argument: &'static str, motif }` — **jamais** le message de `serde`, qui recopie
    les noms et valeurs fournis. Réponse : `isError: true`, texte fixe par motif (C-32-6-17 ; la forme « erreur
    d'exécution » plutôt que `-32602` est à recontrôler à l'étape 0 contre la page `server/tools` de la révision
    `2025-11-25`). Test : `kesh_version` avec `{"PIEGE_ARG_326": 1}` → `isError: true`, `PIEGE_ARG_326` absent du corps.
19. **AC-a19 — les erreurs sans texte libre (E5).** `ErreurOutil` n'a **aucun** champ `String`, `Value` ni
    `Box<dyn Error>` ; chaque variante se traduit en un texte **fixe** (`mcp/textes.rs`, constantes) ; le détail
    d'une erreur interne part au journal du serveur (`tracing::error!`), jamais au client. Délai d'exécution d'un outil
    borné (`DELAI_OUTIL` = 30 s, C-32-6-19) → `isError: true`, texte fixe. Tests `erreur_interne_texte_fixe` :
    sous clé `exempt`, l'outil **piégé** (AC-a20) appelé après `ALTER TABLE accounts RENAME TO accounts_cachee_326` →
    `isError: true`, texte fixe, et **ni** `accounts_cachee_326` **ni** le message de MariaDB dans le corps (c'est la
    fuite du prototype, `proto-curl.out` l. 32). Un outil qui tenterait d'écrire n'est **pas exprimable** hors de
    `services` (le pool n'y est pas lisible, fait 5) : le refus 1792 se prouve à la fabrique (AC-a24).
20. **AC-a20 — le registre de test.** `kesh_api::build_router_avec_registre(state, static_dir, registre)`
    (`#[doc(hidden)]`) ; `build_router` l'appelle avec `Registre::standard()` ; `main.rs` n'appelle que `build_router`
    (garde lexicale, AC-a30). `Registre::vide().avec(Box<dyn McpTool>)` est public pour les tests. Outils de test
    (dans `tests/common/mcp.rs`, neuf) : **compteur** (compte ses exécutions), **piégé** (lit par
    `services::plan_comptable::lister` un compte dont le nom porte `PIEGE_E1_326`), **refus de service**
    (AC-a17), **lent** (dépasse `DELAI_OUTIL` réduit par un paramètre de test, sinon écrit comme non éprouvé).
21. **AC-a21 — E1 côté répartiteur.** Sous une clé **`required`**, la valeur `PIEGE_E1_326` n'apparaît dans **aucune**
    réponse d'**aucune** méthode servie (`initialize`, `ping`, `tools/list`, `tools/call` de l'outil piégé et de
    `kesh_version`, `prompts/list`, `prompts/get`, méthode inconnue, notification) — test `e1_rien_ne_part_sous_cle_requise` ;
    **témoin** : sous une clé `exempt`, `tools/call` de l'outil piégé la contient (le test ne passe pas à vide).
22. **AC-a22 — la construction de la réponse d'outil est restreinte (E6, mutation 2).** Le seul type qui porte des
    données d'outil vers le transport (`DonneesEnvoyables`) est **défini dans `mcp/politique.rs` avec un champ privé** :
    seul ce fichier le construit (le compilateur le tient) ; `politique::appliquer(validation, Donnees)` est sa seule
    fabrique. Les réponses statiques (`initialize`, `tools/list`, erreurs, `prompts/list`) ne se construisent qu'à partir
    de `&'static str` et des descripteurs du registre (`schema_entree` est `&'static str`). Garde lexicale
    `donnees_envoyables_construites_par_la_politique_seule` : `DonneesEnvoyables {` et `DonneesEnvoyables::` n'apparaissent
    dans `crates/kesh-api/src` qu'en `mcp/politique.rs`. Ce que cette restriction **ne ferme pas** est écrit au § *Ce que
    les gardes ne voient pas* (un gestionnaire de prompts qui recopierait des données : 32-10a).
23. **AC-a23 — les prompts (32-6 : le socle).** Trait `McpPrompt` (`mcp/prompts.rs`), **scellé** (`mod scelle { pub
    trait Scelle {} }` privé à `mcp::prompts`), `rendre` sans `Lecture` ; **aucune implémentation** en 32-6 (la 32-10a
    pose `PromptEnBase`). `prompts/list` → `{"prompts": []}` ; `prompts/get` de tout nom → `-32602` à texte fixe.
    Garde lexicale : `impl … McpPrompt for` n'apparaît qu'en `mcp/prompts.rs` (aucune en 32-6). La 32-10a amende ce
    trait en connaissance de cause (C-32-6-23).

**A.7 — La lecture seule (Q11, C-32-6-9 à C-32-6-11)**

24. **AC-a24 — le pool du MCP, une seule fabrique.** `kesh_db::pool::pool_lecture_seule(depuis: &MySqlPool,
    max_connections: u32) -> MySqlPool` : `MySqlPoolOptions` avec `acquire_timeout` = celui de `depuis`
    (`Pool::options`), `after_connect` = `SET SESSION TRANSACTION READ ONLY` (et **aucun autre réglage de session** :
    le pool de production n'en a pas, fait 4 — sinon une même lecture rendrait autre chose par le MCP que par l'API,
    C-32-6-9), `before_acquire` = `SELECT @@session.tx_read_only` → la connexion qui n'est plus à `1` est **jetée**,
    `connect_lazy_with((*depuis.connect_options()).clone())`. Appelée **une seule fois** en production, dans
    `build_router_avec_registre`, `max_connections` = `MCP_POOL_CONNEXIONS` (3, C-32-6-10) ; et par
    `tests/common/lecture_seule.rs`, qui **remplace** son `pool_lecture_seule` maison par elle (DRY : les tests des
    services éprouvent alors la fabrique de production). Tests `crates/kesh-db/tests/pool_lecture_seule.rs`, utilisateur
    `kesh`, base `#[sqlx::test]` : (a) `INSERT`, `UPDATE`, `DELETE` en autocommit → **1792** ; (b) les trois dans
    `pool.begin()` → 1792 ; (c) `SELECT … FOR UPDATE` → 1792 ; (d) `SELECT` → OK ; (e) **séquence B de la spike**
    (§ 4.2) : `max_connections(1)`, `SET SESSION TRANSACTION READ WRITE` sur une connexion, rendue, réacquise → un
    `UPDATE` est **refusé** (la connexion a été jetée par `before_acquire`), et l'identifiant de connexion a changé ;
    (f) une connexion perdue (`KILL`) est remplacée par une connexion qui repasse `after_connect`. ⚠️ (e) exécute
    volontairement une forme interdite : il est l'exception écrite de la garde de l'AC-a26.
25. **AC-a25 — le câblage : les outils lisent sur ce pool, la trace écrit sur l'autre.** La `Lecture` du MCP est
    `Lecture::new(pool_lecture_seule(&state.pool, MCP_POOL_CONNEXIONS))` en **un** site de `lib.rs` ; la `Trace` du MCP
    est `state.trace()`. `FABRIQUES_V2` de `services_motif.rs` gagne ce site (trois lignes), `EXCEPTIONS_V1` le chemin
    `kesh_db::pool::` qui fait rougir (v-1) par son `::pool` (fait 5), chacun avec sa ligne de commentaire. Tests :
    (i) `kesh_version_lit_en_lecture_seule` — sous une clé `exempt`, `kesh_version` rend `lectureSeule: true`, lu par
    le service sur la connexion même de la lecture (une seule requête, AC-a27) ; (ii)
    `l_appel_pose_la_derniere_utilisation` — après un `tools/call` servi, `mcp_keys.last_used_at` de la clé est posée
    (la `Trace` du MCP écrit : elle n'est pas sur le pool en lecture seule — `touch_last_used` étant *best-effort*,
    c'est ce test qui rend sa perte audible).
26. **AC-a26 — garde lexicale des formes qui défont le mode de session** (`mcp_gardes_lexicales.rs`,
    `lecture_seule_jamais_defaite`) : dans tous les `.rs` et `.sql` de `crates/` (squash compris), aucune occurrence de
    `(?i)read\s+write`, `(?i)tx_read_only\s*=`, `(?i)transaction_read_only`, hors de l'exception **écrite** (fichier,
    ligne) du test (e) de l'AC-a24 et du fichier de la garde lui-même. Témoin : la garde appliquée à une source factice
    contenant `SET SESSION TRANSACTION READ WRITE` rougit (fonction pure, motif d'`appels_non_handlers`, `audit_route_registry.rs:668-700`).
27. **AC-a27 — l'outil `kesh_version`** (`mcp/outils/kesh_version.rs`), portée `BooksRead`, schéma `{"type":"object",
    "properties":{},"additionalProperties":false}`, appelle `services::serveur_mcp::etat(&ContexteAcces, &Lecture)` —
    `exiger(EXIGENCE_ETAT)` d'abord (`role_api: Consultation`, `portee_mcp: Some(BooksRead)`), puis **une** requête
    `SELECT @@session.tx_read_only` sur `lecture.pool()` — et rend `structuredContent: {"version": …, "lectureSeule":
    bool}` et un bloc texte qui le sérialise. **Aucune donnée des livres.** La structure de l'outil est une unité sans
    champ (construite par `Registre::standard()`, sans pool).

**A.8 — L'inventaire et ses gardes**

28. **AC-a28 — l'inventaire clos des chemins vers un outil** (§ *L'inventaire clos*) est tenu par les gardes de
    l'AC-a30 ; chaque site **non résolu** y est soit fermé par une garde ou le compilateur, soit écrit comme angle mort.
29. **AC-a29 — aucune donnée réelle** : fixtures de démonstration et valeurs piégées seulement ; `~/kesh-test/` jamais
    ouvert.
30. **AC-a30 — gardes lexicales** (`mcp_gardes_lexicales.rs`, fonctions pures éprouvées sur une source factice,
    lignes de commentaire écartées) :
    - `un_seul_point_d_entree` : dans `crates/kesh-api/src`, `crate::mcp` / `mcp::` hors de `src/mcp/` n'apparaît qu'en
      `lib.rs` (déclaration du module, `build_router_avec_registre`) et `routes/mcp.rs` ; `routes::mcp::` qu'à la ligne
      de route de l'AC-a1 ;
    - `executer_appele_par_le_repartiteur_seul` : `.executer(` n'apparaît qu'en `mcp/repartiteur.rs` ; `.trouver(` et
      `.lister(` du registre aussi ;
    - `contexte_mcp_construit_par_le_repartiteur_seul` ;
    - `registre_standard_en_un_lieu` : `Registre::standard()` n'est appelé qu'en `lib.rs` ; `main.rs` n'appelle ni
      `build_router_avec_registre` ni `Registre::` ;
    - `outils_sans_pool` : les fichiers de `src/mcp/outils/` ne nomment ni `MySqlPool`, ni `sqlx`, ni `AppState`, ni
      `state.`, ni `kesh_db::repositories` (un outil passe par un service) ;
    - `un_seul_ecouteur` (AC-a1) ; `pool_lecture_seule_en_un_lieu` : appelé en `lib.rs` (une ligne) et en
      `tests/common/lecture_seule.rs` seulement ;
    - `donnees_envoyables_construites_par_la_politique_seule` (AC-a22) ; `prompts_scelles` (AC-a23) ;
      `lecture_seule_jamais_defaite` (AC-a26).

### Partie B — 32-6b : le journal des appels

31. **AC-b1 — chaque `tools/call` est journalisé**, servi ou non, après son issue et **avant** l'envoi de la réponse :
    une entrée `audit_log` d'action **`mcp.tool_called`**, type d'entité `mcp_key`, `entity_id` = l'id de la clé,
    `user_id` = le créateur de la clé, `actor_type = 'user'` (C-32-6-21 : l'`ENUM` n'a pas de valeur MCP, et en ajouter
    une serait un `MODIFY COLUMN` qu'un binaire antérieur ne décoderait pas, fait 8) ; détail **exactement**
    `{"cleMcpId": <id>, "outil": <nom>, "issue": <code>, "dureeMs": <entier>, "validation": "required"|"exempt"}`, où
    `outil` est le nom **d'un outil du registre**, ou la constante `"(inconnu)"` — **jamais** le nom fourni par le
    client s'il n'est pas au registre — et `issue` ∈ {`servi`, `validation_requise`, `inconnu`, `arguments_invalides`,
    `erreur`, `delai_depasse`} (énumération fermée ; `inconnu` couvre hors portée et refus de service, sans oracle
    **pour le client** — le journal, lu par l'administrateur, ne distingue pas non plus : C-32-6-21). Écrit par
    `services::serveur_mcp::journaliser_appel(&Trace, …)` — seule fonction de `services/` neuve qui emploie
    `trace.pool()` ; le doc-comment de `Trace::pool` (`poignees.rs:46-49`) la nomme.
32. **AC-b2 — sans arguments ni données.** Test `journal_sans_arguments_ni_donnees` : `tools/call` de l'outil piégé
    (clé `exempt`) avec un argument `PIEGE_ARG_326` et un nom d'outil inconnu `PIEGE_NOM_326` → aucun des trois pièges
    (`PIEGE_E1_326` rendu, `PIEGE_ARG_326`, `PIEGE_NOM_326`) dans `audit_log.details_json` ni `actor_label` ; chaque
    entrée a **exactement** les cinq clés de l'AC-b1.
33. **AC-b3 — le journal fail closed (C-32-6-30).** Si l'écriture du journal échoue, **rien ne part** : la réponse devient
    `isError: true` à texte fixe, sans `structuredContent` ni donnée. Test `journal_indisponible_rien_ne_part` : pool de
    l'application fermé après la construction du routeur (la `Lecture` du MCP, pool distinct, reste ouverte ;
    `touch_last_used` échoue en silence, *best-effort*) → `tools/call` de l'outil piégé sous clé `exempt` ne rend pas
    `PIEGE_E1_326`.
34. **AC-b4 — ce qui n'est pas journalisé, et pourquoi** (C-32-6-24) : `initialize`, `ping`, `tools/list`,
    `prompts/*` (métadonnées du binaire ; la 32-10a journalisera `prompts/get` quand il servira un texte), les
    notifications, et **les refus avant authentification** (route absente, 401, 403, 405, 413, 415) — au **journal du
    serveur** seulement (`tracing`), pour qu'une requête non authentifiée ne puisse pas écrire en base. Test : un 401 et
    un `tools/list` n'ajoutent aucune ligne à `audit_log`.
35. **AC-b5 — libellés.** `mcp.tool_called` à `audit_labels::ACTIONS` (ordre trié), libellé dans les **quatre**
    locales (`audit-log-action-…`, repli égal à la valeur fr-CH) ; `audit_label_registry.rs` vert ; l'action est un
    **littéral** au site `NewAuditLogEntry::user(` (règle R4 de `services`).

## L'inventaire clos des chemins par lesquels une requête atteint un outil

*Règle de `CLAUDE.md` (§ Inventorier les sites NON RÉSOLUS) : non pas les formes qui marchent, mais l'ensemble clos des
sites qui ne résolvent pas, chacun fermé ou écrit comme angle mort.* L'ensemble considéré : **tout appel de
`McpTool::executer`**, et **tout chemin qui ferait partir une donnée lue par un outil**.

**Le seul chemin servi** : écouteur de `main.rs` → routeur de `build_router` → `.route("/mcp", post(routes::mcp::
recevoir))` → `mcp::transport` (règle « servi » → méthode → `Origin` → `Content-Type` → `MCP-Protocol-Version` → clé →
société de la clé servie et son `Origin` → corps borné → JSON-RPC) → `mcp::repartiteur` (méthode de la liste close →
`tools/call` → `Registre::trouver(nom, portées)` → politique de la clé → `executer` sous délai → `politique::appliquer`
→ [32-6b : journal] → réponse).

| # | Site non résolu | Comment il se ferme | Par quoi |
|---|---|---|---|
| N1 | une autre route, un `nest`/`merge`, une tâche de fond (`auto_backup`), un point de test (`/api/v1/_test`) qui appellerait `crate::mcp` | aucun code hors de `src/mcp/`, `lib.rs`, `routes/mcp.rs` ne nomme `mcp::` | garde `un_seul_point_d_entree` (AC-a30) |
| N2 | un second écouteur (port dédié, socket) | une seule ligne `TcpListener::bind`, celle de `main.rs` | garde `un_seul_ecouteur` |
| N3 | un appel de `executer` hors du répartiteur | `.executer(` en `mcp/repartiteur.rs` seulement | garde `executer_appele_par_le_repartiteur_seul` |
| N4 | `trouver`/`lister` court-circuités, un outil obtenu sans filtre de portée | seuls points d'accès au registre, en `repartiteur.rs` ; `outils` privé au `Registre` | garde ; champ privé (compilateur) |
| N5 | une méthode JSON-RPC servie hors liste (`resources/read`…) | énumération fermée, repli `-32601` | test d'inventaire (AC-a11) |
| N6 | une **notification** `tools/call` (sans `id`) | jamais remise à un gestionnaire de requête : 202 | test au compteur (AC-a10) |
| N7 | un **lot** dont un élément serait traité | 400 avant tout traitement | test (AC-a10) |
| N8 | un message de type réponse | 400, jamais traité | test (AC-a10) |
| N9 | une méthode HTTP autre que `POST` (`GET`, `OPTIONS`…) | `autre_methode` ne rend que route absente ou 405 ; n'appelle pas le répartiteur | test (AC-a3, AC-a5) ; garde N3 |
| N10 | un chemin voisin (`/mcp/`, `/MCP`, `/%6Dcp`, `//mcp`) | soit route absente, soit, si Axum le résout vers `/mcp`, **le même gestionnaire** et toutes ses gardes | test `chemins_voisins` (réponse = route absente ou garde complète, jamais un outil sans clé) |
| N11 | l'authentification par cookie de session, par PAT, par JWT | le transport ne lit que `Bearer` et `authentifier` refuse tout préfixe autre que `kesh_mcp_` | tests (AC-a13) |
| N12 | une clé dont la société ne sert pas, ou dont le filtre refuse le pair | contrôle après authentification | tests (AC-a4) |
| N13 | un pair déguisé par `X-Forwarded-For`, `X-Real-IP`, `Forwarded` | en-têtes lus seulement d'un mandataire déclaré, de droite à gauche ; les deux autres jamais | tests (AC-a8) |
| N14 | `ConnectInfo` absent | route absente | test (AC-a6) |
| N15 | une clé `required` qui ferait exécuter un outil | refus avant `executer` | test au compteur (AC-a14) |
| N16 | des données d'outil envoyées hors de la politique | `DonneesEnvoyables` constructible en `politique.rs` seul | compilateur (champ privé) + garde (AC-a22) + test E1 (AC-a21) |
| N17 | une donnée lue logée dans une erreur | `ErreurOutil` sans champ libre ; textes fixes | type + test (AC-a19) |
| N18 | une donnée lue logée dans `tools/list` | `nom`, `description`, `schema_entree` sont `&'static str` | type (AC-a15) |
| N19 | un outil qui capturerait un pool ou l'état à sa construction | outils construits par `Registre::standard()`, unités sans champ | garde `outils_sans_pool` + revue |
| N20 | un outil qui écrirait | un outil n'a que `ContexteAcces` et la `Lecture` du MCP, dont le pool n'est lisible que dans `services` ; la connexion refuse l'écriture | compilateur (`pub(in crate::services)`) + `READ ONLY` + `before_acquire` (AC-a24) — **angle mort** : voir ci-dessous |
| N21 | un refus de service (`Forbidden`) qui révélerait l'existence d'un outil | traduit en « outil inconnu » | test (AC-a17) |
| N22 | un prompt qui porterait des données lues | aucun prompt en 32-6 ; `rendre` sans `Lecture`, trait scellé | garde `prompts_scelles` — **à reprendre par la 32-10a** |
| N23 | `unsafe` (transmutation d'une poignée) | `forbid(unsafe_code)` sur la lib (`lib.rs:9`) | compilateur |

**Angles morts assumés** (écrits, non fermés) — chacun au § *Ce que les gardes ne voient pas* : une écriture **dans un
même emprunt** après une forme qui défait le mode de session (N20) ; une forme interdite **composée à l'exécution** ou
émise par une dépendance (N20) ; un service de `services/` qui emploierait `lecture.pool()` pour écrire (N20, vu par
l'exécution seulement) ; une macro qui fabriquerait un identifiant (N1, N3) ; la configuration de Traefik et de Docker
(N13).

## La frontière avec la 32-8

| | 32-6 | 32-8 |
|---|---|---|
| Lire la politique de la clé | ✅ au répartiteur, seul lecteur | — |
| `Required` | **refus** avant exécution, texte fixe (AC-a14) | **remplace** ce refus par la poignée : exécuter, déposer la proposition, rendre « envoi n° X en attente » |
| `Exempt` | exécution, envoi ; journal (32-6b) | inchangé (« seule voie hors validation », E1) |
| Table des envois, migration, sauvegarde | — | ✅ (🗄️) |
| `kesh_lire_envoi`, règle « même clé, même société », échéance, purge | — | ✅ (E4) |
| Écran de validation, représentation source (`content` ou `structuredContent`) | — | ✅ (E1, M-2 de la spike) |
| Mutations d'E6 | 1, 2, 3, 5 | 4, 6 |
| `instructions` d'`initialize` | constante neutre | ajoute la consigne de relecture |
| `kesh_version` sous validation | refusé sous `required` (Q12 : la politique est celle de la clé, jamais de l'outil) | « l'outil de démonstration y passe » (kickoff) |

Le point de branchement est **une seule fonction**, `politique::appliquer` : la 32-8 change son bras `Required` et rien
d'autre du répartiteur.

## Tâches

### 32-6a
- [ ] **T-a0** — étape 0 (§ *À recontrôler*) : 32-5b, 32-5d, 32-5c fusionnées ; `git log -1` partie de `main` à jour ;
  recomptes et relectures, ligne lue citée au Change Log ; la liste des méthodes client du schéma `2025-11-25` (source
  citée) ; l'observation (h) de la spike § 2.4 faite ou non.
- [ ] **T-a1** — `kesh-db` : `pool_lecture_seule` et ses tests (AC-a24) ; `list_enabled` et ses tests (AC-a2).
- [ ] **T-a2** — `mcp/` : protocole JSON-RPC, transport, textes fixes, traits, registre, politique, répartiteur,
  prompts, `kesh_version` (AC-a10 à AC-a23, AC-a27) ; `services::serveur_mcp` (`reglages_actifs`, `etat`).
- [ ] **T-a3** — garde réseau : client, mandataires, admission, `Origin` (AC-a6 à AC-a9) ; tests unitaires de
  l'analyse (`mcp/reseau.rs`) et `mcp_reseau_e2e.rs`.
- [ ] **T-a4** — câblage `lib.rs` (route, `service_statique`, pool et `Lecture` du MCP, `build_router_avec_registre`),
  `routes/mcp.rs` ; registre des routes ; `services_motif.rs` (v-1, v-2, vi) ; `tests/common/lecture_seule.rs` (AC-a1
  à AC-a5, AC-a25).
- [ ] **T-a5** — `mcp_transport_e2e.rs`, `tests/common/mcp.rs`, gardes lexicales (AC-a13, AC-a14, AC-a17 à AC-a21,
  AC-a26, AC-a30).
- [ ] **T-a6** — mutations Ma1 à Ma30, gates ; Dev Agent Record (n'affirmer que ce qui a tourné, avec périmètre).

### 32-6b
- [ ] **T-b0** — étape 0 : 32-6a fusionnée ; recontrôle des faits 8, 9 et de `audit_labels.rs`.
- [ ] **T-b1** — `journaliser_appel`, appel au répartiteur, fail closed (AC-b1, AC-b3, AC-b4).
- [ ] **T-b2** — libellés, quatre locales (AC-b5) ; `mcp_journal_e2e.rs` (AC-b2 à AC-b4).
- [ ] **T-b3** — mutations Mb1 à Mb6, gates.

## Tests et mutations prévus

Une mutation à la fois, restaurée par `git checkout -- <f> && touch <f>` (`memory/mutation-restauree-binaire-perime`),
diff résiduel nul vérifié ; chacune nomme le test qui doit rougir. Une mutation qui survit se déclare ; une mutation qui
**ne compile pas** se déclare comme telle (la propriété est alors tenue par le compilateur) et ne compte pas comme tuée.

| # | Mutation | Doit rougir |
|---|---|---|
| Ma1 | retirer l'`after_connect` `READ ONLY` de `pool_lecture_seule` | `pool_lecture_seule` (a)–(c) ; `kesh_version_lit_en_lecture_seule` |
| Ma2 | retirer le `before_acquire` | `pool_lecture_seule` (e) (séquence B) |
| Ma3 | `Lecture` du MCP sur `state.pool` (pool ordinaire) | `kesh_version_lit_en_lecture_seule` ; garde v-2 |
| Ma4 | `Trace` du MCP sur le pool en lecture seule | `l_appel_pose_la_derniere_utilisation` (en 32-6b, aussi `chaque_appel_est_journalise`) |
| Ma5 | `tools/list` non filtré par les portées | `tools_list_filtre` |
| Ma6 | `trouver` ignore la portée | `hors_portee_indiscernable` (compteur) |
| Ma7 | hors portée → message distinct (« outil hors portée ») | `hors_portee_indiscernable` (égalité des corps) |
| Ma8 | `Forbidden` d'un service rendu tel quel | `hors_portee_indiscernable` (cas service) |
| Ma9 | filtre d'adresse retiré (tout pair admis) | `mcp_reseau_e2e` (a), (c) |
| Ma10 | `X-Forwarded-For` lu d'un pair non déclaré | (e) |
| Ma11 | `X-Forwarded-For` lu de gauche à droite | (i) |
| Ma12 | adresse mappée non normalisée | (c) |
| Ma13 | entrée stockée invalide traitée comme « tout admettre » (ou faisant échouer tout le filtre) | (n) |
| Ma14 | `X-Real-IP` ou `Forwarded` lu | (m) |
| Ma15 | vérification d'`Origin` retirée | tests de l'AC-a9 (403) |
| Ma16 | `Origin` admise si égale à `Host` | AC-a9 (`Host` forgé) |
| Ma17 | **un PAT accepté** (`kesh_pat_` routé vers `validate_pat`) | `pat_refuse_a_l_endpoint` |
| Ma18 | cookie de session lu | `session_refusee` |
| Ma19 | refus d'authentification à corps distincts selon la cause | `refus_indiscernables` |
| Ma20 | désactivé → `404` littéral (au lieu de déléguer au service statique) | `desactive_indiscernable_d_une_route_absente` |
| Ma21 | contrôle « société de la clé servie » retiré | AC-a4 (i), (ii) |
| Ma22 | erreur de base dans la règle « servi » → 500 | AC-a2 (fail closed, table renommée) |
| Ma23 | `Required` laisse passer (bras de `politique::appliquer` intervertis) | `cle_requise_refusee_sans_execution` ; `e1_rien_ne_part_sous_cle_requise` (E6-1) |
| Ma24 | `DonneesEnvoyables` à champ `pub` construit dans le répartiteur | garde `donnees_envoyables_…` (E6-2) |
| Ma25 | `resources/list` servi (rend `[]`) | `methodes_servies_liste_close` (E6-3) |
| Ma26 | `ErreurOutil::Interne(String)` portant `e.to_string()` | `erreur_interne_texte_fixe` (E6-5) |
| Ma27 | notification `tools/call` exécutée | test au compteur (AC-a10) |
| Ma28 | lot : premier élément traité | AC-a10 (lot) |
| Ma29 | `MCP-Protocol-Version: 2026-07-28` admis | AC-a10 (version) |
| Ma30 | `readOnlyHint` retiré | `tools_list_filtre` (annotation) |
| Mb1 | appel de `journaliser_appel` retiré | `chaque_appel_est_journalise` |
| Mb2 | arguments recopiés au détail | `journal_sans_arguments_ni_donnees` |
| Mb3 | nom d'outil fourni par le client écrit au journal | `journal_sans_arguments_ni_donnees` (`PIEGE_NOM_326`) |
| Mb4 | journal *best-effort* (échec avalé, réponse envoyée) | `journal_indisponible_rien_ne_part` |
| Mb5 | journal écrit sur la `Lecture` du MCP | `chaque_appel_est_journalise` (1792, puis fail closed) |
| Mb6 | un 401 journalisé en base | AC-b4 |

Tests hors mutations, à nommer au Dev Agent Record : `chemins_voisins` (N10), `registre_coherent`, `registre_standard_
inventaire`, tests unitaires de `mcp/reseau.rs` (analyse de `X-Forwarded-For`, normalisation d'origine, appartenance).

## Gates

- **32-6a : 🗄️** (un dépôt de `kesh-db` et sa fabrique de pool) — **gate complet même en cours de boucle**
  (`CLAUDE.md`, exception `kesh-db`) ; base remise à zéro avant chaque gate complet (bases `kesh_326`/`kesh_e2e_326`,
  **sans** redémarrer le conteneur, `docs/agents-orchestration.md` § 2 et § 4), sous `flock …/.gate.lock`, par
  `scripts/test-fast.sh --ci` ; fmt, clippy `-D warnings`, build `--all-targets`. **Aucune migration** : P1 à P8 sans
  objet (à constater : `ls crates/kesh-db/migrations | wc -l` inchangé), P6 sans objet.
- **32-6b** : gate ciblé entre les passes (`binary(mcp_journal_e2e)`, `binary(audit_label_registry)`, fmt et clippy
  sur le workspace), complet au push ; frontend `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` (les quatre
  locales sont lues par le frontend).
- **E2E complet au dernier commit de code de chaque partie** : le routeur change (`/mcp` avant le repli statique) ; les
  échecs attendus se jugent fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **Recette visuelle : sans objet** — aucun contrôle visible ne change (le `GET /mcp` désactivé rend la page de
  l'application, comme toute route absente).
- **Boucle de revue complète, Sonnet ↔ Opus** pour chaque partie (kickoff : risque élevé) ; Haiku en passe ciblée
  seulement.

## Ce que les gardes ne voient pas

- **La lecture seule n'est pas une propriété de la base** (Q11). Le mode de session se défait par quatre instructions
  sans privilège (spike § 4.1) ; `before_acquire` ferme le trou **entre deux emprunts**, pas **dans** un emprunt
  (`START TRANSACTION READ WRITE; UPDATE …`) ; la garde lexicale ne lit que le texte de `crates/` — ni un SQL composé à
  l'exécution, ni une dépendance. Un service qui écrirait par `lecture.pool()` n'est arrêté que par MariaDB, **sur les
  seuls chemins exécutés**.
- **Le filtre d'adresse voit le pair que voit Kesh, non le client réel.** Derrière Traefik, le pair est Traefik : sans
  le déclarer mandataire de confiance, tous les clients paraissent venir de son adresse (privée, donc admise par les
  plages par défaut). Derrière une publication de port Docker par le mandataire en espace utilisateur
  (`userland-proxy`), le pair peut être la passerelle du pont Docker (`172.17.0.1`, privée) : **tout le monde est
  admis**. Kesh ne peut pas le détecter. **Non vérifié sur le NAS** — point 2 pour Guy, texte de la 32-7.
- **Traefik et `X-Forwarded-For`** : selon sa configuration (`forwardedHeaders.trustedIPs`), Traefik conserve ou
  écrase l'en-tête reçu du client ; la lecture de droite à gauche est juste dans les deux cas, **à condition** que seul
  Traefik soit déclaré. Non éprouvé contre un Traefik réel.
- **`Origin` ne protège que d'un navigateur.** Un client hors navigateur n'envoie pas d'`Origin`, ou en forge une ; la
  vraie barrière est la clé. `Host` n'est pas contrôlé (le *DNS rebinding* sans clé échoue au 401).
- **Indiscernable, mais pas invisible au temps de réponse** : la décision « servi » lit la base ; une route absente non.
  Un écart de quelques millisecondes peut trahir l'existence de réglages MCP (oracle temporel, non fermé). Une
  requête non authentifiée provoque une lecture en base (aucune limite de débit sur `/mcp`, comme sur `/health`).
- **Une donnée recopiée dans un prompt** par un humain ou un agent approuvé (cas assumé d'E1) : la 32-10a.
- **La restriction de construction** (AC-a22) ne voit pas un gestionnaire qui sérialiserait des `Donnees` dans un
  texte **statique fabriqué à l'exécution** — impossible tant que les réponses statiques ne prennent que des
  `&'static str`, ce qu'une revue vérifie à chaque ajout de méthode ; `Box::leak` le permettrait (revue).
- **Le comportement des clients réels** (protocole demandé, `Origin` envoyé, tentative OAuth sur un 401 — un `GET
  /.well-known/oauth-protected-resource` rend la page de l'application, que le client ne saura pas lire) : spike
  § 2.4, à observer par Guy ou la 32-11.
- **Le journal (32-6b) ne distingue pas** « outil inconnu » de « hors portée » : c'est voulu (C-32-6-21), et c'est une
  perte de diagnostic pour l'administrateur.
- **Le manuel** : aucun gate ne le lit ; les prompts de revue nomment l'axe.

## Dev Notes

### Ordre des contrôles d'une requête (C-32-6-8)

```text
1. pair (ConnectInfo, to_canonical)                       absent              → route absente
2. S = réglages actifs admettant le client (Lecture MCP)  vide ou erreur base → route absente
3. méthode HTTP                                           ≠ POST              → 405 Allow: POST
4. Origin (propres ∪ allowed_origins de S)                refusée             → 403 JSON-RPC sans id
5. Content-Type                                           ≠ application/json  → 415
6. MCP-Protocol-Version                                   non servie          → 400 text/plain
7. Authorization: Bearer → cles_mcp::authentifier         refus               → 401 fixe ; erreur base → 500 fixe
8. société de la clé ∈ S                                  non                 → route absente
9. Origin (propres ∪ allowed_origins de la société)       refusée             → 403
10. corps ≤ 1 Mio, JSON-RPC                               (AC-a10)
11. répartiteur : liste close → tools/call → trouver(nom, portées) → politique → executer (30 s) → appliquer
    → [32-6b] journaliser_appel (fail closed) → 200
```

### Esquisse des modules (indicative dans la forme, exigée dans le fond)

```text
crates/kesh-api/src/mcp/
  mod.rs          — doc du module : l'inventaire clos, la frontière 32-8, l'ordre des contrôles
  transport.rs    — étapes 1 à 10 ; aucune donnée d'outil ne s'y construit
  reseau.rs       — client depuis pair + X-Forwarded-For, admission, normalisation d'origine (fonctions pures)
  protocole.rs    — types JSON-RPC (requête, notification, erreurs à `&'static str`)
  repartiteur.rs  — liste close des méthodes, seul appelant de `executer`, construit `ContexteMcp`
  politique.rs    — `DonneesEnvoyables` (champ privé), `appliquer(validation, Donnees)`
  registre.rs     — `Registre::{standard, vide, avec}`, `trouver`, `lister`
  outil.rs        — `McpTool`, `ContexteMcp`, `Donnees`, `ErreurOutil`, `arguments::<T>()`
  prompts.rs      — `McpPrompt` scellé, catalogue vide
  textes.rs       — toutes les chaînes envoyées au client (constantes)
  outils/{mod.rs, kesh_version.rs}
```

```rust
// mcp/outil.rs — indicatif
pub trait McpTool: Send + Sync + 'static {
    fn nom(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema_entree(&self) -> &'static str;   // JSON texte, validé par `registre_coherent`
    fn portee(&self) -> Portee;                // SANS défaut ; jamais PromptsWrite (test)
    fn executer<'a>(&'a self, ctx: &'a ContexteMcp<'a>, arguments: serde_json::Value)
        -> Pin<Box<dyn Future<Output = Result<Donnees, ErreurOutil>> + Send + 'a>>;
}
pub enum ErreurOutil { ArgumentsInvalides { argument: &'static str, motif: MotifArgument }, Introuvable, Interne }

// mcp/politique.rs — indicatif
pub struct DonneesEnvoyables { donnees: Donnees }   // champ privé : construit ICI seulement
pub(crate) fn appliquer(validation: SendValidation, d: Donnees) -> Result<DonneesEnvoyables, Refus> { /* … */ }
```

⚠️ Le futur boxé rend le trait compatible `dyn` (registre hétérogène) sans `async-trait` (spike § 5.3.2, note).
⚠️ Visibilité : un type défini dans `politique.rs` à champ privé n'est constructible que dans ce module et ses
descendants — c'est le compilateur qui tient l'AC-a22 ; la garde lexicale nomme le rouge si le champ devenait public.
⚠️ `McpPrompt` scellé : le module `scelle` privé à `mcp::prompts` n'exclut que ce qui n'en descend pas (spike § 5.3.3
L-5) ; `mcp::outils` est un frère, il ne peut pas l'implémenter ; la garde `prompts_scelles` tient le reste.

### Le pool du MCP (C-32-6-9)

```rust
// kesh-db/src/pool.rs — indicatif
pub fn pool_lecture_seule(depuis: &MySqlPool, max_connections: u32) -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(depuis.options().get_acquire_timeout())
        .after_connect(|conn, _| Box::pin(async move {
            conn.execute("SET SESSION TRANSACTION READ ONLY").await?; Ok(())
        }))
        .before_acquire(|conn, _| Box::pin(async move {
            let ro: i64 = sqlx::query_scalar("SELECT @@session.tx_read_only").fetch_one(&mut *conn).await?;
            Ok(ro == 1)   // false : la connexion est jetée
        }))
        .connect_lazy_with((*depuis.connect_options()).clone())
}
```

Construit dans `build_router_avec_registre` (synchrone grâce à `connect_lazy_with`) : **aucun champ neuf dans
`AppState`**, donc aucun des six sites de test qui le construisent par littéral (`password_recovery_e2e.rs:127`,
`invoice_frozen_pdf_e2e.rs:102`, `setup_admin_e2e.rs:81`, `invoice_send_email_e2e.rs:103`, `:1035`, `:1149`) ni
`middleware/auth.rs:267` ne change (fiche 32-3, l. 782-783, l'avait signalé). Le même chemin sert la production et les
tests : `main.rs` appelle `build_router` (`main.rs:385`), qui fabrique le pool. ⚠️ `@@session.tx_read_only` **lu**
n'est pas une forme interdite par la garde de l'AC-a26 (`tx_read_only\s*=`) ; `SET SESSION TRANSACTION READ ONLY`
non plus (`read\s+write`). ⚠️ #669 reste à part : le pool du MCP **copie** les réglages de session de la production
(aucun), pour que l'API et le MCP lisent la même chose ; le jour où #669 change la production, il change ce pool par la
même fabrique (à écrire dans #669 par l'orchestrateur, demande 3).

### Lignes de code lues et citées

`lib.rs:9` (`forbid(unsafe_code)`), `:153-155` et `:162-164` (`lecture()`, `trace()` sur `self.pool`), `:213-214`
(repli statique), `:1165-1172` (`protected` et `require_auth`), `:1174-1183` (routes publiques), `:1204`, `:1225-1227`
(repli, CSP, état) ; `middleware/auth.rs:83-100` (cookie puis `Bearer`), `:134` (préfixe PAT), `:141`, `:172` ;
`middleware/csp.rs:52-60` ; `main.rs:74-78` (pool sans `after_connect`), `:385`, `:397-400` (`connect_info`) ;
`kesh-db/src/pool.rs:28-49` ; `services/poignees.rs:21-33`, `:38-52` ; `services/acces.rs:76-86`, `:137-143`,
`:166-176` ; `services/mod.rs:207` ; `tests/common/lecture_seule.rs:17-32` ; `tests/services_motif.rs:1886`, `:1888-1898`,
`:2539-2546`, `:2559-2563` ; `tests/audit_route_registry.rs:529`, `:540-567`, `:668-700`, `:768`, `:1129` ;
`entities/mcp_server_settings.rs:18-40`, `:44-55`, `:59-70` ; `repositories/mcp_server_settings.rs:69-79`, `:83-90` ;
`repositories/audit_log.rs:94-95` ; `entities/audit_log.rs:38-62` ; `migrations/20260605000002_audit_log_actor.sql:23` ;
`config.rs:180-181`, `:334`, `:1089-1091` ; `tower-http-0.6.11/src/services/fs/serve_dir/mod.rs:317-327`,
`future.rs:172-176`, `:185-190` ; `axum-0.8.9/src/extract/connect_info.rs:163-164` ; `sqlx-core-0.8.6/src/pool/mod.rs:545`, `:567`,
`pool/options.rs:380`, `:435`, `:579` ; `password_recovery_e2e.rs:690-703` ; `test_endpoints_e2e.rs:97-104` ;
`user-manual.tex:190` ; `docs/api-external.md:183`.

### Fichiers touchés (prévus)

- **32-6a** : `crates/kesh-db/src/pool.rs` ; `crates/kesh-db/src/repositories/mcp_server_settings.rs` ;
  `crates/kesh-db/tests/pool_lecture_seule.rs` (neuf) et le fichier de tests du dépôt de réglages (recompté à T-a0) ;
  `crates/kesh-api/src/mcp/**` (neuf) ; `crates/kesh-api/src/routes/{mcp.rs (neuf), mod.rs}` ;
  `crates/kesh-api/src/lib.rs` ; `crates/kesh-api/src/services/{serveur_mcp.rs (neuf), mod.rs, poignees.rs (doc)}` ;
  `crates/kesh-api/tests/{mcp_transport_e2e.rs, mcp_reseau_e2e.rs, mcp_gardes_lexicales.rs, common/mcp.rs (neufs),
  common/mod.rs, common/lecture_seule.rs, services_motif.rs, audit_route_registry.rs}`.
- **32-6b** : `crates/kesh-api/src/mcp/repartiteur.rs` ; `crates/kesh-api/src/services/{serveur_mcp.rs,
  poignees.rs (doc)}` ; `crates/kesh-api/src/audit_labels.rs` ;
  `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ; `crates/kesh-api/tests/mcp_journal_e2e.rs` (neuf).

### Le manuel (axe de revue)

- **Ce que la 32-6 dit au manuel : rien.** Aucune ligne des manuels ne décrit `/mcp` (fait 12) ; la 32-6 n'en écrit
  pas. Elle **vérifie** qu'aucun texte ne devient faux : `docs/api-external.md:183` (borné à « v0.2 », vrai),
  `admin-manual.tex:2925`, la brochure (`marketing-brochure.tex:480`) — tous à la 32-11, avant la release ; et que les
  passages de la 32-5 (`admin-manual.tex:2230`, `:2237`, `user-manual.tex:3028-3030`) restent vrais. Contrôle :
  `grep -n "MCP" docs/manual/fr/*.tex docs/api-external.md` et le PDF aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '
  | grep -oiE ".{60}MCP.{80}"`), sorties citées au Dev Agent Record.
- **Ce qu'elle laisse à la 32-7** (section *Le serveur MCP* du manuel d'administration), à reprendre **tel que le code
  le fait** : désactivé, `/mcp` se comporte comme une route absente (C-32-6-2) ; l'activation est par société ; le
  filtre porte sur le pair TCP, `X-Forwarded-For` n'est lu que des mandataires déclarés, de droite à gauche ; refusé,
  le client voit une route absente ; filtre activé et liste vide = personne ; une entrée invalide est ignorée ;
  l'`Origin` et ses origines propres (`KESH_PUBLIC_BASE_URL`, `localhost`/`127.0.0.1`/`[::1]` au port de Kesh) ;
  **ce que le filtre ne garantit pas** (Traefik à déclarer, passerelle Docker, pas un pare-feu — § *Ce que les gardes
  ne voient pas*) ; une clé `required` ne peut appeler aucun outil avant la 32-8 ; corps limité à 1 Mio, délai de 30 s
  par outil ; ce qui est journalisé (32-6b, AC-b1 à AC-b4) et ce qui ne l'est pas ; un appel MCP **écrit** (dernière
  utilisation, journal) — à ne pas présenter comme « une lecture n'écrit rien » (#730).

### Références

`planning-artifacts/epic-32-serveur-mcp.md` (§ *32-6*, *Ce que le code dit déjà* 2, 6, 7, 8, *Points d'attention*,
*Réponses de Guy*) ; `planning-artifacts/epic-32-spike-mcp.md` (§ 1.1, 1.3, 2.4, 4.1-4.4, 5.3, 6, 6.2, 7 et ses
réponses) ; registres `epic-32-choix-autonomes.md` (C-32-0-6 à 9, 12), `-322.md` (C-32-2-1 à 3, 16, 21, 24),
`-323.md` (C-32-3-5, 22, 30), `-325.md` (C-32-5-5, 7 à 11, 14, 19, 32, 37) ; fiches `32-3-services-motif.md`,
`32-4a-*`, `32-4b-*`, `32-5-cles-et-reglages-mcp.md` (§ *Ce que reçoivent les stories suivantes*, *Pour la 32-5b*) ;
`CLAUDE.md` (*Review Iteration Rule*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Pendant une
boucle de revue* et son exception `kesh-db`, *Recompter*, *Recette visuelle*) ; `docs/agents-orchestration.md`.

## Points à soumettre à Guy

1. **« Désactivé → 404 » ou « indiscernable d'une route absente » ?** Aujourd'hui une route absente ne rend pas 404 :
   un `POST` rend **405** (`Allow: GET,HEAD`, corps vide) et un `GET` la page de l'application (fait 1). Un `/mcp`
   désactivé qui rendrait 404 serait donc **reconnaissable** — exactement l'oracle que la règle veut éviter. La fiche
   retient l'**indiscernabilité** (C-32-6-2 : `/mcp` désactivé délègue au même service que toute route absente). Si tu
   veux un vrai 404 partout, c'est le repli de **toute** l'application qui change (toute méthode autre que `GET` vers un
   chemin absent → 404) : hors de cette story, une issue à ouvrir.
2. **Docker et Traefik devant le filtre.** Le filtre voit l'adresse du pair TCP. Derrière Traefik, c'est Traefik — il
   faut le déclarer **mandataire de confiance**, sinon tous les clients paraissent venir de lui. Derrière une
   publication de port Docker par le mandataire en espace utilisateur, tous peuvent paraître venir de la passerelle
   Docker (`172.17.0.1`), une adresse privée **admise par défaut** : le filtre n'arrêterait alors personne. Je n'ai pas
   pu le vérifier sur le NAS : sais-tu si Kesh y est derrière Traefik, et quelle adresse il voit pour un client (le
   journal du serveur la donnera après la 32-6a) ? La 32-7 l'écrira au manuel.
3. **Le découpage en deux parties** (32-6a garde, 32-6b journal ; six modules distincts sans découpage) et l'ordre a → b.
4. **L'acteur du journal** : chaque appel est écrit au nom du **créateur de la clé** (`actor_type = 'user'`), la clé
   étant nommée au détail — plutôt qu'un type d'acteur « clé MCP », qui exigerait de modifier la colonne `actor_type`
   (un binaire antérieur ne la lirait plus). Te convient-il ? (Même défaut que #431 pour les PAT.)
5. **Ce qui est journalisé** : chaque `tools/call` seulement ; ni `tools/list`, ni les refus avant authentification
   (au journal du serveur seulement, pour qu'une requête anonyme n'écrive pas en base) ; et **si le journal ne
   s'écrit pas, rien ne part** (C-32-6-21, C-32-6-24, C-32-6-30).
6. **D'ici la 32-8, une clé `required` (le défaut) ne peut appeler aucun outil** — seules les métadonnées passent. Pour
   éprouver la 32-6, il faudra une clé `dispensée` (le seul outil, `kesh_version`, ne rend aucune donnée des livres).
7. **Hypothèses héritées** Q3, Q5, Q7 — inchangées.

## À recontrôler à l'étape 0 du développement

Ligne lue citée au Change Log pour chacun :

- **32-5b** : signature réelle de `services::cles_mcp::authentifier` et de `CleMcpAuthentifiee` ; `Portee` = alias de
  `McpScope`, cinq variantes ; le refus d'un préfixe autre **avant** tout accès à la base ; le point **R5** de la revue
  P2 de la 32-5a (avertissement de portée inconnue à **chaque** requête MCP pour une clé mixte) — s'il n'est pas réglé,
  la 32-6a le règle (dédupliquer ou réserver aux chemins d'administration) et le dit ; `tests/common/` (où vit
  `pool_lecture_seule`, `cles_mcp_e2e.rs`).
- **32-5d** : `mcp_keys` à `RESET_PRESERVED_TABLES` (aucune incidence attendue sur la 32-6, à constater).
- **32-5c** : `ipnet` dépendance directe de `kesh-api` ; forme canonique stockée des plages, mandataires, origines
  (C-32-5-19 : origine `http(s)://hôte[:port]` minuscule — la normalisation de l'AC-a9 doit lui être **égale**) ;
  type d'entité `mcp_key` à `audit_labels::ENTITY_TYPES` ; comptes d'`audit_route_registry.rs` (128 / 131 attendus) ;
  `admin_pat_denied_e2e` inchangé par la 32-6 (`/mcp` hors du bloc).
- **Comptes de `services_motif.rs`** : `FABRIQUES_V2` (deux sites ou plus après la 32-5b ?), `EXCEPTIONS_V1`, nombre de
  fichiers de `services/` de (vi) (dix au relevé ; `cles_mcp.rs` de la 32-5b, `serveur_mcp.rs` de la 32-6a),
  `RESIDUS_VI`.
- **`tower-http`** : le comportement du fait 1 (405, `Allow: GET,HEAD`) à la version du `Cargo.lock` d'alors.
- **Spike § 2.4 (h)** : la `protocolVersion` que demandent Claude Code et Hermes — si Guy l'a observée, la liste
  acceptée (AC-a12) s'y ajuste, à condition que les méthodes servies y aient la même forme ; sinon, liste inchangée et
  le Change Log le dit.
- **La révision `2025-11-25`** (si consultable) : la liste des méthodes client (AC-a11), et la forme d'une erreur de
  validation d'arguments (`isError` ou `-32602`, AC-a18).
- **Le schéma de journalisation** des tests (`tests/common/capture_rejeu.rs` ou autre) : s'il permet de capturer
  `tracing`, l'AC-a13 (« jamais journalisé ») gagne un test ; sinon, il reste tenu par la relecture citée.

## Dev Agent Record

### Agent Model Used

### Statut des parties

- **32-6a** — `ready-for-dev`, après la fusion des 32-5b, 32-5d, 32-5c.
- **32-6b** — `ready-for-dev`, après la 32-6a.

### Completion Notes List

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-25 | création | Fiche rédigée en autonomie (Opus 5.5) contre `b723e3d9` ; registre C-32-6-1 à C-32-6-30 ; découpage en 32-6a / 32-6b (six modules distincts sans découpage) ; prompt de validation P1 `326-validate-prompt-p1.md` (Sonnet, lentilles R et F). |
