//! Consultation du journal d'audit — Story 25-1c-a (#378).
//!
//! Le journal était écrit depuis l'Epic 7 et **lisible par personne** : ni
//! route, ni écran. Ce module en ouvre la lecture.
//!
//! ⛔ **`ensure_not_pat` est la PREMIÈRE instruction de chaque handler**, et non
//! une couche de bloc : `comptable_routes` n'a pas de garde anti-clé — seul le
//! bloc admin porte `require_not_pat` (`lib.rs:329-331`) —, et le test
//! `the_admin_guards_have_no_alias_and_no_second_consumer_crate_wide`
//! (`admin_pat_denied_e2e.rs:427-503`) **interdit** tout second consommateur de
//! cette garde-là. ⚠️ Le code d'erreur d'`ensure_not_pat` parle de « gestion de
//! clés » ; il est réutilisé tel quel, un variant de plus pour un libellé serait
//! du bruit.
//!
//! ⛔ **La langue est celle de l'INSTALLATION** (`state.config.locale`), jamais
//! `company.accounting_language` : le journal se lit à l'écran, et l'interface
//! n'a qu'une langue (`KESH_LANG`).
//!
//! ⛔ **Ni la liste, ni le vocabulaire n'écrivent d'entrée d'audit** (AC 9,
//! arbitrage 5) : chaque page en écrirait une, et le journal enflerait de sa
//! propre lecture.

use axum::extract::{Query, State};
use axum::{Extension, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use kesh_db::entities::audit_log::AuditLogEntry;
use kesh_db::repositories::audit_log::{self as audit_log_repo, AuditLogListQuery, MAX_LIMIT};
use kesh_i18n::{I18nBundle, Locale};

use crate::audit_labels::{self, ACTIONS, ENTITY_TYPES};
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::api_keys::ensure_not_pat;
use crate::routes::ListResponse;
use crate::AppState;

/// Longueur de `audit_log.entity_type` en base.
const ENTITY_TYPE_MAX_LEN: usize = 32;
/// Longueur de `audit_log.action` en base.
const ACTION_MAX_LEN: usize = 64;

fn default_limit() -> i64 {
    50
}
fn default_offset() -> i64 {
    0
}

/// Borne basse acceptée pour une date de filtre.
///
/// ⛔ **Ce n'est pas du zèle.** `NaiveDate::from_str("-0001-01-01")` **réussit**,
/// et une année négative — comme une année au-delà de 65535
/// (`sqlx-mysql-0.8.6/src/types/chrono.rs:263-264`, `u16::try_from`) — fait
/// **paniquer** la liaison de paramètre. Une panique n'est pas une 500 : aucune
/// couche ne la rattrape.
fn borne_basse() -> NaiveDate {
    NaiveDate::from_ymd_opt(1000, 1, 1).expect("1000-01-01 est une date valide")
}

/// Borne haute acceptée. `0999-12-31` se lierait sans erreur et rendrait
/// simplement tout le journal ; la plage est refusée **aux deux bornes** pour
/// qu'elle soit la même des deux côtés.
fn borne_haute() -> NaiveDate {
    NaiveDate::from_ymd_opt(9999, 12, 31).expect("9999-12-31 est une date valide")
}

/// Filtres reçus par la liste **et** par l'export.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogQuery {
    #[serde(default)]
    pub date_from: Option<String>,
    #[serde(default)]
    pub date_to: Option<String>,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub entity_id: Option<i64>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default = "default_offset")]
    pub offset: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

/// Une entrée, telle que l'écran la reçoit.
///
/// ⛔ **DTO dédié, et non l'entité sérialisée** : `NaiveDateTime` sérialise
/// **sans** fuseau, et `ActorType` dérive `Serialize` sans renommage — il
/// sortirait `"User"` / `"ApiKey"` là où la base écrit `"user"` / `"api_key"`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntryResponse {
    pub id: i64,
    /// UTC **explicite**, millisecondes toujours écrites — ce que la
    /// sérialisation serde d'un `DateTime<Utc>` ne garantit pas.
    pub created_at: String,
    pub actor_label: String,
    pub actor_type: String,
    pub actor_api_key_id: Option<i64>,
    pub user_id: i64,
    /// Le **code** : l'écran filtre et construit son URL dessus.
    pub action: String,
    pub action_label: String,
    pub entity_type: String,
    pub entity_type_label: String,
    /// Tel quel, `0` compris : `AUDIT_ENTITY_ID_NONE` vaut `0`, et le masquer
    /// ferait disparaître l'information.
    pub entity_id: i64,
    pub details: Option<serde_json::Value>,
}

impl AuditLogEntryResponse {
    fn depuis(entree: AuditLogEntry, i18n: &I18nBundle, locale: &Locale) -> Self {
        Self {
            id: entree.id,
            created_at: entree
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            actor_label: entree.actor_label,
            actor_type: entree.actor_type.as_str().to_string(),
            actor_api_key_id: entree.actor_api_key_id,
            user_id: entree.user_id,
            action_label: audit_labels::action_label(i18n, locale, &entree.action),
            action: entree.action,
            entity_type_label: audit_labels::entity_type_label(i18n, locale, &entree.entity_type),
            entity_type: entree.entity_type,
            entity_id: entree.entity_id,
            details: entree.details_json,
        }
    }
}

/// Un couple `{ code, label }` du vocabulaire.
#[derive(Debug, Serialize)]
pub struct VocabularyEntry {
    pub code: String,
    pub label: String,
}

/// La réponse de `/vocabulary`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyResponse {
    pub entity_types: Vec<VocabularyEntry>,
    pub actions: Vec<VocabularyEntry>,
}

/// Valide les paramètres et construit la requête du dépôt.
///
/// ⛔ **Une seule fonction pour la liste ET l'export** : deux validations
/// séparées dériveraient, et l'export cesserait de montrer les lignes de
/// l'écran — ce que l'AC 10 interdit précisément.
///
/// ⚠️ Un `entityId`, `offset` ou `limit` **non numérique** n'arrive jamais ici :
/// l'extracteur `Query` d'Axum le refuse en 400 **texte**, avant le handler.
/// C'est la convention écrite du dépôt (`journal_entries.rs:275-279`) ; ne pas
/// la « corriger » ici.
pub(crate) fn construire_requete(params: &ListAuditLogQuery) -> Result<AuditLogListQuery, AppError> {
    let date_from = parse_date(params.date_from.as_deref(), "dateFrom")?;
    let date_to = parse_date(params.date_to.as_deref(), "dateTo")?;
    if let (Some(depuis), Some(jusqu)) = (date_from, date_to)
        && depuis > jusqu
    {
        return Err(AppError::Validation(
            "dateFrom doit être inférieur ou égal à dateTo".into(),
        ));
    }

    let entity_type = parse_code(
        params.entity_type.as_deref(),
        "entityType",
        ENTITY_TYPE_MAX_LEN,
    )?;
    let action = parse_code(params.action.as_deref(), "action", ACTION_MAX_LEN)?;

    if let Some(id) = params.entity_id
        && id <= 0
    {
        return Err(AppError::Validation(
            "entityId doit être un identifiant positif".into(),
        ));
    }
    // Un identifiant n'a de sens que rapporté à son type : les `entity_id` se
    // répètent d'une table à l'autre.
    if params.entity_id.is_some() && entity_type.is_none() {
        return Err(AppError::Validation(
            "entityId exige entityType — un identifiant seul ne désigne rien".into(),
        ));
    }

    Ok(AuditLogListQuery {
        date_from,
        date_to,
        entity_type,
        entity_id: params.entity_id,
        action,
        limit: params.limit.clamp(1, MAX_LIMIT),
        offset: params.offset.max(0),
    })
}

fn parse_date(valeur: Option<&str>, nom: &str) -> Result<Option<NaiveDate>, AppError> {
    let Some(texte) = valeur.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let date = NaiveDate::from_str(texte)
        .map_err(|e| AppError::Validation(format!("{nom} invalide ({e})")))?;
    if date < borne_basse() || date > borne_haute() {
        return Err(AppError::Validation(format!(
            "{nom} doit être compris entre 1000-01-01 et 9999-12-31"
        )));
    }
    Ok(Some(date))
}

/// Un code plus long que sa colonne ne peut **rien** trouver : le dire vaut
/// mieux qu'une liste vide muette. ⚠️ Un code **absent du vocabulaire** reste
/// accepté — un code historique doit rester filtrable.
fn parse_code(valeur: Option<&str>, nom: &str, maximum: usize) -> Result<Option<String>, AppError> {
    let Some(texte) = valeur.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if texte.chars().count() > maximum {
        return Err(AppError::Validation(format!(
            "{nom} dépasse {maximum} caractères"
        )));
    }
    Ok(Some(texte.to_string()))
}

/// `GET /api/v1/audit-log` — la page de consultation (AC 5-9).
///
/// Refus : 401 sans authentification, 403 pour le rôle Consultation
/// (`require_comptable_role`, posé au montage), 403
/// `API_KEY_MANAGEMENT_FORBIDDEN` pour une clé API.
pub async fn list_audit_log(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListAuditLogQuery>,
) -> Result<Json<ListResponse<AuditLogEntryResponse>>, AppError> {
    ensure_not_pat(&current_user)?;

    let requete = construire_requete(&params)?;
    let resultat =
        audit_log_repo::list_by_company_paginated(&state.pool, current_user.company_id, requete)
            .await?;

    let locale = state.config.locale;
    Ok(Json(ListResponse {
        items: resultat
            .items
            .into_iter()
            .map(|e| AuditLogEntryResponse::depuis(e, &state.i18n, &locale))
            .collect(),
        total: resultat.total,
        offset: resultat.offset,
        limit: resultat.limit,
    }))
}

/// `GET /api/v1/audit-log/vocabulary` — les deux listes traduites (AC 17).
///
/// Aucune lecture en base : les codes sont ceux des constantes.
///
/// ⚠️ **L'ordre est celui des constantes, par code.** Trier par libellé est
/// l'affaire de l'écran, qui connaît la locale : un tri d'octets côté serveur
/// rangerait « Écriture » après « Utilisateur ».
pub async fn vocabulary(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<VocabularyResponse>, AppError> {
    ensure_not_pat(&current_user)?;

    let locale = state.config.locale;
    Ok(Json(VocabularyResponse {
        entity_types: ENTITY_TYPES
            .iter()
            .map(|code| VocabularyEntry {
                code: (*code).to_string(),
                label: audit_labels::entity_type_label(&state.i18n, &locale, code),
            })
            .collect(),
        actions: ACTIONS
            .iter()
            .map(|code| VocabularyEntry {
                code: (*code).to_string(),
                label: audit_labels::action_label(&state.i18n, &locale, code),
            })
            .collect(),
    }))
}
