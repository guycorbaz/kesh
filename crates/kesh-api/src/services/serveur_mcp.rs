//! Les lectures propres au serveur MCP — Story 32-6a (refs #723).
//!
//! - [`reglages_actifs`] : les réglages des sociétés dont le serveur MCP est
//!   activé, lus à chaque requête vers `/mcp` par la garde du transport, sur
//!   la `Lecture` **ordinaire** (C-32-6-38) — jamais sur le pool des outils.
//! - [`etat`] : ce que rend l'outil de démonstration `kesh_version` — la
//!   version du binaire et le mode de la session qui le sert. **Aucune donnée
//!   des livres.**
//!
//! ⚠️ **Dérogation écrite aux règles R1, R2 et R7 de `services`** pour
//! [`reglages_actifs`] (validation P2 de la 32-6, R14) : elle précède
//! l'authentification — aucune portée ne s'y applique —, n'a donc ni
//! `ContexteAcces` ni `exiger`, et son test de refus est sans objet. [`etat`],
//! lui, suit le motif (`exiger` d'abord).

use kesh_db::entities::{McpServerSettings, Role};
use kesh_db::repositories::mcp_server_settings;

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

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
