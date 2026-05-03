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
//! # Statut spike (2026-05-03)
//!
//! Cette première version contient uniquement les types autonomes
//! [`types::ImportedStatement`] et [`types::ImportedTransaction`].
//! Les modules `camt053` et `csv` (parseurs effectifs) seront ajoutés par
//! les Stories 8-1 (CAMT.053) et 8-2 (CSV) — cf.
//! `_bmad-output/implementation-artifacts/spike-kesh-import.md`.

pub mod types;

pub use types::{ImportedStatement, ImportedTransaction, SourceFormat};
