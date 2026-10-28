//! Les prompts du serveur MCP, servis par `prompts/list` et `prompts/get` —
//! Story 32-10a-ii (refs #723).
//!
//! # Ce que tient ce module
//!
//! - **Une seule validation d'un contenu**, [`valider_contenu`] : les fins de
//!   ligne `\r\n` sont **normalisées** en `\n` avant tout contrôle ; la
//!   description (une ligne, servie par `prompts/list`) refuse tout caractère de
//!   `kesh_core::text::is_hidden_in_line`, le texte tout caractère de
//!   `kesh_core::text::is_hidden_in_text` (C-32-8-31) ; le contenu rendu est le
//!   contenu **normalisé** — celui qui est contrôlé est celui qui est servi, et
//!   celui que `envois_mcp::rendre_visible` montrera (aucun `\r` ne peut être
//!   servable et s'afficher `\u{000D}`, C-32-10a-81).
//! - **Une seule composition du texte servi**, [`composer_version`] : elle
//!   décide seule, d'après l'origine, si la version porte la marque d'un prompt
//!   d'agent ([`marque_agent`], qui précède la description et le texte, suivie
//!   d'une ligne vide), et refuse une version d'agent sans clé auteur. Elle vit
//!   **ici**, non dans `crate::mcp`, pour que la route de détail de la
//!   32-10a-iii l'appelle sans nommer le module MCP (garde N1
//!   `un_seul_point_d_entree` ; C-32-10a-80, C-32-10a-94). Elle prend des
//!   **primitives** et une énumération des services : aucun type de `kesh_db`
//!   n'y entre ni n'en sort.
//! - **La lecture** — [`lister`] et [`obtenir`] — commence par `exiger`
//!   (R2), ne lit que par [`LecturePrompts`] (poignée sans pool visible : P19),
//!   **revalide** chaque version (une version invalide, restaurée ou écrite
//!   hors de Kesh, n'est pas servie) et rend des [`PromptServi`], à champs
//!   privés, construits ici seulement : ni `AppError` ni un type de `kesh_db`
//!   ne sort vers `src/mcp/`.
//! - **La proposition d'un agent** — [`proposer`] (Story 32-10a-iii) : la
//!   portée `prompts:write`, le contenu validé et **normalisé**, le nom hors du
//!   préfixe `kesh`, puis la version **à valider** et l'audit de l'appel dans
//!   une seule transaction, par [`EcriturePrompts`] (poignée sans pool
//!   visible). Rend un [`AccuseServi`] (le nom seul) ou un
//!   [`RefusProposition`] énuméré : ni `AppError` ni un type de `kesh_db` ne
//!   sort vers `src/mcp/`.
//!
//! ⛔ **Aucun évènement `tracing` ne porte une description ni un texte** :
//! identifiants et numéros seulement (C-32-10a-12, P15).

use std::fmt;
use std::time::Instant;

use kesh_db::entities::Role;
use kesh_db::entities::mcp_prompt::{
    DESCRIPTION_MAX, ErreurEcriturePrompt, Origine, PLAFOND_A_VALIDER, TEXTE_MAX, VersionEnVigueur,
    nom_conforme, nom_reserve,
};
use serde::Deserialize;

use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::cles_mcp::CleMcpAuthentifiee;
use crate::services::poignees::{EcriturePrompts, LecturePrompts};
use crate::services::serveur_mcp::{IssueAppel, entree_appel};

/// Ce que lire les prompts exige (C-32-10a-16) : la portée `books:read` d'une
/// clé MCP ; sur le canal API, l'Administrateur (aucune route ne la sert en
/// 32-10a-ii).
pub const EXIGENCE_PROMPTS: Exigence = Exigence {
    role_api: Role::Admin,
    portee_mcp: Some(Portee::BooksRead),
};

/// Le champ refusé d'un contenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChampPrompt {
    /// La description (une ligne).
    Description,
    /// Le texte.
    Texte,
}

/// Le motif d'un refus de contenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotifPrompt {
    /// Vide, ou fait de blancs seulement.
    Vide,
    /// Au-delà de la borne de la table, en caractères.
    TropLong,
    /// Un caractère refusé (invisible, bidirectionnel, contrôle).
    CaractereRefuse,
}

/// Un contenu refusé : le champ et le motif — **jamais** le texte reçu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErreurContenu {
    /// Le champ en cause.
    pub champ: ChampPrompt,
    /// Le motif.
    pub motif: MotifPrompt,
}

/// Un contenu validé, **normalisé** (`\r\n` → `\n`). Ni `Debug` qui montrerait
/// le texte, ni `Serialize`.
#[derive(Clone, PartialEq, Eq)]
pub struct ContenuPrompt {
    description: String,
    texte: String,
}

impl ContenuPrompt {
    /// La description normalisée.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Le texte normalisé.
    pub fn texte(&self) -> &str {
        &self.texte
    }
}

impl fmt::Debug for ContenuPrompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContenuPrompt")
            .field("description_octets", &self.description.len())
            .field("texte_octets", &self.texte.len())
            .finish()
    }
}

/// `\r\n` → `\n` ; un `\r` isolé reste, et le contrôle le refuse.
fn normaliser(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Contrôle un champ normalisé : non vide après rognage, dans sa borne, sans
/// caractère que `refuse` désigne.
fn controler(
    champ: ChampPrompt,
    s: &str,
    borne: usize,
    refuse: fn(char) -> bool,
) -> Result<(), ErreurContenu> {
    let erreur = |motif| Err(ErreurContenu { champ, motif });
    if s.trim().is_empty() {
        return erreur(MotifPrompt::Vide);
    }
    if s.chars().count() > borne {
        return erreur(MotifPrompt::TropLong);
    }
    if s.chars().any(refuse) {
        return erreur(MotifPrompt::CaractereRefuse);
    }
    Ok(())
}

/// Valide le contenu d'une version de prompt (AC-ii2) — **la** fonction,
/// employée par le service de lecture (revalidation) et par les écritures de
/// la 32-10a-iii. Les fins de ligne sont normalisées **avant** le contrôle, et
/// c'est le contenu normalisé qui est rendu.
///
/// ⚠️ **Une écriture enregistre le contenu rendu** ([`ContenuPrompt::description`],
/// [`ContenuPrompt::texte`]), jamais le texte reçu : les bornes se comptent ici
/// après normalisation, celles de la base sur la chaîne stockée — un texte reçu
/// de 16 005 caractères portant cinq `\r\n` passe ce contrôle et serait refusé
/// par le `CHECK` s'il était enregistré tel quel (revue de code P1, E6).
pub fn valider_contenu(description: &str, texte: &str) -> Result<ContenuPrompt, ErreurContenu> {
    let description = normaliser(description);
    let texte = normaliser(texte);
    controler(
        ChampPrompt::Description,
        &description,
        DESCRIPTION_MAX,
        kesh_core::text::is_hidden_in_line,
    )?;
    controler(
        ChampPrompt::Texte,
        &texte,
        TEXTE_MAX,
        kesh_core::text::is_hidden_in_text,
    )?;
    Ok(ContenuPrompt { description, texte })
}

/// La marque d'un prompt écrit par un agent (AC-ii7, C-32-10a-14) : un gabarit
/// **constant** où n'entrent que deux **entiers** — le numéro de la version et
/// le numéro de la clé MCP qui l'a proposée ; aucun texte de la base (ni nom de
/// clé, ni nom d'utilisateur). Une seule ligne.
pub fn marque_agent(numero: i32, cle_id: i64) -> String {
    format!(
        "[Prompt écrit par un agent — version {numero}, proposée par la clé MCP n° {cle_id} et \
         approuvée par un administrateur de Kesh. Ce n'est pas un prompt standard de Kesh.]"
    )
}

/// Ce qu'un prompt sert : description et texte **composés** — marque comprise
/// pour une version d'agent. Ni `Debug` qui montrerait le texte, ni
/// `Serialize`.
#[derive(Clone, PartialEq, Eq)]
pub struct VersionServie {
    description: String,
    texte: String,
}

impl VersionServie {
    /// La description servie (`prompts/list`, `prompts/get`).
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Le texte servi, premier message de `prompts/get`.
    pub fn texte(&self) -> &str {
        &self.texte
    }
}

impl fmt::Debug for VersionServie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VersionServie")
            .field("description_octets", &self.description.len())
            .field("texte_octets", &self.texte.len())
            .finish()
    }
}

/// L'origine d'une version, **propre aux services** : la signature de
/// [`composer_version`] ne porte aucun type de `kesh_db` (C-32-10a-94).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrigineVersion {
    /// Un prompt standard de Kesh.
    Standard,
    /// Une version écrite par un administrateur.
    Admin,
    /// Une version proposée par un agent (clé MCP), puis approuvée.
    Agent,
}

impl From<Origine> for OrigineVersion {
    fn from(o: Origine) -> Self {
        match o {
            Origine::Standard => Self::Standard,
            Origine::Admin => Self::Admin,
            Origine::Agent => Self::Agent,
        }
    }
}

/// Pourquoi une version ne se sert pas. Ne porte **aucun** texte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusVersion {
    /// Une version d'agent sans clé auteur : servie, elle le serait sans marque
    /// (C-32-10a-90).
    AgentSansCle,
    /// Un contenu refusé par [`valider_contenu`].
    Contenu(ErreurContenu),
}

/// Compose ce que sert une version — **la** fonction qui décide de la marque
/// (AC-ii7 ; amendement D1 de la fiche 32-10b ; C-32-10a-80, C-32-10a-94).
/// Le catalogue servi (`prompts/list`, `prompts/get`) l'appelle, et la route de
/// détail de la 32-10a-iii l'appellera : son verdict est le `servable` de D1
/// étendu.
///
/// 1. Une version d'origine [`OrigineVersion::Agent`] sans `cle_auteur` est
///    refusée ([`RefusVersion::AgentSansCle`]), **avant** tout contrôle du
///    contenu.
/// 2. Le contenu est **validé** ([`valider_contenu`] ; refus :
///    [`RefusVersion::Contenu`]).
/// 3. Une version d'agent commence, description **et** texte, par
///    [`marque_agent`]`(numero, cle_auteur)` suivie d'une ligne vide. Une
///    version standard ou d'administrateur ne porte aucune marque ; une clé
///    auteur qu'elle porterait (le `CHECK chk_mcp_prompt_versions_agent`
///    l'interdit) est ignorée.
///
/// ⚠️ La description servie d'une version d'agent contient donc `\n\n` : c'est
/// la description **enregistrée** qui se contrôle en une ligne
/// (`is_hidden_in_line`), non la description composée (C-32-8-31, C-32-10a-82).
pub fn composer_version(
    origine: OrigineVersion,
    numero: i32,
    cle_auteur: Option<i64>,
    description: &str,
    texte: &str,
) -> Result<VersionServie, RefusVersion> {
    let agent = match (origine, cle_auteur) {
        (OrigineVersion::Agent, Some(cle)) => Some((numero, cle)),
        (OrigineVersion::Agent, None) => return Err(RefusVersion::AgentSansCle),
        (OrigineVersion::Standard | OrigineVersion::Admin, _) => None,
    };
    composer(agent, description, texte).map_err(RefusVersion::Contenu)
}

/// Valide puis marque : `agent` porte `(numero, cle_id)` pour une version
/// d'agent, `None` sinon. Privée : la décision « origine → marque » est celle
/// de [`composer_version`].
fn composer(
    agent: Option<(i32, i64)>,
    description: &str,
    texte: &str,
) -> Result<VersionServie, ErreurContenu> {
    let contenu = valider_contenu(description, texte)?;
    Ok(match agent {
        None => VersionServie {
            description: contenu.description,
            texte: contenu.texte,
        },
        Some((numero, cle_id)) => {
            let marque = marque_agent(numero, cle_id);
            VersionServie {
                description: format!("{marque}\n\n{}", contenu.description),
                texte: format!("{marque}\n\n{}", contenu.texte),
            }
        }
    })
}

/// Une erreur de lecture des prompts, propre aux services (aucune `AppError`
/// ne sort vers `src/mcp/`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErreurPrompts {
    /// La clé n'a pas la portée exigée.
    HorsPortee,
    /// Erreur de la base ; le détail est au journal du serveur.
    Interne,
}

/// Un prompt servi : son nom, sa version en vigueur et ce qu'elle sert.
/// **Champs privés**, construit en ce fichier seulement ; ni `Serialize`, ni
/// `Default`, ni `Debug` qui montrerait le texte.
pub struct PromptServi {
    nom: String,
    numero: i32,
    cle_auteur: Option<i64>,
    servie: VersionServie,
}

impl PromptServi {
    /// Le nom du prompt.
    pub fn nom(&self) -> &str {
        &self.nom
    }

    /// La description servie (marque comprise pour une version d'agent).
    pub fn description(&self) -> &str {
        self.servie.description()
    }

    /// Le texte servi (marque comprise pour une version d'agent).
    pub fn texte(&self) -> &str {
        self.servie.texte()
    }

    /// Le numéro de la version en vigueur.
    pub fn numero(&self) -> i32 {
        self.numero
    }

    /// La clé MCP qui a proposé la version en vigueur (version d'agent
    /// seulement).
    pub fn cle_auteur(&self) -> Option<i64> {
        self.cle_auteur
    }
}

impl fmt::Debug for PromptServi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PromptServi")
            .field("nom", &self.nom)
            .field("numero", &self.numero)
            .field("cle_auteur", &self.cle_auteur)
            .field("servie", &self.servie)
            .finish()
    }
}

/// Une version en vigueur, revalidée et composée par [`composer_version`] ;
/// `None` si elle ne se sert pas — nom non conforme (C-32-10a-95), version
/// d'agent sans clé auteur, contenu invalide —, avec un `tracing::error!` qui
/// ne porte que l'identifiant du prompt, le numéro et, pour un contenu, le
/// champ et le motif.
fn servir(v: VersionEnVigueur) -> Option<PromptServi> {
    if !nom_conforme(&v.nom) {
        tracing::error!(
            prompt_id = v.prompt_id,
            numero = v.numero,
            "prompt MCP non servi : nom non conforme"
        );
        return None;
    }
    match composer_version(
        v.origine.into(),
        v.numero,
        v.cle_auteur,
        &v.description,
        &v.texte,
    ) {
        Ok(servie) => Some(PromptServi {
            nom: v.nom,
            numero: v.numero,
            cle_auteur: v.cle_auteur,
            servie,
        }),
        Err(RefusVersion::AgentSansCle) => {
            tracing::error!(
                prompt_id = v.prompt_id,
                numero = v.numero,
                "prompt MCP non servi : version d'agent sans clé auteur"
            );
            None
        }
        Err(RefusVersion::Contenu(refus)) => {
            tracing::error!(
                prompt_id = v.prompt_id,
                numero = v.numero,
                champ = ?refus.champ,
                motif = ?refus.motif,
                "prompt MCP non servi : contenu invalide"
            );
            None
        }
    }
}

fn interne(e: kesh_db::errors::DbError) -> ErreurPrompts {
    tracing::error!("prompts MCP : lecture en échec : {e}");
    ErreurPrompts::Interne
}

/// Les prompts servis de la société (`prompts/list`, AC-ii5) : en vigueur,
/// approuvés, de nature texte, **valides**, triés par nom (octet pour octet,
/// C-32-10a-91). `Err(HorsPortee)` avant toute lecture si la clé n'a pas
/// `books:read`.
///
/// **Deux politiques, distinctes (C-32-10a-96)** : une version dont le contenu
/// est présent mais refusé (caractère, borne, nom, agent sans clé) est
/// **omise seule**, journalisée ; une ligne qui viole le schéma — version de
/// texte sans contenu, valeur d'énumération indécodable, que le dépôt rend en
/// erreur — fait échouer **toute la liste** (`Err(Interne)`, fail closed) : une
/// base corrompue ne se masque pas derrière une liste plus courte.
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &LecturePrompts,
) -> Result<Vec<PromptServi>, ErreurPrompts> {
    acces
        .exiger(EXIGENCE_PROMPTS)
        .map_err(|_| ErreurPrompts::HorsPortee)?;
    let versions = lecture
        .en_vigueur_par_societe(acces.company_id())
        .await
        .map_err(interne)?;
    let mut servis: Vec<PromptServi> = versions.into_iter().filter_map(servir).collect();
    servis.sort_by(|a, b| a.nom.cmp(&b.nom));
    Ok(servis)
}

/// Le prompt `nom` de la société, s'il est servi (`prompts/get`, AC-ii6). Un
/// nom non conforme (majuscule, blanc final…) rend `Ok(None)` **sans lire la
/// base** : la collation de la table ne peut apparier `KESH-ASSISTANT` au
/// standard (validation P3, R13). `Err(HorsPortee)` avant toute lecture.
pub async fn obtenir(
    acces: &ContexteAcces,
    lecture: &LecturePrompts,
    nom: &str,
) -> Result<Option<PromptServi>, ErreurPrompts> {
    acces
        .exiger(EXIGENCE_PROMPTS)
        .map_err(|_| ErreurPrompts::HorsPortee)?;
    if !nom_conforme(nom) {
        return Ok(None);
    }
    Ok(lecture
        .en_vigueur_par_nom(acces.company_id(), nom)
        .await
        .map_err(interne)?
        .and_then(servir))
}

// ---------------------------------------------------------------------------
// La proposition d'un prompt par un agent — Story 32-10a-iii (AC-iii1 à AC-iii3)
// ---------------------------------------------------------------------------

/// Le nom de l'outil de proposition (AC-iii2) — un texte du binaire, nom
/// réservé hors du registre, comme `kesh_lire_envoi`. `mcp::textes` le
/// réexporte.
pub const OUTIL_PROPOSER_PROMPT: &str = "proposer_prompt";

/// Ce que proposer un prompt exige (C-32-10a-7) : la portée `prompts:write`
/// d'une clé MCP ; sur le canal API, l'Administrateur (aucune route ne sert ce
/// service : l'administrateur écrit par les routes de gestion).
pub const EXIGENCE_PROPOSER: Exigence = Exigence {
    role_api: Role::Admin,
    portee_mcp: Some(Portee::PromptsWrite),
};

/// Les arguments de `proposer_prompt` — désérialisés **strictement** : tout
/// autre champ est refusé. Ni `Debug` ni `Serialize` (ils portent le texte).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgsProposer {
    nom: String,
    description: String,
    texte: String,
}

/// L'accusé d'une proposition écrite : le **nom validé** seul — ni numéro, ni
/// texte, ni description (C-32-10a-17). Champ privé, construit en ce fichier
/// seulement.
pub struct AccuseServi {
    nom: String,
}

impl AccuseServi {
    /// Le nom validé (`[a-z0-9_-]{1,64}`, hors du préfixe `kesh`).
    pub fn nom(&self) -> &str {
        &self.nom
    }
}

/// Le refus d'une proposition — **énuméré**, sans champ libre (AC-iii2,
/// point 0). Aucun ne porte le texte reçu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusProposition {
    /// La clé n'a pas la portée `prompts:write`.
    HorsPortee,
    /// Un argument est refusé ; `champ` est un nom du binaire (`nom`,
    /// `description`, `texte` ou `(arguments)`).
    ArgumentsInvalides {
        /// L'argument en cause.
        champ: &'static str,
    },
    /// Le nom commence par `kesh` — réservé aux prompts standard, existants ou
    /// non (C-32-10a-27).
    NomReserve,
    /// La société a déjà [`PLAFOND_A_VALIDER`] propositions à valider.
    PlafondAtteint,
    /// Erreur de la base ; le détail est au journal du serveur, la
    /// transaction est annulée.
    Interne,
}

/// Le champ d'un refus de contenu, nommé comme l'argument de l'outil.
fn champ_de(c: ChampPrompt) -> &'static str {
    match c {
        ChampPrompt::Description => "description",
        ChampPrompt::Texte => "texte",
    }
}

/// La proposition d'un prompt par l'agent de la clé `cle` (AC-iii3) : une
/// version **à valider**, jamais servie avant l'approbation d'un
/// administrateur.
///
/// Dans l'ordre : (1) `exiger(EXIGENCE_PROPOSER)` — la portée `prompts:write`,
/// avant tout le reste ; (2) les arguments, désérialisés strictement ; (3)
/// [`valider_contenu`] — c'est le contenu **normalisé** qui est enregistré
/// (revue de code P1 de la 32-10a-ii, E6), jamais le texte reçu ; (4) le nom,
/// conforme et hors du préfixe `kesh` — ces refus ne lisent pas la base et
/// précèdent tout verrou ; (5) la version et l'entrée d'audit de l'appel
/// (`mcp.tool_called`, issue `servi`, [`entree_appel`]) **dans une seule
/// transaction** ([`EcriturePrompts::proposer`]) : si l'audit échoue, aucune
/// version n'est enregistrée.
///
/// La politique de validation de la clé ne s'applique pas (C-32-10a-26) : une
/// proposition n'est pas une donnée lue. `debut` est l'instant où le
/// répartiteur a reçu l'appel (la durée journalisée).
pub async fn proposer(
    cle: &CleMcpAuthentifiee,
    ecriture: &EcriturePrompts,
    debut: Instant,
    arguments: serde_json::Value,
) -> Result<AccuseServi, RefusProposition> {
    cle.acces
        .exiger(EXIGENCE_PROPOSER)
        .map_err(|_| RefusProposition::HorsPortee)?;
    let args: ArgsProposer =
        serde_json::from_value(arguments).map_err(|_| RefusProposition::ArgumentsInvalides {
            champ: "(arguments)",
        })?;
    let contenu = valider_contenu(&args.description, &args.texte).map_err(|e| {
        RefusProposition::ArgumentsInvalides {
            champ: champ_de(e.champ),
        }
    })?;
    if !nom_conforme(&args.nom) {
        return Err(RefusProposition::ArgumentsInvalides { champ: "nom" });
    }
    if nom_reserve(&args.nom) {
        return Err(RefusProposition::NomReserve);
    }
    let resultat = ecriture
        .proposer(
            cle.acces.company_id(),
            cle.cle_id,
            &args.nom,
            contenu.description(),
            contenu.texte(),
            PLAFOND_A_VALIDER,
            |_| {
                let duree_ms = u64::try_from(debut.elapsed().as_millis()).unwrap_or(u64::MAX);
                entree_appel(
                    cle,
                    OUTIL_PROPOSER_PROMPT,
                    IssueAppel::Servi,
                    duree_ms,
                    None,
                )
            },
        )
        .await;
    match resultat {
        Ok(ecrite) => Ok(AccuseServi { nom: ecrite.nom }),
        Err(ErreurEcriturePrompt::PlafondAtteint) => Err(RefusProposition::PlafondAtteint),
        Err(ErreurEcriturePrompt::NomReserve) => Err(RefusProposition::NomReserve),
        Err(ErreurEcriturePrompt::NomInvalide) => {
            Err(RefusProposition::ArgumentsInvalides { champ: "nom" })
        }
        Err(ErreurEcriturePrompt::ContenuInvalide) => {
            Err(RefusProposition::ArgumentsInvalides { champ: "texte" })
        }
        Err(e) => {
            // Le message d'une erreur du dépôt ne porte ni la description ni
            // le texte (énumération sans chaîne libre ; une erreur de base
            // nomme une contrainte, non une valeur).
            tracing::error!(
                mcp_key_id = cle.cle_id,
                "proposer_prompt : proposition non écrite, transaction annulée : {e}"
            );
            Err(RefusProposition::Interne)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::envois_mcp::rendre_visible;
    use kesh_db::entities::mcp_prompt_standards::STANDARDS;

    fn refus(champ: ChampPrompt, motif: MotifPrompt) -> Result<(), ErreurContenu> {
        Err(ErreurContenu { champ, motif })
    }

    fn verdict(d: &str, t: &str) -> Result<(), ErreurContenu> {
        valider_contenu(d, t).map(|_| ())
    }

    #[test]
    fn standards_valides() {
        assert_eq!(STANDARDS.len(), 2);
        for s in STANDARDS {
            let c = valider_contenu(s.description, s.texte)
                .unwrap_or_else(|e| panic!("{} : {e:?}", s.nom));
            assert_eq!(c.description(), s.description, "{}", s.nom);
            assert_eq!(c.texte(), s.texte, "{}", s.nom);
        }
    }

    #[test]
    fn vide_et_bornes() {
        assert_eq!(
            verdict(" \t", "t"),
            refus(ChampPrompt::Description, MotifPrompt::Vide)
        );
        assert_eq!(
            verdict("d", "\n \n"),
            refus(ChampPrompt::Texte, MotifPrompt::Vide)
        );
        let d = "é".repeat(DESCRIPTION_MAX);
        assert_eq!(verdict(&d, "t"), Ok(()), "borne de la description atteinte");
        assert_eq!(
            verdict(&format!("{d}é"), "t"),
            refus(ChampPrompt::Description, MotifPrompt::TropLong)
        );
        let t = "é".repeat(TEXTE_MAX);
        assert_eq!(verdict("d", &t), Ok(()), "borne du texte atteinte");
        assert_eq!(
            verdict("d", &format!("{t}é")),
            refus(ChampPrompt::Texte, MotifPrompt::TropLong)
        );
    }

    /// La description est une ligne : `\n` et `\t` refusés ; le texte les
    /// admet, refuse `\r` isolé, U+0000, les bidirectionnels, les invisibles
    /// et les sélecteurs de variante ; l'espace insécable passe.
    #[test]
    fn caracteres_par_champ() {
        for c in ['\n', '\t', '\u{0}', '\u{202E}', '\u{200B}', '\u{E0100}'] {
            assert_eq!(
                verdict(&format!("a{c}b"), "t"),
                refus(ChampPrompt::Description, MotifPrompt::CaractereRefuse),
                "description U+{:04X}",
                u32::from(c)
            );
        }
        for c in ['\n', '\t', '\u{A0}'] {
            assert_eq!(
                verdict("d", &format!("a{c}b")),
                Ok(()),
                "texte U+{:04X}",
                u32::from(c)
            );
        }
        assert_eq!(
            verdict("a\u{A0}b", "t"),
            Ok(()),
            "insécable dans la description"
        );
        for c in [
            '\r',
            '\u{0}',
            '\u{202E}',
            '\u{2066}',
            '\u{200B}',
            '\u{FEFF}',
            '\u{E0100}',
            '\u{200D}',
            '\u{FE0F}',
            '\u{FE0E}',
            '\u{E0041}',
            '\u{0600}',
            '\u{E000}',
            '\u{0378}',
            // Un cas par famille de l'AC-ii1 (revue de code P1, A3) : le
            // remplissage hangûl (`Lo`) et le séparateur de ligne (`Zl`).
            '\u{3164}',
            '\u{2028}',
        ] {
            assert_eq!(
                verdict("d", &format!("a{c}b")),
                refus(ChampPrompt::Texte, MotifPrompt::CaractereRefuse),
                "texte U+{:04X}",
                u32::from(c)
            );
        }
    }

    /// Emojis : simples admis, composés refusés (AC-ii1, H5).
    #[test]
    fn emojis() {
        for simple in ["\u{26A0}", "\u{1F4BB}"] {
            assert_eq!(verdict(simple, simple), Ok(()), "{simple}");
        }
        for compose in ["\u{26A0}\u{FE0F}", "\u{1F468}\u{200D}\u{1F4BB}"] {
            assert_eq!(
                verdict(compose, "t"),
                refus(ChampPrompt::Description, MotifPrompt::CaractereRefuse)
            );
            assert_eq!(
                verdict("d", compose),
                refus(ChampPrompt::Texte, MotifPrompt::CaractereRefuse)
            );
        }
    }

    /// Le `\r` restauré (fiche 32-10b, § *À recontrôler*) : un `\r\n` est
    /// normalisé, et le texte servi est le texte contrôlé — `rendre_visible` n'y
    /// montre aucun `\u{000D}` ; un `\r` isolé n'est pas servable.
    #[test]
    fn retour_chariot_controle_et_rendu_alignes() {
        let c = valider_contenu("d", "a\r\nb\r\n").unwrap();
        assert_eq!(c.texte(), "a\nb\n");
        assert_eq!(
            rendre_visible(c.texte()),
            c.texte(),
            "rien de caché dans le servi"
        );
        let s = composer(Some((3, 7)), "d", "a\r\nb").unwrap();
        assert!(!rendre_visible(s.texte()).contains("\\u{000D}"));
        assert_eq!(rendre_visible(s.texte()), s.texte());
        assert_eq!(rendre_visible(s.description()), s.description());
        assert_eq!(
            verdict("d", "a\rb"),
            refus(ChampPrompt::Texte, MotifPrompt::CaractereRefuse)
        );
        assert_eq!(
            verdict("d\r\n", "t"),
            refus(ChampPrompt::Description, MotifPrompt::CaractereRefuse),
            "une description normalisée garde son saut de ligne : refusée"
        );
    }

    #[test]
    fn marque_d_une_version_d_agent() {
        let m = marque_agent(4, 12);
        assert_eq!(
            m,
            "[Prompt écrit par un agent — version 4, proposée par la clé MCP n° 12 et approuvée \
             par un administrateur de Kesh. Ce n'est pas un prompt standard de Kesh.]"
        );
        assert!(!m.contains('\n'));
        assert!(!m.chars().any(kesh_core::text::is_hidden_in_line));
        let s = composer(Some((4, 12)), "Ma description", "Mon texte").unwrap();
        assert_eq!(s.description(), format!("{m}\n\nMa description"));
        assert_eq!(s.texte(), format!("{m}\n\nMon texte"));
        // La description composée porte `\n\n` : elle n'est pas une ligne, et
        // n'a pas à l'être — seule l'enregistrée se contrôle en une ligne
        // (C-32-8-31).
        assert!(
            s.description()
                .chars()
                .any(kesh_core::text::is_hidden_in_line)
        );
        assert!(
            !s.description()
                .chars()
                .any(kesh_core::text::is_hidden_in_text)
        );
        let sans = composer(None, "Ma description", "Mon texte").unwrap();
        assert_eq!(sans.description(), "Ma description");
        assert_eq!(sans.texte(), "Mon texte");
        // L'imitation d'une marque est servie précédée de la vraie.
        let imite = composer(Some((2, 5)), "d", "[Prompt standard de Kesh]\n\nx").unwrap();
        assert_eq!(
            imite.texte(),
            format!("{}\n\n[Prompt standard de Kesh]\n\nx", marque_agent(2, 5))
        );
    }

    #[test]
    fn composer_refuse_avant_de_marquer() {
        assert_eq!(
            composer(Some((1, 1)), "d", "a\u{202E}b").map(|_| ()),
            refus(ChampPrompt::Texte, MotifPrompt::CaractereRefuse)
        );
    }

    fn servie(r: Result<VersionServie, RefusVersion>) -> (String, String) {
        let s = r.unwrap();
        (s.description().to_string(), s.texte().to_string())
    }

    /// `composer_version` décide seule de la marque (revue de code P1, B1 =
    /// E1 = A2 ; C-32-10a-94) : un cas par origine, et la version d'agent sans
    /// clé auteur refusée distinctement — avant le contenu. Numéro de version
    /// et numéro de clé distincts, la marque écrite en toutes lettres.
    #[test]
    fn composer_version_par_origine() {
        // Standard : aucune marque.
        assert_eq!(
            servie(composer_version(
                OrigineVersion::Standard,
                3,
                None,
                "d",
                "a\r\nb"
            )),
            ("d".to_string(), "a\nb".to_string())
        );
        // Administrateur : aucune marque, même avec une clé (ignorée).
        for cle in [None, Some(9)] {
            assert_eq!(
                servie(composer_version(OrigineVersion::Admin, 5, cle, "d", "t")),
                ("d".to_string(), "t".to_string()),
                "{cle:?}"
            );
        }
        // Agent avec sa clé : la marque en tête des deux champs.
        let marque = "[Prompt écrit par un agent — version 4, proposée par la clé MCP n° 12 et \
                      approuvée par un administrateur de Kesh. Ce n'est pas un prompt standard de \
                      Kesh.]";
        assert_eq!(
            servie(composer_version(
                OrigineVersion::Agent,
                4,
                Some(12),
                "d",
                "t"
            )),
            (format!("{marque}\n\nd"), format!("{marque}\n\nt"))
        );
        // Agent sans clé : refusé, et avant le contenu.
        assert_eq!(
            composer_version(OrigineVersion::Agent, 4, None, "d", "t").map(|_| ()),
            Err(RefusVersion::AgentSansCle)
        );
        assert_eq!(
            composer_version(OrigineVersion::Agent, 4, None, "d", "a\u{202E}b").map(|_| ()),
            Err(RefusVersion::AgentSansCle)
        );
        // Un contenu refusé, pour chaque origine.
        for (origine, cle) in [
            (OrigineVersion::Standard, None),
            (OrigineVersion::Admin, None),
            (OrigineVersion::Agent, Some(12)),
        ] {
            assert_eq!(
                composer_version(origine, 4, cle, "d", "a\u{202E}b").map(|_| ()),
                Err(RefusVersion::Contenu(ErreurContenu {
                    champ: ChampPrompt::Texte,
                    motif: MotifPrompt::CaractereRefuse,
                })),
                "{origine:?}"
            );
        }
    }

    #[test]
    fn origine_de_la_base_convertie() {
        assert_eq!(
            OrigineVersion::from(Origine::Standard),
            OrigineVersion::Standard
        );
        assert_eq!(OrigineVersion::from(Origine::Admin), OrigineVersion::Admin);
        assert_eq!(OrigineVersion::from(Origine::Agent), OrigineVersion::Agent);
    }

    fn version(nom: &str, origine: Origine, cle_auteur: Option<i64>) -> VersionEnVigueur {
        VersionEnVigueur {
            prompt_id: 7,
            nom: nom.to_string(),
            numero: 3,
            origine,
            description: "d".to_string(),
            texte: "t".to_string(),
            cle_auteur,
        }
    }

    /// `servir` : le nom lu en base est revalidé (C-32-10a-95) ; une version
    /// d'agent sans clé auteur n'est pas servie (C-32-10a-90) ; témoins servis.
    #[test]
    fn servir_revalide_le_nom_et_la_cle() {
        let p = servir(version("admin-a", Origine::Admin, None)).unwrap();
        assert_eq!((p.nom(), p.texte()), ("admin-a", "t"));
        let p = servir(version("agent-ok", Origine::Agent, Some(11))).unwrap();
        assert_eq!(
            p.texte(),
            format!("{}\n\nt", marque_agent(3, 11)),
            "témoin : une version d'agent avec sa clé est servie marquée"
        );
        let trop_long = "a".repeat(65);
        for nom in [
            "Admin-a",
            "admin-a ",
            "",
            "admin\u{202E}a",
            trop_long.as_str(),
        ] {
            assert!(
                servir(version(nom, Origine::Admin, None)).is_none(),
                "{nom:?}"
            );
        }
        assert!(servir(version("agent-ok", Origine::Agent, None)).is_none());
    }
}
