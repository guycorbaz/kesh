# Story 8.2: Import CSV (multi-encodage & profils banque)

Status: ready-for-dev

<!-- Note: Validation est optionnelle. Lancer `bmad-create-story validate 8-2`
     avant `dev-story` pour stress-test la spec. Cycle CLAUDE.md : auteur =
     Opus, Pass 1 recommandé Sonnet pour briser le biais d'auteur. -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse)**,
I want **importer mes relevés bancaires CSV (UTF-8 ou ISO-8859-1) via la page `/bank-import`, avec détection automatique de l'encodage et profils de format par banque réutilisables**,
so that **je puisse traiter dans Kesh les banques qui n'exposent pas CAMT.053, sans configuration manuelle au-delà du premier import par banque**.

### Contexte

**Story 8-2 = seconde voie d'import** complétant Story 8-1 (CAMT.053 ISO 20022). La couche persistance + UI livrée par 8-1b est **réutilisée intégralement** : tables `bank_imports` + `bank_transactions`, route `POST /bank-imports`, page `/bank-import`. Cette story ajoute :

1. **Module parser `kesh-import::csv`** — autonome (zéro dép `kesh-core`), détection encoding via `encoding_rs` + `chardetng`, parser via `csv` crate.
2. **Table `bank_profiles`** + CRUD API + UI minimal pour créer/éditer un profil par banque (mapping colonnes, format date, séparateur).
3. **Pipeline de mapping** profil → `ImportedTransaction` (type partagé avec CAMT.053 via `kesh-import::types`).

**Dépendances upstream (déjà livrées)** :
- Story 8-1a (PR #69 mergée 2026-05-05) : `kesh-import` crate publiable, types autonomes `ImportedStatement` / `ImportedTransaction` / `SourceFormat`, helpers `kesh-core::bank_imports::{from_imported, validate_balance, validate_currency_supported_v0_1}`, invariants CI `cargo publish --dry-run` + `cargo metadata`.
- Story 8-1b (PR #69 mergée 2026-05-05) : tables DB, route `POST /bank-imports` multipart, audit log atomique, RBAC sub-router, frontend feature `bank-import`, i18n 4 locales, payload limit `KESH_BANK_IMPORT_MAX_MB`.

**Dépendances downstream** :
- Story 8-3 (détection doublons + rejet partiel) — `bank_imports.file_hash` UNIQUE par company déjà en place ; 8-3 ajoutera la détection ligne-par-ligne sur `(date, amount, reference, bank_account_id)`. **8-2 ne fait PAS de dédup ligne** : seul l'unicité fichier (héritée 8-1b via `(company_id, file_hash) UNIQUE`) est en jeu.
- Story 8-4 (réconciliation auto) — ne dépend pas du source_format, consomme `bank_transactions` agnostiquement.

**Status sprint** : `8-2-import-csv-multi-encodage-profils-banque: ready-for-dev` post-création spec 2026-05-05.

### Scope verrouillé — ce qui est livré par 8-2

1. **Migration DB** (T1) — `crates/kesh-db/migrations/2026MMDDHHMMSS_bank_profiles.sql` créant `bank_profiles` (id, company_id, bank_name, column_mapping JSON, date_format, encoding, separator, decimal_separator, header_row_count, created_at, updated_at). **Pas** de modification de `bank_imports`/`bank_transactions` (le `source_format` accepte déjà `'CSV'` — ENUM string `('CAMT053_V04', 'CAMT053_V08', 'CSV')` côté Rust, tableau MAJUSCULES côté DB).
2. **Module parser `kesh-import::csv`** (T2) — `crates/kesh-import/src/csv/{mod.rs, encoding.rs, parser.rs, profile.rs}` + tests `csv_tests.rs`. Détection encoding (BOM → `chardetng`), parser via `csv` crate, mapping profil → `ImportedTransaction`. Variantes `CamtError` étendues ou nouveau type `ImportError` avec sous-type `Csv(CsvError)` (cf. §error-types).
3. **Entités + repositories `bank_profiles`** (T3) — `kesh-db/src/entities/bank_profile.rs` + `kesh-db/src/repositories/bank_profiles.rs` (CRUD + helper `find_matching_profile_for_filename`).
4. **Routes API CRUD `bank_profiles`** (T4) — `kesh-api/src/routes/bank_profiles.rs` (5 endpoints : `POST/GET-list/GET-detail/PUT/DELETE`), RBAC `comptable_routes`, audit log, scoping company.
5. **Extension route `POST /bank-imports`** (T5) — détection CSV vs CAMT.053 par MIME / extension fichier, dispatch vers parser CSV avec profile lookup, integration avec la pipeline existante.
6. **Frontend feature `bank-csv-import`** (T6) — extension `frontend/src/lib/features/bank-import/` :
   - `BankProfileForm.svelte` (créer/éditer profil)
   - `BankProfileSelector.svelte` (sélecteur dans le flow upload, intercalé entre file-input et preview si CSV détecté + profil non-matchant)
   - `bank-profile.api.ts` + `.types.ts`
   - Route `/bank-import/profiles` (liste profiles + bouton « Nouveau profil »).
7. **i18n** (T7) — clés `bank-profile-*` + `bank-csv-errors-*` + `bank-csv-warnings-*` dans 4 locales `fr/de/it/en-CH`, lint-i18n-ownership pass.
8. **Tests E2E Playwright** (T8) — `frontend/tests/e2e/bank-csv-import.spec.ts` (6+ scénarios), fixtures CSV (UTF-8 BOM, ISO-8859-1, multi-séparateur, format date variés, malformed).
9. **Sync sprint-status + README** (T9).

**HORS scope 8-2 (reportés Stories 8-3/8-4) :**
- Détection ligne-par-ligne de doublons sur `(date, amount, reference)` → 8-3.
- Rejet partiel avec ré-import des lignes corrigées → 8-3 (8-2 fait du rejet partiel **avec listing erreurs uniquement**, sans flow de re-import des lignes en erreur).
- Réconciliation auto avec écritures existantes → 8-4.
- Détection profil par contenu (header row matching) en mode auto → 8-2 livre **filename-based matching** uniquement (regex sur le nom de fichier) ; le header-content matching est report 8-3 ou v0.2.

### Décisions de conception

#### §encoding-detection

**Algorithme déterministe** (R4 epic-8.md résolu) :

1. **BOM check** (priorité 1) — `encoding_rs::Encoding::for_bom(&bytes[..3])` :
   - `EF BB BF` → UTF-8 (consommer 3 bytes BOM, parser UTF-8 strict)
   - `FF FE` / `FE FF` → UTF-16 LE/BE → **422 BANK_CSV_UNSUPPORTED_ENCODING** (non v0.1).
2. **Sans BOM** — heuristique en deux passes :
   - Passe 1 : tenter décodage UTF-8 strict via `std::str::from_utf8`. Si OK → UTF-8.
   - Passe 2 : si UTF-8 fail → `chardetng::EncodingDetector` sur les premiers 1024 bytes. Si verdict `windows-1252` ou `iso-8859-1` → ISO-8859-1.
   - Sinon (autre encoding détecté) → `422 BANK_CSV_UNSUPPORTED_ENCODING`.

**Pas d'override utilisateur en v0.1** : le profil stocke `encoding` (nullable, default `null` = auto-détect). Si `encoding` non-null et conflict avec détection → warning preview `bank_csv_encoding_mismatch` + utilise la valeur du profil. Cas extrême (BOM UTF-16 + `encoding=ISO-8859-1` profil) → reject `422`.

**Lib choisies** :
- `encoding_rs` 0.8.x — Mozilla Servo, BOM detection + decoding standardisé W3C Encoding.
- `chardetng` 0.1.x — Mozilla, port chardet repo Mozilla, recommandé pour fallback heuristique.

#### §profile-model

```rust
// kesh-db/src/entities/bank_profile.rs
pub struct BankProfile {
    pub id: i64,
    pub company_id: i64,
    pub bank_name: String,        // 1-100 chars, UNIQUE per company
    pub filename_pattern: Option<String>,  // regex compilable, None = pas d'auto-match
    pub column_mapping: ColumnMapping,     // JSON sérialisé en DB
    pub date_format: String,      // chrono format string ex. "%d.%m.%Y"
    pub decimal_separator: char,  // '.' | ','
    pub field_separator: char,    // ',' | ';' | '\t'
    pub encoding: Option<String>, // None = auto, sinon "UTF-8" | "ISO-8859-1"
    pub header_row_count: u8,     // 0..5, default 1
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// JSON serialized in column_mapping
pub struct ColumnMapping {
    pub date: usize,            // 0-indexed column
    pub amount: usize,          // OR (debit, credit) tuple
    pub debit_credit_split: Option<(usize, usize)>,  // alt à amount
    pub reference: Option<usize>,
    pub details: Option<usize>,
    pub counterparty: Option<usize>,
}
```

**Validation côté API** :
- `bank_name` non-vide, 1..=100 chars
- `column_mapping` : exactement un de `amount` xor `debit_credit_split` (XOR strict)
- `date_format` : valider via `chrono::format::strftime::StrftimeItems::new(fmt).all(|i| !matches!(i, Item::Error))` au create/update
- `field_separator` ∈ `{',', ';', '\t'}` v0.1 (pas de full custom)
- `decimal_separator` ∈ `{'.', ','}`
- `field_separator != decimal_separator` (sinon ambiguïté)
- `header_row_count` ∈ `0..=5`
- `filename_pattern` (si Some) : regex valide, longueur ≤ 200 chars

**Stockage `column_mapping` en DB** : type `JSON` MariaDB (10.2+ supporte JSON natif via `JSON_VALID` CHECK). Sérialisation Rust via `serde_json::to_string` côté insert/update, désérialisation via `serde_json::from_str` côté SELECT. Pas de `sqlx::types::Json` direct (l'integration sqlx-mysql + JSON natif a des subtilités v0.8.6 — préférer string serialization explicite, pattern Story 7-2 pour `vat_rates.metadata`).

**UNIQUE constraint** : `(company_id, bank_name)` — un seul profil par banque par tenant.

#### §profile-matching

À l'upload d'un fichier CSV, ordre de résolution du profil :
1. **bankProfileId explicite** (champ multipart `bankProfileId=N`) — utilisateur a sélectionné un profil dans l'UI → utiliser tel quel.
2. **Auto-match par filename** — si `bankProfileId` absent ET fichier détecté CSV : `SELECT FROM bank_profiles WHERE company_id = ? AND filename_pattern IS NOT NULL` puis tester chaque regex sur le nom du fichier. Premier match (ORDER BY `updated_at DESC`) → applique. Plusieurs matches → utilise le plus récent + warning `bank_csv_multiple_profile_matches`.
3. **Aucun match** → preview retourne `404 BANK_CSV_NO_PROFILE_MATCH` avec `details.available_profiles: [{id, bank_name}]` pour aider l'utilisateur à sélectionner manuellement. **Pas** de fallback inline UI v0.1 (création de profil = flow séparé).

**Important** : le profil n'est consulté QUE pour les CSV. Les CAMT.053 (détectés via MIME `application/xml` ou ext `.xml`) bypassent complètement ce path.

#### §csv-detection

Détection CSV vs CAMT.053 sur le multipart upload :
- **Extension fichier** (priorité 1) : `.xml` → CAMT.053, `.csv` ou `.txt` → CSV.
- **MIME type** (priorité 2) : `application/xml` / `text/xml` → CAMT, `text/csv` / `application/vnd.ms-excel` / `text/plain` → CSV.
- **Sniff content** (priorité 3) : premiers 256 bytes après decoding — si commence par `<?xml` ou `<` puis `BkToCstmrStmt` → CAMT, sinon → CSV.
- **Aucun match** → `415 BANK_IMPORT_UNSUPPORTED_FORMAT` (nouveau).

#### §csv-parser

**Lib** : `csv` crate (BurntSushi) 1.3.x.
- `csv::ReaderBuilder::new().delimiter(profile.field_separator as u8).has_headers(profile.header_row_count > 0).from_reader(decoded_bytes.as_bytes())`.
- Skip `header_row_count - 1` rows additionnels après le `has_headers` skip natif (puisque `csv` crate skip uniquement la première ligne).
- `iter().enumerate()` → row index 1-based **côté utilisateur** (humanly addressable line numbers : si `header_row_count=1` et erreur ligne 5 du fichier, retourner `line: 5` pas `line: 4`).

**Mapping value parsing** :
- **Date** : `chrono::NaiveDate::parse_from_str(value, &profile.date_format)`. Erreur → `CsvError::InvalidDate { line, value, format }`.
- **Amount** :
  - Si `debit_credit_split` : parse les deux colonnes, signe `+` si crédit non-vide, `-` si débit non-vide, `0` si les deux vides → erreur `CsvError::AmbiguousDebitCredit { line }` si les deux non-vides.
  - Sinon : parse `amount` directement (signe préservé). `decimal_separator=','` → remplace `,` par `.` avant parse. Strip apostrophe (séparateur milliers suisse `1'234.56` → `1234.56`).
  - Lib : `rust_decimal::Decimal::from_str_exact` (rejection NaN/infinity, contrairement à `from_str`).
- **Reference / Details / Counterparty** : `Option<String>`, trim, empty → `None`.
- **Currency** : non lisible depuis CSV v0.1, **assumé CHF**. Validation `validate_currency_supported_v0_1` skip pour CSV (currency = "CHF" forced). Document explicit dans dev notes.
- **booking_date == value_date** : pas de distinction CSV v0.1 (la majorité des banques exposent une seule date dans leur CSV). Le helper `from_imported` côté `kesh-core` accepte déjà `booking_date == value_date`.

#### §error-types

**Choix** : nouveau type `ImportError` dans `kesh-import::error.rs` qui wrap `CamtError` ET un nouveau `CsvError` :

```rust
// kesh-import/src/error.rs (extension)
pub enum ImportError {
    Camt(CamtError),
    Csv(CsvError),
}

pub enum CsvError {
    UnsupportedEncoding { detected: Option<String> },
    DecodingFailed { encoding: String, byte_offset: usize },
    MissingHeader,
    InvalidDate { line: usize, value: String, format: String },
    InvalidAmount { line: usize, value: String },
    AmbiguousDebitCredit { line: usize },
    EmptyMandatoryField { line: usize, field: &'static str },
    RowTooShort { line: usize, expected_cols: usize, got: usize },
    Io(String),
}
```

**Impact** : `kesh-import::lib.rs` re-exporte `pub use error::{ImportError, CamtError, CsvError}`. Les types `From<CamtError> for ImportError` et `From<CsvError> for ImportError` permettent la propagation `?`.

`kesh-core::errors::CoreError` étendu avec :
- `BankCsvProfileNotFound`
- `BankCsvUnsupportedEncoding(String)` (encoding détecté pour diag)
- `BankCsvParsePartialFailure(Vec<CsvLineError>)` — wrap pour le rejet partiel structuré FR51
- `BankCsvProfileValidation(String)` (msg de validation profil)

Côté `kesh-api::errors::AppError` : 6 nouvelles variantes mappées vers HTTP :
- `BankCsvProfileNotFound` → 404 `BANK_CSV_NO_PROFILE_MATCH`
- `BankCsvUnsupportedEncoding` → 422 `BANK_CSV_UNSUPPORTED_ENCODING`
- `BankCsvParsePartialFailure` → 422 `BANK_CSV_PARTIAL_FAILURE` avec payload `{lines: [{line, code, value, message}]}`
- `BankCsvProfileValidation` → 422 `BANK_CSV_PROFILE_INVALID`
- `BankCsvProfileDuplicate` → 409 `BANK_CSV_PROFILE_DUPLICATE` (UNIQUE bank_name)
- `BankImportUnsupportedFormat` → 415 `BANK_IMPORT_UNSUPPORTED_FORMAT`

#### §partial-failure-mapping

**Décision FR51 v0.1 — strict reject** : si **N'IMPORTE QUELLE** ligne du CSV échoue au parsing, l'import entier est rejeté (`422 BANK_CSV_PARTIAL_FAILURE`). Le payload de réponse liste **toutes** les lignes en erreur avec `{line, code, value, message_i18n_key}` pour l'UI affiche un panneau « 12 lignes en erreur ». Pas de partial commit en v0.1.

**Justification** : un partial commit nécessite (a) flow de re-import des lignes corrigées, (b) gestion de l'unicité fichier-hash quand le fichier corrigé a un nouveau hash, (c) UI pour éditer les lignes en erreur inline. Tous ces points sont reportés Story 8-3 (rejet partiel + dédup ligne par ligne). Pour v0.1 (Story 8-2), strict reject + listing détaillé suffit pour le scénario Lisa de la PRD (« Lisa consulte les lignes en erreur, crée un profil banque personnalisé, et réimporte les 3 lignes manquantes » → en 8-2 elle réimporte le fichier complet après avoir corrigé le profil ; en 8-3 elle pourra réimporter juste le delta).

#### §upload-limit

Hérité 8-1b : `KESH_BANK_IMPORT_MAX_MB` (default 10, range [1, 100]) appliqué via `DefaultBodyLimit` sur le sub-router. **Aucune nouvelle env var pour 8-2**.

#### §audit-log

Trois nouvelles actions audit :
- `bank_profile.created` (entity_type `bank_profiles`, entity_id, details `{bank_name, filename_pattern}`)
- `bank_profile.updated` (idem, details `{bank_name, fields_changed: [...]}`)
- `bank_profile.deleted` (idem, details `{bank_name}`)

Action existante `bank_import.created` étend son `details_json` avec `source_format: "CSV"` et `bank_profile_id` (nullable si CAMT).

## Acceptance Criteria

Numérotation indépendante de 8-1b. Les ACs marqués **Hérité 8-1b** réutilisent le code persistance/UI sans re-test (8-2 vérifie uniquement les nouveaux comportements CSV-specific).

1. **(FR42 + FR50 — happy path UTF-8)** Given un fichier CSV UTF-8 valide avec BOM, profil banque correspondant existant, When l'utilisateur sélectionne le profil + uploade le fichier + clique « Confirmer », Then toutes les transactions sont persistées dans `bank_transactions` avec `bank_account_id = selected_id` et `bank_imports.source_format = 'CSV'`. *Test : E2E `imports a CSV UTF-8 file end-to-end` + integration `kesh-import::csv::parser::tests::parses_utf8_with_bom`.*

2. **(FR52a — détection BOM UTF-8)** Given un fichier CSV avec BOM `EF BB BF` au début, When parsing, Then les 3 bytes BOM sont consommés et le contenu décodé en UTF-8. *Test unit : `detects_and_strips_utf8_bom`.*

3. **(FR52b — détection ISO-8859-1 sans BOM)** Given un fichier CSV ISO-8859-1 (caractères suisses-français accentués `é`, `ç`, `à` en bytes 0xE9, 0xE7, 0xE0), When parsing sans BOM, Then la détection chardetng identifie `windows-1252`/`iso-8859-1` et le décodage produit la chaîne UTF-8 correcte. *Test unit : `detects_iso_8859_1_via_heuristic` + fixture `csv_iso_8859_1_swiss_accents.csv`.*

4. **(FR52c — UTF-8 sans BOM, ASCII pur)** Given un fichier CSV ASCII pur (tous bytes < 0x80), When parsing sans BOM, Then la passe 1 UTF-8 strict réussit et le décodage est UTF-8. *Test unit : `parses_ascii_as_utf8`.*

5. **(FR52d — encoding non supporté)** Given un fichier UTF-16 LE (BOM `FF FE`), When upload, Then `422 BANK_CSV_UNSUPPORTED_ENCODING` avec `details.detected = "UTF-16LE"`. Aucun parsing n'est tenté. *Test E2E HTTP : `post_csv_rejects_utf16_encoding`.*

6. **(FR53 — création profil banque)** Given un utilisateur Comptable, When `POST /api/v1/bank-profiles` avec body valide, Then `201 Created` + entrée DB + audit log `bank_profile.created`. *Test E2E HTTP : `post_bank_profile_creates_with_audit_log`.*

7. **(FR53 — auto-apply profil par filename)** Given un profil banque avec `filename_pattern = "^export-ubs-\\d{8}\\.csv$"`, When upload `export-ubs-20260315.csv` sans `bankProfileId` explicite, Then le preview applique automatiquement le profil + warning UI `bank_csv_profile_auto_matched: { profile_id, bank_name }`. *Test E2E HTTP : `post_csv_preview_auto_matches_profile_by_filename` + scénario Playwright `auto-applies bank profile on upload`.*

8. **(FR53 — auto-apply ambigu)** Given deux profils avec `filename_pattern` qui matchent tous deux le filename uploadé, When preview, Then le plus récent (`updated_at DESC`) est appliqué + warning `bank_csv_multiple_profile_matches: { matched_profiles: [...] }`. *Test E2E HTTP : `post_csv_preview_warns_on_multiple_profile_matches`.*

9. **(FR53 — aucun profil match)** Given un fichier CSV uploadé sans `bankProfileId` ET aucun `filename_pattern` ne matche, When preview, Then `404 BANK_CSV_NO_PROFILE_MATCH` avec `details.available_profiles = [{id, bank_name}]`. *Test E2E HTTP : `post_csv_preview_404_when_no_profile_matches`.*

10. **(FR51 — rejet partiel avec listing erreurs)** Given un CSV avec 5 lignes valides + 3 lignes invalides (date malformée ligne 7, montant non-numérique ligne 12, ligne 18 trop courte), When `POST /bank-imports`, Then `422 BANK_CSV_PARTIAL_FAILURE` avec body `{ lines: [{line: 7, code: "INVALID_DATE", value: "32.13.2026", message_i18n_key: "bank-csv-errors-invalid-date"}, ...] }`. **Aucune** transaction persistée (strict reject v0.1). *Test E2E HTTP : `post_csv_rejects_partial_failure_with_detailed_lines`.*

11. **(Multi-tenant scoping bank_profiles — KF-002 pattern)** Given un profil créé par `company_A`, When `company_B` appelle `GET /api/v1/bank-profiles/{id}` ou `GET /api/v1/bank-profiles` (list), Then `404 Not Found` (jamais 403). *Tests : `get_profile_returns_404_for_other_company` + `list_profiles_only_returns_own_company`.*

12. **(Multi-tenant DB profile auto-match)** Given `company_A` a un profil avec `filename_pattern = "ubs.csv"`, When `company_B` uploade `ubs.csv` sans bankProfileId, Then `404 BANK_CSV_NO_PROFILE_MATCH` (les profils de `company_A` n'ont pas leaké). *Test integration : `auto_match_only_considers_own_company_profiles`.*

13. **(Sécurité — RBAC profil)** Given `Role::Consultation`, When `POST/PUT/DELETE /api/v1/bank-profiles/...`, Then `403`. `GET` accessible à tous les rôles authentifiés. *Tests : `post_profile_rejects_consultation_role` + `get_profile_allowed_for_consultation_role`.*

14. **(Sécurité — payload limit)** Hérité 8-1b. CSV > 10 MiB → `413 BANK_IMPORT_TOO_LARGE`. *Pas de re-test, lien vers test `post_import_rejects_payload_too_large` 8-1b.*

15. **(Validation profil — XOR amount/debit_credit_split)** Given un payload de création profil avec `column_mapping: { amount: 3, debit_credit_split: [4, 5] }`, When `POST /bank-profiles`, Then `422 BANK_CSV_PROFILE_INVALID` avec message `column_mapping doit contenir exactement un de amount XOR debit_credit_split`. Cas inverse (ni l'un ni l'autre) → même rejet. *Test E2E HTTP : `post_profile_rejects_xor_violation`.*

16. **(Validation profil — date_format chrono)** Given `date_format = "%Q"` (token invalide chrono), When `POST /bank-profiles`, Then `422 BANK_CSV_PROFILE_INVALID` avec message ciblant `date_format`. *Test E2E HTTP : `post_profile_rejects_invalid_chrono_format`.*

17. **(Validation profil — UNIQUE bank_name par company)** Given un profil `bank_name = "UBS"` existe déjà pour `company_A`, When second `POST /bank-profiles` même `bank_name`, Then `409 BANK_CSV_PROFILE_DUPLICATE`. Le même `bank_name` est OK pour `company_B` (multi-tenant). *Tests : `post_profile_rejects_duplicate_bank_name_within_company` + `post_profile_allows_same_bank_name_across_companies`.*

18. **(Doublons fichier — héritage 8-1b `(company_id, file_hash) UNIQUE`)** Hérité 8-1b. CSV ré-importé même fichier → `409 BANK_IMPORT_DUPLICATE_FILE`. *Pas de re-test 8-2, lien `unique_company_hash_blocks_duplicate_within_same_company`.*

19. **(Atomicité)** Hérité 8-1b. Strict reject 8-2 implique aucune persistance partielle, donc rien à tester en plus. *Pas de re-test.*

20. **(Audit log import CSV)** Given un import CSV réussi, When `SELECT FROM audit_log WHERE entity_type='bank_imports'`, Then une entrée `action = bank_import.created` avec `details_json` contient `source_format: "CSV"` ET `bank_profile_id: <id>`. *Test E2E HTTP : `post_csv_import_audit_log_includes_source_format_and_profile`.*

21. **(Audit log profil)** Given un profil créé/modifié/supprimé, When `SELECT FROM audit_log WHERE entity_type='bank_profiles'`, Then trois entrées distinctes (`created` / `updated` / `deleted`) avec `entity_id = profile.id` et `details_json` rempli. *Tests : `post_profile_writes_audit_log_created` + `put_profile_writes_audit_log_updated` + `delete_profile_writes_audit_log_deleted`.*

22. **(i18n)** Given les 4 locales (fr/de/it/en-CH), When `npm run lint-i18n-ownership`, Then le lint passe (toutes clés `bank-profile-*` + `bank-csv-errors-*` + `bank-csv-warnings-*` présentes, préfixe kebab-case strict). *Test : CI Story 6-3 + extension `keyBelongsToFeature` 8-1b déjà supporte multi-segment names (`bank-profile`, `bank-csv`).*

23. **(Accessibilité — page profils + wizard)** Given pages `/bank-import/profiles` (liste) et le formulaire `BankProfileForm`, When `axe-core` scan, Then zéro violation. Les selects `field_separator` et `decimal_separator` ont `aria-label`, le textarea `column_mapping` n'est pas exposé brut (UI structure les colonnes en formulaire row-by-row). *Test E2E : `accessibility — profile pages axe scan zero violations`.*

24. **(Performance NFR)** Given un fichier CSV de 200 transactions UTF-8, When `POST /bank-imports`, Then la durée totale (decode + parse + DB) < 2s sur la machine de dev nominale. *Test instrumentation : `csv_pipeline_handles_500_transactions` smoke `Instant::now()`.*

25. **(Strip apostrophe milliers + decimal_separator virgule)** Given un CSV format suisse `1'234,56` avec `decimal_separator=','`, When parsing amount, Then la valeur résultante est `Decimal::new(123456, 2) = 1234.56`. *Test unit : `parses_swiss_amount_with_apostrophe_thousands_and_comma_decimal`.*

26. **(Profil filename_pattern regex injection safe)** Given un payload de création profil avec `filename_pattern = "(?:.*){10000}"` (catastrophic backtracking pattern), When `POST /bank-profiles`, Then la regex est validée via `regex::Regex::new()` avec **le default size limit** (`regex` crate borne déjà la complexité via `size_limit` 10MB par défaut). Pattern qui dépasse → `422 BANK_CSV_PROFILE_INVALID`. *Test E2E HTTP : `post_profile_rejects_pathological_regex`.*

## Tasks / Subtasks

### T1. Migration DB `bank_profiles` (AC #6, #11, #15, #17, #21)

- [ ] **T1.1** Créer `crates/kesh-db/migrations/{TIMESTAMP}_bank_profiles.sql` :
  ```sql
  CREATE TABLE bank_profiles (
      id BIGINT PRIMARY KEY AUTO_INCREMENT,
      company_id BIGINT NOT NULL,
      bank_name VARCHAR(100) NOT NULL,
      filename_pattern VARCHAR(200) NULL,
      column_mapping JSON NOT NULL,
      date_format VARCHAR(50) NOT NULL,
      decimal_separator CHAR(1) NOT NULL,
      field_separator CHAR(1) NOT NULL,
      encoding VARCHAR(20) NULL,
      header_row_count TINYINT UNSIGNED NOT NULL DEFAULT 1,
      created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
      updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
      CONSTRAINT fk_bank_profiles_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
      CONSTRAINT chk_bank_profiles_field_separator CHECK (field_separator IN (',', ';', '\t')),
      CONSTRAINT chk_bank_profiles_decimal_separator CHECK (decimal_separator IN ('.', ',')),
      CONSTRAINT chk_bank_profiles_separators_distinct CHECK (field_separator <> decimal_separator),
      CONSTRAINT chk_bank_profiles_header_row_count CHECK (header_row_count <= 5),
      CONSTRAINT chk_bank_profiles_bank_name_len CHECK (CHAR_LENGTH(bank_name) BETWEEN 1 AND 100),
      CONSTRAINT chk_bank_profiles_column_mapping_valid CHECK (JSON_VALID(column_mapping)),
      UNIQUE KEY uq_bank_profiles_company_name (company_id, bank_name),
      KEY idx_bank_profiles_company (company_id)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
  ```
- [ ] **T1.2** Mettre à jour `crates/kesh-db/src/test_fixtures.rs::TABLES_TO_TRUNCATE` : ajouter `"bank_profiles"` (enfant de `companies`, ordre = avant `companies`). **Leçon CI 8-1b 2026-05-05** : ne pas oublier sous peine de `truncate_all_inventory_matches_schema` rouge en CI.
- [ ] **T1.3** Test smoke local `cargo test -p kesh-db --lib test_fixtures` avec MariaDB up + KESH_TEST_MODE=true.

### T2. Module parser `kesh-import::csv` (AC #1, #2, #3, #4, #5, #10, #25)

- [ ] **T2.1** `crates/kesh-import/src/csv/encoding.rs` :
  - `pub fn detect_encoding(bytes: &[u8]) -> Result<DetectedEncoding, CsvError>`
  - `pub enum DetectedEncoding { Utf8, Iso8859_1 }` + `Display`
  - BOM check via `encoding_rs::Encoding::for_bom`, fallback heuristique `chardetng::EncodingDetector::feed(bytes, true)` puis `guess(None, true)`.
  - Tests : 6 cases (UTF-8 BOM, UTF-8 no BOM, UTF-8 ASCII, ISO-8859-1 swiss accents, UTF-16 LE BOM → reject, empty → reject).
- [ ] **T2.2** `crates/kesh-import/src/csv/profile.rs` :
  - `pub struct CsvProfile` (mêmes champs que `BankProfile` côté DB sauf `id`/`company_id`/`created_at`/`updated_at` — types autonomes pour rester publiable).
  - `pub struct ColumnMapping` + `serde::{Serialize, Deserialize}`.
  - `pub fn validate(&self) -> Result<(), CsvError>` (XOR amount/debit_credit, chrono format valid, regex valid, separators distincts).
- [ ] **T2.3** `crates/kesh-import/src/csv/parser.rs` :
  - `pub fn parse_csv(bytes: &[u8], profile: &CsvProfile) -> Result<ImportedStatement, CsvError>`
  - Décodage encoding → string, csv crate ReaderBuilder, skip header_row_count, iter rows mappés → `ImportedTransaction`.
  - Strip apostrophe (`'`) milliers + remplacement decimal_separator → `.` avant `Decimal::from_str_exact`.
  - Sur erreur ligne, **collecter tous les erreurs** (Vec) puis si non-vide → `CsvError::PartialFailure(Vec<CsvLineError>)`. Strict reject v0.1.
  - **Pas** de validation balance pour CSV (les CSV n'exposent pas opening/closing balance) — le helper `validate_balance` est skip côté API pour `source_format = CSV`.
- [ ] **T2.4** `crates/kesh-import/src/error.rs` : étendre avec `CsvError` (cf. §error-types) + `ImportError` enum wrapper. `From<CamtError>` + `From<CsvError>` for `ImportError`.
- [ ] **T2.5** `crates/kesh-import/src/lib.rs` : `pub mod csv;` + `pub use csv::{parse_csv, CsvProfile, ColumnMapping, DetectedEncoding};` + `pub use error::{ImportError, CamtError, CsvError, CsvLineError};`.
- [ ] **T2.6** Tests `crates/kesh-import/tests/csv_tests.rs` : 12+ tests d'intégration utilisant les fixtures `tests/fixtures/csv/*.csv`.
- [ ] **T2.7** Fixtures `tests/fixtures/csv/` : `utf8_bom_minimal.csv` (3 tx, BOM, ;), `iso_8859_1_swiss_accents.csv` (5 tx, accents é/ç/à), `utf8_swiss_amount.csv` (apostrophe milliers + virgule decimal), `partial_failure.csv` (8 rows : 5 OK + 3 invalid mixed), `utf16_le.csv` (1 tx UTF-16 LE BOM — reject test).
- [ ] **T2.8** Cargo.toml — ajouter dépendances : `csv = "1.3"`, `encoding_rs = "0.8"`, `chardetng = "0.1"`, `regex = "1.10"` (pour profile filename_pattern validation côté kesh-import). **Vérifier `cargo publish --dry-run` reste vert** (aucune dép interne).

### T3. Entités + repos `bank_profiles` (AC #6, #11, #17, #21)

- [ ] **T3.1** `crates/kesh-db/src/entities/bank_profile.rs` :
  - `pub struct BankProfile` (champs cf. §profile-model, sans `column_mapping` typé — stocké en `String` JSON, désérialisé via méthode `pub fn parse_column_mapping(&self) -> Result<ColumnMapping, DbError>`).
  - `pub struct NewBankProfile` (sans id/created_at/updated_at).
  - **Pattern `BankImportSourceFormat` 8-1b ne s'applique pas ici** (pas d'enum).
- [ ] **T3.2** `crates/kesh-db/src/repositories/bank_profiles.rs` :
  - `create(pool, company_id, profile) -> Result<BankProfile, DbError>` + `INSERT INTO audit_log` atomique dans la même transaction (pattern 8-1b).
  - `find_by_id_for_company(pool, company_id, id) -> Result<Option<BankProfile>, DbError>` (KF-002 helper).
  - `list_by_company(pool, company_id, pagination) -> Result<(Vec<BankProfile>, i64), DbError>`.
  - `update(pool, company_id, id, patch) -> Result<BankProfile, DbError>` + audit log.
  - `delete(pool, company_id, id) -> Result<(), DbError>` + audit log.
  - `find_matching_profiles_for_filename(pool, company_id, filename) -> Result<Vec<BankProfile>, DbError>` — SELECT puis filter Rust-side via `regex::Regex::new(profile.filename_pattern).is_match(filename)` (filtrage SQL impossible, regex MariaDB diffère). ORDER BY `updated_at DESC`.
- [ ] **T3.3** Mapping erreurs SQL :
  - `1062` (Duplicate entry) sur `uq_bank_profiles_company_name` → `DbError::ProfileDuplicate`.
- [ ] **T3.4** Tests `kesh-db` : 8 tests `#[sqlx::test]` (create + audit, find_by_id_own_company, find_by_id_other_company_returns_none, list_paginated, update + audit + race optimistic, delete + audit, duplicate_bank_name_rejected, find_matching_profiles_filters_by_company).

### T4. Routes API `bank_profiles` CRUD (AC #6, #11, #13, #15, #16, #17, #21, #26)

- [ ] **T4.1** `crates/kesh-api/src/routes/bank_profiles.rs` (nouveau fichier) — 5 handlers :
  - `POST /api/v1/bank-profiles` (create, RBAC Comptable+).
  - `GET /api/v1/bank-profiles?page=&per_page=` (list, all roles).
  - `GET /api/v1/bank-profiles/{id}` (detail, all roles).
  - `PUT /api/v1/bank-profiles/{id}` (update, Comptable+).
  - `DELETE /api/v1/bank-profiles/{id}` (delete, Comptable+).
- [ ] **T4.2** `crates/kesh-api/src/errors.rs` : 4 nouvelles variantes `AppError` :
  - `BankCsvProfileNotFound` → 404 `BANK_CSV_PROFILE_NOT_FOUND`
  - `BankCsvProfileValidation(String)` → 422 `BANK_CSV_PROFILE_INVALID`
  - `BankCsvProfileDuplicate` → 409 `BANK_CSV_PROFILE_DUPLICATE`
  - `BankCsvProfilePathologicalRegex` → 422 `BANK_CSV_PROFILE_INVALID` (regex builder failed) — peut être absorbé dans `BankCsvProfileValidation`.
- [ ] **T4.3** Validation côté handler (avant repo) :
  - Parse `column_mapping` JSON → struct → `validate()` (cf. T2.2)
  - Compile `filename_pattern` via `regex::Regex::new` — failure → 422.
  - Return early avec `BankCsvProfileValidation(reason)`.
- [ ] **T4.4** Mount sur `comptable_routes` (write) + `authenticated_routes` (read), pattern 8-1b.
- [ ] **T4.5** Tests E2E HTTP `crates/kesh-api/tests/bank_profiles_e2e.rs` : 12 tests (create+audit, RBAC, list scoping, detail 404 cross-tenant, update+audit, delete+audit, validation errors, duplicate 409, pathological regex 422).

### T5. Extension `POST /bank-imports` pour CSV (AC #1, #5, #7, #8, #9, #10, #20)

- [ ] **T5.1** `crates/kesh-api/src/routes/bank_imports.rs` — détection format upload :
  - Helper `fn detect_import_format(filename: &str, content_type: Option<&str>, first_bytes: &[u8]) -> Result<ImportFormat, AppError>`
  - `enum ImportFormat { Camt053, Csv }`
  - Fallback `415 BANK_IMPORT_UNSUPPORTED_FORMAT`.
- [ ] **T5.2** Handler `preview_bank_import` (existing) extension :
  - Si CSV : extraire `bankProfileId` du multipart (Optional), résoudre profil via repo (explicit ID OR auto-match filename), parse via `kesh-import::parse_csv`, mapper `ImportedStatement` → preview response.
  - Réponse preview enrichie : `appliedProfile: { id, bank_name }` + `warnings.csv: ["profile_auto_matched"|"multiple_profile_matches"]`.
- [ ] **T5.3** Handler `create_bank_import` (existing) extension :
  - Idem détection + dispatch CSV.
  - **Skip** validation balance pour CSV (`source_format = "CSV"`).
  - **Skip** validation currency pour CSV (assumed CHF v0.1, document inline).
  - `audit_log.details_json` enrichi `{source_format: "CSV", bank_profile_id: <id|null>, transaction_count}`.
- [ ] **T5.4** Configuration : aucune nouvelle env var (KESH_BANK_IMPORT_MAX_MB hérité).
- [ ] **T5.5** Tests E2E HTTP étendus dans `bank_imports_e2e.rs` : 8 nouveaux tests CSV (happy UTF-8, ISO-8859-1, UTF-16 reject, partial failure 422, no profile match 404, auto-match by filename, multiple profile match warning, audit log includes source_format).

### T6. Frontend feature `bank-csv-import` (AC #6, #7, #9, #10, #15, #16, #17, #23)

- [ ] **T6.1** `frontend/src/lib/features/bank-import/BankProfileForm.svelte` :
  - Formulaire create/edit profile (bank_name, filename_pattern, separators, encoding, date_format, column_mapping rows).
  - Validation client-side (XOR amount, regex compile preview).
  - i18n `bank-profile-labels-*` + `bank-profile-errors-*`.
  - `data-testid` strict pattern KF-008.
- [ ] **T6.2** `frontend/src/lib/features/bank-import/BankProfileList.svelte` :
  - Liste profils company avec actions edit/delete.
  - Pagination héritée pattern existant.
- [ ] **T6.3** `frontend/src/lib/features/bank-import/BankProfileSelector.svelte` :
  - Dropdown intercalé dans `BankImportUpload` quand CSV détecté + 0 auto-match.
  - Lien « Créer un profil » → route `/bank-import/profiles/new`.
- [ ] **T6.4** `frontend/src/lib/features/bank-import/bank-profile.api.ts` + `.types.ts` :
  - 5 fonctions CRUD + types `BankProfile`, `NewBankProfile`, `ColumnMapping`.
- [ ] **T6.5** `frontend/src/lib/features/bank-import/BankImportUpload.svelte` extension :
  - Détection MIME / extension côté JS pour preview UX (sans parser CSV côté frontend).
  - Si CSV : flow `bankProfileId` ou auto-match via preview API.
  - Affichage erreurs `BANK_CSV_PARTIAL_FAILURE` : panneau listant les lignes (ScrollArea pattern existant).
- [ ] **T6.6** Routes :
  - `frontend/src/routes/(app)/bank-import/profiles/+page.svelte` (list)
  - `frontend/src/routes/(app)/bank-import/profiles/new/+page.svelte` (create)
  - `frontend/src/routes/(app)/bank-import/profiles/[id]/+page.svelte` (edit)
- [ ] **T6.7** Tests Vitest 5+ : api fns CRUD, BankProfileForm validation, BankImportUpload CSV branch.

### T7. i18n (AC #22)

- [ ] **T7.1** Ajouter clés dans `frontend/src/lib/shared/i18n/locales/{fr,de,it,en-CH}/bank-import.json` (extension du fichier 8-1b) :
  - `bank-profile-labels-*` (~15 clés : bank-name, filename-pattern, encoding, date-format, separators, decimal-separator, header-row-count, column-mapping-date, column-mapping-amount, column-mapping-debit, column-mapping-credit, column-mapping-reference, column-mapping-details, column-mapping-counterparty, save-profile, delete-profile)
  - `bank-profile-errors-*` (~8 clés : bank-name-required, bank-name-duplicate, column-mapping-xor-violation, date-format-invalid, regex-invalid, separators-equal, header-row-count-out-of-range, profile-not-found)
  - `bank-csv-errors-*` (~10 clés : invalid-date, invalid-amount, ambiguous-debit-credit, empty-mandatory-field, row-too-short, unsupported-encoding, unsupported-format, no-profile-match, partial-failure, profile-duplicate)
  - `bank-csv-warnings-*` (~3 clés : profile-auto-matched, multiple-profile-matches, encoding-mismatch-with-profile)
- [ ] **T7.2** `npm run lint-i18n-ownership` doit passer. La feature `bank-profile` et `bank-csv` sont déjà supportées par le pattern multi-segment 8-1b (cf. `keyBelongsToFeature` enhancement).

### T8. Tests E2E Playwright (AC #1, #5, #7, #9, #10, #23)

- [ ] **T8.1** `frontend/tests/e2e/bank-csv-import.spec.ts` (nouveau fichier) — 7 scénarios :
  1. `imports a CSV UTF-8 file end-to-end` (AC #1)
  2. `creates a bank profile via wizard then imports` (AC #6 + #7)
  3. `auto-applies bank profile on filename match` (AC #7)
  4. `shows BANK_CSV_NO_PROFILE_MATCH when no profile matches` (AC #9)
  5. `displays partial failure error panel with line numbers` (AC #10)
  6. `rejects UTF-16 with unsupported encoding error` (AC #5)
  7. `accessibility — profile pages axe scan zero violations` (AC #23)
- [ ] **T8.2** Fixtures Playwright `frontend/tests/e2e/fixtures/` : 4 CSV (`csv_utf8_bom_minimal.csv`, `csv_iso_8859_1_swiss.csv`, `csv_partial_failure.csv`, `csv_utf16_le.csv`).
- [ ] **T8.3** Helper `seedTestState('with-bank-profile')` dans `frontend/tests/e2e/helpers/test-state.ts` (extension du seeder existant) — crée company + bank_account + 1 bank_profile pour les scénarios qui en ont besoin.

### T9. Sync sprint-status + README

- [ ] **T9.1** `_bmad-output/implementation-artifacts/sprint-status.yaml` :
  - Transition `8-2-import-csv-multi-encodage-profils-banque: backlog → ready-for-dev` (déjà faite par cette spec via Step 6 du workflow create-story).
  - Au merge : transition `ready-for-dev → done` (post-cycle review CLAUDE.md).
- [ ] **T9.2** `README.md` :
  - Ligne `## Fonctionnalités` : « Import bancaire CAMT.053 — parser + persistance + UI ✓ (CSV multi-banque + réconciliation : *à venir*) » → au merge 8-2 : « Import bancaire CAMT.053 + CSV multi-encodage ✓ (réconciliation : *à venir*) ».
  - Tableau Feuille de route Epic 8 reste 🚧 En cours (4 stories restantes 8-3/8-4/8-5).

## Risque de splitting (CLAUDE.md check)

**Modules touchés par 8-2** :
1. `kesh-import` (nouveau module `csv`)
2. `kesh-db` (migration `bank_profiles` + entités/repos)
3. `kesh-api` (nouvelles routes `bank_profiles` + extension `bank_imports`)
4. `kesh-core` (4 nouvelles variantes `CoreError::BankCsv*` — minimal)
5. `frontend/src/lib/features/bank-import` (extension : 3 nouveaux components + 3 nouvelles routes)
6. `frontend/src/lib/shared/i18n/locales` (extension fichier `bank-import.json` × 4 locales)

**Total : 6 modules**. Au seuil règle CLAUDE.md « splitter si > 5 modules ». Précédent 8-1b unifié à 5-6 modules a converge en 3 passes validate (Sonnet → Haiku → Opus, 20 patches) sans drift.

**Décision : pas de split préventif**. Justifications :
- Frontière naturelle frontend/backend déjà absorbée par 8-1b — 8-2 réutilise les composants drop-zone, page route, api-client, audit log.
- Pas de path dep cargo cassée : `kesh-import::csv` est ajouté dans la même crate que `camt053`, donc pas de réordonnancement workspace.
- Le module CSV est isolé (zéro coupling avec camt053 sauf re-exports `lib.rs`).
- Les 4 nouvelles variantes `CoreError::BankCsv*` sont minimales (pattern Story 8-1a).

**Trigger de re-évaluation** : si `bmad-create-story validate 8-2` boucle au-delà de 4 passes adversariales sans converger sur 0 finding > LOW, splitter en :
- **8-2a** : parser `kesh-import::csv` + entités/repos `bank_profiles` (T1-T3) — backend autonome, testable via `cargo test`.
- **8-2b** : routes API `bank_profiles` + extension `bank_imports` (T4-T5) — dépend 8-2a.
- **8-2c** : frontend feature + i18n + E2E (T6-T8) — dépend 8-2b.

Cette section est documentée pour décision à la passe validate Pass 4 si applicable.

## Dev Notes

### Pipeline réutilisée 8-1b (à respecter strictement)

**Routes** : `bank_imports.rs` reste l'unique route d'upload. CSV dispatch interne via `detect_import_format`. **Pas de nouvelle route `/api/v1/bank-imports/csv`** — le client upload sur le même endpoint avec le bon `Content-Type` ou extension de fichier.

**Audit log** : pattern atomique 8-1b. Toutes les écritures DB de profile vivent dans une transaction qui inclut l'INSERT audit_log. Pas d'audit log post-commit (race avec rollback).

**Multi-tenant** : KF-002 pattern absolu — `find_by_id_for_company` partout, jamais de `find_by_id` cross-tenant. 404 systématique sur cross-tenant access (jamais 403).

**RBAC** : sub-router pattern :
- `comptable_routes` (write : POST/PUT/DELETE bank-profiles + import bank-imports CSV).
- `authenticated_routes` (read : GET bank-profiles + GET bank-imports).

**Errors** : pattern `AppError` énumération exhaustive avec mapping `IntoResponse`. Ne **pas** réutiliser un variant existant pour un nouveau cas — chaque erreur HTTP distincte = un variant.

**Validation env vars** : `KESH_BANK_IMPORT_MAX_MB` hérité. **Pas de nouvelle var** v0.1.

### API surface 8-1a/8-1b consommée

**`kesh-import` re-exports à utiliser** :
- `kesh_import::parse_csv` (T2.5 sera ajouté).
- `kesh_import::{ImportedStatement, ImportedTransaction, SourceFormat}`.
- `kesh_import::{ImportError, CsvError, CsvLineError, CamtError}` (T2.4 sera ajouté).

**`kesh-core::bank_imports` extensions T4** :
- `validate_csv_profile_signature(profile: &CsvProfile, sample_row: &[String]) -> Result<(), CoreError>` — sanity check post-parse que les indices column_mapping sont valides vs longueur des rows. Optionnel v0.1 si trop coûteux.

**`kesh-db::repositories::bank_imports`** : pas de nouvelle méthode. La méthode `create_with_transactions` existante fonctionne pour CSV (le `source_format` est déjà accepté en string).

### Source tree à toucher

**Backend** :
- `crates/kesh-import/Cargo.toml` (deps csv, encoding_rs, chardetng, regex)
- `crates/kesh-import/src/lib.rs` (re-exports)
- `crates/kesh-import/src/csv/{mod.rs, encoding.rs, parser.rs, profile.rs}` (nouveaux)
- `crates/kesh-import/src/error.rs` (extension `ImportError`/`CsvError`)
- `crates/kesh-import/tests/csv_tests.rs` (nouveau)
- `crates/kesh-import/tests/fixtures/csv/*.csv` (5 fixtures, nouvelles)
- `crates/kesh-db/migrations/{TIMESTAMP}_bank_profiles.sql` (nouveau)
- `crates/kesh-db/src/test_fixtures.rs` (TABLES_TO_TRUNCATE update)
- `crates/kesh-db/src/entities/bank_profile.rs` (nouveau)
- `crates/kesh-db/src/repositories/bank_profiles.rs` (nouveau)
- `crates/kesh-db/src/{lib.rs, entities/mod.rs, repositories/mod.rs}` (re-exports)
- `crates/kesh-db/tests/bank_profiles_test.rs` (nouveau, 8 sqlx::test)
- `crates/kesh-core/src/errors.rs` (4 variantes)
- `crates/kesh-core/src/bank_imports.rs` (helper signature optionnel)
- `crates/kesh-api/src/routes/bank_profiles.rs` (nouveau)
- `crates/kesh-api/src/routes/bank_imports.rs` (extension dispatch)
- `crates/kesh-api/src/errors.rs` (4 variantes)
- `crates/kesh-api/src/router.rs` (mount bank_profiles routes)
- `crates/kesh-api/tests/bank_profiles_e2e.rs` (nouveau, 12 tests)
- `crates/kesh-api/tests/bank_imports_e2e.rs` (8 nouveaux tests CSV)

**Frontend** :
- `frontend/src/lib/features/bank-import/BankProfileForm.svelte` (nouveau)
- `frontend/src/lib/features/bank-import/BankProfileList.svelte` (nouveau)
- `frontend/src/lib/features/bank-import/BankProfileSelector.svelte` (nouveau)
- `frontend/src/lib/features/bank-import/bank-profile.api.ts` (nouveau)
- `frontend/src/lib/features/bank-import/bank-profile.types.ts` (nouveau)
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte` (extension CSV branch)
- `frontend/src/routes/(app)/bank-import/profiles/+page.svelte` (nouveau)
- `frontend/src/routes/(app)/bank-import/profiles/new/+page.svelte` (nouveau)
- `frontend/src/routes/(app)/bank-import/profiles/[id]/+page.svelte` (nouveau)
- `frontend/src/lib/shared/i18n/locales/{fr,de,it,en-CH}/bank-import.json` (extension)
- `frontend/tests/e2e/bank-csv-import.spec.ts` (nouveau, 7 scénarios)
- `frontend/tests/e2e/fixtures/csv_*.csv` (4 fixtures)
- `frontend/tests/e2e/helpers/test-state.ts` (extension `with-bank-profile`)

### Standards de test

**Backend** :
- `kesh-import` : tests unitaires + 12 tests d'intégration (fixtures CSV). Couverture cible : encoding detection (5 cases), profile validation (XOR + chrono + regex + separators), parser happy path + partial failure + value parsing edge cases (apostrophe milliers, virgule decimal, debit_credit_split).
- `kesh-db` : 8 `#[sqlx::test]` (audit log atomicity, multi-tenant isolation, optimistic update, soft delete via cascade).
- `kesh-api` : 12 + 8 = 20 nouveaux E2E HTTP tests `#[sqlx::test]` (RBAC, validation, dispatch CSV vs CAMT, audit log details_json).

**Frontend** :
- Vitest : 5+ tests (api fns, profile form validation client-side, BankImportUpload CSV branch).
- Playwright : 7 scénarios (cf. T8.1).

**Pas de mocks DB** côté backend tests — toujours `#[sqlx::test]` (intégration réelle MariaDB), pattern Story 7-2 + 8-1b.

### Checklist locale avant push

Hérité 8-1b + ajout :

```sh
# Backend
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                              # parallel, sans MariaDB-only
cargo test -p kesh-db --lib test_fixtures           # avec MariaDB up + KESH_TEST_MODE=true
cargo publish --dry-run -p kesh-import              # invariant 8-1a

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership
npm run test:unit
npm run build

# E2E (si modif frontend ou routes)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- bank-csv-import.spec.ts
```

**Leçon CI 8-1b** : si la migration touche la DB, **toujours** lancer `cargo test -p kesh-db --lib test_fixtures` avec MariaDB up. Le garde-fou `truncate_all_inventory_matches_schema` skip silencieusement sans DB.

### Références

- Epic 8 — [`_bmad-output/planning-artifacts/epic-8.md`](../planning-artifacts/epic-8.md) §Story 8-2 + §Risques R3/R4
- PRD — [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md) FR42, FR50, FR51, FR52, FR53, scénario Lisa l.168
- Architecture — [`_bmad-output/planning-artifacts/architecture.md`](../planning-artifacts/architecture.md) §11.5 Data Architecture, §17 Frontières architecturales, §kesh-import publiable
- Story 8-1a — [`8-1a-camt053-parser-only.md`](8-1a-camt053-parser-only.md) (`kesh-import` crate, types autonomes, `From`/`Into` pattern)
- Story 8-1b — [`8-1b-camt053-persistence-ui.md`](8-1b-camt053-persistence-ui.md) (pipeline persistance + UI réutilisée intégralement)
- Spec d'origine 8-1 — [`8-1-import-camt053.md`](8-1-import-camt053.md) `archived-split` (décisions §schéma, §upload-limit, §perf, §api-client)
- KF-002 multi-tenant pattern — Story 6-2 et Story 7-1 (404 cross-tenant)
- KF-008 Playwright data-testid pattern — Story 7-5
- CLAUDE.md `Test Locally First` — règle MariaDB up pour modif migration (codifiée 2026-05-05 post-incident CI 8-1b)

## Dev Agent Record

### Agent Model Used

(à compléter par dev-story)

### Debug Log References

### Completion Notes List

### File List

(à compléter par dev-story — toutes les fichiers créés/modifiés listés ici)

### Change Log

| Date | Action | Auteur |
|------|--------|--------|
| 2026-05-05 | Spec créée par `bmad-create-story 8-2` post-merge PR #69 (8-1a + 8-1b done). Status `backlog` → `ready-for-dev`. Branche `story/8-2-import-csv-multi-encodage-profils-banque` créée depuis main `076ac86`. 26 ACs définis (vs 7 ACs epic-8.md → enrichis avec FR52a-d détection encoding, FR53 auto-match, FR51 strict reject mapping, multi-tenant, RBAC, validation profil, audit log, i18n, a11y, perf). 9 tasks T1-T9. Dépendances upstream 8-1a/8-1b validées. Risque de splitting documenté (6 modules au seuil, pas de split préventif, trigger validate Pass 4 si non-convergence). | Claude (Opus 4.7, bmad-create-story) |
