//! Configuration de l'application kesh-api.
//!
//! Chargée depuis les variables d'environnement (via `dotenvy`) au démarrage.
//! Les secrets (`jwt_secret`, `database_url`, `admin_password`) sont masqués
//! dans `Debug` pour éviter toute fuite via les logs `tracing`.

use std::env;
use std::time::Duration;

use chrono::TimeDelta;

/// Erreurs de chargement de la configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// Variable d'environnement obligatoire manquante.
    MissingVar(String),
    /// `KESH_JWT_SECRET` présent mais trop court (< 32 bytes).
    WeakJwtSecret { actual_bytes: usize },
    /// `KESH_ADMIN_PASSWORD` vide ou composé uniquement de whitespace.
    /// Refus explicite — un mot de passe vide permettrait un login avec
    /// une string vide, ce qui est catastrophique.
    EmptyAdminPassword,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVar(var) => {
                write!(f, "Variable d'environnement manquante: {}", var)
            }
            ConfigError::WeakJwtSecret { actual_bytes } => {
                write!(
                    f,
                    "KESH_JWT_SECRET trop court : {} bytes, minimum 32. \
                     Générer un secret via : openssl rand -hex 32",
                    actual_bytes
                )
            }
            ConfigError::EmptyAdminPassword => {
                write!(
                    f,
                    "KESH_ADMIN_PASSWORD est vide ou composé uniquement de whitespace — \
                     un mot de passe non-trivial est obligatoire, même en dev."
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration de l'application, construite une fois au démarrage.
///
/// Tous les champs secrets sont masqués par l'impl `Debug` manuelle.
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub host: String,
    pub admin_username: String,
    pub admin_password: String,
    pub db_connect_timeout: Duration,

    // --- Story 1.5 : authentification ---
    /// Clé secrète HS256 pour signer les JWT. ≥ 32 bytes, obligatoire.
    jwt_secret: String,
    /// Durée de vie d'un access token JWT (défaut 15 min).
    pub jwt_expiry: TimeDelta,
    /// Lifetime absolu d'un refresh token (défaut 30 jours).
    pub refresh_token_max_lifetime: TimeDelta,
}

// Debug personnalisé : masquer les secrets pour éviter toute fuite via logs
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"***")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("admin_username", &self.admin_username)
            .field("admin_password", &"***")
            .field("db_connect_timeout", &self.db_connect_timeout)
            .field("jwt_secret", &"***")
            .field("jwt_expiry", &self.jwt_expiry)
            .field("refresh_token_max_lifetime", &self.refresh_token_max_lifetime)
            .finish()
    }
}

impl Config {
    /// Accès aux bytes du secret JWT, point d'exposition unique.
    ///
    /// Utilisé par le middleware et les handlers auth — ne jamais
    /// exposer `jwt_secret` directement.
    pub fn jwt_secret_bytes(&self) -> &[u8] {
        self.jwt_secret.as_bytes()
    }

    /// Constructeur pour les **tests d'intégration uniquement**.
    ///
    /// Permet de construire un `Config` directement sans passer par
    /// `from_env()`. À ne jamais utiliser en code de production.
    ///
    /// # Pourquoi `pub`
    ///
    /// Les tests d'intégration (`tests/*.rs`) ne peuvent pas accéder
    /// aux modules `#[cfg(test)]` internes du crate. Ce constructeur
    /// doit être `pub` pour être utilisable depuis `tests/auth_e2e.rs`,
    /// mais la docstring est explicite : **ne pas appeler depuis du
    /// code non-test**.
    ///
    /// # Validation
    ///
    /// Applique **les mêmes invariants** que `from_env()` :
    /// - `jwt_secret` ≥ 32 bytes
    /// - `jwt_expiry` strictement positif, ≤ 24 h
    /// - `refresh_token_max_lifetime` strictement positif, ≤ 365 jours
    /// - `admin_password` non vide
    ///
    /// Panique si un invariant est violé — c'est un constructeur de
    /// tests, la panique est le bon signal pour un setup de test
    /// incorrect.
    #[doc(hidden)]
    pub fn from_fields_for_test(
        database_url: String,
        admin_username: String,
        admin_password: String,
        jwt_secret: String,
        jwt_expiry: TimeDelta,
        refresh_token_max_lifetime: TimeDelta,
    ) -> Self {
        assert!(
            jwt_secret.len() >= 32,
            "from_fields_for_test: jwt_secret must be ≥ 32 bytes (got {})",
            jwt_secret.len()
        );
        assert!(
            !admin_password.is_empty(),
            "from_fields_for_test: admin_password must not be empty"
        );
        assert!(
            jwt_expiry > TimeDelta::zero() && jwt_expiry <= TimeDelta::hours(24),
            "from_fields_for_test: jwt_expiry must be in (0, 24h], got {jwt_expiry}"
        );
        assert!(
            refresh_token_max_lifetime > TimeDelta::zero()
                && refresh_token_max_lifetime <= TimeDelta::days(365),
            "from_fields_for_test: refresh_token_max_lifetime must be in (0, 365d], got {refresh_token_max_lifetime}"
        );

        Config {
            database_url,
            port: 3000,
            host: "127.0.0.1".to_string(),
            admin_username,
            admin_password,
            db_connect_timeout: Duration::from_secs(10),
            jwt_secret,
            jwt_expiry,
            refresh_token_max_lifetime,
        }
    }

    /// Charge la configuration depuis les variables d'environnement.
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

        // Trim explicite (patch V1) : un `KESH_ADMIN_USERNAME=" admin"`
        // (espace initial accidentel dans un copier-coller) créerait un
        // admin avec un username contenant un espace. Au login, le handler
        // trim l'input utilisateur — le lookup ne matcherait jamais.
        // On normalise ici pour éviter l'admin inloggable.
        let admin_username = env::var("KESH_ADMIN_USERNAME")
            .unwrap_or_else(|_| "admin".into())
            .trim()
            .to_string();

        let admin_password = env::var("KESH_ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into());

        // Rejeter empty / whitespace-only (patch #7) : on ne veut pas hasher
        // une chaîne vide et créer un admin avec un password vide.
        if admin_password.trim().is_empty() {
            return Err(ConfigError::EmptyAdminPassword);
        }

        if admin_password == "changeme" {
            tracing::warn!(
                "KESH_ADMIN_PASSWORD est 'changeme' — changez-le avant toute utilisation en production"
            );
        }

        let db_connect_timeout = Duration::from_secs(10);

        // --- Story 1.5 : JWT ---

        let jwt_secret =
            env::var("KESH_JWT_SECRET").map_err(|_| ConfigError::MissingVar("KESH_JWT_SECRET".into()))?;

        if jwt_secret.len() < 32 {
            return Err(ConfigError::WeakJwtSecret {
                actual_bytes: jwt_secret.len(),
            });
        }

        if jwt_secret.contains("change-me") {
            tracing::warn!(
                "KESH_JWT_SECRET contient 'change-me' — générez un vrai secret via : openssl rand -hex 32"
            );
        }

        // KESH_JWT_EXPIRY_MINUTES : optionnel, défaut 15, borne 1-1440
        let jwt_expiry_minutes = match env::var("KESH_JWT_EXPIRY_MINUTES") {
            Ok(val) => match val.parse::<i64>() {
                Ok(m) if (1..=1440).contains(&m) => m,
                Ok(m) => {
                    tracing::warn!(
                        "KESH_JWT_EXPIRY_MINUTES={} hors borne [1, 1440], utilisation du défaut 15",
                        m
                    );
                    15
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_JWT_EXPIRY_MINUTES='{}' invalide, utilisation du défaut 15",
                        val
                    );
                    15
                }
            },
            Err(_) => 15,
        };
        let jwt_expiry = TimeDelta::minutes(jwt_expiry_minutes);

        // KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS : optionnel, défaut 30, borne 1-365
        let refresh_token_days = match env::var("KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS") {
            Ok(val) => match val.parse::<i64>() {
                Ok(d) if (1..=365).contains(&d) => d,
                Ok(d) => {
                    tracing::warn!(
                        "KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS={} hors borne [1, 365], utilisation du défaut 30",
                        d
                    );
                    30
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS='{}' invalide, utilisation du défaut 30",
                        val
                    );
                    30
                }
            },
            Err(_) => 30,
        };
        let refresh_token_max_lifetime = TimeDelta::days(refresh_token_days);

        Ok(Config {
            database_url,
            port,
            host,
            admin_username,
            admin_password,
            db_connect_timeout,
            jwt_secret,
            jwt_expiry,
            refresh_token_max_lifetime,
        })
    }
}

/// Helpers de construction de `Config` pour les tests unitaires.
///
/// Évite de passer par `Config::from_env()` (qui touche les variables
/// d'environnement globales) dans les tests qui n'ont pas besoin de
/// tester le chargement ENV lui-même.
#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;

    /// Construit un `Config` de test directement, sans passer par ENV.
    ///
    /// Valeurs par défaut raisonnables pour tests :
    /// - `jwt_secret` : 48 bytes constants
    /// - `jwt_expiry` : 15 min
    /// - `refresh_token_max_lifetime` : 30 jours
    pub fn make_test_config(admin_username: &str, admin_password: &str) -> Config {
        Config {
            database_url: "mysql://test:test@localhost:3306/test".to_string(),
            port: 3000,
            host: "127.0.0.1".to_string(),
            admin_username: admin_username.to_string(),
            admin_password: admin_password.to_string(),
            db_connect_timeout: Duration::from_secs(10),
            jwt_secret: "test-secret-32-bytes-minimum-test-secret-padding".to_string(),
            jwt_expiry: TimeDelta::minutes(15),
            refresh_token_max_lifetime: TimeDelta::days(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    /// Secret de test ≥ 32 bytes, utilisé dans tous les tests qui ont
    /// besoin d'un `KESH_JWT_SECRET` valide.
    const TEST_JWT_SECRET: &str = "test-secret-32-bytes-minimum-test-secret-padding";

    /// Mutex global pour sérialiser les tests qui touchent aux variables
    /// d'environnement. `cargo test` parallélise par défaut, et les env
    /// vars sont un état global process-wide : sans sérialisation, les
    /// tests se marchent dessus. On accepte un peu de poison safety :
    /// si un test panique en tenant le lock, on unwrap le poison et on
    /// continue (les tests suivants re-init l'état).
    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Reset des variables d'environnement communes entre tests.
    fn reset_env() {
        // SAFETY: caller must hold env_lock()
        unsafe {
            env::remove_var("DATABASE_URL");
            env::remove_var("KESH_PORT");
            env::remove_var("KESH_HOST");
            env::remove_var("KESH_ADMIN_USERNAME");
            env::remove_var("KESH_ADMIN_PASSWORD");
            env::remove_var("KESH_JWT_SECRET");
            env::remove_var("KESH_JWT_EXPIRY_MINUTES");
            env::remove_var("KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS");
        }
    }

    fn set_minimum_required() {
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
        }
    }

    #[test]
    fn config_from_env_with_database_url() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.database_url, "mysql://test:test@localhost:3306/test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.jwt_expiry, TimeDelta::minutes(15));
        assert_eq!(config.refresh_token_max_lifetime, TimeDelta::days(30));
    }

    #[test]
    fn config_from_env_missing_database_url() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
        }

        let result = Config::from_env();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::MissingVar(var) => assert_eq!(var, "DATABASE_URL"),
            other => panic!("expected MissingVar(DATABASE_URL), got {other:?}"),
        }
    }

    #[test]
    fn config_debug_hides_secrets() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://user:s3cret_pass@localhost/db");
            env::set_var("KESH_ADMIN_PASSWORD", "my_s3cret_admin");
            env::set_var("KESH_JWT_SECRET", "super-secret-jwt-key-32-bytes-minimum");
        }

        let config = Config::from_env().expect("Config should load");
        let debug_output = format!("{:?}", config);
        assert!(
            !debug_output.contains("s3cret_pass"),
            "DATABASE_URL leaked in debug"
        );
        assert!(
            !debug_output.contains("my_s3cret_admin"),
            "admin_password leaked in debug"
        );
        assert!(
            !debug_output.contains("super-secret-jwt-key"),
            "jwt_secret leaked in debug: {}",
            debug_output
        );
        assert!(debug_output.contains("***"), "secrets should be masked");
    }

    #[test]
    fn config_rejects_missing_jwt_secret() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
        }

        let result = Config::from_env();
        match result {
            Err(ConfigError::MissingVar(var)) => assert_eq!(var, "KESH_JWT_SECRET"),
            other => panic!("expected MissingVar(KESH_JWT_SECRET), got {other:?}"),
        }
    }

    #[test]
    fn config_rejects_weak_jwt_secret() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", "too-short");
        }

        let result = Config::from_env();
        match result {
            Err(ConfigError::WeakJwtSecret { actual_bytes }) => {
                assert_eq!(actual_bytes, 9);
            }
            other => panic!("expected WeakJwtSecret, got {other:?}"),
        }
    }

    #[test]
    fn config_jwt_secret_bytes_matches_configured_value() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.jwt_secret_bytes(), TEST_JWT_SECRET.as_bytes());
    }

    #[test]
    fn config_jwt_expiry_respects_env_override() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_JWT_EXPIRY_MINUTES", "60");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.jwt_expiry, TimeDelta::minutes(60));
    }

    #[test]
    fn config_jwt_expiry_out_of_bounds_falls_back_to_default() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_JWT_EXPIRY_MINUTES", "99999");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.jwt_expiry, TimeDelta::minutes(15));
    }

    #[test]
    fn config_rejects_empty_admin_password() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_PASSWORD", "");
        }

        let result = Config::from_env();
        assert!(
            matches!(result, Err(ConfigError::EmptyAdminPassword)),
            "empty admin password should be rejected, got {result:?}"
        );
    }

    #[test]
    fn config_trims_admin_username() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_USERNAME", "  admin  ");
        }

        let config = Config::from_env().expect("should load");
        assert_eq!(
            config.admin_username, "admin",
            "admin_username should be trimmed to avoid unreachable admin"
        );
    }

    #[test]
    fn config_rejects_whitespace_only_admin_password() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_PASSWORD", "   ");
        }

        let result = Config::from_env();
        assert!(matches!(result, Err(ConfigError::EmptyAdminPassword)));
    }

    #[test]
    fn config_refresh_token_lifetime_respects_env_override() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_REFRESH_TOKEN_MAX_LIFETIME_DAYS", "7");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.refresh_token_max_lifetime, TimeDelta::days(7));
    }
}
