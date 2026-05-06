//! Parseur CSV bancaire multi-encodage avec profils par banque (Story 8-2).
//!
//! Pipeline :
//! 1. [`encoding::detect_encoding`] — BOM check (priorité 1) → UTF-8 strict
//!    (priorité 2) → chardetng heuristique (priorité 3 sur 1024 bytes max).
//! 2. [`parser::parse_csv`] — décodage + skip header + iter rows mappées via
//!    [`profile::CsvProfile.column_mapping`] → `Vec<ImportedTransaction>`.
//! 3. Stricte rejet partiel (FR51 v0.1) : toute erreur ligne agrège dans
//!    [`crate::CsvError::PartialFailure`] cap 100 (Pass 2 H'1).
//!
//! Pas de dépendance sur `kesh-core` ou `kesh-db` (crate publiable, cf.
//! décision architecture #7).

pub mod encoding;
pub mod parser;
pub mod profile;

pub use encoding::{DetectedEncoding, detect_encoding};
pub use parser::{ParseCsvOutcome, empty_valid_sentinel_date, parse_csv, parse_csv_collect};
pub use profile::{ColumnMapping, CsvProfile};
