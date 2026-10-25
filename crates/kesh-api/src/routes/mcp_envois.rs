//! Routes des envois du serveur MCP à valider — `/api/v1/admin/mcp/sends`
//! (Story 32-8b, refs #723).
//!
//! - `GET  /api/v1/admin/mcp/sends` — les envois de la société, **sans contenu
//!   ni résumé** (en attente vivants d'abord) ;
//! - `GET  /api/v1/admin/mcp/sends/{id}` — le détail : résumé, contenu, sa
//!   taille en octets, la version ; 404 pour un envoi d'une autre société ;
//! - `POST /api/v1/admin/mcp/sends/{id}/decision` — `{"version": n,
//!   "decision": "approve"|"refuse", "content"?: string}` : valider tel quel,
//!   valider modifié (`content`, avec `approve` seulement), ou refuser ; la
//!   décision et son entrée d'audit (`mcp_send.approved` / `mcp_send.refused`)
//!   dans **une** transaction ; rend le détail après décision.
//!
//! **Gardes** (C-32-8-10) : les trois routes vivent dans le bloc `admin_routes`
//! de `lib.rs` — `require_admin_role` (un Comptable reçoit `403 FORBIDDEN`)
//! puis `require_not_pat` (un PAT reçoit `403 API_KEY_ADMIN_FORBIDDEN`) ; une
//! clé MCP n'est pas reconnue par `require_auth` (401) : **l'agent ne peut pas
//! valider son propre envoi** par un jeton.
//!
//! **Ce qui part est ce qui a été vu** (§ *La propriété centrale* de la fiche
//! 32-8) : « tel quel » ne renvoie pas le texte — le serveur valide la valeur
//! stockée **à la version** que l'écran a lue, sous verrou ; « modifié » est
//! la valeur reçue, contrôlée (`envois_mcp::verifier_contenu_modifie`) et
//! stockée telle quelle — **jamais** rognée ni normalisée.
//!
//! **Le journal** : le détail d'audit d'une décision est **exactement**
//! `{envoiId, cleMcpId, outil, modifie}` — jamais le contenu ni le résumé.
//! Aucun évènement `tracing` ne nomme le contenu (garde
//! `aucun_evenement_ne_nomme_le_contenu`).

use axum::extract::{Path, State};
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::mcp_send::{Decision, EnvoiDetail, EnvoiResume, ErreurDecision};
use kesh_db::errors::DbError;
use kesh_db::repositories::{audit_log, mcp_sends};

use crate::AppState;
use crate::errors::{AppError, t};
use crate::middleware::auth::CurrentUser;
use crate::services::envois_mcp::{MotifContenu, verifier_contenu_modifie};

/// Une ligne de la liste — **sans** contenu ni résumé.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvoiResumeResponse {
    pub id: i64,
    pub mcp_key_id: i64,
    pub mcp_key_name: String,
    pub mcp_key_revoked: bool,
    pub tool_name: String,
    /// `pending`, `approved`, `refused` ou `expired` (dérivé de l'échéance).
    pub state: &'static str,
    pub modified: bool,
    pub expires_at: NaiveDateTime,
    pub decided_at: Option<NaiveDateTime>,
    pub decided_by: Option<String>,
    pub created_at: NaiveDateTime,
}

impl From<EnvoiResume> for EnvoiResumeResponse {
    fn from(r: EnvoiResume) -> Self {
        Self {
            id: r.id,
            mcp_key_id: r.mcp_key_id,
            mcp_key_name: r.mcp_key_name,
            mcp_key_revoked: r.mcp_key_revoked,
            tool_name: r.tool_name,
            state: r.etat.as_str(),
            modified: r.modified,
            expires_at: r.expires_at,
            decided_at: r.decided_at,
            decided_by: r.decided_by,
            created_at: r.created_at,
        }
    }
}

/// Le détail d'un envoi : la ligne de liste, le résumé et le contenu (nuls
/// pour un envoi refusé ou échu), la taille du contenu en octets, la version.
///
/// ⚠️ Pas de `Debug` : il porte le contenu et le résumé (C-32-8-40).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvoiDetailResponse {
    #[serde(flatten)]
    pub resume: EnvoiResumeResponse,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub content_bytes: Option<u64>,
    pub version: i32,
}

impl From<EnvoiDetail> for EnvoiDetailResponse {
    fn from(d: EnvoiDetail) -> Self {
        Self {
            resume: d.resume.into(),
            summary: d.summary,
            content: d.content,
            content_bytes: d.content_bytes,
            version: d.version,
        }
    }
}

/// Le choix de l'Administrateur.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Choix {
    /// Valider — tel quel, ou modifié si `content` est fourni.
    Approve,
    /// Refuser.
    Refuse,
}

/// Le corps d'une décision. ⚠️ Pas de `Debug` : il porte le contenu modifié.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRequest {
    pub version: i32,
    pub decision: Choix,
    #[serde(default)]
    pub content: Option<String>,
}

/// L'envoi `id` de la société courante, ou 404 (KF-002 : un envoi d'une
/// autre société est indiscernable d'un envoi absent).
async fn trouver(state: &AppState, company_id: i64, id: i64) -> Result<EnvoiDetail, AppError> {
    mcp_sends::trouver(&state.pool, company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))
}

/// La traduction d'un refus de décision. L'ordre des refus est celui de
/// `decider_in_tx` : absent, déjà décidé, échu, version périmée (point 1 de
/// #765, épinglé par `ordre_des_refus_de_la_decision`).
fn erreur_de_decision(e: ErreurDecision) -> AppError {
    match e {
        ErreurDecision::Introuvable => AppError::Database(DbError::NotFound),
        ErreurDecision::DejaDecidee => AppError::IllegalState(t(
            "mcp-send-already-decided",
            "Cet envoi a déjà été décidé.",
        )),
        ErreurDecision::Echue => AppError::IllegalState(t(
            "mcp-send-expired",
            "Cet envoi a expiré : il ne peut plus être décidé.",
        )),
        ErreurDecision::VersionPerimee => AppError::Database(DbError::OptimisticLockConflict),
        ErreurDecision::Db(e) => AppError::Database(e),
    }
}

/// La décision à transmettre au dépôt, contrôlée. Un texte modifié est
/// refusé tel quel ou admis tel quel — **aucune** correction (AC-B9). Un texte
/// vide est un texte : `Some("")` valide un envoi au contenu vide (point 3 de
/// #765, C-32-8-55).
fn decision(req: DecisionRequest) -> Result<(i32, Decision), AppError> {
    if req.version < 1 {
        return Err(AppError::Validation(t(
            "mcp-send-version-invalid",
            "Version invalide : un entier supérieur ou égal à 1 est attendu.",
        )));
    }
    let d = match (req.decision, req.content) {
        (Choix::Refuse, Some(_)) => {
            return Err(AppError::Validation(t(
                "mcp-send-content-with-refusal",
                "Un refus ne porte pas de contenu.",
            )));
        }
        (Choix::Refuse, None) => Decision::Refuser,
        (Choix::Approve, contenu) => {
            if let Some(c) = &contenu {
                verifier_contenu_modifie(c).map_err(|m| match m {
                    MotifContenu::TropVolumineux => AppError::Validation(t(
                        "mcp-send-content-too-large",
                        "Le contenu modifié dépasse la taille d'un envoi (24 576 octets).",
                    )),
                    MotifContenu::CaractereCache => AppError::Validation(t(
                        "mcp-send-content-hidden-character",
                        "Le contenu modifié contient un caractère invisible ou de contrôle : \
                         retirez-le avant de valider.",
                    )),
                })?;
            }
            Decision::Valider { contenu }
        }
    };
    Ok((req.version, d))
}

/// `GET /api/v1/admin/mcp/sends` — les envois de la société, sans contenu.
pub async fn lister_envois(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<EnvoiResumeResponse>>, AppError> {
    let envois = mcp_sends::lister(&state.pool, current_user.company_id).await?;
    Ok(Json(envois.into_iter().map(Into::into).collect()))
}

/// `GET /api/v1/admin/mcp/sends/{id}` — le détail d'un envoi de la société.
pub async fn detail_envoi(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<EnvoiDetailResponse>, AppError> {
    let d = trouver(&state, current_user.company_id, id).await?;
    Ok(Json(d.into()))
}

/// `POST /api/v1/admin/mcp/sends/{id}/decision` — valider (tel quel ou
/// modifié) ou refuser. 404 introuvable ou d'une autre société ; 409
/// `ILLEGAL_STATE_TRANSITION` déjà décidé ou échu ; 409
/// `OPTIMISTIC_LOCK_CONFLICT` version périmée ; 400 contenu refusé. La
/// décision et son entrée d'audit dans une transaction. Rend le détail relu.
pub async fn decider_envoi(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<DecisionRequest>,
) -> Result<Json<EnvoiDetailResponse>, AppError> {
    let (version, d) = decision(req)?;
    let approuve = matches!(d, Decision::Valider { .. });
    let company_id = current_user.company_id;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let decide =
        mcp_sends::decider_in_tx(&mut tx, company_id, id, version, d, current_user.user_id)
            .await
            .map_err(erreur_de_decision)?;
    let detail = serde_json::json!({
        "envoiId": decide.id,
        "cleMcpId": decide.mcp_key_id,
        "outil": decide.tool_name,
        "modifie": decide.modified,
    });
    let entree = if approuve {
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_send.approved",
            "mcp_send",
            decide.id,
            Some(detail),
        )
    } else {
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_send.refused",
            "mcp_send",
            decide.id,
            Some(detail),
        )
    };
    audit_log::insert_in_tx(&mut tx, entree).await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    let apres = trouver(&state, company_id, id).await?;
    Ok(Json(apres.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(version: i32, choix: Choix, content: Option<&str>) -> DecisionRequest {
        DecisionRequest {
            version,
            decision: choix,
            content: content.map(str::to_string),
        }
    }

    #[test]
    fn decision_controlee_jamais_corrigee() {
        assert!(matches!(
            decision(req(1, Choix::Approve, None)),
            Ok((1, Decision::Valider { contenu: None }))
        ));
        match decision(req(2, Choix::Approve, Some("a \n\t é "))) {
            Ok((2, Decision::Valider { contenu: Some(c) })) => assert_eq!(c, "a \n\t é "),
            _ => panic!("contenu admis tel quel"),
        }
        match decision(req(1, Choix::Approve, Some(""))) {
            Ok((1, Decision::Valider { contenu: Some(c) })) => assert_eq!(c, ""),
            _ => panic!("chaîne vide admise"),
        }
        assert!(matches!(
            decision(req(1, Choix::Refuse, None)),
            Ok((1, Decision::Refuser))
        ));
        for refusee in [
            req(1, Choix::Refuse, Some("x")),
            req(0, Choix::Approve, None),
            req(1, Choix::Approve, Some("a\r\nb")),
            req(1, Choix::Approve, Some("\u{202E}")),
        ] {
            assert!(matches!(decision(refusee), Err(AppError::Validation(_))));
        }
    }
}
