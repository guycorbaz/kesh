//! Les lectures propres au serveur MCP — Story 32-6a (refs #723).
//!
//! - [`reglages_actifs`] : les réglages des sociétés dont le serveur MCP est
//!   activé, lus à chaque requête vers `/mcp` par la garde du transport, sur
//!   la `Lecture` **ordinaire** (C-32-6-38) — jamais sur le pool des outils.
//! - [`etat`] : ce que rend l'outil de démonstration `kesh_version` — la
//!   version du binaire et le mode de la session qui le sert. **Aucune donnée
//!   des livres.**
//!
//! - [`journaliser_appel`] (Story 32-6b) : l'entrée du journal d'audit de
//!   chaque `tools/call`, sur la `Trace` — la seule fonction de ce module qui
//!   écrit.
//!
//! ⚠️ **Dérogation écrite aux règles R1, R2 et R7 de `services`** pour
//! [`reglages_actifs`] (validation P2 de la 32-6, R14) : elle précède
//! l'authentification — aucune portée ne s'y applique —, n'a donc ni
//! `ContexteAcces` ni `exiger`, et son test de refus est sans objet. [`etat`],
//! lui, suit le motif (`exiger` d'abord). **Même dérogation** pour
//! [`journaliser_appel`] (AC-b1) : c'est une écriture d'audit, non une lecture
//! des livres — elle n'a ni `ContexteAcces` ni `exiger`, et doit s'écrire pour
//! un appel **refusé** comme pour un appel servi.

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{McpServerSettings, Role, SendValidation};
use kesh_db::repositories::{audit_log, mcp_server_settings};

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::{Lecture, Trace};

/// Les réglages des sociétés **activées**, par `company_id` croissant (AC-a2).
///
/// Une ligne illisible (une liste qui n'est pas un tableau de chaînes) est
/// écartée et journalisée en `error!` avec l'id de sa société — sans quoi le
/// serveur MCP de cette société cesserait de répondre sans trace visible au
/// niveau par défaut ; les autres lignes restent servies (C-32-6-6). Une erreur
/// de la base est rendue : la garde ferme alors l'accès (fail closed).
pub async fn reglages_actifs(lecture: &Lecture) -> Result<Vec<McpServerSettings>, AppError> {
    let (lues, ecartees) = mcp_server_settings::list_enabled(lecture.pool()).await?;
    for company_id in ecartees {
        tracing::error!(
            company_id,
            "serveur MCP : ligne de réglages illisible, la société n'est pas servie"
        );
    }
    Ok(lues)
}

/// Exigence de [`etat`] : portée `BooksRead` (le seul outil de la 32-6a,
/// `kesh_version`) ; tous rôles côté API, qui ne l'appelle pas.
pub const EXIGENCE_ETAT: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::BooksRead),
};

/// L'état de la session qui sert un outil.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EtatServeur {
    /// `CARGO_PKG_VERSION` du binaire.
    pub version: &'static str,
    /// La session qui a lu est-elle en `READ ONLY` ? Lu par **une** requête sur
    /// la `Lecture` reçue, dans la requête même.
    pub lecture_seule: bool,
    /// Le délai au-delà duquel MariaDB interrompt une instruction de cette
    /// session (`@@session.max_statement_time`, secondes ; `0` : aucun) — revue
    /// P1 de la 32-6a, E1.
    pub delai_instruction_s: f64,
}

/// La version du binaire, le mode et le délai d'instruction de la session de
/// `lecture` (AC-a28).
///
/// ⚠️ La lecture du nom de variable de session est l'**exception écrite** de la
/// garde `lecture_seule_jamais_defaite` (`tests/mcp_gardes_lexicales.rs`) : elle
/// lit le mode, elle ne le change pas.
pub async fn etat(acces: &ContexteAcces, lecture: &Lecture) -> Result<EtatServeur, AppError> {
    acces.exiger(EXIGENCE_ETAT)?;
    let (mode, delai): (i64, f64) =
        sqlx::query_as("SELECT @@session.tx_read_only, @@session.max_statement_time")
            .fetch_one(lecture.pool())
            .await
            .map_err(kesh_db::errors::map_db_error)?;
    Ok(EtatServeur {
        version: env!("CARGO_PKG_VERSION"),
        lecture_seule: mode != 0,
        delai_instruction_s: delai,
    })
}

// ---------------------------------------------------------------------------
// Le journal des appels d'outil — Story 32-6b (AC-b1 à AC-b4)
// ---------------------------------------------------------------------------

/// Le nom écrit au journal quand l'appel ne désigne aucun outil servi à la clé
/// — nom absent, inconnu, ou hors des portées de la clé — ou que la demande est
/// mal formée (AC-a19 ; C-32-6-21). Le nom fourni par le client n'est
/// **jamais** écrit.
pub const OUTIL_INCONNU: &str = "(inconnu)";

/// L'issue d'un appel d'outil — une énumération **fermée** (AC-b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueAppel {
    /// L'outil a rendu ses données.
    Servi,
    /// La clé est soumise à la validation avant envoi (refus avant exécution).
    ValidationRequise,
    /// Outil inconnu, hors des portées de la clé, ou refusé par son service :
    /// **une seule** issue (aucun oracle, C-32-6-21).
    Inconnu,
    /// Arguments refusés par l'outil, ou demande mal formée (AC-a19).
    ArgumentsInvalides,
    /// L'objet demandé n'existe pas.
    Introuvable,
    /// Toute autre erreur, dont le panic de l'outil (C-32-6-58).
    Erreur,
    /// Le délai du registre est dépassé.
    DelaiDepasse,
}

impl IssueAppel {
    /// Le code écrit au détail.
    pub fn code(self) -> &'static str {
        match self {
            Self::Servi => "servi",
            Self::ValidationRequise => "validation_requise",
            Self::Inconnu => "inconnu",
            Self::ArgumentsInvalides => "arguments_invalides",
            Self::Introuvable => "introuvable",
            Self::Erreur => "erreur",
            Self::DelaiDepasse => "delai_depasse",
        }
    }
}

/// Ce qu'on journalise d'un appel d'outil — **ni ses arguments, ni ses
/// données** (AC-b2) : aucun champ ne peut en porter.
#[derive(Debug, Clone, Copy)]
pub struct AppelOutil {
    /// `mcp_keys.id` de la clé.
    pub cle_id: i64,
    /// Le créateur de la clé : l'acteur de l'entrée (C-32-6-21).
    pub createur: i64,
    /// Le nom d'un outil du registre, ou [`OUTIL_INCONNU`]. `&'static str` : un
    /// texte du binaire — le nom reçu du client, emprunté au corps de la
    /// requête, n'a pas cette durée de vie.
    pub outil: &'static str,
    /// L'issue.
    pub issue: IssueAppel,
    /// La durée de l'appel, de sa prise en charge par le répartiteur à son
    /// issue, en millisecondes.
    pub duree_ms: u64,
    /// La politique de validation de la clé.
    pub validation: SendValidation,
}

/// Écrit l'entrée `mcp.tool_called` d'un appel d'outil, sur la `Trace`
/// (AC-b1).
///
/// Le détail est **exactement** `{cleMcpId, outil, issue, dureeMs,
/// validation}` ; l'entrée est au nom du créateur de la clé (`actor_type =
/// 'user'`), type d'entité `mcp_key`, `entity_id` = l'id de la clé.
///
/// ⛔ **Fail closed** (C-32-6-30) : une erreur est **rendue**, jamais avalée —
/// le répartiteur ne laisse alors partir aucune donnée. Contrairement aux
/// traces des rapports (`rapports.rs`), qui sont *best-effort*.
pub async fn journaliser_appel(trace: &Trace, appel: &AppelOutil) -> Result<(), AppError> {
    let mut tx = trace
        .pool()
        .begin()
        .await
        .map_err(kesh_db::errors::map_db_error)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            appel.createur,
            "mcp.tool_called",
            "mcp_key",
            appel.cle_id,
            Some(serde_json::json!({
                "cleMcpId": appel.cle_id,
                "outil": appel.outil,
                "issue": appel.issue.code(),
                "dureeMs": appel.duree_ms,
                "validation": appel.validation.as_str(),
            })),
        ),
    )
    .await?;
    tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
    Ok(())
}
