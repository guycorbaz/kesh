//! Erreur centrale de l'application et mapping HTTP.
//!
//! Toutes les fonctions du crate retournent `Result<T, AppError>`.
//! Le mapping `IntoResponse` transforme chaque variante en réponse
//! HTTP avec un code d'erreur structuré et un message générique côté
//! client (les détails internes vont exclusivement au logger).

use std::sync::RwLock;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use kesh_db::errors::DbError;
use kesh_i18n::{FluentArgs, I18nBundle, Locale};
use serde::Serialize;
use thiserror::Error;

/// Bundle i18n global pour les messages d'erreur.
/// `RwLock` au lieu de `OnceLock` pour permettre la réinitialisation en tests.
static I18N: RwLock<Option<(std::sync::Arc<I18nBundle>, Locale)>> = RwLock::new(None);

/// Initialise (ou remplace) le bundle i18n global pour les messages d'erreur.
pub fn init_error_i18n(bundle: std::sync::Arc<I18nBundle>, locale: Locale) {
    let mut guard = I18N.write().expect("I18N write lock");
    *guard = Some((bundle, locale));
}

/// Résout un message d'erreur via i18n, avec fallback sur le message par défaut.
///
/// Exposé `pub(crate)` pour permettre aux handlers de construire des messages
/// d'erreur localisés à la volée (ex. `InvoiceNotPdfReady` qui transporte un
/// message pré-rendu dans son payload).
pub(crate) fn t(key: &str, default: &str) -> String {
    let guard = I18N.read().expect("I18N read lock");
    match guard.as_ref() {
        Some((bundle, locale)) => bundle.format(locale, key, None),
        None => default.to_string(),
    }
}

/// Résout un message i18n avec arguments Fluent, fallback sur `default`.
fn t_args(key: &str, default: &str, args: &FluentArgs<'_>) -> String {
    let guard = I18N.read().expect("I18N read lock");
    match guard.as_ref() {
        Some((bundle, locale)) => bundle.format(locale, key, Some(args)),
        None => default.to_string(),
    }
}

/// Erreurs applicatives de kesh-api.
#[derive(Debug, Error)]
pub enum AppError {
    /// Identifiants invalides au login (username inconnu, mot de passe
    /// incorrect, user inactif) — message générique pour éviter toute
    /// énumération d'utilisateurs.
    #[error("Identifiants invalides")]
    InvalidCredentials,

    /// JWT manquant, mal formé, expiré ou signature invalide.
    /// Le `String` porte le détail pour les logs, jamais le client.
    #[error("Non authentifié : {0}")]
    Unauthenticated(String),

    /// Erreur de validation des données entrantes (400).
    #[error("Validation : {0}")]
    Validation(String),

    /// Erreur interne du serveur (bug, PHC mal formé, config invalide).
    #[error("Erreur interne : {0}")]
    Internal(String),

    /// Erreur remontée depuis la couche de persistance `kesh-db`.
    ///
    /// Le `#[from]` est légitime ici : la classification
    /// sqlx::Error → DbError a déjà eu lieu au niveau kesh-db. On se
    /// contente de wrapper pour le mapping HTTP.
    #[error("Erreur base de données : {0}")]
    Database(#[from] DbError),

    // --- Story 1.7 ---
    /// Accès interdit — rôle insuffisant (403).
    #[error("Accès interdit")]
    Forbidden,

    // --- Story 17-2a — API externe à clé PAT (#100) ---
    /// Une clé API `scope='read'` tente une méthode HTTP mutante
    /// (POST/PUT/PATCH/DELETE) → `403` code `API_KEY_READ_ONLY` (DC3/AC6).
    /// Rejet global en amont (méthode HTTP) — jamais encapsulé en
    /// `FailedProposal` sur les endpoints batch (F-OPUS-7, exception
    /// « 403 RBAC global » du §Pattern batch).
    #[error("Clé API en lecture seule")]
    ApiKeyReadOnly,

    /// Une requête authentifiée par PAT tente d'accéder aux routes de gestion
    /// des clés (`/settings/api-keys`) → `403` code
    /// `API_KEY_MANAGEMENT_FORBIDDEN` (DC6/AC7). Même une clé `read-write` ne
    /// peut pas lister/créer/révoquer des clés (anti auto-propagation d'une
    /// clé fuitée). Gestion réservée à la session JWT cookie (UI web).
    #[error("Gestion des clés API interdite via clé API")]
    ApiKeyManagementForbidden,

    /// L'administrateur tente de désactiver son propre compte (400).
    #[error("Impossible de désactiver son propre compte")]
    CannotDisableSelf,

    /// Tentative de désactivation du dernier administrateur actif (400).
    #[error("Impossible de désactiver le dernier administrateur")]
    CannotDisableLastAdmin,

    // --- Story v011-5 — onboarding self-service ---
    /// **Story v011-5 (AC #13 / 423 Locked)** : la table `users` est vide,
    /// le frontend doit rediriger vers `/setup` pour créer le 1er admin.
    /// Émis par `require_auth` middleware quand `state.users_exist.load() == false`.
    /// Code client `SETUP_REQUIRED`, HTTP 423 Locked (distinct du 401 « pas
    /// authentifié » — permet au frontend de distinguer « pas connecté » de
    /// « pas encore configuré »).
    #[error("Configuration initiale requise")]
    SetupRequired,

    /// **Story v011-5 (AC #10 / 410 Gone)** : `POST /api/v1/setup/admin`
    /// appelé après qu'un admin a déjà été créé (auto-disable). Code client
    /// `SETUP_ALREADY_COMPLETE`, HTTP 410 Gone. Le frontend redirige vers
    /// `/login` à la réception (cf. AC #17).
    #[error("Compte administrateur déjà créé")]
    SetupAlreadyComplete,

    // --- Story 1.6 ---
    /// Rate limiting déclenché : trop de tentatives de login depuis cette IP.
    /// `retry_after` = secondes avant déblocage, transmis dans le header `Retry-After`.
    #[error("Rate limited, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    /// Refresh token invalide (expiré, révoqué, inconnu, user inactif).
    /// Code client unique `INVALID_REFRESH_TOKEN` (anti-enumeration).
    /// Le `String` porte le détail pour les logs serveur.
    #[error("Refresh token invalide : {0}")]
    InvalidRefreshToken(String),

    // --- Story 2.2 ---
    /// Tentative de progression sur un step d'onboarding déjà complété (400).
    #[error("Étape d'onboarding déjà complétée")]
    OnboardingStepAlreadyCompleted,

    /// Reset d'onboarding refusé par policy (production sans `KESH_PRODUCTION_RESET=1`,
    /// ou production user au-delà du step 2). Distinct de `OnboardingStepAlreadyCompleted`
    /// pour donner un signal actionnable au client (cf. Story 7-1, P6-L8).
    /// HTTP 403 Forbidden — code unique `ONBOARDING_RESET_FORBIDDEN`.
    #[error("Reset d'onboarding refusé par la configuration (production)")]
    OnboardingResetForbidden,

    // --- Story 3.2 ---
    /// Écriture comptable déséquilibrée (FR21).
    /// Les totaux (format string décimal) sont inclus dans le message
    /// client pour respecter exactement le wording du PRD.
    #[error("Écriture déséquilibrée : débits={debit}, crédits={credit}")]
    EntryUnbalanced {
        /// Total des débits formaté en string décimal.
        debit: String,
        /// Total des crédits formaté en string décimal.
        credit: String,
    },

    /// Aucun exercice comptable n'existe pour la date fournie.
    /// À distinguer de `FiscalYearClosed` pour l'UX : le message invite
    /// l'utilisateur à créer un exercice plutôt qu'à chercher un exercice
    /// existant fermé.
    #[error("Aucun exercice pour la date {date}")]
    NoFiscalYear {
        /// Date au format ISO (YYYY-MM-DD).
        date: String,
    },

    /// L'exercice pour cette date est clôturé (FR24, CO art. 957-964).
    /// Aucune écriture ne peut être ajoutée ou modifiée dans un exercice clos.
    #[error("Exercice clôturé pour la date {date}")]
    FiscalYearClosed {
        /// Date au format ISO (YYYY-MM-DD).
        date: String,
    },

    /// La nouvelle date d'une écriture ne tombe pas dans l'exercice courant
    /// de l'entité (story 3.3). Empêche le déplacement cross-exercice via
    /// simple édition.
    #[error("Date hors exercice courant : {date}")]
    DateOutsideFiscalYear {
        /// Date au format ISO (YYYY-MM-DD).
        date: String,
    },

    // --- Story 4.1 ---
    /// Un contact avec ce numéro IDE (CHE) existe déjà dans la même company.
    /// Code client dédié (`IDE_ALREADY_EXISTS`) pour UX précise côté form,
    /// distinct du générique `RESOURCE_CONFLICT` (autres UniqueConstraintViolation).
    /// Le `String` porte le message i18n prêt à afficher.
    #[error("{0}")]
    IdeAlreadyExists(String),

    // --- Story 5.3 — génération PDF QR Bill ---
    /// La facture n'est pas validée — impossible de générer un PDF (400).
    #[error("Facture non validée")]
    InvoiceNotValidated,

    /// Un pré-requis applicatif manque pour générer le PDF : adresse contact,
    /// compte bancaire primary, IBAN invalide, etc. Le `String` contient la
    /// description i18n renvoyée au client.
    #[error("Facture non prête pour PDF : {0}")]
    InvoiceNotPdfReady(String),

    /// Trop de lignes pour tenir sur un PDF A4 (v0.1 : limite réelle dans
    /// `routes::invoice_pdf_service::MAX_LINES_PER_PDF` = 9). Le `usize` est la
    /// taille effective, affichée dans le message.
    #[error("Facture trop de lignes pour PDF : {0}")]
    InvoiceTooManyLinesForPdf(usize),

    /// Échec interne de la génération PDF (bug crate, I/O). Le détail est
    /// loggé mais jamais exposé au client (500).
    #[error("Échec génération PDF : {0}")]
    PdfGenerationFailed(String),

    /// Story 9-2a + Pass 1 code-review H1 — Échec interne de la génération CSV
    /// (bug crate `csv`, I/O flush, BOM write). Variant dédié pour i18n message
    /// client utile (« Échec génération CSV » au lieu du « Échec génération PDF »
    /// affiché à tort par le mapping initial). Code HTTP 500, i18n key
    /// `error-csv-generation-failed`. Le détail est loggé mais jamais exposé
    /// au client.
    #[error("Échec génération CSV : {0}")]
    CsvGenerationFailed(String),

    /// Story 9-2b §error-variant — Échec interne du packaging ZIP de l'export
    /// global souveraineté (sérialisation CSV per-table, calcul SHA-256,
    /// écriture ZIP, panne pool DB). Distinct de [`AppError::CsvGenerationFailed`]
    /// car la sémantique côté client est différente (export ZIP ≠ export CSV
    /// d'un rapport). HTTP 500, i18n key `error-global-export-failed`. Le
    /// détail est loggé (ops debug) mais jamais exposé en HTTP body (UX-DR38 :
    /// message client générique actionable + diagnostic serveur).
    #[error("Échec génération export global : {0}")]
    GlobalExportFailed(String),

    /// Story 17-3a — échec de génération du `.keshbackup` (export complet
    /// d'installation : sérialisation NDJSON des 22 tables, SHA-256, ZIP,
    /// panne pool DB, IO fichier temporaire). HTTP 500, i18n key
    /// `error-admin-full-export-failed`. Détail loggé, jamais exposé en HTTP body.
    #[error("Échec génération export installation : {0}")]
    AdminFullExportFailed(String),

    /// Story 17-3c — échec de l'**import** complet d'installation (panne DB
    /// pendant le restore transactionnel, backup pré-import impossible, dataset
    /// source sans aucun compte Admin). HTTP 500 ; sur échec transactionnel,
    /// la destination reste intacte (rollback) + backup pré-import disponible.
    /// Détail loggé, jamais exposé en HTTP body.
    #[error("Échec import installation : {0}")]
    AdminFullImportFailed(String),

    /// Story 17-4b — échec d'envoi d'un email transactionnel via SMTP
    /// (connexion SMTP, auth, build du message, panne réseau). HTTP 500, i18n
    /// key `error-smtp-send-failed`. Détail loggé `tracing::error!`, jamais
    /// exposé en HTTP body. **Note 17-4c** : sur le flux forgot-password,
    /// l'envoi est fire-and-forget (`tokio::spawn` détaché, DC4) — ce variant y
    /// est seulement loggé côté serveur, jamais propagé au client (anti-énum).
    #[error("Échec envoi email SMTP : {0}")]
    SmtpSendFailed(String),

    /// Story 20-3b1 — le transport SMTP n'est pas configuré/prêt
    /// (`AppState.smtp_ready == false`) : l'envoi d'e-mails métier est
    /// indisponible. HTTP 412 `SMTP_NOT_CONFIGURED`, i18n key
    /// `error-smtp-not-configured`. Garde impérative : sans elle, le
    /// `NoopMailer` retournerait Ok et la facture serait marquée « envoyée »
    /// à tort.
    #[error("SMTP non configuré — envoi d'e-mails indisponible")]
    SmtpNotConfigured,

    /// Story 20-3b1 — le contact de la facture n'a pas d'adresse e-mail
    /// (destinataire verrouillé = `contacts.email`, décision #13 epic-20).
    /// HTTP 400 `CONTACT_EMAIL_MISSING`, i18n key `error-contact-email-missing`.
    #[error("Le contact n'a pas d'adresse e-mail")]
    ContactEmailMissing,

    /// Story 20-3b1 — objet ou corps vide (après trim) au moment de l'envoi
    /// manuel. HTTP 422 `INVOICE_EMAIL_EMPTY_CONTENT`, i18n key
    /// `error-invoice-email-empty-content`.
    #[error("Objet ou corps de l'e-mail vide")]
    InvoiceEmailEmptyContent,

    /// Story 21-5a — enregistrement d'un rappel sur une facture déjà payée. 422.
    #[error("Facture déjà payée")]
    InvoiceAlreadyPaid,
    /// Story 21-5a — niveau de rappel demandé absent de la configuration. 422.
    #[error("Niveau de rappel inexistant")]
    DunningLevelNotFound,
    /// Story 21-5a — date d'envoi d'un rappel manuel dans le futur. 422.
    #[error("Date de rappel dans le futur")]
    ReminderDateInFuture,
    /// Story 21-5a — reprise d'une facture non suspendue. 422.
    #[error("Facture non suspendue")]
    InvoiceNotPaused,

    /// Story 20-3b1 (code review Pass 1 ECH-1) — le contact de la facture est
    /// archivé (`active = false`) : le carnet d'adresses le considère « à ne
    /// plus utiliser », on n'envoie pas de facture à son adresse. HTTP 400
    /// `CONTACT_ARCHIVED`, i18n key `error-contact-archived`.
    #[error("Le contact de la facture est archivé")]
    ContactArchived,

    /// Story 20-3b1 (code review Pass 1 ECH-1/BH-3) — l'e-mail a été REMIS au
    /// relay SMTP, mais la facture a disparu (supprimée #219) entre l'envoi et
    /// le marquage `emailed_at`. Le message dit explicitement que l'e-mail est
    /// parti pour dissuader un renvoi en double. HTTP 409
    /// `EMAIL_SENT_INVOICE_GONE`, i18n key `error-email-sent-invoice-gone`.
    #[error("E-mail envoyé mais facture disparue avant le marquage")]
    EmailSentInvoiceGone,

    /// Story 17-4c — lien de réinitialisation de mot de passe invalide ou expiré.
    /// HTTP 400 `INVALID_OR_EXPIRED_TOKEN`, i18n key `error-invalid-or-expired-token`.
    /// **Anti-fuite (DC4)** : couvre de manière indistincte les trois cas
    /// (token inconnu / expiré / déjà utilisé) ainsi que la perte de course
    /// concurrente sur `mark_used` (`DbError::NotFound`). Message générique, pas
    /// de signal permettant de distinguer ces cas.
    #[error("Lien de réinitialisation invalide ou expiré")]
    InvalidOrExpiredToken,

    /// Story 17-3c — `.keshbackup` structurellement invalide ou corrompu :
    /// ZIP malformé, `manifest.json` absent/illisible, `files/` non-vide,
    /// `formatVersion > 1`, NDJSON d'une table absent, ou SHA-256 d'une table
    /// ne correspondant pas au manifeste (tamper). HTTP 400 — refusé **avant
    /// tout DELETE** (la DB n'est jamais mutée).
    #[error("Backup invalide : {0}")]
    InvalidBackupStructure(String),

    /// Story 17-3c — incompatibilité de schéma source↔destination (AC12c) :
    /// colonne source inconnue de la destination (`unknown_columns`) ou colonne
    /// destination `NOT NULL` sans défaut absente de la source
    /// (`missing_required_columns`). HTTP 400 `IMPORT_SCHEMA_MISMATCH` avec
    /// `details: { table, unknownColumns, missingRequiredColumns }`.
    #[error("Schéma incompatible (table {table})")]
    ImportSchemaMismatch {
        table: String,
        unknown_columns: Vec<String>,
        missing_required_columns: Vec<String>,
    },

    /// Story 17-3c — la version minimale requise du backup est plus récente que
    /// le binaire destination (downgrade impossible, DC4 = sémantique 10-2).
    /// HTTP 409 `IMPORT_VERSION_INCOMPATIBLE` avec `details: { sourceMinRequired,
    /// binaryVersion }`.
    #[error(
        "Version incompatible : source exige >= {source_min_required}, binaire {binary_version}"
    )]
    ImportVersionIncompatible {
        source_min_required: String,
        binary_version: String,
    },

    // --- Story 5.4 — Échéancier factures ---
    /// Dépassement du plafond d'export (> 10'000 lignes en v0.1) — 400.
    /// Code client dédié pour permettre au frontend de proposer un raffinage
    /// des filtres (distinct de `VALIDATION_ERROR` générique).
    #[error("Résultat trop volumineux : {0}")]
    ResultTooLarge(String),

    // --- Story 8-1b — Import bancaire CAMT.053 (T6.4) ---
    /// Fichier upload > `KESH_BANK_IMPORT_MAX_MB` MiB → `413`.
    #[error("Fichier trop volumineux")]
    BankImportTooLarge,

    /// XML CAMT.053 mal formé / version inconnue / champ requis manquant /
    /// montant ou date invalide. Le `String` porte le détail (chemin
    /// indexé `stmt[i].ntry[j].field` pour `MissingRequiredField`).
    /// Mappe toutes les variantes `CamtError` vers un seul code HTTP `400`
    /// avec un sous-code dans `details.kind`.
    #[error("Fichier CAMT.053 invalide : {kind} — {message}")]
    BankImportParseFailed {
        /// Sous-code (`MALFORMED_XML`, `UNSUPPORTED_VERSION`,
        /// `MISSING_FIELD`, `INVALID_AMOUNT`, `INVALID_DATE`).
        kind: &'static str,
        message: String,
    },

    /// Solde déclaré incohérent (CR-010 #62, sans `confirmBalanceMismatch`)
    /// → `422`. Les 4 montants sont exposés en `details` pour permettre à
    /// l'UX d'afficher le delta.
    #[error("Solde de clôture incohérent (écart {diff})")]
    BankImportBalanceMismatch {
        opening: String,
        closing: String,
        sum: String,
        diff: String,
    },

    /// Devise non supportée v0.1 (autre que CHF) → `422`.
    #[error("Devise non supportée v0.1 : {0}")]
    BankImportUnsupportedCurrency(String),

    /// Aucun `<Stmt>` du fichier ne matche l'IBAN du `bankAccountId`
    /// sélectionné (multi-stmt, F4 validate Pass 1) → `422`.
    /// `found_ibans` aide l'utilisateur à corriger le compte cible.
    #[error("Aucun statement ne correspond au compte sélectionné")]
    BankImportNoMatchingStatement { found_ibans: Vec<String> },

    /// Story 8-3 — fichier déjà importé pour cette company sans
    /// `confirmDuplicateFile=true`. **Code HTTP changé 8-3 : 409 → 422**
    /// (cohérence avec autres `BankImport*` qui sont aussi des refus
    /// métier, cf. §confirm-flags). Le check applicatif via
    /// `bank_imports::find_by_company_and_hash` (transaction-bound)
    /// remplace la contrainte UNIQUE retirée par la migration
    /// `20260507000001_bank_imports_relax_hash_unique.sql`.
    #[error("Fichier déjà importé")]
    BankImportDuplicateFile {
        existing_import_id: i64,
        existing_filename: String,
    },

    /// Le `bankAccountId` fourni n'existe pas / appartient à une autre
    /// company → `404` (jamais `403`, pattern KF-002 anti-énumération).
    ///
    /// Story 8-5a-zero (F4''') : variant réutilisé pour le PATCH
    /// `/bank-accounts/{id}`. Le code HTTP `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`
    /// est ancré sémantiquement « bank-imports » mais émis aussi sur PATCH
    /// `/bank-accounts/{id}` v0.1 (dette de naming L64 documentée). v0.2 :
    /// renommer le code en `BANK_ACCOUNT_NOT_FOUND` (breaking client) ou
    /// créer un variant dédié si le frontend distingue les contextes.
    #[error("Compte bancaire non trouvé")]
    BankAccountNotFound,

    // ----- Story 8-5a-zero — bank_account.journal_account_id link -----
    /// Le compte du plan comptable référencé par `journalAccountId` n'existe
    /// pas, est archivé, ou appartient à une autre company → `404`
    /// `ACCOUNT_NOT_FOUND` (anti-énumération KF-002 — jamais 403).
    ///
    /// **Story 8-5a-bis (F1''' Pass 3 Opus)** : extension `missing_account_ids`
    /// pour le cas batch (split N comptes contreparties). Quand `Some(vec)`,
    /// `account_id` est le premier id de `vec` trié et `details.missingAccountIds`
    /// est inclus dans le body JSON (cohérent §validation-handler-side-split
    /// step 5). Quand `None` (manual + bank-accounts PATCH), seul
    /// `details.accountId` est inclus — rétro-compat 8-5a-zero / 8-5a-base.
    #[error("Compte du plan comptable non trouvé : id={account_id}")]
    AccountNotFound {
        account_id: i64,
        missing_account_ids: Option<Vec<i64>>,
    },

    /// Le compte référencé n'est pas de type Asset ou Liability → `400`
    /// `INVALID_ACCOUNT_TYPE`. Un bank_account ne peut être lié qu'à un
    /// compte d'actif (1020 Caisse, 1030 Banque) ou de passif rare (2100
    /// découvert chronique). Revenue/Expense rejetés (cf. §validation-account-type).
    #[error("Type de compte invalide : {account_type} (Asset|Liability requis)")]
    InvalidAccountType {
        account_id: i64,
        account_type: String,
    },

    // ----- Story 8-2 — Bank profiles + CSV import -----
    /// Profil CSV introuvable / cross-tenant (pattern KF-002) → `404`.
    /// Aussi utilisé pour `auto-match aucun profil ne matche le filename`
    /// (cf. §profile-matching).
    #[error("Profil bancaire introuvable")]
    BankCsvProfileNotFound {
        available_profiles: Vec<BankProfileSummary>,
    },

    /// Encoding détecté n'est pas dans `{UTF-8, ISO-8859-1}` v0.1 → `422`.
    #[error("Encoding non supporté v0.1")]
    BankCsvUnsupportedEncoding { detected: Option<String> },

    /// Encoding détecté diverge de celui du profil sans
    /// `confirmEncodingMismatch=true` → `422` (Pass 1 H5).
    #[error("Encoding mismatch profil vs détecté")]
    BankCsvEncodingMismatch { profile: String, detected: String },

    /// Au moins une ligne du CSV échoue au parsing → `422` strict reject
    /// FR51. Liste les erreurs (cap 100, Pass 2 H'1).
    ///
    /// Story 8-3 — `reason` optionnel discrimine le cas
    /// `"no_valid_lines_to_commit"` (`confirmPartialImport=true` sur
    /// CSV avec 0 lignes valides — AC #16).
    #[error("Échec parsing CSV partiel")]
    BankCsvParsePartialFailure {
        lines: Vec<CsvLineErrorPayload>,
        total_errors: usize,
        truncated: bool,
        reason: Option<&'static str>,
    },

    /// Validation profil failed (XOR, séparateurs distincts, regex
    /// invalide, chrono format, longueurs, etc.) → `422`.
    #[error("Profil bancaire invalide : {0}")]
    BankCsvProfileValidation(String),

    /// UNIQUE `(company_id, bank_name)` violation → `409`.
    #[error("Profil bancaire dupliqué (bank_name déjà utilisé)")]
    BankCsvProfileDuplicate,

    /// Profil DB corrompu (column_mapping JSON invalide ou indices
    /// hors-borne sur le 1er record) → `500` (jamais utilisateur).
    #[error("Profil bancaire mal configuré : {0}")]
    BankCsvProfileMisconfigured(String),

    /// Fichier CSV vide ou 0 lignes de données après header skip → `422`.
    #[error("Fichier CSV vide : {reason}")]
    BankCsvEmptyFile { reason: String },

    /// Format de fichier non supporté (ni CAMT ni CSV) → `415`.
    #[error("Format de fichier non supporté")]
    BankImportUnsupportedFormat,

    // ----- Story 8-4 — Reconciliation -----
    /// Advisory lock `GET_LOCK('reconcile:{company}:{account}', timeout)`
    /// non acquis dans les `timeout_secs` secondes → `409`.
    #[error("Compte verrouillé par une autre opération de réconciliation")]
    ReconciliationAccountLocked {
        bank_account_id: i64,
        timeout_secs: u32,
    },

    /// `RELEASE_LOCK` failure (HP3-1 Pass 3 + HP4-1/HP5-1 Pass 4-5
    /// caller pattern correction) → `500`. Le lock advisory restera
    /// tenu jusqu'à fin de session MariaDB (cf. L22).
    #[error("Échec de libération du verrou de réconciliation")]
    ReconciliationLockReleaseFailed { bank_account_id: i64 },

    // ----- Story 8-5a-base — réconciliation manuelle FR45 -----
    /// Le `bank_account` ciblé n'a pas de `journal_account_id`
    /// configuré (8-5a-zero foundation). Le user doit configurer le
    /// compte comptable lié via la page `/bank-accounts` avant
    /// d'utiliser le manual match → `412 Precondition Failed`,
    /// code `BANK_ACCOUNT_NOT_CONFIGURED`. Body inclut
    /// `details.bankAccountId` + `details.hint` lien UX.
    #[error("Compte bancaire non configuré (journal_account_id manquant) : id={bank_account_id}")]
    BankAccountNotConfigured { bank_account_id: i64 },

    // ----- Story v014-1 — CRUD bank_accounts post-onboarding -----
    /// Tentative d'archivage d'un compte bancaire qui a encore des
    /// `bank_transactions` associées → `412 Precondition Failed`, code
    /// `BANK_ACCOUNT_HAS_TRANSACTIONS`, body inclut `details.transactionCount`.
    /// Story v014-1 AC#8 — refus inconditionnel toutes statuts confondus
    /// (auditabilité CO Art. 958f).
    #[error("Compte bancaire avec {transaction_count} transaction(s) — archivage refusé")]
    BankAccountHasTransactions { transaction_count: i64 },

    /// Tentative d'archivage du compte principal alors qu'au moins un autre
    /// compte non-archivé existe → `412 Precondition Failed`, code
    /// `BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY`. AC#9 — l'utilisateur doit
    /// d'abord transférer le primary à un autre compte.
    #[error("Compte principal — définir un autre compte comme principal avant d'archiver celui-ci")]
    BankAccountCannotArchivePrimary,

    /// Tentative d'utiliser le CRUD `/api/v1/bank-accounts` pendant
    /// l'onboarding (step < 7) → `412 Precondition Failed`, code
    /// `ONBOARDING_NOT_COMPLETE`. Story v014-1 AC#2 — pendant l'onboarding,
    /// utiliser `POST /api/v1/onboarding/bank-account`.
    #[error("L'onboarding doit être terminé avant de gérer les comptes bancaires")]
    OnboardingNotComplete,

    /// L'exercice fiscal couvrant `entry_date` est inexistant ou
    /// `Closed` lors du flow `/reconciliation/manual` (8-5a-base) →
    /// `409 RECONCILIATION_FISCAL_YEAR_CLOSED`.
    ///
    /// **Distinct de `AppError::FiscalYearClosed { date: String }`**
    /// (Story 3-4 — variant générique journal_entries, mappe → 400
    /// avec code `FISCAL_YEAR_CLOSED`). Le variant reconciliation
    /// utilise `entry_date: NaiveDate` (pas String) cohérent avec
    /// `ReconciliationError::FiscalYearClosed { entry_date }` qui
    /// remonte directement depuis la closure `with_account_lock`.
    #[error("Exercice fiscal clos pour la date {entry_date}")]
    ReconciliationFiscalYearClosed { entry_date: chrono::NaiveDate },

    /// Le `bank_transaction` ciblé n'est pas (ou plus) en status
    /// `pending` — couvre 4 cas distincts en un code
    /// (`find_strictly_pending_by_id_for_account` retourne `None`) :
    /// (a) tx introuvable, (b) tx déjà `reconciled`, (c) tx
    /// cross-tenant, (d) tx cross-account. → `404
    /// RECONCILIATION_TRANSACTION_NOT_PENDING` (anti-énumération
    /// KF-002 — pas 409 puisque le helper ne distingue pas les causes).
    #[error("Transaction bancaire non pending : id={bank_transaction_id}")]
    ReconciliationTransactionNotPending { bank_transaction_id: i64 },

    // ----- Story 8-5a-bis — split FR48 -----
    /// `sum(splits) != tx.amount.abs()` (Decimal exact, pas de tolérance)
    /// → `400 RECONCILIATION_SPLIT_IMBALANCE`. Body `details = {
    /// expected, actual, difference }` (string Decimal cohérent AC #95).
    #[error(
        "Éclatement de transaction non équilibré : attendu {expected}, reçu {actual}, écart {difference}"
    )]
    ReconciliationSplitImbalance {
        expected: rust_decimal::Decimal,
        actual: rust_decimal::Decimal,
        difference: rust_decimal::Decimal,
    },
    //
    // M6 Pass 1 code review — variants supprimés :
    // - `ReconciliationAlreadyReconciled { bank_transaction_id }` :
    //   le cas est représenté par `FailedProposal { error_code:
    //   "RECONCILIATION_ALREADY_RECONCILED" }` per-proposal au lieu
    //   de remonter en `AppError` global → variant dead code.
    // - `ReconciliationInvoiceNotEligible { invoice_id, reason }` :
    //   idem, le 409 invoice-eligibility est représenté en
    //   `FailedProposal` per-proposal (4 reasons enum dans
    //   `details.reason`).

    // ----- Story 8-5b — rules engine FR47 -----
    /// `reconciliation_rule` ciblée par `find_by_id_for_company`
    /// retourne `None` OU rule archivée (active=false) lors du flow
    /// `accept-with-rule` step 2 ou des handlers PATCH/DELETE /rules
    /// → `404 RECONCILIATION_RULE_NOT_FOUND`.
    #[error("Règle de réconciliation introuvable : id={rule_id}")]
    ReconciliationRuleNotFound { rule_id: i64 },

    /// Création/réactivation d'une rule active avec mêmes `(company_id,
    /// match_type, match_value)` qu'une autre rule active existante
    /// (violation `uq_reconciliation_rules_match_active`). Détecté via
    /// [`kesh_db::repositories::reconciliation_rules::is_duplicate_rule_constraint`]
    /// → `409 RECONCILIATION_RULE_DUPLICATE`. Body inclut
    /// `details.matchType` + `details.matchValue`.
    #[error(
        "Règle de réconciliation déjà existante : match_type={match_type}, match_value={match_value}"
    )]
    ReconciliationRuleDuplicate {
        match_type: String,
        match_value: String,
    },

    /// **Pass 4 LOW#6 annotation** : variant défini pour complétude
    /// `From<ReconciliationError>` mais **jamais émis comme HTTP 400
    /// global** — `accept_one_rule` retourne `Err(FailedProposal {
    /// error_code: "RECONCILIATION_RULE_MISMATCH" })` per-proposal.
    /// Garder le variant pour conversion exhaustive ; il est en
    /// pratique unreachable côté handler global.
    #[error("Règle de réconciliation : compte mismatch (rule_id={rule_id})")]
    ReconciliationRuleMismatch { rule_id: i64 },

    /// Idem `ReconciliationRuleMismatch` — variant exhaustif jamais
    /// émis comme HTTP global. `accept_one_rule` retourne
    /// `FailedProposal { error_code: "RECONCILIATION_RULE_NO_LONGER_MATCHES" }`
    /// per-proposal.
    #[error("Règle de réconciliation : ne match plus la transaction (rule_id={rule_id})")]
    ReconciliationRuleNoLongerMatches { rule_id: i64 },

    // --- Story 9-1 (Rapports comptables) ---
    /// L'exercice fiscal demandé n'existe pas (ou n'appartient pas à la company
    /// du current user). 404 `FISCAL_YEAR_NOT_FOUND`.
    #[error("Exercice fiscal introuvable (fiscal_year_id={fiscal_year_id})")]
    ReportFiscalYearNotFound { fiscal_year_id: i64 },

    /// La période demandée dépasse les bornes de l'exercice fiscal. Body 400
    /// avec `details: { fyStart, fyEnd, requestedStart, requestedEnd }`.
    ///
    /// Pass 3 BH3-03 : ce variant utilise un body JSON ad-hoc divergent du
    /// pattern `build_response` standard car `ErrorBody` n'a pas de champ
    /// `details`. Refactor v0.2 (L69 dette tracée).
    #[error(
        "Période hors exercice : start={requested_start} end={requested_end} \
         fy=[{fy_start};{fy_end}]"
    )]
    ReportPeriodOutOfFiscalYear {
        fy_start: chrono::NaiveDate,
        fy_end: chrono::NaiveDate,
        requested_start: chrono::NaiveDate,
        requested_end: chrono::NaiveDate,
    },

    // --- Story 12-5c — import répertoire de factures (#194) ---
    /// Un import du répertoire inbox est **déjà en cours** (verrou de run F6 non
    /// acquis) → `409 INBOX_IMPORT_ALREADY_RUNNING`. Exception globale du pattern
    /// batch (un refus en amont du traitement per-fichier) — distinct d'un rapport
    /// `{accepted, failed}` partiel, pour que 12-5d le discrimine dans son `catch`.
    #[error("Un import du répertoire est déjà en cours")]
    InboxImportAlreadyRunning,

    /// `POST /imported-supplier-invoices/{id}/complete` (ou `/discard`) sur une
    /// row dont le `status != 'to_complete'` → `409 IMPORT_NOT_PENDING_COMPLETION`,
    /// `details: { currentStatus }`. Permet à 12-5d de distinguer « déjà
    /// complétée/écartée » d'une erreur serveur 500.
    #[error("Facture importée non en attente de complétion (statut : {current_status})")]
    ImportNotPendingCompletion { current_status: String },

    /// Rejet métier d'une complétion (steps 3/4/6 pré-`create_in_tx`) → `400`
    /// avec un `error_code` **canonique distinct** (`CURRENCY_NOT_SUPPORTED`,
    /// `IBAN_REFERENCE_MISMATCH`, `AMOUNT_MISMATCH`) pour que 12-5d guide
    /// l'utilisateur sans parser le message. Mono-item (pas un `FailedProposal`).
    /// `details` optionnel (ex. montants de la réconciliation).
    #[error("Complétion refusée [{error_code}] : {message}")]
    ImportCompletionRejected {
        error_code: &'static str,
        message: String,
        details: Option<serde_json::Value>,
    },

    /// `GET .../source-document` : la facture n'a **pas** de justificatif stocké
    /// (row absente, ou facture créée directement 12-2 sans import — L5) →
    /// `404 SOURCE_DOCUMENT_NOT_FOUND`. JAMAIS 500.
    #[error("Justificatif introuvable")]
    SourceDocumentNotFound,

    /// `POST .../complete` ou `.../discard` : la facture **importée** (staging)
    /// n'existe pas pour la company courante (id inconnu ou cross-company IDOR) →
    /// `404 IMPORTED_INVOICE_NOT_FOUND`. Distinct de `SourceDocumentNotFound`
    /// (download d'un justificatif) — un même code sur deux endpoints sémantiques
    /// différents tromperait le `catch` du frontend 12-5d (code-review 12-5c
    /// EC1/BH2/AA4, consensus 3 reviewers).
    #[error("Facture importée introuvable")]
    ImportedInvoiceNotFound,

    /// `GET .../source-document` : la **métadonnée** existe mais le fichier sur
    /// disque est absent (restore métadonnée-seule L1/F7) → `410 SOURCE_DOCUMENT_GONE`.
    #[error("Justificatif non restauré")]
    SourceDocumentGone,

    // --- Story 20-1 — socle templates d'e-mail (#224) ---
    /// `PUT /admin/email-templates/{type}/{language}` : `subject`/`body`
    /// référencent un ou plusieurs tokens `{var}` hors de
    /// `EmailTemplateType::allowed_variables()` → `422`. Les tokens
    /// inconnus sont exposés en `details.unknownVariables` pour que le
    /// futur éditeur Admin (Story 20-2) les surligne sans parser le message.
    #[error("Template invalide : variables inconnues {unknown_vars:?}")]
    EmailTemplateUnknownVariables { unknown_vars: Vec<String> },
}

// --- Story 9-1 : From<ReportError> for AppError ---

impl From<kesh_report::errors::ReportError> for AppError {
    fn from(err: kesh_report::errors::ReportError) -> Self {
        use kesh_report::errors::ReportError;
        match err {
            ReportError::Db(db_err) => AppError::Database(db_err),
            ReportError::FiscalYearNotFound { fiscal_year_id } => {
                AppError::ReportFiscalYearNotFound { fiscal_year_id }
            }
            // Story 19-6a — projet inconnu/cross-company → 404 (cohérent
            // routes/projects.rs qui mappe get_for_company None → DbError::NotFound).
            ReportError::ProjectNotFound { .. } => {
                AppError::Database(kesh_db::errors::DbError::NotFound)
            }
            ReportError::PeriodInvalid { reason } => AppError::Validation(reason),
            ReportError::PeriodOutOfFiscalYear {
                fy_start,
                fy_end,
                requested_start,
                requested_end,
            } => AppError::ReportPeriodOutOfFiscalYear {
                fy_start,
                fy_end,
                requested_start,
                requested_end,
            },
            ReportError::TrialBalanceUnbalanced {
                total_debit,
                total_credit,
            } => {
                tracing::error!(
                    %total_debit,
                    %total_credit,
                    "trial_balance unbalanced — invariant cassé"
                );
                AppError::Internal(format!(
                    "trial balance unbalanced: debit={total_debit} credit={total_credit}"
                ))
            }
            // Story 9-2a T2.0 + Pass 3 ECH3-H2 : variant dédié pour i18n message
            // client utile (cohérent kesh-qrbill::QrBillError::PdfGeneration mapping
            // dans invoice_pdf_service.rs).
            ReportError::PdfGeneration(detail) => AppError::PdfGenerationFailed(detail),
            // Story 9-2a + Pass 1 code-review H1 : variant dédié — sinon un
            // échec CSV se présentait à tort comme un échec PDF côté UI.
            ReportError::CsvGeneration(detail) => {
                tracing::error!(%detail, "CSV generation failed");
                AppError::CsvGenerationFailed(detail)
            }
        }
    }
}

/// Résumé d'un profil pour le payload `BankCsvProfileNotFound`
/// (cap 50 entrées par §profile-matching Pass 1 M11).
#[derive(Debug, Clone, Serialize)]
pub struct BankProfileSummary {
    pub id: i64,
    #[serde(rename = "bankName")]
    pub bank_name: String,
}

/// Payload structuré pour `BankCsvParsePartialFailure.lines`.
/// Sérialisé directement dans `details.lines` du JSON 422.
#[derive(Debug, Clone, Serialize)]
pub struct CsvLineErrorPayload {
    pub line: usize,
    pub code: String,
    pub value: Option<String>,
    #[serde(rename = "messageI18nKey")]
    pub message_i18n_key: String,
}

/// Structure de la réponse d'erreur JSON renvoyée au client.
#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

/// Helper pour construire une `Response` JSON structurée.
fn build_response(status: StatusCode, code: &'static str, message: &str) -> Response {
    (
        status,
        Json(ErrorBody {
            error: ErrorDetail {
                code,
                message: message.to_string(),
            },
        }),
    )
        .into_response()
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InvalidCredentials => build_response(
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                &t("error-invalid-credentials", "Identifiants invalides"),
            ),

            AppError::Unauthenticated(detail) => {
                tracing::warn!("unauth: {detail}");
                build_response(
                    StatusCode::UNAUTHORIZED,
                    "UNAUTHENTICATED",
                    &t("error-unauthenticated", "Non authentifié"),
                )
            }

            AppError::Validation(msg) => {
                build_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &msg)
            }

            // Story v011-5 — onboarding self-service.
            AppError::SetupRequired => build_response(
                StatusCode::LOCKED,
                "SETUP_REQUIRED",
                &t(
                    "error-setup-required",
                    "Configuration initiale requise. Créer le compte administrateur via /setup.",
                ),
            ),

            AppError::SetupAlreadyComplete => build_response(
                StatusCode::GONE,
                "SETUP_ALREADY_COMPLETE",
                &t(
                    "error-setup-already-complete",
                    "Le compte administrateur a déjà été créé.",
                ),
            ),

            AppError::Forbidden => build_response(
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                &t("error-forbidden", "Accès interdit"),
            ),

            // Story 17-2a — clé API en lecture seule (DC3/AC6).
            AppError::ApiKeyReadOnly => build_response(
                StatusCode::FORBIDDEN,
                "API_KEY_READ_ONLY",
                &t(
                    "error-api-key-read-only",
                    "Cette clé API est en lecture seule (scope read). Seules les requêtes GET sont autorisées.",
                ),
            ),

            // Story 17-2a — gestion des clés interdite via PAT (DC6/AC7).
            AppError::ApiKeyManagementForbidden => build_response(
                StatusCode::FORBIDDEN,
                "API_KEY_MANAGEMENT_FORBIDDEN",
                &t(
                    "error-api-key-management-forbidden",
                    "La gestion des clés API n'est pas autorisée via une clé API. Utilisez l'interface web.",
                ),
            ),

            AppError::CannotDisableSelf => build_response(
                StatusCode::BAD_REQUEST,
                "CANNOT_DISABLE_SELF",
                &t(
                    "error-cannot-disable-self",
                    "Impossible de désactiver son propre compte",
                ),
            ),

            AppError::CannotDisableLastAdmin => build_response(
                StatusCode::BAD_REQUEST,
                "CANNOT_DISABLE_LAST_ADMIN",
                &t(
                    "error-cannot-disable-last-admin",
                    "Impossible de désactiver le dernier administrateur",
                ),
            ),

            AppError::Internal(detail) => {
                tracing::error!("internal: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    &t("error-internal", "Erreur interne"),
                )
            }

            AppError::RateLimited { retry_after } => {
                let mut resp = build_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    "RATE_LIMITED",
                    &t("error-rate-limited", "Trop de tentatives"),
                );
                resp.headers_mut().insert(
                    "Retry-After",
                    axum::http::HeaderValue::from_str(&retry_after.to_string())
                        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("60")),
                );
                resp
            }

            AppError::InvalidRefreshToken(detail) => {
                tracing::warn!("invalid refresh token: {detail}");
                build_response(
                    StatusCode::UNAUTHORIZED,
                    "INVALID_REFRESH_TOKEN",
                    &t("error-invalid-refresh-token", "Session expirée"),
                )
            }

            AppError::OnboardingStepAlreadyCompleted => build_response(
                StatusCode::BAD_REQUEST,
                "ONBOARDING_STEP_ALREADY_COMPLETED",
                &t(
                    "error-onboarding-step-already-completed",
                    "Cette étape de configuration a déjà été complétée",
                ),
            ),

            AppError::OnboardingResetForbidden => build_response(
                StatusCode::FORBIDDEN,
                "ONBOARDING_RESET_FORBIDDEN",
                &t(
                    "error-onboarding-reset-forbidden",
                    "Le reset de l'onboarding n'est pas autorisé sur cette instance.",
                ),
            ),

            AppError::EntryUnbalanced { debit, credit } => {
                // FR21 : le wording exact vient du PRD. La version i18n
                // inclut les placeholders via Fluent ; à défaut, on
                // construit la version française à la volée.
                let fallback = format!(
                    "Écriture déséquilibrée — le total des débits ({debit}) ne correspond pas au total des crédits ({credit})"
                );
                build_response(StatusCode::BAD_REQUEST, "ENTRY_UNBALANCED", &fallback)
            }

            AppError::NoFiscalYear { date } => {
                let fallback = format!(
                    "Aucun exercice n'existe pour la date {date}. Créez un exercice comptable avant de saisir des écritures."
                );
                build_response(StatusCode::BAD_REQUEST, "NO_FISCAL_YEAR", &fallback)
            }

            AppError::FiscalYearClosed { date } => {
                let fallback = format!(
                    "L'exercice pour la date {date} est clôturé — aucune écriture ne peut y être ajoutée ou modifiée (CO art. 957-964)."
                );
                build_response(StatusCode::BAD_REQUEST, "FISCAL_YEAR_CLOSED", &fallback)
            }

            AppError::DateOutsideFiscalYear { date } => {
                let fallback =
                    format!("La date {date} n'est pas dans l'exercice courant de cette écriture.");
                build_response(
                    StatusCode::BAD_REQUEST,
                    "DATE_OUTSIDE_FISCAL_YEAR",
                    &fallback,
                )
            }

            // Story 4.1 : code dédié pour l'unicité IDE par company.
            AppError::IdeAlreadyExists(msg) => {
                build_response(StatusCode::CONFLICT, "IDE_ALREADY_EXISTS", &msg)
            }

            // Story 20-3b1 — envoi de facture par e-mail.
            AppError::SmtpNotConfigured => build_response(
                StatusCode::PRECONDITION_FAILED,
                "SMTP_NOT_CONFIGURED",
                &t(
                    "error-smtp-not-configured",
                    "L'envoi d'e-mails n'est pas configuré sur cette instance (variables KESH_SMTP_*).",
                ),
            ),
            AppError::ContactEmailMissing => build_response(
                StatusCode::BAD_REQUEST,
                "CONTACT_EMAIL_MISSING",
                &t(
                    "error-contact-email-missing",
                    "Le contact de la facture n'a pas d'adresse e-mail. Renseignez-la sur la fiche contact.",
                ),
            ),
            AppError::InvoiceAlreadyPaid => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVOICE_ALREADY_PAID",
                &t(
                    "error-invoice-already-paid",
                    "La facture est déjà payée — aucun rappel ne peut être enregistré.",
                ),
            ),
            AppError::DunningLevelNotFound => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "DUNNING_LEVEL_NOT_FOUND",
                &t(
                    "error-dunning-level-not-found",
                    "Le niveau de rappel demandé n'existe pas dans la configuration.",
                ),
            ),
            AppError::ReminderDateInFuture => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "REMINDER_DATE_IN_FUTURE",
                &t(
                    "error-reminder-date-in-future",
                    "La date d'un rappel ne peut pas être dans le futur.",
                ),
            ),
            AppError::InvoiceNotPaused => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVOICE_NOT_PAUSED",
                &t(
                    "error-invoice-not-paused",
                    "Cette facture n'est pas suspendue.",
                ),
            ),
            AppError::InvoiceEmailEmptyContent => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVOICE_EMAIL_EMPTY_CONTENT",
                &t(
                    "error-invoice-email-empty-content",
                    "L'objet et le corps de l'e-mail ne peuvent pas être vides.",
                ),
            ),
            AppError::ContactArchived => build_response(
                StatusCode::BAD_REQUEST,
                "CONTACT_ARCHIVED",
                &t(
                    "error-contact-archived",
                    "Le contact de la facture est archivé. Réactivez-le avant d'envoyer la facture par e-mail.",
                ),
            ),
            AppError::EmailSentInvoiceGone => build_response(
                StatusCode::CONFLICT,
                "EMAIL_SENT_INVOICE_GONE",
                &t(
                    "error-email-sent-invoice-gone",
                    "L'e-mail a bien été envoyé au contact, mais la facture a été supprimée entre-temps — elle n'a pas pu être marquée « envoyée ». Ne renvoyez pas l'e-mail.",
                ),
            ),

            // Story 5.3 — erreurs PDF QR Bill.
            AppError::InvoiceNotValidated => build_response(
                StatusCode::BAD_REQUEST,
                "INVOICE_NOT_VALIDATED",
                &t(
                    "error-invoice-not-validated",
                    "La facture doit être validée avant de pouvoir être générée en PDF.",
                ),
            ),
            AppError::InvoiceNotPdfReady(msg) => {
                build_response(StatusCode::BAD_REQUEST, "INVOICE_NOT_PDF_READY", &msg)
            }
            AppError::InvoiceTooManyLinesForPdf(n) => {
                let max = crate::routes::invoice_pdf_service::MAX_LINES_PER_PDF;
                let fallback = format!(
                    "La facture contient {n} lignes — le PDF A4 est limité à {max} lignes en v0.1."
                );
                let mut args = FluentArgs::new();
                args.set("count", n as i64);
                args.set("max", max as i64);
                let msg = t_args("error-invoice-too-many-lines-for-pdf", &fallback, &args);
                build_response(
                    StatusCode::BAD_REQUEST,
                    "INVOICE_TOO_MANY_LINES_FOR_PDF",
                    &msg,
                )
            }
            // Story 5.4 — overflow export CSV.
            AppError::ResultTooLarge(msg) => {
                build_response(StatusCode::BAD_REQUEST, "RESULT_TOO_LARGE", &msg)
            }

            AppError::PdfGenerationFailed(detail) => {
                tracing::error!("pdf generation failed: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "PDF_GENERATION_FAILED",
                    &t(
                        "error-pdf-generation-failed",
                        "Échec de la génération du PDF.",
                    ),
                )
            }

            // Story 9-2a + Pass 1 code-review H1 — variant dédié CSV (sinon un
            // échec d'export CSV se présentait côté UI comme un échec PDF).
            AppError::CsvGenerationFailed(detail) => {
                tracing::error!("csv generation failed: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CSV_GENERATION_FAILED",
                    &t(
                        "error-csv-generation-failed",
                        "Échec de la génération du CSV.",
                    ),
                )
            }

            // Story 9-2b §error-variant + Pass 3 ECH3-C2 — variant dédié export
            // global ZIP. Pattern strictement aligné avec `PdfGenerationFailed`
            // / `CsvGenerationFailed` ground-truth (build_response = 3 args).
            AppError::GlobalExportFailed(detail) => {
                tracing::error!("global export failed: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "GLOBAL_EXPORT_FAILED",
                    &t(
                        "error-global-export-failed",
                        "Échec de la génération de l'export global. Réessayez dans quelques instants.",
                    ),
                )
            }

            // Story 17-3a — export complet d'installation (.keshbackup).
            AppError::AdminFullExportFailed(detail) => {
                tracing::error!("admin full export failed: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ADMIN_FULL_EXPORT_FAILED",
                    &t(
                        "error-admin-full-export-failed",
                        "Échec de la génération de l'export d'installation. Réessayez dans quelques instants.",
                    ),
                )
            }

            // Story 17-3c — import complet d'installation (.keshbackup).
            AppError::AdminFullImportFailed(detail) => {
                tracing::error!("admin full import failed: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ADMIN_FULL_IMPORT_FAILED",
                    &t(
                        "error-admin-full-import-failed",
                        "Échec de l'import de l'installation. L'état précédent a été préservé.",
                    ),
                )
            }

            // Story 17-4b — échec envoi email SMTP (recovery). Détail loggé,
            // jamais exposé. (17-4c : fire-and-forget, n'atteint pas le client.)
            AppError::SmtpSendFailed(detail) => {
                tracing::error!("smtp send failed: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SMTP_SEND_FAILED",
                    &t(
                        "error-smtp-send-failed",
                        "Échec de l'envoi de l'email. Réessayez dans quelques instants.",
                    ),
                )
            }

            // Story 17-4c — token reset invalide/expiré/utilisé (DC4 anti-fuite).
            // Message générique : ne distingue pas les trois cas.
            AppError::InvalidOrExpiredToken => build_response(
                StatusCode::BAD_REQUEST,
                "INVALID_OR_EXPIRED_TOKEN",
                &t(
                    "error-invalid-or-expired-token",
                    "Lien de réinitialisation invalide ou expiré.",
                ),
            ),

            AppError::InvalidBackupStructure(detail) => {
                tracing::warn!("invalid backup structure: {detail}");
                build_response(
                    StatusCode::BAD_REQUEST,
                    "INVALID_BACKUP_STRUCTURE",
                    &t(
                        "error-invalid-backup-structure",
                        "Le fichier de sauvegarde est invalide ou corrompu.",
                    ),
                )
            }

            AppError::ImportSchemaMismatch {
                table,
                unknown_columns,
                missing_required_columns,
            } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "IMPORT_SCHEMA_MISMATCH",
                        "message": t(
                            "error-import-schema-mismatch",
                            "Le schéma du backup est incompatible avec cette version de Kesh.",
                        ),
                        "details": {
                            "table": table,
                            "unknownColumns": unknown_columns,
                            "missingRequiredColumns": missing_required_columns,
                        }
                    }
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }

            AppError::ImportVersionIncompatible {
                source_min_required,
                binary_version,
            } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "IMPORT_VERSION_INCOMPATIBLE",
                        "message": t(
                            "error-import-version-incompatible",
                            "Ce backup requiert une version de Kesh plus récente que celle installée.",
                        ),
                        "details": {
                            "sourceMinRequired": source_min_required,
                            "binaryVersion": binary_version,
                        }
                    }
                });
                (StatusCode::CONFLICT, Json(body)).into_response()
            }

            // --- Story 8-1b — Import bancaire CAMT.053 (T6.4) ---
            AppError::BankImportTooLarge => build_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                "BANK_IMPORT_TOO_LARGE",
                &t(
                    "bank-import-errors-too-large",
                    "Fichier trop volumineux (>10 MiB).",
                ),
            ),

            AppError::BankImportParseFailed { kind, message } => {
                tracing::warn!("bank import parse failed: {kind} — {message}");
                let (code, default) = match kind {
                    "MALFORMED_XML" => (
                        "BANK_IMPORT_MALFORMED_XML",
                        "Fichier XML mal formé ou tronqué.",
                    ),
                    "UNSUPPORTED_VERSION" => (
                        "BANK_IMPORT_UNSUPPORTED_VERSION",
                        "Version CAMT.053 non supportée.",
                    ),
                    "MISSING_FIELD" => (
                        "BANK_IMPORT_MISSING_FIELD",
                        "Champ requis manquant dans le fichier.",
                    ),
                    "INVALID_AMOUNT" => (
                        "BANK_IMPORT_INVALID_AMOUNT",
                        "Montant invalide dans le fichier.",
                    ),
                    "INVALID_DATE" => {
                        ("BANK_IMPORT_INVALID_DATE", "Date invalide dans le fichier.")
                    }
                    _ => ("BANK_IMPORT_PARSE_FAILED", "Fichier CAMT.053 invalide."),
                };
                let key = match code {
                    "BANK_IMPORT_MALFORMED_XML" => "bank-import-errors-malformed-xml",
                    "BANK_IMPORT_UNSUPPORTED_VERSION" => "bank-import-errors-unsupported-version",
                    "BANK_IMPORT_MISSING_FIELD" => "bank-import-errors-missing-field",
                    "BANK_IMPORT_INVALID_AMOUNT" => "bank-import-errors-invalid-amount",
                    "BANK_IMPORT_INVALID_DATE" => "bank-import-errors-invalid-date",
                    _ => "bank-import-errors-parse-failed",
                };
                // Review code Pass 1 M14 : `code` est `&'static str` donc
                // déjà serialisé en string par `serde_json::json!`. Le
                // bloc `if let Some(err_obj) = body.get_mut(...)` qui
                // re-posait la même valeur était du dead code.
                let body = serde_json::json!({
                    "error": {
                        "code": code,
                        "message": t(key, default),
                        "details": { "kind": kind, "message": message }
                    }
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }

            AppError::BankImportBalanceMismatch {
                opening,
                closing,
                sum,
                diff,
            } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_IMPORT_BALANCE_MISMATCH",
                        "message": t(
                            "bank-import-errors-balance-mismatch",
                            "Solde de clôture incohérent.",
                        ),
                        "details": {
                            "opening": opening,
                            "closing": closing,
                            "sum": sum,
                            "diff": diff,
                        }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankImportUnsupportedCurrency(currency) => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_IMPORT_UNSUPPORTED_CURRENCY",
                        "message": t(
                            "bank-import-errors-unsupported-currency",
                            "Devise non supportée v0.1 (CHF uniquement).",
                        ),
                        "details": { "currency": currency }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankImportNoMatchingStatement { found_ibans } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_IMPORT_NO_MATCHING_STATEMENT",
                        "message": t(
                            "bank-import-errors-no-matching-statement",
                            "Aucun statement ne correspond au compte sélectionné.",
                        ),
                        "details": { "foundIbans": found_ibans }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankImportDuplicateFile {
                existing_import_id,
                existing_filename,
            } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_IMPORT_DUPLICATE_FILE",
                        "message": t(
                            "bank-import-errors-duplicate-file",
                            "Ce fichier a déjà été importé.",
                        ),
                        "details": {
                            "existingImportId": existing_import_id,
                            "existingFilename": existing_filename,
                        }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankAccountNotFound => build_response(
                StatusCode::NOT_FOUND,
                "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND",
                &t(
                    "bank-import-errors-bank-account-not-found",
                    "Compte bancaire non trouvé.",
                ),
            ),

            // ----- Story 8-5a-zero — bank_account.journal_account_id link -----
            // F1''' Pass 3 Opus : `missing_account_ids` optionnel pour batch split.
            AppError::AccountNotFound {
                account_id,
                missing_account_ids,
            } => {
                let mut details = serde_json::json!({ "accountId": account_id });
                if let Some(ids) = missing_account_ids {
                    details["missingAccountIds"] = serde_json::json!(ids);
                }
                let body = serde_json::json!({
                    "error": {
                        "code": "ACCOUNT_NOT_FOUND",
                        "message": t(
                            "bank-accounts-errors-account-not-found",
                            "Compte du plan comptable non trouvé.",
                        ),
                        "details": details
                    }
                });
                (StatusCode::NOT_FOUND, Json(body)).into_response()
            }

            AppError::InvalidAccountType {
                account_id,
                account_type,
            } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "INVALID_ACCOUNT_TYPE",
                        "message": t(
                            "bank-accounts-errors-invalid-account-type",
                            "Type de compte invalide (Actif ou Passif requis).",
                        ),
                        "details": {
                            "accountId": account_id,
                            "accountType": account_type,
                            "allowedTypes": ["Asset", "Liability"],
                        }
                    }
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }

            // ----- Story 8-2 — bank profiles + CSV import -----
            AppError::BankCsvProfileNotFound { available_profiles } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_CSV_NO_PROFILE_MATCH",
                        "message": t(
                            "bank-import-csv-errors-no-profile-match",
                            "Aucun profil bancaire ne matche ce fichier.",
                        ),
                        "details": {
                            "availableProfiles": available_profiles,
                        }
                    }
                });
                (StatusCode::NOT_FOUND, Json(body)).into_response()
            }

            AppError::BankCsvUnsupportedEncoding { detected } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_CSV_UNSUPPORTED_ENCODING",
                        "message": t(
                            "bank-import-csv-errors-unsupported-encoding",
                            "Encoding du fichier non supporté (UTF-8 ou ISO-8859-1 attendu).",
                        ),
                        "details": { "detected": detected }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankCsvEncodingMismatch { profile, detected } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_CSV_ENCODING_MISMATCH",
                        "message": t(
                            "bank-import-csv-errors-encoding-mismatch",
                            "L'encoding détecté diffère du profil. Confirmez via confirmEncodingMismatch=true.",
                        ),
                        "details": {
                            "profileEncoding": profile,
                            "detectedEncoding": detected,
                        }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankCsvParsePartialFailure {
                lines,
                total_errors,
                truncated,
                reason,
            } => {
                let mut details = serde_json::json!({
                    "lines": lines,
                    "totalErrors": total_errors,
                    "truncated": truncated,
                });
                if let Some(r) = reason {
                    details["reason"] = serde_json::Value::String(r.to_string());
                }
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_CSV_PARTIAL_FAILURE",
                        "message": t(
                            "bank-import-csv-errors-partial-failure",
                            "Certaines lignes du CSV n'ont pas pu être parsées.",
                        ),
                        "details": details
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankCsvProfileValidation(reason) => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "BANK_CSV_PROFILE_INVALID",
                &t(
                    "bank-import-csv-errors-profile-invalid",
                    &format!("Profil bancaire invalide : {}", reason),
                ),
            ),

            AppError::BankCsvProfileDuplicate => build_response(
                StatusCode::CONFLICT,
                "BANK_CSV_PROFILE_DUPLICATE",
                &t(
                    "bank-import-csv-errors-profile-duplicate",
                    "Un profil avec ce nom de banque existe déjà.",
                ),
            ),

            AppError::BankCsvProfileMisconfigured(reason) => {
                // Pass 1 review G2-BH-5 + G2-EH-4 : ne pas exposer la `reason`
                // interne au client (peut contenir byte_offset DB, paths,
                // etc.). Logger côté serveur, retourner un message générique.
                tracing::error!("bank_csv profile misconfigured: {}", reason);
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "BANK_CSV_PROFILE_MISCONFIGURED",
                    &t(
                        "bank-import-csv-errors-profile-misconfigured",
                        "Profil bancaire mal configuré.",
                    ),
                )
            }

            AppError::BankCsvEmptyFile { reason } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_CSV_EMPTY_FILE",
                        "message": t(
                            "bank-import-csv-errors-empty-file",
                            "Fichier CSV vide ou aucune ligne de données.",
                        ),
                        "details": { "reason": reason }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::BankImportUnsupportedFormat => build_response(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "BANK_IMPORT_UNSUPPORTED_FORMAT",
                &t(
                    "bank-import-errors-unsupported-format",
                    "Format de fichier non supporté (CAMT.053 XML ou CSV attendus).",
                ),
            ),

            // ----- Story 8-4 — Reconciliation -----
            AppError::ReconciliationAccountLocked {
                bank_account_id,
                timeout_secs,
            } => {
                let msg = t(
                    "reconciliation-errors-account-locked",
                    "Un autre import/réconciliation est en cours sur ce compte, réessayez dans quelques secondes.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "RECONCILIATION_ACCOUNT_LOCKED",
                        "message": msg,
                        "details": { "bankAccountId": bank_account_id, "retryAfterSeconds": timeout_secs },
                    }
                });
                (StatusCode::CONFLICT, Json(body)).into_response()
            }
            AppError::ReconciliationLockReleaseFailed { bank_account_id } => {
                let msg = t(
                    "reconciliation-errors-lock-release-failed",
                    "Échec interne de libération du verrou. Réessayez.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "RECONCILIATION_LOCK_RELEASE_FAILED",
                        "message": msg,
                        "details": { "bankAccountId": bank_account_id },
                    }
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }

            // ----- Story v014-1 — CRUD bank_accounts post-onboarding -----
            AppError::BankAccountHasTransactions { transaction_count } => {
                let msg = t(
                    "bank-accounts-errors-has-transactions",
                    "Le compte bancaire contient des transactions — \
                     archivage refusé pour préserver l'audit comptable.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_ACCOUNT_HAS_TRANSACTIONS",
                        "message": msg,
                        "details": {
                            "transactionCount": transaction_count,
                        },
                    }
                });
                (StatusCode::PRECONDITION_FAILED, Json(body)).into_response()
            }

            AppError::BankAccountCannotArchivePrimary => {
                let msg = t(
                    "bank-accounts-errors-cannot-archive-primary",
                    "Le compte principal ne peut pas être archivé tant qu'un \
                     autre compte non-archivé existe. Définissez d'abord un autre \
                     compte comme principal, puis archivez celui-ci.",
                );
                build_response(
                    StatusCode::PRECONDITION_FAILED,
                    "BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY",
                    &msg,
                )
            }

            AppError::OnboardingNotComplete => {
                let msg = t(
                    "bank-accounts-errors-onboarding-not-complete",
                    "L'onboarding doit être terminé (étape 7 complétée) avant de \
                     pouvoir gérer les comptes bancaires.",
                );
                build_response(
                    StatusCode::PRECONDITION_FAILED,
                    "ONBOARDING_NOT_COMPLETE",
                    &msg,
                )
            }

            // ----- Story 8-5a-base — réconciliation manuelle FR45 -----
            AppError::BankAccountNotConfigured { bank_account_id } => {
                let msg = t(
                    "reconciliation-manual-bank-account-not-configured",
                    "Le compte bancaire n'est pas configuré. Configurer le compte \
                     comptable lié dans /bank-accounts.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "BANK_ACCOUNT_NOT_CONFIGURED",
                        "message": msg,
                        "details": {
                            "bankAccountId": bank_account_id,
                            "hint": "Configurer le compte comptable lié via /bank-accounts",
                        },
                    }
                });
                (StatusCode::PRECONDITION_FAILED, Json(body)).into_response()
            }

            AppError::ReconciliationFiscalYearClosed { entry_date } => {
                let msg = t(
                    "reconciliation-errors-fiscal-year-closed",
                    "Réconciliation impossible : l'exercice comptable n'est pas \
                     ouvert pour cette date.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "RECONCILIATION_FISCAL_YEAR_CLOSED",
                        "message": msg,
                        "details": { "entryDate": entry_date.to_string() },
                    }
                });
                (StatusCode::CONFLICT, Json(body)).into_response()
            }

            AppError::ReconciliationTransactionNotPending {
                bank_transaction_id,
            } => {
                let msg = t(
                    "reconciliation-errors-transaction-not-pending",
                    "Transaction bancaire introuvable ou déjà réconciliée.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "RECONCILIATION_TRANSACTION_NOT_PENDING",
                        "message": msg,
                        "details": { "bankTransactionId": bank_transaction_id },
                    }
                });
                (StatusCode::NOT_FOUND, Json(body)).into_response()
            }

            // Story 8-5a-bis FR48 — split imbalance → 400 avec body Decimal stringifié.
            // P5 Pass 1 code-review (BH-M4+ECH-07+AA-F3) — `Decimal::rescale(2)`
            // force scale 2 ("10500" → "10500.00") cohérent total_amount audit log.
            // round_dp(2) ne padd pas les zéros trailing si scale d'entrée < 2.
            AppError::ReconciliationSplitImbalance {
                expected,
                actual,
                difference,
            } => {
                let msg = t(
                    "reconciliation-split-error-imbalance",
                    "L'éclatement n'équilibre pas le montant de la transaction.",
                );
                let mut expected_s = expected;
                let mut actual_s = actual;
                let mut difference_s = difference;
                expected_s.rescale(2);
                actual_s.rescale(2);
                difference_s.rescale(2);
                let body = serde_json::json!({
                    "error": {
                        "code": "RECONCILIATION_SPLIT_IMBALANCE",
                        "message": msg,
                        "details": {
                            "expected": expected_s.to_string(),
                            "actual": actual_s.to_string(),
                            "difference": difference_s.to_string(),
                        }
                    }
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }
            // M6 Pass 1 code review : variants `ReconciliationAlreadyReconciled` et
            // `ReconciliationInvoiceNotEligible` supprimés (jamais émis comme
            // `AppError` global, représentés en `FailedProposal` per-proposal).

            // Story 8-5b — rules engine FR47 (4 variants).
            AppError::ReconciliationRuleNotFound { rule_id } => {
                let msg = t(
                    "reconciliation-rules-error-not-found",
                    "Règle de réconciliation introuvable.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "RECONCILIATION_RULE_NOT_FOUND",
                        "message": msg,
                        "details": { "ruleId": rule_id },
                    }
                });
                (StatusCode::NOT_FOUND, Json(body)).into_response()
            }
            AppError::ReconciliationRuleDuplicate {
                match_type,
                match_value,
            } => {
                let msg = t(
                    "reconciliation-rules-error-duplicate",
                    "Une règle active existe déjà pour cette combinaison type/valeur.",
                );
                let body = serde_json::json!({
                    "error": {
                        "code": "RECONCILIATION_RULE_DUPLICATE",
                        "message": msg,
                        "details": {
                            "matchType": match_type,
                            "matchValue": match_value,
                        }
                    }
                });
                (StatusCode::CONFLICT, Json(body)).into_response()
            }
            // Pass 4 LOW#6 : ces 2 variants existent pour complétude du
            // From<ReconciliationError>; jamais émis comme HTTP global
            // (accept_one_rule retourne FailedProposal per-proposal).
            // Fallback défensif : 500 si jamais reached.
            AppError::ReconciliationRuleMismatch { rule_id } => {
                tracing::error!(
                    "ReconciliationRuleMismatch reached as HTTP global (rule_id={rule_id}) — \
                     should have been per-proposal FailedProposal"
                );
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "RECONCILIATION_RULE_MISMATCH",
                    "Variant Rule jamais émis comme erreur globale (bug interne)",
                )
            }
            AppError::ReconciliationRuleNoLongerMatches { rule_id } => {
                tracing::error!(
                    "ReconciliationRuleNoLongerMatches reached as HTTP global (rule_id={rule_id}) — \
                     should have been per-proposal FailedProposal"
                );
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "RECONCILIATION_RULE_NO_LONGER_MATCHES",
                    "Variant Rule jamais émis comme erreur globale (bug interne)",
                )
            }

            // --- Story 9-1 : Rapports comptables ---
            AppError::ReportFiscalYearNotFound { fiscal_year_id } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "FISCAL_YEAR_NOT_FOUND",
                        "message": "Exercice comptable introuvable pour cette company.",
                        "details": { "fiscalYearId": fiscal_year_id },
                    }
                });
                (StatusCode::NOT_FOUND, Json(body)).into_response()
            }

            // Pass 3 BH3-03 : variant ad-hoc JSON divergent (ErrorBody standard n'a pas
            // de champ `details`). L69 dette refactor v0.2.
            AppError::ReportPeriodOutOfFiscalYear {
                fy_start,
                fy_end,
                requested_start,
                requested_end,
            } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "REPORT_PERIOD_OUT_OF_FISCAL_YEAR",
                        "message": "La période sélectionnée dépasse les bornes de l'exercice.",
                        "details": {
                            "fyStart": fy_start.format("%Y-%m-%d").to_string(),
                            "fyEnd": fy_end.format("%Y-%m-%d").to_string(),
                            "requestedStart": requested_start.format("%Y-%m-%d").to_string(),
                            "requestedEnd": requested_end.format("%Y-%m-%d").to_string(),
                        },
                    }
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
            }

            // Sous-match exhaustif sur DbError : pas de `_ =>` catch-all,
            // l'ajout futur d'une variante kesh-db casse la compilation
            // ici (propriété désirée).
            // --- Story 12-5c — import répertoire de factures (#194) ---
            AppError::InboxImportAlreadyRunning => build_response(
                StatusCode::CONFLICT,
                "INBOX_IMPORT_ALREADY_RUNNING",
                &t(
                    "error-inbox-import-already-running",
                    "Un import du répertoire est déjà en cours. Réessayez dans quelques instants.",
                ),
            ),

            AppError::ImportNotPendingCompletion { current_status } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "IMPORT_NOT_PENDING_COMPLETION",
                        "message": t(
                            "error-import-not-pending-completion",
                            "Cette facture importée n'est plus en attente de complétion.",
                        ),
                        "details": { "currentStatus": current_status }
                    }
                });
                (StatusCode::CONFLICT, Json(body)).into_response()
            }

            AppError::ImportCompletionRejected {
                error_code,
                message,
                details,
            } => {
                let mut error = serde_json::json!({
                    "code": error_code,
                    "message": message,
                });
                if let Some(d) = details {
                    error["details"] = d;
                }
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": error })),
                )
                    .into_response()
            }

            AppError::SourceDocumentNotFound => build_response(
                StatusCode::NOT_FOUND,
                "SOURCE_DOCUMENT_NOT_FOUND",
                &t(
                    "error-source-document-not-found",
                    "Cette facture n'a pas de justificatif stocké.",
                ),
            ),

            AppError::ImportedInvoiceNotFound => build_response(
                StatusCode::NOT_FOUND,
                "IMPORTED_INVOICE_NOT_FOUND",
                &t(
                    "error-imported-invoice-not-found",
                    "Cette facture importée est introuvable.",
                ),
            ),

            AppError::SourceDocumentGone => build_response(
                StatusCode::GONE,
                "SOURCE_DOCUMENT_GONE",
                &t(
                    "error-source-document-gone",
                    "Le justificatif n'a pas été restauré (métadonnées seules).",
                ),
            ),

            AppError::EmailTemplateUnknownVariables { unknown_vars } => {
                let body = serde_json::json!({
                    "error": {
                        "code": "EMAIL_TEMPLATE_UNKNOWN_VARIABLES",
                        "message": t(
                            "error-email-template-unknown-variables",
                            "Le template contient des variables inconnues.",
                        ),
                        "details": { "unknownVariables": unknown_vars }
                    }
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }

            AppError::Database(db_err) => match db_err {
                DbError::NotFound => build_response(
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    &t("error-not-found", "Ressource introuvable"),
                ),
                DbError::OptimisticLockConflict => build_response(
                    StatusCode::CONFLICT,
                    "OPTIMISTIC_LOCK_CONFLICT",
                    &t(
                        "error-optimistic-lock",
                        "Conflit de version — la ressource a été modifiée",
                    ),
                ),
                DbError::UniqueConstraintViolation(m) => {
                    tracing::warn!("unique violation: {m}");
                    build_response(
                        StatusCode::CONFLICT,
                        "RESOURCE_CONFLICT",
                        &t("error-conflict", "Ressource déjà existante"),
                    )
                }
                DbError::ForeignKeyViolation(m) => {
                    tracing::warn!("fk violation: {m}");
                    // Cas spécifique : suppression d'une écriture comptable encore
                    // référencée par une facture validée (`invoices.journal_entry_id`,
                    // contrainte `fk_invoices_journal_entry`, ON DELETE RESTRICT).
                    // Le message MySQL porté par `m` contient le nom de la contrainte
                    // → on renvoie un message actionable plutôt que le générique. (#184)
                    // On exige le préfixe « Cannot delete » (erreur 1451) pour ne PAS
                    // déclencher ce message sur une 1452 (insertion d'un enfant
                    // invalide), qui partage le même variant mais un sens opposé.
                    if m.contains("fk_invoices_journal_entry") && m.contains("Cannot delete") {
                        build_response(
                            StatusCode::CONFLICT,
                            "JOURNAL_ENTRY_LINKED_TO_INVOICE",
                            &t(
                                "error-journal-entry-linked-to-invoice",
                                "Cette écriture comptable a été générée par une facture validée et ne peut pas être supprimée directement. Annulez d'abord la facture concernée.",
                            ),
                        )
                    } else {
                        build_response(
                            StatusCode::BAD_REQUEST,
                            "FOREIGN_KEY_VIOLATION",
                            &t("error-foreign-key", "Référence invalide"),
                        )
                    }
                }
                DbError::CheckConstraintViolation(m) => {
                    tracing::warn!("check violation: {m}");
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "CHECK_CONSTRAINT_VIOLATION",
                        &t("error-check-constraint", "Valeur invalide"),
                    )
                }
                DbError::IllegalStateTransition(m) => {
                    tracing::warn!("illegal state: {m}");
                    build_response(
                        StatusCode::CONFLICT,
                        "ILLEGAL_STATE_TRANSITION",
                        &t("error-illegal-state", "Transition d'état interdite"),
                    )
                }
                DbError::FiscalYearClosed => build_response(
                    StatusCode::BAD_REQUEST,
                    "FISCAL_YEAR_CLOSED",
                    &t(
                        "error-fiscal-year-closed-generic",
                        "L'exercice comptable est clôturé — aucune écriture ne peut y être ajoutée ou modifiée (CO art. 957-964).",
                    ),
                ),
                DbError::InactiveOrInvalidAccounts => build_response(
                    StatusCode::BAD_REQUEST,
                    "INACTIVE_OR_INVALID_ACCOUNTS",
                    &t(
                        "error-inactive-accounts",
                        "Un ou plusieurs comptes sont archivés ou invalides.",
                    ),
                ),
                DbError::DateOutsideFiscalYear => build_response(
                    StatusCode::BAD_REQUEST,
                    "DATE_OUTSIDE_FISCAL_YEAR",
                    &t(
                        "error-date-outside-fiscal-year-generic",
                        "La date n'est pas dans l'exercice courant de cette écriture.",
                    ),
                ),
                DbError::FiscalYearInvalid => build_response(
                    StatusCode::BAD_REQUEST,
                    "FISCAL_YEAR_INVALID",
                    &t(
                        "error-fiscal-year-invalid",
                        "Aucun exercice ouvert ne couvre cette date.",
                    ),
                ),
                DbError::ConfigurationRequired(field) => {
                    tracing::warn!("configuration required: {field}");
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "CONFIGURATION_REQUIRED",
                        &t(
                            "error-configuration-required",
                            "Configuration incomplète : configurez les paramètres de facturation avant de valider.",
                        ),
                    )
                }
                DbError::InvalidInput(code) => {
                    tracing::warn!("invalid input: {code}");
                    // Dispatch vers une clé FTL dédiée selon le code métier.
                    // H2 (review pass 1 G2) : pour les codes connus, on résout
                    // la clé i18n spec (ex. paidAtBeforeInvoiceDate → clé
                    // `invoice-error-paid-at-before-invoice-date`).
                    let (key, default): (String, &str) = match code.as_str() {
                        "paidAtBeforeInvoiceDate" => (
                            "invoice-error-paid-at-before-invoice-date".to_string(),
                            "La date de paiement ne peut être antérieure à la date de facture.",
                        ),
                        // N2 (review pass 3 B) : code "paidAtFuture" supprimé —
                        // `paid_at` peut être dans le futur (date d'exécution bancaire).
                        "alreadyUnpaid" => (
                            "invoice-error-already-unpaid".to_string(),
                            "Cette facture n'est pas marquée payée.",
                        ),
                        // B13 (review pass 1 G2 B) : whitelist stricte —
                        // un code non listé ne doit PAS construire dynamiquement
                        // une clé FTL (potentielle pollution si le code provient
                        // d'une couche non fiable). Fallback sur clé générique.
                        _ => {
                            tracing::warn!("invalid input code unknown to dispatch: {code}");
                            ("error-invalid-input-generic".to_string(), "Entrée invalide")
                        }
                    };
                    build_response(StatusCode::BAD_REQUEST, "INVALID_INPUT", &t(&key, default))
                }
                DbError::ConnectionUnavailable(m) => {
                    tracing::warn!("db connection unavailable: {m}");
                    build_response(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "SERVICE_UNAVAILABLE",
                        &t(
                            "error-service-unavailable",
                            "Service temporairement indisponible",
                        ),
                    )
                }
                DbError::Invariant(m) => {
                    tracing::error!("db invariant violated: {m}");
                    build_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        &t("error-internal", "Erreur interne"),
                    )
                }
                // Story 12-5c (D2) — donnée trop longue / hors plage (MariaDB
                // 1406/1264). Le service d'import 12-5c intercepte ce variant
                // AVANT qu'il devienne une `AppError` (→ `failed[]` per-fichier,
                // HTTP 200). Si malgré tout il atteint le mapping HTTP global
                // (autre chemin), c'est une donnée d'entrée invalide → 400 (PAS
                // un 500 : la requête est en faute, pas le serveur).
                DbError::DataLengthOrRange(m) => {
                    tracing::warn!("data too long / out of range: {m}");
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "DATA_LENGTH_OR_RANGE",
                        &t(
                            "error-data-length-or-range",
                            "Une valeur fournie est trop longue ou hors plage.",
                        ),
                    )
                }
                DbError::Sqlx(e) => {
                    tracing::error!("sqlx: {e}");
                    build_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        &t("error-internal", "Erreur interne"),
                    )
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    async fn response_body(resp: Response) -> (StatusCode, serde_json::Value) {
        let (parts, body) = resp.into_parts();
        let bytes = body.collect().await.expect("body collect").to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should be JSON");
        (parts.status, json)
    }

    #[tokio::test]
    async fn invalid_credentials_maps_to_401() {
        let resp = AppError::InvalidCredentials.into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
        assert_eq!(body["error"]["message"], "Identifiants invalides");
    }

    #[tokio::test]
    async fn unauthenticated_maps_to_401_with_generic_message() {
        let resp = AppError::Unauthenticated("detailed internal info".to_string()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "UNAUTHENTICATED");
        // Le détail interne ne doit pas leak
        let message = body["error"]["message"].as_str().unwrap();
        assert!(
            !message.contains("detailed internal info"),
            "detail leaked in response: {}",
            message
        );
    }

    #[tokio::test]
    async fn validation_maps_to_400() {
        let resp = AppError::Validation("username must not be empty".into()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn internal_maps_to_500_with_generic_message() {
        let resp = AppError::Internal("stack trace details".to_string()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "INTERNAL_ERROR");
        let message = body["error"]["message"].as_str().unwrap();
        assert!(!message.contains("stack trace"));
    }

    #[tokio::test]
    async fn db_not_found_maps_to_404() {
        let resp = AppError::Database(DbError::NotFound).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "NOT_FOUND");
    }

    #[tokio::test]
    async fn db_optimistic_lock_maps_to_409() {
        let resp = AppError::Database(DbError::OptimisticLockConflict).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    }

    #[tokio::test]
    async fn db_fk_violation_generic_maps_to_400() {
        // FK quelconque (contrainte non reconnue) → message générique, 400.
        let resp = AppError::Database(DbError::ForeignKeyViolation(
            "a foreign key constraint fails (`kesh`.`contacts`, CONSTRAINT `fk_x`)".into(),
        ))
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "FOREIGN_KEY_VIOLATION");
    }

    #[tokio::test]
    async fn db_fk_violation_journal_entry_linked_to_invoice_maps_to_409_actionable() {
        // #184 : suppression d'une écriture liée à une facture validée → message
        // actionable dédié, 409 (et pas le générique « Référence invalide »).
        let resp = AppError::Database(DbError::ForeignKeyViolation(
            "Cannot delete or update a parent row: a foreign key constraint fails \
             (`kesh`.`invoices`, CONSTRAINT `fk_invoices_journal_entry` FOREIGN KEY \
             (`journal_entry_id`) REFERENCES `journal_entries` (`id`))"
                .into(),
        ))
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "JOURNAL_ENTRY_LINKED_TO_INVOICE");
        let message = body["error"]["message"].as_str().unwrap();
        assert!(
            message.contains("facture"),
            "message non actionable: {message}"
        );
    }

    #[tokio::test]
    async fn db_connection_unavailable_maps_to_503() {
        let resp =
            AppError::Database(DbError::ConnectionUnavailable("timeout".into())).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"]["code"], "SERVICE_UNAVAILABLE");
    }

    #[tokio::test]
    async fn db_unique_constraint_maps_to_409() {
        let resp =
            AppError::Database(DbError::UniqueConstraintViolation("dup".into())).into_response();
        let (status, _) = response_body(resp).await;
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn db_check_constraint_maps_to_400() {
        let resp =
            AppError::Database(DbError::CheckConstraintViolation("bad".into())).into_response();
        let (status, _) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // --- Story 5.3 PDF QR Bill ---

    #[tokio::test]
    async fn invoice_not_validated_maps_to_400() {
        let resp = AppError::InvoiceNotValidated.into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVOICE_NOT_VALIDATED");
    }

    #[tokio::test]
    async fn invoice_not_pdf_ready_maps_to_400() {
        let resp = AppError::InvoiceNotPdfReady("Adresse client manquante".into()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVOICE_NOT_PDF_READY");
        assert_eq!(body["error"]["message"], "Adresse client manquante");
    }

    #[tokio::test]
    async fn invoice_too_many_lines_maps_to_400() {
        let resp = AppError::InvoiceTooManyLinesForPdf(42).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVOICE_TOO_MANY_LINES_FOR_PDF");
        assert!(body["error"]["message"].as_str().unwrap().contains("42"));
    }

    #[tokio::test]
    async fn pdf_generation_failed_maps_to_500_without_leaking_detail() {
        let resp = AppError::PdfGenerationFailed("printpdf internal: offset 0xdeadbeef".into())
            .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "PDF_GENERATION_FAILED");
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(!msg.contains("0xdeadbeef"), "detail leaked: {msg}");
    }

    // Story 9-2b T6.4 (Pass 1 AA-LOW-03 promu MEDIUM + Pass 3 ECH3-C2) — couvre
    // AC #17 et AC #18 : variant `GlobalExportFailed` → status 500 + code stable
    // `GLOBAL_EXPORT_FAILED` + détail jamais leaké en body (UX-DR38).
    #[tokio::test]
    async fn global_export_failed_maps_to_500_without_leaking_detail() {
        let resp = AppError::GlobalExportFailed("zip finish: stream truncated 0xfeedface".into())
            .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "GLOBAL_EXPORT_FAILED");
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(!msg.contains("0xfeedface"), "detail leaked: {msg}");
    }
}
