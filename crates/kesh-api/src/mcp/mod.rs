//! Le serveur MCP de Kesh — l'endpoint `/mcp` (Story 32-6a, refs #723).
//!
//! Un seul point d'entrée, sur le port de Kesh : `POST /mcp`, Streamable HTTP
//! **sans session**, JSON-RPC 2.0, révision **`2025-11-25`** (`2025-06-18`
//! acceptée), réponses JSON (aucun `text/event-stream`). Monté dans `lib.rs`
//! par `route_service("/mcp", routes::mcp::recevoir.with_state(EtatMcp))`,
//! sur le **même chemin** d'Axum que le repli statique (C-32-6-40) : tant que
//! `/mcp` n'est pas servi à un client, sa réponse **est** celle d'une route
//! absente (C-32-6-2).
//!
//! # L'ordre des contrôles (C-32-6-8)
//!
//! ```text
//! 1. pair TCP (ConnectInfo), normalisé           absent            → route absente
//! 2. S = sociétés activées dont le filtre admet  vide ou erreur    → route absente
//! 3. méthode HTTP                                ≠ POST            → 405 Allow: POST
//! 4. Origin (origines propres ∪ celles de S)     refusée           → 403
//! 5. Content-Type                                ≠ JSON            → 415
//! 6. MCP-Protocol-Version                        non servie        → 400 ; absente → debug!
//! 7. Authorization: Bearer → authentifier        refus / erreur    → 401 fixe / 500 fixe
//! 8. société de la clé ∈ S                       non               → 401 fixe
//! 9. Origin (propres ∪ celles de la société)     refusée           → 401 fixe (le même qu'en 7)
//! 10. corps (1 Mio) puis JSON-RPC                (transport)
//! 11. répartiteur : liste close des méthodes ; tools/call → outil_pour(nom,
//!     portées) → politique::autoriser → executer (dans une tâche qui possède
//!     ses connexions, attendue sous le délai — C-32-6-56) → politique::appliquer
//!     → serveur_mcp::journaliser_appel (fail closed) — le tout dans la tâche de
//!     l'appel, qu'un abandon du client n'annule pas (C-32-6-58)
//! 12. reponse::rendre — seule sortie
//! ```
//!
//! # Ce que tient le compilateur, ce que tiennent les gardes
//!
//! - les outils ne lisent que par [`LectureOutils`] (pool en lecture seule,
//!   `kesh_db::pool::pool_lecture_seule`), type propre que ni la règle
//!   « servi » ni l'authentification n'acceptent (C-32-6-41) ;
//! - une donnée d'outil ne part que par la politique : `politique::Donnees` est
//!   illisible hors de `politique.rs`, `DonneesEnvoyables` ne se construit que
//!   par `politique::appliquer`, qui exige une `Autorisation` (C-32-6-36) ;
//! - toute réponse sort de `reponse::rendre(ReponseMcp)`, énumération sans
//!   `AppError` (C-32-6-37) ;
//! - le reste — un seul point d'entrée, `executer` appelé par le répartiteur
//!   seul, aucun pool nommé ici, la lecture seule jamais défaite — est tenu par
//!   les gardes lexicales de `tests/mcp_gardes_lexicales.rs`, qui en écrivent
//!   aussi les angles morts.
//!
//! # La frontière avec la 32-8
//!
//! Le point de branchement est le module `politique` : la 32-8 change
//! `autoriser` et `appliquer` (dépôt d'une proposition sous une clé
//! `required`), signatures et sites d'appel compris ; rien d'autre du
//! répartiteur.

use std::sync::Arc;
use std::time::Duration;

use tower_http::services::{ServeDir, ServeFile};

use crate::services::poignees::{Lecture, Trace};

pub mod outil;
pub mod outils;
pub mod politique;
pub mod prompts;
pub mod registre;
pub(crate) mod repartiteur;
pub(crate) mod reponse;
pub mod reseau;
pub mod textes;
pub(crate) mod transport;

pub use outil::{ContexteMcp, ErreurOutil, LectureOutils, McpTool, MotifArgument, arguments};
pub use politique::Donnees;
pub use registre::Registre;
pub(crate) use reseau::origines_propres;

/// Connexions au plus du pool des outils (C-32-6-10) — et donc instructions
/// simultanées au plus côté serveur, un abandon ne rendant pas le permis avant
/// la fin de l'instruction (C-32-6-56).
pub const MCP_POOL_CONNEXIONS: u32 = 3;

/// Taille maximale du corps d'une requête : 1 Mio.
pub const MCP_CORPS_MAX: usize = 1024 * 1024;

/// Délai d'exécution d'un outil en production (C-32-6-19) ; il ne vit que
/// dans le registre (`Registre::delai`).
pub const DELAI_OUTIL: Duration = Duration::from_secs(30);

/// Révisions du protocole servies, la première étant celle de référence.
pub const VERSIONS_SERVIES: [&str; 2] = ["2025-11-25", "2025-06-18"];

/// Le service de fichiers statiques — le **même** que le repli du routeur.
pub(crate) type ServiceStatique = ServeDir<ServeFile>;

/// L'état de `/mcp`, porté par le service (`Handler::with_state`) : ce qui
/// sert, et **rien d'autre** — ni `AppState` (dont le pool public mettrait le
/// pool ordinaire à portée de ce module), ni `Config` (qui porte des secrets).
#[derive(Clone)]
pub struct EtatMcp {
    /// Les origines propres de Kesh, normalisées une fois au démarrage.
    pub(crate) origines_propres: Arc<[String]>,
    /// La `Lecture` **ordinaire** : la règle « servi » et l'authentification
    /// seulement (C-32-6-38).
    pub(crate) garde: Lecture,
    /// La lecture **des outils**, en lecture seule (AC-a26, C-32-6-41).
    pub(crate) lecture: LectureOutils,
    /// La trace : dernière utilisation de la clé, journal des appels d'outil
    /// (32-6b).
    pub(crate) trace: Trace,
    /// Les outils servis et leur délai.
    pub(crate) registre: Arc<Registre>,
    /// Le service du repli statique, pour la réponse de route absente.
    pub(crate) statique: ServiceStatique,
}
