//! Erreur centrale de l'application et mapping HTTP.
//!
//! Toutes les fonctions du crate retournent `Result<T, AppError>`.
//! Le mapping `IntoResponse` transforme chaque variante en réponse
//! HTTP avec un code d'erreur structuré et un message générique côté
//! client (les détails internes vont exclusivement au logger).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use kesh_db::errors::DbError;
use serde::Serialize;
use thiserror::Error;

/// Erreurs applicatives de kesh-api.
#[derive(Debug, Error)]
pub enum AppError {
    /// Identifiants invalides au login (username inconnu, mot de passe
    /// incorrect, user inactif) — message générique pour éviter toute
    /// énumération d'utilisateurs.
    #[error("Identifiants invalides")]
    InvalidCredentials,

    /// JWT manquant, mal formé, expiré ou signature invalide.
    /// Le `String` porte le détail pour les logs, jamais le client.
    #[error("Non authentifié : {0}")]
    Unauthenticated(String),

    /// Erreur de validation des données entrantes (400).
    #[error("Validation : {0}")]
    Validation(String),

    /// Erreur interne du serveur (bug, PHC mal formé, config invalide).
    #[error("Erreur interne : {0}")]
    Internal(String),

    /// Erreur remontée depuis la couche de persistance `kesh-db`.
    ///
    /// Le `#[from]` est légitime ici : la classification
    /// sqlx::Error → DbError a déjà eu lieu au niveau kesh-db. On se
    /// contente de wrapper pour le mapping HTTP.
    #[error("Erreur base de données : {0}")]
    Database(#[from] DbError),

    // --- Story 1.7 ---

    /// Accès interdit — rôle insuffisant (403).
    #[error("Accès interdit")]
    Forbidden,

    /// L'administrateur tente de désactiver son propre compte (400).
    #[error("Impossible de désactiver son propre compte")]
    CannotDisableSelf,

    /// Tentative de désactivation du dernier administrateur actif (400).
    #[error("Impossible de désactiver le dernier administrateur")]
    CannotDisableLastAdmin,

    // --- Story 1.6 ---

    /// Rate limiting déclenché : trop de tentatives de login depuis cette IP.
    /// `retry_after` = secondes avant déblocage, transmis dans le header `Retry-After`.
    #[error("Rate limited, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    /// Refresh token invalide (expiré, révoqué, inconnu, user inactif).
    /// Code client unique `INVALID_REFRESH_TOKEN` (anti-enumeration).
    /// Le `String` porte le détail pour les logs serveur.
    #[error("Refresh token invalide : {0}")]
    InvalidRefreshToken(String),
}

/// Structure de la réponse d'erreur JSON renvoyée au client.
#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

/// Helper pour construire une `Response` JSON structurée.
fn build_response(status: StatusCode, code: &'static str, message: &str) -> Response {
    (
        status,
        Json(ErrorBody {
            error: ErrorDetail {
                code,
                message: message.to_string(),
            },
        }),
    )
        .into_response()
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InvalidCredentials => build_response(
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "Identifiants invalides",
            ),

            AppError::Unauthenticated(detail) => {
                tracing::warn!("unauth: {detail}");
                build_response(
                    StatusCode::UNAUTHORIZED,
                    "UNAUTHENTICATED",
                    "Non authentifié",
                )
            }

            AppError::Validation(msg) => {
                build_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &msg)
            }

            AppError::Forbidden => {
                build_response(StatusCode::FORBIDDEN, "FORBIDDEN", "Accès interdit")
            }

            AppError::CannotDisableSelf => build_response(
                StatusCode::BAD_REQUEST,
                "CANNOT_DISABLE_SELF",
                "Impossible de désactiver son propre compte",
            ),

            AppError::CannotDisableLastAdmin => build_response(
                StatusCode::BAD_REQUEST,
                "CANNOT_DISABLE_LAST_ADMIN",
                "Impossible de désactiver le dernier administrateur",
            ),

            AppError::Internal(detail) => {
                tracing::error!("internal: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Erreur interne",
                )
            }

            AppError::RateLimited { retry_after } => {
                let mut resp = build_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    "RATE_LIMITED",
                    "Trop de tentatives",
                );
                resp.headers_mut().insert(
                    "Retry-After",
                    axum::http::HeaderValue::from_str(&retry_after.to_string())
                        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("60")),
                );
                resp
            }

            AppError::InvalidRefreshToken(detail) => {
                tracing::warn!("invalid refresh token: {detail}");
                build_response(
                    StatusCode::UNAUTHORIZED,
                    "INVALID_REFRESH_TOKEN",
                    "Session expirée",
                )
            }

            // Sous-match exhaustif sur DbError : pas de `_ =>` catch-all,
            // l'ajout futur d'une variante kesh-db casse la compilation
            // ici (propriété désirée).
            AppError::Database(db_err) => match db_err {
                DbError::NotFound => {
                    build_response(StatusCode::NOT_FOUND, "NOT_FOUND", "Ressource introuvable")
                }
                DbError::OptimisticLockConflict => build_response(
                    StatusCode::CONFLICT,
                    "OPTIMISTIC_LOCK_CONFLICT",
                    "Conflit de version — la ressource a été modifiée",
                ),
                DbError::UniqueConstraintViolation(m) => {
                    tracing::warn!("unique violation: {m}");
                    build_response(
                        StatusCode::CONFLICT,
                        "RESOURCE_CONFLICT",
                        "Ressource déjà existante",
                    )
                }
                DbError::ForeignKeyViolation(m) => {
                    tracing::warn!("fk violation: {m}");
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "FOREIGN_KEY_VIOLATION",
                        "Référence invalide",
                    )
                }
                DbError::CheckConstraintViolation(m) => {
                    tracing::warn!("check violation: {m}");
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "CHECK_CONSTRAINT_VIOLATION",
                        "Valeur invalide",
                    )
                }
                DbError::IllegalStateTransition(m) => {
                    tracing::warn!("illegal state: {m}");
                    build_response(
                        StatusCode::CONFLICT,
                        "ILLEGAL_STATE_TRANSITION",
                        "Transition d'état interdite",
                    )
                }
                DbError::ConnectionUnavailable(m) => {
                    tracing::warn!("db connection unavailable: {m}");
                    build_response(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "SERVICE_UNAVAILABLE",
                        "Service temporairement indisponible",
                    )
                }
                DbError::Invariant(m) => {
                    tracing::error!("db invariant violated: {m}");
                    build_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        "Erreur interne",
                    )
                }
                DbError::Sqlx(e) => {
                    tracing::error!("sqlx: {e}");
                    build_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        "Erreur interne",
                    )
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    async fn response_body(resp: Response) -> (StatusCode, serde_json::Value) {
        let (parts, body) = resp.into_parts();
        let bytes = body.collect().await.expect("body collect").to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).expect("body should be JSON");
        (parts.status, json)
    }

    #[tokio::test]
    async fn invalid_credentials_maps_to_401() {
        let resp = AppError::InvalidCredentials.into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
        assert_eq!(body["error"]["message"], "Identifiants invalides");
    }

    #[tokio::test]
    async fn unauthenticated_maps_to_401_with_generic_message() {
        let resp =
            AppError::Unauthenticated("detailed internal info".to_string()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "UNAUTHENTICATED");
        // Le détail interne ne doit pas leak
        let message = body["error"]["message"].as_str().unwrap();
        assert!(
            !message.contains("detailed internal info"),
            "detail leaked in response: {}",
            message
        );
    }

    #[tokio::test]
    async fn validation_maps_to_400() {
        let resp = AppError::Validation("username must not be empty".into()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn internal_maps_to_500_with_generic_message() {
        let resp = AppError::Internal("stack trace details".to_string()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "INTERNAL_ERROR");
        let message = body["error"]["message"].as_str().unwrap();
        assert!(!message.contains("stack trace"));
    }

    #[tokio::test]
    async fn db_not_found_maps_to_404() {
        let resp = AppError::Database(DbError::NotFound).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "NOT_FOUND");
    }

    #[tokio::test]
    async fn db_optimistic_lock_maps_to_409() {
        let resp = AppError::Database(DbError::OptimisticLockConflict).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    }

    #[tokio::test]
    async fn db_connection_unavailable_maps_to_503() {
        let resp = AppError::Database(DbError::ConnectionUnavailable("timeout".into()))
            .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], "SERVICE_UNAVAILABLE");
    }

    #[tokio::test]
    async fn db_unique_constraint_maps_to_409() {
        let resp =
            AppError::Database(DbError::UniqueConstraintViolation("dup".into())).into_response();
        let (status, _) = response_body(resp).await;
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn db_check_constraint_maps_to_400() {
        let resp =
            AppError::Database(DbError::CheckConstraintViolation("bad".into())).into_response();
        let (status, _) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
