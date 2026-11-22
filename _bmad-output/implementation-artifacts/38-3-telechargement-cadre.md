# Story 38-3 : téléchargement — cadre commun des prestataires

Status: split

⛔ **Découpée en 38-3a et 38-3b (statut `split`, décision D-P1-1 du 2026-11-15)** :
[`38-3a-telechargement-schema.md`](38-3a-telechargement-schema.md) (schéma non breaking, entités, décodage tolérant de
`PROVIDER_API`, gardes) et [`38-3b-telechargement-passage.md`](38-3b-telechargement-passage.md) (crate
`kesh-providers`, transport, passage, planification, soldes). Ce qui suit est **historique** ; les deux fiches filles
font foi. Le décodage tolérant de `ReconciliationMatchType` passe à **38-4**.

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`, numéros de ligne relevés au commit `8b3d548d`. Epic 38 :
décisions E4, E5, E6, E7, E8, E9, E10, E12, E18, E20 (a) ; contrats **C-38-0**, **C-38-4**, **C-38-5**,
C-38-11, C-38-12 ; décisions du responsable de l'epic B2, B8, B9, B12, B20, **B22** ; arbitrage **O1** (voir ci-dessous). **Dépend de** :
38-1, **38-2 (qui porte l'unique migration breaking de l'epic)**, l'Epic 37 (client HTTP borné et patron « source
objet-safe + factice en mode test + route `_test` » de 37-2 AC-3, C-37-6 ; liste close de 37-1 AC-6). **Suivie par** :
38-4 (évalue les critères neufs déclarés ici), 38-6a/b/c (propositions et rattachement des transactions téléchargées),
38-7 (connecteur PayPal réel), 38-8 (classement des codes PayPal), 38-9a (écrans, libellé « Téléchargé de … »),
38-10 (relevé joint).*

⚠️ **O1 (arbitrage de l'orchestrateur, 2026-11-15) — plus de migration breaking dans cette story.** L'unique migration
breaking de l'epic est en **38-2** (contrainte d'IBAN + `match_type` en `VARCHAR(40)`, relèvement de
`kesh_version_min_required` en dernière instruction, bump des crates dans le même commit : P2, P2-bis, P7, version
`<X.Y.Z>` = Q1 de Guy, à elle). 38-3 n'a que sa migration **non breaking** (AC-3). Ses écritures de
`source_format = 'PROVIDER_API'` et de la colonne `match_type` élargie **s'appuient sur le relèvement posé par 38-2**
(même release : un binaire antérieur est déjà refusé sur la base migrée). **Prérequis** : la migration de 38-2 est
appliquée avant celle-ci (numéro de date postérieur) ; la version `<X.Y.Z>` n'est pas écrite ici.

## Story

En tant que **Comptable** ou **Administrateur** d'une société qui tient ses soldes PayPal dans Kesh, je veux que Kesh
**télécharge** les mouvements de chaque connexion — chaque jour si elle est réglée ainsi, et à la demande dans tous
les cas — depuis la date « télécharger depuis », **sans doublon** même quand les fenêtres se recouvrent, qu'il
enregistre chaque passage, **liste les soldes de devise qui n'ont pas encore de compte** sans rien créer, et me
signale un téléchargement incomplet en comparant les soldes rapportés aux mouvements reçus, afin que les propositions
d'imputation (38-6) reposent sur un relevé complet et vérifiable.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-3-1` | Télécharger les mouvements d'une connexion depuis `fetch_from`, chaque jour (`daily`) ou à la demande (`POST …/fetch`, Comptable ou Administrateur), dans les deux réglages. | AC-6, AC-7, AC-8 |
| `FR-38-3-2` | Sans doublon : `provider_tx_id` unique par compte, insertion idempotente, recouvrement de trois jours. | AC-3, AC-6 |
| `FR-38-3-3` | Chaque passage est enregistré ; chaque compte touché reçoit un en-tête `bank_imports` en `PROVIDER_API`, affiché « Téléchargé de … », dont la suppression est refusée (`PROVIDER_IMPORT_NOT_DELETABLE`). | AC-3, AC-6, AC-10 |
| `FR-38-3-4` | Un solde de devise sans compte Kesh est listé et proposé (B2) ; ses transactions ne se téléchargent pas. | AC-9 |
| `FR-38-3-5` | Les soldes rapportés s'enregistrent ; un téléchargement incomplet est signalé avec l'écart (E20 a). | AC-9 |
| `FR-38-3-6` | Une source ou un type de règle inconnus se lisent sans erreur (E4). | AC-2 |
| `FR-38-3-7` *(ajout)* | Un compte prestataire ne reçoit ni import de fichier ni relevé saisi à la main, et ses transactions ne reçoivent aucune des propositions existantes (facture, règle, ventilation) avant la variante `provider` de 38-6a. | AC-11 |
| `FR-38-3-8` *(ajout, Q-38-1-a)* | Le Comptable lit l'état des connexions (sans réglages ni secrets) pour lancer un passage. | AC-8 |
| `NFR-38-3-1` | Migration **non breaking** (DDL seul, aucune écriture de données) ; la migration breaking de l'epic est celle de 38-2 (O1), dont cette story dépend. | AC-1, AC-3 |
| `NFR-38-3-2` | Une sauvegarde d'avant l'Epic 38 se réimporte (liste close R1) ; la recette de montée et le retour par la seule sauvegarde d'avant la mise à jour sont ceux de 38-2, étendus aux tables et colonnes de cette story. | AC-3, AC-12 |
| `NFR-38-3-3` | Un passage à la fois par connexion ; un échec garde l'inséré et dit où reprendre. | AC-6 |
| `NFR-38-3-4` | Aucun appel réseau en CI ; transport factice piloté par une route `_test`, inactif hors `KESH_TEST_MODE`. | AC-5, AC-7 |
| `NFR-38-3-5` *(ajout)* | Toute réponse d'un prestataire est validée avant écriture ; une page invalide n'écrit rien et fait échouer le passage avec un état lisible (R5 de 37, C-38-11). | AC-4 |
| `NFR-38-3-6` *(ajout)* | La date comptable d'un mouvement est sa date à Europe/Zurich ; l'instant reçu est gardé en UTC. | AC-3, AC-4 |

## Critères d'acceptation

1. **AC-1 — La migration breaking est celle de 38-2 (O1, E19, C-38-0, NFR-38-3-1) ; ce que cette story en attend.**
   Le fichier `…_provider_api_source_and_rule_types.sql` de la rédaction (type `match_type` en `VARCHAR(40)` +
   relèvement de `kesh_version_min_required`) n'existe plus ici : il est repris par 38-2 avec la contrainte d'IBAN
   (O1, O2). 38-3 **ne contient ni `MODIFY COLUMN`, ni `DROP`, ni `UPDATE _kesh_version`** : ni P2, ni P2-bis, ni P7
   ne la concernent (le détecteur P7 classe sur le premier mot-clé ; ses statements sont des `CREATE`/`ALTER ADD`).
   - **Ce que 38-3 lit de 38-2** : `reconciliation_rules.match_type` est un `VARCHAR(40)` `ascii_bin` (CHECK de
     format) — les bras `Other(String)` et les quatre variantes déclarées de l'AC-2 en dépendent ; `bank_imports.
     source_format` reste un `VARCHAR(32)` sans `CHECK` (`migrations/20260504000001_bank_imports.sql:25`), la valeur
     `'PROVIDER_API'` n'exigeant aucun DDL ; la version minimale relevée par 38-2 protège un binaire antérieur, dont
     le `FromStr` refuse l'inconnu (`crates/kesh-db/src/entities/bank_import.rs:57-67`,
     `entities/reconciliation_rule.rs:66-77`). **Test de garde** (T10) : la liste des migrations de la branche place
     celle de 38-2 avant celle de l'AC-3, et `kesh_version_min_required` y est déjà relevé.
   - **Crate `kesh-providers`** (AC-4) : né **après** le bump de 38-2, il porte la version courante des crates à sa
     création (`version = "<X.Y.Z>"` de 38-2) ; le workspace passe à **onze** membres : `members` du `Cargo.toml`
     racine (`:2-13`), commentaires « 10 crates » de `scripts/prepare-release.sh` (`:7`, `:219` ; le script lit
     `crates/*/Cargo.toml` par motif, `:96`, et ne compte rien) et `CLAUDE.md` § *Repository Structure* (`:60`,
     « workspace de 10 crates »). Un test relit que les onze `Cargo.toml` portent la même version (P2-bis : un crate
     en retard rend le binaire plus ancien que sa base).
   - **P6** (cette story ajoute une migration) : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`
     (relevé au commit : `migrations_upgrade_path.rs` ×27, `invoice_lines_revenue_account_backfill.rs` ×13,
     `lettering_documents_backfill.rs` ×6, `accounts_role_backfill.rs` ×5, `closing_accounts_backfill.rs` ×5,
     `pre_upgrade_backup/tests.rs` ×3, quatre autres ×2), chaque site inspecté, cité au Change Log. **P5** : ligne
     d'audit de l'AC-3. **P8** : aucune migration appliquée retouchée.

2. **AC-2 — Décodage tolérant (E4, FR-38-3-6).**
   - **`BankImportSourceFormat`** gagne `ProviderApi` (`"PROVIDER_API"`) et `Other(String)` ; `FromStr` ne refuse
     plus rien (`Other` pour l'inconnu) ; `Serialize`/`Deserialize` à la main (la chaîne brute, `Other` compris).
     `as_str` perd son `&'static str` (il rend `&str`, emprunté à `Other`) : `Display`, `Encode`
     (`entities/bank_import.rs:~85`) et les sites qui exigent `'static` suivent ; `SourceFormatTag::as_db_str` reste
     `&'static str` (pas de `Other` côté tag).
     `kesh_core::bank_imports::SourceFormatTag` (`crates/kesh-core/src/bank_imports.rs:145-170`, aligné par contrat sur
     `as_db_str`) gagne `ProviderApi` ; aucun chemin d'import de fichier ne le rend (`from_imported`, comme `Manual`).
   - **`ReconciliationMatchType`** gagne les **quatre variantes déclarées** de C-38-6 — `CounterpartyEmailExact`
     (`counterparty_email_exact`), `InvoicePrefix` (`invoice_prefix`), `ItemNameContains` (`item_name_contains`),
     `EventCodePrefix` (`event_code_prefix`) — et `Other(String)`. **Évaluation** : `rule_matches`
     (`crates/kesh-reconciliation/src/rules.rs:46-70` ; `match rule.match_type` devient `match &rule.match_type`, le
     type perdant `Copy`) rend **`false`** pour les quatre neuves et pour `Other` (test nommé par variante) ; 38-4 les
     évalue. **Écriture** (`routes/reconciliation_rules.rs`, `matchType` lu à `:63`,
     `:82`) : un type `Other` → `400 RULE_CRITERION_UNKNOWN` (`details.matchType`) ; les quatre neuves → même refus
     **jusqu'à 38-4**, qui le lève (point de contact écrit des deux côtés) ; une modification d'une règle stockée en
     `Other` → `409 RULE_CRITERION_UNKNOWN` (le type n'est pas modifiable, `:121`, mais la règle ne se réécrit pas non
     plus : refus sans écriture fausse). **Lecture** : la liste des règles rend `matchType` brut ; l'export CSV des
     règles (`exports/csv_tables.rs:1015-1059`) l'écrit brut ; `errors.rs:1150-1159, 3304-3316` relus (lecteurs de
     l'epic) et adaptés à `Other`.
   - **Perte de `Copy`** sur les deux enums (portent un `String`) : inventaire au compilateur ; relevé au commit —
     `BankImportSourceFormat` : 66 occurrences hors de son fichier, dans `backup.rs`, deux routes et 17 fichiers de
     tests ; `ReconciliationMatchType` : 53, dans `rules.rs`, `routes/reconciliation_rules.rs` et un fichier de tests.
     Corriger par `clone()` ou référence, **sans** réintroduire de `match` qui refuse l'inconnu.
   - Tests : décodage de `'PROVIDER_API'`, `'FUTUR_X'` (→ `Other`), d'un type de règle inconnu (→ `Other`, liste
     rendue, aucune proposition) ; un import `Other` s'affiche (code brut) et se supprime comme aujourd'hui.

3. **AC-3 — Migration non breaking `2026111X0000NN_provider_fetch.sql` (C-38-4, C-38-5, E5–E8, E12, E20).** DDL seul,
   de date **postérieure** à la migration breaking de 38-2 et à celle de 38-2 qui crée `payment_provider_accounts`
   (`NN` fixé au développement) ; **seule migration de cette story** (aucun `UPDATE`, aucun `MODIFY`).
   - **`bank_transactions`** (`migrations/20260504000001_bank_imports.sql:54-88`) gagne, toutes `NULL` (lues par
     listes de colonnes nommées — `repositories/bank_transactions.rs:21-24` `COLUMNS`, à étendre — : un binaire
     antérieur les ignore) : `provider_tx_id VARCHAR(64) CHARACTER SET ascii COLLATE ascii_bin`, `booked_at
     DATETIME(3)` (instant du prestataire, **UTC** ; `booking_date` = sa date à Europe/Zurich, NFR-38-3-6),
     `gross_amount DECIMAL(18,2)`, `fee_amount DECIMAL(18,2)` (signés comme ils changent le solde : `amount` = net =
     brut + frais, E7), `movement_kind VARCHAR(20)` (ascii_bin, format `^[a-z_]{1,20}$`, valeurs C-38-4 lues avec
     `Other`), `event_code VARCHAR(16)` (ascii_bin), `counterparty_email VARCHAR(254)`, `invoice_ref VARCHAR(127)` (B24),
     `item_names JSON` (`CHECK (item_names IS NULL OR (JSON_VALID(item_names) AND JSON_TYPE(item_names) = 'ARRAY'))`,
     tableau de chaînes — les noms d'articles, non la réponse brute, E8), `related_provider_tx_id VARCHAR(64)`
     (ascii_bin), `review_reason VARCHAR(30)` (ascii_bin, format, valeurs C-38-4). Contraintes :
     `UNIQUE uq_bank_transactions_provider_tx (bank_account_id, provider_tx_id)` (NULL multiples admis : CAMT, CSV,
     manuel inchangés, E6) ; `chk_bank_transactions_provider_amounts CHECK (provider_tx_id IS NULL OR (gross_amount IS
     NOT NULL AND fee_amount IS NOT NULL AND amount = gross_amount + fee_amount AND booked_at IS NOT NULL))` ; index
     `(company_id, related_provider_tx_id)` (lu par 38-6b). Entité `BankTransaction` (`entities/bank_transaction.rs`)
     : champs `Option` ; `item_names` se lit en chaîne JSON (`item_names_json: Option<String>`, patron de
     `column_mapping_json`, E24) et **l'accesseur `BankTransaction::item_names() -> Vec<String>`** (liste vide si
     `NULL`, `DbError::Invariant` sur un JSON illisible) est celui que lit 38-4 (AC-3 de 38-4).
   - **`provider_fetch_runs`** (applicative) : `id`, `company_id` (FK `RESTRICT`), `connection_id` (FK
     `payment_provider_connections` **`ON DELETE CASCADE`**, AC-6), `trigger_kind VARCHAR(10)` ascii_bin `CHECK BINARY
     IN ('daily','manual')`, `requested_by_user_id BIGINT NULL` (FK `users` `RESTRICT` ; `NULL` pour `daily`),
     `window_from`, `window_to` `DATETIME(3) NOT NULL` (UTC), `started_at DATETIME(3) NOT NULL`, `finished_at NULL`,
     `status VARCHAR(20)` ascii_bin (format seul, `running`|`succeeded`|`failed`, lu avec `Other`, E4), index
     `(connection_id, started_at)` (lecture des passages, AC-8, et recherche du dernier passage, AC-7), `error_code
     VARCHAR(40) NULL`, `error_detail VARCHAR(500) NULL` (jamais un secret ni une URL avec jeton), `inserted_count`,
     `known_count`, `skipped_unlinked_count`, `accounts_touched` `INT NOT NULL DEFAULT 0`.
   - **`bank_imports.provider_fetch_run_id BIGINT NULL`**, FK `provider_fetch_runs` `RESTRICT`, index ; l'entité
     `BankImport` et `COLUMNS` de `repositories/bank_imports.rs` (`:72` et sa constante) gagnent la colonne
     (`Option<i64>`), comme `COLUMNS` de `bank_transactions` (`repositories/bank_transactions.rs:21-24`) gagne les onze.
   - **`payment_provider_accounts.fetched_through DATETIME(3) NULL`** (UTC) — **ajout à l'epic** : le point de reprise
     **par compte** (un compte créé après coup repart de `fetch_from` ; un point par connexion le priverait de tout ce
     qui précède sa création, E12).
   - **`provider_balances_seen`** (applicative, E12) : `connection_id` (FK `CASCADE`), `currency CHAR(3)` ascii_bin,
     `first_seen_at`, `last_seen_at DATETIME(3)`, `last_amount DECIMAL(18,2)`, `ignored BOOLEAN NOT NULL DEFAULT FALSE`,
     `PRIMARY KEY (connection_id, currency)`.
   - **`provider_balance_snapshots`** (applicative, E20) : `id`, `connection_id` (FK `CASCADE`), `run_id` (FK
     `provider_fetch_runs` `CASCADE`), `currency`, `as_of DATETIME(3)` (UTC), `amount DECIMAL(18,2)`, `kind VARCHAR(10)`
     ascii_bin `CHECK BINARY IN ('opening','reported')` (`opening` = solde à `fetch_from`, lu une fois par compte),
     index `(connection_id, currency, as_of)`.
   - **Inscriptions** : `TABLES_TO_TRUNCATE` (`backup.rs:46`, enfants → parents) — `provider_balance_snapshots` puis
     `provider_balances_seen` avant `provider_fetch_runs`, elle-même **après `bank_imports`** (qui la référence,
     `:101`) et **avant `payment_provider_accounts`/`payment_provider_connections`** (38-1, 38-2) ; aucune à
     `RESET_PRESERVED_TABLES`. Export CSV : `bank_transactions` et `bank_imports` gagnent leurs colonnes
     (`csv_tables.rs:689-795`, en-têtes écrits à la main, cas de test) ; les trois tables neuves à
     `TABLES_HORS_EXPORT` (« historique de téléchargement et soldes rapportés par le prestataire, sans écriture
     comptable ; la sauvegarde les porte »). **Liste close R1** (E18) : les trois tables, `bank_imports.provider_fetch_run_id`,
     `payment_provider_accounts.fetched_through` et les onze colonnes de `bank_transactions` ; test d'import d'une
     sauvegarde d'avant l'Epic 38 étendu. **P5** (ligne d'audit), **P6** (inspection), **P7** (aucune écriture), squash
     `test-schema`.

4. **AC-4 — Crate `kesh-providers` : types, validation, temps (E9, C-38-11, NFR-38-3-5, NFR-38-3-6).** Crate neuf du
   workspace, `publish = false`, **sans dépendance à `kesh-db`** ni à `kesh-api` (dépend de `kesh-core` pour les
   réglages, 38-1) ; dépendances `chrono`, `chrono-tz` (déjà au `Cargo.lock`, 0.10.4, `:508-515` — à déclarer),
   `rust_decimal`, `serde`, `serde_json`, `thiserror`.
   - **Types** : `MovementKind { Sale, Purchase, Fee, Conversion, Refund, Dispute, Withdrawal, Deposit, Other(String) }`
     et `ReviewReason { Dispute, RefundOriginMissing, UnknownEvent, Other(String) }` (C-38-4, `as_str` / `FromStr`
     tolérant) ; `ProviderMovement { provider_tx_id, booked_at: DateTime<Utc>, currency, gross, fee, net,
     movement_kind, event_code, counterparty_name, counterparty_email, invoice_ref, item_names, reference, details,
     related_provider_tx_id, review_reason }` ; `ProviderBalance { currency, amount, as_of }` ; `Window { from, to }`
     (UTC, demi-ouverte `[from, to)`).
   - **Trait objet-safe** `Connector` (forme de `RateSource` de 37-2 AC-3 : `Pin<Box<dyn Future<…> + Send + '_>>`, sans
     `async_trait`) : `balances(&self, as_of: Option<DateTime<Utc>>) -> … Result<Vec<ProviderBalance>, ConnectorError>`
     et `movements(&self, window: Window, page: u32) -> … Result<MovementPage, ConnectorError>` (`MovementPage {
     movements, page, total_pages }`) ; `max_window()` (PayPal : 31 jours, 38-7). Trait objet-safe **`Transport`**
     (requête HTTP minimale → réponse bornée) dont l'implémentation `reqwest` vit dans `kesh-api` (AC-5).
     `ConnectorError` : `Auth`, `Permission`, `RateLimited { retry_after }`, `Unavailable`, `InvalidResponse { reason
     }`, `Transport { kind }` — codes constants, **jamais** le texte d'une erreur `reqwest` (qui porte l'URL, précédent
     37-2) ni un secret.
   - **Validation commune** `validate_page(page, window, expected_currencies)` : `provider_tx_id` non vide, ≤ 64,
     ASCII imprimable ; devise ISO à trois lettres majuscules ; montants à deux décimales au plus et
     `|montant| < 10^16` ; `net == gross + fee` ; `booked_at` dans la fenêtre ; textes bornés aux longueurs des
     colonnes (refus, jamais de troncature silencieuse d'un identifiant ; troncature **signalée** des textes libres
     `details`/`counterparty_name` à leur borne) ; `item_names` ≤ 50 noms de ≤ 255 ; **au plus 500 mouvements par page** (borne déclarée du connecteur, jamais lue de
     la réponse) ; pagination cohérente (`page ≤
     total_pages`, `total_pages` constant d'une page à l'autre d'une même fenêtre, ≤ un plafond déclaré par le
     connecteur) ; pas deux fois le même `provider_tx_id` dans une page. Échec → `InvalidResponse { reason }`, **aucune
     ligne de la page écrite**. Tests de réponses **hostiles** (C-38-11) : chacun des refus ci-dessus a son test.
   - **Temps** : `booking_date = booked_at` converti en `Europe/Zurich` (`chrono-tz`), date seule ; tests à 23 h 30 UTC
     le 31 décembre et aux deux changements d'heure.
   - **`FakeConnector`** (`#[doc(hidden)] pub`, précédent `MockRateSource`, 37-2 AC-3 — visible des tests
     d'intégration de `kesh-api`) : soldes et mouvements programmés par fenêtre, erreurs et pages incohérentes à la
     demande ; c'est lui que servent les tests et le mode test. **Aucun connecteur réel ici** : `paypal` arrive en
     38-7 ; en 38-3, une connexion `paypal` hors mode test termine son passage en `failed`, `error_code =
     "connector_unavailable"`.

5. **AC-5 — Transport et mode test (E9, E10, NFR-38-3-4).**
   - `kesh-api::providers::HttpTransport` implémente `Transport` en **réutilisant le client borné de 37-2** (rustls,
     `redirect(Policy::none())`, `https_only(true)`, délais), avec des bornes propres : délai total 30 s, réponse
     ≤ 8 MiB (lecture par morceaux bornée) ; l'URL de base est une **constante du connecteur** (38-7), aucune variable
     d'environnement ne la change ; `with_base_url` `#[doc(hidden)] pub` pour les tests contre un serveur axum local.
   - `AppState` gagne `connector_factory: Arc<dyn ConnectorFactory>` (construit le connecteur d'une connexion à partir
     du prestataire, des réglages typés de 38-1 et du secret ouvert par la **voie unique** de 38-1 AC-1 — la garde des
     appelants d'`open` gagne ce site) et `provider_fetch_guard` (AC-6). Sites de construction littérale d'`AppState` :
     inventaire de 37-2 AC-3, à relancer (`grep -rn "AppState {"`).
   - **`KESH_TEST_MODE`** : `main.rs` installe une fabrique qui rend un `FakeConnector` pour **toute** connexion,
     piloté par `POST /api/v1/_test/provider-connector` (corps : soldes, pages de mouvements par fenêtre, erreur ou
     délai à servir, par `connectionId`), monté avec les autres routes `_test` (`lib.rs:1434`,
     `routes/test_endpoints.rs:49-55`) ; `TEST_ENDPOINT_ROUTES` **+1** (`Exempt("chemin de test…")`). La tâche
     quotidienne **n'est pas lancée** en mode test (AC-7). Aucun appel réseau en CI : la garde est que seul
     `HttpTransport` ouvre une connexion, et qu'aucun test ne le construit sans `with_base_url` local (garde de source).

6. **AC-6 — Le passage (C-38-5, E5, E6, FR-38-3-1/2/3, NFR-38-3-3).** Module `kesh-api::providers::fetch`, fonction
   `run_fetch(state, connection_id, trigger, user)` :
   - **Un passage à la fois par connexion** : verrou en mémoire (`provider_fetch_guard`, ensemble des connexions en
     cours, `tokio::sync::Mutex`) **et** en base — la ligne `provider_fetch_runs` `running` se crée sous `SELECT … FOR
     UPDATE` de la connexion, refusée s'il en existe une **que le verrou en mémoire tient, ou qui a commencé il y a
     moins de 6 h** (`409 PROVIDER_FETCH_IN_PROGRESS`, `details.runId`) ; une ligne `running` plus ancienne et sans
     verrou en mémoire passe `failed`, `error_code = "interrupted"` à cette lecture. *Raison* : une sauvegarde
     importée (restauration complète, sans redémarrage) peut porter un passage `running` d'une autre instance, que
     ni le démarrage ni le verrou en mémoire ne verraient — la connexion resterait bloquée. Au démarrage du binaire
     (`main.rs` seulement), tout passage resté `running` passe de même `failed`, `error_code = "interrupted"`
     (`tracing::warn!`). Test : ligne `running` de 7 h posée par SQL → un passage démarre ; de 1 h → 409.
   - **Préalables** : connexion utilisable (38-1 AC-3 : prestataire connu, réglages lisibles, clé, secret) — sinon
     `409 PROVIDER_CONNECTION_UNUSABLE`, `details.reason` = `unusableReason` de 38-1 ; aucun passage créé.
   - **Fenêtre** : fin = maintenant − 3 h (délai de PayPal) ; début = **min sur les comptes prestataires actifs
     de la connexion** (non archivés, non clos) de `max(fetch_from à 00:00 Europe/Zurich, fetched_through − 3 jours)`
     (`fetched_through` `NULL` → `fetch_from`) ; découpée en sous-fenêtres de `max_window()` au plus. Aucun compte actif
     → passage `succeeded` sans mouvement (soldes lus quand même, AC-9).
   - **Pour chaque sous-fenêtre et chaque page** : `movements(window, page)` → `validate_page` (AC-4) → **une
     transaction** : pour chaque mouvement, compte bancaire = la ligne `payment_provider_accounts` (connexion, devise)
     d'un compte actif ; devise sans compte → compté `skipped_unlinked` et **non écrit** (E12, B2) ; un compte
     **archivé ou clos** entre la sélection et l'écriture est refusé par la même garde que `create_with_transactions`
     (`repositories/bank_imports.rs:~40-66`, Story 28-4a/29-4a : compte relu sous verrou, `NotFound` ou
     `BankAccountClosed`) — la transaction de la page est annulée, le passage se termine `failed`, `error_code =
     "account_unavailable"` (ajouté à la liste ci-dessous) ; le suivant n'élit que les comptes actifs ; mouvement
     d'un compte dont `fetched_through` dépasse déjà la fin de la sous-fenêtre → connu, sans effet (idempotent) ;
     **connu ou inséré se décide par une lecture, non par `rows_affected`** : sqlx pose `CLIENT_FOUND_ROWS`
     (`repositories/letterings.rs:1213`), si bien qu'un `ON DUPLICATE KEY UPDATE id = id` rend **1** pour une ligne
     insérée **et** pour une ligne connue — la distinction annoncée serait fausse. Par page et par compte :
     `SELECT provider_tx_id FROM bank_transactions WHERE bank_account_id = ? AND provider_tx_id IN (…)` (≤ 500, même
     transaction, verrou de passage tenu), puis `INSERT` des seuls identifiants absents ; le `ON DUPLICATE KEY UPDATE
     id = id` ne reste que comme filet d'une course impossible sous le verrou, et un test insère deux fois la même page
     pour vérifier `inserted_count` et `known_count` ; en-tête
     `bank_imports` du (passage, compte) **créé à la première insertion réelle** — jamais avant la lecture ci-dessus :
     une page entièrement connue ne crée aucun en-tête (« un en-tête sans transaction n'existe pas », ci-dessous), et
     l'en-tête déjà créé par une page précédente du même passage est réutilisé (E5 : `source_format = 'PROVIDER_API'`,
     `filename = telechargement-<provider>-<AAAA-MM-JJ>-<AAAA-MM-JJ>` (bornes de la fenêtre du passage, dates Zurich),
     `file_hash` provisoire = SHA-256 de `run:<id>:account:<id>`, `statement_id`, soldes `NULL`, période = fenêtre,
     `imported_by_user_id` = le demandeur, ou `created_by_user_id` de la connexion pour `daily` (colonne `NOT NULL`,
     `migrations/20260504000001_bank_imports.sql:33` ; un utilisateur désactivé reste une ligne valide),
     `provider_fetch_run_id`) ; **`INSERT … ON DUPLICATE KEY UPDATE id = id`** sur `uq_bank_transactions_provider_tx`
     — un mouvement connu n'est **jamais réécrit** (E6) ; colonnes de l'AC-3 remplies, `status = 'pending'`. À la fin
     de la sous-fenêtre (toutes pages) : `fetched_through` de chaque compte traité avancé à la fin de la
     sous-fenêtre, **dans la transaction de la dernière page** — ou, quand la fenêtre n'a aucune page (`total_pages =
     0`), dans une transaction à lui seul, pour que le point de reprise avance quand même ; point de reprise = ce champ.
   - **Fin** : en-têtes finalisés (`transaction_count`, `file_hash` = SHA-256 des enregistrements normalisés insérés,
     triés par `provider_tx_id` — forme écrite au module), passage `succeeded` ; `payment_provider_connections.last_run_at`,
     `last_run_status`, `last_error` mis à jour (38-1 AC-2). Un en-tête sans transaction insérée n'existe pas.
   - **Échec** (erreur du connecteur, page invalide, base) : ce qui est commité **reste** ; passage `failed`,
     `error_code` constant (`auth`, `permission`, `rate_limited`, `unavailable`, `invalid_response`, `transport`,
     `connector_unavailable`, `interrupted`, `account_unavailable`, `database`), `error_detail` sans secret ; `fetched_through` dit où reprendre.
     Le passage suivant repart de là (−3 jours). Test : échec à la page 3 de la 2ᵉ sous-fenêtre → pages 1-2 gardées,
     relance → aucun doublon, total final égal à un passage sans échec.
   - **Statut tardif** : un mouvement déjà connu n'est jamais modifié ; une inversion ou un remboursement arrive chez
     PayPal comme une **transaction distincte** (codes `T11xx`, 38-8) avec son propre identifiant et
     `related_provider_tx_id` — c'est elle que 38-6b traite. Écrit au module, test : un mouvement renvoyé avec d'autres
     montants sous le même identifiant laisse la ligne intacte et se compte `known` (et un `tracing::warn!` sans
     montant).
   - **Volume** : trois ans × trois devises (quelques milliers de mouvements) : insertion par page, une transaction par
     page ; test de charge synthétique (≥ 5 000 mouvements, 40 sous-fenêtres) dans un délai borné écrit au Change Log.
   - **Suppression d'une connexion** (38-1 AC-5) : ses passages, soldes vus et instantanés partent avec elle
     (cascade + `DELETE` explicite dans la transaction) — un passage référencé par un en-tête `bank_imports` l'empêche
     (FK `RESTRICT`), cas qui ne se présente pas : la connexion ne se supprime qu'une fois ses comptes prestataires
     retirés, et un compte qui a des transactions ne se retire pas (38-2 AC-5) ; le test le constate.

7. **AC-7 — Planification et déclenchement (E10, B9).**
   - **Tâche quotidienne** `providers::schedule::spawn(state)` lancée par `main.rs` **seulement**, à côté de
     `auto_backup::spawn` (`main.rs:467`, patron `auto_backup.rs:788-803` : `interval_at`, `MissedTickBehavior::Skip`,
     un tour par `tokio::spawn`), **jamais en `KESH_TEST_MODE`** ; garde lexicale au patron de
     `auto_backup::tests::lancement_par_main_seulement`. Cadence : toutes les heures ; une connexion `daily`
     utilisable dont le dernier passage (tout déclencheur) a commencé il y a plus de 24 h est passée, l'une après
     l'autre. Passage automatique **non audité** (`tracing` + état du passage), C-37-6 ; erreurs journalisées sans
     secret.
   - **À la demande** : `POST /api/v1/provider-connections/{id}/fetch` — **Comptable ou Administrateur, session
     seule** : dans `comptable_routes`, refus d'une clé d'API dans le handler par `ApiKeyOperationForbidden` (patron de
     `bank_imports::delete`, `routes/bank_imports.rs:1681-1690` ; NFR-38-6c-4 : les routes des connexions restent
     fermées aux clés) ; vaut pour `daily` **et** `manual` (B9). Réponse **`202`** `{ runId }` : le passage court
     dans un `tokio::spawn` (trois ans de fenêtres dépassent un délai HTTP raisonnable) ; refus synchrones :
     `404`, `409 PROVIDER_CONNECTION_UNUSABLE`, `409 PROVIDER_FETCH_IN_PROGRESS`. **Audit** : à la fin du passage,
     dans la transaction qui écrit son état, `provider_fetch.completed` (entité `provider_fetch_run`, utilisateur =
     demandeur, détail `{ connectionId, status, errorCode, insertedCount, knownCount, skippedUnlinkedCount, window }`).
     `LIB_ROUTES` **+1** `("post", "provider_connections::fetch", Traced, SansEcritureAuJournal)` (il écrit des
     transactions bancaires, non des écritures comptables).

8. **AC-8 — Lecture de l'état (FR-38-3-8, Q-38-1-a).**
   - `GET /api/v1/provider-fetch/connections` (Comptable, Administrateur ; clés d'API refusées) : par connexion de la
     société `{ id, provider, label, schedule, usable, unusableReason, lastRunAt, lastRunStatus, lastError,
     runningRunId }` — **ni `settings`, ni rien du coffre**. Les routes d'administration de 38-1 restent à l'Administrateur.
   - `GET /api/v1/provider-connections/{id}/runs?limit=` (mêmes droits) : les passages, plus récents d'abord (`limit`
     ≤ 100, défaut 20), colonnes de la table, `error_detail` compris (sans secret par construction, AC-6).

9. **AC-9 — Soldes vus, soldes rapportés, complétude (E12, E20 a, FR-38-3-4, FR-38-3-5).**
   - À chaque passage, **avant** les mouvements : `balances(Some(fin de fenêtre))` → une ligne `provider_balance_snapshots` `reported`
     par devise (lu **à la fin de la fenêtre**, voir ci-dessous) ; devise sans compte prestataire actif → `provider_balances_seen` (insérée ou `last_seen_at`,
     `last_amount` mis à jour ; `ignored` gardé). Au premier passage d'un compte (aucun `opening` pour sa devise) :
     `balances(Some(fetch_from à 00:00 Zurich))` → instantané `opening` (PayPal : `as_of_time`, 38-7). Un échec de
     lecture des soldes n'empêche pas les mouvements : le passage reste `succeeded`, `error_code` reste `NULL`
     (`balances_unavailable` n'est **pas** un code d'échec de l'AC-6) et le motif s'écrit dans `error_detail`
     (`balances_unavailable: <code du connecteur>`) ; contrôle « non disponible ».
   - ⚠️ **Heure des soldes rapportés** : la fenêtre du passage s'arrête à *maintenant − 3 h* (AC-6) ; un solde lu
     « maintenant » compterait des mouvements que le passage n'a pas pu télécharger et ferait de chaque passage un
     faux « téléchargement incomplet ». Le solde `reported` se lit donc **à la fin de la fenêtre du passage** :
     `balances(Some(fin))`, `as_of = fin` (le « solde actuel » `balances(None)` ne sert qu'à « Tester la connexion »,
     38-7). Test : un mouvement daté de *maintenant − 1 h*, compté dans le solde « actuel » mais absent de la fenêtre,
     ne fait pas apparaître d'écart.
   - **Contrôle de complétude**, calculé à la lecture (jamais stocké, jamais d'écriture comptable) : pour chaque compte
     prestataire, `attendu = opening + Σ amount` des transactions du compte de `booked_at ≥ fetch_from 00:00 Zurich`
     et `≤ as_of` du dernier `reported` ; `écart = reported − attendu` ; `≠ 0` → « téléchargement incomplet ». Route
     `GET /api/v1/provider-connections/{id}/balances` (Comptable, Administrateur ; **clés d'API refusées dans le
     handler**, `ApiKeyOperationForbidden`, comme l'AC-8 — NFR-38-6c-4) ; `complete: null` aussi quand
     `fetched_through` du compte est antérieur au `as_of` du `reported` (passage échoué ou en cours) : `{ accounts: [{ bankAccountId,
     currency, opening: {amount, asOf}|null, reported: {amount, asOf}|null, expected, gap, complete }], unlinked:
     [{ currency, firstSeenAt, lastSeenAt, lastAmount, ignored }] }` — `unlinked` exclut les devises désormais tenues
     par un compte. Tests : écart nul, écart d'un mouvement manquant (montant exact), compte sans `opening`
     (`complete: null`).
   - **Ignorer un solde** : `PATCH /api/v1/provider-connections/{id}/balances-seen/{currency}` `{ ignored }`
     (Administrateur, session : `admin_routes`) ; audit `provider_balance_seen.updated` ; `LIB_ROUTES` **+1**
     (`Traced, SansEcritureAuJournal`). « Créer » ou « lier » un compte pour ce solde passe par 38-2 (`POST
     …/accounts`) — **jamais de création d'office** (B2, *Un appariement automatique propose, il ne crée jamais*).

10. **AC-10 — En-tête `PROVIDER_API` partout où `source_format` se lit (E5, FR-38-3-3).**
    - **Suppression refusée** : `DELETE /api/v1/bank-imports/{id}` d'un en-tête portant `provider_fetch_run_id` →
      `409 PROVIDER_IMPORT_NOT_DELETABLE`, décidé **sous le verrou de l'en-tête** (première instruction de
      `delete_unreconciled_in_tx`, `crates/kesh-db/src/repositories/bank_imports.rs:571-590`), avant tout autre refus.
      *Raison* : le recouvrement de trois jours ne retéléchargerait pas ce que la suppression effacerait.
    - **Lecteurs** (inventaire de l'epic, recompté au commit : `source_format`/`SourceFormatTag` dans
      `routes/bank_imports.rs` ×21, `bank_statements_manual.rs` ×5, `kesh-core/src/bank_imports.rs` ×26,
      `csv_tables.rs` ×2, `repositories/bank_imports.rs` ×3, `backup.rs`, `admin.rs`, `invoices.rs`, `errors.rs`,
      `kesh-import` ×6) : chacun relu ; **aucun comportement ne dépend de la valeur** (affichage seul, relevé de
      l'epic) — sinon, le dire au Change Log. La liste et le détail des imports (`GET /api/v1/bank-imports…`) gagnent
      `providerConnection: { id, label, provider } | null` (jointure par `provider_fetch_run_id`) pour que 38-9a écrive
      « Téléchargé de <libellé> » ; le frontend (`frontend/src/lib/features/bank-import/bank-import-display.ts:14-36`,
      qui rend aujourd'hui le code brut pour toute valeur hors `MANUAL`) est **laissé à 38-9a** — d'ici là, la liste
      montre `PROVIDER_API` et le `filename` lisible ; écrit comme limite de la story.
    - **Audit** de la suppression refusée : aucun (rien n'est écrit) ; trace d'import : le passage tient lieu de trace
      (AC-7).

11. **AC-11 — Garde des chemins existants (FR-38-3-7).**
    - **Imports de fichier et relevé saisi** sur un compte prestataire (`POST /api/v1/bank-imports/preview`,
      `/bank-imports`, `/bank-imports/manual/preview`, `/bank-imports/manual`, `lib.rs:831-860`) → `409
      BANK_IMPORT_PROVIDER_ACCOUNT` (`details.bankAccountId`), décidé sur `is_provider_account_in_tx` (38-2 AC-2). Sans
      cette garde, une transaction de fichier (sans `provider_tx_id`) doublerait en silence un mouvement téléchargé
      (clé de doublon par fenêtre, `kesh-core/src/bank_imports.rs:295-360`, sans rapport avec E6). Répond au point de
      contact de 38-2 AC-7.
    - **Propositions existantes** : `get_proposals` (`routes/reconciliation.rs:999`) ne propose **ni facture, ni règle,
      ni ventilation** à une transaction d'un compte prestataire (`TxDocuments::Nothing` **et** pas de règle, à côté du
      saut hors CHF, `:1096-1104`) — une règle à deux lignes au **net** (`kesh-reconciliation/src/manual.rs:68`)
      fausserait B3 (brut par la règle, frais à part) ; `post_accept` refuse de même par proposition (`FailedProposal`
      `PROVIDER_TX_REQUIRES_PROVIDER_PROPOSAL`, *Pattern batch*). 38-6a lève ce saut pour sa variante `provider`
      (point de contact). Les voies manuelles restent : écriture manuelle (34-3), rattachement à une écriture (v017-3a,
      en CHF ; en devise par 38-6c), virement entre ses comptes (34-11b).

12. **AC-12 — Retour, recette, documentation (NFR-38-3-2).**
    - **Recette de montée** (`scripts/recette-montee.sh`, `docs/testing.md`) : la montée depuis l'**image publiée
      précédente**, le relèvement de `min_required`, le refus de l'image précédente et le retour par la sauvegarde sont
      ceux de **38-2** (O1) ; cette story **étend** le `--diff-attendu` à ses objets (`bank_transactions` et
      `bank_imports` : colonnes neuves ; `payment_provider_accounts` : `fetched_through` ; les trois tables neuves) et
      la rejoue au journal de la release.
    - **CHANGELOG** : une ligne de la même entrée non datée que 38-2 (la date est posée par `prepare-release.sh`) ;
      c'est 38-2 qui annonce la migration breaking, la version minimale relevée et la **sauvegarde d'avant la mise à
      jour comme seul retour**, au CHANGELOG et au manuel d'administration (38-9a).
    - **Registres** : `LIB_ROUTES` **+2** (`fetch`, `balances-seen`, toutes deux `Traced, SansEcritureAuJournal`),
      `TEST_ENDPOINT_ROUTES` **+1** ;
      décomptes en delta cumulés avec 38-1 et 38-2. Audit : types d'entité `provider_fetch_run`,
      `provider_balance_seen` ; actions `provider_fetch.completed`, `provider_balance_seen.updated` ; quatre locales.
      Codes neufs et messages aux quatre locales : `PROVIDER_FETCH_IN_PROGRESS`, `PROVIDER_CONNECTION_UNUSABLE`,
      `PROVIDER_IMPORT_NOT_DELETABLE`, `BANK_IMPORT_PROVIDER_ACCOUNT`, `PROVIDER_TX_REQUIRES_PROVIDER_PROPOSAL`,
      `RULE_CRITERION_UNKNOWN`. **MCP** : aucun outil neuf (E23) ; `kesh_ecritures` ne lit pas les transactions
      bancaires — rien à ajouter.
    - **38-2 AC-5** : `PROVIDER_ACCOUNT_IN_USE` gagne la raison `balance_seen` (instantanés du compte).

## Tâches / sous-tâches

- [ ] **T0 — Prérequis (O1)** : la migration breaking de 38-2 est fusionnée dans la branche et la version minimale
      relevée (relu au code : `kesh_version_min_required`, version des crates) ; sinon arrêt.
- [ ] **T1 — Migration non breaking** (AC-3) : SQL, squash, audit d'idempotence, P6, inscriptions, R1, exports CSV.
- [ ] **T2 — Workspace et version** (AC-1) : crate `kesh-providers` à la version courante des crates, `members`,
      commentaires de `prepare-release.sh` et `CLAUDE.md` (« 10 » → « 11 »), test de versions égales ; aucune
      migration breaking, aucun `EXEMPT_MIGRATIONS` (38-2).
- [ ] **T3 — Décodage tolérant** (AC-2) : deux enums, `SourceFormatTag`, `rule_matches`, routes des règles,
      `RULE_CRITERION_UNKNOWN`, perte de `Copy`.
- [ ] **T4 — `kesh-providers`** (AC-4) : types, traits, validation, fuseau, `FakeConnector`, tests hostiles.
- [ ] **T5 — Transport, fabrique, mode test** (AC-5) : `HttpTransport`, `ConnectorFactory`, `AppState`, route `_test`.
- [ ] **T6 — Passage** (AC-6) : verrous, fenêtres, pages, en-têtes, reprise, échec, statut tardif, charge.
- [ ] **T7 — Planification et route `fetch`** (AC-7) ; **lecture** (AC-8) ; **soldes et complétude** (AC-9).
- [ ] **T8 — En-tête et gardes** (AC-10, AC-11) : suppression refusée, lecteurs relus, `providerConnection` des
      imports, refus des imports de fichier, exclusion des propositions existantes.
- [ ] **T9 — Registres, audit, i18n, CHANGELOG** (AC-12) ; recette de montée jouée, journal sous `kesh-gate-logs/`.
- [ ] **T10 — Gates** : gate complet (migrations, dépôts : ciblage interdit) sur base remise à zéro, **gate runtime**
      (`admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install` : relus parce que la story ajoute une
      migration et des tables à l'inventaire de sauvegarde), `scripts/gate-ci.sh` (la CI lance
      `cargo test` par binaire), E2E complet au dernier commit de code (démarrage réel sur `kesh_e2e` : P8 et version).

## Notes de développement

- **Taille** : cette story touche **six** modules (`kesh-providers` neuf, `kesh-db`, `kesh-core`, `kesh-api`,
  `kesh-reconciliation` pour les bras de `rule_matches`, `kesh-i18n` pour les quatre locales) — au-dessus des cinq de
  la règle de découpage, même allégée de la migration breaking (O1) ; le frontend est laissé à 38-9a. **Découpage
  proposé** (Q-38-3-b ; décision à l'orchestrateur, qui renumérote le registre de l'epic) : **38-3a** — la migration
  non breaking, les colonnes de `bank_transactions`/`bank_imports`, le décodage tolérant (AC-2), la suppression
  refusée et les gardes de l'AC-11 (`kesh-db`, `kesh-core`, `kesh-reconciliation`, `kesh-api`) ; **38-3b** — le
  crate `kesh-providers`, le transport, le passage, la planification, la lecture de l'état, les soldes (AC-4 à AC-9).
  38-3b ne dépend de 38-3a que par les colonnes. Les deux restent dans l'Epic 38, avant 38-4 et 38-6a.
- **Point de reprise par compte** (`fetched_through`) plutôt que par connexion : sans lui, un compte créé pour un solde
  vu après coup (B2) ne recevrait jamais ce qui précède sa création. Le prix est de relire, pour une connexion, depuis
  le compte le plus en retard — l'insertion idempotente rend la relecture sans effet.
- **`provider_tx_id` en `ascii_bin`** : sous `utf8mb4_unicode_ci`, l'index unique confondrait des identifiants qui ne
  diffèrent que par la casse.
- **E6 : jamais d'écrasement.** Un `ON DUPLICATE KEY UPDATE` qui recopierait les colonnes ferait d'un statut tardif
  une réécriture silencieuse d'une transaction peut-être déjà rapprochée.
- **Heure** : le serveur n'a aucun fuseau aujourd'hui (`Utc::now().date_naive()`, `services/rapports.rs:862-870`) ;
  `chrono-tz` ne sert qu'à la date comptable d'un mouvement et à `fetch_from` à minuit de Zurich.
- **Références** : epic E4–E10, E12, E19, E20, C-38-0, C-38-4, C-38-5 ; 37-2 AC-3 (patron de source et client borné) ;
  `entities/bank_import.rs:17-110` ; `entities/reconciliation_rule.rs:24-100` ;
  `migrations/20260504000001_bank_imports.sql:19-88` ; `migrations/20260513000001_reconciliation_rules.sql:36-69` ;
  `repositories/bank_transactions.rs:21-24, 83` ; `repositories/bank_imports.rs:72, 571` ; `routes/bank_imports.rs:1681` ;
  `routes/reconciliation.rs:999, 1096-1104` ; `rules.rs:46-71` ; `auto_backup.rs:788-803` ; `main.rs:467` ;
  `lib.rs:831-860, 1434` ; `post_restore.rs:634-652, 1243-1256` ; `backup.rs:46, 99-105` ; `Cargo.toml:2-13` ;
  `scripts/prepare-release.sh:7, 96, 219`.

## Questions ouvertes

- *(Q1 — numéro de release de la migration breaking — est portée par 38-2 ; Q-38-3-a — `CHECK` de format et
  `ascii_bin` sur `match_type` — est tranchée par O2 et vit dans la migration de 38-2.)*
- **Q-38-3-b** — découpage préventif en 38-3a / 38-3b (notes) : **recommandé** (six modules) ; à décider.
- **Q-38-3-c** — refus des imports de fichier sur un compte prestataire (AC-11) : proposé ; une reprise d'historique
  antérieur aux trois ans de PayPal par CSV en serait empêchée (le responsable de l'epic part du 1.1.2024, B8 : sans effet pour lui).

## Change Log

- 2026-11-15 — rédaction (Opus). Écarts à l'epic écrits : onze crates bumpés (non dix) ; `CHECK` de format et
  `ascii_bin` sur `match_type` ; point de reprise par compte (`payment_provider_accounts.fetched_through`) ;
  version `<X.Y.Z>` en attente de Q1.
- 2026-11-15 — validation P1 (Sonnet) : 14 findings (2 HIGH, 6 MEDIUM, 6 LOW) ; remédiés dans la fiche sauf
  38-3-P1-7 (découpage, décision de l'orchestrateur) et 38-3-P1-14 (fiches sœurs). HIGH : P1-1 migration breaking
  déplacée en 38-2 (O1) — AC-1, NFR-38-3-1/2, AC-3, AC-12, T0/T2, questions Q1/Q-38-3-a, en-tête réalignés, aucune
  écriture de `_kesh_version` ni `MODIFY` ici ; P1-2 `rows_affected` ne distingue pas inséré et connu (sqlx pose
  `CLIENT_FOUND_ROWS`, `repositories/letterings.rs:1213` : `ON DUPLICATE KEY UPDATE id = id` rend 1 dans les deux
  cas) — lecture préalable des identifiants (AC-6). MEDIUM : P1-3 en-tête créé avant de savoir si une ligne est neuve
  (en-tête orphelin) ; P1-4 solde rapporté lu « maintenant » contre une fenêtre arrêtée à maintenant − 3 h (faux
  « incomplet ») — lu à la fin de la fenêtre (AC-9) ; P1-5 passage `running` importé par une sauvegarde bloquant la
  connexion (règle de péremption de 6 h, AC-6) ; P1-6 garde compte archivé/clos de `create_with_transactions` non
  reprise ; P1-7 six modules (`kesh-i18n` compté) : découpage 38-3a/38-3b recommandé ; P1-8 route `balances` sans
  refus des clés d'API. LOW : P1-9 fenêtre sans page n'avance pas `fetched_through` ; P1-10 `as_str` perd `'static`,
  `match &rule.match_type` ; P1-11 nombre de mouvements par page non borné ; P1-12 `balances_unavailable` ambigu
  (code d'erreur sur un passage réussi) ; P1-13 entité/`COLUMNS` de `bank_imports` et index des passages absents ;
  P1-14 (inter-fiche) `PATCH …/balances/{currency}` avec `version` de 38-9a contre `balances-seen/{currency}` de
  l'AC-9, et signe des frais de 38-6a (fixé ici : `fee_amount` signé comme il change le solde, négatif pour un frais).
  Axes non exercés : réponses réelles de PayPal (38-7), exécution de la migration sur MariaDB (aucun `cargo`).
- 2026-11-15 — découpage de 38-3 (D-P1-1, Opus) : statut `split` ; contenu repris par 38-3a et 38-3b, où sont
  appliqués les points inter-fiches de la P1 ; fiche gardée comme historique.
