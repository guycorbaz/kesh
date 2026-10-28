//! Routes de gestion des prompts du serveur MCP — `/api/v1/admin/mcp/prompts`
//! (Story 32-10a-iii, refs #723 ; AC-iii6 à AC-iii10).
//!
//! - `GET  /api/v1/admin/mcp/prompts` — la synthèse de la société, **sans
//!   texte** : nom, standard, version en vigueur (numéro, origine, nature),
//!   dernière version, nombre à valider, « le standard du code diffère »,
//!   `version` (une requête, `mcp_prompts::synthese_par_societe`) ;
//! - `GET  /api/v1/admin/mcp/prompts/{id}` — le prompt et **tout** son
//!   historique, textes compris ; pour chaque version, le texte et la
//!   description **tels que `prompts/get` les servirait** (marque d'agent
//!   comprise), calculés par `prompts_mcp::composer_version` — la fonction
//!   du catalogue servi —, le verdict `servable` de cette même fonction, et
//!   chaque texte **rendu visible** (`envois_mcp::rendre_visible`) : l'écran
//!   n'a aucune logique Unicode propre (amendements D1 de la fiche 32-10b) ;
//! - `POST /api/v1/admin/mcp/prompts` — un prompt d'administrateur neuf → 201 ;
//! - `POST /api/v1/admin/mcp/prompts/{id}/versions` — une version
//!   d'administrateur, texte ou retrait → 201 ;
//! - `POST /api/v1/admin/mcp/prompts/{id}/versions/{numero}/approve` et
//!   `…/refuse` — la décision sur une proposition d'agent → 200 ;
//!   l'approbation **revalide** le contenu (D2) : une version que le serveur
//!   ne servirait pas est refusée en 400, comme à la création ;
//! - `POST /api/v1/admin/mcp/prompts/{id}/standard` — le retour au standard du
//!   code → 201.
//!
//! **Gardes** : les sept routes vivent dans le bloc `admin_routes` de `lib.rs`
//! — `require_admin_role` (Comptable : `403 FORBIDDEN`) puis `require_not_pat`
//! (PAT : `403 API_KEY_ADMIN_FORBIDDEN`) ; une clé MCP n'est pas reconnue par
//! `require_auth` (401) : **l'agent ne peut jamais approuver** (AC-iii7, P22).
//!
//! **Écritures** : par le dépôt de la 32-10a-i, **dans la transaction** de
//! leur entrée d'audit (AC-iii9) ; le contenu enregistré est le contenu
//! **normalisé** que rend `prompts_mcp::valider_contenu`, jamais le texte reçu
//! (revue de code P1 de la 32-10a-ii, E6). Un refus de validation (400)
//! n'ouvre aucune transaction : rien d'écrit, aucun audit (AC-iii8).
//!
//! ⚠️ **Précondition de lecture** (#783 point 1) : chaque transaction appelle
//! le dépôt **avant toute lecture** — sa première instruction est le verrou de
//! société, une lecture verrouillante qui ne fige aucune vue cohérente ; la
//! numérotation (`MAX(number) + 1`) et le contrôle de la dernière version
//! (`MAX(number)`), lectures simples sous ce verrou, voient donc l'état validé
//! par la transaction qui le tenait. Les lectures préalables (historique, pour
//! la revalidation et le détail d'audit) se font sur le pool, **hors** de la
//! transaction : une version ne change jamais de contenu, et un état décidé est
//! terminal.
//!
//! **Le journal** : détails `{promptId, nom, numero, …}`, **jamais** la
//! description ni le texte (l'`audit_log` est exporté et lisible, P14) ; et
//! sans `nom` pour le refus d'une proposition d'un prompt qui n'a **aucune**
//! version approuvée (un nom choisi par un agent et refusé n'entre pas au
//! journal exporté, F8). Aucun évènement `tracing` ne nomme un texte (P15).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::mcp_prompt::{
    Contenu, ErreurEcriturePrompt, Etat, McpPrompt, Nature, Origine, ResumeVersion, SynthesePrompt,
    VersionEcrite, VersionHistorique, contenu_attendu, differe_du_standard, en_vigueur,
    nom_conforme, nom_reserve,
};
use kesh_db::entities::mcp_prompt_standards::STANDARDS;
use kesh_db::errors::DbError;
use kesh_db::repositories::{audit_log, mcp_prompts};

use crate::AppState;
use crate::errors::{AppError, t};
use crate::middleware::auth::CurrentUser;
use crate::services::envois_mcp::rendre_visible;
use crate::services::prompts_mcp::{
    ChampPrompt, ContenuPrompt, ErreurContenu, MotifPrompt, OrigineVersion, RefusVersion,
    composer_version, valider_contenu,
};

// ---------------------------------------------------------------------------
// Réponses
// ---------------------------------------------------------------------------

/// Le résumé d'une version — sans texte.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeVersionResponse {
    pub numero: i32,
    /// `standard`, `admin` ou `agent`.
    pub origine: &'static str,
    /// `approved`, `pending` ou `refused`.
    pub etat: &'static str,
    /// `text` ou `withdrawal`.
    pub nature: &'static str,
}

impl From<ResumeVersion> for ResumeVersionResponse {
    fn from(r: ResumeVersion) -> Self {
        Self {
            numero: r.numero,
            origine: r.origine.as_str(),
            etat: r.etat.as_str(),
            nature: r.nature.as_str(),
        }
    }
}

/// Une ligne de la liste — **sans texte**.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptResumeResponse {
    pub id: i64,
    pub nom: String,
    pub standard: bool,
    /// Le verrou optimiste du prompt.
    pub version: i32,
    /// La version de plus grand numéro, quel que soit son état.
    pub derniere: ResumeVersionResponse,
    /// La version en vigueur, texte ou retrait ; nulle si aucune n'est
    /// approuvée.
    pub en_vigueur: Option<ResumeVersionResponse>,
    /// Le nombre de propositions à valider du prompt.
    pub a_valider: i64,
    /// Prompt standard qui ne sert pas le texte du code.
    pub standard_differe: bool,
}

impl From<SynthesePrompt> for PromptResumeResponse {
    fn from(s: SynthesePrompt) -> Self {
        Self {
            id: s.id,
            nom: s.nom,
            standard: s.is_standard,
            version: s.version,
            derniere: s.derniere.into(),
            en_vigueur: s.en_vigueur.map(Into::into),
            a_valider: s.a_valider,
            standard_differe: s.standard_differe,
        }
    }
}

/// Un texte et sa forme rendue visible (caractères cachés en codes
/// `\u{XXXX}`).
///
/// ⚠️ Pas de `Debug` : il porte un texte.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TexteRendu {
    pub exact: String,
    pub visible: String,
}

impl TexteRendu {
    fn de(s: &str) -> Self {
        Self {
            exact: s.to_string(),
            visible: rendre_visible(s),
        }
    }
}

/// Une version de l'historique (amendements D1, D1 étendu et D1 champs du
/// détail de la fiche 32-10b).
///
/// ⚠️ Pas de `Debug` : elle porte les textes.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDetailResponse {
    pub numero: i32,
    pub origine: &'static str,
    pub etat: &'static str,
    pub nature: &'static str,
    /// La description **enregistrée** ; nulle pour un retrait.
    pub description: Option<TexteRendu>,
    /// Le texte **enregistré** ; nul pour un retrait.
    pub texte: Option<TexteRendu>,
    /// `composer_version(…).is_ok()` : la version serait-elle servie si elle
    /// était en vigueur ? Toujours faux pour un retrait.
    pub servable: bool,
    /// La description telle que `prompts/get` la servirait (marque comprise
    /// pour une version d'agent) ; nulle si la version n'est pas servable.
    pub description_servie: Option<TexteRendu>,
    /// Le texte tel que `prompts/get` le servirait ; nul si la version n'est
    /// pas servable.
    pub texte_servi: Option<TexteRendu>,
    pub auteur_nom: Option<String>,
    pub decideur_nom: Option<String>,
    pub cle_auteur_id: Option<i64>,
    pub cle_nom: Option<String>,
    pub cle_revoquee: Option<bool>,
    pub cree_le: NaiveDateTime,
    pub decide_le: Option<NaiveDateTime>,
}

/// Le contenu d'un standard du code, tel qu'un retour au standard le ferait
/// servir.
///
/// ⚠️ Pas de `Debug` : il porte les textes.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardDuCodeResponse {
    pub description: TexteRendu,
    pub texte: TexteRendu,
}

/// Le détail d'un prompt et de tout son historique.
///
/// ⚠️ Pas de `Debug` : il porte les textes.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptDetailResponse {
    pub id: i64,
    pub nom: String,
    pub standard: bool,
    /// Le verrou optimiste du prompt — celui qu'attendent les écritures.
    pub version: i32,
    /// Le numéro de la version en vigueur (approuvée de plus grand numéro,
    /// texte ou retrait) ; nul si aucune.
    pub numero_en_vigueur: Option<i32>,
    pub standard_differe: bool,
    /// Pour un prompt standard encore au code : son texte du code. Nul sinon.
    pub standard_du_code: Option<StandardDuCodeResponse>,
    pub modifie_le: NaiveDateTime,
    pub versions: Vec<VersionDetailResponse>,
}

/// Ce qu'une écriture a écrit — **sans texte**.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionEcriteResponse {
    pub prompt_id: i64,
    pub nom: String,
    pub numero: i32,
    pub origine: &'static str,
    pub nature: &'static str,
    pub etat: &'static str,
    /// La `version` du prompt après l'écriture.
    pub version: i32,
}

impl From<VersionEcrite> for VersionEcriteResponse {
    fn from(v: VersionEcrite) -> Self {
        Self {
            prompt_id: v.prompt_id,
            nom: v.nom,
            numero: v.numero,
            origine: v.origine.as_str(),
            nature: v.nature.as_str(),
            etat: v.etat.as_str(),
            version: v.version,
        }
    }
}

// ---------------------------------------------------------------------------
// Requêtes
// ---------------------------------------------------------------------------

/// Un prompt d'administrateur neuf. ⚠️ Pas de `Debug` : il porte le texte.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreerPromptRequest {
    pub nom: String,
    pub description: String,
    pub texte: String,
}

/// La nature d'une version d'administrateur.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NatureRequete {
    /// Une description et un texte.
    Text,
    /// Le prompt cesse d'être servi.
    Withdrawal,
}

/// Une version d'administrateur. ⚠️ Pas de `Debug` : elle porte le texte.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NouvelleVersionRequest {
    pub version: i32,
    pub nature: NatureRequete,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub texte: Option<String>,
}

/// La `version` attendue du prompt (décision, retour au standard).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionAttendueRequest {
    pub version: i32,
}

// ---------------------------------------------------------------------------
// Traductions
// ---------------------------------------------------------------------------

fn version_attendue(v: i32) -> Result<i32, AppError> {
    if v < 1 {
        return Err(AppError::Validation(t(
            "mcp-prompt-version-invalid",
            "Version invalide : un entier supérieur ou égal à 1 est attendu.",
        )));
    }
    Ok(v)
}

fn erreur_nom_invalide() -> AppError {
    AppError::Validation(t(
        "mcp-prompt-name-invalid",
        "Nom de prompt invalide : de 1 à 64 caractères parmi les minuscules non accentuées, les \
         chiffres, le tiret et le souligné.",
    ))
}

fn erreur_nom_reserve() -> AppError {
    AppError::Validation(t(
        "mcp-prompt-name-reserved",
        "Nom de prompt réservé : un nom commençant par « kesh » est réservé aux prompts standard \
         de Kesh.",
    ))
}

/// Le refus d'un nom de prompt d'administrateur (AC-iii8) : non conforme, puis
/// réservé.
fn controler_nom(nom: &str) -> Result<(), AppError> {
    if !nom_conforme(nom) {
        return Err(erreur_nom_invalide());
    }
    if nom_reserve(nom) {
        return Err(erreur_nom_reserve());
    }
    Ok(())
}

/// Le refus d'un contenu (AC-iii8) : champ et motif, **jamais** le texte reçu.
fn erreur_de_contenu(e: ErreurContenu) -> AppError {
    let message = match (e.champ, e.motif) {
        (ChampPrompt::Description, MotifPrompt::Vide) => t(
            "mcp-prompt-description-empty",
            "La description du prompt est vide.",
        ),
        (ChampPrompt::Description, MotifPrompt::TropLong) => t(
            "mcp-prompt-description-too-long",
            "La description du prompt dépasse 500 caractères.",
        ),
        (ChampPrompt::Description, MotifPrompt::CaractereRefuse) => t(
            "mcp-prompt-description-hidden-character",
            "La description du prompt contient un saut de ligne, une tabulation ou un caractère \
             invisible, bidirectionnel ou de contrôle : retirez-le.",
        ),
        (ChampPrompt::Texte, MotifPrompt::Vide) => {
            t("mcp-prompt-text-empty", "Le texte du prompt est vide.")
        }
        (ChampPrompt::Texte, MotifPrompt::TropLong) => t(
            "mcp-prompt-text-too-long",
            "Le texte du prompt dépasse 16 000 caractères.",
        ),
        (ChampPrompt::Texte, MotifPrompt::CaractereRefuse) => t(
            "mcp-prompt-text-hidden-character",
            "Le texte du prompt contient un caractère invisible, bidirectionnel ou de contrôle : \
             retirez-le.",
        ),
    };
    AppError::Validation(message)
}

/// Le refus d'une version que le serveur ne servirait pas (D2) : le même code
/// et les mêmes messages qu'à la création.
fn erreur_de_version(r: RefusVersion) -> AppError {
    match r {
        RefusVersion::Contenu(e) => erreur_de_contenu(e),
        RefusVersion::AgentSansCle => AppError::Validation(t(
            "mcp-prompt-agent-without-key",
            "Cette proposition n'a pas de clé auteur : le serveur ne la servirait pas.",
        )),
    }
}

/// La traduction d'un refus du dépôt (AC-iii6, AC-iii8) : 404 introuvable ou
/// d'une autre société ; 409 verrou périmé, version non la dernière, déjà
/// décidée, non une proposition, standard inconnu du code ; 400 nom pris ou
/// refusé, contenu hors des bornes.
fn erreur_ecriture(e: ErreurEcriturePrompt) -> AppError {
    match e {
        ErreurEcriturePrompt::NotFound => AppError::Database(DbError::NotFound),
        ErreurEcriturePrompt::VersionPerimee => AppError::Database(DbError::OptimisticLockConflict),
        ErreurEcriturePrompt::PasLaDerniere => AppError::IllegalState(t(
            "mcp-prompt-not-latest",
            "Cette version n'est plus la dernière du prompt : relisez l'historique avant de \
             l'approuver.",
        )),
        ErreurEcriturePrompt::DejaDecidee => AppError::IllegalState(t(
            "mcp-prompt-already-decided",
            "Cette version a déjà été approuvée ou refusée.",
        )),
        ErreurEcriturePrompt::PasUneProposition => AppError::IllegalState(t(
            "mcp-prompt-not-a-proposal",
            "Seule une proposition d'agent s'approuve ou se refuse.",
        )),
        ErreurEcriturePrompt::StandardInconnu => AppError::IllegalState(t(
            "mcp-prompt-standard-unknown",
            "Ce prompt standard n'est plus dans cette version de Kesh : aucun texte du code où \
             revenir.",
        )),
        ErreurEcriturePrompt::NonStandard => AppError::IllegalState(t(
            "mcp-prompt-not-standard",
            "Ce prompt n'est pas un prompt standard : il n'a pas de texte du code.",
        )),
        ErreurEcriturePrompt::NomInvalide => erreur_nom_invalide(),
        ErreurEcriturePrompt::NomReserve => erreur_nom_reserve(),
        ErreurEcriturePrompt::NomDejaPris => AppError::Validation(t(
            "mcp-prompt-name-taken",
            "Un prompt de ce nom existe déjà.",
        )),
        // Inatteignables par ces routes : le contenu est validé avant
        // (`valider_contenu`, aux bornes de la table), et aucune route
        // d'administrateur ne propose.
        ErreurEcriturePrompt::ContenuInvalide | ErreurEcriturePrompt::PlafondAtteint => {
            AppError::Internal(format!("prompt MCP : refus inattendu du dépôt : {e}"))
        }
        ErreurEcriturePrompt::Db(e) => AppError::Database(e),
    }
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

/// Le prompt `id` de la société courante et son historique, ou 404 (un prompt
/// d'une autre société est indiscernable d'un prompt absent).
async fn historique(
    state: &AppState,
    company_id: i64,
    id: i64,
) -> Result<(McpPrompt, Vec<VersionHistorique>), AppError> {
    mcp_prompts::historique(&state.pool, company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))
}

/// Une version de l'historique, composée comme le catalogue la servirait.
fn version_detail(h: VersionHistorique) -> Result<VersionDetailResponse, AppError> {
    let v = &h.version;
    let contenu = v.contenu()?;
    let (description, texte, servie) = match &contenu {
        Contenu::Retrait => (None, None, None),
        Contenu::Texte { description, texte } => (
            Some(TexteRendu::de(description)),
            Some(TexteRendu::de(texte)),
            composer_version(
                OrigineVersion::from(v.origin),
                v.number,
                v.author_mcp_key_id,
                description,
                texte,
            )
            .ok(),
        ),
    };
    Ok(VersionDetailResponse {
        numero: v.number,
        origine: v.origin.as_str(),
        etat: v.state.as_str(),
        nature: v.nature.as_str(),
        description,
        texte,
        servable: servie.is_some(),
        description_servie: servie.as_ref().map(|s| TexteRendu::de(s.description())),
        texte_servi: servie.as_ref().map(|s| TexteRendu::de(s.texte())),
        auteur_nom: h.auteur_nom,
        decideur_nom: h.decideur_nom,
        cle_auteur_id: v.author_mcp_key_id,
        cle_nom: h.cle_nom,
        cle_revoquee: h.cle_revoquee,
        cree_le: v.created_at,
        decide_le: v.decided_at,
    })
}

/// Le détail : drapeaux calculés par les règles pures du dépôt
/// (`en_vigueur`, `differe_du_standard`, `contenu_attendu`), sur `STANDARDS`.
fn detail(
    prompt: McpPrompt,
    versions: Vec<VersionHistorique>,
) -> Result<PromptDetailResponse, AppError> {
    let brutes: Vec<_> = versions.iter().map(|h| h.version.clone()).collect();
    let vigueur = en_vigueur(&brutes);
    let numero_en_vigueur = vigueur.map(|v| v.number);
    let standard_differe = if prompt.is_standard {
        let contenu = vigueur.map(|v| v.contenu()).transpose()?;
        differe_du_standard(
            &prompt.name,
            vigueur.zip(contenu.as_ref()).map(|(v, c)| (v.origin, c)),
            STANDARDS,
        )
    } else {
        false
    };
    let standard_du_code = if prompt.is_standard {
        match contenu_attendu(&prompt.name, STANDARDS) {
            Contenu::Texte { description, texte } => Some(StandardDuCodeResponse {
                description: TexteRendu::de(&description),
                texte: TexteRendu::de(&texte),
            }),
            Contenu::Retrait => None,
        }
    } else {
        None
    };
    Ok(PromptDetailResponse {
        id: prompt.id,
        nom: prompt.name,
        standard: prompt.is_standard,
        version: prompt.version,
        numero_en_vigueur,
        standard_differe,
        standard_du_code,
        modifie_le: prompt.updated_at,
        versions: versions
            .into_iter()
            .map(version_detail)
            .collect::<Result<_, _>>()?,
    })
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/admin/mcp/prompts` — la synthèse de la société, sans texte.
pub async fn lister_prompts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PromptResumeResponse>>, AppError> {
    let synthese =
        mcp_prompts::synthese_par_societe(&state.pool, current_user.company_id, STANDARDS).await?;
    Ok(Json(synthese.into_iter().map(Into::into).collect()))
}

/// `GET /api/v1/admin/mcp/prompts/{id}` — le prompt et tout son historique ;
/// 404 pour un prompt absent ou d'une autre société.
pub async fn detail_prompt(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<PromptDetailResponse>, AppError> {
    let (prompt, versions) = historique(&state, current_user.company_id, id).await?;
    Ok(Json(detail(prompt, versions)?))
}

/// `POST /api/v1/admin/mcp/prompts` — un prompt d'administrateur neuf et sa
/// version 1, approuvée ; audit `mcp_prompt.created`. 400 nom non conforme,
/// réservé ou pris, contenu refusé — rien d'écrit, aucun audit.
pub async fn creer_prompt(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreerPromptRequest>,
) -> Result<(StatusCode, Json<VersionEcriteResponse>), AppError> {
    controler_nom(&req.nom)?;
    let contenu = valider_contenu(&req.description, &req.texte).map_err(erreur_de_contenu)?;
    let mut tx = begin(&state).await?;
    let ecrite = mcp_prompts::creer_prompt_admin_in_tx(
        &mut tx,
        current_user.company_id,
        current_user.user_id,
        &req.nom,
        contenu.description(),
        contenu.texte(),
    )
    .await
    .map_err(erreur_ecriture)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_prompt.created",
            "mcp_prompt",
            ecrite.prompt_id,
            Some(serde_json::json!({
                "promptId": ecrite.prompt_id,
                "nom": ecrite.nom,
                "numero": ecrite.numero,
            })),
        ),
    )
    .await?;
    commit(tx).await?;
    Ok((StatusCode::CREATED, Json(ecrite.into())))
}

/// Le contenu d'une version d'administrateur, contrôlé : un texte validé et
/// normalisé, ou un retrait sans description ni texte.
fn contenu_de(req: &NouvelleVersionRequest) -> Result<Contenu, AppError> {
    match (req.nature, &req.description, &req.texte) {
        (NatureRequete::Withdrawal, None, None) => Ok(Contenu::Retrait),
        (NatureRequete::Withdrawal, _, _) => Err(AppError::Validation(t(
            "mcp-prompt-withdrawal-with-content",
            "Un retrait ne porte ni description ni texte.",
        ))),
        (NatureRequete::Text, Some(d), Some(x)) => {
            let c: ContenuPrompt = valider_contenu(d, x).map_err(erreur_de_contenu)?;
            Ok(Contenu::Texte {
                description: c.description().to_string(),
                texte: c.texte().to_string(),
            })
        }
        (NatureRequete::Text, _, _) => Err(AppError::Validation(t(
            "mcp-prompt-text-required",
            "Une version de texte porte une description et un texte.",
        ))),
    }
}

/// `POST /api/v1/admin/mcp/prompts/{id}/versions` — une version
/// d'administrateur, texte ou retrait (un standard devient personnalisé) ;
/// audit `mcp_prompt.version_created`. 404, 409 verrou périmé, 400 contenu.
pub async fn ajouter_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<NouvelleVersionRequest>,
) -> Result<(StatusCode, Json<VersionEcriteResponse>), AppError> {
    let version = version_attendue(req.version)?;
    let contenu = contenu_de(&req)?;
    let mut tx = begin(&state).await?;
    let ecrite = mcp_prompts::ajouter_version_admin_in_tx(
        &mut tx,
        current_user.company_id,
        id,
        version,
        current_user.user_id,
        &contenu,
    )
    .await
    .map_err(erreur_ecriture)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_prompt.version_created",
            "mcp_prompt",
            ecrite.prompt_id,
            Some(serde_json::json!({
                "promptId": ecrite.prompt_id,
                "nom": ecrite.nom,
                "numero": ecrite.numero,
                "origine": ecrite.origine.as_str(),
                "nature": ecrite.nature.as_str(),
            })),
        ),
    )
    .await?;
    commit(tx).await?;
    Ok((StatusCode::CREATED, Json(ecrite.into())))
}

/// La revalidation d'une proposition avant son approbation (D2) : une version
/// d'agent **à valider**, de texte, que `composer_version` refuserait — nom
/// non conforme, clé auteur absente, contenu refusé — ne s'approuve pas (400,
/// le code de la création). Les autres cas (absente, décidée, non une
/// proposition) sont tranchés par le dépôt, sous verrou.
fn revalider(
    prompt: &McpPrompt,
    versions: &[VersionHistorique],
    numero: i32,
) -> Result<(), AppError> {
    let Some(v) = versions
        .iter()
        .map(|h| &h.version)
        .find(|v| v.number == numero)
    else {
        return Ok(());
    };
    if v.origin != Origine::Agent || v.state != Etat::EnAttente || v.nature != Nature::Texte {
        return Ok(());
    }
    if !nom_conforme(&prompt.name) {
        return Err(erreur_nom_invalide());
    }
    if let Contenu::Texte { description, texte } = v.contenu()? {
        composer_version(
            OrigineVersion::Agent,
            v.number,
            v.author_mcp_key_id,
            &description,
            &texte,
        )
        .map_err(erreur_de_version)?;
    }
    Ok(())
}

/// `POST /api/v1/admin/mcp/prompts/{id}/versions/{numero}/approve` —
/// l'approbation de la **dernière** version, une proposition d'agent à
/// valider, **revalidée** (D2) ; audit `mcp_prompt.version_approved`. 404 ;
/// 409 verrou périmé, non la dernière, déjà décidée ; 400 non servable.
pub async fn approuver_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((id, numero)): Path<(i64, i32)>,
    Json(req): Json<VersionAttendueRequest>,
) -> Result<Json<VersionEcriteResponse>, AppError> {
    let version = version_attendue(req.version)?;
    let (prompt, versions) = historique(&state, current_user.company_id, id).await?;
    revalider(&prompt, &versions, numero)?;
    let mut tx = begin(&state).await?;
    let ecrite = mcp_prompts::approuver_in_tx(
        &mut tx,
        current_user.company_id,
        id,
        numero,
        version,
        current_user.user_id,
    )
    .await
    .map_err(erreur_ecriture)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_prompt.version_approved",
            "mcp_prompt",
            ecrite.prompt_id,
            Some(serde_json::json!({
                "promptId": ecrite.prompt_id,
                "nom": ecrite.nom,
                "numero": ecrite.numero,
            })),
        ),
    )
    .await?;
    commit(tx).await?;
    Ok(Json(ecrite.into()))
}

/// Le détail d'audit d'un refus : **sans `nom`** si le prompt n'a aucune
/// version approuvée (F8, P14). Lu avant la transaction : un état approuvé est
/// terminal, et une approbation concurrente ne ferait qu'omettre un nom
/// devenu servi — le sens prudent.
fn detail_du_refus(ecrite: &VersionEcrite, versions: &[VersionHistorique]) -> serde_json::Value {
    let deja_approuve = versions.iter().any(|h| h.version.state == Etat::Approuvee);
    if deja_approuve {
        serde_json::json!({
            "promptId": ecrite.prompt_id,
            "nom": ecrite.nom,
            "numero": ecrite.numero,
        })
    } else {
        serde_json::json!({
            "promptId": ecrite.prompt_id,
            "numero": ecrite.numero,
        })
    }
}

/// `POST /api/v1/admin/mcp/prompts/{id}/versions/{numero}/refuse` — le refus
/// d'une proposition d'agent à valider, dernière ou non ; audit
/// `mcp_prompt.version_refused`. 404 ; 409 verrou périmé, déjà décidée.
pub async fn refuser_version(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((id, numero)): Path<(i64, i32)>,
    Json(req): Json<VersionAttendueRequest>,
) -> Result<Json<VersionEcriteResponse>, AppError> {
    let version = version_attendue(req.version)?;
    let (_, versions) = historique(&state, current_user.company_id, id).await?;
    let mut tx = begin(&state).await?;
    let ecrite = mcp_prompts::refuser_in_tx(
        &mut tx,
        current_user.company_id,
        id,
        numero,
        version,
        current_user.user_id,
    )
    .await
    .map_err(erreur_ecriture)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_prompt.version_refused",
            "mcp_prompt",
            ecrite.prompt_id,
            Some(detail_du_refus(&ecrite, &versions)),
        ),
    )
    .await?;
    commit(tx).await?;
    Ok(Json(ecrite.into()))
}

/// `POST /api/v1/admin/mcp/prompts/{id}/standard` — le retour au standard du
/// code : une version `standard` au texte de `STANDARDS`, qui redevient la
/// version en vigueur ; audit `mcp_prompt.standard_restored`. 404 ; 409
/// verrou périmé, prompt non standard, standard inconnu du code.
pub async fn revenir_au_standard(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<VersionAttendueRequest>,
) -> Result<(StatusCode, Json<VersionEcriteResponse>), AppError> {
    let version = version_attendue(req.version)?;
    let (prompt, _) = historique(&state, current_user.company_id, id).await?;
    if !prompt.is_standard {
        return Err(erreur_ecriture(ErreurEcriturePrompt::NonStandard));
    }
    let standard = STANDARDS
        .iter()
        .find(|s| s.nom == prompt.name)
        .ok_or_else(|| erreur_ecriture(ErreurEcriturePrompt::StandardInconnu))?;
    let mut tx = begin(&state).await?;
    let ecrite = mcp_prompts::revenir_au_standard_in_tx(
        &mut tx,
        current_user.company_id,
        id,
        version,
        standard,
    )
    .await
    .map_err(erreur_ecriture)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "mcp_prompt.standard_restored",
            "mcp_prompt",
            ecrite.prompt_id,
            Some(serde_json::json!({
                "promptId": ecrite.prompt_id,
                "nom": ecrite.nom,
                "numero": ecrite.numero,
            })),
        ),
    )
    .await?;
    commit(tx).await?;
    Ok((StatusCode::CREATED, Json(ecrite.into())))
}
