//! La politique de validation avant envoi, au répartiteur (AC-a15, AC-a23 ;
//! Q12 ; C-32-6-14, C-32-6-15, C-32-6-36).
//!
//! En deux temps : [`autoriser`] lit la politique de la clé **avant**
//! l'exécution de l'outil, et ne rend une [`Autorisation`] que sous une clé
//! `exempt` ; [`appliquer`] ne fait ensuite que construire la réponse, et ne
//! s'appelle pas sans `Autorisation`. Une clé `required` (le défaut) ne peut
//! donc appeler **aucun** outil avant la 32-8, qui change ces deux fonctions.
//!
//! ⛔ Ce que tient le compilateur : `Donnees` n'a aucun accesseur public — ses
//! champs ne se lisent qu'ici ; `DonneesEnvoyables` et `Autorisation` ont un
//! champ privé — ils ne se construisent qu'ici. **Aucun** des trois ne dérive
//! ni n'implémente `Serialize`, `Default` ni `Debug` (qui permettrait un
//! `format!("{:?}")` vers un texte) ; `Autorisation` n'est ni `Clone` ni
//! `Copy`. Ce qui le garde contre une modification : la garde
//! `donnees_construites_par_la_politique_seule`.

use kesh_db::entities::SendValidation;

/// Ce qu'un outil a lu, à envoyer. Champs privés, aucun accesseur public.
#[derive(Clone)]
pub struct Donnees {
    texte: String,
    structure: Option<serde_json::Value>,
}

impl Donnees {
    /// Les données d'un outil : un bloc de texte et, s'il y a lieu, le contenu
    /// structuré (que le texte, selon la révision `2025-11-25`, doit aussi
    /// sérialiser).
    pub fn nouvelles(texte: String, structure: Option<serde_json::Value>) -> Self {
        Self { texte, structure }
    }
}

/// Le jeton d'une autorisation d'envoi : seul [`autoriser`] le fabrique.
pub struct Autorisation {
    _prive: (),
}

/// Le refus d'une clé soumise à la validation.
pub struct Refus {
    _prive: (),
}

/// Des données que la politique laisse partir ; seule [`appliquer`] en
/// construit.
pub struct DonneesEnvoyables {
    donnees: Donnees,
}

/// Lit la politique de la clé — **seul** lecteur, au répartiteur, **avant**
/// l'exécution : `Required` → refus (l'outil n'est pas exécuté) ; `Exempt` →
/// autorisation.
pub(crate) fn autoriser(validation: SendValidation) -> Result<Autorisation, Refus> {
    match validation {
        SendValidation::Required => Err(Refus { _prive: () }),
        SendValidation::Exempt => Ok(Autorisation { _prive: () }),
    }
}

/// Construit la réponse d'un outil autorisé. Ne relit pas la politique.
pub(crate) fn appliquer(_autorisation: Autorisation, donnees: Donnees) -> DonneesEnvoyables {
    DonneesEnvoyables { donnees }
}

impl DonneesEnvoyables {
    /// Le `result` d'un `tools/call` servi : un bloc de texte, le contenu
    /// structuré s'il y en a, `isError: false`.
    pub(in crate::mcp) fn en_resultat(self) -> serde_json::Value {
        let mut resultat = serde_json::Map::new();
        resultat.insert(
            "content".into(),
            serde_json::json!([{ "type": "text", "text": self.donnees.texte }]),
        );
        if let Some(structure) = self.donnees.structure {
            resultat.insert("structuredContent".into(), structure);
        }
        resultat.insert("isError".into(), serde_json::Value::Bool(false));
        serde_json::Value::Object(resultat)
    }
}
