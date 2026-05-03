# Story 8.1a: Parseur CAMT.053 (kesh-import + kesh-core)

Status: ready-for-dev

<!-- Issue de scission de Story 8-1 (`8-1-import-camt053.md`) le 2026-05-04 :
     Story 8-1 unifiée touchait 6 modules cross-cutting (au seuil CLAUDE.md « splitter si > 5 modules »).
     Décision Guy 2026-05-04 (avant `dev-story`) : split en 8-1a (parser-only, indépendant DB/UI)
     + 8-1b (persistance + API + frontend) afin d'éviter la rechute Story 7-1 (7 passes review).
     La spec d'origine 8-1-import-camt053.md reste comme référence pour les sections de contexte
     (décisions de conception détaillées, patterns architecturaux). -->

## Story

As a **développeur Kesh préparant Story 8-1b (persistance et UI d'import bancaire)**,
I want **un crate `kesh-import` autonome qui parse CAMT.053 v04/v08 + les helpers `kesh-core::bank_imports` (`from_imported`, `validate_balance`, `validate_currency_supported_v0_1`)**,
so that **8-1b consomme cette couche comme path dep stable, testable en isolation (cargo test, cargo publish --dry-run, cargo metadata invariant), sans nécessiter DB ni frontend**.

### Contexte

**Story 8-1a = première moitié de la story unifiée 8-1**, scindée pré-implémentation pour respecter la règle de splitting CLAUDE.md (> 5 modules touchés). Voir [`8-1-import-camt053.md`](8-1-import-camt053.md) (status `archived-split`) pour la spec d'origine — toutes les **décisions de conception** (§iban-tolerant, §currency, §balance-check, §quick-xml, §multi-stmt) y sont documentées en détail et restent valides pour 8-1a/8-1b sans modification.

**Pourquoi 8-1a en premier :** le crate `kesh-import` est publiable indépendamment (décision archi #7), donc tout 8-1b sera bâti **dessus** (path dep). Livrer d'abord la couche parseur + types domaine permet :

- Validation par tests unitaires + `cargo publish --dry-run` + `cargo metadata` (invariant zéro path dep interne) **sans dépendance DB/HTTP/UI**.
- Itération sur les ACs #5, #6, #7, #8, #9 + algorithmes #14 (balance check) et #15 (currency rejection) sans seed-CI ni MariaDB démarré.
- 8-1b reprendra ensuite avec `kesh-import = { path = "../kesh-import" }` déjà stable.

**Spike outcome (rappel) :** [`spike-kesh-import.md`](spike-kesh-import.md) verdict `feasible`. Types autonomes (`ImportedStatement`, `ImportedTransaction`, `SourceFormat`) + `From<ImportedTransaction> for BankTransactionDraft` côté `kesh-core::bank_imports` déjà en place. 8-1a remplit les modules `camt053/` (parseur effectif) et étend `bank_imports.rs` (helpers).

**Status sprint :** `8-1a-camt053-parser-only: backlog → ready-for-dev` au moment de la création (2026-05-04). `8-1b-camt053-persistence-ui` reste `backlog` jusqu'à 8-1a `done`.

### Scope verrouillé — ce qui est livré par 8-1a

1. **Fixtures CAMT.053** (T1) — `crates/kesh-import/tests/fixtures/camt053/` × 10 fichiers + `README.md` provenance synthétique.
2. **Parseur `kesh-import::camt053`** (T2) — modules `camt053/{mod.rs, v04.rs, v08.rs}`, dispatch namespace via `quick_xml::NsReader`, `error.rs` enum `CamtError`, tests unitaires `tests/camt053_tests.rs`.
3. **Extensions `kesh-core::bank_imports`** (T3) — `BankImportDraft`, `from_imported(stmt, bank_account_id, company_id, file_hash, filename, imported_at)`, `validate_balance(stmt) -> Result<(), CoreError>`, `validate_currency_supported_v0_1(stmt) -> Result<(), CoreError>` + variantes `CoreError::BankImportBalanceMismatch` et `CoreError::BankImportUnsupportedCurrency` + bras `error_code()` correspondants.
4. **Invariants CI** (T8.4 + T8.5) — `.github/workflows/ci.yml` : ajouter `cargo publish --dry-run --allow-dirty -p kesh-import` (AC #7) + `cargo metadata -p kesh-import` invariant zéro path dep interne (AC #6).

**HORS scope 8-1a (→ 8-1b) :**
- Migration DB `bank_imports` + `bank_transactions` (T4)
- Entités + repositories `kesh-db` (T5)
- Route API `/api/v1/bank-imports/...` + multipart + audit log (T6)
- Frontend `bank-import` feature (T7)
- i18n `bank-import-*` keys (T8.1-T8.3)
- Tests E2E Playwright (T9)
- Sync README + sprint-status final (T10)

### Décisions de conception (rappel — voir spec d'origine pour le détail)

Toutes les décisions §iban-tolerant, §currency, §balance-check, §quick-xml, §multi-stmt de [`8-1-import-camt053.md`](8-1-import-camt053.md) §317-447 s'appliquent telles quelles à 8-1a. Points critiques pour la couche parseur+core :

- **§iban-tolerant** : `counterparty_iban` reste `Option<String>` brute, jamais validée par `kesh-core::types::Iban` côté parseur. Test `parse_invalid_iban_keeps_transaction` doit produire un `ImportedTransaction` avec l'IBAN cassé conservé, **aucun warning côté parseur**.
- **§currency** : `validate_currency_supported_v0_1` accepte uniquement `"CHF"`. EUR / USD → `Err(CoreError::BankImportUnsupportedCurrency)`. Le mapping vers `422 Unprocessable Entity` côté API vit dans 8-1b.
- **§balance-check** : algorithme `validate_balance` implémenté dans `kesh-core` (CR-010 #62). Tolérance `0.01` (Decimal). Skip silencieux si `opening_balance` ou `closing_balance` est `None`. Le mapping vers `422` ou `200 OK + warnings` côté API vit dans 8-1b.
- **§quick-xml** : `quick-xml = "0.39"` (latest stable, requiert `Reader::config_mut()` ≥ 0.37). `NsReader` (pas `Reader` simple) pour résoudre les namespaces préfixés `<ns:Document xmlns:ns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.04">` indépendamment du préfixe via `resolve_element(qname)`.
- **§multi-stmt** : 8-1a parse **tous** les `<Stmt>` du fichier et retourne `Vec<ImportedStatement>`. Le filtrage par IBAN sélectionné par l'utilisateur (warning `ignoredStatements`) vit dans 8-1b côté handler API.

## Acceptance Criteria

Numérotation héritée de la spec 8-1 d'origine pour traçabilité. Les ACs non listés ici sont du ressort de 8-1b.

2. **(FR49)** Given un fichier CAMT.053 contenant des `<TxDtls>` (sous-transactions) sous un `<Ntry>` agrégé, When parse, Then chaque `<TxDtls>` produit une `ImportedTransaction` distincte (et non l'agrégat seul). *Test : `parse_with_subtxs_extracts_individual_transactions`.*

5. **(FR87 / Décision archi #2)** Given un fichier CAMT.053, When le namespace racine de `<Document>` est `urn:iso:std:iso:20022:tech:xsd:camt.053.001.04` ou `.08` (forme par défaut OU préfixée), Then le parseur de version correspondante est utilisé. Les autres URIs retournent `CamtError::UnsupportedVersion(uri)`. *Tests : `parse_v04_minimal_extracts_all_transactions`, `parse_v04_prefixed_namespace_extracts_all_transactions`, `parse_v08_minimal_extracts_all_transactions`, `parse_unknown_namespace_returns_unsupported_version`.*

6. **(Architecture)** Given le crate `kesh-import`, When `cargo metadata -p kesh-import --format-version 1 | jq '.packages[0].dependencies[] | select(.path != null)'` est exécuté, Then aucune dépendance workspace interne n'apparaît. *Test : step CI ajouté à `.github/workflows/ci.yml` (T8.5).*

7. **(Architecture)** Given le crate `kesh-import`, When `cargo publish --dry-run --allow-dirty -p kesh-import` est exécuté, Then la commande termine sans erreur. *Test : step CI Rust (T8.4).*

8. **(Architecture)** Given un `ImportedTransaction`, When converti via `From`, Then le résultat est un `BankTransactionDraft` valide ; les FK (`bank_account_id`, `import_id`, `company_id`) sont injectées par `kesh-core::bank_imports::from_imported`, **pas** par `kesh-import`. *Test : `from_imported_injects_fk` (étend les tests spike existants).*

9. **(Fixtures synthétiques)** Given la suite de tests d'intégration `kesh-import`, When exécutée, Then les 10 fixtures listées en T1 sont chargées et tous les cas distincts (v04 default-ns / v04 prefixed-ns / v08 / sub-tx / multi-stmt / balance-mismatch / truncated / invalid-iban / EUR / cdt-dbt-indicator) sont validés. *Test : `crates/kesh-import/tests/camt053_tests.rs` × 10.*

**ACs partiels (algorithme dans 8-1a, mapping API dans 8-1b) :**

14a. **(CR-010 #62 — algorithme balance check)** Given un `ImportedStatement` avec `opening_balance.is_some() && closing_balance.is_some()` et `|opening + Σ transactions - closing| > 0.01`, When `kesh_core::bank_imports::validate_balance(&stmt)`, Then `Err(CoreError::BankImportBalanceMismatch { opening, closing, sum, diff })`. Si `opening_balance.is_none()` ou `closing_balance.is_none()` → `Ok(())` (skip silencieux). *Tests : `validate_balance_passes_when_within_tolerance`, `validate_balance_fails_when_diff_exceeds_one_cent`, `validate_balance_skipped_when_balances_missing`.*

15a. **(Devise v0.1 — algorithme rejection)** Given un `ImportedStatement` avec `currency != "CHF"`, When `kesh_core::bank_imports::validate_currency_supported_v0_1(&stmt)`, Then `Err(CoreError::BankImportUnsupportedCurrency(currency))`. *Tests : `validate_currency_chf_passes`, `validate_currency_eur_rejected_v0_1`.*

## Tasks / Subtasks

### T1. Fixtures CAMT.053 (AC #9)

- [ ] T1.1 — Créer `crates/kesh-import/tests/fixtures/camt053/` et `crates/kesh-import/tests/fixtures/README.md` (provenance synthétique, pas de PII, IBAN suisses fictifs MOD-97 valides).
- [ ] T1.2 — Construire les **10** fichiers XML listés § Scope T1 (8-1) à la main à partir de l'XSD `docs/six-references/ig-cash-managment-xml-schemas-v2.0.2-en/camt.053.001.08.xsd` (pour v08) et de la spec ISO 20022 v04 publique :
  - `v04_minimal.xml` (namespace par défaut)
  - `v04_prefixed_namespace.xml` (`<ns:Document xmlns:ns="...">`, **critique**)
  - `v08_minimal.xml`
  - `v04_with_subtxs.xml` (`<Ntry>` agrégée + 3 `<TxDtls>`, FR49)
  - `v04_multi_stmt.xml` (2 `<Stmt>` IBAN différents)
  - `v04_balance_mismatch.xml` (`|opening + Σ - closing| > 0.05`)
  - `v04_truncated.xml` (XML mal formé en plein milieu)
  - `v04_invalid_iban.xml` (IBAN counterparty checksum cassé)
  - `v04_eur_currency.xml` (`<Acct><Ccy>EUR</Ccy>`)
  - `v04_credit_debit_indicator.xml` (1 DBIT + 1 CRDT, montants `<Amt>` non-signés)
- [ ] T1.3 — Vérifier `xmllint --noout fixtures/*.xml` → tous well-formed (sauf `v04_truncated.xml` qui doit échouer).

### T2. Parseur `kesh-import::camt053` (AC #1, #2, #5, #9)

- [ ] T2.1 — Ajouter `quick-xml = "0.39"` à `crates/kesh-import/Cargo.toml` (latest stable, requis pour `Reader::config_mut()`). Aucun ajout `[dev-dependencies]` au-delà du spike.
- [ ] T2.2 — Créer `crates/kesh-import/src/error.rs` :

  ```rust
  #[derive(Debug, thiserror::Error, Clone, PartialEq)]
  pub enum CamtError {
      #[error("XML mal formé : {0}")]
      MalformedXml(String),
      #[error("Version CAMT.053 non supportée : {0}")]
      UnsupportedVersion(String),
      #[error("Champ requis manquant : {0}")]
      MissingRequiredField(&'static str),
      #[error("Montant invalide : {0}")]
      InvalidAmount(String),
      #[error("Date invalide : {0}")]
      InvalidDate(String),
  }
  ```

- [ ] T2.3 — Créer `crates/kesh-import/src/camt053/mod.rs` avec :
  - Constantes `NS_V04 = "urn:iso:std:iso:20022:tech:xsd:camt.053.001.04"`, `NS_V08 = "urn:iso:std:iso:20022:tech:xsd:camt.053.001.08"`.
  - `pub fn parse(xml: &[u8]) -> Result<Vec<ImportedStatement>, CamtError>` qui :
    1. Instancie `quick_xml::NsReader::from_reader(xml)`.
    2. Cherche le premier `Event::Start` au tag local `Document`.
    3. Résout le namespace via `reader.resolve_element(qname)` → URI.
    4. Si URI = NS_V04 → délègue à `v04::parse(reader)`. Si URI = NS_V08 → `v08::parse(reader)`. Sinon → `Err(CamtError::UnsupportedVersion(uri))`.
- [ ] T2.4 — Implémenter `crates/kesh-import/src/camt053/v04.rs` (parser pull-based) — extrait les éléments :
  - `<BkToCstmrStmt>` (racine de la cardinalité `Vec<ImportedStatement>`)
  - `<Stmt>` (1 → 1 `ImportedStatement`)
  - `<Stmt><Id>` → `statement_id: Option<String>`
  - `<Stmt><Acct><Id><IBAN>` → `account_iban: String`
  - `<Stmt><Acct><Ccy>` → `currency: String`
  - `<Stmt><FrToDt><FrDtTm>` → `period_from: NaiveDate` (parse `2026-05-01T00:00:00` → `2026-05-01`)
  - `<Stmt><FrToDt><ToDtTm>` → `period_to: NaiveDate`
  - `<Stmt><Bal>` × N : trouver le `<Bal>` avec `<Tp><CdOrPrtry><Cd>OPBD</Cd>` → `opening_balance` ; `CLBD` → `closing_balance`. `<Bal><Amt Ccy="...">123.45</Amt>` + `<CdtDbtInd>` → `Decimal` signé.
  - `<Ntry>` × N (1 → 1 `ImportedTransaction` si pas de `<TxDtls>`, sinon 1 par `<TxDtls>` cf. FR49) :
    - `<BookgDt><Dt>` ou `<BookgDt><DtTm>` → `booking_date: NaiveDate`
    - `<ValDt><Dt>` → `value_date: Option<NaiveDate>`
    - `<Amt Ccy="...">123.45</Amt>` + `<CdtDbtInd>DBIT|CRDT>` → `amount: Decimal` signé (DBIT négatif, CRDT positif)
    - `<NtryRef>` → `reference: Option<String>` (fallback : `<RmtInf><Strd><CdtrRefInf><Ref>` si présent)
    - `<AddtlNtryInf>` ou `<RmtInf><Ustrd>` (concat) → `details: String` (vide si rien → `""`)
    - `<NtryDtls><TxDtls><Refs><EndToEndId>` → `end_to_end_id: Option<String>`
    - `<AcctSvcrRef>` → `transaction_id: Option<String>`
    - `<NtryDtls><TxDtls><RltdPties><Cdtr><Nm>` ou `<Dbtr><Nm>` → `counterparty_name`
    - `<NtryDtls><TxDtls><RltdPties><CdtrAcct><Id><IBAN>` ou `<DbtrAcct>` → `counterparty_iban`
- [ ] T2.5 — Implémenter `crates/kesh-import/src/camt053/v08.rs`. Réutiliser le code commun via un trait `CamtParser { fn parse(self) -> Result<Vec<ImportedStatement>, CamtError>; }` ; en pratique v08 partage 95% du schéma au niveau des tags utilisés ici, le delta principal `<Othr><SchmeNm>` enrichi n'est pas mobilisé. Documenter inline `// Delta v04 → v08 : ...`.
- [ ] T2.6 — Mettre à jour `crates/kesh-import/src/lib.rs` : ajouter `pub mod camt053; pub mod error;`. Conserver `pub mod types; pub use types::{...};`. Re-exporter `pub use camt053::parse as parse_camt053; pub use error::CamtError;`.
- [ ] T2.7 — Tests unitaires `crates/kesh-import/tests/camt053_tests.rs` (10 tests, AC #9) :
  - `parse_v04_minimal_extracts_all_transactions`
  - `parse_v04_prefixed_namespace_extracts_all_transactions` (régression H4)
  - `parse_v08_minimal_extracts_all_transactions`
  - `parse_with_subtxs_extracts_individual_transactions` (FR49)
  - `parse_multi_stmt_returns_one_per_account`
  - `parse_truncated_returns_malformed_xml_error`
  - `parse_unknown_namespace_returns_unsupported_version`
  - `parse_invalid_iban_keeps_transaction` (tolérance §iban-tolerant)
  - `parse_eur_currency_preserved` (la devise est extraite, le rejet vit dans kesh-core)
  - `parse_credit_debit_indicator_signs_amount_correctly`

### T3. Extensions `kesh-core::bank_imports` (AC #8, #14a, #15a)

- [ ] T3.1 — Étendre `crates/kesh-core/src/bank_imports.rs` avec :

  ```rust
  use chrono::NaiveDateTime;

  #[derive(Clone, Debug, PartialEq)]
  pub struct BankImportDraft {
      pub company_id: i64,
      pub bank_account_id: i64,
      pub filename: String,
      pub file_hash: String,        // SHA-256 hex 64 chars
      pub source_format: SourceFormatTag,  // enum mappant SourceFormat → tag DB
      pub statement_id: Option<String>,
      pub period_from: chrono::NaiveDate,
      pub period_to: chrono::NaiveDate,
      pub opening_balance: Option<crate::types::Money>,
      pub closing_balance: Option<crate::types::Money>,
      pub transaction_count: i32,
      pub imported_at: NaiveDateTime,
      pub imported_by_user_id: i64,
  }

  #[derive(Clone, Debug, PartialEq)]
  pub enum SourceFormatTag {
      Camt053V04,
      Camt053V08,
      // Csv ajouté Story 8-2.
  }
  ```

  `imported_by_user_id` est posé à 0 par défaut côté `from_imported` (le caller — handler API 8-1b — surcharge avec le user du JWT avant persistance). Alternative : passer `imported_by_user_id` en paramètre comme `imported_at`.

- [ ] T3.2 — Implémenter `pub fn from_imported(stmt: &ImportedStatement, bank_account_id: i64, company_id: i64, file_hash: String, filename: String, imported_at: NaiveDateTime, imported_by_user_id: i64) -> (BankImportDraft, Vec<BankTransactionDraft>)`. **`imported_at` et `imported_by_user_id` sont passés en paramètres** — pas de `Utc::now()` ou de lookup user interne (préserve la pureté `kesh-core`).

  Le `SourceFormatTag` est dérivé de `stmt.source_format` :
  ```rust
  let source_format = match &stmt.source_format {
      SourceFormat::Camt053 { version } if version == "001.04" => SourceFormatTag::Camt053V04,
      SourceFormat::Camt053 { version } if version == "001.08" => SourceFormatTag::Camt053V08,
      SourceFormat::Camt053 { version } => return // erreur ou unreachable selon design — OUVRIR : décision impl
      SourceFormat::Csv { .. } => unreachable!("Story 8-1a ne traite pas CSV"),
  };
  ```

  Convention retenue : `SourceFormat::Camt053 { version }` avec une version inconnue est traité comme une erreur du parseur (ne devrait pas arriver, mais on évite `panic!` côté `kesh-core`). Implémenter `from_imported` qui retourne `Result<(BankImportDraft, Vec<BankTransactionDraft>), CoreError>` plutôt qu'un tuple direct, avec une nouvelle variante `CoreError::BankImportUnknownVersion(String)` (Display : `"version CAMT.053 inattendue : {0}"`, code `"BANK_IMPORT_UNKNOWN_VERSION"`).

- [ ] T3.3 — Implémenter `pub fn validate_balance(stmt: &ImportedStatement) -> Result<(), CoreError>` (AC #14a, CR-010 #62) :

  ```rust
  pub fn validate_balance(stmt: &ImportedStatement) -> Result<(), CoreError> {
      let (Some(opening), Some(closing)) = (stmt.opening_balance, stmt.closing_balance) else {
          return Ok(());
      };
      let sum = stmt.sum_transactions();
      let expected = opening + sum;
      let diff = (expected - closing).abs();
      if diff > Decimal::new(1, 2) /* 0.01 */ {
          return Err(CoreError::BankImportBalanceMismatch {
              opening: Money::new(opening),
              closing: Money::new(closing),
              sum: Money::new(sum),
              diff: Money::new(diff),
          });
      }
      Ok(())
  }
  ```

- [ ] T3.4 — Implémenter `pub fn validate_currency_supported_v0_1(stmt: &ImportedStatement) -> Result<(), CoreError>` (AC #15a) :

  ```rust
  pub fn validate_currency_supported_v0_1(stmt: &ImportedStatement) -> Result<(), CoreError> {
      if stmt.currency != "CHF" {
          return Err(CoreError::BankImportUnsupportedCurrency(stmt.currency.clone()));
      }
      Ok(())
  }
  ```

- [ ] T3.5 — Étendre `crates/kesh-core/src/errors.rs` avec **3 nouvelles variantes** + bras `error_code()` correspondants (la fonction est exhaustive sans wildcard) :

  ```rust
  #[error("solde de clôture incohérent : ouverture {opening} + somme {sum} ≠ clôture {closing} (écart {diff})")]
  BankImportBalanceMismatch {
      opening: Money,
      closing: Money,
      sum: Money,
      diff: Money,
  },

  #[error("devise non supportée v0.1 : {0} (seul CHF est accepté)")]
  BankImportUnsupportedCurrency(String),

  #[error("version CAMT.053 inattendue : {0}")]
  BankImportUnknownVersion(String),
  ```

  Bras `error_code()` ajoutés :
  - `Self::BankImportBalanceMismatch { .. } => "BANK_IMPORT_BALANCE_MISMATCH"`
  - `Self::BankImportUnsupportedCurrency(_) => "BANK_IMPORT_UNSUPPORTED_CURRENCY"`
  - `Self::BankImportUnknownVersion(_) => "BANK_IMPORT_UNKNOWN_VERSION"`

- [ ] T3.6 — Tests unitaires (étendre les 4 tests spike existants à ~10) :
  - `validate_balance_passes_when_within_tolerance`
  - `validate_balance_fails_when_diff_exceeds_one_cent`
  - `validate_balance_skipped_when_balances_missing`
  - `validate_currency_chf_passes`
  - `validate_currency_eur_rejected_v0_1`
  - `from_imported_injects_fk` — vérifier que le tuple retourné porte bien `bank_account_id`, `company_id`, `file_hash`, `imported_at`, `imported_by_user_id` à la valeur passée en paramètre.
  - `from_imported_returns_one_draft_per_transaction` — `Vec<BankTransactionDraft>.len() == stmt.transactions.len()`.
  - `from_imported_unknown_version_returns_err` — `SourceFormat::Camt053 { version: "001.99".into() }` → `Err(BankImportUnknownVersion("001.99"))`.

### T8. Invariants CI (AC #6, #7)

- [ ] T8.4 — Étendre le job `Backend (Rust)` du workflow `.github/workflows/ci.yml` avec un step `cargo publish --dry-run --allow-dirty -p kesh-import` (modèle aligné sur le step équivalent `kesh-qrbill` Story 5-3 — vérifier le nom exact du job dans le yml avant édition).
- [ ] T8.5 — Ajouter un step `cargo metadata --format-version 1 -p kesh-import | jq -e '.packages[] | select(.name == "kesh-import") | .dependencies[] | select(.path != null) | length' | wc -l` qui doit retourner `0` (zéro path dep interne). Échec → CI rouge.

## Risque de splitting (CLAUDE.md check)

**Modules touchés par 8-1a** : 2 (`kesh-import`, `kesh-core::bank_imports`) + 1 fichier CI (`.github/workflows/ci.yml`). Bien sous le seuil > 5. **Pas de re-split.**

**Profondeur d'incertitude** : faible. Spike done, types autonomes posés, conversions `From` posées. T2 et T3 sont du parser fill + algorithmes triviaux (validate_balance = 5 lignes, validate_currency = 2 lignes). Les tests unitaires sur les 10 fixtures couvrent les cas edge.

## Dev Notes

### Patterns architecturaux à respecter

- **Indépendance `kesh-import`** : aucune dépendance workspace interne. Vérifier après chaque ajout de dépendance avec `cargo metadata -p kesh-import` (cf. T8.5). Ne PAS ajouter `kesh-core`, `kesh-db`, etc.
- **`From`/`Into` côté `kesh-core`** (décision archi #7) : le crate `kesh-core` connaît `kesh-import`, jamais l'inverse.
- **Pureté `kesh-core::bank_imports`** : zéro I/O, zéro horloge interne, zéro `unwrap()/panic!()` sur input externe. `imported_at` et `imported_by_user_id` passés en paramètre. Les `unreachable!()` autorisés uniquement sur des branches qu'aucun input externe ne peut atteindre.
- **`rust_decimal` arithmétique** : utiliser `Decimal` de bout en bout. **Jamais** `f64`. Comparer avec `==` (scale-invariant côté `rust_decimal::Decimal::eq`).
- **`thiserror` pour les erreurs** : pattern aligné `CoreError`. `Display` en français pour le logging serveur, le mapping i18n vit côté `kesh-api` (8-1b).

### Source tree à toucher

**Backend** :
- `crates/kesh-import/Cargo.toml` (deps `quick-xml`)
- `crates/kesh-import/src/lib.rs` (re-exports)
- `crates/kesh-import/src/error.rs` *(nouveau)*
- `crates/kesh-import/src/camt053/mod.rs` *(nouveau)*
- `crates/kesh-import/src/camt053/v04.rs` *(nouveau)*
- `crates/kesh-import/src/camt053/v08.rs` *(nouveau)*
- `crates/kesh-import/tests/camt053_tests.rs` *(nouveau)*
- `crates/kesh-import/tests/fixtures/camt053/*.xml` × 10 *(nouveaux)*
- `crates/kesh-import/tests/fixtures/README.md` *(nouveau)*
- `crates/kesh-core/src/bank_imports.rs` (extension : `BankImportDraft`, `SourceFormatTag`, `from_imported`, `validate_balance`, `validate_currency_supported_v0_1`)
- `crates/kesh-core/src/errors.rs` (3 variantes + 3 bras `error_code`)

**CI** :
- `.github/workflows/ci.yml` (ajout `cargo publish --dry-run` + `cargo metadata` invariant)

### Standards de test

- **Unitaires Rust** : `cargo test -p kesh-import` et `cargo test -p kesh-core bank_imports`. Pas de DB, pas de seed, pas de Playwright. Lancement local rapide (< 10 secondes).
- **Format / lint / build / test workspace** : full local sweep avant push (cf. CLAUDE.md « Test Locally First → Backend »).

### Checklist locale avant push

```sh
# Format + build + clippy
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings

# Tests workspace (parallèle sans DB suffit pour 8-1a)
cargo test --workspace

# Invariants 8-1a
cargo publish --dry-run --allow-dirty -p kesh-import
cargo metadata -p kesh-import --format-version 1 \
  | jq '.packages[] | select(.name == "kesh-import") | .dependencies[] | select(.path != null)'
# ↑ doit retourner vide
```

### Références

- Spec d'origine (contexte étendu) : [`8-1-import-camt053.md`](8-1-import-camt053.md) (status `archived-split`)
- Spike outcome : [`spike-kesh-import.md`](spike-kesh-import.md)
- ACs Epic 8 : [`epic-8.md`](../planning-artifacts/epic-8.md#story-8-1--import-camt053)
- FR42, FR49, FR50, FR87 : [`prd.md`](../planning-artifacts/prd.md) lignes 436-447 + 520
- Décisions archi #2 (multi-version parsers) + #7 (types autonomes) : [`architecture.md`](../planning-artifacts/architecture.md) §76 + §11.5
- CR-010 #62 (statement balance check) : [github.com/guycorbaz/kesh/issues/62](https://github.com/guycorbaz/kesh/issues/62)

## Dev Agent Record

### Agent Model Used

(à remplir par le dev agent)

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Action | Auteur |
|------|--------|--------|
| 2026-05-04 | Création de la story par split de 8-1 (`8-1-import-camt053.md`) en 8-1a (parser-only) + 8-1b (persistance + UI). Justification : règle CLAUDE.md « splitter si > 5 modules » (8-1 unifiée touchait 6 modules). Précédent rétro Epic 7 : Story 7-1 a explosé à 7 passes review faute de splitting préventif. Décision Guy 2026-05-04. La spec d'origine 8-1 reste comme référence des décisions de conception détaillées. | Claude (Opus 4.7, dev-story split coordinator) |
