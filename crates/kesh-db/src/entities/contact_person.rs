//! Entité `ContactPerson` (#213) — personne de contact rattachée à une entreprise.
//!
//! **Purement informatif** (CRM) : ces personnes ne sont jamais utilisées sur les
//! factures / QR-bill / pain.001. Une entreprise (`Contact` de type `Entreprise`)
//! peut en avoir 0..N. Archivage soft (`active`) cohérent avec `Contact`.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Personne de contact persistée.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ContactPerson {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création (valeurs déjà trimées/validées par le handler).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewContactPerson {
    pub company_id: i64,
    pub contact_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/// Données de mise à jour (remplacement complet ; `version` en paramètre séparé).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactPersonUpdate {
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}
