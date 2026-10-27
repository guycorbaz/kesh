//! Le service des exercices comptables — la liste (Story 32-9a, refs #723),
//! lue par `GET /api/v1/fiscal-years`, puis par l'outil MCP `kesh_exercices`.
//!
//! Une lecture **sans trace** : elle rend directement (règle R5). Ni la
//! société n'est lue d'abord, ni la liste bornée ici : le comportement de la
//! route d'avant la 32-9a est conservé (le dépôt filtre par société et borne
//! lui-même la liste, `MAX_LIST_LIMIT`). `list_by_company` ne fait que lire
//! (`SELECT … WHERE company_id = ? ORDER BY start_date DESC LIMIT ?`).

use kesh_db::entities::{FiscalYear, Role};
use kesh_db::repositories::fiscal_years;

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Exigence de [`lister`] : tous rôles (`GET /api/v1/fiscal-years`,
/// `authenticated_routes`) ; portée MCP `BooksRead`.
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::BooksRead),
};

/// Les exercices de la société du contexte, du plus récent au plus ancien.
pub async fn lister(acces: &ContexteAcces, lecture: &Lecture) -> Result<Vec<FiscalYear>, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    Ok(fiscal_years::list_by_company(lecture.pool(), acces.company_id()).await?)
}
