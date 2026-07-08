//! kesh-api — Serveur HTTP Axum pour Kesh.
//!
//! Cette crate expose à la fois un binaire (`main.rs`) et une
//! bibliothèque (`lib.rs`) pour permettre aux tests d'intégration
//! d'importer `build_router` et les helpers de configuration.

// `pub(crate)` : module manipulant des secrets (dump complet incl. hash de mots
// de passe), pas d'API publique nécessaire hors crate (review 17-3a Pass 1).
pub mod address_input;
pub(crate) mod admin_backup;
pub mod audit;
pub mod auth;
pub mod config;
pub mod document_storage;
pub mod errors;
pub mod exports;
pub mod helpers;
pub mod inbox_import;
pub mod logging;
pub mod mail;
pub mod middleware;
pub mod qr_decode;
pub mod routes;
pub(crate) mod util;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use axum::Router;
use axum::routing::{delete, get, patch, post, put};
use sqlx::MySqlPool;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::Config;
use crate::middleware::rate_limit::RateLimiter;

/// État partagé injecté dans tous les handlers via `State<AppState>`.
///
/// Story v011-5 — ajout `users_exist: Arc<AtomicBool>` : cache mémoire
/// (initialisé au boot post-`ensure_admin_user`) qui dit si la table `users`
/// contient au moins une ligne. Lu lock-free par `require_auth` (`Acquire`)
/// pour décider du gate `423 Locked` (setup-required) vs `401` nominal. Set
/// à `true` par `setup::create_admin` après INSERT (`Release`). Pas de query
/// DB par requête après le boot (perf préservée). Re-synchronisé par les
/// preset seeds en mode test (TRUNCATE+INSERT, cohérent route `/_test/seed`).
#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
    pub rate_limiter: Arc<RateLimiter>,
    /// Story 17-4c (DC5) — instance `RateLimiter` **dédiée** au recovery
    /// (`forgot-password`/`reset-password`), distincte de `rate_limiter` (login).
    /// Seuils hardcodés 5 req / 15 min / blocage 30 min (config par env var =
    /// L5 v0.2+). Sémantique always-200 : chaque requête recovery consomme un
    /// slot via `check_and_record` atomique (jamais `reset`), sinon un simple
    /// check serait inerte (catch P3 Opus F1).
    pub rate_limiter_recovery: Arc<RateLimiter>,
    pub i18n: Arc<kesh_i18n::I18nBundle>,
    pub users_exist: Arc<AtomicBool>,
    /// Story 17-4b — couche d'envoi d'email (recovery). `NoopMailer` par défaut
    /// (feature-off / tests), `SmtpMailer` en prod si `KESH_FEATURE_FORGOT_PASSWORD`.
    /// Champ `pub` : les tests 17-4e le mutent (`Arc::new(MockMailer::new())`).
    pub mailer: Arc<dyn mail::Mailer>,
}

impl AppState {
    /// Story v011-5 — constructeur **test-only** qui défaute
    /// `users_exist: Arc::new(AtomicBool::new(true))` (cohérent presets
    /// E2E qui créent des users post-bootstrap). Minimise le churn sur
    /// les 28+ sites de tests d'intégration qui construisaient `AppState { ... }`
    /// par struct literal — ces sites utilisent désormais `AppState::new_for_tests`.
    ///
    /// **À ne jamais appeler depuis du code de production** : `main.rs`
    /// construit `AppState` inline avec la valeur réelle (`user_count > 0`).
    #[doc(hidden)]
    pub fn new_for_tests(
        pool: MySqlPool,
        config: Arc<Config>,
        rate_limiter: Arc<RateLimiter>,
        i18n: Arc<kesh_i18n::I18nBundle>,
    ) -> Self {
        Self {
            pool,
            config,
            rate_limiter,
            i18n,
            users_exist: Arc::new(AtomicBool::new(true)),
            // Story 17-4c (P4-1) — champ défauté DANS LE CORPS : signature
            // `new_for_tests` inchangée → les ~33 call-sites de test restent
            // intacts. Mêmes seuils recovery que la prod (`build_recovery_rate_limiter`).
            rate_limiter_recovery: Arc::new(build_recovery_rate_limiter()),
            // Story 17-4b (P4-1) — champ défauté DANS LE CORPS : signature
            // `new_for_tests` inchangée → les call-sites de test restent intacts.
            mailer: Arc::new(mail::NoopMailer),
        }
    }
}

/// Story 17-4c (DC5) — construit l'instance `RateLimiter` dédiée recovery avec
/// les seuils hardcodés (5 req / fenêtre 15 min / blocage 30 min). Partagé entre
/// `main.rs` (prod) et `AppState::new_for_tests` pour éviter toute divergence des
/// constantes. Config par env var = limitation L5 (v0.2+).
pub fn build_recovery_rate_limiter() -> RateLimiter {
    RateLimiter::with_thresholds(
        5,
        std::time::Duration::from_secs(15 * 60),
        std::time::Duration::from_secs(30 * 60),
    )
}

/// Construit le routeur principal de l'application (routes publiques
/// uniquement dans cette story).
///
/// - `/health` (public) — healthcheck DB
/// - `/api/v1/auth/login` (public) — authentification
/// - `/api/v1/auth/logout` (public) — invalidation refresh_token
///
/// Les stories futures ajouteront des routes protégées par JWT en
/// construisant un sous-routeur qui applique le middleware
/// `crate::middleware::auth::require_auth` via `route_layer`, puis
/// en le mergeant via `Router::merge()` dans `main.rs`.
///
/// **Note Axum 0.8** : `route_layer` sur un router vide panique
/// (`Adding a route_layer before any routes is a no-op`). On ne
/// construit le sous-routeur protégé qu'au moment où des routes lui
/// sont effectivement ajoutées.
///
/// Le `static_dir` contient le SPA SvelteKit buildé (`frontend/build`),
/// servi en fallback par `ServeDir`/`ServeFile`.
pub fn build_router(state: AppState, static_dir: String) -> Router {
    let fallback =
        ServeDir::new(&static_dir).fallback(ServeFile::new(format!("{}/index.html", static_dir)));

    // Story 1.8 : sous-routeurs par niveau de rôle (RBAC)
    //
    // Ordre de construction : require_role (inner) → merge → require_auth (outer)
    // Ordre d'exécution (oignon) : require_auth EN PREMIER → require_role EN SECOND → handler

    // Admin-only routes : gestion des utilisateurs
    let admin_routes = Router::new()
        .route(
            "/api/v1/users",
            get(routes::users::list_users).post(routes::users::create_user),
        )
        .route(
            "/api/v1/users/{id}",
            get(routes::users::get_user).put(routes::users::update_user),
        )
        .route(
            "/api/v1/users/{id}/disable",
            put(routes::users::disable_user),
        )
        .route(
            "/api/v1/users/{id}/reset-password",
            put(routes::users::reset_password),
        )
        // Story 5.2 : config facturation (Admin uniquement).
        .route(
            "/api/v1/company/invoice-settings",
            put(routes::company_invoice_settings::update_invoice_settings),
        )
        // Story 11-1 : CRUD des taux TVA (Admin, FR54). Le GET /vat-rates reste
        // dans les routes authentifiées (tous rôles) — seules les mutations sont
        // ici (écart intentionnel vs bank_accounts qui est Comptable+).
        .route("/api/v1/vat-rates", post(routes::vat::create_vat_rate))
        .route(
            "/api/v1/vat-rates/{id}",
            put(routes::vat::update_vat_rate).delete(routes::vat::deactivate_vat_rate),
        )
        // #219 : suppression définitive d'une facture (brouillon ou validée
        // avec garde-fous) — Admin uniquement. Même path que le PUT
        // (comptable_routes) mais méthode disjointe : les deux MethodRouter
        // fusionnent au merge sans conflit.
        .route(
            "/api/v1/invoices/{id}",
            delete(routes::invoices::delete_invoice),
        )
        // Story 17-3a : export complet d'installation (.keshbackup, Admin + anti-PAT).
        .route("/api/v1/admin/full-export", get(routes::admin::full_export))
        // Story 17-3c : import complet d'installation (multipart, Admin + anti-PAT).
        // DefaultBodyLimit propre (KESH_ADMIN_IMPORT_MAX_MB / admin_import_max_mib).
        .route(
            "/api/v1/admin/full-import",
            post(routes::admin::full_import).layer(axum::extract::DefaultBodyLimit::max(
                // saturating_mul : évite l'overflow usize sur cible 32-bit
                // (10240 MiB * 1024² > u32::MAX) — review Pass 3.
                (state.config.admin_import_max_mib as usize).saturating_mul(1024 * 1024),
            )),
        )
        // Story 20-1 : socle templates d'e-mail (Admin-only — pas de lecture
        // Comptable+ v1, seul consommateur = section Admin dédiée 20-2).
        .route(
            "/api/v1/admin/email-templates",
            get(routes::email_templates::list_email_templates),
        )
        .route(
            "/api/v1/admin/email-templates/{template_type}/{language}",
            get(routes::email_templates::get_email_template)
                .put(routes::email_templates::update_email_template)
                .delete(routes::email_templates::restore_email_template_default),
        )
        .route_layer(axum::middleware::from_fn(
            crate::middleware::rbac::require_admin_role,
        ));

    // Routes comptable+ (Admin + Comptable) : modification du plan comptable,
    // saisie d'écritures en partie double
    let comptable_routes = Router::new()
        .route("/api/v1/accounts", post(routes::accounts::create_account))
        .route(
            "/api/v1/accounts/{id}",
            put(routes::accounts::update_account),
        )
        .route(
            "/api/v1/accounts/{id}/archive",
            put(routes::accounts::archive_account),
        )
        .route(
            "/api/v1/journal-entries",
            post(routes::journal_entries::create_journal_entry),
        )
        .route(
            "/api/v1/journal-entries/{id}",
            put(routes::journal_entries::update_journal_entry)
                .delete(routes::journal_entries::delete_journal_entry),
        )
        // Story 4.1 : mutations carnet d'adresses
        .route("/api/v1/contacts", post(routes::contacts::create_contact))
        .route(
            "/api/v1/contacts/{id}",
            put(routes::contacts::update_contact),
        )
        .route(
            "/api/v1/contacts/{id}/archive",
            put(routes::contacts::archive_contact),
        )
        // #213 : personnes de contact d'une entreprise (CRM, informatif).
        .route(
            "/api/v1/contacts/{id}/persons",
            post(routes::contact_persons::create_person),
        )
        .route(
            "/api/v1/contact-persons/{id}",
            put(routes::contact_persons::update_person)
                .delete(routes::contact_persons::delete_person),
        )
        // Story 4.2 : mutations catalogue produits
        .route("/api/v1/products", post(routes::products::create_product))
        .route(
            "/api/v1/products/{id}",
            put(routes::products::update_product),
        )
        .route(
            "/api/v1/products/{id}/archive",
            put(routes::products::archive_product),
        )
        // Story 5.1 : mutations factures brouillon (PUT Comptable+).
        // #219 : le DELETE d'une facture (brouillon ou validée) est réservé
        // Admin — enregistré dans `admin_routes` (même path, méthode disjointe,
        // mergé sans conflit, cf. pattern vat-rates GET/POST).
        .route("/api/v1/invoices", post(routes::invoices::create_invoice))
        .route(
            "/api/v1/invoices/{id}",
            put(routes::invoices::update_invoice),
        )
        // Story 12.1 : création d'avoir (create+issue atomique, Comptable+)
        .route(
            "/api/v1/credit-notes",
            post(routes::credit_notes::create_credit_note),
        )
        // Story 12.2 : factures fournisseurs — mutations (Comptable+)
        .route(
            "/api/v1/supplier-invoices",
            post(routes::supplier_invoices::create_supplier_invoice),
        )
        .route(
            "/api/v1/supplier-invoices/{id}/pay",
            post(routes::supplier_invoices::pay_supplier_invoice),
        )
        .route(
            "/api/v1/supplier-invoices/{id}/cancel",
            post(routes::supplier_invoices::cancel_supplier_invoice),
        )
        // Story 12.4 : scan QR-facture → pré-remplissage (Comptable+, lecture seule).
        .route(
            "/api/v1/supplier-invoices/scan-qr",
            post(routes::supplier_invoices::scan_qr_supplier_invoice),
        )
        // Story 12.5c : import répertoire de factures (#194) — Comptable+.
        .route(
            "/api/v1/inbox-import",
            post(routes::imported_supplier_invoices::post_inbox_import),
        )
        .route(
            "/api/v1/imported-supplier-invoices",
            get(routes::imported_supplier_invoices::list_imported),
        )
        .route(
            "/api/v1/imported-supplier-invoices/{id}/complete",
            post(routes::imported_supplier_invoices::complete_import),
        )
        .route(
            "/api/v1/imported-supplier-invoices/{id}/discard",
            post(routes::imported_supplier_invoices::discard_import),
        )
        .route(
            "/api/v1/imported-supplier-invoices/{id}/source-document",
            get(routes::imported_supplier_invoices::get_imported_source_document),
        )
        .route(
            "/api/v1/supplier-invoices/{id}/source-document",
            get(routes::imported_supplier_invoices::get_supplier_invoice_source_document),
        )
        // Story 12.3 : lots de paiement pain.001 — mutations (Comptable+)
        .route(
            "/api/v1/payment-batches",
            post(routes::payment_batches::create_payment_batch),
        )
        .route(
            "/api/v1/payment-batches/{id}/confirm",
            post(routes::payment_batches::confirm_payment_batch),
        )
        .route(
            "/api/v1/payment-batches/{id}/cancel",
            post(routes::payment_batches::cancel_payment_batch),
        )
        // Story 5.2 : validation facture draft → validated
        .route(
            "/api/v1/invoices/{id}/validate",
            post(routes::invoices::validate_invoice_handler),
        )
        // Story 5.4 : marquage manuel paiement + export CSV échéancier
        // (Comptable+ — review pass 1 G2 B / B2).
        .route(
            "/api/v1/invoices/due-dates/export.csv",
            get(routes::invoices::export_due_dates_csv_handler),
        )
        .route(
            "/api/v1/invoices/{id}/mark-paid",
            post(routes::invoices::mark_invoice_paid_handler),
        )
        .route(
            "/api/v1/invoices/{id}/unmark-paid",
            post(routes::invoices::unmark_invoice_paid_handler),
        )
        // Story 3.7 : mutations exercices comptables
        .route(
            "/api/v1/fiscal-years",
            post(routes::fiscal_years::create_fiscal_year),
        )
        .route(
            "/api/v1/fiscal-years/{id}",
            put(routes::fiscal_years::update_fiscal_year),
        )
        .route(
            "/api/v1/fiscal-years/{id}/close",
            post(routes::fiscal_years::close_fiscal_year),
        )
        // Story 8-1b : import bancaire CAMT.053 (Comptable+ uniquement
        // pour les mutations — POST preview + POST create).
        // DefaultBodyLimit appliqué localement pour ces deux endpoints
        // multipart (cf. KESH_BANK_IMPORT_MAX_MB / config.bank_import_max_mib).
        .route(
            "/api/v1/bank-imports/preview",
            post(routes::bank_imports::preview).layer(axum::extract::DefaultBodyLimit::max(
                (state.config.bank_import_max_mib as usize) * 1024 * 1024,
            )),
        )
        .route(
            "/api/v1/bank-imports",
            post(routes::bank_imports::create).layer(axum::extract::DefaultBodyLimit::max(
                (state.config.bank_import_max_mib as usize) * 1024 * 1024,
            )),
        )
        // Story 8-2 : write profils CSV (Comptable+).
        .route("/api/v1/bank-profiles", post(routes::bank_profiles::create))
        .route(
            "/api/v1/bank-profiles/{id}",
            axum::routing::put(routes::bank_profiles::update).delete(routes::bank_profiles::delete),
        )
        // Story 8-4 : routes réconciliation (Comptable+, FR44).
        .route(
            "/api/v1/reconciliation/proposals",
            get(routes::reconciliation::get_proposals),
        )
        .route(
            "/api/v1/reconciliation/accept",
            post(routes::reconciliation::post_accept),
        )
        .route(
            "/api/v1/reconciliation/reject",
            post(routes::reconciliation::post_reject),
        )
        // Story 8-5a-base : route réconciliation manuelle (Comptable+, FR45).
        .route(
            "/api/v1/reconciliation/manual",
            post(routes::reconciliation::post_manual),
        )
        // Story 8-5a-bis : route éclatement de transaction (Comptable+, FR48).
        .route(
            "/api/v1/reconciliation/split",
            post(routes::reconciliation::post_split),
        )
        // Story 8-5a-zero : configuration `bank_account.journal_account_id`
        // (Comptable+ pour la mutation, foundation FR45/FR48).
        // Story v014-1 : CRUD complet post-onboarding (POST/PUT/DELETE).
        .route(
            "/api/v1/bank-accounts",
            post(routes::bank_accounts::create_bank_account),
        )
        .route(
            "/api/v1/bank-accounts/{id}",
            patch(routes::bank_accounts::patch_bank_account_journal_link)
                .put(routes::bank_accounts::update_bank_account)
                .delete(routes::bank_accounts::archive_bank_account),
        )
        // Story 8-5b : CRUD règles d'affectation (Comptable+ mutations).
        .route(
            "/api/v1/reconciliation/rules",
            post(routes::reconciliation_rules::post_create),
        )
        .route(
            "/api/v1/reconciliation/rules/{id}",
            patch(routes::reconciliation_rules::patch).delete(routes::reconciliation_rules::delete),
        )
        // Story 17-2a : gestion des clés API externes (PAT). Guard Comptable+
        // (DC4). DC6 : gestion interdite via PAT — les handlers rejettent
        // 403 API_KEY_MANAGEMENT_FORBIDDEN si current_user.api_key_id.is_some().
        .route(
            "/api/v1/settings/api-keys",
            get(routes::api_keys::list).post(routes::api_keys::create),
        )
        .route(
            "/api/v1/settings/api-keys/{id}",
            delete(routes::api_keys::revoke),
        )
        // Story 19-1 (Epic 19) : CRUD projets analytiques (Comptable+ mutations).
        .route("/api/v1/projects", post(routes::projects::create_project))
        .route(
            "/api/v1/projects/{id}",
            put(routes::projects::update_project),
        )
        .route(
            "/api/v1/projects/{id}/archive",
            post(routes::projects::archive_project),
        )
        .route(
            "/api/v1/projects/{id}/unarchive",
            post(routes::projects::unarchive_project),
        )
        .route_layer(axum::middleware::from_fn(
            crate::middleware::rbac::require_comptable_role,
        ));

    // Routes authentifiées (tout rôle) : changement de mot de passe, i18n, onboarding, companies, consultation plan comptable, lecture écritures
    let authenticated_routes = Router::new()
        .route("/api/v1/accounts", get(routes::accounts::list_accounts))
        .route(
            "/api/v1/journal-entries",
            get(routes::journal_entries::list_journal_entries),
        )
        // Détail d'une écriture (lecture, tout rôle authentifié — symétrique
        // de la liste ci-dessus ; mutations PUT/DELETE en comptable_routes).
        .route(
            "/api/v1/journal-entries/{id}",
            get(routes::journal_entries::get_journal_entry),
        )
        // Story 4.1 : lecture carnet d'adresses (tout rôle authentifié)
        .route("/api/v1/contacts", get(routes::contacts::list_contacts))
        .route("/api/v1/contacts/{id}", get(routes::contacts::get_contact))
        .route(
            "/api/v1/contacts/{id}/persons",
            get(routes::contact_persons::list_persons),
        )
        // Story 4.2 : lecture catalogue produits (tout rôle authentifié)
        .route("/api/v1/products", get(routes::products::list_products))
        .route("/api/v1/products/{id}", get(routes::products::get_product))
        // Story 7.2 (KF-003) : lecture taux TVA configurés pour la company.
        .route("/api/v1/vat-rates", get(routes::vat::list_vat_rates))
        // Story 5.1 : lecture factures (tout rôle authentifié)
        .route("/api/v1/invoices", get(routes::invoices::list_invoices))
        // Story 5.4 : échéancier en lecture (segment statique, prioritaire sur {id}).
        // L'export CSV est restreint au rôle Comptable+ (voir comptable_routes)
        // — décision review pass 1 G2 B (B2) : prévenir l'exfiltration de
        // 10'000 lignes par un rôle Consultation lecture-seule.
        .route(
            "/api/v1/invoices/due-dates",
            get(routes::invoices::list_due_dates_handler),
        )
        .route("/api/v1/invoices/{id}", get(routes::invoices::get_invoice))
        // Story 5.3 : téléchargement PDF QR Bill (tout rôle authentifié)
        .route(
            "/api/v1/invoices/{id}/pdf",
            get(routes::invoice_pdf::get_invoice_pdf),
        )
        // Story 12.1 : lectures avoirs (tout rôle authentifié)
        .route(
            "/api/v1/credit-notes",
            get(routes::credit_notes::list_credit_notes),
        )
        .route(
            "/api/v1/credit-notes/{id}",
            get(routes::credit_notes::get_credit_note),
        )
        .route(
            "/api/v1/credit-notes/{id}/pdf",
            get(routes::credit_notes::get_credit_note_pdf),
        )
        // Story 12.2 : lectures factures fournisseurs (tout rôle authentifié)
        .route(
            "/api/v1/supplier-invoices",
            get(routes::supplier_invoices::list_supplier_invoices),
        )
        .route(
            "/api/v1/supplier-invoices/{id}",
            get(routes::supplier_invoices::get_supplier_invoice),
        )
        // Story 12.3 : lectures lots de paiement + download pain.001 (tout rôle authentifié)
        .route(
            "/api/v1/payment-batches",
            get(routes::payment_batches::list_payment_batches),
        )
        .route(
            "/api/v1/payment-batches/{id}",
            get(routes::payment_batches::get_payment_batch),
        )
        .route(
            "/api/v1/payment-batches/{id}/pain001",
            get(routes::payment_batches::get_payment_batch_pain001),
        )
        // Story 5.2 : lecture config facturation (tout rôle authentifié)
        .route(
            "/api/v1/company/invoice-settings",
            get(routes::company_invoice_settings::get_invoice_settings),
        )
        .route("/api/v1/auth/password", put(routes::auth::change_password))
        // Story 10-5 — endpoint /me pour restaurer l'état frontend post-hydrate
        // (le JWT en cookie HttpOnly n'est pas décodable côté JS).
        .route("/api/v1/auth/me", get(routes::auth::me))
        .route("/api/v1/i18n/messages", get(routes::i18n::get_messages))
        .route(
            "/api/v1/companies/current",
            get(routes::companies::get_current),
        )
        .route("/api/v1/profile/mode", put(routes::profile::set_mode))
        .route(
            "/api/v1/onboarding/state",
            get(routes::onboarding::get_state),
        )
        .route(
            "/api/v1/onboarding/language",
            post(routes::onboarding::set_language),
        )
        .route(
            "/api/v1/onboarding/mode",
            post(routes::onboarding::set_mode),
        )
        .route(
            "/api/v1/onboarding/seed-demo",
            post(routes::onboarding::seed_demo),
        )
        .route("/api/v1/onboarding/reset", post(routes::onboarding::reset))
        .route(
            "/api/v1/onboarding/start-production",
            post(routes::onboarding::start_production),
        )
        .route(
            "/api/v1/onboarding/org-type",
            post(routes::onboarding::set_org_type),
        )
        .route(
            "/api/v1/onboarding/accounting-language",
            post(routes::onboarding::set_accounting_language),
        )
        .route(
            "/api/v1/onboarding/coordinates",
            post(routes::onboarding::set_coordinates),
        )
        .route(
            "/api/v1/onboarding/bank-account",
            post(routes::onboarding::set_bank_account),
        )
        .route(
            "/api/v1/onboarding/skip-bank",
            post(routes::onboarding::skip_bank),
        )
        .route(
            "/api/v1/onboarding/finalize",
            post(routes::onboarding::finalize),
        )
        // Story 3.7 : lecture exercices comptables (tout rôle authentifié)
        .route(
            "/api/v1/fiscal-years",
            get(routes::fiscal_years::list_fiscal_years),
        )
        .route(
            "/api/v1/fiscal-years/{id}",
            get(routes::fiscal_years::get_fiscal_year),
        )
        // Story 8-1b : lecture imports bancaires (tout rôle authentifié,
        // multi-tenant scoping via current_user.company_id du JWT).
        .route("/api/v1/bank-imports", get(routes::bank_imports::list))
        .route(
            "/api/v1/bank-imports/{id}",
            get(routes::bank_imports::detail),
        )
        // Story 8-2 : lecture profils CSV bancaires (tous rôles
        // authentifiés, multi-tenant KF-002).
        .route("/api/v1/bank-profiles", get(routes::bank_profiles::list))
        .route(
            "/api/v1/bank-profiles/{id}",
            get(routes::bank_profiles::detail),
        )
        // Story 8-5a-zero : lecture des bank_accounts de la company
        // (tous rôles authentifiés, multi-tenant KF-002).
        .route(
            "/api/v1/bank-accounts",
            get(routes::bank_accounts::list_bank_accounts),
        )
        // Story 19-1 : lecture projets analytiques (tous rôles authentifiés).
        .route("/api/v1/projects", get(routes::projects::list_projects))
        .route("/api/v1/projects/{id}", get(routes::projects::get_project))
        // Story 8-5b : lecture règles d'affectation (tous rôles authentifiés).
        .route(
            "/api/v1/reconciliation/rules",
            get(routes::reconciliation_rules::list),
        )
        .route(
            "/api/v1/reconciliation/rules/{id}",
            get(routes::reconciliation_rules::detail),
        )
        // Story 9-1 : rapports comptables (tous rôles authentifiés — lecture seule, FR65).
        .route(
            "/api/v1/reports/balance-sheet",
            get(routes::reports::get_balance_sheet),
        )
        .route(
            "/api/v1/reports/income-statement",
            get(routes::reports::get_income_statement),
        )
        .route(
            "/api/v1/reports/trial-balance",
            get(routes::reports::get_trial_balance),
        )
        .route(
            "/api/v1/reports/journals",
            get(routes::reports::get_journal_report),
        )
        // Story 9-2a : export PDF/CSV des 4 rapports comptables. Routes DOIVENT
        // rester dans `authenticated_routes` AVANT le `;` (Pass 1 BH-H1) sinon
        // routes orphelines hors guards → 401 silencieusement bypass → IDOR cross-tenant.
        .route(
            "/api/v1/reports/balance-sheet/export",
            get(routes::reports::export_balance_sheet),
        )
        .route(
            "/api/v1/reports/income-statement/export",
            get(routes::reports::export_income_statement),
        )
        .route(
            "/api/v1/reports/trial-balance/export",
            get(routes::reports::export_trial_balance),
        )
        .route(
            "/api/v1/reports/journals/export",
            get(routes::reports::export_journal_report),
        )
        // Story 11-2 : rapport TVA (JSON + export PDF/CSV). DOIT rester dans
        // `authenticated_routes` AVANT le `;` (anti-IDOR cross-tenant, cf. BH-H1).
        .route("/api/v1/reports/vat", get(routes::reports::get_vat_report))
        .route(
            "/api/v1/reports/vat/export",
            get(routes::reports::export_vat_report),
        )
        // Story 19-6a : rapport « Dépenses par projet » (JSON + export PDF/CSV).
        // DOIT rester dans `authenticated_routes` AVANT le `;` (anti-IDOR, BH-H1).
        .route(
            "/api/v1/reports/project-expenses",
            get(routes::reports::get_project_expenses),
        )
        .route(
            "/api/v1/reports/project-expenses/export",
            get(routes::reports::export_project_expenses),
        )
        // Story 19-6b : rapport « Rendement par projet » (JSON + export PDF/CSV).
        // DOIT rester dans `authenticated_routes` AVANT le `;` (anti-IDOR, BH-H1).
        .route(
            "/api/v1/reports/project-return",
            get(routes::reports::get_project_return),
        )
        .route(
            "/api/v1/reports/project-return/export",
            get(routes::reports::export_project_return),
        )
        // Story 9-2b : export global ZIP de souveraineté. Route DOIT rester
        // dans `authenticated_routes` AVANT le `;` (Pass 1 BH-H1 anti-IDOR) ;
        // sinon middleware auth bypass silencieux → IDOR cross-tenant.
        .route(
            "/api/v1/exports/global.zip",
            get(routes::exports::export_global),
        );

    // Merge + auth JWT (couche de base pour toutes les routes protégées)
    let protected = Router::new()
        .merge(admin_routes)
        .merge(comptable_routes)
        .merge(authenticated_routes)
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth::require_auth,
        ));

    let mut main_router = Router::new()
        .route("/health", get(routes::health::health_check))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/logout", post(routes::auth::logout))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh))
        // Story v011-5 — endpoint setup-admin self-service (route publique,
        // auto-disable au 1er admin créé via gate `user_count > 0` → 410).
        // Pas de `route_layer(require_auth)` — c'est précisément le but : créer
        // le 1er admin SANS auth préalable. Le handler intègre son propre
        // check rate-limit IP (cohérent /login).
        .route("/api/v1/setup/admin", post(routes::setup::create_admin));

    // Story 17-4c (DC9/AC15-16) — recovery self-service. Les 2 endpoints publics
    // (pré-login, hors `require_auth`) ne sont montés QUE si le feature est activé
    // (`KESH_FEATURE_FORGOT_PASSWORD`). Désactivé → routes absentes → `404`
    // (fallback break-glass `KESH_ADMIN_RESET`). Même pattern conditionnel que le
    // bloc `test_mode` ci-dessous. Montés AVANT `.merge(protected)`.
    if state.config.forgot_password_enabled {
        main_router = main_router
            .route(
                "/api/v1/auth/forgot-password",
                post(routes::auth::forgot_password),
            )
            .route(
                "/api/v1/auth/reset-password",
                post(routes::auth::reset_password),
            );
    }

    main_router = main_router.merge(protected);

    // Story 6.4 : routes /api/v1/_test/* gated par test_mode (runtime branch,
    // pas Cargo feature — évite drift build CI vs prod). Le garde-fou
    // `ConfigError::TestModeWithPublicBind` refuse déjà le démarrage si
    // test_mode=true + bind non-loopback, donc arriver ici avec test_mode=true
    // implique un bind loopback strict.
    if state.config.test_mode {
        tracing::warn!(
            "KESH_TEST_MODE=true — /api/v1/_test/* exposé (DEV/CI ONLY, jamais en prod)"
        );
        main_router = main_router.nest("/api/v1/_test", routes::test_endpoints::router());
    }

    // Story 10-5 (T5.2 / Pass 3 F-CSP-API-ATTACK-SURFACE-P3-3) — CSP middleware
    // appliqué APRÈS `fallback_service()` pour envelopper aussi le ServeDir
    // (gotcha Axum 0.8 : `.layer()` AVANT `.fallback_service()` n'enveloppe
    // PAS le fallback qui est un Service séparé). Le filtre content-type
    // dans `csp_html` n'émet le header que pour les réponses HTML, donc
    // application globale safe (pas de pollution JSON `/api/v1/*`).
    main_router
        .fallback_service(fallback)
        .layer(axum::middleware::from_fn(crate::middleware::csp::csp_html))
        .with_state(state)
}

// NOTE: les stories futures ajouteront leurs routes protégées en
// construisant un sous-routeur ainsi :
//
// ```ignore
// let protected = Router::new()
//     .route("/api/v1/accounts", get(...))
//     .route("/api/v1/journal-entries", post(...))
//     .route_layer(axum::middleware::from_fn_with_state(
//         state.clone(),
//         crate::middleware::auth::require_auth,
//     ));
// let app = build_router(state, static_dir).merge(protected);
// ```
//
// **Important Axum 0.8** : le `route_layer` doit venir APRÈS les
// `route(...)`, sinon Axum panique avec « Adding a route_layer before
// any routes is a no-op ».
