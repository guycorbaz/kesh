//! Routes des réglages du serveur MCP — `/api/v1/admin/mcp/settings`
//! (Story 32-5c, refs #723).
//!
//! - `GET /api/v1/admin/mcp/settings` — la ligne de la société, ou les
//!   **défauts du code** (`version: 0`) si elle n'existe pas ; n'écrit jamais
//!   (C-32-5-8).
//! - `PUT /api/v1/admin/mcp/settings` — remplacement complet (tous les champs
//!   et `version`), verrou optimiste : `version: 0` crée la ligne et rend 409
//!   si elle existe déjà (jamais d'écrasement) ; `version ≥ 1` la modifie, 409
//!   si elle est périmée. Audit `mcp_server_settings.updated` (avant, après)
//!   dans la transaction.
//!
//! Gardes : bloc `admin_routes` (cf. `routes::mcp_keys`).
//!
//! # La forme des listes (AC-c4, C-32-5-19)
//!
//! - **Plages admises** et **mandataires de confiance** : chaque entrée se lit
//!   d'abord comme une **adresse** (`IpAddr`, stockée `/32` ou `/128`), sinon
//!   comme un **réseau** (`ipnet::IpNet`, qui exige le préfixe). Un réseau qui
//!   porte des bits d'hôte (`10.1.2.3/8`) est **refusé**, et le message donne la
//!   forme attendue (`10.0.0.0/8`) — jamais tronqué en silence. La forme
//!   stockée est celle d'`IpNet::to_string`.
//! - **Origines** : `http(s)://hôte[:port]`, sans chemin, requête, fragment ni
//!   identifiants ; stockée sous sa sérialisation ASCII d'origine (minuscules,
//!   port par défaut retiré, nom international en `xn--`).
//! - Listes bornées à [`LIST_MAX_ENTRIES`] entrées de [`ENTRY_MAX_CHARS`]
//!   caractères ; doublons retirés (après mise en forme canonique, premier
//!   gardé) ; toute entrée invalide → 400 traduit, rien d'écrit.
//!
//! ⚠️ **Ce que ces routes ne garantissent pas** : le **sens** des réglages —
//! qu'un filtre activé refuse un pair public, qu'un mandataire déclaré soit le
//! seul dont on lise `X-Forwarded-For`, qu'un serveur désactivé rende 404 —
//! relève de la 32-6.
//!
//! ⚠️ **Une ligne à `version = 0`** (SQL à la main, sauvegarde fabriquée ; la
//! colonne n'a pas de `CHECK (version >= 1)`, B3 de la revue de la 32-5a) est
//! rendue telle quelle par le `GET`, et tout `PUT` à `version: 0` rend alors 409
//! (la ligne existe) : l'état ne se répare pas par l'écran. Choix C-32-5-53.

use std::net::IpAddr;

use axum::extract::State;
use axum::{Extension, Json};
use chrono::{NaiveDateTime, Utc};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{McpServerSettings, McpServerSettingsUpdate};
use kesh_db::repositories::{audit_log, mcp_server_settings};
use kesh_i18n::FluentArgs;

use crate::AppState;
use crate::errors::{AppError, t, t_args};
use crate::middleware::auth::CurrentUser;

/// Nombre maximal d'entrées d'une liste.
pub const LIST_MAX_ENTRIES: usize = 64;

/// Longueur maximale d'une entrée, en caractères.
pub const ENTRY_MAX_CHARS: usize = 255;

/// Les réglages tels que les routes les rendent.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerSettingsResponse {
    pub enabled: bool,
    pub address_filter_enabled: bool,
    pub allowed_networks: Vec<String>,
    pub trusted_proxies: Vec<String>,
    pub allowed_origins: Vec<String>,
    /// `0` : aucune ligne (défauts du code).
    pub version: i32,
    /// UTC, avec son `Z` ; `null` pour les défauts.
    pub updated_at: Option<chrono::DateTime<Utc>>,
}

impl From<McpServerSettings> for McpServerSettingsResponse {
    fn from(s: McpServerSettings) -> Self {
        Self {
            enabled: s.enabled,
            address_filter_enabled: s.address_filter_enabled,
            allowed_networks: s.allowed_networks,
            trusted_proxies: s.trusted_proxies,
            allowed_origins: s.allowed_origins,
            version: s.version,
            updated_at: s.updated_at.map(|t: NaiveDateTime| t.and_utc()),
        }
    }
}

/// Corps de `PUT /api/v1/admin/mcp/settings` : tous les champs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMcpServerSettingsRequest {
    pub enabled: bool,
    pub address_filter_enabled: bool,
    pub allowed_networks: Vec<String>,
    pub trusted_proxies: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub version: i32,
}

fn entry_args(entree: &str) -> FluentArgs<'static> {
    let mut args = FluentArgs::new();
    args.set("entry", entree.to_string());
    args
}

/// Une adresse ou un réseau, sous sa forme canonique (`IpNet::to_string`).
pub fn canonical_network(raw: &str) -> Result<String, AppError> {
    let entree = raw.trim();
    if let Ok(adresse) = entree.parse::<IpAddr>() {
        return Ok(IpNet::from(adresse).to_string());
    }
    let reseau = entree.parse::<IpNet>().map_err(|_| {
        AppError::Validation(t_args(
            "admin-mcp-settings-error-network-invalid",
            &format!(
                "Adresse ou réseau invalide : {entree}. Formes attendues : une adresse \
                 (192.168.1.10) ou un réseau (192.168.1.0/24)."
            ),
            &entry_args(entree),
        ))
    })?;
    let attendu = reseau.trunc();
    if attendu != reseau {
        let mut args = entry_args(entree);
        args.set("expected", attendu.to_string());
        return Err(AppError::Validation(t_args(
            "admin-mcp-settings-error-network-host-bits",
            &format!("Le réseau {entree} porte des bits d'hôte : écrivez {attendu}."),
            &args,
        )));
    }
    Ok(reseau.to_string())
}

/// Une origine `http(s)://hôte[:port]`, sous sa sérialisation ASCII — la règle
/// est `util::origine_canonique`, la même que celle qui normalise l'en-tête
/// `Origin` reçu par `/mcp` (Story 32-6a, AC-a10).
pub fn canonical_origin(raw: &str) -> Result<String, AppError> {
    let entree = raw.trim();
    crate::util::origine_canonique(entree).ok_or_else(|| {
        AppError::Validation(t_args(
            "admin-mcp-settings-error-origin-invalid",
            &format!(
                "Origine invalide : {entree}. Forme attendue : http(s)://hôte[:port], sans \
                 chemin."
            ),
            &entry_args(entree),
        ))
    })
}

/// Met une liste en forme : bornes, mise en forme canonique de chaque entrée,
/// doublons retirés (premier gardé).
fn canonical_list(
    raw: &[String],
    canonical: fn(&str) -> Result<String, AppError>,
) -> Result<Vec<String>, AppError> {
    if raw.len() > LIST_MAX_ENTRIES {
        return Err(AppError::Validation(t(
            "admin-mcp-settings-error-too-many-entries",
            "Liste trop longue : 64 entrées au plus.",
        )));
    }
    let mut out: Vec<String> = Vec::with_capacity(raw.len());
    for entree in raw {
        if entree.chars().count() > ENTRY_MAX_CHARS {
            return Err(AppError::Validation(t(
                "admin-mcp-settings-error-entry-too-long",
                "Entrée trop longue : 255 caractères au plus.",
            )));
        }
        let forme = canonical(entree)?;
        if !out.contains(&forme) {
            out.push(forme);
        }
    }
    Ok(out)
}

/// Valide et met en forme le corps d'un enregistrement.
pub fn validate_settings(
    req: &UpdateMcpServerSettingsRequest,
) -> Result<McpServerSettingsUpdate, AppError> {
    Ok(McpServerSettingsUpdate {
        enabled: req.enabled,
        address_filter_enabled: req.address_filter_enabled,
        allowed_networks: canonical_list(&req.allowed_networks, canonical_network)?,
        trusted_proxies: canonical_list(&req.trusted_proxies, canonical_network)?,
        allowed_origins: canonical_list(&req.allowed_origins, canonical_origin)?,
    })
}

/// Les champs réglables, pour un détail d'audit.
fn settings_json(s: &McpServerSettings) -> serde_json::Value {
    serde_json::json!({
        "enabled": s.enabled,
        "addressFilterEnabled": s.address_filter_enabled,
        "allowedNetworks": s.allowed_networks,
        "trustedProxies": s.trusted_proxies,
        "allowedOrigins": s.allowed_origins,
    })
}

/// `GET /api/v1/admin/mcp/settings`.
pub async fn get_mcp_server_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<McpServerSettingsResponse>, AppError> {
    let s = mcp_server_settings::get_or_defaults(&state.pool, current_user.company_id).await?;
    Ok(Json(s.into()))
}

/// `PUT /api/v1/admin/mcp/settings` — 400 `VALIDATION_ERROR` sur une entrée
/// invalide (rien d'écrit, aucun audit) ; 409 `OPTIMISTIC_LOCK_CONFLICT` sur
/// une version périmée, ou `version: 0` alors qu'une ligne existe.
///
/// L'« avant » de l'audit est lu hors verrou, avant l'écriture : le verrou
/// optimiste du dépôt garantit qu'une écriture réussie à `version = n` part de
/// la ligne de version `n`, celle qu'a lue le client — sauf client qui
/// présenterait une version qu'aucune lecture ne lui a rendue.
pub async fn update_mcp_server_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateMcpServerSettingsRequest>,
) -> Result<Json<McpServerSettingsResponse>, AppError> {
    let settings = validate_settings(&req)?;
    let company_id = current_user.company_id;
    let before = mcp_server_settings::get_or_defaults(&state.pool, company_id).await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let saved =
        mcp_server_settings::save_in_tx(&mut tx, company_id, req.version, &settings).await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_server_settings.updated",
            "mcp_server_settings",
            company_id,
            Some(serde_json::json!({
                "before": settings_json(&before),
                "after": settings_json(&saved),
            })),
        ),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(saved.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(r: Result<String, AppError>) -> String {
        match r {
            Err(AppError::Validation(m)) => m,
            other => panic!("400 attendu, obtenu {other:?}"),
        }
    }

    #[test]
    fn an_address_is_stored_as_a_host_network() {
        assert_eq!(
            canonical_network("192.168.1.10").unwrap(),
            "192.168.1.10/32"
        );
        assert_eq!(canonical_network(" ::1 ").unwrap(), "::1/128");
        assert_eq!(canonical_network("FE80::1").unwrap(), "fe80::1/128");
    }

    #[test]
    fn a_network_is_kept_in_its_canonical_form() {
        assert_eq!(canonical_network("10.0.0.0/8").unwrap(), "10.0.0.0/8");
        assert_eq!(canonical_network("100.64.0.0/10").unwrap(), "100.64.0.0/10");
        assert_eq!(canonical_network("FC00::/7").unwrap(), "fc00::/7");
        assert_eq!(canonical_network("10.1.2.3/32").unwrap(), "10.1.2.3/32");
    }

    #[test]
    fn a_network_with_host_bits_is_refused_with_the_expected_form() {
        let m = refused(canonical_network("10.1.2.3/8"));
        assert!(m.contains("10.0.0.0/8"), "forme attendue absente : {m}");
        assert!(m.contains("10.1.2.3/8"), "entrée absente : {m}");
        let m = refused(canonical_network("fe80::1/10"));
        assert!(m.contains("fe80::/10"), "{m}");
    }

    #[test]
    fn malformed_networks_are_refused() {
        for raw in [
            "",
            "10.0.0",
            "10.0.0.0/33",
            "::/129",
            "nas.local",
            "10.0.0.0/8/8",
            "*",
        ] {
            refused(canonical_network(raw));
        }
    }

    #[test]
    fn origins_are_canonical_and_pathless() {
        assert_eq!(
            canonical_origin("HTTPS://NAS.Local:8443").unwrap(),
            "https://nas.local:8443"
        );
        assert_eq!(
            canonical_origin("http://127.0.0.1:3000").unwrap(),
            "http://127.0.0.1:3000"
        );
        assert_eq!(
            canonical_origin("http://[::1]:8080").unwrap(),
            "http://[::1]:8080"
        );
        assert_eq!(
            canonical_origin("https://nas.local:443").unwrap(),
            "https://nas.local"
        );
        for raw in [
            "nas.local",
            "ftp://nas.local",
            "https://nas.local/",
            "https://nas.local/mcp",
            "https://nas.local?x=1",
            "https://nas.local#f",
            "https://user@nas.local",
            "https://",
            "https://nas local",
            "https://nas.local:99999",
        ] {
            refused(canonical_origin(raw));
        }
    }

    fn req(networks: Vec<&str>, origins: Vec<&str>) -> UpdateMcpServerSettingsRequest {
        UpdateMcpServerSettingsRequest {
            enabled: true,
            address_filter_enabled: true,
            allowed_networks: networks.into_iter().map(String::from).collect(),
            trusted_proxies: vec![],
            allowed_origins: origins.into_iter().map(String::from).collect(),
            version: 0,
        }
    }

    #[test]
    fn duplicates_are_removed_after_canonicalisation() {
        let s = validate_settings(&req(
            vec!["10.0.0.1", "10.0.0.1/32", "192.168.0.0/16", "10.0.0.1"],
            vec!["https://NAS.local", "https://nas.local:443"],
        ))
        .unwrap();
        assert_eq!(s.allowed_networks, vec!["10.0.0.1/32", "192.168.0.0/16"]);
        assert_eq!(s.allowed_origins, vec!["https://nas.local"]);
    }

    #[test]
    fn lists_are_bounded() {
        let many: Vec<String> = (0..=LIST_MAX_ENTRIES)
            .map(|i| format!("10.0.0.{i}"))
            .collect();
        let mut r = req(vec![], vec![]);
        r.trusted_proxies = many;
        assert!(matches!(
            validate_settings(&r),
            Err(AppError::Validation(_))
        ));
        let mut r = req(vec![], vec![]);
        r.trusted_proxies = (0..LIST_MAX_ENTRIES)
            .map(|i| format!("10.0.0.{i}"))
            .collect();
        assert_eq!(
            validate_settings(&r).unwrap().trusted_proxies.len(),
            LIST_MAX_ENTRIES
        );
        let long = format!("https://{}.ch", "a".repeat(ENTRY_MAX_CHARS));
        assert!(matches!(
            validate_settings(&req(vec![], vec![&long])),
            Err(AppError::Validation(_))
        ));
    }
}
