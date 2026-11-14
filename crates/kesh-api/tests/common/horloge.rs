//! Story 35-5a (#1022, #907, #929) — fixer l'horloge d'un état de test.
//!
//! Un test qui asserte une date du jour arrête l'horloge de son `AppState` sur
//! un instant choisi, sans lire `KESH_TEST_INSTANT` ni l'heure réelle. Son
//! montage date alors ses pièces sur cet instant (ou en dates fixes), non sur
//! `common::jour::aujourd_hui_test()`, qui ignore cette horloge.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use kesh_api::AppState;
use kesh_core::horloge::HorlogeFixe;

/// `state`, son horloge arrêtée sur `instant`.
pub fn avec_horloge(mut state: AppState, instant: DateTime<Utc>) -> AppState {
    state.horloge = Arc::new(HorlogeFixe(instant));
    state
}
