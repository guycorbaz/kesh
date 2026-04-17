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
    /// `KESH_TEST_MODE=true` combiné avec un bind non-loopback — refus
    /// explicite pour éviter d'exposer `/api/v1/_test/*` en staging/prod
    /// (Story 6.4 garde-fou sécurité). Acceptés : `127.0.0.1`, `::1`,
    /// `localhost`. **`0.0.0.0` est explicitement rejeté** car en Docker
    /// `-p 3000:3000` avec bind interne `0.0.0.0` expose la route au
    /// réseau hôte (cf. décision pass 2 / N3).
    TestModeWithPublicBind { host: String },
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
            ConfigError::TestModeWithPublicBind { host } => {
                write!(
                    f,
                    "KESH_TEST_MODE=true incompatible avec KESH_HOST='{}' — \
                     l'endpoint /api/v1/_test/* ne doit jamais être exposé \
                     publiquement. Utiliser KESH_HOST=127.0.0.1 (ou ::1, localhost). \
                     0.0.0.0 est rejeté (Docker -p expose au réseau hôte).",
                    host
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

    // --- Story 1.6 : session & rate limiting ---
    /// Durée d'inactivité avant expiration du refresh token (défaut 15 min).
    /// Sliding expiration : chaque refresh remet le compteur à zéro.
    pub refresh_inactivity: TimeDelta,
    /// Fenêtre de temps pour compter les tentatives de login échouées (défaut 15 min).
    pub rate_limit_window: TimeDelta,
    /// Nombre maximal de tentatives de login échouées par IP avant blocage (défaut 5).
    pub rate_limit_max_attempts: u32,
    /// Durée de blocage d'une IP après dépassement du seuil de rate limiting (défaut 30 min).
    pub rate_limit_block_duration: TimeDelta,

    // --- Story 1.7 : politique de mot de passe ---
    /// Longueur minimale des mots de passe (défaut 12, borne [8, 128]).
    /// Appliquée à la création d'utilisateur, au changement et à la réinitialisation.
    pub password_min_length: u32,

    // --- Story 2.1 : internationalisation ---
    /// Locale de l'instance (défaut FrCh). Configure la langue des messages d'erreur API.
    pub locale: kesh_i18n::Locale,

    // --- Story 6.4 : fixtures E2E déterministes ---
    /// Mode test : active les endpoints `/api/v1/_test/*` (reset + seed DB).
    /// Lu depuis `KESH_TEST_MODE` (`"true"` / `"1"` → `true`). **Par défaut
    /// `false`** — les routes test n'existent pas dans `build_router` si
    /// `test_mode == false` (404 natif Axum). Incompatible avec un bind
    /// non-loopback (refuse le démarrage).
    pub test_mode: bool,
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
            .field(
                "refresh_token_max_lifetime",
                &self.refresh_token_max_lifetime,
            )
            .field("refresh_inactivity", &self.refresh_inactivity)
            .field("rate_limit_window", &self.rate_limit_window)
            .field("rate_limit_max_attempts", &self.rate_limit_max_attempts)
            .field("rate_limit_block_duration", &self.rate_limit_block_duration)
            .field("password_min_length", &self.password_min_length)
            .field("locale", &self.locale)
            .field("test_mode", &self.test_mode)
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
    #[allow(clippy::too_many_arguments)]
    pub fn from_fields_for_test(
        database_url: String,
        admin_username: String,
        admin_password: String,
        jwt_secret: String,
        jwt_expiry: TimeDelta,
        refresh_token_max_lifetime: TimeDelta,
        refresh_inactivity: TimeDelta,
        rate_limit_window: TimeDelta,
        rate_limit_max_attempts: u32,
        rate_limit_block_duration: TimeDelta,
        password_min_length: u32,
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
        assert!(
            refresh_inactivity > TimeDelta::zero() && refresh_inactivity <= TimeDelta::hours(24),
            "from_fields_for_test: refresh_inactivity must be in (0, 24h], got {refresh_inactivity}"
        );
        assert!(
            rate_limit_window > TimeDelta::zero() && rate_limit_window <= TimeDelta::hours(24),
            "from_fields_for_test: rate_limit_window must be in (0, 24h], got {rate_limit_window}"
        );
        assert!(
            (1..=100).contains(&rate_limit_max_attempts),
            "from_fields_for_test: rate_limit_max_attempts must be in [1, 100], got {rate_limit_max_attempts}"
        );
        assert!(
            rate_limit_block_duration > TimeDelta::zero()
                && rate_limit_block_duration <= TimeDelta::hours(24),
            "from_fields_for_test: rate_limit_block_duration must be in (0, 24h], got {rate_limit_block_duration}"
        );
        assert!(
            (8..=128).contains(&password_min_length),
            "from_fields_for_test: password_min_length must be in [8, 128], got {password_min_length}"
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
            refresh_inactivity,
            rate_limit_window,
            rate_limit_max_attempts,
            rate_limit_block_duration,
            password_min_length,
            locale: kesh_i18n::Locale::FrCh,
            test_mode: false,
        }
    }

    /// Retourne une copie avec la locale modifiée (builder pattern pour tests).
    pub fn with_locale(mut self, locale: kesh_i18n::Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Builder non-breaking pour activer `test_mode` sur un Config existant
    /// (utilisé par les tests d'intégration qui montent l'endpoint
    /// `/api/v1/_test/*`). Cf. Story 6.4 AC #6.
    pub fn with_test_mode(mut self, enabled: bool) -> Self {
        self.test_mode = enabled;
        self
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

        // Défaut `127.0.0.1` (sécurité par défaut — Story 6.4 T7.6). Pour
        // un bind public en prod (reverse proxy en front), set explicitement
        // `KESH_HOST=0.0.0.0` dans `.env` ou docker-compose.prod.yml.
        let host = env::var("KESH_HOST").unwrap_or_else(|_| "127.0.0.1".into());

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

        let jwt_secret = env::var("KESH_JWT_SECRET")
            .map_err(|_| ConfigError::MissingVar("KESH_JWT_SECRET".into()))?;

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

        // --- Story 1.6 : session & rate limiting ---

        let refresh_inactivity_minutes = match env::var("KESH_REFRESH_INACTIVITY_MINUTES") {
            Ok(val) => match val.parse::<i64>() {
                Ok(m) if (1..=1440).contains(&m) => m,
                Ok(m) => {
                    tracing::warn!(
                        "KESH_REFRESH_INACTIVITY_MINUTES={} hors borne [1, 1440], utilisation du défaut 15",
                        m
                    );
                    15
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_REFRESH_INACTIVITY_MINUTES='{}' invalide, utilisation du défaut 15",
                        val
                    );
                    15
                }
            },
            Err(_) => 15,
        };
        let refresh_inactivity = TimeDelta::minutes(refresh_inactivity_minutes);

        let rate_limit_window_minutes = match env::var("KESH_RATE_LIMIT_WINDOW_MINUTES") {
            Ok(val) => match val.parse::<i64>() {
                Ok(m) if (1..=1440).contains(&m) => m,
                Ok(m) => {
                    tracing::warn!(
                        "KESH_RATE_LIMIT_WINDOW_MINUTES={} hors borne [1, 1440], utilisation du défaut 15",
                        m
                    );
                    15
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_RATE_LIMIT_WINDOW_MINUTES='{}' invalide, utilisation du défaut 15",
                        val
                    );
                    15
                }
            },
            Err(_) => 15,
        };
        let rate_limit_window = TimeDelta::minutes(rate_limit_window_minutes);

        let rate_limit_max_attempts = match env::var("KESH_RATE_LIMIT_MAX_ATTEMPTS") {
            Ok(val) => match val.parse::<u32>() {
                Ok(n) if (1..=100).contains(&n) => n,
                Ok(n) => {
                    tracing::warn!(
                        "KESH_RATE_LIMIT_MAX_ATTEMPTS={} hors borne [1, 100], utilisation du défaut 5",
                        n
                    );
                    5
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_RATE_LIMIT_MAX_ATTEMPTS='{}' invalide, utilisation du défaut 5",
                        val
                    );
                    5
                }
            },
            Err(_) => 5,
        };

        let rate_limit_block_minutes = match env::var("KESH_RATE_LIMIT_BLOCK_MINUTES") {
            Ok(val) => match val.parse::<i64>() {
                Ok(m) if (1..=1440).contains(&m) => m,
                Ok(m) => {
                    tracing::warn!(
                        "KESH_RATE_LIMIT_BLOCK_MINUTES={} hors borne [1, 1440], utilisation du défaut 30",
                        m
                    );
                    30
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_RATE_LIMIT_BLOCK_MINUTES='{}' invalide, utilisation du défaut 30",
                        val
                    );
                    30
                }
            },
            Err(_) => 30,
        };
        let rate_limit_block_duration = TimeDelta::minutes(rate_limit_block_minutes);

        // --- Story 1.7 : politique de mot de passe ---
        let password_min_length = match env::var("KESH_PASSWORD_MIN_LENGTH") {
            Ok(val) => match val.parse::<u32>() {
                Ok(n) if (8..=128).contains(&n) => n,
                Ok(n) => {
                    tracing::warn!(
                        "KESH_PASSWORD_MIN_LENGTH={} hors borne [8, 128], utilisation du défaut 12",
                        n
                    );
                    12
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_PASSWORD_MIN_LENGTH='{}' invalide, utilisation du défaut 12",
                        val
                    );
                    12
                }
            },
            Err(_) => 12,
        };

        // --- Story 2.1 : internationalisation ---
        let locale_str = env::var("KESH_LANG").unwrap_or_else(|_| "fr".into());
        let locale = kesh_i18n::Locale::from(locale_str.as_str());
        tracing::info!("Locale instance : {}", locale);

        // --- Story 6.4 : mode test (endpoints /api/v1/_test/*) ---
        let test_mode = match env::var("KESH_TEST_MODE") {
            Ok(val) if val == "true" || val == "1" => true,
            Ok(val) if val.is_empty() => false,
            Ok(val) => {
                tracing::warn!(
                    "KESH_TEST_MODE='{}' non reconnu (attendu 'true' ou '1'), désactivé par défaut",
                    val
                );
                false
            }
            Err(_) => false,
        };

        // Garde-fou sécurité (AC #6bis) : refus de démarrage si test_mode
        // actif avec un bind non-loopback. `0.0.0.0` explicitement rejeté.
        if test_mode && !is_loopback_host(&host) {
            return Err(ConfigError::TestModeWithPublicBind { host });
        }

        if test_mode {
            tracing::warn!(
                "KESH_TEST_MODE=true — /api/v1/_test/* sera exposé (DEV/CI ONLY, jamais en prod)"
            );
        }

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
            refresh_inactivity,
            rate_limit_window,
            rate_limit_max_attempts,
            rate_limit_block_duration,
            password_min_length,
            locale,
            test_mode,
        })
    }
}

/// Teste si le host est une adresse loopback stricte.
///
/// Acceptés : `127.0.0.1`, `::1`, `localhost`. **`0.0.0.0` est explicitement
/// rejeté** car en Docker `-p 3000:3000` avec bind interne `0.0.0.0` expose
/// la route au réseau hôte. Cf. décision H1 + N3 review pass 2.
fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
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
            refresh_inactivity: TimeDelta::minutes(15),
            rate_limit_window: TimeDelta::minutes(15),
            rate_limit_max_attempts: 5,
            rate_limit_block_duration: TimeDelta::minutes(30),
            password_min_length: 12,
            locale: kesh_i18n::Locale::FrCh,
            test_mode: false,
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
            env::remove_var("KESH_REFRESH_INACTIVITY_MINUTES");
            env::remove_var("KESH_RATE_LIMIT_WINDOW_MINUTES");
            env::remove_var("KESH_RATE_LIMIT_MAX_ATTEMPTS");
            env::remove_var("KESH_RATE_LIMIT_BLOCK_MINUTES");
            env::remove_var("KESH_PASSWORD_MIN_LENGTH");
            env::remove_var("KESH_TEST_MODE");
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
        // Set explicite pour neutraliser un éventuel `.env` local qui porterait
        // `KESH_HOST=0.0.0.0` (dotenvy charge `.env` avant de lire les vars).
        // Story 6.4 T7.6 : le défaut Rust est désormais `127.0.0.1`.
        unsafe {
            env::set_var("KESH_HOST", "127.0.0.1");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.database_url, "mysql://test:test@localhost:3306/test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.jwt_expiry, TimeDelta::minutes(15));
        assert_eq!(config.refresh_token_max_lifetime, TimeDelta::days(30));
        assert!(!config.test_mode);
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

    // --- Story 1.6 : tests config session & rate limiting ---

    #[test]
    fn config_refresh_inactivity_defaults_to_15_min() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.refresh_inactivity, TimeDelta::minutes(15));
    }

    #[test]
    fn config_refresh_inactivity_respects_env() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_REFRESH_INACTIVITY_MINUTES", "5");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.refresh_inactivity, TimeDelta::minutes(5));
    }

    #[test]
    fn config_refresh_inactivity_out_of_bounds_falls_back() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_REFRESH_INACTIVITY_MINUTES", "9999");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.refresh_inactivity, TimeDelta::minutes(15));
    }

    #[test]
    fn config_rate_limit_defaults() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.rate_limit_window, TimeDelta::minutes(15));
        assert_eq!(config.rate_limit_max_attempts, 5);
        assert_eq!(config.rate_limit_block_duration, TimeDelta::minutes(30));
    }

    #[test]
    fn config_rate_limit_respects_env_overrides() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_RATE_LIMIT_WINDOW_MINUTES", "10");
            env::set_var("KESH_RATE_LIMIT_MAX_ATTEMPTS", "3");
            env::set_var("KESH_RATE_LIMIT_BLOCK_MINUTES", "60");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.rate_limit_window, TimeDelta::minutes(10));
        assert_eq!(config.rate_limit_max_attempts, 3);
        assert_eq!(config.rate_limit_block_duration, TimeDelta::minutes(60));
    }

    #[test]
    fn config_rate_limit_out_of_bounds_falls_back() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_RATE_LIMIT_MAX_ATTEMPTS", "999");
            env::set_var("KESH_RATE_LIMIT_BLOCK_MINUTES", "0");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.rate_limit_max_attempts, 5);
        assert_eq!(config.rate_limit_block_duration, TimeDelta::minutes(30));
    }

    // --- Story 1.7 : password_min_length ---

    #[test]
    fn config_password_min_length_defaults_to_12() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.password_min_length, 12);
    }

    #[test]
    fn config_password_min_length_respects_env() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_PASSWORD_MIN_LENGTH", "20");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.password_min_length, 20);
    }

    #[test]
    fn config_password_min_length_out_of_bounds_falls_back() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_PASSWORD_MIN_LENGTH", "5");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.password_min_length, 12);
    }

    #[test]
    fn config_password_min_length_too_high_falls_back() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_PASSWORD_MIN_LENGTH", "200");
        }

        let config = Config::from_env().expect("Config should load");
        assert_eq!(config.password_min_length, 12);
    }

    // --- Story 6.4 : test_mode + garde-fou bind public ---

    #[test]
    fn config_test_mode_defaults_to_false() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();

        let config = Config::from_env().expect("Config should load");
        assert!(!config.test_mode);
    }

    #[test]
    fn config_test_mode_true_accepted_with_loopback_host() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "true");
            env::set_var("KESH_HOST", "127.0.0.1");
        }

        let config = Config::from_env().expect("Config should load with loopback host");
        assert!(config.test_mode);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn config_test_mode_accepts_one_as_true() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "1");
            // Set explicite pour éviter que `dotenvy::dotenv()` charge un
            // `.env` local avec `KESH_HOST=0.0.0.0` (qui échouerait le
            // garde-fou TestModeWithPublicBind).
            env::set_var("KESH_HOST", "127.0.0.1");
        }

        let config = Config::from_env().expect("Config should load");
        assert!(config.test_mode);
    }

    #[test]
    fn config_test_mode_rejects_public_bind_0_0_0_0() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "true");
            env::set_var("KESH_HOST", "0.0.0.0");
        }

        let result = Config::from_env();
        match result {
            Err(ConfigError::TestModeWithPublicBind { host }) => {
                assert_eq!(host, "0.0.0.0");
            }
            other => panic!("expected TestModeWithPublicBind, got {other:?}"),
        }
    }

    #[test]
    fn config_test_mode_rejects_ip_other_than_loopback() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "true");
            env::set_var("KESH_HOST", "192.168.1.10");
        }

        let result = Config::from_env();
        assert!(
            matches!(result, Err(ConfigError::TestModeWithPublicBind { .. })),
            "public IP must be rejected when test_mode=true, got {result:?}"
        );
    }

    #[test]
    fn config_test_mode_accepts_ipv6_loopback() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "true");
            env::set_var("KESH_HOST", "::1");
        }

        let config = Config::from_env().expect("::1 loopback must be accepted");
        assert!(config.test_mode);
    }

    #[test]
    fn config_test_mode_off_allows_public_bind() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "false");
            env::set_var("KESH_HOST", "0.0.0.0");
        }

        let config = Config::from_env().expect("public bind OK when test_mode off");
        assert!(!config.test_mode);
        assert_eq!(config.host, "0.0.0.0");
    }

}
