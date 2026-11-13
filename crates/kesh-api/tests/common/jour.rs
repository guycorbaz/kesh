//! Story 35-5a0 (#929, #1022, #907) — la date du jour des tests de `kesh-api`.
//!
//! [`aujourd_hui_test`] rend le jour de l'horloge de test du serveur
//! (`kesh_api::horloge::horloge_de_test`) — celle que prennent
//! `AppState::new_for_tests` et les états de test —, si bien qu'un montage et
//! le serveur qu'il interroge ont le même jour.

use chrono::NaiveDate;

/// La date du jour de l'horloge de test.
pub fn aujourd_hui_test() -> NaiveDate {
    kesh_api::horloge::horloge_de_test().aujourd_hui()
}
