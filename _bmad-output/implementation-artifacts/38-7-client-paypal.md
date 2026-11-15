# Story 38-7 : client PayPal — OAuth2, Transaction Search, soldes, « Tester la connexion »

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`. Epic 38 (`planning-artifacts/epic-38-paypal.md`),
E2, E3, E9, E10, E20, E24 ; contrats C-38-1, C-38-2, C-38-5, C-38-11, C-38-12. Décisions B1, B8, B11, B12, B23.
**Dépend de** 38-3 (crate `kesh-providers`, trait de transport, `Connector`, passage, transport factice) et, par lui,
de 38-1 (coffre `secrets::{seal, open}`, connexions, `settings` typé) et de l'Epic 37 (client HTTP borné, 37-2 AC-3).
**Suivie de** 38-8 (classement des enregistrements que ce client lit) ; **lue par** 38-9a (écran, messages
d'erreur). **Aucune migration.***

## Story

En tant qu'Administrateur d'une société qui tient un compte PayPal *business*, je veux que Kesh s'authentifie auprès
de PayPal avec l'identifiant de mon application REST et son secret, lise les soldes de chaque devise et les
transactions qui changent le solde, toutes les pages, par fenêtres que PayPal accepte, et que « Tester la connexion »
me dise en clair si l'accès marche — et sinon pourquoi —, afin que le téléchargement de 38-3 dispose d'une source
PayPal fiable, bornée et sans fuite de secret.

## Source (relevée le 2026-11-15, documentation officielle de PayPal)

- **Authentification** — `POST /v1/oauth2/token`, en-tête `Authorization: Basic base64(client_id:client_secret)`,
  `Content-Type: application/x-www-form-urlencoded`, corps `grant_type=client_credentials` ; réponse `scope`,
  `access_token`, `token_type` (`Bearer`), `app_id`, `expires_in` (secondes ; l'exemple publié donne 31668), `nonce` ;
  « When your access token expires, call /v1/oauth2/token again to request a new access token »
  (<https://developer.paypal.com/api/rest/authentication/>). Base *live* `https://api-m.paypal.com`, *sandbox*
  `https://api-m.sandbox.paypal.com` (<https://developer.paypal.com/api/rest/requests/> : « The URL to the API service is
  either: » sandbox, live).
- **Transactions** — `GET /v1/reporting/transactions`, portée
  `https://uri.paypal.com/services/reporting/search/read` (<https://developer.paypal.com/api/transaction-search/v1/search-get>) :
  `start_date`, `end_date` obligatoires, **RFC 3339, secondes requises**, écart **≤ 31 jours** ; `fields`
  (`transaction_info` par défaut, `all` pour tout) ; `balance_affecting_records_only` (`Y` par défaut) ;
  `page_size` 1–500 (défaut 100) ; `page` ≥ 1 (la description dit « zero-relative », les exemples commencent à 1 :
  **à constater sur les réponses de Beat**) ; historique de **trois ans** ; une transaction peut mettre **jusqu'à
  trois heures** à apparaître ; « if any optional query parameter is specified, the `ending_balance` response field
  is empty ». Réponse : `transaction_details[]`, `account_number`, `start_date`, `end_date`,
  `last_refreshed_datetime`, `page`, `total_items`, `total_pages`, `links`.
- **Soldes** — `GET /v1/reporting/balances` (<https://developer.paypal.com/api/transaction-search/v1/balances-get>,
  schéma <https://developer.paypal.com/api/transaction-search/v1/schema.json>) : `as_of_time` (« will return the last
  refreshed balance in the system when not provided »), `currency_code` ; réponse `balances[]` (`currency`, `primary`,
  `total_balance`, `available_balance`, `withheld_balance`), `account_id` (« The PayPal payer ID »), `as_of_time`,
  `last_refresh_time`.
- **Erreurs** (schéma JSON ci-dessus) : `AUTHENTICATION_FAILURE` (401), `NOT_AUTHORIZED` (403, « not authorized to
  access this resource, although it may have valid credentials »), `INVALID_REQUEST` (400),
  **`RESULTSET_TOO_LARGE`** (« The request returned more items than the maximum limit. To reduce the result set,
  include additional query parameters »), `RESOURCE_NOT_FOUND` (404), `INTERNAL_SERVER_ERROR` (500),
  `SERVICE_UNAVAILABLE` (503). ⚠️ La page de référence actuelle **ne chiffre plus** la limite de 10 000 résultats
  citée au cadrage (`epics-38-40-cadrage.md`, *Sources consultées*, 2026-11-14) : Kesh traite donc
  `RESULTSET_TOO_LARGE` **et** `total_items > 10 000` (AC-5).

## Exigences

- `FR-38-7-1` — Obtenir un jeton OAuth2 avec `clientId` et le secret de la connexion → AC-3.
- `FR-38-7-2` — Lire les transactions par fenêtres de 31 jours au plus, toutes les pages, en redécoupant au-delà de
  10 000 résultats → AC-4, AC-5.
- `FR-38-7-3` — Lire les soldes par devise, à une date donnée (`as_of_time`) → AC-6.
- `FR-38-7-4` — « Tester la connexion » lit les soldes et pose `payerId` aux réglages (`accountEmail` : voir *Écart à
  l'epic*) → AC-8.
- `FR-38-7-5` — Chaque erreur PayPal devient un état lisible de la connexion → AC-7, AC-10.
- `FR-38-7-6` *(ajoutée)* — La date « télécharger depuis » d'une connexion PayPal ne remonte pas au-delà de
  l'historique de trois ans : refusée à l'écriture, bornée au passage → AC-9.
- `NFR-38-7-1` — URL de base constante, client HTTP borné de 37-2 (délai, taille) → AC-1, AC-2.
- `NFR-38-7-2` — Jeton en mémoire seulement, jamais écrit → AC-3, AC-11.
- `NFR-38-7-3` — Validation stricte des réponses ; tests sur fixtures anonymisées et réponses hostiles → AC-4, AC-6,
  AC-12.
- `NFR-38-7-4` *(ajoutée)* — Aucun secret ni jeton dans les journaux, erreurs, réponses ou audit → AC-11.

## Critères d'acceptation

1. **AC-1 — Transport HTTP de production** (`kesh-api`, module `providers::http`). Implémentation `ReqwestTransport`
   du trait de transport objet-safe posé par 38-3 (C-38-5, E9 ; nom fixé par la fiche 38-3 — s'il diffère de
   `kesh_providers::Transport`, celui de 38-3 prévaut). Elle **réutilise le constructeur du client borné de 37-2**
   (37-2 AC-3 : `reqwest` en production, `default-features = false`, `rustls-tls`, `redirect(Policy::none())`,
   `https_only(true)`, pas de magasin de cookies, `User-Agent: Kesh/<version>`, lecture par `chunk()` bornée) ; si 37-2
   l'a écrit avec des bornes en dur, 38-7 en fait des **paramètres** (`connect_timeout`, délai total, taille maximale
   du corps) sans changer les valeurs de 37-2 (DRY, pas de second client). Bornes PayPal : connexion 5 s, délai total
   **60 s** (Transaction Search à 500 lignes avec `fields=all`), corps **≤ 8 MiB** (`Content-Length` contrôlé, puis
   lecture bornée ; dépassement → `ResponseTooLarge`). `reqwest` est aujourd'hui une dev-dependency seulement
   (`crates/kesh-api/Cargo.toml:109`) ; la dépendance de production est celle de 37-2.
2. **AC-2 — URL de base constante.** `kesh_providers::paypal::LIVE_BASE_URL = "https://api-m.paypal.com"`, **aucune
   variable d'environnement** ne la change (C-37-6). Les tests construisent `PaypalConnector::with_base_url(…)` contre
   un serveur axum local en HTTP — `#[doc(hidden)] pub`, jamais `pub(crate)` ni `#[cfg(test)]` (invisibles des tests
   d'intégration, précédent de l'Epic 31 rappelé à 37-2 AC-3). La base *sandbox* n'est écrite nulle part dans le code
   de production (B12 : la recette se fait sur les comptes réels, en lecture seule, sur une instance jetable).
3. **AC-3 — Jeton OAuth2** (`kesh-providers::paypal::auth`, pur sauf le transport).
   - Requête : `POST {base}/v1/oauth2/token`, `Authorization: Basic base64(clientId:secret)`,
     `Content-Type: application/x-www-form-urlencoded`, corps `grant_type=client_credentials`. `clientId` lu dans
     `PaypalSettings` (E24, `kesh-core::providers::settings`, posé par 38-1) ; le secret obtenu par
     `secrets::open` (C-38-1) **au seul moment de l'appel**, dans la portée du passage ou du test, jamais gardé
     (`Zeroizing<String>` si 38-1 l'expose ainsi ; sinon `drop` explicite après l'en-tête).
   - Réponse validée : `token_type` égal à `Bearer` (casse ignorée), `access_token` non vide ≤ 4096 caractères,
     `expires_in` entier 60 ≤ n ≤ 86 400 ; sinon `ResponseInvalid { what: "token" }`. Le `scope` rendu **doit
     contenir** `https://uri.paypal.com/services/reporting/search/read` ; sinon `PermissionMissing` dès le jeton
     (message plus clair qu'un 403 plus tard).
   - **Cache en mémoire** : `AppState.provider_token_cache: Arc<std::sync::Mutex<HashMap<i64, CachedToken>>>` (clé :
     connexion), entrée `{ token, expires_at, secret_updated_at }` ; réutilisée tant que `now < expires_at − 120 s`
     **et** que `secret_updated_at` égale celui de la connexion (un secret remplacé invalide le jeton sans autre
     signal) ; vidée par `PUT /{id}/secrets` et `DELETE /{id}` de 38-1 (appel ajouté ici), par la restauration
     complète (qui efface les secrets, E2) et par `clear_transient_test_state` (`routes/test_endpoints.rs:349`).
     Champ neuf d'`AppState` (`lib.rs:61`) : sites de construction littérale à compléter — `main.rs:439`,
     `middleware/auth.rs:285`, `AppState::new_for_tests` (`lib.rs:119`), `tests/password_recovery_e2e.rs:126`,
     `tests/invoice_frozen_pdf_e2e.rs:101`, `tests/invoice_send_email_e2e.rs:104, 1036, 1150`,
     `tests/setup_admin_e2e.rs:82`, `tests/common/mcp.rs:215` (inventaire au `grep -rn "AppState {"`, à relancer ;
     l'échec de compilation le complète).
   - Un `401` sur un appel de données avec un jeton en cache → **un** renouvellement du jeton puis **un** nouvel essai ;
     un second `401` → `AuthFailed`.
4. **AC-4 — Transactions** (`PaypalConnector::movements`, forme de `Connector::movements(window, page)` de C-38-5).
   - Requête : `GET {base}/v1/reporting/transactions?start_date=…&end_date=…&fields=all&page_size=500&page=<n>
     &balance_affecting_records_only=Y`, dates en UTC `YYYY-MM-DDTHH:MM:SSZ` (secondes écrites). La fenêtre reçue est
     refusée **avant tout appel** si `end − start > 31 jours` ou `start ≥ end` (`ConnectorError::InvalidWindow`,
     erreur de programmation de l'appelant, `tracing::error!`).
   - Rendu : `RawPage { records: Vec<PaypalRecord>, page, total_pages, total_items }`. `PaypalRecord` est la **forme
     validée** d'un `transaction_details[]` — les champs lus par 38-8 seulement (`transaction_info` : `transaction_id`,
     `paypal_reference_id`, `paypal_reference_id_type`, `transaction_event_code`, `transaction_initiation_date`,
     `transaction_updated_date`, `transaction_amount`, `fee_amount`, `transaction_status`, `transaction_subject`,
     `invoice_id` ; `payer_info` : `account_id`, `email_address`, `payer_name.full_name` /
     `given_name` + `surname`, `payer_info.payer_name.alternate_full_name` ; `cart_info.item_details[].item_name`) ; les
     autres champs sont **ignorés** (PayPal en ajoute : `deny_unknown_fields` est **interdit** ici, à l'inverse des
     réglages E24), la réponse brute n'est **jamais** gardée (E8, données personnelles).
   - **Validation stricte** (R5 de 37) — une violation rejette **la page entière** (`ResponseInvalid { what, page }`,
     rien de la page n'est rendu) : JSON non objet ; `transaction_details` absent ou non tableau ; plus de
     `page_size` enregistrements ; `page` rendu ≠ `page` demandé ; `total_pages` < `page` (sauf `total_items = 0`) ou
     `total_items > total_pages × 500` ; `transaction_id` absent, vide, > 64 caractères ou hors `[A-Za-z0-9-]` ;
     `transaction_event_code` hors `^T\d{4}$` ; `transaction_status` hors `D|P|S|V` ; montant (`value`) qui n'est
     pas un décimal à au plus deux décimales ou dont la valeur absolue dépasse 10¹² ; `currency_code` hors
     `^[A-Z]{3}$` ; `fee_amount.currency_code` ≠ `transaction_amount.currency_code` ; date d'initiation non RFC 3339
     ou hors `[start − 1 jour, end + 1 jour]` ; chaînes > 2000 caractères (sujet, nom, courriel > 254) ;
     `item_details` > 500 éléments. **Unicité** : deux enregistrements de même `transaction_id` **et** même devise dans
     une page → page rejetée (la doc dit que les identifiants ne sont pas uniques ; une collision dans une devise
     casserait `UNIQUE (bank_account_id, provider_tx_id)` de E6 en silence — à constater sur les réponses de Beat, AC-12).
   - **Pages** : l'appelant (38-3) demande `page = 1, 2, …, total_pages` ; `total_pages` qui change d'une page à
     l'autre de la même fenêtre → `ResponseInvalid { what: "pagination" }` (la fenêtre se reprend au passage suivant,
     C-38-5 « un échec garde ce qui est inséré »).
   - `ending_balance` et `available_balance` ne sont **pas** lus (vides dès qu'un paramètre facultatif est donné —
     doc ci-dessus) : la complétude se contrôle par les soldes (AC-6, E20 a).
5. **AC-5 — Fenêtre trop large.** `RESULTSET_TOO_LARGE` (corps d'erreur `name`), **ou** `total_items > 10 000` à la
   page 1 → `ConnectorError::WindowTooLarge { total_items: Option<u32> }` sans lire d'autre page. Le découpage est
   celui du passage (C-38-5 : « fenêtre ≤ 31 jours, découpée ») : la fenêtre se **coupe en deux moitiés** et se
   relit ; sous **une heure**, `WindowTooLarge` est rendu tel quel (`PROVIDER_WINDOW_TOO_LARGE`, AC-7). Si la fiche
   38-3 ne porte pas ce découpage, il se fait dans `PaypalConnector` (méthode `movements_window` qui parcourt pages et
   moitiés) — **à trancher à la validation de 38-3**, un seul des deux, jamais les deux. `Connector::max_window()` rend
   31 jours pour PayPal.
6. **AC-6 — Soldes** (`PaypalConnector::balances(as_of: Option<DateTime<Utc>>)`, C-38-5).
   `GET {base}/v1/reporting/balances[?as_of_time=…]` (sans `currency_code` : toutes les devises). Rendu
   `Balances { payer_id, as_of, last_refresh, items: Vec<{ currency, total, available, withheld, primary }> }`.
   **Le solde comparé (E20) est `total_balance`** — somme de tous les autres selon PayPal ; le classement de 38-8
   écarte les blocages, qui ne changent pas `total_balance`. Validation stricte : `balances` tableau ≤ 50 éléments,
   devises `^[A-Z]{3}$` **uniques**, montants comme AC-4, `total_balance.currency_code` égale `currency`, `account_id`
   non vide ≤ 64, `as_of_time` RFC 3339. `as_of` demandé et `as_of_time` rendu **postérieur** de plus d'un jour →
   `ResponseInvalid { what: "as_of" }` (PayPal ne doit pas rendre un solde d'après la date demandée ; antérieur :
   accepté, c'est « le dernier solde servi »). Un solde de devise sans compte Kesh est l'affaire de 38-3 (E12).
7. **AC-7 — Erreurs PayPal → états lisibles.** `kesh_providers::ConnectorError` (38-3) reçoit, par correspondance
   PayPal, des variantes **sans texte de PayPal recopié** (le `message` et `debug_id` de PayPal vont au
   `tracing::warn!`, jamais à l'écran ni à `last_error`) :

   | Cause | Variante | Code (`last_error`, `errorCode` de `POST /test`, erreur de `POST /fetch`) | HTTP Kesh de `POST /fetch` |
   |---|---|---|---|
   | 401 au jeton (`invalid_client`), 401 répété | `AuthFailed` | `PROVIDER_AUTH_FAILED` | 409 |
   | 403 `NOT_AUTHORIZED` ; portée absente du jeton (AC-3) | `PermissionMissing` | `PROVIDER_PERMISSION_MISSING` | 409 |
   | 429 | `RateLimited { retry_after }` | `PROVIDER_RATE_LIMITED` | 503 |
   | 5xx, délai, réseau, TLS, redirection | `Unavailable` | `PROVIDER_UNAVAILABLE` | 503 |
   | réponse invalide (AC-4, AC-6), corps trop gros | `ResponseInvalid` | `PROVIDER_RESPONSE_INVALID` | 503 |
   | fenêtre indivisible (AC-5) | `WindowTooLarge` | `PROVIDER_WINDOW_TOO_LARGE` | 503 |
   | 400 `INVALID_REQUEST` | `Rejected` | `PROVIDER_REQUEST_REJECTED` | 503 |

   ⛔ **Jamais un 401 ou un 403 de Kesh** pour une erreur de PayPal : le client du frontend lit un 401 comme une session
   expirée (`frontend/src/lib/shared/utils/api-client.ts:6`, chemin 401 → renouvellement puis déconnexion). Le 503
   suit le précédent de 37-2 (`RATE_SOURCE_UNAVAILABLE`) ; un `POST` n'est pas rejoué par le client sur 503
   (méthodes idempotentes seules, `api-client.ts:9, 393`). Un 5xx ou un délai de PayPal est essayé **une seconde
   fois** après 2 s ; un 429 ne l'est pas dans le passage (reprise au passage suivant, C-38-5), `Retry-After` gardé au
   journal. `details` des réponses : `{ connectionId }`, plus `{ page }` pour `PROVIDER_RESPONSE_INVALID`. Codes neufs
   à `AppError` (`errors.rs`), message traduit par `t(…)` comme les autres (`errors.rs:3303-3321`).
8. **AC-8 — « Tester la connexion »** : la route `POST /api/v1/provider-connections/{id}/test` de 38-1 (C-38-2,
   38-1 AC-7) garde sa forme **200** `{ usable, unusableReason, checks: [{ check, ok }] }` et ses contrôles locaux
   (`provider_known` … `secret_decryptable`). Quand ceux-ci passent, elle appelle, pour `provider = 'paypal'`,
   `PaypalConnector::test()` = jeton (AC-3) puis soldes sans `as_of` (AC-6), et ajoute :
   - aux `checks` : `token_obtained`, `permission_present` (portée de l'AC-3) et `provider_reachable` (soldes lus ;
     contrôle annoncé par 38-1 AC-7 pour 38-3 — **un seul** site l'écrit, celui de 38-7 pour PayPal, 38-3 n'en posant
     que la forme générique) ; un contrôle échoué porte `errorCode` (code de l'AC-7) et les suivants ne sont pas
     tentés (`ok: null`) ;
   - à la réponse : `payerId` (ou `null`), `balances: [{ currency, total }]` (lus, non enregistrés — les soldes
     enregistrés sont ceux des passages, E20) et `version` (de la connexion après l'écriture éventuelle).
   - Succès : `payerId` (= `account_id`) **écrit dans `settings`** (`PaypalSettings.payer_id`, champ posé par le serveur,
     38-1 AC-4) s'il diffère, en transaction avec `version` + 1 et l'audit ; sinon rien d'écrit. `last_error` remis à
     `NULL`. Échec d'un contrôle distant : `last_error` posé au code (sans texte de PayPal) ; aucune autre écriture.
     Toujours **200** : le résultat d'un test est une donnée, non une erreur de la requête.
   - Audit : `payment_provider_connection.tested` (entité `payment_provider_connection` de 38-1 AC-4), écrit **à chaque
     appel distant** (succès ou échec), `details` = `{ result: "ok" | <code>, payerIdChanged: bool, currencies: […] }`
     — **ni secret, ni jeton, ni montants**. Libellé à `audit_labels.rs` (`ACTIONS`, trié ; voisinage
     `audit_labels.rs:90`) et aux quatre locales. **Registre des routes** (C-38-12) : l'entrée de `test_connection`
     posée par 38-1 en `NoMatter(…)` passe à `Traced` — delta 0 route, +1 tracée, −1 sans mutation, total et message
     de `the_registry_partition_is_what_the_story_declares` (`tests/audit_route_registry.rs:1009`) suivis.
   - Rôle : Administrateur, session seulement (`admin_routes`, `require_not_pat`, `middleware/rbac.rs:58` ; 38-1 AC-3).
   - Le secret s'ouvre par la **voie unique** de 38-1 AC-1 (fonction `pub(crate)` de `kesh_api::secrets`, appelants
     listés par une garde de source) : 38-7 ajoute à cette liste **le seul** site qui construit l'en-tête `Basic` ;
     `kesh-providers` reçoit le clair en `Zeroizing<String>` le temps de l'appel au jeton.
9. **AC-9 — Historique de trois ans.** `Connector::history_start(now)` rend, pour PayPal, `now − 3 ans + 1 jour`
   (date Europe/Zurich, aide de fuseau de 37-2 `aujourdhui_zurich`) ; `None` pour un prestataire sans limite.
   - **Écriture** : `POST` et `PATCH /provider-connections` (38-1) refusent, pour `paypal`, un `fetchFrom` antérieur :
     400 `PROVIDER_FETCH_FROM_TOO_OLD`, `details: { earliest: "AAAA-MM-JJ" }` (validation ajoutée à la fonction de
     validation des réglages de 38-1, un seul point d'entrée).
   - **Lecture** : `GET /provider-connections/{id}` et la liste gagnent `earliestFetchFrom` (date ou `null`), que
     l'écran de 38-9a lit pour borner le sélecteur (aucune constante recopiée dans le TypeScript).
   - **Passage** : une date acceptée vieillit ; le passage de 38-3 part de `max(début calculé, history_start(now))` et,
     s'il a borné, l'écrit à l'état du passage (`truncated_from`, champ ou détail fixé par 38-3) — affiché par 38-9a.
     Jamais d'appel PayPal avant `history_start` (il rendrait `INVALID_REQUEST`).
10. **AC-10 — Branchement.** `kesh-api` choisit le connecteur par `provider` (fabrique `connector_for(connection,
    settings, transport)` posée par 38-3) : `paypal` → `PaypalConnector` ; en `KESH_TEST_MODE`, le **transport
    factice** de 38-3 sert des réponses **au format PayPal** (le connecteur PayPal tourne en entier, seul le transport
    est simulé) — 38-7 fournit les réponses synthétiques par défaut (jeton, soldes CHF/EUR/USD, une page de
    transactions couvrant les familles de 38-8), écrites en constantes du module et lues par les E2E de 38-9a/38-9b ;
    la route `_test` de 38-3 permet d'en servir d'autres (erreur 401, 403, 429, 500, `RESULTSET_TOO_LARGE`, réponse
    invalide). Aucune route neuve : `LIB_ROUTES` et `TEST_ENDPOINT_ROUTES` sans entrée ajoutée (C-38-12 ; seul le
    statut de `test_connection` change, AC-8).
11. **AC-11 — Aucune fuite.** Le secret, l'en-tête `Authorization` et le jeton n'apparaissent dans **aucun** journal
    (`tracing` : les requêtes se journalisent par méthode et chemin, sans en-têtes ni requête complète), message
    d'erreur, réponse, `last_error`, audit, export, sauvegarde ; `Debug` de `CachedToken` et des requêtes masque
    jeton et en-têtes (implémentation manuelle, `"***"`). Test : un passage et un test réussis puis échoués avec un
    secret et un jeton marqueurs (`kesh-secret-marqueur-…`, `kesh-jeton-marqueur-…`) ; la sortie `tracing` capturée,
    les corps de réponse, `audit_log`, `payment_provider_connections` et l'export CSV ne contiennent aucun des deux
    (patron du test de recherche de 38-1, NFR-38-1-2, étendu au jeton).
12. **AC-12 — Tests sans réseau** (C-38-11, B12).
    - Fixtures **anonymisées** sous `crates/kesh-providers/tests/fixtures/paypal/` : jeton, soldes (trois devises, une
      à `as_of_time`), transactions (page 1/2 et 2/2 d'une fenêtre, fenêtre vide `total_items = 0`), erreurs
      (`AUTHENTICATION_FAILURE`, `NOT_AUTHORIZED`, `RESULTSET_TOO_LARGE`, `INVALID_REQUEST`, 500, 503). Anonymisation :
      courriels, noms, `account_id`, `transaction_id`, `paypal_reference_id`, `invoice_id` remplacés de façon
      **cohérente** (une même valeur réelle → une même valeur factice, liens gardés) ; montants, devises, codes
      d'événement, statuts et dates gardés. Les réponses réelles de Beat sont relevées sur une **instance jetable**
      (jamais sa production), anonymisées hors du dépôt, puis seules les versions anonymisées entrent ; un test vérifie
      qu'aucune fixture ne contient `@` hors du domaine `example.invalid`.
    - Contre un serveur axum local (`with_base_url`) : jeton demandé une fois pour deux appels, renouvelé après
      expiration (horloge injectée) et après un secret remplacé ; 401 → renouvellement et nouvel essai, deux 401 →
      `AuthFailed` ; portée absente → `PermissionMissing` sans appel de données ; 429, 500 puis 200 (second essai
      réussi), 500 deux fois ; délai dépassé ; redirection 302 non suivie ; corps de 9 MiB ; `Content-Length` menteur ;
      fenêtre de 32 jours refusée sans appel ; `RESULTSET_TOO_LARGE` et `total_items = 10 001` → `WindowTooLarge` sans
      page 2 ; pagination incohérente ; chaque règle de validation de l'AC-4 et de l'AC-6 par un cas hostile ;
      `transaction_id` en double dans une devise ; `as_of_time` postérieur ; un test vérifie la forme exacte de la
      requête (chemin, paramètres, `Authorization: Bearer`, dates en `Z` avec secondes).
    - Route `POST /test` (tests d'intégration `kesh-api`) : succès qui pose `payerId` (version + 1, audit), succès sans
      changement (rien d'écrit, audit `tested` seul), `PROVIDER_AUTH_FAILED` et `PROVIDER_UNAVAILABLE` → 200, contrôle
      en échec avec `errorCode`, `last_error` posé, contrôles suivants à `null` ; rôle Comptable → 403 ; clé d'API →
      `403 API_KEY_ADMIN_FORBIDDEN` ; clé `KESH_SECRETS_KEY` absente → contrôle local en échec (38-1 AC-7), **aucun
      appel sortant** (compteur du serveur local à 0).
    - `fetchFrom` : 3 ans moins un jour accepté, 3 ans plus un jour → 400 `PROVIDER_FETCH_FROM_TOO_OLD` ; passage
      borné (connecteur factice).
13. **AC-13 — Ce que la story ne touche pas.** Aucune migration (P1–P8 sans objet) ; aucune table ni colonne ;
    aucun outil MCP (E23 : connexions et passages ne sont pas exposés ; rien à justifier de plus) ; aucun écran (38-9a)
    ni texte de manuel (38-9a écrit la permission *Transaction Search* et la création de l'application REST) ; i18n :
    les messages des sept codes de l'AC-7, de `PROVIDER_FETCH_FROM_TOO_OLD` et le libellé d'audit, quatre locales
    (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`).

## Écart à l'epic

- **`accountEmail`** (E24, FR-38-7-4) : ni `GET /v1/reporting/balances` ni `GET /v1/reporting/transactions` ne
  rendent le courriel du compte *business* (relevé à la documentation ci-dessus : `account_id` et `account_number`
  seulement). L'obtenir demanderait l'API d'identité (`/v1/identity/openidconnect/userinfo`), d'une autre portée,
  non demandée à l'application en lecture seule (B12). **« Tester la connexion » pose `payerId` seulement** ;
  `accountEmail` reste un champ de `PaypalSettings` posé par le serveur (38-1 AC-4 refuse qu'un client l'écrive),
  **toujours `None`** dans cette story, jamais lu pour une décision. La reconnaissance d'un virement entre les deux
  comptes PayPal de Beat (E24) se fait par `payerId` (`payer_info.account_id` de la contrepartie, 38-8). Question
  Q-38-7-1.
- **Limite de 10 000 résultats** : non chiffrée par la référence actuelle (voir *Source*) ; Kesh traite le code
  d'erreur et le seuil (AC-5).
- **Erreurs « en états lisibles »** : l'epic ne fixe ni codes ni statuts HTTP ; la table de l'AC-7 les fixe.

## Questions ouvertes

- **Q-38-7-1** (Beat) : `accountEmail` — le retirer de `PaypalSettings`, le rendre saisissable par l'Administrateur
  (ce qui amende 38-1 AC-4), ou demander la portée d'identité ? Proposé : le retirer ; `payerId` suffit.
- **Q-38-7-2** (validation de 38-3) : le découpage d'une fenêtre trop large est-il au passage (proposé) ou au
  connecteur ? (AC-5.)
- **Q-38-7-3** (recette) : `page` commence-t-il à 1 (exemples) ou 0 (description) ? À constater sur les réponses de
  Beat avant la fixture ; le code n'en suppose qu'un, testé.

## Tâches

- [ ] Transport `ReqwestTransport` (AC-1) sur le constructeur paramétré du client de 37-2 ; tests de bornes.
- [ ] `kesh-providers::paypal` : `LIVE_BASE_URL`, `with_base_url`, `auth` (jeton, portée, cache, renouvellement sur
      401), `movements` (requête, validation, `WindowTooLarge`), `balances`, `test()`, `history_start`, `max_window`
      (AC-2 à AC-6, AC-9).
- [ ] `ConnectorError` : variantes PayPal et correspondance aux codes (AC-7) ; `AppError` : huit codes neufs,
      statuts, messages, quatre locales.
- [ ] `AppState.provider_token_cache` et ses sites de construction ; vidage par `PUT /secrets`, `DELETE`,
      restauration, `clear_transient_test_state` (AC-3).
- [ ] Route `POST /provider-connections/{id}/test` (38-1) branchée sur PayPal ; écriture de `payerId` ; audit
      `payment_provider_connection.tested` (`audit_labels.rs`, quatre locales), registre `test_connection` → `Traced`
      (AC-8).
- [ ] Validation `fetchFrom` et champ `earliestFetchFrom` dans les routes de 38-1 ; borne au passage de 38-3 (AC-9).
- [ ] Réponses synthétiques par défaut du transport factice au format PayPal (AC-10).
- [ ] Fixtures anonymisées, relevé sur instance jetable, test « aucun courriel réel » ; tests hostiles ; tests de
      route ; test de non-fuite (AC-11, AC-12).
- [ ] Gate : `cargo fmt`, `clippy -D warnings`, `cargo nextest` complet (le crate `kesh-providers` et `kesh-api`),
      `cargo test --workspace` en série avant `done` ; `Cargo.lock` commité si une dépendance change.

## Dev Notes

- **Forme du trait** : objet-safe sans `async_trait`, futur boxé comme `Mailer` (`crates/kesh-api/src/mail/mod.rs:33`
  `MailFuture`, trait `:69`) et `RateSource` de 37-2 AC-3 ; porté par `AppState` (`lib.rs:84` `mailer`, `:98`
  `test_mock_mailer` pour le patron de la poignée de test).
- **Fuseau** : `transaction_initiation_date` est en UTC ; la date comptable (Europe/Zurich) est l'affaire de 38-8 ;
  `chrono-tz` arrive en dépendance directe de `kesh-api` par 37-2 (AC-4 de 37-2) ; `kesh-providers` l'ajoute aussi
  (déjà au `Cargo.lock`, 0.10.4) s'il calcule `history_start` en date locale.
- **Pourquoi `total_balance`** : `available + withheld (+ autres) = total` selon PayPal ; les blocages (T15xx, T21xx,
  écartés par 38-8) déplacent entre disponible et retenu sans changer le total — comparer au disponible ferait
  signaler un « téléchargement incomplet » à chaque blocage.
- **Pas de `deny_unknown_fields` sur les réponses** : PayPal ajoute des champs sans préavis ; la sévérité porte sur ce
  qu'on lit (AC-4), non sur ce qu'on ignore. À l'inverse, les **réglages** (E24) refusent l'inconnu.
- **Gabarit `<…>`** : un `clientId` de la forme `<…>` est refusé par la validation de 38-1 comme les autres gabarits
  (`is_template_placeholder`, `crates/kesh-api/src/config.rs:1538`) — à confirmer à la fiche 38-1, sinon ajouté ici.
- **Points de contact** : C-38-5 (`Connector`, transport, découpage, passage) et la fabrique de 38-3 ; C-38-2 (routes,
  `settings`, `version`, audit de 38-1) ; C-38-1 (`secrets::open`) ; 38-8 consomme `PaypalRecord` et `Balances`
  (noms fixés ici) ; 38-9a affiche les codes de l'AC-7, `earliestFetchFrom` et la date bornée du passage.
- **Splitting** : deux modules (`kesh-providers`, `kesh-api`), dans la règle (≤ 5).

### References

- `planning-artifacts/epic-38-paypal.md` § E2, E3, E9, E10, E20, E24, C-38-1, C-38-2, C-38-5, C-38-11, 38-7.
- `implementation-artifacts/37-2-cours-mensuels-afc.md` AC-3 (client borné, source objet-safe, mode test).
- Documentation PayPal citée en *Source* (consultée le 2026-11-15).

## Change Log

- 2026-11-15 — rédaction (Opus).
