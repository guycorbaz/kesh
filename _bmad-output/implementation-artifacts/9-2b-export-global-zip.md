# Story 9.2b: Export global ZIP (souveraineté des données)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a utilisateur du logiciel comptable Kesh,
I want exporter l'ensemble de mes données comptables (comptes, écritures, contacts, factures, transactions bancaires, configuration) en un seul fichier ZIP contenant un CSV par table + un manifeste `metadata.json`,
so that je garde la souveraineté de mes données comptables et puisse migrer ailleurs (ou archiver pour conformité Swiss CO Art. 958f conservation 10 ans) si nécessaire.

## Scope

Crée un nouveau **endpoint HTTP unique** `GET /api/v1/exports/global.zip` qui assemble un **ZIP en mémoire** (`Vec<u8>`) contenant :
- **16 fichiers CSV** — un par table métier scopée `company_id` (liste exhaustive figée §scope-tables).
- **1 fichier `metadata.json`** — manifeste avec version Kesh, date export ISO 8601 UTC, locale, fiscal_year_scope, et hash SHA-256 de chaque CSV pour vérification d'intégrité.

Crée le **nouveau module métier `kesh-api/src/exports/`** (NON-`routes/`, helpers métier — Decision §csv-table-serializer-location) avec 16 serializers CSV table-raw + 1 builder ZIP + 1 builder metadata.json. Pas de nouvelle crate `kesh-export` (overkill v0.1).

Crée le **nouveau variant `AppError::GlobalExportFailed(String)`** (HTTP 500, i18n key `error-global-export-failed`) — distinct de `AppError::CsvGenerationFailed` (Story 9-2a) car le packaging ZIP peut échouer pour d'autres raisons que la sérialisation CSV per-table.

Ajoute **1 nouvelle entrée menu principal** `Export global` pointant sur `/export` (top-level route — PAS dans `/settings`, AC #1 souveraineté UX) avec **1 nouveau module Svelte `frontend/src/lib/features/exports/`** + **nouvelle page `frontend/src/routes/(app)/export/+page.svelte`**.

Audit log nouveau action `exports.global` (séparé de `report.exported` Story 9-2a) — `details_json` inclut `byte_size`, `csv_count=16`, `fiscal_year_scope='all'`, `duration_ms`.

**12 clés i18n nouvelles** × 4 locales (fr/de/it/en-CH) : `export-global-*` (titre page, description, bouton, exporting, erreur générique) + `error-global-export-failed`.

**Hors scope v0.1** :
- Restauration / import ZIP (le retour : déserialiser un ZIP Kesh dans une autre instance) — Epic 15 ou Epic 17 si demande.
- Filtrage par exercice (`?fiscalYearId=...`) — v0.1 = export `all` (toutes les écritures de la company, tous exercices). v0.2 si demande utilisateur.
- Filtrage par table (`?tables=accounts,invoices`) — v0.2.
- Streaming HTTP `Transfer-Encoding: chunked` du ZIP (cohérent §performance + L4).
- Chiffrement du ZIP (mot de passe AES-256) — hors scope sécurité v0.1.
- Tables EXCLUES (cf. §scope-tables-exclusion) : `users`, `refresh_tokens`, `audit_log`, `onboarding_state`, `invoice_number_sequences` — détaillé dans AC #8.

## Acceptance Criteria

### Menu principal + UX souveraineté

1. **Given** un utilisateur authentifié quelle que soit son rôle (Admin / Comptable / Consultation), **When** il regarde le menu principal de l'app, **Then** une nouvelle entrée `Export global` (i18n key `nav-export-global`) est visible dans la sidebar (PAS dans `/settings`, dans un groupe top-level cohérent UX souveraineté — `navGroups` `+layout.svelte`).

2. **Given** un utilisateur clique sur `Export global`, **When** la page `/export` s'affiche, **Then** elle contient : titre i18n `export-global-title`, description i18n `export-global-description` (mention « exporte toutes vos données comptables au format CSV pour migration ou archivage »), bouton `Lancer l'export` (i18n `export-global-button`), et message d'erreur dans une zone d'alerte au cas où.

### Endpoint HTTP + ZIP

3. **Given** un utilisateur authentifié, **When** il appelle `GET /api/v1/exports/global.zip`, **Then** le serveur retourne `Content-Type: application/zip` + `Content-Disposition: attachment; filename="kesh-export-{companyShort}-{YYYY-MM-DD}.zip"` (RFC 5987 + ASCII fallback identique pattern Story 9-2a `build_content_disposition`).

4. **Given** un ZIP téléchargé, **When** ses premiers bytes sont inspectés, **Then** ils commencent par la signature ZIP `PK\x03\x04` (0x50, 0x4B, 0x03, 0x04 — local file header magic number).

5. **Given** un ZIP téléchargé, **When** il est ouvert avec n'importe quel utilitaire (`unzip`, 7-Zip, Windows Explorer, macOS Archive Utility), **Then** il contient **exactement 17 entrées** : 16 fichiers CSV listés (§scope-tables) + 1 fichier `metadata.json`.

### Contenu CSV — 16 tables

6. **Given** chaque CSV dans le ZIP, **When** ouvert dans Excel CH/DE ou LibreOffice, **Then** il respecte le **même format** que Story 9-2a §csv-format : UTF-8 BOM en tête (`\xEF\xBB\xBF`) + séparateur `;` + terminator CRLF (`\r\n`) + escaping RFC 4180 automatique (chaînes contenant `;`/`"`/`\n` entourées de `"..."`, `"` interne doublé en `""`). Helper `csv::WriterBuilder` réutilisé via le pattern Story 9-2a (DRY — cf. §csv-table-serializer-location).

7. **Given** chaque CSV, **When** une colonne contient une date, **Then** elle est sérialisée au format **ISO 8601** (`YYYY-MM-DD` pour `Date`, `YYYY-MM-DDTHH:MM:SSZ` pour `DateTime<Utc>`) — format machine-readable pour ré-import futur (cohérent Story 9-2a CSV format §csv-format).

### Multi-tenant + sécurité (CRITICAL)

8. **Given** un utilisateur authentifié de company A, **When** il appelle `GET /api/v1/exports/global.zip`, **Then** **toutes** les requêtes SQL utilisées pour assembler les 16 CSV scopent strictement `WHERE company_id = current_user.company_id` — **jamais** via param URL ni JWT custom claim. Test IDOR : créer 2 companies, exporter avec user A → décompresser ZIP → assert qu'aucune ligne ne référence la company B (vérification par `id` de chaque entité, qui ne doit jamais apparaître dans le ZIP de A).

9. **Given** le ZIP exporté, **When** son contenu est inspecté, **Then** les **tables suivantes sont EXCLUES** (jamais présentes dans le ZIP, même vides) pour des raisons de sécurité / technicité :
   - `users.csv` — **EXCLU** : contient PII (email) + hashes Argon2 (risque sécurité critique si le ZIP fuite).
   - `refresh_tokens.csv` — **EXCLU** : secrets de session (hashes), expirent rapidement, aucune valeur d'archive utilisateur.
   - `audit_log.csv` — **EXCLU** : volumineux (1 ligne par opération) + log technique interne, pas une donnée utilisateur métier. Si demande compliance Swiss CO 958f → story dédiée Epic 14.
   - `onboarding_state.csv` — **EXCLU** : technique singleton (snapshot de progression onboarding), pas une donnée comptable.
   - `invoice_number_sequences.csv` — **EXCLU** : technique counter state (compteurs auto-incrémentés pour numérotation facture), pas une donnée métier portable. Au ré-import, le nouveau système re-calcule via `MAX(invoice_number) + 1`.

10. **Given** un utilisateur non authentifié (pas de Bearer token / token expiré), **When** il appelle `GET /api/v1/exports/global.zip`, **Then** **401** (middleware auth standard, route DOIT être dans `authenticated_routes` Router — cf. T3.1 anti-IDOR pattern Pass 1 BH-H1 Story 9-2a).

11. **Given** un utilisateur authentifié avec rôle `Consultation` (lecture seule), **When** il appelle `GET /api/v1/exports/global.zip`, **Then** **200** (lecture seule = export autorisé, cohérent Story 9-2a AC #25 RBAC Consultation 200).

### Manifeste `metadata.json`

12. **Given** le `metadata.json` extrait du ZIP, **When** parsé en JSON, **Then** il a la **shape suivante** (camelCase + champs stables) :
    ```json
    {
      "keshVersion": "0.1.0",
      "exportDate": "2026-05-15T16:23:45Z",
      "companyId": 42,
      "companyName": "CI Test Company",
      "locale": "fr-CH",
      "fiscalYearScope": "all",
      "tables": {
        "accounts.csv": { "rowCount": 5, "sha256": "abc123..." },
        "journal_entries.csv": { "rowCount": 123, "sha256": "def456..." },
        ...
      }
    }
    ```
    Avec exactement **16 entrées** dans `tables` (une par CSV), chaque valeur ayant un `rowCount` (nombre de data rows hors header) et un `sha256` (digest hex 64-char SHA-256 des bytes du CSV tel qu'il apparaît dans le ZIP — BOM + header + data + CRLF).

13. **Given** `metadata.json.keshVersion`, **When** comparé à `env!("CARGO_PKG_VERSION")` au build-time de `kesh-api`, **Then** valeur identique (`"0.1.0"` actuellement) — Decision §version-source = lecture compile-time, pas runtime DB.

14. **Given** `metadata.json.exportDate`, **When** parsé, **Then** format strict ISO 8601 UTC avec suffixe `Z` (jamais offset `+01:00`). Cf. `chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)`.

15. **Given** `metadata.json.locale`, **When** lu, **Then** valeur = `company.accounting_language` (déjà BCP47 via mapping `Language::FR/DE/IT/EN → fr-CH/de-CH/it-CH/en-CH` — pattern Story 9-2a `load_pdf_context` §locale-source).

16. **Given** un CSV référencé dans `metadata.json.tables`, **When** son `sha256` est recalculé sur les bytes décompressés (BOM + header + data rows + CRLF terminator final), **Then** identique au `sha256` du manifeste — vérification intégrité.

### Validation + erreurs

17. **Given** export ZIP en cours, **When** une requête SQL échoue (DB down ponctuelle, network blip), **Then** le serveur retourne 500 `AppError::GlobalExportFailed(detail)` avec i18n key `error-global-export-failed` (message UX-DR38 : « Impossible de générer l'export global. Réessayez dans quelques instants. Si le problème persiste, contactez le support. »).

18. **Given** export ZIP en cours, **When** le packaging ZIP échoue (rare : OOM, IO interne), **Then** 500 `AppError::GlobalExportFailed("zip packaging: <detail>")` cohérent UX-DR38.

19. **Given** un utilisateur sans `company_id` (état pathologique post-onboarding non-complété), **When** il appelle `/api/v1/exports/global.zip`, **Then** **403** `AppError::Forbidden` (cohérent middleware existant — `current_user.company_id` est garanti par JWT claim, mais on documente le cas en limitation L1).

### Performance + limites

20. **Given** un dataset de référence (~1000 écritures + ~100 factures + ~500 transactions bancaires), **When** l'export ZIP complet est généré, **Then** durée totale **< 10 secondes** mesurée via test perf `criterion` ou test E2E avec `Instant::now()` (cohérent epic-9.md §Story 9-2b AC original). Cible isolation : rendering pur (queries SQL + serialization CSV + packaging ZIP), pas le download network round-trip.

21. **Given** un dataset large (5000+ écritures), **When** export, **Then** durée **< 30 secondes** (cible élargie hors AC critique). Au-delà = dette tracée L4 streaming v0.2.

22. **Given** le ZIP final, **When** sa taille est mesurée, **Then** **< 5 MB** pour le dataset de référence (cohérent compression `zip` par défaut Deflate level 6, CSV très compressible). Documenté L3 si dépassé.

### Audit + observabilité

23. **Given** un export ZIP réussi 200, **When** la réponse est retournée, **Then** une ligne `audit_log` est insérée avec :
    - `action = 'exports.global'` (distinct de `report.exported` Story 9-2a)
    - `entity_type = 'export'`
    - `entity_id = AUDIT_ENTITY_ID_NONE` (cohérent Story 9-1 — pas d'entité 1:1)
    - `details_json` incluant `byte_size` (octets du ZIP final), `csv_count = 16`, `fiscal_year_scope = "all"`, `duration_ms` (rendering pur)
    - Pattern best-effort : INSERT échec → `warn!` log + retour 200 (NE JAMAIS faire échouer le download — cohérent Story 9-1 ECH-15 et Story 9-2a `emit_report_export_audit`).

24. **Given** export en cours, **When** le monitoring observe les logs structurés, **Then** un span `tracing::info_span!("global_export")` est émis avec attributs `byte_size`, `csv_count`, `duration_ms` via le **pattern `tracing::field::Empty` + `span.record()`** identique Story 9-2a T5.8 (Pass 3 BH3-M1 — `info_span!` évalue les fields à la création).

### Frontend UI

25. **Given** la page `/export`, **When** l'utilisateur clique sur `Lancer l'export`, **Then** le bouton passe en `disabled` avec libellé `Génération de l'export…` (i18n `export-global-loading`) jusqu'à la fin du téléchargement. Flag dédié `exporting` (cohérent Story 9-2a Pass 1 ECH-H2 + AC #36 — PAS partagé avec un autre `loading`).

26. **Given** export en cours, **When** l'utilisateur clique à nouveau sur le bouton, **Then** le second clic est ignoré (guard `if (exporting) return` en première ligne du handler, cohérent Story 9-2a Pass 1 code-review M12).

27. **Given** une erreur backend (500, 401, network), **When** export échoue, **Then** message d'erreur affiché dans une zone d'alerte `errorMsg` avec format UX-DR38 :
    - `isApiError(e) && e.code` → `formatError(e)` (message structuré backend)
    - sinon → fallback `i18nMsg('export-global-error-generic', '...')` (cohérent Story 9-2a Pass 1 code-review M13 pattern).

28. **Given** un téléchargement déclenché, **When** le navigateur reçoit le blob, **Then** filename suggéré = `kesh-export-{companyShort}-{YYYY-MM-DD}.zip` (côté backend via `Content-Disposition`, frontend ne calcule pas — c'est le serveur qui sait `keshVersion`, `companyName` slug, `exportDate`).

### Tests

29. **Given** la story est implémentée, **When** suite de tests exécutée, **Then** **≥ 12 tests E2E HTTP** dans `crates/kesh-api/tests/exports_global_e2e.rs` couvrent :
    - (a) Success path : seed `with-company` + 3 écritures + 2 contacts + 1 facture → 200 + ZIP signature `PK\x03\x04` + `Content-Type: application/zip`
    - (b) Multi-tenant 2 companies : seed A + seed B → export user A → décompresse ZIP → assert `accounts.csv` ne contient AUCUN account_id de B (IDOR test)
    - (c) ZIP structure : décompresse → assert exactement 17 entrées (16 CSV listés + 1 metadata.json)
    - (d) `metadata.json` parsing : extrait du ZIP → désérialise serde → assert shape (keshVersion, exportDate ISO 8601 UTC `Z` suffix, companyId, companyName, locale, fiscalYearScope=`"all"`, `tables` map 16 entries)
    - (e) SHA-256 intégrité : pour chaque CSV dans le ZIP, recalcule sha256 sur bytes décompressés → assert == `metadata.json.tables[name].sha256`
    - (f) Empty company : seed `with-company-no-fy` (ou équivalent SANS écritures) → 200 + ZIP toujours 17 entrées + chaque CSV contient au moins le header row + `metadata.json.tables[name].rowCount = 0` pour les tables vides
    - (g) Large dataset perf : seed + insertion de ~1000 écritures via SQL bulk → assert durée `< 10s` via `Instant::now()` autour de l'appel HTTP
    - (h) Auth 401 : pas de Bearer token → 401 (test isolé GET 1 endpoint, middleware identique)
    - (i) RBAC Consultation 200 : seed user role Consultation + login → 200
    - (j) `Content-Disposition: attachment` + RFC 5987 fallback (réutilise helper Story 9-2a `build_content_disposition`)
    - (k) Filename pattern : assert `download.suggestedFilename()` ou header parsing match regex `/^kesh-export-.+-\d{4}-\d{2}-\d{2}\.zip$/`
    - (l) ZIP byte signature `PK\x03\x04` (4 premiers bytes) — test isolé du success path pour faciliter le diagnostic en cas de régression
    Total : a+b+c+d+e+f+g+h+i+j+k+l = **12 tests minimum**.

30. **Given** la story est implémentée, **When** tests unit `kesh-api::exports::*` exécutés, **Then** **≥ 8 tests** valident :
    - (a) `serialize_accounts_csv(rows: &[Account], writer) -> Result<()>` produit BOM + header + data rows attendus (1 test fixture 2 rows)
    - (b) `serialize_journal_entries_csv` idem (1 test fixture 1 row avec champs date + decimal)
    - (c) `serialize_journal_entry_lines_csv` idem (1 test fixture 2 rows)
    - (d) `build_zip(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, AppError>` produit un ZIP valide commençant par `PK\x03\x04` + contient les entrées attendues (1 test fixture 2 files)
    - (e) `build_metadata_json(company, locale, tables) -> Result<String>` produit le JSON shape attendu avec `tables.*.sha256` non vide (1 test)
    - (f) `sha256_hex(bytes) -> String` retourne 64-char hex (1 test connu : hash de `b""` = `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`)
    - (g) `build_global_export(state, company_id) -> Result<(Vec<u8>, GlobalExportMeta)>` orchestrateur — 1 test avec fixture company + 1 account + assert ZIP bytes valides + meta cohérente
    - (h) Empty table case : `serialize_accounts_csv(&[], writer)` produit BOM + header seul (0 data rows), cohérent Story 9-2a CSV empty pattern

31. **Given** la story est implémentée, **When** Vitest exécuté, **Then** **≥ 2 tests** sur `frontend/src/lib/features/exports/exports.api.test.ts` :
    - (a) `downloadGlobalExport()` appelle `fetch` avec URL `/api/v1/exports/global.zip` + déclenche download via Blob + lien `<a download>` éphémère (mock fetch + `URL.createObjectURL`)
    - (b) Erreur backend (mock 500) → exception formatée propagée à `+page.svelte`

32. **Given** la story est implémentée, **When** Playwright exécuté, **Then** **1 scénario** `frontend/tests/e2e/export-global.spec.ts` : login → `/export` → cliquer `Lancer l'export` → `await page.waitForEvent('download')` → `await download.saveAs('/tmp/kesh-test-9-2b.zip')` → `fs.readFile()` → assert premier 4 bytes `PK\x03\x04` + assert `download.suggestedFilename()` match regex `/^kesh-export-.+-\d{4}-\d{2}-\d{2}\.zip$/`. Pattern `saveAs` mandaté (cohérent Story 9-2a Pass 1 ECH-M5 — `download.path()` peut retourner `null`).

## Tasks / Subtasks

- [ ] **T1** Ajouter dépendances `zip` + `sha2` à `crates/kesh-api/Cargo.toml` (AC: #3, #4, #16)
  - [ ] T1.1 `zip = "2"` (version 2.x — sync API, 4M+ downloads/year, Deflate compression par défaut, pas de complexité async hors scope v0.1). Vérifier au moment du dev la dernière stable via `cargo search zip`. Decision §zip-library figée.
  - [ ] T1.2 `sha2 = "0.10"` (RustCrypto, mature, déjà workspace via auth Argon2 ? — vérifier `cargo tree`). Sinon ajouter en dep directe. SHA-256 hex via `Digest::digest(&bytes) + hex::encode(...)`.
  - [ ] T1.3 `hex = "0.4"` (encoder hex pour SHA-256 64-char string). Si déjà dans le workspace : skip.
  - [ ] T1.4 `cargo build -p kesh-api` clean après ajout.

- [ ] **T2** Créer module `crates/kesh-api/src/exports/` (Decision §csv-table-serializer-location) (AC: #6, #7, #9)
  - [ ] T2.1 `crates/kesh-api/src/exports/mod.rs` — déclarations `pub mod csv_tables; pub mod global; pub mod metadata;` + re-export `build_global_export`.
  - [ ] T2.2 `crates/kesh-api/src/exports/csv_tables.rs` — 16 fonctions publiques `pub fn serialize_<table>_csv<W: Write>(rows: &[<Entity>], writer: W) -> Result<(), AppError>` :
    - `serialize_company_csv` (1 row : la company elle-même, cohérent pattern table mais en réalité 1 entry)
    - `serialize_fiscal_years_csv`
    - `serialize_accounts_csv`
    - `serialize_journal_entries_csv`
    - `serialize_journal_entry_lines_csv`
    - `serialize_contacts_csv`
    - `serialize_products_csv`
    - `serialize_invoices_csv`
    - `serialize_invoice_lines_csv`
    - `serialize_bank_accounts_csv`
    - `serialize_bank_imports_csv`
    - `serialize_bank_transactions_csv`
    - `serialize_vat_rates_csv`
    - `serialize_company_invoice_settings_csv`
    - `serialize_reconciliation_rules_csv`
    - `serialize_bank_profiles_csv`
  - [ ] T2.3 Chaque serializer : BOM en tête (`writer.write_all(&[0xEF, 0xBB, 0xBF])?`), `csv::WriterBuilder::new().delimiter(b';').terminator(csv::Terminator::CRLF).from_writer(writer)`, header row déterministe + 1 record par entity row. Dates au format ISO 8601 (`NaiveDate::format("%Y-%m-%d")` ou `DateTime<Utc>::to_rfc3339_opts(Secs, true)`). Decimal toujours 2 décimales (`format!("{:.2}", ...)` — cohérent Story 9-2a `format_amount_iso`).
  - [ ] T2.4 Helper privé `fn write_csv_bom<W: Write>(writer: &mut W) -> Result<(), AppError>` (DRY, 16 callers) + helper `fn make_csv_writer<W: Write>(writer: W) -> csv::Writer<W>` (DRY, mêmes paramètres séparateur/terminator partout). **Decision §csv-helper-reuse** : on duplique le pattern Story 9-2a `kesh-report::csv::{make_writer, format_amount_iso}` côté `kesh-api/exports` plutôt que d'exposer ces helpers comme API publique de `kesh-report` (DD-12 : `kesh-report` = rapports comptables agrégés, pas raw tables — ne pas étendre son scope).
  - [ ] T2.5 Map d'erreurs CSV → `AppError::GlobalExportFailed(format!("csv {table}: {e}"))` (T6 ajoute le variant).
  - [ ] T2.6 Tests unit en bas du fichier : ≥ 4 tests par batch (cf. T9.1 = 8 tests min : 3 serializers représentatifs + empty table case + build_zip + build_metadata_json + sha256_hex + orchestrateur).

- [ ] **T3** Créer `crates/kesh-api/src/exports/global.rs` — orchestrateur + ZIP builder (AC: #3-#11, #20-#22)
  - [ ] T3.1 `pub async fn build_global_export(pool: &MySqlPool, company_id: i64) -> Result<(Vec<u8>, GlobalExportMeta), AppError>` :
    - Query 16 tables via `repositories::*::list_by_company` (ou helper équivalent — voir T3.2 si pagination doit être traversée).
    - Pour chaque table : appeler `serialize_<table>_csv(rows, &mut buf)` → `Vec<u8>` par CSV.
    - Calcul SHA-256 + row_count par CSV.
    - Construit `metadata.json` via `build_metadata_json(...)`.
    - Construit ZIP via `build_zip(files)`.
    - Retourne `(zip_bytes, GlobalExportMeta { byte_size, csv_count: 16, duration_ms })` pour audit/tracing.
  - [ ] T3.2 **(Decision §pagination-traversal)** Pour les 16 tables, **pas de pagination** v0.1 — toutes les rows fetched en une seule query `SELECT * FROM <table> WHERE company_id = ?`. Si une table devient extrême (10k+ rows) : c'est du domaine de la limitation L4 streaming v0.2. **Action** : pour chaque repository qui n'a qu'un `list_by_company_paginated`, ajouter une nouvelle fn `list_all_by_company(pool, company_id) -> Result<Vec<Entity>, DbError>` (cohérent avec la fn existante `delete_all_by_company` pattern `accounts.rs:497`). Sub-tâches concrètes : ajouter `list_all_by_company` dans `repositories::{contacts, products, invoices, bank_transactions, bank_imports}` (les 5 repos qui n'ont actuellement que `list_by_company_paginated`). Réutiliser `list_by_company` (sans paginated) pour les autres.
  - [ ] T3.3 `pub fn build_zip(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, AppError>` — construit ZIP en mémoire :
    ```rust
    use zip::{ZipWriter, write::FileOptions, CompressionMethod};
    use std::io::{Cursor, Write};

    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let options: FileOptions<()> = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);
        for (name, bytes) in files {
            zip.start_file(name, options)
                .map_err(|e| AppError::GlobalExportFailed(format!("zip start_file {name}: {e}")))?;
            zip.write_all(bytes)
                .map_err(|e| AppError::GlobalExportFailed(format!("zip write {name}: {e}")))?;
        }
        zip.finish()
            .map_err(|e| AppError::GlobalExportFailed(format!("zip finish: {e}")))?;
    }
    Ok(cursor.into_inner())
    ```
    **Note pour le dev** : vérifier l'API exacte `zip 2.x` au moment de l'implémentation — la signature `FileOptions::default()` et `unix_permissions` peuvent évoluer entre 2.0 et 2.x. Le test `T9.1(l)` (signature `PK\x03\x04`) capture toute régression API.
  - [ ] T3.4 `struct GlobalExportMeta { byte_size: usize, csv_count: usize, duration_ms: u64 }` retournée pour audit + tracing post-handler.
  - [ ] T3.5 **Logging tracing** : `tracing::info_span!("global_export", byte_size = tracing::field::Empty, csv_count = tracing::field::Empty, duration_ms = tracing::field::Empty)` créé par le handler T5, fields populated post-`build_global_export` via `span.record(...)` (cohérent Story 9-2a T5.8 pattern Pass 3 BH3-M1).

- [ ] **T4** Créer `crates/kesh-api/src/exports/metadata.rs` — manifeste JSON (AC: #12-#16)
  - [ ] T4.1 `pub struct GlobalExportMetadata { kesh_version: String, export_date: String, company_id: i64, company_name: String, locale: String, fiscal_year_scope: String, tables: BTreeMap<String, TableMeta> }` (camelCase via `#[serde(rename_all = "camelCase")]`). **`BTreeMap`** pour ordre déterministe (key alphabetique) — facilite tests de byte-stability et hash reproductibles.
  - [ ] T4.2 `pub struct TableMeta { row_count: usize, sha256: String }` (camelCase).
  - [ ] T4.3 `pub fn build_metadata_json(company: &Company, locale_bcp47: &str, tables: BTreeMap<String, TableMeta>) -> Result<Vec<u8>, AppError>` :
    - `kesh_version = env!("CARGO_PKG_VERSION")` (compile-time, cohérent Decision §version-source).
    - `export_date = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)` (suffixe `Z` strict).
    - `company_id = company.id`, `company_name = company.name.clone()`.
    - `locale = locale_bcp47` (déjà résolu côté handler via mapping `Language → fr-CH/de-CH/it-CH/en-CH`, cohérent Story 9-2a `load_pdf_context`).
    - `fiscal_year_scope = "all"` (string literal v0.1, prévu pour futur `"fy:2026"` v0.2).
    - Sérialisation : `serde_json::to_vec_pretty(&meta).map_err(|e| AppError::GlobalExportFailed(format!("metadata serialize: {e}")))?`.
  - [ ] T4.4 `pub fn sha256_hex(bytes: &[u8]) -> String` — helper public (utilisé par tests + `build_global_export`) : `hex::encode(sha2::Sha256::digest(bytes))`.

- [ ] **T5** Créer `crates/kesh-api/src/routes/exports.rs` — handler HTTP (AC: #3, #10, #11, #17, #19, #23, #24)
  - [ ] T5.1 `pub async fn export_global(State(state): State<AppState>, Extension(current_user): Extension<CurrentUser>) -> Result<Response, AppError>` :
    - Validation : `current_user.company_id` doit être `> 0` (sinon 403 Forbidden — AC #19 état pathologique).
    - Charge `company` via `repositories::companies::find_by_id(&state.pool, current_user.company_id).await?.ok_or(AppError::Forbidden)?`.
    - Résout `locale_bcp47` via mapping `company.accounting_language → "fr-CH"/"de-CH"/"it-CH"/"en-CH"` (réutiliser **directement** le helper `load_pdf_context::map_language_to_bcp47` Story 9-2a s'il est public, sinon dupliquer le `match` 4-branches — cohérent Pass 3 ECH3-C1 + Pass 1 code-review M3 arm `"FR" =>` explicit). **Decision §locale-helper** : créer un helper `pub(crate) fn map_language_to_bcp47(lang: Language) -> &'static str` dans `crates/kesh-api/src/util.rs` (nouveau ou existant) pour partager entre `routes/reports.rs` et `routes/exports.rs`. Refactor Story 9-2a `load_pdf_context` pour réutiliser le helper.
    - Tracing span `global_export` créé + `Instant::now()` start.
    - Appelle `exports::global::build_global_export(&state.pool, current_user.company_id).await?` → `(zip_bytes, meta)`.
    - `span.record("byte_size", meta.byte_size); span.record("csv_count", meta.csv_count); span.record("duration_ms", meta.duration_ms);`
    - Construit `Response::builder().header(CONTENT_TYPE, "application/zip").header(CONTENT_DISPOSITION, build_content_disposition(&filename, &locale_bcp47)?).body(Body::from(zip_bytes))`.
    - Émet audit `emit_global_export_audit(...)` (best-effort, T5.3).
    - Return 200.
  - [ ] T5.2 Filename helper : `fn build_global_filename(company_name: &str, export_date: NaiveDate) -> String` — pattern `kesh-export-{companyShort}-{YYYY-MM-DD}.zip`. Réutiliser **directement** `routes::reports::build_filename` Story 9-2a si possible (signature `(type_slug, company_name, period, ext)`) en passant `type_slug = "export"` + dérivant `period` factice avec start=end=export_date. **Decision §filename-helper-reuse** : préférer une nouvelle fn dédiée `build_global_filename` (signature plus simple, pas de `ReportPeriod` factice qui prête à confusion). Mais réutiliser le helper `slugify` private de `routes/reports.rs` — **action** : promouvoir `slugify` de `routes/reports.rs` → `crates/kesh-api/src/util.rs::slugify` `pub(crate)`. Refactor Story 9-2a en passant.
  - [ ] T5.3 `async fn emit_global_export_audit(pool: &MySqlPool, user_id: i64, company_id: i64, byte_size: usize, csv_count: usize, fiscal_year_scope: &str, duration_ms: u64)` — pattern best-effort identique Story 9-2a `emit_report_export_audit` :
    - `tx.begin()` + `audit_log::insert_in_tx(action="exports.global", entity_type="export", entity_id=AUDIT_ENTITY_ID_NONE, details_json=...)` + commit.
    - INSERT fail → `warn!` log + retour 200 (NE JAMAIS faire échouer le download).
  - [ ] T5.4 Réutiliser **directement** `routes::reports::build_content_disposition` Story 9-2a — **action** : promouvoir cette fn `pub(crate)` (actuellement privée dans `routes/reports.rs`) ou la déplacer dans `crates/kesh-api/src/util.rs::build_content_disposition`. Tests Story 9-2a `content_disposition_with_locale_tag_includes_language` doivent rester verts post-refactor.

- [ ] **T6** Étendre `crates/kesh-api/src/errors.rs` — nouveau variant (AC: #17, #18)
  - [ ] T6.1 Ajouter `GlobalExportFailed(String)` au `enum AppError` (Decision §error-variant).
  - [ ] T6.2 Ajouter bras dans `IntoResponse for AppError` (cohérent Story 9-2a `PdfGenerationFailed`/`CsvGenerationFailed` blocs ~ligne 742+756) :
    ```rust
    AppError::GlobalExportFailed(detail) => {
        build_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "GLOBAL_EXPORT_FAILED",
            "error-global-export-failed",
            "Impossible de générer l'export global. Réessayez dans quelques instants.",
            Some(serde_json::json!({ "detail": detail })),
        )
    }
    ```
  - [ ] T6.3 Pas de bras `From<ReportError>` — le variant est dédié `kesh-api`, jamais retourné depuis `kesh-report`.
  - [ ] T6.4 Test unit : `crates/kesh-api/src/errors.rs` (en bas du fichier `#[cfg(test)] mod tests`) — `AppError::GlobalExportFailed("test detail")` → `into_response()` → status 500 + body JSON code `GLOBAL_EXPORT_FAILED` (pattern cohérent `pdf_generation_failed_into_response_returns_500` Story 9-2a).

- [ ] **T7** Mount route dans `crates/kesh-api/src/lib.rs` (AC: #3, #10)
  - [ ] T7.1 **(CRITICAL — anti-IDOR Pass 1 BH-H1 Story 9-2a)** Insérer la route DANS `authenticated_routes` (AVANT le `;` de fermeture du let binding), en chaînant `.route("/api/v1/exports/global.zip", get(routes::exports::export_global))` sur la dernière route. Vérifier après merge : `grep -A1 "authenticated_routes = Router" crates/kesh-api/src/lib.rs | head -50` doit montrer la route export AVANT le `;`. Si test E2E auth 401 (T9 AC #29(h)) retourne 200, c'est ce bug.
  - [ ] T7.2 Ajouter `pub mod exports;` dans le module declarations de `lib.rs` (ou dans `src/exports/mod.rs` selon convention existante).
  - [ ] T7.3 Ajouter `pub mod exports;` dans `routes/mod.rs`.

- [ ] **T8** Créer frontend `frontend/src/lib/features/exports/` + page `/export` (AC: #1, #2, #25-#28)
  - [ ] T8.1 Créer `frontend/src/lib/features/exports/exports.api.ts` :
    - `export async function downloadGlobalExport(): Promise<void>` — appelle `apiClient.getBlob('/api/v1/exports/global.zip')` → blob → déclenche download via `<a download>` éphémère + `URL.createObjectURL` + cleanup `try { appendChild + click } finally { removeChild + revokeObjectURL }` (pattern Story 9-2a M11 `triggerDownload`).
    - Le filename est imposé par le backend via `Content-Disposition` — frontend ne calcule pas.
  - [ ] T8.2 Créer `frontend/src/routes/(app)/export/+page.svelte` :
    - Title : `i18nMsg('export-global-title', 'Export global')`.
    - Description : `i18nMsg('export-global-description', 'Exportez toutes vos données comptables au format CSV pour migration ou archivage.')` — mention RGPD / souveraineté.
    - Bouton `Lancer l'export` avec flag dédié `exporting` (`$state`, PAS partagé avec un autre `loading`).
    - Handler `async function startExport()` :
      ```ts
      if (exporting) return;  // guard re-entrancy première ligne (Story 9-2a M12)
      exporting = true;
      errorMsg = '';
      try {
        await downloadGlobalExport();
      } catch (e) {
        if (isApiError(e) && e.code) {
          errorMsg = formatError(e);
        } else {
          errorMsg = i18nMsg('export-global-error-generic', '...');
        }
      } finally {
        exporting = false;
      }
      ```
    - Zone alerte `errorMsg` (cohérent pattern `+page.svelte` Story 9-2a).
  - [ ] T8.3 Créer `frontend/src/routes/(app)/export/+page.ts` :
    - `export const ssr = false;` (cohérent reports page).
    - Pas de `load()` initial — pas de données à pré-charger (le bouton fait tout). Si la page nécessite plus tard `companyName` pour pré-afficher le filename : ajouter à ce moment-là.
  - [ ] T8.4 Modifier `frontend/src/routes/(app)/+layout.svelte` `navGroups` (~ligne 43) — ajouter une nouvelle entrée :
    ```ts
    {
      label: null,  // ou nouveau groupe 'Souveraineté' si pertinent UX
      items: [
        { i18nKey: 'nav-export-global', fallback: 'Export global', href: '/export' },
      ],
    },
    ```
    Placement : **AVANT** l'entrée `nav-settings` (cohérent UX souveraineté = pas caché dans paramètres, AC #1). Si Sally (UX designer) a une opinion sur le placement exact, déférer à la dev-story.
  - [ ] T8.5 Tests Vitest `frontend/src/lib/features/exports/exports.api.test.ts` : ≥ 2 tests (cf. AC #31 — `downloadGlobalExport` mock fetch success + reject 500).

- [ ] **T9** Tests E2E HTTP `crates/kesh-api/tests/exports_global_e2e.rs` (AC: #29)
  - [ ] T9.1 **12 tests minimum** (voir AC #29 décomposition a-l).
  - [ ] T9.2 **Pattern fixture identique** Story 9-2a `reports_export_e2e.rs` : `seed_accounting_company` + insertions SQL via `state.pool`. Stratégie spécifique :
    - **Tests positifs** : `with-company` + insertion de 3 écritures + 2 contacts + 1 facture + 1 produit + 1 bank_account via SQL.
    - **Test multi-tenant 404/IDOR (AC #29(b))** : créer 2 seeds (`with-company` × 2 ou `seed_accounting_company` 2 appels avec emails distincts), login user A, télécharger ZIP A, décompresser via `zip` crate côté test (dev-dep `zip = "2"` test-only), parser `accounts.csv` → assert aucun account_id de B.
    - **Test ZIP structure (AC #29(c))** : décompresse + assert `zip.len() == 17` + listing exact des 17 noms attendus (16 CSV + `metadata.json`).
    - **Test metadata.json (AC #29(d))** : extract entry `metadata.json` → `serde_json::from_slice::<GlobalExportMetadata>` → assert champs.
    - **Test SHA-256 (AC #29(e))** : pour chaque CSV entry : décompresse bytes → `sha2::Sha256::digest` → hex encode → assert == `meta.tables[name].sha256`.
    - **Test empty company (AC #29(f))** : `with-company-no-fy` ou seed minimal sans écritures → ZIP toujours 17 entrées + each CSV header-only (row_count=0).
    - **Test perf (AC #29(g))** : insertion bulk 1000 journal_entries via SQL `INSERT ... VALUES (...), (...), ...` + `Instant::now()` autour de l'appel HTTP → `assert!(elapsed < Duration::from_secs(10))`. Marquer `#[ignore]` si la CI sandbox est lente (à exécuter manuellement pré-PR via `cargo test --ignored`).
    - **Test auth 401 (AC #29(h))** : pas de Bearer token → 401 sur GET `/api/v1/exports/global.zip` (test isolé, middleware identique sur `authenticated_routes`).
    - **Test RBAC Consultation (AC #29(i))** : créer 1 user role Consultation + login → 200.
  - [ ] T9.3 Helpers `assert_zip_response(body: &Bytes) -> Vec<(String, Vec<u8>)>` (décompresse + retourne liste fichiers) — factoriser dans le fichier test, pas dans `kesh-db::test_fixtures` (scope test seulement).

- [ ] **T10** Tests unit `crates/kesh-api/src/exports/` (AC: #30)
  - [ ] T10.1 8 tests unit minimum (voir AC #30 décomposition a-h).
  - [ ] T10.2 Fixtures construites en code Rust (pas de DB) via factories : `make_account(id, number, name)`, `make_journal_entry(id, date)`, etc. — cohérent Story 9-2a Q6 (factories code pur).

- [ ] **T11** i18n 12 clés × 4 locales (AC: #1, #2, #25-#27)
  - [ ] T11.1 Clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` :
    - `nav-export-global = Export global`
    - `export-global-title = Export global de vos données`
    - `export-global-description = Exportez toutes vos données comptables (comptes, écritures, contacts, factures, transactions bancaires) au format CSV dans un fichier ZIP. Utilisez cet export pour archiver, migrer vers un autre logiciel, ou conserver vos données 10 ans (Swiss CO Art. 958f).`
    - `export-global-button = Lancer l'export`
    - `export-global-loading = Génération de l'export…`
    - `export-global-success = Export téléchargé.`
    - `export-global-error-generic = Impossible de générer l'export global. Vérifiez votre connexion et réessayez.`
    - `export-global-filename-hint = Le fichier sera téléchargé sous le nom kesh-export-{ $companyShort }-{ $date }.zip`
    - `error-global-export-failed = L'export global n'a pas pu être généré. Si le problème persiste, contactez le support.`
    - `export-global-content-includes = L'export contient : plan comptable, écritures, contacts, produits, factures, comptes bancaires, transactions, règles de réconciliation, et un manifeste metadata.json avec hash SHA-256 de chaque fichier pour vérification d'intégrité.`
    - `export-global-content-excludes = Ne contient pas : utilisateurs (PII + mots de passe), tokens de session, journal d'audit interne, état d'onboarding (raisons de sécurité et technicité).`
    - `export-global-souverainete-note = Vos données vous appartiennent. Kesh ne fait aucune copie de cet export sur ses serveurs.`
  - [ ] T11.2 Idem pour `de-CH`, `it-CH`, `en-CH` (traductions de base, validation native v0.2 — L5 héritée). EN ex : `Global Export of your data`, `Launch export`, etc. DE/IT équivalents.
  - [ ] T11.3 `npm run lint-i18n-ownership` PASS — toutes les clés `export-global-*` + `nav-export-global` + `error-global-export-failed` appartiennent à `lib/features/exports/` ou `routes/(app)/export/`. Vérifier le manifest `lib/i18n/key-ownership.json` (ou équivalent existant) — ajouter la nouvelle feature `exports`.
  - [ ] T11.4 Validation manuelle 12 clés présentes dans les 4 locales (cohérent Story 9-2a T8.4).

- [ ] **T12** Tests Vitest frontend `frontend/src/lib/features/exports/exports.api.test.ts` (AC: #31)
  - [ ] T12.1 ≥ 2 tests (cf. AC #31 décomposition).

- [ ] **T13** Playwright `frontend/tests/e2e/export-global.spec.ts` (AC: #32)
  - [ ] T13.1 1 scénario : login → `/export` → cliquer `Lancer l'export` → `waitForEvent('download')` → `saveAs('/tmp/kesh-test-9-2b.zip')` → `fs.readFile` → assert premier 4 bytes = `[0x50, 0x4B, 0x03, 0x04]` (`PK\x03\x04`) + assert `download.suggestedFilename()` match regex.
  - [ ] T13.2 Non exécuté automatiquement par dev-story (nécessite MariaDB + browsers Playwright installés, manuel pré-push — cohérent Story 9-2a T12).

- [ ] **T14** CI green + Test Locally First (règle CLAUDE.md)
  - [ ] T14.1 `cargo fmt --all -- --check` clean.
  - [ ] T14.2 `cargo build --workspace --all-targets` clean.
  - [ ] T14.3 `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - [ ] T14.4 `cargo test --workspace -j1 -- --test-threads=1` — 100% pass sur les nouveaux tests + 0 régression Story 9-1 (`reports_e2e.rs` 28 tests) + 0 régression Story 9-2a (`reports_export_e2e.rs` 20 tests). Régressions résiduelles pré-existantes `config::tests::*` 20/24 documentées dans Completion Notes (héritées 9-2a).
  - [ ] T14.5 `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` — clean (0 errors).
  - [ ] T14.6 Playwright `npx playwright test export-global.spec.ts` — green (manuel, MariaDB + browsers requis).

## Dev Notes

### Décisions de conception verrouillées

#### Decision §csv-table-serializer-location — Module `kesh-api/src/exports/`

**Choix v0.1** : nouveau module `kesh-api/src/exports/` (non-`routes/`, helpers métier) avec 16 serializers CSV table-raw.

Rejeté :
- **Option B : extension `kesh-report::csv::tables`** — viole DD-12 (kesh-report = reports comptables agrégés type BalanceSheet/IncomeStatement, pas raw tables comme `accounts.csv`). Mélange sémantique = futur entanglement difficile à splitter.
- **Option C : nouvelle crate `kesh-export`** — overkill v0.1 (~16 serializers + 1 ZIP builder = ~600 lignes Rust, pas la surface qui justifie une crate publishable indépendante).

Trade-off accepté : duplication du pattern `csv::WriterBuilder + BOM + format_amount_iso` entre `kesh-report::csv` (Story 9-2a) et `kesh-api::exports::csv_tables` (Story 9-2b). Mitigé via T2.4 helpers privés dans chaque module. Si > 2 modules dupliquent : story Epic 15 v0.2 factorisation vers crate `kesh-format` ou `kesh-export-common`.

#### Decision §zip-library — `zip 2.x`

**Choix v0.1** : `zip = "2"` (crate de référence Rust pour ZIP — sync API, ~4M downloads/mois, MIT/Apache, support Deflate par défaut).

Rejeté :
- **`async-zip`** — complexité async hors scope v0.1, gain perf marginal sur datasets ~1000 écritures où la sérialisation CSV domine.
- **`zip-rs` legacy** — déprécié au profit de `zip 2.x`.

Vérifier au moment du dev la dernière stable via `cargo search zip` ou docs.rs/zip — l'API `FileOptions` / `CompressionMethod::Deflated` est stable depuis 1.x mais signatures peuvent évoluer. Test `T9.1(l)` (signature ZIP `PK\x03\x04`) capture toute régression d'API.

#### Decision §error-variant — `AppError::GlobalExportFailed(String)`

**Choix v0.1** : nouveau variant dédié `AppError::GlobalExportFailed(String)` + i18n key `error-global-export-failed` (code HTTP 500).

Rejeté :
- **Réutiliser `AppError::CsvGenerationFailed`** (Story 9-2a) — sémantique différente : un échec de packaging ZIP n'est pas un échec CSV. Message client UX différent.
- **Réutiliser `AppError::Internal`** — masque la sémantique, message client générique non-actionable (UX-DR38 violation).

Trade-off accepté : 1 variant supplémentaire dans l'enum (déjà ~30 variants post-Story 9-2a). Cohérent avec le pattern Story 9-2a qui a ajouté `PdfGenerationFailed` + `CsvGenerationFailed` séparés.

#### Decision §audit-action — `exports.global`

**Choix v0.1** : action `exports.global` distincte de `report.exported` (Story 9-2a).

Justification : scope sémantique différent (toutes tables vs 1 rapport agrégé). Distinct dans les requêtes audit :
- `report.generated` (Story 9-1) : génération JSON view UI.
- `report.exported` (Story 9-2a) : téléchargement fichier 1 rapport.
- `exports.global` (Story 9-2b) : téléchargement export complet souveraineté.

`details_json` inclut `byte_size`, `csv_count=16`, `fiscal_year_scope='all'`, `duration_ms`. Pas de `report_type` (non applicable). Pas de `format` (toujours `zip`, redondant avec l'action).

#### Decision §filename — `kesh-export-{companyShort}-{YYYY-MM-DD}.zip`

**Choix v0.1** : pattern cohérent avec Story 9-2a `build_filename` mais sans `{type}` (export global, pas per-report) et sans `{periodStart}_{periodEnd}` (scope = `all`, juste la date d'export).

Exemple : `kesh-export-ci-test-company-2026-05-15.zip`.

Helper : nouvelle fn `build_global_filename` (T5.2) — pas de réutilisation directe de `build_filename` Story 9-2a (signature inadaptée avec `ReportPeriod` factice). En revanche, le helper `slugify` privé Story 9-2a est promu `pub(crate)` dans `crates/kesh-api/src/util.rs` (T5.2 sub-task) + réutilisé tel quel.

`Content-Disposition` : RFC 5987 + ASCII fallback identique pattern Story 9-2a `build_content_disposition` (helper promu `pub(crate)` aussi, T5.4).

#### Decision §performance — Buffer ZIP `Vec<u8>` + non-streaming v0.1

**Choix v0.1** : ZIP entièrement assemblé en `Vec<u8>` côté serveur avant retour `Body::from(zip_bytes)`. Pas de streaming HTTP `Transfer-Encoding: chunked`.

Cible AC #20 : `< 10s` sur dataset référence (~1000 écritures + ~100 factures + ~500 transactions). Sur dataset extrême (5000+ écritures) : durée élargie `< 30s` AC #21, dette tracée L4 streaming v0.2.

Trade-off accepté : RAM peak côté serveur = `total_bytes(16 CSV) + zip_bytes + transient compression buffer` — estimation `< 50 MB` pour dataset 1000 écritures. Acceptable pour cibles PME suisse v0.1 (médiane attendue ~1k écritures/an). Documenté L3 si dépassé.

Streaming ZIP chunked (`zip 2.x` supporte écriture incrémentale via `ZipWriter::start_file + write_all + finish` sans consommer 100% RAM upfront) → v0.2 si OOM observé.

#### Decision §menu-placement — Top-level route `/export`

**Choix v0.1** : nouvelle route top-level `/export` (PAS sous `/settings`, PAS sous `/reports`).

Justification : l'export global est sémantiquement un acte de **souveraineté** (récupérer ses données, migrer ailleurs), pas une « configuration » (settings) ni un « rapport métier » (reports). Le placer dans le menu principal envoie le message UX : « vos données vous appartiennent, voici comment les récupérer en 1 clic ».

Placement dans `navGroups` (`+layout.svelte` ~ligne 43) : nouveau groupe top-level ou groupe `null` existant **AVANT** l'entrée `nav-settings`. Si Sally (UX designer) recommande un groupe dédié `Souveraineté` ou similaire — déférer à la dev-story.

Rejeté :
- `/settings/export` — caché, viole AC #1 (« menu principal, pas dans les paramètres »).
- `/reports/global-export` — sémantique fausse (l'export global n'est pas un rapport comptable).

#### Decision §version-source — `env!("CARGO_PKG_VERSION")` compile-time

**Choix v0.1** : `kesh_version` du `metadata.json` lu via `env!("CARGO_PKG_VERSION")` au build de `kesh-api` (= `0.1.0` actuellement).

Rejeté :
- **Lecture runtime via query DB sur une table `app_metadata`** — overkill v0.1, pas de table de ce type, pas d'usage récurrent à la version au runtime.
- **Workspace `[workspace.package] version = "0.1.0"`** — pas défini actuellement dans `Cargo.toml` racine (cf. lecture ground-truth `Cargo.toml:14`). Si v0.2 ajoute `[workspace.package] version`, migrer vers `env!("CARGO_PKG_VERSION")` qui résout au workspace via inherit.

Trade-off : si l'API `kesh-api` est rebuild sans bump de version, le `metadata.json` continue de dire `0.1.0`. Acceptable v0.1 (releases manuelles), à reconsidérer Epic 10 Déploiement quand CI/CD automatise les bumps.

#### Decision §locale-source — `company.accounting_language` mapping

**Choix v0.1** : `metadata.json.locale` = `company.accounting_language` (enum `Language::FR/DE/IT/EN`) mappé en BCP47 (`fr-CH`/`de-CH`/`it-CH`/`en-CH`) côté handler.

Réutilise le mapping helper `map_language_to_bcp47` Story 9-2a (introduit par Pass 3 ECH3-C1 dans `load_pdf_context`). T5.1 promeut le helper `pub(crate)` dans `crates/kesh-api/src/util.rs` pour partage entre `routes/reports.rs` (Story 9-2a) et `routes/exports.rs` (Story 9-2b).

Cohérent avec Story 9-2a Pass 1 code-review M3 (arm `"FR" =>` explicite + warn fallback `fr-CH` sur valeur DB inconnue).

#### Decision §scope-tables — 16 tables incluses, 5 exclues

**Tables INCLUSES (16)** :
| # | Table | Source repository | Justification |
|---|---|---|---|
| 1 | `company.csv` | `repositories::companies::find_by_id` | Référentiel principal — 1 row |
| 2 | `fiscal_years.csv` | `repositories::fiscal_years::list_by_company` | Référentiel comptable |
| 3 | `accounts.csv` | `repositories::accounts::list_by_company` | Plan comptable user-customisé |
| 4 | `journal_entries.csv` | `repositories::journal_entries::list_all_by_company` (nouvelle fn T3.2) | Cœur comptable |
| 5 | `journal_entry_lines.csv` | `repositories::journal_entries::list_all_lines_by_company` (nouvelle fn T3.2) | Détail écritures |
| 6 | `contacts.csv` | `repositories::contacts::list_all_by_company` (nouvelle fn T3.2) | Carnet d'adresses |
| 7 | `products.csv` | `repositories::products::list_all_by_company` (nouvelle fn T3.2) | Catalogue |
| 8 | `invoices.csv` | `repositories::invoices::list_all_by_company` (nouvelle fn T3.2) | Facturation |
| 9 | `invoice_lines.csv` | via `list_invoices_with_lines` ou fn dédiée | Détail factures |
| 10 | `bank_accounts.csv` | `repositories::bank_accounts::list_by_company` | Comptes bancaires |
| 11 | `bank_imports.csv` | `repositories::bank_imports::list_all_by_company` (nouvelle fn T3.2) | Historique imports |
| 12 | `bank_transactions.csv` | `repositories::bank_transactions::list_all_by_company` (nouvelle fn T3.2) | Transactions importées |
| 13 | `vat_rates.csv` | `repositories::vat_rates::list_by_company` | Taux TVA configurés |
| 14 | `company_invoice_settings.csv` | `repositories::company_invoice_settings::find_by_company` | Config facturation (1 row) |
| 15 | `reconciliation_rules.csv` | `repositories::reconciliation_rules::list_by_company` (Story 8-5b) | Règles user-config |
| 16 | `bank_profiles.csv` | `repositories::bank_profiles::list_by_company` (Story 8-2) | Profils import |

**Tables EXCLUES (5)** — détaillé AC #9 :
- `users` (PII + Argon2 hashes — sécurité critique)
- `refresh_tokens` (secrets de session)
- `audit_log` (technique interne, volumineux)
- `onboarding_state` (technique singleton)
- `invoice_number_sequences` (technique counter state)

Note pour le dev : si certaines fn `list_all_by_company` n'existent pas encore (cf. T3.2 sub-task — 5 nouveaux repos à étendre), elles doivent être créées en respectant le pattern `delete_all_by_company` `accounts.rs:497` (signature `pool, company_id → Result<Vec<Entity>, DbError>`).

### Architecture compliance (architecture.md §17 + §11)

- `kesh-api` étendu avec nouveau module `src/exports/` — pas de nouvelle crate (Decision §csv-table-serializer-location). Cohérent §11 workspace Cargo.
- `kesh-db` étendu avec 5+ nouvelles fn `list_all_by_company` (Decision §scope-tables) — additif, ne casse aucun caller existant.
- `kesh-i18n` transverse (decision #13) — 12 nouvelles clés × 4 locales = 48 entries.
- Multi-tenant `company_id` (Story 7-1 / KF-002) — pattern `current_user.company_id` strict, jamais bypass. Vérifié par test IDOR T9 AC #29(b).
- Pas de nouvelle dépendance circulaire ou de crate cross-coupling.

### Library / framework requirements

| Item | Version | Source | Justification |
|---|---|---|---|
| `zip` | 2.x (latest stable, vérifier `cargo search zip` au dev) | Nouvelle dep `kesh-api` | Decision §zip-library, sync API mature |
| `sha2` | 0.10 | Nouvelle dep `kesh-api` (ou déjà workspace via Argon2) | RustCrypto standard pour SHA-256 |
| `hex` | 0.4 | Nouvelle dep `kesh-api` (ou déjà workspace) | Encoder hex pour SHA-256 64-char string |
| `csv` | 1.3 | Déjà dans `kesh-report` (Story 9-2a) | Réutilisé via re-import dans `kesh-api` Cargo.toml |
| `serde_json` | déjà workspace | Métadata serde | Aucun ajout |
| `chrono` | déjà workspace | `Utc::now().to_rfc3339_opts(Secs, true)` | Aucun ajout |

### File structure

**Nouveaux fichiers** :
- `crates/kesh-api/src/exports/mod.rs` (~20 lignes — déclarations + re-exports)
- `crates/kesh-api/src/exports/csv_tables.rs` (~600 lignes — 16 serializers + helpers + 5+ tests unit fixtures représentatives)
- `crates/kesh-api/src/exports/global.rs` (~200 lignes — `build_global_export` + `build_zip` + `GlobalExportMeta`)
- `crates/kesh-api/src/exports/metadata.rs` (~120 lignes — `GlobalExportMetadata` + `TableMeta` + `build_metadata_json` + `sha256_hex`)
- `crates/kesh-api/src/routes/exports.rs` (~150 lignes — `export_global` handler + `emit_global_export_audit` + `build_global_filename`)
- `crates/kesh-api/src/util.rs` (~80 lignes — ou extension d'un fichier existant si présent — `pub(crate) fn slugify`, `pub(crate) fn build_content_disposition`, `pub(crate) fn map_language_to_bcp47`) — **action** : refactor de Story 9-2a privés vers `pub(crate)` partagés.
- `crates/kesh-api/tests/exports_global_e2e.rs` (~500 lignes — 12 tests E2E HTTP + helper `assert_zip_response`)
- `frontend/src/lib/features/exports/exports.api.ts` (~50 lignes — `downloadGlobalExport`)
- `frontend/src/lib/features/exports/exports.api.test.ts` (~70 lignes Vitest, 2+ tests)
- `frontend/src/routes/(app)/export/+page.svelte` (~120 lignes — page UI avec bouton + handler + zone alerte)
- `frontend/src/routes/(app)/export/+page.ts` (~10 lignes — `export const ssr = false;`)
- `frontend/tests/e2e/export-global.spec.ts` (~50 lignes Playwright, 1 scénario)

**Fichiers UPDATE (existants, ne PAS recréer)** :

- `crates/kesh-api/Cargo.toml` — ajout `zip 2.x` + `sha2 0.10` + `hex 0.4` (sauf si déjà workspace).
- `crates/kesh-api/src/lib.rs` — ajout `pub mod exports;` (module métier) + 1 nouvelle route `.route("/api/v1/exports/global.zip", get(routes::exports::export_global))` DANS `authenticated_routes` AVANT le `;`. **Anti-IDOR T7.1 critique.**
- `crates/kesh-api/src/routes/mod.rs` — ajout `pub mod exports;`.
- `crates/kesh-api/src/errors.rs` — ajout variant `AppError::GlobalExportFailed(String)` + bras `IntoResponse` 500 + i18n key.
- `crates/kesh-api/src/routes/reports.rs` — refactor : `slugify` privé → `pub(crate)` dans `util.rs` (importé par les 2 routes) ; `build_content_disposition` privé → `pub(crate)` ; `map_language_to_bcp47` extrait de `load_pdf_context` → `pub(crate) util::map_language_to_bcp47`. **Préserver** : signatures publiques des 4 handlers export + `emit_report_export_audit` + tests unit existants 13/13 doivent rester verts.
- `crates/kesh-db/src/repositories/contacts.rs` — ajout `pub async fn list_all_by_company(pool, company_id) -> Result<Vec<Contact>, DbError>` (pas de pagination).
- `crates/kesh-db/src/repositories/products.rs` — idem `list_all_by_company`.
- `crates/kesh-db/src/repositories/invoices.rs` — idem `list_all_by_company` + helper `list_all_lines_by_company` ou méthode alternative pour récupérer toutes les `invoice_lines` d'une company en un coup.
- `crates/kesh-db/src/repositories/journal_entries.rs` — idem `list_all_by_company` + `list_all_lines_by_company`.
- `crates/kesh-db/src/repositories/bank_imports.rs` — idem `list_all_by_company`.
- `crates/kesh-db/src/repositories/bank_transactions.rs` — idem `list_all_by_company`.
- 4 fichiers `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — ajout 12 clés (`nav-export-global` + `export-global-*` × 10 + `error-global-export-failed`).
- `frontend/src/routes/(app)/+layout.svelte` — ajout entrée `nav-export-global` dans `navGroups` (~ligne 43) AVANT `nav-settings`. **Préserver** : `navTestid` helper + 3 groupes existants.
- Manifest i18n ownership (`lib/i18n/key-ownership.json` ou équivalent — vérifier au dev) — ajout feature `exports` avec préfixes `export-global-*` + `nav-export-global` + `error-global-export-failed`.

### Testing standards

- **Tests d'intégration Rust** : `crates/kesh-api/tests/exports_global_e2e.rs` — pattern `reqwest` + `spawn_app` identique `reports_export_e2e.rs` Story 9-2a. Helper `assert_zip_response(body) -> Vec<(String, Vec<u8>)>` factoriser dans le fichier (pas dans `kesh-db::test_fixtures` — scope test seulement). Décompression via `zip 2.x` dev-dep (déjà dans Cargo.toml runtime, donc dispo en test sans ajout).
- **Tests unit Rust** : `crates/kesh-api/src/exports/{csv_tables, global, metadata}.rs` — modules `#[cfg(test)] mod tests` en bas de chaque fichier. Fixtures `Account`/`Contact`/etc. construites en code (pas de DB), cohérent Story 9-2a Q6.
- **Tests frontend Vitest** : `frontend/src/lib/features/exports/exports.api.test.ts` — mock `fetch` via `vi.mock`.
- **Playwright** : `frontend/tests/e2e/export-global.spec.ts` — pattern `await page.waitForEvent('download')` + `download.suggestedFilename()` + `await download.saveAs('/tmp/...')` + `fs.readFile` (cohérent Story 9-2a Pass 1 ECH-M5 — **PAS** `download.path()` qui peut retourner `null`).
- **Benchmark criterion (optionnel)** : si la performance de `build_global_export` devient sensible, ajouter `crates/kesh-api/benches/global_export.rs` — hors scope v0.1 sauf si AC #29(g) test perf flake.

### Previous Story Intelligence (Story 9-2a)

**Patterns à réutiliser tels quels** :
- **`csv::WriterBuilder + BOM UTF-8 + ';' delimiter + CRLF terminator + format_amount_iso(...)`** — re-implémenté dans `kesh-api::exports::csv_tables` (Decision §csv-table-serializer-location).
- **`build_content_disposition(filename, locale_bcp47) -> HeaderValue` (RFC 5987 `filename*=UTF-8'<lang>'<percent>` + ASCII fallback)** — promu `pub(crate)` dans `crates/kesh-api/src/util.rs` (T5.4), partagé entre `routes::reports` (Story 9-2a) et `routes::exports` (Story 9-2b).
- **`slugify(name, max_len, fallback) -> String` (NFD strip diacritics + lowercase + `[^a-z0-9-]` → `-` + collapse + truncate + strip trailing `-`)** — promu `pub(crate)` dans `crates/kesh-api/src/util.rs` (T5.2).
- **`map_language_to_bcp47(lang: Language) -> &'static str`** — extrait de `load_pdf_context` Story 9-2a Pass 3 ECH3-C1, promu `pub(crate) util::map_language_to_bcp47` (T5.1).
- **`emit_<action>_audit` best-effort pattern** : `tx.begin/insert_in_tx/commit`, `warn!` on error, retour 200 quoi qu'il arrive — Story 9-2a `emit_report_export_audit` (ligne `reports.rs:866`) répliqué dans `routes/exports.rs::emit_global_export_audit` (T5.3).
- **`tracing::info_span!` + `tracing::field::Empty` placeholder + `span.record()` post-render** — pattern Pass 3 BH3-M1 Story 9-2a (T5.8) répliqué dans `routes/exports.rs` (T3.5 + T5.1).
- **Frontend `triggerDownload` try/finally cleanup objectURL** — pattern Story 9-2a Pass 1 code-review M11 répliqué dans `exports.api.ts::downloadGlobalExport`.
- **Frontend flag `exporting` dédié, guard re-entrancy first-line, `finally { exporting = false }`** — pattern Story 9-2a Pass 1 ECH-H2 + code-review M12.
- **Frontend `isApiError(e) && e.code` → `formatError(e)`, fallback `i18nMsg('...-error-generic')`** — pattern Story 9-2a code-review M13 répliqué dans `+page.svelte`.
- **Anti-IDOR pattern Pass 1 BH-H1** : route mountée DANS `authenticated_routes` AVANT le `;` (T7.1 critique).
- **Filename ASCII fallback + RFC 5987 `filename*=UTF-8'<lang>'<percent>` pour `Content-Disposition`** — Story 9-2a Pass 1 code-review M14 (avec tag langue BCP47).

**Helpers publics API stable consommée** :
- `kesh_db::repositories::companies::find_by_id` — Story 1-x stable.
- `kesh_db::repositories::audit_log::insert_in_tx` — Story 9-1 + Story 9-2a.
- `kesh_db::repositories::*` — list_by_company stable, list_all_by_company nouvelle (T3.2).
- `AppError::Forbidden`, `AppError::Database`, `AppError::Internal` — variants existants pré-9-2b.

**Régressions résiduelles pré-existantes documentées Story 9-2a** (à hériter sans rouvrir dans 9-2b Completion Notes) :
- `kesh-api::config::tests::*` 20/24 fail local `.env` + `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true` collision.
- KF-027 #91 a11y `#bits-c1` DropdownMenu pré-existant — pas de scope 9-2b sauf si T8.4 introduit un nouveau DropdownMenu (improbable, juste 1 entrée nav inline).
- KF-026 #76 multi-candidates UI v0.2 — hors scope.
- KF #70 frontend wiring bankProfileId — hors scope.

### Git intelligence (5 derniers commits sur main)

| Commit | Title | Pertinence 9-2b |
|---|---|---|
| `b891bff` | Story 9-2a: Export PDF & CSV par rapport (#93) | **Foundation directe** — `csv::WriterBuilder`, `build_content_disposition`, `slugify`, `emit_*_export_audit`, frontend `triggerDownload`, tracing span pattern, RFC 5987 tag langue — tous réutilisés par 9-2b |
| `ef07548` | chore(test-infra): preset `with-company-no-fy` (#92) | T9 AC #29(f) — réutilise preset `with-company-no-fy` pour test empty company |
| `6495731` | Story 9-1: Rapports comptables (#89) | Foundation `kesh-report` + audit pattern `emit_report_audit` |
| `b3331dc` | review(9-1): Pass 2 Haiku 4.5 — STOP cycle convergence 0>LOW | Pattern review codifié — ground-truth grep avant flag (à appliquer pour 9-2b validate passes) |
| `25b1b5a` | review(9-1): Pass 1 code review Sonnet 4.6 — 24 patches | Patterns audit best-effort, sections fixes — réutilisables |

### Limitations connues v0.1

| # | Limite | Raison | Tracking dette |
|---|---|---|---|
| L1 | Pas d'import / restauration du ZIP (le retour : déserialiser un ZIP Kesh dans une autre instance Kesh ou une instance fresh) | Hors scope souveraineté v0.1 — l'utilisateur peut récupérer ses données, l'import est un autre projet | Epic 15 ou Epic 17 si demande |
| L2 | Pas de filtrage par exercice (`?fiscalYearId=...`) — export = `all` (tous exercices + toutes données) | v0.1 simple, sémantique « souveraineté complète ». Si filtré, perd la propriété « migration totale » | v0.2 si demande utilisateur |
| L3 | ZIP buffered en `Vec<u8>` en RAM (pas de streaming Axum body `Transfer-Encoding: chunked`) | Simplicité v0.1, dataset référence ~1000 écritures = ~500 KB-2 MB ZIP final estimé | v0.2 si OOM observé ou dataset > 5000 écritures rapporté |
| L4 | Pas de pagination intra-table (toutes les rows fetched en une seule query `SELECT * FROM <table> WHERE company_id = ?`) | Simplicité v0.1, repositories n'ont pas tous `list_all_by_company` paginé (cohérent avec L3) | v0.2 streaming + pagination simultanées si gros tenant |
| L5 | Traductions DE/IT/EN basiques (machine ou auteur, pas natif speaker review) | Hérité Story 9-1 L4 et 9-2a L4 | v0.2 review native speakers |
| L6 | Pas d'horodatage signé / certificat sur le ZIP (Swiss CO Art. 958f conformité partielle) | Recherche réglementaire R2 epic-9.md non complétée — décision audit-trail-only acceptée pour v0.1 (action `exports.global` dans audit_log + SHA-256 dans `metadata.json` fournissent un mécanisme d'intégrité non-signé) | Si recherche R2 conclut signature requise : story Epic 14 ou 15 |
| L7 | `metadata.json.keshVersion` lu via `env!("CARGO_PKG_VERSION")` compile-time — si l'API est rebuild sans bump de version, le ZIP rapporte une version stale | Acceptable v0.1 (releases manuelles). Epic 10 Déploiement automatisera les bumps via CI/CD | v0.2 quand CI/CD auto-bump |
| L8 | Pas de chiffrement du ZIP (mot de passe AES-256 ou similaire) | Hors scope sécurité v0.1 — si l'utilisateur partage le ZIP, c'est sa responsabilité (TLS protège le transit). Le ZIP contient des données financières sensibles | v0.2 si feedback utilisateur — option `?encrypted=true&password=...` |
| L9 | Tables EXCLUES (`users`, `refresh_tokens`, `audit_log`, `onboarding_state`, `invoice_number_sequences`) — pas négociable v0.1, documenté AC #9 | Sécurité (PII/secrets) + technicité (state interne non portable) | v0.2 si compliance externe demande `audit_log` (story dédiée Epic 14) |
| L10 | Export entièrement synchrone (pas de queue/background job avec progression UI) | Acceptable < 10s dataset référence ; au-delà = boucle de chargement bloquante côté UI | v0.2 si dataset extrême avec UX progress bar / WebSocket |

### Risques & questions ouvertes — pour spec validate

Les éléments ci-dessous restent à clarifier ou à décider lors de `bmad-create-story validate 9-2b` ou en dev-story si la question survient en cours d'implémentation. Si un risque devient un blocker, créer un GitHub Issue (KF ou CR) et ne pas modifier silencieusement les ACs.

| # | Risque / question | À traiter |
|---|---|---|
| Q1 | **Exact 16 tables figées** : la liste §scope-tables est ground-truth based mais le dev pourrait découvrir une table omise (ex. `invoice_attachments` future ou table jointure secondaire). Si oui, l'ajouter à la liste + bumper la count = 17 + ajuster AC #5 + tests E2E (AC #29(c)). | Spec validate Pass 1 + dev-story T2.2 grep `entities/` |
| Q2 | **Repository `list_all_by_company` à ajouter** : 5+ nouvelles fn dans 5+ repos (T3.2). Pattern strict `pool, company_id → Result<Vec<Entity>, DbError>` (cohérent `delete_all_by_company` `accounts.rs:497`). Faut-il un trait commun `ListAllByCompany` ? **Recommandation** : non, duplication acceptable v0.1 (5 fn `pub async`, ~10 lignes chacune). | Dev-story T3.2 |
| Q3 | **Refactor Story 9-2a privés → `pub(crate) util`** (T5.1/T5.2/T5.4) : promouvoir `slugify`, `build_content_disposition`, `map_language_to_bcp47` doit-il faire l'objet d'un commit séparé avant T2 ? **Recommandation** : commit dédié `refactor(9-2b): promote 9-2a helpers to pub(crate) util` en première étape, puis T2+ s'appuie dessus. Cohérent règle CLAUDE.md « commit par étape BMAD ». | Dev-story T5 |
| Q4 | **`zip 2.x` exact API stability** : la version 2.0 a renommé `FileOptions` paramètres vs 1.x. Vérifier au dev exact via `cargo doc -p zip --open` ou docs.rs/zip/latest. Test `T9.1(l)` (signature `PK\x03\x04`) capture toute régression. | Dev-story T3.3 |
| Q5 | **`sha256_hex` perf sur gros CSV** : SHA-256 de 16 buffers `Vec<u8>` cumulés ~5 MB = ~50ms estimé (RustCrypto, 100-200 MB/s sur CPU moderne). Acceptable v0.1. Si > 200ms observé : passer en streaming `Digest::update` chunked. | Dev-story T4.4 |
| Q6 | **`metadata.json` ordering deterministe** : `BTreeMap` (clé alphabétique) ou `Vec<(String, TableMeta)>` (ordre d'insertion = ordre §scope-tables liste) ? Recommandation : `BTreeMap` (cohérent serde_json + facilite tests byte-stability). | Dev-story T4.1 |
| Q7 | **Si `accounting_language` est NULL ou non-mapped** (e.g. value DB stale `"XX"`) : fallback `fr-CH` + warn log (cohérent Story 9-2a code-review M3 pattern). Documenter en Completion Notes. | Dev-story T5.1 |
| Q8 | **Empty company ZIP shape** : si la company a 0 row dans toutes les tables sauf `company.csv` (1 row), le ZIP doit-il quand même contenir les 16 CSV (chacun avec header-only) ? **Recommandation** : OUI (AC #5 + AC #29(f) — 17 entrées toujours). Cohérent Story 9-2a CSV empty pattern Pass 1 ECH-M1. | Dev-story T2.3 (each serializer must write header even for empty input) |
| Q9 | **Test perf `< 10s` flake en CI sandbox** : si la sandbox CI est lente, marquer `#[ignore]` et exécuter manuellement pré-PR via `cargo test --ignored` (cohérent Story 9-2a AC #31 `pdf_10k_journal_report_size_under_5mb` ignored). | Dev-story T9.2 |
| Q10 | **Sally (UX designer) input sur menu placement** : nouveau groupe `Souveraineté` ou simplement nouvelle entrée dans groupe `null` final ? Cohérence UX globale Kesh à confirmer. **Recommandation** : ouvrir une discussion brève Sally en dev-story si T8.4 a un doute. Sinon : nouveau groupe top-level `Souveraineté` (ou équivalent) AVANT `Paramètres`. | Dev-story T8.4 ou skill `bmad-agent-ux-designer` |

### Project Structure Notes

Alignement avec `architecture.md` §11 (workspace) + §17 (FR68 → souveraineté des données) :
- ✅ Nouveau module `kesh-api/src/exports/` — additif, pas de nouvelle crate.
- ✅ Routes API dans `kesh-api/routes/exports.rs` — nouveau fichier (sémantique distincte de `routes/reports.rs`).
- ✅ Frontend nouvelle feature `lib/features/exports/` + nouvelle page `/export` — additif, pas de modification des features existantes.
- ✅ i18n dans `kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — section `export-global-*` nouvelle.
- ✅ Helpers Story 9-2a refactorés `pub(crate)` dans `util.rs` — DRY, cohérent règle DRY CLAUDE.md.

**Splitting check CLAUDE.md (Story 7-1 lesson + Epic 8 lessons)** :

Modules touchés par 9-2b :
1. `kesh-api/src/exports/` (nouveau module)
2. `kesh-api/src/routes/exports.rs` (nouveau)
3. `kesh-api/src/lib.rs` + `routes/mod.rs` (mount route + module declarations)
4. `kesh-api/src/errors.rs` (1 nouveau variant)
5. `kesh-api/src/util.rs` (refactor 9-2a privés → `pub(crate)`)
6. `kesh-db/src/repositories/*` (5+ fn `list_all_by_company` additives)
7. `frontend/src/lib/features/exports/` (nouveau)
8. `frontend/src/routes/(app)/export/+page.{ts,svelte}` (nouvelle page)
9. `frontend/src/routes/(app)/+layout.svelte` (1 entrée nav)
10. `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (12 clés × 4 = 48 entries)

= ~10 modules. **Borderline > 5 threshold CLAUDE.md splitting préventif.**

**Justification non-split** :
- **Pattern simple** : 1 endpoint, 1 ZIP packaging, 1 page UI. Pas de logique métier complexe nouvelle (vs Story 7-1 KF-002 audit + multi-tenant scoping refactor cross-cutting qui a justifié split en 7-1a/b/c).
- **Réutilisation massive Story 9-2a** : `csv::WriterBuilder`, `build_content_disposition`, `slugify`, `triggerDownload`, `emit_*_audit`, tracing span — tous patterns importés tel quel. La surface de **code nouveau** est ~1500 lignes (vs Story 9-2a ~3500-5000 lignes).
- **Backend serializers triviaux** : 16 fonctions CSV-table très similaires, juste mapping `Entity → CSV row`. Pas de logique conditionnelle complexe.
- **Frontend minimal** : 1 page avec 1 bouton, ~150 lignes Svelte.
- **i18n large mais mécanique** : 12 clés × 4 locales = 48 entries, traduction de strings simples.

**Décision** : **garder Story 9-2b unique**, ne pas splitter. Si `bmad-create-story validate 9-2b` boucle au-delà de **4 passes adversariales** (critère CLAUDE.md profondeur d'incertitude) : reconsidérer split en 9-2b-zero (refactor `pub(crate) util` Story 9-2a) + 9-2b-core (endpoint + serializers + page).

**Dérogation règle de splitting non requise.**

### References

- `_bmad-output/planning-artifacts/epic-9.md` §Story 9-2b (ACs originaux 7 bullets)
- `_bmad-output/planning-artifacts/prd.md` FR68 (export global per table CSV), UX-DR38 (messages d'erreur actionnables — héritée Story 9-2a)
- `_bmad-output/planning-artifacts/architecture.md` decision #12 (kesh-report crate, scope rapports — NE PAS étendre à raw tables), decision #13 (kesh-i18n transverse), §11 (workspace Cargo), §17 (cartographie FR68)
- `_bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md` — **foundation directe**, patterns réutilisés : `build_content_disposition` (Pass 1 code-review M14 RFC 5987 lang tag), `slugify` (Pass 1 BH-M1 regex inline), `emit_report_export_audit` best-effort, tracing span `tracing::field::Empty` (Pass 3 BH3-M1), frontend `triggerDownload` cleanup (M11), `exporting` flag dédié + guard re-entrancy (M12), `formatError` + fallback `*-error-generic` (M13), `map_language_to_bcp47` (Pass 3 ECH3-C1)
- `_bmad-output/implementation-artifacts/9-1-rapports-comptables-bilan-resultat-balance-journaux.md` — pattern audit best-effort Story 9-1 ECH-15
- `crates/kesh-api/src/routes/reports.rs` — patterns Story 9-2a à promouvoir `pub(crate)` (T5.1/T5.2/T5.4)
- `crates/kesh-api/src/errors.rs:184` + `:193` — variants `PdfGenerationFailed` + `CsvGenerationFailed` existants, pattern à reproduire pour `GlobalExportFailed`
- `crates/kesh-report/src/csv.rs:24-50` — pattern BOM + `csv::WriterBuilder` à dupliquer dans `kesh-api/exports/csv_tables.rs` (Decision §csv-table-serializer-location)
- `crates/kesh-db/src/repositories/accounts.rs:497` — pattern `delete_all_by_company(pool, company_id) -> Result<u64, DbError>` à reproduire pour les `list_all_by_company` (T3.2)
- `crates/kesh-db/src/entities/{company,fiscal_year,account,journal_entry,contact,product,invoice,bank_account,bank_import,bank_transaction,vat_rate,company_invoice_settings,reconciliation_rule,bank_profile}.rs` — entités sources des 16 CSV (Decision §scope-tables)
- `frontend/src/routes/(app)/+layout.svelte:43-68` — `navGroups` structure à étendre (T8.4)
- `frontend/src/lib/features/reports/reports.api.ts` — pattern `triggerDownload` Story 9-2a Pass 1 code-review M11
- `frontend/src/routes/(app)/reports/+page.svelte` — pattern handler `exporting` + try/finally + fallback i18n Story 9-2a
- Issue GitHub à surveiller : KF-027 #91 (a11y `#bits-c1` pré-existant, hors scope 9-2b sauf si `+page.svelte` introduit DropdownMenu — improbable)
- Memory `feedback_haiku_review_diff_combined` — pattern Haiku 4.5 reviewers hallucinent sur diff multi-commit, toujours grep ground-truth pre-flag (à appliquer aux passes spec validate 9-2b)

## Dev Agent Record

### Agent Model Used

(À compléter par dev-story.)

### Debug Log References

(À compléter par dev-story.)

### Completion Notes List

(À compléter par dev-story.)

### File List

(À compléter par dev-story.)

## Change Log

- **2026-05-15** (Opus 4.7 create-story) — Spec initiale créée sur branche `story/9-2b-export-global-zip`. Path-dep Story 9-2a `done` (`b891bff` mergée). 32 ACs (Menu+UX 2, Endpoint+ZIP 3, CSV 2, Multi-tenant 4, Metadata 5, Validation 3, Performance 3, Audit 2, Frontend 4, Tests 4). 14 tasks T1-T14 avec sub-tasks. 8 décisions verrouillées (§csv-table-serializer-location, §zip-library, §error-variant, §audit-action, §filename, §performance, §menu-placement, §version-source, §locale-source, §scope-tables). 10 limitations v0.1 (L1-L10). 10 questions ouvertes Q1-Q10 pour spec validate. Splitting check : ~10 modules touchés borderline > 5 threshold CLAUDE.md mais justifié non-split (pattern simple + réutilisation massive 9-2a). Status `backlog → ready-for-dev`. Cycle CLAUDE.md prochaine étape : `bmad-create-story validate 9-2b` Pass 1 Sonnet 4.6 (briser biais Opus auteur, ground-truth grep aggressif).
