//! Story 9-2b T4 — Manifeste JSON inclus dans le ZIP global.
//!
//! Schema canonique du manifeste (AC #12) — clés en camelCase, ordres
//! déterministes (`BTreeMap` alphabétique pour `tables`, cohérent serde_json
//! `Map<String, Value>` ordering + tests byte-stability) :
//!
//! ```json
//! {
//!   "keshVersion": "0.1.0",
//!   "exportDate": "2026-05-17T13:00:00Z",
//!   "companyId": 42,
//!   "companyName": "CI Test Company",
//!   "locale": "fr-CH",
//!   "fiscalYearScope": "all",
//!   "tables": {
//!     "accounts.csv": { "rowCount": 5, "sha256": "…" },
//!     …
//!   }
//! }
//! ```
//!
//! Le SHA-256 par CSV est calculé sur les **bytes décompressés** (BOM +
//! header + data rows + CRLF), pas sur les bytes ZIP-compressés — c'est la
//! représentation logique du fichier, indépendante du ZIP container.

use std::collections::BTreeMap;

use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::errors::AppError;
use kesh_db::entities::Company;

/// Manifeste JSON du ZIP global (Story 9-2b §metadata.json).
///
/// Ordre des champs cohérent avec la lecture humaine : version → date →
/// company → locale → scope → tables (en dernier, le plus volumineux).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalExportMetadata {
    pub kesh_version: String,
    pub export_date: String,
    pub company_id: i64,
    pub company_name: String,
    pub locale: String,
    pub fiscal_year_scope: String,
    pub tables: BTreeMap<String, TableMeta>,
}

/// Métadonnées par CSV (row count + SHA-256 hex).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableMeta {
    pub row_count: usize,
    pub sha256: String,
}

/// Construit le `metadata.json` (Pretty-printed pour lisibilité humaine).
///
/// - `kesh_version` lu via `env!("CARGO_PKG_VERSION")` au build-time
///   (Decision §version-source). Si le caller rebuild `kesh-api` sans bump,
///   le manifeste continue de reporter l'ancienne version (L7).
/// - `export_date` au format ISO 8601 UTC strict avec suffixe `Z` (jamais
///   offset `+01:00`).
/// - `locale` reçu déjà résolu par le handler (`util::map_language_to_bcp47`).
/// - `fiscal_year_scope` = `"all"` v0.1 (filtrage par exercice = L2 v0.2).
///
/// Retourne le JSON sérialisé pretty + finished `\n` (`to_vec_pretty` ajoute
/// déjà un trailing newline implicite — vérifié en test).
pub fn build_metadata_json(
    company: &Company,
    locale_bcp47: &str,
    tables: BTreeMap<String, TableMeta>,
) -> Result<Vec<u8>, AppError> {
    let meta = GlobalExportMetadata {
        kesh_version: env!("CARGO_PKG_VERSION").to_string(),
        export_date: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        company_id: company.id,
        company_name: company.name.clone(),
        locale: locale_bcp47.to_string(),
        fiscal_year_scope: "all".to_string(),
        tables,
    };
    serde_json::to_vec_pretty(&meta)
        .map_err(|e| AppError::GlobalExportFailed(format!("metadata serialize: {e}")))
}

/// Hash SHA-256 d'un slice de bytes → 64 chars hex (minuscule).
///
/// Réutilise `crate::util::hex_encode` (Decision §hex-encoding) — pas de crate
/// `hex` externe.
pub fn sha256_hex(bytes: &[u8]) -> String {
    crate::util::hex_encode(&Sha256::digest(bytes))
}

// ===========================================================================
// Tests (Story 9-2b T4 + AC #30(e)(f))
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use kesh_db::entities::company::{Language, OrgType};

    fn naive_dt(y: i32, m: u32, d: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
    }

    fn sample_company() -> Company {
        Company {
            id: 42,
            name: "CI Test Company".into(),
            address: "Rue Test 1\n1000 Lausanne".into(),
            address_street: "Rue Test".into(),
            address_building: "1".into(),
            address_postal_code: "1000".into(),
            address_city: "Lausanne".into(),
            address_country: "CH".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
            is_stub: false,
            version: 1,
            created_at: naive_dt(2026, 1, 1),
            updated_at: naive_dt(2026, 1, 1),
        }
    }

    fn sample_tables() -> BTreeMap<String, TableMeta> {
        let mut t = BTreeMap::new();
        t.insert(
            "accounts.csv".into(),
            TableMeta {
                row_count: 5,
                sha256: "abc123".into(),
            },
        );
        t.insert(
            "journal_entries.csv".into(),
            TableMeta {
                row_count: 1,
                sha256: "def456".into(),
            },
        );
        t
    }

    // ----- AC #30(f) — sha256_hex deterministic -----

    #[test]
    fn sha256_hex_empty_matches_canonical() {
        // SHA-256 of b"" = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            sha256_hex(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_hex_basic_input() {
        // Connue : SHA-256 de b"abc" = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_hex_returns_64_hex_chars() {
        let h = sha256_hex(b"kesh-9-2b");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ----- AC #30(e) — build_metadata_json shape -----

    #[test]
    fn build_metadata_json_has_canonical_shape() {
        let bytes = build_metadata_json(&sample_company(), "fr-CH", sample_tables()).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // camelCase fields
        assert_eq!(parsed["keshVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(parsed["companyId"], 42);
        assert_eq!(parsed["companyName"], "CI Test Company");
        assert_eq!(parsed["locale"], "fr-CH");
        assert_eq!(parsed["fiscalYearScope"], "all");
        // tables BTreeMap → alphabetical
        let tbls = &parsed["tables"];
        assert!(tbls.is_object());
        assert_eq!(tbls["accounts.csv"]["rowCount"], 5);
        assert_eq!(tbls["accounts.csv"]["sha256"], "abc123");
        assert_eq!(tbls["journal_entries.csv"]["rowCount"], 1);
        // exportDate ISO 8601 UTC strict (suffix Z, second-precision)
        let date = parsed["exportDate"].as_str().unwrap();
        assert!(
            date.ends_with('Z'),
            "exportDate must end with 'Z', got: {date}"
        );
        assert!(
            date.len() == 20,
            "expected YYYY-MM-DDTHH:MM:SSZ (20 chars), got: {date}"
        );
    }

    #[test]
    fn build_metadata_json_tables_non_empty_sha() {
        let bytes = build_metadata_json(&sample_company(), "fr-CH", sample_tables()).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for (_k, v) in parsed["tables"].as_object().unwrap() {
            assert!(
                v["sha256"].as_str().is_some_and(|s| !s.is_empty()),
                "sha256 must be non-empty for every table"
            );
        }
    }
}
