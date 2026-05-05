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

1. **Migration DB** (T1) — `crates/kesh-db/migrations/2026MMDDHHMMSS_bank_profiles.sql` créant `bank_profiles` (id, company_id, bank_name, column_mapping JSON, date_format, encoding, separator, decimal_separator, header_row_count, created_at, updated_at). Pas de migration ALTER sur `bank_imports`/`bank_transactions` côté DB — la colonne `source_format VARCHAR(32) NOT NULL` n'a pas de CHECK constraint (cf. comment migration 8-1b ligne 17), donc `'CSV'` est insérable. **Mais l'extension Rust est obligatoire** (cf. point 1bis).
1bis. **Extension enums Rust `source_format`** (T2.4 + T2.5 + T5.0) — **point critique non livré par 8-1b** :
    - `kesh-db/src/entities/bank_import.rs` : ajouter variante `BankImportSourceFormat::Csv` avec `as_db_str() = "CSV"` + parser `from_str("CSV")` + supprimer le test `source_format_unknown_rejected` qui vérifie le rejet de "CSV" (héritage 8-1b documenté).
    - `kesh-core/src/bank_imports.rs` : ajouter variante `SourceFormatTag::Csv` avec `as_db_str() = "CSV"` + étendre la fn `from_imported()` pour brancher `kesh_import::SourceFormat::Csv { encoding, profile_name }` → `SourceFormatTag::Csv` (sans erreur). Aujourd'hui `from_imported()` retourne `Err(CoreError::BankImportUnknownVersion("csv"))` sur ce branch — ce qui doit être supprimé.
    - `kesh-api/src/routes/bank_imports.rs::version_to_source_format()` : étendre pour mapper `kesh_import::SourceFormat::Csv { .. }` → `BankImportSourceFormat::Csv`.
2. **Module parser `kesh-import::csv`** (T2) — `crates/kesh-import/src/csv/{mod.rs, encoding.rs, parser.rs, profile.rs}` + tests `csv_tests.rs`. Détection encoding (BOM → `chardetng`), parser via `csv` crate, mapping profil → `ImportedTransaction`. Variantes `CamtError` étendues ou nouveau type `ImportError` avec sous-type `Csv(CsvError)` (cf. §error-types).
3. **Entités + repositories `bank_profiles`** (T3) — `kesh-db/src/entities/bank_profile.rs` + `kesh-db/src/repositories/bank_profiles.rs` (CRUD + helper `find_matching_profiles_for_filename`, retourne `Vec<BankProfile>` pour gérer multi-match).
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

0. **Empty file guard** (priorité 0) — si `bytes.len() == 0` → `422 BANK_CSV_EMPTY_FILE` (cf. §error-types `CsvError::EmptyFile`). Aucun parsing n'est tenté.
1. **BOM check** (priorité 1) — `encoding_rs::Encoding::for_bom(bytes)` (**pas** `&bytes[..3]` — la lib gère elle-même les slices courtes ; le slice `[..3]` panic sur fichiers de 1 ou 2 bytes, ex. BOM tronqué). Si `Some((Encoding::UTF_8, bom_len))` → UTF-8 (skip `bom_len`). Si `Some((UTF_16LE | UTF_16BE, _))` → **422 BANK_CSV_UNSUPPORTED_ENCODING** (non v0.1).
2. **Sans BOM** — heuristique en deux passes :
   - **Passe 1** : tenter décodage UTF-8 strict via `std::str::from_utf8`. Si OK → UTF-8 (couvre ASCII pur AC #4 et UTF-8 sans BOM).
   - **Passe 2** : si UTF-8 fail → `chardetng::EncodingDetector::feed(&bytes[..bytes.len().min(1024)], true)` puis `detector.guess(None, true)`. Cohérence : on échantillonne **1024 bytes max** (texte de la spec), pas le fichier entier — perf prévisible sur fichiers 50 MB. Si verdict `windows-1252` ou `iso-8859-1` → ISO-8859-1. Sinon (autre encoding détecté) → `422 BANK_CSV_UNSUPPORTED_ENCODING`.
   - **Garde-fou petit fichier** : si `bytes.len() < 64` ET passe 1 a échoué, `chardetng` est probabiliste peu fiable → `422 BANK_CSV_UNSUPPORTED_ENCODING` avec `details.detected = null` (refus explicite plutôt que faux positif silencieux).

**Override utilisateur via profil** : le profil stocke `encoding` (`Option<String>`, default `null` = auto-détect).

- **Pas d'override silencieux** (révision Pass 1 H5 mojibake) : si `profile.encoding` est non-null ET diverge du résultat de l'algo de détection (ex. profil dit `UTF-8` mais BOM détecte `UTF-16LE`, ou profil dit `ISO-8859-1` mais bytes sont valides UTF-8 avec accents `0xC3 0xA9`), comportement strict :
  - **`POST /preview`** : `200 OK` + warning `bank_csv_encoding_mismatch` (UI peut afficher) avec `details.profileEncoding` + `details.detectedEncoding`. La preview utilise **l'encoding détecté** (pas le profil) pour éviter le mojibake silencieux à l'affichage.
  - **`POST /bank-imports` final** : `422 BANK_CSV_ENCODING_MISMATCH` sauf si form contient `confirmEncodingMismatch=true` → `201 Created` + audit log `bank_import.created_with_encoding_mismatch`. Le décodage utilise alors la valeur du profil (utilisateur est explicitement responsable du mojibake potentiel).

Cas extrême (BOM UTF-16 + `encoding=ISO-8859-1` profil ou vice-versa) → toujours reject `422 BANK_CSV_UNSUPPORTED_ENCODING` (pas de bypass car UTF-16 est non v0.1, peu importe le profil).

**Anti-bypass `confirmEncodingMismatch=true` sans preview préalable** (Pass 2 H'3) :

Le pattern preview → confirm est volontaire pour que l'utilisateur **voie** le mojibake potentiel avant d'autoriser l'import. Risque de bypass : un client malveillant ou un script peut envoyer directement `POST /bank-imports` avec `confirmEncodingMismatch=true` **sans avoir consulté le preview**, contournant le warning UI.

**Défense backend (mandatory)** : la détection d'encoding (BOM check + heuristique UTF-8/chardetng) est **systématiquement re-exécutée** dans `POST /bank-imports`, indépendamment de tout flag client. Le flag `confirmEncodingMismatch=true` est consulté **uniquement** quand le serveur a effectivement détecté un mismatch (i.e., `detect_encoding(bytes) != profile.encoding`). Si pas de mismatch détecté, le flag est ignoré (pas d'effet de bord).

Conséquence : le pattern attaquant `POST /bank-imports + confirmEncodingMismatch=true` sur un fichier dont l'encoding détecté **matche** le profil ne peut **pas** déclencher de bypass — il n'y a juste pas de mismatch à confirmer. Et sur un fichier où ça mismatche, l'utilisateur a explicitement coché la confirmation côté UI (ou son équivalent côté script) et accepte le risque mojibake — c'est par design.

**Pas de session token** côté backend (KISS v0.1) : la cohérence est garantie par la re-détection systématique. Test E2E HTTP `post_csv_with_confirm_flag_but_no_real_mismatch_ignores_flag` (assert audit log `bank_import.created` standard, pas `_with_encoding_mismatch`).

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
    pub date: usize,                                  // 0-indexed column, REQUIRED
    pub amount: Option<usize>,                        // XOR avec debit_credit_split (Pass 1 H6)
    pub debit_credit_split: Option<(usize, usize)>,   // (debit_col, credit_col) — XOR avec amount
    pub reference: Option<usize>,
    pub details: Option<usize>,
    pub counterparty: Option<usize>,
}
```

**Important sérialisation** : `amount` est `Option<usize>` (pas `usize`) précisément parce que la contrainte XOR est exprimée par `Some`/`None`. Un payload de création peut envoyer `{"date": 0, "debit_credit_split": [3, 4], ...}` sans champ `amount` (deserialize → `None`). Côté DB, le JSON sérialisé n'inclut pas le champ `amount` quand `None` (`#[serde(skip_serializing_if = "Option::is_none")]`).

**Validation côté API** (T2.2 + T4.3) :
- `bank_name` non-vide, 1..=100 chars
- `column_mapping` : exactement un de `amount.is_some()` XOR `debit_credit_split.is_some()` (XOR strict — fail si les deux ou aucun)
- **Unicité indices** (Pass 1 M13) : tous les indices spécifiés (`date`, `amount`, `reference`, `details`, `counterparty`, et les 2 valeurs de `debit_credit_split`) doivent être deux à deux distincts. Erreur `BankCsvProfileValidation("column_mapping.{a} ({i}) conflicts with column_mapping.{b} ({i})")` sinon. Évite les diagnostics cryptiques (« montant invalide : 2026-01-15 ») quand `date` et `amount` pointent la même colonne.
- `date_format` : valider via `chrono::format::strftime::StrftimeItems::new(fmt).all(|i| !matches!(i, Item::Error))` au create/update
- `field_separator` ∈ `{',', ';', '\t'}` v0.1 (pas de full custom)
- `decimal_separator` ∈ `{'.', ','}`
- `field_separator != decimal_separator` (Pass 2 M'6 justification) — la contrainte prévient l'ambiguïté de parsing : si les deux étaient identiques (ex. `,`), la ligne `"1,234,56"` se splitterait en `["1", "234", "56"]` (3 colonnes) au lieu d'être interprétée comme le montant unique `1234.56`. Le crate `csv` gère les quotes (`"1,234,56"` quoted = 1 champ), mais peu d'exporters bancaires émettent des quotes systématiques sur les amounts non-string. Donc rejet au save profil = défense au plus tôt, plus utile qu'un message d'erreur cryptique au parse. Cf. AC #15b.
- `header_row_count` ∈ `0..=5`
- `filename_pattern` (si Some) : regex valide via `regex::Regex::new(...)`, longueur ≤ 200 chars

**Priorité au parse en cas de DB corrompue** (Pass 1 M16) : la validation API est la première ligne de défense, mais une DB compromise ou une migration manuelle peut créer un `column_mapping` JSON avec **les deux** champs `amount` ET `debit_credit_split` non-null. Au moment du parse (T2.3), priorité explicite : `if debit_credit_split.is_some() { utiliser split } else if amount.is_some() { utiliser amount } else { CsvError::ProfileMisconfigured("column_mapping.amount + debit_credit_split tous deux null en DB") }`. Pas de fallback silencieux. Test unit `parses_corrupt_profile_with_both_amount_and_split_prefers_split` + log `tracing::warn!`.

**Stockage `column_mapping` en DB** : type `JSON` MariaDB (10.2+ supporte JSON natif via `JSON_VALID` CHECK). Sérialisation Rust via `serde_json::to_string` côté insert/update, désérialisation via `serde_json::from_str` côté SELECT. Pas de `sqlx::types::Json` direct (l'integration sqlx-mysql + JSON natif a des subtilités v0.8.6 — préférer string serialization explicite, pattern Story 7-2 pour `vat_rates.metadata`).

**UNIQUE constraint** : `(company_id, bank_name)` — un seul profil par banque par tenant.

#### §profile-matching

À l'upload d'un fichier CSV, ordre de résolution du profil :
1. **bankProfileId explicite** (champ multipart `bankProfileId=N`) — utilisateur a sélectionné un profil dans l'UI → résoudre via `repository::find_by_id_for_company(company_id, profile_id)`.
   - **1a** — profil trouvé pour cette company → utiliser tel quel.
   - **1b** — profil inexistant OU appartenant à une autre company (Pass 1 H8) → `404 BANK_CSV_NO_PROFILE_MATCH` (jamais 403, pattern KF-002). **Pas** de fallback auto-match pour ne pas masquer le bug client (UI a envoyé un ID périmé). Cf. AC #11bis pour test confused-deputy explicite `bankProfileId = profil_company_B + bankAccountId = own`.
2. **Auto-match par filename** — si `bankProfileId` absent ET fichier détecté CSV : `SELECT FROM bank_profiles WHERE company_id = ? AND filename_pattern IS NOT NULL` puis tester chaque regex compilée sur le nom du fichier. Premier match (ORDER BY `updated_at DESC`) → applique. Plusieurs matches → utilise le plus récent + warning `bank_csv_multiple_profile_matches`. Le scoping `company_id` dans le SELECT garantit que les profils d'un autre tenant ne sont jamais consultés.

   **Sensibilité à la casse (Pass 2 M'5)** : le matching regex est **case-sensitive par défaut** (standard `regex` crate Rust). Un profil avec `filename_pattern = "^export_.*\\.csv$"` ne matchera **pas** `Export_20260315.csv` (E majuscule). L'utilisateur peut activer le case-insensitive via le flag inline regex `(?i)`, ex. `filename_pattern = "(?i)^export_.*\\.csv$"`. UI doit afficher cette information dans l'aide contextuelle de `BankProfileForm.svelte` (placeholder + tooltip i18n `bank-profile-labels-filename-pattern-help`).
3. **Aucun match** → preview retourne `404 BANK_CSV_NO_PROFILE_MATCH` avec `details.available_profiles: [{id, bank_name}]` (cap 50 entrées par réponse, cf. Pass 1 M11 anti-amplification). **Pas** de fallback inline UI v0.1 (création de profil = flow séparé).

**Race condition profile delete + import concurrent** (Pass 2 H'2 + Pass 3 M''4 séquencement) :

Pendant l'exécution de `POST /bank-imports`, un autre utilisateur de la même company peut supprimer le profil que l'on vient de résoudre (race entre l'étape de résolution et l'INSERT bank_imports). Le code doit s'en prémunir :

- **Séquencement Pass 3 M''4 — Interprétation A retenue (parse dans la transaction)** :
  ```text
  pool.begin() → tx
  ├─ find_by_id_for_company(&mut tx, ...)  // résolution profil étape 1/2/3
  ├─ parse_csv(bytes, &profile)            // parse CPU-bound, dans tx
  ├─ create_with_transactions(&mut tx, ...) // INSERT bank_imports + transactions
  ├─ audit_log_insert(&mut tx, ...)         // INSERT audit_log
  └─ tx.commit()
  ```
  **Conséquence acceptée v0.1** : la transaction MariaDB reste ouverte pendant tout le parse (qui peut prendre quelques secondes sur 50 MB de CSV avec 100k lignes). Avantage : un seul read du profil garantit cohérence parse/persist sur exactement la même version de `column_mapping`. Pas besoin de version optimiste sur `bank_profiles`. Pas de CHECK SQL FOR UPDATE explicite — le row lock implicite sur `bank_profiles.id` (lecture transactionnelle) suffit pour bloquer DELETE/UPDATE concurrents.
  **Limitation v0.1 → cf. L7 Limitations connues** : SLA latence d'autres requêtes touchant `bank_profiles` peut se dégrader sur uploads > 50 MB. Acceptable v0.1 (uploads asynchrones rares) ; v0.2 si observé en prod, pivoter vers Interprétation B (pre-parse hors tx + revalider `profile.version` dans la tx).
- **Pattern transaction-bound** : la résolution du profil (étapes 1, 2, ou 3 ci-dessus) DOIT s'effectuer **dans la même transaction DB** que l'INSERT `bank_imports` + `bank_transactions` + `audit_log`. Concrètement : le handler obtient une `&mut Transaction<'_, MySql>` (depuis `pool.begin().await`), passe cette transaction à `find_by_id_for_company` ou `find_matching_profiles_for_filename`, puis enchaîne les `parse_csv` + `create_with_transactions` + `audit_log_insert` sans relâcher le commit. Tout DELETE concurrent attendrait le commit.
- **Helper repo signature** : `find_by_id_for_company(executor: impl sqlx::Executor<'_, Database = MySql>, ...)` accepte à la fois `&MySqlPool` et `&mut Transaction` via le trait `Executor` (pattern Story 7-1 KF-002).
- **Validation post-fetch** : si `find_by_id_for_company(&mut tx, company_id, profile_id)` retourne `None` à l'intérieur de la transaction, retourner `404 BANK_CSV_NO_PROFILE_MATCH` (le profil n'existe plus ou n'a jamais existé). **Jamais** d'`unwrap()` ou d'indexing `[0]` sans check.
- **Test E2E** : `concurrent_profile_delete_during_upload_aborts_with_404` — démarrer un import CSV en thread A (latence simulée via sleep dans le handler de test), lancer DELETE profile en thread B, attendre commit thread A → vérifier que A a soit committé avec succès (B attend le lock), soit reçu `404` (si B a réussi à passer entre la fin de la transaction A et le check). Le pattern transaction garantit la cohérence : on n'observe **jamais** un import committé avec un `bank_profile_id` inexistant (vu de l'extérieur).

**Important** : le profil n'est consulté QUE pour les CSV. Les CAMT.053 (détectés via MIME `application/xml` ou ext `.xml`) bypassent complètement ce path.

#### §csv-detection

Détection CSV vs CAMT.053 sur le multipart upload, **avant** decoding (raw bytes) :
- **Extension fichier** (priorité 1) : `.xml` → CAMT.053. `.csv` ou `.txt` → CSV. Match exact lowercase, après `truncate_to_byte_len` héritage 8-1b.
- **MIME type** (priorité 2 — utilisée si extension absente ou ambiguë) : `application/xml` / `text/xml` → CAMT. `text/csv` / `application/vnd.ms-excel` / `text/plain` → CSV.
- **Sniff content sur raw bytes** (priorité 3 — utilisée si extension+MIME ambigus, Pass 1 M19) : opère sur les premiers 256 raw bytes (**pas decoded** — au moment de la détection format, l'encoding n'est pas encore connu, et le profil non plus). La détection XML cherche `<?xml` ou `<Document` ou `BkToCstmrStmt` en ASCII brut. Robuste aux encodages non-UTF-8 (les séquences ASCII sont préservées dans ISO-8859-x et UTF-8). Sniff CSV : présence d'un séparateur courant (`,;\t`) avec moins de 5% de bytes hors `[\x09\x0A\x0D\x20-\x7E\xA0-\xFF]` (heuristique simple).
- **Aucun match** → `415 BANK_IMPORT_UNSUPPORTED_FORMAT` (nouveau, §error-types). Body de réponse `{ code, message, details: { extension, mime, detected_marker } }` pour aider le diagnostic UI. **Cf. AC #5bis pour les 3 tests E2E HTTP couvrant chaque priorité.**

**Note priorité** : si extension `.xml` dit CAMT mais MIME dit `text/csv`, l'extension gagne (priorité 1). Cas concret : fichier `releve.xml` exporté par un outil mal configuré qui met `Content-Type: text/csv` — extension wins, parse CAMT.053 (qui rejettera proprement si le contenu n'est pas du XML valide).

#### §csv-parser

**Lib** : `csv` crate (BurntSushi) 1.3.x.

**Lecture** :
- `csv::ReaderBuilder::new().delimiter(profile.field_separator as u8).has_headers(profile.header_row_count > 0).from_reader(decoded_bytes.as_bytes())`.
- **Skip header additionnel** (Pass 1 H4) : `let extra_skip = profile.header_row_count.saturating_sub(1) as usize;` (saturating, pas naïf — `header_row_count = 0` donne `extra_skip = 0`, `header_row_count = 1` donne `0`, `header_row_count = 5` donne `4`). Skipper `extra_skip` rows additionnels via `reader.records().take(extra_skip).count()` avant l'itération de données.
- **Cas `header_row_count = 0`** : `has_headers(false)` → le crate ne skip rien. `extra_skip = 0` → aucun skip additionnel. Toutes les lignes sont des données. Test unit dédié `parses_csv_with_zero_header_rows`.

**Line numbers** (Pass 1 M9) :
- **NE PAS utiliser** `iter().enumerate()` — rompt avec les champs CSV multi-lignes quoted (un champ `"Loyer\nmensuel"` est une cellule mais 2 lignes physiques).
- Utiliser `csv::ReaderBuilder::new().has_headers(...).from_reader(...).records()` puis pour chaque `record`, appeler `record.position()` qui retourne `Option<&Position>` avec `position.line()` (1-based, ligne réelle du fichier source).
- **Formule de stockage** : `line_in_file = position.line()` (déjà absolue dans le fichier, incluant les lignes embarquées dans les champs quoted ET le header). C'est le numéro qu'on retourne à l'utilisateur dans `CsvLineError.line`. Test unit `line_numbers_are_file_absolute_with_multi_header` couvrant `header_row_count=2` + erreur sur 5e ligne de données → `line = 7`.

**Empty file / no transactions guard** (Pass 1 M14) :
- Si après itération complète `transactions.is_empty()` (fichier de 0 byte est déjà rejeté par §encoding-detection priorité 0, mais cas dérivés : header-only, toutes lignes en commentaire, etc.) → `CsvError::EmptyFile { reason: "no data rows after header skip" }` → `422 BANK_CSV_EMPTY_FILE`.
- Évite un import `201 Created` avec `transaction_count = 0` qui consomme un `file_hash` et bloque le réimport légitime.

**Calcul `period_from` / `period_to`** (Pass 1 H2) :
- Le DB schema 8-1b a `period_from DATE NOT NULL` et `period_to DATE NOT NULL`. Pour CAMT.053 ces dates viennent de `<FrToDt>`. Pour CSV, **aucune source globale** — le parser doit les calculer depuis les transactions.
- Algorithme : `period_from = min(transactions.iter().map(|t| t.booking_date))` et `period_to = max(...)`. Si `transactions.is_empty()` → déjà rejeté par EmptyFile guard ci-dessus, donc ces unwraps sont safe.
- Le `ImportedStatement` retourné par `parse_csv()` pour CSV a donc `period_from = min(booking_dates)` et `period_to = max(booking_dates)` (pas `Option`, comme pour CAMT).
- Test unit `csv_period_from_to_calculated_from_transactions` couvre 3 cas : 1 transaction (period_from == period_to), N transactions ordre quelconque (min/max), une seule date répétée.

**Mapping value parsing** :
- **Date** : `chrono::NaiveDate::parse_from_str(value, &profile.date_format)`. Erreur → `CsvError::InvalidDate { line, value, format }`.
- **Amount** :
  - Si `debit_credit_split.is_some()` (priorité parse cf. §profile-model M16) : parse les deux colonnes.
    - `debit non-empty + credit empty` → amount = `-debit` (négatif).
    - `debit empty + credit non-empty` → amount = `+credit` (positif).
    - `debit empty + credit empty` → **`CsvError::EmptyMandatoryField { line, field: "amount" }`** (Pass 1 M10 : ne PAS retourner `Decimal::ZERO` silencieusement — bloque l'import au lieu de persister une transaction de 0 CHF cryptique).
    - `debit non-empty + credit non-empty` → `CsvError::AmbiguousDebitCredit { line }`.
  - Sinon (`amount.is_some()` après XOR) : parse `amount` directement (signe préservé). `decimal_separator=','` → remplace `,` par `.` avant parse. Strip apostrophe (séparateur milliers suisse `1'234.56` → `1234.56`).
  - **Zéro non-blocking** : un `amount = "0.00"` ou `"0,00"` parsé explicitement par l'utilisateur est valide (transaction d'annulation, écriture de mémoire). Seul le cas `debit + credit tous deux empty` rejette.
  - Lib : `rust_decimal::Decimal::from_str_exact` (rejection NaN/infinity/scientific notation, contrairement à `from_str`).
- **Reference** : `Option<String>` (cf. `kesh_import::ImportedTransaction.reference: Option<String>`), trim, empty → `None`.
- **Details** (Pass 2 M'4 drift) : `String` (pas `Option`) — alignement avec `kesh_import::ImportedTransaction.details: String` ET schéma `bank_transactions.details TEXT NOT NULL` 8-1b. Trim, empty → `String::new()` (chaîne vide acceptable, pas NULL). Le DB CHECK constraint accepte `''` (longueur 0 OK).
- **Counterparty (iban + name)** : `Option<String>` (cf. `kesh_import::ImportedTransaction.counterparty_iban` et `counterparty_name`), trim, empty → `None`. CSV v0.1 n'extrait que `counterparty_name` (les CSV bancaires n'exposent généralement pas l'IBAN de contrepartie) : `counterparty_iban = None` toujours pour CSV.
- **Currency** : non lisible depuis CSV v0.1, **assumé CHF**. Validation `validate_currency_supported_v0_1` skip pour CSV (currency = "CHF" forced dans `ImportedStatement.currency`). Document explicit dans Dev Notes.
- **booking_date == value_date** : pas de distinction CSV v0.1. Le helper `from_imported` côté `kesh-core` accepte déjà `booking_date == value_date`.

#### §error-types

**Choix** : nouveau type `ImportError` dans `kesh-import::error.rs` qui wrap `CamtError` ET un nouveau `CsvError` :

```rust
// kesh-import/src/error.rs (extension)
pub enum ImportError {
    Camt(CamtError),
    Csv(CsvError),
}

pub enum CsvError {
    EmptyFile { reason: String },                                // Pass 1 M14 — fichier 0 byte ou 0 transactions après header skip
    UnsupportedEncoding { detected: Option<String> },
    EncodingMismatch { profile: String, detected: String },      // Pass 1 H5 — non-bloquant en preview, bloquant en final sans confirm
    DecodingFailed { encoding: String, byte_offset: usize },
    MissingHeader,
    InvalidDate { line: usize, value: String, format: String },
    InvalidAmount { line: usize, value: String },
    AmbiguousDebitCredit { line: usize },                        // debit ET credit non-empty sur même ligne
    EmptyMandatoryField { line: usize, field: &'static str },    // ex. amount empty quand debit+credit tous deux empty
    RowTooShort { line: usize, expected_cols: usize, got: usize },
    ProfileMisconfigured(String),                                // Pass 1 M16 — DB JSON corrompu (amount + debit_credit_split tous deux null/présents) ou indices hors-borne
    PartialFailure { errors: Vec<CsvLineError>, total_errors: usize, truncated: bool }, // Pass 1 H7 + Pass 2 H'1 cap
    Io(String),
}

/// Cap anti-DoS sur le Vec d'erreurs (Pass 2 H'1) : au-delà de cette
/// limite, le parser arrête la collecte et retourne `truncated: true`
/// avec `total_errors` qui reflète le compteur full (mais `errors`
/// limité à 100 entrées).
pub const MAX_CSV_LINE_ERRORS: usize = 100;

/// Cap anti-DoS sur la taille de la valeur fautive stockée dans
/// `CsvLineError.value` (Pass 3 M''3). Sans ce cap, un attaquant peut
/// uploader un CSV avec 100 lignes invalides où chaque cellule
/// `amount` fait 1 MB → `errors: Vec<CsvLineError>` ferait 100 MB de
/// strings malgré le cap H'1 sur le **nombre** d'erreurs. Truncation
/// UTF-8-aware via `s.chars().take(MAX_CSV_LINE_ERROR_VALUE_CHARS)
/// .collect::<String>()` (pas `s[..N]` qui panique sur boundary
/// non-ASCII), suffixée d'`…` (un seul char) si tronquée.
pub const MAX_CSV_LINE_ERROR_VALUE_CHARS: usize = 100;

/// Erreur ligne-par-ligne pour `PartialFailure`. (Pass 1 H7)
pub struct CsvLineError {
    pub line: usize,                  // numéro absolu fichier (cf. §csv-parser line numbers)
    pub code: CsvLineErrorCode,       // discriminant pour mapping i18n + tri UI
    pub value: Option<String>,        // valeur fautive tronquée à `MAX_CSV_LINE_ERROR_VALUE_CHARS` (100 chars) UTF-8-aware + suffix `…` si dépassement (Pass 3 M''3 anti-DoS)
    pub message_i18n_key: String,     // clé i18n frontend, ex. "bank-csv-errors-invalid-date"
}

pub enum CsvLineErrorCode {
    InvalidDate,
    InvalidAmount,
    AmbiguousDebitCredit,
    EmptyMandatoryField,
    RowTooShort,
}
```

**Impact** : `kesh-import::lib.rs` re-exporte `pub use error::{ImportError, CamtError, CsvError, CsvLineError, CsvLineErrorCode}`. Les types `From<CamtError> for ImportError` et `From<CsvError> for ImportError` permettent la propagation `?`.

`kesh-core::errors::CoreError` étendu avec :
- `BankCsvProfileNotFound`
- `BankCsvUnsupportedEncoding(Option<String>)` (encoding détecté pour diag, `None` si fichier trop court < 64 bytes)
- `BankCsvEncodingMismatch { profile: String, detected: String }` — Pass 1 H5
- `BankCsvParsePartialFailure(Vec<CsvLineError>)` — wrap pour rejet partiel structuré FR51
- `BankCsvProfileValidation(String)` (msg de validation profil)
- `BankCsvProfileMisconfigured(String)` — DB corrompue (cf. §profile-model M16)
- `BankCsvEmptyFile`

Côté `kesh-api::errors::AppError` (cf. T5.0 pour la subtask de déclaration) : **9 nouvelles variantes** (Pass 1 M6 — liste consolidée canonique, exhaustive ; T4.2 et T5 référencent cette liste plutôt que de la redupliquer) :

| Variante AppError | HTTP | Code | Origin (CoreError ou direct) |
|---|---|---|---|
| `BankCsvProfileNotFound` | 404 | `BANK_CSV_NO_PROFILE_MATCH` | `CoreError::BankCsvProfileNotFound` (1a/1b/3 §profile-matching) |
| `BankCsvUnsupportedEncoding` | 422 | `BANK_CSV_UNSUPPORTED_ENCODING` | `CoreError::BankCsvUnsupportedEncoding` (UTF-16, fichier <64 bytes, encoding inconnu) |
| `BankCsvEncodingMismatch` | 422 | `BANK_CSV_ENCODING_MISMATCH` | `CoreError::BankCsvEncodingMismatch` (final sans `confirmEncodingMismatch=true`) |
| `BankCsvParsePartialFailure` | 422 | `BANK_CSV_PARTIAL_FAILURE` | `CoreError::BankCsvParsePartialFailure` (FR51 strict reject) |
| `BankCsvProfileValidation` | 422 | `BANK_CSV_PROFILE_INVALID` | direct (handler validation pré-repo) |
| `BankCsvProfileDuplicate` | 409 | `BANK_CSV_PROFILE_DUPLICATE` | mapping SQL 1062 sur `uq_bank_profiles_company_name` |
| `BankCsvProfileMisconfigured` | 500 | `BANK_CSV_PROFILE_MISCONFIGURED` | `CoreError::BankCsvProfileMisconfigured` (corrupted DB JSON) |
| `BankCsvEmptyFile` | 422 | `BANK_CSV_EMPTY_FILE` | `CoreError::BankCsvEmptyFile` |
| `BankImportUnsupportedFormat` | 415 | `BANK_IMPORT_UNSUPPORTED_FORMAT` | direct (helper `detect_import_format`) |

Payload `BankCsvParsePartialFailure` (Pass 2 H'1 — cap 100 + truncation flag) :
```json
{
  "code": "BANK_CSV_PARTIAL_FAILURE",
  "message": "...",
  "details": {
    "lines": [
      {"line": 7, "code": "INVALID_DATE", "value": "32.13.2026", "message_i18n_key": "bank-csv-errors-invalid-date"},
      {"line": 12, "code": "INVALID_AMOUNT", "value": "abc", "message_i18n_key": "bank-csv-errors-invalid-amount"},
      {"line": 18, "code": "ROW_TOO_SHORT", "value": null, "message_i18n_key": "bank-csv-errors-row-too-short"}
    ],
    "total_errors": 3,
    "truncated": false
  }
}
```

**Cas truncation** (>= 100 erreurs) :
```json
{
  "code": "BANK_CSV_PARTIAL_FAILURE",
  "message": "...",
  "details": {
    "lines": [/* 100 entrées max, ordre fichier */],
    "total_errors": 5847,
    "truncated": true
  }
}
```

UI doit afficher un badge « 5747 erreurs supplémentaires non listées » quand `truncated: true`. Évite OOM serveur (10k erreurs × 200 bytes JSON ≈ 2 MB par requête × N concurrent = DoS mémoire trivial).

#### §preview-csv-response-shape

(Pass 1 M7) — La réponse `POST /preview` pour CSV partage la struct `BankImportPreviewResponse` (héritée 8-1b) avec ces différences :

```jsonc
{
  "selectedStatement": {
    "accountIban": null,                   // CSV n'a pas d'IBAN (CHF assumé, pas d'extraction depuis fichier)
    "currency": "CHF",                     // forcé pour CSV v0.1
    "periodFrom": "2026-01-15",            // calculé min(booking_dates), cf. §csv-parser
    "periodTo": "2026-01-31",              // calculé max(booking_dates)
    "openingBalance": null,                // CSV n'expose pas opening (vs CAMT.053 qui a <Bal>)
    "closingBalance": null,                // idem
    "transactionCount": 27
  },
  "transactions": [...],                   // Vec<ImportedTransaction> identique à CAMT
  "ignoredStatements": [],                 // toujours vide pour CSV (single-statement par fichier)
  "warnings": ["bank_csv_profile_auto_matched"],   // discriminé selon le path
  "appliedProfile": {                      // **nouveau pour CSV**, null pour CAMT
    "id": 42,
    "bankName": "UBS"
  },
  "sourceFormat": "CSV"                    // **nouveau** pour disambiguer côté frontend
}
```

Côté Rust, étendre `BankImportPreviewResponse` (kesh-api) avec :
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub applied_profile: Option<AppliedProfileSummary>,
pub source_format: String,  // "CAMT053_V04" | "CAMT053_V08" | "CSV"
```

Pour CAMT.053, `applied_profile = None` (pas de breaking change frontend grâce à `skip_serializing_if`). `source_format` est nouveau : impose un patch frontend mineur 8-1b (rétrocompat acceptée car post-merge ≤ 24h, pas de client externe).

#### §partial-failure-mapping

**Décision FR51 v0.1 — strict reject** : si **N'IMPORTE QUELLE** ligne du CSV échoue au parsing, l'import entier est rejeté (`422 BANK_CSV_PARTIAL_FAILURE`). Le payload de réponse liste **toutes** les lignes en erreur avec `{line, code, value, message_i18n_key}` pour l'UI affiche un panneau « 12 lignes en erreur ». Pas de partial commit en v0.1.

**Justification** : un partial commit nécessite (a) flow de re-import des lignes corrigées, (b) gestion de l'unicité fichier-hash quand le fichier corrigé a un nouveau hash, (c) UI pour éditer les lignes en erreur inline. Tous ces points sont reportés Story 8-3 (rejet partiel + dédup ligne par ligne).

**Scénario Lisa adapté 8-2 (Pass 1 L8)** : Lisa uploade un fichier CSV de 15 lignes, dont 3 invalides (date malformée). Comportement 8-2 : `422 BANK_CSV_PARTIAL_FAILURE` listant les 3 lignes. Lisa corrige son profil banque (ou modifie le fichier source). Elle réimporte le **fichier complet corrigé** (les 15 lignes). Comme le strict-reject 8-2 garantit qu'aucun import partiel n'a eu lieu, le hash-doublon (`(company_id, file_hash) UNIQUE` 8-1b) ne bloque pas le réimport (le hash du fichier corrigé est nouveau, distinct du fichier d'origine). En Story 8-3, Lisa pourra réimporter uniquement le delta (3 lignes corrigées) avec un détecteur ligne-par-ligne. Le scénario PRD original (« réimporte les 3 lignes manquantes ») correspond donc à **8-3**, pas à **8-2** — la spec 8-2 livre la moitié infrastructure (listing détaillé), 8-3 livre l'autre moitié (re-import partiel + dédup).

#### §upload-limit

Hérité 8-1b : `KESH_BANK_IMPORT_MAX_MB` (default 10, range [1, 100]) appliqué via `DefaultBodyLimit` sur le sub-router. **Aucune nouvelle env var pour 8-2**.

#### §multipart-guards

(Pass 1 M8) — `parse_multipart` (handler `bank_imports.rs`) doit appliquer des guards anti-duplication pour les nouveaux champs CSV, suivant strictement le pattern 8-1b qui guarde déjà `file` et `bankAccountId` :

```rust
// Pseudo-code dans parse_multipart()
let mut bank_profile_id: Option<i64> = None;
let mut confirm_encoding_mismatch: Option<bool> = None;

while let Some(field) = multipart.next_field().await? {
    match field.name() {
        // ... existant 8-1b: file, bankAccountId, confirmBalanceMismatch ...
        Some("bankProfileId") => {
            if bank_profile_id.is_some() {
                return Err(AppError::Validation("Champ 'bankProfileId' dupliqué dans multipart"));
            }
            let raw = field.text().await?;
            let id: i64 = raw.trim().parse().map_err(|_| AppError::Validation("bankProfileId doit être un entier"))?;
            if id <= 0 {
                return Err(AppError::Validation("bankProfileId doit être strictement positif"));
            }
            bank_profile_id = Some(id);
        }
        Some("confirmEncodingMismatch") => {
            if confirm_encoding_mismatch.is_some() {
                return Err(AppError::Validation("Champ 'confirmEncodingMismatch' dupliqué"));
            }
            confirm_encoding_mismatch = Some(field.text().await?.eq_ignore_ascii_case("true"));
        }
        // ... unknown field handling existant 8-1b ...
    }
}
```

Pattern : (1) duplicate guard, (2) parse + validation `> 0`, (3) cohérence avec `bankAccountId` qui valide déjà strict. Test E2E HTTP `post_csv_rejects_duplicate_bank_profile_id_field` couvre.

#### §audit-log

Trois nouvelles actions audit :
- `bank_profile.created` (entity_type `bank_profiles`, entity_id, details `{bank_name, filename_pattern}`)
- `bank_profile.updated` (idem, details `{bank_name, fields_changed: [...]}`)
- `bank_profile.deleted` (idem, details `{bank_name}`)

Action existante `bank_import.created` étend son `details_json` avec :
- `source_format: "CSV" | "CAMT053_V04" | "CAMT053_V08"`
- `bank_profile_id: <id> | null` (null si CAMT)
- **`bank_profile_name: <bank_name> | null`** (Pass 2 M'3 snapshot) — le `bank_name` du profil au moment de l'import est dupliqué dans le JSON audit pour préserver la trace humaine **après suppression du profil**. Corollaire L4 (FK orpheline acceptable v0.1) : sans cette snapshot, l'audit log historique afficherait juste un `bank_profile_id` inexistant. Avec snapshot, on peut afficher « Import via profil 'UBS' (supprimé depuis) » dans l'historique audit.

## Acceptance Criteria

Numérotation indépendante de 8-1b. Les ACs marqués **Hérité 8-1b** réutilisent le code persistance/UI sans re-test (8-2 vérifie uniquement les nouveaux comportements CSV-specific).

1. **(FR42 + FR50 — happy path UTF-8 avec `bankProfileId` explicite, Pass 1 M3)** Given un fichier CSV UTF-8 valide avec BOM, profil banque créé pour la company de l'utilisateur, When l'utilisateur uploade le fichier via multipart contenant `bankProfileId=<profile.id>` + `bankAccountId=<account.id>` + `file`, Then `POST /preview` retourne `200 OK` avec `appliedProfile.id = profile.id` + `sourceFormat = "CSV"`, et `POST /bank-imports` persiste toutes les transactions avec `bank_account_id = selected_id` et `bank_imports.source_format = 'CSV'`. *Test : E2E `imports a CSV UTF-8 file end-to-end` + E2E HTTP `post_csv_with_explicit_bank_profile_id_uses_it` + integration `kesh-import::csv::parser::tests::parses_utf8_with_bom`.*

2. **(FR52a — détection BOM UTF-8)** Given un fichier CSV avec BOM `EF BB BF` au début, When parsing, Then les 3 bytes BOM sont consommés et le contenu décodé en UTF-8. *Test unit : `detects_and_strips_utf8_bom`.*

3. **(FR52b — détection ISO-8859-1 sans BOM)** Given un fichier CSV ISO-8859-1 (caractères suisses-français accentués `é`, `ç`, `à` en bytes 0xE9, 0xE7, 0xE0), When parsing sans BOM, Then la détection chardetng identifie `windows-1252`/`iso-8859-1` et le décodage produit la chaîne UTF-8 correcte. *Test unit : `detects_iso_8859_1_via_heuristic` + fixture `csv_iso_8859_1_swiss_accents.csv`.*

4. **(FR52c — UTF-8 sans BOM, ASCII pur)** Given un fichier CSV ASCII pur (tous bytes < 0x80), When parsing sans BOM, Then la passe 1 UTF-8 strict réussit et le décodage est UTF-8. *Test unit : `parses_ascii_as_utf8`.*

5. **(FR52d — encoding non supporté)** Given un fichier UTF-16 LE (BOM `FF FE`), When upload, Then `422 BANK_CSV_UNSUPPORTED_ENCODING` avec `details.detected = "UTF-16LE"`. Aucun parsing n'est tenté. *Test E2E HTTP : `post_csv_rejects_utf16_encoding`.*

5bis. **(Pass 1 H3 + Pass 2 M'2 — fichiers tronqués/courts/header-only)** Given un fichier de 0, 1 ou 2 bytes (cas limites BOM check) OU un fichier avec header présent mais 0 ligne de données :
   - **5bis-a** : 0 byte → `422 BANK_CSV_EMPTY_FILE` avec `details.reason = "0 bytes"` (priorité 0 §encoding-detection).
   - **5bis-b** : 1-2 bytes → BOM check `for_bom(bytes)` retourne `None` (lib gère slices courts), passe 1 UTF-8 strict OK si bytes ASCII → `422 BANK_CSV_EMPTY_FILE` (0 transactions parsées). Sinon → `422 BANK_CSV_UNSUPPORTED_ENCODING` (passe 2 chardetng < 64 bytes).
   - **5bis-c (Pass 2 M'2)** : header-only file (ex. `"date,amount\n"` 11 bytes, `header_row_count = 1`, aucune ligne de données après skip) → `422 BANK_CSV_EMPTY_FILE` avec `details.reason = "0 data rows after header skip"` (post-iteration EmptyFile guard §csv-parser).
   *Tests E2E HTTP : `post_csv_rejects_empty_file` + `post_csv_handles_truncated_bom_2_bytes` + `post_csv_rejects_header_only_file` + integration `kesh-import::csv::encoding::tests::for_bom_handles_short_slices_safely`.*

5ter. **(Pass 1 H4 — header_row_count = 0)** Given un profil avec `header_row_count = 0` et un CSV sans ligne d'en-tête (toutes lignes sont des données), When parsing, Then aucune ligne n'est skippée (extra_skip = `saturating_sub(0, 1) = 0`) et toutes les lignes sont parsées comme transactions. *Test unit : `parses_csv_with_zero_header_rows` + fixture `csv_no_header.csv` (5 transactions sans header).*

5quater. **(Pass 1 M1 — détection format CSV vs CAMT)** Given trois sous-cas :
   - **5quater-a** : fichier `.bin` MIME `application/octet-stream` content non-XML, non-CSV-like → `415 BANK_IMPORT_UNSUPPORTED_FORMAT`.
   - **5quater-b** : fichier `releve.xml` mais MIME `text/csv` → l'extension wins (priorité 1), parser CAMT.053 invoqué (rejette proprement si non valide XML).
   - **5quater-c** : fichier `releve.txt` MIME `text/plain` content commençant par `<?xml ...>` → sniff content (priorité 3) détecte XML → CAMT.053.
   *Tests E2E HTTP : `post_unknown_format_returns_415` + `post_xml_extension_wins_over_csv_mime` + `post_txt_with_xml_content_sniffed_as_camt`.*

5quinquies. **(Pass 1 H5 — encoding mismatch profil vs détection)** Given un fichier CSV UTF-8 (auto-détecté UTF-8 par BOM ou heuristique) et un profil avec `encoding = "ISO-8859-1"` :
   - **5quinquies-a — preview** : `POST /preview` retourne `200 OK` + warning `bank_csv_encoding_mismatch` avec `details.profileEncoding = "ISO-8859-1"` + `details.detectedEncoding = "UTF-8"`. Le contenu décodé pour la preview utilise l'encoding **détecté** (UTF-8) — l'utilisateur voit les caractères corrects.
   - **5quinquies-b — final sans confirmation** : `POST /bank-imports` sans `confirmEncodingMismatch=true` → `422 BANK_CSV_ENCODING_MISMATCH`.
   - **5quinquies-c — final avec confirmation** : `POST /bank-imports` avec `confirmEncodingMismatch=true` → `201 Created` + audit log `bank_import.created_with_encoding_mismatch`. Le décodage utilise l'encoding du **profil** (ISO-8859-1) — l'utilisateur a explicitement accepté le risque mojibake.
   *Tests E2E HTTP : `post_csv_preview_warns_on_encoding_mismatch` + `post_csv_rejects_encoding_mismatch_without_confirm` + `post_csv_accepts_encoding_mismatch_with_confirm_writes_audit`.*

6. **(FR53 — création profil banque)** Given un utilisateur Comptable, When `POST /api/v1/bank-profiles` avec body valide, Then `201 Created` + entrée DB + audit log `bank_profile.created`. *Test E2E HTTP : `post_bank_profile_creates_with_audit_log`.*

7. **(FR53 — auto-apply profil par filename)** Given un profil banque avec `filename_pattern = "^export-ubs-\\d{8}\\.csv$"`, When upload `export-ubs-20260315.csv` sans `bankProfileId` explicite, Then le preview applique automatiquement le profil + warning UI `bank_csv_profile_auto_matched: { profile_id, bank_name }`. *Test E2E HTTP : `post_csv_preview_auto_matches_profile_by_filename` + scénario Playwright `auto-applies bank profile on upload`.*

8. **(FR53 — auto-apply ambigu)** Given deux profils avec `filename_pattern` qui matchent tous deux le filename uploadé, When preview, Then le plus récent (`updated_at DESC`) est appliqué + warning `bank_csv_multiple_profile_matches: { matched_profiles: [...] }`. *Test E2E HTTP : `post_csv_preview_warns_on_multiple_profile_matches`.*

9. **(FR53 — aucun profil match, Pass 1 M11)** Given un fichier CSV uploadé sans `bankProfileId` ET aucun `filename_pattern` ne matche, When preview, Then `404 BANK_CSV_NO_PROFILE_MATCH` avec `details.available_profiles = [{id, bank_name}]` **borné à 50 entrées max** (cap anti-amplification : si la company a plus de 50 profils, retourner les 50 plus récents `ORDER BY updated_at DESC`). *Test E2E HTTP : `post_csv_preview_404_when_no_profile_matches` + `post_csv_404_caps_available_profiles_at_50` (créer 51 profils + assert `details.available_profiles.length == 50`).*

10. **(FR51 — rejet partiel avec listing erreurs)** Given un CSV avec 5 lignes valides + 3 lignes invalides (date malformée ligne 7, montant non-numérique ligne 12, ligne 18 trop courte), When `POST /bank-imports`, Then `422 BANK_CSV_PARTIAL_FAILURE` avec body `{ lines: [{line: 7, code: "INVALID_DATE", value: "32.13.2026", message_i18n_key: "bank-csv-errors-invalid-date"}, ...] }`. **Aucune** transaction persistée (strict reject v0.1). *Test E2E HTTP : `post_csv_rejects_partial_failure_with_detailed_lines`.*

11. **(Multi-tenant scoping bank_profiles — KF-002 pattern)** Given un profil créé par `company_A`, When `company_B` appelle `GET /api/v1/bank-profiles/{id}` ou `GET /api/v1/bank-profiles` (list), Then `404 Not Found` (jamais 403). *Tests : `get_profile_returns_404_for_other_company` + `list_profiles_only_returns_own_company`.*

11bis. **(Pass 1 H8 — confused deputy `bankProfileId` cross-tenant)** Given `company_A` a un profil `id=42`, When un utilisateur de `company_B` (avec `bankAccountId` valide pour B) fait `POST /bank-imports` avec multipart `bankProfileId=42`, Then `404 BANK_CSV_NO_PROFILE_MATCH` (le repo `find_by_id_for_company(company_B, 42)` retourne `None`). Aucun leak de configuration profil cross-tenant. *Test E2E HTTP : `post_csv_import_rejects_bank_profile_id_from_other_tenant`.*

11ter. **(Pass 1 H8 — `bankProfileId` inexistant)** Given un utilisateur authentifié, When `POST /bank-imports` avec `bankProfileId=999999` (ID qui n'existe pas), Then `404 BANK_CSV_NO_PROFILE_MATCH`. *Test E2E HTTP : `post_csv_import_rejects_nonexistent_bank_profile_id`.*

11quater. **(Pass 1 M8 — `bankProfileId` dupliqué multipart)** Given un multipart contenant deux champs `bankProfileId=1` ET `bankProfileId=99`, When `POST /bank-imports`, Then `400 Validation` avec message `"Champ 'bankProfileId' dupliqué dans multipart"`. *Test E2E HTTP : `post_csv_rejects_duplicate_bank_profile_id_field` (pattern identique à `bankAccountId` 8-1b).*

12. **(Multi-tenant DB profile auto-match)** Given `company_A` a un profil avec `filename_pattern = "ubs.csv"`, When `company_B` uploade `ubs.csv` sans bankProfileId, Then `404 BANK_CSV_NO_PROFILE_MATCH` (les profils de `company_A` n'ont pas leaké). *Test integration : `auto_match_only_considers_own_company_profiles`.*

13. **(Sécurité — RBAC profil)** Given `Role::Consultation`, When `POST/PUT/DELETE /api/v1/bank-profiles/...`, Then `403`. `GET` accessible à tous les rôles authentifiés. *Tests : `post_profile_rejects_consultation_role` + `get_profile_allowed_for_consultation_role`.*

14. **(Sécurité — payload limit)** Hérité 8-1b. CSV > 10 MiB → `413 BANK_IMPORT_TOO_LARGE`. *Pas de re-test, lien vers test `post_import_rejects_payload_too_large` 8-1b.*

15. **(Validation profil — règles cumulées Pass 1 M4 + M5 + M13)** Given des payloads de création profil invalides, When `POST /bank-profiles`, Then `422 BANK_CSV_PROFILE_INVALID` avec message ciblé. Cas couverts :
   - **15a (XOR)** : `column_mapping: { amount: 3, debit_credit_split: [4, 5] }` → `"column_mapping doit contenir exactement un de amount XOR debit_credit_split"`. Cas inverse (ni l'un ni l'autre) → même rejet.
   - **15b (séparateurs distincts, M4)** : `field_separator = ',' AND decimal_separator = ','` → `"field_separator (',') doit être différent de decimal_separator (',')"`. Test couvre aussi `field_separator = ';' AND decimal_separator = '.'` → OK (créé).
   - **15c (collision indices colonnes, M13)** : `column_mapping: { date: 2, amount: 2, ... }` → `"column_mapping.date (2) conflicts with column_mapping.amount (2)"`. Test couvre aussi `debit_credit_split: [3, 3]` → conflict débit/crédit même colonne.
   *Tests E2E HTTP : `post_profile_rejects_xor_violation` + `post_profile_rejects_equal_separators` + `post_profile_rejects_column_mapping_collision`.*

15bis. **(Pass 1 M5 — `AmbiguousDebitCredit` au parse)** Given un profil avec `debit_credit_split = [3, 4]` et un CSV où la ligne 5 a `debit = "100.00"` ET `credit = "50.00"` simultanément, When `POST /bank-imports`, Then `422 BANK_CSV_PARTIAL_FAILURE` avec `details.lines[0] = {line: 5, code: "AMBIGUOUS_DEBIT_CREDIT", value: null, message_i18n_key: "bank-csv-errors-ambiguous-debit-credit"}`. *Test integration : `parses_debit_credit_both_filled_returns_ambiguous_error` + fixture `csv_debit_credit_both_filled.csv`.*

15ter. **(Pass 1 M10 — montant zéro vs empty mandatory)** Given un profil avec `debit_credit_split = [3, 4]` et un CSV où la ligne 8 a `debit = "" AND credit = ""` (les deux empty), When `POST /bank-imports`, Then `422 BANK_CSV_PARTIAL_FAILURE` avec `details.lines[0].code = "EMPTY_MANDATORY_FIELD"`. **Distinct du cas zéro explicite** : `debit = "0.00"` ou `credit = "0,00"` est accepté (transaction d'annulation valide). *Tests integration : `rejects_empty_debit_credit_columns` + `accepts_explicit_zero_debit_or_credit`.*

15quinquies. **(Pass 3 M''3 — `CsvLineError.value` truncation 100 chars UTF-8-aware)** Given un CSV avec une ligne invalide où la colonne `amount` contient une chaîne de 5000 caractères Unicode (mix latin + cyrillique pour stress UTF-8 boundary), When `POST /bank-imports`, Then `details.lines[0].value` retournée fait **exactement** 100 chars + suffixe `…` (101 caractères Unicode total). Garantit que le payload JSON 422 reste borné même si l'attaquant fournit des cellules géantes (100 lignes × 5000 chars = 500 KB unbounded sinon, vs 100 × 100 chars = 10 KB borné). *Tests : `csv_line_error_value_truncated_at_100_chars` (unit, vérifie boundary UTF-8 ne panique pas) + `partial_failure_total_payload_bounded` (E2E HTTP, fixture avec 100 lignes × valeurs 5KB → assert response body < 30 KB).*

15quater. **(Pass 2 M'1 + Pass 3 M''2 — indices hors-borne early-reject parser-side)** Given un profil DB avec `column_mapping.amount = 99` (index hors-borne pour un CSV à 5 colonnes), When `POST /bank-imports` avec ce CSV, Then `422 BANK_CSV_PROFILE_MISCONFIGURED` retourné via `CsvError::ProfileMisconfigured`. **Localisation early-reject** : le check vit dans `kesh-import::csv::parser::parse_csv` qui lit le **premier record de données** (post header skip), vérifie indices vs `first_record.len()`, et `Err` immédiatement sans entrer dans la boucle de collection — n'attend **pas** que toutes les lignes échouent en `RowTooShort`. Le payload final est un `CsvError::ProfileMisconfigured` simple, pas un `PartialFailure(Vec)` cappé à 100. *Tests : `post_csv_rejects_profile_column_index_out_of_bounds` (E2E HTTP) + `parser_early_rejects_oob_indices_on_first_record` (unit `kesh-import`).*

16. **(Validation profil — date_format chrono)** Given `date_format = "%Q"` (token invalide chrono), When `POST /bank-profiles`, Then `422 BANK_CSV_PROFILE_INVALID` avec message ciblant `date_format`. *Test E2E HTTP : `post_profile_rejects_invalid_chrono_format`.*

17. **(Validation profil — UNIQUE bank_name par company)** Given un profil `bank_name = "UBS"` existe déjà pour `company_A`, When second `POST /bank-profiles` même `bank_name`, Then `409 BANK_CSV_PROFILE_DUPLICATE`. Le même `bank_name` est OK pour `company_B` (multi-tenant). *Tests : `post_profile_rejects_duplicate_bank_name_within_company` + `post_profile_allows_same_bank_name_across_companies`.*

18. **(Doublons fichier — héritage 8-1b `(company_id, file_hash) UNIQUE`)** Hérité 8-1b. CSV ré-importé même fichier → `409 BANK_IMPORT_DUPLICATE_FILE`. *Pas de re-test 8-2, lien `unique_company_hash_blocks_duplicate_within_same_company`.*

19. **(Atomicité)** Hérité 8-1b. Strict reject 8-2 implique aucune persistance partielle, donc rien à tester en plus. *Pas de re-test.*

20. **(Audit log import CSV — Pass 2 M'3 snapshot)** Given un import CSV réussi via profil `id=42, bank_name="UBS"`, When `SELECT FROM audit_log WHERE entity_type='bank_imports'`, Then une entrée `action = bank_import.created` avec `details_json` contient :
   - `source_format: "CSV"`
   - `bank_profile_id: 42`
   - `bank_profile_name: "UBS"` (snapshot pour traçabilité post-delete profil)
   - `transaction_count`, `filename` (hérité 8-1b)
   *Test E2E HTTP : `post_csv_import_audit_log_includes_source_format_and_profile_snapshot` (vérifie les 3 nouveaux champs + supprime ensuite le profil et asserte que l'audit log historique reste intact avec `bank_profile_name = "UBS"`).*

21. **(Audit log profil)** Given un profil créé/modifié/supprimé, When `SELECT FROM audit_log WHERE entity_type='bank_profiles'`, Then trois entrées distinctes (`created` / `updated` / `deleted`) avec `entity_id = profile.id` et `details_json` rempli. *Tests : `post_profile_writes_audit_log_created` + `put_profile_writes_audit_log_updated` + `delete_profile_writes_audit_log_deleted`.*

22. **(i18n)** Given les 4 locales (fr/de/it/en-CH), When `npm run lint-i18n-ownership`, Then le lint passe (toutes clés `bank-profile-*` + `bank-csv-errors-*` + `bank-csv-warnings-*` présentes, préfixe kebab-case strict). *Test : CI Story 6-3 + extension `keyBelongsToFeature` 8-1b déjà supporte multi-segment names (`bank-profile`, `bank-csv`).*

23. **(Accessibilité — page profils + wizard)** Given pages `/bank-import/profiles` (liste) et le formulaire `BankProfileForm`, When `axe-core` scan, Then zéro violation. Les selects `field_separator` et `decimal_separator` ont `aria-label`, le textarea `column_mapping` n'est pas exposé brut (UI structure les colonnes en formulaire row-by-row). *Test E2E : `accessibility — profile pages axe scan zero violations`.*

24. **(Performance NFR)** Given un fichier CSV de 200 transactions UTF-8, When `POST /bank-imports`, Then la durée totale (decode + parse + DB) < 2s sur la machine de dev nominale. *Test instrumentation : `csv_pipeline_handles_500_transactions` smoke `Instant::now()`.*

25. **(Strip apostrophe milliers + decimal_separator virgule)** Given un CSV format suisse `1'234,56` avec `decimal_separator=','`, When parsing amount, Then la valeur résultante est `Decimal::new(123456, 2) = 1234.56`. *Test unit : `parses_swiss_amount_with_apostrophe_thousands_and_comma_decimal`.*

26. **(Profil filename_pattern regex injection safe, Pass 1 M20 reformulé)** Given un payload de création profil avec `filename_pattern` problématique :
   - **Properties du `regex` crate** (Rust) : moteur **NFA Thompson** — garantit O(N) sur le matching, **pas de catastrophic backtracking** (contrairement à PCRE/Java/Python). Le `size_limit` (10MB par défaut) limite la taille du **NFA compilé** (protection contre la compilation lente d'un pattern gigantesque).
   - **26a (compilation trop coûteuse)** : `filename_pattern = "(?:a|aa|aaa|aaaa){50}"` ou un pattern dont le NFA dépasse 10MB → `regex::Regex::new()` retourne `Err(CompiledTooBig)` → `422 BANK_CSV_PROFILE_INVALID` avec message ciblant `filename_pattern`. *Test : `post_profile_rejects_pathological_regex_compilation`.*
   - **26b (longueur input bornée)** : le filename uploadé est borné par `truncate_to_byte_len` (255 bytes, hérité 8-1b) → matching toujours O(255) max. Aucun input attacker-controlled au-delà → pas de DoS au matching grâce aux deux bornes (NFA + input).
   - **26c (longueur pattern bornée)** : `filename_pattern.len() > 200` au save → `422 BANK_CSV_PROFILE_INVALID`. *Test : `post_profile_rejects_pattern_over_200_chars`.*

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
  - **Pass 3 M''2 — early-reject indices OOB** : avant la boucle de collection d'erreurs, lire le **premier record de données** (post header skip) via `reader.records().next()`. Vérifier que tous les indices du `column_mapping` (`date`, `amount` si Some, `debit_credit_split.0` et `.1` si Some, `reference`, `details`, `counterparty`) sont strictement `< first_record.len()`. Si un index est OOB → return `Err(CsvError::ProfileMisconfigured("column_mapping.{field} (index {i}) out of bounds for {n} columns"))` **immédiatement**. Préserve le premier record pour le passer ensuite dans la boucle principale (`std::iter::once(first_record).chain(reader.records())` pattern).
  - Strip apostrophe (`'`) milliers + remplacement decimal_separator → `.` avant `Decimal::from_str_exact`.
  - Sur erreur ligne, **collecter les erreurs jusqu'à `MAX_CSV_LINE_ERRORS = 100`** (Pass 2 H'1 cap anti-DoS). Au-delà, continuer à itérer pour incrémenter `total_errors` mais ne pas accumuler de nouvelles entrées dans le Vec. Si non-vide à la fin → `CsvError::PartialFailure { errors, total_errors, truncated: total_errors > MAX_CSV_LINE_ERRORS }`. Strict reject v0.1. Test unit `partial_failure_caps_at_100_errors_for_huge_invalid_csv` : fixture avec 500 lignes invalides → assert `errors.len() == 100`, `total_errors == 500`, `truncated == true`.
  - **Pas** de validation balance pour CSV (les CSV n'exposent pas opening/closing balance) — le helper `validate_balance` est skip côté API pour `source_format = CSV`.
- [ ] **T2.4** `crates/kesh-import/src/error.rs` : étendre avec `CsvError` (cf. §error-types — **12 variantes** post Pass 1 incluant `EmptyFile`, `EncodingMismatch`, `ProfileMisconfigured`, `PartialFailure`) + struct `CsvLineError` + enum `CsvLineErrorCode` + `ImportError` enum wrapper. `From<CamtError>` + `From<CsvError>` for `ImportError`. **Pass 1 H1 + Pass 1 H7** : ajouter aussi variante `kesh_import::SourceFormat::Csv { encoding, profile_name }` (struct existe déjà mais doit être consommable par le pipeline 8-1b — vérifier que `kesh_import::types.rs` la définit déjà ou l'ajouter).
- [ ] **T2.5** `crates/kesh-import/src/lib.rs` : `pub mod csv;` + `pub use csv::{parse_csv, CsvProfile, ColumnMapping, DetectedEncoding};` + `pub use error::{ImportError, CamtError, CsvError, CsvLineError, CsvLineErrorCode};`.
- [ ] **T2.6** Tests `crates/kesh-import/tests/csv_tests.rs` : 12+ tests d'intégration utilisant les fixtures `tests/fixtures/csv/*.csv`.
- [ ] **T2.7** Fixtures `tests/fixtures/csv/` (10 fixtures post Pass 1) :
  - `utf8_bom_minimal.csv` (3 tx, BOM, `;` separator)
  - `iso_8859_1_swiss_accents.csv` (5 tx, accents é/ç/à en bytes 0xE9/0xE7/0xE0)
  - `utf8_swiss_amount.csv` (apostrophe milliers + virgule decimal `1'234,56`)
  - `partial_failure.csv` (8 rows : 5 OK + 3 invalid mixed — date/amount/row_too_short)
  - `utf16_le.csv` (1 tx UTF-16 LE BOM — reject test)
  - **Pass 1 H4** `csv_no_header.csv` (5 tx, `header_row_count = 0` test)
  - **Pass 1 M5** `csv_debit_credit_both_filled.csv` (5 OK + 1 row avec debit ET credit non-empty → `AmbiguousDebitCredit`)
  - **Pass 1 M10** `csv_debit_credit_both_empty.csv` (5 OK + 1 row avec debit ET credit empty → `EmptyMandatoryField`) + variant `csv_debit_credit_explicit_zero.csv` (5 OK avec une ligne `debit="0,00"` valide)
  - **Pass 1 H3** `csv_2_bytes_truncated_bom.csv` (raw bytes `EF BB` — BOM tronqué)
  - **Pass 1 H5** `csv_utf8_content_iso_profile.csv` (bytes UTF-8 `0xC3 0xA9` mais sera testé avec un profil `encoding=ISO-8859-1` → encoding mismatch)
  - **Pass 1 M14** `csv_header_only.csv` (header présent, 0 lignes de données → `EmptyFile`).
- [ ] **T2.8** Cargo.toml — ajouter dépendances : `csv = "1.3"`, `encoding_rs = "0.8"`, `chardetng = "0.1"`, `regex = "1.10"` (pour profile filename_pattern validation côté kesh-import). **Vérifier `cargo publish --dry-run` reste vert** (aucune dép interne).

### T3. Entités + repos `bank_profiles` (AC #6, #11, #17, #21)

- [ ] **T3.1** `crates/kesh-db/src/entities/bank_profile.rs` :
  - `pub struct BankProfile` (champs cf. §profile-model, sans `column_mapping` typé — stocké en `String` JSON, désérialisé via méthode `pub fn parse_column_mapping(&self) -> Result<ColumnMapping, DbError>`).
  - `pub struct NewBankProfile` (sans id/created_at/updated_at).
  - **Pattern `BankImportSourceFormat` 8-1b ne s'applique pas ici** (pas d'enum).
- [ ] **T3.2** `crates/kesh-db/src/repositories/bank_profiles.rs` (Pass 3 L''2 — signatures génériques `Executor` pour permettre l'appel transaction-bound §profile-matching race) :
  - `create(executor: impl sqlx::Executor<'_, Database = MySql>, company_id, profile) -> Result<BankProfile, DbError>` + `INSERT INTO audit_log` atomique dans la même transaction (pattern 8-1b). Note : `create` est typiquement appelé avec un `&mut Transaction` côté handler pour grouper avec audit_log.
  - `find_by_id_for_company(executor: impl sqlx::Executor<'_, Database = MySql>, company_id, id) -> Result<Option<BankProfile>, DbError>` (KF-002 helper). Accepte `&MySqlPool` ET `&mut Transaction` via le trait `Executor` — appel critique pour `POST /bank-imports` (transaction-bound).
  - `list_by_company(executor, company_id, pagination) -> Result<(Vec<BankProfile>, i64), DbError>`.
  - `update(executor, company_id, id, new_profile: NewBankProfile) -> Result<BankProfile, DbError>` + audit log (Pass 1 M18 — PUT full replacement, param renamed `new_profile`).
  - `delete(executor, company_id, id) -> Result<(), DbError>` + audit log.
  - `find_matching_profiles_for_filename(pool, company_id, filename) -> Result<Vec<BankProfile>, DbError>` — SELECT puis filter Rust-side via `regex::Regex::new(profile.filename_pattern).is_match(filename)` (filtrage SQL impossible, regex MariaDB diffère). ORDER BY `updated_at DESC`. **Pass 1 L1** : pas de cache regex v0.1 (acceptable pour < 50 profils par company, typique en v0.1) ; documenter dans Dev Notes que le cache `Arc<HashMap<String, Regex>>` invalidé sur CRUD profil sera adressé v0.2 si une company > 100 profils est observée. La compilation O(N) à chaque upload reste bornée par `T1.1 chk_bank_profiles_filename_pattern_len ≤ 200` + complexité NFA Thompson.
- [ ] **T3.3** Mapping erreurs SQL :
  - `1062` (Duplicate entry) sur `uq_bank_profiles_company_name` → `DbError::ProfileDuplicate`.
- [ ] **T3.4** Tests `kesh-db` : 8 tests `#[sqlx::test]` (create + audit, find_by_id_own_company, find_by_id_other_company_returns_none, list_paginated, update + audit + race optimistic, delete + audit, duplicate_bank_name_rejected, find_matching_profiles_filters_by_company).

### T4. Routes API `bank_profiles` CRUD (AC #6, #11, #13, #15, #16, #17, #21, #26)

- [ ] **T4.1** `crates/kesh-api/src/routes/bank_profiles.rs` (nouveau fichier) — 5 handlers :
  - `POST /api/v1/bank-profiles` (create, RBAC Comptable+).
  - `GET /api/v1/bank-profiles?page=&per_page=` (list, all roles).
  - `GET /api/v1/bank-profiles/{id}` (detail, all roles).
  - `PUT /api/v1/bank-profiles/{id}` (**update full replacement**, Comptable+) — Pass 1 M18 : sémantique REST PUT stricte = remplacement complet. Body doit contenir tous les champs requis (bank_name, column_mapping, date_format, decimal_separator, field_separator, header_row_count + champs Optionnels même si null). Tout champ omis → `422 BANK_CSV_PROFILE_INVALID`. Pas de PATCH partiel v0.1.
  - `DELETE /api/v1/bank-profiles/{id}` (delete, Comptable+) — Pass 1 M17 : delete libre, **pas de RESTRICT** sur les imports historiques. Le `bank_profile_id` dans `audit_log.details_json` reste comme référence orpheline (acceptable v0.1, documenté §audit-log et Dev Notes Limitations connues). Le `bank_name` est dupliqué dans le `details_json` au moment de l'import pour préserver la trace humaine même après suppression du profil.
- [ ] **T4.2** `crates/kesh-api/src/errors.rs` : **9 nouvelles variantes** `AppError` (Pass 1 M6 — liste consolidée canonique cf. §error-types tableau ; T4.2 + T5.0 pointent toutes les deux vers cette même liste pour éviter la divergence) :
  - `BankCsvProfileNotFound` → 404 `BANK_CSV_NO_PROFILE_MATCH`
  - `BankCsvUnsupportedEncoding` → 422 `BANK_CSV_UNSUPPORTED_ENCODING`
  - `BankCsvEncodingMismatch` → 422 `BANK_CSV_ENCODING_MISMATCH`
  - `BankCsvParsePartialFailure(Vec<CsvLineError>)` → 422 `BANK_CSV_PARTIAL_FAILURE` avec payload structuré
  - `BankCsvProfileValidation(String)` → 422 `BANK_CSV_PROFILE_INVALID`
  - `BankCsvProfileDuplicate` → 409 `BANK_CSV_PROFILE_DUPLICATE` (UNIQUE bank_name)
  - `BankCsvProfileMisconfigured(String)` → 500 `BANK_CSV_PROFILE_MISCONFIGURED`
  - `BankCsvEmptyFile` → 422 `BANK_CSV_EMPTY_FILE`
  - `BankImportUnsupportedFormat` → 415 `BANK_IMPORT_UNSUPPORTED_FORMAT`
- [ ] **T4.3** Validation côté handler (avant repo) :
  - Parse `column_mapping` JSON → struct → `validate()` (cf. T2.2)
  - Compile `filename_pattern` via `regex::Regex::new` — failure → 422.
  - Return early avec `BankCsvProfileValidation(reason)`.
- [ ] **T4.4** Mount sur `comptable_routes` (write) + `authenticated_routes` (read), pattern 8-1b.
- [ ] **T4.5** Tests E2E HTTP `crates/kesh-api/tests/bank_profiles_e2e.rs` : 12 tests (create+audit, RBAC, list scoping, detail 404 cross-tenant, update+audit, delete+audit, validation errors, duplicate 409, pathological regex 422).

### T5. Extension `POST /bank-imports` pour CSV (AC #1, #5, #5bis, #5ter, #5quater, #5quinquies, #7, #8, #9, #10, #11bis, #11ter, #11quater, #20)

- [ ] **T5.0** **Pré-requis Pass 1 H1** — extension enums `source_format` côté Rust (point critique non livré par 8-1b, **doit être fait avant T5.1+**) :
  - **T5.0.a** `crates/kesh-db/src/entities/bank_import.rs` :
    - Ajouter variante `BankImportSourceFormat::Csv` à l'enum.
    - `as_db_str()` retourne `"CSV"` pour la nouvelle variante.
    - `from_str()` parse `"CSV"` → `Ok(BankImportSourceFormat::Csv)` + supprimer le test `source_format_unknown_rejected` qui asserte le rejet de "CSV" + ajouter test `source_format_csv_roundtrip` symétrique aux tests CAMT053.
    - Mise à jour des `#[derive(sqlx::Type)]` impl manuelle pour MySql encoder/decoder.
  - **T5.0.b** `crates/kesh-core/src/bank_imports.rs` :
    - Ajouter variante `SourceFormatTag::Csv` à l'enum.
    - `as_db_str()` retourne `"CSV"`.
    - Étendre `from_imported()` : la branch `kesh_import::SourceFormat::Csv { encoding, profile_name }` doit retourner `Ok((... SourceFormatTag::Csv, ...))` au lieu de `Err(CoreError::BankImportUnknownVersion("csv"))`. Le `profile_name` peut alimenter un nouveau champ `BankImportDraft.source_format_label` (optionnel UI) ou être ignoré v0.1.
    - Test `from_imported_csv_succeeds` symétrique aux tests CAMT.
  - **T5.0.c** `crates/kesh-api/src/routes/bank_imports.rs::version_to_source_format()` :
    - Étendre pour mapper `kesh_import::SourceFormat::Csv { .. }` → `BankImportSourceFormat::Csv`.
    - Pas d'erreur sur le path CSV (vs aujourd'hui qui fallback `BankImportParseFailed`).

- [ ] **T5.1** `crates/kesh-api/src/routes/bank_imports.rs` — détection format upload (Pass 1 M1 + M19) :
  - Helper `fn detect_import_format(filename: &str, content_type: Option<&str>, raw_first_bytes: &[u8]) -> Result<ImportFormat, AppError>`
  - **Important** : le sniff opère sur **raw bytes** (pas decoded — l'encoding n'est pas encore connu). Patterns ASCII-safe : `<?xml`, `<Document`, `BkToCstmrStmt` pour CAMT ; présence de séparateur courant pour CSV.
  - `enum ImportFormat { Camt053, Csv }`
  - Priorité : extension > MIME > sniff. Cf. §csv-detection.
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
- [ ] **T5.5** Tests E2E HTTP étendus dans `bank_imports_e2e.rs` : **18 nouveaux tests CSV post Pass 1** :
  - Happy paths : `post_csv_with_explicit_bank_profile_id_uses_it`, `imports_csv_iso_8859_1_with_swiss_accents`
  - Encoding : `post_csv_rejects_utf16_encoding`, `post_csv_rejects_empty_file` (Pass 1 H3), `post_csv_handles_truncated_bom_2_bytes` (Pass 1 H3)
  - Encoding mismatch (Pass 1 H5) : `post_csv_preview_warns_on_encoding_mismatch`, `post_csv_rejects_encoding_mismatch_without_confirm`, `post_csv_accepts_encoding_mismatch_with_confirm_writes_audit`
  - Format detection (Pass 1 M1) : `post_unknown_format_returns_415`, `post_xml_extension_wins_over_csv_mime`, `post_txt_with_xml_content_sniffed_as_camt`
  - Profile resolution (Pass 1 H8) : `post_csv_import_rejects_bank_profile_id_from_other_tenant`, `post_csv_import_rejects_nonexistent_bank_profile_id`, `post_csv_rejects_duplicate_bank_profile_id_field` (Pass 1 M8)
  - Profile match : `post_csv_preview_auto_matches_profile_by_filename`, `post_csv_preview_warns_on_multiple_profile_matches`, `post_csv_preview_404_when_no_profile_matches`, `post_csv_404_caps_available_profiles_at_50` (Pass 1 M11)
  - Partial failure : `post_csv_rejects_partial_failure_with_detailed_lines`
  - Audit log : `post_csv_import_audit_log_includes_source_format_and_profile`

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

- [ ] **T8.1** `frontend/tests/e2e/bank-csv-import.spec.ts` (nouveau fichier) — **9 scénarios post Pass 1** :
  1. `imports a CSV UTF-8 file end-to-end` (AC #1)
  2. `creates a bank profile via wizard then imports` (AC #6 + #7)
  3. `auto-applies bank profile on filename match` (AC #7)
  4. `shows BANK_CSV_NO_PROFILE_MATCH when no profile matches` (AC #9)
  5. `displays partial failure error panel with line numbers` (AC #10)
  6. `rejects UTF-16 with unsupported encoding error` (AC #5)
  7. `accessibility — profile pages axe scan zero violations` (AC #23)
  8. **Pass 1 H5** `shows encoding mismatch warning then accepts override` (AC #5quinquies) — preview affiche warning + checkbox confirmation, confirm passe `confirmEncodingMismatch=true`.
  9. **Pass 1 M1** `rejects unsupported file format with 415` (AC #5quater-a) — upload `.bin` → erreur UI.
- [ ] **T8.2** Fixtures Playwright `frontend/tests/e2e/fixtures/` : **6 CSV post Pass 1** (`csv_utf8_bom_minimal.csv`, `csv_iso_8859_1_swiss.csv`, `csv_partial_failure.csv`, `csv_utf16_le.csv`, `csv_utf8_for_iso_profile.csv` mojibake test, `unknown_format.bin` 415 test). **Décision Pass 1 L6** (BH-9 fixtures dupliquées kesh-import vs Playwright) : les fixtures Playwright sont **différentes** (plus enrichies, contenu réel banque suisse) ; les fixtures kesh-import sont synthétiques minimales pour les unit tests. Documenter cette divergence en commentaire en tête de chaque dossier.
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
4. `kesh-core` (7 nouvelles variantes `CoreError::BankCsv*` post Pass 1+2 — liste canonique §error-types)
5. `frontend/src/lib/features/bank-import` (extension : 3 nouveaux components + 3 nouvelles routes)
6. `frontend/src/lib/shared/i18n/locales` (extension fichier `bank-import.json` × 4 locales)

**Total : 6 modules**. Au seuil règle CLAUDE.md « splitter si > 5 modules ». Précédent 8-1b unifié à 5-6 modules a converge en 3 passes validate (Sonnet → Haiku → Opus, 20 patches) sans drift.

**Décision : pas de split préventif**. Justifications :
- Frontière naturelle frontend/backend déjà absorbée par 8-1b — 8-2 réutilise les composants drop-zone, page route, api-client, audit log.
- Pas de path dep cargo cassée : `kesh-import::csv` est ajouté dans la même crate que `camt053`, donc pas de réordonnancement workspace.
- Le module CSV est isolé (zéro coupling avec camt053 sauf re-exports `lib.rs`).
- Les 7 nouvelles variantes `CoreError::BankCsv*` (liste canonique §error-types post Pass 1+2) restent isolées au domaine import (pattern Story 8-1a).

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

**Localisation `validate_csv_profile_signature` — décision Pass 3 M''2 : parser-side** :

Le check d'indices hors-borne est implémenté **dans le parser `kesh-import::csv::parser`**, pas dans `kesh-core`. Justification :

- Cohérent avec `CsvError::ProfileMisconfigured(String)` qui existe déjà §error-types — pas besoin de helper `kesh-core` séparé.
- **Early-reject avant la collection d'erreurs** : `parse_csv` lit le premier record de données, vérifie que tous les indices du `column_mapping` (`date`, `amount` ou les 2 indices `debit_credit_split`, `reference`, `details`, `counterparty`) sont strictement < `first_record.len()`. Si un index est OOB → return `Err(CsvError::ProfileMisconfigured("column_mapping.{field} (index {i}) out of bounds for {n} columns"))` **immédiatement**, sans entrer dans la boucle de collection des `RowTooShort`. C'est ça qui défait l'objectif anti-amplification de Pass 1 M21 si on faisait le check côté handler post-parse.
- Pas de violation `kesh-import` zéro-dep `kesh-core` : `CsvProfile` est défini côté `kesh-import` (T2.2), donc le parser accède librement aux champs.
- AC #15quater testable directement en unit `kesh-import` (pas besoin d'intégration handler).

Pas de fonction `kesh-core::bank_imports::validate_csv_profile_signature` séparée — le check vit entièrement côté parser.

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
- `crates/kesh-core/src/errors.rs` (7 variantes — liste canonique §error-types)
- `crates/kesh-core/src/bank_imports.rs` (helper signature optionnel)
- `crates/kesh-api/src/routes/bank_profiles.rs` (nouveau)
- `crates/kesh-api/src/routes/bank_imports.rs` (extension dispatch)
- `crates/kesh-api/src/errors.rs` (9 variantes — liste canonique §error-types tableau)
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

### Limitations connues v0.1 (Pass 1 documentées)

- **L4 — FK `bank_profile_id` orpheline dans audit log** : `bank_imports` n'a pas de colonne `bank_profile_id` avec FK. Le `bank_profile_id` apparaît dans `audit_log.details_json` comme métadonnée, mais aussi le `bank_name` snapshot pour préserver la trace humaine après suppression du profil. Le delete profil est donc libre (pas de RESTRICT). **v0.2** : ajouter `bank_imports.bank_profile_id BIGINT NULL` avec FK `ON DELETE SET NULL` pour permettre les joins relationnels propres post-delete.
- **L5 — `filename_pattern` catch-all** : un profil avec `filename_pattern = "^.*$"` matche tous les filenames CSV uploadés et capture l'auto-match (en prenant le `updated_at` le plus récent). UI doit afficher un warning visuel au save quand le pattern est trop large, mais pas de blocage v0.1 (laisse à l'utilisateur sa responsabilité).
- **L1 — pas de cache regex** : compilation O(N profils) à chaque upload. Acceptable pour < 50 profils par company. v0.2 si nécessaire.
- **§audit-log dette** : delete profile crée une référence orpheline dans audit log (mitigée par snapshot `bank_profile_name` Pass 2 M'3). Acceptable v0.1, FK ajoutée v0.2.
- **L7 — parse CSV dans la transaction** (Pass 3 M''4) : `parse_csv()` se déroule à l'intérieur de la transaction MariaDB ouverte pour résoudre le profil + INSERT. Sur uploads > 50 MB (~100k transactions), la transaction reste ouverte plusieurs secondes, dégradant le SLA latence d'autres requêtes touchant `bank_profiles` (row lock implicite). v0.2 si observé en prod : pivoter vers pre-parse hors tx + revalidation `profile.version` dans la tx (Interprétation B).

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
- PRD — [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md) FR42, FR50, FR51, FR52, FR53, scénario Lisa import partiel (Pass 1 L3 : section UX Scenarios — chercher « Lisa importe un fichier CSV » dans le PRD plutôt qu'une ligne précise, le numéro change selon les passes d'édition PRD)
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
| 2026-05-05 | **Validate Pass 3 Opus 4.7 — closure cycle review** — cycle CLAUDE.md complet Sonnet → Haiku → Opus. 3 reviewers parallèles. Verdict **Acceptance Auditor Opus : GO sans condition**, 0 finding > LOW Auditor (juste AA3-1 LOW non-blocking sur AC #24 perf NFR rattachement task). Blind Hunter et Edge Case Hunter ont remonté **6 findings résiduels** : 4 MEDIUM + 2 LOW. Décision Guy Option A : appliquer les 6 patches puis STOP cycle review (Auditor a déjà dit GO, les 4 MEDIUM sont des affinements documentaires + 1 décision design + 1 cap résiduel, pas des trous structurels). Patches appliqués : M''1 drift documentaire « 4 variantes » → 7/9 (§Risque de splitting + §Source tree alignés sur liste canonique §error-types) ; M''2 décision **parser-side** pour `validate_csv_profile_signature` (early-reject sur 1er record dans `kesh-import::csv::parser`, pas de helper séparé `kesh-core` — préserve l'objectif anti-amplification de Pass 1 M21) ; M''3 truncation `CsvLineError.value` UTF-8-aware via const `MAX_CSV_LINE_ERROR_VALUE_CHARS = 100` + AC #15quinquies (ferme le DoS payload-size que Pass 2 H'1 prétendait clore mais laissait à 100 × 1 MB) ; M''4 séquencement parse-vs-tx **Interprétation A** (parse dans la transaction, simple + cohérence cohésive) avec Limitations connues L7 v0.1 documentée (SLA latence > 50 MB) ; L''1 nommage `find_matching_profiles_for_filename` aligné en pluriel ; L''2 T3.2 signatures génériques `impl sqlx::Executor` pour transaction-bound. Trend complet : Pass 1 = 29 → Pass 2 = 9 → Pass 3 = 6 → 0 post-patches. **Critère d'arrêt CLAUDE.md atteint** : 0 finding > LOW + Verdict Auditor GO. Cycle review 3 passes Sonnet → Haiku → Opus complet, ~44 patches appliqués au total (29 + 9 + 6). Spec finale : 900 → ~960 lignes, 37 → 38 ACs (#15quinquies ajouté Pass 3). Spec status : `ready-for-dev` confirmé. Prochaine étape : `bmad-dev-story 8-2`. | Claude (Opus 4.7, validate Pass 3 application + closure) |
| 2026-05-05 | **Validate Pass 2 Haiku 4.5** — cycle CLAUDE.md Sonnet → Haiku, fenêtre fraîche. 3 reviewers parallèles. Verdict **Acceptance Auditor : GO sans condition**. 18 findings bruts → triage : **3 HIGH** (H'1 PartialFailure Vec non borné DoS — cap 100 avec `truncated` flag ; H'2 race profile delete + import concurrent — pattern transaction-bound + helper executor générique pattern KF-002 ; H'3 mojibake bypass `confirmEncodingMismatch=true` sans preview — re-detect encoding systématique côté final, ignore flag si pas de mismatch effectif) + **6 MEDIUM** (M'1 `validate_csv_profile_signature` AC explicite + 15quater ; M'2 header-only file AC #5bis-c ; M'3 snapshot `bank_profile_name` audit log + AC #20 enrichi ; M'4 drift `details: String` pas `Option` — alignement avec `kesh_import::ImportedTransaction.details: String` + DB `bank_transactions.details TEXT NOT NULL` 8-1b ; M'5 regex case-sensitive doc + flag `(?i)` ; M'6 justification field_separator distinct) + **3 LOW** (BH2-9/10, EH2-7) + **5 rejets/faux-positifs** (BH2-1 T5.0 déjà documenté, BH2-4 sniff acceptable, EH2-5 multipart code 8-1b OK, EH2-8/9 déjà couverts). **Option A appliquée** : 9 patches HIGH+MEDIUM appliqués. Trend : Pass 1 = 29 findings > LOW → Pass 2 = 9 findings > LOW → -69%. Critère d'arrêt CLAUDE.md non atteint en Pass 2 (mais 0 > LOW post-patches). Nouvelles sections : `MAX_CSV_LINE_ERRORS = 100` const, payload `truncated` flag, anti-bypass mojibake re-detect, race transaction-bound profile resolution, snapshot `bank_profile_name` audit log. ACs : 5bis-c + 15quater nouveaux. Stats : 843 → ~920 lignes. Prochaine étape : Pass 3 Opus 4.7 (cycle Sonnet → Haiku → Opus, validation finale). | Claude (Opus 4.7, validate Pass 2 application) |
| 2026-05-05 | **Validate Pass 1 Sonnet 4.6** — 3 reviewers parallèles (Blind Hunter / Edge Case Hunter / Acceptance Auditor), cycle CLAUDE.md (auteur=Opus, Pass 1=Sonnet pour briser biais). Verdict Acceptance Auditor : **CONDITIONAL GO** (2 HIGH bloquants AA-1 + AA-2). 45 findings bruts → triage : **8 HIGH** (H1 hypothèse fausse `BankImportSourceFormat::Csv` non livrée 8-1b ; H2 `period_from`/`period_to` NOT NULL CSV stratégie manquante ; H3 `bytes[..3]` panic fichier < 3 bytes ; H4 underflow u8 `header_row_count = 0` ; H5 mojibake silencieux profil ISO-8859-1 + fichier UTF-8 ; H6 `ColumnMapping.amount` non-Option ; H7 `CsvError::PartialFailure` + `CsvLineError` non définis ; H8 `bankProfileId` cross-tenant/invalide non spécifié) + **21 MEDIUM** + **9 LOW** + **2 rejects** (AA-11 ref existe ; BH-9 fixtures dupliquées reclassée L6 décision documentée). **Option A appliquée** : 29 patches HIGH+MEDIUM appliqués. Spec passe de 597 → ~830 lignes. Sections enrichies : §Scope verrouillé point 1bis (extension enums Csv obligatoire), §encoding-detection (algo 0+3 priorités, mojibake bloquant final), §profile-model (`Option<usize>` + collision check + priorité parse DB corrompue), §profile-matching (étape 1b 404), §csv-detection (sniff raw bytes priorités), §csv-parser (saturating skip, `position.line()`, period_from min/max, EmptyFile guard), §error-types (12 variantes CsvError + CsvLineError + 9 AppError canonique), §preview-csv-response-shape, §multipart-guards. ACs : 5bis/5ter/5quater/5quinquies/11bis/11ter/11quater/15bis/15ter ajoutés (35 ACs total post Pass 1 vs 26 initiaux). Tasks : T5.0 sub-tasks enums, T5.5 18 E2E HTTP, T8.1 9 scénarios Playwright, T2.7 10 fixtures. Limitations connues v0.1 documentées (L1/L4/L5). Trend Pass 1 : 29 findings > LOW raw → 0 (tous appliqués). Prochaine étape : commit + Pass 2 Haiku 4.5 (cycle Sonnet → Haiku). | Claude (Opus 4.7, validate Pass 1 application) |
| 2026-05-05 | Spec créée par `bmad-create-story 8-2` post-merge PR #69 (8-1a + 8-1b done). Status `backlog` → `ready-for-dev`. Branche `story/8-2-import-csv-multi-encodage-profils-banque` créée depuis main `076ac86`. 26 ACs définis (vs 7 ACs epic-8.md → enrichis avec FR52a-d détection encoding, FR53 auto-match, FR51 strict reject mapping, multi-tenant, RBAC, validation profil, audit log, i18n, a11y, perf). 9 tasks T1-T9. Dépendances upstream 8-1a/8-1b validées. Risque de splitting documenté (6 modules au seuil, pas de split préventif, trigger validate Pass 4 si non-convergence). | Claude (Opus 4.7, bmad-create-story) |
