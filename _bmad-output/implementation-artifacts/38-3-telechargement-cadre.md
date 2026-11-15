# Story 38-3 : téléchargement — cadre commun des prestataires, et la migration breaking

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`, numéros de ligne relevés au commit `8b3d548d`. Epic 38 :
décisions E4, E5, E6, E7, E8, E9, E10, E12, E18, **E19**, E20 (a) ; contrats **C-38-0**, **C-38-4**, **C-38-5**,
C-38-11, C-38-12 ; décisions de Beat B2, B8, B9, B12, B20, **B22**. **Dépend de** : 38-1, 38-2, l'Epic 37 (client HTTP
borné et patron « source objet-safe + factice en mode test + route `_test` » de 37-2 AC-3, C-37-6 ; liste close de
37-1 AC-6) et du **numéro de release fixé par Guy (Q1)**. **Suivie par** : 38-4 (évalue les critères neufs déclarés
ici), 38-6a/b/c (propositions et rattachement des transactions téléchargées), 38-7 (connecteur PayPal réel), 38-8
(classement des codes PayPal), 38-9a (écrans, libellé « Téléchargé de … », sauvegarde d'avant la mise à jour au
manuel), 38-10 (relevé joint).*

⛔ **Prérequis bloquant — Q1 (Guy).** La migration breaking fige dans son SQL la version `<X.Y.Z>` de la release qui
la porte (P2) et impose le bump des crates **dans le même commit** (P2-bis). Proposition de l'epic : `0.18.0`. Cette
fiche écrit `<X.Y.Z>` partout ; la tâche T0 le remplace par la valeur obtenue, **avant toute ligne de code**. Si la
valeur change après coup, la migration, non publiée, se corrige délibérément (P8 : bases de dev reconstruites).

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
| `NFR-38-3-1` | La seule migration breaking de l'epic : `UPDATE _kesh_version … kesh_version_min_required` en dernière instruction, crates bumpés dans le même commit, gate runtime complet avant `done` (P2, P2-bis). | AC-1 |
| `NFR-38-3-2` | Retour à la version précédente par la seule sauvegarde d'avant la mise à jour ; recette de montée depuis l'image publiée précédente. | AC-12 |
| `NFR-38-3-3` | Un passage à la fois par connexion ; un échec garde l'inséré et dit où reprendre. | AC-6 |
| `NFR-38-3-4` | Aucun appel réseau en CI ; transport factice piloté par une route `_test`, inactif hors `KESH_TEST_MODE`. | AC-5, AC-7 |
| `NFR-38-3-5` *(ajout)* | Toute réponse d'un prestataire est validée avant écriture ; une page invalide n'écrit rien et fait échouer le passage avec un état lisible (R5 de 37, C-38-11). | AC-4 |
| `NFR-38-3-6` *(ajout)* | La date comptable d'un mouvement est sa date à Europe/Zurich ; l'instant reçu est gardé en UTC. | AC-3, AC-4 |

## Critères d'acceptation

1. **AC-1 — La migration breaking `2026111X000004_provider_api_source_and_rule_types.sql` (E19, C-38-0, NFR-38-3-1).**
   Fichier **à part** de la migration non breaking de l'AC-3 (pour que l'exemption P7 ne couvre qu'elle), numéro de
   date postérieur à elle.
   - En tête : `-- BREAKING (P1, P3 : MODIFY COLUMN) : relève kesh_version_min_required à <X.Y.Z> (dernière
     instruction).` puis la raison, au patron de `20261110000001_bank_accounts_without_iban.sql:1-6` et de
     `20261030000001_bank_imports_manual_source.sql` : un binaire antérieur décode `bank_imports.source_format` et
     `reconciliation_rules.match_type` par un `FromStr` **qui refuse l'inconnu**
     (`crates/kesh-db/src/entities/bank_import.rs:57-67`, `entities/reconciliation_rule.rs:66-77`) — toute liste
     d'imports ou de règles échouerait sur `'PROVIDER_API'` ou sur un critère neuf.
   - (1) `ALTER TABLE reconciliation_rules MODIFY COLUMN match_type VARCHAR(40) CHARACTER SET ascii COLLATE ascii_bin
     NOT NULL, ADD CONSTRAINT chk_reconciliation_rules_match_type_format CHECK (BINARY match_type REGEXP
     '^[a-z][a-z_]{1,39}$')` — aujourd'hui `ENUM('counterparty_contains','counterparty_exact','reference_contains',
     'iban_exact')` (`migrations/20260513000001_reconciliation_rules.sql:39`). Les quatre valeurs restent, **aucune
     ligne réécrite**. *Écart léger à l'epic* (E14 : « `VARCHAR(40)` sans `CHECK` ») : le `CHECK` proposé est de
     **format seul**, sans énumération — Wise et Stripe ajoutent un critère sans DDL (E4) — et la collation `ascii_bin`
     ferme la casse et l'espace final que `utf8mb4_unicode_ci` confondrait dans l'index unique et que le décodage
     exact refuserait (précédent `BINARY` de 34-11a1 et de 37-1) ; à retirer si Beat tient à la lettre de E14.
     **Index** : `uq_reconciliation_rules_match_active (company_id, match_type, active_uniq)` (`:65-66`) se reconstruit
     avec l'`ALTER` (MariaDB, `ALGORITHM=COPY`) — **aucun `DROP INDEX`** ; un test lit `information_schema.STATISTICS`
     après la migration (l'index existe, mêmes colonnes) et vérifie qu'une règle active en double reste refusée.
   - (2) **Aucun DDL** pour `bank_imports.source_format` (`VARCHAR(32)` sans `CHECK`,
     `migrations/20260504000001_bank_imports.sql:25`) : seul le relèvement protège un binaire antérieur.
   - (3) **En dernière instruction** : `UPDATE _kesh_version SET kesh_version_min_required = '<X.Y.Z>' WHERE id = 1;`
     (P2). **P3/P4** : pas de `breaking-skip-bump`. **P2-bis** : les crates du workspace passent à `<X.Y.Z>` **dans le
     même commit** — ⚠️ *écart à l'epic* : **onze** crates et non dix, `kesh-providers` (AC-5) naissant à `<X.Y.Z>`
     dans ce même commit ; `scripts/prepare-release.sh` lit `crates/*/Cargo.toml` par motif (`:96`) et ne compte rien,
     seuls ses commentaires disent « 10 » (`:7`, `:219`) — à corriger, comme `CLAUDE.md` § *Repository Structure*
     (« workspace de 10 crates ») et le membre neuf au `Cargo.toml` racine (`members`, `:2-13`).
   - **P5** : ligne à `docs/migrations-idempotence-audit.md` (`tracked-by-sqlx` ; **BREAKING** écrit en clair, comme la
     ligne de `20261110000001`, `:113`) ; compteurs recomptés à la source. **P6** : `grep -rn
     "migrations.len()\|apply_migrations_up_to" crates/` (relevé au commit : `migrations_upgrade_path.rs` ×27,
     `invoice_lines_revenue_account_backfill.rs` ×13, `lettering_documents_backfill.rs` ×6,
     `accounts_role_backfill.rs` ×5, `closing_accounts_backfill.rs` ×5, `pre_upgrade_backup/tests.rs` ×3, quatre autres
     ×2) — chaque site inspecté, cité au Change Log. **P7** : `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`,
     patron des entrées `20261030000001`, `:634-637`, et `20261110000001`, `:646-652`), `ExemptionBasis::Durable`,
     justification « UPDATE sur _kesh_version (bump à <X.Y.Z>), table système jamais restaurée ; le reste est du DDL
     (match_type en VARCHAR, contrainte de format) : aucune valeur à recalculer » ; l'assertion de taille du registre
     (`:1252-1256`, **24** au commit de relevé) suit en delta, cumulée avec l'Epic 37. **P8** : aucune migration
     appliquée retouchée.
   - **Valeur à greper** (CLAUDE.md, *Propagation*) : `grep -rn '"0\.17\.6"' crates` rend les assertions de version
     minimale de `migrations_upgrade_path.rs` (`:552-620, 704, 1144, 1654, 2598`), `migrations_fresh_install.rs` et
     `admin_full_import_e2e.rs:4650` — chacune trie : relevé historique (garder) ou valeur courante (passer à
     `<X.Y.Z>`). Gate **runtime complet** avant `done` : `admin_backup_e2e`, `admin_full_import_e2e`,
     `migrations_fresh_install` (P2-bis), plus le démarrage réel contre une base persistante (E2E).
   - **Retour** : un binaire antérieur refuse la base migrée (`check_downgrade_protection`) et un `.keshbackup` de
     cette version (version minimale du manifeste, `routes/admin.rs`, étape 3b) ; **le retour passe uniquement par la
     sauvegarde d'avant la mise à jour** (`backup/pre-upgrade/`, #847) — dit au manuel par 38-9a (E19) ; **aucun SQL
     de retour** (retiré par B22).

2. **AC-2 — Décodage tolérant (E4, FR-38-3-6).**
   - **`BankImportSourceFormat`** gagne `ProviderApi` (`"PROVIDER_API"`) et `Other(String)` ; `FromStr` ne refuse
     plus rien (`Other` pour l'inconnu) ; `Serialize`/`Deserialize` à la main (la chaîne brute, `Other` compris).
     `kesh_core::bank_imports::SourceFormatTag` (`crates/kesh-core/src/bank_imports.rs:145-170`, aligné par contrat sur
     `as_db_str`) gagne `ProviderApi` ; aucun chemin d'import de fichier ne le rend (`from_imported`, comme `Manual`).
   - **`ReconciliationMatchType`** gagne les **quatre variantes déclarées** de C-38-6 — `CounterpartyEmailExact`
     (`counterparty_email_exact`), `InvoicePrefix` (`invoice_prefix`), `ItemNameContains` (`item_name_contains`),
     `EventCodePrefix` (`event_code_prefix`) — et `Other(String)`. **Évaluation** : `rule_matches`
     (`crates/kesh-reconciliation/src/rules.rs:46-71`) rend **`false`** pour les quatre neuves et pour `Other` (test
     nommé par variante) ; 38-4 les évalue. **Écriture** (`routes/reconciliation_rules.rs`, `matchType` lu à `:63`,
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

3. **AC-3 — Migration non breaking `2026111X000003_provider_fetch.sql` (C-38-4, C-38-5, E5–E8, E12, E20).** DDL seul.
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
     : champs `Option`.
   - **`provider_fetch_runs`** (applicative) : `id`, `company_id` (FK `RESTRICT`), `connection_id` (FK
     `payment_provider_connections` **`ON DELETE CASCADE`**, AC-6), `trigger_kind VARCHAR(10)` ascii_bin `CHECK BINARY
     IN ('daily','manual')`, `requested_by_user_id BIGINT NULL` (FK `users` `RESTRICT` ; `NULL` pour `daily`),
     `window_from`, `window_to` `DATETIME(3) NOT NULL` (UTC), `started_at DATETIME(3) NOT NULL`, `finished_at NULL`,
     `status VARCHAR(20)` ascii_bin (format seul, `running`|`succeeded`|`failed`, lu avec `Other`, E4), `error_code
     VARCHAR(40) NULL`, `error_detail VARCHAR(500) NULL` (jamais un secret ni une URL avec jeton), `inserted_count`,
     `known_count`, `skipped_unlinked_count`, `accounts_touched` `INT NOT NULL DEFAULT 0`.
   - **`bank_imports.provider_fetch_run_id BIGINT NULL`**, FK `provider_fetch_runs` `RESTRICT`, index.
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
     `details`/`counterparty_name` à leur borne) ; `item_names` ≤ 50 noms de ≤ 255 ; pagination cohérente (`page ≤
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
     UPDATE` de la connexion, refusée s'il en existe une (`409 PROVIDER_FETCH_IN_PROGRESS`, `details.runId`). Au
     démarrage du binaire, un passage resté `running` passe `failed`, `error_code = "interrupted"` (`tracing::warn!`).
   - **Préalables** : connexion utilisable (38-1 AC-3 : prestataire connu, réglages lisibles, clé, secret) — sinon
     `409 PROVIDER_CONNECTION_UNUSABLE`, `details.reason` = `unusableReason` de 38-1 ; aucun passage créé.
   - **Fenêtre** : fin = maintenant − 3 h (délai de PayPal) ; début = **min sur les comptes prestataires actifs
     de la connexion** (non archivés, non clos) de `max(fetch_from à 00:00 Europe/Zurich, fetched_through − 3 jours)`
     (`fetched_through` `NULL` → `fetch_from`) ; découpée en sous-fenêtres de `max_window()` au plus. Aucun compte actif
     → passage `succeeded` sans mouvement (soldes lus quand même, AC-9).
   - **Pour chaque sous-fenêtre et chaque page** : `movements(window, page)` → `validate_page` (AC-4) → **une
     transaction** : pour chaque mouvement, compte bancaire = la ligne `payment_provider_accounts` (connexion, devise)
     d'un compte actif ; devise sans compte → compté `skipped_unlinked` et **non écrit** (E12, B2) ; mouvement d'un
     compte dont `fetched_through` dépasse déjà la fin de la sous-fenêtre → réécrit sans effet (idempotent) ; en-tête
     `bank_imports` du (passage, compte) **créé à la première insertion** (E5 : `source_format = 'PROVIDER_API'`,
     `filename = telechargement-<provider>-<AAAA-MM-JJ>-<AAAA-MM-JJ>` (bornes de la fenêtre du passage, dates Zurich),
     `file_hash` provisoire = SHA-256 de `run:<id>:account:<id>`, `statement_id`, soldes `NULL`, période = fenêtre,
     `imported_by_user_id` = le demandeur, ou `created_by_user_id` de la connexion pour `daily` (colonne `NOT NULL`,
     `migrations/20260504000001_bank_imports.sql:33` ; un utilisateur désactivé reste une ligne valide),
     `provider_fetch_run_id`) ; **`INSERT … ON DUPLICATE KEY UPDATE id = id`** sur `uq_bank_transactions_provider_tx`
     — un mouvement connu n'est **jamais réécrit** (E6) ; `rows_affected` distingue inséré et connu ; colonnes de
     l'AC-3 remplies, `status = 'pending'`. À la fin de la sous-fenêtre (toutes pages) : `fetched_through` de chaque
     compte traité avancé à la fin de la sous-fenêtre, **dans la transaction de la dernière page** ; point de reprise
     = ce champ.
   - **Fin** : en-têtes finalisés (`transaction_count`, `file_hash` = SHA-256 des enregistrements normalisés insérés,
     triés par `provider_tx_id` — forme écrite au module), passage `succeeded` ; `payment_provider_connections.last_run_at`,
     `last_run_status`, `last_error` mis à jour (38-1 AC-2). Un en-tête sans transaction insérée n'existe pas.
   - **Échec** (erreur du connecteur, page invalide, base) : ce qui est commité **reste** ; passage `failed`,
     `error_code` constant (`auth`, `permission`, `rate_limited`, `unavailable`, `invalid_response`, `transport`,
     `connector_unavailable`, `interrupted`, `database`), `error_detail` sans secret ; `fetched_through` dit où reprendre.
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
   - À chaque passage, **avant** les mouvements : `balances(None)` → une ligne `provider_balance_snapshots` `reported`
     par devise ; devise sans compte prestataire actif → `provider_balances_seen` (insérée ou `last_seen_at`,
     `last_amount` mis à jour ; `ignored` gardé). Au premier passage d'un compte (aucun `opening` pour sa devise) :
     `balances(Some(fetch_from à 00:00 Zurich))` → instantané `opening` (PayPal : `as_of_time`, 38-7). Un échec de
     lecture des soldes n'empêche pas les mouvements : passage `succeeded`, `error_code = "balances_unavailable"`
     n'est **pas** un échec — écrit dans `error_detail`, contrôle « non disponible ».
   - **Contrôle de complétude**, calculé à la lecture (jamais stocké, jamais d'écriture comptable) : pour chaque compte
     prestataire, `attendu = opening + Σ amount` des transactions du compte de `booked_at ≥ fetch_from 00:00 Zurich`
     et `≤ as_of` du dernier `reported` ; `écart = reported − attendu` ; `≠ 0` → « téléchargement incomplet ». Route
     `GET /api/v1/provider-connections/{id}/balances` (Comptable, Administrateur) : `{ accounts: [{ bankAccountId,
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
    - **Recette de montée** (`scripts/recette-montee.sh`, `docs/testing.md`) depuis l'**image publiée précédente**,
      jouée à cette story et au journal de la release : sauvegarde `pre-upgrade/` écrite avant la migration ;
      `--diff-attendu` : `migrations`, `min_required`, `bank_transactions`, `bank_imports`, `reconciliation_rules` (type
      de colonne), `payment_provider_accounts`, plus les tables neuves ; refus de l'image précédente sur la base migrée
      et sur un `.keshbackup` de cette version ; retour par le dump et import du `.keshbackup` de l'image précédente,
      empreintes identiques.
    - **CHANGELOG** : entrée non datée (la date est posée par `prepare-release.sh`) qui annonce la migration breaking,
      la version minimale relevée et la **sauvegarde d'avant la mise à jour comme seul retour** ; le manuel
      d'administration (38-9a) en fait autant.
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

- [ ] **T0 — Q1** : version `<X.Y.Z>` obtenue de Guy, remplacée dans la fiche et au Change Log ; sinon arrêt.
- [ ] **T1 — Migration non breaking** (AC-3) : SQL, squash, audit d'idempotence, P6, inscriptions, R1, exports CSV.
- [ ] **T2 — Migration breaking et version** (AC-1) : SQL, `EXEMPT_MIGRATIONS` et sa taille, bump des onze crates dans
      le même commit, `members`, commentaires de `prepare-release.sh` et `CLAUDE.md`, grep de `"0.17.6"` trié, test de
      l'index unique reconstruit.
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
      (`admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install`), `scripts/gate-ci.sh` (la CI lance
      `cargo test` par binaire), E2E complet au dernier commit de code (démarrage réel sur `kesh_e2e` : P8 et version).

## Notes de développement

- **Taille** : cette story touche cinq modules (`kesh-providers` neuf, `kesh-db`, `kesh-core`, `kesh-api`,
  `kesh-reconciliation` pour les bras de `rule_matches`) — à la limite de la règle de découpage, sans la franchir ; le
  frontend est laissé à 38-9a. **Découpage proposé si la validation ne converge pas** (Q-38-3-b) : **38-3a** — les
  deux migrations, le décodage tolérant, les colonnes, la suppression refusée, les gardes de l'AC-11 ; **38-3b** — le
  crate, le transport, le passage, la planification, les soldes.
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

- **Q1 (Guy, bloquante)** — numéro de la release `<X.Y.Z>` (proposition : `0.18.0`).
- **Q-38-3-a** — `CHECK` de format sur `match_type` et collation `ascii_bin` (AC-1) : écart léger à E14 (« sans
  `CHECK` »), proposé ; à confirmer.
- **Q-38-3-b** — découpage préventif en 38-3a / 38-3b (notes) : à décider à la validation si elle stagne.
- **Q-38-3-c** — refus des imports de fichier sur un compte prestataire (AC-11) : proposé ; une reprise d'historique
  antérieur aux trois ans de PayPal par CSV en serait empêchée (Beat part du 1.1.2024, B8 : sans effet pour lui).

## Change Log

- 2026-11-15 — rédaction (Opus). Écarts à l'epic écrits : onze crates bumpés (non dix) ; `CHECK` de format et
  `ascii_bin` sur `match_type` ; point de reprise par compte (`payment_provider_accounts.fetched_through`) ;
  version `<X.Y.Z>` en attente de Q1.
