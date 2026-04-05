//! Authentification : hash de mot de passe, JWT, bootstrap admin.
//!
//! Story 1.5. Les handlers HTTP vivent dans `routes/auth.rs`, le
//! middleware d'extraction JWT dans `middleware/auth.rs`. Ce module
//! regroupe la logique pure (pas de HTTP).

pub mod bootstrap;
pub mod jwt;
pub mod password;
