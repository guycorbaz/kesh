//! Le service du plan comptable — le pilote « simple » du motif : une lecture
//! sans trace (aujourd'hui aucune), qui rend directement (règle R5).

use kesh_db::entities::Role;
use kesh_db::entities::account::Account;
use kesh_db::errors::map_db_error;
use kesh_db::repositories::{accounts, letterings};

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Un compte du plan, avec sa lettrabilité.
#[derive(Debug, Clone)]
pub struct CompteDuPlan {
    /// Le compte.
    pub compte: Account,
    /// Le compte peut-il porter des lettrages (Story 15-1b) ?
    pub lettrable: bool,
    /// La devise du compte est-elle verrouillée (Story 37-1, AC-4) : le compte
    /// porte au moins une ligne d'écriture.
    pub devise_verrouillee: bool,
}

/// Exigence de [`lister`] : tous rôles (`GET /api/v1/accounts`,
/// `authenticated_routes`) ; portée MCP `BooksRead`.
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::BooksRead),
};

/// Les comptes du plan de la société du contexte, archivés compris sur demande.
///
/// Dans l'ordre d'aujourd'hui : la société est lue d'abord (erreur `Internal` si
/// elle manque — jeton orphelin, cf. `helpers::get_company_for`), puis les
/// comptes, puis la lettrabilité de tout le plan en **une** requête, puis le
/// verrou de devise de tout le plan en une requête (Story 37-1).
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &Lecture,
    inclure_archives: bool,
) -> Result<Vec<CompteDuPlan>, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    // Société existante (défensif : fenêtre d'obsolescence du company_id).
    let _ = crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
        .await?;

    let list =
        accounts::list_by_company(lecture.pool(), acces.company_id(), inclure_archives).await?;
    // Story 15-1b (AC11) : la lettrabilité de tout le plan en UNE requête.
    let mut conn = lecture.pool().acquire().await.map_err(map_db_error)?;
    let lettrables = letterings::letterable_account_ids(&mut conn, acces.company_id()).await?;
    // Story 37-1 (AC-4) : le verrou de devise de tout le plan, en UNE requête.
    let verrouilles = accounts::currency_locked_account_ids(&mut conn, acces.company_id()).await?;
    Ok(list
        .into_iter()
        .map(|compte| {
            let lettrable = lettrables.contains(&compte.id);
            let devise_verrouillee = verrouilles.contains(&compte.id);
            CompteDuPlan {
                compte,
                lettrable,
                devise_verrouillee,
            }
        })
        .collect())
}
