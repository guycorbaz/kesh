//! Le service des postes ouverts d'un compte (Story 32-4a), lu par
//! `GET /api/v1/accounts/{id}/open-items`, puis par l'outil MCP de la 32-9a.
//!
//! Une lecture **sans trace** : elle rend directement (règle R5). L'analyse de
//! `asOf` et les bornes de pagination vivent ici (C-32-4a-5) ; le dépôt
//! (`letterings::open_items`) n'écrête rien — son doc-comment le dit : la borne
//! de ce service est la **seule**.

use chrono::NaiveDate;
use kesh_db::entities::Role;
use kesh_db::errors::map_db_error;
use kesh_db::repositories::letterings;

/// Réexportés pour l'outil MCP de la 32-9a (C-32-9-25) : `src/mcp/` ne nomme
/// pas `kesh_db::repositories`.
pub use kesh_db::repositories::journal_entries::{DocumentKind, DocumentOwner};
pub use kesh_db::repositories::letterings::{
    DocumentState, OpenItem, OpenItems, OpenReason, Origin,
};

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Taille de page par défaut de [`lister`] (patron de la liste des écritures).
pub const LIMITE_DEFAUT: i64 = 50;
/// Plafond de la taille de page — **commun** aux postes ouverts et aux
/// propositions de lettrage (`routes::letterings::get_lettering_proposals`,
/// qui l'importe d'ici : une seule constante, C-32-4a-5).
pub const LIMITE_MAX: i64 = 500;

/// Exigence de [`lister`] : tous rôles (`GET /api/v1/accounts/{id}/open-items`,
/// `authenticated_routes`) ; portée MCP `BooksRead`.
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::BooksRead),
};

/// La demande de [`lister`].
#[derive(Debug, Clone, Default)]
pub struct DemandePostesOuverts {
    /// Le compte dont on lit les postes ouverts.
    pub account_id: i64,
    /// `AAAA-MM-JJ` ; absente ou vide → aujourd'hui (UTC).
    pub as_of: Option<String>,
    /// Taille de page ; absente → [`LIMITE_DEFAUT`], ramenée entre 1 et
    /// [`LIMITE_MAX`].
    pub limit: Option<i64>,
    /// Décalage ; absent → 0, ramené à 0 s'il est négatif.
    pub offset: Option<i64>,
}

/// `asOf` : lu en chaîne, parsé comme `dateFrom` de la liste des écritures ;
/// mal formé → 400 `VALIDATION_ERROR` ; absent ou vide → aujourd'hui, selon la
/// convention de la balance âgée (`Utc::now().naive_utc().date()` — l'écart
/// UTC/heure suisse la nuit est hérité). Aucune borne métier ; seule la plage
/// des dates de MariaDB (années 1000 à 9999) est exigée : chrono lit aussi
/// `+10000-01-01` ou une année négative, que sqlx refuse d'encoder ou que la
/// base lirait comme une date invalide (revue P1, B-1 = E-1) — un 400, jamais
/// un 500 ni un solde faux.
fn parse_as_of(as_of: Option<&str>) -> Result<NaiveDate, AppError> {
    use chrono::Datelike;
    match as_of {
        Some(s) if !s.is_empty() => {
            let date = s
                .parse::<NaiveDate>()
                .map_err(|e| AppError::Validation(format!("asOf invalide ({e})")))?;
            if !(1000..=9999).contains(&date.year()) {
                return Err(AppError::Validation(
                    "asOf invalide (année hors de 1000 à 9999)".into(),
                ));
            }
            Ok(date)
        }
        _ => Ok(chrono::Utc::now().naive_utc().date()),
    }
}

/// Les postes ouverts du compte de la demande à `asOf` (Story 15-1b, AC1).
///
/// Ordre des refus (AC7 de la 15-1b) : `asOf` (400) **avant toute lecture** ;
/// puis la société (`Internal` si elle manque) ; compte introuvable ou d'une
/// autre société (`AppError::Database(DbError::NotFound)`, 404) ; compte non
/// lettrable (`DbError::LetteringAccountNotLetterable`, 409). Le dépôt ouvre sa
/// propre transaction de lecture sur la connexion prêtée.
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandePostesOuverts,
) -> Result<OpenItems, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    let as_of = parse_as_of(demande.as_of.as_deref())?;
    let limit = demande.limit.unwrap_or(LIMITE_DEFAUT).clamp(1, LIMITE_MAX);
    let offset = demande.offset.unwrap_or(0).max(0);
    let company =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;
    let mut conn = lecture.pool().acquire().await.map_err(map_db_error)?;
    Ok(letterings::open_items(
        &mut conn,
        company.id,
        demande.account_id,
        as_of,
        limit,
        offset,
    )
    .await?)
}
