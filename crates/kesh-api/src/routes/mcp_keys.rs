//! Routes de gestion des clés du serveur MCP — `/api/v1/admin/mcp/keys`
//! (Story 32-5c, refs #723).
//!
//! - `GET    /api/v1/admin/mcp/keys` — toutes les clés de la société,
//!   révoquées comprises, les plus récentes d'abord ;
//! - `POST   /api/v1/admin/mcp/keys` — crée une clé, rend le secret
//!   `kesh_mcp_…` **une seule fois** (201) ; audit `mcp_key.created` ;
//! - `DELETE /api/v1/admin/mcp/keys/{id}` — révocation, verrou optimiste
//!   (`{ "version": n }`), 404 pour une clé d'une autre société ; audit
//!   `mcp_key.revoked` ;
//! - `PUT    /api/v1/admin/mcp/keys/{id}/send-validation` — politique de
//!   validation avant envoi (Q12), verrou optimiste ; audit
//!   `mcp_key.send_validation_changed` (avant, après).
//!
//! **Gardes** (C-32-5-17) : les quatre routes vivent dans le bloc
//! `admin_routes` de `lib.rs` — `require_admin_role` (un Comptable reçoit
//! `403 FORBIDDEN`) puis `require_not_pat` (un PAT reçoit
//! `403 API_KEY_ADMIN_FORBIDDEN`, quel que soit son créateur). Une clé MCP n'y
//! arrive jamais : `require_auth` ne la reconnaît pas (401, Story 32-5b). Les
//! handlers ne portent donc pas `ensure_not_pat` : la couche suffit, et le test
//! `admin_pat_denied_e2e` le prouve route par route.
//!
//! **Secret** : le jeton n'existe qu'en mémoire, le temps de la réponse de
//! création ; seul son condensat est persisté. Aucune réponse de liste, aucun
//! détail d'audit ne porte ni l'un ni l'autre ([`McpKeyResponse`] n'a pas de
//! champ pour eux).
//!
//! **Validation** (AC-c3) : tout refus rend 400 `VALIDATION_ERROR`, avant
//! toute écriture et sans audit ; les messages passent par `t`/`t_args`, repli
//! égal à la valeur fr-CH (garde G9 de `textes_coherents.rs`).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use kesh_db::entities::SendValidation;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::mcp_key::{McpKey, McpScope, McpScopeError, McpScopes, NewMcpKey};
use kesh_db::errors::DbError;
use kesh_db::repositories::{audit_log, mcp_keys};
use kesh_i18n::FluentArgs;

use crate::AppState;
use crate::auth::api_key::generate_mcp_key;
use crate::errors::{AppError, t, t_args};
use crate::middleware::auth::CurrentUser;

/// Longueur maximale du nom d'une clé, en caractères (`mcp_keys.name`,
/// `VARCHAR(255)`).
pub const NAME_MAX_CHARS: usize = 255;

/// Une clé telle que la liste et les modifications la rendent — **jamais** le
/// condensat ni le secret.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpKeyResponse {
    pub id: i64,
    pub name: String,
    /// Portées connues de cette version, dans l'ordre canonique.
    pub scopes: Vec<&'static str>,
    /// Portées stockées que cette version ne connaît pas (C-32-5-37) : elles ne
    /// sont jamais accordées ; une clé qui n'a qu'elles n'authentifie pas.
    pub unknown_scopes: Vec<String>,
    pub send_validation: &'static str,
    pub created_by_user_id: i64,
    pub created_at: NaiveDateTime,
    pub last_used_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub version: i32,
}

impl From<McpKey> for McpKeyResponse {
    fn from(k: McpKey) -> Self {
        Self {
            scopes: k.scopes.known().iter().map(McpScope::as_str).collect(),
            unknown_scopes: k.scopes.unknown().to_vec(),
            id: k.id,
            name: k.name,
            send_validation: k.send_validation.as_str(),
            created_by_user_id: k.created_by_user_id,
            created_at: k.created_at,
            last_used_at: k.last_used_at,
            revoked_at: k.revoked_at,
            expires_at: k.expires_at,
            version: k.version,
        }
    }
}

/// Corps de création d'une clé.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMcpKeyRequest {
    pub name: String,
    /// Au moins une portée ; les doublons sont normalisés.
    pub scopes: Vec<String>,
    /// Expiration facultative (RFC 3339), future.
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    /// `required` (défaut) ou `exempt`.
    #[serde(default)]
    pub send_validation: Option<String>,
}

/// Réponse de création : le secret en clair, **une seule fois**.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMcpKeyResponse {
    #[serde(flatten)]
    pub cle: McpKeyResponse,
    /// Secret `kesh_mcp_…` — affiché une seule fois, jamais re-récupérable.
    pub key: String,
}

/// Corps de révocation (verrou optimiste).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeMcpKeyRequest {
    pub version: i32,
}

/// Corps du changement de politique de validation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSendValidationRequest {
    pub send_validation: String,
    pub version: i32,
}

/// Le nom d'une clé, rogné de tout caractère invisible de bord
/// (`kesh_core::text::is_invisible` : blancs Unicode — U+00A0, U+3000 —,
/// caractères de contrôle et de largeur nulle — U+200B, U+FEFF…) ; refusé s'il
/// ne reste rien ou plus de [`NAME_MAX_CHARS`] caractères (B1 = E4 de la revue
/// de la 32-5a : le `CHECK` de la colonne ne voit que `[:space:]`).
pub fn validate_name(raw: &str) -> Result<String, AppError> {
    let name = raw.trim_matches(kesh_core::text::is_invisible);
    if name.is_empty() {
        return Err(AppError::Validation(t(
            "admin-mcp-keys-error-name-required",
            "Le nom de la clé est requis.",
        )));
    }
    if name.chars().count() > NAME_MAX_CHARS {
        return Err(AppError::Validation(t(
            "admin-mcp-keys-error-name-too-long",
            "Le nom de la clé est trop long : 255 caractères au plus.",
        )));
    }
    Ok(name.to_string())
}

/// Longueur maximale, en caractères, d'une portée inconnue **recopiée** dans le
/// message de refus (revue de code P1, B4 = E3 = A2) : la chaîne vient du
/// client et n'est bornée par rien d'autre que la taille du corps.
pub const SCOPE_CITED_MAX_CHARS: usize = 64;

/// La chaîne du client telle qu'un message la cite : coupée après
/// [`SCOPE_CITED_MAX_CHARS`] caractères (frontière de caractère, jamais
/// d'octet), suivie de `…` si elle l'a été.
pub fn cite(raw: &str) -> String {
    let mut chars = raw.chars();
    let tete: String = chars.by_ref().take(SCOPE_CITED_MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{tete}…")
    } else {
        tete
    }
}

/// Année maximale d'une expiration : la colonne `expires_at` est un
/// `DATETIME(3)`, borné à `9999-12-31` (revue de code P1, E1 = B9 — au-delà,
/// l'écriture échouait en base, en 500).
pub const EXPIRY_MAX_YEAR: i32 = 9999;

/// Les portées demandées, lues **strictement** : une chaîne inconnue (dont
/// `books:write`, `admin`, `read`) ou une liste vide est refusée ; les doublons
/// sont normalisés (un ensemble ne les connaît pas). La portée inconnue citée au
/// message est bornée par [`cite`].
pub fn validate_scopes(raw: &[String]) -> Result<McpScopes, AppError> {
    McpScopes::parse(raw.iter().map(String::as_str)).map_err(|e| match e {
        McpScopeError::Unknown(inconnue) => {
            let portee = cite(&inconnue);
            let mut args = FluentArgs::new();
            args.set("scope", portee.clone());
            AppError::Validation(t_args(
                "admin-mcp-keys-error-scope-unknown",
                &format!(
                    "Portée inconnue : {portee}. Portées admises : books:read, invoices:read, \
                     contacts:read, documents:read, prompts:write."
                ),
                &args,
            ))
        }
        McpScopeError::Empty | McpScopeError::InvalidJson(_) => AppError::Validation(t(
            "admin-mcp-keys-error-scopes-empty",
            "Une clé du serveur MCP porte au moins une portée.",
        )),
    })
}

/// La politique de validation demandée : `required` ou `exempt`, comparaison
/// exacte.
pub fn validate_send_validation(raw: &str) -> Result<SendValidation, AppError> {
    raw.parse::<SendValidation>().map_err(|_| {
        AppError::Validation(t(
            "admin-mcp-keys-error-send-validation-unknown",
            "Politique de validation inconnue : « required » ou « exempt » attendu.",
        ))
    })
}

/// Une version de verrou optimiste présentée par le client : 1 au moins (une
/// clé naît en version 1).
fn validate_version(version: i32) -> Result<(), AppError> {
    if version < 1 {
        return Err(AppError::Validation(t(
            "admin-mcp-keys-error-version-invalid",
            "Version invalide : un entier supérieur ou égal à 1 est attendu.",
        )));
    }
    Ok(())
}

/// Les portées connues d'une clé, pour un détail d'audit.
fn scopes_json(k: &McpKey) -> Vec<&'static str> {
    k.scopes.known().iter().map(McpScope::as_str).collect()
}

async fn begin(state: &AppState) -> Result<sqlx::Transaction<'static, sqlx::MySql>, AppError> {
    state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))
}

async fn commit(tx: sqlx::Transaction<'static, sqlx::MySql>) -> Result<(), AppError> {
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))
}

/// La clé `id` de la société courante, ou 404 (anti-énumération KF-002 : une
/// clé d'une autre société est indiscernable d'une clé absente).
async fn find_for_company(
    state: &AppState,
    current_user: &CurrentUser,
    id: i64,
) -> Result<McpKey, AppError> {
    mcp_keys::find_by_id_for_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))
}

/// `GET /api/v1/admin/mcp/keys` — toutes les clés de la société, révoquées
/// comprises.
pub async fn list_mcp_keys(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<McpKeyResponse>>, AppError> {
    let keys = mcp_keys::list_by_company(&state.pool, current_user.company_id, true).await?;
    Ok(Json(keys.into_iter().map(McpKeyResponse::from).collect()))
}

/// `POST /api/v1/admin/mcp/keys` — crée une clé et rend son secret une seule
/// fois. Audit `mcp_key.created` (nom, portées, politique) dans la transaction.
pub async fn create_mcp_key(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateMcpKeyRequest>,
) -> Result<(StatusCode, Json<CreateMcpKeyResponse>), AppError> {
    let name = validate_name(&req.name)?;
    let scopes = validate_scopes(&req.scopes)?;
    let send_validation = match req.send_validation.as_deref() {
        None => SendValidation::default(),
        Some(raw) => validate_send_validation(raw)?,
    };
    // Une clé « mort-née » (expiration passée) serait créée, son secret rendu,
    // et refusée aussitôt à l'authentification : refusée ici, comme pour les
    // clés d'API.
    if let Some(exp) = req.expires_at
        && exp <= Utc::now()
    {
        return Err(AppError::Validation(t(
            "admin-mcp-keys-error-expiry-past",
            "La date d'expiration doit être dans le futur.",
        )));
    }
    if let Some(exp) = req.expires_at
        && exp.year() > EXPIRY_MAX_YEAR
    {
        return Err(AppError::Validation(t(
            "admin-mcp-keys-error-expiry-out-of-range",
            "La date d'expiration ne peut pas dépasser le 9999-12-31 (UTC).",
        )));
    }

    let (token, key_hash) = generate_mcp_key();
    let new = NewMcpKey {
        company_id: current_user.company_id,
        created_by_user_id: current_user.user_id,
        name,
        key_hash,
        scopes,
        send_validation,
        expires_at: req.expires_at.map(|dt| dt.naive_utc()),
    };

    let mut tx = begin(&state).await?;
    let created = mcp_keys::create_in_tx(&mut tx, new).await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_key.created",
            "mcp_key",
            created.id,
            Some(serde_json::json!({
                "name": created.name,
                "scopes": scopes_json(&created),
                "sendValidation": created.send_validation.as_str(),
            })),
        ),
    )
    .await?;
    commit(tx).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateMcpKeyResponse {
            cle: McpKeyResponse::from(created),
            key: token,
        }),
    ))
}

/// `DELETE /api/v1/admin/mcp/keys/{id}` — révocation (verrou optimiste).
/// 404 si la clé est absente ou d'une autre société ; 409 si la version est
/// périmée ou la clé déjà révoquée. Audit `mcp_key.revoked`.
pub async fn revoke_mcp_key(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<RevokeMcpKeyRequest>,
) -> Result<StatusCode, AppError> {
    validate_version(req.version)?;
    let existing = find_for_company(&state, &current_user, id).await?;

    let mut tx = begin(&state).await?;
    mcp_keys::revoke_in_tx(&mut tx, current_user.company_id, id, req.version).await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_key.revoked",
            "mcp_key",
            id,
            Some(serde_json::json!({
                "name": existing.name,
                "scopes": scopes_json(&existing),
                "sendValidation": existing.send_validation.as_str(),
            })),
        ),
    )
    .await?;
    commit(tx).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// `PUT /api/v1/admin/mcp/keys/{id}/send-validation` — change la politique
/// de validation avant envoi d'une clé non révoquée (verrou optimiste). 404 si
/// la clé est absente ou d'une autre société ; 409 si la version est périmée
/// ou la clé révoquée (C-32-5-30). Audit `mcp_key.send_validation_changed`
/// (avant, après). Rend la clé relue.
pub async fn set_mcp_key_send_validation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<SetSendValidationRequest>,
) -> Result<Json<McpKeyResponse>, AppError> {
    let after = validate_send_validation(&req.send_validation)?;
    validate_version(req.version)?;
    let existing = find_for_company(&state, &current_user, id).await?;

    let mut tx = begin(&state).await?;
    mcp_keys::set_send_validation_in_tx(&mut tx, current_user.company_id, id, req.version, after)
        .await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_key.send_validation_changed",
            "mcp_key",
            id,
            Some(serde_json::json!({
                "name": existing.name,
                "before": existing.send_validation.as_str(),
                "after": after.as_str(),
            })),
        ),
    )
    .await?;
    commit(tx).await?;

    let updated = find_for_company(&state, &current_user, id).await?;
    Ok(Json(McpKeyResponse::from(updated)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_trimmed_of_unicode_blanks_and_zero_width_characters() {
        assert_eq!(
            validate_name("\u{00A0}\u{3000} agent\u{200B}\u{FEFF}\t").unwrap(),
            "agent"
        );
        // Les invisibles intérieurs ne sont pas des bords : ils restent.
        assert_eq!(validate_name(" a b ").unwrap(), "a b");
    }

    #[test]
    fn a_name_of_invisible_characters_only_is_refused() {
        for raw in [
            "",
            "   ",
            "\u{00A0}",
            "\u{3000}",
            "\u{200B}",
            "\u{FEFF}\u{2060}",
            "\t\n",
        ] {
            assert!(
                matches!(validate_name(raw), Err(AppError::Validation(_))),
                "nom {raw:?} accepté"
            );
        }
    }

    #[test]
    fn name_length_is_bounded_in_characters() {
        assert!(validate_name(&"é".repeat(NAME_MAX_CHARS)).is_ok());
        assert!(matches!(
            validate_name(&"é".repeat(NAME_MAX_CHARS + 1)),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn scopes_are_strict_and_deduplicated() {
        let s = validate_scopes(&[
            "invoices:read".into(),
            "books:read".into(),
            "books:read".into(),
        ])
        .unwrap();
        assert_eq!(
            s.iter().collect::<Vec<_>>(),
            vec![McpScope::BooksRead, McpScope::InvoicesRead]
        );
        for unknown in ["books:write", "admin", "read", "Books:read", ""] {
            assert!(
                matches!(
                    validate_scopes(&[unknown.to_string()]),
                    Err(AppError::Validation(_))
                ),
                "portée {unknown:?} acceptée"
            );
        }
        assert!(matches!(validate_scopes(&[]), Err(AppError::Validation(_))));
    }

    #[test]
    fn an_unknown_scope_is_cited_within_bounds() {
        // À la borne : intacte.
        let a_la_borne = "é".repeat(SCOPE_CITED_MAX_CHARS);
        assert_eq!(cite(&a_la_borne), a_la_borne);
        // Au-delà : coupée sur une frontière de caractère, suivie de « … ».
        let long = "é".repeat(SCOPE_CITED_MAX_CHARS + 1000);
        let cite_long = cite(&long);
        assert_eq!(cite_long.chars().count(), SCOPE_CITED_MAX_CHARS + 1);
        assert!(cite_long.ends_with('…'));
        assert_eq!(
            cite_long.trim_end_matches('…'),
            "é".repeat(SCOPE_CITED_MAX_CHARS)
        );
        // Au message de refus.
        let m = match validate_scopes(&[format!("x{long}")]) {
            Err(AppError::Validation(m)) => m,
            other => panic!("400 attendu, obtenu {other:?}"),
        };
        assert!(m.contains('…'), "{m}");
        assert!(
            m.chars().count() < SCOPE_CITED_MAX_CHARS + 200,
            "message non borné : {} caractères",
            m.chars().count()
        );
    }

    #[test]
    fn send_validation_is_exact() {
        assert_eq!(
            validate_send_validation("required").unwrap(),
            SendValidation::Required
        );
        assert_eq!(
            validate_send_validation("exempt").unwrap(),
            SendValidation::Exempt
        );
        for raw in ["Required", "EXEMPT", "none", "", "required "] {
            assert!(validate_send_validation(raw).is_err(), "{raw:?} accepté");
        }
    }
}
