//! Envois du serveur MCP en attente de validation (`mcp_sends`, Story 32-8a,
//! refs #723).
//!
//! Une clé MCP dont la politique est `required` ne transmet rien à son agent
//! sans l'accord d'un administrateur : le texte qui partirait est **déposé**,
//! puis validé (tel quel ou modifié) ou refusé ; l'agent relit l'envoi validé
//! par son numéro. Ce module porte les types d'entrée et de sortie du dépôt
//! ([`crate::repositories::mcp_sends`]) ; aucun ne sort vers le serveur MCP
//! (`kesh-api/src/mcp/`), ce sont les services qui les traduisent.
//!
//! **L'état « échu » n'est jamais écrit** : il se dérive à la lecture, à
//! l'horloge de MariaDB, par l'expression unique [`ETAT_DERIVE_SQL`].
//!
//! **Aucun `Debug` dérivé sur un type qui porte le contenu ou le résumé**
//! (C-32-8-40) : leur `Debug` est écrit à la main et ne montre que la taille.
//! Un `tracing::debug!(?envoi)` ou un `{:?}` dans un message d'erreur ne peut
//! ainsi pas recopier dans les journaux ce qui a été lu dans les livres.

use std::fmt;

use chrono::NaiveDateTime;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::mysql::{MySql, MySqlTypeInfo};
use sqlx::{Decode, Encode, Type};

use crate::errors::DbError;

/// L'état dérivé d'un envoi, en SQL : `'expired'` pour un envoi en attente ou
/// validé dont l'échéance est passée **à l'horloge de la base**, l'état stocké
/// sinon. **Seule** définition de l'échéance : toutes les lectures du dépôt et
/// la purge l'emploient, la table étant toujours désignée par l'alias `s`.
pub const ETAT_DERIVE_SQL: &str = "CASE WHEN s.state IN ('pending', 'approved') \
     AND s.expires_at <= NOW(3) THEN 'expired' ELSE s.state END";

/// L'état d'un envoi, tel que la lecture le dérive ([`ETAT_DERIVE_SQL`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatEnvoi {
    /// `pending`, échéance à venir : l'écran peut le valider ou le refuser.
    EnAttente,
    /// `approved`, échéance à venir : l'agent peut le relire.
    Valide,
    /// `refused` : rien ne part, le contenu est effacé.
    Refuse,
    /// `pending` ou `approved` dont l'échéance est passée — jamais écrit.
    Echu,
}

impl EtatEnvoi {
    /// La chaîne que rend [`ETAT_DERIVE_SQL`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EnAttente => "pending",
            Self::Valide => "approved",
            Self::Refuse => "refused",
            Self::Echu => "expired",
        }
    }
}

impl std::str::FromStr for EtatEnvoi {
    type Err = String;
    /// Décodage **strict** : la casse et les blancs comptent.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::EnAttente),
            "approved" => Ok(Self::Valide),
            "refused" => Ok(Self::Refuse),
            "expired" => Ok(Self::Echu),
            other => Err(format!("état d'envoi MCP inconnu : {other:?}")),
        }
    }
}

impl Type<MySql> for EtatEnvoi {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for EtatEnvoi {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for EtatEnvoi {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Décrit un texte sans le montrer : sa taille en octets, ou son absence.
struct Masque<'a>(&'a Option<String>);

impl fmt::Debug for Masque<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(s) => write!(f, "<{} octets masqués>", s.len()),
            None => f.write_str("None"),
        }
    }
}

/// Un envoi à déposer ([`crate::repositories::mcp_sends::deposer_in_tx`]).
/// `content` est le texte **déjà rendu visible** par l'appelant.
#[derive(Clone)]
pub struct NouvelEnvoi {
    pub company_id: i64,
    pub mcp_key_id: i64,
    pub tool_name: String,
    pub summary: Option<String>,
    pub content: String,
}

impl fmt::Debug for NouvelEnvoi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NouvelEnvoi")
            .field("company_id", &self.company_id)
            .field("mcp_key_id", &self.mcp_key_id)
            .field("tool_name", &self.tool_name)
            .field("summary", &Masque(&self.summary))
            .field("content", &Masque(&Some(self.content.clone())))
            .finish()
    }
}

/// La décision d'un administrateur sur un envoi en attente.
#[derive(Clone, PartialEq, Eq)]
pub enum Decision {
    /// Valider : tel quel (`contenu: None`), ou avec le texte reçu, stocké
    /// octet pour octet ; `modified` dit s'il diffère du texte déposé.
    Valider { contenu: Option<String> },
    /// Refuser : contenu et résumé effacés.
    Refuser,
}

impl fmt::Debug for Decision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Valider { contenu } => f
                .debug_struct("Valider")
                .field("contenu", &Masque(contenu))
                .finish(),
            Self::Refuser => f.write_str("Refuser"),
        }
    }
}

/// Le refus d'un dépôt.
#[derive(Debug, thiserror::Error)]
pub enum ErreurDepot {
    /// La clé a déjà atteint le plafond d'envois en attente vivants : rien
    /// n'est écrit.
    #[error("trop d'envois MCP en attente pour cette clé")]
    TropNombreux,
    #[error(transparent)]
    Db(#[from] DbError),
}

/// Le refus d'une décision. L'ordre des contrôles est celui des variantes.
#[derive(Debug, thiserror::Error)]
pub enum ErreurDecision {
    /// Aucun envoi de ce numéro **dans la société**.
    #[error("envoi MCP introuvable")]
    Introuvable,
    /// L'envoi n'est plus en attente (validé ou refusé).
    #[error("envoi MCP déjà décidé")]
    DejaDecidee,
    /// L'échéance est passée, à l'horloge de la base.
    #[error("envoi MCP échu")]
    Echue,
    /// La version reçue n'est pas la version stockée.
    #[error("envoi MCP modifié entre-temps (version périmée)")]
    VersionPerimee,
    #[error(transparent)]
    Db(#[from] DbError),
}

/// La ligne décidée, telle que la décision l'a écrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvoiDecide {
    pub id: i64,
    pub mcp_key_id: i64,
    pub tool_name: String,
    /// [`EtatEnvoi::Valide`] ou [`EtatEnvoi::Refuse`].
    pub etat: EtatEnvoi,
    pub modified: bool,
    /// Version après la décision.
    pub version: i32,
}

/// Un envoi relu par la clé qui l'a déposé
/// ([`crate::repositories::mcp_sends::lire_pour_cle`]). `contenu` n'est
/// présent **que** si l'état est [`EtatEnvoi::Valide`].
#[derive(Clone, PartialEq, Eq)]
pub struct EnvoiPourCle {
    pub id: i64,
    pub etat: EtatEnvoi,
    pub modified: bool,
    pub contenu: Option<String>,
}

impl fmt::Debug for EnvoiPourCle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnvoiPourCle")
            .field("id", &self.id)
            .field("etat", &self.etat)
            .field("modified", &self.modified)
            .field("contenu", &Masque(&self.contenu))
            .finish()
    }
}

/// Une ligne de la liste des envois d'une société — **ni contenu ni résumé**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvoiResume {
    pub id: i64,
    pub mcp_key_id: i64,
    pub mcp_key_name: String,
    pub mcp_key_revoked: bool,
    pub tool_name: String,
    pub etat: EtatEnvoi,
    pub modified: bool,
    pub expires_at: NaiveDateTime,
    pub decided_at: Option<NaiveDateTime>,
    /// Nom de l'utilisateur qui a décidé, s'il y a eu décision.
    pub decided_by: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Le détail d'un envoi pour l'écran : la ligne de liste, plus le résumé, le
/// contenu (tous deux `None` si l'envoi est refusé ou échu), la taille du
/// contenu en octets et la version.
#[derive(Clone, PartialEq, Eq)]
pub struct EnvoiDetail {
    pub resume: EnvoiResume,
    pub summary: Option<String>,
    pub content: Option<String>,
    /// Taille de `content` en octets UTF-8, `None` si `content` l'est.
    pub content_bytes: Option<u64>,
    pub version: i32,
}

impl fmt::Debug for EnvoiDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnvoiDetail")
            .field("resume", &self.resume)
            .field("summary", &Masque(&self.summary))
            .field("content", &Masque(&self.content))
            .field("content_bytes", &self.content_bytes)
            .field("version", &self.version)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etat_se_decode_strictement() {
        for e in [
            EtatEnvoi::EnAttente,
            EtatEnvoi::Valide,
            EtatEnvoi::Refuse,
            EtatEnvoi::Echu,
        ] {
            assert_eq!(e.as_str().parse::<EtatEnvoi>(), Ok(e));
        }
        for faux in ["Pending", "pending ", " approved", "EXPIRED", "", "echu"] {
            assert!(faux.parse::<EtatEnvoi>().is_err(), "{faux:?} accepté");
        }
    }

    #[test]
    fn debug_ne_montre_ni_contenu_ni_resume() {
        let secret = "PIEGE_DEBUG_328A";
        let sorties = [
            format!(
                "{:?}",
                NouvelEnvoi {
                    company_id: 1,
                    mcp_key_id: 2,
                    tool_name: "outil".into(),
                    summary: Some(secret.into()),
                    content: secret.into(),
                }
            ),
            format!(
                "{:?}",
                Decision::Valider {
                    contenu: Some(secret.into())
                }
            ),
            format!(
                "{:?}",
                EnvoiPourCle {
                    id: 1,
                    etat: EtatEnvoi::Valide,
                    modified: false,
                    contenu: Some(secret.into()),
                }
            ),
            format!(
                "{:?}",
                EnvoiDetail {
                    resume: EnvoiResume {
                        id: 1,
                        mcp_key_id: 2,
                        mcp_key_name: "cle".into(),
                        mcp_key_revoked: false,
                        tool_name: "outil".into(),
                        etat: EtatEnvoi::EnAttente,
                        modified: false,
                        expires_at: NaiveDateTime::default(),
                        decided_at: None,
                        decided_by: None,
                        created_at: NaiveDateTime::default(),
                    },
                    summary: Some(secret.into()),
                    content: Some(secret.into()),
                    content_bytes: Some(16),
                    version: 1,
                }
            ),
        ];
        for s in &sorties {
            assert!(!s.contains(secret), "{s}");
            assert!(s.contains("16 octets masqués"), "témoin : {s}");
        }
    }
}
