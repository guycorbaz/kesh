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
    /// `-p 80:80` avec bind interne `0.0.0.0` expose la route au
    /// réseau hôte (cf. décision pass 2 / N3).
    TestModeWithPublicBind { host: String },
    /// `KESH_TEST_MODE` présent mais pas `"true"` / `"1"` / vide (code
    /// review P7). Au lieu du warn + défaut `false` silencieux, on refuse
    /// explicitement pour éviter qu'un opérateur ayant tapé `"True"` ou
    /// `"yes"` croie test_mode activé alors que l'endpoint est 404.
    InvalidTestModeValue { got: String },
    /// `KESH_JWT_SECRET` contient la sous-chaîne `change-me` (placeholder
    /// du `.env.example`). Story 10-1 : refus fail-fast au lieu du warn
    /// existant pour empêcher un boot prod sur un secret non-changé.
    /// **Display ne loggue jamais la valeur** — fuite via `tracing::error!`.
    InsecureJwtSecret,
    /// `KESH_ADMIN_PASSWORD` égale (case-insensitive après trim) le
    /// placeholder `"changeme"` du `.env.example`. Story 10-1 : refus
    /// fail-fast au lieu du warn existant. **Display ne loggue jamais
    /// la valeur**.
    InsecureAdminPassword,
    /// `KESH_ADMIN_PASSWORD` trop court (< 12 caractères après trim).
    /// Story 10-1 : minimum OWASP / NIST SP 800-63B. **Display loggue
    /// uniquement le count, jamais la valeur**.
    WeakAdminPassword { actual_chars: usize },
    /// v0.1.3 hotfix (Issue #136) — `KESH_COOKIE_SECURE` présent mais pas
    /// `"true"` / `"1"` / `"false"` / `"0"`. Pattern strict identique à
    /// `KESH_TEST_MODE` pour éviter ambiguïté (`"True"`, `"yes"`, `"on"`...).
    InvalidCookieSecureValue { got: String },
    /// Story 17-4b — var booléenne stricte (`KESH_SMTP_TLS`,
    /// `KESH_FEATURE_FORGOT_PASSWORD`) présente mais pas
    /// `"true"`/`"1"`/`"false"`/`"0"`/vide. Pattern strict identique à
    /// `KESH_COOKIE_SECURE` — refus explicite pour éviter qu'un `"True"`/`"yes"`
    /// soit silencieusement interprété comme `false`.
    InvalidBoolValue { var: String, got: String },
    /// Story 17-4b (DC7) — `KESH_FEATURE_FORGOT_PASSWORD=true` mais la config
    /// SMTP/recovery est incomplète : une ou plusieurs vars requises
    /// (`KESH_SMTP_HOST`, `KESH_SMTP_USER`, `KESH_SMTP_PASSWORD`,
    /// `KESH_SMTP_FROM`, `KESH_PUBLIC_BASE_URL`) sont absentes/vides, **ou**
    /// `KESH_SMTP_FROM` n'est pas un email valide. Fail-fast au boot : si le
    /// recovery par email est activé, il DOIT être pleinement configuré (sinon
    /// l'instance démarrerait avec un recovery silencieusement cassé). `detail`
    /// liste ce qui manque — **ne loggue jamais le mot de passe SMTP**.
    IncompleteSmtpConfig { detail: String },
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
            ConfigError::InvalidTestModeValue { got } => {
                write!(
                    f,
                    "KESH_TEST_MODE='{}' non reconnu — valeurs acceptées : \
                     'true', '1' (actif), ou vide/absente (inactif). \
                     'True', 'TRUE', 'yes', 'on', etc. sont explicitement refusés \
                     pour éviter les ambiguïtés de configuration.",
                    got
                )
            }
            ConfigError::InsecureJwtSecret => {
                write!(
                    f,
                    "KESH_JWT_SECRET contient le placeholder 'change-me'. \
                     Générer un vrai secret via : openssl rand -hex 32"
                )
            }
            ConfigError::InsecureAdminPassword => {
                write!(
                    f,
                    "KESH_ADMIN_PASSWORD est la valeur par défaut 'changeme' \
                     (case-insensitive). Changez-le avant la mise en production."
                )
            }
            ConfigError::WeakAdminPassword { actual_chars } => {
                write!(
                    f,
                    "KESH_ADMIN_PASSWORD trop court : {} chars, minimum 12.",
                    actual_chars
                )
            }
            ConfigError::InvalidCookieSecureValue { got } => {
                write!(
                    f,
                    "KESH_COOKIE_SECURE='{}' non reconnu — valeurs acceptées : \
                     'true'/'1' (cookies Secure activés, HTTPS requis client-side — défaut) \
                     ou 'false'/'0' (cookies sans Secure, HTTP-only LAN strict — \
                     ⚠️ cookies sniffables, ne jamais activer en prod Internet sans HTTPS).",
                    got
                )
            }
            ConfigError::InvalidBoolValue { var, got } => {
                write!(
                    f,
                    "{}='{}' non reconnu — valeurs acceptées : 'true'/'1' (actif), \
                     'false'/'0' (inactif), ou vide/absente (défaut). 'True', 'yes', \
                     'on', etc. sont refusés pour éviter les ambiguïtés.",
                    var, got
                )
            }
            ConfigError::IncompleteSmtpConfig { detail } => {
                write!(
                    f,
                    "KESH_FEATURE_FORGOT_PASSWORD=true mais la configuration SMTP/recovery \
                     est incomplète : {}. Renseigner toutes les vars (KESH_SMTP_HOST, \
                     KESH_SMTP_USER, KESH_SMTP_PASSWORD, KESH_SMTP_FROM, KESH_PUBLIC_BASE_URL) \
                     ou désactiver le recovery par email (KESH_FEATURE_FORGOT_PASSWORD=false ; \
                     le fallback break-glass KESH_ADMIN_RESET reste disponible).",
                    detail
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
    /// Story v011-5 : double-usage `KESH_ADMIN_USERNAME` (bootstrap déclaratif
    /// si DB vide, OU recovery break-glass si admin existe avec ce username +
    /// hash diff). **Optionnel** — var absente ou vide après trim → `None`.
    /// **Invariant garanti par `from_env`** : `Some(s) ⟹ !s.is_empty()`.
    pub admin_username: Option<String>,
    /// Story v011-5 : voir `admin_username`. **Optionnel** — var absente ou
    /// vide après trim → `None`. **Invariant garanti par `from_env`** :
    /// `Some(p) ⟹ !p.is_empty()`. Les validations sécu
    /// (`InsecureAdminPassword` "changeme", `WeakAdminPassword` < 12 chars)
    /// s'appliquent uniquement si `Some(non-empty)`.
    pub admin_password: Option<String>,
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

    // --- Story 8-1b : import bancaire CAMT.053 ---
    /// Taille max d'un fichier upload bank-import en **MiB binaires**
    /// (1 MiB = 1024² bytes). Appliqué via `axum::extract::DefaultBodyLimit::max`
    /// sur le sub-router bank-import (T6.10 + M3 validate Pass 2).
    /// Lu depuis `KESH_BANK_IMPORT_MAX_MB` ; défaut 10, borne [1, 100]
    /// (O4 validate Pass 3 — borne haute évite OOM côté `axum::body::Bytes`).
    pub bank_import_max_mib: u32,

    /// Story 17-3a (DC8) — plafond d'assemblage **in-memory** du `.keshbackup`
    /// (export complet d'installation). Au-delà, la réponse est spillée vers un
    /// fichier temporaire et streamée. Lu depuis `KESH_ADMIN_EXPORT_INMEM_MB` ;
    /// défaut 50, borne [1, 2048] (borne haute évite l'OOM sur grosses installs).
    pub admin_export_inmem_mib: u32,

    /// Story 17-3c (DC5) — taille max d'un upload `.keshbackup` à l'**import**,
    /// en MiB binaires. Appliqué via `axum::extract::DefaultBodyLimit::max` sur
    /// la route `POST /api/v1/admin/full-import`. Lu depuis
    /// `KESH_ADMIN_IMPORT_MAX_MB` ; défaut 512, borne [1, 10240].
    pub admin_import_max_mib: u32,

    /// Story 17-3c (DC5) — répertoire où écrire le **backup automatique
    /// pré-import** (`.keshbackup` de l'état courant, filet de sécurité avant
    /// le restore destructeur). Lu depuis `KESH_ADMIN_BACKUP_DIR` ; défaut
    /// `/tmp`. Créé s'il n'existe pas. La purge est à la charge de l'opérateur.
    pub admin_backup_dir: String,

    // --- v0.1.3 hotfix (Issue #136) — cookies Secure flag override ---
    /// Émettre les cookies session (`kesh_access_token`, `kesh_refresh_token`)
    /// avec le flag `Secure` (requiert HTTPS côté client). **Défaut `true`**
    /// (sécurité maximale). Lu depuis `KESH_COOKIE_SECURE` (`"true"` / `"1"`
    /// → `true` ; `"false"` / `"0"` → `false`). Toute autre valeur fail-fast
    /// `ConfigError::InvalidCookieSecureValue`.
    ///
    /// **⚠️ Sécurité** : passer à `false` désactive la protection navigateur
    /// qui force HTTPS pour transmettre le cookie. Les cookies session
    /// deviennent sniffables sur tout réseau HTTP non-TLS (LAN privé compris).
    /// **Utiliser uniquement** pour les déploiements LAN strict HTTP-only
    /// (Docker dev local, NAS LAN privé sans reverse proxy HTTPS).
    /// **NE JAMAIS** activer en production exposée Internet sans HTTPS.
    ///
    /// L'alternative recommandée est de mettre HTTPS via reverse proxy
    /// (Traefik + Let's Encrypt, Caddy, nginx, Synology DSM intégré, ...).
    pub cookie_secure: bool,

    // --- Story 17-4b : recovery de mot de passe par email (SMTP) ---
    /// `KESH_SMTP_HOST` — hôte du serveur SMTP. Optionnel (None si recovery
    /// désactivé). **Invariant `from_env`** : `Some(s) ⟹ !s.is_empty()` (trim).
    pub smtp_host: Option<String>,
    /// `KESH_SMTP_PORT` — port SMTP. Défaut **587** (STARTTLS submission).
    /// Borne [1, 65535] (= `u16`, toute valeur non-nulle est valide).
    pub smtp_port: u16,
    /// `KESH_SMTP_USER` — utilisateur d'authentification SMTP. Optionnel.
    /// **Invariant `from_env`** : `Some(s) ⟹ !s.is_empty()`.
    pub smtp_user: Option<String>,
    /// `KESH_SMTP_PASSWORD` — secret d'authentification SMTP. **Champ privé,
    /// masqué dans `Debug`** (calque `jwt_secret`). Accès via
    /// [`Config::smtp_password`]. **Invariant `from_env`** : `Some ⟹ non vide`.
    smtp_password: Option<String>,
    /// `KESH_SMTP_FROM` — adresse expéditrice des emails. Optionnel ; si
    /// présent, validé comme email (`is_valid_email_simple`) au boot.
    pub smtp_from: Option<String>,
    /// `KESH_SMTP_TLS` — STARTTLS sur la connexion SMTP. **Défaut `true`**.
    /// Le `reset_url` contient le token brut → l'email transite en clair si
    /// SMTP non-TLS (la doc 17-4f recommande de garder `true`).
    pub smtp_tls: bool,
    /// `KESH_PUBLIC_BASE_URL` — base publique de l'URL du lien de reset
    /// (`{base}/reset-password?token=...`), consommée par 17-4c. Optionnel.
    /// **Invariant `from_env`** : `Some(s) ⟹ !s.is_empty()` (trim).
    pub public_base_url: Option<String>,
    /// `KESH_FEATURE_FORGOT_PASSWORD` — active le recovery self-service par
    /// email (DC7). **Défaut `false`**. Si `true`, la config SMTP complète est
    /// requise au boot (fail-fast [`ConfigError::IncompleteSmtpConfig`]). Si
    /// `false`, recovery = break-glass `KESH_ADMIN_RESET` (#121).
    pub forgot_password_enabled: bool,
}

// Debug personnalisé : masquer les secrets pour éviter toute fuite via logs
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"***")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("admin_username", &self.admin_username)
            .field(
                "admin_password",
                &self.admin_password.as_ref().map(|_| "***"),
            )
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
            .field("cookie_secure", &self.cookie_secure)
            .field("smtp_host", &self.smtp_host)
            .field("smtp_port", &self.smtp_port)
            .field("smtp_user", &self.smtp_user)
            .field("smtp_password", &self.smtp_password.as_ref().map(|_| "***"))
            .field("smtp_from", &self.smtp_from)
            .field("smtp_tls", &self.smtp_tls)
            .field("public_base_url", &self.public_base_url)
            .field("forgot_password_enabled", &self.forgot_password_enabled)
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

    /// Accès au secret SMTP (Story 17-4b), point d'exposition unique du champ
    /// privé `smtp_password`. Utilisé par `SmtpMailer` ; ne jamais exposer ni
    /// logger directement. `None` si `KESH_SMTP_PASSWORD` non configuré.
    pub fn smtp_password(&self) -> Option<&str> {
        self.smtp_password.as_deref()
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

        // Story v011-5 : `Config::admin_username`/`admin_password` sont
        // `Option<String>`. Le constructeur test wrap `Some(...)` après
        // l'assertion non-vide (cf. l.257) — la signature reste `String, String`
        // pour éviter la migration des 30 sites de tests d'intégration.
        Config {
            database_url,
            port: 80,
            host: "127.0.0.1".to_string(),
            admin_username: Some(admin_username),
            admin_password: Some(admin_password),
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
            bank_import_max_mib: 10,
            admin_export_inmem_mib: 50,
            admin_import_max_mib: 512,
            admin_backup_dir: "/tmp".to_string(),
            cookie_secure: true,
            // Story 17-4b : recovery email désactivé par défaut en test (les
            // tests qui l'exercent muteront le champ ou utiliseront un Config
            // dédié). Défauts dans le corps → signature `from_fields_for_test`
            // inchangée, call-sites de test intacts.
            smtp_host: None,
            smtp_port: 587,
            smtp_user: None,
            smtp_password: None,
            smtp_from: None,
            smtp_tls: true,
            public_base_url: None,
            forgot_password_enabled: false,
        }
    }

    /// Retourne une copie avec la locale modifiée (builder pattern pour tests).
    pub fn with_locale(mut self, locale: kesh_i18n::Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Builder **test-only** (cf. `#[doc(hidden)]`) pour activer
    /// `test_mode` sur un Config existant — utilisé par les tests
    /// d'intégration qui montent l'endpoint `/api/v1/_test/*`. Cf. Story
    /// 6.4 AC #6.
    ///
    /// **Panique** (code review P6) si `enabled == true` **et** `host`
    /// n'est pas loopback — miroir du garde-fou runtime `from_env`.
    /// Empêche qu'un caller de `from_fields_for_test` muté avec
    /// `host = "0.0.0.0"` active silencieusement `test_mode` en bypassant
    /// le check. La panic est intentionnelle et acceptable ici : le
    /// builder est `#[doc(hidden)]`, ne doit jamais être appelé depuis
    /// un chemin de production.
    #[doc(hidden)]
    pub fn with_test_mode(mut self, enabled: bool) -> Self {
        if enabled && !is_loopback_host(&self.host) {
            panic!(
                "with_test_mode(true) exige un bind loopback — host actuel: '{}' (interdit, cf. ConfigError::TestModeWithPublicBind)",
                self.host
            );
        }
        self.test_mode = enabled;
        self
    }

    /// Charge la configuration depuis les variables d'environnement.
    ///
    /// **Note isolation tests (Story 9-5-2)** : `dotenvy::dotenv()` n'est chargé
    /// qu'en mode non-test (`#[cfg(not(test))]`). En mode test, le module
    /// `#[cfg(test)] mod tests` ci-dessous gère l'environnement explicitement
    /// via `reset_env()` + `env::set_var(...)` per test ; recharger `.env`
    /// projet (qui peut contenir `KESH_HOST=0.0.0.0` + `KESH_TEST_MODE=true`)
    /// annulerait `reset_env()` et provoquerait des fails déterministes sur
    /// `ConfigError::TestModeWithPublicBind` (20/34 tests fail observé avant
    /// ce patch). Aucun test ne valide le comportement « lit `.env` » — tous
    /// set leurs vars explicitement.
    pub fn from_env() -> Result<Self, ConfigError> {
        #[cfg(not(test))]
        dotenvy::dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::MissingVar("DATABASE_URL".into()))?;

        let port = match env::var("KESH_PORT") {
            Ok(val) => match val.parse::<u16>() {
                Ok(0) => {
                    tracing::warn!("KESH_PORT=0 invalide, utilisation du port par défaut 80");
                    80
                }
                Ok(p) => p,
                Err(_) => {
                    tracing::warn!(
                        "KESH_PORT='{}' n'est pas un numéro de port valide, utilisation du port par défaut 80",
                        val
                    );
                    80
                }
            },
            Err(_) => 80,
        };

        // Défaut `127.0.0.1` (sécurité par défaut — Story 6.4 T7.6). Pour
        // un bind public en prod (reverse proxy en front), set explicitement
        // `KESH_HOST=0.0.0.0` dans `.env` ou docker-compose.prod.yml.
        let host = env::var("KESH_HOST").unwrap_or_else(|_| "127.0.0.1".into());

        // Story v011-5 — admin vars OPTIONNELLES (refactor AC #0).
        //
        // Comportement :
        // - Var absente OU vide après trim → `None`.
        // - Var renseignée non-vide → `Some(s)`, validée sécu.
        //
        // Rationale : un opérateur qui set explicitement les vars s'engage à
        // respecter la politique sécu (longueur, pas "changeme"). Un opérateur
        // qui omet les vars délègue au flow self-service `/setup` (route web
        // au 1er boot sur DB vide). Cf. AC #0 / story v011-5.
        //
        // Préserve le trim (patch V1 — admin inloggable si username/password
        // trimmé côté client mais stocké brut côté DB).
        let admin_username: Option<String> = match env::var("KESH_ADMIN_USERNAME") {
            Ok(v) => {
                let trimmed = v.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }
            Err(_) => None,
        };

        let admin_password: Option<String> = match env::var("KESH_ADMIN_PASSWORD") {
            Ok(v) => {
                let trimmed = v.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }
            Err(_) => None,
        };

        // Validations sécu UNIQUEMENT si Some(non-empty) (l'invariant
        // « Some ⟹ !is_empty() » est garanti par le branchement ci-dessus).
        if let Some(ref p) = admin_password {
            // Story 10-1 T1.3 : refus fail-fast case-insensitive de "changeme"
            // (placeholder `.env.example`) — sécurité fail-fast.
            if p.eq_ignore_ascii_case("changeme") {
                return Err(ConfigError::InsecureAdminPassword);
            }

            // Story 10-1 T1.4 : minimum 12 chars (OWASP / NIST SP 800-63B).
            // `.chars().count()` (pas `.len()`) pour Unicode correct.
            let admin_password_len = p.chars().count();
            if admin_password_len < 12 {
                return Err(ConfigError::WeakAdminPassword {
                    actual_chars: admin_password_len,
                });
            }
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

        // Story 10-1 T1.5 : refus fail-fast si le secret contient le
        // placeholder `change-me` — au lieu du warn existant.
        // Code review Pass 1 ECH-4 : check **case-insensitive** pour cohérence
        // avec l'admin password check (`eq_ignore_ascii_case` ligne 408). Sans
        // `to_ascii_lowercase()`, un secret `"CHANGE-ME-32bytes-..."` (majuscules)
        // passe le check alors qu'il signale clairement un placeholder.
        if jwt_secret.to_ascii_lowercase().contains("change-me") {
            return Err(ConfigError::InsecureJwtSecret);
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
        // Parsing strict (code review P7) : seules `"true"` et `"1"`
        // sont acceptées comme vérité. Toute autre valeur non vide
        // (ex: `"True"`, `"yes"`, `" true"`) est rejetée avec une
        // erreur explicite — évite qu'un opérateur croie test_mode
        // actif alors que l'endpoint est 404.
        let test_mode = match env::var("KESH_TEST_MODE") {
            Ok(val) if val == "true" || val == "1" => true,
            Ok(val) if val.is_empty() => false,
            Ok(val) => return Err(ConfigError::InvalidTestModeValue { got: val }),
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

        // KESH_BANK_IMPORT_MAX_MB : optionnel, défaut 10, borne [1, 100]
        // (Story 8-1b T6.10 + O4 validate Pass 3). Interprétation MiB
        // binaire (1 MiB = 1024² bytes), voir M3 validate Pass 2.
        let bank_import_max_mib = match env::var("KESH_BANK_IMPORT_MAX_MB") {
            Ok(val) => match val.parse::<u32>() {
                Ok(m) if (1..=100).contains(&m) => m,
                Ok(m) => {
                    tracing::warn!(
                        "KESH_BANK_IMPORT_MAX_MB={} hors borne [1, 100], utilisation du défaut 10",
                        m
                    );
                    10
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_BANK_IMPORT_MAX_MB='{}' invalide, utilisation du défaut 10",
                        val
                    );
                    10
                }
            },
            Err(_) => 10,
        };

        // Story 17-3a (DC8) — KESH_ADMIN_EXPORT_INMEM_MB : optionnel, défaut 50,
        // borne [1, 2048]. Au-delà du plafond, l'export `.keshbackup` spille sur
        // fichier temporaire + streaming. Log WARN si > 500 (RAM à surveiller).
        let admin_export_inmem_mib = match env::var("KESH_ADMIN_EXPORT_INMEM_MB") {
            Ok(val) => match val.parse::<u32>() {
                Ok(m) if (1..=2048).contains(&m) => {
                    if m > 500 {
                        tracing::warn!(
                            "KESH_ADMIN_EXPORT_INMEM_MB={} > 500 MiB — assurez-vous d'avoir la RAM suffisante",
                            m
                        );
                    }
                    m
                }
                Ok(m) => {
                    tracing::warn!(
                        "KESH_ADMIN_EXPORT_INMEM_MB={} hors borne [1, 2048], utilisation du défaut 50",
                        m
                    );
                    50
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_ADMIN_EXPORT_INMEM_MB='{}' invalide, utilisation du défaut 50",
                        val
                    );
                    50
                }
            },
            Err(_) => 50,
        };

        // Story 17-3c (DC5) — KESH_ADMIN_IMPORT_MAX_MB : optionnel, défaut 512,
        // borne [1, 10240]. Plafond de l'upload `.keshbackup` à l'import
        // (DefaultBodyLimit). Pattern parse+borne+warn identique à bank-import.
        let admin_import_max_mib = match env::var("KESH_ADMIN_IMPORT_MAX_MB") {
            Ok(val) => match val.parse::<u32>() {
                Ok(m) if (1..=10240).contains(&m) => m,
                Ok(m) => {
                    tracing::warn!(
                        "KESH_ADMIN_IMPORT_MAX_MB={} hors borne [1, 10240], utilisation du défaut 512",
                        m
                    );
                    512
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_ADMIN_IMPORT_MAX_MB='{}' invalide, utilisation du défaut 512",
                        val
                    );
                    512
                }
            },
            Err(_) => 512,
        };

        // Story 17-3c (DC5) — KESH_ADMIN_BACKUP_DIR : répertoire du backup
        // automatique pré-import. Défaut `/tmp`. Créé si absent (au moment de
        // l'import, pas au boot). Pas de borne (chemin arbitraire opérateur).
        let admin_backup_dir =
            env::var("KESH_ADMIN_BACKUP_DIR").unwrap_or_else(|_| "/tmp".to_string());

        // v0.1.3 hotfix (Issue #136) — KESH_COOKIE_SECURE override.
        // Parsing strict (cohérent KESH_TEST_MODE P7) : seules `"true"`/`"1"`/
        // `"false"`/`"0"` acceptées. Toute autre valeur (`"True"`, `"yes"`,
        // `" true"`...) → fail-fast pour éviter qu'un opérateur croie avoir
        // désactivé Secure alors qu'il reste actif (= cookies sniffables
        // silencieusement OU UX broken silencieusement).
        let cookie_secure = match env::var("KESH_COOKIE_SECURE") {
            Ok(val) if val == "true" || val == "1" => true,
            Ok(val) if val == "false" || val == "0" => false,
            Ok(val) if val.is_empty() => true,
            Ok(val) => return Err(ConfigError::InvalidCookieSecureValue { got: val }),
            Err(_) => true,
        };

        // Warning explicite au boot si Secure désactivé — l'opérateur doit voir
        // que les cookies session sont sniffables sur le LAN. Ne pas afficher
        // si Secure actif (régime nominal).
        if !cookie_secure {
            tracing::warn!(
                "KESH_COOKIE_SECURE=false — cookies session émis SANS le flag `Secure`. \
                 Acceptable uniquement pour déploiement HTTP-only LAN strict (e.g. domaine \
                 RFC 8375 `*.home.arpa`, NAS privé). NE JAMAIS activer en prod exposée \
                 Internet sans HTTPS — cookies sniffables sur tout réseau non-TLS."
            );
        }

        // Story 17-4b (DC7) — Recovery de mot de passe par email (SMTP).
        // Vars optionnelles tant que le feature est désactivé. Pattern
        // opt-string trim+filter (cf. KESH_ADMIN_USERNAME) pour les strings.
        let smtp_host = opt_trimmed_env("KESH_SMTP_HOST");
        let smtp_user = opt_trimmed_env("KESH_SMTP_USER");
        let smtp_password = opt_trimmed_env("KESH_SMTP_PASSWORD");
        let smtp_from = opt_trimmed_env("KESH_SMTP_FROM");
        // Review 17-4b Pass 1 (P4-4 umbrella) — strip du slash final pour éviter
        // le double-slash `{base}//reset-password` côté 17-4c. Un base-url réduit
        // à "/" (ou "///") devient vide → None (re-filter après strip).
        let public_base_url = opt_trimmed_env("KESH_PUBLIC_BASE_URL")
            .map(|s| s.trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty());

        // KESH_SMTP_PORT : int borné [1, 65535] (= u16), défaut 587 (STARTTLS
        // submission). Pattern parse+borne+warn (cf. KESH_ADMIN_EXPORT_INMEM_MB).
        let smtp_port: u16 = match env::var("KESH_SMTP_PORT") {
            Ok(val) => match val.parse::<u16>() {
                Ok(p) if p >= 1 => p,
                Ok(_) => {
                    tracing::warn!("KESH_SMTP_PORT=0 invalide, utilisation du défaut 587");
                    587
                }
                Err(_) => {
                    tracing::warn!(
                        "KESH_SMTP_PORT='{}' invalide, utilisation du défaut 587",
                        val
                    );
                    587
                }
            },
            Err(_) => 587,
        };

        // KESH_SMTP_TLS : strict bool, défaut true (STARTTLS).
        let smtp_tls = parse_strict_bool("KESH_SMTP_TLS", true)?;
        // KESH_FEATURE_FORGOT_PASSWORD : strict bool, défaut false (DC7).
        let forgot_password_enabled = parse_strict_bool("KESH_FEATURE_FORGOT_PASSWORD", false)?;

        // Fail-fast boot (DC7) : si le recovery par email est activé, la config
        // SMTP/recovery DOIT être complète. Sinon l'instance démarrerait avec un
        // recovery silencieusement cassé (oracle d'énumération côté 17-4c). Le
        // port a un défaut (toujours présent) ; on valide les 5 autres vars + le
        // format de smtp_from. NE LOGGUE JAMAIS le mot de passe SMTP.
        if forgot_password_enabled {
            let mut missing: Vec<&str> = Vec::new();
            if smtp_host.is_none() {
                missing.push("KESH_SMTP_HOST");
            }
            if smtp_user.is_none() {
                missing.push("KESH_SMTP_USER");
            }
            if smtp_password.is_none() {
                missing.push("KESH_SMTP_PASSWORD");
            }
            if smtp_from.is_none() {
                missing.push("KESH_SMTP_FROM");
            }
            if public_base_url.is_none() {
                missing.push("KESH_PUBLIC_BASE_URL");
            }
            if !missing.is_empty() {
                return Err(ConfigError::IncompleteSmtpConfig {
                    detail: format!("vars absentes/vides : {}", missing.join(", ")),
                });
            }
            // smtp_from présent ici (sinon déjà signalé manquant) → valider format.
            if let Some(ref from) = smtp_from {
                if !crate::routes::contacts::is_valid_email_simple(from) {
                    return Err(ConfigError::IncompleteSmtpConfig {
                        detail: "KESH_SMTP_FROM n'est pas un email valide (format attendu : \
                                 email@domaine.tld, sans display-name « Nom <email> »)"
                            .to_string(),
                    });
                }
            }
            // Review 17-4b Pass 1 — un `:` dans le host trahit un format
            // `host:port` copié d'une doc SMTP : le SNI rustls le refuserait
            // seulement au premier envoi (« domain isn't a valid DNS name »),
            // erreur cryptique non détectée au boot. Exception : IPv6 literal
            // (contient des `:` légitimes, utilisable en SMTP plaintext LAN).
            if let Some(ref host) = smtp_host {
                if host.contains(':') && host.parse::<std::net::Ipv6Addr>().is_err() {
                    return Err(ConfigError::IncompleteSmtpConfig {
                        detail: format!(
                            "KESH_SMTP_HOST='{}' contient un port — renseigner uniquement \
                             le nom d'hôte (le port va dans KESH_SMTP_PORT)",
                            host
                        ),
                    });
                }
            }
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
            bank_import_max_mib,
            admin_export_inmem_mib,
            admin_import_max_mib,
            admin_backup_dir,
            cookie_secure,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
            smtp_from,
            smtp_tls,
            public_base_url,
            forgot_password_enabled,
        })
    }
}

// =============================================================================
// Story v011-1 — Configuration des logs fichier (tracing-appender)
// =============================================================================
//
// `LogConfig` est volontairement **séparé** de `Config` : il est lu **avant**
// l'initialisation du subscriber tracing (cf. `main.rs`), alors que `Config`
// est validé après. Son parsing est **infaillible** (jamais de panic ni
// d'erreur fatale — tout est fallback) car une mauvaise valeur de log ne doit
// jamais empêcher le serveur de démarrer. Les messages de fallback sont
// **collectés** dans un `Vec<String>` retourné au caller, qui les rejoue via
// `tracing::warn!` une fois le subscriber installé (émettre `tracing::*` avant
// l'init les perdrait silencieusement).

/// Stratégie de rotation des fichiers de log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogRotation {
    Daily,
    Hourly,
    Never,
}

impl LogRotation {
    /// Parse case-insensitive avec fallback `Daily` sur valeur inconnue.
    fn parse(raw: &str, warnings: &mut Vec<String>) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "daily" => Self::Daily,
            "hourly" => Self::Hourly,
            "never" => Self::Never,
            other => {
                warnings.push(format!(
                    "KESH_LOG_FILE_ROTATION='{other}' invalide (attendu daily/hourly/never), fallback 'daily'"
                ));
                Self::Daily
            }
        }
    }
}

/// Format de sérialisation des entrées de log fichier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

impl LogFormat {
    /// Parse case-insensitive avec fallback `Pretty` sur valeur inconnue.
    fn parse(raw: &str, warnings: &mut Vec<String>) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "pretty" => Self::Pretty,
            "json" => Self::Json,
            other => {
                warnings.push(format!(
                    "KESH_LOG_FILE_FORMAT='{other}' invalide (attendu pretty/json), fallback 'pretty'"
                ));
                Self::Pretty
            }
        }
    }
}

/// Nombre de fichiers de log conservés par défaut (rotation).
const DEFAULT_LOG_MAX_FILES: usize = 7;

fn parse_max_files(raw: &str, warnings: &mut Vec<String>) -> usize {
    match raw.trim().parse::<usize>() {
        Ok(0) => {
            warnings.push(format!(
                "KESH_LOG_FILE_MAX_FILES=0 invalide, fallback {DEFAULT_LOG_MAX_FILES}"
            ));
            DEFAULT_LOG_MAX_FILES
        }
        Ok(n) => n,
        Err(_) => {
            warnings.push(format!(
                "KESH_LOG_FILE_MAX_FILES='{}' invalide, fallback {DEFAULT_LOG_MAX_FILES}",
                raw.trim()
            ));
            DEFAULT_LOG_MAX_FILES
        }
    }
}

/// Configuration des logs fichier (Story v011-1, Issue #119).
///
/// - `file_path == None` → logs fichier désactivés (stdout-only, comportement
///   identique à v0.1.0). Rétro-compatibilité totale.
/// - Toutes les valeurs invalides retombent sur un défaut sain avec un warning
///   collecté (cf. doc du module).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogConfig {
    /// Chemin complet du fichier de log (ex. `/var/log/kesh/kesh.log`).
    /// `None` désactive la sortie fichier.
    pub file_path: Option<String>,
    pub rotation: LogRotation,
    pub max_files: usize,
    pub format: LogFormat,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            file_path: None,
            rotation: LogRotation::Daily,
            max_files: DEFAULT_LOG_MAX_FILES,
            format: LogFormat::Pretty,
        }
    }
}

impl LogConfig {
    /// Parsing pur depuis des valeurs déjà extraites de l'environnement.
    /// Testable sans toucher l'état global du process.
    fn from_raw(
        file_path: Option<String>,
        rotation: Option<String>,
        max_files: Option<String>,
        format: Option<String>,
    ) -> (LogConfig, Vec<String>) {
        let mut warnings = Vec::new();
        let file_path = file_path
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        let rotation = rotation
            .map(|v| LogRotation::parse(&v, &mut warnings))
            .unwrap_or(LogRotation::Daily);
        let max_files = max_files
            .map(|v| parse_max_files(&v, &mut warnings))
            .unwrap_or(DEFAULT_LOG_MAX_FILES);
        let format = format
            .map(|v| LogFormat::parse(&v, &mut warnings))
            .unwrap_or(LogFormat::Pretty);
        (
            LogConfig {
                file_path,
                rotation,
                max_files,
                format,
            },
            warnings,
        )
    }

    /// Charge la config logging depuis l'environnement. **Infaillible** :
    /// retourne toujours une `LogConfig` valide + la liste des warnings de
    /// fallback à rejouer après l'init du subscriber.
    ///
    /// Précondition : `dotenvy::dotenv()` doit avoir été appelé en amont par
    /// `main.rs` pour que les vars d'un fichier `.env` soient visibles (cet
    /// appel précède `Config::from_env()` qui charge `.env` de son côté).
    pub fn from_env() -> (LogConfig, Vec<String>) {
        Self::from_raw(
            env::var("KESH_LOG_FILE_PATH").ok(),
            env::var("KESH_LOG_FILE_ROTATION").ok(),
            env::var("KESH_LOG_FILE_MAX_FILES").ok(),
            env::var("KESH_LOG_FILE_FORMAT").ok(),
        )
    }
}

/// Teste si le host est une adresse loopback stricte.
///
/// Accepte :
///
/// - `localhost` / `Localhost` / `LOCALHOST` / `localhost.` (case-insensitive + trailing dot FQDN, RFC 1035 hostname matching — code review pass 2 E5 pour compat Windows où `.env` et shell traitent les hostnames case-insensitive).
/// - Toute adresse qui parse via `IpAddr` et dont `is_loopback()` est vrai (127.0.0.0/8, ::1). Supporte les IPv6 bracketés (`[::1]`) et les zone IDs (`::1%eth0`).
///
/// **`0.0.0.0` est explicitement rejeté** car en Docker `-p 80:80`
/// avec bind interne `0.0.0.0` expose la route au réseau hôte.
/// `0.0.0.0` ne passe pas `is_loopback()` (c'est `is_unspecified()`).
///
/// **Note sur `localhost`** (code review P4) : accepter la chaîne littérale
/// `localhost` reste pragmatique pour les devs, mais une configuration
/// DNS / `/etc/hosts` compromise pourrait résoudre `localhost` vers une
/// IP non-loopback. Le risque est atténué par le fait que `KESH_TEST_MODE`
/// est gated par env-var (opt-in explicite).
/// Story 17-4b — lit une var d'env optionnelle, trim, et filtre la chaîne vide
/// → `None`. Pattern partagé des vars optionnelles (cf. `KESH_ADMIN_USERNAME`).
/// **Invariant garanti** : `Some(s) ⟹ !s.is_empty()`.
fn opt_trimmed_env(var: &str) -> Option<String> {
    match env::var(var) {
        Ok(v) => {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        // Review 17-4b Pass 1 — distinguer NotUnicode de NotPresent : une var
        // présente mais non-UTF-8 traitée silencieusement comme absente
        // produirait un message fail-fast trompeur (« var absente/vide »).
        Err(env::VarError::NotUnicode(_)) => {
            tracing::warn!(
                "{} contient des octets non-UTF-8 — ignorée (traitée comme absente)",
                var
            );
            None
        }
        Err(env::VarError::NotPresent) => None,
    }
}

/// Story 17-4b — parse une var booléenne **stricte** (pattern `KESH_COOKIE_SECURE`).
/// `"true"`/`"1"` → `true` ; `"false"`/`"0"` → `false` ; vide/absente → `default` ;
/// toute autre valeur → `ConfigError::InvalidBoolValue` (refus fail-fast, évite
/// qu'un `"True"`/`"yes"` soit silencieusement interprété comme `false`).
fn parse_strict_bool(var: &str, default: bool) -> Result<bool, ConfigError> {
    match env::var(var) {
        // Review 17-4b Pass 1 — trim avant comparaison, cohérent avec
        // `opt_trimmed_env` : un espace invisible dans un `.env` édité à la
        // main (`KESH_SMTP_TLS=true `) ne doit pas échouer le boot.
        Ok(raw) => match raw.trim() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            "" => Ok(default),
            other => Err(ConfigError::InvalidBoolValue {
                var: var.to_string(),
                got: other.to_string(),
            }),
        },
        Err(_) => Ok(default),
    }
}

fn is_loopback_host(host: &str) -> bool {
    // RFC 1035 : hostname matching case-insensitive. Trailing dot FQDN accepté.
    let normalized = host.strip_suffix('.').unwrap_or(host);
    if normalized.eq_ignore_ascii_case("localhost") {
        return true;
    }
    // Strip brackets IPv6 : `[::1]` → `::1`.
    let stripped = host
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host);
    // Strip zone ID IPv6 link-local : `fe80::1%eth0` → `fe80::1`.
    let without_zone = stripped.split('%').next().unwrap_or(stripped);
    match without_zone.parse::<std::net::IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => false,
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
            port: 80,
            host: "127.0.0.1".to_string(),
            // Story v011-5 : `Config::admin_username`/`admin_password` sont
            // `Option<String>`. Signature de `make_test_config` inchangée pour
            // ne pas migrer les call-sites tests existants — les callers
            // passent explicitement creds (AC #0 Pass 2 BH2-1).
            admin_username: Some(admin_username.to_string()),
            admin_password: Some(admin_password.to_string()),
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
            bank_import_max_mib: 10,
            admin_export_inmem_mib: 50,
            admin_import_max_mib: 512,
            admin_backup_dir: "/tmp".to_string(),
            cookie_secure: true,
            // Story 17-4b — recovery email off par défaut dans le helper test.
            smtp_host: None,
            smtp_port: 587,
            smtp_user: None,
            smtp_password: None,
            smtp_from: None,
            smtp_tls: true,
            public_base_url: None,
            forgot_password_enabled: false,
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
    pub(super) fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Reset des variables d'environnement communes entre tests.
    pub(super) fn reset_env() {
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
            // Code review Pass 1 ECH-6 : vars manquantes du reset.
            env::remove_var("KESH_BANK_IMPORT_MAX_MB");
            env::remove_var("KESH_LANG");
            // v0.1.3 hotfix Issue #136 — KESH_COOKIE_SECURE reset entre tests.
            env::remove_var("KESH_COOKIE_SECURE");
            // Story 17-4b — vars recovery/SMTP reset entre tests.
            env::remove_var("KESH_SMTP_HOST");
            env::remove_var("KESH_SMTP_PORT");
            env::remove_var("KESH_SMTP_USER");
            env::remove_var("KESH_SMTP_PASSWORD");
            env::remove_var("KESH_SMTP_FROM");
            env::remove_var("KESH_SMTP_TLS");
            env::remove_var("KESH_PUBLIC_BASE_URL");
            env::remove_var("KESH_FEATURE_FORGOT_PASSWORD");
        }
    }

    fn set_minimum_required() {
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            // Story 10-1 T1.2.1 : KESH_ADMIN_PASSWORD désormais obligatoire
            // (suppression du fallback `"changeme"` ligne 358 T1.2). Sans ce
            // set, les 24 tests appelant `set_minimum_required()` cassent par
            // `ConfigError::MissingVar("KESH_ADMIN_PASSWORD")`.
            env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");
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
        assert_eq!(config.port, 80);
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
            // Story 10-1 T1.2.2 : KESH_ADMIN_PASSWORD désormais obligatoire,
            // check ADMIN_PASSWORD intervient AVANT JWT_SECRET dans from_env().
            // Sans ce set, le test reçoit MissingVar(KESH_ADMIN_PASSWORD) au
            // lieu de MissingVar(KESH_JWT_SECRET) qu'il attend.
            env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");
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
            // Story 10-1 T1.2.2 : ADMIN_PASSWORD check intervient avant
            // WeakJwtSecret check → set pour atteindre l'assertion attendue.
            env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");
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

    /// Story v011-5 AC #0 — vars admin OPTIONNELLES.
    /// Var `KESH_ADMIN_PASSWORD` vide après trim → `Config::admin_password = None`
    /// (plus d'erreur `EmptyAdminPassword`). Le flow setup-UI prend le relais sur DB vide.
    #[test]
    fn from_env_admin_vars_empty_returns_none() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_USERNAME", "");
            env::set_var("KESH_ADMIN_PASSWORD", "");
        }

        let config = Config::from_env().expect("empty admin vars should be Ok (Option=None)");
        assert!(config.admin_username.is_none());
        assert!(config.admin_password.is_none());
    }

    /// Story v011-5 AC #0 — admin_username trim. `Some("  admin  ")` après
    /// trim devient `Some("admin")`, non-vide après trim.
    #[test]
    fn config_trims_admin_username() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_USERNAME", "  admin  ");
            env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");
        }

        let config = Config::from_env().expect("should load");
        assert_eq!(
            config.admin_username.as_deref(),
            Some("admin"),
            "admin_username should be trimmed to avoid unreachable admin"
        );
    }

    /// Story v011-5 AC #0 — `KESH_ADMIN_PASSWORD="   "` (whitespace-only) →
    /// trim donne string vide → `None` (cohérent avec absent). Pas d'erreur.
    #[test]
    fn from_env_admin_password_whitespace_only_returns_none() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_PASSWORD", "   ");
        }

        let config = Config::from_env().expect("whitespace-only password should be Ok (None)");
        assert!(config.admin_password.is_none());
    }

    /// Story v011-5 AC #0 — vars admin ABSENTES → `None`/`None`.
    /// Vérifie qu'on peut booter sans aucun `KESH_ADMIN_*` (le flow setup-UI
    /// au 1er boot DB-vide prend le relais).
    #[test]
    fn from_env_admin_vars_absent_returns_none_none() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            // KESH_ADMIN_* NOT set
        }

        let config = Config::from_env().expect("absent admin vars should be Ok");
        assert!(config.admin_username.is_none(), "username should be None");
        assert!(config.admin_password.is_none(), "password should be None");
    }

    /// Story v011-5 AC #0 — vars admin renseignées valides → `Some`/`Some`.
    /// Le check « bootstrap déclaratif » downstream lit `has_admin_env`.
    #[test]
    fn from_env_admin_vars_set_valid_returns_some() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_USERNAME", "admin");
            env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");
        }

        let config = Config::from_env().expect("valid admin vars should be Ok");
        assert_eq!(config.admin_username.as_deref(), Some("admin"));
        assert_eq!(
            config.admin_password.as_deref(),
            Some("valid-test-pw-12chars")
        );
    }

    // --- Story 10-1 T1.7 : 4 nouveaux tests fail-fast secrets ---

    /// Story 10-1 AC #17(a) — `KESH_JWT_SECRET` ≥ 32 chars **mais** contenant
    /// le placeholder `change-me` → `InsecureJwtSecret`. La longueur ≥ 32 est
    /// critique car sinon `WeakJwtSecret` court-circuite le check (ordre des
    /// checks `from_env()` : ligne `len() < 32` AVANT ligne `contains("change-me")`).
    #[test]
    fn config_rejects_jwt_secret_containing_change_me() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");
            // 34 chars, ≥ 32 → passe WeakJwtSecret, contient "change-me" → fail
            env::set_var("KESH_JWT_SECRET", "abcdefghij-change-me-abcdefghijabc");
        }

        let result = Config::from_env();
        assert!(
            matches!(result, Err(ConfigError::InsecureJwtSecret)),
            "expected InsecureJwtSecret, got {result:?}"
        );
    }

    /// Story 10-1 AC #17(b) — `KESH_ADMIN_PASSWORD` égale (case-insensitive
    /// après trim) le placeholder `"changeme"` → `InsecureAdminPassword`.
    /// Couvre lowercase, uppercase, et variantes avec whitespace.
    #[test]
    fn config_rejects_admin_password_changeme_case_insensitive() {
        // Code review Pass 1 BH-5 : acquérir env_lock() UNE fois avant la
        // boucle pour éviter qu'un autre test ne s'intercale entre 2 itérations
        // (sinon `_guard` drop en fin de scope de chaque itération → fenêtre
        // de race condition sur env vars process-global).
        let _guard = env_lock();
        for value in &[
            "changeme",
            "CHANGEME",
            "Changeme",
            "  changeme  ",
            "  CHANGEME  ",
        ] {
            reset_env();
            unsafe {
                env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
                env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
                env::set_var("KESH_ADMIN_PASSWORD", value);
            }

            let result = Config::from_env();
            assert!(
                matches!(result, Err(ConfigError::InsecureAdminPassword)),
                "expected InsecureAdminPassword for {value:?}, got {result:?}"
            );
        }
    }

    /// Story 10-1 AC #17(c) — `KESH_ADMIN_PASSWORD` < 12 chars (après trim)
    /// → `WeakAdminPassword`. Utilise `chars().count()` (pas `len()`) pour
    /// gérer Unicode correctement.
    #[test]
    fn config_rejects_admin_password_short() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_PASSWORD", "short-pw-11"); // 11 chars
        }

        let result = Config::from_env();
        match result {
            Err(ConfigError::WeakAdminPassword { actual_chars }) => {
                assert_eq!(actual_chars, 11);
            }
            other => panic!("expected WeakAdminPassword(11), got {other:?}"),
        }
    }

    /// Story 10-1 AC #17(d) — `KESH_ADMIN_PASSWORD` = 12 chars exactement
    /// passe (limite basse acceptée).
    #[test]
    fn config_accepts_admin_password_exactly_12_chars() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_PASSWORD", "abcdefghijkl"); // 12 chars exact
        }

        let config = Config::from_env().expect("12-char password should pass");
        assert_eq!(config.admin_password.as_deref(), Some("abcdefghijkl"));
    }

    /// Code review Pass 1 ECH-3 + BH-1 : `admin_password` doit être trimmé
    /// AVANT stockage pour éviter l'admin lock-out (login `routes/auth.rs:139`
    /// ne trim PAS le password, donc si bootstrap hash `"  pw  "` avec espaces,
    /// l'admin tape `pw` (sans espaces) et le hash ne match jamais).
    #[test]
    fn config_trims_admin_password_before_storage() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var("KESH_JWT_SECRET", TEST_JWT_SECRET);
            env::set_var("KESH_ADMIN_PASSWORD", "  pw-12-chars-x  "); // 17 chars avec spaces, 13 après trim
        }

        let config = Config::from_env().expect("trimmed password should pass");
        assert_eq!(
            config.admin_password.as_deref(),
            Some("pw-12-chars-x"),
            "admin_password must be trimmed before storage to avoid admin lock-out at login"
        );
    }

    /// Code review Pass 1 ECH-4 : `KESH_JWT_SECRET` contenant `"CHANGE-ME"`
    /// (majuscules) doit aussi être rejeté → cohérent avec le check
    /// admin_password case-insensitive (`eq_ignore_ascii_case`).
    #[test]
    fn config_rejects_jwt_secret_containing_change_me_case_insensitive() {
        let _guard = env_lock();
        for variant in &[
            "abcdefghij-CHANGE-ME-abcdefghijabc", // uppercase
            "abcdefghij-Change-Me-abcdefghijabc", // mixed case
            "abcdefghij-cHaNgE-mE-abcdefghijabc", // alternating
        ] {
            reset_env();
            unsafe {
                env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
                env::set_var("KESH_ADMIN_PASSWORD", "valid-test-pw-12chars");
                env::set_var("KESH_JWT_SECRET", variant);
            }
            let result = Config::from_env();
            assert!(
                matches!(result, Err(ConfigError::InsecureJwtSecret)),
                "expected InsecureJwtSecret for {variant:?}, got {result:?}"
            );
        }
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
            env::set_var("KESH_TEST_MODE", "");
            env::set_var("KESH_HOST", "0.0.0.0");
        }

        let config = Config::from_env().expect("public bind OK when test_mode off");
        assert!(!config.test_mode);
        assert_eq!(config.host, "0.0.0.0");
    }

    /// Code review P7 : parsing strict de `KESH_TEST_MODE`. Les variantes
    /// `"True"`, `"yes"`, `" true"` ne doivent PAS être silencieusement
    /// interprétées comme `false` — elles produisent `InvalidTestModeValue`
    /// pour éviter l'ambiguïté "je pensais que test_mode était actif".
    #[test]
    fn config_test_mode_rejects_capital_true_value() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "True");
        }

        let result = Config::from_env();
        match result {
            Err(ConfigError::InvalidTestModeValue { got }) => assert_eq!(got, "True"),
            other => panic!("expected InvalidTestModeValue, got {other:?}"),
        }
    }

    #[test]
    fn config_test_mode_rejects_yes() {
        let _guard = env_lock();
        reset_env();
        set_minimum_required();
        unsafe {
            env::set_var("KESH_TEST_MODE", "yes");
        }

        let result = Config::from_env();
        assert!(
            matches!(result, Err(ConfigError::InvalidTestModeValue { .. })),
            "`yes` should be rejected, got {result:?}"
        );
    }

    /// Code review P4 + pass 2 E5 : `is_loopback_host` reconnaît IPv6
    /// bracketé, zone ID, et `localhost` case-insensitive + trailing dot.
    #[test]
    fn is_loopback_host_accepts_ipv6_variants() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("::1"));
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("[::1]"), "bracketed IPv6 loopback");
        assert!(is_loopback_host("127.0.0.2"), "any 127.0.0.0/8 is loopback");
        // Pass 2 E5 : RFC 1035 hostname case-insensitive + trailing dot FQDN.
        assert!(is_loopback_host("Localhost"), "case-insensitive localhost");
        assert!(is_loopback_host("LOCALHOST"), "case-insensitive localhost");
        assert!(is_loopback_host("localhost."), "trailing dot FQDN");
        assert!(
            is_loopback_host("LocalHost."),
            "case-insensitive + trailing dot"
        );
        // Rejets explicites.
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("192.168.1.1"));
        assert!(!is_loopback_host("::"), "unspecified IPv6 must be rejected");
        assert!(
            !is_loopback_host("localhost.evil.com"),
            "FQDN with domain suffix must be rejected"
        );
    }

    /// Code review P6 : `with_test_mode(true)` doit paniquer si host
    /// n'est pas loopback — miroir du garde-fou runtime.
    #[test]
    #[should_panic(expected = "with_test_mode(true) exige un bind loopback")]
    fn with_test_mode_panics_on_public_host() {
        // Construit un Config via from_fields_for_test (host="127.0.0.1"
        // par défaut) puis mute le host pour simuler un caller fautif.
        let mut config = Config::from_fields_for_test(
            "mysql://test:test@localhost:3306/test".to_string(),
            "admin".to_string(),
            "admin123".to_string(),
            TEST_JWT_SECRET.to_string(),
            TimeDelta::minutes(15),
            TimeDelta::days(30),
            TimeDelta::minutes(15),
            TimeDelta::minutes(15),
            100,
            TimeDelta::minutes(30),
            12,
        );
        config.host = "0.0.0.0".to_string();
        let _ = config.with_test_mode(true);
    }

    #[test]
    fn with_test_mode_ok_on_loopback() {
        let config = Config::from_fields_for_test(
            "mysql://test:test@localhost:3306/test".to_string(),
            "admin".to_string(),
            "admin123".to_string(),
            TEST_JWT_SECRET.to_string(),
            TimeDelta::minutes(15),
            TimeDelta::days(30),
            TimeDelta::minutes(15),
            TimeDelta::minutes(15),
            100,
            TimeDelta::minutes(30),
            12,
        )
        .with_test_mode(true);
        assert!(config.test_mode);
    }

    // --- Story v011-1 : parsing LogConfig (pur, sans env) ---

    #[test]
    fn log_config_defaults_when_all_absent() {
        let (cfg, warnings) = LogConfig::from_raw(None, None, None, None);
        assert_eq!(cfg, LogConfig::default());
        assert_eq!(cfg.file_path, None);
        assert_eq!(cfg.rotation, LogRotation::Daily);
        assert_eq!(cfg.max_files, 7);
        assert_eq!(cfg.format, LogFormat::Pretty);
        assert!(warnings.is_empty());
    }

    #[test]
    fn log_config_empty_path_disables_file_output() {
        let (cfg, warnings) = LogConfig::from_raw(Some("   ".to_string()), None, None, None);
        assert_eq!(cfg.file_path, None, "whitespace-only path → opt-out");
        assert!(warnings.is_empty());
    }

    #[test]
    fn log_config_path_is_trimmed() {
        let (cfg, _) = LogConfig::from_raw(
            Some("  /var/log/kesh/kesh.log  ".to_string()),
            None,
            None,
            None,
        );
        assert_eq!(cfg.file_path.as_deref(), Some("/var/log/kesh/kesh.log"));
    }

    #[test]
    fn log_config_rotation_valid_values_case_insensitive() {
        for (raw, expected) in [
            ("daily", LogRotation::Daily),
            ("HOURLY", LogRotation::Hourly),
            ("Never", LogRotation::Never),
        ] {
            let mut w = Vec::new();
            assert_eq!(LogRotation::parse(raw, &mut w), expected);
            assert!(w.is_empty(), "valeur valide ne doit pas warner: {raw}");
        }
    }

    #[test]
    fn log_config_rotation_invalid_falls_back_with_warning() {
        let (cfg, warnings) = LogConfig::from_raw(None, Some("weekly".to_string()), None, None);
        assert_eq!(cfg.rotation, LogRotation::Daily);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("KESH_LOG_FILE_ROTATION"));
    }

    #[test]
    fn log_config_format_valid_and_invalid() {
        let mut w = Vec::new();
        assert_eq!(LogFormat::parse("JSON", &mut w), LogFormat::Json);
        assert_eq!(LogFormat::parse("pretty", &mut w), LogFormat::Pretty);
        assert!(w.is_empty());
        let (cfg, warnings) = LogConfig::from_raw(None, None, None, Some("xml".to_string()));
        assert_eq!(cfg.format, LogFormat::Pretty);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("KESH_LOG_FILE_FORMAT"));
    }

    #[test]
    fn log_config_max_files_valid_zero_and_invalid() {
        // valeur valide
        let (cfg, w) = LogConfig::from_raw(None, None, Some("30".to_string()), None);
        assert_eq!(cfg.max_files, 30);
        assert!(w.is_empty());
        // zéro → fallback 7 + warning
        let (cfg, w) = LogConfig::from_raw(None, None, Some("0".to_string()), None);
        assert_eq!(cfg.max_files, 7);
        assert_eq!(w.len(), 1);
        // non-numérique → fallback 7 + warning
        let (cfg, w) = LogConfig::from_raw(None, None, Some("abc".to_string()), None);
        assert_eq!(cfg.max_files, 7);
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn log_config_accumulates_multiple_warnings() {
        let (cfg, warnings) = LogConfig::from_raw(
            Some("/tmp/k.log".to_string()),
            Some("bad".to_string()),
            Some("nope".to_string()),
            Some("bad".to_string()),
        );
        assert_eq!(cfg.file_path.as_deref(), Some("/tmp/k.log"));
        assert_eq!(cfg.rotation, LogRotation::Daily);
        assert_eq!(cfg.max_files, 7);
        assert_eq!(cfg.format, LogFormat::Pretty);
        assert_eq!(warnings.len(), 3, "3 vars invalides → 3 warnings");
    }
}

// =============================================================================
// v0.1.3 hotfix (Issue #136) — KESH_COOKIE_SECURE tests
// =============================================================================

#[cfg(test)]
mod cookie_secure_tests {
    use super::tests::{env_lock, reset_env};
    use super::*;
    use std::env;

    /// Cas nominal : var absente → cookie_secure = true (défaut sécurisé).
    #[test]
    fn cookie_secure_defaults_to_true_when_var_absent() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::remove_var("KESH_COOKIE_SECURE");
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var(
                "KESH_JWT_SECRET",
                "test-secret-32-bytes-minimum-test-secret-padding",
            );
            env::set_var("KESH_HOST", "127.0.0.1");
        }
        let config = Config::from_env().expect("config should load");
        assert!(config.cookie_secure, "défaut sécurisé = true");
    }

    /// `KESH_COOKIE_SECURE=true` → cookie_secure = true.
    #[test]
    fn cookie_secure_true_string_parses_true() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var(
                "KESH_JWT_SECRET",
                "test-secret-32-bytes-minimum-test-secret-padding",
            );
            env::set_var("KESH_HOST", "127.0.0.1");
            env::set_var("KESH_COOKIE_SECURE", "true");
        }
        let config = Config::from_env().expect("true is valid");
        assert!(config.cookie_secure);
    }

    /// `KESH_COOKIE_SECURE=1` → cookie_secure = true (équivalent à "true").
    #[test]
    fn cookie_secure_one_parses_true() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var(
                "KESH_JWT_SECRET",
                "test-secret-32-bytes-minimum-test-secret-padding",
            );
            env::set_var("KESH_HOST", "127.0.0.1");
            env::set_var("KESH_COOKIE_SECURE", "1");
        }
        let config = Config::from_env().expect("1 is valid");
        assert!(config.cookie_secure);
    }

    /// `KESH_COOKIE_SECURE=false` → cookie_secure = false (override LAN HTTP).
    #[test]
    fn cookie_secure_false_string_parses_false() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var(
                "KESH_JWT_SECRET",
                "test-secret-32-bytes-minimum-test-secret-padding",
            );
            env::set_var("KESH_HOST", "127.0.0.1");
            env::set_var("KESH_COOKIE_SECURE", "false");
        }
        let config = Config::from_env().expect("false is valid");
        assert!(!config.cookie_secure, "explicit override pour LAN HTTP");
    }

    /// `KESH_COOKIE_SECURE=0` → cookie_secure = false (équivalent à "false").
    #[test]
    fn cookie_secure_zero_parses_false() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var(
                "KESH_JWT_SECRET",
                "test-secret-32-bytes-minimum-test-secret-padding",
            );
            env::set_var("KESH_HOST", "127.0.0.1");
            env::set_var("KESH_COOKIE_SECURE", "0");
        }
        let config = Config::from_env().expect("0 is valid");
        assert!(!config.cookie_secure);
    }

    /// `KESH_COOKIE_SECURE=True` (capitalisé) → fail-fast.
    /// Strict parse pour éviter ambiguïté (cohérent KESH_TEST_MODE).
    #[test]
    fn cookie_secure_invalid_value_fails_fast() {
        let _guard = env_lock();
        for invalid_val in &["True", "TRUE", "yes", "on", "  true  ", "True ", "no"] {
            unsafe {
                env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
                env::set_var(
                    "KESH_JWT_SECRET",
                    "test-secret-32-bytes-minimum-test-secret-padding",
                );
                env::set_var("KESH_HOST", "127.0.0.1");
                env::set_var("KESH_COOKIE_SECURE", invalid_val);
            }
            let result = Config::from_env();
            assert!(
                matches!(result, Err(ConfigError::InvalidCookieSecureValue { .. })),
                "value '{}' should fail-fast, got {:?}",
                invalid_val,
                result
            );
        }
    }

    /// `KESH_COOKIE_SECURE` vide → traité comme absent (défaut true).
    #[test]
    fn cookie_secure_empty_string_defaults_to_true() {
        let _guard = env_lock();
        reset_env();
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var(
                "KESH_JWT_SECRET",
                "test-secret-32-bytes-minimum-test-secret-padding",
            );
            env::set_var("KESH_HOST", "127.0.0.1");
            env::set_var("KESH_COOKIE_SECURE", "");
        }
        let config = Config::from_env().expect("empty is treated as absent");
        assert!(config.cookie_secure);
    }
}

// =============================================================================
// Story 17-4b — KESH_SMTP_* / KESH_FEATURE_FORGOT_PASSWORD (recovery email)
// =============================================================================

#[cfg(test)]
mod smtp_config_tests {
    use super::tests::{env_lock, reset_env};
    use super::*;
    use std::env;

    /// Pose les vars minimales requises pour un boot config valide (hors SMTP).
    fn set_minimum() {
        unsafe {
            env::set_var("DATABASE_URL", "mysql://test:test@localhost:3306/test");
            env::set_var(
                "KESH_JWT_SECRET",
                "test-secret-32-bytes-minimum-test-secret-padding",
            );
            env::set_var("KESH_HOST", "127.0.0.1");
        }
    }

    /// Pose une config SMTP complète et valide.
    fn set_complete_smtp() {
        unsafe {
            env::set_var("KESH_SMTP_HOST", "smtp.example.com");
            env::set_var("KESH_SMTP_USER", "kesh@example.com");
            env::set_var("KESH_SMTP_PASSWORD", "s3cr3t-app-password");
            env::set_var("KESH_SMTP_FROM", "noreply@example.com");
            env::set_var("KESH_PUBLIC_BASE_URL", "https://kesh.example.com");
        }
    }

    /// Feature off + SMTP absent → OK, recovery désactivé (défauts).
    #[test]
    fn feature_off_without_smtp_loads_ok() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        let config = Config::from_env().expect("feature off → SMTP optionnel");
        assert!(!config.forgot_password_enabled);
        assert!(config.smtp_host.is_none());
        assert_eq!(config.smtp_port, 587, "défaut STARTTLS submission");
        assert!(config.smtp_tls, "défaut true");
    }

    /// Feature on + SMTP complet → OK, champs renseignés.
    #[test]
    fn feature_on_with_complete_smtp_loads_ok() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        set_complete_smtp();
        unsafe {
            env::set_var("KESH_FEATURE_FORGOT_PASSWORD", "true");
            env::set_var("KESH_SMTP_PORT", "2525");
        }
        let config = Config::from_env().expect("feature on + complet → OK");
        assert!(config.forgot_password_enabled);
        assert_eq!(config.smtp_host.as_deref(), Some("smtp.example.com"));
        assert_eq!(config.smtp_port, 2525);
        assert_eq!(config.smtp_password(), Some("s3cr3t-app-password"));
        assert_eq!(
            config.public_base_url.as_deref(),
            Some("https://kesh.example.com")
        );
    }

    /// Feature on + une var manquante (PUBLIC_BASE_URL) → fail-fast.
    #[test]
    fn feature_on_missing_var_fails_fast() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        set_complete_smtp();
        unsafe {
            env::set_var("KESH_FEATURE_FORGOT_PASSWORD", "true");
            env::remove_var("KESH_PUBLIC_BASE_URL");
        }
        let err = Config::from_env().expect_err("var manquante → Err");
        assert!(
            matches!(err, ConfigError::IncompleteSmtpConfig { .. }),
            "attendu IncompleteSmtpConfig, obtenu {err:?}"
        );
    }

    /// Feature on + SMTP_FROM invalide → fail-fast.
    #[test]
    fn feature_on_invalid_from_fails_fast() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        set_complete_smtp();
        unsafe {
            env::set_var("KESH_FEATURE_FORGOT_PASSWORD", "true");
            env::set_var("KESH_SMTP_FROM", "pas-un-email");
        }
        let err = Config::from_env().expect_err("from invalide → Err");
        assert!(
            matches!(err, ConfigError::IncompleteSmtpConfig { .. }),
            "attendu IncompleteSmtpConfig, obtenu {err:?}"
        );
    }

    /// Feature flag avec valeur non-stricte → InvalidBoolValue.
    #[test]
    fn feature_flag_invalid_bool_fails_fast() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        unsafe {
            env::set_var("KESH_FEATURE_FORGOT_PASSWORD", "yes");
        }
        let err = Config::from_env().expect_err("'yes' non strict → Err");
        assert!(
            matches!(err, ConfigError::InvalidBoolValue { .. }),
            "attendu InvalidBoolValue, obtenu {err:?}"
        );
    }

    /// `smtp_password` masqué dans `Debug` (jamais en clair dans les logs).
    #[test]
    fn smtp_password_masked_in_debug() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        set_complete_smtp();
        unsafe {
            env::set_var("KESH_FEATURE_FORGOT_PASSWORD", "true");
        }
        let config = Config::from_env().expect("complet → OK");
        let dbg = format!("{config:?}");
        assert!(
            !dbg.contains("s3cr3t-app-password"),
            "le mot de passe SMTP ne doit JAMAIS apparaître dans Debug"
        );
        // Review Pass 1 — vérifier la présence du masque lui-même, pas
        // seulement l'absence du secret (détecte une régression du masquage).
        assert!(dbg.contains("***"), "masque *** présent dans Debug");
        assert!(dbg.contains("smtp_password"), "champ présent (masqué ***)");
    }

    /// SMTP_TLS strict bool : 'false' → false.
    #[test]
    fn smtp_tls_false_parses() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        unsafe {
            env::set_var("KESH_SMTP_TLS", "false");
        }
        let config = Config::from_env().expect("tls=false valide");
        assert!(!config.smtp_tls);
    }

    /// Review Pass 1 — bool strict avec espaces parasites (`.env` édité à la
    /// main) : trim avant comparaison, cohérent avec `opt_trimmed_env`.
    #[test]
    fn strict_bool_trims_whitespace() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        unsafe {
            env::set_var("KESH_SMTP_TLS", " false ");
        }
        let config = Config::from_env().expect("' false ' trimé → valide");
        assert!(!config.smtp_tls);
    }

    /// Review Pass 1 (P4-4 umbrella) — trailing slash strippé de
    /// PUBLIC_BASE_URL pour éviter `{base}//reset-password` côté 17-4c.
    #[test]
    fn public_base_url_trailing_slash_stripped() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        unsafe {
            env::set_var("KESH_PUBLIC_BASE_URL", "https://kesh.example.com/");
        }
        let config = Config::from_env().expect("base url valide");
        assert_eq!(
            config.public_base_url.as_deref(),
            Some("https://kesh.example.com")
        );
    }

    /// Review Pass 1 — base url réduite à des slashes → None (pas une string
    /// vide qui fabriquerait des reset_url relatifs).
    #[test]
    fn public_base_url_only_slashes_is_none() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        unsafe {
            env::set_var("KESH_PUBLIC_BASE_URL", "///");
        }
        let config = Config::from_env().expect("hors feature, pas de fail-fast");
        assert!(config.public_base_url.is_none());
    }

    /// Review Pass 1 — `KESH_SMTP_HOST=host:port` (copié d'une doc SMTP)
    /// détecté au boot plutôt qu'au premier envoi (erreur SNI cryptique).
    #[test]
    fn feature_on_host_with_port_fails_fast() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        set_complete_smtp();
        unsafe {
            env::set_var("KESH_FEATURE_FORGOT_PASSWORD", "true");
            env::set_var("KESH_SMTP_HOST", "smtp.example.com:587");
        }
        let err = Config::from_env().expect_err("host:port → Err au boot");
        match err {
            ConfigError::IncompleteSmtpConfig { detail } => {
                assert!(
                    detail.contains("KESH_SMTP_PORT"),
                    "message guide : {detail}"
                );
            }
            other => panic!("attendu IncompleteSmtpConfig, obtenu {other:?}"),
        }
    }

    /// Review Pass 1 — IPv6 literal accepté comme host (SMTP plaintext LAN) :
    /// les `:` d'une adresse IPv6 ne sont pas un port imbriqué.
    #[test]
    fn feature_on_ipv6_host_allowed() {
        let _guard = env_lock();
        reset_env();
        set_minimum();
        set_complete_smtp();
        unsafe {
            env::set_var("KESH_FEATURE_FORGOT_PASSWORD", "true");
            env::set_var("KESH_SMTP_HOST", "fd00::25");
        }
        let config = Config::from_env().expect("IPv6 literal accepté");
        assert_eq!(config.smtp_host.as_deref(), Some("fd00::25"));
    }
}
