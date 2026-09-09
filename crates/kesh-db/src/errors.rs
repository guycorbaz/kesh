//! Erreurs de la couche de persistance.

use thiserror::Error;

/// Raison du refus d'un compte de produit référencé par une ligne de facture
/// (Story 16-1a, #152 — décision D3).
///
/// Les quatre critères sont contrôlés à la **saisie** (création / modification
/// du brouillon) et **re-contrôlés au posting** (validation). Deux d'entre eux
/// ne sont couverts par aucune garde existante : `validate_lines_accounts_in_tx`
/// vérifie `active` inconditionnellement mais laisse passer `postable` (le flux
/// facture appelle `create_in_tx` avec `enforce_postable = false`) et ne
/// consulte **jamais** `account_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevenueAccountRejection {
    /// Compte inexistant, ou appartenant à une autre société (anti-IDOR — on
    /// ne distingue pas les deux cas, pour ne pas révéler l'existence d'un id).
    UnknownOrCrossCompany,
    /// Compte archivé (`active = FALSE`).
    Inactive,
    /// `account_type` différent de `Revenue` — un compte peut avoir été retypé
    /// par `accounts::update` après avoir été choisi sur une ligne.
    NotRevenue,
    /// Compte non imputable (`postable = FALSE`) **et** différent du compte de
    /// produit par défaut de la société, qui bénéficie de l'exemption D3-bis.
    NotPostable,
}

/// Un site en défaut lors du contrôle des comptes de produit (Story 16-1a).
///
/// Le contrôle est batché (une requête pour toute la facture, décision D6) et
/// remonte **tous** les sites en défaut à la fois : un compte partagé archivé
/// invalide plusieurs lignes d'un coup, et l'utilisateur doit toutes les voir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedRevenueAccount {
    /// Numéro de ligne **1-based**, tel qu'affiché à l'utilisateur.
    ///
    /// `None` désigne le **compte de produit par défaut de la société** :
    /// aucune ligne ne le porte, il ne peut donc pas être nommé par un numéro
    /// (AC8-bis). L'API le nomme explicitement dans le message.
    pub line_number: Option<i32>,
    pub account_id: i64,
    /// Numéro du compte au plan comptable. `None` quand le compte est inconnu
    /// de la société — il n'y a alors rien à afficher.
    pub account_number: Option<String>,
    pub reason: RevenueAccountRejection,
}

/// Motif pour lequel une écriture ne peut **pas** être contre-passée
/// (Story 24-4a, #380).
///
/// ⚠️ **Les causes se CUMULENT** — une écriture peut être possédée par une
/// facture, déjà contre-passée *et* porter un compte archivé. Le champ exposé
/// étant scalaire, la précédence est figée par l'ordre des variantes ci-dessous,
/// sans quoi le test « une cause, un test » deviendrait non déterministe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReversalBlocker {
    /// L'écriture EST une contre-passation. En contre-passer une reviendrait à
    /// réécrire l'original en trois écritures au lieu d'une.
    IsAReversal,
    /// Une contre-passation existe déjà (garantie par `uq_journal_entries_reverses`).
    AlreadyReversed,
    /// Écriture de vente d'une facture client → le chemin est l'**avoir**.
    OwnedByInvoice,
    /// L'avoir EST déjà la contre-passation de la facture.
    OwnedByCreditNote,
    /// Écriture d'achat ou de règlement d'une facture fournisseur → le chemin
    /// est `supplier_invoices::cancel`, ou l'issue #414 pour le règlement.
    OwnedBySupplierInvoice,
    /// ⛔ Le cas le plus grave : le résiduel se calcule depuis
    /// `invoice_settlements.amount`, que la contre-passation ne toucherait pas —
    /// grand livre et résiduel divergeraient **en silence**. Chemin : #414.
    OwnedBySettlement,
    /// Écriture rapprochée d'une transaction bancaire. ⚠️ Aucune route de
    /// dé-rapprochement n'existe (#418) : le refus laisse un manque, assumé,
    /// parce que l'alternative recrée une désynchronisation muette.
    MatchedBankTransaction,
    /// Un compte de l'écriture a été **archivé** depuis.
    ///
    /// ⛔ **En DERNIER de la précédence, et c'est voulu** : c'est le seul motif
    /// que l'utilisateur peut lever lui-même (`PUT /accounts/{id}/reactivate`).
    /// Le dire en premier ferait croire qu'une écriture possédée par une facture
    /// deviendrait contre-passable une fois le compte réactivé — elle ne le
    /// serait pas.
    ///
    /// ⚠️ Le refus à l'ÉCRITURE reste un **400** [`DbError::ReversalAccountsArchived`],
    /// qui NOMME les comptes ; ce code-ci sert la LECTURE, pour que l'écran
    /// masque le bouton **avant** le clic (AC 11).
    AccountArchived,
}

impl ReversalBlocker {
    /// Code canonique, jamais une phrase : la traduction se fait à l'écran.
    pub fn code(self) -> &'static str {
        match self {
            Self::IsAReversal => "IS_A_REVERSAL",
            Self::AlreadyReversed => "ALREADY_REVERSED",
            Self::OwnedByInvoice => "OWNED_BY_INVOICE",
            Self::OwnedByCreditNote => "OWNED_BY_CREDIT_NOTE",
            Self::OwnedBySupplierInvoice => "OWNED_BY_SUPPLIER_INVOICE",
            Self::OwnedBySettlement => "OWNED_BY_SETTLEMENT",
            Self::MatchedBankTransaction => "MATCHED_BANK_TRANSACTION",
            Self::AccountArchived => "ACCOUNT_ARCHIVED",
        }
    }
}

/// Un compte de l'écriture d'origine, archivé depuis (Story 24-4a, #380).
///
/// Le refus **nomme** le compte à réactiver — un « interdit » sec ne serait pas
/// utilisable. Gabarit : [`RejectedRevenueAccount`] et son `details.rejected[]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivedAccount {
    pub account_id: i64,
    /// `None` quand le compte est inconnu de la société — rien à afficher.
    pub account_number: Option<String>,
}

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

    /// Transition d'état métier interdite (ex: re-clôturer un exercice déjà
    /// clos — idempotence de `close`). Mappé vers HTTP 409 Conflict côté API.
    ///
    /// **Note (Story 14-2)** : la réouverture d'un exercice clos est désormais
    /// **autorisée** (via `fiscal_years::reopen`, Admin + motif + audit). Ses
    /// conflits métier (déjà ouvert, garde LIFO) sont émis en
    /// [`DbError::Invariant`] namespacés (message distinct localisé au mapper),
    /// **pas** via cette variante dont le `Display` est log-only.
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

    /// Un rôle de compte **singleton** est déjà porté par un autre compte actif
    /// de la même société (Story 14-3a).
    ///
    /// Variante dédiée — et non le générique [`DbError::UniqueConstraintViolation`]
    /// — pour que l'API puisse **nommer le compte en conflit** : le mapping
    /// générique du code MariaDB 1062 produit un message fixe et jette le détail.
    /// Les champs viennent d'un `SELECT` fait dans la même transaction, avant
    /// l'écriture ; la contrainte `uq_accounts_company_singleton_role` reste la
    /// source de vérité (elle rattrape les courses perdues en 1062).
    #[error("Le rôle {role} est déjà attribué au compte {account_number}")]
    AccountRoleAlreadyAssigned {
        /// Rôle en conflit, en PascalCase (ex. `"Receivable"`).
        role: String,
        /// ID du compte qui porte déjà le rôle.
        account_id: i64,
        /// Numéro du compte qui porte déjà le rôle.
        account_number: String,
        /// Libellé du compte qui porte déjà le rôle.
        account_name: String,
    },

    /// Réactivation refusée : le compte parent est archivé (Story 14-3a, #269).
    ///
    /// Variante dédiée — et non le générique [`DbError::IllegalStateTransition`]
    /// — parce que ce dernier est mappé sur un message fixe (« Transition d'état
    /// interdite ») qui ne dit **pas** à l'utilisateur ce qu'il doit faire. Le
    /// code review Pass 1 a montré que le message explicatif écrit côté
    /// repository n'atteignait jamais le client. Le numéro du parent permet à
    /// l'API de produire un message actionnable et traduit.
    #[error("Le compte parent {parent_number} est archivé")]
    AccountParentArchived {
        /// Numéro du compte parent archivé.
        parent_number: String,
    },

    /// Le rôle demandé est incompatible avec le type du compte (Story 14-3a).
    ///
    /// Contrainte volontairement **minimale** : seule la frontière bilan /
    /// résultat est vérifiée (cf. `AccountRole::accepts_account_type`). Mappée
    /// vers HTTP 400 côté API.
    #[error("Le rôle {role} est incompatible avec un compte de type {account_type}")]
    AccountRoleInvalidForType {
        /// Rôle demandé, en PascalCase (ex. `"Payable"`).
        role: String,
        /// Type du compte, en PascalCase (ex. `"Expense"`).
        account_type: String,
    },

    /// Un ou plusieurs comptes de produit de ligne de facture sont refusés
    /// (Story 16-1a, #152 — D3, D3-bis, D6, AC8-bis).
    ///
    /// Variante dédiée — et non le générique [`DbError::InactiveOrInvalidAccounts`]
    /// — parce que ce dernier est mappé sur « Un ou plusieurs comptes sont
    /// archivés ou invalides », qui **ne nomme aucune ligne**. Sur une facture
    /// pouvant porter 200 lignes, ce message n'est pas actionnable. Les sites
    /// en défaut sont donc transportés en structuré jusqu'à `kesh-api`, qui
    /// compose un message traduit les nommant tous.
    ///
    /// Mappé vers HTTP 400 `INVOICE_LINE_REVENUE_ACCOUNT_INVALID`.
    #[error("Comptes de produit de ligne invalides ({} site(s) en défaut)", .0.len())]
    InvalidRevenueAccounts(Vec<RejectedRevenueAccount>),

    /// Émission d'avoir bloquée : au moins un compte de produit du snapshot de
    /// la facture est archivé (Story 16-1a, décision D5-bis).
    ///
    /// La contre-passation doit viser **les mêmes comptes** que l'écriture
    /// d'origine — se replier sur le défaut société recréerait exactement le
    /// résidu que D5 combat. Poster sur un compte archivé est impossible (la
    /// garde `active` de `create_in_tx` est inconditionnelle). L'avoir échoue
    /// donc, en nommant la ligne et le compte à réactiver : un avoir bloqué
    /// est préférable à un avoir sur le mauvais compte.
    ///
    /// Seul `active` est concerné — ni `postable` ni `account_type` ne sont
    /// re-vérifiés côté avoir (D5-bis).
    ///
    /// Mappé vers HTTP 400 `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED`.
    #[error("Comptes de produit archivés sur l'avoir ({} ligne(s))", .0.len())]
    CreditNoteRevenueAccountsArchived(Vec<RejectedRevenueAccount>),

    /// L'écriture ne peut pas être contre-passée (Story 24-4a, #380).
    ///
    /// C'est un **conflit d'état**, pas une donnée invalide → HTTP **409**, avec
    /// le code canonique du [`ReversalBlocker`] et l'identifiant de la pièce
    /// propriétaire quand il y en a une. Le message doit nommer la pièce ET le
    /// chemin de correction : un utilisateur qui lit « corrigez la facture
    /// F-2026-014 par un avoir » sait quoi faire, un « interdit » sec non.
    #[error("Écriture non contre-passable ({})", .blocker.code())]
    EntryNotReversable {
        blocker: ReversalBlocker,
        /// Identifiant de la pièce propriétaire, quand le motif en désigne une.
        document_id: Option<i64>,
        /// **Étiquette lisible** de ce qui bloque : le numéro de la pièce
        /// (`F-2026-014`, `AV-2026-3`…), ou le **numéro du compte** archivé.
        ///
        /// ⚠️ Un identifiant de base de données ne se comprend pas ; c'est ce que
        /// l'utilisateur voit sur son document, ou dans son plan comptable.
        document_label: Option<String>,
    },

    /// Un ou plusieurs comptes de l'écriture d'origine ont été **archivés**
    /// depuis (Story 24-4a, #380).
    ///
    /// ⛔ `enforce_postable = false` NE SUFFIT PAS : la clause `active = TRUE` de
    /// la validation des comptes est **inconditionnelle**, seule `postable` est
    /// gouvernée par le drapeau.
    ///
    /// Mappé vers HTTP **400** `ACCOUNT_ARCHIVED` — même statut que le gabarit
    /// [`DbError::CreditNoteRevenueAccountsArchived`] dont il reprend la forme.
    #[error("Comptes archivés sur l'écriture à contre-passer ({})", .0.len())]
    ReversalAccountsArchived(Vec<ArchivedAccount>),

    /// L'écriture a été contre-passée : on ne la supprime plus (Story 24-4a).
    ///
    /// Supprimer une écriture qu'on a corrigée **effacerait la correction** —
    /// exactement ce que l'art. 958f CO interdit. Le refus est donc voulu ; il
    /// est rendu explicite plutôt que laissé remonter comme une violation de
    /// clé étrangère au message opaque. Mappé vers HTTP **409**
    /// `ENTRY_IS_REVERSED`.
    #[error("Écriture contre-passée : suppression refusée")]
    EntryIsReversed,

    /// L'écriture est comptabilisée : elle ne se réécrit ni ne se supprime
    /// (Story 24-4b, #380).
    ///
    /// ⛔ Toute écriture l'est **dès son insertion** — il n'existe pas de statut
    /// brouillon, et la story n'en introduit pas. Le refus est donc
    /// inconditionnel, et c'est l'exigence de l'art. 958f CO : la correction
    /// doit être **apparente**, ce que seule la contre-passation permet.
    ///
    /// ⚠️ Ce refus vient **après** [`DbError::EntryIsReversed`] : sur une
    /// écriture déjà contre-passée, conseiller la contre-passation serait un
    /// conseil faux. Mappé vers HTTP **409** `ENTRY_IS_POSTED`.
    #[error("Écriture comptabilisée : modification et suppression refusées")]
    EntryIsPosted,

    /// La date de l'écriture tombe dans une période verrouillée
    /// (Story 24-4c, #380).
    ///
    /// ⛔ Le seuil est **inclusif** : une borne au 31.03 refuse le 31.03.
    ///
    /// ⚠️ C'est un **400**, pas un 409 : ce qui est invalide, c'est la **date
    /// proposée**, pas l'état d'une ressource qu'on voudrait changer. La 24-4b
    /// a figé l'asymétrie — `ENTRY_IS_POSTED` porte sur l'écriture qu'on veut
    /// modifier, `PERIOD_LOCKED` sur la date qu'on propose.
    ///
    /// Les deux dates voyagent avec l'erreur pour que le message les NOMME :
    /// un refus qui ne dit pas jusqu'où les livres sont fermés n'est pas
    /// utilisable.
    #[error(
        "Les écritures sont verrouillées jusqu'au {locked_through} ; celle-ci est datée du {attempted}"
    )]
    PeriodLocked {
        locked_through: chrono::NaiveDate,
        attempted: chrono::NaiveDate,
    },

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

    /// Entrée invalide détectée côté repository (validation métier qui
    /// nécessite un round-trip DB, ex. `paid_at` antérieur à `invoice.date`).
    /// Le payload est un code stable (non i18n) — le handler le mappe vers
    /// une clé i18n FTL. Mappé vers HTTP 400 côté API.
    #[error("Entrée invalide : {0}")]
    InvalidInput(String),

    /// Invariant du crate violé (ex: AUTO_INCREMENT retourne une valeur impossible).
    /// Indique un bug ou un état de DB corrompu, jamais une erreur utilisateur.
    #[error("Invariant kesh-db violé : {0}")]
    Invariant(String),

    /// Donnée trop longue pour sa colonne (MariaDB **1406** `ER_DATA_TOO_LONG`)
    /// ou hors de la plage du type (**1264** `ER_WARN_DATA_OUT_OF_RANGE`).
    /// Story 12-5c (dette D2) : un champ d'un QR tiers **non conforme SIX 2.2**
    /// (nom créancier > 70 chars, message > 140, etc.) dépasse la largeur de sa
    /// colonne à l'INSERT staging. Variante typée (séparée du repli `Sqlx`) pour
    /// que la couche d'ingestion 12-5c mappe cet échec **par-fichier** (`failed[]`
    /// avec `error_code = "FIELD_TOO_LONG"`, HTTP 200) au lieu d'un 500 global.
    #[error("Donnée trop longue ou hors plage : {0}")]
    DataLengthOrRange(String),

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
            Self::AccountRoleAlreadyAssigned { .. } => "ACCOUNT_ROLE_ALREADY_ASSIGNED",
            Self::AccountParentArchived { .. } => "ACCOUNT_PARENT_ARCHIVED",
            Self::AccountRoleInvalidForType { .. } => "ACCOUNT_ROLE_INVALID_FOR_TYPE",
            Self::InvalidRevenueAccounts(_) => "INVOICE_LINE_REVENUE_ACCOUNT_INVALID",
            Self::CreditNoteRevenueAccountsArchived(_) => "CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED",
            // ⚠️ Le code EXPOSÉ est celui du `ReversalBlocker` (huit valeurs) ;
            // celui-ci n'est que le repli générique du mapping structuré.
            Self::EntryNotReversable { .. } => "ENTRY_NOT_REVERSABLE",
            Self::ReversalAccountsArchived(_) => "ACCOUNT_ARCHIVED",
            Self::EntryIsReversed => "ENTRY_IS_REVERSED",
            Self::EntryIsPosted => "ENTRY_IS_POSTED",
            Self::PeriodLocked { .. } => "PERIOD_LOCKED",
            Self::FiscalYearInvalid => "FISCAL_YEAR_INVALID",
            Self::ConfigurationRequired(_) => "CONFIGURATION_REQUIRED",
            Self::ConnectionUnavailable(_) => "CONNECTION_UNAVAILABLE",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::Invariant(_) => "INVARIANT_VIOLATION",
            Self::DataLengthOrRange(_) => "DATA_LENGTH_OR_RANGE",
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
/// - **1406** : `ER_DATA_TOO_LONG` — donnée trop longue pour la colonne (D2 12-5c)
/// - **1264** : `ER_WARN_DATA_OUT_OF_RANGE` — valeur hors plage du type (D2 12-5c)
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

    if let Some(db_err) = err.as_database_error()
        && let Some(my_err) = db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
    {
        match my_err.number() {
            1062 => return DbError::UniqueConstraintViolation(my_err.message().to_string()),
            1451 | 1452 => {
                return DbError::ForeignKeyViolation(my_err.message().to_string());
            }
            4025 | 3819 => {
                return DbError::CheckConstraintViolation(my_err.message().to_string());
            }
            // Story 12-5c (D2) : donnée trop longue / hors plage à l'INSERT
            // staging d'un QR tiers non conforme → variante typée pour un
            // échec par-fichier propre (vs 500 global) côté ingestion 12-5c.
            1406 | 1264 => {
                return DbError::DataLengthOrRange(my_err.message().to_string());
            }
            _ => {}
        }
    }
    DbError::Sqlx(err)
}
