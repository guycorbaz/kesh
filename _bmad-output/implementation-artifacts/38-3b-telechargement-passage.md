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
   (`:2-13`), commentaires « 10 crates » de `scripts/prepare-release.sh` (`:7`, `:219` ; le script lit
   `crates/*/Cargo.toml` par motif, `:96`, et ne compte rien) et `CLAUDE.md` § *Repository Structure* (`:60`,
   « workspace de 10 crates »). Un test relit que les onze `Cargo.toml` portent la même version (P2-bis : un crate en
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
       38-8 AC-7, O11 ; le connecteur factice : 3 jours).
     Trait objet-safe **`Transport`** (requête HTTP minimale **avec ses bornes** → réponse bornée), dont
     l'implémentation `reqwest` vit dans `kesh-api` (AC-3).
   - **`ConnectorError`** : `Auth`, `Permission`, `RateLimited { retry_after }`, `Unavailable`, `InvalidResponse {
     reason }`, `Transport { kind }`, **`WindowTooLarge { total_items: Option<u32> }`**, **`Rejected`**,
     **`InvalidWindow`** (38-7 AC-7) — codes constants, **jamais** le texte d'une erreur `reqwest` (qui porte l'URL,
     précédent 37-2) ni un secret. **Une seule** `code()` (code du passage : `auth`, `permission`, `rate_limited`,
     `unavailable`, `invalid_response`, `transport`, `window_too_large`, `rejected`, `invalid_window`) et **une seule**
     `app_code()` (code d'API : table de 38-7 AC-7 — `PROVIDER_AUTH_FAILED`, `PROVIDER_PERMISSION_MISSING`,
     `PROVIDER_RATE_LIMITED`, `PROVIDER_UNAVAILABLE`, `PROVIDER_RESPONSE_INVALID`, `PROVIDER_WINDOW_TOO_LARGE`,
     `PROVIDER_REQUEST_REJECTED` ; `InvalidWindow`, erreur de programmation de l'appelant → `PROVIDER_RESPONSE_INVALID`
     et `tracing::error!`). Test : chaque variante a son couple de codes.
   - **Validation commune** `validate_page(page, window, expected_currencies)` : `provider_tx_id` non vide, ≤ 64, ASCII
     imprimable ; devise ISO à trois lettres majuscules ; montants à deux décimales au plus et `|montant| < 10^16` ;
     `net == gross + fee` ; `booked_at` dans la fenêtre ; textes bornés aux longueurs des colonnes de 38-3a (refus,
     jamais de troncature silencieuse d'un identifiant ; troncature **signalée** des textes libres `details` /
     `counterparty_name` à leur borne) ; `item_names` ≤ 50 noms de ≤ 255 ; au plus un plafond de mouvements par page
     déclaré par le connecteur (jamais lu de la réponse ; PayPal, page unique d'une fenêtre : 10 000, 38-7 AC-4) ;
     pagination cohérente (`page ≤ total_pages`, `total_pages` constant d'une page à l'autre d'une même fenêtre) ; pas
     deux fois le même `provider_tx_id` dans une page ; **`sums` égale la somme recalculée** par devise. Échec →
     `InvalidResponse { reason }`, **aucune ligne de la page écrite**. Tests de réponses **hostiles** (C-38-11) : chacun
     des refus ci-dessus a son test.
   - **Temps** : `booking_date = booked_at` converti en `Europe/Zurich` (`chrono-tz`), date seule ; tests à 23 h 30 UTC
     le 31 décembre et aux deux changements d'heure.
   - **`FakeConnector`** (`#[doc(hidden)] pub`, précédent `MockRateSource`, 37-2 AC-3 — visible des tests d'intégration
     de `kesh-api`) : soldes et mouvements programmés par fenêtre, `skipped`, `history_start`, `overlap`, erreurs et
     pages incohérentes à la demande. **Aucun connecteur réel ici** : `paypal` arrive en 38-7 ; en 38-3b, une connexion
     `paypal` hors mode test termine son passage en `failed`, `error_code = "connector_unavailable"`.

3. **AC-3 — Transport, fabrique et mode test (E9, E10, NFR-38-3b-2, NFR-38-3b-5).**
   - `kesh-api::providers::HttpTransport` implémente `Transport` en **réutilisant le client borné de 37-2** (rustls,
     `redirect(Policy::none())`, `https_only(true)`) ; **bornes par requête** (`TransportLimits { connect_timeout,
     total_timeout, max_body }`, passées par le connecteur avec chaque requête ; défaut 5 s, 30 s, 8 MiB pour un
     connecteur qui n'en déclare pas ; PayPal : 5 s, **60 s**, 8 MiB, 38-7 AC-1 — il n'y a plus de valeur unique
     contradictoire) ; réponse lue par morceaux bornée (`Content-Length` contrôlé puis lecture bornée, dépassement →
     `InvalidResponse`). L'URL de base est une **constante du connecteur** (38-7), aucune variable d'environnement ne la
     change ; `with_base_url` `#[doc(hidden)] pub` pour les tests contre un serveur axum local.
   - `AppState` gagne `connector_factory: Arc<dyn ConnectorFactory>` et `provider_fetch_guard` (AC-4). La fabrique
     construit le connecteur d'une connexion à partir d'un **`ConnectorContext`** : prestataire, réglages typés de
     38-1, **secret ouvert** (`Zeroizing<String>`, par la **voie unique** de 38-1 AC-1 — la garde des appelants
     d'`open` gagne **ce seul site**), **`secret_updated_at`**, **cache de jeton** (champ d'`AppState` posé par 38-7
     AC-3, passé tel quel), **`payerId` des autres connexions PayPal de la société** (38-8 AC-5, lus par la fabrique)
     et transport. La route `POST …/test` (38-1 AC-7) construit son connecteur par la même fabrique (38-7). Sites de
     construction littérale d'`AppState` : inventaire de 37-2 AC-3, à relancer (`grep -rn "AppState {"`).
   - **`KESH_TEST_MODE`** : `main.rs` installe une fabrique qui rend un `FakeConnector` pour **toute** connexion,
     piloté par `POST /api/v1/_test/provider-connector` (corps : soldes, pages de mouvements par fenêtre, `skipped`,
     erreur ou délai à servir, par `connectionId`), monté avec les autres routes `_test` (`lib.rs:1434`,
     `routes/test_endpoints.rs:49-55`) ; `TEST_ENDPOINT_ROUTES` **+1** (`Exempt("chemin de test…")`). 38-7 remplace ce
     factice, pour `paypal`, par un **transport** factice au format PayPal (38-7 AC-10). La tâche quotidienne **n'est
     pas lancée** en mode test (AC-5). Aucun appel réseau en CI : seul `HttpTransport` ouvre une connexion, et aucun
     test ne le construit sans `with_base_url` local (garde de source).

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
   - **Préalables** : connexion utilisable (38-1 AC-3 : prestataire connu, réglages lisibles, clé, secret) — sinon `409
     PROVIDER_CONNECTION_UNUSABLE`, `details.reason` = `unusableReason` de 38-1 ; aucun passage créé.
   - **Fenêtre** : fin = maintenant − 3 h (délai de PayPal) ; début = **min sur les comptes prestataires actifs de la
     connexion** (non archivés, non clos) de `max(fetch_from à 00:00 Europe/Zurich, fetched_through − overlap())`
     (`fetched_through` `NULL` → `fetch_from`) ; puis **`max(début, history_start(now) à 00:00 Zurich)`** — s'il borne,
     le début effectif s'écrit à `truncated_from` (affiché par 38-9a ; jamais d'appel avant `history_start`, 38-7
     AC-9) ; découpée en sous-fenêtres de `max_window()` au plus. Aucun compte actif → passage `succeeded` sans
     mouvement (soldes lus quand même, AC-7).
   - **Pour chaque sous-fenêtre et chaque page** : `movements(window, page)` → `validate_page` (AC-2) → **une
     transaction** : `ignored_count += skipped.denied + skipped.pending + skipped.non_balance` ; pour chaque mouvement,
     compte bancaire = la ligne `payment_provider_accounts` (connexion, devise) d'un compte actif ; devise sans compte →
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
     une ligne valide), `provider_fetch_run_id`) ; colonnes de 38-3a AC-3 remplies, `status = 'pending'`. À la fin de
     la sous-fenêtre (toutes pages) : `fetched_through` de chaque compte traité avancé à la fin de la sous-fenêtre,
     **dans la transaction de la dernière page** — ou, quand la fenêtre n'a aucune page (`total_pages = 0`), dans une
     transaction à lui seul ; point de reprise = ce champ.
   - **Fin** : en-têtes finalisés (`transaction_count`, `file_hash` = SHA-256 des enregistrements normalisés insérés,
     triés par `provider_tx_id` — forme écrite au module), passage `succeeded` ; `payment_provider_connections.
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
     `auto_backup::spawn` (`main.rs:467`, patron `auto_backup.rs:788-803` : `interval_at`, `MissedTickBehavior::Skip`,
     un tour par `tokio::spawn`), **jamais en `KESH_TEST_MODE`** ; garde lexicale au patron de
     `auto_backup::tests::lancement_par_main_seulement`. Cadence : toutes les heures ; une connexion `daily` utilisable
     dont le dernier passage (tout déclencheur) a commencé il y a plus de 24 h est passée, l'une après l'autre. Passage
     automatique **non audité** (`tracing` + état du passage), C-37-6 ; erreurs journalisées sans secret.
   - **À la demande** : `POST /api/v1/provider-connections/{id}/fetch` — **Comptable ou Administrateur, session
     seule** : dans `comptable_routes`, refus d'une clé d'API dans le handler par `ApiKeyOperationForbidden` (patron de
     `bank_imports::delete`, `routes/bank_imports.rs:1681-1690` ; NFR-38-6c-4) ; vaut pour `daily` **et** `manual`
     (B9). Réponse **`202`** `{ runId }` : le passage court dans un `tokio::spawn` ; refus synchrones : `404`, `409
     PROVIDER_CONNECTION_UNUSABLE`, `409 PROVIDER_FETCH_IN_PROGRESS`. **Aucune erreur du connecteur n'est un statut
     HTTP** de cette route (38-7 AC-7) : elle se lit à l'état du passage. **Audit** : à la fin du passage, dans la
     transaction qui écrit son état, `provider_fetch.completed` (entité `provider_fetch_run`, utilisateur = demandeur,
     détail `{ connectionId, status, errorCode, insertedCount, knownCount, skippedUnlinkedCount, ignoredCount,
     truncatedFrom, window }`). `LIB_ROUTES` **+1** `("post", "provider_connections::fetch", Traced,
     SansEcritureAuJournal)`.

6. **AC-6 — Lecture de l'état (FR-38-3b-6, O7, O12).**
   - `GET /api/v1/provider-fetch/connections` (Comptable, Administrateur ; clés d'API refusées) : par connexion de la
     société `{ id, provider, label, schedule, usable, unusableReason, lastRunAt, lastRunStatus, lastError,
     runningRunId }` — **ni `settings`, ni rien du coffre**. Les routes d'administration de 38-1 restent à
     l'Administrateur.
   - `GET /api/v1/provider-connections/{id}/runs?limit=` (mêmes droits) : les passages, plus récents d'abord (`limit`
     ≤ 100, défaut 20), colonnes de la table — `truncatedFrom` et `ignoredCount` compris —, `error_detail` compris
     (sans secret par construction, AC-4).

7. **AC-7 — Soldes vus, soldes rapportés, complétude (E12, E20 a, FR-38-3b-4, FR-38-3b-5).**
   - À chaque passage, **avant** les mouvements : `balances(Some(fin de fenêtre))` → une ligne
     `provider_balance_snapshots` `reported` par devise ; devise sans compte prestataire actif →
     `provider_balances_seen` (insérée ou `last_seen_at`, `last_amount` mis à jour ; `ignored` gardé). Au premier
     passage d'un compte (aucun `opening` pour sa devise) : `balances(Some(fetch_from à 00:00 Zurich))` → instantané
     `opening` (PayPal : `as_of_time`, 38-7). Un échec de lecture des soldes n'empêche pas les mouvements : le passage
     reste `succeeded`, `error_code` reste `NULL` et le motif s'écrit dans `error_detail` (`balances_unavailable:
     <code()>`) ; contrôle « non disponible ».
   - ⚠️ **Heure des soldes rapportés** : la fenêtre s'arrête à *maintenant − 3 h* ; un solde lu « maintenant »
     compterait des mouvements que le passage n'a pas pu télécharger. Le solde `reported` se lit donc **à la fin de la
     fenêtre** (`as_of = fin` ; `balances(None)` ne sert qu'à « Tester la connexion », 38-7). Test : un mouvement daté
     de *maintenant − 1 h* ne fait pas apparaître d'écart.
   - **Contrôle de complétude**, calculé à la lecture (jamais stocké, jamais d'écriture comptable) : pour chaque compte
     prestataire, `attendu = opening + Σ amount` des transactions du compte de `booked_at ≥ fetch_from 00:00 Zurich` et
     `≤ as_of` du dernier `reported` ; `écart = reported − attendu` ; `≠ 0` → « téléchargement incomplet ». Route `GET
     /api/v1/provider-connections/{id}/balances` (Comptable, Administrateur ; **clés d'API refusées dans le handler**,
     `ApiKeyOperationForbidden`) ; `complete: null` sans `opening`, ou quand `fetched_through` du compte est antérieur
     au `as_of` du `reported` : `{ accounts: [{ bankAccountId, currency, opening: {amount, asOf}|null, reported:
     {amount, asOf}|null, expected, gap, complete }], unlinked: [{ currency, firstSeenAt, lastSeenAt, lastAmount,
     ignored }] }` — `unlinked` exclut les devises désormais tenues par un compte. Tests : écart nul, écart d'un
     mouvement manquant (montant exact), compte sans `opening`.
   - **Ignorer un solde** : `PATCH /api/v1/provider-connections/{id}/balances-seen/{currency}` `{ ignored }`
     (Administrateur, session : `admin_routes`) ; audit `provider_balance_seen.updated` ; `LIB_ROUTES` **+1**
     (`Traced, SansEcritureAuJournal`). « Créer » ou « lier » un compte pour ce solde passe par 38-2 (`POST …/accounts`)
     — **jamais de création d'office** (B2).

8. **AC-8 — Registres, audit, i18n, documentation.**
   - `LIB_ROUTES` **+2** (`fetch`, `balances-seen`, `Traced, SansEcritureAuJournal`), `TEST_ENDPOINT_ROUTES` **+1** ;
     décomptes en delta cumulés avec 38-1 et 38-2 (38-3a n'en ajoute aucune).
   - Audit : types d'entité `provider_fetch_run`, `provider_balance_seen` ; actions `provider_fetch.completed`,
     `provider_balance_seen.updated` ; quatre locales.
   - Codes neufs et messages aux quatre locales : `PROVIDER_FETCH_IN_PROGRESS`, `PROVIDER_CONNECTION_UNUSABLE`,
     `PROVIDER_CONNECTOR_UNAVAILABLE`, `PROVIDER_FETCH_INTERRUPTED`, `PROVIDER_ACCOUNT_UNAVAILABLE`,
     `PROVIDER_FETCH_FAILED` ; ceux de `app_code()` (`PROVIDER_AUTH_FAILED` … `PROVIDER_REQUEST_REJECTED`) reçoivent
     leurs messages en 38-7, qui les rend.
   - **CHANGELOG** : une ligne de l'entrée non datée de l'epic. **MCP** : aucun outil neuf (E23).

## Tâches / sous-tâches

- [ ] **T0 — Prérequis** : 38-3a fusionnée, version minimale de 38-2 relevée ; sinon arrêt.
- [ ] **T1 — Workspace et version** (AC-1) : crate `kesh-providers` à la version courante, `members`, commentaires de
      `prepare-release.sh` et `CLAUDE.md` (« 10 » → « 11 »), test de versions égales.
- [ ] **T2 — `kesh-providers`** (AC-2) : types, `Connector` (`history_start`, `overlap`, `MovementPage` avec `skipped`
      et `sums`), `ConnectorError` (`code()`, `app_code()`), validation, fuseau, `FakeConnector`, tests hostiles.
- [ ] **T3 — Transport, fabrique, mode test** (AC-3) : `HttpTransport` à bornes par requête, `ConnectorFactory` et
      `ConnectorContext`, `AppState`, route `_test`, garde des appelants d'`open`.
- [ ] **T4 — Passage** (AC-4) : verrous, fenêtres (`overlap()`, `history_start`, `truncated_from`), pages, en-têtes,
      reprise, échec (`code()`, `app_code()`), statut tardif, charge.
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
  `repositories/letterings.rs:1213` ; `routes/bank_imports.rs:1681` ; `auto_backup.rs:788-803` ; `main.rs:467` ;
  `lib.rs:1434` ; `Cargo.toml:2-13` ; `scripts/prepare-release.sh:7, 96, 219`.

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
