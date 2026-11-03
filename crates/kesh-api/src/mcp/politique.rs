//! La politique de validation avant envoi, au répartiteur (AC-a15, AC-a23 de
//! la 32-6a, amendées par l'AC-B1 de la 32-8b ; Q10 (c), Q12 ; C-32-6-36,
//! C-32-8-6).
//!
//! En deux temps : [`autoriser`] lit la politique de la clé **avant**
//! l'exécution de l'outil et rend une [`Autorisation`] à deux sorties —
//! `Envoi` sous une clé `exempt`, `Depot` sous une clé `required` (le défaut) ;
//! l'outil s'exécute dans les deux cas ; [`appliquer`] ne fait ensuite que
//! construire la [`Sortie`] : `Envoyer` les données (la seule voie hors
//! validation, E1) ou les `Deposer` pour validation par l'utilisateur — un
//! [`EnvoiAValider`] illisible, que sa seule méthode dépose et qui ne rend
//! qu'un numéro.
//!
//! **Une seule représentation, le texte** (C-32-8-4, E1, M-2 ; v016-1,
//! C-v016-1-1 et C-v016-1-5) : `Donnees` ne porte qu'un bloc de texte ; sous
//! une clé `required`, c'est lui ([`texte_a_valider`]) qui est déposé et relu,
//! sous une clé `exempt` lui qui part — depuis la 0.17.0, aucun résultat ne
//! porte de contenu structuré (#811) ; le résumé ([`Donnees::avec_resume`]) va
//! à l'écran seulement.
//!
//! ⛔ Ce que tient le compilateur : `Donnees` n'a aucun accesseur public — ses
//! champs ne se lisent qu'ici ; `DonneesEnvoyables`, `EnvoiAValider` et les
//! deux jetons ont des champs privés — ils ne se construisent qu'ici. **Aucun**
//! ne dérive ni n'implémente `Serialize`, `Default` ni `Debug` (qui
//! permettrait un `format!("{:?}")` vers un texte) ; `Autorisation`, ses jetons
//! et `EnvoiAValider` ne sont ni `Clone` ni `Copy`. Ce qui le garde contre une
//! modification : la garde `donnees_construites_par_la_politique_seule`.

use std::time::Instant;

use kesh_db::entities::SendValidation;

use crate::services::cles_mcp::CleMcpAuthentifiee;
use crate::services::envois_mcp::{self, AccuseEnvoi, RefusDepot};
use crate::services::poignees::EcritureEnvois;

/// Ce qu'un outil a lu, à envoyer. Champs privés, aucun accesseur public.
#[derive(Clone)]
pub struct Donnees {
    texte: String,
    resume: Option<String>,
}

impl Donnees {
    /// Les données d'un outil : un bloc de texte, le seul canal (v016-1,
    /// C-v016-1-5 ; #811).
    pub fn nouvelles(texte: String) -> Self {
        Self {
            texte,
            resume: None,
        }
    }

    /// Le résumé de la demande, écrit par Kesh (≤ 1 Kio) : montré à l'écran de
    /// validation sous une clé `required`, **jamais envoyé** (C-32-8-25,
    /// C-32-8-50).
    pub fn avec_resume(mut self, resume: String) -> Self {
        self.resume = Some(resume);
        self
    }
}

/// Le jeton d'un envoi sans validation : seul [`autoriser`] le fabrique.
pub struct JetonEnvoi {
    _prive: (),
}

/// Le jeton d'un dépôt pour validation : seul [`autoriser`] le fabrique.
pub struct JetonDepot {
    _prive: (),
}

/// Ce que la politique de la clé autorise — rendu par [`autoriser`] seul.
pub enum Autorisation {
    /// Clé `exempt` : les données partent.
    Envoi(JetonEnvoi),
    /// Clé `required` : les données sont déposées, rien ne part.
    Depot(JetonDepot),
}

/// Des données que la politique laisse partir ; seule [`appliquer`] en
/// construit.
pub struct DonneesEnvoyables {
    donnees: Donnees,
}

/// Des données à faire valider : le résumé et le texte, **aucun accesseur**.
/// Sa seule méthode, [`EnvoiAValider::deposer`], le consomme.
pub struct EnvoiAValider {
    resume: Option<String>,
    contenu: String,
}

/// Ce que [`appliquer`] rend.
pub enum Sortie {
    /// Les données partent (clé `exempt`).
    Envoyer(DonneesEnvoyables),
    /// Les données se déposent (clé `required`).
    Deposer(EnvoiAValider),
}

/// Lit la politique de la clé — **seule** à décider, au répartiteur, **avant**
/// l'exécution : `Required` → [`Autorisation::Depot`] ; `Exempt` →
/// [`Autorisation::Envoi`].
pub(crate) fn autoriser(validation: SendValidation) -> Autorisation {
    match validation {
        SendValidation::Required => Autorisation::Depot(JetonDepot { _prive: () }),
        SendValidation::Exempt => Autorisation::Envoi(JetonEnvoi { _prive: () }),
    }
}

/// Le texte unique d'un envoi à valider : le bloc de texte de l'outil, tel
/// quel. `Donnees` n'a qu'un bloc de texte — aucune variante non textuelle
/// (image, ressource) n'existe à la 32-8b : le cas « bloc non textuel refusé »
/// de l'AC-B2 est sans objet (constaté au Change Log). Le résumé n'y entre
/// jamais.
fn texte_a_valider(d: &Donnees) -> String {
    d.texte.clone()
}

/// Construit la sortie d'un outil exécuté. Ne relit pas la politique ; ne
/// rend rien visible elle-même — le service de dépôt le fait, avant tout
/// contrôle (C-32-8-51).
pub(crate) fn appliquer(autorisation: Autorisation, donnees: Donnees) -> Sortie {
    match autorisation {
        Autorisation::Envoi(_) => Sortie::Envoyer(DonneesEnvoyables { donnees }),
        Autorisation::Depot(_) => Sortie::Deposer(EnvoiAValider {
            contenu: texte_a_valider(&donnees),
            resume: donnees.resume,
        }),
    }
}

impl EnvoiAValider {
    /// Dépose l'envoi pour la clé de l'appel — la seule sortie de son texte
    /// hors de ce fichier, comme argument du service de dépôt. Rend un accusé
    /// sans donnée.
    pub(in crate::mcp) async fn deposer(
        self,
        cle: &CleMcpAuthentifiee,
        ecriture: &EcritureEnvois,
        outil: &'static str,
        debut: Instant,
    ) -> Result<AccuseEnvoi, RefusDepot> {
        envois_mcp::deposer(
            cle,
            ecriture,
            outil,
            debut,
            self.resume.as_deref(),
            &self.contenu,
        )
        .await
    }
}

impl DonneesEnvoyables {
    /// Le `result` d'un `tools/call` servi : un bloc de texte et
    /// `isError: false`, rien d'autre (v016-1, C-v016-1-1 ; #811).
    pub(in crate::mcp) fn en_resultat(self) -> serde_json::Value {
        serde_json::json!({
            "content": [{ "type": "text", "text": self.donnees.texte }],
            "isError": false,
        })
    }
}
