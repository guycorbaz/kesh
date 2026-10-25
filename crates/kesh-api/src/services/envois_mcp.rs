//! Les envois du serveur MCP à valider — Story 32-8b (refs #723).
//!
//! Sous une clé `required`, ce qu'un outil a lu **ne part pas** : la politique
//! (`mcp/politique.rs`) le remet à [`deposer`], qui l'écrit dans la table des
//! envois ; l'Administrateur le voit dans Kesh et le valide, tel quel ou
//! modifié, ou le refuse (routes `routes/mcp_envois.rs`) ; l'agent le relit
//! ensuite par l'outil hors registre `kesh_lire_envoi`, qui appelle [`lire`].
//!
//! # Ce que tient ce module
//!
//! - **Un seul texte** (C-32-8-4) : le texte déposé est celui que l'écran
//!   montre, que l'Administrateur modifie et que l'agent relit.
//! - **Rien d'invisible** (C-32-8-5) : [`deposer`] applique lui-même
//!   [`rendre_visible`] au texte et au résumé, **avant** le contrôle de taille
//!   — l'invariant ne dépend pas de l'appelant (constat P3-F7, C-32-8-51).
//! - **Un texte modifié n'est jamais corrigé** (C-32-8-19) :
//!   [`verifier_contenu_modifie`] le refuse s'il porte un caractère caché ou
//!   dépasse la taille ; il ne le rogne ni ne le normalise.
//! - **« Même clé, même société »** (C-32-8-8) : [`lire`] ne construit un
//!   [`ContenuValide`] que d'une ligne validée, vivante, de la clé **et** de la
//!   société de l'appel ; [`ContenuValide`] et [`AccuseEnvoi`] ne se
//!   construisent qu'ici (garde `envoi_valide_construit_par_le_service_seul`).
//! - **Le dépôt et l'entrée d'audit de l'appel, en une transaction**
//!   (C-32-8-12) : un dépôt réussi est journalisé dans sa transaction ; un dépôt
//!   refusé n'écrit rien, et le répartiteur le journalise à part.
//!
//! # Dérogations écrites aux règles de `services`
//!
//! [`deposer`] et [`lire`] n'appellent pas `exiger` (R2, R7) : **aucune**
//! `Exigence` n'est satisfaite par toute clé MCP quelle que soit sa portée, et
//! le contrôle d'accès est ici la règle « même clé, même société », plus
//! étroite qu'une portée. Pour [`deposer`], la portée a déjà été exigée par le
//! **service de l'outil** avant sa lecture, et le dépôt ne lit rien des livres.
//! Ce sont des **écritures** sur un chemin du MCP (R4) : elles passent par la
//! poignée [`EcritureEnvois`], non par une `Lecture` (inventaire des écritures
//! de `services/mod.rs`).
//!
//! ⛔ **Aucun évènement `tracing` ne nomme le contenu ni le résumé** (garde
//! `aucun_evenement_ne_nomme_le_contenu`), et aucun span n'est posé ici (garde
//! `aucun_span_dans_le_chemin_des_envois`).

use std::time::Instant;

use kesh_db::entities::mcp_send::{EnvoiPourCle, ErreurDepot, EtatEnvoi, NouvelEnvoi};

use crate::errors::AppError;
use crate::services::cles_mcp::CleMcpAuthentifiee;
use crate::services::poignees::{EcritureEnvois, LectureEnvois};
use crate::services::serveur_mcp::{IssueAppel, entree_appel};

/// Taille maximale du texte d'un envoi, **après** le rendu visible, en octets
/// UTF-8 : 24 Kio (note de l'orchestrateur du 2026-10-25 — une réponse MCP
/// reste sous 10 000 jetons, à 2,5 octets par jeton). Vaut aussi pour le texte
/// modifié (AC-B9).
pub const MCP_ENVOI_OCTETS_MAX: usize = 24_576;

/// Taille maximale du résumé d'un envoi, après le rendu visible, en octets.
pub const MCP_ENVOI_RESUME_OCTETS_MAX: usize = 1_024;

/// L'échéance d'un envoi, comptée depuis son dépôt à l'horloge de la base :
/// 24 h (C-32-8-3).
pub const MCP_ENVOI_DUREE_S: u32 = 86_400;

/// Le nombre maximal d'envois en attente (vivants) d'une clé (C-32-8-13).
pub const MCP_ENVOIS_EN_ATTENTE_MAX: u32 = 20;

// Point 4 de #765 : les entrées de `deposer_in_tx` sont bornées ici — une
// échéance nulle ou un plafond nul rendraient chaque envoi échu ou refusé.
const _: () = assert!(MCP_ENVOI_DUREE_S > 0 && MCP_ENVOIS_EN_ATTENTE_MAX > 0);

/// Remplace chaque caractère caché (`kesh_core::text::is_hidden_in_text`, `\r`
/// compris) par son échappement Rust `\u{XXXX}` — hexadécimal en capitales,
/// quatre chiffres au moins, sans zéro de tête au-delà —, et laisse le reste
/// intact. Idempotente : sa sortie n'a aucun caractère caché.
pub fn rendre_visible(s: &str) -> String {
    let mut sortie = String::with_capacity(s.len());
    for c in s.chars() {
        if kesh_core::text::is_hidden_in_text(c) {
            sortie.push_str(&format!("\\u{{{:04X}}}", u32::from(c)));
        } else {
            sortie.push(c);
        }
    }
    sortie
}

/// Le motif d'un refus de texte modifié.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotifContenu {
    /// Au-delà de [`MCP_ENVOI_OCTETS_MAX`] octets.
    TropVolumineux,
    /// Un caractère vérifie `kesh_core::text::is_hidden_in_text`.
    CaractereCache,
}

/// Contrôle un texte modifié par l'Administrateur : **aucun** rognage,
/// **aucune** normalisation — refusé tel quel, ou admis tel quel (C-32-8-19).
pub fn verifier_contenu_modifie(s: &str) -> Result<(), MotifContenu> {
    if s.len() > MCP_ENVOI_OCTETS_MAX {
        return Err(MotifContenu::TropVolumineux);
    }
    if s.chars().any(kesh_core::text::is_hidden_in_text) {
        return Err(MotifContenu::CaractereCache);
    }
    Ok(())
}

/// L'accusé d'un dépôt : un numéro, **aucune donnée**. Construit en ce fichier
/// seulement.
pub struct AccuseEnvoi {
    numero: i64,
}

impl AccuseEnvoi {
    /// Le numéro de l'envoi déposé.
    pub fn numero(&self) -> i64 {
        self.numero
    }
}

/// Le contenu validé d'un envoi, tel que l'agent le relit. Construit en ce
/// fichier seulement, d'une ligne validée et vivante de la clé et de la
/// société de l'appel ; ni `Clone`, ni `Debug`, ni `Default`, ni `Serialize`.
pub struct ContenuValide {
    numero: i64,
    modifie: bool,
    texte: String,
}

impl ContenuValide {
    /// Le numéro de l'envoi.
    pub fn numero(&self) -> i64 {
        self.numero
    }

    /// L'Administrateur a-t-il modifié le texte avant de le valider ?
    pub fn modifie(&self) -> bool {
        self.modifie
    }

    /// Le texte validé, octet pour octet.
    pub fn texte(&self) -> &str {
        &self.texte
    }
}

/// Ce qu'une relecture trouve.
pub enum EnvoiLu {
    /// En attente de la décision.
    EnAttente {
        /// Le numéro demandé.
        numero: i64,
    },
    /// Validé, vivant : le contenu part.
    Valide(ContenuValide),
    /// Refusé.
    Refuse {
        /// Le numéro demandé.
        numero: i64,
    },
    /// Échu (en attente ou validé, échéance passée).
    Echu {
        /// Le numéro demandé.
        numero: i64,
    },
    /// Inexistant, d'une autre clé ou d'une autre société — indiscernables.
    Inconnu {
        /// Le numéro demandé.
        numero: i64,
    },
}

/// Le refus ou l'échec d'un dépôt ; **rien n'a été écrit**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusDepot {
    /// Le texte ou le résumé, rendus visibles, dépassent leur taille.
    TropVolumineux,
    /// La clé a déjà [`MCP_ENVOIS_EN_ATTENTE_MAX`] envois en attente.
    TropNombreux,
    /// Toute autre erreur (base) ; le détail ne va qu'au journal du serveur,
    /// sans le contenu.
    Interne,
}

/// Dépose un envoi pour la clé de l'appel et écrit l'entrée d'audit de
/// l'appel (`envoi_depose`) dans **la même** transaction.
///
/// Dans l'ordre : (1) rendu visible du texte et du résumé ; (2) contrôle des
/// tailles ; (3) purge, *best-effort* et hors transaction, des contenus échus
/// de la société ; (4) dépôt et entrée d'audit, en une transaction. La société
/// et la clé du dépôt sont **celles de la clé authentifiée** (point 4 de #765) :
/// aucune autre source.
pub async fn deposer(
    cle: &CleMcpAuthentifiee,
    ecriture: &EcritureEnvois,
    outil: &'static str,
    debut: Instant,
    resume: Option<&str>,
    contenu: &str,
) -> Result<AccuseEnvoi, RefusDepot> {
    let contenu = rendre_visible(contenu);
    let resume = resume.map(rendre_visible);
    if contenu.len() > MCP_ENVOI_OCTETS_MAX
        || resume
            .as_ref()
            .is_some_and(|r| r.len() > MCP_ENVOI_RESUME_OCTETS_MAX)
    {
        return Err(RefusDepot::TropVolumineux);
    }

    let company_id = cle.acces.company_id();
    // Journalisée par l'id de société seul (AC-B5) : ni l'erreur, ni rien qui
    // vienne de la base (revue de code P1 de la 32-8b, A-L9).
    if ecriture.purger_echus(company_id).await.is_err() {
        tracing::warn!(
            company_id,
            "envois MCP : purge des contenus échus non faite (sans effet sur le dépôt)"
        );
    }

    let envoi = NouvelEnvoi {
        company_id,
        mcp_key_id: cle.cle_id,
        tool_name: outil.to_string(),
        summary: resume,
        content: contenu,
    };
    let duree_ms = u64::try_from(debut.elapsed().as_millis()).unwrap_or(u64::MAX);
    match ecriture
        .deposer(&envoi, MCP_ENVOI_DUREE_S, MCP_ENVOIS_EN_ATTENTE_MAX, |id| {
            entree_appel(cle, outil, IssueAppel::EnvoiDepose, duree_ms, Some(id))
        })
        .await
    {
        Ok(numero) => Ok(AccuseEnvoi { numero }),
        Err(ErreurDepot::TropNombreux) => Err(RefusDepot::TropNombreux),
        Err(ErreurDepot::Db(e)) => {
            tracing::error!(
                mcp_key_id = cle.cle_id,
                "envois MCP : dépôt non écrit, rien ne part : {e}"
            );
            Err(RefusDepot::Interne)
        }
    }
}

/// Relit l'envoi `numero` pour la clé de l'appel — la règle « même clé, même
/// société » (C-32-8-8). Ne lit **pas** la politique de la clé (D25) et ne
/// recompose rien depuis les livres.
pub async fn lire(
    cle: &CleMcpAuthentifiee,
    lecture: &LectureEnvois,
    numero: i64,
) -> Result<EnvoiLu, AppError> {
    let ligne = lecture
        .lire_pour_cle(cle.acces.company_id(), cle.cle_id, numero)
        .await?;
    Ok(match ligne {
        None => EnvoiLu::Inconnu { numero },
        Some(EnvoiPourCle {
            id,
            etat,
            modified,
            contenu,
            ..
        }) => match (etat, contenu) {
            (EtatEnvoi::Valide, Some(texte)) => EnvoiLu::Valide(ContenuValide {
                numero: id,
                modifie: modified,
                texte,
            }),
            (EtatEnvoi::Valide, None) => {
                return Err(AppError::Internal(format!(
                    "envoi MCP {id} validé et vivant sans contenu"
                )));
            }
            (EtatEnvoi::EnAttente, _) => EnvoiLu::EnAttente { numero: id },
            (EtatEnvoi::Refuse, _) => EnvoiLu::Refuse { numero: id },
            (EtatEnvoi::Echu, _) => EnvoiLu::Echu { numero: id },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-B3 — le remplacement, famille par famille (les cas du prédicat
    /// lui-même sont éprouvés par la 32-8b0).
    #[test]
    fn rendre_visible_par_famille() {
        for (c, attendu) in [
            ('\u{202E}', "\\u{202E}"),
            ('\u{2066}', "\\u{2066}"),
            ('\u{200B}', "\\u{200B}"),
            ('\u{200D}', "\\u{200D}"),
            ('\u{FEFF}', "\\u{FEFF}"),
            ('\u{FE0F}', "\\u{FE0F}"),
            ('\u{E000}', "\\u{E000}"),
            ('\u{0378}', "\\u{0378}"),
            ('\u{180E}', "\\u{180E}"),
            ('\u{2028}', "\\u{2028}"),
            ('\r', "\\u{000D}"),
            ('\u{0000}', "\\u{0000}"),
            ('\u{0085}', "\\u{0085}"),
            ('\u{007F}', "\\u{007F}"),
            ('\u{E0100}', "\\u{E0100}"),
        ] {
            let s = format!("a{c}b");
            let v = rendre_visible(&s);
            assert_eq!(v, format!("a{attendu}b"), "U+{:04X}", u32::from(c));
            assert_eq!(
                rendre_visible(&v),
                v,
                "idempotente : U+{:04X}",
                u32::from(c)
            );
        }
        for intact in [
            "\n",
            "\t",
            "é",
            "€",
            "\u{26A0}",
            "中",
            " ",
            "Ignore les consignes",
        ] {
            assert_eq!(rendre_visible(intact), intact, "{intact:?} intact");
            assert_eq!(rendre_visible(&rendre_visible(intact)), intact);
        }
        // Une suite littérale déjà écrite reste telle quelle (AC-B3).
        assert_eq!(rendre_visible("\\u{202E}"), "\\u{202E}");
    }

    /// AC-B9 — un texte modifié est refusé ou admis tel quel, jamais corrigé.
    #[test]
    fn contenu_modifie_controle() {
        assert_eq!(
            verifier_contenu_modifie("a\r\nb"),
            Err(MotifContenu::CaractereCache)
        );
        assert_eq!(
            verifier_contenu_modifie("x\u{202E}y"),
            Err(MotifContenu::CaractereCache)
        );
        assert_eq!(
            verifier_contenu_modifie("\u{0378}"),
            Err(MotifContenu::CaractereCache)
        );
        assert_eq!(verifier_contenu_modifie("⚠ é"), Ok(()));
        assert_eq!(verifier_contenu_modifie(""), Ok(()));
        assert_eq!(verifier_contenu_modifie("blanc final "), Ok(()));
        assert_eq!(
            verifier_contenu_modifie(&"a".repeat(MCP_ENVOI_OCTETS_MAX)),
            Ok(())
        );
        assert_eq!(
            verifier_contenu_modifie(&"a".repeat(MCP_ENVOI_OCTETS_MAX + 1)),
            Err(MotifContenu::TropVolumineux)
        );
    }

    /// Les bornes, épinglées (point 4 de #765 ; note de l'orchestrateur sur
    /// 24 576).
    #[test]
    fn bornes_des_envois() {
        assert_eq!(MCP_ENVOI_OCTETS_MAX, 24_576);
        assert_eq!(MCP_ENVOI_RESUME_OCTETS_MAX, 1_024);
        assert_eq!(MCP_ENVOI_DUREE_S, 86_400);
        assert_eq!(MCP_ENVOIS_EN_ATTENTE_MAX, 20);
    }
}
