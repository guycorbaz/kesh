//! Le trait d'un outil MCP, son contexte et ses erreurs (AC-a16, AC-a19,
//! AC-a20).

use std::future::Future;
use std::pin::Pin;

use serde::de::DeserializeOwned;

use crate::mcp::politique::Donnees;
use crate::mcp::sortie::{PORTEE_DES_TIERS, tiers_visibles};
use crate::services::acces::{Canal, ContexteAcces, Portee};
use crate::services::poignees::{Lecture, Trace};
use crate::services::rapports::{Affichage, Rapport};

/// La lecture des outils : une `Lecture` sur le pool **en lecture seule**
/// (`kesh_db::pool::pool_lecture_seule`), d'un type que rien d'autre
/// n'accepte (C-32-6-41).
///
/// Champ privé, jamais déballé hors de ce fichier : seul
/// [`ContexteMcp::lecture`] en rend la `&Lecture`, à l'outil. La règle
/// « servi » et l'authentification exigent une `Lecture` : leur passer celle-ci
/// est une erreur de type, comme passer la `garde` à un outil. ⚠️ Un accesseur
/// public ajouté ici défait la propriété : la revue le lit.
#[derive(Clone)]
pub struct LectureOutils(Lecture);

impl LectureOutils {
    /// Enveloppe la lecture des outils — appelé **une fois**, au câblage de
    /// `lib.rs` (garde `outils_et_politique_en_leur_lieu`).
    pub(crate) fn nouvelle(lecture: Lecture) -> Self {
        Self(lecture)
    }
}

/// Le contexte d'un appel d'outil : qui lit (le contexte d'accès de la clé),
/// par quoi (la lecture des outils) et, pour les seuls rapports, où s'inscrit
/// leur consultation (la `Trace`, privée — Story 32-9a, AC-a7, C-32-9-10).
/// **Rien d'autre** : aucun état.
///
/// ⚠️ La `Trace` ne sort jamais d'ici : seule [`ContexteMcp::afficher`] s'en
/// sert, pour écrire la trace de consultation d'un rapport (garde
/// `trace_confinee_dans_outil`).
pub struct ContexteMcp<'a> {
    acces: &'a ContexteAcces,
    lecture: &'a LectureOutils,
    trace: &'a Trace,
}

impl<'a> ContexteMcp<'a> {
    /// Construit par le **répartiteur seul** (garde
    /// `contexte_mcp_construit_par_le_repartiteur_seul`) ; `pub(super)` le rend
    /// visible du module `mcp` et de ses descendants.
    pub(super) fn nouveau(
        acces: &'a ContexteAcces,
        lecture: &'a LectureOutils,
        trace: &'a Trace,
    ) -> Self {
        Self {
            acces,
            lecture,
            trace,
        }
    }

    /// Le contenu d'un rapport affiché, après l'écriture de sa trace de
    /// consultation (règle R5 de `services`) — la seule voie d'un outil vers
    /// un rapport.
    pub async fn afficher<T>(&self, rapport: Rapport<T, Affichage>) -> T {
        rapport.afficher(self.trace).await
    }

    /// La clé voit-elle les coordonnées des tiers (IBAN, QR-IBAN du créancier,
    /// adresse d'envoi d'une facture) ? Selon [`PORTEE_DES_TIERS`] ; un
    /// contexte de l'API, qui n'atteint pas `src/mcp/`, n'ouvre rien (bras
    /// fermé, validation P2, R7).
    pub fn tiers_visibles(&self) -> bool {
        tiers_visibles_du_canal(self.acces.canal())
    }

    /// Le contexte d'accès que reçoivent les services.
    pub fn acces(&self) -> &ContexteAcces {
        self.acces
    }

    /// La lecture des outils, en lecture seule.
    pub fn lecture(&self) -> &Lecture {
        &self.lecture.0
    }
}

/// La règle de [`ContexteMcp::tiers_visibles`], sur le canal seul — pure, pour
/// que le bras `Canal::Api`, inatteignable par un outil, soit éprouvé
/// (`tiers_visibles_selon_le_canal`, revue de code P1, A3, mutation Ma36).
fn tiers_visibles_du_canal(canal: &Canal) -> bool {
    match canal {
        Canal::Mcp { portees } => tiers_visibles(portees, PORTEE_DES_TIERS),
        Canal::Api { .. } => false,
    }
}

/// Le futur que rend [`McpTool::executer`].
pub type FuturOutil<'a> = Pin<Box<dyn Future<Output = Result<Donnees, ErreurOutil>> + Send + 'a>>;

/// Un outil du serveur MCP.
///
/// Le nom, la description et le schéma sont des textes du binaire : aucune
/// donnée lue n'entre dans `tools/list`. La portée n'a **pas** de défaut.
/// L'annotation `readOnlyHint: true` est posée par le répartiteur, non par
/// l'outil.
pub trait McpTool: Send + Sync + 'static {
    /// Nom unique, `[a-z0-9_]{1,64}` (test `registre_coherent`).
    fn nom(&self) -> &'static str;
    /// Description servie à `tools/list`.
    fn description(&self) -> &'static str;
    /// Schéma JSON des arguments, en texte : un objet, `additionalProperties:
    /// false`.
    fn schema_entree(&self) -> &'static str;
    /// La portée qu'exige l'outil ; jamais `PromptsWrite`.
    fn portee(&self) -> Portee;
    /// Exécute l'outil ; ses arguments se lisent par [`arguments`].
    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        arguments: serde_json::Value,
    ) -> FuturOutil<'a>;
}

/// Le motif d'un refus d'arguments ; chacun a son texte fixe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotifArgument {
    /// Argument absent, inconnu, ou d'un type ou d'une valeur refusés.
    Invalide,
}

/// L'erreur d'un outil — **aucun texte libre** (E5) : chaque variante se
/// traduit en un texte fixe de `mcp::textes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErreurOutil {
    /// Arguments refusés ; `argument` est un nom du binaire.
    ArgumentsInvalides {
        /// L'argument en cause, ou `"(arguments)"`.
        argument: &'static str,
        /// Le motif.
        motif: MotifArgument,
    },
    /// Le service refuse l'accès : rendu comme un outil inconnu (aucun oracle).
    Inconnu,
    /// L'objet demandé n'existe pas.
    Introuvable,
    /// Toute autre erreur ; le détail ne va qu'au journal du serveur.
    Interne,
    /// Sous une clé `required` : le résultat dépasse ce qu'un envoi peut
    /// porter ; rien n'est déposé (Story 32-8b, AC-B4).
    TropVolumineux,
    /// Une limite est atteinte — à la 32-8b, trop d'envois en attente pour la
    /// clé (AC-B5) ; la 32-10a-iii la réemploie.
    LimiteAtteinte,
    /// L'exercice demandé n'existe pas dans la société de la clé (Story 32-9a,
    /// AC-a22) — aucun champ : l'identifiant demandé n'est pas recopié.
    ExerciceIntrouvable,
    /// La période demandée sort de l'exercice (AC-a22) — les dates de
    /// l'exercice ne sont pas recopiées.
    PeriodeHorsExercice,
    /// Le compte demandé n'est pas lettrable : il n'a pas de postes ouverts
    /// (AC-a22).
    CompteNonLettrable,
}

/// Lit les arguments d'un outil. Le type `T` doit porter
/// `#[serde(deny_unknown_fields)]` ; le message de `serde` n'est **jamais**
/// recopié.
pub fn arguments<T: DeserializeOwned>(valeur: serde_json::Value) -> Result<T, ErreurOutil> {
    serde_json::from_value(valeur).map_err(|_| ErreurOutil::ArgumentsInvalides {
        argument: "(arguments)",
        motif: MotifArgument::Invalide,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use kesh_db::entities::Role;

    use super::*;

    /// AC-a6 : un contexte de l'API n'ouvre aucune coordonnée de tiers, quel
    /// que soit le rôle ; sous le MCP, `contacts:read` les ouvre (témoin).
    #[test]
    fn tiers_visibles_selon_le_canal() {
        for role in [Role::Admin, Role::Comptable, Role::Consultation] {
            assert!(!tiers_visibles_du_canal(&Canal::Api { role }), "{role:?}");
        }
        let avec = BTreeSet::from([Portee::InvoicesRead, Portee::ContactsRead]);
        assert!(tiers_visibles_du_canal(&Canal::Mcp { portees: avec }));
        let sans = BTreeSet::from([Portee::InvoicesRead]);
        assert!(!tiers_visibles_du_canal(&Canal::Mcp { portees: sans }));
    }
}
