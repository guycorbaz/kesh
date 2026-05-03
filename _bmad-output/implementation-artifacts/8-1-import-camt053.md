# Story 8.1: Import CAMT.053

Status: ready-for-dev

<!-- Note: Validation est optionnelle. Lancer `bmad-create-story validate` pour une revue qualité multi-passes avant `dev-story`. -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse)**,
I want **importer mes relevés bancaires au format CAMT.053 (ISO 20022, versions 001.04 et 001.08)**,
so that **les transactions apparaissent dans Kesh, prêtes pour la réconciliation (Stories 8-3/8-4) sans ressaisie manuelle**.

### Contexte

**Story 8-1 = première story d'Epic 8 (Import bancaire & Réconciliation).** Epic créé 2026-05-02 par migration de la section legacy « Epic 7 » dans `epics.md` (renumérotage 2026-04-20 — décisions rétro Epic 5 et Epic 6). Voir [`_bmad-output/planning-artifacts/epic-8.md`](../planning-artifacts/epic-8.md).

**Pourquoi maintenant :** Epic 7 (Tech Debt Closure) a livré les 5 fondations critiques attendues (multi-tenant scoping, FULLTEXT, update no-op, deadlock-retry, SELECT FOR UPDATE). Le **prep sprint Epic 8** a clos les 4/4 critical path items qui bloquaient Story 8-1 :

| Item prep sprint | Statut | Impact Story 8-1 |
|---|---|---|
| Spec planning artifact `epic-8.md` | ✅ done (2026-05-02) | ACs cadrés, 8 risques R1-R8 listés |
| KF-020 SELECT FOR UPDATE invoices::update [#49](https://github.com/guycorbaz/kesh/issues/49) | ✅ closed via PR #64 (commit `ebdea4b`) | Pattern foundation pour mutex 8-4 |
| KF-002-H-002 deadlock-retry middleware [#43](https://github.com/guycorbaz/kesh/issues/43) | ✅ closed via PR #65 (commit `5893515`) | Helper `kesh_db::retry::retry_on_deadlock` disponible |
| Spike `kesh-import` crate design | ✅ done (PR #66, commit `15f11b4`) — verdict `feasible` | Types autonomes + `BankTransactionDraft` scaffold posés |
| CR-010 #62 (statement balance check FR42-bis) | ⚠️ ouvert — décision intégrée dans cette story (cf. §balance-check) | AC #14 |

**Spike outcome (résumé) :** `kesh-import` est publiable indépendamment, types autonomes (`ImportedStatement`, `ImportedTransaction`, `SourceFormat`), conversion `From<ImportedTransaction> for BankTransactionDraft` côté `kesh-core::bank_imports`. Vérifié : `cargo publish --dry-run -p kesh-import` passe, `cargo test -p kesh-import` 7/7, `cargo test -p kesh-core bank_imports` 4/4. Voir [`spike-kesh-import.md`](spike-kesh-import.md) pour la décision archi #7 confirmée et les RS-1 à RS-4 résiduels.

**Status sprint :** `epic-8: backlog → in-progress` (cette story est la première de l'epic, automatique), `8-1-import-camt053: backlog → ready-for-dev` à la fin de cette spec.

### Scope verrouillé — ce qui RESTE à faire

Story 8-1 livre **uniquement le parseur CAMT.053 + persistance + UI d'import + prévisualisation**. La détection de doublons (Story 8-3), la réconciliation (Story 8-4), le matching auto (Story 8-4) et les règles d'affectation (Story 8-5) **ne sont PAS dans le scope**. Voir §Scope HORS story.

1. **Fixtures CAMT.053** — créer `crates/kesh-import/tests/fixtures/camt053/` avec **au minimum** :
   - `v04_minimal.xml` — 1 `<BkToCstmrStmt>`, 1 compte CH IBAN, 3 `<Ntry>` (1 crédit + 2 débits), namespace `urn:iso:std:iso:20022:tech:xsd:camt.053.001.04`.
   - `v08_minimal.xml` — équivalent pour `camt.053.001.08`.
   - `v04_with_subtxs.xml` — 1 `<Ntry>` agrégée avec 3 `<TxDtls>` (FR49).
   - `v04_multi_stmt.xml` — 1 fichier avec 2 `<Stmt>` pour 2 IBAN différents (cf. décision §multi-stmt).
   - `v04_balance_mismatch.xml` — `<OpngBal> + Σ<Ntry> ≠ <ClsgBal>` à 0.05 près (CR-010 #62 régression test).
   - `v04_truncated.xml` — fichier coupé en plein milieu d'un `<Ntry>` (XML mal formé → erreur parseur).
   - `v04_invalid_iban.xml` — 1 `<Ntry>` avec `<RltdPties><Cdtr>` IBAN au checksum cassé (cf. décision §iban-tolerant).
   - `v04_eur_currency.xml` — 1 `<Stmt>` en EUR (cf. décision §currency).

   **Source pour les fixtures** : le dépôt contient l'XSD officiel `docs/six-references/ig-cash-managment-xml-schemas-v2.0.2-en/camt.053.001.08.xsd` mais **aucun sample XML customer-bank**. Construire les fixtures à la main à partir du XSD (minimum d'éléments obligatoires) — c'est l'usage standard pour les parseurs. Documenter la provenance synthétique dans un `crates/kesh-import/tests/fixtures/README.md`. **Ne pas inclure de données bancaires réelles** (PII).

2. **Parseur `kesh-import::camt053`** — implémenter :
   - `crates/kesh-import/src/camt053/mod.rs` — point d'entrée `pub fn parse(xml: &[u8]) -> Result<Vec<ImportedStatement>, CamtError>`. Détecte la version via le namespace racine `xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.XX"` et dispatch vers le parseur de version.
   - `crates/kesh-import/src/camt053/v04.rs` — parseur v04 (champs requis : `<Stmt><Id>`, `<Acct><Id><IBAN>`, `<Acct><Ccy>`, `<FrToDt>`, `<Bal>` open/close, `<Ntry>` × N).
   - `crates/kesh-import/src/camt053/v08.rs` — parseur v08 (delta avec v04 documenté inline ; en pratique v08 partage 95% du schéma au niveau des tags utilisés ici, le delta principal est `<Othr>` `SchmeNm` enrichi — non utilisé v0.1).
   - `crates/kesh-import/src/error.rs` — enum `CamtError` (`MalformedXml`, `UnsupportedVersion(String)`, `MissingRequiredField(&'static str)`, `InvalidAmount(String)`, `InvalidDate(String)`).
   - `crates/kesh-import/src/lib.rs` — exposer `pub mod camt053; pub mod error;`. Garder le scaffold spike intact (`mod types`, re-exports).
   - **Dépendance ajoutée** : `quick-xml = "0.36"` (feature `serialize` non requise — on lit en mode pull-parser pour streaming et minimisation mémoire). **Ne pas ajouter** `serde_xml_rs` (déprécié) ni `xml-rs` (lent, pas streaming).
   - `crates/kesh-import/Cargo.toml` : ajouter `quick-xml = "0.36"` en `[dependencies]`. Aucun ajout dans `[dev-dependencies]` au-delà du spike (`rust_decimal_macros`, `serde_json`).
   - **Invariant à préserver** : zéro dépendance workspace interne (`grep "kesh-" Cargo.toml | grep -v "name = "` doit retourner vide). Test CI ajouté en T8.

3. **Extensions `kesh-core::bank_imports`** — étendre le scaffold spike :
   - `crates/kesh-core/src/bank_imports.rs` — ajouter `BankImportDraft` (méta-fichier : `file_hash: String`, `filename: String`, `imported_at: DateTime<Utc>`, `bank_account_id: i64`, `company_id: i64`, `source_format: SourceFormat`).
   - Ajouter `pub fn from_imported(stmt: &ImportedStatement, bank_account_id: i64, company_id: i64, file_hash: String, filename: String) -> (BankImportDraft, Vec<BankTransactionDraft>)` — wrapper qui injecte les FK dans tous les drafts du statement et retourne le couple `(import_meta, transactions)` à persister atomiquement.
   - Ajouter `pub fn validate_balance(stmt: &ImportedStatement) -> Result<(), CoreError>` — implémente CR-010 #62 : si `opening_balance.is_some() && closing_balance.is_some()`, vérifier `|opening + sum_transactions - closing| <= 0.01`. Erreur `CoreError::BankImportBalanceMismatch { opening, closing, sum, diff }`.
   - Ajouter `pub fn validate_currency_supported_v0_1(stmt: &ImportedStatement) -> Result<(), CoreError>` — n'accepte que `"CHF"` v0.1 (cf. décision §currency). Erreur `CoreError::BankImportUnsupportedCurrency(String)`.
   - Étendre `crates/kesh-core/src/errors.rs` : ajouter les 2 variantes ci-dessus à l'enum `CoreError`.
   - **Conserver l'invariant spike** : `kesh-import` ne dépend pas de `kesh-core` (vérifié par cargo metadata), seul `kesh-core` connaît `kesh-import`. Ne PAS déplacer `validate_balance` dans `kesh-import`.

4. **Migration DB** — nouvelle migration `crates/kesh-db/migrations/2026MMDD000001_bank_imports.sql` :

   ```sql
   CREATE TABLE bank_imports (
       id BIGINT NOT NULL AUTO_INCREMENT,
       company_id BIGINT NOT NULL,
       bank_account_id BIGINT NOT NULL,
       filename VARCHAR(255) NOT NULL,
       file_hash CHAR(64) NOT NULL,                  -- SHA-256 hex
       source_format VARCHAR(32) NOT NULL,           -- 'camt053_v04', 'camt053_v08' (v0.1) ; 'csv' Story 8-2
       statement_id VARCHAR(255) NULL,               -- <Stmt><Id> CAMT, NULL pour CSV
       period_from DATE NOT NULL,
       period_to DATE NOT NULL,
       opening_balance DECIMAL(18,2) NULL,
       closing_balance DECIMAL(18,2) NULL,
       transaction_count INT NOT NULL DEFAULT 0,
       imported_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
       imported_by_user_id BIGINT NOT NULL,
       PRIMARY KEY (id),
       CONSTRAINT fk_bank_imports_company
           FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
       CONSTRAINT fk_bank_imports_bank_account
           FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
       CONSTRAINT fk_bank_imports_user
           FOREIGN KEY (imported_by_user_id) REFERENCES users(id) ON DELETE RESTRICT,
       CONSTRAINT uq_bank_imports_company_hash UNIQUE (company_id, file_hash),  -- support FR43 doublon fichier (Story 8-3 finalise)
       CONSTRAINT chk_bank_imports_period CHECK (period_to >= period_from),
       INDEX idx_bank_imports_company_account_imported (company_id, bank_account_id, imported_at)
   ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

   CREATE TABLE bank_transactions (
       id BIGINT NOT NULL AUTO_INCREMENT,
       company_id BIGINT NOT NULL,
       import_id BIGINT NOT NULL,
       bank_account_id BIGINT NOT NULL,
       booking_date DATE NOT NULL,
       value_date DATE NULL,
       amount DECIMAL(18,2) NOT NULL,           -- signé : positif = crédit titulaire
       currency CHAR(3) NOT NULL,
       reference VARCHAR(255) NULL,
       details TEXT NOT NULL,                    -- toujours présent même vide
       end_to_end_id VARCHAR(255) NULL,
       transaction_id VARCHAR(255) NULL,         -- <AcctSvcrRef> CAMT
       counterparty_iban VARCHAR(34) NULL,       -- chaîne brute, non validée DB
       counterparty_name VARCHAR(255) NULL,
       status VARCHAR(16) NOT NULL DEFAULT 'pending',     -- 'pending' | 'reconciled' (Story 8-4)
       matched_entry_id BIGINT NULL,                       -- FK journal_entries (Story 8-4)
       version INT NOT NULL DEFAULT 1,
       created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
       updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
       PRIMARY KEY (id),
       CONSTRAINT fk_bank_transactions_company
           FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
       CONSTRAINT fk_bank_transactions_import
           FOREIGN KEY (import_id) REFERENCES bank_imports(id) ON DELETE CASCADE,
       CONSTRAINT fk_bank_transactions_bank_account
           FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
       CONSTRAINT fk_bank_transactions_matched_entry
           FOREIGN KEY (matched_entry_id) REFERENCES journal_entries(id) ON DELETE SET NULL,
       CONSTRAINT chk_bank_transactions_status CHECK (status IN ('pending', 'reconciled')),
       CONSTRAINT chk_bank_transactions_currency_iso4217 CHECK (CHAR_LENGTH(currency) = 3),
       INDEX idx_bank_transactions_company_account_date (company_id, bank_account_id, booking_date),
       INDEX idx_bank_transactions_import (import_id),
       INDEX idx_bank_transactions_pending (company_id, bank_account_id, status, booking_date)  -- support Story 8-4
   ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
   ```

   **Invariants schéma :**
   - `company_id` sur les deux tables — pattern multi-tenant Story 6-2 / 7-1, validé sur 9 entités existantes.
   - `(company_id, file_hash)` UNIQUE — bloque le réimport silencieux du même fichier. Story 8-3 ajoutera l'UI « forcer le réimport » (FR43). En attendant, l'erreur SQL `1062` est mappée vers `409 Conflict` côté API (cf. T6.5).
   - `amount DECIMAL(18,2)` — cohérent avec `invoice_lines.line_total` et `journal_entry_lines.amount` ; précision 2 décimales suffisante pour CHF/EUR (les fichiers CAMT en sont 2 décimales en pratique).
   - `status` est volontairement String + CHECK plutôt qu'enum MariaDB : aligné avec le pattern `journal_entries.kind` (Story 3-2).
   - `idx_bank_transactions_pending` couvre la requête principale Story 8-4 « SELECT pending transactions for matching » sans nécessiter une seconde index Story 8-4.
   - `matched_entry_id` posé maintenant (Story 8-1) bien qu'utilisé Story 8-4, pour éviter une migration `ALTER TABLE ... ADD COLUMN` Story 8-4. La FK `ON DELETE SET NULL` permet de supprimer une écriture comptable sans casser la transaction bancaire (la transaction redevient `pending`).
   - **Pas de FULLTEXT index Story 8-1** — le matching FULLTEXT (Story 8-4) ne sera pas piloté par les `details` mais par jointure sur `journal_entries` (qui a déjà FULLTEXT — Story 7-4).

5. **Entités + repositories** — `crates/kesh-db/src/entities/bank_import.rs` + `bank_transaction.rs` + `crates/kesh-db/src/repositories/bank_imports.rs` + `bank_transactions.rs` :
   - `entities::BankImport` (struct miroir DB), `entities::NewBankImport` (struct insert), `entities::BankImportSourceFormat` (enum String-mappé `Camt053V04` / `Camt053V08` ; `Csv` ajouté Story 8-2 ; `as_str()` + `try_from_str(&str)` cohérents avec le pattern `Role::as_str` de `users.rs`).
   - `entities::BankTransaction`, `entities::NewBankTransaction`, `entities::BankTransactionStatus` (enum `Pending` / `Reconciled` avec `as_str` / `try_from_str`).
   - `repositories::bank_imports::create_with_transactions(tx, company_id, NewBankImport, Vec<NewBankTransaction>) -> Result<(BankImport, Vec<BankTransaction>), DbError>` — **insertion atomique dans la même transaction** : INSERT bank_imports + bulk INSERT bank_transactions (un seul SQL `INSERT ... VALUES (...), (...), ...` jusqu'à 1000 lignes max ; au-delà, batch ; cf. décision §perf).
   - `repositories::bank_imports::find_by_company_id(pool, company_id, query) -> Result<ListResponse<BankImport>, DbError>` — pagination offset/limit cohérente avec le pattern existant (cf. `invoices::list_by_company_paginated`).
   - `repositories::bank_transactions::list_by_import(pool, company_id, import_id) -> Result<Vec<BankTransaction>, DbError>` — filtré par `company_id` ET `import_id` (double-scope IDOR protection ; cf. KF-002 audit Story 7-1).
   - **Pas de `update` ni `delete` exposés Story 8-1** — l'utilisateur ne peut pas modifier un import (les transactions sont immuables une fois importées ; seul `status` change Story 8-4).
   - `repositories::bank_imports::find_by_company_and_hash(pool, company_id, file_hash) -> Result<Option<BankImport>, DbError>` — préparé pour Story 8-3 (détection `409 Conflict` au upload).
   - Inscription dans `crates/kesh-db/src/repositories/mod.rs` et `crates/kesh-db/src/entities/mod.rs`.

6. **Route API `kesh-api/routes/bank_imports.rs`** — endpoints :

   | Méthode | Path | Auth | Body / Response |
   |---|---|---|---|
   | `POST` | `/api/v1/bank-imports/preview` | authenticated, role ≥ Comptable | multipart `file` + form field `bankAccountId` → `200 OK` `{ statementId, accountIban, periodFrom, periodTo, openingBalance, closingBalance, transactionCount, transactions: [...preview...], warnings: [...] }`. **Ne persiste rien.** |
   | `POST` | `/api/v1/bank-imports` | authenticated, role ≥ Comptable | multipart `file` + form field `bankAccountId` → `201 Created` `{ id, importedAt, transactionCount, ... }`. **Persiste atomiquement.** |
   | `GET` | `/api/v1/bank-imports?bankAccountId=...&page=1&pageSize=20` | authenticated, tout rôle | `200 OK` `ListResponse<BankImportSummary>`. |
   | `GET` | `/api/v1/bank-imports/{id}` | authenticated, tout rôle | `200 OK` `{ ...meta..., transactions: [...] }` ou `404`. |

   **Détails implémentation :**

   - **Multipart** : ajouter feature `axum = { version = "0.8", features = ["multipart"] }` dans `kesh-api/Cargo.toml`. Extracteur `axum::extract::Multipart`.
   - **Limite upload** : 10 MiB par défaut (configurable via env `KESH_BANK_IMPORT_MAX_MB=10`, cf. décision §upload-limit). Refus `413 Payload Too Large` au-delà. La limite est appliquée à la fois côté Axum (`DefaultBodyLimit::max(10 * 1024 * 1024)`) ET en double-check dans le handler (compteur de bytes lu, pour ne pas dépendre uniquement du middleware).
   - **Hash fichier** : SHA-256 calculé en streaming pendant la lecture du multipart (un seul `Vec<u8>` du contenu, hash + parse depuis ce buffer). Format hex lower-case 64 chars.
   - **Scoping multi-tenant** : récupérer `current_user.company_id`, valider que `bankAccountId` appartient à cette company via `bank_accounts::find_by_id_for_company(pool, current_user.company_id, bank_account_id)` (helper à créer si absent — pattern KF-002). Si non-match → `404 Not Found` (jamais 403 — cf. KF-002 pattern « hide existence »).
   - **Vérification balance** : appeler `kesh_core::bank_imports::validate_balance(&stmt)` après parse. Si `Err(BankImportBalanceMismatch)` → l'erreur est traitée comme un **warning non-bloquant** dans la réponse `preview` (champ `warnings`) MAIS comme un **rejet `422 Unprocessable Entity`** sur le `POST /bank-imports` final, sauf si le client envoie `confirmBalanceMismatch=true` dans le form (CR-010 décision : import permis avec confirmation explicite, traçable côté audit log).
   - **Vérification currency** : `validate_currency_supported_v0_1` → si EUR ou autre → `422 Unprocessable Entity` (pas de bypass v0.1).
   - **Pas de `retry_on_deadlock`** Story 8-1 : l'INSERT atomique ne tient pas de SELECT FOR UPDATE concurrent (une transaction n'attend pas une autre transaction sur des ranges disjoints). Le helper deadlock-retry sera mobilisé Story 8-4 (multi-locks finalize). Documenter `// PAS de retry_on_deadlock — INSERT-only atomique, voir Story 8-4 pour la mécanique réconciliation` dans le handler.
   - **Audit log** : ajouter une entrée `audit_log` (table existante depuis Story 1-8) sur le `POST /bank-imports` réussi. Action `bank_import.created`, target = `bank_imports.id`. Pas d'audit sur preview (lecture pure, sans persistance).
   - **AppError mapping** : ajouter dans `kesh-api/src/errors.rs` les variantes `MultipartTooLarge`, `MultipartMalformed`, `BankImportParseFailed(String)`, `BankImportBalanceMismatch(...)` → JSON structuré `{ code: "BANK_IMPORT_PARSE_FAILED", message: "...", details: {...} }`. Codes : `BANK_IMPORT_TOO_LARGE` (413), `BANK_IMPORT_MALFORMED_XML` (400), `BANK_IMPORT_UNSUPPORTED_VERSION` (400), `BANK_IMPORT_DUPLICATE_FILE` (409 — pour Story 8-3 mais préparé), `BANK_IMPORT_BALANCE_MISMATCH` (422), `BANK_IMPORT_UNSUPPORTED_CURRENCY` (422), `BANK_ACCOUNT_NOT_FOUND` (404), `RBAC_FORBIDDEN` (403 si role < Comptable).
   - Mountant dans `crates/kesh-api/src/lib.rs` : ajouter les 4 routes dans `authenticated_routes`.
   - **RBAC** : vérifier `role >= Role::Accountant` dans le handler POST (helper `require_role` existant si présent ; sinon ajouter — cohérent avec le pattern Story 1-8). GET sans restriction de rôle (lecture).

7. **Frontend `frontend/src/lib/features/bank-import/`** — nouveau dossier (route `bank-import` existe déjà mais est un placeholder « à venir Epic 6 ») :
   - `bank-import.types.ts` — types miroirs API (`BankImportPreview`, `BankImportSummary`, `BankImportDetail`, `ParsedTransaction`).
   - `bank-import.api.ts` — `previewBankImport(file, bankAccountId)`, `confirmBankImport(file, bankAccountId, confirmBalanceMismatch?)`, `listBankImports(query)`, `getBankImport(id)`. Multipart via `FormData` + `fetch` (le wrapper existant de `shared/utils/api-client.ts` doit être étendu pour accepter `body: FormData` sans `Content-Type` JSON forcé — cf. décision §api-client).
   - `BankImportUpload.svelte` — drag-drop + sélection fichier (single file). État local : `idle`, `parsing`, `preview`, `confirming`, `done`, `error`. Limite client 10 MiB (alignée backend). Affichage progress bar pendant upload. Afficher la liste des transactions parsées en tableau (dates, montants, références, contrepartie) avant confirmation.
   - `BankImportPreviewTable.svelte` — sous-composant table de prévisualisation (réutilise les patterns `JournalEntriesTable` et `InvoiceLineList` ; tri par date booking, formatage montants `Intl.NumberFormat('de-CH')` selon convention Story 1-9).
   - `BankImportList.svelte` — liste paginée des imports passés (réutilise `Pagination.svelte` + `Table.svelte`).
   - `BankImportDetail.svelte` — vue d'un import + ses transactions (panel droit ou route séparée).
   - `BankAccountSelector.svelte` — `<select>` des comptes bancaires de la company (charge via `companies/current` qui retourne déjà `bankAccounts[]` — Story 6-2 / 1-3). Première option = `is_primary` ; obligatoire avant upload.
   - **Drag-drop accessibilité** : input `<input type="file">` toujours présent et visible (aria-label), drag-drop est une augmentation, pas un substitut. Conformité axe-core (Story 6-3 / KF-023).
   - **Avertissement balance mismatch** : si la réponse `preview` contient `warnings: ["balance_mismatch"]`, afficher un banner `<Alert variant="warning">` avec message i18n `bankImport.warnings.balanceMismatch` + checkbox `confirmBalanceMismatch` qui doit être cochée pour activer le bouton « Confirmer l'import ».

8. **Route SvelteKit `frontend/src/routes/(app)/bank-import/`** — remplacer le placeholder par :
   - `+page.svelte` — orchestrateur (mount `BankImportUpload` + `BankImportList`).
   - `+page.ts` — `load` qui pré-charge `getCompaniesCurrent()` (pour le sélecteur de compte) + `listBankImports({ pageSize: 20 })`.
   - `[id]/+page.svelte` — détail d'un import (mount `BankImportDetail`).
   - `[id]/+page.ts` — `load` qui appelle `getBankImport(id)`.

9. **i18n** — clés ajoutées dans **les 4 locales** (`fr-CH`, `de-CH`, `it-CH`, `en-CH`) `crates/kesh-i18n/locales/{locale}/messages.ftl` :

   ```fluent
   # Bank import (Story 8.1)
   bankImport-title = Importer un relevé bancaire
   bankImport-uploadPrompt = Glissez votre fichier CAMT.053 ou cliquez pour sélectionner
   bankImport-fileMaxSize = Taille maximale : 10 Mo
   bankImport-bankAccountLabel = Compte bancaire de destination
   bankImport-bankAccountRequired = Sélectionnez un compte avant de téléverser
   bankImport-parsing = Analyse du fichier en cours…
   bankImport-previewTitle = Aperçu — { $count } transactions
   bankImport-confirmButton = Confirmer l'import
   bankImport-cancelButton = Annuler
   bankImport-success = { $count } transactions importées avec succès
   bankImport-errors-malformedXml = Le fichier n'est pas un XML CAMT.053 valide
   bankImport-errors-unsupportedVersion = Version CAMT.053 non supportée : { $version }
   bankImport-errors-tooLarge = Le fichier dépasse 10 Mo
   bankImport-errors-duplicateFile = Ce fichier a déjà été importé le { $importedAt }
   bankImport-errors-unsupportedCurrency = Devise non supportée : { $currency }. Seuls les comptes en CHF sont importables en v0.1.
   bankImport-warnings-balanceMismatch = Le solde de clôture du fichier ({ $closing }) ne correspond pas à la somme calculée ({ $sum }). Écart : { $diff }. Confirmez pour importer quand même.
   bankImport-warnings-balanceMismatchConfirm = Je confirme l'import malgré l'écart
   bankImport-list-empty = Aucun import effectué pour ce compte
   bankImport-list-importedAt = Importé le { $date }
   bankImport-list-transactionCount = { $count } transactions
   bankImport-list-period = Période : { $from } → { $to }
   bankImport-detail-statementId = Identifiant du relevé
   bankImport-detail-source = Format source
   bankImport-detail-openingBalance = Solde d'ouverture
   bankImport-detail-closingBalance = Solde de clôture
   ```

   - **Préfixe `bankImport-`** (camelCase) cohérent avec `invoiceForm-`, `journalEntries-`, etc. (cf. `docs/i18n-key-ownership-pattern.md`).
   - **Traductions DE / IT / EN** : dev fournit les 4 langues. Pour DE/IT, réutiliser le vocabulaire bancaire suisse standard (« Bankauszug » / « Estratto conto »). Validation via `npm run lint-i18n-ownership` (Story 6-3 garde-fou).

10. **Tests** — couverture exhaustive :

    - **kesh-import unitaires** (`crates/kesh-import/tests/camt053_tests.rs`) :
      - `parse_v04_minimal_extracts_all_transactions` (FR42)
      - `parse_v08_minimal_extracts_all_transactions` (multi-version)
      - `parse_with_subtxs_extracts_individual_transactions` (FR49)
      - `parse_multi_stmt_returns_one_per_account` (multi-Stmt)
      - `parse_truncated_returns_malformed_xml_error`
      - `parse_unknown_namespace_returns_unsupported_version`
      - `parse_invalid_iban_keeps_transaction` (tolérance IBAN)
      - `parse_eur_currency_preserved` (la devise est extraite, le rejet v0.1 vit dans kesh-core)

    - **kesh-core unitaires** (`crates/kesh-core/src/bank_imports.rs::tests`, étendre les 4 existants) :
      - `validate_balance_passes_when_within_tolerance`
      - `validate_balance_fails_when_diff_exceeds_one_cent`
      - `validate_balance_skipped_when_balances_missing`
      - `validate_currency_chf_passes`
      - `validate_currency_eur_rejected_v0_1`
      - `from_imported_injects_fk` (BankTransactionDraft + import_id + bank_account_id + company_id présents)

    - **kesh-db intégration** (`crates/kesh-db/tests/bank_imports_test.rs` ou inline `#[sqlx::test]`) :
      - `create_with_transactions_atomic_success`
      - `create_with_transactions_rolls_back_on_constraint_violation`
      - `find_by_company_id_only_returns_own_imports` (IDOR)
      - `find_by_company_and_hash_finds_existing`
      - `unique_company_hash_blocks_duplicate_within_same_company`
      - `unique_company_hash_allows_same_hash_across_companies` (multi-tenant safety)
      - `bulk_insert_handles_500_transactions` (perf smoke)

    - **kesh-api E2E HTTP** (`crates/kesh-api/tests/bank_imports_e2e.rs`) :
      - `post_preview_returns_parsed_statement` (happy path)
      - `post_import_creates_rows_atomically`
      - `post_import_rejects_when_role_below_accountant` (403)
      - `post_import_rejects_payload_too_large` (413)
      - `post_import_rejects_other_company_bank_account_id` (404 IDOR)
      - `post_import_rejects_eur_currency` (422)
      - `post_import_rejects_balance_mismatch_without_confirm` (422)
      - `post_import_accepts_balance_mismatch_with_confirm` (201 + audit log entry)
      - `post_import_rejects_duplicate_file_hash_in_same_company` (409, préparé pour Story 8-3)
      - `get_imports_lists_only_own_company` (IDOR)
      - `get_import_returns_404_for_other_company_id` (IDOR)

    - **Playwright E2E** (`frontend/tests/e2e/bank-import.spec.ts` — nouveau fichier) :
      - `imports a CAMT.053 v04 file end-to-end` (drag-drop ou file input, preview, confirm, vérifier la liste contient l'import).
      - `shows balance mismatch warning and requires confirmation`
      - `rejects file > 10 MB client-side avec message d'erreur`
      - `requires bank account selection before upload`
      - `lists previous imports paginated`
      - `accessibility — axe scan zero violations` (continuer la baseline Story 6-3)
      - **Selectors** : `data-testid` partout (lesson Story 7-5/KF-008). Aucun `getByText()` brittle, aucun `.first()` ou `.nth()`. Strict mode ON.
      - **Fixture E2E** : ajouter `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` (copie du fichier de test kesh-import).

### Scope volontairement HORS story — décisions tranchées

- **CSV multi-encodage** : Story 8-2. Le module `kesh-import::csv` est créé vide (`pub mod csv;` dans `lib.rs` puis `csv/mod.rs` avec un `// TODO Story 8-2` commenté). Aucune API publique n'est exposée Story 8-1.
- **Détection de doublons fine** (date+montant+référence intra-fichier) : Story 8-3. Story 8-1 livre uniquement la contrainte UNIQUE `(company_id, file_hash)` côté DB et le mapping `409 Conflict` côté API. La logique « détecter un overlap partiel entre deux imports » n'est PAS dans le scope.
- **Réconciliation et matching automatique** : Story 8-4. Story 8-1 pose `bank_transactions.status = 'pending'` + `matched_entry_id = NULL` et c'est tout. Aucune table `reconciliation_proposals` n'est créée.
- **Règles d'affectation** : Story 8-5. Aucune table `import_rules` Story 8-1.
- **Mutex per-bank-account** : Story 8-4. Story 8-1 INSERT-only ne nécessite pas de mutex (la clé UNIQUE `(company_id, file_hash)` empêche le double-import simultané du même fichier ; les imports concurrents de fichiers différents sont parallélisables sans contrainte).
- **`pain.001` paiements** : Epic 12 (v0.2).
- **Sélection automatique de compte par IBAN du fichier** : v0.2. v0.1 = sélection manuelle obligatoire de `bankAccountId`. Justification : un fichier peut contenir plusieurs `<Stmt>` pour différents IBAN, et Kesh n'a pas la pré-validation IBAN suisse standard (Story 1-3 a `Iban::new` mais l'auto-mapping risque de matcher un IBAN homonyme entre companies). Décision conservatrice : l'utilisateur choisit. Si fichier multi-Stmt → Story 8-1 ne traite **que les `<Stmt>` dont l'IBAN matche celui du `bankAccountId` choisi** ; les autres `<Stmt>` sont ignorés (warning explicite dans la réponse preview avec liste des IBAN ignorés). À promouvoir post-v0.1 si demande utilisateur.
- **Multi-currency** (EUR, USD) : v0.2. Story 8-1 n'accepte que CHF (cf. décision §currency). Le champ `currency CHAR(3)` DB accepte n'importe quelle devise ISO 4217 — la restriction vit dans `kesh_core::bank_imports::validate_currency_supported_v0_1`. v0.2 changera juste la liste autorisée.
- **Édition / suppression d'un import** : v0.2. v0.1 = un import est immuable une fois persisté. Si l'utilisateur s'aperçoit d'une erreur, il devra contacter un admin (intervention SQL). Documenté en KF si demande.
- **Statement balance check FR42-bis** : intégré ICI (CR-010 #62) — décision tranchée 2026-05-03 plutôt que reporté Epic 9. Justification : `validate_balance` est trivial à implémenter et la régression silencieuse (fichier tronqué) serait coûteuse à débugger plus tard.
- **Streaming parser pour fichiers > 10 Mo** : pas v0.1. Limite 10 Mo dure (NFR § Performance : « relevé mensuel ~200 transactions < 2s »). Un relevé annuel (~2400 transactions) tient largement en 10 Mo XML. Au-delà : `413` côté API + message clair.
- **CRUD `bank_profiles`** : Story 8-2. Aucune table `bank_profiles` Story 8-1.
- **CRUD `import_rules`** : Story 8-5.
- **UI consultation transaction-par-transaction avec édition manuelle des comptes contrepartie** : Story 8-5.
- **Notifications email post-import** : non v0.1, hors PRD.
- **Versioning des parseurs FR87** : implémenté de facto par la dispatch sur namespace, mais l'UI exposant la version utilisée est v0.2.

### Décisions de conception

#### §schéma — Tables `bank_imports` et `bank_transactions`

Voir le bloc SQL en T4. Points clés :

- Choix du **type CHAR(64) pour `file_hash`** : SHA-256 hex lower-case, 64 chars exacts. Préféré à `BINARY(32)` pour la lisibilité côté `mysql` shell et la portabilité des fixtures. Coût stockage négligeable (~32 bytes / import vs `BINARY(32)`, et avec 12 imports/an/company × 100 companies × 5 ans = 6000 lignes).
- **Pas de colonne `version` sur `bank_imports`** : table append-only (immuable côté utilisateur v0.1). `bank_transactions.version` est posée pour préparer Story 8-4 (changement `pending → reconciled` avec optimistic locking).
- **Pas de soft-delete** : `ON DELETE CASCADE` sur `bank_transactions.import_id` permet une suppression admin SQL propre si nécessaire. Pas d'UI de suppression v0.1.
- **`details TEXT NOT NULL`** : pas `TEXT NULL` — si le fichier ne fournit pas de description, on stocke `""` (string vide) plutôt que NULL. Cohérent avec le pattern Story 4-1 (`contacts.notes TEXT NOT NULL DEFAULT ''`).
- **Index composite `(company_id, bank_account_id, status, booking_date)`** : couvre la requête principale Story 8-4 sans nouvelle migration.

#### §iban-tolerant — IBAN brute, validation différée

Décision : conserver `counterparty_iban VARCHAR(34) NULL` en `String` brute (non validée par `kesh-core::types::Iban`). Si le fichier source contient un IBAN au checksum cassé (cas réel : coquille banque, fichier corrompu d'un correspondant), la transaction est conservée AVEC l'IBAN brute, pas rejetée.

**Justification :** un IBAN cassé est une métadonnée de la transaction, pas une condition d'existence. Rejeter la transaction = perdre une donnée comptable. Les utilisateurs corrigent l'IBAN manuellement Story 8-5 (réconciliation manuelle).

**Test associé** : `parse_invalid_iban_keeps_transaction` doit produire un `ImportedTransaction` avec `counterparty_iban: Some("CH00...".to_string())` et **aucun warning** (un IBAN cassé est silencieux côté parseur).

`account_iban` (IBAN du compte titulaire) : idem brute. La validation que cet IBAN matche celui du `bankAccountId` sélectionné par l'utilisateur vit dans le handler API (string equality après normalisation `replace(' ', '').to_uppercase()`). Cette validation est case-sensitive alignée ISO 13616 (les IBAN suisses sont stockés sans espaces, en majuscules — Story 2-3 / 1-3 — donc la comparaison après normalisation est fiable).

#### §currency — CHF uniquement v0.1

Décision : `validate_currency_supported_v0_1` accepte uniquement `"CHF"`. Tout fichier contenant un `<Stmt><Acct><Ccy>` ≠ `"CHF"` → `422 Unprocessable Entity` au moment du POST `/bank-imports` (mais le **preview reste possible** et affiche les transactions, avec un warning `unsupportedCurrency`).

**Justification PRD :** §FR42 « relevés bancaires » dans le contexte « Swiss accounting software » (PRD §Vue d'ensemble). Aucun FR ne mentionne de gestion multi-devises v0.1. Les comptes CH bancaires courants sont CHF dans 95%+ des cas PME ; les comptes EUR sont des comptes secondaires gérés v0.2 avec conversion taux.

**Pas de bypass v0.1** — pas de flag `force_currency=EUR` ; un fichier EUR doit attendre v0.2. Documenté côté UI (i18n `bankImport-errors-unsupportedCurrency`).

#### §balance-check — CR-010 #62 statement balance reconciliation

**Algorithme** (`kesh_core::bank_imports::validate_balance`) :

```rust
// pseudocode
if stmt.opening_balance.is_none() || stmt.closing_balance.is_none() {
    return Ok(()); // skip — pas de soldes dans le fichier (cas CSV ou partial CAMT)
}
let expected_closing = stmt.opening_balance.unwrap() + stmt.sum_transactions();
let diff = (expected_closing - stmt.closing_balance.unwrap()).abs();
if diff > Decimal::new(1, 2) /* 0.01 */ {
    return Err(CoreError::BankImportBalanceMismatch {
        opening: stmt.opening_balance.unwrap(),
        closing: stmt.closing_balance.unwrap(),
        sum: stmt.sum_transactions(),
        diff,
    });
}
Ok(())
```

**Tolérance 0.01** : reflète l'arrondi commercial standard suisse (centime). Au-delà → fichier suspect (tronqué, transactions manquantes, double-comptage).

**Comportement côté API :**
- `POST /preview` : balance mismatch → `200 OK` + `warnings: ["balance_mismatch"]` dans la réponse, transactions tout de même listées.
- `POST /bank-imports` (final) : balance mismatch → `422 Unprocessable Entity` (`code: BANK_IMPORT_BALANCE_MISMATCH`) sauf si form contient `confirmBalanceMismatch=true` → import procède + entrée audit_log dédiée (`bank_import.created_with_balance_mismatch`, `details = JSON{ diff, opening, closing, sum }`).

**Pas de bypass automatique** — la confirmation explicite est toujours requise. Cohérent avec le principe « pas d'import silencieux d'un fichier suspect ».

#### §multi-stmt — fichiers multi-statement

Décision : un fichier CAMT.053 peut contenir plusieurs `<Stmt>` (jusqu'à 1 par compte titulaire). v0.1 traite **uniquement le ou les `<Stmt>` dont `<Acct><IBAN>` matche le `bankAccountId.iban` sélectionné par l'utilisateur** (après normalisation espaces / casse). Les autres `<Stmt>` sont ignorés silencieusement côté persistance MAIS listés dans la réponse `preview` :

```json
{
  "transactions": [...],
  "warnings": ["balance_mismatch"],
  "ignoredStatements": [
    { "iban": "CH9300762011623852957", "transactionCount": 12, "reason": "iban_mismatch_with_selected_account" }
  ]
}
```

**Justification :** l'auto-mapping IBAN → bankAccountId du compte titulaire (i.e. trouver dans `bank_accounts` la ligne avec `iban = stmt_iban`) est risqué v0.1 sans audit cross-tenant. Décision conservatrice : utilisateur choisit explicitement. Promouvoir auto-mapping post-v0.1 si demande.

Si **aucun `<Stmt>` ne matche** l'IBAN du compte sélectionné → `422 Unprocessable Entity` `code: BANK_IMPORT_NO_MATCHING_STATEMENT` avec liste des IBAN trouvés.

#### §upload-limit — 10 MiB hard limit

`KESH_BANK_IMPORT_MAX_MB=10` env var (default 10). Appliqué :
1. **Côté SvelteKit** (UX) : check `file.size > maxBytes` avant POST, message i18n `bankImport-errors-tooLarge`.
2. **Côté Axum middleware** : `tower-http::limit::RequestBodyLimitLayer` à `max_mb * 1024 * 1024 + 4096` (overhead multipart).
3. **Côté handler** : compteur de bytes lu pendant le streaming multipart, abort si dépassement (défense en profondeur, au cas où la couche 2 serait contournée par un futur changement).

10 MiB couvre largement un relevé annuel CAMT.053 (~2400 transactions ≈ 1.5-3 MiB XML). Au-delà → fichier probablement non-CAMT (PDF, scan, fichier multi-banques fusionné par erreur).

#### §perf — bulk INSERT bank_transactions

Décision : **un seul `INSERT INTO bank_transactions (...) VALUES (...), (...), ...`** par import (jusqu'à 1000 lignes par requête). `sqlx::QueryBuilder::push_values` pattern, déjà utilisé dans `journal_entries` Story 3-2.

**Performance ciblée** (NFR PRD) : « relevé mensuel 200 transactions en < 2s » incluant parse + DB. Le bulk insert seul doit être < 100 ms pour 200 lignes. Au-delà de 1000 transactions / fichier (cas rarissime — relevé annuel), batcher en chunks de 1000 dans la même transaction (pas plusieurs transactions distinctes — atomicité requise).

#### §api-client — extension wrapper fetch pour multipart

Le wrapper `frontend/src/lib/shared/utils/api-client.ts` (Story 1-11) force `Content-Type: application/json` et `JSON.stringify(body)`. Pour Story 8-1 il doit accepter `body: FormData` :

- Si `body instanceof FormData` → ne pas setter `Content-Type` (le navigateur le calcule avec le `boundary=...`) ET ne pas `JSON.stringify`. Passer `FormData` direct à `fetch`.
- Sinon (cas existant) → comportement inchangé.

**Pas de breaking change** sur les autres callers. Ajouter test `api-client.test.ts` pour le branch FormData.

#### §quick-xml — choix librairie

`quick-xml = "0.36"` (latest 2026 stable) :
- Pull parser zéro-allocation (mode `Reader::from_reader`). Pas de DOM intermédiaire pour fichiers de 10 MiB.
- Mature, maintenue, dépendance unique (pas de cascade `xml-rs` / `serde-xml-rs`).
- Compatible no-std côté core (pas requis ici mais préserve l'option de réutilisation kesh-import sur d'autres cibles).
- **À éviter** : `serde-xml-rs` (déprécié 2025 — auteur archivé), `xml-rs` (lent, allocation par event), `roxmltree` (DOM, OK pour petits fichiers mais non-streaming).

**Pattern d'usage** :

```rust
use quick_xml::Reader;
use quick_xml::events::Event;

let mut reader = Reader::from_reader(xml);
reader.config_mut().trim_text(true);
let mut buf = Vec::new();
loop {
    match reader.read_event_into(&mut buf)? {
        Event::Start(e) if e.name().as_ref() == b"Stmt" => { /* enter statement */ }
        Event::End(e) if e.name().as_ref() == b"Stmt" => { /* close statement */ }
        Event::Eof => break,
        _ => {}
    }
    buf.clear();
}
```

Pour la dispatch v04 vs v08 : extraire la `xmlns="..."` du tag racine `<Document>` au premier `Event::Start`, puis instancier `v04::Parser::new(reader)` ou `v08::Parser::new(reader)`. Les deux parsers partagent un trait `CamtParser { fn parse(self) -> Result<Vec<ImportedStatement>, CamtError>; }`.

## Acceptance Criteria

1. **(FR42)** Given un fichier CAMT.053 valide (v04 ou v08), When l'utilisateur clique « Importer », Then toutes les transactions sont extraites et persistées avec date booking, montant signé, référence, détails, contrepartie. *Test : E2E `imports a CAMT.053 v04 file end-to-end`.*

2. **(FR49)** Given un fichier CAMT.053 contenant des `<TxDtls>` (sous-transactions) sous un `<Ntry>` agrégé, When parse, Then chaque `<TxDtls>` produit une `ImportedTransaction` distincte (et non l'agrégat seul). *Test : `parse_with_subtxs_extracts_individual_transactions`.*

3. **(FR50)** Given un import, When l'utilisateur a sélectionné un `bankAccountId`, Then toutes les transactions persistées ont `bank_account_id = selected_id`. Si le fichier contient des `<Stmt>` pour d'autres IBAN, ces statements sont **ignorés** (pas persistés) avec un warning explicite dans la réponse preview (`ignoredStatements`). *Test : `post_import_creates_rows_atomically` + `parse_multi_stmt_returns_one_per_account`.*

4. **(UX)** Given l'utilisateur sur la page `/bank-import`, When il glisse un fichier ou clique « Sélectionner », Then une **prévisualisation des transactions** s'affiche avant la persistance définitive. La confirmation explicite (bouton « Confirmer l'import ») déclenche le `POST /bank-imports`. *Test E2E + `BankImportUpload.test.ts`.*

5. **(FR87 / Décision archi #2)** Given un fichier CAMT.053, When le namespace racine est `urn:iso:std:iso:20022:tech:xsd:camt.053.001.04` ou `.08`, Then le parseur de version correspondante est utilisé. Les autres versions retournent `400 BANK_IMPORT_UNSUPPORTED_VERSION` avec le namespace détecté dans le message. *Tests : `parse_v04_minimal_extracts_all_transactions`, `parse_v08_minimal_extracts_all_transactions`, `parse_unknown_namespace_returns_unsupported_version`.*

6. **(Architecture)** Given le crate `kesh-import`, When `cargo metadata -p kesh-import --format-version 1 | jq '.packages[0].dependencies[] | select(.path != null)'` est exécuté, Then aucune dépendance workspace interne n'apparaît (zéro path dep). *Test : script CI ajouté à `.github/workflows/ci.yml` (cf. T8.5).*

7. **(Architecture)** Given le crate `kesh-import`, When `cargo publish --dry-run --allow-dirty -p kesh-import` est exécuté en local, Then la commande termine sans erreur. *Test : ajouté au job CI Rust (cf. T8.5).*

8. **(Architecture)** Given un `ImportedTransaction`, When converti via `From`, Then le résultat est un `BankTransactionDraft` valide ; les FK (`bank_account_id`, `import_id`, `company_id`) sont injectées par `kesh-core::bank_imports::from_imported`, **pas** par `kesh-import`. *Test : `from_imported_injects_fk` (existing spike test étendu).*

9. **(Fixtures SIX)** Given le suite de tests d'intégration `kesh-import`, When exécutée, Then les 8 fixtures listées en T1 sont chargées et au moins 6 cas distincts (v04 / v08 / sub-tx / multi-stmt / balance-mismatch / truncated / invalid-iban / EUR) sont validés. *Test : `crates/kesh-import/tests/camt053_tests.rs` × 8.*

10. **(Schéma multi-tenant — KF-002 pattern)** Given une transaction bancaire persistée pour `company_A`, When `company_B` appelle `GET /bank-imports/{id}` ou `GET /bank-imports?bankAccountId=...`, Then la réponse est `404 Not Found` (jamais 403 — pattern KF-002). *Tests : `get_imports_lists_only_own_company`, `get_import_returns_404_for_other_company_id`.*

11. **(Multi-tenant DB)** Given la migration appliquée, When `cargo test -p kesh-db bank_imports IDOR`, Then **aucun cross-tenant leak** n'est observé sur les 7 cas listés en T10. *Tests : `find_by_company_id_only_returns_own_imports` etc.*

12. **(Sécurité — RBAC)** Given un utilisateur avec `role = Consultation`, When il tente `POST /bank-imports`, Then la réponse est `403 RBAC_FORBIDDEN`. Les rôles `Accountant` et `Admin` peuvent importer ; tous les rôles peuvent lire. *Test : `post_import_rejects_when_role_below_accountant`.*

13. **(Sécurité — payload limit)** Given un fichier > 10 MiB, When upload, Then la réponse est `413 BANK_IMPORT_TOO_LARGE`. Aucun parsing n'est tenté. *Test : `post_import_rejects_payload_too_large`.*

14. **(CR-010 #62 — balance check)** Given un fichier où `|opening + Σ transactions - closing| > 0.01`, When `POST /bank-imports` sans `confirmBalanceMismatch`, Then `422 BANK_IMPORT_BALANCE_MISMATCH`. When avec `confirmBalanceMismatch=true`, Then `201 Created` + entrée audit_log `bank_import.created_with_balance_mismatch`. *Tests : `post_import_rejects_balance_mismatch_without_confirm` + `post_import_accepts_balance_mismatch_with_confirm`.*

15. **(Devise v0.1)** Given un fichier en `<Acct><Ccy>EUR</Ccy>`, When `POST /bank-imports`, Then `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`. Le `POST /preview` affiche les transactions avec un warning `unsupportedCurrency`. *Test : `post_import_rejects_eur_currency`.*

16. **(Doublons fichier — préparation Story 8-3)** Given un fichier déjà importé pour la même company (même `file_hash`), When second import, Then `409 BANK_IMPORT_DUPLICATE_FILE`. Le même fichier importé pour une autre company → `201 Created` (multi-tenant safety). *Test : `unique_company_hash_blocks_duplicate_within_same_company` + `unique_company_hash_allows_same_hash_across_companies`.*

17. **(Atomicité)** Given un import qui échoue côté `bank_transactions` (ex. erreur SQL ligne 150 sur 200), When la transaction DB rollback, Then `bank_imports` ET `bank_transactions` sont vides en DB (rien d'orphelin). *Test : `create_with_transactions_rolls_back_on_constraint_violation`.*

18. **(Audit log)** Given un import réussi, When `GET /audit-log`, Then une entrée `bank_import.created` (ou `bank_import.created_with_balance_mismatch`) est présente avec `target_id = bank_import.id`, `user_id = importer`, `details JSON` contenant `{ filename, transaction_count, source_format }`. *Test : `post_import_creates_rows_atomically` étendu pour vérifier l'audit log.*

19. **(i18n)** Given les 4 locales (fr/de/it/en-CH), When `npm run lint-i18n-ownership`, Then le lint passe sans erreur (toutes les clés `bankImport-*` présentes dans les 4 fichiers). *Test : CI Story 6-3.*

20. **(Accessibilité)** Given la page `/bank-import` rendue, When `axe-core` scan, Then zéro violation. Le drag-drop est une augmentation : un input `<input type="file" aria-label>` reste navigable au clavier. *Test : E2E `accessibility — axe scan zero violations`.*

21. **(Performance NFR)** Given un fichier de 200 transactions CAMT.053 v04, When `POST /bank-imports`, Then la durée totale (parse + DB) < 2s sur la machine de dev nominale. *Test : `bulk_insert_handles_500_transactions` instrumenté avec `Instant::now()` (smoke, pas un seuil CI strict).*

## Tasks / Subtasks

### T1. Fixtures CAMT.053 (AC #9)

- [ ] T1.1 — Créer `crates/kesh-import/tests/fixtures/camt053/` et `crates/kesh-import/tests/fixtures/README.md` (provenance synthétique, pas de PII).
- [ ] T1.2 — Construire les 8 fichiers XML listés au §Scope T1 à la main à partir du XSD `docs/six-references/.../camt.053.001.08.xsd` (pour v08) et de la spec ISO 20022 v04 publique. **IBAN suisses fictifs** (générer via le pattern `CH00 + 02 chiffres + 9300762011623852957` recalculé MOD-97).
- [ ] T1.3 — Vérifier que les 8 fichiers passent le check d'XML well-formed (`xmllint --noout fixture.xml`). Pas de validation XSD requise (les fichiers de prod ne sont pas tous strictement valides, le parseur doit être tolérant aux extensions).

### T2. Parseur `kesh-import::camt053` (AC #1, #2, #5)

- [ ] T2.1 — Ajouter `quick-xml = "0.36"` à `crates/kesh-import/Cargo.toml`.
- [ ] T2.2 — Créer `crates/kesh-import/src/error.rs` avec l'enum `CamtError` (5 variantes listées en T2 du scope).
- [ ] T2.3 — Créer `crates/kesh-import/src/camt053/mod.rs` avec le dispatcher namespace + le trait `CamtParser`.
- [ ] T2.4 — Implémenter `crates/kesh-import/src/camt053/v04.rs` (parser pull-based des éléments `<BkToCstmrStmt>`, `<Stmt>`, `<Acct>`, `<Bal>`, `<Ntry>`, `<NtryDtls>`, `<TxDtls>`, `<RltdPties>`, `<RmtInf>`).
- [ ] T2.5 — Implémenter `crates/kesh-import/src/camt053/v08.rs` (réutiliser le code commun via le trait, documenter le delta v04 ↔ v08).
- [ ] T2.6 — Mettre à jour `crates/kesh-import/src/lib.rs` (ajouter `pub mod camt053; pub mod error;` ; conserver `pub mod types`).
- [ ] T2.7 — Tests unitaires `crates/kesh-import/tests/camt053_tests.rs` (8 tests minimum, AC #9).

### T3. Extensions `kesh-core::bank_imports` (AC #3, #8, #14, #15, #17)

- [ ] T3.1 — Étendre `crates/kesh-core/src/bank_imports.rs` avec `BankImportDraft` (struct + From conversions).
- [ ] T3.2 — Implémenter `from_imported(stmt, bank_account_id, company_id, file_hash, filename)` (fonction wrapper).
- [ ] T3.3 — Implémenter `validate_balance(stmt) -> Result<(), CoreError>` (CR-010 #62, AC #14).
- [ ] T3.4 — Implémenter `validate_currency_supported_v0_1(stmt) -> Result<(), CoreError>` (AC #15).
- [ ] T3.5 — Étendre `crates/kesh-core/src/errors.rs` avec `BankImportBalanceMismatch` et `BankImportUnsupportedCurrency`.
- [ ] T3.6 — Tests unitaires (étendre les 4 existants à ~10 tests, cf. §Tests).

### T4. Migration DB (AC #11, #16, #17)

- [ ] T4.1 — Créer `crates/kesh-db/migrations/2026MMDD000001_bank_imports.sql` (date du jour de l'impl).
- [ ] T4.2 — Vérifier que la migration applique proprement sur une DB `kesh_dev` fraîche : `cargo run -p kesh-seed --bin reset` puis `sqlx migrate run`.
- [ ] T4.3 — Vérifier qu'`appliquer-puis-rollback` est propre : pas de migration descendante (sqlx convention) mais `DROP TABLE bank_transactions; DROP TABLE bank_imports;` doit être réversible manuellement.

### T5. Entités + repositories (AC #11, #16, #17)

- [ ] T5.1 — Créer `crates/kesh-db/src/entities/bank_import.rs` + `bank_transaction.rs` (structs + enum String-mappés + `as_str` / `try_from_str`).
- [ ] T5.2 — Inscrire dans `crates/kesh-db/src/entities/mod.rs`.
- [ ] T5.3 — Créer `crates/kesh-db/src/repositories/bank_imports.rs` avec les 5 fonctions listées T5 du scope.
- [ ] T5.4 — Créer `crates/kesh-db/src/repositories/bank_transactions.rs` avec `list_by_import` et helpers internes.
- [ ] T5.5 — Inscrire dans `crates/kesh-db/src/repositories/mod.rs`.
- [ ] T5.6 — Tests intégration `#[sqlx::test]` (7 tests cf. §Tests).

### T6. Route API `bank_imports.rs` (AC #1, #4, #10, #12, #13, #14, #15, #16, #18)

- [ ] T6.1 — Activer feature multipart : `axum = { version = "0.8", features = ["multipart"] }` dans `crates/kesh-api/Cargo.toml`.
- [ ] T6.2 — Créer `crates/kesh-api/src/routes/bank_imports.rs` avec les 4 handlers (preview, create, list, detail).
- [ ] T6.3 — Helper `bank_accounts::find_by_id_for_company(pool, company_id, id) -> Result<Option<BankAccount>, DbError>` (à ajouter dans `crates/kesh-db/src/repositories/bank_accounts.rs` si absent — cohérent avec le pattern Story 7-1).
- [ ] T6.4 — Étendre `crates/kesh-api/src/errors.rs` avec les variantes `MultipartTooLarge`, `MultipartMalformed`, `BankImportParseFailed`, `BankImportBalanceMismatch`, etc. + mapping vers les codes HTTP listés au §scope T6.
- [ ] T6.5 — Mapping erreur SQL `1062` (UNIQUE violation `uq_bank_imports_company_hash`) → `409 BANK_IMPORT_DUPLICATE_FILE` dans `bank_imports.rs` (helper local, pattern aligné `invoices::create` numérotation).
- [ ] T6.6 — Mountant routes dans `crates/kesh-api/src/lib.rs` (4 routes dans `authenticated_routes`).
- [ ] T6.7 — Helper RBAC `require_role_at_least(current_user, Role::Accountant)?` à utiliser dans les POST.
- [ ] T6.8 — Audit log entry sur POST réussi (réutiliser le helper Story 1-8 + 3-5).
- [ ] T6.9 — Tests E2E `crates/kesh-api/tests/bank_imports_e2e.rs` (11 tests cf. §Tests).
- [ ] T6.10 — Limite multipart : `tower-http::limit::RequestBodyLimitLayer` à 10 MiB + 4 KiB overhead. Configurable via `KESH_BANK_IMPORT_MAX_MB` (`kesh-api/src/config.rs`).

### T7. Frontend feature `bank-import` (AC #1, #4, #20)

- [ ] T7.1 — Étendre `frontend/src/lib/shared/utils/api-client.ts` pour accepter `body: FormData` (cf. §api-client).
- [ ] T7.2 — Créer `frontend/src/lib/features/bank-import/` (types, api, store si nécessaire).
- [ ] T7.3 — Créer `BankImportUpload.svelte` avec drag-drop + file input accessibles + état machine.
- [ ] T7.4 — Créer `BankImportPreviewTable.svelte` (tri, formatage Intl.NumberFormat de-CH).
- [ ] T7.5 — Créer `BankImportList.svelte` + `BankImportDetail.svelte` + `BankAccountSelector.svelte`.
- [ ] T7.6 — Remplacer `frontend/src/routes/(app)/bank-import/+page.svelte` (placeholder Epic 6) par l'orchestrateur réel + ajouter `+page.ts` + `[id]/+page.svelte` + `[id]/+page.ts`.
- [ ] T7.7 — Ajouter les `data-testid` partout (lesson Story 7-5 / KF-008).
- [ ] T7.8 — Tests Vitest unitaires `BankImportUpload.test.ts` (state machine, validations client).

### T8. i18n + invariants CI (AC #6, #7, #19)

- [ ] T8.1 — Ajouter les clés `bankImport-*` dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (FR donné en §scope T9).
- [ ] T8.2 — Traduire en DE / IT / EN (les 3 autres locales).
- [ ] T8.3 — Vérifier `npm run lint-i18n-ownership` pass.
- [ ] T8.4 — Ajouter `kesh-import` au check `cargo publish --dry-run` du workflow CI Rust (job existant pour `kesh-qrbill` Story 5-3, étendre à `kesh-import`).
- [ ] T8.5 — Ajouter dans `.github/workflows/ci.yml` un step `cargo metadata -p kesh-import --format-version 1 | jq '...'` pour AC #6 (test invariant zero internal dep).

### T9. Tests E2E Playwright (AC #1, #4, #13, #14, #20)

- [ ] T9.1 — Créer `frontend/tests/e2e/bank-import.spec.ts` (6 scénarios listés §Tests Playwright).
- [ ] T9.2 — Ajouter fixture E2E `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` (copie depuis kesh-import fixtures).
- [ ] T9.3 — Vérifier `npm run test:e2e -- bank-import.spec.ts` localement (avec MariaDB up + seed CI). Adapter selon `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` si environnement Ubuntu 26.04+.
- [ ] T9.4 — Vérifier zéro `getByText()` brittle, zéro `.first()`/`.nth()`, strict mode ON.

### T10. Sync README + sprint-status

- [ ] T10.1 — README `## Feuille de route` : ne change pas (Epic 8 reste « Backlog » jusqu'à la fin de la dernière story de l'epic).
- [ ] T10.2 — Vérifier que `_bmad-output/implementation-artifacts/sprint-status.yaml` reflète `epic-8: in-progress` + `8-1-import-camt053: ready-for-dev` (mis à jour automatiquement par le workflow `bmad-create-story`).
- [ ] T10.3 — Pas de mise à jour de `## Fonctionnalités` du README v0.1 — l'item « Import bancaire CAMT.053 » sera retiré de la liste *(à venir)* uniquement quand Story 8-1 est `done` (à la PR de merge).

## Risque de splitting (CLAUDE.md check)

**Application de la règle CLAUDE.md « splitter si > 5 modules ou > 4 passes spec validate »** :

- **Modules touchés** : 6 distincts — `kesh-import`, `kesh-core::bank_imports`, `kesh-db` (migration + 2 entités + 2 repositories), `kesh-api/routes/bank_imports`, `frontend/lib/features/bank-import`, `kesh-i18n` (4 locales). **Au seuil.**
- **Profondeur d'incertitude** : faible. Le spike a déjà :
  - Validé l'architecture `kesh-import` autonome (RS-1 à RS-4 acceptés).
  - Posé les types + From/Into (T2.1-T2.6 = parser fill, pas conception).
  - Confirmé que les patterns existants (multi-tenant, repository, audit_log, RBAC, optimistic locking) s'appliquent sans réinvention.

**Recommandation** : créer la story comme spec unique et lancer `bmad-create-story validate`. Si la première passe spec validate produit > 4 findings MEDIUM/HIGH ou si le LLM validateur signale un scope trop large, **splitter alors** en :

- **8-1a — Parser `kesh-import` + types `kesh-core` + tests** (T1, T2, T3, T8.4, T8.5) : indépendant de la DB, livrable seul, validable par tests unitaires + cargo publish dry-run.
- **8-1b — Persistance + API + Frontend** (T4, T5, T6, T7, T8.1-T8.3, T9, T10) : consomme 8-1a comme dépendance, livré dans un second PR.

**Argument pour splitter pré-validate** (alternative) : Story 7-1 a explosé à 7 passes review sur 7+ modules cross-cutting ; éviter la rechute. La séparation parseur ↔ intégration est une frontière naturelle (cargo path dep, pas mécanique de patches dispersés).

**Argument pour rester unifié** (statu quo) : la story est cohérente comme livraison fonctionnelle « un utilisateur peut importer un CAMT.053 » ; la splitter en 8-1a + 8-1b livre un 8-1a sans valeur utilisateur perceptible (parseur sans UI). Le spike a déjà absorbé la profondeur technique du parseur.

**Décision à trancher avant `bmad-create-story validate`** : Guy choisit le format unique ou pré-split.

> **Si Guy choisit pré-split** : déplacer T1-T3 + T8.4-T8.5 dans une nouvelle story `8-1a-camt053-parser-only.md` (status `ready-for-dev`), garder le reste dans `8-1b-camt053-persistence-ui.md` (status `backlog` jusqu'à 8-1a `done`). Mettre à jour `sprint-status.yaml`.

## Dev Notes

### Patterns architecturaux à respecter

- **Multi-tenant scoping (Story 6-2 / 7-1, KF-002)** : tout repository appelle `WHERE company_id = ?` sur la première condition. Helper `bank_accounts::find_by_id_for_company` à créer (pattern aligné). Routes API utilisent `current_user.company_id` du JWT, pas une input client. Réponses cross-tenant = `404`, jamais `403`.
- **Optimistic locking** : `bank_transactions.version` posé pour Story 8-4. Pas mobilisé Story 8-1.
- **Audit log (Story 1-8 / 3-5)** : helper `audit_log::insert_in_tx(tx, user_id, action, target_table, target_id, details_json)`.
- **Erreurs structurées** : `AppError::Custom { status, code, message, details }` avec sérialisation JSON `{ error: { code, message, details } }` cohérente Story 1-11.
- **i18n key ownership (Story 6-3 / KF-006)** : préfixe `bankImport-` réservé exclusivement à ce module. Documenté dans `docs/i18n-key-ownership-pattern.md` (à étendre).
- **`rust_decimal` arithmétique** : utiliser `Decimal` de bout en bout. **Jamais** `f64`. Comparer avec `==` (scale-invariant côté `rust_decimal::Decimal::eq`).
- **Repository pattern + sqlx** : `pool: &MySqlPool` ou `&mut Transaction<'_, MySql>` ; pas d'ORM. SQL inline dans le repository, pas de query builder magique. `sqlx::QueryBuilder::push_values` pour bulk INSERT.
- **`From`/`Into` côté `kesh-core`** : décision archi #7. Ne pas implémenter `From<BankTransactionDraft> for ImportedTransaction` (pas utile, l'import est unidirectionnel).
- **No-op short-circuit (Story 7-3 / KF-004)** : non applicable Story 8-1 (pas d'update sur `bank_imports`).
- **Test locally first (CLAUDE.md)** : avant chaque push, lancer la séquence backend + frontend + (optionnel E2E si modif frontend).

### Source tree à toucher

**Backend** :
- `crates/kesh-import/Cargo.toml` (deps quick-xml)
- `crates/kesh-import/src/lib.rs` (re-exports)
- `crates/kesh-import/src/error.rs` *(nouveau)*
- `crates/kesh-import/src/camt053/mod.rs` *(nouveau)*
- `crates/kesh-import/src/camt053/v04.rs` *(nouveau)*
- `crates/kesh-import/src/camt053/v08.rs` *(nouveau)*
- `crates/kesh-import/tests/camt053_tests.rs` *(nouveau)*
- `crates/kesh-import/tests/fixtures/camt053/*.xml` × 8 *(nouveaux)*
- `crates/kesh-import/tests/fixtures/README.md` *(nouveau)*
- `crates/kesh-core/src/bank_imports.rs` (extension)
- `crates/kesh-core/src/errors.rs` (2 variantes)
- `crates/kesh-db/migrations/2026MMDD000001_bank_imports.sql` *(nouveau)*
- `crates/kesh-db/src/entities/bank_import.rs` *(nouveau)*
- `crates/kesh-db/src/entities/bank_transaction.rs` *(nouveau)*
- `crates/kesh-db/src/entities/mod.rs` (re-exports)
- `crates/kesh-db/src/repositories/bank_imports.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/bank_transactions.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/bank_accounts.rs` (extension : `find_by_id_for_company` si absent)
- `crates/kesh-db/src/repositories/mod.rs` (re-exports)
- `crates/kesh-db/tests/bank_imports_test.rs` *(nouveau ou inline `#[sqlx::test]`)*
- `crates/kesh-api/Cargo.toml` (feature multipart)
- `crates/kesh-api/src/routes/bank_imports.rs` *(nouveau)*
- `crates/kesh-api/src/routes/mod.rs` (re-export)
- `crates/kesh-api/src/lib.rs` (mountant 4 routes)
- `crates/kesh-api/src/errors.rs` (variantes)
- `crates/kesh-api/src/config.rs` (env var `KESH_BANK_IMPORT_MAX_MB`)
- `crates/kesh-api/tests/bank_imports_e2e.rs` *(nouveau)*

**i18n** :
- `crates/kesh-i18n/locales/fr-CH/messages.ftl`
- `crates/kesh-i18n/locales/de-CH/messages.ftl`
- `crates/kesh-i18n/locales/it-CH/messages.ftl`
- `crates/kesh-i18n/locales/en-CH/messages.ftl`

**Frontend** :
- `frontend/src/lib/shared/utils/api-client.ts` (extension FormData)
- `frontend/src/lib/features/bank-import/bank-import.types.ts` *(nouveau)*
- `frontend/src/lib/features/bank-import/bank-import.api.ts` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportPreviewTable.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportList.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportDetail.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankAccountSelector.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportUpload.test.ts` *(nouveau)*
- `frontend/src/routes/(app)/bank-import/+page.svelte` (remplacement)
- `frontend/src/routes/(app)/bank-import/+page.ts` *(nouveau)*
- `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` *(nouveau)*
- `frontend/src/routes/(app)/bank-import/[id]/+page.ts` *(nouveau)*
- `frontend/tests/e2e/bank-import.spec.ts` *(nouveau)*
- `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` *(nouveau)*

**CI** :
- `.github/workflows/ci.yml` (cargo publish --dry-run + cargo metadata invariant)

### Conventions de naming

- DB : `snake_case`, pluriel pour les tables (`bank_imports`, `bank_transactions`).
- Rust : `snake_case` modules / fns / fields ; `PascalCase` types ; `SCREAMING_SNAKE_CASE` const.
- API JSON : `camelCase` via `#[serde(rename_all = "camelCase")]`.
- Frontend types : `PascalCase` interfaces, `camelCase` fields.
- i18n : `bankImport-{kebab-case-id}` (préfixe + tiret).
- Routes API : `/api/v1/bank-imports/...` (kebab-case plural).
- Routes SvelteKit : `/bank-import` (kebab-case singulier — convention existante des routes app).

### Standards de test

- **Unitaires Rust** : `cargo test --workspace -j1 -- --test-threads=1` aligné CI.
- **Intégration sqlx** : `#[sqlx::test]` avec migration auto + fixtures inline.
- **E2E HTTP** : `crates/kesh-api/tests/bank_imports_e2e.rs` avec helper `setup_test_app()` existant (cf. `invoices_e2e.rs`).
- **Vitest frontend** : `npm run test:unit -- bank-import` ; couverture sur la state machine + validations client.
- **Playwright** : `npm run test:e2e -- bank-import.spec.ts` ; pré-requis MariaDB + seed CI + browsers installés (cf. CLAUDE.md « Test Locally First → E2E »).

### Références

#### Source paths (citations exactes)

- ACs Epic 8 : [`_bmad-output/planning-artifacts/epic-8.md`](../planning-artifacts/epic-8.md#story-8-1--import-camt053) lignes 90-108
- FR42, FR43, FR49, FR50, FR51, FR87 : [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md) lignes 436-447 + 520
- Décision archi #2 (multi-version parsers) + #7 (types autonomes) : [`_bmad-output/planning-artifacts/architecture.md`](../planning-artifacts/architecture.md) §76 + §11.5 ligne 686
- Structure `kesh-import` cible : architecture.md lignes 530-540
- Mapping FR → modules : architecture.md ligne 639
- Spike outcome + 4 implications Story 8-1 : [`spike-kesh-import.md`](spike-kesh-import.md) §Implications Story 8-1
- Pattern multi-tenant scoping : [`docs/MULTI-TENANT-SCOPING-PATTERNS.md`](../../docs/MULTI-TENANT-SCOPING-PATTERNS.md) (Pattern 1-5)
- Pattern i18n key ownership : [`docs/i18n-key-ownership-pattern.md`](../../docs/i18n-key-ownership-pattern.md)
- Pattern optimistic locking : [`docs/optimistic-locking-patterns.md`](../../docs/optimistic-locking-patterns.md)
- CR-010 #62 (statement balance check) : [github.com/guycorbaz/kesh/issues/62](https://github.com/guycorbaz/kesh/issues/62)
- Helper deadlock-retry : `crates/kesh-db/src/retry.rs` (KF-002-H-002 / #43)
- Pattern bulk INSERT : `crates/kesh-db/src/repositories/journal_entries.rs` (Story 3-2 — `QueryBuilder::push_values`)
- Pattern multipart Axum 0.8 : [docs.rs/axum/0.8/axum/extract/struct.Multipart.html](https://docs.rs/axum/0.8/axum/extract/struct.Multipart.html)

#### Issues et CRs

- CR-009 #61 (epics.md drift Epic 7 → Epic 8) — non bloquant pour 8-1, à clore avant `epic-9.md`.
- CR-010 #62 (statement balance check FR42-bis) — **intégré ICI** comme AC #14.
- KF-022 #54 (E2E helpers cascade 401), KF-023 #55 (axe-core a11y), KF-025 #57 (E2E timing) — cleanup parallèle, non bloquants pour 8-1 mais à surveiller pour ne pas hériter d'un baseline E2E fragile.

### Checklist locale avant push (cf. CLAUDE.md « Test Locally First »)

```sh
# Backend
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1
cargo publish --dry-run --allow-dirty -p kesh-import   # AC #7
cargo metadata -p kesh-import --format-version 1 | jq '.packages[0].dependencies[] | select(.path != null)'   # AC #6 (doit retourner vide)

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # AC #19
npm run test:unit
npm run build

# E2E (si MariaDB up + seed CI)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- bank-import.spec.ts
```

## Dev Agent Record

### Agent Model Used

(à remplir par le dev agent au moment de l'implémentation — Opus 4.7 [1M context] recommandé pour cette story compte tenu du scope)

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Action | Auteur |
|------|--------|--------|
| 2026-05-03 | Création de la story via `/bmad-create-story 8-1`. Spec construite à partir d'`epic-8.md`, du spike `kesh-import`, du retro Epic 7, et des patterns établis Stories 6-2 / 7-1 / 7-3. Splitting check appliqué (6 modules — au seuil). | Claude (SM, Opus 4.7) |
