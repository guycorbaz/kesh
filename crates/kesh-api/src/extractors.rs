//! Extracteurs de corps JSON partagés — Story v016-2a (#816).
//!
//! La macro [`impl_validation_extractor`] vivait dans `routes/bank_accounts.rs`
//! (Story 8-5a-zero), locale à ce fichier. Elle en est **extraite**, sans
//! changement de comportement, pour servir aussi la saisie d'un relevé à la main
//! (`routes/bank_statements_manual.rs`) — décision de l'orchestrateur à la
//! validation P3 de la v016-2a (C-v016-2-19, a). Les deux autres copies du dépôt
//! (`routes/test_endpoints.rs`, `routes/reconciliation.rs`) sont tracées en #829.
//!
//! **Une seule règle** : tout rejet du corps — JSON illisible, champ requis absent
//! ou d'un autre type, `Content-Type` absent, corps au-delà de la limite
//! (`DefaultBodyLimit`) — rend `400 VALIDATION_ERROR` (`AppError::Validation`),
//! et non le `422` natif d'Axum 0.8, statut des refus métier.

/// Déclare un extracteur `$extractor(pub $body)` qui désérialise le corps JSON
/// en `$body` et convertit tout rejet en `AppError::Validation` (`400`).
/// `$field_doc` décrit les champs attendus dans le message d'erreur.
macro_rules! impl_validation_extractor {
    ($extractor:ident, $body:ty, $field_doc:expr) => {
        pub struct $extractor(pub $body);

        impl<S> ::axum::extract::FromRequest<S> for $extractor
        where
            S: Send + Sync,
        {
            type Rejection = ::axum::response::Response;

            async fn from_request(
                req: ::axum::extract::Request,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                use ::axum::extract::rejection::JsonRejection;
                use ::axum::response::IntoResponse;
                match ::axum::Json::<$body>::from_request(req, state).await {
                    Ok(::axum::Json(body)) => Ok(Self(body)),
                    Err(rej) => {
                        let message = match &rej {
                            JsonRejection::JsonDataError(_) | JsonRejection::JsonSyntaxError(_) => {
                                format!(
                                    "corps JSON malformé ou champ requis manquant ({})",
                                    $field_doc
                                )
                            }
                            JsonRejection::MissingJsonContentType(_) => {
                                "Content-Type attendu : application/json".to_string()
                            }
                            _ => {
                                tracing::warn!(
                                    rejection = %rej,
                                    "BodyExtractor: unhandled JsonRejection"
                                );
                                "requête invalide (corps non-parsable)".to_string()
                            }
                        };
                        Err($crate::errors::AppError::Validation(message).into_response())
                    }
                }
            }
        }
    };
}

pub(crate) use impl_validation_extractor;
