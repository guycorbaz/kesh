//! Story 17-3a — Export complet d'installation Kesh au format `.keshbackup`.
//!
//! ⚠️ **Le `.keshbackup` est un secret** : il contient l'intégralité de la base
//! (dont `users.password_hash` et `refresh_tokens`). Il n'est **pas chiffré**
//! par défaut (responsabilité utilisateur — recommander GPG/age pour tout
//! transit hors infra contrôlée). Le SHA-256 du manifeste sert à la détection
//! d'altération, pas à la confidentialité.
//!
//! - [`manifest`] : schéma `manifest.json` + (dé)sérialisation.
//! - [`export`] : cœur `build_keshbackup` (assemblage ZIP sans audit).
//! - [`import`] : lecture + validation d'un `.keshbackup` au restore (17-3c).

pub mod export;
pub mod import;
pub mod manifest;
