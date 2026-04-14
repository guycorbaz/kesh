//! Erreurs de la couche de persistance.

use thiserror::Error;

/// Erreurs des opérations de persistance MariaDB.
///
/// Les messages `Display` sont destinés au logging serveur uniquement.
/// `kesh-api` mappe chaque variante vers un code HTTP et un message traduit
/// via `kesh-i18n`. Ne jamais exposer le `Display` au frontend.
///
/// **Important** : cette enum ne dérive PAS `From<sqlx::Error>` pour forcer
/// tous les call sites à passer par `map_db_error`, garantissant ainsi que
/// les violations de contraintes sont correctement classifiées.
#[derive(Debug, Error)]
pub enum DbError {
    /// Entité introuvable (SELECT sans résultat sur une opération qui en attend un).
    #[error("Entité non trouvée")]
    NotFound,

    /// Verrouillage optimiste : version en base ≠ version fournie dans l'UPDATE.
    #[error("Conflit de version — l'entité a été modifiée par un autre utilisateur")]
    OptimisticLockConflict,

    /// Contrainte d'unicité violée (code MariaDB 1062).
    #[error("Contrainte d'unicité violée : {0}")]
    UniqueConstraintViolation(String),

    /// Contrainte de clé étrangère violée (codes MariaDB 1451/1452).
    #[error("Contrainte de clé étrangère violée : {0}")]
    ForeignKeyViolation(String),

    /// Contrainte CHECK violée (code MariaDB 4025, MySQL 3819).
    #[error("Contrainte CHECK violée : {0}")]
    CheckConstraintViolation(String),

    /// Transition d'état métier interdite (ex: clôturer un exercice déjà clos,
    /// réouvrir un exercice clos). Mappé vers HTTP 409 Conflict côté API.
    #[error("Transition d'état interdite : {0}")]
    IllegalStateTransition(String),

    /// L'exercice comptable est clôturé (FR24, CO art. 957-964) — aucune
    /// écriture ne peut y être ajoutée, modifiée ou supprimée. Variante
    /// dédiée (séparée d'`IllegalStateTransition`) pour permettre un
    /// mapping API stable, non dépendant du contenu du message texte.
    #[error("Exercice clôturé — modification interdite (CO art. 957-964)")]
    FiscalYearClosed,

    /// Un ou plusieurs comptes référencés sont archivés ou n'appartiennent
    /// pas à la company courante. Variante dédiée pour exposer un message
    /// UX clair sans leak du détail interne.
    #[error("Un ou plusieurs comptes sont archivés ou invalides")]
    InactiveOrInvalidAccounts,

    /// La date fournie ne tombe pas dans l'exercice courant de l'entité
    /// modifiée. Story 3.3 : empêche le déplacement d'une écriture vers
    /// un autre exercice via un simple changement de date.
    #[error("La date n'est pas dans l'exercice courant de cette écriture")]
    DateOutsideFiscalYear,

    /// Aucun exercice ouvert ne couvre la date fournie (Story 5.2).
    /// Distinct de `FiscalYearClosed` — l'exercice est peut-être
    /// inexistant (date hors de tous les exercices connus) OU clôturé.
    /// Mappé vers HTTP 400 `FISCAL_YEAR_INVALID` côté API.
    #[error("Aucun exercice ouvert ne couvre cette date")]
    FiscalYearInvalid,

    /// Un champ de configuration requis pour l'opération est absent
    /// (Story 5.2 : `default_receivable_account_id` ou
    /// `default_revenue_account_id` manquant dans `company_invoice_settings`).
    /// Mappé vers HTTP 400 `CONFIGURATION_REQUIRED` côté API.
    #[error("Configuration manquante : {0}")]
    ConfigurationRequired(String),

    /// Pool épuisé ou timeout d'acquisition (retry-able côté API → 503).
    #[error("Pool de connexions épuisé ou timeout : {0}")]
    ConnectionUnavailable(String),

    /// Invariant du crate violé (ex: AUTO_INCREMENT retourne une valeur impossible).
    /// Indique un bug ou un état de DB corrompu, jamais une erreur utilisateur.
    #[error("Invariant kesh-db violé : {0}")]
    Invariant(String),

    /// Erreur SQLx non classifiée (syntaxe, type mismatch, etc.).
    ///
    /// `#[source]` préserve la chaîne d'erreur pour anyhow/tracing —
    /// `DbError::source()` renvoie bien la `sqlx::Error` sous-jacente.
    #[error("Erreur SQLx : {0}")]
    Sqlx(#[source] sqlx::Error),
}

impl DbError {
    /// Code d'erreur structuré pour le mapping API (utilisé par `kesh-api`
    /// pour construire les réponses d'erreur JSON).
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound => "NOT_FOUND",
            Self::OptimisticLockConflict => "OPTIMISTIC_LOCK_CONFLICT",
            Self::UniqueConstraintViolation(_) => "UNIQUE_CONSTRAINT_VIOLATION",
            Self::ForeignKeyViolation(_) => "FOREIGN_KEY_VIOLATION",
            Self::CheckConstraintViolation(_) => "CHECK_CONSTRAINT_VIOLATION",
            Self::IllegalStateTransition(_) => "ILLEGAL_STATE_TRANSITION",
            Self::FiscalYearClosed => "FISCAL_YEAR_CLOSED",
            Self::InactiveOrInvalidAccounts => "INACTIVE_OR_INVALID_ACCOUNTS",
            Self::DateOutsideFiscalYear => "DATE_OUTSIDE_FISCAL_YEAR",
            Self::FiscalYearInvalid => "FISCAL_YEAR_INVALID",
            Self::ConfigurationRequired(_) => "CONFIGURATION_REQUIRED",
            Self::ConnectionUnavailable(_) => "CONNECTION_UNAVAILABLE",
            Self::Invariant(_) => "INVARIANT_VIOLATION",
            Self::Sqlx(_) => "DATABASE_ERROR",
        }
    }
}

/// Convertit une `sqlx::Error` en `DbError` en détectant les violations de
/// contraintes via les codes d'erreur numériques MariaDB/MySQL (stables et
/// locale-indépendants).
///
/// Codes détectés :
/// - **1062** : `ER_DUP_ENTRY` — contrainte unique
/// - **1451/1452** : violations de clé étrangère
/// - **4025** : `ER_CONSTRAINT_FAILED` (MariaDB 10.2+)
/// - **3819** : `ER_CHECK_CONSTRAINT_VIOLATED` (MySQL 8.0.16+, fallback)
///
/// Les erreurs de connexion (pool timeout, pool closed, IO) sont mappées
/// vers `DbError::ConnectionUnavailable` pour permettre un retry côté API.
pub fn map_db_error(err: sqlx::Error) -> DbError {
    // Erreurs de connexion / pool — retry-able
    match &err {
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => {
            return DbError::ConnectionUnavailable(err.to_string());
        }
        sqlx::Error::Io(io_err) => {
            return DbError::ConnectionUnavailable(io_err.to_string());
        }
        sqlx::Error::RowNotFound => {
            return DbError::NotFound;
        }
        _ => {}
    }

    if let Some(db_err) = err.as_database_error() {
        if let Some(my_err) = db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>() {
            match my_err.number() {
                1062 => return DbError::UniqueConstraintViolation(my_err.message().to_string()),
                1451 | 1452 => {
                    return DbError::ForeignKeyViolation(my_err.message().to_string());
                }
                4025 | 3819 => {
                    return DbError::CheckConstraintViolation(my_err.message().to_string());
                }
                _ => {}
            }
        }
    }
    DbError::Sqlx(err)
}
