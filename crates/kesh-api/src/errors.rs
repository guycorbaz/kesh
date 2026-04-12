//! Erreur centrale de l'application et mapping HTTP.
//!
//! Toutes les fonctions du crate retournent `Result<T, AppError>`.
//! Le mapping `IntoResponse` transforme chaque variante en réponse
//! HTTP avec un code d'erreur structuré et un message générique côté
//! client (les détails internes vont exclusivement au logger).

use std::sync::RwLock;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use kesh_db::errors::DbError;
use kesh_i18n::{I18nBundle, Locale};
use serde::Serialize;
use thiserror::Error;

/// Bundle i18n global pour les messages d'erreur.
/// `RwLock` au lieu de `OnceLock` pour permettre la réinitialisation en tests.
static I18N: RwLock<Option<(std::sync::Arc<I18nBundle>, Locale)>> = RwLock::new(None);

/// Initialise (ou remplace) le bundle i18n global pour les messages d'erreur.
pub fn init_error_i18n(bundle: std::sync::Arc<I18nBundle>, locale: Locale) {
    let mut guard = I18N.write().expect("I18N write lock");
    *guard = Some((bundle, locale));
}

/// Résout un message d'erreur via i18n, avec fallback sur le message par défaut.
fn t(key: &str, default: &str) -> String {
    let guard = I18N.read().expect("I18N read lock");
    match guard.as_ref() {
        Some((bundle, locale)) => bundle.format(locale, key, None),
        None => default.to_string(),
    }
}

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

    // --- Story 2.2 ---
    /// Tentative de progression sur un step d'onboarding déjà complété (400).
    #[error("Étape d'onboarding déjà complétée")]
    OnboardingStepAlreadyCompleted,

    // --- Story 3.2 ---
    /// Écriture comptable déséquilibrée (FR21).
    /// Les totaux (format string décimal) sont inclus dans le message
    /// client pour respecter exactement le wording du PRD.
    #[error("Écriture déséquilibrée : débits={debit}, crédits={credit}")]
    EntryUnbalanced {
        /// Total des débits formaté en string décimal.
        debit: String,
        /// Total des crédits formaté en string décimal.
        credit: String,
    },

    /// Aucun exercice comptable n'existe pour la date fournie.
    /// À distinguer de `FiscalYearClosed` pour l'UX : le message invite
    /// l'utilisateur à créer un exercice plutôt qu'à chercher un exercice
    /// existant fermé.
    #[error("Aucun exercice pour la date {date}")]
    NoFiscalYear {
        /// Date au format ISO (YYYY-MM-DD).
        date: String,
    },

    /// L'exercice pour cette date est clôturé (FR24, CO art. 957-964).
    /// Aucune écriture ne peut être ajoutée ou modifiée dans un exercice clos.
    #[error("Exercice clôturé pour la date {date}")]
    FiscalYearClosed {
        /// Date au format ISO (YYYY-MM-DD).
        date: String,
    },

    /// La nouvelle date d'une écriture ne tombe pas dans l'exercice courant
    /// de l'entité (story 3.3). Empêche le déplacement cross-exercice via
    /// simple édition.
    #[error("Date hors exercice courant : {date}")]
    DateOutsideFiscalYear {
        /// Date au format ISO (YYYY-MM-DD).
        date: String,
    },

    // --- Story 4.1 ---
    /// Un contact avec ce numéro IDE (CHE) existe déjà dans la même company.
    /// Code client dédié (`IDE_ALREADY_EXISTS`) pour UX précise côté form,
    /// distinct du générique `RESOURCE_CONFLICT` (autres UniqueConstraintViolation).
    /// Le `String` porte le message i18n prêt à afficher.
    #[error("{0}")]
    IdeAlreadyExists(String),
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
                &t("error-invalid-credentials", "Identifiants invalides"),
            ),

            AppError::Unauthenticated(detail) => {
                tracing::warn!("unauth: {detail}");
                build_response(
                    StatusCode::UNAUTHORIZED,
                    "UNAUTHENTICATED",
                    &t("error-unauthenticated", "Non authentifié"),
                )
            }

            AppError::Validation(msg) => {
                build_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &msg)
            }

            AppError::Forbidden => build_response(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                &t("error-forbidden", "Accès interdit"),
            ),

            AppError::CannotDisableSelf => build_response(
                StatusCode::BAD_REQUEST,
                "CANNOT_DISABLE_SELF",
                &t(
                    "error-cannot-disable-self",
                    "Impossible de désactiver son propre compte",
                ),
            ),

            AppError::CannotDisableLastAdmin => build_response(
                StatusCode::BAD_REQUEST,
                "CANNOT_DISABLE_LAST_ADMIN",
                &t(
                    "error-cannot-disable-last-admin",
                    "Impossible de désactiver le dernier administrateur",
                ),
            ),

            AppError::Internal(detail) => {
                tracing::error!("internal: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    &t("error-internal", "Erreur interne"),
                )
            }

            AppError::RateLimited { retry_after } => {
                let mut resp = build_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    "RATE_LIMITED",
                    &t("error-rate-limited", "Trop de tentatives"),
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
                    &t("error-invalid-refresh-token", "Session expirée"),
                )
            }

            AppError::OnboardingStepAlreadyCompleted => build_response(
                StatusCode::BAD_REQUEST,
                "ONBOARDING_STEP_ALREADY_COMPLETED",
                &t(
                    "error-onboarding-step-already-completed",
                    "Cette étape de configuration a déjà été complétée",
                ),
            ),

            AppError::EntryUnbalanced { debit, credit } => {
                // FR21 : le wording exact vient du PRD. La version i18n
                // inclut les placeholders via Fluent ; à défaut, on
                // construit la version française à la volée.
                let fallback = format!(
                    "Écriture déséquilibrée — le total des débits ({debit}) ne correspond pas au total des crédits ({credit})"
                );
                build_response(StatusCode::BAD_REQUEST, "ENTRY_UNBALANCED", &fallback)
            }

            AppError::NoFiscalYear { date } => {
                let fallback = format!(
                    "Aucun exercice n'existe pour la date {date}. Créez un exercice comptable avant de saisir des écritures."
                );
                build_response(StatusCode::BAD_REQUEST, "NO_FISCAL_YEAR", &fallback)
            }

            AppError::FiscalYearClosed { date } => {
                let fallback = format!(
                    "L'exercice pour la date {date} est clôturé — aucune écriture ne peut y être ajoutée ou modifiée (CO art. 957-964)."
                );
                build_response(StatusCode::BAD_REQUEST, "FISCAL_YEAR_CLOSED", &fallback)
            }

            AppError::DateOutsideFiscalYear { date } => {
                let fallback =
                    format!("La date {date} n'est pas dans l'exercice courant de cette écriture.");
                build_response(
                    StatusCode::BAD_REQUEST,
                    "DATE_OUTSIDE_FISCAL_YEAR",
                    &fallback,
                )
            }

            // Story 4.1 : code dédié pour l'unicité IDE par company.
            AppError::IdeAlreadyExists(msg) => {
                build_response(StatusCode::CONFLICT, "IDE_ALREADY_EXISTS", &msg)
            }

            // Sous-match exhaustif sur DbError : pas de `_ =>` catch-all,
            // l'ajout futur d'une variante kesh-db casse la compilation
            // ici (propriété désirée).
            AppError::Database(db_err) => match db_err {
                DbError::NotFound => build_response(
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    &t("error-not-found", "Ressource introuvable"),
                ),
                DbError::OptimisticLockConflict => build_response(
                    StatusCode::CONFLICT,
                    "OPTIMISTIC_LOCK_CONFLICT",
                    &t(
                        "error-optimistic-lock",
                        "Conflit de version — la ressource a été modifiée",
                    ),
                ),
                DbError::UniqueConstraintViolation(m) => {
                    tracing::warn!("unique violation: {m}");
                    build_response(
                        StatusCode::CONFLICT,
                        "RESOURCE_CONFLICT",
                        &t("error-conflict", "Ressource déjà existante"),
                    )
                }
                DbError::ForeignKeyViolation(m) => {
                    tracing::warn!("fk violation: {m}");
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "FOREIGN_KEY_VIOLATION",
                        &t("error-foreign-key", "Référence invalide"),
                    )
                }
                DbError::CheckConstraintViolation(m) => {
                    tracing::warn!("check violation: {m}");
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "CHECK_CONSTRAINT_VIOLATION",
                        &t("error-check-constraint", "Valeur invalide"),
                    )
                }
                DbError::IllegalStateTransition(m) => {
                    tracing::warn!("illegal state: {m}");
                    build_response(
                        StatusCode::CONFLICT,
                        "ILLEGAL_STATE_TRANSITION",
                        &t("error-illegal-state", "Transition d'état interdite"),
                    )
                }
                DbError::FiscalYearClosed => build_response(
                    StatusCode::BAD_REQUEST,
                    "FISCAL_YEAR_CLOSED",
                    &t(
                        "error-fiscal-year-closed-generic",
                        "L'exercice comptable est clôturé — aucune écriture ne peut y être ajoutée ou modifiée (CO art. 957-964).",
                    ),
                ),
                DbError::InactiveOrInvalidAccounts => build_response(
                    StatusCode::BAD_REQUEST,
                    "INACTIVE_OR_INVALID_ACCOUNTS",
                    &t(
                        "error-inactive-accounts",
                        "Un ou plusieurs comptes sont archivés ou invalides.",
                    ),
                ),
                DbError::DateOutsideFiscalYear => build_response(
                    StatusCode::BAD_REQUEST,
                    "DATE_OUTSIDE_FISCAL_YEAR",
                    &t(
                        "error-date-outside-fiscal-year-generic",
                        "La date n'est pas dans l'exercice courant de cette écriture.",
                    ),
                ),
                DbError::ConnectionUnavailable(m) => {
                    tracing::warn!("db connection unavailable: {m}");
                    build_response(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "SERVICE_UNAVAILABLE",
                        &t(
                            "error-service-unavailable",
                            "Service temporairement indisponible",
                        ),
                    )
                }
                DbError::Invariant(m) => {
                    tracing::error!("db invariant violated: {m}");
                    build_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        &t("error-internal", "Erreur interne"),
                    )
                }
                DbError::Sqlx(e) => {
                    tracing::error!("sqlx: {e}");
                    build_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        &t("error-internal", "Erreur interne"),
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
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should be JSON");
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
        let resp = AppError::Unauthenticated("detailed internal info".to_string()).into_response();
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
        let resp =
            AppError::Database(DbError::ConnectionUnavailable("timeout".into())).into_response();
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
