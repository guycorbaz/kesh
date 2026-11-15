# Story 38-7 : client PayPal — OAuth2, Transaction Search, soldes, « Tester la connexion »

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`. Epic 38 (`planning-artifacts/epic-38-paypal.md`),
E2, E3, E9, E10, E20, E24 ; contrats C-38-1, C-38-2, C-38-5, C-38-11, C-38-12. Décisions B1, B8, B11, B12, B23.
**Dépend de** 38-3b (crate `kesh-providers`, trait de transport, `Connector`, passage, transport factice) et, par lui,
de 38-3a (colonnes et tables du passage, décodage tolérant),
de 38-1 (coffre `secrets::{seal, open}`, connexions, `settings` typé) et de l'Epic 37 (client HTTP borné, 37-2 AC-3).
**Suivie de** 38-8 (classement des enregistrements que ce client lit) ; **lue par** 38-9a (écran, messages
d'erreur). **Aucune migration.***

## Story

En tant qu'Administrateur d'une société qui tient un compte PayPal *business*, je veux que Kesh s'authentifie auprès
de PayPal avec l'identifiant de mon application REST et son secret, lise les soldes de chaque devise et les
transactions qui changent le solde, toutes les pages, par fenêtres que PayPal accepte, et que « Tester la connexion »
me dise en clair si l'accès marche — et sinon pourquoi —, afin que le téléchargement de 38-3b dispose d'une source
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
- `FR-38-7-4` — « Tester la connexion » lit les soldes et pose `payerId` aux réglages (`accountEmail` retiré : O10,
  voir *Écart à l'epic*) → AC-8.
- `FR-38-7-5` — Chaque erreur PayPal devient un état lisible de la connexion → AC-7, AC-10.
- `FR-38-7-6` *(ajoutée)* — La date « télécharger depuis » d'une connexion PayPal ne remonte pas au-delà de
  l'historique de trois ans : refusée à l'écriture, bornée au passage → AC-9.
- `NFR-38-7-1` — URL de base constante, client HTTP borné de 37-2 (délai, taille) → AC-1, AC-2.
- `NFR-38-7-2` — Jeton en mémoire seulement, jamais écrit → AC-3, AC-11.
- `NFR-38-7-3` — Validation stricte des réponses ; tests sur fixtures anonymisées et réponses hostiles → AC-4, AC-6,
  AC-12.
- `NFR-38-7-4` *(ajoutée)* — Aucun secret ni jeton dans les journaux, erreurs, réponses ou audit → AC-11.

## Critères d'acceptation

1. **AC-1 — Transport HTTP de production** (`kesh-api`, module `providers`). **Pas de second transport** : 38-3b
   (transport) pose `kesh_providers::Transport` (trait objet-safe) et son implémentation `kesh-api::providers::HttpTransport` ;
   38-7 n'écrit que les **bornes propres à PayPal** qu'elle lui passe. Le client **réutilise le constructeur du client borné de 37-2**
   (37-2 AC-3 : `reqwest` en production, `default-features = false`, `rustls-tls`, `redirect(Policy::none())`,
   `https_only(true)`, pas de magasin de cookies, `User-Agent: Kesh/<version>`, lecture par `chunk()` bornée) ; si 37-2
   l'a écrit avec des bornes en dur, 38-7 en fait des **paramètres** (`connect_timeout`, délai total, taille maximale
   du corps) sans changer les valeurs de 37-2 (DRY, pas de second client). Bornes (38-3b AC-3) : le délai de
   **connexion**, 5 s, est une borne **du client**, posée à sa construction (`reqwest` n'a pas de `connect_timeout` par
   requête) ; **par requête** (`TransportLimits` passées par le connecteur PayPal avec chaque requête) : délai total
   **60 s** (Transaction Search à 500 lignes avec `fields=all` ; dépassé → `Unavailable`), corps **≤ 8 MiB**
   (`Content-Length` contrôlé, puis lecture bornée ; dépassement → `InvalidResponse`). Les tests de bornes passent par
   `HttpTransport::for_local_tests()` (HTTP admis, serveur axum local ; 38-3b AC-3), jamais par `HttpTransport::new()`.
   `reqwest` est aujourd'hui une dev-dependency seulement (`crates/kesh-api/Cargo.toml:109`) ; la dépendance de
   production est celle de 37-2.
2. **AC-2 — URL de base constante.** `kesh_providers::paypal::LIVE_BASE_URL = "https://api-m.paypal.com"`, **aucune
   variable d'environnement** ne la change (C-37-6). Les tests construisent `PaypalConnector::with_base_url(…)` contre
   un serveur axum local en HTTP — `#[doc(hidden)] pub`, jamais `pub(crate)` ni `#[cfg(test)]` (invisibles des tests
   d'intégration, précédent de l'Epic 31 rappelé à 37-2 AC-3). La base *sandbox* n'est écrite nulle part dans le code
   de production (B12 : la recette se fait sur les comptes réels, en lecture seule, sur une instance jetable).
3. **AC-3 — Jeton OAuth2** (`kesh-providers::paypal::auth`, pur sauf le transport).
   - Requête : `POST {base}/v1/oauth2/token`, `Authorization: Basic base64(clientId:secret)`,
     `Content-Type: application/x-www-form-urlencoded`, corps `grant_type=client_credentials`. `clientId` lu dans
     `PaypalSettings` (E24, `kesh-core::providers::settings`, posé par 38-1) ; le secret est ouvert **une seule fois
     par la fabrique de connecteurs de 38-3b** (`ConnectorFactory`, transport et mode test de 38-3b : c'est **son** site qui entre à la
     garde des appelants d'`open`, 38-1 AC-1) et remis en `Zeroizing<String>` au `PaypalConnector`, qui ne le garde
     que pour la vie du connecteur (un passage, ou un test) ; **38-7 n'ajoute aucun appelant d'`open`** : l'en-tête
     `Basic` se construit dans `kesh_providers::paypal::auth` (dépendances neuves du crate : `base64`, `zeroize`,
     déjà au `Cargo.lock`), à partir de ce clair, puis est effacé. Le connecteur reçoit aussi le `secret_updated_at`
     de la connexion (clé de validité du cache ci-dessous).
   - Réponse validée : `token_type` égal à `Bearer` (casse ignorée), `access_token` non vide ≤ 4096 caractères,
     `expires_in` entier 60 ≤ n ≤ 86 400 ; sinon `InvalidResponse { reason: "token" }`. Le `scope` rendu **doit
     contenir** `https://uri.paypal.com/services/reporting/search/read` ; sinon `Permission` dès le jeton
     (message plus clair qu'un 403 plus tard).
   - **Cache en mémoire** : type `PaypalTokenCache` défini dans `kesh_providers::paypal::auth` (un `Mutex<HashMap<i64,
     CachedToken>>`, clé : connexion ; le connecteur étant reconstruit par la fabrique à chaque passage, le cache vit
     hors de lui), porté par `AppState.provider_token_cache: Arc<PaypalTokenCache>` et passé au connecteur par la
     fabrique — **38-7 ajoute le champ à `ConnectorContext`** (38-3b ne le pose pas : son type est d'ici) ; entrée `{ token, expires_at, secret_updated_at }` ; réutilisée tant que `now < expires_at − 120 s`
     **et** que `secret_updated_at` égale celui de la connexion (un secret remplacé invalide le jeton sans autre
     signal) ; vidée par `PUT /{id}/secrets` et `DELETE /{id}` de 38-1 (appel ajouté ici), par la restauration
     complète (qui efface les secrets, E2) et par `clear_transient_test_state` (`routes/test_endpoints.rs:349`).
     Champ neuf d'`AppState` (`lib.rs:61`) : sites de construction littérale à compléter — `main.rs:439`,
     `middleware/auth.rs:285`, `AppState::new_for_tests` (`lib.rs:119`), `tests/password_recovery_e2e.rs:126`,
     `tests/invoice_frozen_pdf_e2e.rs:101`, `tests/invoice_send_email_e2e.rs:104, 1036, 1150`,
     `tests/setup_admin_e2e.rs:82`, `tests/common/mcp.rs:215` (inventaire au `grep -rn "AppState {"`, à relancer ;
     l'échec de compilation le complète).
   - Un `401` sur un appel de données avec un jeton en cache → **un** renouvellement du jeton puis **un** nouvel essai ;
     un second `401` → `Auth`.
4. **AC-4 — Transactions** (`PaypalConnector::movements`, forme de `Connector::movements(window, page)` de 38-3b).
   - **Contrat avec 38-3b (porté par 38-3b AC-2)** : 38-3b rend des `ProviderMovement` **classés**, page par page, et écrit une
     page par transaction ; or le classement de 38-8 (paire de conversion, remboursement lié) exige **toute la
     fenêtre**, et O10 veut le découpage d'une fenêtre trop large **dans le connecteur**. D'où : `PaypalConnector::
     movements(window, page)` **lit en interne toutes les pages PayPal de la fenêtre** (`page_size=500`, `page` de 1 à
     `total_pages`, redécoupage AC-5), valide, passe l'ensemble à `classify_window` (38-8) et rend **une seule page**
     `MovementPage { movements, page: 1, total_pages: 1, skipped, sums }` ; `page ≥ 2` → `InvalidWindow`. `skipped:
     SkippedCounts { denied, pending, non_balance }` et `sums: Vec<(devise, Σ net)>` sont des champs de
     `MovementPage` (38-3b AC-2 ; rendus par `classify_window`, 38-8 AC-1, AC-6) ; le passage écrit `skipped` à son état. Bornes : une fenêtre de 31 jours reste sous 10 000 enregistrements
     (AC-5) : au plus 10 000 enregistrements validés en mémoire (chaque page ≤ 8 MiB, libérée après lecture).
   - Requête (par page PayPal) : `GET {base}/v1/reporting/transactions?start_date=…&end_date=…&fields=all&page_size=500&page=<n>
     &balance_affecting_records_only=Y`, dates **envoyées** en UTC `YYYY-MM-DDTHH:MM:SSZ` (secondes écrites) ; `page`
     part de **1** (défaut et minimum de la référence, bien que sa description dise « zero-relative »). La fenêtre
     reçue est refusée **avant tout appel** si `end − start > 31 jours` ou `start ≥ end` (`ConnectorError::InvalidWindow`,
     erreur de programmation de l'appelant, `tracing::error!`).
   - **Dates rendues** : PayPal peut rendre un décalage sans deux-points (`2024-03-05T10:15:00+0000`), non conforme à
     RFC 3339 strict (l'en-tête `PayPal-Enforce-ISO8601-Format` existe pour cela) ; le validateur accepte `Z`, `±HH:MM`
     **et** `±HHMM`, convertit en UTC, et un test fige chaque forme — **à constater sur les réponses de Beat** ; sans
     cette tolérance, une page entière serait rejetée à tort. Les liens `links` de la réponse ne sont **jamais
     suivis** (URL de base constante).
   - Rendu interne : `RawPage { records: Vec<PaypalRecord>, page, total_pages, total_items }`. `PaypalRecord` est la **forme
     validée** d'un `transaction_details[]` — les champs lus par 38-8 seulement (`transaction_info` : `transaction_id`,
     `paypal_reference_id`, `paypal_reference_id_type`, `transaction_event_code`, `transaction_initiation_date`,
     `transaction_updated_date`, `transaction_amount`, `fee_amount`, `transaction_status`, `transaction_subject`,
     `invoice_id` ; `payer_info` : `account_id`, `email_address`, `payer_name.full_name` /
     `given_name` + `surname`, `payer_info.payer_name.alternate_full_name` ; `cart_info.item_details[].item_name`) ; les
     autres champs sont **ignorés** (PayPal en ajoute : `deny_unknown_fields` est **interdit** ici, à l'inverse des
     réglages E24), la réponse brute n'est **jamais** gardée (E8, données personnelles).
   - **Validation stricte** (R5 de 37) — une violation rejette **la page entière** (`InvalidResponse { reason:
     &'static str }`, la variante de 38-3b, inchangée (AC-7) : `reason` est **une constante** qui nomme la règle ou le
     champ en défaut — `"token"`, `"page"`, `"pagination"`, `"as_of"`, `"record.transaction_id"`,
     `"record.amount"`, … —, **jamais la valeur** ; il finit à `error_detail` du passage, car un enregistrement qui viole
     une règle bloque la fenêtre à chaque passage et doit se diagnostiquer sans lire la réponse brute ; la page PayPal
     et le rang de l'enregistrement, non constants, vont au `tracing::warn!` seulement ; rien de la page n'est rendu) : JSON non objet ; `transaction_details` absent ou non tableau ; plus de
     `page_size` enregistrements ; `page` rendu ≠ `page` demandé ; `total_pages` < `page` (sauf `total_items = 0`) ou
     `total_items > total_pages × 500` ; `transaction_id` absent, vide, > 64 caractères ou hors `[A-Za-z0-9-]` ;
     `transaction_event_code` hors `^T\d{4}$` ; `transaction_status` hors `D|P|S|V` ; montant (`value`) qui n'est
     pas un décimal à au plus deux décimales ou dont la valeur absolue dépasse 10¹² ; `currency_code` hors
     `^[A-Z]{3}$` ; `fee_amount.currency_code` ≠ `transaction_amount.currency_code` ; date d'initiation non RFC 3339
     ou hors `[start − 1 jour, end + 1 jour]` (au-delà : réponse hostile ; **dans** cette tolérance mais hors de la
     fenêtre demandée `[start, end)` : l'enregistrement est **retiré** avant le classement, sans compteur — il
     appartient à la fenêtre voisine, qui le lit ; d'où une frontière juste quelle que soit la façon dont PayPal
     borne `end_date`, et aucun `booked_at` hors fenêtre remis à `validate_page` de 38-3b, qui rejetterait la page —
     validation P2) ; chaînes > 2000 caractères (sujet, nom, courriel > 254) ;
     `item_details` > 500 éléments. **Unicité** : deux enregistrements de même `transaction_id` **et** même devise dans
     une page → page rejetée (la doc dit que les identifiants ne sont pas uniques ; une collision dans une devise
     casserait `UNIQUE (bank_account_id, provider_tx_id)` de E6 en silence — à constater sur les réponses de Beat, AC-12).
     ⚠️ Le même `transaction_id` dans **deux devises** est admis ici, mais `validate_page` de 38-3b refuse « deux fois
     le même `provider_tx_id` dans une page » sans regarder la devise : aligné à la P2 — 38-3b AC-2 refuse « le même
     couple `(provider_tx_id, currency)` », la clé unique étant par compte bancaire, donc par devise.
   - **Pages PayPal** (lues par le connecteur lui-même) : `total_pages` qui change d'une page à l'autre de la même
     fenêtre → `InvalidResponse { reason: "pagination" }` (la fenêtre se reprend au passage suivant, passage de 38-3b,
     « un échec garde ce qui est inséré » : rien n'est rendu au passage tant que la fenêtre n'est pas lue en entier).
   - `ending_balance` et `available_balance` ne sont **pas** lus (vides dès qu'un paramètre facultatif est donné —
     doc ci-dessus) : la complétude se contrôle par les soldes (AC-6, E20 a).
5. **AC-5 — Fenêtre trop large.** `RESULTSET_TOO_LARGE` (corps d'erreur `name`), **ou** `total_items > 10 000` à la
   page 1 → `ConnectorError::WindowTooLarge { total_items: Option<u32> }` sans lire d'autre page. **Le découpage se fait
   dans le connecteur (O10, décidé ; Q-38-7-2 close)** : 38-3b ne découpe qu'en sous-fenêtres de `max_window()`
   (31 jours) ; dans `PaypalConnector::movements`, la fenêtre se **coupe en deux moitiés** (récursivement) et chaque
   moitié se relit ; sous **une heure**, `WindowTooLarge` est rendu tel quel (`PROVIDER_WINDOW_TOO_LARGE`, AC-7). Les
   moitiés sont disjointes (`[from, mid)`, `[mid, to)`, demi-ouvertes comme `Window` de 38-3b) : aucune transaction
   n'est lue deux fois ni perdue, un test le vérifie à la frontière exacte. `Connector::max_window()` rend 31 jours
   pour PayPal.
6. **AC-6 — Soldes** (`Connector::balances(as_of: Option<DateTime<Utc>>) -> Vec<ProviderBalance>` de 38-3b :
   `{ currency, amount = total_balance, as_of }`, **`as_of` = `as_of_time` rendu par PayPal** (non l'instant demandé :
   38-3b AC-7 écrit ce `as_of` aux instantanés et arrête la somme des mouvements du contrôle à cet instant ; `as_of_time`
   absent de la réponse → `InvalidResponse { reason: "as_of" }`) ; la forme détaillée ci-dessous est la méthode **propre** à
   `PaypalConnector`, `balances_detailed`, que son `Connector::test()` lit pour `account_ref`, AC-8).
   `GET {base}/v1/reporting/balances[?as_of_time=…]` (sans `currency_code` : toutes les devises). Rendu
   `Balances { payer_id, as_of, last_refresh, items: Vec<{ currency, total, available, withheld, primary }> }`.
   **Le solde comparé (E20) est `total_balance`** — somme de tous les autres selon PayPal ; le classement de 38-8
   écarte les blocages, qui ne changent pas `total_balance`. Validation stricte : `balances` tableau ≤ 50 éléments,
   devises `^[A-Z]{3}$` **uniques**, montants comme AC-4, `total_balance.currency_code` égale `currency`, `account_id`
   non vide ≤ 64, `as_of_time` RFC 3339. `as_of` demandé et `as_of_time` rendu **postérieur** de plus d'un jour →
   `InvalidResponse { reason: "as_of" }` (PayPal ne doit pas rendre un solde d'après la date demandée ; antérieur :
   accepté, c'est « le dernier solde servi ») ; dans les deux cas admis, c'est `as_of_time` qui est rendu en `as_of` :
   postérieur (≤ 1 jour) à la fin de fenêtre, il laisse le contrôle « non disponible » pour ce relevé (38-3b AC-7 :
   `fetched_through` antérieur au `as_of` du `reported`), sans faux écart. Un solde de devise sans compte Kesh est l'affaire de 38-3a (soldes vus, E12), écrit par le
   passage de 38-3b.
7. **AC-7 — Erreurs PayPal → états lisibles.** `kesh_providers::ConnectorError` est **celui de 38-3b** (`Auth`,
   `Permission`, `RateLimited { retry_after }`, `Unavailable`, `InvalidResponse { reason: &'static str }`, `Transport { kind }`) ;
   38-7 **n'en renomme aucune variante** et en ajoute deux : `WindowTooLarge { total_items }` et `Rejected` (400),
   plus `InvalidWindow`. Aucun texte de PayPal recopié (le `message` et `debug_id` de PayPal vont au
   `tracing::warn!`, jamais à l'écran ni à `last_error`). **Une seule fonction** `ConnectorError::code()` rend le code
   du passage de 38-3b (`auth`, `permission`, `rate_limited`, `unavailable`, `invalid_response`, `transport` ;
   38-3b y ajoute `window_too_large`, `rejected`, `invalid_window`) et une seule `app_code()` le code d'API ci-dessous ;
   `last_error` de la connexion et l'erreur de « Tester » portent le **code d'API** :

   | Cause | Variante | Code d'API (`last_error`, `errorCode` de `POST /test`) | `code()` du passage |
   |---|---|---|---|
   | 401 au jeton (`invalid_client`), 401 répété | `Auth` | `PROVIDER_AUTH_FAILED` | `auth` |
   | 403 `NOT_AUTHORIZED` ; portée absente du jeton (AC-3) | `Permission` | `PROVIDER_PERMISSION_MISSING` | `permission` |
   | 429 | `RateLimited { retry_after }` | `PROVIDER_RATE_LIMITED` | `rate_limited` |
   | 5xx, délai | `Unavailable` | `PROVIDER_UNAVAILABLE` | `unavailable` |
   | réseau, TLS, redirection | `Transport { kind }` | `PROVIDER_UNAVAILABLE` | `transport` |
   | réponse invalide (AC-4, AC-6), corps trop gros | `InvalidResponse` | `PROVIDER_RESPONSE_INVALID` | `invalid_response` |
   | fenêtre indivisible (AC-5) | `WindowTooLarge` | `PROVIDER_WINDOW_TOO_LARGE` | `window_too_large` |
   | 400 `INVALID_REQUEST` | `Rejected` | `PROVIDER_REQUEST_REJECTED` | `rejected` |

   ⚠️ **Aucune de ces erreurs n'est un statut HTTP de `POST …/fetch`** : 38-3b (déclenchement) répond `202 { runId }` et le
   passage court en tâche de fond ; l'erreur se lit à l'état du passage (`GET …/runs`, `error_code`) et à `lastError`
   de la connexion. Seul `POST …/test` rend l'état de l'appel distant, **toujours en 200** (AC-8) : aucune de ces
   erreurs ne sort donc en 401, 403 **ni 503** — un 503 allumerait le bandeau « service dégradé » de tout le
   frontend (`api-client.ts:393-394`), conçu pour une base injoignable, et le client lit un 401 comme une session
   expirée (`api-client.ts:6`). Un 5xx ou un délai de PayPal est essayé **une seconde fois** après 2 s ; un 429 ne
   l'est pas dans le passage (reprise au passage suivant), `Retry-After` gardé au journal. Les codes d'API ne sont donc
   pas des `AppError` de réponse mais des chaînes constantes ; leur libellé lisible vient de 38-9a (quatre locales,
   `provider-labels.ts`), et le catalogue serveur n'en porte **que** pour `errorCode`/`lastError` rendus en clair
   (clés `provider-error-*` aux quatre locales, `messages.ftl`).
8. **AC-8 — « Tester la connexion »** : la route `POST /api/v1/provider-connections/{id}/test` de 38-1 (C-38-2,
   38-1 AC-7) garde sa forme **200** `{ usable, unusableReason, checks: [{ check, ok }] }` et ses contrôles locaux
   (`provider_known` … `secret_decryptable`). Quand ceux-ci passent, elle construit le connecteur par la **fabrique**
   de 38-3b (`Box<dyn Connector>`) et appelle **`Connector::test()`** (méthode du trait, 38-3b AC-2 — aucun
   transtypage vers `PaypalConnector`) ; `PaypalConnector` l'implémente : jeton (AC-3) puis soldes sans `as_of`
   (`balances_detailed`, AC-6), rendus en `ConnectorTest { account_ref: Some(account_id), balances }` ; la route
   ajoute :
   - aux `checks` : `token_obtained`, `permission_present` (portée de l'AC-3) et `provider_reachable` (soldes lus ;
     contrôle annoncé par 38-1 AC-7 pour 38-3 (désormais 38-3b) — **un seul** site l'écrit, celui de 38-7 pour PayPal, 38-3b n'en posant
     que la forme générique) ; un contrôle échoué porte `errorCode` (code de l'AC-7) et les suivants ne sont pas
     tentés (`ok: null`) ;
   - à la réponse : `payerId` (ou `null`), `balances: [{ currency, total }]` (lus, non enregistrés — les soldes
     enregistrés sont ceux des passages, E20) et `version` (de la connexion après l'écriture éventuelle).
   - Succès : `payerId` (= `account_ref` = `account_id`) **écrit dans `settings`** (`PaypalSettings.payer_id`, champ posé par le serveur,
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
   - Le secret s'ouvre par la **voie unique** de 38-1 AC-1, au site de la fabrique de 38-3b (AC-3 ci-dessus) : 38-7
     **n'ajoute aucun appelant** à la garde de source ; la route `test` construit son connecteur par cette fabrique.
     Le contrôle local `secret_decryptable` de 38-1 reste le seul autre site.
9. **AC-9 — Historique de trois ans.** `Connector::history_start(now)` rend, pour PayPal, `now − 3 ans + 1 jour`
   (date Europe/Zurich, calculée **dans `kesh-providers`** par `chrono-tz`, déjà dépendance du crate — « Temps » de
   38-3b AC-2 ; l'aide `aujourdhui_zurich` de 37-2 vit dans `kesh-api`, que `kesh-providers` ne peut pas appeler) ;
   `None` pour un prestataire sans limite.
   - **Écriture** : `POST` et `PATCH /provider-connections` (38-1) refusent, pour `paypal`, un `fetchFrom` antérieur :
     400 `PROVIDER_FETCH_FROM_TOO_OLD`, `details: { earliest: "AAAA-MM-JJ" }` (validation ajoutée à la fonction de
     validation des réglages de 38-1, un seul point d'entrée). **Sans construire de connecteur** (la création n'a
     parfois pas encore de secret) : la règle est une **fonction libre** `kesh_providers::paypal::history_start(now)`,
     que `PaypalConnector::history_start` appelle aussi (une seule règle) ; la route choisit la fonction par
     `provider` (`kesh_providers::earliest_fetch_from(provider, now) -> Option<NaiveDate>`, `None` pour un prestataire
     inconnu ou sans limite) — c'est aussi ce que lit `earliestFetchFrom` (ci-dessous). **Le passage, lui, lit
     `Connector::history_start`** du connecteur construit (38-3b AC-4), non `earliest_fetch_from` : pour PayPal les
     deux rendent la même date (même fonction libre), et le `FakeConnector` garde son `history_start` programmé.
   - **Lecture** : `GET /provider-connections/{id}` et la liste gagnent `earliestFetchFrom` (date ou `null`), que
     l'écran de 38-9a lit pour borner le sélecteur (aucune constante recopiée dans le TypeScript).
   - **Passage** : une date acceptée vieillit ; le passage de 38-3b part de `max(début calculé, history_start(now))` et,
     s'il a borné, l'écrit à l'état du passage (`provider_fetch_runs.truncated_from`, colonne de 38-3a, rendue par
     `GET …/runs` de 38-3b) — affiché par 38-9a.
     Jamais d'appel PayPal avant `history_start` (il rendrait `INVALID_REQUEST`).
10. **AC-10 — Branchement.** `kesh-api` choisit le connecteur par `provider` (`ConnectorFactory`, `AppState.
    connector_factory`, posée par 38-3b ; elle passe au `PaypalConnector` les réglages, le secret ouvert, le
    `secret_updated_at`, le cache de jeton, la liste des `payerId` des **autres** connexions PayPal de la société
    — 38-8 AC-5 —, et le transport) : `paypal` → `PaypalConnector` ; en `KESH_TEST_MODE`, le **transport
    factice** de 38-3b sert des réponses **au format PayPal** (le connecteur PayPal tourne en entier, seul le transport
    est simulé) — 38-7 fournit **un jeu de réponses synthétiques** (jeton, soldes CHF/EUR/USD, une page de
    transactions couvrant les familles de 38-8), écrit en constantes du module et que les E2E de 38-9a/38-9b demandent
    par `paypalResponses: "default"` (38-3b AC-3 ; sans variante, la connexion garde le `FakeConnector`, donc rien
    n'est « servi par défaut » sans demande) ;
    la route `_test` de 38-3b permet d'en servir d'autres (erreur 401, 403, 429, 500, `RESULTSET_TOO_LARGE`, réponse
    invalide). **Forme, décidée ici (validation P2)** : le corps de `POST /api/v1/_test/provider-connector` (38-3b,
    pensé pour le `FakeConnector` : soldes, pages de mouvements, `skipped`) gagne, par `connectionId`, une variante
    **additive** `paypalResponses: [{ method, path, query?, status, body }]` (JSON au format PayPal, servi dans
    l'ordre, puis la dernière réponse répétée) ; une connexion `paypal` **sans** cette variante garde le
    `FakeConnector` de 38-3b — les E2E de 38-9a/38-9b qui programment des mouvements ne changent pas —, une connexion
    qui la porte tourne le `PaypalConnector` sur le transport factice (forme portée par 38-3b AC-3, chaîne
    `"default"` comprise). Aucune route neuve : `LIB_ROUTES` et `TEST_ENDPOINT_ROUTES` sans entrée ajoutée (C-38-12 ; seul le
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
      `Auth` ; portée absente → `Permission` sans appel de données ; 429, 500 puis 200 (second essai
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
    les messages des sept codes d'API de l'AC-7 (clés `provider-error-*`), de `PROVIDER_FETCH_FROM_TOO_OLD` et le
    libellé d'audit, quatre locales
    (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`).

## Écart à l'epic

- **`accountEmail`** (E24, FR-38-7-4) : ni `GET /v1/reporting/balances` ni `GET /v1/reporting/transactions` ne
  rendent le courriel du compte *business* (relevé à la documentation ci-dessus : `account_id` et `account_number`
  seulement). L'obtenir demanderait l'API d'identité (`/v1/identity/openidconnect/userinfo`), d'une autre portée,
  non demandée à l'application en lecture seule (B12). **« Tester la connexion » pose `payerId` seulement.**
  **O10 (arbitrage de l'orchestrateur) : `accountEmail` est retiré du JSON** — `PaypalSettings` n'en porte pas
  (38-1 AC-4 et E24 à aligner : champ, test de refus d'un champ serveur fourni par le client, instantané d'audit).
  La reconnaissance d'un virement entre les deux comptes PayPal de Beat se fait par `payerId`
  (`payer_info.account_id` de la contrepartie, 38-8 ; *à constater sur les réponses de Beat*, Q-38-8-4).
  Q-38-7-1 **close**.
- **Limite de 10 000 résultats** : non chiffrée par la référence actuelle (voir *Source*) ; Kesh traite le code
  d'erreur et le seuil (AC-5).
- **Erreurs « en états lisibles »** : l'epic ne fixe ni codes ni statuts HTTP ; la table de l'AC-7 fixe les codes, et
  **aucun statut HTTP** (le passage est asynchrone, 38-3b).
- **Un appel de `movements` rend la fenêtre entière** (AC-4) : écart au `MovementPage` paginé de 38-3b (qui admet une
  page unique, `total_pages = 1`), justifié par
  le classement de 38-8 et le découpage de O10.

## Questions ouvertes

- ~~Q-38-7-1~~ close (O10 : `accountEmail` retiré). ~~Q-38-7-2~~ close (O10 : découpage dans le connecteur).
- **Q-38-7-3** (recette) : `page` commence-t-il à 1 (défaut et minimum de la référence) ou 0 (sa description) ? À
  constater sur les réponses de Beat avant la fixture ; le code n'en suppose qu'un, testé. Idem : forme du décalage
  des dates rendues (`Z`, `+00:00`, `+0000`).

## Contrats à amender chez 38-3b (texte proposé, cf. rapport de validation ; portés par la liste inter-fiches P1)

`Connector` gagne `history_start(now) -> Option<NaiveDate>` (AC-9), `overlap()` (38-8 AC-7),
`max_movements_per_page()` et `test()` (AC-8 ; portés à 38-3b et C-38-5 à la P2) ; `MovementPage` gagne
`skipped` et `sums` ; `ConnectorError` gagne `WindowTooLarge`, `Rejected`, `InvalidWindow` ; `HttpTransport` prend ses
bornes par requête ; le passage lit `code()` ; `last_error` porte le code d'API.

## Tâches

- [ ] Bornes PayPal passées au `HttpTransport` de 38-3b (AC-1) sur le constructeur paramétré du client de 37-2 ; tests
      de bornes.
- [ ] `kesh-providers::paypal` : `LIVE_BASE_URL`, `with_base_url`, `auth` (jeton, portée, cache, renouvellement sur
      401), `movements` (requête, validation, `WindowTooLarge`), `balances`, `Connector::test()`, `history_start`, `max_window`, `max_movements_per_page()` (10 000)
      (AC-2 à AC-6, AC-9).
- [ ] `ConnectorError` : trois variantes ajoutées, `code()` et `app_code()` (AC-7) ; messages `provider-error-*` et
      `PROVIDER_FETCH_FROM_TOO_OLD`, quatre locales.
- [ ] `AppState.provider_token_cache` et ses sites de construction ; champ du cache ajouté à `ConnectorContext` et à
      son constructeur unique (38-3b AC-3) ; vidage par `PUT /secrets`, `DELETE`,
      restauration, `clear_transient_test_state` (AC-3).
- [ ] Route `POST /provider-connections/{id}/test` (38-1) branchée sur PayPal ; écriture de `payerId` ; audit
      `payment_provider_connection.tested` (`audit_labels.rs`, quatre locales), registre `test_connection` → `Traced`
      (AC-8).
- [ ] Fonctions libres `paypal::history_start(now)` et `earliest_fetch_from(provider, now)` (`kesh-providers`) ;
      validation `fetchFrom` et champ `earliestFetchFrom` dans les routes de 38-1 ; borne au passage de 38-3b par
      `Connector::history_start` (AC-9).
- [ ] Réponses synthétiques par défaut du transport factice au format PayPal (AC-10).
- [ ] Fixtures anonymisées, relevé sur instance jetable, test « aucun courriel réel » ; tests hostiles ; tests de
      route ; test de non-fuite (AC-11, AC-12).
- [ ] Gate : `cargo fmt`, `clippy -D warnings`, `cargo nextest` complet (le crate `kesh-providers` et `kesh-api`),
      `cargo test --workspace` en série avant `done` ; `Cargo.lock` commité si une dépendance change.

## Dev Notes

- **Forme du trait** : objet-safe sans `async_trait`, futur boxé comme `Mailer` (`crates/kesh-api/src/mail/mod.rs:33`
  `MailFuture`, trait `:69`) et `RateSource` de 37-2 AC-3 ; porté par `AppState` (`lib.rs:84` `mailer`, `:98`
  `test_mock_mailer` pour le patron de la poignée de test).
- **Fuseau** : `transaction_initiation_date` est en UTC ; la date comptable (Europe/Zurich) est l'affaire de 38-3b
  (« Temps », AC-2 ; 38-8 AC-4 et AC-8 n'écrivent que l'instant UTC) ; `chrono-tz` arrive en dépendance directe de
  `kesh-api` par 37-2 (AC-4 de 37-2) et de `kesh-providers` par 38-3b AC-2 (0.10.4, déjà au `Cargo.lock`) ;
  `history_start` (AC-9) s'en sert dans `kesh-providers`.
- **Pourquoi `total_balance`** : `available + withheld (+ autres) = total` selon PayPal ; les blocages (T15xx, T21xx,
  écartés par 38-8) déplacent entre disponible et retenu sans changer le total — comparer au disponible ferait
  signaler un « téléchargement incomplet » à chaque blocage.
- **Pas de `deny_unknown_fields` sur les réponses** : PayPal ajoute des champs sans préavis ; la sévérité porte sur ce
  qu'on lit (AC-4), non sur ce qu'on ignore. À l'inverse, les **réglages** (E24) refusent l'inconnu.
- **Gabarit `<…>`** : un `clientId` de la forme `<…>` est refusé par la validation de 38-1 comme les autres gabarits
  (`is_template_placeholder`, `crates/kesh-api/src/config.rs:1538`) — à confirmer à la fiche 38-1, sinon ajouté ici.
- **Points de contact** : C-38-5 (`Connector`, transport, découpage, passage) et la fabrique de 38-3b ; C-38-2 (routes,
  `settings`, `version`, audit de 38-1) ; C-38-1 (`secrets::open`) ; 38-8 consomme `PaypalRecord` et `Balances`
  (noms fixés ici) ; 38-9a affiche les codes de l'AC-7, `earliestFetchFrom` et la date bornée du passage.
- **Splitting** : deux modules (`kesh-providers`, `kesh-api`), dans la règle (≤ 5).

### References

- `planning-artifacts/epic-38-paypal.md` § E2, E3, E9, E10, E20, E24, C-38-1, C-38-2, C-38-5, C-38-11, 38-7.
- `implementation-artifacts/37-2-cours-mensuels-afc.md` AC-3 (client borné, source objet-safe, mode test).
- Documentation PayPal citée en *Source* (consultée le 2026-11-15).

## Change Log

- 2026-11-15 — rédaction (Opus).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 1 HIGH, 6 MEDIUM, 3 LOW ; remédiés dans la fiche sauf les
  inter-fiches 38-3 (`Connector`, `MovementPage`, `ConnectorError`, bornes du transport, codes du passage, `last_error`),
  38-1 (`accountEmail`, O10) et 38-8 (contrat de classement par fenêtre).
- 2026-11-15 — alignement inter-fiches P1 (Opus) : AC-7 vérifié — aucun statut HTTP sur `POST …/fetch` (202, erreur
  lue à l'état du passage), rien à changer ; AC-1 : la contradiction 30 s/60 s est levée (bornes par requête du
  `HttpTransport` de 38-3b, valeurs PayPal ici) ; AC-9 : `truncated_from` nommé colonne de `provider_fetch_runs`
  (38-3a), rendue par `GET …/runs` (38-3b) ; découpage D-P1-1 : renvois à 38-3 portés sur 38-3b (crate, `Connector`,
  transport, fabrique, passage, déclenchement, transport factice, route `_test`) ou 38-3a (soldes vus, colonnes) ;
  dépendance à 38-3a ajoutée.
- 2026-11-15 — validation P2 (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 0 LOW ; remédiés dans la fiche sauf les textes à
  porter à 38-3b (P2-3, P2-4). P2-1 : `InvalidResponse { what, page, record_index?, field? }` (AC-4) contredisait
  `InvalidResponse { reason }` de 38-3b et l'AC-7 (« aucune variante renommée ») → `reason` composé de constantes ;
  P2-2 : un enregistrement hors de `[start, end)` dans la tolérance d'un jour passait à `validate_page` de 38-3b, qui
  rejette la page (`booked_at` hors fenêtre), et la frontière de l'AC-5 dépendait de la borne de `end_date` →
  retiré avant le classement ; P2-3 : unicité par devise ici, par page chez 38-3b → inter-fiche ; P2-4 : la route
  `_test` de 38-3b ne savait pas servir de réponses au format PayPal → variante additive `paypalResponses`,
  `FakeConnector` gardé sans elle. Code relu : `Cargo.toml:109`, `config.rs:1538`, `mail/mod.rs:33,69`,
  `lib.rs:61,84,98,119`, `main.rs:439`, `middleware/auth.rs:285`, `test_endpoints.rs:349`, `api-client.ts:6,393`
  (sous `frontend/src/lib/shared/utils/`), `audit_route_registry.rs:1009` — conformes ; documentation PayPal des
  codes d'événement relue (WebFetch).
- 2026-11-15 — alignement inter-fiches P2 (Opus) : AC-1 — délai de connexion 5 s posé sur le client, délai total
  60 s et corps ≤ 8 MiB par requête (`TransportLimits`), tests par `HttpTransport::for_local_tests()`, corps trop
  gros → `InvalidResponse` (`ResponseTooLarge` retiré) ; AC-3, AC-4, AC-6, AC-7 — `InvalidResponse { reason:
  &'static str }` (les `{ what: … }` remplacés ; page et rang de l'enregistrement, non constants, au `tracing::warn!`
  seulement) ; AC-8 — `PaypalConnector` implémente `Connector::test()` → `ConnectorTest { account_ref = account_id,
  balances }`, appelé par la route sur le connecteur de la fabrique ; contrats à 38-3b complétés
  (`max_movements_per_page()`, `test()`).
- 2026-11-15 — validation P3 (Sonnet) : 5 findings (0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW) ; tous remédiés dans la
  fiche ou, pour leur part commune, dans 38-3b (déjà alignée). MEDIUM : P3-1 AC-10 — les réponses « par défaut » ne
  pouvaient jamais être servies (sans `paypalResponses`, la connexion garde le `FakeConnector`) → demandées par
  `paypalResponses: "default"` (38-3b AC-3 amendée en conséquence) ; P3-2 AC-9 — la validation de `fetchFrom` à
  l'écriture appelait `Connector::history_start`, méthode d'instance, alors qu'une connexion peut être créée sans secret
  donc sans connecteur → fonction libre `paypal::history_start(now)` et `earliest_fetch_from(provider, now)`, une seule
  règle ; P3-3 AC-6 — `ProviderBalance.as_of` non défini : instant demandé ou rendu ? → `as_of_time` rendu (38-3b AC-7
  l'écrit tel quel pour que la somme du contrôle s'arrête où le solde s'arrête). LOW : P3-4 AC-3 — le champ du cache de
  jeton ajouté à `ConnectorContext` par 38-7 (38-3b ne référence plus un type d'une story suivante) ; P3-5 AC-8 —
  « 38-3b AC-1 » → AC-2 (le trait est à l'AC-2) et AC-4, `MovementPage` écrit avec `sums` (« gagne » périmé). Lignes
  relues et conformes : `Cargo.toml:106-109` (`reqwest` en `[dev-dependencies]`) ; `Cargo.lock` (`base64` 0.22.1 et
  0.23.1, `zeroize` 1.9.1) ; `lib.rs:61, 84, 98, 119` ; `main.rs:439` ; `middleware/auth.rs:285` ;
  `middleware/rbac.rs:58` ; `mail/mod.rs:33, 69` ; `config.rs:1538` ; `audit_route_registry.rs:1009` ; `api-client.ts:6,
  393-394` ; sites `AppState {` du `grep` (tous présents ; `tests/common/mcp.rs:215` est `fn etat`, qui passe par
  `new_for_tests` — l'inventaire reste au `grep`). Axes non exercés : documentation PayPal non relue en P3 ; API
  `reqwest` de mémoire ; aucune exécution.
- 2026-11-15 — validation P4 ciblée (Opus) : 5 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW), braqués sur la
  remédiation P3 (`git diff f005ba2f a5682b2e`) ; tous remédiés dans la fiche. MEDIUM : P4-1 AC-9 — « c'est aussi ce
  que lisent `earliestFetchFrom` et le passage » contredisait 38-3b AC-4, où le passage lit `Connector::history_start`
  du connecteur (et le `FakeConnector` le sien, programmé) → le passage lit la méthode du trait, la route
  `earliest_fetch_from` (placée dans `kesh_providers`), même fonction libre pour PayPal ; P4-2 AC-9 et Dev Notes —
  la fonction libre `kesh_providers::paypal::history_start` s'appuyait sur `aujourdhui_zurich` de 37-2, qui vit dans
  `kesh-api` (`37-2-cours-mensuels-afc.md:586`), que `kesh-providers` ne peut appeler → calcul par `chrono-tz` dans
  `kesh-providers` (38-3b AC-2, « Temps ») ; la note « Fuseau » attribuait encore la date comptable à 38-8 → 38-3b.
  LOW : P4-3 AC-6 — `as_of_time` postérieur admis (≤ 1 jour) : son effet, une fois rendu en `as_of`, dit (contrôle
  « non disponible » pour ce relevé, sans faux écart) ; P4-4 renvois périmés « (à y écrire) » (AC-4), « texte à
  aligner chez 38-3b » (AC-4) et « texte à porter à 38-3b AC-3 » (AC-10, phrase « Aucune route neuve » doublée)
  → renvois aux AC de 38-3b qui les portent ; P4-5 Tâches — le champ de cache de `ConnectorContext` et les deux
  fonctions libres ajoutés. Vérifié conforme : 38-3b AC-2 (`MovementPage { …, skipped, sums }`, trait `Connector`
  avec `test()`), AC-3 (`ConnectorContext` sans le cache, ajouté par 38-7 ; `paypalResponses` ou `"default"`), AC-7
  (`as_of` rendu écrit aux instantanés) ; 38-8 AC-1 (`Classified` rend `skipped` et `sums`). Axes non exercés :
  documentation PayPal (`as_of_time`) non relue (aucun réseau) ; reste de la fiche hors remédiation ; aucune
  exécution.
