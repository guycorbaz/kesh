//! kesh-import — Parseurs bancaires CAMT.053 et CSV (publiable, zéro dépendance interne).
//!
//! Ce crate est volontairement autonome : il ne dépend d'aucun autre crate du
//! workspace Kesh (`kesh-core`, `kesh-db`, etc.) et ne référence pas leurs
//! types. Les conversions vers les types domaine de `kesh-core` sont
//! implémentées **côté `kesh-core`** via `From`/`Into` (cf. décision
//! architecture #7).
//!
//! Cette indépendance permet :
//!
//! - Publication indépendante sur crates.io (vérification :
//!   `cargo publish --dry-run -p kesh-import`).
//! - Réutilisation par d'autres projets Rust ayant besoin de parser des
//!   fichiers CAMT.053 ou CSV bancaires.
//! - Découplage de l'évolution du domaine Kesh par rapport au format de
//!   relevé bancaire (versions ISO 20022 multiples, profils CSV par banque).
//!
//! # Modules
//!
//! - [`types`] : types autonomes [`ImportedStatement`] / [`ImportedTransaction`]
//!   représentant un relevé importé.
//! - [`error`] : enum [`CamtError`] pour les défaillances du parseur CAMT.053.
//! - [`camt053`] : parseur CAMT.053 v04 / v08 (Story 8-1a).
//! - `csv` : parseur CSV multi-encodage / multi-profils (Story 8-2 — à venir).

pub mod camt053;
pub mod error;
pub mod types;

pub use camt053::parse as parse_camt053;
pub use error::CamtError;
pub use types::{ImportedStatement, ImportedTransaction, SourceFormat};
