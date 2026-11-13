//! Story 35-5a0 (#929, #1022, #907), AC3 — l'horloge de l'état atteint la
//! poignée `Lecture`, et donc le serveur MCP (dont le câblage est tenu par
//! les gardes lexicales de `mcp_gardes_lexicales.rs`).
//!
//! Aucun accès à la base : le pool est paresseux, jamais sollicité.

use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use kesh_api::AppState;
use kesh_api::config::Config;
use kesh_core::horloge::HorlogeFixe;

fn etat_de_test() -> AppState {
    let config = Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "admin-password-12chars".to_string(),
        "dev-secret-at-least-32-bytes-long-for-testing".to_string(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    );
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .connect_lazy("mysql://x:y@127.0.0.1:1/z")
        .unwrap();
    AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n)
}

fn instant(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

/// AC3 — un jour distinct du jour réel, pour que l'égalité ne tienne pas par
/// deux horloges système.
#[tokio::test]
async fn la_lecture_de_l_etat_porte_l_horloge_de_l_etat() {
    let mut state = etat_de_test();
    state.horloge = Arc::new(HorlogeFixe(instant("2025-06-15T10:00:00Z")));
    let jour = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    assert_eq!(state.lecture().aujourd_hui(), jour);
    assert_eq!(state.lecture().horloge().aujourd_hui(), jour);
    assert_eq!(state.horloge.aujourd_hui(), jour);
}
