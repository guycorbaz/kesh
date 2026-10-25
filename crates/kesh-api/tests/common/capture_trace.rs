//! Capture **générique** de `tracing` — Story 32-6a (AC-a14, validation P3 de
//! la 32-6, R3).
//!
//! Retient **tous** les évènements — tous niveaux, `TRACE` et `DEBUG`
//! compris, toutes cibles —, chacun sous la forme d'un texte : niveau, cible,
//! puis chaque champ formaté (`message` compris). Un test peut ainsi exiger
//! qu'une valeur n'apparaisse dans **aucun** évènement.
//!
//! ⚠️ Montage par fil (`tracing::subscriber::set_default`), comme
//! `capture_rejeu.rs` : `#[sqlx::test]` et `#[tokio::test]` exécutent le test
//! sur un runtime à un seul fil, où tourne aussi la requête envoyée par
//! `oneshot`. Un évènement émis sur un autre fil échapperait à la capture :
//! d'où le **témoin** que chaque test doit exiger ([`CaptureTrace::exiger`]) —
//! sans lui, une capture vide ferait passer un test d'absence à vide.

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::subscriber::DefaultGuard;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};

struct Couche {
    evenements: Arc<Mutex<Vec<String>>>,
}

struct Formate<'a>(&'a mut String);

impl Visit for Formate<'_> {
    fn record_str(&mut self, field: &Field, value: &str) {
        let _ = write!(self.0, " {}={}", field.name(), value);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let _ = write!(self.0, " {}={:?}", field.name(), value);
    }
}

impl<S: Subscriber> Layer<S> for Couche {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let mut texte = format!("{} {}", meta.level(), meta.target());
        event.record(&mut Formate(&mut texte));
        self.evenements.lock().unwrap().push(texte);
    }
}

/// La capture en cours.
pub struct CaptureTrace {
    _garde: DefaultGuard,
    evenements: Arc<Mutex<Vec<String>>>,
}

impl CaptureTrace {
    /// Installe la capture pour la durée de la valeur rendue.
    pub fn installer() -> Self {
        let evenements = Arc::new(Mutex::new(Vec::new()));
        let couche = Couche {
            evenements: evenements.clone(),
        };
        let garde = tracing::subscriber::set_default(tracing_subscriber::registry().with(couche));
        Self {
            _garde: garde,
            evenements,
        }
    }

    /// Les évènements captés, formatés, dans l'ordre.
    pub fn evenements(&self) -> Vec<String> {
        self.evenements.lock().unwrap().clone()
    }

    /// Le témoin : exige un évènement qui contienne `fragment`.
    pub fn exiger(&self, fragment: &str) {
        let vus = self.evenements();
        assert!(
            vus.iter().any(|e| e.contains(fragment)),
            "⛔ témoin de la capture : aucun évènement ne contient {fragment:?} \
             (vus : {vus:?}) — la capture ne voit pas les évènements du serveur, \
             et un test d'absence passerait à vide"
        );
    }

    /// Exige qu'**aucun** évènement ne contienne `fragment`.
    pub fn exiger_absent(&self, fragment: &str) {
        let fautifs: Vec<String> = self
            .evenements()
            .into_iter()
            .filter(|e| e.contains(fragment))
            .collect();
        assert!(
            fautifs.is_empty(),
            "⛔ {fragment:?} apparaît au journal du serveur : {fautifs:?}"
        );
    }
}
