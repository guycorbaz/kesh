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

/// Rejette (400) un texte dépassant `max` **caractères** — un 400 explicite plutôt
/// qu'une erreur MariaDB brute (1406) au-delà de la borne de la colonne.
///
/// **Choisir `max`** : les colonnes `TEXT` bornent en **octets** (65 535), pas en
/// caractères. Tout `max` passé ici doit donc rester sous `65535 / 4` (pire cas
/// UTF-8, 4 octets/caractère) pour garantir l'insertion quel que soit le contenu.
///
/// Introduit en 21-5b (code review Pass 3) : `send_reminder` écrivait un `subject`
/// et un `body` non bornés dans `invoice_reminders` **après** l'envoi SMTP — un
/// corps trop long partait chez le débiteur puis échouait à l'INSERT, laissant
/// l'envoi sans trace et rejouable à l'identique. `field` nomme le champ dans le
/// message d'erreur (« objet », « corps », « note »).
pub fn validate_text_len(value: &str, max: usize, field: &str) -> Result<(), AppError> {
    if exceeds_len(value, max) {
        // Formulation sans adjectif : « {field} trop long » forcerait un accord que le
        // helper ne peut pas connaître (« note trop longue », « objet trop long »).
        return Err(AppError::Validation(format!(
            "{field} : {max} caractères maximum"
        )));
    }
    Ok(())
}

/// Prédicat de dépassement de longueur, en **caractères** (cf. [`validate_text_len`]
/// pour le choix de `max` face aux colonnes `TEXT`).
///
/// Existe pour que l'envoi par lot — dont l'échec est per-facture (`FailedProposal`)
/// et non une `AppError` — partage la **même** comparaison que la validation 400 de
/// l'envoi unitaire, plutôt que de la ré-implémenter et de dériver au prochain
/// changement de borne (review Pass 4).
pub fn exceeds_len(value: &str, max: usize) -> bool {
    value.chars().count() > max
}

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

    #[test]
    fn current_user_fields_accessible() {
        let user = CurrentUser {
            user_id: 123,
            role: kesh_db::entities::Role::Admin,
            company_id: 5,
            exp: 9_999_999_999, // Story 10-5 — exp claim added to CurrentUser
            api_key_id: None,   // Story 17-2a — chemin JWT
        };
        assert_eq!(user.user_id, 123);
        assert_eq!(user.company_id, 5);
    }
}
