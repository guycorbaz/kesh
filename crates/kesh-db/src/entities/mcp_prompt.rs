//! Prompts du serveur MCP et leur historique (`mcp_prompts`,
//! `mcp_prompt_versions`, Story 32-10a-i, refs #723).
//!
//! Un **prompt** est un nom, propre à une société ; ses **versions** forment un
//! historique qui ne se réécrit ni ne s'efface. Chaque version a une
//! [`Origine`] (le code, un administrateur, un agent), un [`Etat`] (seule une
//! version d'agent passe par « à valider ») et une [`Nature`] (un texte, ou un
//! retrait). La **version en vigueur** est la version approuvée de plus grand
//! numéro ([`en_vigueur`]) ; un prompt dont la version en vigueur est un retrait,
//! ou qui n'en a aucune, n'est pas servi.
//!
//! Ce module porte aussi **les types que rend le dépôt**
//! ([`crate::repositories::mcp_prompts`]) — [`VersionEnVigueur`],
//! [`VersionEcrite`], [`SynthesePrompt`], [`VersionHistorique`],
//! [`ErreurEcriturePrompt`] — si bien que les services les nomment par
//! `kesh_db::entities::mcp_prompt::…` sans jamais nommer le dépôt (AC-i4), et
//! les règles pures que partagent le dépôt, l'amorçage et les routes de la
//! 32-10a-iii : [`nom_conforme`], [`nom_reserve`], [`en_vigueur`],
//! [`differe_du_standard`].
//!
//! **Aucun `Debug` dérivé sur un type qui porte un texte** : leur `Debug` ne
//! montre que la taille (patron de `entities::mcp_send`, C-32-8-40). Aucune
//! entité ne dérive `Serialize` : les routes construisent leurs DTO.

use std::fmt;

use chrono::NaiveDateTime;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::mysql::{MySql, MySqlTypeInfo};
use sqlx::{Decode, Encode, Type};

use super::mcp_prompt_standards::Standard;
use super::mcp_send::Masque;
use crate::errors::DbError;

/// Longueur maximale du nom d'un prompt, en caractères (`VARCHAR(64)`).
pub const NOM_MAX: usize = 64;
/// Longueur maximale de la description d'une version, en caractères
/// (`VARCHAR(500)`).
pub const DESCRIPTION_MAX: usize = 500;
/// Longueur maximale du texte d'une version, en caractères (C-32-10a-13 ;
/// `CHECK` de la migration).
pub const TEXTE_MAX: usize = 16_000;
/// Le préfixe réservé aux prompts standard (C-32-10a-7). Un prompt **non**
/// standard n'a aucun nom commençant par `kesh` ([`nom_reserve`]).
pub const PREFIXE_STANDARD: &str = "kesh-";
/// Plafond de production des versions **à valider** par société
/// (C-32-10a-13) : au-delà, une proposition d'agent est refusée.
pub const PLAFOND_A_VALIDER: u32 = 20;

/// Déclare une énumération stockée en `VARCHAR`, à décodage **strict** (une
/// valeur inconnue, une casse ou un blanc différents sont une erreur ; patron
/// de `SendValidation`, 32-5a).
macro_rules! enum_texte {
    ($(#[$m:meta])* $nom:ident { $($(#[$vm:meta])* $var:ident => $s:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $nom { $($(#[$vm])* $var),+ }

        impl $nom {
            /// La valeur stockée.
            pub fn as_str(&self) -> &'static str {
                match self { $(Self::$var => $s),+ }
            }
        }

        impl std::str::FromStr for $nom {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($s => Ok(Self::$var),)+
                    other => Err(format!(concat!(stringify!($nom), " inconnu : {:?}"), other)),
                }
            }
        }

        impl Type<MySql> for $nom {
            fn type_info() -> MySqlTypeInfo {
                <String as Type<MySql>>::type_info()
            }
            fn compatible(ty: &MySqlTypeInfo) -> bool {
                <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
            }
        }

        impl<'q> Encode<'q, MySql> for $nom {
            fn encode_by_ref(
                &self,
                buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
            ) -> Result<IsNull, BoxDynError> {
                <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
            }
        }

        impl<'r> Decode<'r, MySql> for $nom {
            fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
                let s = <String as Decode<MySql>>::decode(value)?;
                s.parse().map_err(Into::into)
            }
        }
    };
}

enum_texte! {
    /// D'où vient une version.
    Origine {
        /// Le texte du code (`STANDARDS`), amorcé par Kesh.
        Standard => "standard",
        /// Écrite par un administrateur.
        Admin => "admin",
        /// Proposée par une clé MCP de portée `prompts:write`.
        Agent => "agent",
    }
}

enum_texte! {
    /// L'état d'une version. Seule une version d'agent est d'abord
    /// [`Etat::EnAttente`] ; les états décidés sont terminaux.
    Etat {
        Approuvee => "approved",
        EnAttente => "pending",
        Refusee => "refused",
    }
}

enum_texte! {
    /// Ce que porte une version : un texte, ou le retrait du prompt.
    Nature {
        Texte => "text",
        Retrait => "withdrawal",
    }
}

/// Le contenu d'une version.
#[derive(Clone, PartialEq, Eq)]
pub enum Contenu {
    /// Une description (une ligne, montrée par `prompts/list`) et un texte.
    Texte { description: String, texte: String },
    /// Le prompt cesse d'être servi.
    Retrait,
}

impl Contenu {
    /// La nature de ce contenu.
    pub fn nature(&self) -> Nature {
        match self {
            Self::Texte { .. } => Nature::Texte,
            Self::Retrait => Nature::Retrait,
        }
    }

    /// Le contenu d'un standard du code.
    pub fn du_standard(s: &Standard) -> Self {
        Self::Texte {
            description: s.description.to_string(),
            texte: s.texte.to_string(),
        }
    }

    /// Reconstitue un contenu depuis les colonnes d'une version. Une nature
    /// `text` sans description ou sans texte viole le `CHECK` de la table :
    /// erreur, jamais un texte vide.
    pub fn depuis_colonnes(
        nature: Nature,
        description: Option<String>,
        texte: Option<String>,
    ) -> Result<Self, DbError> {
        match (nature, description, texte) {
            (Nature::Texte, Some(description), Some(texte)) => {
                Ok(Self::Texte { description, texte })
            }
            (Nature::Retrait, None, None) => Ok(Self::Retrait),
            (nature, _, _) => Err(DbError::Invariant(format!(
                "version de prompt de nature {} aux colonnes incohérentes",
                nature.as_str()
            ))),
        }
    }

    /// Les deux colonnes `(description, body)` à écrire.
    pub fn colonnes(&self) -> (Option<&str>, Option<&str>) {
        match self {
            Self::Texte { description, texte } => (Some(description), Some(texte)),
            Self::Retrait => (None, None),
        }
    }
}

impl fmt::Debug for Contenu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Texte { description, texte } => f
                .debug_struct("Texte")
                .field("description", &Masque(&Some(description.clone())))
                .field("texte", &Masque(&Some(texte.clone())))
                .finish(),
            Self::Retrait => f.write_str("Retrait"),
        }
    }
}

/// Une ligne de `mcp_prompts`.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct McpPrompt {
    pub id: i64,
    pub company_id: i64,
    pub name: String,
    pub is_standard: bool,
    /// Verrou optimiste : incrémenté par **toute** écriture d'une version.
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Une ligne de `mcp_prompt_versions`.
#[derive(Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct McpPromptVersion {
    pub id: i64,
    pub prompt_id: i64,
    pub number: i32,
    pub origin: Origine,
    pub state: Etat,
    pub nature: Nature,
    pub description: Option<String>,
    pub body: Option<String>,
    pub author_user_id: Option<i64>,
    pub author_mcp_key_id: Option<i64>,
    pub decided_by_user_id: Option<i64>,
    pub decided_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl McpPromptVersion {
    /// Le contenu de la version ([`Contenu::depuis_colonnes`]).
    pub fn contenu(&self) -> Result<Contenu, DbError> {
        Contenu::depuis_colonnes(self.nature, self.description.clone(), self.body.clone())
    }
}

impl fmt::Debug for McpPromptVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpPromptVersion")
            .field("id", &self.id)
            .field("prompt_id", &self.prompt_id)
            .field("number", &self.number)
            .field("origin", &self.origin)
            .field("state", &self.state)
            .field("nature", &self.nature)
            .field("description", &Masque(&self.description))
            .field("body", &Masque(&self.body))
            .field("author_user_id", &self.author_user_id)
            .field("author_mcp_key_id", &self.author_mcp_key_id)
            .field("decided_by_user_id", &self.decided_by_user_id)
            .field("decided_at", &self.decided_at)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// Une version de l'historique d'un prompt, avec les noms que l'écran affiche
/// (C-32-10a-60, pour la route de détail de la 32-10a-iii, amendement D1) :
/// l'auteur et le décideur (`users.username`), la clé auteur (`mcp_keys.name`)
/// et sa révocation. Lus par une jointure, jamais par une requête par version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionHistorique {
    pub version: McpPromptVersion,
    pub auteur_nom: Option<String>,
    pub decideur_nom: Option<String>,
    pub cle_nom: Option<String>,
    /// `Some(true)` si la clé auteur est révoquée ; `None` sans clé auteur.
    pub cle_revoquee: Option<bool>,
}

/// La version en vigueur d'un prompt, quand c'est un **texte** — ce que le
/// catalogue servi (32-10a-ii) lit, et rien d'autre.
#[derive(Clone, PartialEq, Eq)]
pub struct VersionEnVigueur {
    pub prompt_id: i64,
    pub nom: String,
    pub numero: i32,
    pub origine: Origine,
    pub description: String,
    pub texte: String,
    /// La clé MCP qui l'a proposée (version d'agent seulement).
    pub cle_auteur: Option<i64>,
}

impl fmt::Debug for VersionEnVigueur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VersionEnVigueur")
            .field("prompt_id", &self.prompt_id)
            .field("nom", &self.nom)
            .field("numero", &self.numero)
            .field("origine", &self.origine)
            .field("description", &Masque(&Some(self.description.clone())))
            .field("texte", &Masque(&Some(self.texte.clone())))
            .field("cle_auteur", &self.cle_auteur)
            .finish()
    }
}

/// Ce qu'une écriture du dépôt a écrit — tout ce que l'audit et les réponses
/// des routes de la 32-10a-iii écrivent, **sans texte**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionEcrite {
    pub prompt_id: i64,
    pub nom: String,
    pub numero: i32,
    pub origine: Origine,
    pub nature: Nature,
    pub etat: Etat,
    /// La `version` du prompt **après** l'écriture.
    pub version: i32,
}

/// Le résumé d'une version dans la synthèse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumeVersion {
    pub numero: i32,
    pub origine: Origine,
    pub etat: Etat,
    pub nature: Nature,
}

/// Une ligne de la synthèse d'une société ([`crate::repositories::
/// mcp_prompts::synthese_par_societe`]), **sans texte** : les textes lus pour
/// calculer [`SynthesePrompt::standard_differe`] n'en sortent pas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesePrompt {
    pub id: i64,
    pub nom: String,
    pub is_standard: bool,
    pub version: i32,
    /// La version de plus grand numéro, quel que soit son état.
    pub derniere: ResumeVersion,
    /// La version en vigueur, texte ou retrait ; `None` si aucune n'est
    /// approuvée.
    pub en_vigueur: Option<ResumeVersion>,
    /// Le nombre de versions à valider **du prompt**.
    pub a_valider: i64,
    /// Prompt standard dont la version en vigueur n'est pas celle du code
    /// ([`differe_du_standard`]) ; toujours `false` hors standard.
    pub standard_differe: bool,
}

/// Le rapport compté d'un amorçage, **sans texte**.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RapportAmorcage {
    /// Sociétés amorcées.
    pub societes: u64,
    /// Prompts standard créés.
    pub creations: u64,
    /// Versions standard ajoutées au texte du code.
    pub mises_a_jour: u64,
    /// Versions standard de retrait (standard retiré du code).
    pub retraits: u64,
}

/// Le refus d'une écriture du dépôt — **énuméré**, sans chaîne libre.
#[derive(Debug, thiserror::Error)]
pub enum ErreurEcriturePrompt {
    /// Aucun prompt, ou aucune version, de cet identifiant **dans la société**
    /// — un prompt d'une autre société est indiscernable d'un prompt absent.
    #[error("prompt MCP introuvable")]
    NotFound,
    /// La `version` reçue n'est pas celle du prompt (verrou optimiste).
    #[error("prompt MCP modifié entre-temps (version périmée)")]
    VersionPerimee,
    /// L'approbation vise une version qui n'est plus la dernière du prompt.
    #[error("la version n'est plus la dernière du prompt")]
    PasLaDerniere,
    /// La version est déjà approuvée ou refusée : les états sont terminaux.
    #[error("version de prompt déjà décidée")]
    DejaDecidee,
    /// Une version `standard` ou `admin` ne s'approuve ni ne se refuse.
    #[error("seule une proposition d'agent se décide")]
    PasUneProposition,
    /// Nom hors du motif `[a-z0-9_-]{1,64}`.
    #[error("nom de prompt non conforme")]
    NomInvalide,
    /// Nom en `kesh` pour un prompt non standard (C-32-10a-7, C-32-10a-27).
    #[error("nom de prompt réservé")]
    NomReserve,
    /// Un prompt de ce nom existe déjà dans la société.
    #[error("nom de prompt déjà pris")]
    NomDejaPris,
    /// Description ou texte vide, ou hors des bornes.
    #[error("contenu de prompt invalide")]
    ContenuInvalide,
    /// La société a déjà `plafond` versions à valider.
    #[error("trop de propositions de prompt en attente")]
    PlafondAtteint,
    /// Le standard reçu ne porte pas le nom du prompt.
    #[error("standard inconnu du code pour ce prompt")]
    StandardInconnu,
    /// Le retour au standard vise un prompt non standard.
    #[error("le prompt n'est pas un prompt standard")]
    NonStandard,
    #[error(transparent)]
    Db(#[from] DbError),
}

/// Vrai si `nom` est conforme au motif `^[a-z0-9_-]{1,64}$` — **octet pour
/// octet** : aucune majuscule, aucun blanc, aucun saut de ligne final.
pub fn nom_conforme(nom: &str) -> bool {
    (1..=NOM_MAX).contains(&nom.len())
        && nom
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Vrai si `nom` commence par `kesh` : réservé aux standards (`kesh-…`), refusé
/// à tout prompt d'administrateur ou d'agent (`kesh-x`, `kesh_x`, `keshx`).
pub fn nom_reserve(nom: &str) -> bool {
    nom.starts_with("kesh")
}

/// Vrai si le contenu est dans les bornes de la table : description et texte
/// portant chacun un caractère qui n'est pas un blanc, au plus
/// [`DESCRIPTION_MAX`] et [`TEXTE_MAX`] caractères. Un retrait est toujours
/// valide. Les caractères refusés (invisibles, contrôles) sont l'affaire du
/// service (32-10a-ii, `valider_contenu`), qui les contrôle avant d'appeler le
/// dépôt.
pub fn contenu_dans_les_bornes(contenu: &Contenu) -> bool {
    match contenu {
        Contenu::Retrait => true,
        Contenu::Texte { description, texte } => {
            let porte = |s: &str| s.chars().any(|c| !c.is_whitespace());
            porte(description)
                && porte(texte)
                && description.chars().count() <= DESCRIPTION_MAX
                && texte.chars().count() <= TEXTE_MAX
        }
    }
}

/// La version en vigueur parmi les versions d'un prompt : la version
/// **approuvée** de plus grand numéro (C-32-10a-4). Aucun pointeur, aucune
/// horloge.
pub fn en_vigueur(versions: &[McpPromptVersion]) -> Option<&McpPromptVersion> {
    versions
        .iter()
        .filter(|v| v.state == Etat::Approuvee)
        .max_by_key(|v| v.number)
}

/// Le contenu qu'un prompt standard **devrait** servir : le texte du standard
/// de ce nom dans `standards`, ou un retrait s'il n'y est plus.
pub fn contenu_attendu(nom: &str, standards: &[Standard]) -> Contenu {
    standards
        .iter()
        .find(|s| s.nom == nom)
        .map(Contenu::du_standard)
        .unwrap_or(Contenu::Retrait)
}

/// Vrai si un prompt **standard** ne sert pas ce que le code attend : sa
/// version en vigueur n'est pas d'origine `standard`, ou son contenu diffère
/// de [`contenu_attendu`] — comparaison **en Rust, octet pour octet**
/// (validation P2, F12 : jamais sous la collation des tables, insensible à la
/// casse et aux accents). `en_vigueur` vaut `None` si aucune version n'est
/// approuvée.
pub fn differe_du_standard(
    nom: &str,
    en_vigueur: Option<(Origine, &Contenu)>,
    standards: &[Standard],
) -> bool {
    match en_vigueur {
        None => true,
        Some((origine, contenu)) => {
            origine != Origine::Standard || *contenu != contenu_attendu(nom, standards)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerations_se_decodent_strictement() {
        for o in [Origine::Standard, Origine::Admin, Origine::Agent] {
            assert_eq!(o.as_str().parse::<Origine>(), Ok(o));
        }
        for e in [Etat::Approuvee, Etat::EnAttente, Etat::Refusee] {
            assert_eq!(e.as_str().parse::<Etat>(), Ok(e));
        }
        for n in [Nature::Texte, Nature::Retrait] {
            assert_eq!(n.as_str().parse::<Nature>(), Ok(n));
        }
        for faux in ["Standard", "admin ", " agent", "AGENT", ""] {
            assert!(faux.parse::<Origine>().is_err(), "{faux:?} accepté");
        }
        for faux in ["Approved", "pending ", "REFUSED", "en_attente"] {
            assert!(faux.parse::<Etat>().is_err(), "{faux:?} accepté");
        }
        for faux in ["Text", "withdrawal ", "retrait"] {
            assert!(faux.parse::<Nature>().is_err(), "{faux:?} accepté");
        }
    }

    #[test]
    fn noms() {
        for bon in ["a", "mon-prompt", "x_1", "a".repeat(64).as_str()] {
            assert!(nom_conforme(bon), "{bon:?} refusé");
        }
        for mauvais in [
            "",
            "Valide",
            "valide\n",
            "valide ",
            " valide",
            "é",
            "a.b",
            "a".repeat(65).as_str(),
        ] {
            assert!(!nom_conforme(mauvais), "{mauvais:?} accepté");
        }
        for reserve in ["kesh", "kesh-x", "kesh_x", "keshx"] {
            assert!(nom_reserve(reserve), "{reserve:?} non réservé");
        }
        for libre in ["mon-kesh", "ke-sh", "k"] {
            assert!(!nom_reserve(libre), "{libre:?} réservé");
        }
    }

    #[test]
    fn bornes_du_contenu() {
        let t = |d: &str, x: &str| Contenu::Texte {
            description: d.into(),
            texte: x.into(),
        };
        assert!(contenu_dans_les_bornes(&t("d", "x")));
        assert!(contenu_dans_les_bornes(&Contenu::Retrait));
        assert!(contenu_dans_les_bornes(&t(
            &"é".repeat(500),
            &"é".repeat(16_000)
        )));
        assert!(!contenu_dans_les_bornes(&t(" \t\n", "x")));
        assert!(!contenu_dans_les_bornes(&t("d", "  ")));
        assert!(!contenu_dans_les_bornes(&t(&"a".repeat(501), "x")));
        assert!(!contenu_dans_les_bornes(&t("d", &"a".repeat(16_001))));
    }

    #[test]
    fn debug_ne_montre_aucun_texte() {
        let secret = "PIEGE_DEBUG_3210A";
        let contenu = Contenu::Texte {
            description: secret.into(),
            texte: secret.into(),
        };
        let version = McpPromptVersion {
            id: 1,
            prompt_id: 1,
            number: 1,
            origin: Origine::Admin,
            state: Etat::Approuvee,
            nature: Nature::Texte,
            description: Some(secret.into()),
            body: Some(secret.into()),
            author_user_id: Some(1),
            author_mcp_key_id: None,
            decided_by_user_id: None,
            decided_at: None,
            created_at: NaiveDateTime::default(),
        };
        let vigueur = VersionEnVigueur {
            prompt_id: 1,
            nom: "p".into(),
            numero: 1,
            origine: Origine::Admin,
            description: secret.into(),
            texte: secret.into(),
            cle_auteur: None,
        };
        for s in [
            format!("{contenu:?}"),
            format!("{version:?}"),
            format!("{vigueur:?}"),
        ] {
            assert!(!s.contains(secret), "{s}");
            assert!(s.contains("17 octets masqués"), "témoin : {s}");
        }
    }
}
