//! Projet analytique (Epic 19, Story 19-1) — dimension de comptabilité analytique.
//!
//! Un projet regroupe des dépenses/revenus pour les analyser isolément (rénovation
//! déductible, investissement à rendement). Hiérarchie à **2 niveaux** : un projet
//! racine (`parent_id IS NULL`) peut avoir des sous-projets ; un sous-projet ne peut
//! pas lui-même en avoir (contrainte appliquée côté repo). Scopé `company_id`.

use chrono::{NaiveDate, NaiveDateTime};

/// Projet analytique persisté, scopé `company_id`.
///
/// **Pas de dérivation `Serialize`** (cf. `VatRate`) : toute exposition REST passe
/// par la projection `routes/projects::ProjectResponse` pour ne pas fuiter
/// `company_id` au client.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Project {
    pub id: i64,
    pub company_id: i64,
    /// `None` = projet racine ; sinon id du projet parent (toujours une racine).
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un projet.
#[derive(Debug, Clone)]
pub struct NewProject {
    pub company_id: i64,
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

/// Champs modifiables d'un projet existant (l'archivage a ses propres endpoints).
#[derive(Debug, Clone)]
pub struct UpdateProject {
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}
