//! Le registre des outils (AC-a21) : ce que `/mcp` sert, et le délai d'un
//! outil.
//!
//! Les deux points d'accès du répartiteur — `outil_pour` et
//! `descripteurs_pour` — filtrent par les portées de la clé ; ce sont les
//! seuls, et ils ne sont appelés qu'en `mcp/repartiteur.rs` (garde
//! `executer_appele_par_le_repartiteur_seul`). La liste des outils est
//! privée.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use crate::mcp::DELAI_OUTIL;
use crate::mcp::outil::McpTool;
use crate::mcp::outils::kesh_version::KeshVersion;
use crate::services::acces::Portee;

/// Un outil et son schéma. L'outil est partagé (`Arc`) : le répartiteur
/// l'exécute dans une tâche qui le possède (C-32-6-56, revue P2, H1).
struct Entree {
    outil: Arc<dyn McpTool>,
    schema: serde_json::Value,
}

/// Les outils servis, dans leur ordre, et le délai d'exécution d'un outil.
pub struct Registre {
    outils: Vec<Entree>,
    delai: Duration,
}

impl Registre {
    /// Le registre de production : `kesh_version` seul (la 32-9 l'amendera),
    /// délai [`DELAI_OUTIL`]. Appelé en `lib.rs` seulement (garde
    /// `registre_standard_en_un_lieu`).
    pub fn standard() -> Self {
        Self::vide().avec(Box::new(KeshVersion))
    }

    /// Un registre sans outil, délai [`DELAI_OUTIL`] — pour les tests.
    pub fn vide() -> Self {
        Self {
            outils: Vec::new(),
            delai: DELAI_OUTIL,
        }
    }

    /// Ajoute un outil.
    ///
    /// # Panics
    ///
    /// Si son schéma d'entrée n'est pas du JSON — un texte du binaire, que le
    /// test `registre_coherent` contrôle pour le registre de production.
    pub fn avec(mut self, outil: Box<dyn McpTool>) -> Self {
        let schema = serde_json::from_str(outil.schema_entree())
            .unwrap_or_else(|_| panic!("schéma d'entrée de `{}` : JSON invalide", outil.nom()));
        self.outils.push(Entree {
            outil: Arc::from(outil),
            schema,
        });
        self
    }

    /// Remplace le délai d'exécution d'un outil (tests).
    pub fn avec_delai(mut self, delai: Duration) -> Self {
        self.delai = delai;
        self
    }

    /// Le délai d'exécution d'un outil — sa seule source.
    pub fn delai(&self) -> Duration {
        self.delai
    }

    /// `(nom, schéma, portée)` de chaque outil, dans l'ordre — pour les tests
    /// d'inventaire et de cohérence.
    #[doc(hidden)]
    pub fn inventaire(&self) -> Vec<(&'static str, &'static str, Portee)> {
        self.outils
            .iter()
            .map(|e| (e.outil.nom(), e.outil.schema_entree(), e.outil.portee()))
            .collect()
    }

    /// L'outil de ce nom, **si** sa portée appartient à `portees` ; sinon
    /// `None`, comme un nom inconnu (aucun oracle). Rendu partagé, pour que la
    /// tâche qui l'exécute le possède (C-32-6-56).
    pub(in crate::mcp) fn outil_pour(
        &self,
        nom: &str,
        portees: &BTreeSet<Portee>,
    ) -> Option<Arc<dyn McpTool>> {
        self.outils
            .iter()
            .find(|e| e.outil.nom() == nom && portees.contains(&e.outil.portee()))
            .map(|e| Arc::clone(&e.outil))
    }

    /// Les descripteurs de `tools/list` des outils que `portees` ouvrent, dans
    /// l'ordre du registre, annotés `readOnlyHint: true`.
    pub(in crate::mcp) fn descripteurs_pour(
        &self,
        portees: &BTreeSet<Portee>,
    ) -> Vec<serde_json::Value> {
        self.outils
            .iter()
            .filter(|e| portees.contains(&e.outil.portee()))
            .map(|e| {
                serde_json::json!({
                    "name": e.outil.nom(),
                    "description": e.outil.description(),
                    "inputSchema": e.schema.clone(),
                    "annotations": { "readOnlyHint": true },
                })
            })
            .collect()
    }
}
