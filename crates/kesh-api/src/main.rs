//! Point d'entrée du serveur kesh-api.
//!
//! Ordre de démarrage :
//! 1. Logging (tracing)
//! 2. Config (échec fatal si `KESH_JWT_SECRET` absent ou trop court)
//! 3. Pool MariaDB **obligatoire** — revirement story 1.5 vs 1.2 :
//!    sans DB, l'auth ne peut pas fonctionner, le serveur refuse de
//!    démarrer. Le healthcheck `/health` conserve son comportement
//!    dégradé (503) pour les pertes de DB **après** démarrage.
//! 4. Migrations (`MIGRATOR.run()`) — avance partielle de la story 8.2,
//!    strictement limitée à `run()`.
//! 5. Bootstrap admin (`ensure_admin_user`) si la table users est vide.
//! 6. Build router + axum::serve.

use std::sync::Arc;

use kesh_api::{auth::bootstrap, build_router, config::Config, AppState};
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

    // 4. Migrations
    if let Err(e) = kesh_db::MIGRATOR.run(&pool).await {
        tracing::error!("Échec des migrations : {}", e);
        std::process::exit(1);
    }
    tracing::info!("Migrations appliquées");

    // 5. Bootstrap admin
    if let Err(e) = bootstrap::ensure_admin_user(&pool, &config).await {
        tracing::error!("Échec du bootstrap admin : {}", e);
        std::process::exit(1);
    }

    // 5b. Pré-chauffage du DUMMY_HASH — fait au démarrage pour que toute
    // défaillance d'Argon2 (OsRng indisponible) apparaisse ici et pas
    // dans le handler du premier login.
    kesh_api::auth::password::warm_up_dummy_hash();

    // 6. Build router + serve
    let static_dir = std::env::var("KESH_STATIC_DIR").unwrap_or_else(|_| "frontend/build".into());
    let bind_addr = format!("{}:{}", config.host, config.port);

    let state = AppState {
        pool,
        config: Arc::new(config),
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

    axum::serve(listener, app).await.expect("Erreur serveur");
}
