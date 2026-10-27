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
use crate::mcp::outils::balance_agee::KeshBalanceAgee;
use crate::mcp::outils::ecritures::{KeshEcriture, KeshEcritures};
use crate::mcp::outils::exercices::KeshExercices;
use crate::mcp::outils::factures::{KeshEcheancier, KeshFacture, KeshFactures};
use crate::mcp::outils::factures_fournisseur::{KeshFactureFournisseur, KeshFacturesFournisseur};
use crate::mcp::outils::grand_livre::KeshGrandLivre;
use crate::mcp::outils::kesh_version::KeshVersion;
use crate::mcp::outils::plan_comptable::KeshPlanComptable;
use crate::mcp::outils::postes_ouverts::KeshPostesOuverts;
use crate::mcp::outils::rapports::{KeshBalance, KeshBilan, KeshCompteDeResultat};
use crate::services::acces::Portee;

/// Le nom de l'outil de relecture des envois, **hors registre** (Story 32-8b,
/// AC-B6) : le répartiteur le résout avant le registre ; il n'est pas un
/// `McpTool` et ne passe pas par la politique.
pub const NOM_LIRE_ENVOI: &str = "kesh_lire_envoi";

/// Les noms réservés de la 32-8, qu'aucun outil du registre ne peut prendre
/// (`Registre::avec` refuse ; test `registre_coherent`).
pub const NOMS_RESERVES: &[&str] = &[NOM_LIRE_ENVOI];

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
    /// Le registre de production — **seize** outils, dans cet ordre
    /// (Story 32-9a, AC-a9) : `kesh_version`, les neuf outils `books:read`,
    /// puis les six outils `invoices:read` ; délai [`DELAI_OUTIL`]. Appelé en
    /// `lib.rs` seulement (garde `registre_standard_en_un_lieu`).
    pub fn standard() -> Self {
        Self::vide()
            .avec(Box::new(KeshVersion))
            .avec(Box::new(KeshExercices))
            .avec(Box::new(KeshPlanComptable))
            .avec(Box::new(KeshEcritures))
            .avec(Box::new(KeshEcriture))
            .avec(Box::new(KeshGrandLivre))
            .avec(Box::new(KeshBalance))
            .avec(Box::new(KeshBilan))
            .avec(Box::new(KeshCompteDeResultat))
            .avec(Box::new(KeshPostesOuverts))
            .avec(Box::new(KeshFactures))
            .avec(Box::new(KeshFacture))
            .avec(Box::new(KeshEcheancier))
            .avec(Box::new(KeshBalanceAgee))
            .avec(Box::new(KeshFacturesFournisseur))
            .avec(Box::new(KeshFactureFournisseur))
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
    /// test `registre_coherent` contrôle pour le registre de production —, ou si
    /// son nom est **réservé** ([`NOMS_RESERVES`], Story 32-8b, AC-B6) : un
    /// outil du registre ne peut pas prendre la place de `kesh_lire_envoi`.
    pub fn avec(mut self, outil: Box<dyn McpTool>) -> Self {
        assert!(
            !NOMS_RESERVES.contains(&outil.nom()),
            "outil `{}` : nom réservé hors registre",
            outil.nom()
        );
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
