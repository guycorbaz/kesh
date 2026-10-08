//! Témoin du rejeu — Story 15-5e1 (AC4, montage commun ; choix C74).
//!
//! Un test « route victime » qui ne formerait pas son cycle passerait au vert
//! sans qu'aucun rejeu ait eu lieu : le code de retour et les comptes ne
//! départagent pas « annulée puis rejouée » de « réussie du premier coup ».
//! Ce module capte l'événement que `kesh_db::retry::retry_with` émet **avant**
//! chaque nouvelle tentative — `WARN`, cible `kesh_db::retry`, champ
//! `operation` — pour que le test exige qu'il ait eu lieu.
//!
//! ⚠️ L'abonné est installé par `tracing::subscriber::set_default`, pour le
//! seul fil courant : `#[sqlx::test]` exécute le test sur un runtime tokio à
//! un seul fil, où tournent aussi le serveur de test et la requête. Si un test
//! passait à un runtime multi-fil, les événements échapperaient à la capture
//! et le témoin **rougirait** — un faux rouge, jamais un faux vert.

use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::subscriber::DefaultGuard;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};

/// Couche qui retient la valeur du champ `operation` de chaque événement
/// `WARN` (un rejeu) et de chaque événement `ERROR` (un rejeu épuisé) de cible
/// `kesh_db::retry`.
struct CoucheRejeu {
    operations: Arc<Mutex<Vec<String>>>,
    epuisements: Arc<Mutex<Vec<String>>>,
}

/// Lit le champ `operation` d'un événement.
#[derive(Default)]
struct LitOperation(Option<String>);

impl Visit for LitOperation {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "operation" {
            self.0 = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "operation" && self.0.is_none() {
            self.0 = Some(format!("{value:?}"));
        }
    }
}

impl<S: Subscriber> Layer<S> for CoucheRejeu {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        if meta.target() != "kesh_db::retry" {
            return;
        }
        let liste = match *meta.level() {
            Level::WARN => &self.operations,
            Level::ERROR => &self.epuisements,
            _ => return,
        };
        let mut lecteur = LitOperation::default();
        event.record(&mut lecteur);
        if let Some(operation) = lecteur.0 {
            liste.lock().unwrap().push(operation);
        }
    }
}

/// La capture en cours : la garde de l'abonné et les opérations rejouées vues.
pub struct CaptureRejeu {
    _garde: DefaultGuard,
    operations: Arc<Mutex<Vec<String>>>,
    epuisements: Arc<Mutex<Vec<String>>>,
}

impl CaptureRejeu {
    /// Installe la capture pour la durée de la valeur rendue (à appeler
    /// **avant** la première requête du test).
    pub fn installer() -> Self {
        let operations = Arc::new(Mutex::new(Vec::new()));
        let epuisements = Arc::new(Mutex::new(Vec::new()));
        let couche = CoucheRejeu {
            operations: operations.clone(),
            epuisements: epuisements.clone(),
        };
        let garde = tracing::subscriber::set_default(tracing_subscriber::registry().with(couche));
        Self {
            _garde: garde,
            operations,
            epuisements,
        }
    }

    /// Les opérations dont un rejeu a été journalisé, dans l'ordre.
    pub fn operations(&self) -> Vec<String> {
        self.operations.lock().unwrap().clone()
    }

    /// Les opérations dont l'épuisement du rejeu a été journalisé (`error!`).
    pub fn epuisements(&self) -> Vec<String> {
        self.epuisements.lock().unwrap().clone()
    }

    /// Exige au moins un rejeu journalisé pour `operation` — sans quoi le test
    /// a réussi **sans** rejeu, et ne prouve rien.
    pub fn exiger_un_rejeu(&self, operation: &str) {
        let vues = self.operations();
        assert!(
            vues.iter().any(|o| o == operation),
            "⛔ témoin du rejeu : aucun `warn!` de `kesh_db::retry` portant \
             operation = {operation:?} (vus : {vues:?}). La requête a réussi sans \
             être rejouée : le montage n'a pas formé son cycle — après une montée \
             de version de MariaDB, c'est le montage qui est à revoir, non le rejeu."
        );
    }
}
