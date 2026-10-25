# Story 32.6 : l'endpoint `/mcp` — protocole, garde, lecture seule, filtre, journal

Status: in-progress (32-6a en review, 32-6b ready-for-dev)

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-6`, branche `story/32-6` partie de `main`
`b723e3d9` (32-5a fusionnée). Registre : `epic-32-choix-autonomes-326.md` (C-32-6-1 à C-32-6-41). Clé courte : `326`.
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
  la 32-6 (§ 5.3.3) ; conséquences pour la 32-6 (§ 6, l. 932-944) ; six LOW de la relecture P4 (§ 6.2). Le point L-7 du
  § 5.3.3 et la mutation « donner `INSERT` à l'utilisateur », portée sur un analyseur de `SHOW GRANTS` (§ 4.3, § 6),
  sont **sans objet** sous Q11 (aucun second compte MariaDB) : ils relevaient des options (b) et (c) (validation P5, R10).
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
   `main_router` (`lib.rs:1174-1184`), le bloc protégé fusionné ensuite (`lib.rs:1204`). `require_auth` lit **d'abord**
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
   (`pool/options.rs:380`), `before_acquire` (`:435`), `after_release` (`:492` ; `Ok(false)` ferme la connexion,
   `pool/connection.rs:289-296`), `connect_lazy_with` (`:579`). La `Lecture` de test en lecture
   seule copie déjà les options du pool du test et passe `SET SESSION TRANSACTION READ ONLY` en `after_connect`
   (`tests/common/lecture_seule.rs:17-32`), **sans** `before_acquire`.
5. **Les poignées.** `Lecture(MySqlPool)` et `Trace(MySqlPool)`, champ privé, `pool()` en `pub(in crate::services)`
   (`services/poignees.rs:21-33`, `:38-52`) ; fabriquées en production par `AppState::lecture()` et `AppState::trace()`,
   **toutes deux sur `self.pool`** (`lib.rs:153-155`, `:162-164`). La garde lexicale (v-2) de
   `tests/services_motif.rs` exige que `Lecture::new(`/`Trace::new(` n'apparaissent qu'à ces **deux** lignes
   (`FABRIQUES_V2`, `:1888-1898` ; test `:2539-2546`) ; la garde (v-1) refuse tout `.pool(`/`::pool` hors de
   `services/` sauf `EXCEPTIONS_V1`, **vide** (`:1886`) ; (vi) compte **dix** fichiers dans `services/` (`:2559-2563`).
   `forbid(unsafe_code)` hors tests sur la lib (`lib.rs:9`) et sur `services` (`services/mod.rs:207`).
6. **Le contrôle d'accès des services** : `ContexteAcces::mcp(company_id, user_id, portees)` (`acces.rs:84-90`, **public**,
   appelé aujourd'hui par aucun code de production — N24), `exiger` rend `AppError::Forbidden` si la portée manque ou si
   l'opération n'est pas servie au MCP (`:113-123`) ;
   C-32-3-5 (fiche 32-3, l. 747) : « la 32-6 le traduira en “outil inconnu” côté MCP (aucun oracle) ». `Portee` n'a
   encore que trois variantes (`acces.rs:23-33`) : la 32-5b la remplace par `McpScope` (AC-b2).
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
   la forme `verbe(routes::module::handler)` pour `post`, `put`, `delete`, `patch` (`extract_counted`, `:496-517`) ;
   toute autre forme est un « site non résolu » qui fait rougir le test (`:540-567`) ; les appels à `routes::…` qui ne
   sont pas un handler d'un constructeur `post(`/`put(`/`delete(`/`patch(`/`get(` sont inventoriés à part
   (`appels_non_handlers`, `:676-698`) ; ni l'un ni l'autre ne lit `route_service(` : la 32-6a étend le détecteur
   (AC-a6). *(La P1 et la P2 l'étendaient à `.fallback(` ; la P4, F1, a remplacé le montage — C-32-6-40.)* Inventaire à **124** routes, registre à **127** (`:768`,
   `:1129`) — **128 et 131 après la 32-5c** (AC-c2 de la 32-5) : à recompter.
10. **La configuration** : `port` et `host` (`config.rs:180-181`), `public_base_url` facultatif, sans barre finale
    (`config.rs:334`, `:1089-1091`). « L'installation ne porte qu'une société » (`user-manual.tex:190`) : la règle par
    société est surtout théorique, mais le schéma en permet plusieurs (`mcp_server_settings.company_id` clé primaire).
11. **Garde lexicale de lecture seule, état de départ** : `grep -rniE "read write|tx_read_only|transaction_read" crates`
    ne rend **aucune ligne** (le `SET SESSION TRANSACTION READ ONLY` de `tests/common/lecture_seule.rs:23` n'est pas
    visé) — état **vierge** des formes qui défont le mode de session (spike § 4.1, quatre formes).
12. **Le manuel** : `docs/api-external.md:183` dit « Il n'existe pas de serveur MCP “Kesh-natif” **en v0.2** » (vrai,
    borné) ; `admin-manual.tex:2925` renvoie à ce guide ; ces textes changent avec la 32-11 (prémisse 10 du kickoff).
    `admin-manual.tex:2230`, `:2237`, `:2249`, `:2324` et `user-manual.tex:3028-3030` nomment déjà les clés, les
    réglages et les tables du serveur MCP (32-5a ; `:2249` et `:2324` ajoutés en validation P4, R11).
    Aucune ligne des manuels ne décrit `/mcp`.

## Le découpage (C-32-6-1)

Convention de compte (celle de la 32-5) : un crate pour `kesh-db` et `kesh-i18n` ; un module de second niveau pour
`kesh-api` ; `lib.rs` et le `Cargo.toml` de `kesh-api` comptent avec `routes` (validation P6, H8) ; un fichier de test compte avec le module qu'il éprouve ; `tests/common/`
avec le module dont il sert les tests.

| Partie | Objet | Modules | 🗄️ |
|---|---|---|---|
| **32-6a** | transport et protocole, traits et registre, `kesh_version`, garde de clé, règle « servi », filtre d'adresse et `X-Forwarded-For`, `Origin`, pool en lecture seule, politique de validation au répartiteur | `kesh-api/mcp` (neuf, et `tests/mcp_*`) ; `kesh-api/routes` (`routes/mcp.rs`, `lib.rs`, `tests/audit_route_registry.rs`) ; `kesh-api/services` (`serveur_mcp.rs`, `tests/services_motif.rs`, `tests/common/lecture_seule.rs`) ; `kesh-db` (`pool.rs`, `repositories/mcp_server_settings.rs`, leurs tests) — **4** | oui (un dépôt : gate complet en boucle ; **aucune migration**) |
| **32-6b** | le journal d'audit de chaque `tools/call` | `kesh-api/mcp` (répartiteur) ; `kesh-api/services` (`serveur_mcp.rs`, doc de `poignees.rs`) ; `kesh-api/audit_labels` ; `kesh-i18n` (quatre locales) ; `kesh-api/routes` (`tests/audit_route_registry.rs`, AC-b6 — validation P1, F7) — **5** | non |

**Sans découpage : six modules distincts** (`kesh-db`, `mcp`, `routes`, `services`, `audit_labels`, `kesh-i18n`), **neuf
en somme** (4 + 5, recompté en validation P1) — au-delà du plafond de cinq. Le journal se détache sans laisser d'accès ouvert : **après la 32-6a, aucun
outil ne rend de donnée des livres** — `Registre::standard()` ne porte que `kesh_version`, qui rend la version du
binaire et l'état de la session (AC-a13, figé par un test d'inventaire), et une clé `required` ne peut appeler aucun
outil (AC-a15). Les deux parties se fusionnent **avant toute release** ; aucune n'est publiable seule (la 32-11 précède
la release).

## Critères d'acceptation

Conventions des tests : fichiers neufs `crates/kesh-api/tests/mcp_transport_e2e.rs` (protocole, clé, politique, par
HTTP réel, motif `spawn_app` des fichiers du fait 3 — **sauf** `desactive_indiscernable_d_une_route_absente`, entièrement
par `oneshot`, AC-a3), `crates/kesh-api/tests/mcp_reseau_e2e.rs` (filtre, mandataires,
`Origin`, par `tower::ServiceExt::oneshot` et `MockConnectInfo`, fait 3), `crates/kesh-api/tests/mcp_gardes_lexicales.rs`,
`crates/kesh-db/tests/pool_lecture_seule.rs`, `crates/kesh-api/tests/mcp_traduction.rs`,
`crates/kesh-api/tests/common/capture_trace.rs` (déclaré `pub mod capture_trace;` dans `tests/common/mod.rs`),
`crates/kesh-api/tests/mcp_journal_e2e.rs` (32-6b). Chaque AC dit sa
commande ; « vert » s'entend `cargo nextest run -E 'binary(<fichier>)'` puis le gate complet (§ *Gates*). Bases
`kesh_326`/`kesh_e2e_326` propres au worktree (`docs/agents-orchestration.md` § 2), utilisateur `kesh` du fichier
versionné, **aucun `GRANT` posé à la main**.

⛔ **« Égal » veut dire égal octet pour octet** (validation P1, R3) : un test qui contrôle qu'une réponse ne porte
aucune donnée compare le corps **entier** à la constante attendue de `mcp/textes.rs` (ou à la réponse témoin d'une route
absente), jamais à l'absence d'une sous-chaîne devinée. Une absence de sous-chaîne ne s'emploie qu'en complément, pour un
marqueur que le test a lui-même posé là où la donnée **serait** émise (`PIEGE_*`).

⛔ **Règle commune des gardes lexicales** (validation P3, décision de l'orchestrateur : R1 = F1, R2 et R3 venaient de
gardes et de tests écrits dans la fiche sans être confrontés au dépôt). La fiche prescrit, pour chaque garde, la
**propriété** à garantir, son **périmètre** et les **formes à attraper** ; un motif qu'elle écrit n'est qu'un **exemple**,
et ne vaut que s'il porte sa mesure. Au développement :
1. **à l'étape 0**, chaque motif retenu est greppé sur le **dépôt réel**, dans le périmètre de sa garde, et doit rendre
   **vide** hors des sites écrits en liste blanche (fichier, ligne réduite de ses blancs) ;
2. chaque garde a **un témoin rouge** (une forme qu'elle doit attraper) et **un témoin vert** (une forme **légitime et
   existante** au dépôt qu'elle ne doit pas attraper), sur source factice, par une fonction pure ;
3. les commandes et leurs sorties sont **citées au Dev Agent Record**.
Il en va de même des tests qui reposent sur un outillage existant (une capture, un montage) : on lit ce que l'outillage
**capte** avant de s'y adosser, et le test porte un **témoin** qui prouve qu'il ne passe pas à vide.

### Partie A — 32-6a : le point d'entrée et ses gardes

**A.1 — Le montage, le canal et la règle « servi »**

1. **AC-a1 — un seul point d'entrée, sur le même port, monté comme le repli (C-32-6-31, C-32-6-40 ; validation P4, F1).**
   `build_router_avec_registre` monte, **dans `lib.rs`**, sur `main_router`, **avant** `.fallback_service(…)` et
   `.layer(csp_html)` (`lib.rs:1225-1227`) :
   `.route_service("/mcp", routes::mcp::recevoir.with_state(EtatMcp { … }))` — un **service** (`Handler::with_state`,
   `axum-0.8.9/src/handler/mod.rs:202`, rend un `HandlerService`, `handler/service.rs:148`), **non** un `MethodRouter`.
   *Pourquoi, prouvé au code d'Axum* : `route_service` enregistre un `Endpoint::Route` (`routing/path_router.rs:138`),
   appelé par `call_owned` (`:413`), qui marque son futur **non « top level »** (`routing/route.rs:44-46`) — **le même
   chemin** que le repli statique (`routing/mod.rs:366-370`, `Endpoint::Route` du routeur de repli). Un `MethodRouter`
   (`post(…)`, `.fallback(…)`, `any(…)`) rend au contraire un futur « top level », qui **ajoute** `content-length`
   (`route.rs:163-167`) et, **sauf `any`/`any_service`** (`skip_allow_header`, `method_routing.rs:514`, `:436`),
   l'`Allow` accumulé sur son **repli** (`method_routing.rs:1168-1174`) — validation P6, H3 : un `GET /mcp` désactivé porterait `Allow: POST` là où la route absente n'en
   porte pas — l'oracle que l'AC-a3 ferme. La **méthode** est donc tranchée par le transport (étape 3 des Dev Notes) ;
   `autre_methode` et l'extension du détecteur à `fallback(` disparaissent (C-32-6-20, C-32-6-32 annotés).
   `EtatMcp` (défini dans `mcp/mod.rs`, `Clone`) porte **seulement** : `origines_propres: Arc<[String]>` (calculées une
   fois par `mcp::origines_propres(port, public_base_url)`, définie en `src/mcp/reseau.rs` et appelée en `lib.rs` — les
   littéraux d'URL ne vont pas dans `lib.rs`, que `admin_pat_denied_e2e.rs:193-222`,
   `lib_rs_has_no_double_slash_inside_literals`, garde ; validation P4, R7 ; pas de `Config` entier, qui porte
   `database_url` et `admin_password`, `config.rs:179`, `:193`), `garde: Lecture` (`state.lecture()`, la `Lecture`
   **ordinaire**, pour la règle « servi » et l'authentification seulement, C-32-6-38), `lecture: LectureOutils` (la
   lecture **des outils**, en lecture seule, d'un **type propre** que rien d'autre n'accepte, AC-a26, C-32-6-41),
   `trace: Trace` (`state.trace()`), `registre: Arc<Registre>` (qui porte aussi le délai d'outil, AC-a21) et
   `statique` (le **même** service que le repli, AC-a3) — **pas d'`AppState`**, dont le champ public `pool`
   (`lib.rs:56`) mettrait le pool ordinaire à portée de tout `src/mcp/`. **Aucun champ neuf dans `AppState`, aucune
   `Extension`** : sans `EtatMcp`, le gestionnaire ne compile pas. Signature imposée : `pub async fn
   recevoir(State(etat): State<EtatMcp>, pair: Result<ConnectInfo<SocketAddr>, ExtensionRejection>, requete: Request)
   -> Response` — `Request` **dernier** extracteur, **aucun** extracteur de corps (`Bytes`, `String`, `Json`) ; corps
   **exactement** `reponse::rendre(&etat, transport::traiter(&etat, pair, requete).await).await` (AC-a31). Aucun
   `TcpListener::bind` neuf (garde `un_seul_ecouteur`, AC-a30). ⚠️ **À T-a0** : le test de l'AC-a3 se lance d'abord, sur
   ce montage, avant le reste du code ; il doit comparer aussi `Allow` et `content-length`.
2. **AC-a2 — l'ensemble des réglages qui « servent » ce pair.** Pour chaque requête vers `/mcp`, juste après
   l'extraction du pair (étape 1 des Dev Notes, dont elle est une entrée — validation P2, R16), le service `services::serveur_mcp::reglages_actifs(&Lecture)` lit sur **`etat.garde`** (la `Lecture` ordinaire,
   connexions réutilisées — **jamais** sur le pool des outils, C-32-6-38 ; validation P3, R9) les lignes
   `enabled = TRUE` (dépôt neuf `mcp_server_settings::list_enabled`, **sans écriture**, testé : aucune, une, deux
   sociétés ; une ligne `enabled = FALSE` absente ; **une ligne dont une colonne JSON n'est pas un tableau de chaînes
   — écrite `[1]` dans le test, le `CHECK` de la migration imposant un tableau (`20261025000001_…sql:81-83`) — est
   écartée** et journalisée par `tracing::error!` avec l'id de société, les autres lignes restant servies —
   C-32-6-6, validation P1, F16). L'ensemble **S** retient celles dont le filtre admet le client (A.2). **S vide →
   réponse de route absente** (AC-a3). Une erreur de la base pendant cette décision → réponse de route absente et
   `tracing::error!` (fail closed, C-32-6-3) — test : dans la base éphémère du test, `ALTER TABLE mcp_server_settings
   RENAME TO mcp_server_settings_cachee` par le pool ordinaire → réponse **égale** à celle d'un chemin absent
   (l'utilisateur `kesh` a tous les droits sur sa base de test, `01-dev-grants.sql`). ⚠️ **Dérogation écrite aux règles
   R1, R2 et R7 de `services`** (validation P2, R14) : `reglages_actifs` n'a ni `ContexteAcces` ni `exiger` — il précède
   l'authentification, aucune portée ne s'y applique —, et son test de refus est sans objet ; `services/mod.rs` le dit.
3. **AC-a3 — « désactivé » est indiscernable d'une route absente (C-32-6-2).** Quand `/mcp` n'est pas servi, le
   gestionnaire **remet la `Request` entière, corps non lu**, au même service de fichiers statiques que le repli du
   routeur (une seule fonction `service_statique(static_dir)` dans `lib.rs`, appelée pour le repli et pour `EtatMcp` —
   DRY), et sa réponse traverse la même couche `csp_html`. Test `desactive_indiscernable_d_une_route_absente` : pour
   `POST`, `GET`, `HEAD`, `DELETE`, `OPTIONS`, avec et sans `Authorization` (clé valide comprise), avec et sans corps,
   **et avec un corps de 3 Mio** (au-delà de la limite d'Axum et de `MCP_CORPS_MAX`), la réponse de `/mcp` est **égale**
   à celle de `/mcp-route-absente-326` — statut, **tous** les en-têtes hors `date` (dont **`Allow` et `content-length`**,
   que le montage de la P1 à la P3 aurait fait différer, validation P4, F1), octets du corps —, dans trois
   états : aucune ligne de réglages, ligne `enabled = FALSE`, ligne `enabled = TRUE` dont le filtre refuse le pair. Le
   test compare aussi à un **`static_dir` réel** (un `index.html` de test) pour que le `GET` ne passe pas à vide sur
   deux 404 identiques. ⛔ **Toute la matrice méthodes × états passe par `tower::ServiceExt::oneshot`** sur le routeur
   (avec `MockConnectInfo`), non par une connexion TCP : hyper pose lui-même `content-length: 0` sur toute réponse vide
   qui n'en porte pas, quand statut et méthode le permettent — ni 1xx, ni 204, ni 304, ni `HEAD` (`hyper-1.10.1/src/
   proto/h1/role.rs:512-521`, `:924-931`), donc sur les 405 de `POST`, `DELETE`, `OPTIONS` en jeu (validation P6, H4), si bien que sur le fil un retour à `any_service`
   (Ma20b (ii)), qui ne diffère que par ce `content-length`, passerait à vide ; **les témoins de ce test passent aussi par `oneshot`** —
   aucun ne garde une vraie connexion (validation P6, H2 ; la P5 écrivait « 401 et 200 », dont aucun 200 n'existait). Témoins : `enabled = TRUE` et pair admis → `POST` sans clé rend **401**, y compris avec un corps
   de 3 Mio (le corps n'est pas lu avant l'authentification) ; avec une clé valide **dont la société est dans S** (sinon 401, AC-a4 ; validation P5,
   F7), ce corps rend **413**. Tous les
   témoins « 401 » portent `Content-Type: application/json` (le 415 les précède, AC-a11 ; validation P2, R16). ⚠️ **Les
   cas à 3 Mio passent par `tower::ServiceExt::oneshot`** sur le routeur, non par une connexion TCP réelle : le serveur
   répond sans lire le corps, et hyper fermerait la connexion pendant que le client écrit encore (validation P2, F12).
4. **AC-a4 — la société de la clé doit elle-même servir (C-32-6-3, révisé en validation P1).** Après
   l'authentification (A.5), la société de la clé (`ContexteAcces::company_id`) doit avoir sa ligne dans **S** ; sinon,
   la réponse est **le 401 fixe**, égal octet pour octet à celui d'une clé inconnue (AC-a14) — **non** la réponse de
   route absente, qui ferait de `/mcp` un oracle de validité d'une clé volée (validation P1, F11 : une clé inconnue
   reçoit 401). Tests à deux sociétés : (i) société A activée, société B non, clé de B → 401 fixe ; (ii) A et B
   activées, le filtre de B refuse le pair, celui de A l'admet, clé de B → 401 fixe ; (iii) clé de A → servi. Le
   basculement est immédiat : désactiver la ligne entre deux requêtes de la même application change la réponse (aucun
   cache). Ce que ce cas laisse ouvert (la dernière utilisation de la clé de B est posée) est écrit au § *Ce que les
   gardes ne voient pas*.
5. **AC-a5 — méthodes HTTP quand `/mcp` est servi.** `POST` seul ; `GET`, `HEAD`, `DELETE`, `PUT`, `PATCH`, `OPTIONS`
   → **405** avec `Allow: POST`, posé **par le transport** (l'en-tête n'est plus celui d'Axum, AC-a1), corps vide,
   **sans** lire l'authentification ni le corps (spike § 1.3 : `GET`/`DELETE` → 405). **`/mcp` n'a pas de CORS, à
   dessein** : les clients visés sont des agents locaux (Claude Code, Hermes), non des pages web ; `OPTIONS` → 405 ferme
   toute pré-vérification, si bien qu'aucune page d'une autre origine ne joint `/mcp`, même inscrite à
   `allowed_origins` ; l'`Origin` sert contre le *DNS rebinding* (AC-a10). Un client web demanderait un CR (validation
   P4, R13). Aucun `Mcp-Session-Id` émis ni lu (sans session).
6. **AC-a6 — le registre des routes mutantes voit `/mcp` (C-32-6-20, C-32-6-32 annotés ; validation P4, F1).**
   Aujourd'hui, `extract_counted` (`audit_route_registry.rs:496-517`) ne lit que les verbes `post`, `put`, `delete`,
   `patch` (`:498`), et `appels_non_handlers` (`:676-698`) ne reconnaît un gestionnaire que précédé d'un constructeur de
   méthode (`:686-688`) : la forme `route_service("/mcp", routes::mcp::recevoir.with_state(…))` serait, pour le second,
   un « appel non-handler » inconnu (le chemin `routes::mcp::recevoir.with_state` va jusqu'à la `(` suivante, sans
   blanc) et, pour le premier, invisible. La 32-6a **étend le détecteur à `route_service(`** :
   - `extract_counted` s'applique à `lib.rs` **lignes de commentaire retirées** (témoin factice `// .route_service("/x",
     routes::a::b)` → rien — validation P3, F7) ; il lit `route_service(` comme un « verbe », dont l'argument est ce qui
     suit la **première virgule** (le chemin de la route écarté), **rogné**, jusqu'à `.with_state` ou à la `)`
     fermante ; un argument qui ne commence pas par `routes::` va à l'**inventaire des non-résolus** (`:554-567`) ;
   - `appels_non_handlers` reconnaît comme gestionnaire un `routes::…` qui suit `route_service("…", ` ;
   - tests sur **source factice**, au motif de `the_third_file_guard_sees_forms_that_are_not_named_router`
     (`:729-749`) : `.route_service("/x", routes::a::b.with_state(e))` → `extract_counted` rend `("route_service",
     "a::b")` et `appels_non_handlers` rend `[]` ; `.route_service("/x", users::h.with_state(e))` → **non résolu**
     (rouge) ; la même forme coupée par rustfmt sur trois lignes → même clé **et** `appels_non_handlers` rend toujours `[]`
     (témoin vert, sa reconnaissance tolère blancs et sauts de ligne entre `route_service(`, le littéral et la virgule ;
     validation P5, F4) ; **témoin discriminant** : sans
     l'extension, la première forme fait rendre à `appels_non_handlers` `["routes::a::b.with_state"]` ;
   - `LIB_ROUTES` gagne **une** entrée `("route_service", "mcp::recevoir", Exempt("issue #723 — journal des appels MCP :
     Story 32-6b"), SansEcritureAuJournal)` ;
   - **chaque compteur littéral qui bouge est nommé** (valeurs au relevé de `b723e3d9`, **plus** ce que la 32-5c y aura
     ajouté, recompté à T-a0) : `the_registry_partition_is_what_the_story_declares` — `LIB_ROUTES.len()` (`:768`, +1),
     `exempt` (`:788`, +1, message complété), `traced` (`:770-771`) et `no_matter` (`:789-791`) inchangés, total
     `LIB_ROUTES + TEST_ENDPOINT_ROUTES` (`:793-797`, +1) ; `the_replay_partition_is_what_the_story_declares` —
     `sans_ecriture` (`:1119-1127`, +1, message complété), `tout.len()` (`:1129`, +1) ; le doc-comment du module
     (« 124 routes », `:27`, `:31` ; « 127 », `:32`). Chaque nombre se **recompte** depuis la liste, la généalogie du
     message gagne « + 1 : `/mcp` (32-6a) ».
   - Le statut de **rejeu** reste `SansEcritureAuJournal` en 32-6a comme en 32-6b : la colonne parle du journal
     **comptable** (`audit_route_registry.rs:284-300`) ; précédent : `backup_schedule::update_backup_schedule`,
     `Traced, SansEcritureAuJournal` (`:326`).

**A.2 — Le filtre d'adresse et les mandataires (C-32-6-4 à C-32-6-6)**

7. **AC-a7 — le client se déduit du pair TCP.** Le pair vient de l'extracteur `Result<ConnectInfo<SocketAddr>,
   ExtensionRejection>` (validation P1, R1 = F1 : `ConnectInfo` n'implémente pas `OptionalFromRequestParts`,
   `axum-0.8.9/src/extract/connect_info.rs:153-158` ; `Option<ConnectInfo<_>>` ne compile pas) — cet extracteur garde le
   repli sur `MockConnectInfo` (`:163-164`) dont dépendent les tests ; **lire l'extension à la main est interdit** (il
   raterait `MockConnectInfo`). **`Err` → réponse de route absente** (fail closed ; test `oneshot` sans
   `MockConnectInfo`, **avec** une ligne `enabled = TRUE` aux défauts — `127.0.0.0/8` admis — pour que le test ne passe
   pas à vide sur un S vide ; validation P2, F9). Toute adresse — pair **et** entrées de `X-Forwarded-For` — est normalisée par
   `IpAddr::to_canonical` (une IPv4 mappée `::ffff:a.b.c.d` devient IPv4). Pour un réglage `s` :
   - pair **hors** de `s.trusted_proxies` → client = pair ; **`X-Forwarded-For` ignoré** ;
   - pair **dans** `s.trusted_proxies` → les valeurs de **toutes** les lignes `X-Forwarded-For`, dans l'ordre, coupées
     aux virgules et rognées, sont lues **de droite à gauche** ; les entrées qui sont des mandataires de confiance sont
     sautées ; la **première qui ne l'est pas** est le client ; en-tête **absent** → client = pair ; **toutes** les
     entrées de confiance → client = la plus à gauche ; une valeur d'en-tête non ASCII (`HeaderValue::to_str` en
     échec), ou une entrée qui n'est pas une adresse IP nue (port, crochets, `unknown`, vide) → **aucun client**
     (refus) ;
   - `Forwarded` (RFC 7239), `X-Real-IP` et tout autre en-tête sont **toujours** ignorés.
8. **AC-a8 — l'admission.** `s` admet le client si `s.address_filter_enabled = FALSE`, ou si le client appartient à une
   entrée de `s.allowed_networks` (`ipnet::IpNet::contains` ; une entrée stockée en adresse nue vaut `/32` ou `/128`,
   C-32-5-19). Les entrées stockées passent, à la lecture, par la **même** normalisation que le client
   (tout réseau IPv6 inclus dans `::ffff:0:0/96` est converti en réseau IPv4 de préfixe `p − 96` — `::ffff:172.18.0.2/128`
   devient `172.18.0.2/32`, `::ffff:172.16.0.0/108` devient `172.16.0.0/12` ; validation P5, R11 —, si bien qu'un
   mandataire saisi `::ffff:172.18.0.2` est reconnu pour le pair `172.18.0.2` ; validation P4, R9 ; à recontrôler contre ce que la 32-5c stocke). Filtre activé et liste vide →
   **personne**. Une entrée stockée de `allowed_networks` qui ne s'analyse pas est **ignorée** — jamais élargissante —
   ; une entrée invalide de **`trusted_proxies`**, elle, **écarte la société de S** (fail closed, comme la ligne
   illisible de l'AC-a2), car l'ignorer ferait prendre le mandataire pour le client et admettrait tout client derrière
   lui (validation P5, F2). Journal, à chaque requête, sans état partagé, colonne et id de société sans la valeur :
   l'entrée ignorée de `allowed_networks` en **`tracing::debug!`** (validation P2, R8 = F10 : la 32-5c valide la forme
   à l'écriture, si bien qu'une entrée invalide vient d'un SQL à la main) ; l'**exclusion** de la société pour
   `trusted_proxies` en **`tracing::error!`**, comme la ligne illisible de l'AC-a2 — sans quoi le serveur MCP d'une
   société cesserait de répondre sans trace visible au niveau de journal par défaut (validation P6, H1).
   *(Précisé le 2026-10-25, étape 0 de la 32-6a — P7-H3 : la règle « une entrée invalide de `trusted_proxies` écarte la
   société de S » vaut **quel que soit l'état du filtre** — avec `address_filter_enabled = FALSE` aussi, fail closed.
   Réalisé ainsi : `mcp::reseau::decider` lit les mandataires avant de consulter l'état du filtre.)*
9. **AC-a9 — les tests réseau** (`mcp_reseau_e2e.rs`, réglages posés par le dépôt de la 32-5a, `enabled = TRUE`) ;
   « refusé » s'entend **réponse égale à celle d'une route absente** (AC-a3), « admis » s'entend **401 sans clé** (puis
   200 avec clé) :
   - (a) pair `8.8.8.8` → refusé ; (b) pair `192.168.1.10` → admis ; (c) pair `::ffff:8.8.8.8` → refusé et
     `::ffff:192.168.1.10` → admis ; (d) filtre désactivé, pair `8.8.8.8` → admis ;
   - (e) pair `8.8.8.8` non déclaré, `X-Forwarded-For: 192.168.1.10` **forgé** → refusé (en-tête ignoré) ; (f) pair
     `192.168.1.10` non déclaré, `X-Forwarded-For: 8.8.8.8` → admis (en-tête ignoré, le pair compte — comportement
     soumis à Guy, point 2 bis) ;
   - (g) pair `172.18.0.2` déclaré mandataire, `X-Forwarded-For: 192.168.1.20` → admis ; (h) même mandataire,
     `X-Forwarded-For: 8.8.8.8` → refusé ; (i) même mandataire, `X-Forwarded-For: 192.168.1.20, 8.8.8.8` → **refusé** ;
     (j) deux mandataires déclarés, `X-Forwarded-For: 192.168.1.20, 172.18.0.3` → admis ; (k) mandataire déclaré,
     en-tête absent, plages par défaut → client = pair `172.18.0.2`, **admis** (`172.16.0.0/12`) ; (k') même cas,
     `allowed_networks = ["192.168.0.0/16"]` → **refusé** (tue « en-tête absent → aucun client » ; validation P4, R10) ; (l) mandataire déclaré,
     entrée `unknown` ou `8.8.8.8:443` → refusé ; (m) `X-Real-IP: 192.168.1.10` et `Forwarded: for=192.168.1.10` d'un
     pair public → refusé ; (m') mandataire déclaré, `X-Forwarded-For: 8.8.8.8`, `X-Real-IP: 192.168.1.10` → refusé
     (`X-Real-IP` n'est pas lu non plus dans la branche mandataire ; validation P2, F9) ;
   - (n) entrée stockée invalide (`"pas-un-réseau"`, écrite par SQL dans le test) à côté de `192.168.0.0/16` →
     `192.168.1.10` admis, `8.8.8.8` refusé ; (n') entrée invalide de **`trusted_proxies`** (`["pas-un-mandataire",
     "172.18.0.2"]`, écrite par SQL) : pair `172.18.0.2` avec `X-Forwarded-For: 192.168.1.20` → refusé, et pair
     `192.168.1.10` sans en-tête → refusé **aussi** (la société entière est écartée de S, non la seule entrée), avec un
     évènement `error!` capté ; témoin : la même ligne sans l'entrée invalide admet les deux (validation P6, H1) ;
     *(précisé le 2026-10-25, P7-H3 : second cas, la même ligne **filtre désactivé** → les deux requêtes refusées, Ma13b
     tuée par les deux ; avec une seconde société servie, la clé de la société écartée reçoit le 401 fixe de l'AC-a4 —
     corps égal à `TEXTE_NON_AUTHENTIFIE` ; avec une seule société, la réponse de route absente — test
     `n_prime_mandataire_invalide_ecarte_la_societe`)* ;
     (o) filtre activé, liste vide → `127.0.0.1` refusé ;
   - (p) mandataire déclaré, **deux lignes** d'en-tête `X-Forwarded-For: 192.168.1.20` puis `X-Forwarded-For: 8.8.8.8`
     → refusé (tue la lecture de la **première** ligne seule, qui rendrait `192.168.1.20`) ; (p') mandataire
     `172.18.0.2` déclaré, deux lignes `8.8.8.8` puis `172.18.0.2` → refusé (toutes lues : `8.8.8.8` ; tue la lecture de
     la **dernière** ligne seule, qui ne verrait que le mandataire et le prendrait pour client, admis par
     `172.16.0.0/12`) — validation P3, F6 ; (q) mandataire déclaré, `X-Forwarded-For: ::ffff:8.8.8.8` → refusé et
     `::ffff:192.168.1.20` → admis ; (r) valeur d'en-tête non ASCII → refusé ; (s) mandataire **stocké**
     `::ffff:172.18.0.2/128`, pair `172.18.0.2`, `X-Forwarded-For: 192.168.1.20` → admis (normalisation des entrées
     stockées, AC-a8).

**A.3 — L'`Origin` (C-32-6-7)**

10. **AC-a10 — validation de l'en-tête `Origin`**, après la règle « servi » et la méthode, **avant** l'authentification
    (spike § 1.1). Absent → admis. Une **seule** fonction de normalisation (schéma et hôte en minuscules, port par défaut **du schéma**
    retiré — `:80` pour `http`, `:443` pour `https`, et seulement eux (`http://kesh.example:443` reste distinct de
    `http://kesh.example`, cas de test ; validation P5, R6) —, aucun chemin, ni identifiants, ni requête) s'applique **des deux côtés** : à l'`Origin` reçue et
    à chaque origine admise (validation P1, R4 = F9 : la forme stockée par la 32-5c garde le port tel qu'écrit,
    C-32-5-19). Admise si elle égale une **origine propre** — `config.public_base_url` réduite à son origine,
    `http://127.0.0.1:<port>`, `http://localhost:<port>`, `http://[::1]:<port>` (`config.port`, fait 10) — ou une entrée
    de `allowed_origins` d'un réglage de **S** ; après l'authentification, la même règle se rejoue avec les seules
    `allowed_origins` de la société de la clé *(amendé le 2026-10-25, revue de code P2, H2 : ce second refus, à
    l'étape 9, rend le **401 fixe** d'une clé inconnue, non le 403 qui suit — C-32-6-52 ; le 403 ne vaut plus que pour
    le premier contrôle, à l'étape 4, avant la clé)*. Sinon → **403**, `Content-Type: application/json`, corps JSON-RPC à
    **`"id": null`** (JSON-RPC 2.0 § 5 : un `id` qu'on ne peut déterminer **doit** être nul ; à recontrôler contre le
    schéma `2025-11-25` à T-a0 — validation P4, R8), égal à la constante `TEXTE_ORIGINE_REFUSEE`. **Jamais déduite de `Host`**. `public_base_url` qui ne
    s'analyse pas en origine `http(s)://hôte[:port]` (sans schéma, avec identifiants ou schéma autre) → **aucune**
    origine propre n'en vient, avertissement au démarrage du serveur (`tracing::warn!`, une fois) (validation P1, F10).
    Tests : `Origin` absent → 401 sans clé ; `http://localhost:<port>` → admis ; `https://evil.example` → 403 ;
    `Origin: null` → 403 ; `Origin` égale à `Host` forgé (`Host: evil.example`, `Origin: http://evil.example`) → 403 ;
    origine listée dans la société A, clé de B — **B servie** (dans S), sinon l'étape 8 rend le 401 avant l'étape 9
    (validation P4, F11) — → 403 après authentification *(amendé le 2026-10-25, revue de code P2, H2 : → **401 fixe**,
    égal à celui d'une clé inconnue, C-32-6-52 ; test `origine`, mutation P1-Mp4)* ; `HTTP://LOCALHOST:<port>` → admis ; ⚠️ `<port>` est le `config.port` **du test** — un test d'intégration
    obtient sa `Config` par une fonction locale `fn test_config()` de son propre fichier (p. ex.
    `tests/accounts_e2e.rs:51`, `tests/api_keys_e2e.rs:47` ; `tests/common/` n'en fournit aucune), qui appelle
    `Config::from_fields_for_test` (`config.rs:443`, `port: 80` à `:502`) ; `<port>` vaut donc 80, normalisé en
    `http://localhost` (validation P6, H4), **non** le port de l'écouteur lié sur `127.0.0.1:0` —
    un test qui l'écrirait rougirait à tort (validation P5, R4) ;
    entrée stockée `https://kesh.example:443` et `Origin: https://kesh.example` → admis ; `public_base_url =
    "kesh.example"` (sans schéma) → `Origin: https://kesh.example` refusée.

**A.4 — Le transport (C-32-6-8, C-32-6-18)**

11. **AC-a11 — l'enveloppe HTTP**, dans cet ordre (tests à chaque échelon ; chaque corps d'erreur **égal** à sa
    constante) : `Content-Type` qui n'est pas `application/json` (paramètres admis, casse ignorée) → **415** ;
    `MCP-Protocol-Version` présent et ni `2025-11-25` ni `2025-06-18` (dont `2026-07-28`) → **400**, `text/plain`, texte
    fixe, **jamais** un code `-32022` (spike § 1.3) ; **absent → admis**, et noté au journal du serveur
    (`tracing::debug!`, sans autre donnée — spike § 1.3, « absent → admis […] journalisé » ; validation P1, R7) ;
    authentification (A.5) ; corps lu **après** l'authentification, **à la main** par `axum::body::to_bytes(corps,
    MCP_CORPS_MAX)` (1 Mio) → **413** au-delà ; toute autre erreur de lecture du corps (flux interrompu, corps tronqué :
    la même `Err` d'`axum_core`, `axum-0.8.9/src/body/mod.rs:48`) → **400** à texte fixe, que le transport distingue du
    dépassement par la nature de l'erreur (`LengthLimitError` dans la chaîne des causes ; à constater à T-a0 —
    validation P5, R8) (validation P1, F2 : jamais par un extracteur, qui le lirait avant la règle
    « servi » sous la limite par défaut d'Axum et rendrait 413 là où une route absente rend 405) ; JSON invalide →
    **400** et `-32700` à `"id": null` ; **tableau** (lot) → **400** et `-32600` à `"id": null` ; `jsonrpc` ≠ `"2.0"`, `id` nul
    ou ni chaîne ni nombre → `-32600` ; **message de type réponse** (`result` ou `error`, sans `method`) → **400** ;
    **notification** (pas d'`id`) → **202**, corps vide, **jamais traitée par un gestionnaire de requête** —
    une notification **complète** `{"jsonrpc":"2.0","method":"tools/call","params":{"name":"<compteur>","arguments":{}}}`,
    **sans `id`**, sous une clé **`exempt`**, n'exécute aucun outil — compteur à 0 —, avec un **témoin** : le même
    message **avec** un `id` porte le compteur à 1 (validation P4, F5 : sans nom d'outil, ou sous une clé `required`, le
    test passait à vide) ; requête → **200**,
    `application/json`, `Cache-Control: no-store`. L'en-tête `Accept` n'est pas contrôlé. Aucune réponse en
    `text/event-stream` (Q10 (c) : JSON partout).
12. **AC-a12 — les méthodes servies (E2), liste close.** Le répartiteur associe chaque `method` à une entrée d'une
    **énumération fermée** ; tout le reste → **`-32601`** à texte fixe. Servies : `initialize`, `ping`, `tools/list`,
    `tools/call`, `prompts/list`, `prompts/get` ; notifications acceptées : toute `notifications/*` (202). Test
    d'inventaire `methodes_servies_liste_close` : l'ensemble des méthodes acceptées égale la liste **écrite en dur dans
    le test** ; chaque méthode client du schéma `2025-11-25` qui n'y est pas rend `-32601` — liste tirée du schéma à
    l'étape 0 (T-a0) et citée avec sa source, comprenant au moins `resources/list`, `resources/read`,
    `resources/templates/list`, `resources/subscribe`, `resources/unsubscribe`, `completion/complete`,
    `logging/setLevel`, `tasks/get`, `tasks/list`, `tasks/result`, `tasks/cancel`, et un nom inventé.
13. **AC-a13 — `initialize`, `ping`, et l'inventaire des outils.** `initialize` répond `protocolVersion` = la version
    demandée si c'est `2025-11-25` ou `2025-06-18`, sinon `2025-11-25` (spike § 1.3) ; `serverInfo` `{name: "kesh",
    version: CARGO_PKG_VERSION}` ; `capabilities` = `tools` et `prompts` (`listChanged: false`) **seulement** —
    **aucune** `resources`, `logging`, `completions`, `tasks` (E3) ; `instructions` est une **constante** du binaire.
    `ping` → `{}`. Le cycle de vie n'est pas imposé (C-32-6-25). Test `registre_standard_inventaire` :
    `Registre::standard()` porte **exactement** `["kesh_version"]` — la 32-9 l'amendera.

**A.5 — La clé (reports de la 32-5)**

14. **AC-a14 — clé MCP seulement.** Le jeton se lit **uniquement** dans `Authorization: Bearer <jeton>` (schéma
    insensible à la casse, RFC 7235) ; **le cookie `kesh_access_token` est ignoré** ; il est passé à
    `services::cles_mcp::authentifier(&etat.garde, &etat.trace, jeton)` (32-5b ; sur la `Lecture` ordinaire, comme
    `validate_pat` de l'API, `auth/api_key.rs:153-155` — le pool des outils n'est emprunté qu'après l'authentification,
    C-32-6-38), dont le résultat est **filtré** en
    `ReponseMcp::NonAuthentifie` ou `ReponseMcp::ErreurInterne` par un `match` du transport, l'un des deux seuls sites
    qui nomment `AppError` (AC-a31). **Même réponse 401** — statut,
    en-têtes hors `date`, corps **égal** à `TEXTE_NON_AUTHENTIFIE` (JSON-RPC à `"id": null`, **aucun `WWW-Authenticate`**,
    C-32-6-27) — pour : en-tête absent, schéma autre que `Bearer`, jeton vide, **PAT actif `read-write`**, JWT valide en
    `Bearer`, cookie de session valide sans en-tête, clé inconnue, révoquée, expirée, créateur inactif ou rétrogradé,
    jeton d'un PAT derrière le préfixe `kesh_mcp_`, clé valide d'une société qui ne sert pas (AC-a4). Tests :
    `pat_refuse_a_l_endpoint` (et `api_keys.last_used_at` du PAT **inchangée**, report (1) de la 32-5),
    `session_refusee`, `refus_indiscernables`. Une erreur de la base pendant l'authentification n'est pas un 401 :
    **500**, corps **égal** à `TEXTE_ERREUR_INTERNE` (test : réglages actifs, puis `ALTER TABLE mcp_keys RENAME TO
    mcp_keys_cachee` → 500 et corps égal à la constante — validation P1, R3). Le jeton, l'en-tête et le corps ne sont
    **jamais** journalisés : test `jeton_jamais_journalise`, par une **couche de capture générique** neuve
    (`tests/common/capture_trace.rs` : tous niveaux, `TRACE` et `DEBUG` compris, toutes cibles, message et champs
    formatés) montée par `tracing::subscriber::set_default`, le montage par fil de `tests/common/capture_rejeu.rs:10`,
    `:85` — **non** sa couche, qui ne retient que le champ `operation` des `WARN`/`ERROR` de cible `kesh_db::retry`
    (`capture_rejeu.rs:34-64` ; validation P3, R3). ⛔ **Le test exige un témoin émis par le serveur pendant la
    requête éprouvée** (validation P4, F4 : un évènement émis par le corps du test ne prouve que l'installation de
    l'abonné sur le fil du test) : la requête est envoyée **sans** `MCP-Protocol-Version`, si bien que le transport émet
    le `tracing::debug!` de l'AC-a11 ; le test constate que **cet** évènement est capté **avant** de constater qu'une
    requête à jeton `kesh_mcp_PIEGE…` et corps piégé, refusée puis servie, ne laisse ni le jeton ni le piège dans les
    évènements
    capturés — sans témoin, le test passerait à vide ; et `grep -rnE
    "jeton|token|authorization|corps|body" crates/kesh-api/src/mcp crates/kesh-api/src/routes/mcp.rs` relu ligne à
    ligne, sortie citée au Dev Agent Record.
15. **AC-a15 — la politique de validation, contrôlée AVANT l'exécution (Q12 ; frontière avec la 32-8 ; validation P2,
    F2, C-32-6-14 révisé, C-32-6-36).** `CleMcpAuthentifiee::validation` n'est lue que par **une** fonction nommée,
    `politique::autoriser(validation) -> Result<Autorisation, Refus>`, appelée par le répartiteur **avant** `executer` ;
    un outil ne la voit pas. `Autorisation` est un jeton à champ privé défini dans `politique.rs` : seul `autoriser` le
    fabrique. Pour un `tools/call` d'un `McpTool` trouvé dans les portées : `Required` → `Err(Refus)` → **l'outil n'est
    pas exécuté**, réponse `result` avec `isError: true` et le texte fixe `TEXTE_VALIDATION_REQUISE` ; `Exempt` →
    `Ok(Autorisation)`, l'outil s'exécute, puis `politique::appliquer(Autorisation, Donnees) -> DonneesEnvoyables` ne
    fait **que** construire la réponse (elle ne relit pas la politique, et ne s'appelle pas sans `Autorisation`). Tests :
    `cle_requise_refusee_sans_execution` (outil de test à compteur : 0 exécution ; corps égal à la réponse attendue),
    `cle_dispensee_servie` (compteur à 1 ; corps égal à la réponse attendue). Mutation Ma23 : `autoriser` rend `Ok` pour
    `Required` → `cle_requise_refusee_sans_execution` et `e1_rien_ne_part_sous_cle_requise` rougissent (non
    `cle_dispensee_servie`, que la mutation ne touche pas ; validation P3, R5) ; Ma50 : `executer` appelé **avant**
    `autoriser` → `cle_requise_refusee_sans_execution` (compteur à 1) ; Ma23b : appeler `appliquer` sans `autoriser` **ne compile pas**
    (le **constructeur** d'`Autorisation`, champ privé, est inaccessible hors de `politique.rs` : E0451) — déclarée comme
    telle. Les méthodes de métadonnées (`initialize`, `ping`, `tools/list`, `prompts/*`)
    sont servies quelle que soit la politique (E2 : textes du binaire).

**A.6 — Les outils, les portées, les erreurs et l'absence d'oracle**

16. **AC-a16 — le trait `McpTool`** (`mcp/outil.rs`) : `nom() -> &'static str`, `description() -> &'static str`,
    `schema_entree() -> &'static str` (JSON **texte** du binaire, spike § 5.3.3 L-5), `portee() -> Portee` **sans
    implémentation par défaut**, `executer(&self, &ContexteMcp, serde_json::Value) -> Pin<Box<dyn Future<Output =
    Result<Donnees, ErreurOutil>> + Send + '_>>`. `ContexteMcp` (défini dans `mcp/outil.rs`, champs privés, accesseurs
    publics `acces()` et `lecture()`) porte `&ContexteAcces` et la `Lecture` des outils, reçue sous la forme d'un
    **`&LectureOutils`** (C-32-6-41) : `LectureOutils` est défini dans `mcp/outil.rs`, à champ privé (une `Lecture`),
    construit **une fois** en `lib.rs` par `LectureOutils::nouvelle(Lecture)` et **jamais déballé** hors de ce fichier —
    seul `ContexteMcp::lecture()` en rend la `&Lecture` à l'outil. `reglages_actifs(&Lecture)` et `authentifier(&Lecture,
    …)` ne peuvent donc pas le recevoir, ni `ContexteMcp::nouveau` la `garde` : les deux confusions sont des **erreurs de
    type** (Ma51, Ma52, déclarées non compilables ; validation P4, F2 = R1). `ContexteMcp` ne porte **rien d'autre** ; son
    constructeur est `pub(super) fn nouveau(…)` — visible du module `mcp` et de ses descendants (validation P1, F12 :
    un constructeur privé au module de `outil.rs` ne serait pas appelable du répartiteur) ; qu'il ne soit appelé qu'en
    `mcp/repartiteur.rs` est tenu par la garde `contexte_mcp_construit_par_le_repartiteur_seul` (AC-a30), le
    compilateur n'excluant pas `mcp::outils`. ⚠️ **Pour la 32-9** (validation P1, R16) : les services de rapport
    exigent une `Trace` (`Rapport::afficher(self, trace: &Trace)`, `services/rapports.rs:347`) ; la 32-9a amendera
    `ContexteMcp` et cette garde en connaissance de cause. Test `registre_coherent` : chaque outil du registre a un
    nom `[a-z0-9_]{1,64}` unique, un schéma qui s'analyse, de type `object`, avec `additionalProperties: false`, et une
    portée **différente de `PromptsWrite`**.
17. **AC-a17 — `tools/list` filtré.** Rend les seuls outils dont la portée appartient aux portées de la clé, dans
    l'ordre du registre, chacun avec `name`, `description`, `inputSchema` et `annotations: {readOnlyHint: true}` —
    l'annotation est posée **par le répartiteur**, non par l'outil. `cursor` fourni → `-32602`. Test `tools_list_filtre`
    (validation P1, F13) : clé `books:read` → `result.tools` **égal** à la liste attendue (un élément, `kesh_version`,
    annoté) ; clé `contacts:read` seule → `[]` ; clé `books:read` sur un registre de test à deux outils (`BooksRead` et
    `InvoicesRead`) → le seul premier.
18. **AC-a18 — hors portée = inconnu, sans oracle, et la traduction complète des erreurs de service (validation P1,
    R15, C-32-6-33).** `tools/call` d'un outil inconnu, hors portée de la clé, ou dont le **service** refuse l'accès
    (`AppError::Forbidden`, fait 6, C-32-3-5) → **la même** erreur JSON-RPC `-32602` à texte fixe
    (`TEXTE_OUTIL_INCONNU`) ; test `hors_portee_indiscernable` : les trois corps sont égaux octet pour octet, `id` mis à
    part ; l'outil hors portée **n'est pas exécuté** (compteur) ; le cas « refus du service » passe par un outil de test
    de portée `BooksRead` qui appelle `factures::lister` (portée `InvoicesRead`), **sous une clé `exempt`** — sous
    `required`, `autoriser` refuserait avant l'exécution (validation P4, F11). Toute autre erreur d'un service passe
    par **une seule** fonction `mcp::textes::depuis_app_error(AppError) -> ErreurOutil` (AC-a31), qui applique une
    **liste blanche courte et écrite** (validation P2, R1 = F1, C-32-6-33 révisé — `AppError` compte environ 125 variantes,
    `errors.rs:139-1182`, et n'a ni `NotFound` ni `Conflict`) :

    | Variante (lue à `errors.rs`) | Rend |
    |---|---|
    | `AppError::Forbidden` (`:170`) | outil inconnu, `-32602`, `TEXTE_OUTIL_INCONNU` |
    | `AppError::Database(DbError::NotFound)` (`:165` ; `DbError::NotFound`, `kesh-db/src/errors.rs:575` ; forme des services, `services/ecritures.rs:241`, `services/carnet.rs:135`) | `ErreurOutil::Introuvable`, `TEXTE_INTROUVABLE` |
    | `AppError::Validation(_)` (`:153`) | `ErreurOutil::ArgumentsInvalides { argument: "(demande)", motif: Invalide }` — le message n'est **jamais** recopié |
    | **toute autre variante**, `Database(_)` compris (bras `_`) | `ErreurOutil::Interne`, `TEXTE_ERREUR_INTERNE` |

    Le bras `_` est **voulu** : une variante neuve tombe dans l'erreur interne fixe, le cas le plus fermé ; élargir la
    liste blanche est un choix écrit à la fiche qui le fait. Test `traduction_des_erreurs_de_service`, dans
    **`crates/kesh-api/tests/mcp_traduction.rs`** (hors de `src/`, pour que ses constructions d'`AppError` ne tombent pas
    sous la garde d'inventaire de l'AC-a31 ; `depuis_app_error` est donc `pub`, `#[doc(hidden)]` — validation P3, R10) : chaque
    bras de la liste rend sa constante (corps **égal**, `Validation("PIEGE_APP_326")` compris) ; et des variantes **hors
    liste**, prises dans plusieurs familles — `Internal("PIEGE_APP_326")`, `Database(DbError::OptimisticLockConflict)`,
    `FiscalYearClosed { … }`, `AccountNotFound`, `RateLimited { … }`, `IllegalState("PIEGE_APP_326")` (noms recontrôlés
    à T-a0 contre l'énumération) — rendent **toutes** la réponse `TEXTE_ERREUR_INTERNE`, égale octet pour octet.
19. **AC-a19 — les arguments.** `params.arguments` absent vaut `{}` ; chaque outil lit ses arguments par une aide
    commune `arguments::<T: DeserializeOwned>()` (`deny_unknown_fields` exigé par la revue) qui rend
    `ErreurOutil::ArgumentsInvalides { argument: &'static str, motif }` — **jamais** le message de `serde`. Réponse :
    `isError: true`, texte fixe par motif (C-32-6-17 ; la forme « erreur d'exécution » plutôt que `-32602` est à
    recontrôler à l'étape 0 contre la page `server/tools` de la révision `2025-11-25`). Test : `kesh_version` avec
    `{"PIEGE_ARG_326": 1}` → corps **égal** à la réponse attendue. **Demande mal formée** (validation P6, H5) : `params`
    absent, `name` absent ou non chaîne, `arguments` qui n'est pas un objet → au **répartiteur** (non au transport :
    le message JSON-RPC est déjà valide), erreur `-32602` à texte fixe `TEXTE_PARAMS_INVALIDES`, corps **égal** ; test
    `params_mal_formes` (trois cas). Étant au répartiteur, elle **est journalisée** en 32-6b (issue
    `arguments_invalides`, `outil` = `"(inconnu)"`) — ce qui ne contredit pas l'AC-b4, qui n'exclut que les refus **du
    transport**.
20. **AC-a20 — les erreurs sans texte libre (E5).** `ErreurOutil` n'a **aucun** champ `String`, `Value` ni
    `Box<dyn Error>` ; chaque variante se traduit en un texte **fixe** (`mcp/textes.rs`, constantes) ; le détail d'une
    erreur interne part au journal du serveur (`tracing::error!`), jamais au client. Délai d'exécution d'un outil borné
    (le délai du registre, `Registre::delai()` — `DELAI_OUTIL` = 30 s en production, C-32-6-19 ; une seule source,
    validation P2, R12 = F11) → `isError: true`, texte fixe. Tests
    `erreur_interne_texte_fixe` : sous clé `exempt`, l'outil **piégé** (AC-a21) appelé après `ALTER TABLE accounts
    RENAME TO accounts_cachee_326` → corps **égal** à la réponse `isError: true` de `TEXTE_ERREUR_INTERNE` (validation
    P1, R3 : le message de MariaDB nomme la table **demandée**, non le nouveau nom, si bien qu'une recherche de
    `accounts_cachee_326` passerait à vide) ; `delai_depasse` : l'outil **lent** sous un registre de délai réduit
    (`Registre::vide().avec(lent).avec_delai(Duration::from_millis(50))`) → corps égal à la réponse attendue. Un outil qui tenterait
    d'écrire n'est **pas exprimable** hors de `services` (le pool n'y est pas lisible, fait 5) : le refus 1792 se prouve
    à la fabrique (AC-a25).
21. **AC-a21 — le registre de test.** `kesh_api::build_router_avec_registre(state, static_dir, registre)`
    (`#[doc(hidden)]`) ; `build_router` l'appelle avec `Registre::standard()` ; `main.rs` n'appelle que `build_router`
    (garde lexicale). `Registre::vide()`, et les **méthodes** `avec(self, Box<dyn McpTool>) -> Self` et
    `avec_delai(self, Duration) -> Self`, sont publics pour les tests ; les deux points d'accès du répartiteur s'appellent
    **`outil_pour(nom, portées)`** et **`descripteurs_pour(portées)`** — noms qui n'existent nulle part dans le dépôt
    (`grep -rnE "\b(outil_pour|descripteurs_pour)\s*\(" crates` → 0 au relevé de `c6523903` (code de `b723e3d9`)), là où `lister` est le nom
    de six fonctions de services de lecture (`grep -n "pub async fn lister" crates/kesh-api/src/services/*.rs` → 6 ;
    validation P5, R5 = F6) ; `Registre::standard()` porte `DELAI_OUTIL` ; le
    délai ne vit **que** dans le registre (`Registre::delai()`), `EtatMcp` n'en a pas de copie. Outils de test
    (`tests/common/mcp.rs`, neuf) : **compteur**, **piégé** (lit par `services::plan_comptable::lister` un compte dont
    le nom porte `PIEGE_E1_326`), **refus de service** (AC-a18), **lent** (dort plus que le délai réduit), et, pour la
    32-6b, **introuvable** (appelle un service avec un identifiant inexistant : `Database(DbError::NotFound)`, issue
    `introuvable` — validation P2, F14). ⚠️ Faute de `Debug` sur `Donnees` (AC-a23), un outil ne s'éprouve que par la
    réponse du répartiteur ou par un `match` sur son résultat — `unwrap_err()` et `assert_eq!` sur un `Donnees` ne
    compilent pas ; à savoir pour la 32-9 (validation P2, R15). ⚠️ Le délai réduit est **obligatoire** : le test de
    délai n'a pas de porte de sortie « non éprouvé » (validation P1, F14).
22. **AC-a22 — E1 côté répartiteur.** Sous une clé **`required`**, la valeur `PIEGE_E1_326` n'apparaît dans **aucune**
    réponse d'**aucune** méthode servie (`initialize`, `ping`, `tools/list`, `tools/call` de l'outil piégé et de
    `kesh_version`, `prompts/list`, `prompts/get`, méthode inconnue, notification) — test `e1_rien_ne_part_sous_cle_requise`,
    chaque corps comparé en outre à sa réponse attendue ; **témoin** : sous une clé `exempt`, `tools/call` de l'outil
    piégé la contient (le test ne passe pas à vide).
23. **AC-a23 — `Donnees` et la réponse d'outil ne se fabriquent que par la politique (E6, mutation 2 ; validation P1,
    F6, C-32-6-15 révisé).** `Donnees` **et** `DonneesEnvoyables` sont **définis dans `mcp/politique.rs`** :
    - `Donnees` : champs privés, constructeur public `Donnees::nouvelles(resume, contenu, structure)` (les outils, y
      compris de test, l'emploient), **aucun accesseur public** — ses champs ne se lisent qu'en `politique.rs` ;
    - `DonneesEnvoyables` : champ privé, construit par `politique::appliquer(Autorisation, Donnees)` seul (AC-a15) ;
      `ReponseMcp::Outil` n'accepte que `DonneesEnvoyables` ;
    - **ni l'un ni l'autre**, **ni `Autorisation`** (validation P3, R6), ne dérive ni n'implémente `Serialize`, `Default`
      ni `Debug` (un `Debug` permettrait `format!("{:?}")` vers un texte) ; `Clone` est permis pour `Donnees`, **ni
      `Clone` ni `Copy`** pour `Autorisation` (une autorisation ne se duplique pas) ;
    - `reponse::rendre` tire le JSON de `DonneesEnvoyables` par **une** méthode définie en `politique.rs`,
      `pub(in crate::mcp) fn en_resultat(self) -> serde_json::Value` (validation P3, F4) ;
    - garde `donnees_construites_par_la_politique_seule` : dans `crates/kesh-api/src`, `DonneesEnvoyables {`,
      `DonneesEnvoyables::` et `impl … for DonneesEnvoyables` n'apparaissent qu'en `mcp/politique.rs` ; les attributs
      `#[derive(…)]` qui précèdent `struct Donnees`, `struct DonneesEnvoyables` et `struct Autorisation` ne nomment ni
      `Serialize`, ni `Default`, ni `Debug` (ni `Clone`/`Copy` pour `Autorisation`) ; aucun `impl
      (Default|Serialize|Debug|Clone|Copy) for (Donnees|DonneesEnvoyables|Autorisation)` hors des combinaisons permises
      — ce qui ferme `Default::default()` et `<DonneesEnvoyables as Default>::default()` (aucune implémentation n'existe
      à appeler) ;
    - les réponses statiques (`initialize`, `tools/list`, erreurs, `prompts/list`) ne se construisent qu'à partir de
      `&'static str` et des descripteurs du registre.
24. **AC-a24 — les prompts (32-6 : le socle).** Trait `McpPrompt` (`mcp/prompts.rs`), **scellé** (`mod scelle { pub
    trait Scelle {} }` privé à `mcp::prompts`), `rendre` sans `Lecture` ; **aucune implémentation** en 32-6.
    `prompts/list` → `{"prompts": []}` ; `prompts/get` de tout nom → `-32602` à texte fixe. Garde : `impl … McpPrompt
    for` n'apparaît qu'en `mcp/prompts.rs` (aucune en 32-6). La 32-10a amende ce trait en connaissance de cause
    (C-32-6-23).

**A.7 — La lecture seule (Q11, C-32-6-9 à C-32-6-11)**

25. **AC-a25 — le pool du MCP, une seule fabrique.** `kesh_db::pool::pool_lecture_seule(depuis: &MySqlPool,
    max_connections: u32) -> MySqlPool` : `MySqlPoolOptions` avec `acquire_timeout` = celui de `depuis`
    (`Pool::options().get_acquire_timeout()`), `after_connect` = `SET SESSION TRANSACTION READ ONLY` (et **aucun autre
    réglage de session** : le pool de production n'en a pas, fait 4, C-32-6-9), **`after_release` qui rend toujours
    `Ok(false)`** — **toute connexion est fermée à sa restitution**, aucune n'est jamais reprise du repos (validation P2,
    R4, C-32-6-35 : `sqlx-core-0.8.6/src/pool/connection.rs:289-296` ferme la connexion sur `Ok(false)`) —,
    `connect_lazy_with((*depuis.connect_options()).clone())`. Chaque emprunt part donc d'une connexion **neuve**, passée
    par `after_connect` (`pool/inner.rs:355-358`) : **aucun état de session ne survit d'un emprunt à l'autre**, quelle
    que soit la forme qui l'aurait défait — y compris la forme 2 de la spike (`SET TRANSACTION READ WRITE`, qui vaut
    pour la transaction **suivante** et que `@@session.tx_read_only` ne montre pas, spike § 4.1). Le `before_acquire` de
    la première rédaction est **retiré** (il ne voyait pas la forme 2, R4, et ne serait plus jamais appelé). Coût : une
    connexion par emprunt — une poignée de millisecondes, pour un trafic MCP faible. ⚠️ `connect_lazy_with` lance les tâches d'entretien du pool : il exige un **runtime
    Tokio** (`sqlx-core` `rt::spawn`) — `build_router` n'est appelé que depuis des fonctions `async` (validation P1, R9).
    Appelée **une seule fois** en production, dans `build_router_avec_registre`, `max_connections` =
    `MCP_POOL_CONNEXIONS` (3, C-32-6-10) ; et par `tests/common/lecture_seule.rs`, dont la fonction `async fn
    pool_lecture_seule(pool: &MySqlPool) -> MySqlPool` **garde sa signature** (ses appelants, dont
    `services_motif.rs:45` et `:1537`, `services_rollout.rs:60` et `:519` — l'AC11 (c) de la 32-4a, qui rejoue désormais
    sur un pool dont chaque emprunt est neuf —, ne changent pas ; validation P3, R13) et délègue à la fabrique avec `max_connections = 4` (C-32-6-26).
    Tests `crates/kesh-db/tests/pool_lecture_seule.rs`, utilisateur `kesh`, base `#[sqlx::test]` : (a) `INSERT`,
    `UPDATE`, `DELETE` en autocommit → **1792** ; (b) les trois dans `pool.begin()` → 1792 ; (c) `SELECT … FOR UPDATE`
    → 1792 ; (d) `SELECT` → OK ; (e) **séquence B de la spike** (§ 4.2), forme 4 : `max_connections(1)`, `SET SESSION
    TRANSACTION READ WRITE` sur une connexion, rendue, réacquise → un `UPDATE` en autocommit est **refusé**, et
    l'identifiant de connexion (`CONNECTION_ID()`) a changé ; (e bis) **forme 2** (validation P2, R4) : même montage,
    `SET TRANSACTION READ WRITE` (sans `SESSION`), rendue sans autre instruction, réacquise → `UPDATE` **refusé**,
    identifiant changé ; (f) une connexion tuée **pendant** un emprunt (`max_connections(1)`, `CONNECTION_ID()` lu, `KILL`
    par le pool ordinaire, requête suivante du même emprunt en échec) n'empêche pas l'emprunt suivant, qui part d'une
    connexion neuve, refuse une écriture en 1792 et porte un autre identifiant (validation P3, F5 : depuis
    `after_release`, aucune connexion ne reste au repos, et l'ancien (f) passait à vide).
    ⚠️ (e) et (e bis) exécutent volontairement une forme interdite : ce sont les exceptions écrites de l'AC-a27. Aucune
    infrastructure nouvelle : même fichier, même base `#[sqlx::test]`.
26. **AC-a26 — le câblage : les outils lisent sur ce pool, la trace écrit sur l'autre.** `EtatMcp.lecture` est
    `mcp::LectureOutils::nouvelle(Lecture::new(kesh_db::pool::pool_lecture_seule(&state.pool, MCP_POOL_CONNEXIONS)))`,
    en **un** site de `lib.rs` ; `EtatMcp.garde` est `state.lecture()` ;
    `EtatMcp.trace` est `state.trace()`. `FABRIQUES_V2` de `services_motif.rs` gagne ce site (trois lignes, recompté à
    T-a0), `EXCEPTIONS_V1` le chemin `kesh_db::pool::` qui fait rougir (v-1) par son `::pool` (`MOTIF_V1`,
    `services_motif.rs:1913`), chacun avec sa ligne de commentaire. Tests : (i) `kesh_version_lit_en_lecture_seule` —
    sous une clé `exempt`, `kesh_version` rend `lectureSeule: true`, lu par le service **dans la même requête** sur la
    connexion de la lecture (AC-a28) ; (ii) `l_appel_pose_la_derniere_utilisation` — après un `tools/call` servi,
    `mcp_keys.last_used_at` de la clé est posée (la `Trace` du MCP écrit : elle n'est pas sur le pool en lecture
    seule ; `touch_last_used` étant *best-effort*, c'est ce test qui rend sa perte audible).
27. **AC-a27 — garde lexicale des formes qui défont le mode de session** (`mcp_gardes_lexicales.rs`,
    `lecture_seule_jamais_defaite` ; règle commune des gardes lexicales, en tête des AC).
    - **Propriété** : aucun texte de `crates/` (`.rs` et `.sql`, squash compris, lignes de commentaire écartées) n'émet
      une des quatre formes de la spike § 4.1 qui défont `SET SESSION TRANSACTION READ ONLY`, hors des sites écrits en
      liste blanche.
    - **Formes à attraper** (témoins rouges sur source factice) : `SET SESSION TRANSACTION READ WRITE`, `START
      TRANSACTION READ WRITE`, `SET TRANSACTION READ WRITE`, `SET SESSION tx_read_only=0`, `SET @@session.tx_read_only
      := 0`, ``SET SESSION `tx_read_only`=0``, `READ/**/WRITE` (le commentaire de bloc est remplacé **par une espace**
      avant l'analyse, comme le fait MariaDB), `transaction_read_only`.
    - **Formes légitimes à ne pas attraper** (témoins verts, **existants** au dépôt — validation P3, R1 = F1) : le
      `scope` de clé d'API `read-write` (`routes/api_keys.rs:136`, `migrations/20260605000001_api_keys.sql:34`, une
      trentaine de tests), `use std::io::{Read, Write};` (`document_storage.rs:18`) ; et, **sur source factice**, une
      ligne `/// … READ WRITE …` (aucune n'existe au dépôt : c'est le témoin du retrait des commentaires ; validation P4,
      R4 = F6).
      ⚠️ Le motif `\bread\W+write\b` de la P2 rendait **53 lignes** de code et de SQL (mesuré par la P3) : il est
      retiré.
    - **Exemples mesurés** (au relevé de `c6523903` (code de `b723e3d9`), sur `crates`, `.rs` et `.sql`) : `(?i)\bread\s+write\b` → **0**
      ligne ; `(?i)tx_read_only|transaction_read_only` → **0** ligne. À re-mesurer à l'étape 0.
    - **Liste blanche** (fichier, ligne) : les tests (e) et (e bis) de l'AC-a25 — **non** (f), qui n'exécute aucune forme
      interdite (validation P4, F7 = R4) —, la lecture `SELECT
      @@session.tx_read_only` de `services::serveur_mcp::etat` (AC-a28), le fichier de la garde.
    - **Angle mort écrit** : une forme coupée sur deux lignes (`READ` puis `WRITE` à la ligne suivante) échappe à une
      garde par ligne (§ *Ce que les gardes ne voient pas*).
28. **AC-a28 — l'outil `kesh_version`** (`mcp/outils/kesh_version.rs`), portée `BooksRead`, schéma `{"type":"object",
    "properties":{},"additionalProperties":false}`, appelle `services::serveur_mcp::etat(&ContexteAcces, &Lecture)` —
    `exiger(EXIGENCE_ETAT)` d'abord (`role_api: Consultation`, `portee_mcp: Some(BooksRead)`), puis **une** requête
    `SELECT @@session.tx_read_only` sur `lecture.pool()` — et rend `structuredContent: {"version": …, "lectureSeule":
    bool}` et un bloc texte qui le sérialise. **Aucune donnée des livres.** La structure de l'outil est une unité sans
    champ (construite par `Registre::standard()`, sans pool). *(Amendé le 2026-10-25, revue de code P1, E1, C-32-6-54 :
    la même requête lit aussi `@@session.max_statement_time`, rendu en `delaiInstructionS` — un réglage de la session,
    non une donnée des livres.)*

**A.8 — L'inventaire et ses gardes**

29. **AC-a29 — l'inventaire clos des chemins vers un outil** (§ *L'inventaire clos*, N1 à N28) est tenu par les gardes
    de l'AC-a30 et les tests nommés ; chaque site **non résolu** y est soit fermé, soit écrit comme angle mort. Aucune
    donnée réelle : fixtures de démonstration et valeurs piégées seulement ; `~/kesh-test/` jamais ouvert.
30. **AC-a30 — gardes lexicales des chemins** (`mcp_gardes_lexicales.rs` ; règle commune en tête des AC). Pour chacune,
    la **propriété**, son **périmètre** et les **formes à attraper** ; le motif exact se fixe au développement, mesuré
    sur le dépôt réel. Témoins rouges : les formes listées ; témoins verts : une forme légitime **existante**.
    - `un_seul_point_d_entree` — *propriété* : dans `crates/kesh-api/src`, hors de `src/mcp/`, seuls `lib.rs`
      (déclaration du module, `build_router_avec_registre`) et `routes/mcp.rs` nomment le module `mcp` ; `routes::mcp::`
      n'apparaît qu'à la ligne de route de l'AC-a1. *Formes* : `crate::mcp::…`, `mcp::…` en tête de chemin, `use
      crate::mcp`, `super::mcp`. *Témoins verts existants* : `ContexteAcces::mcp` (`services/acces.rs:84`, `pub fn mcp(`)
      et `portee_mcp` (`services/acces.rs:58`) ; `cles_mcp::` et `serveur_mcp::`, qui n'existent pas encore au relevé, se
      recontrôlent à T-a0 (validation P4, F6, R12).
    - `executer_appele_par_le_repartiteur_seul` — *propriété* : dans `crates/kesh-api/src`, l'appel d'`executer` d'un
      `McpTool` et des deux points d'accès du registre (`outil_pour`, `descripteurs_pour`, AC-a21) ne se fait qu'en
      `mcp/repartiteur.rs` ; les lignes de **définition** (`fn executer`, `fn outil_pour`, `fn descripteurs_pour`, dans
      le trait, les `impl` et `registre.rs`) sont hors du compte. *Formes* : appel de méthode, forme qualifiée
      (`McpTool::executer(&*o, …)`, `Registre::outil_pour(&r, …)`). *Témoin vert* : **sur source factice** — aucun n'existe
      dans le périmètre (validation P5, R7) — les lignes de définition ; l'`executer` d'un test existant
      (`tests/bank_accounts_closed_e2e.rs:1692`) est hors périmètre. Même chose pour `autoriser`/`appliquer`. *Mesure* : `\b(executer|outil_pour|
      descripteurs_pour)\s*\(` sur `crates/kesh-api/src` → 0 au relevé (validation P3, R2 : `lister` et `trouver`,
      retenus en P1, rougissaient sur six fonctions de services).
    - `contexte_mcp_construit_par_le_repartiteur_seul` — *propriété* : `ContexteMcp::nouveau` n'est appelé qu'en
      `mcp/repartiteur.rs`.
    - `contexte_acces_mcp_construit_par_authentifier_seul` (N24 ; forme simplifiée en validation P4, F9) —
      *propriété* : (1) dans `crates/kesh-api/src`, `ContexteAcces::mcp` n'est nommée qu'en `services/acces.rs`
      (définition) et `services/cles_mcp.rs` ; (2) **hors de `services/`**, le jeton `ContexteAcces` n'apparaît dans
      **aucune** ligne de code de `src/mcp/` sauf `mcp/outil.rs` (le type du champ de `ContexteMcp`) ; (3) `impl
      ContexteAcces` n'apparaît qu'en `services/acces.rs`. Ce qui ferme aussi `<ContexteAcces>::mcp(…)` et un `Self::mcp`
      dans un `impl` inhérent écrit ailleurs.
    - `registre_standard_en_un_lieu` — *propriété* : `Registre::standard()` n'est appelé qu'en `lib.rs` ; `main.rs`
      n'appelle ni `build_router_avec_registre` ni `Registre::`.
    - `mcp_sans_pool` (tout `src/mcp/**` et `routes/mcp.rs` — validation P2 F3, P3 F3) — *propriété* : aucun accès à
      un pool ni à ce qui en fabrique un. *Formes* : `MySqlPool`, `sqlx`, `AppState`, `.pool`, `kesh_db::repositories`,
      **`kesh_db::pool`**, **`create_pool`**, **`database_url`**, **`Config`** ; et, dans `src/mcp/outils/`, en plus,
      `EtatMcp` et `ContexteAcces::`.
    - `outils_et_politique_en_leur_lieu` (N27) — *propriété* : `impl … McpTool for` n'apparaît dans `crates/kesh-api/src`
      qu'en `src/mcp/outils/` ; `autoriser` et `appliquer` de la politique ne sont appelés qu'en `mcp/repartiteur.rs`, les
      lignes de **définition** (`fn autoriser`, `fn appliquer`) et le `mod tests` de `politique.rs` étant hors du compte
      (validation P4, F10 = R12) ; `LectureOutils::nouvelle` n'est appelé qu'en `lib.rs` ;
      `Donnees::nouvelles` qu'en `src/mcp/outils/`. *Mesure* : `\b(autoriser|appliquer)\s*\(` sur `crates` → 0 au
      relevé.
    - `un_seul_ecouteur` — *périmètre* `crates/kesh-api/src` (les soixante-dix fichiers de test qui lient un écouteur
      sont hors du compte ; validation P4, F10 = R12) : un écouteur n'est lié qu'en `main.rs` (`:390`). *Formes* :
      `tokio::net::TcpListener::bind`, `std::net::TcpListener`, `tokio::net::TcpSocket`, `UnixListener::bind` ; `pool_lecture_seule_en_un_lieu` — dans
      `crates/kesh-api/src`, appelé en `lib.rs` seulement (une ligne) ; `tests/common/lecture_seule.rs` et
      `crates/kesh-db/tests/pool_lecture_seule.rs` sont **hors** du périmètre, ce que la garde dit.
    - `donnees_construites_par_la_politique_seule` (AC-a23) ; `prompts_scelles` (AC-a24) ;
      `lecture_seule_jamais_defaite` (AC-a27) ; `reponses_par_rendre_seul` (AC-a31).
31. **AC-a31 — une seule fonction produit les réponses de `/mcp`, à partir de types propres au MCP (validation P2,
    R2, C-32-6-33 révisé, C-32-6-37 ; N25).** `impl IntoResponse for AppError` (`errors.rs:1520`) rend le corps **de l'API**, dont
    `Validation(msg)` recopie le message (`errors.rs:1538-1540`). Plutôt qu'une garde sur les formes qui y mèneraient, la
    conception ferme la classe :
    - toute réponse de `/mcp` sort de **`pub async fn rendre(etat: &EtatMcp, r: ReponseMcp) -> Response`**
      (`mcp/reponse.rs`), où `ReponseMcp` est une énumération **propre au MCP** — `RouteAbsente(Request)` (la requête
      entière, corps non lu, que `rendre` remet à `etat.statique` par `ServiceExt::oneshot` puis convertit en `Response`
      par `.map(Body::new)`), 401, 403, 405, 413, 415, 400 texte, erreur JSON-RPC, `Outil(DonneesEnvoyables)` (JSON tiré
      par `en_resultat`, AC-a23), `result` statique, 202, 500 — qui ne contient **ni** `AppError` **ni** `Response` ; le
      gestionnaire `recevoir` de `routes/mcp.rs` — le seul depuis la P4 — a pour corps **exactement** `rendre(&etat, transport::traiter(&etat, pair,
      requete).await).await`. *Pourquoi `async` et un état* (validation P3, R4 = F4, C-32-6-39) : la réponse de route
      absente est celle du service statique, qui ne se produit qu'en l'appelant avec la `Request` ; une `rendre`
      synchrone et sans état ne le pouvait pas. C'est la forme la plus simple qui garde **une** sortie, la délégation
      comprise ;
    - `transport::traiter` et le répartiteur rendent `ReponseMcp`, jamais `Result<_, AppError>` vers le framework ;
    - `AppError` n'est **convertible** que par `textes::depuis_app_error` (AC-a18) et par le `match` du résultat
      d'`authentifier` (AC-a14) ;
    - garde `reponses_par_rendre_seul` — **inventaire des sites** (non des formes) : dans `src/mcp/**` et
      `routes/mcp.rs`, les lignes de code qui contiennent `into_response`, `IntoResponse` ou `AppError` appartiennent à
      une liste **écrite** dans le test (fichier, ligne réduite de ses blancs) : les lignes de `rendre` (`reponse.rs`,
      dont la conversion de la branche statique), `depuis_app_error` (`textes.rs`), le `match` d'`authentifier`
      (`transport.rs`) et leurs `use` ; toute autre ligne rougit ; le test `traduction_des_erreurs_de_service` vit hors
      de `src/` (AC-a18). Témoin sur source factice, la forme que la P2 donnait : `match traiter(…).await { Ok(j) =>
      j.into_response(), Err(e) => e.into_response() }` → rouge.
    Test : pour chaque refus (route absente, 401, 403, 405, 413, 415, 400, 500), le corps est **égal** à sa constante —
    aucune réponse au format `{"error":{"code":…}}` de l'API (`build_response`, `errors.rs`).

### Partie B — 32-6b : le journal des appels

32. **AC-b1 — chaque `tools/call` est journalisé**, servi ou non, après son issue et **avant** l'envoi de la réponse :
    une entrée `audit_log` d'action **`mcp.tool_called`**, type d'entité `mcp_key`, `entity_id` = l'id de la clé,
    `user_id` = le créateur de la clé, `actor_type = 'user'` (C-32-6-21) ; détail **exactement**
    `{"cleMcpId": <id>, "outil": <nom>, "issue": <code>, "dureeMs": <entier>, "validation": "required"|"exempt"}`, où
    `outil` est le nom **d'un outil du registre**, ou la constante `"(inconnu)"` — **jamais** le nom fourni par le
    client s'il n'est pas au registre — et `issue` ∈ {`servi`, `validation_requise`, `inconnu`, `arguments_invalides`,
    `introuvable`, `erreur`, `delai_depasse`} (énumération fermée ; `inconnu` couvre hors portée et refus de service).
    Écrit par `services::serveur_mcp::journaliser_appel(&Trace, …)` — seule fonction neuve de `services/` qui emploie
    `trace.pool()` ; le doc-comment de `Trace::pool` (`poignees.rs:46-49`) la nomme ; **dérogation écrite** aux règles
    R1, R2 et R7 (une écriture d'audit, sans `exiger`), et l'inventaire clos des écritures sur les chemins de lecture
    (`services/mod.rs:77-90`, « 3. Aucune autre ») gagne cette écriture — `services/mod.rs` est aux fichiers de la 32-6b
    (validation P2, R14). Test
    `chaque_appel_est_journalise` (validation P1, F13) : un `tools/call` pour chaque issue — et, pour
    `arguments_invalides`, ses **deux** sources, argument refusé par l'outil et demande mal formée (AC-a19 ; validation
    P6, H5) — (registre de test) ajoute
    **exactement une** ligne, dont le détail est **égal** au JSON attendu (`dureeMs` contrôlé par son type et ≥ 0).
33. **AC-b2 — sans arguments ni données.** Test `journal_sans_arguments_ni_donnees` : `tools/call` de l'outil piégé
    (clé `exempt`) avec un argument `PIEGE_ARG_326` et un nom d'outil inconnu `PIEGE_NOM_326` → aucun des trois pièges
    dans `audit_log.details_json` ni `actor_label` ; chaque entrée a **exactement** les cinq clés de l'AC-b1.
34. **AC-b3 — le journal fail closed (C-32-6-30).** Si l'écriture du journal échoue, **rien ne part** : la réponse
    devient `isError: true` à texte fixe, sans `structuredContent` ni donnée. Test `journal_indisponible_rien_ne_part` :
    `ALTER TABLE audit_log RENAME TO audit_log_cachee_326` par le pool ordinaire, après la construction du routeur
    (l'authentification, sur `mcp_keys`, et l'outil, sur `accounts`, restent possibles ; seule l'écriture du journal
    échoue — validation P3 : fermer le pool de l'application, comme l'écrivait la P1, ferait désormais échouer
    l'authentification, qui lit sur `etat.garde`) → `tools/call` de l'outil piégé sous clé `exempt` rend un corps
    **égal** à la réponse d'erreur attendue.
35. **AC-b4 — ce qui n'est pas journalisé, et pourquoi** (C-32-6-24) : `initialize`, `ping`, `tools/list`,
    `prompts/*` (la 32-10a journalisera `prompts/get` quand il servira un texte), les notifications, et **les refus du
    transport, avant le répartiteur** (route absente, 401, 403, 405, 413, 415, 400 — certains après l'authentification,
    comme le 401 d'une société qui ne sert pas et le 413 ; validation P2, F13) — au **journal du serveur** seulement
    (`tracing`). Test :
    un 401 et un `tools/list` n'ajoutent aucune ligne à `audit_log`.
36. **AC-b5 — libellés.** `mcp.tool_called` à `audit_labels::ACTIONS` (ordre trié), libellé dans les **quatre**
    locales (`audit-log-action-…`, repli égal à la valeur fr-CH) ; `audit_label_registry.rs` vert ; l'action est un
    **littéral** au site `NewAuditLogEntry::user(` (règle R4 de `services`).
37. **AC-b6 — le registre des routes suit (validation P1, F7).** `tests/audit_route_registry.rs` : `("route_service",
    "mcp::recevoir", …)` (le verbe de l'AC-a6 ; validation P5, R1) passe de `Exempt("issue #723 — …")` à **`Traced`** (rejeu `SansEcritureAuJournal`
    inchangé, AC-a6) ; compteurs littéraux qui bougent, nommés : `traced` (`:770-771`, +1, généalogie « plus chaque appel
    d'outil MCP (32-6b, #723) »), `exempt` (`:788`, −1, message rétabli) ; `LIB_ROUTES.len()`, totaux et partition de
    rejeu **inchangés** (recomptés, cités au Change Log).

## L'inventaire clos des chemins par lesquels une requête atteint un outil

*Règle de `CLAUDE.md` (§ Inventorier les sites NON RÉSOLUS) : non pas les formes qui marchent, mais l'ensemble clos des
sites qui ne résolvent pas, chacun fermé ou écrit comme angle mort.* L'ensemble considéré : **tout appel de
`McpTool::executer`**, **tout chemin qui ferait partir une donnée lue par un outil**, et **toute fabrication d'un
contexte qui ouvrirait les services au nom d'une clé**.

**Le seul chemin servi** : écouteur de `main.rs` → routeur de `build_router` → `route_service("/mcp",
routes::mcp::recevoir.with_state(EtatMcp))` → `mcp::transport` (pair → règle « servi » → méthode → `Origin` → `Content-Type` →
`MCP-Protocol-Version` → clé → société de la clé servie et son `Origin` → corps borné → JSON-RPC) →
`mcp::repartiteur` (méthode de la liste close → `tools/call` → `Registre::outil_pour(nom, portées)` → `politique::autoriser`
→ `executer` sous délai → `politique::appliquer` → [32-6b : journal] → réponse).

| # | Site non résolu | Comment il se ferme | Par quoi |
|---|---|---|---|
| N1 | une autre route, un `nest`/`merge`, une tâche de fond (`auto_backup`), un point de test (`/api/v1/_test`) qui appellerait `crate::mcp` | aucun code hors de `src/mcp/`, `lib.rs`, `routes/mcp.rs` ne nomme `crate::mcp`/`mcp::` | garde `un_seul_point_d_entree` (AC-a30) |
| N2 | un second écouteur (port dédié, socket) | une seule ligne `TcpListener::bind`, celle de `main.rs` | garde `un_seul_ecouteur` |
| N3 | un appel de `executer` hors du répartiteur | `executer(` (méthode et forme qualifiée) en `mcp/repartiteur.rs` seulement | garde `executer_appele_par_le_repartiteur_seul` |
| N4 | `outil_pour`/`descripteurs_pour` court-circuités, un outil obtenu sans filtre de portée | seuls points d'accès au registre, en `repartiteur.rs` ; `outils` privé au `Registre` | garde ; champ privé (compilateur) |
| N5 | une méthode JSON-RPC servie hors liste (`resources/read`…) | énumération fermée, repli `-32601` | test d'inventaire (AC-a12) |
| N6 | une **notification** `tools/call` (sans `id`) | jamais remise à un gestionnaire de requête : 202 | test au compteur (AC-a11) |
| N7 | un **lot** dont un élément serait traité | 400 avant tout traitement | test (AC-a11) |
| N8 | un message de type réponse | 400, jamais traité | test (AC-a11) |
| N9 | une méthode HTTP autre que `POST` (`GET`, `OPTIONS`…) | `/mcp` est un service (`route_service`) : **toute** méthode arrive au transport, qui rend route absente ou 405 avant toute authentification ; aucun `MethodRouter`, donc aucun en-tête ajouté par Axum | détecteur étendu à `route_service(` (AC-a6) ; tests (AC-a3, AC-a5) ; garde N3 |
| N10 | un chemin voisin (`/mcp/`, `/MCP`, `/%6Dcp`, `//mcp`) | soit route absente, soit, si Axum le résout vers `/mcp`, **le même gestionnaire** et toutes ses gardes | test `chemins_voisins` (réponse égale à la route absente, ou garde complète ; jamais un outil sans clé) |
| N11 | l'authentification par cookie de session, par PAT, par JWT | le transport ne lit que `Bearer` ; `authentifier` refuse tout préfixe autre que `kesh_mcp_` | tests (AC-a14) |
| N12 | une clé dont la société ne sert pas, ou dont le filtre refuse le pair | contrôle après authentification → 401 fixe | tests (AC-a4) |
| N13 | un pair déguisé par `X-Forwarded-For`, `X-Real-IP`, `Forwarded` | en-têtes lus seulement d'un mandataire déclaré, de droite à gauche ; les deux autres jamais | tests (AC-a9) |
| N14 | `ConnectInfo` absent | `Err` de l'extracteur → route absente | test `oneshot` sans `MockConnectInfo` (AC-a7) |
| N15 | une clé `required` qui ferait exécuter un outil | refus avant `executer` | test au compteur (AC-a15) |
| N16 | des données d'outil envoyées hors de la politique, ou un outil exécuté sans elle | `autoriser` avant `executer`, `Autorisation` constructible en `politique.rs` seul ; `Donnees` illisible hors de `politique.rs`, `DonneesEnvoyables` constructible là seulement, ni `Serialize` ni `Default` ni `Debug` | compilateur (champs privés) + gardes (AC-a23, AC-a30) + tests (AC-a15, AC-a22) |
| N17 | une donnée lue logée dans une erreur d'outil | `ErreurOutil` sans champ libre ; textes fixes | type + test (AC-a20) |
| N18 | une donnée lue logée dans `tools/list` | `nom`, `description`, `schema_entree` sont `&'static str` | type (AC-a16) |
| N19 | un outil qui capturerait un pool ou l'état à sa construction | outils construits par `Registre::standard()`, unités sans champ ; aucun pool nommé dans `src/mcp/` | garde `mcp_sans_pool` + revue |
| N20 | un outil qui écrirait | un outil n'a que `ContexteAcces` et la `Lecture` des outils (via `LectureOutils`), dont le pool n'est lisible que dans `services` ; la connexion refuse l'écriture | compilateur (`pub(in crate::services)`) + `READ ONLY` + connexion fermée à chaque restitution (`after_release`, AC-a25) — **angle mort** : voir ci-dessous |
| N21 | un refus de service (`Forbidden`) qui révélerait l'existence d'un outil | traduit en « outil inconnu » | test (AC-a18) |
| N22 | un prompt qui porterait des données lues | aucun prompt en 32-6 ; `rendre` sans `Lecture`, trait scellé | garde `prompts_scelles` — **à reprendre par la 32-10a** |
| N23 | `unsafe` (transmutation d'une poignée) | `forbid(unsafe_code)` sur la lib (`lib.rs:9`) | compilateur |
| N24 | **un contexte d'accès fabriqué** à société, utilisateur et portées arbitraires (`ContexteAcces::mcp`, public, `services/acces.rs:84`) et passé à un service par un outil, le transport ou une tâche | `ContexteAcces::mcp(` appelé en production par `authentifier` seul ; un outil ne nomme pas `ContexteAcces::` | gardes `contexte_acces_mcp_construit_par_authentifier_seul` et `mcp_sans_pool` (AC-a30) — la fonction reste publique pour les tests d'intégration (`services_motif.rs`), d'où une garde et non une visibilité |
| N25 | **une `AppError` rendue par `IntoResponse`** (corps de l'API, message recopié) | toute réponse sort de `reponse::rendre(ReponseMcp)`, énumération sans `AppError` ; `AppError` convertie par `depuis_app_error` et le `match` d'`authentifier` seuls | conception (AC-a31) + garde d'inventaire `reponses_par_rendre_seul` + test d'égalité de chaque refus |
| N26 | **un panic** dans un outil ou le transport ; **l'abandon** de la requête par le client | un panic interrompt la connexion, l'abandon annule le futur : **aucune donnée ne part** dans les deux cas *(amendé le 2026-10-25, revue de code P2, C-32-6-56 : dans un **outil**, le panic est pris dans sa tâche et rend `TEXTE_ERREUR_INTERNE`, détail au journal du serveur ; l'abandon n'annule plus l'outil, qui va au bout dans sa tâche, résultat jeté — aucune donnée ne part non plus)* | **angle mort** : ni réponse, ni ligne de journal (32-6b) — voir ci-dessous |
| N27 | **un appel de `appliquer`/`autoriser` hors du répartiteur, ou un `impl McpTool` hors de `src/mcp/outils/`** (qui échapperait aux gardes de ce dossier) ; **le pool ordinaire** à portée du transport | `EtatMcp` sans `AppState` ; sites de la politique, des outils et de `Donnees::nouvelles` écrits ; ⚠️ depuis la P3, le pool ordinaire **est** à portée du transport sous `etat.garde` (lisible des seuls `services`) : il ne peut pas servir aux outils, `ContexteMcp::nouveau` exigeant un `LectureOutils` (N28) | gardes `outils_et_politique_en_leur_lieu` et `mcp_sans_pool` (AC-a30) |
| N28 | **la `garde` passée aux outils, ou la `lecture` des outils à la règle « servi » ou à l'authentification** (validation P4, F2 = R1) | deux types distincts : `Lecture` (garde) et `LectureOutils` (outils), ce dernier jamais déballé hors de `mcp/outil.rs` | compilateur (Ma51, Ma52 déclarées non compilables) ; garde `outils_et_politique_en_leur_lieu` (`LectureOutils::nouvelle` en `lib.rs` seul) — **angle mort** : un accesseur public ajouté à `LectureOutils` (revue) |

**Angles morts assumés** (écrits, non fermés) — chacun au § *Ce que les gardes ne voient pas* : une écriture **dans un
même emprunt** après une forme qui défait le mode de session (N20) ; une forme interdite **composée à l'exécution** ou
émise par une dépendance (N20) ; un service de `services/` qui emploierait `lecture.pool()` pour écrire (N20, vu par
l'exécution seulement) ; une macro qui fabriquerait un identifiant (N1, N3, N24) ; la configuration de Traefik et de
Docker (N13) ; un appel paniqué ou abandonné, non journalisé (N26).

## La frontière avec la 32-8

| | 32-6 | 32-8 |
|---|---|---|
| Lire la politique de la clé | ✅ au répartiteur, seul lecteur | — |
| `Required` | **refus** par `autoriser`, avant exécution, texte fixe (AC-a15) | change `autoriser` (le bras `Required` devient une autorisation de **dépôt**) et `appliquer` (qui dépose la proposition et rend un accusé sans donnée) : exécuter, déposer, rendre « envoi n° X en attente » |
| `Exempt` | exécution, envoi ; journal (32-6b) | inchangé (« seule voie hors validation », E1) |
| Table des envois, migration, sauvegarde | — | ✅ (🗄️) |
| `kesh_lire_envoi`, règle « même clé, même société », échéance, purge | — | ✅ (E4) |
| Écran de validation, représentation source (`content` ou `structuredContent`) | — | ✅ (E1, M-2 de la spike) ; lit `Donnees` **en `politique.rs`** (AC-a23) |
| Mutations d'E6 | 1, 2, 3, 5 | 4, 6 |
| `instructions` d'`initialize` | constante neutre | ajoute la consigne de relecture |
| `kesh_version` sous validation | refusé sous `required` (Q12 : la politique est celle de la clé) | « l'outil de démonstration y passe » (kickoff) |

Le point de branchement est **le module `politique`** : la 32-8 change `autoriser` et `appliquer` — **signatures
comprises** (le dépôt écrit dans la table des envois, sur un accès en écriture distinct de `Lecture` et de `Trace`,
spike E4) — et leurs **deux** sites d'appel au répartiteur ; rien d'autre du répartiteur (validation P2, F2, R15 : la
phrase « son bras `Required` et rien d'autre » allait au-delà de ce que la conception permettait).

## Tâches

### 32-6a
- [ ] **T-a0** — étape 0 (§ *À recontrôler*) : 32-5b, 32-5d, 32-5c fusionnées ; `git log -1` partie de `main` à jour ;
  recomptes et relectures, ligne lue citée au Change Log ; la liste des méthodes client du schéma `2025-11-25` (source
  citée) ; l'observation (h) de la spike § 2.4 faite ou non.
- [ ] **T-a1** — `kesh-db` : `pool_lecture_seule` et ses tests (AC-a25) ; `list_enabled` et ses tests (AC-a2).
- [ ] **T-a2** — `mcp/` : protocole JSON-RPC, transport, textes fixes et `depuis_app_error`, traits, `Donnees` et
  politique, registre, répartiteur, prompts, `kesh_version` (AC-a11 à AC-a24, AC-a28, AC-a31) ;
  `services::serveur_mcp` (`reglages_actifs`, `etat`).
- [ ] **T-a3** — garde réseau : client, mandataires, admission, `Origin` (AC-a7 à AC-a10) ; tests unitaires de
  l'analyse (`mcp/reseau.rs`) et `mcp_reseau_e2e.rs`.
- [ ] **T-a4** — câblage `lib.rs` (`route_service` et `EtatMcp`, `service_statique`, pool, `LectureOutils`,
  `build_router_avec_registre`), `routes/mcp.rs` ; registre des routes et son détecteur étendu à `route_service(`
  (AC-a6) ; `services_motif.rs` (v-1, v-2, vi) ; `tests/common/lecture_seule.rs` (AC-a1 à AC-a6, AC-a26) ; les
  doc-comments que la 32-6a rend faux, chacun avec son texte nouveau (validation P4, R6 ; recontrôlés à T-a0 après
  les 32-5b et 32-5c, qui en reprennent une part) : `services/poignees.rs:24` (« seul `AppState::lecture()`
  l'appelle » → « `AppState::lecture()` et le câblage du MCP, `build_router_avec_registre` ») et `:11`,
  `lib.rs:147-150`, `services/mod.rs:53-54` et `:199-202` (au futur « réalisés par la 32-6 » → au présent, et
  `AppState::lecture()` reste ordinaire, servant aussi de `garde`). La ligne `route_service("/mcp", …)` se place **avant**
  le marqueur `// NOTE: les stories futures ajouteront leurs routes protégées`, où les deux volets du registre des
  routes coupent `lib.rs` (`audit_route_registry.rs:529-533`, `:704-708`) — après lui, `extract_counted` ne la verrait
  pas (validation P5, R12 (a)).
- [ ] **T-a5** — `mcp_transport_e2e.rs`, `tests/common/mcp.rs`, gardes lexicales (AC-a14, AC-a15, AC-a17 à AC-a23,
  AC-a27, AC-a30, AC-a31).
- [ ] **T-a6** — mutations Ma1 à Ma52 (et Ma13b, Ma20b, Ma23b), gates ; Dev Agent Record (n'affirmer que ce qui a tourné, avec périmètre).

### 32-6b
- [ ] **T-b0** — étape 0 : 32-6a fusionnée ; recontrôle des faits 8, 9, de `audit_labels.rs` et des compteurs
  d'`audit_route_registry.rs`.
- [ ] **T-b1** — `journaliser_appel`, appel au répartiteur, fail closed (AC-b1, AC-b3, AC-b4).
- [ ] **T-b2** — libellés, quatre locales (AC-b5) ; registre des routes (AC-b6) ; `mcp_journal_e2e.rs` (AC-b1 à AC-b4).
- [ ] **T-b3** — mutations Mb1 à Mb6, gates.

## Tests et mutations prévus

Une mutation à la fois, restaurée par `git checkout -- <f> && touch <f>` (`memory/mutation-restauree-binaire-perime`),
diff résiduel nul vérifié ; chacune nomme le test qui doit rougir. Une mutation qui survit se déclare ; une mutation qui
**ne compile pas** se déclare comme telle (la propriété est alors tenue par le compilateur) et ne compte pas comme tuée.

| # | Mutation | Doit rougir |
|---|---|---|
| Ma1 | retirer l'`after_connect` `READ ONLY` de `pool_lecture_seule` | `pool_lecture_seule` (a)–(c) ; `kesh_version_lit_en_lecture_seule` |
| Ma2 | `after_release` qui rend `Ok(true)` (connexion rendue au repos) | `pool_lecture_seule` (e) et (e bis) |
| Ma3 | `EtatMcp.lecture` sur `state.pool` (pool ordinaire) | `kesh_version_lit_en_lecture_seule` ; garde v-2 |
| Ma4 | `EtatMcp.trace` construite sur le pool en lecture seule (site de `lib.rs`) | `l_appel_pose_la_derniere_utilisation` |
| Ma5 | `tools/list` non filtré par les portées | `tools_list_filtre` |
| Ma6 | `outil_pour` ignore la portée | `hors_portee_indiscernable` (compteur) |
| Ma7 | hors portée → message distinct | `hors_portee_indiscernable` (égalité des corps) |
| Ma8 | `Forbidden` d'un service rendu en `Interne` (ou tel quel) | `hors_portee_indiscernable` (cas service) |
| Ma9 | filtre d'adresse retiré (tout pair admis) | `mcp_reseau_e2e` (a), (c) |
| Ma10 | `X-Forwarded-For` lu d'un pair non déclaré | (e) |
| Ma11 | `X-Forwarded-For` lu de gauche à droite | (i) |
| Ma12 | pair mappé non normalisé | (c) |
| Ma13 | entrée invalide de **`allowed_networks`** traitée comme « tout admettre » (ou faisant écarter la société) | (n) |
| Ma13b | entrée invalide de **`trusted_proxies`** ignorée (la société reste dans S) | (n') |
| Ma14 | `X-Real-IP` ou `Forwarded` lu | (m) |
| Ma15 | vérification d'`Origin` retirée | tests de l'AC-a10 (403) |
| Ma16 | `Origin` admise si égale à `Host` | AC-a10 (`Host` forgé) |
| Ma17 | **un PAT accepté** : dans `services/cles_mcp.rs` (32-5b), le refus du préfixe d'`authentifier` est retiré et un jeton `kesh_pat_` est cherché dans `api_keys` par la `Lecture` (le seul site où un PAT pourrait passer : depuis la P2, `src/mcp/` ne peut nommer aucun pool, `validate_pat` exige un `&MySqlPool`, `auth/api_key.rs:153-155` — validation P3, F2) | `pat_refuse_a_l_endpoint`, dont le **témoin inverse** (même route, même requête au jeton près : une clé MCP valide passe, un PAT actif rend le 401) |
| Ma18 | cookie de session lu | `session_refusee` |
| Ma19 | refus d'authentification à corps distincts selon la cause | `refus_indiscernables` |
| Ma20 | désactivé → `404` littéral (au lieu de déléguer au service statique) | `desactive_indiscernable_d_une_route_absente` |
| Ma20b | `/mcp` remonté par un `MethodRouter`, sous ses **deux formes compilables** (validation P5, F1 = R2 : `any(routes::mcp::recevoir)` exigeait `EtatMcp: FromRef<AppState>` et ne compilait pas) : **(i)** `.route("/mcp", post_service(routes::mcp::recevoir.with_state(etat)))` → un `GET` gagne `Allow: POST` (`method_routing.rs:1168-1171`, écrit en `route.rs:164`) ; **(ii)** `.route("/mcp", any_service(routes::mcp::recevoir.with_state(etat)))` → aucun `Allow` (`skip_allow_header`, `method_routing.rs:427-436`), mais une 405 gagne `content-length: 0` (`route.rs:163-167`) | (i) et (ii) : `desactive_indiscernable_d_une_route_absente`, **par `oneshot`** — (i) aussi sur le fil, (ii) **seulement** par `oneshot` : sur une vraie connexion, hyper pose lui-même `content-length: 0` sur une réponse vide dont statut et méthode le permettent (`hyper-1.10.1/src/proto/h1/role.rs:512-521`, `:924-931`) |
| Ma21 | contrôle « société de la clé servie » retiré | AC-a4 (i), (ii) |
| Ma22 | erreur de base dans la règle « servi » → 500 | AC-a2 (table renommée, égalité à la route absente) |
| Ma23 | `politique::autoriser` rend `Ok` pour `Required` | `cle_requise_refusee_sans_execution` ; `e1_rien_ne_part_sous_cle_requise` (E6-1) |
| Ma24 | `DonneesEnvoyables` à champ `pub`, construit dans le répartiteur | garde `donnees_construites_par_la_politique_seule` (E6-2) |
| Ma25 | `resources/list` servi (rend `[]`) | `methodes_servies_liste_close` (E6-3) |
| Ma26 | `ErreurOutil::Interne(String)` portant `e.to_string()` | `erreur_interne_texte_fixe` (égalité du corps, E6-5) |
| Ma27 | notification `tools/call` exécutée | test au compteur (AC-a11) |
| Ma28 | lot : premier élément traité | AC-a11 (lot) |
| Ma29 | `MCP-Protocol-Version: 2026-07-28` admis | AC-a11 (version) |
| Ma30 | `readOnlyHint` retiré | `tools_list_filtre` (égalité de la liste) |
| Ma31 | `Err` de `ConnectInfo` traité comme `127.0.0.1` | AC-a7 (`oneshot` sans `MockConnectInfo`) |
| Ma32 | corps lu par un extracteur `Bytes` dans `recevoir` | `desactive_indiscernable_…` (corps de 3 Mio : 413 au lieu de 405) ; témoin 401 |
| Ma33 | message de `serde` recopié dans l'erreur d'arguments | AC-a19 (égalité du corps) |
| Ma34 | `Origin` de la société non rejouée après l'authentification | AC-a10 (clé de B, origine de A) |
| Ma35 | délai d'outil retiré | `delai_depasse` (AC-a20) |
| Ma36 | une seule ligne `X-Forwarded-For` lue | AC-a9 (p) |
| Ma37 | entrée `X-Forwarded-For` non normalisée | AC-a9 (q) |
| Ma38 | `Content-Type` non contrôlé | AC-a11 (415) |
| Ma39 | `MCP_CORPS_MAX` retiré (`usize::MAX`) | AC-a3 (témoin 413) |
| Ma40 | un outil fabrique `ContexteAcces::mcp(…, toutes les portées)` | garde `contexte_acces_mcp_construit_par_authentifier_seul` |
| Ma41 | une réponse produite hors de `rendre` (`match traiter(…) { Ok(j) => j.into_response(), Err(e) => e.into_response() }`, `traiter` rendant `Result<_, AppError>`) | garde `reponses_par_rendre_seul` ; `refus_indiscernables` |
| Ma42 | `#[derive(Default, Debug)]` sur `Donnees` | garde `donnees_construites_par_la_politique_seule` |
| Ma43 | `route_service` retiré des formes lues par `extract_counted` (test, non production) | test sur source factice (AC-a6) ; `every_mutating_route_of_lib_is_in_the_registry` |
| Ma44 | origines stockées non normalisées | AC-a10 (entrée `:443`) |
| Ma45 | `extract_counted` n'envoie pas à l'inventaire un `route_service` à argument non `routes::` | test factice `.route_service("/x", users::h.with_state(e))` → non résolu (AC-a6) |
| Ma46 | `Database(DbError::NotFound)` rendu en `Interne` | `traduction_des_erreurs_de_service` |
| Ma47 | `Validation(msg)` recopie `msg` | `traduction_des_erreurs_de_service` (égalité, `PIEGE_APP_326`) |
| Ma48 | `EtatMcp` regagne un champ `app: AppState` (ou `pool`) | garde `mcp_sans_pool` |
| Ma49 | `X-Real-IP` lu dans la branche mandataire | AC-a9 (m') |
| Ma50 | `executer` appelé **avant** `autoriser` | `cle_requise_refusee_sans_execution` (compteur à 1) |
| Ma51 | `ContexteMcp::nouveau(&acces, &etat.garde)` (les outils sur la `Lecture` ordinaire) | **ne compile pas** (`LectureOutils` attendu) — déclarée ; le témoin d'exécution reste `kesh_version_lit_en_lecture_seule` |
| Ma52 | `reglages_actifs(&etat.lecture)` ou `authentifier(&etat.lecture, …)` (la règle « servi » sur le pool des outils) | **ne compile pas** (`Lecture` attendue) — déclarée |
| Mb1 | appel de `journaliser_appel` retiré | `chaque_appel_est_journalise` |
| Mb2 | arguments recopiés au détail | `journal_sans_arguments_ni_donnees` |
| Mb3 | nom d'outil fourni par le client écrit au journal | `journal_sans_arguments_ni_donnees` (`PIEGE_NOM_326`) |
| Mb4 | journal *best-effort* (échec avalé, réponse envoyée) | `journal_indisponible_rien_ne_part` |
| Mb5 | `journaliser_appel` appelé avec la `Lecture` des outils au lieu de la `Trace` (signature élargie : site distinct de Ma4, qui mute la construction de `EtatMcp`) | `chaque_appel_est_journalise` (1792, puis fail closed) |
| Mb6 | un 401 journalisé en base | AC-b4 |

Tests hors mutations, nommés au Dev Agent Record : `chemins_voisins` (N10), `registre_coherent`,
`registre_standard_inventaire`, `traduction_des_erreurs_de_service`, `jeton_jamais_journalise`, tests unitaires de `mcp/reseau.rs` (analyse de
`X-Forwarded-For`, normalisation d'origine, appartenance), témoins des gardes lexicales sur source factice.

## Gates

- **32-6a : 🗄️** (un dépôt de `kesh-db` et sa fabrique de pool) — **gate complet même en cours de boucle**
  (`CLAUDE.md`, exception `kesh-db`) ; base remise à zéro avant chaque gate complet (bases `kesh_326`/`kesh_e2e_326`,
  **sans** redémarrer le conteneur, `docs/agents-orchestration.md` § 2 et § 4), sous `flock …/.gate.lock`, par
  `scripts/test-fast.sh --ci` ; fmt, clippy `-D warnings`, build `--all-targets`. **Aucune migration** : P1 à P8 sans
  objet (à constater : `ls crates/kesh-db/migrations | wc -l` inchangé), P6 sans objet.
- **32-6b** : gate ciblé entre les passes (`binary(mcp_journal_e2e)`, `binary(audit_label_registry)`,
  `binary(audit_route_registry)`, fmt et clippy sur le workspace), complet au push ; frontend `npm run check`,
  `lint-i18n-ownership`, `test:unit`, `build` (par prudence : les `.ftl` sont lus par quelques tests Vitest — `contacts-i18n-realpath.test.ts`,
  `failed-proposal-label.test.ts` —, l'écran du journal ne traduisant pas les actions ; validation P5, F6).
- **E2E complet au dernier commit de code de chaque partie** : le routeur change (`/mcp` avant le repli statique) ; les
  échecs attendus se jugent fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **Recette visuelle : sans objet** — aucun contrôle visible ne change.
- **Boucle de revue complète, Sonnet ↔ Opus** pour chaque partie ; Haiku en passe ciblée seulement.

## Ce que les gardes ne voient pas

- **La lecture seule n'est pas une propriété de la base** (Q11). Le mode de session se défait par quatre instructions
  sans privilège (spike § 4.1) ; la fermeture de **chaque** connexion à sa restitution (`after_release`, AC-a25) ferme
  le trou **entre deux emprunts** pour les quatre formes (tests (e) et (e bis)), pas **dans** un emprunt (`START
  TRANSACTION READ WRITE; UPDATE …`, ou `SET TRANSACTION READ WRITE` suivi d'une écriture dans le même emprunt) ; la garde lexicale ne lit que le texte de `crates/`, commentaires
  écartés — ni un SQL composé à l'exécution, ni une dépendance. Un service qui écrirait par `lecture.pool()` n'est
  arrêté que par MariaDB, **sur les seuls chemins exécutés**.
- **Le filtre d'adresse voit le pair que voit Kesh, non le client réel.** Derrière Traefik non déclaré mandataire, tous
  les clients paraissent venir de son adresse (privée, admise par défaut) ; derrière une publication de port Docker
  par le mandataire en espace utilisateur (`userland-proxy`), tous peuvent paraître venir de la passerelle du pont
  Docker (`172.17.0.1`) : **tout le monde est admis**. Kesh ne le détecte pas (sauf si Guy retient la garde du point
  2 bis). **Non vérifié sur le NAS** — point 2 pour Guy, texte de la 32-7.
- **Traefik et `X-Forwarded-For`** : selon `forwardedHeaders.trustedIPs`, Traefik conserve ou écrase l'en-tête reçu ;
  la lecture de droite à gauche est juste dans les deux cas, à condition que seul Traefik soit déclaré. Non éprouvé
  contre un Traefik réel.
- **`Origin` ne protège que d'un navigateur.** La vraie barrière est la clé ; `Host` n'est pas contrôlé.
- **Indiscernable, mais pas invisible au temps de réponse** : la décision « servi » lit la base, une route absente non
  (oracle temporel, non fermé) ; une requête anonyme provoque une lecture en base (aucune limite de débit sur `/mcp`).
- **Une inondation du réseau local** (validation P3, R9, C-32-6-38) : la règle « servi » et l'authentification lisent
  sur `etat.garde` — la `Lecture` ordinaire, cinq connexions **réutilisées** (`main.rs:75`) — et **jamais** sur le pool
  des outils, qui n'est emprunté qu'après l'authentification : une requête anonyme n'ouvre donc aucune connexion neuve.
  Mais un pair admis qui inonde `/mcp` occupe ces cinq connexions comme il occuperait celles de `/api/v1/auth/login` ;
  Kesh ralentit pour tous, et le MCP rend alors la route absente (fail closed). **Non fermé** : aucune limite de débit
  sur `/mcp` ; soumis à Guy (point 8).
- **La clé valide d'une société qui ne sert pas** reçoit le 401 d'une clé inconnue, mais `authentifier` a posé sa
  dernière utilisation (validation P1, F11) : la colonne `last_used_at` ne prouve donc pas qu'un appel a été **servi**.
  Fermer ce point demanderait de séparer la vérification de la clé de l'écriture de sa dernière utilisation dans la
  32-5b ; à proposer si la 32-7 affiche cette colonne comme « dernier appel ». Ce 401 est égal **en octets**, non **en
  temps** : la vérification complète et l'écriture de `last_used_at` le rendent plus lent que celui d'une clé inconnue
  (oracle temporel, faible : il ne sert qu'à qui détient déjà une clé valide ; validation P2, F15).
- **Un panic ou un abandon (N26)** : aucune donnée ne part, mais l'appel n'est pas journalisé (32-6b) ; seul le journal
  du serveur garde le panic. *(Amendé le 2026-10-25, revue de code P2, C-32-6-56 : l'outil s'exécute désormais dans une
  tâche que l'abandon n'annule pas ; la 32-6b décidera si la ligne de journal d'un appel abandonné s'écrit dans cette
  tâche, à sa fin.)*
- **Une donnée recopiée dans un prompt** par un humain ou un agent approuvé (cas assumé d'E1) : la 32-10a.
- **La restriction de construction** (AC-a23) ne voit pas un `Box::leak` d'un texte fabriqué à l'exécution logé dans
  une réponse statique : la revue le lit à chaque ajout de méthode.
- **Les gardes lexicales** ne voient ni une macro qui fabriquerait un identifiant, ni un alias de chemin hors des formes
  qu'elles refusent explicitement (`use … as`) : la revue.
- **Le comportement des clients réels** (protocole demandé, `Origin` envoyé, tentative OAuth sur un 401 — un `GET
  /.well-known/oauth-protected-resource` rend la page de l'application) : spike § 2.4, Guy ou la 32-11.
- **Le journal (32-6b) ne distingue pas** « outil inconnu » de « hors portée » : voulu (C-32-6-21).
- **Le manuel** : aucun gate ne le lit ; les prompts de revue nomment l'axe.
- *(Ajouté le 2026-10-25, revue de code P1 de la 32-6a.)* **L'amplification du journal du serveur** (B3 = E7) : tant
  qu'une ligne de réglages est illisible ou qu'une entrée de `trusted_proxies` ne s'analyse pas, **chaque** requête
  vers `/mcp` — anonyme, de tout pair du réseau local, avant la méthode et la clé — écrit un `error!` par société en
  cause (et une erreur de la base, un `error!` par requête). Aucun secret n'y figure ; le débit n'est pas borné, faute
  de limite de débit sur `/mcp` (point 8 pour Guy). Laissé tel quel, sans dédoublonnage (C-32-6-6 : « sans état
  partagé ») : à reprendre avec la limite de débit, si Guy en pose une.
- *(Ajouté le 2026-10-25, revue P1, E1 = B2.)* **Une instruction SQL en vol au délai de l'outil** : fermée côté
  client, elle était poursuivie par MariaDB jusqu'à son terme — **mesuré** : sans borne, une `SLEEP(5)` abandonnée
  tourne encore 1,5 s après l'abandon (`326a-mut-P1-Mp1-g2-seul.log`). Depuis la remédiation, chaque connexion du pool
  des outils porte `max_statement_time` = le délai du registre (`DELAI_OUTIL`, 30 s) : MariaDB l'interrompt (1969).
  ~~**Ce qui reste** : la fenêtre entre la fin du délai côté Kesh et l'interruption par le serveur (les deux bornes sont
  égales ; une instruction peut donc occuper le serveur quelques millisecondes après que son permis est rendu) ;~~
  *(Corrigé le 2026-10-25, revue de code P2, H1 : la phrase barrée était fausse. `max_statement_time` se compte depuis
  le début **de l'instruction**, le délai de Kesh depuis l'entrée dans l'outil, emprunt compris : une instruction partie
  à `t0` survivait `t0` après l'abandon, jusqu'à presque 30 s ; et une **déconnexion** du client, qui abandonne le futur
  du handler, rendait le permis sur-le-champ, si bien qu'un porteur de clé pouvait empiler des instructions au-delà
  des trois connexions. **Fermé par construction** (C-32-6-56) : l'outil s'exécute dans une tâche `tokio::spawn` qui
  possède ses connexions ; le handler n'attend que la tâche, sous le délai. Un abandon, au délai ou par déconnexion,
  n'annule plus la tâche : chaque instruction garde son permis jusqu'à sa fin côté serveur — au plus
  `max_statement_time` après **son** début — et le nombre d'instructions simultanées reste borné par
  `MCP_POOL_CONNEXIONS`. Tests `abandon_au_delai_le_permis_reste_pris` et
  `abandon_par_deconnexion_le_permis_reste_pris` ; mutation P2-Mq1.)* **Ce qui reste, depuis C-32-6-56** : (i) une
  tâche abandonnée va **au bout de l'outil**, résultat jeté — pour un outil à plusieurs instructions (la 32-9), au plus
  une `max_statement_time` par instruction, plus l'attente d'un permis (le délai d'acquisition du pool) ; (ii) les
  tâches abandonnées **en attente d'un permis** ne sont pas bornées en nombre : un porteur de clé qui coupe ses
  requêtes en boucle fait attendre les appels légitimes derrière les siennes, sans dépasser trois instructions côté
  serveur — à reprendre avec la limite de débit (point 8 pour Guy) ; (iii) `max_statement_time` ne borne pas une
  instruction **en attente d'un verrou** de métadonnées au-delà de ce que MariaDB compte dans le temps d'exécution
  (non mesuré). **Critère pour la 32-9** : chaque outil de lecture des livres doit tenir sous `DELAI_OUTIL` sur les
  volumes de démonstration, et un test par outil le constate.

## Dev Notes

### Ordre des contrôles d'une requête (C-32-6-8)

```text
1. pair (Result<ConnectInfo>, to_canonical)               Err                 → route absente (Request entière remise)
2. S = réglages actifs admettant le client (etat.garde)  vide ou erreur base → route absente (Request entière remise)
3. méthode HTTP                                           ≠ POST              → 405 Allow: POST
4. Origin (propres ∪ allowed_origins de S, normalisées)   refusée             → 403 JSON-RPC, "id": null
5. Content-Type                                           ≠ application/json  → 415
6. MCP-Protocol-Version                                   non servie          → 400 text/plain ; absente → debug!
7. Authorization: Bearer → cles_mcp::authentifier         refus               → 401 fixe ; erreur base → 500 fixe
8. société de la clé ∈ S                                  non                 → 401 fixe (même corps qu'en 7)
9. Origin (propres ∪ allowed_origins de la société)       refusée             → 403
   (amendé le 2026-10-25, revue de code P2, H2 : → 401 fixe, même corps qu'en 7 — C-32-6-52)
10. to_bytes(corps, 1 Mio), JSON-RPC                      (AC-a11)
11. répartiteur : liste close → tools/call → outil_pour(nom, portées) → politique::autoriser (Required → refus, sans
    exécution ; premier emprunt du pool des outils après ce point seulement) → executer (délai du registre) → politique::appliquer → [32-6b] journaliser_appel (fail closed)
12. reponse::rendre(&etat, ReponseMcp).await → Response — seule sortie, à chaque étape (route absente comprise)
```

### Le canal (C-32-6-31, C-32-6-40, C-32-6-41)

```rust
// lib.rs, dans build_router_avec_registre — indicatif
let statique = service_statique(&static_dir);              // même fonction que le repli
let etat = mcp::EtatMcp {
    origines_propres: mcp::origines_propres(state.config.port, state.config.public_base_url.as_deref()), // reseau.rs
    garde: state.lecture(),                                // Lecture ordinaire : règle « servi », authentification
    lecture: mcp::LectureOutils::nouvelle(services::poignees::Lecture::new(
        kesh_db::pool::pool_lecture_seule(&state.pool, mcp::MCP_POOL_CONNEXIONS))), // type propre aux outils
    trace: state.trace(),
    registre: Arc::new(registre),                          // porte le délai d'outil (une seule source)
    statique: statique.clone(),
};
main_router = main_router
    .route_service("/mcp", routes::mcp::recevoir.with_state(etat)); // Endpoint::Route, non top level : comme le repli
// … puis, plus bas et inchangé : .fallback_service(statique).layer(csp_html).with_state(state)
```

*Pourquoi `route_service` et non `route(…, post(…))`* (validation P4, F1, vérifié au code d'Axum 0.8.9 par
l'orchestrateur et par la fiche) : `route_service` enregistre un `Endpoint::Route` (`routing/path_router.rs:138`),
appelé par `route.clone().call_owned(req)` (`:413`), qui marque le futur `not_top_level()` (`routing/route.rs:44-46`) ;
le repli statique est lui aussi un `Endpoint::Route` (`routing/mod.rs:366-370`). Un `MethodRouter` est appelé par
`call_with_state` (`path_router.rs:410-411`), son repli reçoit l'`Allow` accumulé (`method_routing.rs:1168-1174`) et
son futur reste « top level », d'où `set_allow_header` et `set_content_length` (`route.rs:163-167`). Avec
`route_service`, **ni** `Allow` **ni** `content-length` ne s'ajoutent du côté de `/mcp` plus que du côté de la route
absente. `Handler::with_state` rend un `HandlerService` (`handler/mod.rs:202`, `handler/service.rs:148`), qui garde les
extracteurs (`State<EtatMcp>`, `Result<ConnectInfo<_>, _>`, `Request`).

*Pourquoi pas un champ d'`AppState`* (Q11 interdit une variable `.env` et un second compte MariaDB, **non** un champ) :
`AppState` ne connaît pas `static_dir`, et un champ obligerait les huit constructions par littéral — la production et
sept de test (`main.rs:355`, `middleware/auth.rs:267`, `password_recovery_e2e.rs:127`, `invoice_frozen_pdf_e2e.rs:102`,
`setup_admin_e2e.rs:81`, `invoice_send_email_e2e.rs:103`, `:1035`, `:1149`) — à fournir un pool que rien ne garantirait
en lecture seule. L'état porté par le service rend `EtatMcp` **obligatoire à la compilation**, sans `Extension`. Et
**pas d'`AppState` dans `EtatMcp`** : son champ public `pool` (`lib.rs:56`) mettrait le pool ordinaire à portée du
transport ; `EtatMcp` ne porte que ce qui sert, et la `garde` (pool ordinaire) n'est **pas interchangeable** avec la
lecture des outils (`LectureOutils`, type propre).

### Esquisse des modules (indicative dans la forme, exigée dans le fond)

```text
crates/kesh-api/src/mcp/
  mod.rs          — EtatMcp (origines_propres, garde, lecture: LectureOutils, trace, registre, statique), constantes (MCP_POOL_CONNEXIONS,
                    MCP_CORPS_MAX, DELAI_OUTIL) ; doc : l'inventaire clos, la frontière 32-8, l'ordre des contrôles
  reponse.rs      — `ReponseMcp` (énumération propre au MCP, sans AppError) et `rendre`, seule sortie (AC-a31)
  transport.rs    — étapes 1 à 10, rend `ReponseMcp` ; aucune donnée d'outil ne s'y construit
  reseau.rs       — client depuis pair + X-Forwarded-For, admission, normalisation d'origine (fonctions pures)
  protocole.rs    — types JSON-RPC (requête, notification, erreurs à `&'static str`)
  repartiteur.rs  — liste close des méthodes, seul appelant de `executer`, construit `ContexteMcp`
  politique.rs    — `Donnees`, `Autorisation`, `DonneesEnvoyables` (champs privés), `autoriser(validation)`,
                    `appliquer(Autorisation, Donnees)`
  registre.rs     — `Registre::{standard, vide}`, méthodes `avec`, `avec_delai`, `delai`, `outil_pour`, `descripteurs_pour`
  outil.rs        — `McpTool`, `ContexteMcp` (`pub(super) fn nouveau`), `ErreurOutil`, `arguments::<T>()`
  prompts.rs      — `McpPrompt` scellé, catalogue vide
  textes.rs       — toutes les chaînes envoyées au client (constantes) ; `depuis_app_error`
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

// mcp/politique.rs — indicatif ; ni Serialize, ni Default, ni Debug
#[derive(Clone)] pub struct Donnees { resume: String, contenu: Vec<BlocContenu>, structure: Option<serde_json::Value> }
impl Donnees { pub fn nouvelles(resume: String, contenu: Vec<BlocContenu>, structure: Option<serde_json::Value>) -> Self { … } }
pub struct Autorisation { _prive: () }                 // seul `autoriser` le fabrique
pub struct DonneesEnvoyables { donnees: Donnees }
pub(crate) fn autoriser(validation: SendValidation) -> Result<Autorisation, Refus> { /* Required → Err, Exempt → Ok */ }
pub(crate) fn appliquer(_: Autorisation, d: Donnees) -> DonneesEnvoyables { /* construit la réponse, rien d'autre */ }
impl DonneesEnvoyables { pub(in crate::mcp) fn en_resultat(self) -> serde_json::Value { /* content + structuredContent */ } }
```

⚠️ Le futur boxé rend le trait compatible `dyn` sans `async-trait` (spike § 5.3.2, note). ⚠️ Un type défini dans
`politique.rs` à champs privés n'est construit ni lu que dans ce module et ses descendants : c'est le compilateur qui
tient l'AC-a23 ; la garde nomme le rouge si un champ devenait public ou si `Default`/`Serialize`/`Debug` apparaissait.
⚠️ `McpPrompt` scellé : le module `scelle` privé à `mcp::prompts` n'exclut que ce qui n'en descend pas (spike § 5.3.3
L-5) ; la garde `prompts_scelles` tient le reste.

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
        .after_release(|_conn, _| Box::pin(async move {
            Ok(false)     // toute connexion est fermée à sa restitution : chaque emprunt part d'une connexion neuve
        }))
        .connect_lazy_with((*depuis.connect_options()).clone())   // exige un runtime Tokio (R9)
}
```

⚠️ La lecture `SELECT @@session.tx_read_only` de `serveur_mcp::etat` (AC-a28) est une **exception écrite** de l'AC-a27,
qui interdit désormais le nom entier ; `SET SESSION TRANSACTION READ ONLY` n'est pas visé. Le type SQL rendu par
`@@session.tx_read_only` (entier) et son décodage se constatent au test `kesh_version_lit_en_lecture_seule`. ⚠️ #669 reste à part : le pool du MCP **copie** les réglages de session de la
production (aucun), pour que l'API et le MCP lisent la même chose (demande 3 à l'orchestrateur).

### Lignes de code lues et citées

`lib.rs:9` (`forbid(unsafe_code)`), `:153-155` et `:162-164` (`lecture()`, `trace()` sur `self.pool`), `:213-214`
(repli statique), `:1165-1172` (`protected` et `require_auth`), `:1174-1184` (routes publiques), `:1204`, `:1225-1227`
(repli, CSP, état) ; `middleware/auth.rs:83-100` (cookie puis `Bearer`), `:134` (préfixe PAT), `:141`, `:172` ;
`middleware/csp.rs:52-60` ; `main.rs:74-78` (pool sans `after_connect`), `:355`, `:385`, `:397-400` (`connect_info`) ;
`lib.rs:56` (`AppState.pool` public) ; `errors.rs:139-1182` (l'énumération), `:153`, `:165`, `:170`, `:1520`,
`:1538-1540` ; `kesh-db/src/errors.rs:575` (`DbError::NotFound`) ; `services/ecritures.rs:241`, `services/carnet.rs:135` ;
`tests/common/capture_rejeu.rs:10`, `:34-64`, `:85` ; `config.rs:179`, `:193` ; `auth/api_key.rs:153-155` ; `main.rs:75` ;
`routes/api_keys.rs:136`, `migrations/20260605000001_api_keys.sql:34`, `document_storage.rs:18` (témoins verts de
l'AC-a27) ; `tests/services_rollout.rs:60`, `:519` ; `axum-0.8.9/src/routing/mod.rs:252-254`, `:408` ;
`sqlx-core-0.8.6/src/pool/options.rs:492`, `pool/connection.rs:289-296` ; `kesh-db/src/pool.rs:28-49` ; `repositories/mcp_server_settings.rs:40-46` (`decode_list`),
`:69-79`, `:83-90` ; `services/poignees.rs:21-33`, `:38-52` ; `services/acces.rs:23-33` (`Portee`), `:84-90` (`mcp`),
`:113-123` (`exiger`) ; `services/mod.rs:207` ; `services/rapports.rs:347` ; `tests/common/lecture_seule.rs:17-32` ;
`tests/services_motif.rs:45`, `:1537`, `:1886`, `:1888-1898`, `:1913`, `:2539-2546`, `:2559-2563` ;
`tests/audit_route_registry.rs:27-32`, `:284-300`, `:326`, `:496-517`, `:529`, `:540-567`, `:676-698`, `:703-720`,
`:713`, `:729-749`, `:768-797`, `:801-814`, `:1121-1129` ; `entities/mcp_server_settings.rs:18-40`, `:44-55`, `:59-70` ;
`repositories/audit_log.rs:94-95` ; `entities/audit_log.rs:38-62` ;
`migrations/20260605000002_audit_log_actor.sql:23` ; `config.rs:180-181`, `:334`, `:1089-1091` ;
`tower-http-0.6.11/src/services/fs/serve_dir/mod.rs:317-327`, `future.rs:172-176`, `:185-190` ;
`axum-0.8.9/src/extract/connect_info.rs:153-158`, `:163-164` ; `sqlx-core-0.8.6/src/pool/mod.rs:545`, `:567`,
`pool/options.rs:267`, `:380`, `:435`, `:579`, `pool/inner.rs:255-262`, `:355-358`, `:469-495` ;
`password_recovery_e2e.rs:690-703` ; `test_endpoints_e2e.rs:97-104` ; `user-manual.tex:190`, `:3152-3153` ;
`docs/api-external.md:183`, `:829`.

### Fichiers touchés (prévus)

- **32-6a** : `crates/kesh-db/src/pool.rs` ; `crates/kesh-db/src/repositories/mcp_server_settings.rs` ;
  `crates/kesh-db/tests/pool_lecture_seule.rs` (neuf) et le fichier de tests du dépôt de réglages (recompté à T-a0) ;
  si le point R5 de la 32-5a n'est pas réglé par la 32-5b, `crates/kesh-db/src/repositories/mcp_keys.rs` (même module
  `kesh-db`) ; `crates/kesh-api/src/mcp/**` (neuf) ; `crates/kesh-api/src/routes/{mcp.rs (neuf), mod.rs}` ;
  `crates/kesh-api/src/lib.rs` ; `crates/kesh-api/Cargo.toml` (`tower = { version = "0.5", features = ["util"] }` en
  dépendance **normale** : `rendre` emploie `ServiceExt::oneshot` en production, que seule l'unification des
  fonctionnalités par Axum rend aujourd'hui disponible, `axum-0.8.9/Cargo.toml:268-271` — validation P5, R3 ; aucun module
  de plus, selon la convention du découpage) ; `crates/kesh-api/src/services/{serveur_mcp.rs (neuf), mod.rs,
  poignees.rs (doc)}` ;
  `crates/kesh-api/tests/{mcp_transport_e2e.rs, mcp_reseau_e2e.rs, mcp_gardes_lexicales.rs, mcp_traduction.rs,
  common/mcp.rs, common/capture_trace.rs (neufs),
  common/mod.rs, common/lecture_seule.rs, services_motif.rs, audit_route_registry.rs}`.
- **32-6b** : `crates/kesh-api/src/mcp/repartiteur.rs` ; `crates/kesh-api/src/services/{serveur_mcp.rs,
  poignees.rs (doc), mod.rs (doc : inventaire des écritures, R14)}` ; `crates/kesh-api/tests/common/mcp.rs` (outil
  **introuvable**, F14 ; module `mcp`) ; `crates/kesh-api/src/audit_labels.rs` ;
  `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ; `crates/kesh-api/tests/{mcp_journal_e2e.rs (neuf),
  audit_route_registry.rs}`.

### Le manuel (axe de revue)

- **Ce que la 32-6 dit au manuel : rien.** Aucune ligne des manuels ne décrit `/mcp` (fait 12). Elle **vérifie**
  qu'aucun texte ne devient faux : `docs/api-external.md:183` (borné à « v0.2 ») et `:829` (« Hors périmètre … serveur
  MCP Kesh-natif », borné lui aussi à la v0.2 par sa parenthèse « non planifié pour v0.2 » — validation P3, R11 ; tous
  deux trompeurs à la release, à la 32-11), `admin-manual.tex:2925`, la brochure
  (`marketing-brochure.tex:480`) — tous à la 32-11 ; les passages de la 32-5 (`admin-manual.tex:2230`, `:2237`,
  `:2249`, `:2324`, `user-manual.tex:3028-3030`) restent vrais. Contrôle : `grep -n "MCP" docs/manual/fr/*.tex docs/api-external.md` et le
  PDF aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}MCP.{80}"`), sorties citées au Dev Agent Record.
- **Ce qu'elle laisse à la 32-7** (section *Le serveur MCP* du manuel d'administration), à reprendre **tel que le code
  le fait** : désactivé, `/mcp` se comporte comme une route absente (C-32-6-2) ; l'activation est par société, et une
  clé d'une société qui ne sert pas est refusée comme une clé inconnue ; le filtre porte sur le pair TCP,
  `X-Forwarded-For` n'est lu que des mandataires déclarés, de droite à gauche ; refusé, le client voit une route
  absente ; filtre activé et liste vide = personne ; une entrée invalide de la liste des réseaux admis est ignorée, mais une entrée
  invalide de la liste des **mandataires** **coupe le serveur MCP de la société** (réponse de route absente, erreur au
  journal du serveur), *filtre activé ou non* (précisé le 2026-10-25, P7-H3) ; l'`Origin` et ses origines
  propres (`KESH_PUBLIC_BASE_URL`, `localhost`/`127.0.0.1`/`[::1]` au port de Kesh), la normalisation du port par
  défaut ; **ce que le filtre ne garantit pas** (Traefik à déclarer, passerelle Docker, pas un pare-feu) ; une clé
  `required` ne peut appeler aucun outil avant la 32-8 ; **seul `Authorization: Bearer <clé MCP>` est lu** — un PAT ou
  le navigateur connecté reçoit le 401 (validation P5, R9) ; corps limité à 1 Mio, délai de 30 s par outil ; ce qui est
  journalisé (32-6b, AC-b1 à AC-b4) et ce qui ne l'est pas ; un appel MCP **écrit** (dernière utilisation, journal) — à
  ne pas présenter comme « une lecture n'écrit rien » (#730) ; **au journal d'audit, un appel MCP paraît au nom du
  créateur de la clé**, sans la marque « clé API » que l'écran pose pour un PAT (`user-manual.tex:3152-3153`) : la clé
  se lit au détail (validation P1, R10 ; point 4 pour Guy) ; `last_used_at` n'est pas « dernier appel servi » (§ *Ce
  que les gardes ne voient pas*) ; un `KESH_PUBLIC_BASE_URL` invalide supprime **en silence** (avertissement au seul
  journal du serveur) l'origine propre qui en viendrait (AC-a10) ; une ligne de réglages illisible fait paraître la
  société **désactivée** (AC-a2) ; toute requête anonyme vers `/mcp` lit la base, sans limite de débit (validation P2,
  R13) ; `Forwarded` et `X-Real-IP` sont **toujours ignorés** — derrière un mandataire qui ne pose que `X-Real-IP`, le
  client vu est le mandataire ; `Content-Type: application/json` est exigé (415) et une `MCP-Protocol-Version` autre
  que `2025-11-25`/`2025-06-18` est refusée (400) — un client mal configuré le verra ; chaque appel d'outil ouvre une
  connexion MariaDB neuve (AC-a25) (validation P3, R12).
- **Deux textes publics qu'une clé `dispensée` rend faux, à la 32-7 ou à la 32-11** (validation P2, R13 ;
  l'orchestrateur l'inscrit aussi au registre de l'epic) : `README.md:233` (« les données **montrées à l'utilisateur
  avant de partir à l'agent** ») et `website/roadmap.html:375` (« the data shown to the user before it is sent to the
  agent ») — sous une clé `dispensée` (Q12), l'AC-a15 fait partir le résultat **sans** le montrer. Ils ne mentent pas
  avant la release (32-6 non publiée seule), mais doivent dire l'exception de la clé dispensée au plus tard à la 32-11.
- **`/mcp` n'a pas de CORS, à dessein** (validation P4, R13 ; à la liste de la 32-7) : les clients visés sont des
  agents locaux (Claude Code, Hermes), non des pages web ; `OPTIONS` rend 405 et Kesh n'émet aucun en-tête
  `Access-Control-*`, si bien qu'une origine inscrite à `allowed_origins` **n'ouvre `/mcp` à aucune page web
  étrangère** — elle ne sert qu'aux clients qui envoient un `Origin` sans être des navigateurs ; l'`Origin` protège
  contre le *DNS rebinding*. Un client web serait un CR.
- **La matrice portée × outil** que le kickoff attribue à « 32-6, 32-9 » (`epic-32-serveur-mcp.md:178` : « matrice
  portée × outil **générée** et comparée au manuel ») revient à la **32-9a**, qui la porte déjà au registre de sprint ;
  la 32-6 n'y contribue que par `registre_coherent` et `registre_standard_inventaire` (validation P2, F16).
- **Reportés par la revue de code P1 de la 32-6a (2026-10-25), à la 32-7 ou à la 32-11** :
  - **B4** — un message de type réponse (`result` ou `error`, sans `method`) reçu par `/mcp` rend **400** `-32600` ; le
    transport Streamable HTTP de la révision `2025-11-25` veut **202** pour une réponse reçue. Kesh, sans session,
    n'émet jamais de requête vers le client : toute réponse est non sollicitée, et le refus reste défendable. À
    confronter à la spike (§ 1.3) et au comportement des clients réels (§ 2.4) par la 32-11 ; à écrire au manuel par
    la 32-7.
  - **E8** — la liste de la 32-7 gagne : `/mcp` répond comme une route absente quand il n'est pas servi (dont au client
    que le filtre refuse) ; le 401 unique (clé absente, refusée, d'une société qui ne sert pas, **ou présentée sous une
    origine que sa société n'admet pas**, C-32-6-52) ; ni le cookie ni un PAT ne sont lus ; `Content-Type` JSON exigé ;
    1 Mio ; `kesh_version` seul outil, qui rend aussi le délai d'instruction ; pas de CORS (`OPTIONS` → 405) ; l'URL est
    exactement `/mcp` (`/mcp/` rend la page de l'application) ; en-têtes de contrôle répétés refusés ; toute
    notification reçoit 202 ; les erreurs JSON-RPC du transport n'ont pas d'`id` (C-32-6-42) ; un `0.0.0.0/0` dans les
    plages admet tout client (#738) ; un `KESH_PUBLIC_BASE_URL` à sous-chemin donne son origine (A2).
  - **A3** — `admin-manual.tex:2935` (et son PDF) dit « 122 des 128 routes à verbe mutant », « 122 + 4 + 2 = 128 » : le
    registre compte désormais **129** routes (`exempt` 5, avec `/mcp`) — et la 32-6b passera `/mcp` à `Traced` (123). À
    corriger par la 32-11 avec la synchronisation des documents de release.
  - **`docs/api-external.md:185` et `:831`** (« pas de serveur MCP Kesh-natif en v0.2 », « non planifié pour v0.2 ») :
    bornés à la v0.2, trompeurs à la 0.16.0 — à la 32-11.

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
   désactivé qui rendrait 404 serait donc **reconnaissable**. La fiche retient l'**indiscernabilité** (C-32-6-2). Si tu
   veux un vrai 404 partout, c'est le repli de **toute** l'application qui change : hors de cette story, une issue à
   ouvrir. **Et un écart de plus au kickoff, à ta connaissance** (validation P4, F12) : le kickoff donnait pour exemple
   « le même 404 si la société de la clé ne l'a pas » ; la fiche rend, pour une clé **valide** dont la société ne sert
   pas, le **401** d'une clé inconnue (C-32-6-3 révisé), parce qu'une réponse de route absente dirait à qui détient une
   clé volée qu'elle est valide.
2. **Docker et Traefik devant le filtre.** Le filtre voit l'adresse du pair TCP. Derrière Traefik, il faut le déclarer
   **mandataire de confiance**, sinon tous les clients paraissent venir de lui. Derrière une publication de port Docker
   par le mandataire en espace utilisateur, tous peuvent paraître venir de la passerelle Docker (`172.17.0.1`), adresse
   privée **admise par défaut** : le filtre n'arrêterait personne. Je n'ai pas pu le vérifier sur le NAS : Kesh y
   est-il derrière Traefik, et quelle adresse voit-il pour un client (le journal du serveur la donnera après la
   32-6a) ? La 32-7 l'écrira au manuel.
   - **2 bis — une garde peu coûteuse, que je propose sans l'imposer** (validation P1, F15 = R13) : un pair **non
     déclaré** qui envoie `X-Forwarded-For`, `Forwarded` ou `X-Real-IP` est soit un mandataire oublié, soit un
     client qui forge — dans les deux cas, répondre comme une route absente (et le dire au journal du serveur). Elle
     arrêterait le cas Traefik non déclaré ; elle refuserait aussi un client local qui enverrait ces en-têtes de
     lui-même (rare). Aujourd'hui la fiche fait l'inverse (test (f) : en-tête ignoré, pair admis). **Ce qu'elle ne
     couvre pas, et ce qu'elle coûte** (validation P2, F16) : le cas `userland-proxy` de Docker n'ajoute aucun
     en-tête — la garde n'y peut rien ; un Traefik non déclaré rendrait `/mcp` **muet** pour tous (route absente), le
     diagnostic n'étant qu'au journal du serveur ; et un « oui » inverse le test (f) et appelle une mutation de plus.
     Oui ou non ?
3. **Le découpage en deux parties** (32-6a garde, 32-6b journal) et l'ordre a → b — confirmé par l'orchestrateur le
   2026-10-25 ; pour information.
4. **L'acteur du journal** : chaque appel est écrit au nom du **créateur de la clé** (`actor_type = 'user'`), la clé
   étant nommée au détail — plutôt qu'un type d'acteur « clé MCP », qui exigerait de modifier la colonne `actor_type`
   (un binaire antérieur ne la lirait plus). Conséquence visible : à l'écran du journal, l'appel ne porte pas la marque
   « clé API » (même défaut que #431 pour les PAT). Te convient-il ?
5. **Ce qui est journalisé** : chaque `tools/call` seulement ; ni `tools/list`, ni les refus avant authentification
   (au journal du serveur seulement) ; et **si le journal ne s'écrit pas, rien ne part** (C-32-6-21, C-32-6-24,
   C-32-6-30).
6. **D'ici la 32-8, une clé `required` (le défaut) ne peut appeler aucun outil** — seules les métadonnées passent. Pour
   éprouver la 32-6, il faudra une clé `dispensée` (le seul outil, `kesh_version`, ne rend aucune donnée des livres).
7. **Hypothèses héritées** Q3, Q5, Q7 — inchangées.
8. **Une inondation du réseau local** (validation P3, R9, C-32-6-38) : j'ai fait passer la règle « servi » et
   l'authentification sur la connexion ordinaire de Kesh, réutilisée, avant tout emprunt du pool des outils — une
   requête anonyme n'ouvre donc aucune connexion neuve. Reste qu'un poste du réseau local qui inonderait `/mcp`
   ralentirait Kesh pour tous, comme il le peut déjà par la page de connexion ; aucune limite de débit n'est posée.
   Faut-il en poser une (par adresse, comme celle du login), ou l'accepter pour un serveur qui n'est pas exposé ?
   **Et une lecture de Q11 à confirmer** (validation P4, F12) : ta réponse dit que « la connexion du MCP » se déclare
   `READ ONLY`. La fiche lit **en lecture seule** tout ce que lisent les **outils** ; mais la règle « servi » et
   l'authentification — deux requêtes fixes de Kesh, sans outil ni donnée de l'agent — lisent sur la connexion
   ordinaire, qui ne l'est pas (C-32-6-38). Ce partage te convient-il ?

## À recontrôler à l'étape 0 du développement


Ligne lue citée au Change Log pour chacun :

- **32-5b** : signature réelle de `services::cles_mcp::authentifier` et de `CleMcpAuthentifiee` ; `Portee` = alias de
  `McpScope`, cinq variantes ; le refus d'un préfixe autre **avant** tout accès à la base ; `ContexteAcces::mcp` appelé
  en `services/cles_mcp.rs` (garde N24) ; le point **R5 de la revue P2 de la 32-5a** (`auth_row` journalise deux fois
  une clé sans portée connue, et une clé **mixte** émet son `warn!` à **chaque** requête MCP — fiche 32-5, § *Pour la
  32-5b*) : **si la 32-5b ne l'a pas réglé**, la 32-6a le règle dans `repositories/mcp_keys.rs` (avertissement
  dédupliqué par clé et version, ou réservé aux chemins d'administration, sans jamais nommer le condensat), avec un
  test qui compte les avertissements sur deux requêtes, et le dit au Change Log ; sinon, le constat « réglé par la
  32-5b » est cité ; `tests/common/` (où vit `pool_lecture_seule`, `cles_mcp_e2e.rs`).
- **32-5d** : `mcp_keys` à `RESET_PRESERVED_TABLES` (aucune incidence attendue sur la 32-6, à constater).
- **32-5c** : `ipnet` dépendance directe de `kesh-api` ; forme stockée des plages, mandataires, origines (C-32-5-19 ;
  la normalisation de l'AC-a10 s'applique aux deux côtés) ; type d'entité `mcp_key` à `audit_labels::ENTITY_TYPES` ;
  **tous** les compteurs littéraux d'`audit_route_registry.rs` que nomme l'AC-a6 (128 / 131 attendus pour les totaux),
  ligne par ligne ; `admin_pat_denied_e2e` inchangé par la 32-6 (`/mcp` hors du bloc).
- **Comptes de `services_motif.rs`** : `FABRIQUES_V2`, `EXCEPTIONS_V1`, nombre de fichiers de `services/` de (vi) (dix
  au relevé ; `cles_mcp.rs` de la 32-5b, `serveur_mcp.rs` de la 32-6a), `RESIDUS_VI`.
- **Axum** : le montage de l'AC-a1 — `route_service` d'un `HandlerService` (`routing/path_router.rs:138`, `:413`,
  `handler/mod.rs:202`), ses extracteurs (`State<EtatMcp>`, `Result<ConnectInfo<_>, ExtensionRejection>`,
  `connect_info.rs:158`, `Request`) — et le test de l'AC-a3 lancé d'abord (validation P5, F3).
- **`list_enabled` et la ligne illisible** (validation P5, R12 (b)) : le dépôt rend `(Vec<McpServerSettings>, Vec<i64>)`
  — les lignes lues et les identifiants de société des lignes écartées —, que `reglages_actifs` journalise ; signature
  à confirmer au code de `decode_list` (`repositories/mcp_server_settings.rs:40-46`).
- **`tools/call` à `params` mal formés** : la règle est désormais à l'AC-a19 (validation P6, H5) ; seule la forme
  `-32602` reste à confronter au schéma `2025-11-25`, avec le reste.
- **`tower-http`** : le comportement du fait 1 (405, `Allow: GET,HEAD`) à la version du `Cargo.lock` d'alors.
- **Spike § 2.4 (h)** : la `protocolVersion` que demandent Claude Code et Hermes — si Guy l'a observée, la liste
  acceptée (AC-a13) s'y ajuste ; sinon, liste inchangée et le Change Log le dit.
- **La révision `2025-11-25`** (si consultable) : la liste des méthodes client (AC-a12), la forme d'une erreur de
  validation d'arguments (`isError` ou `-32602`, AC-a19), et l'**`id` d'une erreur** qu'on ne peut rattacher à une
  requête (`-32700`, `-32600`, 401, 403) : `"id": null` (JSON-RPC 2.0 § 5, retenu) ou absent (validation P4, R8).
- **Ce que la 32-5c stocke pour une adresse IPv4 mappée** (`::ffff:a.b.c.d`) dans `trusted_proxies` et
  `allowed_networks` : refus à l'écriture, ou forme mappée — la normalisation à la lecture de l'AC-a8 et le cas (s) de
  l'AC-a9 en dépendent (validation P4, R9).
- **Le comportement de MariaDB 1146** : une requête sur une table renommée nomme la table **demandée** (raisonné en
  validation P1, R3, non exécuté) — sans incidence sur les tests, qui comparent le corps entier à sa constante.
- **La capture de `tracing`** : `tests/common/capture_rejeu.rs` ne retient que le champ `operation` des `WARN`/`ERROR`
  de cible `kesh_db::retry` (`:34-64`) — la 32-6a écrit une **couche générique** (`tests/common/capture_trace.rs`),
  reprend seulement le montage par fil (`set_default`, `:10`, `:85`), et son test porte un **témoin** (AC-a14 ;
  validation P3, R3).
- **`touch_last_used` de la clé MCP, *best-effort* et sur la `Trace`** (32-5b, AC-b3 ; spike § 6) : l'AC-a26 (ii) et
  l'AC-b3 en dépendent (validation P2, F15) — à constater au code de la 32-5b.
- **Les noms des variantes « hors liste »** de l'AC-a18 (`FiscalYearClosed`, `AccountNotFound`, `RateLimited`,
  `IllegalState`, `DbError::OptimisticLockConflict`) : à relire contre `errors.rs` et `kesh-db/src/errors.rs` du
  moment ; en remplacer un disparu par une autre variante de la même famille.

### Constats de la validation P7 (0/0/0/4), non remédiés à la fiche — à corriger à l'étape 0, ligne lue citée

- **P7-H1** — la plage `method_routing.rs:1168-1171` de Ma20b (i) et de l'entrée « validation P6 » s'arrête au bras
  `None` ; le bras qui pose l'`Allow` d'un `post_service` est la ligne **1173** (`AllowHeader::Bytes(allow_header) =>
  future.allow_header(…)`). Correction attendue : citer `method_routing.rs:1168-1174` (ou `:1173`), comme l'AC-a1, les
  Dev Notes et le registre — dans l'entrée P6, par annotation datée.
- **P7-H2** — le *Contexte* de C-32-6-40 range encore `any(…)` parmi les `MethodRouter` qui ajoutent `Allow`, quand sa
  ligne « Écartées » dit le contraire. Correction attendue : annotation datée de C-32-6-40 — le *Contexte* vaut pour
  `post(…)` et `.fallback(…)` ; `any` et `any_service` n'ajoutent pas `Allow` (`skip_allow_header`,
  `method_routing.rs:514`, `:436`).
- **P7-H3** — la portée de la règle « une entrée invalide de `trusted_proxies` écarte la société de S ». Correction
  attendue, à l'AC-a8, à l'AC-a9 (n') et à la liste de la 32-7 :
  - la règle vaut **quel que soit l'état du filtre** : avec `address_filter_enabled = FALSE`, une entrée invalide de
    `trusted_proxies` écarte **aussi** la société de S (fail closed ; l'administrateur voit l'`error!` au journal du
    serveur), le cas « filtre désactivé » entrant à (n') comme second cas — tous pairs refusés, réponse de route
    absente — et Ma13b tuée par les deux ;
  - dans (n'), une requête de la **clé d'une société ainsi écartée**, depuis un pair que S admettrait par une autre
    société servie, reçoit le **401 fixe** de l'AC-a4 (corps égal à `TEXTE_NON_AUTHENTIFIE`) ; s'il n'y a qu'une
    société, la réponse est celle d'une route absente (S vide, AC-a2) — les deux cas sont écrits au test.
- **P7-H4** — l'entrée « validation P6 » dit « six des huit LOW nés du commit P5 », alors que le rapport P6 classe H6
  **antérieur** (cinq nés du commit : H2, H4, H5, H7, H8 ; trois antérieurs : H3, H6, H9). Correction attendue :
  annotation datée de l'entrée P6 — « cinq des huit LOW nés du commit P5 selon le rapport ; H6 y est classé antérieur,
  sa propagation ayant été manquée par le commit » ; le total 8 et le trend `0/0/1/8` sont justes.

## Dev Agent Record

### Agent Model Used

32-6a : Opus 5.5 (Claude Code), en autonomie, le 2026-10-25, worktree `kesh-32-6`, branche `story/32-6`, partie de
`3dbd3692` (`main` `76869cc9` fusionné : toute la 32-5).

### Statut des parties

- **32-6a** — `review` (développement terminé le 2026-10-25 ; revue P1 à lancer, `326a-review-prompt-p1.md`).
- **32-6b** — `ready-for-dev`, après la 32-6a.

### Completion Notes List

#### 32-6a — l'étape 0 (lignes lues à `76869cc9`, sorties dans `kesh-gate-logs/326a-etape0-greps.txt` et `-2.txt`)

- **32-5b** : `services/cles_mcp.rs:78` `pub async fn authentifier(lecture: &Lecture, trace: &Trace, jeton: &str) ->
  Result<CleMcpAuthentifiee, AppError>` ; `CleMcpAuthentifiee` (`:55`) porte `cle_id`, `acces`, `validation` ; refus du
  préfixe **avant** la base (`:83`, `if !jeton.as_bytes().starts_with(MCP_PREFIX.as_bytes())`) ; `ContexteAcces::mcp(`
  appelé en `:110` ; dernière utilisation *best-effort* sur la `Trace` (`:101`, `touch_last_used(trace.pool(), …)`, échec
  en `warn!` sans le condensat) — l'AC-a26 (ii) et le fail closed du journal (32-6b) en dépendent. `Portee` =
  `kesh_db::entities::McpScope` (`services/acces.rs:31`, `pub use … McpScope as Portee`), cinq variantes
  (`entities/mcp_key.rs:33-44`). **R5 de la 32-5a réglé par la 32-5b** : `repositories/mcp_keys.rs:127` `auth_row` n'émet
  qu'un `warn!` pour une clé sans portée connue et un `debug!` (`:138`) pour une clé mixte ; `read_scopes` (`:86`) garde
  l'avertissement aux chemins d'administration — rien à faire en 32-6a.
- **32-5d** : `mcp_keys` à `RESET_PRESERVED_TABLES` (`kesh-db/src/backup.rs:121-124`) ; aucune incidence.
- **32-5c** : `ipnet = "2.12"` dépendance directe (`kesh-api/Cargo.toml:96`, 2.12.0 au `Cargo.lock`) ; type d'entité
  `"mcp_key"` (`audit_labels.rs:46`) ; forme stockée — adresse nue → `IpNet::from(adresse).to_string()`
  (`routes/mcp_server_settings.rs:111-114`) : **une IPv4 mappée est stockée `::ffff:a.b.c.d/128`**, normalisée à la
  lecture (`mcp::reseau::reseau_canonique`, cas (s)) ; `/0` accepté (#738, constaté, C-32-6-51) ; origines par
  `url.origin().ascii_serialization()` (`:166`), port par défaut retiré — la normalisation de l'AC-a10 est désormais
  **la même fonction** des deux côtés (`util::origine_canonique`, C-32-6-44). Compteurs d'`audit_route_registry.rs` à
  `76869cc9` : `LIB_ROUTES.len()` 128, `traced` 122, `exempt` 4, `no_matter` 2, total 131, `sans_ecriture` 103,
  `tout.len()` 131, doc d'en-tête 128 / 131 — conformes à l'attendu de la fiche. `admin_pat_denied_e2e` : `/mcp` est
  hors du bloc admin ; `lib_rs_has_no_double_slash_inside_literals` reste vert (aucun littéral d'URL dans `lib.rs`).
- **`services_motif.rs`** à `76869cc9` : `EXCEPTIONS_V1` vide (`:1885`), `FABRIQUES_V2` deux lignes (`:1888-1897`),
  `RESIDUS_VI` deux lignes, compte de `services/` **onze** (la 32-5b y avait mis `cles_mcp.rs`, non dix) ; `MOTIF_V1`
  `r"\.\s*pool\s*\(|::\s*pool\b"` (`:1912`).
- **Axum 0.8.9 / tower-http 0.6.11** (`Cargo.lock:236-237`, `:4401-4402`) : montage par `route_service` d'un
  `HandlerService` ; le test de l'AC-a3 **lancé le premier**, sur ce seul montage et par `oneshot`, avant le reste du
  code — vert au premier essai (`326a-T-a0-ac-a3.log`) ; une route absente rend bien `405`, `Allow: GET,HEAD`, corps vide
  pour un `POST` (lu dans `326a-mut-Ma20b-i.log`). `method_routing.rs:1173` (P7-H1), `:436` et `:514` (P7-H2) lus à la
  source.
- **Schéma `2025-11-25`** (consulté : `schema/2025-11-25/schema.ts` du dépôt `modelcontextprotocol`, et la page
  `specification/2025-11-25/server/tools`) : méthodes client de l'union `ClientRequest` — `ping`, `initialize`,
  `completion/complete`, `logging/setLevel`, `prompts/get`, `prompts/list`, `resources/list`, `resources/templates/list`,
  `resources/read`, `resources/subscribe`, `resources/unsubscribe`, `tools/call`, `tools/list`, `tasks/get`,
  `tasks/result`, `tasks/list`, `tasks/cancel` (la liste du test `methodes_servies_liste_close`) ;
  `JSONRPCErrorResponse { id?: RequestId }`, `RequestId = string | number` → erreurs **sans `id`**, jamais `null`
  (C-32-6-42) ; erreurs d'arguments en `isError: true`, outil inconnu et demande mal formée en `-32602` (§ *Error
  Handling*) — conformes aux AC-a18 et AC-a19.
- **Spike § 2.4 (h)** : non observée (aucune trace d'observation dans la spike ni l'epic) ; liste acceptée inchangée.
- **#754** : aucune migration en 32-6a (84 fichiers, inchangé) ; rien de fait. **#738** : constaté, ci-dessus.
- **P7-H1 à P7-H4** : H1 et H4 par annotation datée de l'entrée P6 du Change Log ; H2 par annotation datée de C-32-6-40 ;
  H3 à l'AC-a8, à l'AC-a9 (n') et à la liste de la 32-7, et au code (`decider` lit les mandataires avant l'état du
  filtre), avec le test `n_prime_mandataire_invalide_ecarte_la_societe` (filtre activé **et** désactivé ; 401 fixe avec
  deux sociétés).
- **MariaDB 1146** : sans incidence (les tests comparent le corps entier). **Variantes hors liste de l'AC-a18** : toutes
  existent (`errors.rs` : `Internal`, `FiscalYearClosed { date }`, `AccountNotFound { account_id,
  missing_account_ids }`, `RateLimited { retry_after }`, `IllegalState` ; `DbError::OptimisticLockConflict`).

#### 32-6a — la règle commune des gardes : motifs mesurés sur le dépôt réel à l'étape 0

Sortie : `kesh-gate-logs/326a-etape0-greps.txt`. Sur `crates`, `.rs` et `.sql` : `(?i)\bread\s+write\b` → **0** ligne ;
`tx_read_only|transaction_read_only` → **0** ; `\b(outil_pour|descripteurs_pour|executer|autoriser|appliquer)\s*\(` sur
`kesh-api/src` → **0** ; `\b(autoriser|appliquer)\s*\(` sur `crates` → **0** ; `\bmcp::|crate::mcp|super::mcp` sur
`kesh-api/src` → **0** ; `ContexteAcces::mcp|impl ContexteAcces` → la doc et l'`impl` d'`acces.rs`, l'appel de
`cles_mcp.rs:110` ; `TcpListener|TcpSocket|UnixListener` → `main.rs:390` seul ;
`McpTool|McpPrompt|DonneesEnvoyables|Autorisation|LectureOutils|Registre::` → **0** ; `x-forwarded-for` → un
commentaire de `routes/mcp_server_settings.rs`. Chaque garde de `mcp_gardes_lexicales.rs` porte ses témoins rouges et
verts sur source factice ; les verts **existants** cités : `ContexteAcces::mcp(`, `portee_mcp`, `cles_mcp::`,
`serveur_mcp::` ; `use axum::extract::{ConnectInfo, Request};`, `use kesh_db::entities::McpServerSettings;` ;
`read-write`, `use std::io::{Read, Write};` ; le sceau homonyme `impl Scelle for super::Affichage {}` de
`services/rapports.rs:86-87`, **découvert au premier gate ciblé** (la garde `prompts_scelles` rougissait sur lui) : le
périmètre de « `impl … Scelle for` » est désormais `src/mcp/`, et ces deux lignes sont son témoin vert. Le grep de
l'AC-a14 (`jeton|token|authorization|corps|body` sur `src/mcp` et `routes/mcp.rs`, `326a-etape0-greps-2.txt`, 31
lignes) relu ligne à ligne : toutes ses lignes sont des doc-comments, des noms de variable du transport (`jeton_bearer`, `jeton`) ou
des corps de réponse (`brut(…, corps)`) ; **aucune** n'est un argument d'un `tracing::`.

#### 32-6a — ce qui a été fait

T-a1 (`kesh-db`) : `pool::pool_lecture_seule` (AC-a25) et `mcp_server_settings::list_enabled` (AC-a2) ; T-a2 à T-a4 :
le module `kesh-api/src/mcp/` (`mod`, `transport`, `reseau`, `reponse`, `repartiteur`, `politique`, `registre`,
`outil`, `prompts`, `textes`, `outils/kesh_version`), `routes/mcp.rs`, `services/serveur_mcp.rs`, le câblage de `lib.rs`
(`service_statique`, `build_router_avec_registre`, `route_service("/mcp", …)` avant le marqueur `// NOTE: …`), les
doc-comments que la 32-6a rendait faux (`services/poignees.rs`, `services/mod.rs`, `services/acces.rs`,
`services/cles_mcp.rs`, `lib.rs`) ; T-a5 : les tests et gardes. Écarts à la fiche : au registre, C-32-6-42 à C-32-6-51.
⚠️ **Modules touchés : cinq**, non quatre — `kesh-api/util` s'ajoute (`util::origine_canonique`, C-32-6-44) ; le
plafond de cinq tient.

⚠️ **Écart de méthode, déclaré** : la seconde partie de `tests/mcp_transport_e2e.rs` (les tests après l'AC-a3) a été
ajoutée par un `cat >> … <<'EOF'` du shell, non par l'outil Edit comme la consigne l'exigeait ; le contenu a ensuite été
relu, corrigé par Edit et formaté par `cargo fmt`.

#### 32-6a — les mutations (Ma1 à Ma52, Ma13b, Ma20b, Ma23b), une à une

Banc : motif vérifié **unique** avant chaque remplacement (refus sinon), restauration par `git checkout -- <f>` puis
`touch <f>`, diff résiduel vérifié nul après chacune ; seuls les binaires de test concernés reconstruits (`--test
<binaire>`). Journaux `kesh-gate-logs/326a-mut-<id>.log` (59 journaux). **55 mutations de la fiche : 52 tuées, 3
déclarées non compilables** — Ma23b (`E0451`, `Autorisation { _prive: () }` hors de `politique.rs`), Ma51 et Ma52
(`E0308` : `LectureOutils` et `Lecture` non interchangeables ; Ma52 jouée sur ses deux sites, `reglages_actifs` et
`authentifier`). Aucune ne survit. Notes :
- **Ma1** tuée par les quatre tests de `pool_lecture_seule` **et** par `kesh_version_lit_en_lecture_seule` (journal
  `Ma1-api`) ; **Ma2** par (e) et (e bis) seuls, comme attendu.
- **Ma12** n'est tuée que si **les deux** normalisations du pair sont retirées (transport **et** `reseau::client`) ; la
  variante qui ne retire que celle du transport **survit** (`326a-mut-Ma12-transport-seul.log`) : défense en
  profondeur, déclarée.
- **Ma20b** : (i) `post_service` et (ii) `any_service` tuées toutes deux par `desactive_indiscernable_…`, **au premier
  cas** de la matrice — le `content-length: 0` qu'ajoute le futur « top level » à la 405 d'un `POST`.
- **Ma18** : tuée parce que `session_refusee` éprouve une **clé MCP valide** dans le cookie (ajouté au développement :
  avec un JWT dans le cookie, `authentifier` refusait de toute façon et la mutation survivait).
- **Ma23** rougit `cle_requise_refusee_sans_execution` et `e1_rien_ne_part_sous_cle_requise`, non `cle_dispensee_servie`.
- **Ma26, Ma33, Ma47** : `ErreurOutil` n'ayant aucun champ `String`, chacune s'est jouée en changeant le type ou en
  logeant un `Box::leak` du message ; Ma47 n'est visible qu'au test de traduction (le texte envoyé reste fixe par motif).
- **Ma41** tuée par la garde seule ; `refus_indiscernables` n'est pas atteint par sa forme (aucune `AppError` produite).

#### 32-6a — les gates (réellement exécutés)

- Gates ciblés pendant l'itération : `binary(pool_lecture_seule) | test(list_enabled)` 6/6 ; `binary(mcp_transport_e2e)`
  (AC-a3 seul 1/1, puis 30/31 — un témoin de capture mal formé — puis 31/31) ; clippy `--workspace --all-targets -D
  warnings` (trois passages, deux corrections) ; le lot `mcp_* | audit_route_registry | services_motif |
  services_rollout | cles_mcp_e2e | admin_pat_denied_e2e | pool_lecture_seule | mcp_keys_repository | lib kesh-api` :
  735 exécutés, 734 verts, 1 rouge (`prompts_scelles`, le sceau homonyme ci-dessus), corrigé, puis
  `binary(mcp_gardes_lexicales)` 13/13.
- **Gate complet** au dernier commit de code (`11ec6ca7`), base `kesh_326a` **remise à zéro juste avant**
  (`DROP`/`CREATE`, `sqlx migrate run`, seed), sous `flock`, `scripts/test-fast.sh --ci` : `cargo fmt --check`, clippy
  `-D warnings`, nextest profil `ci` — **4177 exécutés, 4177 verts, 7 ignorés** (`326a-gate-complet-1.log`).
- **Frontend** : non touché ; `npm run build` seul, vert (`326a-frontend-build.log`).
- **E2E complet** (commandes séparées) : binaire `target/debug/kesh-api` reconstruit à 12:26, après le dernier commit
  (12:11), base `kesh_e2e_326a` remise à zéro, port 3331, dossiers neufs `/tmp/kesh-e2e-326a/{inbox,documents,backup}`,
  runner avec `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR` : **315 passés, 7 échecs, 17 ignorés** — les 7 échecs sont
  exactement les KF-029 de `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150`) ; KF-052 ne s'est pas déclenchée (`326a-e2e-1.log`). Contrôle au fil avant la
  suite : `POST /mcp` sur le backend E2E (aucune ligne de réglages) → `405`, comme une route absente. Backend arrêté par
  son PID : le premier PID noté était celui du shell d'enveloppe ; le processus `kesh-api` a été retrouvé par `ps` et
  arrêté par **son** PID, `/health` ne répond plus.
- Recette visuelle : sans objet (aucun contrôle visible).

#### 32-6a — l'axe du manuel

`grep -n "MCP" docs/manual/fr/*.tex docs/api-external.md` → 20 lignes, dont **aucune** ne décrit `/mcp` (`grep -n
"/mcp\b"` → rien) ; les PDF aplatis ne contiennent pas `/mcp`. `docs/api-external.md:185` et `:831` (183 et 829 à
`b723e3d9`) restent bornés à la v0.2 ; `admin-manual.tex:2237`, `:2249`, `:2925`, `user-manual.tex:3028-3030`, `:3204`,
`:3214` et la brochure (`:480`) restent vrais. À ajouter à la liste de la 32-7 : un `0.0.0.0/0` (ou `::/0`) dans les
plages admet tout client (#738) ; un mandataire saisi en IPv4 mappée est reconnu.

#### 32-6a — décomptes (recomptés, périmètre `76869cc9..11ec6ca7`)

- **Tests neufs : 65** (`grep -cE '#\[(sqlx::)?test'` aux deux bornes) — `mcp_transport_e2e.rs` 31, `mcp_reseau_e2e.rs`
  7, `mcp_gardes_lexicales.rs` 13, `mcp_traduction.rs` 2, `kesh-db/tests/pool_lecture_seule.rs` 5, tests unitaires de
  `mcp/reseau.rs` 4, `audit_route_registry.rs` 11 → 12, `kesh-db/tests/mcp_keys_repository.rs` 25 → 27.
  *(Annoté le 2026-10-25, revue de code P1, A5 : le total juste est **64**, non 65 — `pool_lecture_seule.rs` porte
  **4** tests, le « 5 » comptait une ligne de doc-comment `#[sqlx::test]` ; recompté par `grep -cE
  '^\s*#\[(sqlx::|tokio::)?test'` aux deux bornes, 36 → 100.)*
- **Fichiers touchés : 37** (`git diff --name-status 76869cc9..11ec6ca7 -- crates`) : 21 neufs, 16 modifiés ; aucune
  migration ; `Cargo.lock` inchangé.
- Registre des routes : `LIB_ROUTES` 129, `traced` 122, `exempt` 5, `no_matter` 2, total 132, `sans_ecriture` 104,
  `tout` 132. `services_motif.rs` : `EXCEPTIONS_V1` 1, `FABRIQUES_V2` 3, `services/` 12.

### File List

32-6a — neufs : `crates/kesh-api/src/mcp/{mod,outil,politique,prompts,registre,repartiteur,reponse,reseau,textes,transport}.rs`,
`crates/kesh-api/src/mcp/outils/{mod,kesh_version}.rs`, `crates/kesh-api/src/routes/mcp.rs`,
`crates/kesh-api/src/services/serveur_mcp.rs`,
`crates/kesh-api/tests/{mcp_transport_e2e,mcp_reseau_e2e,mcp_gardes_lexicales,mcp_traduction}.rs`,
`crates/kesh-api/tests/common/{mcp,capture_trace}.rs`, `crates/kesh-db/tests/pool_lecture_seule.rs` ; modifiés :
`crates/kesh-api/Cargo.toml`, `crates/kesh-api/src/{lib,util}.rs`,
`crates/kesh-api/src/routes/{mod,mcp_server_settings}.rs`, `crates/kesh-api/src/services/{acces,cles_mcp,mod,poignees}.rs`,
`crates/kesh-api/tests/{audit_route_registry,services_motif}.rs`, `crates/kesh-api/tests/common/{mod,lecture_seule}.rs`,
`crates/kesh-db/src/pool.rs`, `crates/kesh-db/src/repositories/mcp_server_settings.rs`,
`crates/kesh-db/tests/mcp_keys_repository.rs` ; artefacts : cette fiche, `epic-32-choix-autonomes-326.md`,
`326a-review-prompt-p1.md`, `sprint-status.yaml`.

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-25 | création | Fiche rédigée en autonomie (Opus 5.5) contre `b723e3d9` ; registre C-32-6-1 à C-32-6-30 ; découpage en 32-6a / 32-6b (six modules distincts sans découpage) ; prompt de validation P1 `326-validate-prompt-p1.md` (Sonnet, lentilles R et F). |
| 2026-10-25 | validation P1 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `326-validate-prompt-p1.md` ; rapports `kesh-gate-logs/326-validate-p1-{R,F}.md`) : R 0/0/3/14, F 0/0/8/8 ⇒ **trend P1 : 0 CRITICAL / 0 HIGH / 11 MEDIUM / 22 LOW** (doublons R1 = F1, R2 = F8, R4 = F9, R6 = F3, R8 ⊂ F7, R13 ⊂ F15 comptés une fois par lentille). **Prémisse R1 = F1 vérifiée par l'orchestrateur** (`connect_info.rs:153`, aucun `OptionalFromRequestParts`). **MEDIUM remédiés** : R1 = F1 → `Result<ConnectInfo<_>, ExtensionRejection>` (AC-a7, Ma31) ; R2 = F8 → détecteur du registre étendu à `fallback(`, avec témoins sur source factice (AC-a6, C-32-6-32, Ma43) ; R3 → tout test de non-fuite compare le corps **entier** à sa constante (principe en tête des AC ; AC-a2, a14, a19, a20) ; F2 → aucun extracteur de corps, `Request` dernier, remise entière au service statique, test du corps de 3 Mio (AC-a1, a3, a11, Ma32, Ma39) ; F3 = R6 → **sous-routeur à état propre `EtatMcp`**, aucun champ d'`AppState` ni `Extension` (AC-a1, C-32-6-31, Dev Notes *Le canal*) ; F4 → N24, garde `contexte_acces_mcp_construit_par_authentifier_seul` (Ma40) ; F5 → N25, AC-a31, garde `mcp_ne_rend_jamais_une_app_error` (Ma41) ; F6 → `Donnees` et `DonneesEnvoyables` en `politique.rs`, sans `Serialize`/`Default`/`Debug`, garde élargie (AC-a23, C-32-6-15 révisé, Ma42) ; F7 → `Exempt("issue #723 …")` en 32-6a puis `Traced` en 32-6b, compteurs littéraux nommés, `audit_route_registry.rs` aux fichiers de la 32-6b (AC-a6, AC-b6). **Écart à F7** : le rejeu reste `SansEcritureAuJournal` (la colonne parle du journal comptable ; précédent `backup_schedule::update_backup_schedule`, `audit_route_registry.rs:326`). **LOW remédiés** : R4 = F9 (normalisation des deux côtés, Ma44), R5 (lignes d'`acces.rs`), R7 (version absente notée), R8 (dans F7), R9 (runtime Tokio), R10 (manuel : marque « clé API », `api-external.md:829`), R11 (R5 de la 32-5a à l'étape 0, avertissements dédupliqués), R12 (commentaires écartés, périmètre des gardes, mutations Ma31–Ma39), R13 (entrées `X-Forwarded-For` normalisées, non-ASCII, tests (p)–(r)), R14 (« par construction » retiré de C-32-6-2), R15 (traduction complète `AppError` → `ErreurOutil`, `match` exhaustif, AC-a18), R16 (note pour la 32-9 : `Trace` des rapports), R17 (dans F7) ; F10 (`public_base_url` invalide), F11 (clé d'une société qui ne sert pas → 401 fixe, C-32-6-3 révisé ; dernière utilisation en angle mort), F12 (frontières de mot, forme qualifiée, `ContexteMcp::nouveau` en `pub(super)`), F13 (tests `chaque_appel_est_journalise` et `tools_list_filtre` définis ; N26 panic et abandon), F14 (mutations Ma31–Ma39 ; Ma17 localisée ; Mb5 distinguée), F16 (ligne de réglages corrompue écartée). **F15 = R13 (garde derrière Traefik)** : proposée à Guy (point 2 bis), non imposée. **Décomptes après remédiation** (recomptés par `grep` sur la fiche) : **37 AC** (31 / 6), 11 tâches, **50 mutations** (44 / 6), registre C-32-6-1 à **34** ; modules : 32-6a **4** (inchangée), 32-6b **5** (`routes` entre par `audit_route_registry.rs`) — plafond tenu, pas de nouveau découpage ; six distincts, neuf en somme. Prompt de la passe suivante : `326-validate-prompt-p2.md` (Opus, R et F). |
| 2026-10-25 | validation P2 | **Opus ×2** (lentilles R et F, contexte frais, prompt `326-validate-prompt-p2.md` ; rapports `kesh-gate-logs/326-validate-p2-{R,F}.md`) : R 0/0/4/12, F 0/0/5/11 ⇒ **trend P1 0/0/11/22 → P2 0/0/9/23** (doublons R1 = F1, R3 = F4, R5 = F6, R6 ⊂ F5, R9 = F8, R10 = F7, R12 = F11 comptés une fois par lentille). ⚠️ **Recyclage déclaré** : **sept des neuf MEDIUM sont nés de la remédiation P1** (R1 = F1 par R15, R2 par F5, R3 = F4 par R2 = F8, F3 par C-32-6-31, F5 par la non-propagation au registre) ; deux sont d'origine (R4, et F2, que la P1 a rendu plus net). Sévérité égale (MEDIUM → MEDIUM) **par recyclage** : c'est le signal de découpage de `CLAUDE.md` (amendement D5) — **déclaré au Project Lead** ; consigne de l'orchestrateur appliquée en réponse : **la conception la plus simple qui ferme la classe, plutôt qu'une garde de plus**. **MEDIUM remédiés** : R1 = F1 → liste blanche courte et écrite (`Forbidden`, `Database(DbError::NotFound)`, `Validation(_)`), tout le reste en erreur interne fixe, exigence « sans `_` » retirée, test sur les bras et sur des variantes hors liste de plusieurs familles (AC-a18, C-32-6-33 annoté, Ma46, Ma47) ; R2 → **sortie unique** `reponse::rendre(ReponseMcp)`, énumération propre au MCP, `AppError` convertie en deux sites seulement, garde d'inventaire des sites (AC-a31, C-32-6-37, Ma41) ; F2 → `politique::autoriser` **avant** l'exécution, jeton `Autorisation`, `appliquer` ne fait que construire, phrase sur la 32-8 corrigée (AC-a15, frontière, C-32-6-36, Ma23, Ma23b) ; F3 → `AppState` retiré d'`EtatMcp` (`config: Arc<Config>`), garde `mcp_sans_pool` sur tout `src/mcp/`, N27 (sites de la politique et des `impl McpTool`) (AC-a1, AC-a30, C-32-6-31 annoté, Ma48) ; R3 = F4 → **tout** `fallback(` extrait, argument non `routes::` à l'inventaire des non-résolus moins une liste écrite `ServeFile::new(` (AC-a6, C-32-6-32 annoté, Ma45) ; R4 → **`after_release` qui ferme toute connexion à sa restitution**, `before_acquire` retiré, test (e bis) de la forme 2 (AC-a25, C-32-6-35, Ma2) ; F5 = R6 → C-32-6-1, 3 et 8 **annotés** et datés, sans réécriture. **LOW remédiés** : R5 = F6 (renvoi AC-a14), R7 (huit constructions), R8 = F10 (avertissement en `debug!`, sans état ; capture de `tracing` existante, test `jeton_jamais_journalise`), R9 = F8 (nom `tx_read_only` interdit en entier, commentaires de bloc, témoins `:=` et accent grave), R10 = F7 (rognage, témoin discriminant), R11 (motif `ContexteAcces\s*::\s*mcp`, `type … =` refusé), R12 = F11 (délai dans le registre seul), R13 (`README.md:233`, `website/roadmap.html:375`, trois comportements, pour la 32-7 ou la 32-11), R14 (dérogations R1/R2/R7 écrites, `services/mod.rs` à la 32-6b), R15 (pas de `Debug` : note pour la 32-9 ; phrase sur la 32-8), R16 (`Content-Type` des témoins 401, ordre de l'AC-a2, ligne corrompue `[1]`), F9 (Ma31 avec une ligne active, (m') et Ma49), F12 (3 Mio par `oneshot`), F13 (motif de l'AC-b4), F14 (outil introuvable, `tests/common/mcp.rs` à la 32-6b), F15 (`touch_last_used` à l'étape 0 ; oracle temporel du 401 écrit), F16 (point 2 bis complété ; matrice portée × outil attribuée à la 32-9a). **Décomptes** (recomptés par `grep` sur la fiche) : 37 AC (31 / 6), 11 tâches, **55 mutations** (49 / 6), registre C-32-6-1 à **37** ; modules inchangés : 32-6a **4**, 32-6b **5**, six distincts, neuf en somme — pas de nouveau découpage. Prompt de la passe suivante : `326-validate-prompt-p3.md` (Sonnet, R et F). |
| 2026-10-25 | validation P3 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `326-validate-prompt-p3.md` ; rapports `kesh-gate-logs/326-validate-p3-{R,F}.md`) : R 0/0/4/9, F 0/0/3/4 ⇒ **trend P1 0/0/11/22 → P2 0/0/9/23 → P3 0/0/7/13** (doublons R1 = F1, R8 ⊂ F3, R4 ⊂ F4 comptés une fois par lentille). ⚠️ **Recyclage déclaré** : **les sept MEDIUM sont nés d'une remédiation** — R1 = F1 (le motif `read\W+write` de la P2, 53 lignes au dépôt), R2 (les motifs `lister(`/`trouver(` de la P1), R3 (la « capture qui existe déjà » de la P2), R4 (la `rendre` synchrone de la P2), F2 (Ma17 sans mutant depuis la P2), F3 (le `Config` entier laissé par la P2). Motif dominant : **des gardes et des tests écrits dans la fiche sans être confrontés au dépôt réel**. Décision de l'orchestrateur appliquée : une **règle commune des gardes lexicales** en tête des AC (propriété, périmètre et formes ; motif greppé sur le dépôt réel à l'étape 0, vide hors liste blanche ; un témoin rouge et un témoin vert existant ; sorties citées) ; les motifs ne restent qu'à titre d'exemples **mesurés**. **MEDIUM remédiés** : R1 = F1 → AC-a27 réécrit en propriété et formes, témoins verts existants (`read-write`, `use std::io::{Read, Write}`), exemple mesuré `\bread\s+write\b` (0 ligne) ; R2 → points d'accès du registre renommés `outil_pour` / `descripteurs_pour` (0 occurrence au dépôt), AC-a30 réécrit ; R3 → couche de capture **générique** neuve (`tests/common/capture_trace.rs`), témoin `PIEGE_TRACE_326` exigé ; R4 = F4 → `async fn rendre(&EtatMcp, ReponseMcp)`, `RouteAbsente(Request)`, `DonneesEnvoyables::en_resultat` (C-32-6-39) ; F2 → Ma17 visé sur le refus du préfixe d'`authentifier` (`services/cles_mcp.rs`), test inverse au même test ; F3 = R8 → `EtatMcp.origines_propres` au lieu de `Config`, `mcp_sans_pool` étendue à `kesh_db::pool`, `create_pool`, `database_url`, `Config`. **R9** (une connexion par requête anonyme) → la règle « servi » et l'authentification lisent sur `etat.garde`, la `Lecture` ordinaire réutilisée, avant tout emprunt du pool des outils (C-32-6-38) ; l'inondation du réseau local reste un angle mort, soumis à Guy (point 8) ; le test fail closed du journal (AC-b3) passe en conséquence par une table `audit_log` renommée. **LOW remédiés** : R5 (Ma23 : les tests qu'elle rougit), R6 (`Autorisation` à la garde de dérivations, ni `Clone` ni `Copy` ; Ma50 « `executer` avant `autoriser` »), R7 (C-32-6-6 annoté), R10 (`traduction_des_erreurs_de_service` dans `tests/mcp_traduction.rs`), R11 (`api-external.md:829` borné à la v0.2), R12 (trois comportements de plus pour la 32-7), R13 (`services_rollout.rs:60`, `:519`), F5 (test (f) : `KILL` pendant un emprunt), F6 (cas (p) et (p') qui distinguent première et dernière ligne), F7 (`extract_counted` sur `lib.rs` lignes de commentaire retirées). **Décomptes** (recomptés par `grep` sur la fiche) : 37 AC (31 / 6), 11 tâches, **56 mutations** (50 / 6), registre C-32-6-1 à **39** ; modules inchangés : 32-6a **4**, 32-6b **5**, six distincts, neuf en somme. **3 passes sur 8.** Prompt de la passe suivante : `326-validate-prompt-p4.md` (Opus, R et F, chaque motif encore écrit greppé sur le dépôt réel). |
| 2026-10-25 | validation P4 | **Opus ×2** (lentilles R et F, contexte frais, prompt `326-validate-prompt-p4.md` ; rapports `kesh-gate-logs/326-validate-p4-{R,F}.md`) : F 0/1/4/7, R 0/0/1/12 ⇒ **trend P1 0/0/11/22 → P2 0/0/9/23 → P3 0/0/7/13 → P4 0/1/5/19** (doublons F2 = R1 = F3 en partie, F3 ⊃ R2, F7 = R4, F8 = R3, F10 = R12 comptés une fois par lentille). ⚠️ **Recyclage déclaré** : quatre des six findings au-dessus de LOW sont nés de la remédiation P3 (F2 = R1, F3, F4) ; le **HIGH F1** (le montage par `MethodRouter`, discernable) et F5 (Ma27 à vide) étaient **d'origine**, présents depuis la création ou la P1, et c'est l'axe neuf de la P4 — chaque motif et chaque affirmation sur une dépendance confrontés à la source — qui les a trouvés. **F1 vérifié par l'orchestrateur au code d'Axum 0.8.9** (`route.rs:163-167`, `method_routing.rs:1168-1174`). **Remédiés** : F1 → `/mcp` monté par **`route_service`** (`Endpoint::Route` appelé `not_top_level`, comme le repli : `path_router.rs:138`, `:413`, `route.rs:44-46`, `mod.rs:366-370`), méthode tranchée par le transport, `autre_methode` et l'extension du détecteur à `fallback(` **retirées**, le détecteur lisant `route_service(` ; l'AC-a3 compare `Allow` et `content-length` ; Ma20b (C-32-6-40, C-32-6-20 et 32 annotés) ; F2 = R1 = F3 → **type propre `LectureOutils`**, jamais déballé hors de `mcp/outil.rs` ; Ma51 et Ma52 **déclarées non compilables** ; N28 ; ordre des contrôles (étape 2) et esquisse d'`EtatMcp` corrigés (C-32-6-41) ; F4 → le témoin de `jeton_jamais_journalise` est le `debug!` émis par le serveur pendant la requête éprouvée ; F5 → notification complète sous clé `exempt`, témoin à `id` dont le compteur vaut 1. **LOW remédiés** : F6 (témoins verts existants `ContexteAcces::mcp`, `portee_mcp` ; témoin factice dit tel), F7 = R4 ((f) hors liste blanche), F8 = R3 (`mcp_traduction.rs`, `capture_trace.rs` aux listes), F9 (garde N24 simplifiée), F10 = R12 (périmètres et formes d'`un_seul_ecouteur`, définitions de la politique hors du compte), F11 (préconditions : clé `exempt`, société B servie), F12 (deux points à Guy : la lecture de Q11 par C-32-6-38, le 401 au lieu du « même 404 »), R2, R5 (`:713`, `:729-749`, `:1174-1184`), R6 (doc-comments nommés à T-a4), R7 (`origines_propres` en `reseau.rs`, garde `lib_rs_has_no_double_slash_inside_literals`), R8 (`"id": null`, à recontrôler), R9 (entrées stockées normalisées, cas (s)), R10 ((k) et (k')), R11 (`:2249`, `:2324`), R13 (**pas de CORS, à dessein** ; à la liste de la 32-7). **Décomptes** (recomptés par `grep` sur la fiche) : 37 AC (31 / 6), 11 tâches, **59 mutations** (53 / 6 : Ma1 à Ma52 et Ma20b ; Ma23b déclarée hors tableau), registre C-32-6-1 à **41** ; modules inchangés : 32-6a **4**, 32-6b **5**. **4 passes sur 8.** Prompt de la passe suivante : `326-validate-prompt-p5.md` (Sonnet, R et F). |
| 2026-10-25 | validation P5 | **Sonnet ×2** (lentilles R et F, contexte frais, prompt `326-validate-prompt-p5.md` ; rapports `kesh-gate-logs/326-validate-p5-{R,F}.md`) : R 0/0/1/12, F 0/0/1/6 ⇒ **trend P1 0/0/11/22 → P2 0/0/9/23 → P3 0/0/7/13 → P4 0/1/5/19 → P5 0/0/2/18** (doublons F1 = R2, F6 = R5 comptés une fois par lentille). ⚠️ **Recyclage déclaré** : les deux MEDIUM sont **nés de la remédiation P4** — R1 (l'AC-b6 au verbe `"post"` et l'AC-a31 aux « deux gestionnaires », que le changement de montage n'avait pas suivis) et F1 = R2 (Ma20b écrite sous une forme qui ne compilait pas, et le dernier maillon de l'argument « `content-length` discrimine », faux sur le fil). Les deux lentilles ont **confirmé à la source** la remédiation P4 elle-même (`route_service`, `LectureOutils`). **MEDIUM remédiés** : R1 → AC-b6 en `("route_service", "mcp::recevoir", …)`, AC-a31 « le gestionnaire `recevoir` » ; grep de `autre_methode`, « deux gestionnaires », `"post", "mcp::recevoir"`, « sous-routeur » sur la fiche et le registre — ne restent que les explications du retrait et les choix annotés ; F1 = R2 → Ma20b sous ses deux formes compilables, **(i)** `post_service(recevoir.with_state(etat))` (tuée par `Allow: POST` sur un `GET`, par `oneshot` comme sur le fil) et **(ii)** `any_service(…)` (tuée par `content-length: 0` sur une 405, **seulement** par `oneshot` : hyper pose lui-même ce `content-length` sur le fil, `hyper-1.10.1/src/proto/h1/role.rs:924-931`) ; **toute la matrice** méthodes × états de `desactive_indiscernable_d_une_route_absente` passe par `oneshot`. **LOW remédiés** (une ligne de fiche chacun) : F2 (entrée invalide de `trusted_proxies` : société écartée de S), F3 (ligne Axum de l'étape 0), F4 (témoin multi-ligne d'`appels_non_handlers`), F5 (`params` mal formés → `-32602`, issue `arguments_invalides`, à l'étape 0), F6 = R5 (six fonctions `lister` ; motif des gates frontend), F7 (413 : société dans S), R3 (`tower` avec `util` en dépendance normale, `Cargo.toml` aux fichiers), R4 (`<port>` = `config.port` du test), R6 (port par défaut **du schéma**, cas `http://…:443`), R7 (témoins verts factices pour `executer`, `autoriser`, `appliquer`), R8 (toute autre erreur de lecture du corps → 400), R9 (seul `Bearer <clé MCP>` lu, à la liste de la 32-7), R10 (L-7 et l'analyseur de `SHOW GRANTS` sans objet sous Q11), R11 (préfixes IPv6 mappés convertis), R12 ((a) `route_service` avant le marqueur `// NOTE: …` ; (b) signature de `list_enabled` à l'étape 0). **Décomptes** (recomptés par `grep`) : 37 AC, 11 tâches, 59 mutations (53 / 6 ; Ma20b en deux variantes sur une ligne), registre C-32-6-1 à 41 (C-32-6-34 annoté) ; modules inchangés (4 et 5). **5 passes sur 8.** La remédiation ne touche que la fiche et le registre : la passe suivante est **ciblée** (regression hunter, Opus, `326-validate-prompt-p6.md`, commit à part). |
| 2026-10-25 | validation P6 | **Passe ciblée** (regression hunter, **Opus**, une lentille, braquée sur `ac52657a` ; prompt `326-validate-prompt-p6.md` ; rapport `kesh-gate-logs/326-validate-p6.md`) : **0/0/1/8** ⇒ trend P1 0/0/11/22 → P2 0/0/9/23 → P3 0/0/7/13 → P4 0/1/5/19 → P5 0/0/2/18 → **P6 0/0/1/8**. ⚠️ **Recyclage déclaré** : le MEDIUM H1 et six des huit LOW (H2, H4, H5, H7, H8, et H6 par propagation manquée) sont nés du commit P5 ; H3 et H9 lui sont antérieurs. **H1 (MEDIUM) remédié, règle gardée** (« une entrée invalide de `trusted_proxies` écarte la société de S ») et propagée : C-32-6-6 annoté (titre et « Écartées ») ; Ma13 bornée à `allowed_networks` ; Ma13b et le test (n') à l'AC-a9 ; la liste de la 32-7 nuancée ; l'exclusion journalisée en `error!`, comme la ligne illisible de l'AC-a2. **Tri du grep « ignor »** sur la fiche et le registre : relèvent de la règle et ont été corrigés — AC-a8, Ma13, la liste de la 32-7, C-32-6-6 ; sans rapport, laissés tels quels — `X-Forwarded-For` d'un pair non déclaré (AC-a7, AC-a9 (e), (f), point 2 bis), `Forwarded` et `X-Real-IP` (AC-a7, liste de la 32-7, C-32-6-5), casse du `Content-Type` (AC-a11), cookie de session (AC-a14, C-32-6-29), `outil_pour` qui « ignore la portée » (Ma6), « `AppState` ignore `static_dir` » (C-32-6-31). **LOW remédiés** : H2 (les témoins de `desactive_indiscernable_d_une_route_absente` aussi par `oneshot` ; « 200 » retiré), H3 (`any`/`any_service` sans `Allow`, `method_routing.rs:514`, `:436`), H4 (`method_routing.rs:1168-1171` et `route.rs:164` ; hyper n'ajoute `content-length: 0` que si statut et méthode le permettent, `role.rs:512-521` ; la `Config` d'un test d'intégration vient d'une fonction locale `test_config()` — `tests/accounts_e2e.rs:51` — qui appelle `Config::from_fields_for_test`, `config.rs:443`, `tests/common/` n'en fournissant aucune), H5 (AC court à l'AC-a19 : demande mal formée au **répartiteur**, `-32602`, journalisée — l'AC-b4 n'exclut que les refus du transport ; deuxième source de `arguments_invalides` dans `chaque_appel_est_journalise`), H6 (C-32-6-37 et 39 annotés au singulier), H7 (*annotation de l'entrée P5, datée du 2026-10-25* : « R 0/0/1/12 » compte R12 (a) et (b) à part — onze identifiants LOW au rapport R ; le trend P5 0/0/2/18 suit cette convention), H8 (convention du découpage : le `Cargo.toml` de `kesh-api` compte avec `routes`), H9 (ligne `32-6a` de `sprint-status.yaml` : `after_release`, non `before_acquire`). **Décomptes** (recomptés par `grep`) : 37 AC, 11 tâches, **60 mutations** (54 / 6 : Ma1 à Ma52, Ma13b, Ma20b), registre C-32-6-1 à 41 (C-32-6-6, 37, 39 annotés) ; modules inchangés (4 et 5). **6 passes sur 8.** Remédiation sur la fiche, le registre et `sprint-status.yaml` seulement : passe suivante **ciblée** (regression hunter, Sonnet, `326-validate-prompt-p7.md`, commit à part). *(Annoté le 2026-10-25, étape 0 de la 32-6a — P7-H1 : « `method_routing.rs:1168-1171` » se lit `:1168-1174` ; le bras qui pose l'`Allow` d'un `post_service` est la ligne 1173, `AllowHeader::Bytes(allow_header) => future.allow_header(allow_header.clone().freeze())`, lue à la source d'Axum 0.8.9 — même correction à la ligne Ma20b (i) du tableau des mutations, qui cite `:1168-1171`. P7-H4 : « six des huit LOW (H2, H4, H5, H7, H8, et H6 par propagation manquée) sont nés du commit P5 » se lit « cinq des huit LOW nés du commit P5 selon le rapport (H2, H4, H5, H7, H8) ; H6 y est classé antérieur, sa propagation ayant été manquée par le commit » ; le total 8 et le trend 0/0/1/8 sont justes.)* |
| 2026-10-25 | **validation P7 — VALIDATION TERMINÉE** | **Passe ciblée** (regression hunter, **Sonnet**, une lentille, braquée sur `3b884bd7` ; prompt `326-validate-prompt-p7.md` ; rapport `kesh-gate-logs/326-validate-p7.md`) : **0/0/0/4**. **Trend complet** : P1 Sonnet ×2 (R, F) 0/0/11/22 → P2 Opus ×2 0/0/9/23 → P3 Sonnet ×2 0/0/7/13 → P4 Opus ×2 0/1/5/19 → P5 Sonnet ×2 0/0/2/18 → P6 Opus ciblée 0/0/1/8 → **P7 Sonnet ciblée 0/0/0/4**. ⚠️ **Recyclage déclaré sur cinq passes** (P2 à P6) : à chacune, la majorité des findings au-dessus de LOW naissait de la remédiation précédente — gardes écrites sans être greppées sur le dépôt, affirmations sur les dépendances non lues à la source. Il a été fermé, non par des gardes de plus, mais par **quatre simplifications de conception** : la **liste blanche courte des erreurs de service** (`depuis_app_error`, P2) ; **`after_release`**, qui ferme chaque connexion du pool des outils à sa restitution (P2) ; le type **`LectureOutils`**, qui rend la `garde` et la lecture des outils non interchangeables (P4) ; le montage par **`route_service`**, qui fait passer `/mcp` par le chemin du repli statique (P4) — et par la **règle commune des gardes lexicales** (P3) et les deux axes obligatoires des passes P4 à P7. **Clôture** : la boucle se clôt sur une passe à **0 MEDIUM** (`CLAUDE.md` § *La passe ciblée*, décision Q3 (c) du 2026-10-23), sept passes sur huit, la dernière remédiation (P6) n'ayant touché que la fiche, le registre et `sprint-status.yaml`. Les quatre LOW de la P7 ne sont **pas** remédiés à la fiche : ils sont au § *À recontrôler à l'étape 0*, chacun avec sa correction attendue (P7-H1 à P7-H4). Développement de la 32-6a après la fusion des 32-5d et 32-5c (la 32-5b l'est désormais, à constater à T-a0), puis la 32-6b. |
| 2026-10-25 | **Développement 32-6a** | **Opus 5.5**, en autonomie, contre `76869cc9` (toute la 32-5 fusionnée). **Étape 0** : lignes lues citées au Dev Agent Record — `authentifier` (`cles_mcp.rs:78`), `Portee` = `McpScope` (`acces.rs:31`), R5 de la 32-5a réglé par la 32-5b, IPv4 mappée stockée en `/128` (#738), compteurs du registre des routes 128 / 131 et de `services_motif.rs` (onze fichiers de `services/`) ; schéma `2025-11-25` consulté (méthodes de `ClientRequest`, `JSONRPCErrorResponse.id?` → erreurs **sans `id`**, C-32-6-42) ; spike § 2.4 (h) non observée ; #754 sans objet (aucune migration) ; P7-H1 à P7-H4 corrigés par annotations datées (entrée P6, C-32-6-40) et à l'AC-a8, à l'AC-a9 (n') et à la liste de la 32-7. **T-a0** : le test `desactive_indiscernable_d_une_route_absente` lancé d'abord, par `oneshot`, sur le seul montage `route_service` : vert. **Livré** : `kesh_db::pool::pool_lecture_seule`, `mcp_server_settings::list_enabled`, le module `kesh-api/src/mcp/` (transport, réseau, réponse unique, répartiteur à liste close, politique en deux temps, registre, `McpTool`, `McpPrompt` scellé, textes fixes, `kesh_version`), `routes/mcp.rs`, `services/serveur_mcp.rs`, le câblage de `lib.rs`, `util::origine_canonique` (DRY avec la 32-5c). **Choix** : C-32-6-42 à C-32-6-51. **Tests neufs** (`76869cc9..11ec6ca7`) : **65** *(annoté le 2026-10-25, revue P1, A5 : **64**, recompté)*. **Mutations** : 55 de la fiche, **52 tuées, 3 non compilables déclarées** (Ma23b E0451, Ma51 et Ma52 E0308) ; une variante hors fiche, Ma12 sur le seul transport, survit (défense en profondeur, déclarée). **Gates** : gate complet au dernier commit de code `11ec6ca7`, base remise à zéro juste avant, `test-fast.sh --ci` — **4177/4177**, 7 ignorés ; frontend non touché, `npm run build` vert ; **E2E complet 315 passés, 7 échecs = les 7 KF-029**, 17 ignorés. **Écart de méthode déclaré** : une partie de `mcp_transport_e2e.rs` ajoutée par `cat >>` du shell au lieu de l'outil Edit. Clé `32-6a-endpoint-mcp-garde` → `review`. Prompt de la revue P1 : `326a-review-prompt-p1.md` (Sonnet, lentilles B, E et A ; rapports `kesh-gate-logs/326a-review-p1-{B,E,A}.md`). |
| 2026-10-25 | **Revue de code P1 (32-6a)** | **Sonnet ×3** (lentilles B, E, A, contexte frais, prompt `326a-review-prompt-p1.md` ; rapports `kesh-gate-logs/326a-review-p1-{B,E,A}.md`) : B 0/0/0/4, E 0/0/2/6, A 0/0/0/6 ⇒ **trend P1 : 0 CRITICAL / 0 HIGH / 2 MEDIUM / 16 LOW** (doublons E1 = B2, B1 = E3 = A4, B3 = E7 comptés une fois par lentille). Tous **nés du développement** de la 32-6a (A3 : pour l'écart d'une unité seulement). L'écart C-32-6-42 (erreurs sans `id`) est **accepté par l'orchestrateur**, noté au registre. **MEDIUM remédiés** : E1 = B2 → `pool_lecture_seule(depuis, max, delai_instruction)` pose `SET SESSION max_statement_time` (délai du registre) dans `after_connect` ; **vérifié par un test réel**, le test (g) de `kesh-db/tests/pool_lecture_seule.rs` — une `SLEEP(5)` attendue échoue en 1969 avant 3 s, une `SLEEP(5)` abandonnée par son futur n'est plus exécutée 1,5 s après, l'emprunt suivant part d'une connexion neuve ; sans la borne, la partie (2) rougit seule (`326a-mut-P1-Mp1-g2-seul.log` : MariaDB poursuivait l'instruction abandonnée) ; `kesh_version` rend `delaiInstructionS` (C-32-6-54, AC-a28 amendé) ; un outil de test qui lancerait `SLEEP` par l'API est impossible sans ouvrir le pool hors des services : l'angle mort qui reste et le critère de la 32-9 sont écrits au § *Ce que les gardes ne voient pas* ; E2 → test `enveloppe_avant_la_cle` (sans clé, clé inconnue, clé valide : même 415, même 400). **LOW de sécurité traités** : B1 = E3 = A4 → l'étape 9 rend le 401 fixe, égal à celui d'une clé inconnue ; le 403 reste à l'étape 4, avant la clé (C-32-6-52) ; A1 → `lecture_seule_jamais_defaite` retire les lignes de commentaire **avant** les blocs (un `/*` de glob n'avale plus le code), garde lisibles les commentaires exécutables `/*!` et `/*M!` et les refuse en eux-mêmes dans le Rust (les `.sql` en sont exemptés : le squash de test en porte des dizaines, qui ne passent pas sur la connexion du MCP) ; la limite du `/*` dans une chaîne est écrite ; E4 → `transport::unique` : `Origin`, `Authorization`, `Content-Type`, `MCP-Protocol-Version` répétés refusés, test `en_tetes_repetes_refuses`, `Bearer  x` admis, décision pour les autres en-têtes écrite (C-32-6-53). **Autres LOW traités** : B3 = E7 (angle mort écrit, sans dédoublonnage) ; E5 (`chemins_voisins` : un attendu par chemin) ; E6 (bras `kesh_api::mcp` au motif) ; A2 (sous-chemin de `KESH_PUBLIC_BASE_URL` retiré, C-32-6-55) ; A5 (le total des tests neufs du développement est 64, non 65 — annotation datée) ; A6 (imports groupés de `kesh_db`, `socket2`, `from_raw_fd` aux motifs ; angle mort écrit ; doublons de `[dev-dependencies]` retirés). **Reportés à la 32-7 ou à la 32-11, écrits au § *Le manuel*** : B4 (message de type réponse : 400 au lieu de 202), E8 (liste de la 32-7 complétée), A3 (`admin-manual.tex:2935`, 128 → 129), `docs/api-external.md:185` et `:831`. **Tests neufs** (`11ec6ca7..63d3f659`) : **4** — `pool_lecture_seule.rs` 4 → 5, `mcp_transport_e2e.rs` 31 → 34 ; total depuis `76869cc9` : **68** (`grep -cE '^\s*#\[(sqlx::|tokio::)?test'`, 36 → 104). **Mutations** de la remédiation : **13 journaux** (`326a-mut-P1-*.log`), **toutes tuées** par l'assertion visée — Mp1 (borne retirée ; au dépôt et à l'API), Mp1-g2-seul (variante qui isole la partie (2) du test), Mp2 (délai constant au câblage), Mp3 (`Content-Type` déplacé après la clé), Mp4 (403 rétabli à l'étape 9), Mp5 à Mp8 (chaque en-tête répété lu en silence ; chaque rouge nomme son en-tête), Mp9 (commentaire exécutable effacé), Mp10 (bras `kesh_api::mcp` retiré), Mp11 (sous-chemin conservé). **Gates** : ciblé (lot `mcp_* | audit_route_registry | services_motif | services_rollout | cles_mcp_e2e | admin_pat_denied_e2e | pool_lecture_seule | mcp_keys_repository | mcp_admin_routes_e2e | lib kesh-api`) **739/739**, clippy `-D warnings` vert ; **gate complet** au dernier commit de code `63d3f659`, base remise à zéro juste avant, `test-fast.sh --ci` : **4181/4181**, 7 ignorés (`326a-p1-gate-complet.log`) ; frontend non touché, `npm run build` vert ; **E2E complet** (binaire reconstruit à 13:17 après le commit de 13:08, `/proc/<pid>/exe` vérifié, base `kesh_e2e_326a` remise à zéro, dossier neuf `/tmp/kesh-e2e-326a-p1/`) : **314 passés, 8 échecs, 17 ignorés** — les 7 KF-029 et **KF-052** (`sidebar-navigation.spec.ts:75`), **rejouée seule au vert** (`326a-p1-e2e-kf052.log` : 4 passés) ; backend arrêté par son PID. Le correctif touche la production : passe suivante **ciblée** (regression hunter, **Opus**, `326a-review-prompt-p2.md`, commit à part ; rapport `kesh-gate-logs/326a-review-p2.md`). |
| 2026-10-25 | **Revue de code P2 (32-6a)** | **Passe ciblée** (regression hunter, **Opus**, une lentille, braquée sur `63d3f659` ; prompt `326a-review-prompt-p2.md` ; rapport `kesh-gate-logs/326a-review-p2.md`) : **0/0/2/7**. **Trend** : P1 Sonnet ×3 (B, E, A) 0/0/2/16 → **P2 Opus ciblée 0/0/2/7**. Les deux MEDIUM sont **nés de la remédiation P1** (H1 : une garantie affirmée que la borne ne donnait pas ; H2 : le « 403 » de l'étape 9 non propagé à la fiche). **Remédiation** (commit `dd4b8398`) : **H1** — fermé **par construction** (C-32-6-56) : l'outil s'exécute dans un `tokio::spawn` qui possède l'outil (`Registre::outil_pour` rend un `Arc<dyn McpTool>`), une copie du `ContexteAcces` et de `LectureOutils` ; le handler n'attend que la tâche, sous le délai ; un panic de l'outil rend `TEXTE_ERREUR_INTERNE`. Option `Drop` + `KILL QUERY` écartée, raison au registre. Prémisse de H1 relue à la source (`hyper-1.10.1/src/proto/h1/conn.rs:495-504`, `axum-0.8.9/src/serve/mod.rs:385-396`). Tests neufs `abandon_au_delai_le_permis_reste_pris` et `abandon_par_deconnexion_le_permis_reste_pris` (trois lectures du plan tenues par `LOCK TABLES accounts WRITE`, abandonnées ; témoin : le serveur les exécute encore ; une sonde `kesh_version` ne trouve aucun permis 300 ms ; servie après `UNLOCK`) ; verts trois fois de suite au binaire ciblé (`326a-p2-cible-2.log`). Textes faux corrigés : doc de `pool_lecture_seule` (`pool.rs`) et § *Ce que les gardes ne voient pas* (phrase barrée, correction datée, ce qui reste depuis C-32-6-56) ; valeur « quelques millisecondes » greppée sur tout le dépôt : aucun autre site ne parle de la borne ; N26 et l'angle mort « panic ou abandon » annotés ; doc du module `mcp` et de `MCP_POOL_CONNEXIONS`. **H2** — AC-a10 (deux sites) et ordre des contrôles des Dev Notes annotés (étape 9 → 401 fixe, C-32-6-52) ; « 403 » greppé sur la fiche, `docs/`, manuels `.tex`, `website/`, le code `mcp/` et le fichier d'epic : les autres sites parlent de l'étape 4 ou de listes de refus, toujours justes ; aucun manuel ne parle de `/mcp`. **LOW traités** : H3 (C-32-6-57, `max_statement_time` dans `lecture_seule_jamais_defaite`, deux exceptions de plus, un message de test reformulé) ; H4 (angles morts de `MOTIF_SANS_POOL` écrits) ; H5 (faux positif `/*!` de doc Rust écrit) ; H6 (preuve reformulée par un `grep` brut, rejoué : cinq exceptions, rien d'autre hors commentaires) ; H7 (doc de module de `kesh_version`) ; H9 (emprunt avant la fenêtre de 100 ms, connexion fermée explicitement). **H8** : le périmètre de « 36 → 104 » de l'entrée P1 est celui des fichiers de `crates/` touchés par `76869cc9..63d3f659` **dont le décompte change** — `services_motif.rs` (24), `util.rs` (17) et `routes/mcp_server_settings.rs` (7) écartés ; sur tous les fichiers touchés, 84 → 152 ; delta 68 dans les deux cas (recompté ici, fichier par fichier ; l'entrée P1 n'est pas réécrite). **Tests neufs** (`4d99cdcd..dd4b8398`, `grep -cE '^\s*#\[(sqlx::\|tokio::)?test'`) : **2** — `mcp_transport_e2e.rs` 34 → 36 ; `mcp_gardes_lexicales.rs` 13 → 13 et `pool_lecture_seule.rs` 5 → 5 (témoins ajoutés à des tests existants). **Mutations** : **3, toutes tuées** par l'assertion visée — P2-Mq1 (attente directe du futur de l'outil, sans tâche : les deux tests rougissent à « le permis n'est pas rendu avant la fin de l'instruction », témoin passé ; `326a-mut-P2-Mq1-sans-tache.log`), P2-Mq2 (motif sans `max_statement_time` : « rouge (SQL) : SET SESSION max_statement_time = 0 » ; `-Mq2-motif-sans-borne.log`), P2-Mq3 (une pose `= 0` ajoutée à `pool.rs` : trouvée par le balayage ; `-Mq3-borne-defaite.log`). **Gates au dernier commit de code `dd4b8398`** : bases `kesh_326a` et `kesh_e2e_326a` supprimées puis recréées (84 migrations du worktree, seed) ; `test-fast.sh --ci` sous verrou — fmt, clippy `-D warnings`, nextest **4183/4183**, 7 ignorés (`326a-p2-gate-complet.log`) ; frontend non touché : `npm run check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` PASS, `test:unit` **1939/1939** (163 fichiers), `build` vert (`326a-p2-frontend.log`) ; **E2E complet** (port 3326, `kesh_e2e_326a`, dossiers neufs `/tmp/kesh-e2e-326a-p2/`) : **315 passés, 7 échecs = les 7 KF-029**, 17 ignorés, KF-052 non déclenchée (`326a-p2-e2e.log`) ; backend arrêté par son PID. **Écart de méthode déclaré** : le premier arrêt du backend a visé le PID de l'enveloppe `bash` ; le processus `kesh-api` a été arrêté ensuite par son propre PID, port 3326 constaté libre. Le correctif touche la production : passe suivante **ciblée** (regression hunter, **Sonnet**, `32-6a-review-prompt-p3.md` ; rapport `kesh-gate-logs/326a-review-p3.md`). |
| 2026-10-25 | **Revue de code P3 (32-6a) — REVUE TERMINÉE** | **Passe ciblée** (regression hunter, **Sonnet**, une lentille, braquée sur `dd4b8398` ; prompt `32-6a-review-prompt-p3.md` ; rapport `kesh-gate-logs/326a-review-p3.md`) : **0/0/0/1**. **Trend** : P1 Sonnet ×3 (B, E, A) 0/0/2/16 → P2 Opus ciblée 0/0/2/7 → **P3 Sonnet ciblée 0/0/0/1**. Recyclage déclaré en P2 (les deux MEDIUM nés de la remédiation P1). **Clôture** sur une passe sans finding au-dessus de LOW (`CLAUDE.md`, décision Q3 (c)), **3 passes sur 8**. Le LOW R1-P3 (marges de temps des deux tests d'abandon, faux rouge possible sous charge, fréquence non établie) n'est pas remédié ici : il est tracé en commentaire de #759 et repris à l'étape 0 de la 32-6b (même worktree). Dernier commit de code `dd4b8398` : gate complet 4183/4183, frontend 1939/1939, E2E 315 passés et 7 KF-029 (entrée P2). Fusion de `origin/main` (`966baed5`, documentation seule : `CLAUDE.md`, fichier d'epic) sans nouveau gate, aucun code touché. *(Écrit par l'orchestrateur.)* |
