//! Garde des acteurs du journal d'audit — Story 35-8a1 (#431, #892 item 11 ;
//! C-e350-40, C-e350-49, C-e350-52).
//!
//! # Ce qu'elle établit
//!
//! ✅ Que **chaque site de production qui écrit ou construit un acteur sans clé
//! d'API** est inventorié, **site par site**, avec sa raison. Un dépôt qui
//! reçoit un `user_id: i64` nu ne peut pas savoir qu'une clé agit : il écrit
//! `NewAuditLogEntry::user`, et l'action d'une intégration est journalisée comme
//! celle d'une personne (#431). Cette garde rend impossible d'en ajouter un, ou
//! d'en déplacer un, sans que rien ne rougisse.
//!
//! # L'inventaire des sites NON RÉSOLUS (D4-ter du `CLAUDE.md`)
//!
//! On n'énumère pas les formes qui attribuent la clé ; on inventorie l'ensemble
//! clos de ce qui ne l'attribue pas. **Cinq motifs**, cherchés sur la source
//! assainie **jointe** (blancs et retours à la ligne tolérés entre les jetons) :
//!
//! 1. `NewAuditLogEntry::user(` — la trace écrite sans clé ;
//! 2. `Acteur::utilisateur(` — l'acteur construit sans clé ;
//! 3. `api_key_id: None` — `actor_api_key_id: None` compris : un acteur ou une
//!    entrée littéralement sans clé (`letterings::Actor { …, api_key_id: None }`) ;
//! 4. `for_actor(<user_id>, None, …)` — la forme threadée, clé jetée ;
//! 5. `delete_in_tx(…, None, …)` — le `None` **positionnel** de la suppression
//!    d'une écriture (C-e350-52).
//!
//! # Sa clé : le SITE, non le fichier
//!
//! La table [`SITES`] est close : `(fichier, fonction englobante, motif) →
//! (nombre, raison, appelant)`. Tout écart rougit en nommant fichier, fonction et
//! motif — un site neuf, un site converti sans que la table suive, un site qui
//! change de fonction. ⚠️ Un compte **par fichier** resterait vert quand on
//! ajoute un `::user` dans `lock_books` en convertissant `unlock_books`.
//!
//! La fonction englobante d'un site est le dernier `fn <nom>` ouvert avant lui,
//! par balayage textuel à profondeur d'accolades suivie (pas `syn` : un site dans
//! une macro y serait invisible). Deux fonctions homonymes d'un même fichier
//! partagent une entrée.
//!
//! # Les raisons
//!
//! Chaque site compté en a une ([`Raison`]) et cite l'appelant qui l'établit.
//! Une fonction atteinte aussi par une route comptable n'entre dans aucune raison
//! finale : elle est [`Raison::AConvertir`].
//!
//! # Angles morts, écrits
//!
//! 1. Le `None` positionnel passé à une fonction **autre que** `delete_in_tx` qui
//!    prend `actor_api_key_id: Option<i64>` (26 fonctions de `src` hors
//!    `delete_in_tx` à la fiche, P3 ; `reconciliation_cancel.rs` (`cancel` →
//!    `cancel_in_tx`) et `kesh-seed/src/pieges.rs` (`invoices::mark_emailed`)
//!    aujourd'hui) — à reprendre au déploiement (Epic 36).
//! 2. Un fichier de `src` **entièrement** sous `#[cfg(test)]` par un `mod x;` ou
//!    un `#[path]` de son parent (`pre_upgrade_backup/tests.rs`,
//!    `kesh-core/src/bank_imports/manual_tests.rs`,
//!    `kesh-api/src/qr_decode_synthetique.rs`) est lu comme de la production :
//!    aucun site aujourd'hui, mais un site futur y serait compté (faux rouge, non
//!    faux vert).
//! 3. Les limites de `source_assainie` (`common/source_assainie.rs`, écrites à son
//!    doc-comment) : un `//` dans une chaîne tronque la ligne, les commentaires de
//!    bloc ne sont pas retirés, les chaînes brutes ne sont pas comprises.
//! 4. Le motif 4 ne voit pas un premier argument qui porte une parenthèse ou une
//!    virgule (`for_actor(u.id(), None, …)`).
//! 5. Un `#[cfg(test)]` qui n'est pas seul sur sa ligne n'est pas masqué : le site
//!    qu'il garde est compté (faux rouge).

#[path = "common/source_assainie.rs"]
mod assainissement;

use std::collections::BTreeMap;
use std::sync::LazyLock;

use assainissement::{
    accolades_hors_chaines, args_de, fichiers_de_production, racine_crates, source_assainie,
};
use regex::Regex;

/// Les cinq motifs d'un acteur sans clé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Motif {
    /// `NewAuditLogEntry::user(`
    User,
    /// `Acteur::utilisateur(`
    Utilisateur,
    /// `api_key_id: None` (`actor_api_key_id: None` compris)
    ApiKeyNone,
    /// `for_actor(<user_id>, None, …)`
    ForActorNone,
    /// `delete_in_tx(…, None, …)`
    DeleteInTxNone,
}

/// Pourquoi un site écrit ou construit un acteur sans clé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Raison {
    /// Avant toute authentification : aucune clé ne peut agir.
    PreAuth,
    /// Le démarrage ou un geste sans requête ; l'acteur est choisi par l'appelant.
    Systeme,
    /// La restauration d'une sauvegarde signe par l'administrateur du jeu
    /// restauré (`audit_uid`), délibérément ; route sous `require_not_pat`.
    Restauration,
    /// Ce qui construit un acteur sans être une trace (corps d'un constructeur).
    Definition,
    /// Le serveur MCP : une clé MCP n'est pas une clé REST (suivi à #1057).
    Mcp,
    /// Atteint seulement par `admin_routes` sous `require_not_pat`, ou par un
    /// handler qui refuse une clé (`ensure_not_pat`).
    AdminSansCle,
    /// Tout le reste : une clé `read-write` peut y agir et n'est pas attribuée.
    /// Les écritures à la 35-8a2, le reste au déploiement (Epic 36).
    AConvertir,
}

use Motif::{ApiKeyNone, DeleteInTxNone, User};
use Raison::{AConvertir, AdminSansCle, Definition, Mcp, PreAuth, Restauration, Systeme};

/// **La table close des sites.** `(fichier, fonction englobante, motif, nombre,
/// raison, appelant qui établit la raison)` — les lignes de `lib.rs` sont celles
/// de la Story 35-8a1 ; une route comptable (`comptable_routes`) est atteinte
/// par une clé `read-write`.
///
/// Recomptée depuis la source à la Story 35-8a1 (Change Log de la fiche).
#[rustfmt::skip]
const SITES: &[(&str, &str, Motif, usize, Raison, &str)] = &[
    // ── kesh-api ────────────────────────────────────────────────────────────
    ("crates/kesh-api/src/auth/bootstrap.rs", "ensure_admin_user", User, 2, Systeme,
        "démarrage (`main.rs` → `auth::bootstrap`), sans requête"),
    ("crates/kesh-api/src/middleware/auth.rs", "require_auth", ApiKeyNone, 1, Definition,
        "le `CurrentUser` d'une session JWT"),
    ("crates/kesh-api/src/routes/admin.rs", "run_backup_and_restore", User, 2, Restauration,
        "`full_import` (`POST /api/v1/admin/full-import`, lib.rs:342, `require_not_pat` ; `ensure_not_pat` admin.rs:207) — `user_id = audit_uid`"),
    ("crates/kesh-api/src/routes/api_keys.rs", "create", User, 1, AdminSansCle,
        "`POST /api/v1/settings/api-keys` (lib.rs:989), `ensure_not_pat` (api_keys.rs:96)"),
    ("crates/kesh-api/src/routes/api_keys.rs", "revoke", User, 1, AdminSansCle,
        "`DELETE /api/v1/settings/api-keys/{id}` (lib.rs:993), `ensure_not_pat`"),
    ("crates/kesh-api/src/routes/auth.rs", "change_password_with", User, 1, AdminSansCle,
        "`PUT /api/v1/auth/password` (lib.rs:1151), `change_password` refuse une clé (auth.rs:633)"),
    ("crates/kesh-api/src/routes/auth.rs", "process_forgot_password_match", User, 1, PreAuth,
        "`forgot_password` (lib.rs:1385), sans session"),
    ("crates/kesh-api/src/routes/auth.rs", "reset_password", User, 1, PreAuth,
        "`POST` de fin de réinitialisation (lib.rs:1389), sans session"),
    ("crates/kesh-api/src/routes/auth.rs", "tracer_vol_de_jeton", User, 1, PreAuth,
        "`refresh` (lib.rs:1367) : vol de session détecté, sans jeton d'accès"),
    ("crates/kesh-api/src/routes/companies.rs", "update_company_address", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:472), `require_not_pat`"),
    ("crates/kesh-api/src/routes/companies.rs", "update_company_contact_details", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:467), `require_not_pat`"),
    ("crates/kesh-api/src/routes/companies.rs", "update_company_email", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:460), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_envois.rs", "decider_envoi", User, 2, AdminSansCle,
        "`admin_routes` (lib.rs:413), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_keys.rs", "create_mcp_key", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:383), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_keys.rs", "revoke_mcp_key", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:387), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_keys.rs", "set_mcp_key_send_validation", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:391), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_prompts.rs", "ajouter_version", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:430), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_prompts.rs", "approuver_version", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:434), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_prompts.rs", "creer_prompt", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:422), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_prompts.rs", "refuser_version", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:438), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_prompts.rs", "revenir_au_standard", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:442), `require_not_pat`"),
    ("crates/kesh-api/src/routes/mcp_server_settings.rs", "update_mcp_server_settings", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:396), `require_not_pat`"),
    ("crates/kesh-api/src/routes/setup.rs", "create_admin", User, 1, PreAuth,
        "`POST /api/v1/setup/admin` (lib.rs:1373), avant tout compte"),
    ("crates/kesh-api/src/routes/users.rs", "create_user", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:273), `require_not_pat`"),
    ("crates/kesh-api/src/routes/users.rs", "disable_user", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:281), `require_not_pat`"),
    ("crates/kesh-api/src/routes/users.rs", "reset_password", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:285), `require_not_pat`"),
    ("crates/kesh-api/src/routes/users.rs", "update_user", User, 1, AdminSansCle,
        "`admin_routes` (lib.rs:277), `require_not_pat`"),
    ("crates/kesh-api/src/services/serveur_mcp.rs", "entree_appel", User, 1, Mcp,
        "le serveur MCP (`serveur_mcp.rs:230`, `envois_mcp.rs:231`, `prompts_mcp.rs:590`), clé MCP — #1057"),
    ("crates/kesh-api/src/services/serveur_mcp.rs", "entree_prompt", User, 1, Mcp,
        "le serveur MCP (`serveur_mcp.rs:327`), clé MCP — #1057"),
    // ── kesh-db ─────────────────────────────────────────────────────────────
    ("crates/kesh-db/src/entities/audit_log.rs", "user", ApiKeyNone, 1, Definition,
        "corps de `NewAuditLogEntry::user`"),
    ("crates/kesh-db/src/entities/audit_log.rs", "utilisateur", ApiKeyNone, 1, Definition,
        "corps d'`Acteur::utilisateur`"),
    ("crates/kesh-db/src/repositories/accounts.rs", "archive_guarded", User, 1, AConvertir,
        "`archive_account` (lib.rs:544)"),
    ("crates/kesh-db/src/repositories/accounts.rs", "create", User, 1, AConvertir,
        "`create_account` (lib.rs:537)"),
    ("crates/kesh-db/src/repositories/accounts.rs", "reactivate", User, 1, AConvertir,
        "`reactivate_account` (lib.rs:549)"),
    ("crates/kesh-db/src/repositories/accounts.rs", "update", User, 1, AConvertir,
        "`update_account` (lib.rs:540)"),
    ("crates/kesh-db/src/repositories/backup_schedule.rs", "update_settings", User, 1, AdminSansCle,
        "`PUT /api/v1/admin/backup-schedule` (lib.rs:353), `require_not_pat`"),
    ("crates/kesh-db/src/repositories/bank_profiles.rs", "create", User, 1, AConvertir,
        "`bank_profiles::create` (lib.rs:879)"),
    ("crates/kesh-db/src/repositories/bank_profiles.rs", "delete", User, 1, AConvertir,
        "`bank_profiles::delete` (lib.rs:882)"),
    ("crates/kesh-db/src/repositories/bank_profiles.rs", "update", User, 1, AConvertir,
        "`bank_profiles::update` (lib.rs:882)"),
    ("crates/kesh-db/src/repositories/companies.rs", "lock_books", User, 1, AConvertir,
        "`lock_company_books` (lib.rs:565)"),
    ("crates/kesh-db/src/repositories/companies.rs", "repair_installation_in_tx", User, 1, Systeme,
        "démarrage (`bootstrap.rs:411`) et restauration (`admin.rs:553`, `require_not_pat`) ; acteur choisi par l'appelant"),
    ("crates/kesh-db/src/repositories/companies.rs", "unlock_books", User, 1, AdminSansCle,
        "`unlock_company_books` (lib.rs:488), `require_not_pat`"),
    ("crates/kesh-db/src/repositories/company_dunning_settings.rs", "update", User, 1, AdminSansCle,
        "`update_dunning_settings` (lib.rs:302), `require_not_pat`"),
    ("crates/kesh-db/src/repositories/company_invoice_settings.rs", "update", User, 1, AdminSansCle,
        "`update_invoice_settings` (lib.rs:289), `require_not_pat`"),
    ("crates/kesh-db/src/repositories/contacts.rs", "archive", User, 1, AConvertir,
        "`archive_contact` (lib.rs:614)"),
    ("crates/kesh-db/src/repositories/contacts.rs", "create", User, 1, AConvertir,
        "`create_contact` (lib.rs:607)"),
    ("crates/kesh-db/src/repositories/contacts.rs", "update", User, 1, AConvertir,
        "`update_contact` (lib.rs:610)"),
    ("crates/kesh-db/src/repositories/credit_notes.rs", "create_credit_note", User, 2, AConvertir,
        "`create_credit_note` (lib.rs:650)"),
    ("crates/kesh-db/src/repositories/credit_notes.rs", "create_credit_note", ApiKeyNone, 1, AConvertir,
        "`create_credit_note` (lib.rs:650)"),
    ("crates/kesh-db/src/repositories/fiscal_years.rs", "build_audit_entry", User, 1, AConvertir,
        "`create_fiscal_year` (lib.rs:826), `close_fiscal_year` (lib.rs:834)"),
    ("crates/kesh-db/src/repositories/installation_settings.rs", "update_session_idle_minutes", User, 1, AdminSansCle,
        "`update_session_settings` (lib.rs:375), `require_not_pat`"),
    ("crates/kesh-db/src/repositories/invoice_settlements_write.rs", "cancel_settlement_in_tx", User, 1, AConvertir,
        "`cancel_invoice_settlement_handler` (lib.rs:775)"),
    ("crates/kesh-db/src/repositories/invoice_settlements_write.rs", "cancel_settlement_in_tx", ApiKeyNone, 1, AConvertir,
        "`cancel_invoice_settlement_handler` (lib.rs:775)"),
    ("crates/kesh-db/src/repositories/invoice_settlements_write.rs", "settle_invoice", User, 1, AConvertir,
        "`settle_invoice_handler` (lib.rs:771)"),
    ("crates/kesh-db/src/repositories/invoice_settlements_write.rs", "settle_invoice", ApiKeyNone, 1, AConvertir,
        "`settle_invoice_handler` (lib.rs:771)"),
    ("crates/kesh-db/src/repositories/invoice_settlements_write.rs", "write_off_invoice", User, 1, AConvertir,
        "`write_off_invoice_handler` (lib.rs:780)"),
    ("crates/kesh-db/src/repositories/invoice_settlements_write.rs", "write_off_invoice", ApiKeyNone, 1, AConvertir,
        "`write_off_invoice_handler` (lib.rs:780)"),
    ("crates/kesh-db/src/repositories/invoices.rs", "create", User, 1, AConvertir,
        "`create_invoice` (lib.rs:642)"),
    ("crates/kesh-db/src/repositories/invoices.rs", "delete", User, 1, AdminSansCle,
        "`delete_invoice` (lib.rs:322), `require_not_pat`"),
    ("crates/kesh-db/src/repositories/invoices.rs", "set_dunning_pause", User, 1, AConvertir,
        "`pause_dunning` (lib.rs:790)"),
    ("crates/kesh-db/src/repositories/invoices.rs", "unvalidate", User, 1, AConvertir,
        "`unvalidate_invoice_handler` (lib.rs:748)"),
    ("crates/kesh-db/src/repositories/invoices.rs", "unvalidate", DeleteInTxNone, 1, AConvertir,
        "`unvalidate_invoice_handler` (lib.rs:748) — 35-8a2"),
    ("crates/kesh-db/src/repositories/invoices.rs", "update", User, 1, AConvertir,
        "`update_invoice` (lib.rs:645)"),
    ("crates/kesh-db/src/repositories/invoices.rs", "validate_invoice", User, 1, AConvertir,
        "`validate_invoice_handler` (lib.rs:742)"),
    ("crates/kesh-db/src/repositories/journal_entries.rs", "create_in_tx_inner", User, 1, AConvertir,
        "`create_journal_entry` (lib.rs:553) — 35-8a2"),
    ("crates/kesh-db/src/repositories/journal_entries.rs", "reverse_in_tx_inner", User, 1, AConvertir,
        "`reverse_journal_entry` (lib.rs:571) — 35-8a2"),
    ("crates/kesh-db/src/repositories/journal_entries.rs", "reverse_in_tx_inner", ApiKeyNone, 1, AConvertir,
        "`reverse_journal_entry` (lib.rs:571), lettrage `reversal` — 35-8a2"),
    ("crates/kesh-db/src/repositories/payment_batches.rs", "cancel_batch", User, 1, AConvertir,
        "`cancel_payment_batch` (lib.rs:737)"),
    ("crates/kesh-db/src/repositories/payment_batches.rs", "confirm_batch", User, 1, AConvertir,
        "`confirm_payment_batch` (lib.rs:733)"),
    ("crates/kesh-db/src/repositories/payment_batches.rs", "create_batch", User, 1, AConvertir,
        "`create_payment_batch` (lib.rs:729)"),
    ("crates/kesh-db/src/repositories/products.rs", "archive", User, 1, AConvertir,
        "`archive_product` (lib.rs:634)"),
    ("crates/kesh-db/src/repositories/products.rs", "create", User, 1, AConvertir,
        "`create_product` (lib.rs:627)"),
    ("crates/kesh-db/src/repositories/products.rs", "update", User, 1, AConvertir,
        "`update_product` (lib.rs:630)"),
    ("crates/kesh-db/src/repositories/reconciliation_rules.rs", "create_in_tx", User, 1, AConvertir,
        "`reconciliation_rules::post_create` (lib.rs:978)"),
    ("crates/kesh-db/src/repositories/reconciliation_rules.rs", "soft_delete_by_id_for_company", User, 1, AConvertir,
        "`reconciliation_rules::delete` (lib.rs:982)"),
    ("crates/kesh-db/src/repositories/reconciliation_rules.rs", "update_in_tx", User, 1, AConvertir,
        "`reconciliation_rules::patch` (lib.rs:982)"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "cancel_in_tx", User, 1, AConvertir,
        "`cancel_supplier_invoice` (lib.rs:677)"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "cancel_in_tx", ApiKeyNone, 1, AConvertir,
        "`cancel_supplier_invoice` (lib.rs:677), lettrage `reversal`"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "cancel_settlement_in_tx", User, 1, AConvertir,
        "`cancel_supplier_invoice_settlement` (lib.rs:683)"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "cancel_settlement_in_tx", ApiKeyNone, 1, AConvertir,
        "`cancel_supplier_invoice_settlement` (lib.rs:683)"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "create_draft_in_tx", User, 1, AConvertir,
        "`create_supplier_invoice` (lib.rs:655)"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "delete_draft_in_tx", User, 2, AdminSansCle,
        "`delete_supplier_invoice` (lib.rs:330), `require_not_pat`"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "pay_in_tx", ApiKeyNone, 1, AConvertir,
        "`pay_supplier_invoice` (lib.rs:673)"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "update_draft_in_tx", User, 1, AConvertir,
        "`update_supplier_invoice` (lib.rs:661)"),
    ("crates/kesh-db/src/repositories/supplier_invoices.rs", "validate_in_tx", User, 1, AConvertir,
        "`validate_supplier_invoice` (lib.rs:665)"),
];

// ---------------------------------------------------------------------------
// Le relevé
// ---------------------------------------------------------------------------

static RE_USER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"NewAuditLogEntry::user\s*\(").unwrap());
static RE_UTILISATEUR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Acteur::utilisateur\s*\(").unwrap());
static RE_API_KEY_NONE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"api_key_id:\s*None\b").unwrap());
static RE_FOR_ACTOR_NONE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"for_actor\(\s*[^,()]+,\s*None\b").unwrap());
static RE_DELETE_IN_TX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"delete_in_tx\s*\(").unwrap());
static RE_FN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());

/// Hors de toute fonction (constante, statique, `impl` sans méthode…).
const HORS_FONCTION: &str = "<hors fonction>";

/// La fonction englobante de chaque ligne d'une source assainie : le dernier
/// `fn <nom>` dont le corps est ouvert à cette ligne.
///
/// Une signature peut courir sur plusieurs lignes : le nom attend la première
/// accolade ouvrante qui suit ; une déclaration sans corps (`fn f(&self);`, dans
/// un trait) l'abandonne à son `;`.
fn fonctions_par_ligne(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pile: Vec<(String, i32)> = Vec::new();
    let mut en_attente: Option<String> = None;
    let mut profondeur = 0i32;
    for ligne in source.lines() {
        if let Some(c) = RE_FN.captures_iter(ligne).last() {
            en_attente = Some(c[1].to_string());
        }
        let (o, f) = accolades_hors_chaines(ligne);
        if o > 0
            && let Some(nom) = en_attente.take()
        {
            pile.push((nom, profondeur + 1));
        }
        out.push(
            pile.last()
                .map_or(HORS_FONCTION.to_string(), |(n, _)| n.clone()),
        );
        profondeur += o - f;
        while pile.last().is_some_and(|(_, p)| profondeur < *p) {
            pile.pop();
        }
        if en_attente.is_some() && o == 0 && ligne.trim_end().ends_with(';') {
            en_attente = None;
        }
    }
    out
}

/// Les sites d'une source **brute** : `(fonction englobante, motif)`, un par
/// occurrence. La source passe par `source_assainie` (commentaires retirés,
/// items `#[cfg(test)]` masqués).
fn sites_de(brut: &str) -> Vec<(String, Motif)> {
    let source = source_assainie(brut);
    let fonctions = fonctions_par_ligne(&source);
    let ligne_de = |pos: usize| source[..pos].matches('\n').count();
    let mut out = Vec::new();
    let mut noter = |pos: usize, motif: Motif| {
        let f = fonctions
            .get(ligne_de(pos))
            .cloned()
            .unwrap_or_else(|| HORS_FONCTION.to_string());
        out.push((f, motif));
    };
    for (re, motif) in [
        (&*RE_USER, Motif::User),
        (&*RE_UTILISATEUR, Motif::Utilisateur),
        (&*RE_API_KEY_NONE, Motif::ApiKeyNone),
        (&*RE_FOR_ACTOR_NONE, Motif::ForActorNone),
    ] {
        for m in re.find_iter(&source) {
            noter(m.start(), motif);
        }
    }
    for m in RE_DELETE_IN_TX.find_iter(&source) {
        // La définition porte le même nom : `fn delete_in_tx(`.
        if source[..m.start()].trim_end().ends_with("fn") {
            continue;
        }
        let Some(args) = args_de(&source[m.end()..]) else {
            continue;
        };
        if args.iter().any(|a| a.trim() == "None") {
            noter(m.start(), Motif::DeleteInTxNone);
        }
    }
    out
}

/// Le relevé d'un ensemble de sources : `(fichier, fonction, motif) → nombre`.
type Releve = BTreeMap<(String, String, Motif), usize>;

fn relever<'a>(sources: impl IntoIterator<Item = (String, &'a str)>) -> Releve {
    let mut r = Releve::new();
    for (fichier, brut) in sources {
        for (fonction, motif) in sites_de(brut) {
            *r.entry((fichier.clone(), fonction, motif)).or_default() += 1;
        }
    }
    r
}

/// Le relevé du dépôt : tout `crates/*/src/**/*.rs`.
fn releve_du_depot() -> Releve {
    let racine = racine_crates();
    let fichiers: Vec<(String, String)> = fichiers_de_production()
        .into_iter()
        .map(|chemin| {
            let relatif = format!(
                "crates/{}",
                chemin
                    .strip_prefix(&racine)
                    .expect("sous crates/")
                    .to_string_lossy()
            );
            let brut = std::fs::read_to_string(&chemin).expect("lire un fichier source");
            (relatif, brut)
        })
        .collect();
    relever(fichiers.iter().map(|(f, b)| (f.clone(), b.as_str())))
}

/// La table, en relevé.
fn releve_declare(table: &[(&str, &str, Motif, usize, Raison, &str)]) -> Releve {
    let mut r = Releve::new();
    for (fichier, fonction, motif, nombre, _, _) in table {
        let ancien = r.insert(
            ((*fichier).to_string(), (*fonction).to_string(), *motif),
            *nombre,
        );
        assert!(
            ancien.is_none(),
            "⛔ entrée en double dans SITES : {fichier} / {fonction} / {motif:?}"
        );
    }
    r
}

/// Les écarts entre le code et la table, chacun nommant fichier, fonction et
/// motif. Vide si et seulement si la table dit exactement le code.
fn ecarts(code: &Releve, table: &Releve) -> Vec<String> {
    let mut out = Vec::new();
    for (cle, n) in code {
        match table.get(cle) {
            None => out.push(format!(
                "site NON INVENTORIÉ : {} / fn {} / {:?} × {n}",
                cle.0, cle.1, cle.2
            )),
            Some(m) if m != n => out.push(format!(
                "compte faux : {} / fn {} / {:?} — code {n}, table {m}",
                cle.0, cle.1, cle.2
            )),
            Some(_) => {}
        }
    }
    for (cle, m) in table {
        if !code.contains_key(cle) {
            out.push(format!(
                "entrée ÉVANOUIE (site converti ou déplacé) : {} / fn {} / {:?} — table {m}, code 0",
                cle.0, cle.1, cle.2
            ));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// La garde
// ---------------------------------------------------------------------------

#[test]
fn chaque_acteur_sans_cle_de_la_production_est_inventorie_site_par_site() {
    let code = releve_du_depot();
    let table = releve_declare(SITES);
    let e = ecarts(&code, &table);
    assert!(
        e.is_empty(),
        "⛔ La table SITES de `audit_acteur_garde.rs` ne dit plus le code :\n  {}\n\n\
         Un site NEUF : passer l'acteur (`Acteur`, `NewAuditLogEntry::par`, \
         `Acteur::from(&current_user)`) plutôt qu'un `user_id` nu ; à défaut, \
         l'inscrire avec sa raison et l'appelant qui l'établit. Un site CONVERTI : \
         retirer ou décrémenter son entrée. ⚠️ Ne pas élargir une entrée pour \
         faire passer un site d'une autre fonction.",
        e.join("\n  ")
    );
}

#[test]
fn chaque_entree_cite_son_appelant() {
    for (fichier, fonction, motif, nombre, _, appelant) in SITES {
        assert!(
            *nombre > 0,
            "{fichier} / {fonction} / {motif:?} : un nombre nul n'est pas une entrée"
        );
        assert!(
            !appelant.trim().is_empty(),
            "{fichier} / {fonction} / {motif:?} : l'appelant qui établit la raison manque"
        );
    }
}

// ---------------------------------------------------------------------------
// Les sondes (AC5) — chacune un rouge attendu sur une source synthétique
// ---------------------------------------------------------------------------

fn un(fichier: &str, fonction: &str, motif: Motif, n: usize) -> ((String, String, Motif), usize) {
    ((fichier.to_string(), fonction.to_string(), motif), n)
}

/// (a) Un `NewAuditLogEntry::user(` ajouté dans une fonction non listée rougit,
/// en la nommant.
#[test]
fn sonde_a_un_site_neuf_rougit() {
    let table: Releve = [un("x.rs", "connue", Motif::User, 1)].into();
    let code = relever([(
        "x.rs".to_string(),
        "fn connue() { NewAuditLogEntry::user(1, \"a.b\", \"a\", 1, None); }\n\
         fn neuve() {\n    NewAuditLogEntry::user(1, \"a.c\", \"a\", 1, None);\n}\n",
    )]);
    let e = ecarts(&code, &table);
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(
        e[0].contains("NON INVENTORIÉ") && e[0].contains("fn neuve"),
        "{e:?}"
    );
}

/// (b) Un site converti sans que la table suive rougit (entrée évanouie).
#[test]
fn sonde_b_un_site_converti_sans_la_table_rougit() {
    let table: Releve = [un("x.rs", "f", Motif::User, 1)].into();
    let code = relever([(
        "x.rs".to_string(),
        "fn f(a: Acteur) {\n    NewAuditLogEntry::par(a, \"a.b\", \"a\", 1, None);\n}\n",
    )]);
    let e = ecarts(&code, &table);
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].contains("ÉVANOUIE") && e[0].contains("fn f"), "{e:?}");
}

/// (c) La forme coupée sur deux lignes est comptée.
#[test]
fn sonde_c_la_forme_coupee_est_comptee() {
    let s =
        sites_de("fn f() {\n    NewAuditLogEntry::user\n    (1, \"a.b\", \"a\", 1, None);\n}\n");
    assert_eq!(s, [("f".to_string(), Motif::User)]);
}

/// (d) Un `#[cfg(test)] mod …;` qui précède la production (modèle
/// `auth/bootstrap.rs`) ne cache pas le site qui le suit.
#[test]
fn sonde_d_un_mod_de_test_en_tete_ne_cache_pas_la_production() {
    let s = sites_de(
        "#[cfg(test)]\nmod tests;\n\nfn f() {\n    NewAuditLogEntry::user(1, \"a.b\", \"a\", 1, None);\n}\n\
         #[cfg(test)]\nmod t {\n    fn g() { NewAuditLogEntry::user(1, \"a.b\", \"a\", 1, None); }\n}\n",
    );
    assert_eq!(
        s,
        [("f".to_string(), Motif::User)],
        "le site de test est masqué, celui de f non"
    );
}

/// (e) L'échange — un site ajouté dans `lock_books`, un autre retiré de
/// `unlock_books` — rougit des deux côtés, alors qu'un compte par fichier
/// resterait vert.
#[test]
fn sonde_e_l_echange_entre_deux_fonctions_rougit() {
    let table: Releve = [un("c.rs", "unlock_books", Motif::User, 1)].into();
    let code = relever([(
        "c.rs".to_string(),
        "fn lock_books() {\n    NewAuditLogEntry::user(1, \"a.b\", \"a\", 1, None);\n}\n\
         fn unlock_books(a: Acteur) {\n    NewAuditLogEntry::par(a, \"a.c\", \"a\", 1, None);\n}\n",
    )]);
    let e = ecarts(&code, &table);
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(e.iter().any(|x| x.contains("fn lock_books")), "{e:?}");
    assert!(e.iter().any(|x| x.contains("fn unlock_books")), "{e:?}");
}

/// (f) `for_actor(user_id, None, …)` est compté — la clé jetée par la forme
/// threadée.
#[test]
fn sonde_f_for_actor_sans_cle_est_compte() {
    let s = sites_de(
        "fn f(user_id: i64) {\n    NewAuditLogEntry::for_actor(\n        user_id,\n        None,\n        \"a.b\", \"a\", 1, None);\n}\n\
         fn g(user_id: i64, k: Option<i64>) {\n    NewAuditLogEntry::for_actor(user_id, k, \"a.b\", \"a\", 1, None);\n}\n",
    );
    assert_eq!(s, [("f".to_string(), Motif::ForActorNone)]);
}

/// (g) Un site sous `#[cfg(not(test))]` (modèle `config.rs`) **reste compté** :
/// c'est de la production.
#[test]
fn sonde_g_cfg_not_test_reste_compte() {
    let s = sites_de(
        "#[cfg(not(test))]\nfn f() {\n    NewAuditLogEntry::user(1, \"a.b\", \"a\", 1, None);\n}\n",
    );
    assert_eq!(s, [("f".to_string(), Motif::User)]);
}

/// (h) Un `delete_in_tx(…, None, …)` ajouté dans une fonction non listée est
/// compté ; la clé passée ne l'est pas, ni la définition.
#[test]
fn sonde_h_delete_in_tx_sans_cle_est_compte() {
    let s = sites_de(
        "pub(crate) async fn delete_in_tx(tx: T, id: i64, user_id: i64, k: Option<i64>) {}\n\
         fn f(tx: T) {\n    super::journal_entries::delete_in_tx(\n        tx, 1, 2, None, false,\n    )\n    .await;\n}\n\
         fn g(tx: T, a: Acteur) {\n    delete_in_tx(tx, 1, a.user_id, a.api_key_id, false).await;\n}\n",
    );
    assert_eq!(s, [("f".to_string(), Motif::DeleteInTxNone)]);
}

/// Les motifs 2 et 3 : `Acteur::utilisateur(` et `api_key_id: None`
/// (`actor_api_key_id: None` compris), à leur fonction ; une signature sur
/// plusieurs lignes et une déclaration de trait sans corps ne trompent pas
/// l'attribution.
#[test]
fn sonde_motifs_2_et_3_et_signatures() {
    let s = sites_de(
        "trait T {\n    fn decl(&self);\n}\n\
         pub async fn longue(\n    a: i64,\n) -> Result<(), E> {\n    let x = Acteur::utilisateur(a);\n    let y = Actor {\n        user_id: a,\n        api_key_id: None,\n    };\n}\n\
         fn h() -> E {\n    E { actor_api_key_id: None }\n}\n",
    );
    assert_eq!(
        s,
        [
            ("longue".to_string(), Motif::Utilisateur),
            ("longue".to_string(), Motif::ApiKeyNone),
            ("h".to_string(), Motif::ApiKeyNone),
        ]
    );
}
