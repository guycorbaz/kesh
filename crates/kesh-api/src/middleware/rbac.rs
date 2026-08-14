//! Middleware RBAC : vérification du rôle minimum requis.
//!
//! Fonctions nommées par niveau de rôle, utilisées via `axum::middleware::from_fn`.
//! Doit être appliqué APRÈS `require_auth` (qui injecte `CurrentUser` dans les extensions).

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use kesh_db::entities::Role;

use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Vérifie que l'utilisateur courant a au moins le rôle `min_role`.
///
/// Retourne `AppError::Unauthenticated` si `CurrentUser` n'est pas dans les extensions
/// (ne devrait jamais arriver si `require_auth` est appliqué en amont).
/// Retourne `AppError::Forbidden` si le rôle est insuffisant.
fn check_role(req: &Request, min_role: Role) -> Result<(), AppError> {
    let current_user = req
        .extensions()
        .get::<CurrentUser>()
        .ok_or_else(|| AppError::Unauthenticated("missing CurrentUser in extensions".into()))?;
    if current_user.role < min_role {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

/// Middleware : requiert au minimum le rôle `Admin`.
pub async fn require_admin_role(req: Request, next: Next) -> Result<Response, AppError> {
    check_role(&req, Role::Admin)?;
    Ok(next.run(req).await)
}

/// Middleware : requiert au minimum le rôle `Comptable` (Admin hérite).
pub async fn require_comptable_role(req: Request, next: Next) -> Result<Response, AppError> {
    check_role(&req, Role::Comptable)?;
    Ok(next.run(req).await)
}

/// Middleware : refuse toute requête authentifiée par clé API (PAT).
///
/// Story 22-4a (#167). Posé en `route_layer` sur `admin_routes`, il ferme le
/// contournement qui rendait une révocation de clé inopérante : un PAT
/// `read-write` créé par un Admin pouvait créer un nouvel administrateur, s'y
/// connecter par l'interface, et se forger de nouvelles clés.
///
/// **Le discriminant est `api_key_id`, jamais le rôle** : un Admin devant son
/// navigateur porte un `CurrentUser` dont `api_key_id` vaut `None` (chemin JWT)
/// et n'est donc pas affecté.
///
/// **Le code rendu est le même quel que soit le rôle du créateur de la clé**
/// (décision D6) : un PAT créé par un Comptable reçoit lui aussi
/// `API_KEY_ADMIN_FORBIDDEN`, et non le `Forbidden` du RBAC. C'est la précédence
/// obtenue en posant cette couche **après** [`require_admin_role`] — chaque
/// `route_layer` enveloppant le précédent, la dernière posée répond la première.
pub async fn require_not_pat(req: Request, next: Next) -> Result<Response, AppError> {
    let current_user = req
        .extensions()
        .get::<CurrentUser>()
        .ok_or_else(|| AppError::Unauthenticated("missing CurrentUser in extensions".into()))?;
    if current_user.api_key_id.is_some() {
        return Err(AppError::ApiKeyAdminForbidden);
    }
    Ok(next.run(req).await)
}
