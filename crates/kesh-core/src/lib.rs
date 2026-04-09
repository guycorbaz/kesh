//! kesh-core — Logique métier pure, zéro I/O.
//!
//! Ce crate contient les types domaine, les validations métier et les règles
//! comptables. Il n'a aucune dépendance sur la base de données, le réseau
//! ou le filesystem.

pub mod chart_of_accounts;
pub mod errors;
pub mod types;
