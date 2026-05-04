# Story 8.1b: Persistance + API + UI Import CAMT.053

Status: ready-for-dev

<!-- Issue de scission de Story 8-1 (`8-1-import-camt053.md`) le 2026-05-04 :
     8-1b reste backlog jusqu'à 8-1a (parser kesh-import + helpers kesh-core) `done`.
     Dépend de 8-1a comme path dep stable. Voir 8-1-import-camt053.md (status `archived-split`)
     pour les décisions de conception détaillées. -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse)**,
I want **importer mes relevés bancaires CAMT.053 via la page `/bank-import` (drag-drop, prévisualisation, confirmation), avec persistance atomique multi-tenant + audit log**,
so that **mes transactions apparaissent dans Kesh, prêtes pour la réconciliation (Stories 8-3/8-4) sans ressaisie manuelle**.

### Contexte

**Story 8-1b = seconde moitié de la story unifiée 8-1**, scindée pré-implémentation pour respecter la règle de splitting CLAUDE.md (> 5 modules touchés). Voir [`8-1-import-camt053.md`](8-1-import-camt053.md) (status `archived-split`) pour la spec d'origine — décisions §schéma, §upload-limit, §perf, §api-client, §multi-stmt y sont détaillées.

**Dépendance bloquante** : Story 8-1a (`8-1a-camt053-parser-only`) doit être stable (review-closed avec 0 findings > LOW) avant que 8-1b ne soit `ready-for-dev`. 8-1b consomme :
- `kesh_import::parse_camt053` (re-export public canonique du parser CAMT.053 — F5 validate Pass 1, équivalent à `kesh_import::camt053::parse` mais signale l'API surface publiable stable)
- `kesh_import::{ImportedStatement, ImportedTransaction, SourceFormat}` (types autonomes re-exportés via `pub use types::{...}`)
- `kesh_core::bank_imports::{BankImportDraft, SourceFormatTag, from_imported, validate_balance, validate_currency_supported_v0_1}` (extensions 8-1a) + `SourceFormatTag::as_db_str()` (projection MariaDB)
- `kesh_core::errors::CoreError::{BankImportBalanceMismatch, BankImportUnsupportedCurrency, BankImportUnknownVersion}` (variantes 8-1a)
- `kesh_import::CamtError` (5 variantes : `MalformedXml(String)`, `UnsupportedVersion(String)`, `MissingRequiredField(String)` — **note F7 validate Pass 1** : type passé de `&'static str` à `String` post-Pass-1 8-1a code review pour permettre les dot-paths indexés `stmt[i].ntry[j].field`, `InvalidAmount(String)`, `InvalidDate(String)`)

**Status sprint** : `8-1b-camt053-persistence-ui: ready-for-dev` post-validate Pass 1 Sonnet 2026-05-04. La transition `backlog → ready-for-dev` a été appliquée après que 8-1a ait clos son cycle code review 3 passes (commits e1c3052/399f761/65e3f56, 0 findings > LOW Pass 3). 8-1a et 8-1b seront mergés en **PR groupée** (mémoire utilisateur PR grouping when stories share files — chevauchement attendu sur `kesh-core::bank_imports` et `kesh-import::lib`).

### Scope verrouillé — ce qui est livré par 8-1b

1. **Migration DB** (T4) — `crates/kesh-db/migrations/2026MMDD000001_bank_imports.sql` créant `bank_imports` + `bank_transactions` (cf. spec d'origine §T4 pour le DDL exact).
2. **Entités + repositories `kesh-db`** (T5) — `entities::{BankImport, NewBankImport, BankImportSourceFormat, BankTransaction, NewBankTransaction, BankTransactionStatus}` + `repositories::{bank_imports, bank_transactions}` + helper `bank_accounts::find_by_id_for_company`.
3. **Route API `kesh-api/routes/bank_imports.rs`** (T6) — 4 endpoints (`POST /preview`, `POST /bank-imports`, `GET /bank-imports`, `GET /bank-imports/{id}`), feature `axum/multipart`, dépendance `sha2`, `DefaultBodyLimit`, audit log, mapping erreurs SQL 1062 → `409`, RBAC via `comptable_routes`.
4. **Frontend feature `bank-import`** (T7) — `frontend/src/lib/features/bank-import/` (components Svelte + types + api), routes `/bank-import` et `/bank-import/[id]`, extension `api-client.ts` pour `FormData`.
5. **i18n** (T8.1-T8.3) — clés `bank-import-*` dans 4 locales `fr/de/it/en-CH`, lint-i18n-ownership pass.
6. **Tests E2E Playwright** (T9) — `frontend/tests/e2e/bank-import.spec.ts` (6 scénarios), fixture `camt053_v04_minimal.xml`.
7. **Sync sprint-status + README** (T10).

**HORS scope 8-1b (héritages 8-1a, déjà livrés) :**
- Parseur `kesh-import::camt053`
- Helpers `kesh-core::bank_imports`
- Variantes `CoreError::BankImport*`
- Invariants CI `cargo publish --dry-run` + `cargo metadata`
- Fixtures CAMT.053 (`crates/kesh-import/tests/fixtures/camt053/*.xml`)

### Décisions de conception (rappel — voir spec d'origine pour le détail)

Toutes les décisions §schéma, §upload-limit, §perf, §api-client, §multi-stmt (exécution UI), §balance-check (mapping API), §currency (mapping API) de [`8-1-import-camt053.md`](8-1-import-camt053.md) §317-447 s'appliquent telles quelles à 8-1b. Points critiques pour la couche persistance + API + UI :

- **§schéma** : `bank_imports` + `bank_transactions` avec `company_id` partout (multi-tenant Pattern 1 KF-002), `(company_id, file_hash) UNIQUE`, `idx_bank_transactions_pending` couvre Story 8-4.
- **§upload-limit** : **10 MiB** hard limit (binaire = `10 * 1024 * 1024` = 10 485 760 bytes), configurable via env var `KESH_BANK_IMPORT_MAX_MB` (entier en MiB malgré le suffixe `_MAX_MB` — convention industrielle de raccourci, voir M3 validate Pass 2). Appliqué via `axum::extract::DefaultBodyLimit::max(max_mib * 1024 * 1024)` sur le sub-router bank-import. Pas besoin de feature `tower-http "limit"`. **Cohérence à respecter dans T6.10 + .env.example + tests E2E `rejects file > 10 MB` : tous les sites utilisent l'interprétation binaire (1 MiB = 1024² bytes), pas décimale (10⁶).**
- **§perf** : bulk INSERT via `sqlx::QueryBuilder::push_values` (pattern Story 3-2), batch chunks de 1000 si > 1000 transactions / fichier.
- **§api-client** : extension `frontend/src/lib/shared/utils/api-client.ts` pour accepter `body: FormData` sans forcer `Content-Type: application/json`. Test unitaire dédié.
- **§multi-stmt (exécution)** : handler API filtre `Vec<ImportedStatement>` par IBAN sélectionné par l'utilisateur (normalisation `replace(' ', '').to_uppercase()`). Statements non-matchants → liste `ignoredStatements` dans la réponse `preview`. Aucun match → `422 BANK_IMPORT_NO_MATCHING_STATEMENT`.
- **§balance-check (mapping API)** :
  - `POST /preview` : balance mismatch → `200 OK + warnings: ["balance_mismatch"]` (transactions tout de même listées).
  - `POST /bank-imports` final : balance mismatch → `422 BANK_IMPORT_BALANCE_MISMATCH` sauf si form contient `confirmBalanceMismatch=true` → `201 Created` + audit log `bank_import.created_with_balance_mismatch`.
- **§currency (mapping API)** :
  - `POST /preview` : currency != CHF → `200 OK + warnings: ["unsupported_currency"]`.
  - `POST /bank-imports` final : currency != CHF → `422 BANK_IMPORT_UNSUPPORTED_CURRENCY` (pas de bypass v0.1).

## Acceptance Criteria

Numérotation héritée de la spec 8-1 d'origine pour traçabilité. Les ACs marqués **Hérité 8-1a** sont déjà validés à la fin de 8-1a et non re-testés ici (hors scope).

1. **(FR42)** Given un fichier CAMT.053 valide (v04 ou v08), When l'utilisateur clique « Importer », Then toutes les transactions sont extraites et persistées avec date booking, montant signé, référence, détails, contrepartie. *Test : E2E `imports a CAMT.053 v04 file end-to-end`.*

2. **(FR49)** *Hérité 8-1a — algorithme couvert par `parse_with_subtxs_extracts_individual_transactions`. 8-1b ne re-teste pas mais relie le test E2E `imports a CAMT.053 v04 file end-to-end` qui consomme la fixture `v04_with_subtxs.xml`.*

3. **(FR50 — multi-stmt scoping + ignored)** Given un import, When l'utilisateur a sélectionné un `bankAccountId`, Then deux comportements distincts doivent être validés :

   - **3a** (FK persistance) — toutes les transactions persistées ont `bank_account_id = selected_id`. *Test : `post_import_creates_rows_atomically` (vérifie que chaque ligne `bank_transactions` a la FK attendue).*
   - **3b** (statements non-matchants `ignoredStatements`) — si le fichier contient des `<Stmt>` pour d'autres IBAN, ces statements sont **ignorés** (pas persistés) avec un warning explicite dans la réponse preview (`response.warnings.ignoredStatements: [{statementId, iban}]`). *Test : `post_preview_returns_ignored_statements_for_multi_stmt_file` (E2E HTTP, charge `v04_multi_stmt.xml`, sélectionne un seul IBAN, asserte `warnings.ignoredStatements.length == 1`).*
   - **3c** (FR50 — multi-stmt aucun matching) — Given un fichier multi-stmt dont aucun `<Stmt>` ne contient l'IBAN du `bankAccountId` sélectionné, When `POST /bank-imports`, Then `422 BANK_IMPORT_NO_MATCHING_STATEMENT` avec `details.found_ibans = [...]` listant les IBANs présents dans le fichier (pour aider l'utilisateur à corriger le compte cible). *Test : `post_import_rejects_when_no_stmt_matches_selected_account` (E2E HTTP).* — F4 validate Pass 1.

   Scénario E2E Playwright complémentaire : `requires bank account selection before upload` (vérifie le chemin UI complet : aucun upload sans bank account selected).

4. **(UX)** Given l'utilisateur sur la page `/bank-import`, When il glisse un fichier ou clique « Sélectionner », Then une **prévisualisation des transactions** s'affiche avant la persistance définitive. La confirmation explicite (bouton « Confirmer l'import ») déclenche le `POST /bank-imports`. *Test E2E + `BankImportUpload.test.ts`.*

5. **(FR87)** *Hérité 8-1a.*

6. **(Architecture zéro path dep `kesh-import`)** *Hérité 8-1a.*

7. **(Architecture cargo publish dry-run)** *Hérité 8-1a.*

8. **(`From`/`Into` côté `kesh-core`)** *Hérité 8-1a.*

9. **(Fixtures synthétiques)** *Hérité 8-1a.*

10. **(Schéma multi-tenant — KF-002 pattern)** Given une transaction bancaire persistée pour `company_A`, When `company_B` appelle `GET /bank-imports/{id}` ou `GET /bank-imports?bankAccountId=...`, Then la réponse est `404 Not Found` (jamais 403 — pattern KF-002). *Tests : `get_imports_lists_only_own_company`, `get_import_returns_404_for_other_company_id`.*

11. **(Multi-tenant DB)** Given la migration appliquée, When `cargo test -p kesh-db bank_imports`, Then **aucun cross-tenant leak** n'est observé sur les **9 tests** d'intégration listés en T5.6 (7 tests `bank_imports` + 2 tests `bank_accounts::find_by_id_for_company` IDOR — F2 validate Pass 1, alignement T5.6 / AC #11).

12. **(Sécurité — RBAC)** Given un utilisateur avec `role = Role::Consultation`, When il tente `POST /bank-imports`, Then la réponse est `403` (renvoyée par le middleware `require_comptable_role` du sub-router `comptable_routes`). Les rôles `Role::Comptable` et `Role::Admin` peuvent importer ; tous les rôles authentifiés peuvent lire (GET sur `authenticated_routes`). *Test : `post_import_rejects_when_role_consultation`.*

13. **(Sécurité — payload limit)** Given un fichier > 10 MiB (= 10 485 760 bytes binaire, voir §upload-limit), When upload, Then la réponse est `413 BANK_IMPORT_TOO_LARGE`. Aucun parsing n'est tenté. *Test : `post_import_rejects_payload_too_large` (génère un buffer de 10 485 761 bytes pour reproduire l'overflow d'un seul byte au-dessus de la limite).*

14. **(CR-010 #62 — balance check mapping API)** Given un fichier où `|opening + Σ transactions - closing| > 0.01`, When `POST /bank-imports` sans `confirmBalanceMismatch`, Then `422 BANK_IMPORT_BALANCE_MISMATCH`. When avec `confirmBalanceMismatch=true`, Then `201 Created` + entrée audit_log `bank_import.created_with_balance_mismatch`. *Tests : `post_import_rejects_balance_mismatch_without_confirm` + `post_import_accepts_balance_mismatch_with_confirm`. (L'algo `validate_balance` est hérité 8-1a — AC #14a.)*

15. **(Devise v0.1 — mapping API)** Given un fichier en `<Acct><Ccy>EUR</Ccy>`, When `POST /bank-imports`, Then `422 BANK_IMPORT_UNSUPPORTED_CURRENCY`. Le `POST /preview` affiche les transactions avec un warning `unsupportedCurrency`. *Test : `post_import_rejects_eur_currency`. (L'algo est hérité 8-1a — AC #15a.)*

16. **(Doublons fichier — préparation Story 8-3)** Given un fichier déjà importé pour la même company (même `file_hash`), When second import, Then `409 BANK_IMPORT_DUPLICATE_FILE`. Le même fichier importé pour une autre company → `201 Created` (multi-tenant safety). *Test : `unique_company_hash_blocks_duplicate_within_same_company` + `unique_company_hash_allows_same_hash_across_companies`.*

17. **(Atomicité)** Given un import qui échoue côté `bank_transactions` (ex. erreur SQL ligne 150 sur 200), When la transaction DB rollback, Then `bank_imports` ET `bank_transactions` sont vides en DB (rien d'orphelin). *Test : `create_with_transactions_rolls_back_on_constraint_violation`.*

18. **(Audit log)** Given un import réussi, When `SELECT FROM audit_log`, Then une entrée `action = bank_import.created` (ou `bank_import.created_with_balance_mismatch`) est présente avec `entity_type = "bank_imports"`, `entity_id = bank_import.id`, `user_id = importer`, `details_json` contenant `{ filename, transaction_count, source_format }`. *Test dédié (F3 validate Pass 1) : `post_import_audit_log_contains_correct_entry` — assert isolément l'entrée audit_log post-création (pas un side-assert dans `post_import_creates_rows_atomically` qui couvre déjà la persistance atomique). Le test charge un fichier valide, `POST /bank-imports`, puis `SELECT * FROM audit_log WHERE entity_type='bank_imports' AND entity_id=?` et vérifie chaque champ (action, entity_type, entity_id, user_id, details_json).*

19. **(i18n)** Given les 4 locales (fr/de/it/en-CH), When `npm run lint-i18n-ownership`, Then le lint passe sans erreur (toutes les clés `bank-import-*` présentes dans les 4 fichiers, préfixe kebab-case strict). *Test : CI Story 6-3.*

20. **(Accessibilité)** Given la page `/bank-import` rendue, When `axe-core` scan, Then zéro violation. Le drag-drop est une augmentation : un input `<input type="file" aria-label>` reste navigable au clavier. *Test : E2E `accessibility — axe scan zero violations`.*

21. **(Performance NFR)** Given un fichier de 200 transactions CAMT.053 v04, When `POST /bank-imports`, Then la durée totale (parse + DB) < 2s sur la machine de dev nominale. *Test : `bulk_insert_handles_500_transactions` instrumenté avec `Instant::now()` (smoke, pas un seuil CI strict).*

## Tasks / Subtasks

Sections T4 → T10 héritées **telles quelles** de [`8-1-import-camt053.md`](8-1-import-camt053.md) §493-606. Voir cette spec pour le détail intégral des subtasks. Résumé pour mémoire :

### T4. Migration DB (AC #11, #16, #17)

- [ ] T4.1 — Créer `crates/kesh-db/migrations/2026MMDD000001_bank_imports.sql` (DDL exact dans la spec d'origine §T4). **Note F8 validate Pass 1** : le commentaire de la spec d'origine donne des exemples de valeurs `source_format` en minuscules (`'camt053_v04'`, `'camt053_v08'`). Le code livré 8-1a (`SourceFormatTag::as_db_str()`) produit des **MAJUSCULES** (`'CAMT053_V04'`, `'CAMT053_V08'`). Utiliser les MAJUSCULES dans le DDL (commentaires inclus) et dans tout futur `CHECK` constraint — c'est ce que le code persistera réellement.
- [ ] T4.2 — Vérifier l'application sur DB fraîche.
- [ ] T4.3 — Réversibilité manuelle vérifiée (`DROP TABLE bank_transactions; DROP TABLE bank_imports;`).

### T5. Entités + repositories (AC #11, #16, #17)

- [ ] T5.1 — Entités `BankImport` / `BankTransaction` + enums `BankImportSourceFormat` / `BankTransactionStatus` avec `#[derive(sqlx::FromRow)]` (cf. spec d'origine §T5 + Pass 2 H3).

  **Pattern sqlx enum (F10 validate Pass 1 + M1 validate Pass 2)** : les enums `BankImportSourceFormat` et `BankTransactionStatus` sont stockés en `VARCHAR` MariaDB et **doivent** soit (a) implémenter manuellement `sqlx::Type<MySql> + Encode + Decode` (pattern `Role` dans `crates/kesh-db/src/entities/user.rs`), soit (b) utiliser `#[sqlx(try_from = "String")]` sur les champs des structs parentes `BankImport` / `BankTransaction`. **Sans ce pattern**, `sqlx::query_as::<_, BankImport>(...)` ne compile pas.

  **Option (b) recommandée pour 8-1b** (moins de boilerplate). Exemple minimal :

  ```rust
  // entities/bank_import.rs
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum BankImportSourceFormat {
      Camt053V04,
      Camt053V08,
  }

  impl TryFrom<String> for BankImportSourceFormat {
      type Error = String;
      fn try_from(s: String) -> Result<Self, Self::Error> {
          match s.as_str() {
              "CAMT053_V04" => Ok(Self::Camt053V04),
              "CAMT053_V08" => Ok(Self::Camt053V08),
              other => Err(format!("BankImportSourceFormat inconnu : {other}")),
          }
      }
  }

  impl From<BankImportSourceFormat> for String {
      fn from(v: BankImportSourceFormat) -> Self {
          match v {
              BankImportSourceFormat::Camt053V04 => "CAMT053_V04".into(),
              BankImportSourceFormat::Camt053V08 => "CAMT053_V08".into(),
          }
      }
  }

  #[derive(sqlx::FromRow)]
  pub struct BankImport {
      pub id: i64,
      pub company_id: i64,
      // ...
      #[sqlx(try_from = "String")]
      pub source_format: BankImportSourceFormat,
      // ...
  }
  ```

  Référence canonique pour l'option (a) : `crates/kesh-db/src/entities/user.rs::Role` impl `sqlx::Type<MySql>`. Aligner les valeurs string sur `kesh_core::bank_imports::SourceFormatTag::as_db_str()` (= `"CAMT053_V04"`/`"CAMT053_V08"`, MAJUSCULES — voir T4.1 note F8).

- [ ] T5.2 — Inscription `entities/mod.rs`.
- [ ] T5.3 — `repositories::bank_imports` (`create_with_transactions`, `find_by_company_id`, `find_by_company_and_hash`).
- [ ] T5.4 — `repositories::bank_transactions::list_by_import` (filtré company_id + import_id, KF-002 double-scope).
- [ ] T5.5 — Inscription `repositories/mod.rs`.
- [ ] T5.6 — Tests intégration `#[sqlx::test]` (**9 tests** — F2 validate Pass 1 inline ci-dessous au lieu du renvoi vers spec d'origine) :

  1. `create_with_transactions_atomic_success` — INSERT bank_import + N bank_transactions dans une transaction, COMMIT, vérifie les rows persistées.
  2. `create_with_transactions_rolls_back_on_constraint_violation` — INSERT avec une violation FK (ex. `bank_account_id` inexistant) → tx rollback, aucune ligne en DB.
  3. `find_by_company_id_only_returns_own_imports` — multi-tenant : 2 companies, 2 imports, `find_by_company_id(company_A)` ne retourne que les imports de A.
  4. `find_by_company_and_hash_finds_existing` — détection doublon happy-path.
  5. `unique_company_hash_blocks_duplicate_within_same_company` — INSERT 2× même `(company_id, file_hash)` → second échec sur `uq_bank_imports_company_hash` (erreur SQL 1062).
  6. `unique_company_hash_allows_same_hash_across_companies` — multi-tenant safety : même hash sur companies différentes → 2 INSERT successful.
  7. `bulk_insert_handles_500_transactions` — perf smoke test : 500 lignes via `QueryBuilder::push_values` en chunks de 1000.
  8. `find_by_id_for_company_rejects_wrong_company` — IDOR sur `bank_accounts::find_by_id_for_company` : company_B ne peut pas lire un bank_account de company_A → `Ok(None)`.
  9. `find_by_id_for_company_returns_account_when_owned` — happy path correspondant à #8.

  Tests #8 et #9 sont des tests de sécurité IDOR sur le helper `bank_accounts::find_by_id_for_company` créé en T6.3. **Localisation figée (M4 validate Pass 2)** : ces tests vivent dans le **fichier de test du module `bank_accounts`** (cohésion fonction-tests, le helper testé est `bank_accounts::find_by_id_for_company` — pas un appel via `bank_imports`). Pratique : soit étendre `crates/kesh-db/src/repositories/bank_accounts.rs` avec un module `#[cfg(test)] mod tests { ... }` inline, soit créer/étendre `crates/kesh-db/tests/bank_accounts_test.rs`. **Ne pas mélanger avec les 7 autres tests bank_imports** (qui restent dans `bank_imports_test.rs` ou inline `bank_imports.rs`).

### T6. Route API `bank_imports.rs` (AC #1, #4, #10, #12, #13, #14, #15, #16, #18)

- [ ] T6.1 — Activer `axum = { features = ["multipart"] }` + ajouter `sha2 = "0.10"` (`kesh-api/Cargo.toml`).
- [ ] T6.2 — 4 handlers (`preview`, `create`, `list`, `detail`).
- [ ] T6.3 — Implémenter `bank_accounts::find_by_id_for_company(pool, company_id, id)` + 2 tests (cross-tenant + happy path) — H5 Pass 1.
- [ ] T6.4 — Étendre `AppError` avec **7 variantes + 7 bras `IntoResponse`** (cf. spec d'origine §T6.4 — table exhaustive). **Note F4 validate Pass 1** : la liste inclut `BankImportNoMatchingStatement { found_ibans: Vec<String> }` (mappé `422 BANK_IMPORT_NO_MATCHING_STATEMENT`, AC #3c) — bien rappeler ce bras lors de l'implémentation, il est facile à oublier en se basant uniquement sur la liste des codes HTTP de 8-1b sans relire la table de la spec d'origine.
- [ ] T6.5 — Mapping erreur SQL 1062 sur `uq_bank_imports_company_hash` → `409 BANK_IMPORT_DUPLICATE_FILE` via catch avant `?` (pattern `invoices::create` — cf. spec d'origine §T6.5).
- [ ] T6.6 — Mountant routes : POST → `comptable_routes`, GET → `authenticated_routes`.
- [ ] T6.7 — Vérifier RBAC = middleware sub-router (`require_comptable_role`), pas de check inline.
- [ ] T6.8 — Audit log via `audit_log::insert_in_tx(tx, NewAuditLogEntry { ... })` (signature exacte cf. spec d'origine §T6.8).
- [ ] T6.9 — Tests E2E HTTP `crates/kesh-api/tests/bank_imports_e2e.rs` (**13 tests** — total porté de 12 → 13 par F1+F3+F4 validate Pass 1) :

  Liste indicative (à raffiner pendant dev) :
  1. `post_import_creates_rows_atomically` (AC #1, #3a, #17) — happy path, vérifie INSERT atomique bank_import + bank_transactions.
  2. `post_preview_returns_ignored_statements_for_multi_stmt_file` (AC #3b — F1) — preview multi-stmt, IBAN sélectionné parmi plusieurs, asserte `warnings.ignoredStatements`.
  3. `post_import_rejects_when_no_stmt_matches_selected_account` (AC #3c — F4) — multi-stmt aucun matching, asserte `422 BANK_IMPORT_NO_MATCHING_STATEMENT` + `details.found_ibans`.
  4. `post_import_rejects_when_role_consultation` (AC #12) — RBAC, role = Consultation → 403.
  5. `post_import_rejects_payload_too_large` (AC #13) — `> KESH_BANK_IMPORT_MAX_MB` → 413 BANK_IMPORT_TOO_LARGE.
  6. `post_import_rejects_balance_mismatch_without_confirm` (AC #14) — diff > 0.01 sans `confirmBalanceMismatch` → 422.
  7. `post_import_accepts_balance_mismatch_with_confirm` (AC #14) — `confirmBalanceMismatch=true` → 201 + audit `created_with_balance_mismatch`.
  8. `post_import_rejects_eur_currency` (AC #15) — `<Acct><Ccy>EUR</Ccy>` → 422 BANK_IMPORT_UNSUPPORTED_CURRENCY.
  9. `unique_company_hash_blocks_duplicate_within_same_company` (AC #16) — second import même `file_hash` même company → 409.
  10. `unique_company_hash_allows_same_hash_across_companies` (AC #16) — multi-tenant safety.
  11. `post_import_audit_log_contains_correct_entry` (AC #18 — F3) — assert isolément l'audit_log post-création.
  12. `get_imports_lists_only_own_company` (AC #10) — multi-tenant scoping GET.
  13. `get_import_returns_404_for_other_company_id` (AC #10) — IDOR cross-tenant → 404 (jamais 403, pattern KF-002).
- [ ] T6.10 — `DefaultBodyLimit::max(max_mib * 1024 * 1024)` (interprétation binaire MiB) + env `KESH_BANK_IMPORT_MAX_MB` (default 10) + `.env.example` mis à jour avec commentaire explicite : `# Taille max upload bank-import en MiB binaire (1 MiB = 1024² bytes ; 10 = 10 485 760 bytes)`. Cohérence M3 validate Pass 2.

### T7. Frontend feature `bank-import` (AC #1, #4, #20)

- [ ] T7.1 — Étendre `frontend/src/lib/shared/utils/api-client.ts` pour `body: FormData`.
- [ ] T7.2 — `frontend/src/lib/features/bank-import/` (types, api, store).
- [ ] T7.3 — `BankImportUpload.svelte` (drag-drop + file input accessibles + state machine).
- [ ] T7.4 — `BankImportPreviewTable.svelte`.
- [ ] T7.5 — `BankImportList.svelte` + `BankImportDetail.svelte` + `BankAccountSelector.svelte`.
- [ ] T7.6 — Remplacer placeholder `frontend/src/routes/(app)/bank-import/+page.svelte` + `+page.ts` + `[id]/+page.svelte` + `[id]/+page.ts`.
- [ ] T7.7 — `data-testid` partout (lesson Story 7-5/KF-008).
- [ ] T7.8 — Tests Vitest `BankImportUpload.test.ts`.

### T8. i18n (AC #19)

- [ ] T8.1 — Clés `bank-import-*` dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (FR donné §scope T9 spec d'origine).
- [ ] T8.2 — Traductions DE / IT / EN.
- [ ] T8.3 — Vérifier `npm run lint-i18n-ownership` pass.

### T9. Tests E2E Playwright (AC #1, #4, #13, #14, #20)

- [ ] T9.1 — `frontend/tests/e2e/bank-import.spec.ts` — **6 scénarios** (F13 validate Pass 1, inline pour éviter renvoi à la spec d'origine) :

  1. `imports a CAMT.053 v04 file end-to-end` (AC #1) — drag-drop `v04_minimal.xml`, sélectionne bank account, preview, confirm, vérifie redirection sur `/bank-import/[id]` + transactions listées.
  2. `requires bank account selection before upload` (AC #3 / #4) — l'upload est désactivé tant qu'aucun `bankAccountId` n'est sélectionné ; un message d'erreur ARIA s'affiche en cas de tentative.
  3. `shows balance mismatch warning and accepts override` (AC #14) — preview affiche `warnings.balance_mismatch`, checkbox `confirmBalanceMismatch` apparaît, confirm → 201.
  4. `rejects file > 10 MiB` (AC #13) — upload d'un fichier 12 MiB binaire (12 × 1024² bytes) → toast erreur `bank-import-error-too-large`. **Note** : tester en MiB cohérent avec §upload-limit (pas en MB décimal).
  5. `lists previous imports paginated` (AC #10) — page `/bank-import` liste les imports précédents avec pagination.
  6. `accessibility — axe scan zero violations` (AC #20) — `axe-core` scan sur la page principale + page detail, zéro violation.

- [ ] T9.2 — Fixture `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` (copie depuis `crates/kesh-import/tests/fixtures/camt053/v04_minimal.xml`). Pas besoin de copier les 12 fixtures kesh-import — uniquement le minimal pour le scénario happy path. Pour le scénario balance_mismatch, copier aussi `v04_balance_mismatch.xml`.
- [ ] T9.3 — `npm run test:e2e -- bank-import.spec.ts` localement.
- [ ] T9.4 — Zéro `getByText()` brittle, zéro `.first()/.nth()`, strict mode ON (lessons KF-008 Story 7-5, KF-010 Story 7-6).

### T10. Sync README + sprint-status

- [ ] T10.1 — README `## Feuille de route` : ne change pas (Epic 8 reste « Backlog » jusqu'à la fin de la dernière story de l'epic).
- [ ] T10.2 — `sprint-status.yaml` : `8-1b-camt053-persistence-ui: review` après push final.
- [ ] T10.3 — README `## Fonctionnalités` : retirer le marqueur *(à venir)* sur l'item « Import bancaire CAMT.053 » UNIQUEMENT à la PR de merge 8-1b (pas avant).

## Risque de splitting (CLAUDE.md check)

**Modules touchés par 8-1b** : 5 (`kesh-db`, `kesh-api`, `frontend`, `kesh-i18n`, fichiers CI/migrations). Au seuil > 5 ? Non — exactement à la limite acceptable. **Pas de re-split** : ces 5 modules forment une livraison cohérente « persister + servir l'UI » et ne se réduisent pas naturellement à des sous-livraisons indépendantes (le frontend a besoin de l'API, l'API a besoin de la DB, l'i18n est consommé par le frontend, les E2E tournent contre la pile complète).

Argument additionnel : l'incertitude technique de chaque module est faible car les patterns sont établis (multi-tenant scoping Story 7-1, audit log Story 1-8, RBAC Story 1-8, optimistic locking Stories 6-2/7-3, fetch wrapper Story 1-11, i18n key ownership Story 6-3, E2E selectors Story 7-5/7-6). Le risque résiduel principal est **la quantité** (lignes de code), pas la profondeur conceptuelle.

## Dev Notes

### API surface 8-1a livrée — drifts vs spec d'origine à connaître (validate Pass 1)

Pendant le cycle code review 8-1a (3 passes Opus → Sonnet → Haiku), 22 patches F1-F22 ont été appliqués. Plusieurs touchent l'API surface que 8-1b consomme. **Lire avant d'implémenter T6 (mapping `CoreError` / `CamtError` → `AppError`)** :

- **F6 — `SourceFormat::Csv → CoreError::BankImportUnknownVersion("csv")`** : si pour une raison quelconque un `SourceFormat::Csv { .. }` arrive à `from_imported(...)` (improbable dans 8-1b où le parser est `parse_camt053` — qui n'émet que `SourceFormat::Camt053`, mais possible si une future intégration partage `from_imported`), la fonction retourne `Err(CoreError::BankImportUnknownVersion("csv".into()))`. Mapper côté handler vers le **même bras** que les versions CAMT inconnues : `400 BANK_IMPORT_UNSUPPORTED_VERSION` avec `details.version = "csv"`. **Ne pas créer de bras dédié `BANK_IMPORT_NOT_CSV` ou similaire** — le code 8-1a a délibérément réutilisé `BankImportUnknownVersion` plutôt que d'introduire `BankImportFormatNotSupported` (cf. Pass 2 finding F4 reject).

- **F7 — `CamtError::MissingRequiredField` est `String`, pas `&'static str`** : le type a évolué de `&'static str` (spec d'origine 8-1 §T2.2) à `String` (patch Pass 1 F12 8-1a) pour permettre les **dot-paths indexés** dans le message, par ex. `"stmt[2].ntry[5].amount"` ou `"stmt[0].bal[OPBD].cdt_dbt_ind"`. Côté handler 8-1b, **mapper le `String` directement vers `details.field` ou `details.message`** plutôt que `&'static str`. Aucun parsing du dot-path n'est nécessaire — l'utilisateur final voit le chemin tel quel pour debugger un fichier malformé. Mapping suggéré :
  ```rust
  CamtError::MissingRequiredField(path) => AppError::BankImportParseFailed {
      code: "BANK_IMPORT_MISSING_FIELD",
      message: format!("Champ requis manquant : {path}"),
      details: serde_json::json!({ "field_path": path }),
  }
  ```

- **5 variantes `CamtError`** : `MalformedXml(String)`, `UnsupportedVersion(String)`, `MissingRequiredField(String)` *(F7)*, `InvalidAmount(String)`, `InvalidDate(String)` — tous `String`. Le mapping AppError côté T6.4 doit les couvrir toutes (probablement groupés sous `BankImportParseFailed` avec `code` discriminant `BANK_IMPORT_MALFORMED_XML` / `BANK_IMPORT_UNSUPPORTED_VERSION` / `BANK_IMPORT_MISSING_FIELD` / `BANK_IMPORT_INVALID_AMOUNT` / `BANK_IMPORT_INVALID_DATE`).

- **F13 8-1a — Ntry sign optionnel quand TxDtls porte le sien** : `parse_camt053` accepte un fichier où `<Ntry>` n'a pas de `<CdtDbtInd>` si chaque `<TxDtls>` en porte un. Le handler n'a rien de spécial à faire — la transaction est déjà signée correctement à la sortie du parseur. Mais **NE PAS rejeter les transactions où `signed_amount` ne match pas `Ntry CdtDbtInd`** (qui n'existe pas dans ce cas).

### Patterns architecturaux à respecter

(Hérités de la spec d'origine § Patterns architecturaux, lignes 632-643)

- **Multi-tenant scoping (Story 6-2 / 7-1, KF-002)** : tout repository appelle `WHERE company_id = ?` sur la première condition. Helper `bank_accounts::find_by_id_for_company` à créer (T6.3). Routes API utilisent `current_user.company_id` du JWT, pas une input client. Réponses cross-tenant = `404`, jamais `403`.
- **Optimistic locking** : `bank_transactions.version` posé pour Story 8-4. Pas mobilisé en 8-1b.
- **Audit log (Story 1-8 / 3-5)** : helper `audit_log::insert_in_tx(tx, NewAuditLogEntry { user_id, action, entity_type, entity_id, details_json })`. Signature exacte = `entity_type` / `entity_id`, **pas** `target_table` / `target_id`.
- **Erreurs structurées** : `AppError::Custom { status, code, message, details }` avec sérialisation JSON `{ error: { code, message, details } }` cohérente Story 1-11.
- **i18n key ownership (Story 6-3 / KF-006)** : préfixe `bank-import-` (kebab-case, matchant le nom du dossier `frontend/src/lib/features/bank-import/`) réservé exclusivement à ce module.
- **`rust_decimal` arithmétique** : `Decimal` de bout en bout. **Jamais** `f64`.
- **Repository pattern + sqlx** : `pool: &MySqlPool` ou `&mut Transaction<'_, MySql>`. SQL inline. `sqlx::QueryBuilder::push_values` pour bulk INSERT.
- **No-op short-circuit (Story 7-3 / KF-004)** : non applicable 8-1b (pas d'update sur `bank_imports`).
- **Test locally first (CLAUDE.md)** : avant chaque push, lancer la séquence backend + frontend + (E2E si modif frontend ou routes API consommées par les pages).

### Source tree à toucher

**DB / Backend** (héritage spec d'origine §645-700) :
- `crates/kesh-db/migrations/2026MMDD000001_bank_imports.sql` *(nouveau)*
- `crates/kesh-db/src/entities/bank_import.rs` *(nouveau)*
- `crates/kesh-db/src/entities/bank_transaction.rs` *(nouveau)*
- `crates/kesh-db/src/entities/mod.rs` (re-exports)
- `crates/kesh-db/src/repositories/bank_imports.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/bank_transactions.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/bank_accounts.rs` (extension : `find_by_id_for_company`)
- `crates/kesh-db/src/repositories/mod.rs` (re-exports)
- `crates/kesh-db/tests/bank_imports_test.rs` *(nouveau ou inline `#[sqlx::test]`)*
- `crates/kesh-api/Cargo.toml` (feature multipart + sha2)
- `crates/kesh-api/src/routes/bank_imports.rs` *(nouveau)*
- `crates/kesh-api/src/routes/mod.rs` (extension)
- `crates/kesh-api/src/lib.rs` (mountant + sub-router avec `DefaultBodyLimit`)
- `crates/kesh-api/src/errors.rs` (7 variantes)
- `crates/kesh-api/src/config.rs` (env var `KESH_BANK_IMPORT_MAX_MB`)
- `crates/kesh-api/tests/bank_imports_e2e.rs` *(nouveau)*
- `.env.example` (ajout `KESH_BANK_IMPORT_MAX_MB=10`)

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`

**Frontend** :
- `frontend/src/lib/shared/utils/api-client.ts` (extension FormData)
- `frontend/src/lib/features/bank-import/{bank-import.types.ts, bank-import.api.ts, BankImportUpload.svelte, BankImportPreviewTable.svelte, BankImportList.svelte, BankImportDetail.svelte, BankAccountSelector.svelte, BankImportUpload.test.ts}`
- `frontend/src/routes/(app)/bank-import/{+page.svelte, +page.ts, [id]/+page.svelte, [id]/+page.ts}`
- `frontend/tests/e2e/bank-import.spec.ts` *(nouveau)*
- `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` *(nouveau)*

### Standards de test

- **Intégration sqlx** : `#[sqlx::test]` avec migration auto + fixtures inline.
- **E2E HTTP** : `crates/kesh-api/tests/bank_imports_e2e.rs` avec helper `spawn_app(pool)` existant — F11 validate Pass 1 (`setup_test_app()` n'existe pas dans le codebase ; le helper canonique est `spawn_app(pool: MySqlPool) -> TestApp` défini dans `crates/kesh-api/tests/auth_e2e.rs:92` et utilisé dans tous les fichiers `*_e2e.rs`).

  **Chargement des fixtures CAMT.053 (M2 validate Pass 2)** : ne pas dupliquer les fixtures dans `crates/kesh-api/tests/fixtures/`. Lire directement depuis le crate sister `kesh-import` via `env!("CARGO_MANIFEST_DIR")` (analogue au test `validate_balance_detects_balance_mismatch_fixture` de 8-1a Pass 2 F16) :

  ```rust
  let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("../kesh-import/tests/fixtures/camt053/v04_multi_stmt.xml");
  let xml = std::fs::read(&fixture_path).expect("fixture introuvable");
  ```

  Fixtures à utiliser depuis `crates/kesh-import/tests/fixtures/camt053/` : `v04_minimal.xml`, `v04_multi_stmt.xml` (test #2 ignoredStatements), `v04_balance_mismatch.xml` (tests #6+#7), `v04_eur_currency.xml` (test #8), `v04_truncated.xml` (mapping erreur parse).
- **Vitest frontend** : `npm run test:unit -- bank-import`.
- **Playwright** : `npm run test:e2e -- bank-import.spec.ts` ; pré-requis MariaDB + seed CI + browsers installés (cf. CLAUDE.md « Test Locally First → E2E »).

### Checklist locale avant push

```sh
# Backend
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB requis pour les tests d'intégration

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # AC #19
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- bank-import.spec.ts
```

### Références

- Spec d'origine (contexte étendu) : [`8-1-import-camt053.md`](8-1-import-camt053.md) (status `archived-split`)
- Story prérequise : [`8-1a-camt053-parser-only.md`](8-1a-camt053-parser-only.md)
- Spike outcome : [`spike-kesh-import.md`](spike-kesh-import.md)
- Pattern multi-tenant scoping : [`docs/MULTI-TENANT-SCOPING-PATTERNS.md`](../../docs/MULTI-TENANT-SCOPING-PATTERNS.md)
- Pattern i18n key ownership : [`docs/i18n-key-ownership-pattern.md`](../../docs/i18n-key-ownership-pattern.md)
- Pattern bulk INSERT : `crates/kesh-db/src/repositories/journal_entries.rs` (Story 3-2)
- Pattern multipart Axum 0.8 : [docs.rs/axum/0.8/axum/extract/struct.Multipart.html](https://docs.rs/axum/0.8/axum/extract/struct.Multipart.html)
- CR-010 #62 (statement balance check) : [github.com/guycorbaz/kesh/issues/62](https://github.com/guycorbaz/kesh/issues/62)

## Dev Agent Record

### Agent Model Used

(à remplir par le dev agent — Opus 4.7 [1M context] recommandé compte tenu du scope cross-module)

### Debug Log References

### Completion Notes List

### File List

(F15 validate Pass 1 — pré-rempli depuis §Source tree pour éviter d'oublier un fichier en fin de story. Le dev agent confirme/ajuste après implémentation.)

**DB / Backend** :
- `crates/kesh-db/migrations/2026MMDD000001_bank_imports.sql` *(nouveau)*
- `crates/kesh-db/src/entities/bank_import.rs` *(nouveau)*
- `crates/kesh-db/src/entities/bank_transaction.rs` *(nouveau)*
- `crates/kesh-db/src/entities/mod.rs`
- `crates/kesh-db/src/repositories/bank_imports.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/bank_transactions.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/bank_accounts.rs`
- `crates/kesh-db/src/repositories/mod.rs`
- `crates/kesh-db/tests/bank_imports_test.rs` *(nouveau)* OU tests inline `#[sqlx::test]`
- `crates/kesh-api/Cargo.toml`
- `crates/kesh-api/src/routes/bank_imports.rs` *(nouveau)*
- `crates/kesh-api/src/routes/mod.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/config.rs`
- `crates/kesh-api/tests/bank_imports_e2e.rs` *(nouveau)*
- `.env.example`

**i18n** :
- `crates/kesh-i18n/locales/fr-CH/messages.ftl`
- `crates/kesh-i18n/locales/de-CH/messages.ftl`
- `crates/kesh-i18n/locales/it-CH/messages.ftl`
- `crates/kesh-i18n/locales/en-CH/messages.ftl`

**Frontend** :
- `frontend/src/lib/shared/utils/api-client.ts`
- `frontend/src/lib/features/bank-import/bank-import.types.ts` *(nouveau)*
- `frontend/src/lib/features/bank-import/bank-import.api.ts` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportPreviewTable.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportList.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportDetail.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankAccountSelector.svelte` *(nouveau)*
- `frontend/src/lib/features/bank-import/BankImportUpload.test.ts` *(nouveau)*
- `frontend/src/routes/(app)/bank-import/+page.svelte`
- `frontend/src/routes/(app)/bank-import/+page.ts`
- `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` *(nouveau)*
- `frontend/src/routes/(app)/bank-import/[id]/+page.ts` *(nouveau)*
- `frontend/tests/e2e/bank-import.spec.ts` *(nouveau)*
- `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` *(nouveau)*
- `frontend/tests/e2e/fixtures/camt053_v04_balance_mismatch.xml` *(nouveau, pour scénario E2E #3)*

**Story file & sprint** :
- `_bmad-output/implementation-artifacts/8-1b-camt053-persistence-ui.md` (Dev Agent Record)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `README.md` (T10.3 retrait *(à venir)*)

### Change Log

| Date | Action | Auteur |
|------|--------|--------|
| 2026-05-04 | Validate Pass 2 Haiku 4.5 (cycle Sonnet → Haiku, fenêtre fraîche). 12/12 patches Pass 1 confirmés APPLIED sans régression. 6 nouveaux findings Haiku → triage : **0 HIGH** + **1 MEDIUM** (M3 ambiguïté MiB/MB §upload-limit + env var KESH_BANK_IMPORT_MAX_MB) + **3 LOW** (M1 sqlx pattern T5.1 inline code, M2 fixture path convention §Standards de test, M4 localisation figée tests #8-9 IDOR dans bank_accounts) + **2 reject** (L1 dual coverage cosmétique, L2 audit log details_json déjà documenté ailleurs ; H3 « 7 vs 5 NEW variantes » faux positif vérifié). 4 patches appliqués. Trend : 10 → 1 > LOW (90% reduction). Verdict Haiku : GO-WITH-PATCHES. Relance Pass 3 Opus par règle CLAUDE.md (cycle Sonnet → Haiku → Opus, fenêtre fraîche, dernière passe avant convergence). | Claude (Opus 4.7, validate Pass 2 application) |
| 2026-05-04 | Validate Pass 1 Sonnet 4.6 (cycle Opus → Sonnet par règle CLAUDE.md, fenêtre fraîche). 14 findings bruts → triage : **4 HIGH** (F1 AC #3 ignoredStatements test, F2 T5.6/AC#11 9 tests inline + count, F3 AC #18 audit log dédié, F4 BANK_IMPORT_NO_MATCHING_STATEMENT AC #3c + test) + **6 MEDIUM** (F5 `parse_camt053` canonique, F6 SourceFormat::Csv mapping, F7 CamtError::MissingRequiredField(String) drift, F8 DDL uppercase note, F10 sqlx try_from pattern, F11 spawn_app vs setup_test_app) + **2 LOW** (F13 6 Playwright scenarios inline, F15 File List pré-rempli) + **1 reject** (F12 lib.rs déjà présent ligne 206). 12 patches appliqués. Verdict Sonnet : GO-WITH-PATCHES. Trend findings > LOW : 10 → relance Pass 2 Haiku par règle CLAUDE.md. Status : `backlog` → `ready-for-dev`. | Claude (Opus 4.7, validate Pass 1 application) |
| 2026-05-04 | Création de la story par split de 8-1 (`8-1-import-camt053.md`) en 8-1a (parser-only) + 8-1b (persistance + UI). Justification : règle CLAUDE.md « splitter si > 5 modules » (8-1 unifiée touchait 6 modules). Précédent rétro Epic 7 : Story 7-1 a explosé à 7 passes review faute de splitting préventif. Décision Guy 2026-05-04. La spec d'origine 8-1 reste comme référence des décisions de conception détaillées. 8-1b dépend de 8-1a (path dep) — status `backlog` jusqu'à 8-1a `done`. | Claude (Opus 4.7, dev-story split coordinator) |
