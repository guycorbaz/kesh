//! Point d'entrée du serveur kesh-api.
//!
//! Ordre de démarrage :
//! 1. Logging (tracing)
//! 2. Config (échec fatal si `KESH_JWT_SECRET` absent ou trop court)
//! 3. Pool MariaDB **obligatoire** — revirement story 1.5 vs 1.2 :
//!    sans DB, l'auth ne peut pas fonctionner, le serveur refuse de
//!    démarrer. Le healthcheck `/health` conserve son comportement
//!    dégradé (503) pour les pertes de DB **après** démarrage.
//!    Downgrade protection (story 10-2) — `check_downgrade_protection`
//!    lit `_kesh_version.kesh_version_min_required` et refuse le boot si
//!    le binaire courant est plus ancien que ce min_required (évite une
//!    corruption silencieuse sur downgrade par erreur).
//! 4. Migrations (`MIGRATOR.run()`) — avance partielle de la story 8.2,
//!    strictement limitée à `run()`.
//!    Record boot version (story 10-2) — `record_boot_version` met à
//!    jour `_kesh_version.kesh_version_last_applied` + `last_boot_at`
//!    pour audit. Non-fatal si échec.
//! 5. Bootstrap admin (`ensure_admin_user`) si la table users est vide.
//! 6. Build router + axum::serve.

use std::sync::Arc;

use std::net::SocketAddr;

use kesh_api::{
    AppState, auth::bootstrap, build_router, config::Config, middleware::rate_limit::RateLimiter,
};
use sqlx::mysql::MySqlPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // 1. Logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // 2. Config
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Erreur de configuration: {}", e);
            std::process::exit(1);
        }
    };

    // 3. Pool MariaDB — obligatoire (story 1.5)
    let pool = match MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(config.db_connect_timeout)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("Base de données : connectée");
            pool
        }
        Err(e) => {
            tracing::error!(
                "Base de données indisponible au démarrage — l'authentification ne peut pas fonctionner sans DB. Arrêt. Erreur : {}",
                e
            );
            std::process::exit(1);
        }
    };

    // 3b. Downgrade protection (story 10-2)
    //
    // Hypothèse mono-instance v0.1 : Kesh tourne en single-process sur NAS
    // Synology (docker-compose 1 replica). Le pattern check → MIGRATOR.run()
    // → record n'est pas protégé contre 2 instances concurrentes — le bump
    // multi-instance sera traité en v0.2 (advisory lock GET_LOCK ou similaire).
    //
    // Note `std::process::exit(1)` : termine le processus sans exécuter les
    // destructors Rust (pool MariaDB pas fermé gracefully). Acceptable v0.1
    // sur ce path car le pool n'a fait qu'un SELECT court (pas de transactions
    // ouvertes côté serveur). Pattern à reviser si une future Story introduit
    // des transactions writeable avant le check.
    match kesh_db::version::check_downgrade_protection(&pool, env!("CARGO_PKG_VERSION")).await {
        Ok(kesh_db::version::DowngradeCheckOutcome::FreshInstall) => {
            tracing::info!(
                "Vérification de version DB : fresh install (table _kesh_version absente, sera créée par MIGRATOR.run())"
            );
        }
        Ok(kesh_db::version::DowngradeCheckOutcome::Aligned) => {
            tracing::info!(
                "Vérification de version DB : binaire v{} aligné avec kesh_version_min_required",
                env!("CARGO_PKG_VERSION")
            );
        }
        Ok(kesh_db::version::DowngradeCheckOutcome::BinaryAhead { db_min, binary }) => {
            tracing::info!(
                "Vérification de version DB : binaire v{} > kesh_version_min_required v{} (upgrade)",
                binary,
                db_min
            );
        }
        Err(err @ kesh_db::version::VersionError::DowngradeRefused { .. }) => {
            // Réutilise le Display français du variant plutôt que de dupliquer
            // le message ici (évite drift maintenance).
            tracing::error!("FATAL : {}", err);
            std::process::exit(1);
        }
        Err(e) => {
            tracing::error!(
                "Vérification de version DB échouée (refus du boot pour éviter une corruption silencieuse de données) : {}",
                e
            );
            std::process::exit(1);
        }
    }

    // 4. Migrations
    if let Err(e) = kesh_db::MIGRATOR.run(&pool).await {
        tracing::error!("Échec des migrations : {}", e);
        std::process::exit(1);
    }
    tracing::info!("Migrations appliquées");

    // 4b. Record boot version (story 10-2) — non-fatal si échec
    if let Err(e) = kesh_db::version::record_boot_version(&pool, env!("CARGO_PKG_VERSION")).await {
        tracing::warn!(
            "Impossible d'enregistrer la version de boot (non-fatal) : {}",
            e
        );
    }

    // 5. Bootstrap admin
    if let Err(e) = bootstrap::ensure_admin_user(&pool, &config).await {
        tracing::error!("Échec du bootstrap admin : {}", e);
        std::process::exit(1);
    }

    // 5b. Pré-chauffage du DUMMY_HASH — fait au démarrage pour que toute
    // défaillance d'Argon2 (OsRng indisponible) apparaisse ici et pas
    // dans le handler du premier login.
    kesh_api::auth::password::warm_up_dummy_hash();

    // 5c. Nettoyage des refresh tokens expirés/révoqués > 7 jours (story 1.6)
    let cleanup_cutoff = (chrono::Utc::now() - chrono::TimeDelta::days(7)).naive_utc();
    match kesh_db::repositories::refresh_tokens::delete_expired_and_revoked(&pool, cleanup_cutoff)
        .await
    {
        Ok(count) => {
            if count > 0 {
                tracing::info!("startup cleanup: {} expired/revoked tokens removed", count);
            }
        }
        Err(e) => {
            tracing::warn!("startup cleanup failed (non-fatal): {}", e);
        }
    }

    // 5d. Vérification de cohérence refresh_inactivity vs max_lifetime (story 1.6)
    if config.refresh_inactivity > config.refresh_token_max_lifetime {
        tracing::warn!(
            "refresh_inactivity ({:?}) exceeds max_lifetime ({:?}), sessions will expire by inactivity only",
            config.refresh_inactivity,
            config.refresh_token_max_lifetime
        );
    }

    // 6. i18n (story 2.1)
    let locales_dir =
        std::path::PathBuf::from(std::env::var("KESH_LOCALES_DIR").unwrap_or_else(|_| {
            // En dev : relatif au binaire, en prod : /app/locales
            if std::path::Path::new("crates/kesh-i18n/locales").exists() {
                "crates/kesh-i18n/locales".to_string()
            } else {
                "locales".to_string()
            }
        }));
    let i18n_bundle = match kesh_i18n::I18nBundle::load(&locales_dir) {
        Ok(b) => {
            tracing::info!(
                "i18n : {} locales chargées depuis {}",
                kesh_i18n::Locale::ALL.len(),
                locales_dir.display()
            );
            Arc::new(b)
        }
        Err(e) => {
            tracing::error!("Échec du chargement i18n : {}", e);
            std::process::exit(1);
        }
    };

    // Initialiser les messages d'erreur i18n
    kesh_api::errors::init_error_i18n(i18n_bundle.clone(), config.locale);

    // 7. Build router + serve
    let static_dir = std::env::var("KESH_STATIC_DIR").unwrap_or_else(|_| "frontend/build".into());
    let bind_addr = format!("{}:{}", config.host, config.port);

    let rate_limiter = RateLimiter::new(&config);

    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n: i18n_bundle,
    };

    let app = build_router(state, static_dir);

    tracing::info!("Kesh démarré sur http://{}", bind_addr);
    tracing::info!("Healthcheck : http://{}/health", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Impossible de bind sur {} : {}", bind_addr, e);
            std::process::exit(1);
        });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Erreur serveur");
}
