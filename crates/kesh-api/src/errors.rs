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
use kesh_db::errors::{
    DbError, ModificationGuard, RejectedRevenueAccount, RevenueAccountRejection, ReversalBlocker,
    SettlementCancelBlocker, UnvalidationBlocker,
};
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
///
/// ⚠️ Fluent entoure chaque variable interpolée de marques d'isolation BiDi
/// (U+2068 / U+2069), invisibles à l'écran mais **copiées** avec le texte. Un
/// message d'erreur se lit, et parfois se copie : celui du PDF figé manquant
/// (Story 25-6-b) nomme le fichier `{empreinte}.pdf` à chercher dans une
/// sauvegarde — copié avec ses marques, il ne trouverait rien. Elles sont donc
/// retirées (précédent : `routes/contacts.rs`, libellé des conditions).
fn t_args(key: &str, default: &str, args: &FluentArgs<'_>) -> String {
    let guard = I18N.read().expect("I18N read lock");
    match guard.as_ref() {
        Some((bundle, locale)) => bundle
            .format(locale, key, Some(args))
            .replace(['\u{2068}', '\u{2069}'], ""),
        None => default.to_string(),
    }
}

/// Code stable d'une raison de refus de compte de produit, exposé dans
/// `details.rejected[].reason` (Story 16-1a). Non traduit — c'est un
/// discriminant machine, le texte lisible est dans `message`.
fn revenue_account_rejection_code(reason: RevenueAccountRejection) -> &'static str {
    match reason {
        RevenueAccountRejection::UnknownOrCrossCompany => "UNKNOWN_OR_CROSS_COMPANY",
        RevenueAccountRejection::Inactive => "INACTIVE",
        RevenueAccountRejection::NotRevenue => "NOT_REVENUE",
        RevenueAccountRejection::NotPostable => "NOT_POSTABLE",
    }
}

/// Compose la partie variable du message d'erreur des comptes de produit de
/// ligne (Story 16-1a, D6) : un fragment localisé **par site en défaut**,
/// joints par « ; ».
///
/// Chaque fragment nomme son sujet — « Ligne 3 », ou « le compte de produit par
/// défaut de la société » quand aucune ligne ne porte le compte (AC8-bis) —
/// puis la raison. Tous les sites sont listés : quand un compte partagé est
/// archivé, plusieurs lignes tombent ensemble, et n'en nommer qu'une imposerait
/// autant d'allers-retours que de lignes fautives.
fn format_rejected_revenue_accounts(rejected: &[RejectedRevenueAccount]) -> String {
    rejected
        .iter()
        .map(|r| {
            let subject = match r.line_number {
                Some(n) => {
                    let fallback = format!("Ligne {n}");
                    let mut args = FluentArgs::new();
                    args.set("line", n);
                    t_args("invoice-line-account-subject-line", &fallback, &args)
                }
                None => t(
                    "invoice-line-account-subject-default",
                    "le compte de produit par défaut de la société",
                ),
            };
            let number = r.account_number.clone().unwrap_or_default();
            let (key, fallback) = match r.reason {
                RevenueAccountRejection::UnknownOrCrossCompany => (
                    "invoice-line-account-unknown",
                    format!(
                        "{subject} : le compte sélectionné est introuvable ou n'appartient pas à cette société"
                    ),
                ),
                RevenueAccountRejection::Inactive => (
                    "invoice-line-account-inactive",
                    format!("{subject} : le compte {number} est archivé"),
                ),
                RevenueAccountRejection::NotRevenue => (
                    "invoice-line-account-not-revenue",
                    format!("{subject} : le compte {number} n'est pas un compte de produit"),
                ),
                // La condition du variant est « non imputable **et** différent
                // du compte par défaut » (exemption D3-bis). Ne PAS mentionner
                // le défaut dans le message : « n'est plus le défaut »
                // présupposerait qu'il l'a été, ce qui est faux dans le cas
                // courant — un compte que l'utilisateur vient de choisir et qui
                // n'a jamais été le défaut de la société.
                RevenueAccountRejection::NotPostable => (
                    "invoice-line-account-not-postable",
                    format!(
                        "{subject} : le compte {number} n'est pas imputable — choisissez un autre compte"
                    ),
                ),
            };
            let mut args = FluentArgs::new();
            args.set("subject", subject.clone());
            args.set("number", number);
            t_args(key, &fallback, &args)
        })
        .collect::<Vec<_>>()
        .join(" ; ")
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

    // --- Story 14-2 ---
    /// Transition d'état métier interdite avec **message client distinct** (409
    /// `ILLEGAL_STATE_TRANSITION`). Contrairement à
    /// `AppError::Database(DbError::IllegalStateTransition)` — dont le `Display`
    /// est log-only et produit un message générique figé —, le `String` porté
    /// ici est **déjà localisé** (construit via `t(...)` au mapper) et rendu
    /// tel quel au client. Cohérent avec le précédent `AppError::Validation`
    /// (que `map_create_error` construit déjà via `Validation(t(...))`).
    ///
    /// Utilisé par `map_reopen_error` (Story 14-2, D7) pour distinguer les deux
    /// conflits de réouverture (« déjà ouvert » vs garde LIFO) qui partagent le
    /// code machine mais ont des messages utilisateur distincts.
    #[error("Transition d'état interdite : {0}")]
    IllegalState(String),

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

    // --- Story 22-4a — un PAT n'atteint aucune route d'administration (#167) ---
    /// Une requête authentifiée par PAT atteint une route `require_admin_role`
    /// → `403` code `API_KEY_ADMIN_FORBIDDEN`.
    ///
    /// **Distincte de [`AppError::ApiKeyManagementForbidden`] à dessein** (D2) :
    /// réutiliser celle-ci pour « vous ne pouvez pas rouvrir un exercice avec un
    /// jeton » mentirait à l'appelant, son message parlant de gestion de clés.
    /// Les deux coexistent — la gestion des clés vit dans `comptable_routes` et
    /// garde son propre code (D5).
    ///
    /// Rendue par la couche `require_not_pat` posée sur `admin_routes`, donc
    /// **avant** le handler, quel que soit le rôle du créateur de la clé (D6).
    #[error("Administration interdite via clé API")]
    ApiKeyAdminForbidden,

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

    // --- Story 16-2a (#144) ---
    /// Compte de produit invalide sur une **fiche produit** du catalogue.
    ///
    /// Variante dédiée, et non `AppError::Validation` — décision **D10**.
    /// Deux raisons, chacune suffisante :
    ///
    /// 1. `Validation` rend un code **figé** à `VALIDATION_ERROR`
    ///    (cf. son bras d'`IntoResponse`), il n'y a nulle part où glisser
    ///    `PRODUCT_REVENUE_ACCOUNT_INVALID` qu'exige l'AC-A3 ;
    /// 2. la seule autre voie du dépôt vers un code spécifique sur ce sujet
    ///    passe par `DbError::InvalidRevenueAccounts` — c'est-à-dire la couche
    ///    repository, que **D4** interdit puisque la validation doit rester à
    ///    la route pour disposer de l'état antérieur.
    ///
    /// Le motif réutilise [`RevenueAccountRejection`] : l'enum n'est jamais
    /// dupliquée. Le sujet, lui, est propre à cette variante — le formateur
    /// `format_rejected_revenue_accounts` choisit le sien par `match` sur un
    /// `Option<i32>`, structure qui n'admet pas de troisième cas.
    #[error("Compte de produit invalide sur la fiche produit")]
    ProductRevenueAccountInvalid(RevenueAccountRejection),

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

    /// Story 15-12a (#543, choix C-15-12a-1) — la **création** d'un exercice est
    /// refusée : un exercice postérieur à sa date de début est clôturé. Même
    /// code, même statut et mêmes `details` que `DbError::LaterFiscalYearClosed`
    /// (`400 LATER_FISCAL_YEAR_CLOSED`), mais un message **propre à la création**
    /// (`error-fiscal-year-create-later-closed`) : celui des écritures conseille
    /// une contre-passation, qui n'a pas d'objet ici. Produite par
    /// `routes::fiscal_years::map_create_error`.
    #[error("Exercice postérieur {fiscal_year_name} clôturé — création refusée")]
    FiscalYearBeforeClosedYear {
        fiscal_year_id: i64,
        fiscal_year_name: String,
    },

    // --- Story 4.1 ---
    /// Un contact avec ce numéro IDE (CHE) existe déjà dans la même company.
    /// Code client dédié (`IDE_ALREADY_EXISTS`) pour UX précise côté form,
    /// distinct du générique `RESOURCE_CONFLICT` (autres UniqueConstraintViolation).
    /// Le `String` porte le message i18n prêt à afficher.
    #[error("{0}")]
    IdeAlreadyExists(String),

    // --- Story 16-3b (#151) ---
    /// Un contact ACTIF de la même company porte déjà ce numéro de client.
    /// Même nature que `IdeAlreadyExists` — unicité par société sur la même
    /// table — donc **même code HTTP (409)**, avec son propre code client pour
    /// que le formulaire puisse afficher un message dédié.
    #[error("{0}")]
    ClientNumberAlreadyExists(String),

    // --- Story 5.3 — génération PDF QR Bill ---
    /// La facture n'est pas validée — impossible de générer un PDF (400).
    #[error("Facture non validée")]
    InvoiceNotValidated,

    // --- Story 25-6-b (#387) — le PDF figé ---
    /// La facture est annulée par un avoir — pendant le rendu de son PDF, ou
    /// avant un refigeage : rien n'a été figé (400, sur le patron
    /// d'[`Self::InvoiceNotValidated`]).
    #[error("Facture annulée")]
    InvoiceCancelled,

    /// Le PDF figé est référencé mais son fichier manque sous
    /// `KESH_DOCUMENTS_DIR` (410). Jamais régénéré en silence : ce serait
    /// fabriquer une pièce qui n'a pas été émise. Porte l'empreinte, que le
    /// message nomme.
    #[error("PDF figé introuvable : {sha256}.pdf")]
    InvoicePdfGone {
        sha256: String,
        /// Le refigeage est-il possible ? Faux pour une facture annulée : le
        /// message ne propose alors que la restauration (revue P2).
        refreezable: bool,
    },

    /// La facture a changé deux fois de suite pendant le rendu de son PDF
    /// (409) : rien n'a été figé, l'utilisateur réessaie.
    #[error("Facture modifiée pendant le rendu du PDF")]
    InvoiceChanged,

    /// Refigeage refusé : le fichier du PDF figé est présent (409) — on ne
    /// remplace pas un document qui existe.
    #[error("PDF figé présent")]
    InvoicePdfPresent,

    /// Refigeage refusé : la facture n'a pas de PDF figé (409) — son
    /// prochain rendu la figera.
    #[error("PDF non figé")]
    InvoicePdfNotFrozen,

    /// Refigeage refusé : le fichier est présent mais son empreinte ne
    /// correspond plus (409). Un fichier altéré se diagnostique, il ne se
    /// recouvre pas d'un nouveau document. À la **lecture**, la même
    /// altération répond 500 (`Internal`), rien n'est servi.
    #[error("PDF figé altéré")]
    InvoicePdfIntegrity,

    /// Un pré-requis applicatif manque pour générer le PDF : adresse contact,
    /// compte bancaire primary, IBAN invalide, etc. Le `String` contient la
    /// description i18n renvoyée au client.
    #[error("Facture non prête pour PDF : {0}")]
    InvoiceNotPdfReady(String),

    /// Trop de lignes pour tenir sur un PDF A4 mono-page. La limite réelle est
    /// **géométrique** (garde de `kesh-qrbill::pdf`) : elle dépend du nombre de
    /// taux du récap TVA (#151) et du type de document (une facture s'arrête au
    /// séparateur QR, un avoir dispose de la pleine page). `MAX_LINES_PER_PDF`
    /// reste un pré-filtre grossier pour les factures. Le `usize` = nb de lignes.
    #[error("Facture trop de lignes pour PDF : {0}")]
    InvoiceTooManyLinesForPdf(usize),

    /// En-tête du PDF (émetteur + destinataire) débordant sur le tableau des
    /// lignes — Story 16-3a (#151).
    ///
    /// ⚠️ **Variante DÉDIÉE, et non `Validation`.** L'écran de facture résout le
    /// message par une **liste blanche fermée de codes** (`PDF_ERROR_KEYS`) :
    /// un `VALIDATION_ERROR` n'y figure pas et retombe sur « Erreur lors du
    /// téléchargement du PDF », **jetant le message soigné**. C'est le patron
    /// d'`InvoiceTooManyLinesForPdf` ci-dessus qu'il faut suivre — sans quoi les
    /// quatre traductions sont mortes sur ce chemin, alors que le manuel promet
    /// un message explicite. *(Revue de code, passe 3.)*
    #[error("En-tête du PDF trop haut pour la page")]
    InvoicePdfHeaderOverflow,

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

    /// Story 17-3c — échec de l'**import** complet d'installation **après**
    /// l'écriture réussie de la sauvegarde pré-import (panne DB pendant le
    /// restore transactionnel, rejeu des backfills, dataset source sans aucun
    /// compte Admin, commit). HTTP 500 ; la destination reste intacte
    /// (rollback) et la sauvegarde pré-import existe. Story 15-13b (#576) : un
    /// échec **antérieur** à la sauvegarde rend `AdminPreImportBackupFailed`.
    /// ⚠️ Cette variante est encore **construite** avant la sauvegarde par
    /// `admin_backup::import::check_schema_compat` (lecture du schéma) : c'est
    /// la **conversion à l'appel** (`routes::admin::avant_sauvegarde`) qui la
    /// change en `AdminPreImportBackupFailed` — tout nouvel appelant d'avant
    /// sauvegarde doit la reprendre (revue P1, A-3).
    /// Détail loggé, jamais exposé en HTTP body.
    #[error("Échec import installation : {0}")]
    AdminFullImportFailed(String),

    /// Story 15-13b (#576) — échec de l'import d'installation **avant** que la
    /// sauvegarde de sécurité soit sur disque : transaction, verrou, lecture du
    /// schéma ou des données, création du dossier, écriture du fichier. Rien
    /// n'a été supprimé et **aucune sauvegarde n'existe** : le message ne la
    /// promet pas, et nomme les deux pistes (dossier de sauvegarde, base de
    /// données). HTTP 500, code `ADMIN_PRE_IMPORT_BACKUP_FAILED`, clé
    /// `error-admin-pre-import-backup-failed`. Détail loggé, jamais exposé.
    #[error("Échec sauvegarde pré-import : {0}")]
    AdminPreImportBackupFailed(String),

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
    /// Story 21-5b — rappel sur une facture aux rappels suspendus. 422.
    #[error("Rappels suspendus pour cette facture")]
    DunningPaused,
    /// Story 25-4-b2 (#416) — rappel d'une facture dont le reste dû, arrondi au
    /// centime, est nul ou négatif (état hérité : trop-perçu d'avant la 0.12.1).
    /// Refus NOMMÉ plutôt qu'un `INVOICE_NOT_PDF_READY` trompeur (la QR refuse
    /// un montant ≤ 0). 422.
    #[error("Rien à réclamer sur cette facture")]
    ReminderNothingDue,
    /// Story 25-4-b2 (#416) — les montants d'un rappel ont changé entre l'aperçu et
    /// l'envoi (un règlement est arrivé) : le texte validé ne correspond plus à la
    /// QR. Rien n'est envoyé ; rouvrir l'aperçu. 409.
    #[error("Montants du rappel modifiés depuis l'aperçu")]
    ReminderAmountsChanged,
    /// Story 21-5b — envoi d'un niveau de rappel > prochain attendu (saut interdit,
    /// ou niveau déjà couvert par un envoi concurrent). 409.
    #[error("Niveau de rappel déjà couvert")]
    LevelAlreadySent,
    /// Story 21-5b — l'e-mail de rappel est PARTI mais la facture a **réellement
    /// disparu** avant l'enregistrement (#219). 409 — le fait est tracé best-effort
    /// (audit). Réservé au cas `NotFound` : toute autre panne d'enregistrement
    /// utilise [`AppError::ReminderSentButNotRecorded`] (review Pass 3 — annoncer
    /// une facture disparue sur un simple hoquet DB envoyait chercher une
    /// suppression qui n'avait pas eu lieu).
    #[error("Rappel envoyé mais facture disparue")]
    ReminderSentButInvoiceGone,
    /// Story 21-5b (code review Pass 3) — l'e-mail de rappel est PARTI mais son
    /// enregistrement a échoué pour une raison **autre** qu'une facture disparue
    /// (deadlock, timeout de pool, panne au commit). 409 — tracé best-effort.
    /// Pendant unitaire du code per-facture `RECORD_FAILED_EMAIL_SENT` du lot.
    #[error("Rappel envoyé mais non enregistré")]
    ReminderSentButNotRecorded,
    /// Story 21-5b — envoi par lot dépassant le cap dur (20). 422.
    #[error("Lot de rappels trop volumineux")]
    BatchTooLarge,
    /// Story 21-5b (code review Pass 3) — le lot dépasse le quota d'envoi par
    /// fenêtre du rate-limiter. 422 et non 429 : aucune attente ne peut le rendre
    /// acceptable (la fenêtre ne libère jamais plus de `max_attempts` slots).
    #[error("Lot supérieur au quota d'envoi ({max})")]
    BatchExceedsSendQuota { max: u32 },

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

    /// Story 22-1 (#294/#295) — le backup porte des numéros de client dont les
    /// formes canoniques se percutent : le backfill D6 refuse, l'import est
    /// rejeté **avant COMMIT** (l'état précédent est préservé). HTTP 400
    /// `IMPORT_CLIENT_NUMBER_COLLISION` avec `details.report` — le rapport
    /// nominatif (société, ids, valeurs affichées, invisibles échappés) est
    /// l'outil de réparation : un backup en collision ne s'installe pas, il se
    /// répare d'abord.
    #[error("Collisions de numéros de client dans le backup")]
    ImportClientNumberCollision { report: String },

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
    /// Advisory lock `GET_LOCK('reconcile:{base}:{company}:{account}', timeout)`
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

    /// Story 25-4-c2 (#480) — la transaction d'un lot d'acceptation a été
    /// annulée par InnoDB (interblocage). **Rejouable** : `post_accept` la
    /// rejoue ; elle n'arrive au client, en `500`, qu'une fois les tentatives
    /// épuisées.
    #[error("Transaction de réconciliation annulée par un interblocage")]
    ReconciliationTransactionAborted,

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

    /// Contrôle de forme refusé sur `POST /api/v1/opening-balances/complete`
    /// (Story 25-7, #445) → **400** avec un code `OPENING_COMPLEMENT_*` par
    /// cause (aucune ligne, trop de lignes, montant, compte en double).
    #[error("Complément des soldes de départ refusé [{error_code}] : {message}")]
    OpeningComplementInvalid {
        error_code: &'static str,
        message: String,
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
            // Story 25-4-d2c — même traitement qu'un invariant cassé : journalisé,
            // 500, jamais un rapport faux.
            ReportError::CorruptData(detail) => {
                tracing::error!(%detail, "rapport : donnée corrompue — invariant cassé");
                AppError::Internal(format!("corrupt data: {detail}"))
            }
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

/// Corps `400 LATER_FISCAL_YEAR_CLOSED` (Stories 15-8a, 15-12a) : un seul
/// gabarit pour les deux messages — celui des écritures
/// (`error-later-fiscal-year-closed`) et celui de la création d'un exercice
/// (`error-fiscal-year-create-later-closed`) —, pour que leurs `details` ne
/// divergent jamais.
fn later_fiscal_year_closed_response(
    fiscal_year_id: i64,
    fiscal_year_name: &str,
    key: &str,
    fallback: &str,
) -> Response {
    let mut args = FluentArgs::new();
    args.set("name", fiscal_year_name.to_string());
    let message = t_args(key, fallback, &args);
    let body = serde_json::json!({
        "error": {
            "code": "LATER_FISCAL_YEAR_CLOSED",
            "message": message,
            "details": {
                "fiscalYearId": fiscal_year_id,
                "fiscalYearName": fiscal_year_name,
            },
        }
    });
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

/// Clé i18n et repli d'un motif de contre-passation (Story 24-4a), partagés
/// par le refus de contre-passation et le refus de modification (Story 15-8a,
/// D2 — mêmes codes, mêmes messages : ils nomment déjà le chemin de la pièce).
fn reversal_blocker_message(blocker: ReversalBlocker) -> (&'static str, &'static str) {
    match blocker {
        ReversalBlocker::IsAReversal => (
            "journal-entries-reverse-blocked-is-a-reversal",
            "Cette écriture est elle-même une contre-passation.",
        ),
        ReversalBlocker::AlreadyReversed => (
            "journal-entries-reverse-blocked-already-reversed",
            "Cette écriture a déjà été contre-passée.",
        ),
        ReversalBlocker::OwnedByInvoice => (
            "journal-entries-reverse-blocked-invoice",
            "Cette écriture appartient à une facture client : dévalidez la facture, ou corrigez-la par un avoir.",
        ),
        ReversalBlocker::OwnedByCreditNote => (
            "journal-entries-reverse-blocked-credit-note",
            "Cette écriture est celle d'un avoir, qui est déjà une contre-passation.",
        ),
        ReversalBlocker::OwnedBySupplierInvoice => (
            "journal-entries-reverse-blocked-supplier-invoice",
            "Cette écriture appartient à une facture fournisseur : elle se corrige depuis la fiche de la facture, qui indique ce qui est possible.",
        ),
        ReversalBlocker::OwnedBySettlement => (
            "journal-entries-reverse-blocked-settlement",
            "Cette écriture est un règlement de facture : annulez le règlement depuis la fiche de la facture, qui indique si c'est possible.",
        ),
        ReversalBlocker::MatchedBankTransaction => (
            "journal-entries-reverse-blocked-bank-match",
            "Cette écriture est rapprochée d'une transaction bancaire : annulez le rapprochement depuis le détail de l'import bancaire.",
        ),
        // ⚠️ **Aucun chemin ne construit ce cas aujourd'hui** :
        // l'écriture l'exclut délibérément pour atteindre
        // `ReversalAccountsArchived`, un 400 qui NOMME tous les
        // comptes. La branche existe pour que le `match` reste
        // exhaustif. *(Le commentaire précédent la disait servie
        // par la lecture : c'était faux, la lecture ne construit
        // aucun `DbError`. Passe 2 de revue.)*
        ReversalBlocker::AccountArchived => (
            "journal-entries-reverse-blocked-account-archived",
            "Un compte de cette écriture a été archivé : réactivez-le pour pouvoir la contre-passer.",
        ),
    }
}

/// Réponse **409** d'une écriture qu'une pièce (ou un état) empêche de toucher —
/// contre-passation (24-4a) ou modification (15-8a) : code du motif, message
/// traduit suffixé du **numéro** de la pièce quand elle en a un, et
/// `details.documentId` / `details.documentNumber`.
///
/// ⚠️ Le numéro est suffixé au message : l'utilisateur ne connaît pas les
/// identifiants de la base, il connaît le numéro sur son document.
fn entry_document_refusal_response(
    code: &'static str,
    fallback_key: &'static str,
    fallback: &'static str,
    document_id: Option<i64>,
    document_label: Option<String>,
) -> Response {
    let base = t(fallback_key, fallback);
    let message = match document_label.as_deref() {
        Some(numero) => format!("{base} ({numero})"),
        None => base,
    };
    let body = serde_json::json!({
        "error": {
            "code": code,
            "message": message,
            "details": {
                "documentId": document_id,
                "documentNumber": document_label,
            },
        }
    });
    (StatusCode::CONFLICT, Json(body)).into_response()
}

/// Réponse **400 `ACCOUNT_NOT_POSTABLE`** commune aux deux variantes qui la
/// portent — [`DbError::AccountsNotPostable`] (Story 15-5a) et
/// [`DbError::DesignatedAccountsNotPostable`] (Story 15-5d) : même code, même
/// détail ([`kesh_db::errors::NonPostableAccounts::details`], choix C29), seul
/// le **message** diffère (`key`).
///
/// ⚠️ `count` est passé comme NOMBRE Fluent : en chaîne, le sélecteur `[one]`
/// ne s'appliquerait jamais. Chaque `key` est inscrite à
/// `SELECTEURS_RESOLUS_COTE_SERVEUR` (kesh-i18n).
fn account_not_postable_response(
    key: &str,
    fallback: &str,
    accounts: &kesh_db::errors::NonPostableAccounts,
) -> Response {
    let mut args = FluentArgs::new();
    args.set("numbers", accounts.numbers().join(", "));
    args.set("count", accounts.len());
    let msg = t_args(key, fallback, &args);
    let body = serde_json::json!({
        "error": {
            "code": "ACCOUNT_NOT_POSTABLE",
            "message": msg,
            "details": accounts.details(),
        }
    });
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

/// Réponse d'un refus de complément des soldes de départ (Story 25-7, AC 5) :
/// statut, code et message **par cause**. Les refus qui portent sur un compte le
/// nomment par son numéro — un identifiant de base ne se comprend pas.
fn opening_complement_refusal_response(
    reason: kesh_db::repositories::opening_complement::OpeningComplementRefusal,
    account_number: Option<&str>,
) -> Response {
    use kesh_db::repositories::opening_complement::OpeningComplementRefusal as R;
    let account = account_number.unwrap_or("?");
    let mut args = FluentArgs::new();
    args.set("account", account.to_string());
    let (status, key, fallback) = match reason {
        R::NoEntries => (
            StatusCode::CONFLICT,
            "error-opening-complement-no-entries",
            "La société n'a encore aucune écriture : saisissez les soldes de départ avec la génération du bilan d'ouverture.".to_string(),
        ),
        R::NoOpenFiscalYear => (
            StatusCode::CONFLICT,
            "error-opening-complement-no-open-fiscal-year",
            "Aucun exercice ouvert ne couvre la date du jour : le complément ne peut pas être daté.".to_string(),
        ),
        R::DateLocked => (
            StatusCode::CONFLICT,
            "error-opening-complement-date-locked",
            "La date du jour tombe dans la période verrouillée : le complément ne peut pas être daté.".to_string(),
        ),
        R::NoRetainedEarnings => (
            StatusCode::CONFLICT,
            "error-opening-complement-no-retained-earnings",
            "Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : attribuez-le dans le plan comptable, la contrepartie du complément y est portée.".to_string(),
        ),
        R::RetainedEarningsNotPostable => (
            StatusCode::CONFLICT,
            "error-opening-complement-retained-earnings-not-postable",
            "Le compte de bénéfice reporté n'est pas imputable : rendez-le imputable dans le plan comptable.".to_string(),
        ),
        R::AccountInvalid => (
            StatusCode::BAD_REQUEST,
            "error-opening-complement-account-invalid",
            format!("Le compte {account} est inconnu, archivé ou non imputable."),
        ),
        R::RetainedEarningsLine => (
            StatusCode::BAD_REQUEST,
            "error-opening-complement-retained-earnings-line",
            format!("Le compte {account} est le compte de bénéfice reporté : sa contrepartie est calculée par Kesh, ne le saisissez pas."),
        ),
        R::NotBalanceAccount => (
            StatusCode::BAD_REQUEST,
            "error-opening-complement-not-balance-account",
            format!("Le compte {account} est un compte de résultat : seuls les comptes de bilan se complètent."),
        ),
        R::AccountMoved => (
            StatusCode::CONFLICT,
            "error-opening-complement-account-moved",
            format!("Le compte {account} a déjà des mouvements : corrigez-le dans le journal, en modifiant l'écriture, par une contre-passation ou une écriture de correction."),
        ),
    };
    build_response(status, reason.code(), &t_args(key, &fallback, &args))
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

            // Story 16-2a (#144) — D10. Le sujet désigne **l'article**, jamais
            // le réglage société : réutiliser le formateur de 16-1a ferait lire,
            // à qui édite une fiche produit, un message pointant un autre objet.
            AppError::ProductRevenueAccountInvalid(reason) => {
                let (key, fallback) = match reason {
                    RevenueAccountRejection::UnknownOrCrossCompany => (
                        "product-revenue-account-unknown",
                        "Le compte de produit de cet article est introuvable ou n'appartient pas à cette société.",
                    ),
                    RevenueAccountRejection::Inactive => (
                        "product-revenue-account-inactive",
                        "Le compte de produit de cet article est archivé.",
                    ),
                    RevenueAccountRejection::NotRevenue => (
                        "product-revenue-account-not-revenue",
                        // ⚠️ Le sujet est « le compte DE cet article », pas « le
                        // compte DE PRODUIT de cet article » : la seconde forme
                        // énonce « X n'est pas X ». Les trois autres locales
                        // nomment correctement le sujet depuis l'origine ; le FR
                        // — la langue de service — était le seul à boucler.
                        "Le compte de cet article n'est pas un compte de produit.",
                    ),
                    // D3 exclut délibérément `postable` de la validation de la
                    // fiche article : aucun des quatre sites de construction de
                    // cette variante (`routes/products.rs:318`, `:323`, `:330`,
                    // `:337`) ne produit `NotPostable`. Le bras subsiste pour
                    // l'exhaustivité du `match` — un `unreachable!()` tuerait la
                    // task sans laisser de trace, ce que le garde-fou défensif
                    // du CLAUDE.md proscrit — mais il CRIE et retombe sur un
                    // message générique **déjà traduit ailleurs**.
                    //
                    // ⚠️ Ne PAS y remettre une clé `product-revenue-account-not-postable`
                    // dédiée : ses quatre traductions ont été retirées en passe 1
                    // de revue précisément parce qu'aucun chemin ne les atteint.
                    // Si ce bras s'exécute un jour, c'est qu'un quatrième critère
                    // a été ajouté à `validate_revenue_account` sans son message.
                    RevenueAccountRejection::NotPostable => {
                        tracing::error!(
                            "ProductRevenueAccountInvalid(NotPostable) émis alors que D3 exclut `postable` — un critère a-t-il été ajouté sans son message ?"
                        );
                        (
                            "common-account-invalid",
                            "Compte invalide — non imputable, archivé ou de type inattendu",
                        )
                    }
                };
                let msg = t(key, fallback);
                let body = serde_json::json!({
                    "error": {
                        "code": "PRODUCT_REVENUE_ACCOUNT_INVALID",
                        "message": msg,
                        // `revenue_account_rejection_code` est RÉUTILISÉ, jamais
                        // réécrit : un second `match` sur le même enum
                        // divergerait au premier variant ajouté.
                        "details": { "reason": revenue_account_rejection_code(reason) },
                    }
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
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

            // Story 14-2 (D7) — 409 avec message déjà localisé (distinct du
            // générique DbError::IllegalStateTransition, log-only).
            AppError::IllegalState(msg) => {
                build_response(StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION", &msg)
            }

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

            // Story 22-4a — administration interdite via PAT (#167).
            AppError::ApiKeyAdminForbidden => build_response(
                StatusCode::FORBIDDEN,
                "API_KEY_ADMIN_FORBIDDEN",
                &t(
                    "error-api-key-admin-forbidden",
                    "Les routes d'administration ne sont pas accessibles via une clé API, quel que soit le rôle de son créateur. Utilisez l'interface web.",
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

            AppError::FiscalYearBeforeClosedYear {
                fiscal_year_id,
                fiscal_year_name,
            } => {
                let fallback = format!(
                    "L'exercice « {fiscal_year_name} », postérieur, est clôturé, et son bilan reprend tout ce qui le précède : aucun exercice ne peut être créé avant sa date de début tant qu'il l'est. Pour créer celui-ci, un administrateur rouvre d'abord les exercices clôturés, en commençant par le plus récent."
                );
                later_fiscal_year_closed_response(
                    fiscal_year_id,
                    &fiscal_year_name,
                    "error-fiscal-year-create-later-closed",
                    &fallback,
                )
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

            // Story 16-3b : contrainte jumelle de l'IDE → même 409.
            AppError::ClientNumberAlreadyExists(msg) => {
                build_response(StatusCode::CONFLICT, "CLIENT_NUMBER_ALREADY_EXISTS", &msg)
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
            AppError::DunningPaused => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "DUNNING_PAUSED",
                &t(
                    "error-dunning-paused",
                    "Les rappels sont suspendus pour cette facture.",
                ),
            ),
            AppError::ReminderAmountsChanged => build_response(
                StatusCode::CONFLICT,
                "REMINDER_AMOUNTS_CHANGED",
                &t(
                    "error-reminder-amounts-changed",
                    "Le montant dû a changé depuis l'aperçu (un règlement est arrivé) : rouvrez l'aperçu avant d'envoyer.",
                ),
            ),
            AppError::ReminderNothingDue => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "REMINDER_NOTHING_DUE",
                &t(
                    "error-reminder-nothing-due",
                    "Il ne reste rien à réclamer sur cette facture : aucun rappel à envoyer.",
                ),
            ),
            AppError::LevelAlreadySent => build_response(
                StatusCode::CONFLICT,
                "LEVEL_ALREADY_SENT",
                &t(
                    "error-level-already-sent",
                    "Ce niveau de rappel a déjà été traité (envoi concurrent ou saut de niveau interdit).",
                ),
            ),
            AppError::ReminderSentButInvoiceGone => build_response(
                StatusCode::CONFLICT,
                "REMINDER_SENT_BUT_INVOICE_GONE",
                &t(
                    "error-reminder-sent-but-invoice-gone",
                    "Le rappel a été envoyé mais la facture a disparu avant l'enregistrement.",
                ),
            ),
            AppError::ReminderSentButNotRecorded => build_response(
                StatusCode::CONFLICT,
                "REMINDER_SENT_BUT_NOT_RECORDED",
                &t(
                    "error-reminder-sent-but-not-recorded",
                    "Le rappel a été envoyé mais n'a pas pu être enregistré. \
                     L'envoi est tracé dans le journal d'audit.",
                ),
            ),
            AppError::BatchTooLarge => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "BATCH_TOO_LARGE",
                &t(
                    "error-batch-too-large",
                    "Trop de factures dans le lot (maximum 20).",
                ),
            ),
            AppError::BatchExceedsSendQuota { max } => build_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "BATCH_EXCEEDS_SEND_QUOTA",
                &t(
                    "error-batch-exceeds-send-quota",
                    &format!("Le lot dépasse le quota d'envoi ({max} e-mails par fenêtre)."),
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

            // Story 25-6-b (#387) — le PDF figé.
            AppError::InvoiceCancelled => build_response(
                StatusCode::BAD_REQUEST,
                "INVOICE_CANCELLED",
                &t(
                    "error-invoice-cancelled",
                    "La facture est annulée par un avoir : son PDF ne peut pas être produit, et aucun document n'a été figé.",
                ),
            ),
            AppError::InvoicePdfGone {
                sha256,
                refreezable,
            } => {
                // Une facture annulée ne se refige pas : son message ne propose
                // que la restauration (revue P2, R2-2).
                let (key, fallback) = if refreezable {
                    (
                        "error-invoice-pdf-gone",
                        format!(
                            "Le PDF émis de cette facture est introuvable : le fichier {sha256}.pdf manque dans le répertoire des documents (KESH_DOCUMENTS_DIR). Restaurez-le depuis la sauvegarde de ce répertoire ; à défaut, un administrateur peut refiger la facture."
                        ),
                    )
                } else {
                    (
                        "error-invoice-pdf-gone-cancelled",
                        format!(
                            "Le PDF émis de cette facture annulée est introuvable : le fichier {sha256}.pdf manque dans le répertoire des documents (KESH_DOCUMENTS_DIR). Seule sa restauration depuis la sauvegarde de ce répertoire le répare."
                        ),
                    )
                };
                let mut args = FluentArgs::new();
                args.set("sha256", sha256);
                let msg = t_args(key, &fallback, &args);
                build_response(StatusCode::GONE, "INVOICE_PDF_GONE", &msg)
            }
            AppError::InvoiceChanged => build_response(
                StatusCode::CONFLICT,
                "INVOICE_CHANGED",
                &t(
                    "error-invoice-changed",
                    "La facture a été modifiée pendant la préparation de son PDF. Réessayez.",
                ),
            ),
            AppError::InvoicePdfPresent => build_response(
                StatusCode::CONFLICT,
                "INVOICE_PDF_PRESENT",
                &t(
                    "error-invoice-pdf-present",
                    "Le PDF émis de cette facture est présent : il n'y a rien à refiger.",
                ),
            ),
            AppError::InvoicePdfNotFrozen => build_response(
                StatusCode::CONFLICT,
                "INVOICE_PDF_NOT_FROZEN",
                &t(
                    "error-invoice-pdf-not-frozen",
                    "Cette facture n'a pas encore de PDF figé : son prochain téléchargement le figera.",
                ),
            ),
            AppError::InvoicePdfIntegrity => build_response(
                StatusCode::CONFLICT,
                "INVOICE_PDF_INTEGRITY",
                &t(
                    "error-invoice-pdf-integrity",
                    "Le fichier du PDF émis a été altéré : son empreinte ne correspond plus. Restaurez-le depuis la sauvegarde, ou écartez-le du répertoire des documents avant de refiger.",
                ),
            ),
            AppError::InvoiceTooManyLinesForPdf(n) => {
                // #151 code-review : le cap n'est plus un nombre fixe (il dépend du
                // récap TVA et du type de document), donc message sans « max ».
                let fallback = format!(
                    "La facture contient {n} lignes — le PDF A4 mono-page ne peut pas toutes les afficher avec le récapitulatif TVA. Réduisez le nombre de lignes ou scindez la facture."
                );
                let mut args = FluentArgs::new();
                args.set("count", n as i64);
                let msg = t_args("error-invoice-too-many-lines-for-pdf", &fallback, &args);
                build_response(
                    StatusCode::BAD_REQUEST,
                    "INVOICE_TOO_MANY_LINES_FOR_PDF",
                    &msg,
                )
            }
            AppError::InvoicePdfHeaderOverflow => {
                let msg = t(
                    "error-invoice-pdf-header-overflow",
                    "L'en-tête du document ne tient pas sur la page. Supprimez une \
                     coordonnée — téléphone, e-mail ou site web — dans les réglages : \
                     les raccourcir ne libère aucune place, chaque coordonnée occupe \
                     une ligne entière. Ou réduisez le nombre de lignes de l'adresse \
                     du destinataire.",
                );
                build_response(StatusCode::BAD_REQUEST, "INVOICE_PDF_HEADER_OVERFLOW", &msg)
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
                        "Échec de l'import de l'installation. L'état précédent a été préservé ; \
                         une sauvegarde de sécurité a été écrite avant l'opération.",
                    ),
                )
            }

            // Story 15-13b (#576) — échec antérieur à la sauvegarde pré-import.
            AppError::AdminPreImportBackupFailed(detail) => {
                tracing::error!("admin pre-import backup failed: {detail}");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "ADMIN_PRE_IMPORT_BACKUP_FAILED",
                    &t(
                        "error-admin-pre-import-backup-failed",
                        "L'import de l'installation a échoué avant ou pendant l'écriture de la \
                         sauvegarde de sécurité : aucune sauvegarde n'a été créée et rien n'a été \
                         supprimé. Vérifiez que le dossier de sauvegarde est inscriptible et que la \
                         base de données est joignable (les journaux du serveur indiquent la cause), \
                         puis réessayez.",
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

            AppError::ImportClientNumberCollision { report } => {
                tracing::warn!("import refused, client number collisions:\n{report}");
                let body = serde_json::json!({
                    "error": {
                        "code": "IMPORT_CLIENT_NUMBER_COLLISION",
                        "message": t(
                            "error-import-client-number-collision",
                            "Le backup contient des numéros de client en collision. Corrigez les fiches nommées dans le rapport, ré-exportez, puis réessayez.",
                        ),
                        "details": { "report": report }
                    }
                });
                (StatusCode::BAD_REQUEST, Json(body)).into_response()
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
            AppError::ReconciliationTransactionAborted => {
                tracing::error!("reconciliation accept: transaction aborted, retries exhausted");
                build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    &t("error-internal", "Erreur interne"),
                )
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

            AppError::OpeningComplementInvalid {
                error_code,
                message,
            } => build_response(StatusCode::BAD_REQUEST, error_code, &message),

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
                DbError::OpeningComplementRefused {
                    reason,
                    account_id,
                    account_number,
                } => {
                    // Le numéro quand le compte est de la société ; sinon
                    // l'identifiant que le client a lui-même envoyé (rien n'est
                    // révélé d'une autre société).
                    let label = account_number.or_else(|| account_id.map(|id| format!("#{id}")));
                    opening_complement_refusal_response(reason, label.as_deref())
                }
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
                // Story 25-2-a ([#382], [#274]) — **409 et non 400** : le payload
                // est valide, c'est l'ÉTAT du compte qui s'y oppose. Même lecture
                // que `OptimisticLockConflict` ci-dessus, et symétrique inverse de
                // `ReversalAccountsArchived`, qui reste un 400 parce qu'un compte
                // archivé y est une donnée d'entrée invalide.
                //
                // ⛔ **Le refus NOMME l'ampleur**, sans quoi il ne serait qu'un
                // obstacle à contourner : l'écran doit pouvoir écrire « douze
                // écritures, dont l'exercice 2025 qui est clos » AVANT de demander
                // la confirmation. C'est tout l'écart entre un avertissement
                // bloquant et un refus sec.
                DbError::AccountHasEntries {
                    entry_count,
                    closed_fiscal_years,
                    from_type,
                    to_type,
                } => {
                    let fallback = format!(
                        "Ce compte porte {entry_count} écriture(s) : changer son type reclasserait tout son historique, exercices clos compris."
                    );
                    let mut args = FluentArgs::new();
                    args.set("count", entry_count);
                    let msg = t_args("error-account-has-entries", &fallback, &args);
                    let body = serde_json::json!({
                        "error": {
                            "code": "ACCOUNT_HAS_ENTRIES",
                            "message": msg,
                            "details": {
                                "entryCount": entry_count,
                                "closedFiscalYears": closed_fiscal_years,
                                "fromType": from_type,
                                "toType": to_type,
                            }
                        }
                    });
                    (StatusCode::CONFLICT, Json(body)).into_response()
                }
                DbError::UniqueConstraintViolation(m) => {
                    tracing::warn!("unique violation: {m}");
                    // Course perdue sur un rôle singleton : deux requêtes
                    // concurrentes passent le pré-SELECT du repository, la
                    // seconde heurte la contrainte. Le message MariaDB porte le
                    // nom de la contrainte → on renvoie le code métier plutôt
                    // que « Ressource déjà existante », qui ferait afficher au
                    // formulaire « ce numéro existe déjà ». On ne peut pas
                    // nommer le compte détenteur ici (l'info n'est pas dans
                    // l'erreur), d'où le message sans argument.
                    // Story 24-4a (#380) : deux contre-passations concurrentes
                    // de la MÊME écriture. L'`UNIQUE` porte l'idempotence
                    // structurellement ; il reste à la nommer à l'utilisateur.
                    if m.contains("uq_journal_entries_reverses") {
                        return build_response(
                            StatusCode::CONFLICT,
                            "ALREADY_REVERSED",
                            &t(
                                "journal-entries-reverse-blocked-already-reversed",
                                "Cette écriture a déjà été contre-passée.",
                            ),
                        );
                    }
                    if m.contains("uq_accounts_company_singleton_role") {
                        build_response(
                            StatusCode::CONFLICT,
                            "ACCOUNT_ROLE_ALREADY_ASSIGNED",
                            &t(
                                "accounts-role-conflict-generic",
                                "Ce rôle vient d'être attribué à un autre compte. Rechargez la page.",
                            ),
                        )
                    } else {
                        build_response(
                            StatusCode::CONFLICT,
                            "RESOURCE_CONFLICT",
                            &t("error-conflict", "Ressource déjà existante"),
                        )
                    }
                }
                // Story 14-3a : code client dédié, distinct du générique
                // RESOURCE_CONFLICT, pour que le formulaire puisse NOMMER le
                // compte qui porte déjà le rôle (le mapping 1062 générique
                // renvoie un message fixe et jette le détail). Même motivation
                // que `IdeAlreadyExists`.
                DbError::AccountRoleAlreadyAssigned {
                    role,
                    account_id,
                    account_number,
                    account_name,
                } => {
                    let fallback = format!(
                        "Le rôle est déjà attribué au compte {account_number} — {account_name}. Retirez-le d'abord de ce compte."
                    );
                    let mut args = FluentArgs::new();
                    args.set("number", account_number.clone());
                    args.set("name", account_name.clone());
                    let msg = t_args("accounts-role-conflict", &fallback, &args);
                    let body = serde_json::json!({
                        "error": {
                            "code": "ACCOUNT_ROLE_ALREADY_ASSIGNED",
                            "message": msg,
                            "details": {
                                "role": role,
                                "accountId": account_id,
                                "accountNumber": account_number,
                                "accountName": account_name,
                            },
                        }
                    });
                    (StatusCode::CONFLICT, Json(body)).into_response()
                }
                // Story 14-3a / code review : ces deux cas passaient par des
                // variantes génériques (`ILLEGAL_STATE_TRANSITION` → « Transition
                // d'état interdite », `CHECK_CONSTRAINT_VIOLATION` → « Valeur
                // invalide »), qui ne disent pas à l'utilisateur quoi corriger.
                DbError::AccountParentArchived { parent_number } => {
                    let fallback = format!(
                        "Le compte parent {parent_number} est archivé. Réactivez-le d'abord."
                    );
                    let mut args = FluentArgs::new();
                    args.set("number", parent_number.clone());
                    build_response(
                        StatusCode::CONFLICT,
                        "ACCOUNT_PARENT_ARCHIVED",
                        &t_args("accounts-parent-archived", &fallback, &args),
                    )
                }
                DbError::AccountRoleInvalidForType { role, account_type } => {
                    let fallback = format!(
                        "Le rôle {role} ne peut pas être attribué à un compte de type {account_type}."
                    );
                    let mut args = FluentArgs::new();
                    args.set("role", role.clone());
                    args.set("type", account_type.clone());
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "ACCOUNT_ROLE_INVALID_FOR_TYPE",
                        &t_args("accounts-role-invalid-for-type", &fallback, &args),
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
                                "Cette écriture comptable a été générée par une facture validée et ne peut pas être supprimée directement. Dévalidez d'abord la facture concernée, ou corrigez-la par un avoir.",
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
                // Story 24-4a (#380) — la contre-passation.
                //
                // ⚠️ Le code exposé est celui du `ReversalBlocker`, PAS un code
                // générique : l'écran doit pouvoir dire POURQUOI le bouton est
                // absent, et vers quelle correction se tourner.
                DbError::EntryNotReversable {
                    blocker,
                    document_id,
                    document_label,
                } => {
                    let (fallback_key, fallback) = reversal_blocker_message(blocker);
                    entry_document_refusal_response(
                        blocker.code(),
                        fallback_key,
                        fallback,
                        document_id,
                        document_label,
                    )
                }
                // Story 15-8a (#532, D2) — la modification refusée. ⛔ DRY : pour
                // un motif de pièce, EXACTEMENT la réponse de la contre-passation
                // (même code, mêmes clés, mêmes `details`).
                DbError::EntryNotModifiable(guard) => {
                    let code = guard.code();
                    let document_id = guard.document_id();
                    let document_label = guard.label();
                    let (fallback_key, fallback) = match guard {
                        ModificationGuard::Owned { blocker, .. } => {
                            reversal_blocker_message(blocker)
                        }
                        ModificationGuard::DetachedSupplierSettlement { .. } => (
                            "journal-entries-modify-blocked-detached-settlement",
                            "Ce paiement appartient à une facture fournisseur annulée : l'argent est sorti, il reste figé. Corrigez-le par une contre-passation.",
                        ),
                    };
                    entry_document_refusal_response(
                        code,
                        fallback_key,
                        fallback,
                        document_id,
                        document_label,
                    )
                }
                // Story 15-1a-i (#518) — les refus du lettrage. Un code par
                // cause, dans l'ordre des rangs de la primitive (AC3) ; le 404
                // (rang 2) est le `NotFound` générique, sans message dédié (AC11).
                DbError::LetteringTooFewLines => build_response(
                    StatusCode::BAD_REQUEST,
                    "LETTERING_TOO_FEW_LINES",
                    &t(
                        "error-lettering-too-few-lines",
                        "Un lettrage réunit au moins deux lignes distinctes.",
                    ),
                ),
                DbError::LetteringTooManyLines { max } => {
                    // Le plafond voyage dans la variante : le message le lit,
                    // au lieu d'écrire « 200 » en dur (revue P1, E-4).
                    let fallback = format!("Un lettrage réunit au plus {max} lignes.");
                    let mut args = FluentArgs::new();
                    args.set("max", max as i64);
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "LETTERING_TOO_MANY_LINES",
                        &t_args("error-lettering-too-many-lines", &fallback, &args),
                    )
                }
                DbError::LetteringAccountsDiffer => build_response(
                    StatusCode::CONFLICT,
                    "LETTERING_ACCOUNTS_DIFFER",
                    &t(
                        "error-lettering-accounts-differ",
                        "Les lignes d'un lettrage doivent toutes porter sur le même compte.",
                    ),
                ),
                DbError::LetteringAccountNotLetterable => build_response(
                    StatusCode::CONFLICT,
                    "LETTERING_ACCOUNT_NOT_LETTERABLE",
                    &t(
                        "error-lettering-account-not-letterable",
                        "Ce compte ne se lettre pas : seuls les comptes d'actif et de passif qui ne sont pas des comptes bancaires se lettrent.",
                    ),
                ),
                DbError::LetteringAllLinesInClosedPeriods => build_response(
                    StatusCode::CONFLICT,
                    "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
                    &t(
                        "error-lettering-all-lines-in-closed-periods",
                        "Toutes ces lignes sont dans une période close — exercice clôturé, exercice suivi d'un exercice clôturé, ou période verrouillée : le lettrage n'y change plus.",
                    ),
                ),
                // ⚠️ Texte NEUTRE (« ne se lettre ni ne se délettre ») : la même
                // clé sert le refus 5 du lettrage et le refus 2 du délettrage
                // (AC5). Mêmes `details` que la contre-passation refusée.
                DbError::LetteringLineOwnedByDocument {
                    document_id,
                    document_label,
                    ..
                } => entry_document_refusal_response(
                    "LETTERING_LINE_OWNED_BY_DOCUMENT",
                    "error-lettering-line-owned-by-document",
                    "Une de ces lignes appartient à une pièce : elle ne se lettre ni ne se délettre à la main.",
                    document_id,
                    document_label,
                ),
                DbError::LetteringLineAlreadyLettered { code } => {
                    let fallback = format!("Une de ces lignes est déjà lettrée (code {code}).");
                    let mut args = FluentArgs::new();
                    args.set("code", code.clone());
                    let body = serde_json::json!({
                        "error": {
                            "code": "LETTERING_LINE_ALREADY_LETTERED",
                            "message": t_args("error-lettering-line-already-lettered", &fallback, &args),
                            "details": { "code": code },
                        }
                    });
                    (StatusCode::CONFLICT, Json(body)).into_response()
                }
                DbError::LetteringUnbalanced { difference } => {
                    // Décimal en chaîne, sans zéros de remplissage (`0.0100` → `0.01`).
                    let difference = difference.normalize().to_string();
                    let fallback = format!("Ces lignes ne se soldent pas : écart de {difference}.");
                    let mut args = FluentArgs::new();
                    args.set("difference", difference.clone());
                    let body = serde_json::json!({
                        "error": {
                            "code": "LETTERING_UNBALANCED",
                            "message": t_args("error-lettering-unbalanced", &fallback, &args),
                            "details": { "difference": difference },
                        }
                    });
                    (StatusCode::CONFLICT, Json(body)).into_response()
                }
                DbError::LetteringIsDocument => build_response(
                    StatusCode::CONFLICT,
                    "LETTERING_IS_DOCUMENT",
                    &t(
                        "error-lettering-is-document",
                        "Ce lettrage est celui d'une pièce : annulez le règlement plutôt que de délettrer.",
                    ),
                ),
                // R7 point 4 — un refus MÉTIER (« réessayez »), jamais un 500.
                DbError::LetteringConcurrentChange => build_response(
                    StatusCode::CONFLICT,
                    "LETTERING_CONCURRENT_CHANGE",
                    &t(
                        "error-lettering-concurrent-change",
                        "Le lettrage a changé entre-temps ; réessayez.",
                    ),
                ),
                // Story 15-8a (#532, C-15-8-22) — un exercice POSTÉRIEUR est
                // clos : son bilan cumulatif reprend ce qui le précède. 400, comme
                // `FISCAL_YEAR_CLOSED` : l'état d'un exercice, pas un conflit sur
                // l'objet. Story 15-12a (AC 9) : message neutre, clé
                // `error-later-fiscal-year-closed` — il ne présuppose pas que
                // l'objet visé existe, garde le conseil de contre-passation et
                // ne prescrit jamais de rouvrir l'exercice nommé (la garde LIFO
                // le refuserait dès qu'un plus récent est clos). La création
                // d'un exercice a son propre message
                // (`AppError::FiscalYearBeforeClosedYear`). Le texte nomme la
                // saisie, la modification et la suppression : depuis la Story
                // 15-12b (#543), `LaterFiscalYearClosed` garde les trois sur une
                // écriture (`create_in_tx_inner`, `update`, `delete_in_tx`) ; la
                // 15-12a l'avait borné aux deux dernières (revue P1, B-1).
                //
                // Pourquoi deux clés voisines (revue P1, B-5) :
                // `journal-entries-modify-blocked-later-fiscal-year-closed` est
                // l'explication **d'écran**, posée avant tout geste sur une
                // écriture affichée (« cette écriture ») ; celle-ci est la
                // réponse du **serveur**, qui ne présuppose pas que l'objet visé
                // existe. Même prescription dans les deux (contre-passation, puis
                // réouverture en commençant par le plus récent).
                DbError::LaterFiscalYearClosed {
                    fiscal_year_id,
                    fiscal_year_name,
                } => {
                    let fallback = format!(
                        "L'exercice « {fiscal_year_name} », postérieur, est clôturé, et son bilan reprend tout ce qui le précède : aucune écriture datée avant sa date de début ne peut être enregistrée, modifiée ni supprimée tant qu'il l'est. Une écriture existante se corrige par une contre-passation ; sinon, un administrateur rouvre les exercices clôturés, en commençant par le plus récent."
                    );
                    later_fiscal_year_closed_response(
                        fiscal_year_id,
                        &fiscal_year_name,
                        "error-later-fiscal-year-closed",
                        &fallback,
                    )
                }
                // Story 15-12a (#543) — la clôture d'un exercice précédé d'un
                // exercice ouvert. 409, comme les autres refus de transition de
                // l'écran des exercices, mais un code DÉDIÉ : l'écran traduit
                // tout `ILLEGAL_STATE_TRANSITION` de la clôture en « déjà
                // clôturé », et une intégration doit pouvoir distinguer les deux.
                DbError::EarlierFiscalYearOpen {
                    fiscal_year_id,
                    fiscal_year_name,
                } => {
                    let fallback = format!(
                        "Clôturez d'abord l'exercice « {fiscal_year_name} », plus ancien et encore ouvert : le bilan est cumulatif, et un exercice ne se clôt qu'après tous ceux qui le précèdent."
                    );
                    let mut args = FluentArgs::new();
                    args.set("name", fiscal_year_name.clone());
                    let message = t_args("error-fiscal-year-close-earlier-open", &fallback, &args);
                    let body = serde_json::json!({
                        "error": {
                            "code": "EARLIER_FISCAL_YEAR_OPEN",
                            "message": message,
                            "details": {
                                "fiscalYearId": fiscal_year_id,
                                "fiscalYearName": fiscal_year_name,
                            },
                        }
                    });
                    (StatusCode::CONFLICT, Json(body)).into_response()
                }
                // Story 25-2-b-1 (#440) — la dévalidation refusée.
                //
                // ⛔ **Un code par motif**, jamais le générique : les trois
                // gardes que la suppression de #219 portait rendaient toutes
                // `ILLEGAL_STATE_TRANSITION`, dont le message n'était que
                // journalisé — à l'écran, « transition interdite » ne dit ni ce
                // qui bloque ni quoi faire. Même forme que la contre-passation
                // ci-dessus, jusqu'aux `details`.
                DbError::InvoiceNotUnvalidatable {
                    blocker,
                    document_id,
                    document_label,
                } => {
                    let (fallback_key, fallback) = match blocker {
                        UnvalidationBlocker::Settled => (
                            "error-invoice-unvalidate-blocked-settled",
                            "Cette facture porte un règlement, même partiel : annulez-le d'abord.",
                        ),
                        UnvalidationBlocker::Credited => (
                            "error-invoice-unvalidate-blocked-credited",
                            "Cette facture est créditée par un avoir, qui en est déjà la correction.",
                        ),
                        UnvalidationBlocker::HasReminders => (
                            "error-invoice-unvalidate-blocked-reminders",
                            "Cette facture a un historique de rappels : le dévalider effacerait la preuve du recouvrement.",
                        ),
                        UnvalidationBlocker::Emailed => (
                            "error-invoice-unvalidate-blocked-emailed",
                            "Cette facture a été envoyée au client : corrigez-la par un avoir.",
                        ),
                        UnvalidationBlocker::MatchedBankTransaction => (
                            "error-invoice-unvalidate-blocked-matched",
                            "L'écriture de cette facture est rapprochée d'une transaction bancaire : annulez le rapprochement d'abord.",
                        ),
                    };
                    let base = t(fallback_key, fallback);
                    let message = match document_label.as_deref() {
                        Some(etiquette) => format!("{base} ({etiquette})"),
                        None => base,
                    };
                    let body = serde_json::json!({
                        "error": {
                            "code": blocker.code(),
                            "message": message,
                            "details": {
                                "documentId": document_id,
                                "documentNumber": document_label,
                            },
                        }
                    });
                    (StatusCode::CONFLICT, Json(body)).into_response()
                }
                // Story 25-4-a (#456) — l'avoir refusé sur une facture réglée,
                // même en partie. ⛔ Un code et un message propres : l'ancienne
                // garde rendait `ILLEGAL_STATE_TRANSITION`, dont le texte n'était
                // que journalisé. Le numéro de la facture est suffixé, comme
                // pour la dévalidation.
                DbError::CreditNoteBlockedBySettlement {
                    invoice_id,
                    settlement_id,
                    invoice_number,
                } => {
                    let base = t(
                        "error-credit-note-blocked-settled",
                        "Cette facture porte un règlement, même partiel : annulez-le d'abord pour pouvoir émettre un avoir.",
                    );
                    let message = match invoice_number.as_deref() {
                        Some(numero) => format!("{base} ({numero})"),
                        None => base,
                    };
                    let body = serde_json::json!({
                        "error": {
                            "code": "CREDIT_NOTE_INVOICE_SETTLED",
                            "message": message,
                            "details": {
                                "invoiceId": invoice_id,
                                "settlementId": settlement_id,
                            },
                        }
                    });
                    (StatusCode::CONFLICT, Json(body)).into_response()
                }
                // Story 15-6b (#474) — un règlement viserait le compte qu'il
                // solde. 400 (la donnée proposée est invalide), un code pour les
                // quatre rôles ; le message suit le rôle (le remède diffère).
                DbError::SettlementCounterpartyIsClaimAccount {
                    account_id,
                    account_number,
                    claim,
                    role,
                    batch,
                } => settlement_counterparty_response(
                    account_id,
                    account_number.as_deref(),
                    claim,
                    role,
                    batch.as_ref(),
                ),
                // Story 15-6c (#474) — la configuration qui préparerait
                // l'écriture nulle : un compte bancaire lié au compte de
                // créance, ou un compte de créance lié à un compte bancaire.
                DbError::BankAccountLedgerIsClaimAccount {
                    account_id,
                    account_number,
                    claim,
                } => {
                    claim_configuration_response(account_id, account_number.as_deref(), claim, None)
                }
                DbError::ClaimAccountLinkedToBankAccount {
                    account_id,
                    account_number,
                    claim,
                    bank_account_id,
                    bank_name,
                } => claim_configuration_response(
                    account_id,
                    account_number.as_deref(),
                    claim,
                    Some((bank_account_id, &bank_name)),
                ),
                // Story 25-3-a-1 (#414) — l'annulation d'un règlement refusée.
                //
                // ⚠️ Seuls les rangs que le GESTE refuse lui-même arrivent ici
                // (facture créditée, exercice clos) ; les autres sont refusés
                // par la contre-passation, avec son erreur propre. Les branches
                // restantes gardent le `match` exhaustif. ⛔ **Les clés sont
                // celles de l'écran** : un seul texte par motif.
                DbError::SettlementNotCancellable { blocker } => {
                    let (key, fallback) = match blocker {
                        SettlementCancelBlocker::InvoiceCredited => (
                            "invoices-settlement-cancel-blocked-credited",
                            "Cette facture a été créditée par un avoir : ce règlement reste ouvert au compte débiteurs, il ne s'annule pas.",
                        ),
                        SettlementCancelBlocker::WriteOffExists => (
                            "invoices-settlement-cancel-blocked-written-off",
                            "Le reste de cette facture a été soldé : annulez d'abord le solde.",
                        ),
                        SettlementCancelBlocker::SupplierInvoiceNotPaid => (
                            "supplier-invoices-settlement-cancel-blocked-not-paid",
                            "Cette facture fournisseur n'est pas payée : il n'y a pas de règlement à annuler.",
                        ),
                        // #569 : ce texte prescrit « rouvrir l'exercice » sans l'ordre qu'impose
                        // la garde LIFO (en commençant par le plus récent) — hors périmètre de la
                        // Story 15-12a, qui n'aligne que les messages de `LATER_FISCAL_YEAR_CLOSED` (C122).
                        SettlementCancelBlocker::FiscalYearClosed => (
                            "settlement-cancel-blocked-fiscal-year-closed",
                            "Ce règlement appartient à un exercice clôturé : un administrateur doit rouvrir l'exercice pour pouvoir l'annuler.",
                        ),
                        SettlementCancelBlocker::MatchedBankTransaction => (
                            "settlement-cancel-blocked-bank-match",
                            "Ce règlement est rapproché d'une transaction bancaire : annulez d'abord le rapprochement, depuis cette fiche ou le détail de l'import bancaire.",
                        ),
                        SettlementCancelBlocker::AccountArchived => (
                            "settlement-cancel-blocked-account-archived",
                            "Un compte de ce règlement a été archivé : réactivez-le pour pouvoir annuler le règlement.",
                        ),
                        SettlementCancelBlocker::NoOpenFiscalYearToday => (
                            "settlement-cancel-blocked-no-fiscal-year",
                            "Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler ce règlement.",
                        ),
                        // Tête du dé-rapprochement : jamais produite par
                        // l'annulation d'un règlement ; son texte est celui
                        // du dé-rapprochement.
                        SettlementCancelBlocker::BankTransactionNotReconciled => {
                            reconciliation_cancel_blocked_text(blocker)
                        }
                        // Motifs propres à l'annulation d'une FACTURE
                        // fournisseur (25-3-c) : jamais produits ici.
                        SettlementCancelBlocker::SupplierInvoiceCancelled
                        | SettlementCancelBlocker::SupplierInvoiceInPaymentBatch => {
                            supplier_invoice_cancel_blocked_text(blocker)
                        }
                    };
                    build_response(StatusCode::CONFLICT, blocker.code(), &t(key, fallback))
                }
                // Story 25-3-b (#418) — le dé-rapprochement refusé par le geste
                // lui-même (rang 0 : transaction non rapprochée ; rang 2 :
                // exercice clos). ⛔ Ses PROPRES textes : ceux du règlement
                // disent « ce règlement », faux pour un éclatement ou un
                // rapprochement manuel.
                DbError::ReconciliationNotCancellable { blocker } => {
                    let (key, fallback) = reconciliation_cancel_blocked_text(blocker);
                    build_response(StatusCode::CONFLICT, blocker.code(), &t(key, fallback))
                }
                // Story 25-3-c (#454) — l'annulation d'une facture fournisseur
                // refusée par le geste lui-même (déjà annulée, exercice de
                // l'achat clos, lot en cours). ⛔ Ses PROPRES textes, qui
                // disent « cette facture » : ceux du règlement disent « ce
                // règlement ».
                DbError::SupplierInvoiceNotCancellable { blocker } => {
                    let (key, fallback) = supplier_invoice_cancel_blocked_text(blocker);
                    build_response(StatusCode::CONFLICT, blocker.code(), &t(key, fallback))
                }
                // Story 25-2-b-1 (#440) — un brouillon numéroté qu'on redate
                // hors de son exercice. Conflit d'état → 409.
                DbError::InvoiceNumberFiscalYearMismatch => build_response(
                    StatusCode::CONFLICT,
                    "INVOICE_NUMBER_FISCAL_YEAR_MISMATCH",
                    &t(
                        "error-invoice-number-fiscal-year-mismatch",
                        "Cette facture porte déjà un numéro : sa date ne peut pas sortir de l'exercice qui l'a émis.",
                    ),
                ),
                // Story 25-2-b-2 (#440) — `delete` sur une facture validée.
                // ⚠️ Le message ORIENTE : sans lui, l'utilisateur lit un refus
                // sec sur le seul chemin qui lui reste.
                DbError::InvoiceMustBeUnvalidatedFirst => build_response(
                    StatusCode::CONFLICT,
                    "INVOICE_MUST_BE_UNVALIDATED_FIRST",
                    &t(
                        "error-invoice-must-be-unvalidated-first",
                        "Cette facture est validée : dévalidez-la d'abord, puis supprimez le brouillon.",
                    ),
                ),
                // ⚠️ **400** qui NOMME les comptes : raisons au doc-comment de
                // `archived_accounts_response`.
                DbError::ReversalAccountsArchived(archived) => archived_accounts_response(
                    &archived,
                    "journal-entries-reverse-account-archived",
                    "Impossible de contre-passer",
                ),
                // Story 15-6a (#473, #523 ; C-15-6-25) — la jumelle de l'avoir :
                // même code, même forme ; seul le message parle du geste.
                DbError::CreditNoteAccountsArchived(archived) => archived_accounts_response(
                    &archived,
                    "credit-note-account-archived",
                    "Impossible d'émettre l'avoir",
                ),
                // ⚠️ Refuser est VOULU : supprimer une écriture qu'on a corrigée
                // effacerait la correction. Rendu explicite plutôt que laissé
                // remonter comme une violation de clé étrangère au message opaque.
                DbError::EntryIsReversed => build_response(
                    StatusCode::CONFLICT,
                    "ENTRY_IS_REVERSED",
                    &t(
                        "journal-entries-delete-blocked-reversed",
                        "Cette écriture a été contre-passée : elle ne peut plus être modifiée ni supprimée.",
                    ),
                ),
                // ⛔ Story 24-4c (#380) — le verrou de période. **400 et non 409** :
                // ce qui est invalide, c'est la DATE PROPOSÉE, pas l'état de la
                // ressource visée (asymétrie figée par la 24-4a et suivie par la
                // 24-4b).
                //
                // ⚠️ Le message NOMME LES DEUX BORNES — jusqu'où les livres sont
                // fermés, et de quand est l'écriture refusée. Un refus qui ne dit
                // pas où s'arrête le verrou envoie l'utilisateur deviner.
                // Story 25-4-e (#495) — la facture sous le montant minimum : le
                // message nomme les deux montants, au centime.
                DbError::InvoiceBelowMinimum { total, minimum } => {
                    let money = |d: rust_decimal::Decimal| {
                        d.round_dp_with_strategy(
                            2,
                            rust_decimal::RoundingStrategy::MidpointAwayFromZero,
                        )
                        .to_string()
                    };
                    let fallback = format!(
                        "Le total de cette facture, CHF {}, est inférieur au montant minimum fixé dans Paramètres → Facturation, CHF {}.",
                        money(total),
                        money(minimum)
                    );
                    let mut args = FluentArgs::new();
                    args.set("total", money(total));
                    args.set("minimum", money(minimum));
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "INVOICE_BELOW_MINIMUM",
                        &t_args("error-invoice-below-minimum", &fallback, &args),
                    )
                }
                DbError::PeriodLocked {
                    locked_through,
                    attempted,
                } => {
                    let fallback = format!(
                        "Les écritures sont verrouillées jusqu'au {locked_through} ; \
                         celle-ci est datée du {attempted}."
                    );
                    let mut args = FluentArgs::new();
                    args.set("lockedThrough", locked_through.to_string());
                    args.set("attempted", attempted.to_string());
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "PERIOD_LOCKED",
                        &t_args("journal-entries-period-locked", &fallback, &args),
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
                        "L'exercice comptable est clôturé — aucune écriture ne peut y être ajoutée, modifiée ou supprimée (CO art. 957-964).",
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
                // Story 15-5a (#429, choix C3/C16/C18/C29) — un compte de la
                // société, actif, mais non imputable. Le message NOMME les
                // comptes et la cause ; le détail est celui du jumeau
                // `ACCOUNT_ARCHIVED`, construit par l'accesseur unique
                // `NonPostableAccounts::details()`.
                //
                // ⚠️ `count` est passé comme NOMBRE Fluent : en chaîne, le
                // sélecteur `[one]` ne s'appliquerait jamais et le singulier
                // retomberait sur `*[other]`. La clé est inscrite à
                // `SELECTEURS_RESOLUS_COTE_SERVEUR` (kesh-i18n) : elle n'est
                // résolue qu'ici, avec ses arguments.
                DbError::AccountsNotPostable(accounts) => {
                    let numbers = accounts.numbers().join(", ");
                    let count = accounts.len();
                    let fallback = if count == 1 {
                        format!(
                            "Le compte {numbers} n’est pas imputable (compte de regroupement, de résultat ou de clôture) : choisissez un compte imputable."
                        )
                    } else {
                        format!(
                            "Les comptes {numbers} ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : choisissez des comptes imputables."
                        )
                    };
                    account_not_postable_response(
                        "error-account-not-postable",
                        &fallback,
                        &accounts,
                    )
                }
                // Story 15-5d (#429, choix C28, C36) — un compte DÉSIGNÉ dans les
                // réglages de facturation (créance, TVA due, créanciers, TVA
                // récupérable) est devenu non imputable et un flux veut y écrire.
                // Même code et même détail que le bras précédent (un seul contrat
                // pour l'intégrateur) ; le message dit OÙ agir et QUI peut le
                // faire — la validation et la saisie fournisseur sont ouvertes au
                // Comptable, la page des réglages à l'Admin seul.
                //
                // ⚠️ `count` en NOMBRE Fluent, comme ci-dessus ; clé inscrite à
                // `SELECTEURS_RESOLUS_COTE_SERVEUR` (kesh-i18n).
                DbError::DesignatedAccountsNotPostable(accounts) => {
                    let numbers = accounts.numbers().join(", ");
                    let count = accounts.len();
                    let fallback = if count == 1 {
                        format!(
                            "Le compte {numbers}, désigné dans Paramètres → Facturation, n’est pas imputable (compte de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à sa place un compte imputable."
                        )
                    } else {
                        format!(
                            "Les comptes {numbers}, désignés dans Paramètres → Facturation, ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à leur place des comptes imputables."
                        )
                    };
                    account_not_postable_response(
                        "error-designated-account-not-postable",
                        &fallback,
                        &accounts,
                    )
                }
                // Story 25-4-c3-b (#476) — un écart d'arrondi à écrire, et pas de
                // compte utilisable pour le recevoir. Le message dit OÙ agir.
                // Story 25-4-c4-a : le message suit le CONTEXTE — une pièce émise
                // arrondie à 5 centimes n'est pas un paiement.
                DbError::RoundingAccountNotConfigured { context } => {
                    let (key, fallback) = match context {
                        kesh_db::errors::RoundingContext::Payment => (
                            "error-rounding-account-not-configured",
                            "Ce paiement solde la facture au centime, mais aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Paramètres → Facturation.",
                        ),
                        kesh_db::errors::RoundingContext::Issuance => (
                            "error-rounding-account-not-configured-issuance",
                            "Le total de cette pièce est arrondi à 5 centimes, mais aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Paramètres → Facturation.",
                        ),
                    };
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "ROUNDING_ACCOUNT_NOT_CONFIGURED",
                        &t(key, fallback),
                    )
                }
                // Story 25-4-d2a (#384) — le compte de la nature d'un solde : un
                // message par nature, qui nomme le réglage à remplir.
                DbError::WriteOffAccountNotConfigured { nature } => {
                    let (key, fallback) = match nature {
                        "discount" => (
                            "error-write-off-account-not-configured-discount",
                            "Aucun compte d'escompte utilisable n'est désigné : choisissez-en un dans Paramètres → Facturation, section Solde du reste.",
                        ),
                        "bank_fees" => (
                            "error-write-off-account-not-configured-bank-fees",
                            "Aucun compte de frais bancaires utilisable n'est désigné : choisissez-en un dans Paramètres → Facturation, section Solde du reste.",
                        ),
                        "bad_debt" => (
                            "error-write-off-account-not-configured-bad-debt",
                            "Aucun compte de pertes sur créances utilisable n'est désigné : choisissez-en un dans Paramètres → Facturation, section Solde du reste.",
                        ),
                        _ => (
                            "error-write-off-account-not-configured-rounding",
                            "Aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Paramètres → Facturation.",
                        ),
                    };
                    build_response(
                        StatusCode::BAD_REQUEST,
                        "WRITE_OFF_ACCOUNT_NOT_CONFIGURED",
                        &t(key, fallback),
                    )
                }
                // Story 16-1a (#152) — comptes de produit de ligne de facture.
                // Le générique `INACTIVE_OR_INVALID_ACCOUNTS` ci-dessus ne nomme
                // aucune ligne ; sur une facture pouvant en porter 200, ce
                // n'est pas actionnable. On compose ici un message qui les
                // nomme toutes, à partir du détail structuré remonté par le
                // repository.
                DbError::InvalidRevenueAccounts(rejected) => {
                    let detail = format_rejected_revenue_accounts(&rejected);
                    let fallback = format!("Compte de produit invalide — {detail}");
                    let mut args = FluentArgs::new();
                    args.set("detail", detail.clone());
                    let msg = t_args("invoice-line-revenue-account-invalid", &fallback, &args);
                    let body = serde_json::json!({
                        "error": {
                            "code": "INVOICE_LINE_REVENUE_ACCOUNT_INVALID",
                            "message": msg,
                            "details": {
                                "rejected": rejected.iter().map(|r| serde_json::json!({
                                    "lineNumber": r.line_number,
                                    "accountId": r.account_id,
                                    "accountNumber": r.account_number,
                                    "reason": revenue_account_rejection_code(r.reason),
                                })).collect::<Vec<_>>(),
                            },
                        }
                    });
                    (StatusCode::BAD_REQUEST, Json(body)).into_response()
                }
                // Story 16-1a (D5-bis) — l'avoir ne peut pas contre-passer sur
                // un compte archivé. Message nommant ligne et compte : la
                // correction est de réactiver le compte, l'utilisateur doit
                // savoir lequel.
                DbError::CreditNoteRevenueAccountsArchived(rejected) => {
                    let detail = format_rejected_revenue_accounts(&rejected);
                    let fallback = format!(
                        "Impossible d'émettre l'avoir — {detail}. Réactivez le ou les comptes concernés."
                    );
                    let mut args = FluentArgs::new();
                    args.set("detail", detail.clone());
                    let msg = t_args("credit-note-revenue-account-archived", &fallback, &args);
                    let body = serde_json::json!({
                        "error": {
                            "code": "CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED",
                            "message": msg,
                            "details": {
                                // `reason` est exposé ici comme sur le chemin
                                // facture : la structure transportée est la même
                                // (`RejectedRevenueAccount`), et un client qui
                                // écrit un gestionnaire générique sur
                                // `details.rejected[]` doit trouver le même jeu
                                // de clés des deux côtés (revue 16-1a passe 2).
                                "rejected": rejected.iter().map(|r| serde_json::json!({
                                    "lineNumber": r.line_number,
                                    "accountId": r.account_id,
                                    "accountNumber": r.account_number,
                                    "reason": revenue_account_rejection_code(r.reason),
                                })).collect::<Vec<_>>(),
                            },
                        }
                    });
                    (StatusCode::BAD_REQUEST, Json(body)).into_response()
                }
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
                    // la clé i18n spec.
                    //
                    // ⚠️ **Story 24-3 (#372) : `paidAtBeforeInvoiceDate` →
                    // `settledOnBeforeInvoiceDate`.** L'ancien code n'était émis
                    // que par `mark_as_paid`, supprimé ; le laisser aurait laissé
                    // une branche morte et une clé FTL que personne ne demande —
                    // l'angle mort exact décrit au `CLAUDE.md`, qu'aucune garde
                    // ne voit dans ce sens-là.
                    let (key, default): (String, &str) = match code.as_str() {
                        "settledOnBeforeInvoiceDate" => (
                            "invoice-error-settled-on-before-invoice-date".to_string(),
                            "La date de règlement ne peut être antérieure à la date de facture.",
                        ),
                        // Story 25-4-d2a (#384, #490) — le solde du reste.
                        "nothingToWriteOff" => (
                            "invoice-error-nothing-to-write-off".to_string(),
                            "Il ne reste rien à solder sur cette facture.",
                        ),
                        "writeOffRoundingTooLarge" => (
                            "invoice-error-write-off-rounding-too-large".to_string(),
                            "Un reste d'arrondi ne dépasse pas 5 centimes : choisissez une autre nature pour solder ce reste.",
                        ),
                        "invoiceAlreadyPaid" => (
                            "invoice-error-already-paid".to_string(),
                            "Cette facture est déjà payée : il n'y a rien à solder.",
                        ),
                        // N2 (review pass 3 B) : code "paidAtFuture" supprimé —
                        // `paid_at` peut être dans le futur (date d'exécution bancaire).
                        "alreadyUnpaid" => (
                            "invoice-error-already-unpaid".to_string(),
                            "Cette facture n'est pas marquée payée.",
                        ),
                        // Story 16-1a (D4-bis) — pièce entièrement à zéro.
                        // Avant cette story, le cas produisait un 500 SQL sur
                        // `chk_jel_debit_credit_exclusive`.
                        "invoiceTotalZero" => (
                            "invoice-error-total-zero".to_string(),
                            "Cette facture est d'un montant total nul : elle ne peut pas être validée. Renseignez au moins une ligne avec un prix unitaire supérieur à zéro.",
                        ),
                        "creditNoteTotalZero" => (
                            "credit-note-error-total-zero".to_string(),
                            "Cette facture est d'un montant total nul : aucun avoir ne peut être émis.",
                        ),
                        // Story 24-4c (#380) — le verrou de période. ⛔ Sans ces
                        // deux entrées, les refus de `lock_books` / `unlock_books`
                        // retomberaient sur « Entrée invalide » : le geste que la
                        // garde existe pour rattraper recevrait un message qui ne
                        // dit ni la date, ni la raison, ni quoi faire.
                        "booksLockBoundNotPast" => (
                            "settings-books-lock-bound-not-past".to_string(),
                            "La borne du verrou doit être antérieure à aujourd'hui : verrouiller le jour même refuserait toute correction faite aujourd'hui.",
                        ),
                        "booksUnlockMotifRequired" => (
                            "settings-books-lock-motif-required".to_string(),
                            "Un motif est obligatoire pour déverrouiller.",
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

/// Story 25-3-b (#418) — le texte d'un motif de dé-rapprochement refusé, un
/// par variante de [`SettlementCancelBlocker`] : clé FTL de la famille
/// `reconciliation-cancel-blocked-*` et repli en dur, **mot pour mot** le FTL
/// fr-CH. ⛔ Les clés sont celles de l'écran (`features/reconciliation/`) : un
/// seul texte par motif.
fn reconciliation_cancel_blocked_text(
    blocker: SettlementCancelBlocker,
) -> (&'static str, &'static str) {
    match blocker {
        SettlementCancelBlocker::BankTransactionNotReconciled => (
            "reconciliation-cancel-blocked-not-reconciled",
            "Cette transaction bancaire n'est pas rapprochée : il n'y a pas de rapprochement à annuler.",
        ),
        SettlementCancelBlocker::InvoiceCredited => (
            "reconciliation-cancel-blocked-credited",
            "La facture de ce rapprochement a été créditée par un avoir : son règlement reste ouvert au compte débiteurs, il ne s'annule pas.",
        ),
        SettlementCancelBlocker::WriteOffExists => (
            "reconciliation-cancel-blocked-written-off",
            "Le reste de la facture de ce rapprochement a été soldé : annulez d'abord le solde.",
        ),
        // Aucun rapprochement ne règle une facture fournisseur : le motif ne
        // peut pas naître ici ; le `match` reste exhaustif.
        SettlementCancelBlocker::SupplierInvoiceNotPaid => (
            "supplier-invoices-settlement-cancel-blocked-not-paid",
            "Cette facture fournisseur n'est pas payée : il n'y a pas de règlement à annuler.",
        ),
        // #569 : ce texte prescrit « rouvrir l'exercice » sans l'ordre qu'impose
        // la garde LIFO (en commençant par le plus récent) — hors périmètre de la
        // Story 15-12a, qui n'aligne que les messages de `LATER_FISCAL_YEAR_CLOSED` (C122).
        SettlementCancelBlocker::FiscalYearClosed => (
            "reconciliation-cancel-blocked-fiscal-year-closed",
            "Ce rapprochement appartient à un exercice clôturé : un administrateur doit rouvrir l'exercice pour pouvoir l'annuler.",
        ),
        SettlementCancelBlocker::MatchedBankTransaction => (
            "reconciliation-cancel-blocked-bank-match",
            "L'écriture de ce rapprochement est aussi rapprochée d'une autre transaction bancaire : annulez d'abord cet autre rapprochement.",
        ),
        SettlementCancelBlocker::AccountArchived => (
            "reconciliation-cancel-blocked-account-archived",
            "Un compte de l'écriture de ce rapprochement a été archivé : réactivez-le pour pouvoir annuler le rapprochement.",
        ),
        SettlementCancelBlocker::NoOpenFiscalYearToday => (
            "reconciliation-cancel-blocked-no-fiscal-year",
            "Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler ce rapprochement.",
        ),
        // Motifs propres à l'annulation d'une facture fournisseur : jamais
        // produits par un dé-rapprochement ; le `match` reste exhaustif.
        SettlementCancelBlocker::SupplierInvoiceCancelled
        | SettlementCancelBlocker::SupplierInvoiceInPaymentBatch => {
            supplier_invoice_cancel_blocked_text(blocker)
        }
    }
}

/// Clé FTL et repli fr-CH d'un motif qui refuse l'**annulation d'une facture
/// fournisseur** (Story 25-3-c, #454). Mêmes clés que l'écran
/// (`features/supplier-invoices/invoice-cancel.ts`) : un seul texte par motif,
/// et les replis disent **mot pour mot** le FTL fr-CH.
fn supplier_invoice_cancel_blocked_text(
    blocker: SettlementCancelBlocker,
) -> (&'static str, &'static str) {
    match blocker {
        SettlementCancelBlocker::SupplierInvoiceCancelled => (
            "supplier-invoices-cancel-blocked-cancelled",
            "Cette facture fournisseur est déjà annulée.",
        ),
        // #569 : ce texte prescrit « rouvrir l'exercice » sans l'ordre qu'impose
        // la garde LIFO (en commençant par le plus récent) — hors périmètre de la
        // Story 15-12a, qui n'aligne que les messages de `LATER_FISCAL_YEAR_CLOSED` (C122).
        SettlementCancelBlocker::FiscalYearClosed => (
            "supplier-invoices-cancel-blocked-fiscal-year-closed",
            "Cette facture appartient à un exercice clôturé : un administrateur doit rouvrir l'exercice pour pouvoir l'annuler.",
        ),
        SettlementCancelBlocker::MatchedBankTransaction => (
            "supplier-invoices-cancel-blocked-bank-match",
            "L'écriture d'achat de cette facture est rapprochée d'une transaction bancaire : annulez d'abord ce rapprochement.",
        ),
        SettlementCancelBlocker::AccountArchived => (
            "supplier-invoices-cancel-blocked-account-archived",
            "Un compte de l'écriture d'achat de cette facture a été archivé : réactivez-le pour pouvoir annuler la facture.",
        ),
        SettlementCancelBlocker::NoOpenFiscalYearToday => (
            "supplier-invoices-cancel-blocked-no-fiscal-year",
            "Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler cette facture.",
        ),
        SettlementCancelBlocker::SupplierInvoiceInPaymentBatch => (
            "supplier-invoices-cancel-blocked-in-payment-batch",
            "Cette facture figure dans un lot de paiement en cours : annulez d'abord le lot.",
        ),
        // Têtes d'autres gestes : jamais produites par l'annulation d'une
        // facture fournisseur ; leurs textes sont les leurs.
        SettlementCancelBlocker::BankTransactionNotReconciled
        | SettlementCancelBlocker::InvoiceCredited
        | SettlementCancelBlocker::WriteOffExists
        | SettlementCancelBlocker::SupplierInvoiceNotPaid => {
            reconciliation_cancel_blocked_text(blocker)
        }
    }
}

/// Clé i18n et repli français du refus « contrepartie = compte soldé »
/// (Story 15-6b, AC2) — **un rôle, une clé**, toutes **plates** : un sélecteur
/// Fluent neuf rougirait `no_new_select_expression_reaches_the_frontend_dictionary`
/// (kesh-i18n), et le dictionnaire du frontend le résoudrait sans argument.
///
/// ⚠️ `t_args` rend le repli **tel quel**, sans les arguments : il est donc
/// construit par `format!`, numéro et facture déjà en place.
fn settlement_counterparty_message(
    account: &str,
    claim: kesh_db::errors::ClaimSide,
    role: kesh_db::errors::SettlementAccountRole,
    batch: Option<&kesh_db::errors::SettlementBatchContext>,
) -> String {
    use kesh_db::errors::{ClaimSide, SettlementAccountRole};
    let mut args = FluentArgs::new();
    args.set("account", account.to_string());
    let (key, fallback) = match (role, claim) {
        (SettlementAccountRole::Counterparty, ClaimSide::Payable) => match batch {
            Some(ctx) => {
                let invoice = ctx
                    .supplier_invoice_number
                    .clone()
                    .unwrap_or_else(|| format!("#{}", ctx.supplier_invoice_id));
                args.set("invoice", invoice.clone());
                (
                    "error-settlement-counterparty-is-payable-in-batch",
                    format!(
                        "Le compte bancaire de ce lot est lié au compte {account}, le compte créanciers de la facture {invoice} : le lot ne peut pas être confirmé. Reliez le compte bancaire à son propre compte de banque puis confirmez de nouveau, ou annulez le lot et réglez la facture depuis sa fiche."
                    ),
                )
            }
            None => (
                "error-settlement-counterparty-is-payable",
                format!(
                    "Le compte {account} est le compte créanciers de cette facture : un règlement doit éteindre la dette par un autre compte (banque, caisse, compensation…)."
                ),
            ),
        },
        (SettlementAccountRole::Counterparty, ClaimSide::Receivable) => (
            "error-settlement-counterparty-is-receivable",
            format!(
                "Le compte {account} est le compte débiteurs de cette facture : un règlement doit faire sortir la créance vers un autre compte (banque, caisse, compensation…)."
            ),
        ),
        (SettlementAccountRole::Rounding, _) => (
            "error-settlement-rounding-account-is-receivable",
            format!(
                "Le compte {account}, désigné dans les réglages comme compte de différences d'arrondi, est le compte débiteurs de cette facture : un administrateur doit désigner un autre compte dans Paramètres → Facturation."
            ),
        ),
        (SettlementAccountRole::WriteOffNature, _) => (
            "error-settlement-write-off-account-is-receivable",
            format!(
                "Le compte {account}, désigné dans les réglages comme compte de cette nature de solde, est le compte débiteurs de cette facture : un administrateur doit désigner un autre compte dans Paramètres → Facturation."
            ),
        ),
        (SettlementAccountRole::VatPayable, _) => (
            "error-settlement-vat-payable-account-is-receivable",
            format!(
                "Le compte {account}, désigné dans les réglages comme compte de TVA due, est le compte débiteurs de cette facture : un administrateur doit désigner un autre compte dans Paramètres → Facturation."
            ),
        ),
    };
    t_args(key, &fallback, &args)
}

/// Réponse **400** des deux refus de configuration de la Story 15-6c (#474 ;
/// AC1) — une seule construction, pour que leurs `details` ne divergent pas :
///
/// - `bank = None` → `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (lien d'un compte
///   bancaire au compte de créance désigné) ;
/// - `bank = Some((id, nom))` → `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT`
///   (désignation d'un compte lié à ce compte bancaire), `details` enrichis de
///   `bankAccountId` et `bankName`.
///
/// Une clé de message par refus **et** par `claim` (quatre), chacune nommant le
/// bon compte ; `$account` = le numéro, repli `#<id>` (patron de la 15-6b).
fn claim_configuration_response(
    account_id: i64,
    account_number: Option<&str>,
    claim: kesh_db::errors::ClaimSide,
    bank: Option<(i64, &str)>,
) -> Response {
    use kesh_db::errors::ClaimSide;
    let account = account_number
        .map(str::to_string)
        .unwrap_or_else(|| format!("#{account_id}"));
    let mut args = FluentArgs::new();
    args.set("account", account.clone());
    let (code, key, fallback) = match (bank, claim) {
        (None, ClaimSide::Receivable) => (
            "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT",
            "error-bank-account-ledger-is-receivable",
            format!(
                "Le compte {account} est le compte débiteurs désigné dans Paramètres → Facturation : un compte bancaire doit être lié à son propre compte de banque."
            ),
        ),
        (None, ClaimSide::Payable) => (
            "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT",
            "error-bank-account-ledger-is-payable",
            format!(
                "Le compte {account} est le compte créanciers désigné dans Paramètres → Facturation : un compte bancaire doit être lié à son propre compte de banque."
            ),
        ),
        (Some((_, bank_name)), ClaimSide::Receivable) => {
            args.set("bank", bank_name.to_string());
            (
                "CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT",
                "error-claim-account-linked-to-bank-account-receivable",
                format!(
                    "Le compte {account} est lié au compte bancaire {bank_name} : il ne peut pas servir de compte débiteurs."
                ),
            )
        }
        (Some((_, bank_name)), ClaimSide::Payable) => {
            args.set("bank", bank_name.to_string());
            (
                "CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT",
                "error-claim-account-linked-to-bank-account-payable",
                format!(
                    "Le compte {account} est lié au compte bancaire {bank_name} : il ne peut pas servir de compte créanciers."
                ),
            )
        }
    };
    let message = t_args(key, &fallback, &args);
    let mut details = serde_json::json!({
        "accountId": account_id,
        "accountNumber": account_number,
        "claim": claim.as_str(),
    });
    if let Some((bank_account_id, bank_name)) = bank {
        details["bankAccountId"] = serde_json::json!(bank_account_id);
        details["bankName"] = serde_json::json!(bank_name);
    }
    let body = serde_json::json!({
        "error": {
            "code": code,
            "message": message,
            "details": details,
        }
    });
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

/// Réponse **400** `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (Story 15-6b,
/// AC2). `details` vient de `claim_account_refusal_details` — la forme que
/// partagent le rapprochement et la création d'un lot ; les clés **de lot**
/// (`paymentBatchId`, `supplierInvoiceId`) ne sont ajoutées qu'ici, seul
/// consommateur qui en ait.
fn settlement_counterparty_response(
    account_id: i64,
    account_number: Option<&str>,
    claim: kesh_db::errors::ClaimSide,
    role: kesh_db::errors::SettlementAccountRole,
    batch: Option<&kesh_db::errors::SettlementBatchContext>,
) -> Response {
    let account = account_number
        .map(str::to_string)
        .unwrap_or_else(|| format!("#{account_id}"));
    let message = settlement_counterparty_message(&account, claim, role, batch);
    let mut details = kesh_db::repositories::invoice_settlements::claim_account_refusal_details(
        account_id,
        account_number,
        claim,
        role,
        None,
    );
    if let Some(ctx) = batch {
        details["paymentBatchId"] = serde_json::json!(ctx.payment_batch_id);
        details["supplierInvoiceId"] = serde_json::json!(ctx.supplier_invoice_id);
    }
    let body = serde_json::json!({
        "error": {
            "code": "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT",
            "message": message,
            "details": details,
        }
    });
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

/// Le 400 `ACCOUNT_ARCHIVED` d'une contre-passation dont un compte a été
/// archivé — [`DbError::ReversalAccountsArchived`] et sa jumelle de l'avoir
/// [`DbError::CreditNoteAccountsArchived`] (Story 15-6a) : même code, même
/// `details.rejected[]` (`{ accountId, accountNumber }`), seul le message
/// (`key`, et son repli français commençant par `fallback_lead`) diffère.
///
/// ⚠️ **400**, comme le gabarit `CreditNoteRevenueAccountsArchived` : un compte
/// archivé est une donnée d'entrée invalide, là où un refus de propriété est un
/// conflit d'état. Le refus NOMME les comptes — un « interdit » sec ne serait
/// pas utilisable, la réactivation étant le chemin de sortie
/// (`PUT /api/v1/accounts/{id}/reactivate`).
fn archived_accounts_response(
    archived: &[kesh_db::errors::ArchivedAccount],
    key: &str,
    fallback_lead: &str,
) -> Response {
    let detail = archived
        .iter()
        .map(|a| {
            a.account_number
                .clone()
                .unwrap_or_else(|| a.account_id.to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let fallback = format!(
        "{fallback_lead} — compte(s) archivé(s) : {detail}. Réactivez le ou les comptes concernés."
    );
    let mut args = FluentArgs::new();
    args.set("detail", detail.clone());
    let msg = t_args(key, &fallback, &args);
    let body = serde_json::json!({
        "error": {
            "code": "ACCOUNT_ARCHIVED",
            "message": msg,
            "details": {
                "rejected": archived.iter().map(|a| serde_json::json!({
                    "accountId": a.account_id,
                    "accountNumber": a.account_number,
                })).collect::<Vec<_>>(),
            },
        }
    });
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
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

    /// Story 15-13b (#576, test 15) — l'échec antérieur à la sauvegarde rend
    /// un 500 `ADMIN_PRE_IMPORT_BACKUP_FAILED` dont le repli (aucun catalogue
    /// n'est chargé dans ces tests) ne promet aucune sauvegarde, dit qu'aucune
    /// n'a été créée et nomme les deux pistes. Témoin : `AdminFullImportFailed`
    /// garde son code et un repli différent.
    #[tokio::test]
    async fn pre_import_backup_failed_maps_to_500_without_promise() {
        let resp =
            AppError::AdminPreImportBackupFailed("détail-interne-xyz".into()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "ADMIN_PRE_IMPORT_BACKUP_FAILED");
        let message = body["error"]["message"].as_str().unwrap().to_string();
        assert!(
            !message.contains("détail-interne-xyz"),
            "détail exposé : {message}"
        );
        assert!(!message.contains("a été préservé"), "promesse : {message}");
        for jeton in [
            "aucune sauvegarde n'a été créée",
            "dossier de sauvegarde",
            "base de données",
        ] {
            assert!(message.contains(jeton), "« {jeton} » absent : {message}");
        }
        let resp = AppError::AdminFullImportFailed("d".into()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "ADMIN_FULL_IMPORT_FAILED");
        assert_ne!(body["error"]["message"].as_str().unwrap(), message);
    }

    /// Story 15-13b (#576, test 16) — les quatre catalogues distinguent l'échec
    /// antérieur à la sauvegarde. Chargés par `I18nBundle::load` sur un chemin
    /// indépendant du répertoire courant, sans toucher au catalogue global.
    #[test]
    fn catalogues_distinguent_l_echec_de_sauvegarde() {
        let dir =
            std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../kesh-i18n/locales"));
        let bundle = I18nBundle::load(dir).expect("catalogues");
        // (locale, promesse interdite, négation exigée, pistes exigées)
        let attendus = [
            (
                Locale::FrCh,
                "un backup automatique a été créé avant l'opération",
                "aucune sauvegarde n'a été créée",
                ["dossier de sauvegarde", "base de données"],
            ),
            (
                Locale::DeCh,
                "vor dem Vorgang wurde automatisch ein Backup erstellt",
                "Es wurde keine Sicherung erstellt",
                ["Sicherungsordner", "Datenbank"],
            ),
            (
                Locale::EnCh,
                "an automatic backup was created before the operation",
                "No backup was created",
                ["backup folder", "database"],
            ),
            (
                Locale::ItCh,
                "prima dell'operazione è stato creato un backup automatico",
                "Non è stato creato alcun backup",
                ["cartella di backup", "database"],
            ),
        ];
        let neuf_fr = bundle.format(&Locale::FrCh, "error-admin-pre-import-backup-failed", None);
        for (locale, interdit, negation, pistes) in attendus {
            let neuf = bundle.format(&locale, "error-admin-pre-import-backup-failed", None);
            let ancien = bundle.format(&locale, "error-admin-full-import-failed", None);
            // Assertion de montage : l'interdit est bien la promesse de l'ancien
            // texte de SA locale — sinon il serait une coquille et le test
            // passerait à vide.
            assert!(
                ancien.contains(interdit),
                "{locale:?} : interdit absent de l'ancien texte"
            );
            assert_ne!(
                neuf, ancien,
                "{locale:?} : texte neuf = texte d'après sauvegarde"
            );
            if locale != Locale::FrCh {
                assert_ne!(neuf, neuf_fr, "{locale:?} : retombé sur le français");
            }
            assert!(
                !neuf.contains(interdit),
                "{locale:?} : promesse reprise : {neuf}"
            );
            assert!(
                neuf.contains(negation),
                "{locale:?} : négation absente : {neuf}"
            );
            for piste in pistes {
                assert!(
                    neuf.contains(piste),
                    "{locale:?} : piste « {piste} » absente : {neuf}"
                );
            }
        }
    }

    /// Story 15-1a-i (#518) — chaque refus du lettrage a son statut, son code
    /// et son message, et le repli français est **identique au catalogue**
    /// `fr-CH` (variables substituées). `LETTERING_CONCURRENT_CHANGE` n'est
    /// atteignable que par un défaut : ce test est sa seule preuve côté HTTP
    /// (R7 point 4).
    #[tokio::test]
    async fn lettering_refusals_map_to_their_status_code_and_catalog_text() {
        let catalogue = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../kesh-i18n/locales/fr-CH/messages.ftl"
        ))
        .expect("catalogue fr-CH");
        let texte = |cle: &str| -> String {
            catalogue
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{cle} = ")))
                .unwrap_or_else(|| panic!("clé {cle} absente du catalogue"))
                .to_string()
        };
        let sans = String::new;
        let cas: Vec<(DbError, StatusCode, &str, &str, String)> = vec![
            (
                DbError::LetteringTooFewLines,
                StatusCode::BAD_REQUEST,
                "LETTERING_TOO_FEW_LINES",
                "error-lettering-too-few-lines",
                sans(),
            ),
            (
                // Un plafond autre que 200 : le message doit le lire (E-4).
                DbError::LetteringTooManyLines { max: 7 },
                StatusCode::BAD_REQUEST,
                "LETTERING_TOO_MANY_LINES",
                "error-lettering-too-many-lines",
                "7".into(),
            ),
            (
                DbError::LetteringAccountsDiffer,
                StatusCode::CONFLICT,
                "LETTERING_ACCOUNTS_DIFFER",
                "error-lettering-accounts-differ",
                sans(),
            ),
            (
                DbError::LetteringAccountNotLetterable,
                StatusCode::CONFLICT,
                "LETTERING_ACCOUNT_NOT_LETTERABLE",
                "error-lettering-account-not-letterable",
                sans(),
            ),
            (
                DbError::LetteringAllLinesInClosedPeriods,
                StatusCode::CONFLICT,
                "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
                "error-lettering-all-lines-in-closed-periods",
                sans(),
            ),
            (
                DbError::LetteringLineOwnedByDocument {
                    blocker: ReversalBlocker::OwnedByInvoice,
                    document_id: Some(47),
                    document_label: None,
                },
                StatusCode::CONFLICT,
                "LETTERING_LINE_OWNED_BY_DOCUMENT",
                "error-lettering-line-owned-by-document",
                sans(),
            ),
            (
                DbError::LetteringLineAlreadyLettered { code: "AA".into() },
                StatusCode::CONFLICT,
                "LETTERING_LINE_ALREADY_LETTERED",
                "error-lettering-line-already-lettered",
                "AA".into(),
            ),
            (
                DbError::LetteringUnbalanced {
                    difference: rust_decimal::Decimal::new(100, 4),
                },
                StatusCode::CONFLICT,
                "LETTERING_UNBALANCED",
                "error-lettering-unbalanced",
                "0.01".into(),
            ),
            (
                DbError::LetteringIsDocument,
                StatusCode::CONFLICT,
                "LETTERING_IS_DOCUMENT",
                "error-lettering-is-document",
                sans(),
            ),
            (
                DbError::LetteringConcurrentChange,
                StatusCode::CONFLICT,
                "LETTERING_CONCURRENT_CHANGE",
                "error-lettering-concurrent-change",
                sans(),
            ),
        ];
        assert_eq!(cas.len(), 10, "les dix codes du lettrage (T10)");
        for (err, statut, code, cle, valeur) in cas {
            assert_eq!(err.error_code(), code, "repli de error_code()");
            let (status, body) = response_body(AppError::Database(err).into_response()).await;
            assert_eq!(status, statut, "{code}");
            assert_eq!(body["error"]["code"], code);
            let attendu = texte(cle)
                .replace("{ $code }", &valeur)
                .replace("{ $difference }", &valeur)
                .replace("{ $max }", &valeur);
            assert_eq!(body["error"]["message"], attendu, "{code}");
        }
    }

    /// Les `details` des refus qui en portent (AC3).
    #[tokio::test]
    async fn lettering_refusals_carry_their_details() {
        let (_, body) = response_body(
            AppError::Database(DbError::LetteringLineOwnedByDocument {
                blocker: ReversalBlocker::OwnedByInvoice,
                document_id: Some(47),
                document_label: Some("F-2026-014".into()),
            })
            .into_response(),
        )
        .await;
        assert_eq!(body["error"]["details"]["documentId"], 47);
        assert_eq!(body["error"]["details"]["documentNumber"], "F-2026-014");
        let (_, body) = response_body(
            AppError::Database(DbError::LetteringLineAlreadyLettered { code: "AB".into() })
                .into_response(),
        )
        .await;
        assert_eq!(body["error"]["details"]["code"], "AB");
        let (_, body) = response_body(
            AppError::Database(DbError::LetteringUnbalanced {
                difference: rust_decimal::Decimal::new(-55000, 4),
            })
            .into_response(),
        )
        .await;
        assert_eq!(body["error"]["details"]["difference"], "-5.5");
    }

    #[tokio::test]
    async fn invalid_credentials_maps_to_401() {
        let resp = AppError::InvalidCredentials.into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
        assert_eq!(body["error"]["message"], "Identifiants invalides");
    }

    /// Story 25-4-d2c — un `write_off_vat` de forme fausse rend un 500 générique,
    /// jamais un rapport faux, et le détail interne ne sort pas.
    #[tokio::test]
    async fn report_corrupt_data_maps_to_500_without_leak() {
        let err = kesh_report::errors::ReportError::CorruptData("solde 42 : clé absente".into());
        let resp = AppError::from(err).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "INTERNAL_ERROR");
        let message = body["error"]["message"].as_str().unwrap();
        assert!(!message.contains("solde 42"), "detail leaked: {message}");
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

    // -----------------------------------------------------------------------
    // Story 16-1a (#152) — comptes de produit de ligne.
    //
    // Ces tests existent parce que le mapping par défaut de `DbError` remplace
    // tout message venu du repository par un texte fixe. Sans variante dédiée,
    // le détail nommant les lignes n'atteindrait jamais le client — c'est la
    // leçon déjà tirée sur `AccountParentArchived` (14-3a).
    // -----------------------------------------------------------------------

    fn rejected(
        line_number: Option<i32>,
        number: &str,
        reason: RevenueAccountRejection,
    ) -> RejectedRevenueAccount {
        RejectedRevenueAccount {
            line_number,
            account_id: 42,
            account_number: Some(number.into()),
            reason,
        }
    }

    /// Le message nomme **toutes** les lignes en défaut, pas seulement la
    /// première, et le body porte le détail structuré.
    #[tokio::test]
    async fn invalid_revenue_accounts_names_every_offending_line() {
        let resp = AppError::from(DbError::InvalidRevenueAccounts(vec![
            rejected(Some(2), "3200", RevenueAccountRejection::Inactive),
            rejected(Some(5), "4000", RevenueAccountRejection::NotRevenue),
        ]))
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body["error"]["code"],
            "INVOICE_LINE_REVENUE_ACCOUNT_INVALID"
        );

        let msg = body["error"]["message"].as_str().unwrap();
        // « Ligne N » et pas seulement « N » : le compte de la ligne 2 est
        // `3200`, qui contient déjà un `2`. Un `msg.contains("2")` passerait
        // donc même si le sujet disparaissait entièrement du message — le test
        // ne prouverait plus rien (revue de code 16-1a passe 2).
        assert!(
            msg.contains("Ligne 2"),
            "la ligne 2 doit être nommée : {msg}"
        );
        assert!(
            msg.contains("Ligne 5"),
            "la ligne 5 doit être nommée : {msg}"
        );
        assert!(
            msg.contains("3200"),
            "le compte 3200 doit être nommé : {msg}"
        );
        assert!(
            msg.contains("4000"),
            "le compte 4000 doit être nommé : {msg}"
        );

        let rejected = body["error"]["details"]["rejected"].as_array().unwrap();
        assert_eq!(rejected.len(), 2);
        assert_eq!(rejected[0]["lineNumber"], 2);
        assert_eq!(rejected[0]["reason"], "INACTIVE");
        assert_eq!(rejected[1]["reason"], "NOT_REVENUE");
    }

    /// AC8-bis — le compte par défaut de la société est désigné **en toutes
    /// lettres**, jamais par un numéro de ligne : aucune ligne ne le porte.
    #[tokio::test]
    async fn invalid_revenue_accounts_names_company_default_explicitly() {
        let resp = AppError::from(DbError::InvalidRevenueAccounts(vec![rejected(
            None,
            "3000",
            RevenueAccountRejection::Inactive,
        )]))
        .into_response();
        let (_, body) = response_body(resp).await;
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(
            msg.contains("défaut"),
            "le message doit désigner le compte par défaut de la société : {msg}"
        );
        assert!(
            !msg.contains("Ligne"),
            "aucun numéro de ligne ne doit apparaître : {msg}"
        );
        assert!(body["error"]["details"]["rejected"][0]["lineNumber"].is_null());
    }

    /// D5-bis — l'avoir bloqué a son propre code et nomme le compte à
    /// réactiver.
    #[tokio::test]
    async fn credit_note_archived_account_has_dedicated_code() {
        let resp = AppError::from(DbError::CreditNoteRevenueAccountsArchived(vec![rejected(
            Some(1),
            "3200",
            RevenueAccountRejection::Inactive,
        )]))
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body["error"]["code"],
            "CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED"
        );
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("3200"), "le compte doit être nommé : {msg}");
    }

    /// Story 15-6a (test 18) — le bras de `CreditNoteAccountsArchived` : 400,
    /// code `ACCOUNT_ARCHIVED` (celui de la contre-passation), `details.rejected[]`
    /// par compte (`accountId`, `accountNumber`), et un message du geste de
    /// l'avoir — clé `credit-note-account-archived`, ou son repli français si
    /// l'i18n n'est pas initialisée — qui nomme le compte.
    #[tokio::test]
    async fn credit_note_accounts_archived_is_400_account_archived() {
        let resp = AppError::from(DbError::CreditNoteAccountsArchived(vec![
            kesh_db::errors::ArchivedAccount {
                account_id: 17,
                account_number: Some("1100".into()),
            },
        ]))
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "ACCOUNT_ARCHIVED");
        assert_eq!(
            body["error"]["details"]["rejected"],
            serde_json::json!([{ "accountId": 17, "accountNumber": "1100" }])
        );
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(
            msg.contains("1100") && msg.contains("avoir"),
            "le compte et le geste doivent être nommés : {msg}"
        );
    }

    fn claim_refusal(
        role: kesh_db::errors::SettlementAccountRole,
        claim: kesh_db::errors::ClaimSide,
        batch: Option<kesh_db::errors::SettlementBatchContext>,
    ) -> DbError {
        DbError::SettlementCounterpartyIsClaimAccount {
            account_id: 17,
            account_number: Some("1100".into()),
            claim,
            role,
            batch,
        }
    }

    /// Story 15-6b (#474, AC2) — 400, un code, `details` de la forme commune,
    /// et un repli français **formaté** : sans i18n initialisée, `t_args` rend
    /// le repli tel quel, qui ne doit pas laisser fuir le gabarit `{ $account }`.
    #[tokio::test]
    async fn settlement_counterparty_is_claim_account_is_400_with_details() {
        use kesh_db::errors::{ClaimSide, SettlementAccountRole};
        let resp = AppError::from(claim_refusal(
            SettlementAccountRole::Counterparty,
            ClaimSide::Receivable,
            None,
        ))
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body["error"]["code"],
            "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
        );
        assert_eq!(
            body["error"]["details"],
            serde_json::json!({
                "accountId": 17,
                "accountNumber": "1100",
                "claim": "receivable",
                "role": "counterparty",
            })
        );
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("1100") && msg.contains("débiteurs"), "{msg}");
        assert!(!msg.contains('$'), "gabarit non résolu : {msg}");
    }

    /// Story 15-6c (#474, AC1) — les deux refus de configuration : 400, leur
    /// code, des `details` en camelCase, et un repli français qui nomme le
    /// compte (repli `#<id>` sans numéro) et, au second, le compte bancaire.
    #[tokio::test]
    async fn claim_configuration_refusals_are_400_with_details() {
        use kesh_db::errors::ClaimSide;
        let resp = AppError::from(DbError::BankAccountLedgerIsClaimAccount {
            account_id: 17,
            account_number: Some("1100".into()),
            claim: ClaimSide::Receivable,
        })
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body["error"]["code"],
            "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT"
        );
        assert_eq!(
            body["error"]["details"],
            serde_json::json!({ "accountId": 17, "accountNumber": "1100", "claim": "receivable" })
        );
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("1100") && msg.contains("débiteurs"), "{msg}");

        let resp = AppError::from(DbError::ClaimAccountLinkedToBankAccount {
            account_id: 18,
            account_number: None,
            claim: ClaimSide::Payable,
            bank_account_id: 4,
            bank_name: "UBS".into(),
        })
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body["error"]["code"],
            "CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT"
        );
        assert_eq!(
            body["error"]["details"],
            serde_json::json!({
                "accountId": 18,
                "accountNumber": null,
                "claim": "payable",
                "bankAccountId": 4,
                "bankName": "UBS",
            })
        );
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(
            msg.contains("#18") && msg.contains("UBS") && msg.contains("créanciers"),
            "{msg}"
        );
        assert!(!msg.contains("{ $"), "gabarit non formaté : {msg}");
    }

    /// Story 15-6b (AC2) — la clé suit le RÔLE : un compte désigné renvoie à
    /// Paramètres → Facturation, jamais à « banque, caisse ».
    #[tokio::test]
    async fn settlement_counterparty_message_follows_the_role() {
        use kesh_db::errors::{ClaimSide, SettlementAccountRole};
        for (role, marker) in [
            (SettlementAccountRole::Rounding, "différences d'arrondi"),
            (SettlementAccountRole::WriteOffNature, "nature de solde"),
            (SettlementAccountRole::VatPayable, "TVA due"),
        ] {
            let resp =
                AppError::from(claim_refusal(role, ClaimSide::Receivable, None)).into_response();
            let (_, body) = response_body(resp).await;
            assert_eq!(body["error"]["details"]["role"], role.as_str());
            let msg = body["error"]["message"].as_str().unwrap();
            assert!(msg.contains(marker), "{role:?} : {msg}");
            assert!(msg.contains("Paramètres → Facturation"), "{role:?} : {msg}");
            assert!(!msg.contains("banque, caisse"), "{role:?} : {msg}");
        }
        let resp = AppError::from(claim_refusal(
            SettlementAccountRole::Counterparty,
            ClaimSide::Payable,
            None,
        ))
        .into_response();
        let (_, body) = response_body(resp).await;
        assert_eq!(body["error"]["details"]["claim"], "payable");
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("créanciers"), "{msg}");
    }

    /// Story 15-6b (AC2, AC6) — à la confirmation d'un lot, le message dit les
    /// deux issues et `details` gagne les clés de lot, et elles seules.
    #[tokio::test]
    async fn settlement_counterparty_in_batch_carries_batch_keys() {
        use kesh_db::errors::{ClaimSide, SettlementAccountRole, SettlementBatchContext};
        let resp = AppError::from(claim_refusal(
            SettlementAccountRole::Counterparty,
            ClaimSide::Payable,
            Some(SettlementBatchContext {
                payment_batch_id: 5,
                supplier_invoice_id: 42,
                supplier_invoice_number: Some("FF-7".into()),
            }),
        ))
        .into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["details"]["paymentBatchId"], 5);
        assert_eq!(body["error"]["details"]["supplierInvoiceId"], 42);
        assert!(body["error"]["details"].get("bankAccountId").is_none());
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(
            msg.contains("FF-7") && msg.contains("1100") && msg.contains("annulez le lot"),
            "{msg}"
        );

        // Sans numéro de facture : repli `#<id>`.
        let resp = AppError::from(claim_refusal(
            SettlementAccountRole::Counterparty,
            ClaimSide::Payable,
            Some(SettlementBatchContext {
                payment_batch_id: 5,
                supplier_invoice_id: 42,
                supplier_invoice_number: None,
            }),
        ))
        .into_response();
        let (_, body) = response_body(resp).await;
        assert!(
            body["error"]["message"].as_str().unwrap().contains("#42"),
            "{body}"
        );
    }

    /// Story 16-3b — le conflit de numéro de client porte **son propre code**,
    /// et pas celui de l'IDE.
    ///
    /// ⚠️ C'est la CHAÎNE qui est l'interface, pas le statut. Le frontend branche
    /// dessus littéralement (`+page.svelte`, `err.code === 'CLIENT_NUMBER_ALREADY_EXISTS'`)
    /// pour choisir le message traduit ; un copier-coller de la variante IDE
    /// voisine — huit lignes plus haut dans le `match` — garderait le 409 et
    /// afficherait « numéro IDE » sur un conflit de numéro de client. Un test
    /// qui n'assert que `StatusCode::CONFLICT` ne voit rien de cette confusion.
    #[tokio::test]
    async fn client_number_conflict_has_its_own_code() {
        let resp =
            AppError::ClientNumberAlreadyExists("CLI-1 est déjà pris".into()).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "CLIENT_NUMBER_ALREADY_EXISTS");
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("CLI-1"), "le numéro doit être nommé : {msg}");
    }

    /// D4-bis — la facture à montant nul répond en **400 métier**, plus en 500
    /// SQL. Le code non listé retomberait sur « Entrée invalide », donc ce test
    /// vérifie aussi que le dispatch de la whitelist connaît le code.
    #[tokio::test]
    async fn zero_total_invoice_maps_to_business_400() {
        let resp = AppError::from(DbError::InvalidInput("invoiceTotalZero".into())).into_response();
        let (status, body) = response_body(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "INVALID_INPUT");
        let msg = body["error"]["message"].as_str().unwrap();
        assert_ne!(
            msg, "Entrée invalide",
            "le code doit être dans la whitelist de dispatch, sinon message générique"
        );
    }
}
