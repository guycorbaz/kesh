# Story 38-3a : téléchargement — schéma, lecture de la source `PROVIDER_API` et gardes des chemins existants

Status: ready-for-dev

*Fiche née le 2026-11-15 du découpage de 38-3 (décision **D-P1-1** de l'orchestrateur, `38-p1-inter-fiches.md` :
`38-3-telechargement-cadre.md` touchait six modules). Branche `epic/38-paypal`, numéros de ligne relevés au commit
`8b3d548d` (repris de 38-3). Epic 38 : E4 (source seule), E5, E6 (schéma), E7, E8, E12 (schéma), E18, E20 (schéma) ;
contrats **C-38-0**, **C-38-4**, **C-38-5** (schéma, en-tête, gardes) ; décisions de Beat B2, B22 ; arbitrages **O1**,
O9. **Dépend de** : 38-1 (connexions), **38-2** (unique migration breaking de l'epic, `payment_provider_accounts`,
`is_provider_account_in_tx`), Epic 37 (liste close de 37-1 AC-6). **Suivie par** : **38-3b** (le passage, qui écrit
dans ce schéma), 38-4 (lit `counterparty_email`, `invoice_ref`, `item_names`, `event_code`), 38-6a, 38-6b, 38-6c1, 38-6c2
(lisent les colonnes ; 38-6a lève la garde de l'AC-6 pour sa variante `provider`), 38-9a (libellé « Téléchargé de … »). **Migration non breaking** (DDL seul).
Aucune route neuve, aucun écran.*

*Modules : `kesh-db`, `kesh-core` (`SourceFormatTag`), `kesh-api` (routes d'import, réconciliation, export CSV,
erreurs), `kesh-i18n` — quatre (règle de découpage : ≤ 5).*

## Story

En tant que **Comptable** d'une société qui tiendra ses soldes PayPal dans Kesh, je veux que la base sache porter les
mouvements d'un prestataire (identifiant unique, brut, frais, net, genre, payeur, facture, articles), ses passages et
ses soldes rapportés, que Kesh lise sans erreur l'en-tête d'un téléchargement et refuse de le supprimer, et qu'aucun
chemin existant (import de fichier, relevé saisi, propositions par facture ou par règle) ne touche un compte
prestataire, afin que le téléchargement de 38-3b écrive dans un schéma sûr et que rien ne double ni ne fausse ses
mouvements en attendant les propositions de 38-6a.

## Exigences

| Exigence | Texte | AC | Origine (38-3) |
|---|---|---|---|
| `FR-38-3a-1` | Le schéma porte les mouvements de prestataire (C-38-4 : identifiant du prestataire **unique par compte**, brut, frais, net, instant, genre, code d'événement, courriel, référence de facture, articles, lien, motif d'examen), les passages, les soldes vus et rapportés, et le point de reprise par compte. | AC-3 | FR-38-3-2 (schéma) |
| `FR-38-3a-2` | Un en-tête `PROVIDER_API` se lit avec sa connexion (`providerConnection`, liste et détail des imports) et sa suppression est refusée (`PROVIDER_IMPORT_NOT_DELETABLE`). | AC-5 | FR-38-3-3 (lecture, refus) |
| `FR-38-3a-3` | Une source d'import inconnue se lit sans erreur (E4) ; `PROVIDER_API` en est une valeur connue. | AC-2 | FR-38-3-6 (source ; le type de règle va en 38-4) |
| `FR-38-3a-4` | Un compte prestataire ne reçoit ni import de fichier ni relevé saisi à la main, et ses transactions ne reçoivent aucune des propositions existantes (facture, règle, ventilation) avant la variante `provider` de 38-6a. | AC-6 | FR-38-3-7 |
| `FR-38-3a-5` | Le retrait des réglages d'un compte dont un solde a été rapporté est refusé (`PROVIDER_ACCOUNT_IN_USE`, raison `balance_seen`) ; la suppression d'une connexion emporte ses passages, soldes vus et instantanés. | AC-4 | 38-2 AC-5 (point de contact), 38-1 AC-5 |
| `NFR-38-3a-1` | Migration **non breaking** (DDL seul, aucune écriture de données) ; la migration breaking de l'epic est celle de 38-2 (O1), dont cette story dépend. | AC-1, AC-3 | NFR-38-3-1 |
| `NFR-38-3a-2` | Une sauvegarde d'avant l'Epic 38 se réimporte (liste close R1) ; la recette de montée de 38-2 est étendue aux objets de cette story. | AC-3, AC-7 | NFR-38-3-2 |
| `NFR-38-3a-3` | Aucun comportement existant ne dépend de la valeur de `source_format` (lecteurs relus, affichage seul) ; un écart se dit au Change Log. | AC-5 | AC-10 de 38-3 |

## Critères d'acceptation

1. **AC-1 — Prérequis et périmètre (O1, D-P1-1, NFR-38-3a-1).** La migration breaking de l'epic est celle de 38-2
   (contrainte d'IBAN, `match_type` en `VARCHAR(40)`, relèvement de `kesh_version_min_required`) ; le fichier breaking
   de la rédaction de 38-3 n'existe plus (vérifié à la P1 : aucun `MODIFY`, `DROP` ni `UPDATE _kesh_version` dans
   cette story). 38-3a **ne contient ni P2, ni P2-bis, ni P7** (le détecteur P7 classe sur le premier mot-clé ; ses
   statements sont des `CREATE`/`ALTER … ADD`). **Test de garde** : la liste des migrations de la branche place celle
   de 38-2 avant celle de l'AC-3, et `kesh_version_min_required` y est déjà relevé. Ses écritures futures de
   `'PROVIDER_API'` (38-3b) s'appuient sur ce relèvement : `bank_imports.source_format` reste un `VARCHAR(32)` sans
   `CHECK` (`migrations/20260504000001_bank_imports.sql:25`), aucun DDL n'est requis, et un binaire antérieur, dont le
   `FromStr` refuse l'inconnu (`crates/kesh-db/src/entities/bank_import.rs:57-67`), ne démarre pas sur la base migrée.
   **Hors périmètre** : `ReconciliationMatchType` (décodage tolérant, quatre variantes neuves, `RULE_CRITERION_UNKNOWN`)
   arrive avec la première écriture d'un critère neuf, **en 38-4** (liste inter-fiches, point 1) ; le crate
   `kesh-providers`, le passage et ses routes sont **38-3b**.
   - **P6** (cette story ajoute une migration) : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`
     (relevé au commit : `migrations_upgrade_path.rs` ×27, `invoice_lines_revenue_account_backfill.rs` ×13,
     `lettering_documents_backfill.rs` ×6, `accounts_role_backfill.rs` ×5, `closing_accounts_backfill.rs` ×5,
     `pre_upgrade_backup/tests.rs` ×3, quatre autres ×2), chaque site inspecté, cité au Change Log. **P5** : ligne
     d'audit de l'AC-3. **P8** : aucune migration appliquée retouchée.

2. **AC-2 — Décodage tolérant de la source (E4, FR-38-3a-3).**
   - **`BankImportSourceFormat`** gagne `ProviderApi` (`"PROVIDER_API"`) et `Other(String)` ; `FromStr` ne refuse plus
     rien (`Other` pour l'inconnu) ; `Serialize`/`Deserialize` à la main (la chaîne brute, `Other` compris). `as_str`
     perd son `&'static str` (il rend `&str`, emprunté à `Other`) : `Display`, `Encode` (`entities/bank_import.rs:~85`)
     et les sites qui exigent `'static` suivent.
   - `kesh_core::bank_imports::SourceFormatTag` (`crates/kesh-core/src/bank_imports.rs:145-170`, aligné par contrat sur
     `as_db_str`) gagne `ProviderApi` ; `SourceFormatTag::as_db_str` reste `&'static str` (pas de `Other` côté tag) ;
     aucun chemin d'import de fichier ne le rend (`from_imported`, comme `Manual`).
   - **Perte de `Copy`** (l'enum porte un `String`) : inventaire au compilateur ; relevé au commit — 66 occurrences hors
     de son fichier, dans `backup.rs`, deux routes et 17 fichiers de tests. Corriger par `clone()` ou référence, **sans**
     réintroduire de `match` qui refuse l'inconnu.
   - Tests : décodage de `'PROVIDER_API'` et de `'FUTUR_X'` (→ `Other`) ; un import `Other` posé par SQL s'affiche (code
     brut) et se supprime comme aujourd'hui.

3. **AC-3 — Migration non breaking `<AAAAMMJJ>0000NN_provider_fetch.sql` (C-38-4, C-38-5, E5–E8, E12, E20).** DDL seul,
   de date **postérieure** à la migration breaking de 38-2 (`NN` fixé au développement) ; **seule migration de cette
   story** (aucun `UPDATE`, aucun `MODIFY`).
   - **`bank_transactions`** (`migrations/20260504000001_bank_imports.sql:54-88`) gagne, toutes `NULL` (lues par listes
     de colonnes nommées : un binaire antérieur les ignore) — ⚠️ **trois** listes décodent aujourd'hui en
     `BankTransaction` (`sqlx::FromRow` exige chaque champ de l'entité dans le `SELECT`, faute de `#[sqlx(default)]` :
     un champ neuf absent d'une liste fait échouer **à l'exécution** chaque lecture qui l'emploie) :
     `repositories/bank_transactions.rs:21-24` (`COLUMNS`, aussi lue par `reconciliation_cancel.rs:219`),
     `repositories/bank_imports.rs:34-37` (`TX_COLUMNS`, lecture des transactions d'un import, `:188`) et
     `repositories/reconciliation.rs:111` (`BANK_TX_COLUMNS`, propositions et lectures sous verrou, `:974, 1131, 1192,
     1261`) ; les deux dernières se **remplacent** par `bank_transactions::COLUMNS` (une seule définition, DRY) et un
     test lit une transaction de prestataire par chacun des sites (`grep -rn "query_as::<_, BankTransaction>" crates/`
     relevé au Change Log) : `provider_tx_id VARCHAR(64) CHARACTER SET ascii COLLATE ascii_bin`, `booked_at DATETIME(3)` (instant du
     prestataire, **UTC** ; `booking_date` en sera la date à Europe/Zurich, écrite par 38-3b), `gross_amount
     DECIMAL(18,2)`, `fee_amount DECIMAL(18,2)` (signés comme ils changent le solde : `amount` = net = brut + frais, E7 ;
     un frais est négatif), `movement_kind VARCHAR(20)` (ascii_bin, format `^[a-z_]{1,20}$`, valeurs C-38-4 lues avec
     `Other`), `event_code VARCHAR(16)` (ascii_bin), `counterparty_email VARCHAR(254)`, `invoice_ref VARCHAR(127)`
     (B24), `item_names JSON` (`CHECK (item_names IS NULL OR (JSON_VALID(item_names) AND JSON_TYPE(item_names) =
     'ARRAY'))`, tableau de chaînes — les noms d'articles, non la réponse brute, E8), `related_provider_tx_id
     VARCHAR(64)` (ascii_bin), `review_reason VARCHAR(30)` (ascii_bin, format, valeurs C-38-4). Contraintes : `UNIQUE
     uq_bank_transactions_provider_tx (bank_account_id, provider_tx_id)` (NULL multiples admis : CAMT, CSV, manuel
     inchangés, E6) ; `chk_bank_transactions_provider_amounts CHECK (provider_tx_id IS NULL OR (gross_amount IS NOT
     NULL AND fee_amount IS NOT NULL AND amount = gross_amount + fee_amount AND booked_at IS NOT NULL))` ; index
     `(company_id, related_provider_tx_id)` (lu par 38-6b). Entité `BankTransaction` (`entities/bank_transaction.rs`) :
     champs `Option` ; `item_names` se lit en chaîne JSON (`item_names_json: Option<String>`, patron de
     `column_mapping_json`, E24) et **l'accesseur `BankTransaction::item_names() -> Result<Vec<String>,
     DbError>`** (`Ok` d'une liste vide si `NULL`, `Err(DbError::Invariant)` sur un JSON illisible) est celui que lisent
     38-4 (AC-3 de 38-4) et 38-6c1 (AC-6, `itemNames` de la recherche).
   - **`provider_fetch_runs`** (applicative) : `id`, `company_id` (FK `RESTRICT`), `connection_id` (FK
     `payment_provider_connections` **`ON DELETE CASCADE`**, AC-4), `trigger_kind VARCHAR(10)` ascii_bin `CHECK BINARY
     IN ('daily','manual')`, `requested_by_user_id BIGINT NULL` (FK `users` `RESTRICT` ; `NULL` pour `daily`),
     `window_from`, `window_to` `DATETIME(3) NOT NULL` (UTC), **`truncated_from DATETIME(3) NULL`** (UTC : début
     effectif quand le passage a été borné par l'historique du prestataire, 38-7 AC-9 ; `NULL` sinon), `started_at
     DATETIME(3) NOT NULL`, `finished_at NULL`, `status VARCHAR(20)` ascii_bin (format seul, `running`|`succeeded`|
     `failed`, lu avec `Other`, E4), `error_code VARCHAR(40) NULL` (ascii_bin, format `^[a-z_]{1,40}$` ; valeurs de
     38-3b AC-4), `error_detail VARCHAR(500) NULL` (jamais un secret ni une URL avec jeton), `inserted_count`,
     `known_count`, `skipped_unlinked_count`, **`ignored_count`** (enregistrements écartés par le connecteur : refusés,
     en attente, hors solde, 38-8 AC-1), `accounts_touched` `INT NOT NULL DEFAULT 0` ; index `(connection_id,
     started_at)` (lecture des passages et recherche du dernier, 38-3b).
   - **`bank_imports.provider_fetch_run_id BIGINT NULL`**, FK `provider_fetch_runs` **`RESTRICT`**, index ; l'entité
     `BankImport` et `COLUMNS` de `repositories/bank_imports.rs` (`:30-32`, seule liste qui décode en `BankImport`,
     huit lectures, `:176-577`) gagnent la colonne (`Option<i64>`), comme `COLUMNS` de `bank_transactions` gagne les
     onze.
   - **`payment_provider_accounts.fetched_through DATETIME(3) NULL`** (UTC) — **ajout à l'epic** : le point de reprise
     **par compte** (un compte créé après coup repart de `fetch_from` ; un point par connexion le priverait de tout ce
     qui précède sa création, E12). L'entité et le dépôt de 38-2 (AC-2) la lisent ; l'export CSV
     `payment_provider_accounts.csv` de 38-2 (dont le cas de test confronte l'en-tête au schéma) **la gagne**.
   - **`provider_balances_seen`** (applicative, E12) : `connection_id` (FK `CASCADE`), `currency CHAR(3)` ascii_bin,
     `first_seen_at`, `last_seen_at DATETIME(3)`, `last_amount DECIMAL(18,2)`, `ignored BOOLEAN NOT NULL DEFAULT FALSE`,
     `PRIMARY KEY (connection_id, currency)`.
   - **`provider_balance_snapshots`** (applicative, E20) : `id`, `connection_id` (FK `CASCADE`), `run_id` (FK
     `provider_fetch_runs` `CASCADE`), `currency`, `as_of DATETIME(3)` (UTC), `amount DECIMAL(18,2)`, `kind VARCHAR(10)`
     ascii_bin `CHECK BINARY IN ('opening','reported')` (`opening` = solde à `fetch_from`, lu une fois par compte),
     index `(connection_id, currency, as_of)`.
   - **Entités** (lecture) : `ProviderFetchRun` (`status` et `error_code` tolérants), `ProviderBalanceSeen`,
     `ProviderBalanceSnapshot`, et les dépôts de **lecture** qu'exigent l'AC-4 et l'AC-5 ; les écritures sont à 38-3b.
   - **Inscriptions** : `TABLES_TO_TRUNCATE` (`backup.rs:46`, enfants → parents) — `provider_balance_snapshots` puis
     `provider_balances_seen` avant `provider_fetch_runs`, elle-même **après `bank_imports`** (qui la référence,
     `:101`) et **avant `payment_provider_accounts`/`payment_provider_connections`** (38-1, 38-2) ; aucune à
     `RESET_PRESERVED_TABLES`. Export CSV : `bank_transactions` et `bank_imports` gagnent leurs colonnes
     (`csv_tables.rs:689-795`, en-têtes écrits à la main, cas de test) ; les trois tables neuves à `TABLES_HORS_EXPORT`
     (« historique de téléchargement et soldes rapportés par le prestataire, sans écriture comptable ; la sauvegarde les
     porte »). **Liste close R1** (E18) : les trois tables, `bank_imports.provider_fetch_run_id`,
     `payment_provider_accounts.fetched_through` et les onze colonnes de `bank_transactions` ; test d'import d'une
     sauvegarde d'avant l'Epic 38 étendu. **P5** (ligne d'audit, total aux deux sites et partition recomptés à la
     source), **P6** (AC-1), **P7** (aucune écriture), squash `test-schema` (`test_schema_guard.rs`). Sites de
     dénombrement (même grep que 38-1/38-2) : `admin_full_export_e2e.rs:~281` (`data_count`, +3 cumulés),
     `onboarding_audit_e2e.rs:2046` (`EMPTY_BEFORE_RESET`). Test de schéma des `CHECK` (montants, `item_names`,
     `kind`, `trigger_kind`, formats) au patron de `sans_iban_les_invariants_du_schema`.

4. **AC-4 — Suppression d'une connexion et retrait des réglages (FR-38-3a-5) — `RESTRICT` ou suppression des passages :
   tranché, suppression.** Les passages, soldes vus et instantanés **partent avec leur connexion** : FK `ON DELETE
   CASCADE` (AC-3) **et**, dans la transaction de `DELETE /api/v1/provider-connections/{id}` (38-1 AC-5), un `DELETE`
   explicite de `provider_balance_snapshots`, `provider_balances_seen` puis `provider_fetch_runs` de la connexion
   avant la ligne de la connexion (la cascade ne joue pas sous `FOREIGN_KEY_CHECKS = 0` ; patron de l'effacement du
   coffre de 38-1). *Raison* : ces lignes n'ont de sens que par leur connexion et ne portent aucune écriture comptable ;
   les garder imposerait un `RESTRICT` qui rendrait toute connexion déjà téléchargée insupprimable sans rien protéger.
   Le **seul** `RESTRICT` est celui de `bank_imports.provider_fetch_run_id` : un passage qui a inséré une transaction
   ne part pas. Ce cas ne se présente pas — la connexion ne se supprime qu'une fois ses comptes prestataires retirés
   (38-1 AC-5, 38-2 AC-9), et un compte qui a des transactions ne se retire pas (38-2 AC-5) — et l'erreur 1451 de cette
   FK, si elle survenait, se mappe sur `PROVIDER_CONNECTION_IN_USE` (nom de contrainte). Test : connexion sans compte,
   avec passages, soldes vus et instantanés posés par SQL → suppression `204`, tables vides pour elle ; passage portant
   un en-tête → refus, rien d'effacé.
   - **Retrait des réglages** (38-2 AC-5) : `PROVIDER_ACCOUNT_IN_USE` gagne la raison **`balance_seen`** — il existe
     une ligne `provider_balance_snapshots` (`opening` ou `reported`) de la connexion **dans la devise du compte**
     (instantanés compris, liste inter-fiches) ; `details.reasons` reste complète et triée. Message relu aux quatre
     locales. Conséquence écrite (manuel, 38-9a) : une connexion dont un passage a lu les soldes d'un compte ne se
     supprime plus ; on la met en `manual`.

5. **AC-5 — En-tête `PROVIDER_API` partout où `source_format` se lit (E5, FR-38-3a-2, NFR-38-3a-3).**
   - **Suppression refusée** : `DELETE /api/v1/bank-imports/{id}` d'un en-tête portant `provider_fetch_run_id` → `409
     PROVIDER_IMPORT_NOT_DELETABLE`, décidé **sous le verrou de l'en-tête** (première instruction de
     `delete_unreconciled_in_tx`, `crates/kesh-db/src/repositories/bank_imports.rs:571-590`), avant tout autre refus.
     *Raison* : le recouvrement de 38-3b ne retéléchargerait pas ce que la suppression effacerait. Aucun audit (rien
     n'est écrit). Test : en-tête et transactions posés par SQL.
   - **Lecteurs** (inventaire de l'epic, recompté au commit : `source_format`/`SourceFormatTag` dans
     `routes/bank_imports.rs` ×21, `bank_statements_manual.rs` ×5, `kesh-core/src/bank_imports.rs` ×26, `csv_tables.rs`
     ×2, `repositories/bank_imports.rs` ×3, `backup.rs`, `admin.rs`, `invoices.rs`, `errors.rs`, `kesh-import` ×6) :
     chacun relu ; **aucun comportement ne dépend de la valeur** (affichage seul) — sinon, le dire au Change Log.
   - La liste et le détail des imports (`GET /api/v1/bank-imports…`) gagnent `providerConnection: { id, label, provider
     } | null` (jointure par `provider_fetch_run_id`) pour que 38-9a écrive « Téléchargé de <libellé> » ; le frontend
     (`frontend/src/lib/features/bank-import/bank-import-display.ts:14-36`) est **laissé à 38-9a** — d'ici là, la liste
     montre `PROVIDER_API` et le `filename` lisible ; limite écrite. Le DTO des **transactions** (`TransactionResponse`,
     `routes/bank_imports.rs:302-345`) **n'est pas étendu ici** : son extension aux champs de prestataire est portée par
     38-6c1 (AC-6).

6. **AC-6 — Garde des chemins existants (FR-38-3a-4, O9).**
   - **Imports de fichier et relevé saisi** sur un compte prestataire (`POST /api/v1/bank-imports/preview`,
     `/bank-imports`, `/bank-imports/manual/preview`, `/bank-imports/manual`, `lib.rs:831-860`) → `409
     BANK_IMPORT_PROVIDER_ACCOUNT` (`details.bankAccountId`), décidé sur `is_provider_account_in_tx` (38-2 AC-2) sous
     le verrou du compte. Sans cette garde, une transaction de fichier (sans `provider_tx_id`) doublerait en silence un
     mouvement téléchargé (clé de doublon par fenêtre, `kesh-core/src/bank_imports.rs:295-360`, sans rapport avec E6).
     Répond au point de contact de 38-2 AC-7 (le sens inverse — réglages posés sur un compte qui a déjà des
     transactions — est refusé par 38-2 AC-3, `PROVIDER_ACCOUNT_HAS_TRANSACTIONS`). Le test de course (c) de 38-2 AC-6
     (retrait des réglages contre import CSV) change de sens ici : il devient « import contre retrait » — l'import voit
     les réglages et refuse, ou le retrait passe avant et l'import est admis ; réécrit dans cette story.
   - **Propositions existantes** : `get_proposals` (`routes/reconciliation.rs:999`) ne propose **ni facture, ni règle,
     ni ventilation** à une transaction d'un compte prestataire (`TxDocuments::Nothing` **et** pas de règle, à côté du
     saut hors CHF, `:1096-1104`) — une règle à deux lignes au **net** (`kesh-reconciliation/src/manual.rs:68`)
     fausserait B3 (brut par la règle, frais à part) ; `post_accept` refuse de même par proposition (`FailedProposal`
     **`PROVIDER_PROPOSAL_REQUIRED`**, le code de C-38-8 que reprend 38-6a AC-8 — un seul code pour ce refus, posé ici ;
     `details: { movementKind }`, `null` tant que 38-3b n'écrit rien —, *Pattern batch*). **38-6a** (point de contact)
     lève ce saut pour sa variante `provider` ; **`rule` et `split` restent refusés** sur un compte prestataire (même
     code) ; une facture (client ou fournisseur) n'y est proposée **que si `fee_amount = 0`**, sinon la proposition
     `provider` (D-P2-2, 38-6a AC-8 ; le règlement d'une facture au brut avec ligne de frais est tracé en issue,
     C-38-13). Les voies manuelles restent : écriture manuelle (34-3), rattachement à une écriture (v017-3a, en CHF ;
     en devise par 38-6c1), virement entre ses comptes (34-11b). Tests : transaction posée par SQL sur un compte
     prestataire CHF → aucune proposition ; lot qui la vise → `failed[]` avec le code.

7. **AC-7 — Registres, i18n, recette, documentation (NFR-38-3a-2).**
   - **Routes** : aucune neuve ; `LIB_ROUTES` et `TEST_ENDPOINT_ROUTES` inchangés.
   - **Codes neufs** et messages aux quatre locales : `PROVIDER_IMPORT_NOT_DELETABLE`, `BANK_IMPORT_PROVIDER_ACCOUNT`,
     `PROVIDER_PROPOSAL_REQUIRED` (C-38-8, repris par 38-6a) ; raison `balance_seen` de `PROVIDER_ACCOUNT_IN_USE`.
   - **Manuel** : aucun écran ici ; les refus visibles (import ou relevé saisi refusé sur un compte prestataire,
     suppression d'un en-tête téléchargé refusée, connexion qui ne se supprime plus après une lecture de soldes) sont
     écrits au manuel utilisateur et d'administration par **38-9a** avec les écrans, PDF contrôlé aplati ; 38-3a lui
     transmet la liste.
   - **Recette de montée** (`scripts/recette-montee.sh`, `docs/testing.md`) : la montée depuis l'image publiée
     précédente, le relèvement, le refus de l'image précédente et le retour par la sauvegarde sont ceux de **38-2**
     (O1) ; cette story **étend** le `--diff-attendu` à ses objets (`bank_transactions`, `bank_imports` : colonnes
     neuves ; `payment_provider_accounts.fetched_through` ; les trois tables neuves) et la rejoue au journal de la
     release.
   - **CHANGELOG** : une ligne de la même entrée non datée que 38-2 (la date est posée par `prepare-release.sh`).
   - **MCP** : aucun outil neuf (E23) ; `kesh_ecritures` ne lit pas les transactions bancaires — rien à ajouter.

## Tâches / sous-tâches

- [ ] **T0 — Prérequis (O1)** : la migration breaking de 38-2 est fusionnée dans la branche et la version minimale
      relevée (relu au code) ; sinon arrêt.
- [ ] **T1 — Migration non breaking** (AC-3) : SQL, squash, audit d'idempotence et compteurs, P6, inscriptions, R1,
      sites de dénombrement, exports CSV, test des `CHECK`.
- [ ] **T2 — Entités et lectures** (AC-3) : colonnes des entités et `COLUMNS`, `item_names()`, entités des trois tables.
- [ ] **T3 — Décodage tolérant de la source** (AC-2) : `BankImportSourceFormat`, `SourceFormatTag`, perte de `Copy`.
- [ ] **T4 — Suppression et retrait** (AC-4) : `DELETE` explicites dans la suppression d'une connexion, mappage 1451,
      raison `balance_seen`.
- [ ] **T5 — En-tête et gardes** (AC-5, AC-6) : suppression refusée, lecteurs relus, `providerConnection`, refus des
      imports de fichier, exclusion des propositions existantes.
- [ ] **T6 — Registres, i18n, CHANGELOG** (AC-7) ; recette de montée étendue, journal sous `kesh-gate-logs/`.
- [ ] **T7 — Gates** : gate complet (migration, dépôts : ciblage interdit) sur base remise à zéro, **gate runtime**
      (`admin_backup_e2e`, `admin_full_import_e2e`, `migrations_fresh_install`), `scripts/gate-ci.sh`, E2E complet au
      dernier commit de code (démarrage réel sur `kesh_e2e` : P8).

## Notes de développement

- **Découpage (D-P1-1)** : 38-3a pose le schéma et les gardes ; 38-3b (crate `kesh-providers`, trait `Connector`,
  transport, passage, planification, lecture de l'état, soldes) n'a **aucune migration** et ne dépend de 38-3a que par
  le schéma et les entités. Les deux restent avant 38-4 et 38-6a.
- **Point de reprise par compte** (`fetched_through`) plutôt que par connexion : sans lui, un compte créé pour un solde
  vu après coup (B2) ne recevrait jamais ce qui précède sa création.
- **`provider_tx_id` en `ascii_bin`** : sous `utf8mb4_unicode_ci`, l'index unique confondrait des identifiants qui ne
  diffèrent que par la casse.
- **Tests sans passage** : les gardes et lectures se testent sur des lignes posées par SQL (aucune écriture de
  prestataire n'existe avant 38-3b).
- **Références** : epic E4–E8, E12, E18, E20, C-38-0, C-38-4, C-38-5 ; `entities/bank_import.rs:17-110` ;
  `migrations/20260504000001_bank_imports.sql:19-88` ; `repositories/bank_transactions.rs:21-24, 83` ;
  `repositories/bank_imports.rs:30-37, 571` ; `repositories/reconciliation.rs:111` ; `routes/reconciliation.rs:999, 1096-1104` ; `lib.rs:831-860` ;
  `backup.rs:46, 99-105` ; `csv_tables.rs:689-795` ; `kesh-core/src/bank_imports.rs:145-170, 295-360`.

## Questions ouvertes

- **Q-38-3-c** (repris de 38-3, arbitré par O9) — refus des imports de fichier sur un compte prestataire (AC-6) : une
  reprise d'historique antérieur aux trois ans de PayPal par CSV en serait empêchée (Beat part du 1.1.2024, B8 : sans
  effet pour lui).

## Change Log

- 2026-11-15 — découpage de 38-3 (D-P1-1, Opus). Reprend de 38-3 : AC-1 (moins le crate), AC-2 (source seule), AC-3,
  AC-10, AC-11, la part « suppression d'une connexion » de l'AC-6 et la part schéma de l'AC-12 ; exigences
  renumérotées (`FR-38-3a-1` à `-5`, `NFR-38-3a-1` à `-3`, origine en colonne). Points inter-fiches appliqués :
  fichier breaking absent (vérifié) ; `provider_fetch_runs` gagne `truncated_from` et `ignored_count` ;
  `error_code` au format seul ; extension de `TransactionResponse` laissée à 38-6c ; `PROVIDER_ACCOUNT_IN_USE` gagne
  `balance_seen` (instantanés) ; **tranché : suppression des passages, soldes vus et instantanés avec leur connexion**
  (cascade + `DELETE` explicite), seul `bank_imports.provider_fetch_run_id` en `RESTRICT`. Retiré vers **38-4** :
  `ReconciliationMatchType` tolérant, ses quatre variantes, `RULE_CRITERION_UNKNOWN` (liste inter-fiches, point 1).
- 2026-11-15 — validation P2 (Opus), première passe de cette fiche née du découpage : 0 CRITICAL, 0 HIGH, 2 MEDIUM,
  3 LOW ; remédiés dans la fiche (aucun report inter-fiches : 38-6a et C-38-8 portent déjà le code retenu). MEDIUM :
  deux codes pour un même refus — `PROVIDER_TX_REQUIRES_PROVIDER_PROPOSAL` ici, `PROVIDER_PROPOSAL_REQUIRED` en C-38-8
  et 38-6a AC-8, qui réadmet `invoice`, `supplier_invoice` et `split` → code de C-38-8, levée de 38-6a écrite (AC-6,
  AC-7) ; **trois** listes de colonnes décodent en `BankTransaction` (`bank_transactions.rs:21`,
  `bank_imports.rs:34` `TX_COLUMNS`, `reconciliation.rs:111` `BANK_TX_COLUMNS`), la fiche n'en nommait qu'une : un
  champ neuf de l'entité absent des deux autres casse l'import et les propositions à l'exécution → les trois
  inventoriées, ramenées à une (AC-3). LOW : `COLUMNS` de `BankImport` à `:30`, non `:72` ; `fetched_through` dans
  l'export CSV de 38-2 ; manuel nommé (38-9a) ; course (c) de 38-2 reprise ici ; `onboarding_audit_e2e.rs:2046`.
  Lignes lues : `20260504000001_bank_imports.sql:19-88` ; `entities/bank_import.rs:24, 41, 57, 79` ;
  `kesh-core/src/bank_imports.rs:145, 160, 193` ; `repositories/bank_imports.rs:30-37, 70-76, 176-188, 560-600` ;
  `repositories/bank_transactions.rs:21-34` ; `repositories/reconciliation.rs:110-111, 969-974, 1131, 1192, 1261` ;
  `reconciliation_cancel.rs:219-222` ; `routes/reconciliation.rs:855-880, 995-1002, 1090-1110` ; `lib.rs:825-862,
  875-932` ; `routes/bank_imports.rs:304` ; `csv_tables.rs:689, 736` ; `backup.rs:95-110, 1060-1215` ;
  `38-6a-…md:296-310`.
- 2026-11-15 — alignement inter-fiches P2 (Opus) : D-P2-1 — `PROVIDER_PROPOSAL_REQUIRED` seul code, confirmé (AC-6,
  AC-7) ; point de contact de l'AC-6 récrit selon D-P2-2 — 38-6a lève le saut pour sa variante `provider`, `rule` et
  `split` restent refusés, factures proposées seulement si `fee_amount = 0` (le « réadmet `invoice`,
  `supplier_invoice` et `split` » de la P2 est caduc) ; `BankTransaction::item_names() -> Result<Vec<String>,
  DbError>` (la signature disait `Vec<String>` en annonçant une erreur) ; renvois 38-6c → 38-6c1 (AC-5, AC-6) et
  38-6c1/38-6c2 (en-tête), D-P2-3.
