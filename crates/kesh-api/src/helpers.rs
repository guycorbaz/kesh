//! Helpers partagés — Story 6.2.
//!
//! Ce module fournit des fonctions utilitaires réutilisables par plusieurs
//! routes. Il est séparé des routes pour éviter la duplication et améliorer
//! la maintenabilité.

use kesh_db::entities::Company;
use kesh_db::repositories::companies;
use sqlx::MySqlPool;

use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Récupère la company de l'utilisateur courant.
///
/// Utilisé par les handlers pour charger la Company complète depuis le
/// `company_id` du JWT (via `CurrentUser.company_id`).
///
/// **Sémantique** :
/// - Retourne `Ok(Company)` si la company existe et correspond au JWT.
/// - Retourne `Err(AppError::Internal(...))` si la company n'existe pas
///   (situation défensive : le JWT porte un company_id orphelin, ce qui ne
///   devrait jamais arriver grâce à la FK RESTRICT de `users.company_id`).
///
/// Le 404 du scoping « resource not found in your company » est implémenté
/// par le handler lui-même, pas par ce helper.
pub async fn get_company_for(
    current_user: &CurrentUser,
    pool: &MySqlPool,
) -> Result<Company, AppError> {
    companies::find_by_id(pool, current_user.company_id)
        .await?
        .ok_or_else(|| {
            AppError::Internal(format!(
                "company_id {} from JWT not found in DB (user {} orphaned?)",
                current_user.company_id, current_user.user_id
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_db::entities::{OrgType, Language};

    /// Mock company pour les tests.
    fn mock_company(id: i64) -> Company {
        Company {
            id,
            name: "Test Company".to_string(),
            address: "Test Address".to_string(),
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
            version: 1,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    #[test]
    fn current_user_fields_accessible() {
        let user = CurrentUser {
            user_id: 123,
            role: kesh_db::entities::Role::Admin,
            company_id: 5,
        };
        assert_eq!(user.user_id, 123);
        assert_eq!(user.company_id, 5);
    }
}
