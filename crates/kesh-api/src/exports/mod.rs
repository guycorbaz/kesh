//! Module exports (Story 9-2b — Export global ZIP de souveraineté).
//!
//! Trois sous-modules :
//!
//! - [`csv_tables`] — 16 fonctions `serialize_<table>_csv` qui sérialisent les
//!   tables comptables au format CSV Story 9-2a (BOM UTF-8 + `;` + CRLF +
//!   RFC 4180).
//! - [`metadata`] — `GlobalExportMetadata` + `build_metadata_json` +
//!   `sha256_hex` (manifeste JSON inclus dans le ZIP, hash SHA-256 par CSV
//!   pour vérification d'intégrité).
//! - [`global`] — `build_global_export` orchestrateur (queries 16 tables +
//!   serialize + sha + metadata + zip) + `build_zip` (assemble le ZIP en
//!   mémoire) + `GlobalExportMeta` (struct retournée pour audit + tracing).
//!
//! Decision §csv-table-serializer-location : ce module vit dans `kesh-api`
//! (PAS dans `kesh-report`) — `kesh-report` est dédié aux rapports comptables
//! agrégés (Story 9-1 + 9-2a). Inclure des serializers "raw table" violerait
//! DD-12.

pub mod csv_tables;
pub mod global;
pub mod metadata;

pub use global::{GlobalExportMeta, build_global_export};
pub use metadata::{GlobalExportMetadata, TableMeta, build_metadata_json, sha256_hex};
