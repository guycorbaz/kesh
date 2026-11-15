# Story 38-3b : téléchargement — crate `kesh-providers`, transport, passage, planification et contrôle des soldes

Status: ready-for-dev

*Fiche née le 2026-11-15 du découpage de 38-3 (décision **D-P1-1** de l'orchestrateur, `38-p1-inter-fiches.md` :
`38-3-telechargement-cadre.md` touchait six modules). Branche `epic/38-paypal`, numéros de ligne relevés au commit
`8b3d548d` (repris de 38-3). Epic 38 : E5 (écriture de l'en-tête), E6, E9, E10, E12, E20 (a) ; contrats **C-38-5**,
C-38-11, C-38-12 ; décisions de Beat B2, B8, B9, B12, B20 ; arbitrages O7, O10, O11, O12. **Dépend de** : 38-1
(connexions, coffre, voie unique d'`open`), 38-2 (comptes prestataires), **38-3a** (schéma, entités, gardes), l'Epic 37
(client HTTP borné et patron « source objet-safe + factice en mode test + route `_test` » de 37-2 AC-3, C-37-6).
**Suivie par** : 38-7 (connecteur PayPal réel), 38-8 (classement des codes PayPal), 38-9a (écrans), 38-6a/b/c
(transactions téléchargées). **Aucune migration** (le schéma est en 38-3a).*

*Modules : `kesh-providers` (neuf), `kesh-api` (transport, fabrique, passage, planification, routes), `kesh-db`
(écritures des passages, soldes, transactions de prestataire), `kesh-i18n` — quatre (≤ 5). Hors crates : `Cargo.toml`
racine, `scripts/prepare-release.sh` (commentaires), `CLAUDE.md`.*

## Story

En tant que **Comptable** ou **Administrateur** d'une société qui tient ses soldes PayPal dans Kesh, je veux que Kesh
**télécharge** les mouvements de chaque connexion — chaque jour si elle est réglée ainsi, et à la demande dans tous
les cas — depuis la date « télécharger depuis », **sans doublon** même quand les fenêtres se recouvrent, qu'il
enregistre chaque passage, **liste les soldes de devise qui n'ont pas encore de compte** sans rien créer, et me
signale un téléchargement incomplet en comparant les soldes rapportés aux mouvements reçus, afin que les propositions
d'imputation (38-6) reposent sur un relevé complet et vérifiable.

## Exigences

| Exigence | Texte | AC | Origine (38-3) |
|---|---|---|---|
| `FR-38-3b-1` | Télécharger les mouvements d'une connexion depuis `fetch_from` (borné par l'historique du prestataire), chaque jour (`daily`) ou à la demande (`POST …/fetch`, Comptable ou Administrateur), dans les deux réglages. | AC-4, AC-5 | FR-38-3-1 |
| `FR-38-3b-2` | Sans doublon : insertion idempotente décidée par lecture préalable des identifiants connus, recouvrement `overlap()` du connecteur (`overlapDays`, défaut 3 jours). | AC-4 | FR-38-3-2 |
| `FR-38-3b-3` | Chaque passage est enregistré (fenêtre, début borné `truncated_from`, compteurs insérés, connus, non liés, écartés, erreur) ; chaque compte touché reçoit un en-tête `bank_imports` en `PROVIDER_API`, créé à la première insertion réelle. | AC-4, AC-6 | FR-38-3-3 |
| `FR-38-3b-4` | Un solde de devise sans compte Kesh est listé et proposé (B2), ignorable ; ses transactions ne se téléchargent pas. | AC-7 | FR-38-3-4 |
| `FR-38-3b-5` | Les soldes rapportés s'enregistrent ; un téléchargement incomplet est signalé avec l'écart (E20 a). | AC-7 | FR-38-3-5 |
| `FR-38-3b-6` | Le Comptable lit l'état des connexions (sans réglages ni secrets) et l'historique des passages (O7). | AC-6 | FR-38-3-8 |
| `NFR-38-3b-1` | Un passage à la fois par connexion ; un échec garde l'inséré et dit où reprendre. | AC-4 | NFR-38-3-3 |
| `NFR-38-3b-2` | Aucun appel réseau en CI ; transport factice piloté par une route `_test`, inactif hors `KESH_TEST_MODE`. | AC-3, AC-5 | NFR-38-3-4 |
| `NFR-38-3b-3` | Toute réponse d'un prestataire est validée avant écriture ; une page invalide n'écrit rien et fait échouer le passage avec un état lisible (R5 de 37, C-38-11). | AC-2 | NFR-38-3-5 |
| `NFR-38-3b-4` | La date comptable d'un mouvement est sa date à Europe/Zurich ; l'instant reçu est gardé en UTC. | AC-2, AC-4 | NFR-38-3-6 |
| `NFR-38-3b-5` | Bornes du transport **par requête** (connexion, délai total, taille du corps) ; ni le texte d'une erreur `reqwest` ni un secret dans un état, une réponse ou un journal. | AC-2, AC-3 | AC-4/AC-5 de 38-3, inter-fiches |

## Critères d'acceptation

1. **AC-1 — Prérequis, crate et workspace.** 38-3a est fusionnée (schéma, entités) et la version minimale relevée par
   38-2 ; sinon arrêt. **Crate `kesh-providers`** : né **après** le bump de 38-2, il porte la version courante des
   crates (`version = "<X.Y.Z>"` de 38-2) ; le workspace passe à **onze** membres : `members` du `Cargo.toml` racine
   (`:2-13`), « 10 crates » / « dix » de `scripts/prepare-release.sh` (commentaires `:7`, `:219`, `:271` et le
   **message affiché** `:356` ; le script lit `crates/*/Cargo.toml` par motif, `:96`, `:280`, et ne compte rien) et
   `CLAUDE.md` § *Repository Structure* (`:60`, « workspace de 10 crates », et une ligne `kesh-providers/` dans
   l'arbre qui suit). Grep de la valeur au Change Log (`grep -rnE "\b(10|dix) crates" --include=*.md --include=*.sh
   .` hors `_bmad-output/` et CHANGELOG ; `docs/agents-orchestration.md:88` est un relevé daté, laissé tel quel). Un test relit que les onze `Cargo.toml` portent la même version (P2-bis : un crate en
   retard rend le binaire plus ancien que sa base). **Aucune migration** : ni P5, ni P6, ni P7 ne s'appliquent.

2. **AC-2 — Crate `kesh-providers` : types, contrat du connecteur, validation, temps (E9, C-38-11, NFR-38-3b-3/4/5).**
   `publish = false`, **sans dépendance à `kesh-db`** ni à `kesh-api` (dépend de `kesh-core` pour les réglages, 38-1) ;
   dépendances `chrono`, `chrono-tz` (déjà au `Cargo.lock`, 0.10.4, `:508-515` — à déclarer), `rust_decimal`, `serde`,
   `serde_json`, `thiserror`.
   - **Types** : `MovementKind { Sale, Purchase, Fee, Conversion, Refund, Dispute, Withdrawal, Deposit, Other(String) }`
     et `ReviewReason { Dispute, RefundOriginMissing, UnknownEvent, Other(String) }` (C-38-4, `as_str` / `FromStr`
     tolérant ; `refund_origin_missing` est calculé par 38-6b, jamais rendu par un connecteur, O11) ;
     `ProviderMovement { provider_tx_id, booked_at: DateTime<Utc>, currency, gross, fee, net, movement_kind,
     event_code, counterparty_name, counterparty_email, invoice_ref, item_names, reference, details,
     related_provider_tx_id, review_reason }` ; `ProviderBalance { currency, amount, as_of }` ; `Window { from, to }`
     (UTC, demi-ouverte `[from, to)`).
   - **Trait objet-safe** `Connector` (forme de `RateSource` de 37-2 AC-3 : `Pin<Box<dyn Future<…> + Send + '_>>`, sans
     `async_trait`) :
     - `balances(&self, as_of: Option<DateTime<Utc>>) -> … Result<Vec<ProviderBalance>, ConnectorError>` ;
     - `movements(&self, window: Window, page: u32) -> … Result<MovementPage, ConnectorError>` ; **`MovementPage {
       movements, page, total_pages, skipped: SkippedCounts { denied, pending, non_balance }, sums: Vec<(String,
       Decimal)> }`** — `sums` = Σ `net` par devise des mouvements rendus ; **une page unique est admise**
       (`total_pages = 1` : PayPal lit et classe la fenêtre entière, 38-7 AC-4, 38-8 AC-1) ;
     - `max_window() -> Duration` (PayPal : 31 jours ; une fenêtre plus large que le prestataire ne sait servir se
       **redécoupe dans le connecteur**, O10 — le passage ne découpe qu'en sous-fenêtres de `max_window()`) ;
     - **`history_start(now) -> Option<NaiveDate>`** (date la plus ancienne que le prestataire sert ; PayPal : 38-7
       AC-9 ; `None` = sans limite) ;
     - **`overlap() -> Duration`** (recouvrement de relecture ; PayPal : `overlapDays` des réglages, 1 à 30, défaut 3,
       38-8 AC-7, O11 ; le connecteur factice : 3 jours) ;
     - **`max_movements_per_page() -> u32`** (plafond **déclaré par le connecteur**, lu par `validate_page`, jamais lu
       de la réponse ; PayPal : 10 000, 38-7 AC-4 ; le factice : 10 000) ;
     - **`test(&self) -> … Result<ConnectorTest, ConnectorError>`** — la **forme générique** de « Tester la
       connexion » (38-7 AC-8) : `ConnectorTest { account_ref: Option<String>, balances: Vec<ProviderBalance> }`
       (PayPal : `account_ref` = `account_id` → `payerId`, soldes sans `as_of`). *Raison* : la fabrique rend un `Box<dyn
       Connector>` ; une méthode propre à `PaypalConnector` ne serait pas appelable par la route `POST …/test` sans
       transtypage. Le factice rend ce qu'on lui programme.
     Trait objet-safe **`Transport`** (requête HTTP minimale **avec ses bornes** → réponse bornée), dont
     l'implémentation `reqwest` vit dans `kesh-api` (AC-3).
   - **`ConnectorError`** : `Auth`, `Permission`, `RateLimited { retry_after }`, `Unavailable`, `InvalidResponse {
     reason: &'static str }` (motif **constant** — le champ ou la règle en défaut —, jamais une valeur lue de la
     réponse : il finit dans `error_detail`, que le Comptable lit), `Transport { kind }`, **`WindowTooLarge { total_items: Option<u32> }`**, **`Rejected`**,
     **`InvalidWindow`** (38-7 AC-7) — codes constants, **jamais** le texte d'une erreur `reqwest` (qui porte l'URL,
     précédent 37-2) ni un secret. **Une seule** `code()` (code du passage : `auth`, `permission`, `rate_limited`,
     `unavailable`, `invalid_response`, `transport`, `window_too_large`, `rejected`, `invalid_window`) et **une seule**
     `app_code()` (code d'API : table de 38-7 AC-7 — `PROVIDER_AUTH_FAILED`, `PROVIDER_PERMISSION_MISSING`,
     `PROVIDER_RATE_LIMITED`, `PROVIDER_UNAVAILABLE`, `PROVIDER_RESPONSE_INVALID`, `PROVIDER_WINDOW_TOO_LARGE`,
     `PROVIDER_REQUEST_REJECTED` ; `InvalidWindow`, erreur de programmation de l'appelant → `PROVIDER_RESPONSE_INVALID`
     et `tracing::error!`). Test : chaque variante a son couple de codes.
   - **Validation commune** `validate_page(page, window, max_movements)` (`max_movements` = `max_movements_per_page()`
     du connecteur) : `provider_tx_id` non vide, ≤ 64, ASCII imprimable ; devise `^[A-Z]{3}$` (une devise bien formée
     **sans compte** n'est pas une réponse invalide : elle se compte `skipped_unlinked`, AC-4 ; aucune liste de devises
     « attendues » n'est passée) ; montants à deux décimales au plus et `|montant| < 10^16` ;
     `net == gross + fee` ; `booked_at` dans la fenêtre ; textes bornés aux longueurs des colonnes de 38-3a (refus,
     jamais de troncature silencieuse d'un identifiant ; troncature **signalée** des textes libres `details` /
     `counterparty_name` à leur borne) ; `item_names` ≤ 50 noms de ≤ 255 ; au plus `max_movements` mouvements ;
     pagination cohérente (`page ≤ total_pages`, `total_pages` constant d'une page à l'autre d'une même fenêtre) ; pas
     deux fois le même couple **`(provider_tx_id, currency)`** dans une page (unicité du couple, non de l'identifiant
     seul : le doublon se juge par compte, donc par devise — index `(bank_account_id, provider_tx_id)` de 38-3a ; un
     même identifiant dans deux devises vise deux comptes) ; **`sums` égale la somme recalculée** par devise. Échec →
     `InvalidResponse { reason }`, **aucune ligne de la page écrite**. Tests de réponses **hostiles** (C-38-11) : chacun
     des refus ci-dessus a son test.
   - **Temps** : `booking_date = booked_at` converti en `Europe/Zurich` (`chrono-tz`), date seule ; tests à 23 h 30 UTC
     le 31 décembre et aux deux changements d'heure.
   - **`FakeConnector`** (`#[doc(hidden)] pub`, précédent `MockRateSource`, 37-2 AC-3 — visible des tests d'intégration
     de `kesh-api`) : soldes et mouvements programmés par fenêtre, `skipped`, `history_start`, `overlap`, erreurs et
     pages incohérentes à la demande. **Aucun connecteur réel ici** : `paypal` arrive en 38-7 ; en 38-3b, une connexion
     `paypal` hors mode test termine son passage en `failed`, `error_code = "connector_unavailable"`.

3. **AC-3 — Transport, fabrique et mode test (E9, E10, NFR-38-3b-2, NFR-38-3b-5).**
   - `kesh-api::providers::HttpTransport` implémente `Transport` en **réutilisant le constructeur du client borné de
     37-2** (rustls, `redirect(Policy::none())`, pas de magasin de cookies, `User-Agent: Kesh/<version>`) ; **bornes**
     (`TransportLimits { total_timeout, max_body }`, passées par le connecteur **avec chaque requête** : délai par
     `RequestBuilder::timeout`, corps lu par morceaux borné — `Content-Length` contrôlé puis lecture bornée,
     dépassement → `InvalidResponse`) ; défaut 30 s, 8 MiB pour un connecteur qui n'en déclare pas ; PayPal : **60 s**,
     8 MiB (38-7 AC-1). Le délai de **connexion** (5 s, valeur de 37-2 et de PayPal) est une borne **du client**, posée
     à sa construction : `reqwest` n'a pas de `connect_timeout` par requête (`ClientBuilder` seulement) — il n'y a plus
     de valeur unique contradictoire. Deux constructeurs : `HttpTransport::new()` (production, `https_only(true)`) et
     `HttpTransport::for_local_tests()` `#[doc(hidden)] pub` (HTTP admis, pour un serveur axum local ; patron de 37-2,
     où `https_only` vaut « pour l'URL de production »). L'URL de base est une **constante du connecteur**, aucune
     variable d'environnement ne la change ; sa substitution de test est `PaypalConnector::with_base_url` (38-7 AC-2),
     non une méthode du transport.
   - `AppState` gagne `connector_factory: Arc<dyn ConnectorFactory>` et `provider_fetch_guard` (AC-4). La fabrique
     construit le connecteur d'une connexion à partir d'un **`ConnectorContext`** : prestataire, réglages typés de
     38-1, **secret ouvert** (`Zeroizing<String>`, par la **voie unique** de 38-1 AC-1 — la garde des appelants
     d'`open` gagne **ce seul site**), **`secret_updated_at`**, **`other_payer_ids: Vec<String>`** (les `payerId` des
     autres connexions PayPal de la société, lus par la fabrique dans leurs réglages typés ; vide quand il n'y en a
     pas — consommés par 38-8 AC-5) et transport. **Le cache de jeton n'est pas posé ici** : son type
     (`PaypalTokenCache`) est de `kesh_providers::paypal::auth` (38-7 AC-3), qui ajoute lui-même le champ à
     `ConnectorContext` et à `AppState` (`ConnectorContext` et le trait `ConnectorFactory` vivent dans
     `kesh-api::providers`, à côté de la fabrique — `kesh-api` voit les types de `kesh-providers`, l'inverse non ;
     construit par un **constructeur unique**, seul site que l'ajout d'un champ touche) — 38-3b ne
     référence aucun type d'une story suivante. La route `POST …/test` (38-1 AC-7) construit son connecteur par la même fabrique (38-7). Sites de
     construction littérale d'`AppState` : inventaire de 37-2 AC-3, à relancer (`grep -rn "AppState {"`).
   - **`KESH_TEST_MODE`** : `main.rs` installe une fabrique qui rend un `FakeConnector` pour **toute** connexion,
     piloté par `POST /api/v1/_test/provider-connector` (corps : soldes, pages de mouvements par fenêtre, `skipped`,
     erreur ou délai à servir, par `connectionId`), monté avec les autres routes `_test` (`lib.rs:1434`,
     `routes/test_endpoints.rs:49-55`) ; `TEST_ENDPOINT_ROUTES` **+1** (`Exempt("chemin de test…")`). 38-7 remplace ce
     factice, pour `paypal`, par un **transport** factice au format PayPal (38-7 AC-10), programmé par une **variante
     additive** du corps de la même route : `paypalResponses: [{ method, path, query?, status, body }]` (par
     `connectionId`) — réponses servies **dans l'ordre**, la dernière se répète — **ou la chaîne `"default"`** (le jeu de
     réponses synthétiques par défaut de 38-7 AC-10, qui sinon ne servirait jamais : sans la variante, une connexion
     garde le `FakeConnector`) ; quand elle est fournie, la fabrique de test construit un vrai `PaypalConnector` sur ce
     transport factice au lieu du `FakeConnector` ; sans elle, comportement ci-dessus. Le corps est défini ici (forme et validation, champs facultatifs), le connecteur qui le lit
     en 38-7 ; **validation posée ici** : `paypalResponses` est soit un tableau non vide d'objets `{ method ∈ GET|POST,
     path commençant par "/", query? objet de chaînes, status 100 à 599, body JSON }`, soit **exactement** la chaîne
     `"default"` — toute autre chaîne ou forme → `400` (`AppError::Validation`, patron de
     `routes/test_endpoints.rs:391-395`), rien de programmé ; **en 38-3b**, faute de `PaypalConnector`, la variante est
     validée et gardée dans l'état programmé, et la fabrique de test rend encore le `FakeConnector` (38-7 branche la
     variante). Tests : `"default"` admis, `"autre"` → 400, tableau vide ou `status` hors bornes → 400 ; **`TEST_ENDPOINT_ROUTES` inchangé** par cette variante (même route). L'état programmé vit dans une
     poignée d'`AppState` (`test_provider_connector`, patron de `test_rate_source` de 37-2) et se vide par
     `clear_transient_test_state` (`routes/test_endpoints.rs:349`, appelé par `/_test/reset`). La tâche quotidienne
     **n'est pas lancée** en mode test (AC-5). Aucun appel réseau en CI : seul `HttpTransport` ouvre une connexion, et
     aucun test ne construit `HttpTransport::new()` (garde de source : seuls `main.rs` et la fabrique de production
     l'appellent).
   - **Tests** (serveur axum local, `for_local_tests`) : délai total dépassé → `Unavailable` ; `Content-Length` au-delà
     de `max_body` → `InvalidResponse` sans lire le corps ; corps sans `Content-Length` qui dépasse → lecture arrêtée,
     `InvalidResponse` ; `302` → refusé (`Transport { kind }`, redirection non suivie) ; pour chaque variante,
     `ConnectorError::to_string()` et `Debug` ne contiennent ni l'URL ni un en-tête `Authorization` (NFR-38-3b-5) ;
     `HttpTransport::new()` refuse une URL `http://`.

4. **AC-4 — Le passage (C-38-5, E5, E6, FR-38-3b-1/2/3, NFR-38-3b-1).** Module `kesh-api::providers::fetch`, fonction
   `run_fetch(state, connection_id, trigger, user)` :
   - **Un passage à la fois par connexion** : verrou en mémoire (`provider_fetch_guard`, ensemble des connexions en
     cours, `tokio::sync::Mutex`) **et** en base — la ligne `provider_fetch_runs` `running` se crée sous `SELECT … FOR
     UPDATE` de la connexion, refusée s'il en existe une **que le verrou en mémoire tient, ou qui a commencé il y a
     moins de 6 h** (`409 PROVIDER_FETCH_IN_PROGRESS`, `details.runId`) ; une ligne `running` plus ancienne et sans
     verrou en mémoire passe `failed`, `error_code = "interrupted"` à cette lecture. *Raison* : une sauvegarde importée
     peut porter un passage `running` d'une autre instance, que ni le démarrage ni le verrou en mémoire ne verraient. Au
     démarrage du binaire (`main.rs` seulement), tout passage resté `running` passe de même `failed`, `error_code =
     "interrupted"` (`tracing::warn!`). Test : ligne `running` de 7 h posée par SQL → un passage démarre ; de 1 h → 409.
     **Libération du verrou en mémoire** : par une garde RAII (`Drop`) prise avant le `tokio::spawn`, qui retire la
     connexion de l'ensemble **même si la tâche panique** ; la tâche du passage est elle-même lancée dans un
     `tokio::spawn` dont l'issue est attendue (patron d'`auto_backup.rs:795-797`) : une fin anormale (panique) passe la
     ligne `failed`, `error_code = "interrupted"`. Sans cela, une panique laisserait la connexion tenue par le verrou en
     mémoire, donc refusée en 409 jusqu'au redémarrage. Test : un factice qui panique → ligne `failed`/`interrupted`,
     passage suivant admis.
   - **Préalables** : connexion utilisable (38-1 AC-3 : prestataire connu, réglages lisibles, clé, secret) — sinon `409
     PROVIDER_CONNECTION_UNUSABLE`, `details.reason` = `unusableReason` de 38-1 ; aucun passage créé.
   - **Fenêtre** : fin = maintenant − 3 h (délai de PayPal) ; début = **min sur les comptes prestataires actifs de la
     connexion** (non archivés, non clos) de `max(fetch_from à 00:00 Europe/Zurich, fetched_through − overlap())`
     (`fetched_through` `NULL` → `fetch_from`) ; puis **`max(début, history_start(now) à 00:00 Zurich)`** — s'il borne,
     le début effectif s'écrit à `truncated_from` (affiché par 38-9a ; jamais d'appel avant `history_start`, 38-7
     AC-9) ; découpée en sous-fenêtres de `max_window()` au plus. Aucun compte actif → passage `succeeded` sans
     mouvement (soldes lus quand même, AC-7). **Fenêtre vide** (début ≥ fin : `fetch_from` du jour ou du lendemain,
     que 38-1 admet — « postérieure au lendemain » seule est refusée —, la fin étant *maintenant − 3 h*) → passage
     `succeeded` sans appel de `movements` (38-7 AC-4 refuserait `start ≥ end` en `InvalidWindow`) ; ni `opening` ni
     `reported` lus tant que `fetch_from` à 00:00 Zurich n'est pas antérieur à la fin (AC-7). Test : connexion créée
     avec `fetchFrom` = demain → `succeeded`, aucun appel au factice.
   - **Comptes élus** : l'ensemble des comptes prestataires **actifs** de la connexion lu au début du passage (sous le
     verrou de la connexion). Ce sont eux, et eux seuls, dont `fetched_through` avance (ci-dessous) — **qu'ils aient
     reçu un mouvement ou non** : un compte sans activité qui n'avancerait pas maintiendrait le début de fenêtre de
     toute la connexion à `fetch_from`, chaque jour. Une devise dont le compte est **archivé ou clos** n'est pas
     élue : ses mouvements se comptent `skipped_unlinked` et ne s'écrivent pas, mais elle **n'entre pas** à
     `provider_balances_seen` (sa devise est tenue par un compte : « Créer le compte » échouerait en
     `PROVIDER_ACCOUNT_DUPLICATE_CURRENCY`, 38-2).
   - **Pour chaque sous-fenêtre et chaque page** : `movements(window, page)` → `validate_page` (AC-2) → **une
     transaction** : `ignored_count += skipped.denied + skipped.pending + skipped.non_balance` ; pour chaque mouvement,
     compte bancaire = la ligne `payment_provider_accounts` (connexion, devise) d'un compte **élu** ; devise sans compte élu →
     compté `skipped_unlinked` et **non écrit** (E12, B2) ; un compte **archivé ou clos** entre la sélection et
     l'écriture est refusé par la même garde que `create_with_transactions` (`repositories/bank_imports.rs:~40-66`,
     Story 28-4a/29-4a : compte relu sous verrou, `NotFound` ou `BankAccountClosed`) — la transaction de la page est
     annulée, le passage se termine `failed`, `error_code = "account_unavailable"` ; le suivant n'élit que les comptes
     actifs. **Connu ou inséré se décide par une lecture, non par `rows_affected`** : sqlx pose `CLIENT_FOUND_ROWS`
     (`repositories/letterings.rs:1213`), si bien qu'un `ON DUPLICATE KEY UPDATE id = id` rend **1** pour une ligne
     insérée **et** pour une ligne connue. Par page et par compte : `SELECT provider_tx_id FROM bank_transactions WHERE
     bank_account_id = ? AND provider_tx_id IN (…)` (par lots bornés, même transaction, verrou de passage tenu), puis
     `INSERT` des seuls identifiants absents ; le `ON DUPLICATE KEY UPDATE id = id` (sur
     `uq_bank_transactions_provider_tx`) ne reste que comme filet d'une course impossible sous le verrou — un mouvement
     connu n'est **jamais réécrit** (E6). Test : la même page insérée deux fois → `inserted_count` et `known_count`
     justes. En-tête `bank_imports` du (passage, compte) **créé à la première insertion réelle** — une page entièrement
     connue n'en crée aucun ; l'en-tête déjà créé par une page précédente du même passage est réutilisé (E5 :
     `source_format = 'PROVIDER_API'`, `filename = telechargement-<provider>-<AAAA-MM-JJ>-<AAAA-MM-JJ>` (bornes de la
     fenêtre du passage, dates Zurich), `file_hash` provisoire = SHA-256 de `run:<id>:account:<id>`, `statement_id`,
     soldes `NULL`, période = fenêtre, `imported_by_user_id` = le demandeur, ou `created_by_user_id` de la connexion
     pour `daily` (colonne `NOT NULL`, `migrations/20260504000001_bank_imports.sql:33` ; un utilisateur désactivé reste
     une ligne valide), `provider_fetch_run_id`) ; `status = 'pending'`. **Correspondance** `ProviderMovement` →
     `bank_transactions` (E7, E8 ; une seule fonction, testée) : `amount` = `net` ; `currency` = `currency` ;
     `booking_date` = date Zurich de `booked_at` ; `value_date` `NULL` ; `reference` = `reference` ; `details` =
     `details`, **chaîne vide** si absent (colonne `TEXT NOT NULL`, `20260504000001_bank_imports.sql:64`) ;
     `counterparty_name` = `counterparty_name` ; `transaction_id`, `end_to_end_id`, `counterparty_iban` `NULL` (le
     doublon se lit sur `provider_tx_id`, E6) ; colonnes de 38-3a AC-3 (`provider_tx_id`, `booked_at`, `gross_amount`,
     `fee_amount`, `movement_kind`, `event_code`, `counterparty_email`, `invoice_ref`, `item_names` en tableau JSON,
     `related_provider_tx_id`, `review_reason`) remplies telles que rendues. `transaction_count` de l'en-tête est
     **incrémenté dans la transaction de chaque page** (il dit vrai à tout instant, échec compris). À la fin de la
     sous-fenêtre (toutes pages) : `fetched_through` de chaque **compte élu** passe à `max(fetched_through, fin de la
     sous-fenêtre)` (jamais en arrière : les premières sous-fenêtres d'un passage rattrapant un compte neuf précèdent
     le point des autres), **dans la transaction de la dernière page** — ou, quand la fenêtre n'a aucune page
     (`total_pages = 0`), dans une transaction à lui seul ; point de reprise = ce champ. Un interblocage (1213) d'une
     page se **rejoue** (`kesh_db::retry::retry_on_deadlock`, `kesh-db/src/retry.rs:151`, la page étant relue des
     identifiants connus) ; au-delà, échec
     `database`. Test : deux comptes, l'un sans aucun mouvement → les deux avancent ; compte neuf ajouté à une
     connexion déjà à jour → l'ancien compte ne recule pas.
   - **Course avec le retrait des réglages** (38-2 AC-6 (c), rejouée ici contre l'insertion d'un passage, comme 38-3a
     AC-6 la rejoue contre un import) : `DELETE …/accounts/{bankAccountId}` concurrent de l'écriture d'une page sur ce
     compte — le retrait voit les transactions insérées (`PROVIDER_ACCOUNT_IN_USE`, raison des transactions de
     prestataire, 38-2) ou la page attend son verrou puis, relisant sous ce verrou la ligne de réglages
     (`is_provider_account_in_tx`, 38-2 AC-2, à côté de la garde d'archivage ci-dessus) et ne la trouvant plus, annule
     sa transaction et se termine comme un compte devenu indisponible (`account_unavailable`) ; **jamais** une transaction de prestataire écrite sur un compte qui
     n'a plus de réglages. Test de course au patron de 38-2 AC-6 (attente sur le texte du verrou), les deux ordres.
   - **Fin** — et de même à l'**échec**, dans la transaction qui écrit l'état `failed` : en-têtes finalisés (`file_hash`
     = SHA-256 des enregistrements normalisés **de l'en-tête**, relus en base par `import_id`, triés par
     `provider_tx_id` — forme écrite au module ; la période reste la fenêtre du passage), passage `succeeded` ou
     `failed` ; aucun en-tête ne garde l'empreinte provisoire (test : échec à la
     page 3 → en-tête au compte et à l'empreinte des seules lignes commitées) ; `payment_provider_connections.
     last_run_at`, `last_run_status`, `last_error` mis à jour (38-1 AC-2).
   - **Échec** (erreur du connecteur, page invalide, base) : ce qui est commité **reste** ; passage `failed`,
     **`error_code = err.code()`** pour une erreur du connecteur (AC-2), sinon un code propre du passage
     (`connector_unavailable`, `interrupted`, `account_unavailable`, `database`) ; `error_detail` sans secret ;
     **`last_error` de la connexion porte le code d'API** (`err.app_code()` ; codes propres du passage :
     `PROVIDER_CONNECTOR_UNAVAILABLE`, `PROVIDER_FETCH_INTERRUPTED`, `PROVIDER_ACCOUNT_UNAVAILABLE`,
     `PROVIDER_FETCH_FAILED` pour `database`) ; `fetched_through` dit où reprendre (−`overlap()`). Test : échec à la
     page 3 de la 2ᵉ sous-fenêtre → pages 1-2 gardées, relance → aucun doublon, total final égal à un passage sans
     échec.
   - **Statut tardif** : un mouvement déjà connu n'est jamais modifié ; une inversion ou un remboursement arrive comme
     une **transaction distincte** (codes `T11xx`, 38-8) avec son propre identifiant et `related_provider_tx_id` — c'est
     elle que 38-6b traite. Test : un mouvement renvoyé avec d'autres montants sous le même identifiant laisse la ligne
     intacte et se compte `known` (un `tracing::warn!` sans montant).
   - **Volume** : trois ans × trois devises (quelques milliers de mouvements) : une transaction par page ; test de
     charge synthétique (≥ 5 000 mouvements, 40 sous-fenêtres) dans un délai borné écrit au Change Log.

5. **AC-5 — Planification et déclenchement (E10, B9).**
   - **Tâche quotidienne** `providers::schedule::spawn(state)` lancée par `main.rs` **seulement**, à côté de
     `auto_backup::spawn` (`main.rs:466`, patron `auto_backup.rs:788-803` : `interval_at`, `MissedTickBehavior::Skip`,
     un tour par `tokio::spawn`), **jamais en `KESH_TEST_MODE`** ; garde lexicale au patron de
     `auto_backup::tests::lancement_par_main_seulement`. Cadence : toutes les heures ; une connexion `daily` utilisable
     dont le dernier passage (tout déclencheur) a commencé il y a plus de 24 h est passée, l'une après l'autre. Passage
     automatique **non audité** (`tracing` + état du passage), C-37-6 ; erreurs journalisées sans secret.
   - **À la demande** : `POST /api/v1/provider-connections/{id}/fetch` — **Comptable ou Administrateur, session
     seule** : dans `comptable_routes`, refus d'une clé d'API dans le handler par `ApiKeyOperationForbidden` (patron de
     `bank_imports::delete`, `routes/bank_imports.rs:1681-1690` ; NFR-38-6c1-4) ; vaut pour `daily` **et** `manual`
     (B9). Réponse **`202`** `{ runId }` : le passage court dans un `tokio::spawn` ; refus synchrones : `404`, `409
     PROVIDER_CONNECTION_UNUSABLE`, `409 PROVIDER_FETCH_IN_PROGRESS`. **Aucune erreur du connecteur n'est un statut
     HTTP** de cette route (38-7 AC-7) : elle se lit à l'état du passage. **Audit** : à la fin du passage, dans la
     transaction qui écrit son état, `provider_fetch.completed` (entité `provider_fetch_run`, utilisateur = demandeur,
     détail `{ connectionId, status, errorCode, insertedCount, knownCount, skippedUnlinkedCount, ignoredCount,
     truncatedFrom, window }`). `LIB_ROUTES` **+1** `("post", "provider_connections::fetch", Traced,
     SansEcritureAuJournal)`.
   - **Tests** (HTTP, fabrique factice) : Comptable → `202 { runId }`, puis la ligne passe `succeeded` ; Consultation →
     `403` ; clé d'API `read-write` → `ApiKeyOperationForbidden` ; connexion d'une autre société → `404` ; connexion
     inutilisable (secret absent) → `409 PROVIDER_CONNECTION_UNUSABLE`, `details.reason`, aucune ligne de passage ;
     second `POST` pendant un passage (factice à délai) → `409 PROVIDER_FETCH_IN_PROGRESS`, `details.runId` ; erreur
     `Auth` du factice → toujours `202`, puis `failed`/`auth` et `lastError = PROVIDER_AUTH_FAILED` ; une trace
     `provider_fetch.completed` par passage à la demande, aucune pour un passage `daily` (appel direct de la fonction
     du tour, `#[doc(hidden)] pub` comme `un_tour` de 37-2). Planification : une connexion `daily` dont le dernier
     passage a 23 h n'est pas passée, 25 h l'est ; `manual` jamais.

6. **AC-6 — Lecture de l'état (FR-38-3b-6, O7, O12).**
   - `GET /api/v1/provider-fetch/connections` (Comptable, Administrateur, dans `comptable_routes` ; clés d'API
     refusées **dans le handler**, `ApiKeyOperationForbidden`, comme AC-5) : par connexion de la société `{ id,
     provider, label, schedule, usable, unusableReason, lastRunAt, lastRunStatus, lastError, runningRunId }` — **ni
     `settings`, ni rien du coffre**. Les routes d'administration de 38-1 restent à l'Administrateur.
   - `GET /api/v1/provider-connections/{id}/runs?limit=` (mêmes droits, même refus des clés) : les passages, plus
     récents d'abord (`limit` 1 à 100, défaut 20 ; hors bornes → 400), chacun `{ id, triggerKind, requestedByUserId,
     windowFrom, windowTo, truncatedFrom, startedAt, finishedAt, status, errorCode, errorDetail, insertedCount,
     knownCount, skippedUnlinkedCount, ignoredCount, accountsTouched }` (colonnes de 38-3a ; `status` et `errorCode`
     bruts, lecture tolérante). **La ventilation `skipped { denied, pending, non_balance }` n'est pas gardée** : 38-3a
     n'en a pas de colonne, seul `ignoredCount` (leur somme) est rendu — ce qui clôt Q-38-9a-1 (38-9a n'affiche que le
     total). `errorDetail` est sans secret par construction (AC-2 : motifs constants ; AC-4).
   - **Tests** : le Comptable lit les deux routes ; la réponse de `provider-fetch/connections` ne porte ni `settings`
     ni champ du coffre (assertion sur l'ensemble exact des clés) ; clé d'API → refus ; `limit=0` et `limit=101` →
     400 ; connexion d'une autre société → 404.

7. **AC-7 — Soldes vus, soldes rapportés, complétude (E12, E20 a, FR-38-3b-4, FR-38-3b-5).**
   - À chaque passage, **avant** les mouvements : `balances(Some(fin de fenêtre))` → une ligne
     `provider_balance_snapshots` `reported` par devise ; devise sans compte prestataire actif →
     `provider_balances_seen` (insérée ou `last_seen_at`, `last_amount` mis à jour ; `ignored` gardé ; devise d'un
     compte archivé ou clos exclue, AC-4). Au premier passage d'un compte (aucun `opening` pour sa devise) :
     `balances(Some(début effectif à 00:00 Zurich))` → instantané `opening`, où le **début effectif** est
     `max(fetch_from, history_start(now))` — c'est-à-dire `truncated_from` quand l'historique a borné (PayPal :
     `as_of_time`, 38-7 ; un `as_of` hors de l'historique serait refusé et le contrôle resterait « non disponible »
     pour toujours). **Le `as_of` écrit** aux instantanés (`reported`, et `opening` hors de l'exception ci-dessous) est
     celui que **le connecteur rend** dans `ProviderBalance.as_of` (PayPal : `as_of_time` de la réponse, 38-7 AC-6), **non l'instant demandé** :
     PayPal peut servir « le dernier solde » d'avant l'instant demandé, et la somme des mouvements du contrôle doit
     s'arrêter là où le solde s'arrête, faute de quoi un mouvement intermédiaire ferait un faux écart. Test : un
     factice qui rend `as_of` antérieur de deux heures à la fin demandée, avec un mouvement entre les deux → écart nul.
     Un `reported` rendu **postérieur** à la fin demandée (38-7 AC-6 l'admet jusqu'à un jour) laisse le contrôle
     `complete: null` par la règle de `fetched_through` ci-dessous (les mouvements d'après la fenêtre ne sont pas
     téléchargés) ; test. **Exception de l'`opening` rendu antérieur** : la borne basse de la somme est l'`as_of` de
     l'`opening`, et rien n'est téléchargé avant le début effectif — un `opening` à `as_of` rendu `A` antérieur au
     début effectif `D` laisserait hors de la somme les mouvements de `[A, D)`, déjà comptés dans le solde, d'où un
     écart faux et **durable**. Dans ce cas seulement, le passage lit `movements(Window { from: A, to: D })`
     (sous-fenêtres de `max_window()`, pages validées par `validate_page`, **rien d'écrit**, ni transaction ni
     compteur) et écrit l'`opening` à **`as_of = D`**, montant = solde rendu + `Σ sums` de la devise sur `[A, D)` ;
     si cette lecture échoue (`A` antérieur à `history_start`, erreur du connecteur) **ou n'est pas tentée** (`D − A`
     supérieur à trois `max_window()` : au plus trois appels supplémentaires par passage et par `A`, un `A` très
     ancien — compte dormant — ne multiplie pas les appels à chaque passage), **aucun `opening`** n'est
     écrit (contrôle « non disponible », relu au passage suivant — et **durablement** non disponible tant que
     PayPal sert le même `A` hors de portée ; motif dans `error_detail`, `opening_unavailable: <code()>` ou
     `opening_unavailable: span` quand la lecture n'est pas tentée ; le passage reste `succeeded`). Une lecture par
     `A` distinct (PayPal : un seul `as_of_time` par réponse, donc une seule lecture) : `Σ sums` ne vaut que pour la
     bande `[A, D)` de sa devise. Un `opening` rendu à `A ≥ D` s'écrit tel quel (les mouvements de
     `[D, A)` sont téléchargés et déjà dans le solde). Tests : `opening` rendu deux heures avant `D` avec un mouvement
     dans l'intervalle → `opening` à `D`, montant corrigé du mouvement, écart nul ; même cas, lecture de `[A, D)` en
     échec → aucun `opening`, `complete: null`.
     Un échec de lecture des soldes n'empêche pas les mouvements : le passage
     reste `succeeded`, `error_code` reste `NULL` et le motif s'écrit dans `error_detail` (`balances_unavailable:
     <code()>`) ; contrôle « non disponible ». `error_detail` étant une seule colonne, **plusieurs motifs** d'un même
     passage (`balances_unavailable`, `opening_unavailable`, un par devise au plus) s'y joignent par `; ` dans cet
     ordre, chacun constant (jamais de texte du prestataire) ; 38-9a lit chaque motif séparément. Test : deux motifs
     dans un passage → les deux présents.
   - ⚠️ **Heure des soldes rapportés** : la fenêtre s'arrête à *maintenant − 3 h* ; un solde lu « maintenant »
     compterait des mouvements que le passage n'a pas pu télécharger. Le solde `reported` se lit donc **à la fin de la
     fenêtre** (`as_of = fin` ; `balances(None)` ne sert qu'à « Tester la connexion », 38-7). Test : un mouvement daté
     de *maintenant − 1 h* ne fait pas apparaître d'écart.
   - **Contrôle de complétude**, calculé à la lecture (jamais stocké, jamais d'écriture comptable) : pour chaque compte
     prestataire, `attendu = opening + Σ amount` des transactions du compte de `booked_at ≥ as_of de l'opening` et
     `< as_of` du dernier `reported` (fenêtre demi-ouverte, comme `Window` : un mouvement à l'instant exact de
     l'`opening` y est compté une fois, à l'instant exact du `reported` — non encore téléchargé — jamais) ; la borne
     basse est celle de l'`opening` lui-même, non `fetch_from` (les deux diffèrent quand l'historique a borné) ; `écart = reported − attendu` ; `≠ 0` → « téléchargement incomplet ». Route `GET
     /api/v1/provider-connections/{id}/balances` (Comptable, Administrateur ; **clés d'API refusées dans le handler**,
     `ApiKeyOperationForbidden`) ; `complete: null` sans `opening`, ou quand `fetched_through` du compte est antérieur
     au `as_of` du `reported` : `{ accounts: [{ bankAccountId, currency, opening: {amount, asOf}|null, reported:
     {amount, asOf}|null, expected, gap, complete }], unlinked: [{ currency, firstSeenAt, lastSeenAt, lastAmount,
     ignored }] }` — `unlinked` exclut les devises désormais tenues par un compte. Tests : écart nul, écart d'un
     mouvement manquant (montant exact), compte sans `opening`, `history_start` postérieur à `fetch_from` (`opening`
     lu au début effectif, écart nul sur la période téléchargée).
   - **Ignorer un solde** : `PATCH /api/v1/provider-connections/{id}/balances-seen/{currency}` `{ ignored }`
     (Administrateur, session : `admin_routes`) ; audit `provider_balance_seen.updated` ; `LIB_ROUTES` **+1**
     (`Traced, SansEcritureAuJournal`). « Créer » ou « lier » un compte pour ce solde passe par 38-2 (`POST …/accounts`)
     — **jamais de création d'office** (B2).

8. **AC-8 — Registres, audit, i18n, documentation.**
   - `LIB_ROUTES` **+2** (`fetch`, `balances-seen`, `Traced, SansEcritureAuJournal`), `TEST_ENDPOINT_ROUTES` **+1** ;
     décomptes en delta cumulés avec 38-1 et 38-2 (38-3a n'en ajoute aucune).
   - Audit : types d'entité `provider_fetch_run`, `provider_balance_seen` ; actions `provider_fetch.completed`,
     `provider_balance_seen.updated` ; quatre locales.
   - Codes neufs et messages aux quatre locales : `PROVIDER_FETCH_IN_PROGRESS`, `PROVIDER_CONNECTION_UNUSABLE`
     (variantes d'`AppError`, statut 409) ; `PROVIDER_CONNECTOR_UNAVAILABLE`, `PROVIDER_FETCH_INTERRUPTED`,
     `PROVIDER_ACCOUNT_UNAVAILABLE`, `PROVIDER_FETCH_FAILED` — **chaînes constantes** de `lastError`, non des
     `AppError` (aucune ne sort en statut HTTP), libellées au catalogue `provider-error-*` que 38-7 AC-7 ouvre pour
     `lastError`/`errorCode` ; ceux de `app_code()` (`PROVIDER_AUTH_FAILED` … `PROVIDER_REQUEST_REJECTED`) reçoivent
     leurs messages en 38-7, qui les rend.
   - **Manuel** : aucun écran ici ; **38-9a** (AC-11, manuel d'administration, PDF contrôlé aplati) reçoit de cette
     story les faits à écrire : passage quotidien vérifié **toutes les heures**, lancé 24 h après le **début** du
     dernier passage (un échec attend donc le lendemain, sauf « Télécharger maintenant ») et jamais en mode test ;
     passage **asynchrone** (bouton → suivi) ; un passage interrompu (redémarrage, panique) passe « interrompu » ; un
     passage `running` venu d'une **sauvegarde importée** bloque la connexion **6 h** au plus ; recouvrement de 3 jours
     (`overlapDays`) ; date de départ bornée par l'historique (`truncated_from`) ; devise d'un compte archivé ou clos
     ni téléchargée ni proposée ; contrôle de complétude et ses états. Liste au Change Log.
   - **CHANGELOG** : une ligne de l'entrée non datée de l'epic. **MCP** : aucun outil neuf (E23).

## Tâches / sous-tâches

- [ ] **T0 — Prérequis** : 38-3a fusionnée, version minimale de 38-2 relevée ; sinon arrêt.
- [ ] **T1 — Workspace et version** (AC-1) : crate `kesh-providers` à la version courante, `members`, commentaires de
      `prepare-release.sh` et `CLAUDE.md` (« 10 » → « 11 »), test de versions égales.
- [ ] **T2 — `kesh-providers`** (AC-2) : types, `Connector` (`history_start`, `overlap`, `max_movements_per_page`,
      `test`, `MovementPage` avec `skipped` et `sums`), `ConnectorError` (`code()`, `app_code()`, motif constant),
      validation, fuseau, `FakeConnector`, tests hostiles.
- [ ] **T3 — Transport, fabrique, mode test** (AC-3) : `HttpTransport` (`new`, `for_local_tests`, délai de connexion
      au client, délai total et corps par requête) et ses tests, `ConnectorFactory` et `ConnectorContext`, `AppState`,
      route `_test` et `clear_transient_test_state`, garde des appelants d'`open`.
- [ ] **T4 — Passage** (AC-4) : verrous (garde RAII, panique), fenêtres (`overlap()`, `history_start`,
      `truncated_from`, fenêtre vide), comptes élus, correspondance des colonnes, pages, en-têtes (compte par page,
      finalisation à l'échec), reprise, interblocage, échec (`code()`, `app_code()`), statut tardif, charge.
- [ ] **T5 — Planification et route `fetch`** (AC-5) ; **lecture** (AC-6) ; **soldes et complétude** (AC-7).
- [ ] **T6 — Registres, audit, i18n, CHANGELOG** (AC-8).
- [ ] **T7 — Gates** : gate complet (dépôts : ciblage interdit) sur base remise à zéro, `scripts/gate-ci.sh` (la CI
      lance `cargo test` par binaire), E2E complet au dernier commit de code.

## Notes de développement

- **Découpage (D-P1-1)** : le schéma, la lecture de `PROVIDER_API` et les gardes des chemins existants sont en
  **38-3a** ; cette story n'écrit aucune migration.
- **Une page par fenêtre chez PayPal** : `PaypalConnector::movements` lit et classe la fenêtre entière et rend
  `total_pages = 1` (38-7 AC-4, 38-8 AC-1) ; le passage ne suppose rien d'autre que le contrat de `MovementPage`.
- **E6 : jamais d'écrasement.** Un `ON DUPLICATE KEY UPDATE` qui recopierait les colonnes ferait d'un statut tardif une
  réécriture silencieuse d'une transaction peut-être déjà rapprochée.
- **Heure** : le serveur n'a aucun fuseau aujourd'hui (`Utc::now().date_naive()`, `services/rapports.rs:862-870`) ;
  `chrono-tz` ne sert qu'à la date comptable d'un mouvement et à `fetch_from` à minuit de Zurich.
- **Références** : epic E5, E6, E9, E10, E12, E20, C-38-5 ; 37-2 AC-3 (patron de source et client borné) ; 38-7 AC-1,
  AC-3, AC-4, AC-7, AC-9, AC-10 ; 38-8 AC-1, AC-5, AC-7 ; `repositories/bank_imports.rs:~40-66` ;
  `repositories/letterings.rs:1213` ; `routes/bank_imports.rs:1681` ; `auto_backup.rs:788-803` ; `main.rs:466` ;
  `lib.rs:1434` ; `Cargo.toml:2-13` ; `scripts/prepare-release.sh:7, 96, 219, 271, 280, 356` ;
  `routes/test_endpoints.rs:349` ; `kesh-db/src/retry.rs:151` ; `20260504000001_bank_imports.sql:54-88`.
- **Limite non traitée (LOW, P2)** : la fin de fenêtre « maintenant − 3 h » est le délai de PayPal écrit dans le
  socle commun (C-38-5) ; Wise et Stripe pourront vouloir un autre délai — une méthode `Connector::settle_delay()`
  serait la forme générique, à décider à l'Epic 39 (contrat d'epic, non tranché ici).

## Change Log

- 2026-11-15 — découpage de 38-3 (D-P1-1, Opus). Reprend de 38-3 : AC-1 (crate et workspace), AC-4 à AC-9 et la part
  « passage » de l'AC-12 ; exigences renumérotées (`FR-38-3b-1` à `-6`, `NFR-38-3b-1` à `-5`, origine en colonne).
  Points inter-fiches appliqués : `Connector` gagne `history_start(now)` et `overlap()` (le passage lit `overlap()` au
  lieu de −3 jours ; borne `truncated_from`) ; `MovementPage` gagne `skipped { denied, pending, non_balance }` et
  `sums`, page unique admise ; `ConnectorError` gagne `WindowTooLarge`, `Rejected`, `InvalidWindow`, le passage écrit
  `error_code = err.code()` et `lastError` porte `app_code()` ; `HttpTransport` à bornes par requête (PayPal 60 s,
  8 MiB ; fin du 30 s/60 s contradictoire) ; la fabrique passe le secret ouvert, `secret_updated_at`, le cache de jeton
  et les `payerId` des autres connexions PayPal ; `GET …/runs` rend `truncatedFrom` et `ignoredCount` (colonnes de
  38-3a). Codes propres du passage au niveau API ajoutés (`PROVIDER_CONNECTOR_UNAVAILABLE`,
  `PROVIDER_FETCH_INTERRUPTED`, `PROVIDER_ACCOUNT_UNAVAILABLE`, `PROVIDER_FETCH_FAILED`).
- 2026-11-15 — validation P2 (Opus), première passe complète de cette fiche née du découpage : 21 findings (0
  CRITICAL, 0 HIGH, 10 MEDIUM, 11 LOW) ; remédiés dans la fiche sauf 38-3b-P2-19 (LOW, noté aux *Notes*, contrat
  d'epic) et les reports inter-fiches de P2-1, P2-2, P2-3, P2-13, P2-15, P2-17 (38-7, 38-9a, C-38-5). MEDIUM : P2-1
  transport — `reqwest` n'a pas de délai de connexion **par requête** (`ClientBuilder` seul), `https_only(true)`
  interdisait le serveur axum local en HTTP, et `with_base_url` était prêté au transport alors que l'URL est au
  connecteur (38-7 AC-2) → délai de connexion au client, délai total et corps par requête, `new`/`for_local_tests`
  (AC-3) ; P2-2 « Tester » de 38-7 AC-8 appelait `PaypalConnector::test()` sur un `Box<dyn Connector>` →
  `Connector::test()` générique (AC-2) ; P2-3 `validate_page(…, expected_currencies)` sans sens défini et plafond par
  page « déclaré » sans méthode → `max_movements_per_page()`, paramètre retiré, devise sans compte = `skipped_unlinked`
  ; P2-4 fenêtre vide (`fetchFrom` du jour ou du lendemain admis par 38-1) → `InvalidWindow` de 38-7 et passage en
  échec → `succeeded` sans appel ; P2-5 « chaque compte traité » laissait un compte sans mouvement figer le début de
  fenêtre à `fetch_from` (trois ans relus chaque jour) et pouvait faire reculer `fetched_through` → comptes élus,
  `max(…)` ; P2-6 `opening` lu à `fetch_from` même quand l'historique borne (`as_of` hors historique : contrôle à
  jamais « non disponible ») et bornes de la somme → début effectif, bornes demi-ouvertes ; P2-7 verrou en mémoire
  jamais rendu sur panique (409 jusqu'au redémarrage) → garde RAII, `interrupted` ; P2-8 à l'échec, en-têtes laissés à
  `transaction_count = 0` et à l'empreinte provisoire alors que leurs lignes sont commitées → compte par page,
  finalisation à l'échec ; P2-9 aucune correspondance champ → colonne (`details TEXT NOT NULL`,
  `20260504000001_bank_imports.sql:64`) ; P2-10 tests absents des AC-3, AC-5, AC-6. LOW : P2-11 `main.rs:466`, non
  `:467` ; P2-12 « 10 crates » aussi à `prepare-release.sh:271` et au message `:356`, ligne `kesh-providers/` de
  l'arbre de `CLAUDE.md` ; P2-13 motif d'`InvalidResponse` constant (`&'static str`) ; P2-14 devise d'un compte
  archivé ou clos (ni écrite ni « solde sans compte ») ; P2-15 forme de `GET …/runs`, ventilation `skipped` non gardée
  (clôt Q-38-9a-1) ; P2-16 état du factice vidé par `clear_transient_test_state` ; P2-17 faits du manuel transmis à
  38-9a ; P2-18 interblocage d'une page rejoué ; P2-19 « maintenant − 3 h » propre à PayPal dans le socle (non
  appliqué) ; P2-20 codes de `lastError` = chaînes constantes, non `AppError` ; P2-21 refus des clés d'API des
  lectures, mécanisme écrit. Lignes lues : `Cargo.toml:1-14` ; `Cargo.lock:508-515` ; `CLAUDE.md:58-62` ;
  `prepare-release.sh:5-9, 94-98, 216-221, 271, 280, 356` ; `letterings.rs:1208-1216` ;
  `repositories/bank_imports.rs:36-70` ; `20260504000001_bank_imports.sql:19-88` ; `main.rs:462-470` ;
  `auto_backup.rs:786-805, 1204` ; `routes/bank_imports.rs:1678-1692` ; `lib.rs:258, 518, 524, 1017, 1430-1437` ;
  `test_endpoints.rs:45-58, 349` ; `services/rapports.rs:860-871` ; `audit_route_registry.rs:330-400, 600-640,
  750-780` ; `kesh-db/src/retry.rs:151` ; `entities/bank_account.rs:161-174` ; fiches 37-2 AC-3, 38-1 AC-3/AC-4,
  38-3a AC-3/AC-4, 38-7 AC-1 à AC-10, 38-8 AC-1, 38-9a AC-5 à AC-8, AC-11. Axes non exercés : l'API de `reqwest`
  relue de mémoire (registre local absent ; `RequestBuilder::timeout` existe, `connect_timeout` est du seul
  `ClientBuilder`) ; la documentation PayPal (`as_of_time`, borne de l'historique) non relue ; aucune exécution.
- 2026-11-15 — alignement inter-fiches P2 (Opus) : AC-4 — course de 38-2 AC-6 (c) rejouée contre l'insertion d'un
  passage (retrait des réglages contre écriture d'une page : réglages relus sous le verrou du compte, jamais une
  transaction de prestataire sur un compte sans réglages, les deux ordres testés) ; AC-2 — `validate_page` : unicité
  du couple `(provider_tx_id, currency)` dans une page (et non de l'identifiant seul) ; AC-3 — variante additive
  `paypalResponses: [{ method, path, query?, status, body }]` du corps de `POST /api/v1/_test/provider-connector`
  (servies dans l'ordre, la dernière se répète ; `PaypalConnector` sur le transport factice ; `TEST_ENDPOINT_ROUTES`
  inchangé) ; renvoi `NFR-38-6c-4` → `NFR-38-6c1-4` (D-P2-3).
- 2026-11-15 — validation P3 (Sonnet) : 3 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW) ; remédiés dans la fiche sauf
  38-3b-P3-3 (LOW, inter-fiche 38-3a). P3-1 (MEDIUM) : 38-7 AC-10 promet des réponses PayPal « par défaut » servies par
  le transport factice, mais l'AC-3 ne les servait que sur `paypalResponses` explicite (sinon `FakeConnector`) → le
  corps admet la chaîne `"default"` ; P3-2 (MEDIUM) : le `as_of` des instantanés n'était pas dit — l'instant demandé
  aurait fait un faux écart quand PayPal sert un solde antérieur → `as_of` rendu par le connecteur, test ; P3-3 (LOW,
  38-3a, non appliqué ici) : l'instantané `opening` y est défini « à `fetch_from` » alors qu'il se lit au **début
  effectif** (AC-7), à aligner. Aussi : `ConnectorContext` ne référence plus le cache de jeton (type de 38-7, story
  suivante) — `other_payer_ids: Vec<String>` posé ici, le cache ajouté par 38-7. Lignes relues et conformes :
  `Cargo.toml:2-13` ; `Cargo.lock:508-515` ; `CLAUDE.md:60` ; `prepare-release.sh:7, 96, 219, 271, 280, 356` ;
  `letterings.rs:1213` ; `bank_imports.rs:36-70` ; `20260504000001_bank_imports.sql:33, 64` ; `lib.rs:1434` ;
  `test_endpoints.rs:49-55, 349` ; `main.rs:466` ; `auto_backup.rs:788-803` ; `routes/bank_imports.rs:1681-1690` ;
  `kesh-db/src/retry.rs:151` ; registre `audit_route_registry.rs:335-377` (`SansEcritureAuJournal` = journal
  comptable, cohérent avec 38-1). Axes non exercés : API `reqwest`, documentation PayPal, aucune exécution ; `ls`
  du registre `TEST_ENDPOINT_ROUTES` seulement lu aux lignes 618-620.
- 2026-11-15 — validation P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW ; remédiés dans la fiche. Périmètre :
  `git diff f005ba2f a5682b2e` de la fiche (remédiation P3). MEDIUM 38-3b-P4-1 (AC-7) : le `as_of` rendu, appliqué
  aussi à l'`opening`, faisait de la borne basse de la somme un instant `A` antérieur au début effectif `D` quand
  PayPal sert un solde plus ancien (38-7 AC-6 l'admet) — les mouvements de `[A, D)`, comptés dans le solde mais jamais
  téléchargés, auraient fait un écart faux et durable → dans ce cas seul, lecture sans écriture de `[A, D)` et
  `opening` écrit à `as_of = D`, montant corrigé de `Σ sums` ; lecture en échec → aucun `opening` (contrôle « non
  disponible », relu au passage suivant), tests. MEDIUM 38-3b-P4-2 (AC-3) : la forme `"default"` de `paypalResponses`
  n'avait ni règle de validation (quelle chaîne ?), ni test, ni comportement en 38-3b, avant tout `PaypalConnector` →
  validation (tableau d'objets bornés ou exactement `"default"`, sinon 400 `AppError::Validation`), variante gardée
  mais `FakeConnector` jusqu'à 38-7, tests. LOW 38-3b-P4-3 (AC-7) : `reported` rendu postérieur (≤ 1 jour, 38-7 AC-6)
  → `complete: null` par la règle de `fetched_through`, dit et testé. LOW 38-3b-P4-4 (AC-3) : `#[non_exhaustive]` est
  sans effet dans le crate qui construit ; `ConnectorContext` et `ConnectorFactory` placés dans `kesh-api::providers`,
  constructeur unique. Lignes lues : `routes/test_endpoints.rs:380-396` ; `errors.rs:139-153` ; `38-7-…md:100-110,
  181-196, 262-276` ; `38-8-…md:149-176, 265-272` ; `38-9a-…md:206`. Axes non exercés : le reste de la fiche (hors
  périmètre), la documentation PayPal (sens exact de `as_of_time` : « dernier solde rafraîchi »), aucune exécution.
  Inter-fiches signalés, non appliqués : 38-7 AC-6 pourrait dire que l'`opening` rendu antérieur se complète par
  38-3b AC-7 ; 38-9a (contrôle) : motif `opening_unavailable` dans `errorDetail`.
- 2026-11-15 — validation P5 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés dans la fiche. Périmètre :
  `git diff 7e64e03b c6ea4402` de la fiche (remédiation P4). LOW 38-3b-P5-1 (AC-7) : la lecture de `[A, D)` de
  l'`opening` rendu antérieur n'avait pas de borne (jusqu'à trois ans de sous-fenêtres, rejouées à chaque passage tant
  que l'`opening` manque) → au plus trois `max_window()`, sinon non tentée (`opening_unavailable: span`) ; une lecture par
  `A` distinct (`Σ sums` ne vaut que pour la bande de sa devise) ; l'indisponibilité durable (même `A` hors de portée)
  dite. LOW 38-3b-P5-2 (AC-7) : `error_detail` est une colonne, deux motifs (`balances_unavailable`,
  `opening_unavailable`) pouvaient s'y écraser → jointure par `; `, testée. LOW 38-3b-P5-3 (AC-3) : renvoi
  `test_endpoints.rs:387-392` (commentaire) → `:391-395` (le `AppError::Validation`). Vérifié conforme : la somme
  `opening` corrigée (solde à `A` + `Σ net` de `[A, D)` = solde à `D`, `sums` = Σ `net` des seuls mouvements rendus,
  comme les mouvements écrits), `D < fin` garanti par la règle de la fenêtre vide, `fetched_through` pour le `reported`
  postérieur, `paypalResponses` ≡ 38-7 AC-10, `ConnectorContext` dans `kesh-api` (le crate voit `kesh-providers`).
  Lignes lues : `38-3b` AC-2 (trait, `sums`), AC-4 (fenêtre), `38-3a:120-135` ; `38-7` AC-6, AC-10 ; `test_endpoints.rs:
  384-395`. Axes non exercés : documentation PayPal (`as_of_time`), reste de la fiche, aucune exécution. Inter-fiches
  signalés, non appliqués : 38-9a — motifs `opening_unavailable` (dont `span`) et jointure `; ` dans `errorDetail`,
  libellé et état « contrôle non disponible » aux quatre locales.
