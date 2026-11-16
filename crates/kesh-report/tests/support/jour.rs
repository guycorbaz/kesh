//! Story 35-5a0 (#929, #1022, #907) — la date du jour des tests de
//! `kesh-report`, jumelle de `kesh-db/tests/support/jour.rs`.
//!
//! [`aujourd_hui_test`] rend le jour de l'horloge du serveur : celle qu'arrête
//! `KESH_TEST_INSTANT` (RFC 3339) si la variable est posée, l'heure réelle
//! sinon — lue à chaque appel, sans mémoire. Vide ou blanche, la variable vaut
//! absente, comme côté `kesh-api` (`instant_de_test` rogne la valeur).
//!
//! Un binaire l'inclut par `#[path = "support/jour.rs"] mod jour;`.

use chrono::{NaiveDate, Utc};
use kesh_core::horloge::{Horloge, HorlogeFixe, NOM_VARIABLE_INSTANT, instant_de_test};

/// La date du jour de l'horloge de test.
pub fn aujourd_hui_test() -> NaiveDate {
    HorlogeFixe(
        instant_de_test(std::env::var(NOM_VARIABLE_INSTANT).ok().as_deref())
            .unwrap_or_else(Utc::now),
    )
    .aujourd_hui()
}
