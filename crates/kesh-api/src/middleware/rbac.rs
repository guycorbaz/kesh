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
