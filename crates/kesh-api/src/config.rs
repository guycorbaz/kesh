use std::env;
use std::time::Duration;

#[derive(Debug)]
pub enum ConfigError {
    MissingVar(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVar(var) => {
                write!(f, "Variable d'environnement manquante: {}", var)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub host: String,
    pub admin_username: String,
    pub admin_password: String,
    pub db_connect_timeout: Duration,
}

// Debug personnalisé : masquer les secrets
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"***")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("admin_username", &self.admin_username)
            .field("admin_password", &"***")
            .field("db_connect_timeout", &self.db_connect_timeout)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::MissingVar("DATABASE_URL".into()))?;

        let port = match env::var("KESH_PORT") {
            Ok(val) => match val.parse::<u16>() {
                Ok(0) => {
                    tracing::warn!("KESH_PORT=0 invalide, utilisation du port par défaut 3000");
                    3000
                }
                Ok(p) => p,
                Err(_) => {
                    tracing::warn!(
                        "KESH_PORT='{}' n'est pas un numéro de port valide, utilisation du port par défaut 3000",
                        val
                    );
                    3000
                }
            },
            Err(_) => 3000,
        };

        let host = env::var("KESH_HOST").unwrap_or_else(|_| "0.0.0.0".into());

        let admin_username =
            env::var("KESH_ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());

        let admin_password =
            env::var("KESH_ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into());

        if admin_password == "changeme" {
            tracing::warn!(
                "KESH_ADMIN_PASSWORD est 'changeme' — changez-le avant toute utilisation en production"
            );
        }

        let db_connect_timeout = Duration::from_secs(10);

        Ok(Config {
            database_url,
            port,
            host,
            admin_username,
            admin_password,
            db_connect_timeout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_env_with_database_url() {
        // SAFETY: tests run sequentially with --test-threads=1 or in isolation
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::remove_var("KESH_PORT");
            env::remove_var("KESH_HOST");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.database_url, "mysql://test:test@localhost:3306/test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "0.0.0.0");
    }

    #[test]
    fn config_from_env_missing_database_url() {
        // SAFETY: tests run sequentially with --test-threads=1 or in isolation
        unsafe {
            env::remove_var("DATABASE_URL");
        }

        let result = Config::from_env();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::MissingVar(var) => assert_eq!(var, "DATABASE_URL"),
        }
    }

    #[test]
    fn config_debug_hides_secrets() {
        unsafe {
            env::set_var("DATABASE_URL", "mysql://user:s3cret_pass@localhost/db");
            env::set_var("KESH_ADMIN_PASSWORD", "my_s3cret");
        }

        let config = Config::from_env().expect("Config should load");
        let debug_output = format!("{:?}", config);
        assert!(!debug_output.contains("s3cret_pass"), "DATABASE_URL leaked in debug");
        assert!(!debug_output.contains("my_s3cret"), "admin_password leaked in debug");
        assert!(debug_output.contains("***"), "secrets should be masked");
    }
}
