//! Story 17-3a — Export complet d'installation Kesh au format `.keshbackup`.
//!
//! ⚠️ **Le `.keshbackup` est un secret** : il contient l'intégralité de la base
//! (dont `users.password_hash` et `refresh_tokens`). Il n'est **pas chiffré**
//! par défaut (responsabilité utilisateur — recommander GPG/age pour tout
//! transit hors infra contrôlée). Le SHA-256 du manifeste sert à la détection
//! d'altération, pas à la confidentialité.
//!
//! - [`manifest`] : schéma `manifest.json` + (dé)sérialisation.
//! - [`export`] : cœur `build_keshbackup` (lecture de la base, puis écriture
//!   de l'archive par `write_keshbackup`, pièces comprises — format 2, Story
//!   31-1a ; sans audit).
//! - [`import`] : lecture + validation d'un `.keshbackup` au restore (17-3c),
//!   formats 1 et 2, et restauration des pièces (31-1a).
//!
//! Le `.keshbackup` de format 2 emporte aussi les **pièces** du dossier des
//! documents référencées par la base (documents fournisseur, PDF figés) : il
//! est tout autant un secret.

pub mod export;
pub mod import;
pub mod manifest;
