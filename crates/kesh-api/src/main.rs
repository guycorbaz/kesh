mod config;
mod routes;

use axum::{routing::get, Router};
use sqlx::mysql::MySqlPoolOptions;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // Charger la configuration
    let config = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Erreur de configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Connexion à la base de données (gracieuse si indisponible — FR89)
    let pool = match MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            tracing::info!("Base de données : connectée");
            Some(pool)
        }
        Err(e) => {
            tracing::warn!("Base de données : indisponible ({})", e);
            tracing::warn!("L'application démarre sans connexion DB — healthcheck retournera 503");
            None
        }
    };

    // Construire le routeur
    let static_dir =
        std::env::var("KESH_STATIC_DIR").unwrap_or_else(|_| "frontend/build".into());

    let app = Router::new()
        .route("/health", get(routes::health::health_check))
        .fallback_service(
            ServeDir::new(&static_dir)
                .fallback(ServeFile::new(format!("{}/index.html", static_dir))),
        )
        .with_state(pool);

    // Démarrer le serveur
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Kesh démarré sur http://{}:{}", config.host, config.port);
    tracing::info!("Healthcheck : http://{}:{}/health", config.host, config.port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Impossible de bind le port");

    axum::serve(listener, app)
        .await
        .expect("Erreur serveur");
}
