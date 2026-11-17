//! Repository pour `company_invoice_settings` (Story 5.2 — FR35).
//!
//! Relation 1-1 avec `companies`. Row créée à la volée (lazy) via
//! `INSERT IGNORE` au premier accès. Pattern « upsert read ».
//!
//! **Deux signatures de get-or-create** pour éviter les transactions
//! imbriquées :
//! - [`get_or_create_default`] : version pool-level pour le handler
//!   `GET /company/invoice-settings`.
//! - [`get_or_create_default_in_tx`] : version tx-level utilisée par
//!   [`invoices::validate_invoice`](super::invoices::validate_invoice)
//!   pour charger la config dans la transaction atomique de validation.
//!
//! **Duplication contrôlée** : le corps métier est dupliqué (5 lignes)
//! entre les deux fonctions avec un commentaire `MIRROR` de part et
//! d'autre. Le fallback HRTB générique sur `Executor` est notoirement
//! fragile avec SQLx 0.8 — duplication préférée (cf. spec pass 2 P13).
//!
//! ⚠️ La ligne créée à la volée n'a **aucun compte** : un `GET` des réglages fait
//! avant la fin de l'assistant (journal, comptes bancaires, factures) la laisse
//! vierge. Depuis la Story 36-19 (#1200), la finalisation la **complète**
//! ([`complete_default_accounts_in_tx`]) au lieu de la refuser.

use std::collections::BTreeSet;

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, QueryBuilder, Row};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{
    AccountRole, AccountType, CompanyInvoiceSettings, CompanyInvoiceSettingsUpdate, Journal,
};
use crate::errors::{
    ClaimSide, DbError, DefaultInvoiceAccountCause, DefaultInvoiceSetting, DesignatedSetting,
    NonPostableAccount, NonPostableAccounts, RoundingContext, UnavailableDefaultInvoiceAccount,
    WrongTypeDesignatedAccount, WrongTypeDesignatedAccounts, map_db_error,
};
use crate::repositories::{accounts, audit_log};

const COLUMNS: &str = "company_id, invoice_number_format, default_receivable_account_id, \
    default_revenue_account_id, default_vat_payable_account_id, \
    default_vat_recoverable_account_id, default_vat_decompte_account_id, \
    default_sales_journal, journal_entry_description_template, \
    credit_note_number_format, supplier_internal_number_format, default_payable_account_id, \
    default_rounding_account_id, round_to_5_centimes, minimum_invoice_amount, \
    default_discount_account_id, default_bank_fees_account_id, default_bad_debt_account_id, \
    version, created_at, updated_at";

/// Les dix comptes que les paramètres de facturation désignent : la colonne, et le
/// code stable que l'API rend pour elle (Story 33-8, #486). Un test compare cette
/// liste aux colonnes `default_%_account_id` réelles : un onzième réglage ne se tait
/// pas.
pub const DESIGNATED_ACCOUNT_COLUMNS: [(&str, &str); 10] = [
    ("default_receivable_account_id", "receivable"),
    ("default_revenue_account_id", "revenue"),
    ("default_vat_payable_account_id", "vatPayable"),
    ("default_vat_recoverable_account_id", "vatRecoverable"),
    ("default_vat_decompte_account_id", "vatDecompte"),
    ("default_payable_account_id", "payable"),
    ("default_rounding_account_id", "rounding"),
    ("default_discount_account_id", "discount"),
    ("default_bank_fees_account_id", "bankFees"),
    ("default_bad_debt_account_id", "badDebt"),
];

/// Les codes des réglages de `company_id` qui désignent `account_id`, dans l'ordre
/// de [`DESIGNATED_ACCOUNT_COLUMNS`] (Story 33-8, #486). Lecture seule, sans verrou :
/// une société sans ligne de réglages ne désigne rien.
pub async fn designations_of_account<'e, E>(
    executor: E,
    company_id: i64,
    account_id: i64,
) -> Result<Vec<&'static str>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    designations_of_account_with(executor, company_id, account_id, "").await
}

/// [`designations_of_account`] en lecture **verrouillante** (`LOCK IN SHARE
/// MODE`) — Story 37-1 (AC-4) : la garde « compte désigné dans les réglages »
/// d'un changement de devise, après le `FOR UPDATE` du compte (la ligne des
/// réglages est déjà tenue en partage par l'appelant, avant le compte).
pub async fn designations_of_account_in_share_mode<'e, E>(
    executor: E,
    company_id: i64,
    account_id: i64,
) -> Result<Vec<&'static str>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    designations_of_account_with(executor, company_id, account_id, " LOCK IN SHARE MODE").await
}

async fn designations_of_account_with<'e, E>(
    executor: E,
    company_id: i64,
    account_id: i64,
    lock_clause: &str,
) -> Result<Vec<&'static str>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let columns = DESIGNATED_ACCOUNT_COLUMNS
        .iter()
        .map(|(column, _)| *column)
        .collect::<Vec<_>>()
        .join(", ");
    let rows = sqlx::query(&format!(
        "SELECT {columns} FROM company_invoice_settings WHERE company_id = ?{lock_clause}"
    ))
    .bind(company_id)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)?;
    let mut codes = Vec::new();
    for row in &rows {
        for (column, code) in DESIGNATED_ACCOUNT_COLUMNS {
            let designated: Option<i64> = row.try_get(column).map_err(map_db_error)?;
            if designated == Some(account_id) {
                codes.push(code);
            }
        }
    }
    Ok(codes)
}

/// L'instantané des réglages que portent les entrées d'audit
/// `company_invoice_settings.updated` (`before`, `after`, clés en camelCase) —
/// public depuis la Story 36-19 : la finalisation de l'assistant l'emploie pour
/// le complément d'une ligne existante.
pub fn settings_snapshot_json(s: &CompanyInvoiceSettings) -> serde_json::Value {
    serde_json::json!({
        "companyId": s.company_id,
        "invoiceNumberFormat": s.invoice_number_format,
        "defaultReceivableAccountId": s.default_receivable_account_id,
        "defaultRevenueAccountId": s.default_revenue_account_id,
        "defaultVatPayableAccountId": s.default_vat_payable_account_id,
        "defaultVatRecoverableAccountId": s.default_vat_recoverable_account_id,
        "defaultVatDecompteAccountId": s.default_vat_decompte_account_id,
        "defaultSalesJournal": s.default_sales_journal.as_str(),
        "journalEntryDescriptionTemplate": s.journal_entry_description_template,
        "creditNoteNumberFormat": s.credit_note_number_format,
        "supplierInternalNumberFormat": s.supplier_internal_number_format,
        "defaultPayableAccountId": s.default_payable_account_id,
        "defaultRoundingAccountId": s.default_rounding_account_id,
        "roundTo5Centimes": s.round_to_5_centimes,
        "minimumInvoiceAmount": s.minimum_invoice_amount,
        "defaultDiscountAccountId": s.default_discount_account_id,
        "defaultBankFeesAccountId": s.default_bank_fees_account_id,
        "defaultBadDebtAccountId": s.default_bad_debt_account_id,
        "version": s.version,
    })
}

/// Retourne la config de la company (ou la crée avec les DEFAULT si
/// absente). Version **pool-level** — ouvre sa propre transaction pour
/// l'INSERT IGNORE + SELECT afin de garantir l'atomicité lazy-create.
pub async fn get_or_create_default(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // MIRROR: garder synchronisé avec get_or_create_default_in_tx.
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let settings = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(settings)
}

/// Retourne la config, tx-aware. Utilisé par `validate_invoice` pour
/// éviter d'ouvrir une transaction imbriquée. Le caller fournit la
/// transaction déjà ouverte.
///
/// **SELECT FOR UPDATE** obligatoire (review pass 1 P1) : verrouille la
/// row pour empêcher un `PUT /company/invoice-settings` concurrent de
/// modifier la config entre la lecture et le commit de `validate_invoice`.
/// Sans ce verrou, l'écriture comptable pourrait utiliser les anciens
/// comptes pendant que l'audit snapshotait les nouveaux (TOCTOU).
pub async fn get_or_create_default_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, DbError> {
    // MIRROR: garder synchronisé avec get_or_create_default.
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

    let settings = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ? FOR UPDATE"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    Ok(settings)
}

// ---------------------------------------------------------------------------
// Story 15-6c (#474) — la configuration ne prépare plus l'écriture nulle
// ---------------------------------------------------------------------------

/// Les deux **comptes de créance** désignés dans les réglages : le compte
/// débiteurs et le compte créanciers (Story 15-6c, AC2). `Default` = deux
/// `None` — ce que passent les gestes bancaires **sans** compte lié (aucune
/// lecture, aucun verrou) et les tests qui appellent le dépôt directement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClaimAccounts {
    /// `default_receivable_account_id`.
    pub receivable: Option<i64>,
    /// `default_payable_account_id`.
    pub payable: Option<i64>,
}

impl ClaimAccounts {
    /// Le réglage qu'occupe `account_id`, s'il en occupe un — le compte
    /// débiteurs d'abord.
    pub fn side_of(&self, account_id: i64) -> Option<ClaimSide> {
        if self.receivable == Some(account_id) {
            Some(ClaimSide::Receivable)
        } else if self.payable == Some(account_id) {
            Some(ClaimSide::Payable)
        } else {
            None
        }
    }
}

/// Lit les deux comptes de créance **sous verrou partagé** (Story 15-6c, AC2 ;
/// choix C-15-6-13, C-15-6-21) — ligne absente → [`ClaimAccounts::default`].
///
/// ⛔ **Ordre des verrous** : la ligne des réglages est **toujours verrouillée
/// avant** les comptes bancaires — ordre global `companies` (sentinelle, quand
/// elle est prise) → réglages → `bank_accounts`. Côté compte bancaire, cet
/// appel est le **premier verrou après la sentinelle** (création, remplacement)
/// ou la première instruction de la transaction (lien) ; côté réglages,
/// [`update`] tient la ligne en `FOR UPDATE` avant de lire
/// `bank_accounts::first_active_bank_account_linked_to`. Lire les réglages
/// **après** le `FOR UPDATE` de la ligne bancaire ouvrirait un interblocage
/// entre les deux gestes. Ainsi un lien et une désignation concurrents ne
/// passent plus tous les deux : le second attend le premier, lit sa valeur
/// validée, puis refuse.
///
/// ⚠️ `LOCK IN SHARE MODE` et non `FOR SHARE`, erreur de syntaxe sur MariaDB
/// 10.11. Le test de sérialisation côté compte bancaire attend sur le texte
/// `FROM company_invoice_settings` + `LOCK IN SHARE MODE` : changer la forme du
/// verrou, c'est changer ce motif.
///
/// ⚠️ **Latence** : le verrou S attend toute transaction qui tient la ligne en
/// X — PUT des réglages, validation de facture, avoir, facture fournisseur —
/// le temps de cette transaction. Une dé-liaison ne l'appelle pas.
pub async fn claim_accounts_in_share_mode(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<ClaimAccounts, DbError> {
    let row: Option<(Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT default_receivable_account_id, default_payable_account_id FROM company_invoice_settings WHERE company_id = ? LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(row
        .map(|(receivable, payable)| ClaimAccounts {
            receivable,
            payable,
        })
        .unwrap_or_default())
}

/// Refuse de désigner `requested` comme compte de créance `claim` s'il **change**
/// (≠ `current`, lu sous verrou) et qu'il est lié à un compte bancaire non
/// archivé (Story 15-6c, AC5) — [`DbError::ClaimAccountLinkedToBankAccount`],
/// nommant le premier compte bancaire par `id`.
///
/// Valeur inchangée ou `NULL` : aucun contrôle — une configuration antérieure
/// fautive ne bloque pas l'enregistrement des autres réglages (patron C4 de la
/// 15-5b) ; la garde à l'usage de la 15-6b reste son filet.
async fn refuse_if_linked_to_bank_account(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    requested: Option<i64>,
    current: Option<i64>,
    claim: ClaimSide,
) -> Result<(), DbError> {
    let Some(account_id) = requested else {
        return Ok(());
    };
    if requested == current {
        return Ok(());
    }
    let Some((bank_account_id, bank_label)) =
        super::bank_accounts::first_active_bank_account_linked_to(tx, company_id, account_id)
            .await?
    else {
        return Ok(());
    };
    let account_number = super::accounts::number_in_company(tx, company_id, account_id).await?;
    Err(DbError::ClaimAccountLinkedToBankAccount {
        account_id,
        account_number,
        claim,
        bank_account_id,
        bank_label,
    })
}

/// Compare l'état persisté au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
fn is_no_op_change(
    before: &CompanyInvoiceSettings,
    changes: &CompanyInvoiceSettingsUpdate,
) -> bool {
    before.invoice_number_format == changes.invoice_number_format
        && before.default_receivable_account_id == changes.default_receivable_account_id
        && before.default_revenue_account_id == changes.default_revenue_account_id
        && before.default_vat_payable_account_id == changes.default_vat_payable_account_id
        && before.default_vat_recoverable_account_id == changes.default_vat_recoverable_account_id
        && before.default_vat_decompte_account_id == changes.default_vat_decompte_account_id
        && before.default_sales_journal == changes.default_sales_journal
        && before.journal_entry_description_template == changes.journal_entry_description_template
        && before.credit_note_number_format == changes.credit_note_number_format
        && before.supplier_internal_number_format == changes.supplier_internal_number_format
        && before.default_payable_account_id == changes.default_payable_account_id
        && before.default_rounding_account_id == changes.default_rounding_account_id
        && before.round_to_5_centimes == changes.round_to_5_centimes
        && before.minimum_invoice_amount == changes.minimum_invoice_amount
        && before.default_discount_account_id == changes.default_discount_account_id
        && before.default_bank_fees_account_id == changes.default_bank_fees_account_id
        && before.default_bad_debt_account_id == changes.default_bad_debt_account_id
}

/// Les dix comptes désignés, `(code, demandé, en place)`, dans l'ordre de
/// [`DESIGNATED_ACCOUNT_COLUMNS`] (Story 37-1, AC-4) — un test tient les deux
/// listes égales.
fn designated_changes(
    before: &CompanyInvoiceSettings,
    changes: &CompanyInvoiceSettingsUpdate,
) -> [(&'static str, Option<i64>, Option<i64>); 10] {
    [
        (
            "receivable",
            changes.default_receivable_account_id,
            before.default_receivable_account_id,
        ),
        (
            "revenue",
            changes.default_revenue_account_id,
            before.default_revenue_account_id,
        ),
        (
            "vatPayable",
            changes.default_vat_payable_account_id,
            before.default_vat_payable_account_id,
        ),
        (
            "vatRecoverable",
            changes.default_vat_recoverable_account_id,
            before.default_vat_recoverable_account_id,
        ),
        (
            "vatDecompte",
            changes.default_vat_decompte_account_id,
            before.default_vat_decompte_account_id,
        ),
        (
            "payable",
            changes.default_payable_account_id,
            before.default_payable_account_id,
        ),
        (
            "rounding",
            changes.default_rounding_account_id,
            before.default_rounding_account_id,
        ),
        (
            "discount",
            changes.default_discount_account_id,
            before.default_discount_account_id,
        ),
        (
            "bankFees",
            changes.default_bank_fees_account_id,
            before.default_bank_fees_account_id,
        ),
        (
            "badDebt",
            changes.default_bad_debt_account_id,
            before.default_bad_debt_account_id,
        ),
    ]
}

/// Met à jour la config (tous les champs) avec verrou optimiste et audit.
///
/// Le caller doit avoir validé les données en amont (format, comptes,
/// journal). Audit log wrapper `{before, after}`.
///
/// **Ordre des erreurs** (Story 15-6c, AC6 ; choix C-15-6-21) :
/// `DbError::OptimisticLockConflict` (version périmée, jugée contre `before`
/// lu en `FOR UPDATE`) → `DbError::ClaimAccountLinkedToBankAccount` (compte
/// débiteurs, puis compte créanciers, **changé** vers un compte lié à un compte
/// bancaire non archivé) → court-circuit no-op. Le 409 d'abord : l'exemption
/// « inchangé » se calcule contre `before`, que le client doit avoir vu.
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    expected_version: i32,
    user_id: i64,
    changes: CompanyInvoiceSettingsUpdate,
) -> Result<CompanyInvoiceSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Story 37-1 (AC-4) — `companies` en partage AVANT les réglages (ordre
    // global `companies` → réglages → `accounts`) : le réglage multi-devises
    // et les comptes de différence de la société, lus sous ce verrou, servent
    // la garde « compte de différence désigné compte de produit par défaut ».
    let currency_state =
        accounts::lock_company_currency_in_tx(&mut tx, company_id, accounts::CompanyLock::Share)
            .await?;

    // S'assurer que la row existe (création lazy si absente).
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Story 15-6c (AC2) — `before` lu en `FOR UPDATE` : la ligne des réglages
    // est verrouillée AVANT les comptes bancaires (ordre de
    // `claim_accounts_in_share_mode`), et un lien bancaire concurrent qui la
    // tient en S fait attendre ce PUT jusqu'à sa validation.
    let before = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ? FOR UPDATE"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    if before.version != expected_version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    // Story 15-6c (AC5, AC6) — après le 409 (le client voit l'état courant
    // avant qu'on juge son changement), avant le court-circuit no-op : un compte
    // de créance qui CHANGE ne peut pas être lié à un compte bancaire non
    // archivé. Le compte débiteurs d'abord, puis le compte créanciers.
    for (requested, current, claim) in [
        (
            changes.default_receivable_account_id,
            before.default_receivable_account_id,
            ClaimSide::Receivable,
        ),
        (
            changes.default_payable_account_id,
            before.default_payable_account_id,
            ClaimSide::Payable,
        ),
    ] {
        if let Err(e) =
            refuse_if_linked_to_bank_account(&mut tx, company_id, requested, current, claim).await
        {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(e);
        }
    }

    // Story 37-1 (AC-4) — un compte **changé** (motif C4), dans l'ordre de
    // `DESIGNATED_ACCOUNT_COLUMNS`, lu `LOCK IN SHARE MODE` après le
    // `FOR UPDATE` des réglages : en devise → `ACCOUNT_IN_FOREIGN_CURRENCY` ;
    // le compte de produit par défaut, compte de gains ou de pertes de change
    // (société ou comptes propres) → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`.
    for (code, requested, current) in designated_changes(&before, &changes) {
        let Some(account_id) = requested else {
            continue;
        };
        if requested == current {
            continue;
        }
        accounts::refuse_foreign_currency_in_tx(&mut tx, company_id, account_id).await?;
        if code == "revenue" {
            accounts::refuse_fx_account_as_revenue_in_tx(
                &mut tx,
                company_id,
                &currency_state,
                account_id,
            )
            .await?;
        }
    }

    // KF-004 : court-circuit no-op AVANT toute mutation.
    // NOTE concurrence (KF-004, réécrite Story 15-6c) : `before` est lu en
    // `FOR UPDATE`, si bien que ce PUT est SÉRIALISÉ contre les gestes bancaires
    // vers un compte (lien, création, remplacement — verrou S de
    // `claim_accounts_in_share_mode`) et contre les autres écrivains de la
    // ligne : une version validée entre notre BEGIN et ce point est vue, et rend
    // le 409. Ce qui ne l'est PAS : l'`INSERT IGNORE` ci-dessus pose sur une
    // ligne existante un verrou PARTAGÉ, et le `FOR UPDATE` est une promotion
    // S → X. Deux écrivains qui font tous deux `INSERT IGNORE` puis `FOR UPDATE`
    // — deux PUT, ou un PUT et `get_or_create_default_in_tx` (validation de
    // facture, avoir, facture fournisseur), ou, depuis la Story 36-19, un PUT et
    // la finalisation de l'assistant (`complete_default_accounts_in_tx`) —
    // peuvent s'interbloquer (1213). Préexistant pour les premiers (l'`UPDATE`
    // promouvait déjà), neuf pour la finalisation, borné : la route rejoue
    // l'interblocage (`retry_on_deadlock`, Story 15-5e1), et la `version` est
    // rejugée à chaque tentative.
    if is_no_op_change(&before, &changes) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }

    let rows = sqlx::query(
        "UPDATE company_invoice_settings \
         SET invoice_number_format = ?, default_receivable_account_id = ?, \
             default_revenue_account_id = ?, default_vat_payable_account_id = ?, \
             default_vat_recoverable_account_id = ?, default_vat_decompte_account_id = ?, \
             default_sales_journal = ?, \
             journal_entry_description_template = ?, credit_note_number_format = ?, \
             supplier_internal_number_format = ?, \
             default_payable_account_id = ?, default_rounding_account_id = ?, \
             round_to_5_centimes = ?, minimum_invoice_amount = ?, \
             default_discount_account_id = ?, default_bank_fees_account_id = ?, \
             default_bad_debt_account_id = ?, version = version + 1 \
         WHERE company_id = ? AND version = ?",
    )
    .bind(&changes.invoice_number_format)
    .bind(changes.default_receivable_account_id)
    .bind(changes.default_revenue_account_id)
    .bind(changes.default_vat_payable_account_id)
    .bind(changes.default_vat_recoverable_account_id)
    .bind(changes.default_vat_decompte_account_id)
    .bind(changes.default_sales_journal)
    .bind(&changes.journal_entry_description_template)
    .bind(&changes.credit_note_number_format)
    .bind(&changes.supplier_internal_number_format)
    .bind(changes.default_payable_account_id)
    .bind(changes.default_rounding_account_id)
    .bind(changes.round_to_5_centimes)
    .bind(changes.minimum_invoice_amount)
    .bind(changes.default_discount_account_id)
    .bind(changes.default_bank_fees_account_id)
    .bind(changes.default_bad_debt_account_id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let after = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let audit_details = serde_json::json!({
        "before": settings_snapshot_json(&before),
        "after": settings_snapshot_json(&after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "company_invoice_settings.updated".to_string(),
            "company_invoice_settings".to_string(),
            company_id,
            Some(audit_details),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Les comptes qu'une société reçoit d'office à la création de ses réglages,
/// parce que le plan de sa forme juridique les **marque** : le compte de
/// différences d'arrondi (`roundingDifference`, Story 25-4-c3-a2, #476) et ceux
/// des natures d'écart soldé (`writeOffNature`, Story 25-4-d1, #384).
#[derive(Debug, Default)]
struct ChartDesignatedAccounts {
    rounding: Option<i64>,
    discount: Option<i64>,
    bank_fees: Option<i64>,
    bad_debt: Option<i64>,
}

/// Résout les comptes **marqués** du plan de la société (cf.
/// [`ChartDesignatedAccounts`]).
///
/// Lu par [`complete_default_accounts_in_tx`] (et donc par ses enveloppes
/// `insert_with_defaults*`), avant la ligne des réglages. Le plan est **relu** ici
/// parce que la finalisation de l'onboarding arrive dans une autre requête que
/// l'étape qui a semé les comptes ; la forme juridique ne change plus après
/// l'étape 3, donc c'est bien le plan qui les a semés.
///
/// ⛔ **Facultatifs : une absence n'est jamais une erreur** — contrairement à la
/// créance et au produit : plan sans marqueur (l'escompte des associations),
/// compte renuméroté, archivé, rendu non imputable ou retypé depuis l'étape 4 →
/// `None`, et la société le choisira dans les paramètres. ⚠️ Une **erreur SQL**,
/// elle, remonte : elle survient dans la transaction même des recherches
/// obligatoires, qu'on ne poursuit pas dans un état douteux.
async fn chart_designated_accounts(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<ChartDesignatedAccounts, DbError> {
    use kesh_core::chart_of_accounts::{
        WriteOffNature, rounding_difference_number, write_off_account_number,
    };

    let org_type: Option<String> =
        sqlx::query_scalar("SELECT org_type FROM companies WHERE id = ?")
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
    let Some(org_type) = org_type else {
        return Ok(ChartDesignatedAccounts::default());
    };
    let chart = match kesh_core::chart_of_accounts::load_chart(&org_type) {
        Ok(chart) => chart,
        Err(e) => {
            // Une forme juridique sans plan livré est possible ; un plan livré
            // invalide ne l'est pas — ne pas l'avaler sans trace.
            tracing::warn!(company_id, org_type = %org_type, error = %e,
                "comptes désignés par le plan : plan introuvable ou invalide, réglages laissés vides");
            return Ok(ChartDesignatedAccounts::default());
        }
    };
    // Ordre de verrouillage fixe : arrondi, puis les natures dans l'ordre de
    // `WriteOffNature::ALL`.
    Ok(ChartDesignatedAccounts {
        rounding: designated_account_id(tx, company_id, rounding_difference_number(&chart)).await?,
        discount: designated_account_id(
            tx,
            company_id,
            write_off_account_number(&chart, WriteOffNature::Discount),
        )
        .await?,
        bank_fees: designated_account_id(
            tx,
            company_id,
            write_off_account_number(&chart, WriteOffNature::BankFees),
        )
        .await?,
        bad_debt: designated_account_id(
            tx,
            company_id,
            write_off_account_number(&chart, WriteOffNature::BadDebt),
        )
        .await?,
    })
}

/// Le compte de la société qui porte le numéro qu'un plan désigne, s'il est
/// utilisable par un réglage. Mêmes exigences que sa validation
/// (`validate_account_of`, `kesh-api`) : actif, imputable, charge ou produit.
///
/// `FOR UPDATE`, comme les recherches par rôle (F1) : une désactivation
/// concurrente ne doit pas laisser un réglage pointer sur un compte mort.
async fn designated_account_id(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    number: Option<&str>,
) -> Result<Option<i64>, DbError> {
    let Some(number) = number else {
        return Ok(None);
    };
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM accounts WHERE company_id = ? AND number = ? AND active = TRUE \
         AND postable = TRUE AND account_type IN (?, ?) \
         ORDER BY id LIMIT 1 FOR UPDATE",
    )
    .bind(company_id)
    .bind(number)
    .bind(AccountType::Expense)
    .bind(AccountType::Revenue)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Le réglage `round_to_5_centimes` de la société, en **lecture pure** (Story
/// 25-4-c4-b) — pour l'aperçu d'un brouillon, servi par des `GET` que rien ne
/// doit transformer en écriture, clés d'API en lecture seule comprises.
/// ⛔ Pas de `get_or_create_default` ici : son `INSERT IGNORE` ouvrirait une
/// transaction d'écriture à chaque lecture. Ligne absente → le défaut de la
/// colonne, actif.
pub async fn round_to_5_centimes<'e, E>(executor: E, company_id: i64) -> Result<bool, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT round_to_5_centimes FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)?
    .unwrap_or(true))
}

/// Le compte de différences d'arrondi, **au moment d'écrire** un écart
/// (Story 25-4-c3-b, AC 4).
///
/// Lit le réglage (`default_rounding_account_id`, c3-a1) et revérifie le compte :
/// **de la société, actif, imputable, charge ou produit** — les exigences de la
/// validation du réglage (`validate_account_of`, `kesh-api`). Le compte a pu être
/// archivé depuis sa désignation (#486) : c'est ici, et non à l'enregistrement du
/// réglage, que l'écriture est protégée.
///
/// `FOR UPDATE` sur le compte, comme le compte interne du règlement manuel : une
/// désactivation concurrente attend la fin de l'écriture au lieu de la précéder.
///
/// Absent ou invalide → [`DbError::RoundingAccountNotConfigured`], avec le
/// `context` de l'appelant (paiement ou pièce émise). ⛔ Jamais de
/// repli sur un compte déduit (produit par défaut, charges diverses).
pub async fn rounding_account_for_write(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    context: RoundingContext,
) -> Result<i64, DbError> {
    usable_designated_account(
        conn,
        company_id,
        "SELECT default_rounding_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .await?
    .ok_or(DbError::RoundingAccountNotConfigured { context })
}

/// Le compte de la **nature** d'un solde (Story 25-4-d2a, #384), relu au moment
/// d'écrire et revérifié comme le compte d'arrondi : actif, imputable, charge ou
/// produit, verrouillé `FOR UPDATE`. Le reste d'arrondi lit le compte de
/// différences d'arrondi.
///
/// Absent ou inutilisable → `WriteOffAccountNotConfigured { nature }`.
/// ⛔ Jamais de repli sur un compte déduit.
pub async fn write_off_account_for_write(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    nature: crate::entities::SettlementWriteOffNature,
) -> Result<i64, DbError> {
    use crate::entities::SettlementWriteOffNature as N;
    // Un littéral SQL complet par nature — jamais de colonne interpolée.
    let select = match nature {
        N::Discount => {
            "SELECT default_discount_account_id FROM company_invoice_settings WHERE company_id = ?"
        }
        N::BankFees => {
            "SELECT default_bank_fees_account_id FROM company_invoice_settings WHERE company_id = ?"
        }
        N::BadDebt => {
            "SELECT default_bad_debt_account_id FROM company_invoice_settings WHERE company_id = ?"
        }
        N::Rounding => {
            "SELECT default_rounding_account_id FROM company_invoice_settings WHERE company_id = ?"
        }
    };
    usable_designated_account(conn, company_id, select)
        .await?
        .ok_or(DbError::WriteOffAccountNotConfigured {
            nature: nature.as_str(),
        })
}

/// Le compte de **TVA due** d'un solde qui corrige la TVA (Story 25-4-d2a),
/// relu au moment d'écrire, de la société, `FOR UPDATE`, et jugé dans cet ordre
/// (Story 36-9, #1044, D2 — l'ordre de la validation, `check_written`) :
///
/// 1. réglage vide, compte absent, d'une autre société ou **archivé** →
///    `ConfigurationRequired` (divergence assumée avec la validation, C28 : le
///    même refus que l'avoir pour un réglage absent) ;
/// 2. **type** autre que celui du réglage
///    ([`DesignatedRole::VatPayable`]`.expected_type()`, un passif) →
///    [`DbError::DesignatedAccountsWrongType`], le refus de la validation d'une
///    facture (Story 35-3) : un compte retypé depuis sa désignation ;
/// 3. **non imputable** → `ConfigurationRequired`.
///
/// Le type et l'imputabilité sont lus dans **la** requête verrouillante : une
/// lecture séparée, non verrouillante, lirait l'instantané REPEATABLE READ de
/// l'appelant, ouvert avant le verrou, et manquerait un retypage validé
/// entre-temps (D5 de la 35-3).
pub async fn vat_payable_account_for_write(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
) -> Result<i64, DbError> {
    let missing = || DbError::ConfigurationRequired("default_vat_payable_account_id".into());
    let designated: Option<Option<i64>> = sqlx::query_scalar(
        "SELECT default_vat_payable_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let account_id = designated.flatten().ok_or_else(missing)?;
    let (id, number, postable, account_type) =
        sqlx::query_as::<_, (i64, String, bool, AccountType)>(
            "SELECT id, number, postable, account_type FROM accounts WHERE id = ? \
             AND company_id = ? AND active = TRUE FOR UPDATE",
        )
        .bind(account_id)
        .bind(company_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?
        .ok_or_else(missing)?;
    let role = DesignatedRole::VatPayable;
    let expected = role.expected_type();
    if account_type != expected {
        return Err(DbError::DesignatedAccountsWrongType(
            WrongTypeDesignatedAccounts::new([WrongTypeDesignatedAccount {
                account_id: id,
                account_number: number,
                setting: role.setting(),
                expected_type: expected,
                actual_type: account_type,
            }]),
        ));
    }
    if !postable {
        return Err(missing());
    }
    Ok(id)
}

/// Le cœur commun des relectures de compte désigné : lit le réglage par
/// `select` (un littéral de l'appelant), puis exige un compte actif, imputable,
/// de charge ou de produit, de la société, verrouillé `FOR UPDATE`. `None` si le
/// réglage est vide ou le compte inutilisable — chaque appelant pose **son**
/// erreur.
async fn usable_designated_account(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    select: &'static str,
) -> Result<Option<i64>, DbError> {
    let designated: Option<Option<i64>> = sqlx::query_scalar(select)
        .bind(company_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?;
    let Some(account_id) = designated.flatten() else {
        return Ok(None);
    };
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM accounts WHERE id = ? AND company_id = ? AND active = TRUE \
         AND postable = TRUE AND account_type IN (?, ?) FOR UPDATE",
    )
    .bind(account_id)
    .bind(company_id)
    .bind(AccountType::Expense)
    .bind(AccountType::Revenue)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Story 15-5d (#429) — la garde À L'USAGE des comptes de réglage
// ---------------------------------------------------------------------------

/// Un **champ** des réglages de facturation qu'un générateur d'écriture a
/// effectivement écrit (Story 15-5d, choix C39, C44).
///
/// Type neuf plutôt qu'[`AccountRole`] (C44) : il désigne un champ des
/// réglages, non le rôle que le plan attribue au compte (un compte désigné ne
/// porte pas forcément ce rôle) ; ses quatre variantes se traduisent en champs
/// par un `match` exhaustif ([`DesignatedRole::designated_id`]) ; et il dérive
/// `Ord`, qu'exige le `BTreeSet` de [`GeneratedLines`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::repositories) enum DesignatedRole {
    /// `default_receivable_account_id` — la créance de la facture de vente.
    Receivable,
    /// `default_vat_payable_account_id` — la TVA due de la facture de vente.
    VatPayable,
    /// `default_payable_account_id` — le compte créanciers de la facture
    /// fournisseur.
    Payable,
    /// `default_vat_recoverable_account_id` — la TVA récupérable (impôt
    /// préalable) de la facture fournisseur.
    VatRecoverable,
}

impl DesignatedRole {
    /// Les rôles qu'une **validation de facture** peut écrire : ses candidats.
    pub(in crate::repositories) const SALE: [DesignatedRole; 2] =
        [DesignatedRole::Receivable, DesignatedRole::VatPayable];
    /// Les rôles qu'une **saisie de facture fournisseur** peut écrire.
    pub(in crate::repositories) const PURCHASE: [DesignatedRole; 2] =
        [DesignatedRole::Payable, DesignatedRole::VatRecoverable];

    /// L'identifiant que les réglages désignent pour ce rôle (`None` : champ
    /// vide).
    pub(in crate::repositories) fn designated_id(
        self,
        settings: &CompanyInvoiceSettings,
    ) -> Option<i64> {
        match self {
            DesignatedRole::Receivable => settings.default_receivable_account_id,
            DesignatedRole::VatPayable => settings.default_vat_payable_account_id,
            DesignatedRole::Payable => settings.default_payable_account_id,
            DesignatedRole::VatRecoverable => settings.default_vat_recoverable_account_id,
        }
    }

    /// Le type que le **réglage** exige (Story 35-3, #537, D4) : lu dans les
    /// deux constantes publiques que lit aussi la route de désignation —
    /// [`CLAIM_SETTING_ACCOUNT_TYPES`] pour la créance et les créanciers,
    /// [`VAT_SETTING_ACCOUNT_TYPES`] pour la TVA due et la TVA récupérable.
    pub(in crate::repositories) fn expected_type(self) -> AccountType {
        match self {
            DesignatedRole::Receivable => CLAIM_SETTING_ACCOUNT_TYPES.receivable,
            DesignatedRole::VatPayable => VAT_SETTING_ACCOUNT_TYPES.payable,
            DesignatedRole::Payable => CLAIM_SETTING_ACCOUNT_TYPES.payable,
            DesignatedRole::VatRecoverable => VAT_SETTING_ACCOUNT_TYPES.recoverable,
        }
    }

    /// Le réglage, sous sa forme publique ([`DesignatedSetting`]), que nomme le
    /// refus [`DbError::DesignatedAccountsWrongType`].
    pub(in crate::repositories) fn setting(self) -> DesignatedSetting {
        match self {
            DesignatedRole::Receivable => DesignatedSetting::Receivable,
            DesignatedRole::VatPayable => DesignatedSetting::VatPayable,
            DesignatedRole::Payable => DesignatedSetting::Payable,
            DesignatedRole::VatRecoverable => DesignatedSetting::VatRecoverable,
        }
    }

    /// Les identifiants désignés pour `roles` — les **candidats** d'un flux,
    /// qu'il les écrive ou non ; un champ vide n'y entre pas.
    pub(in crate::repositories) fn candidate_ids(
        roles: &[DesignatedRole],
        settings: &CompanyInvoiceSettings,
    ) -> Vec<i64> {
        roles
            .iter()
            .filter_map(|r| r.designated_id(settings))
            .collect()
    }
}

/// Les lignes d'une écriture générée, **avec** les rôles de réglage qu'elle
/// écrit effectivement (Story 15-5d, choix C39, C50).
///
/// La garde ne contrôle que les comptes de ces rôles : aucun recalcul de la
/// TVA hors du générateur, aucune inspection des `account_id` des lignes —
/// ambiguë dès qu'un même compte joue deux rôles, ou qu'un compte de réglage
/// coïncide avec un compte de produit ou de charge d'une ligne.
#[derive(Debug)]
pub(in crate::repositories) struct GeneratedLines {
    /// Les lignes de l'écriture, telles que `journal_entries::create_in_tx` les
    /// attend.
    pub lines: Vec<crate::entities::NewJournalEntryLine>,
    /// Les rôles de réglage effectivement écrits par ces lignes.
    pub roles: BTreeSet<DesignatedRole>,
}

/// Un compte tel que l'a lu le verrou de [`lock_designated_accounts_in_tx`].
///
/// Public depuis la Story 15-6a (choix C-15-6-36, C-15-6a-2) : l'avoir lit ses
/// lignes, et un test d'intégration appelle le helper directement. **Un seul
/// type** de ligne pour le verrou d'une liste de comptes.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LockedDesignatedAccount {
    pub id: i64,
    pub number: String,
    pub active: bool,
    pub postable: bool,
    /// Le type, lu dans la même requête verrouillante (Story 35-3, #537, D5) :
    /// la garde à l'usage le compare au type qu'exige le réglage. L'avoir, qui
    /// lit aussi ces lignes, ne s'en sert pas.
    pub account_type: AccountType,
}

/// L'instantané **verrouillé** des comptes candidats d'un flux, rendu par
/// [`lock_designated_accounts_in_tx`] ; ses lignes restent verrouillées (en
/// partagé) jusqu'au commit, il n'y a donc rien à relire pour le contrôle.
///
/// Son champ reste privé : un instantané ne se fabrique que par le verrou.
#[derive(Debug)]
pub struct DesignatedAccountsSnapshot(Vec<LockedDesignatedAccount>);

/// **Premier temps** de la garde à l'usage des comptes de réglage (Story 15-5d,
/// #429 ; choix C27, C43, C49, C51, C87, C88) : verrouille **tous les comptes
/// candidats** du flux et rend leur instantané, **sans rien refuser**.
///
/// Les candidats sont les comptes que les réglages désignent pour les rôles que
/// le flux **peut** écrire, qu'il les écrive ou non ([`DesignatedRole::SALE`],
/// [`DesignatedRole::PURCHASE`]) : la génération, qui dira lesquels sont
/// effectivement écrits, vient après l'exercice, et le verrou doit le précéder.
/// Verrouiller un candidat que le générateur n'écrira pas (TVA due d'une
/// facture sans TVA) coûte un verrou partagé superflu, jamais un refus. Le
/// **second temps** est [`DesignatedAccountsSnapshot::check_written`].
///
/// **Seul helper de verrou d'une liste de comptes** (Story 15-6a, choix
/// C-15-6-32, C-15-6-35, C-15-6a-2) : l'avoir l'emploie aussi, pour verrouiller
/// en partagé tous les comptes qu'il écrit, et fait lui-même ses contrôles
/// ([`super::credit_notes::create_credit_note`]). D'où sa visibilité publique et
/// celle de [`LockedDesignatedAccount`].
///
/// # Le verrou : partagé, une requête, `ORDER BY id`
///
/// `SELECT id, number, active, postable, account_type FROM accounts FORCE INDEX
/// (PRIMARY) WHERE company_id = ? AND id IN (…) ORDER BY id LOCK IN SHARE MODE`
/// (syntaxe MariaDB 10.11 : pas de `FOR SHARE`). L'ordre de verrouillage entre les comptes est celui des
/// identifiants. Le type (`account_type`, en dernière colonne, Story 35-3) est
/// lu dans **cette** requête : une lecture séparée, non verrouillante, lirait
/// l'instantané REPEATABLE READ de l'appelant, ouvert avant le verrou, et
/// manquerait un retypage validé entre-temps.
///
/// **Pourquoi partagé suffit** (C87) : le verrou n'a qu'un but, qu'aucun
/// archivage, passage à non imputable ni **retypage** (Story 35-3, #537) ne
/// s'insère entre le contrôle et l'insertion de l'écriture. Or ces gestes
/// **écrivent** la ligne du compte —
/// `accounts::update` (case *imputable*, rôle, retypage), `accounts::archive`,
/// la création d'un sous-compte (`UPDATE accounts SET postable = FALSE` sur le
/// parent) — et tout `UPDATE` prend un verrou **exclusif** de ligne, qui attend
/// la fin d'une transaction tenant un verrou partagé. Une lecture verrouillante,
/// partagée comme exclusive, lit la **dernière version validée** (et attend un
/// `UPDATE` non encore validé), non l'instantané de la transaction.
///
/// **Pourquoi pas `FOR UPDATE`** : il n'ajouterait rien à cette garantie, et il
/// formait un cycle **systématique** avec les flux qui prennent l'exercice puis
/// reprennent la créance, la TVA due ou les créanciers en **partagé** par la clé
/// étrangère `fk_jel_account` (règlement client, solde du reste, rapprochement,
/// avoir, règlement fournisseur) — finding F5-1. Deux partagés sont
/// compatibles. Les cycles qui **restent** sont rejoués par les routes (Story
/// 15-5e1 ; règle au doc-comment de
/// [`super::invoices::validate_invoice`]).
///
/// # Un identifiant d'une autre société n'est jamais verrouillé
///
/// Une lecture verrouillante par clé primaire verrouille la ligne **avant** que
/// le filtre `company_id` ne l'écarte (mesuré : `opening_complement.rs`, revue
/// de code P1 de la story du complément d'ouverture, B-F1 ; remesuré en T0 de
/// la Story 15-5d). D'où le patron `owned_account_ids` (C88) : une lecture
/// **non verrouillante** des identifiants de la société — un compte ne change
/// jamais de société, elle est exacte sans verrou —, puis la lecture
/// verrouillante sur ces **seuls** identifiants, le filtre `company_id` gardé en
/// défense. Un identifiant écarté est absent de l'instantané, et le contrôle le
/// refuse comme tel.
///
/// **Angle mort assumé** (revue de code P1, E4 ; C-15-5d-6) : cette lecture se
/// fait dans l'instantané REPEATABLE READ de l'appelant, ouvert avant le verrou
/// des réglages. Un compte **créé et désigné** par un enregistrement des
/// réglages entre ces deux instants en est absent, et la garde le refuse
/// (`InactiveOrInvalidAccounts`) bien qu'il soit valide. Le refus est sûr
/// (rien n'est écrit) et un nouvel essai passe.
///
/// # Verrous d'intervalle
///
/// En REPEATABLE READ, une lecture verrouillante qu'InnoDB parcourt **par
/// plage** pose des verrous *next-key* ; une recherche d'égalité sur la clé
/// primaire ne verrouille que la ligne trouvée. Le plan de cette requête est
/// `range` sur `PRIMARY` (`const` pour un seul identifiant), sans `filesort` :
/// une liste `IN` de clés primaires, que MariaDB 10.11 lit point par point — un
/// compte voisin, de la société ou non, reste libre (mesuré en T0 de la Story
/// 15-5d par une sonde `FOR UPDATE NOWAIT`). Ce plan est **épinglé** par
/// `FORCE INDEX (PRIMARY)` (revue de code P1, B-1) : sur une table réelle,
/// l'optimiseur pourrait sinon choisir un index secondaire commençant par
/// `company_id` et y poser des verrous *next-key*, qui bloqueraient la création
/// d'un compte de la société pendant une validation. Précédent qui a écarté `LOCK IN
/// SHARE MODE` sur un parcours de plage : `journal_entries.rs`, C-15-8-23.
///
/// # Ce qui n'est pas contrôlé ici
///
/// - le **type** du compte est lu ici mais **jugé** au second temps
///   ([`DesignatedAccountsSnapshot::check_written`], Story 35-3, #537) : cet
///   angle mort, assumé depuis la 15-5d — un compte désigné retypé après sa
///   désignation restait utilisé —, est fermé pour les quatre comptes de cette
///   garde. Le refus vient **après** le verrou : un compte désigné retypé en
///   charge et imputé par une ligne prend toujours ses deux verrous avant d'être
///   refusé (le compte d'arrondi, lui, contrôle son type dans son propre
///   accesseur) ;
/// - le **compte bancaire** (D-A0, C6), le **compte de produit par défaut**
///   (D3-bis de la 16-1a), le compte de **décompte TVA** (lu par aucun flux
///   d'écriture) ;
/// - **l'avoir**, délibérément (C35) : il emploie ce helper pour son propre
///   verrou (Story 15-6a), mais ne contrôle que l'état **actif** de ce qu'il
///   écrit — la créance et l'arrondi lus **sur l'écriture de vente** (#473,
///   #523), la TVA due relue dans les réglages (#525, angle mort tracé), les
///   comptes de produit — et jamais `postable` : une contre-passation vise les
///   mêmes comptes que l'origine ;
/// - le **solde du reste** garde son refus `ConfigurationRequired` pour une TVA
///   due absente, archivée ou non imputable ([`vat_payable_account_for_write`])
///   — divergence assumée (C28) ; son **type**, en revanche, y est contrôlé
///   comme ici, par la même constante et le même refus
///   `DesignatedAccountsWrongType`, avant l'imputabilité (Story 36-9, #1044).
///
/// `journal_entries::create_in_tx` garde `enforce_postable = false` (D-A0) : la
/// garde est en amont, sur les seuls comptes de réglage. Elle révise la limite
/// **L2** de D-A0 (`14-3b-consommateurs-roles.md`) pour ces quatre comptes.
pub async fn lock_designated_accounts_in_tx(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    ids: &[i64],
) -> Result<DesignatedAccountsSnapshot, DbError> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Ok(DesignatedAccountsSnapshot(Vec::new()));
    }

    // Les identifiants de la société, lus SANS verrou (patron `owned_account_ids`,
    // C88) : un identifiant étranger n'est jamais verrouillé.
    let mut qb: QueryBuilder<MySql> =
        QueryBuilder::new("SELECT id FROM accounts WHERE company_id = ");
    qb.push_bind(company_id).push(" AND id IN (");
    {
        let mut sep = qb.separated(", ");
        for id in &ids {
            sep.push_bind(*id);
        }
    }
    qb.push(") ORDER BY id");
    let owned: Vec<i64> = qb
        .build_query_scalar()
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    if owned.is_empty() {
        return Ok(DesignatedAccountsSnapshot(Vec::new()));
    }

    // Le verrou PARTAGÉ, sur les seuls identifiants de la société, par ordre
    // d'identifiant ; le filtre `company_id` reste en défense.
    // `FORCE INDEX (PRIMARY)` épingle le plan (revue de code P1, B-1) : sans lui,
    // l'optimiseur pourrait parcourir un index secondaire commençant par
    // `company_id` (`uq_accounts_company_number`,
    // `uq_accounts_company_singleton_role`) et y poser des verrous d'intervalle.
    let mut qb: QueryBuilder<MySql> = QueryBuilder::new(
        "SELECT id, number, active, postable, account_type FROM accounts FORCE INDEX (PRIMARY) WHERE company_id = ",
    );
    qb.push_bind(company_id).push(" AND id IN (");
    {
        let mut sep = qb.separated(", ");
        for id in &owned {
            sep.push_bind(*id);
        }
    }
    qb.push(") ORDER BY id LOCK IN SHARE MODE");
    let locked: Vec<LockedDesignatedAccount> = qb
        .build_query_as()
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    Ok(DesignatedAccountsSnapshot(locked))
}

impl DesignatedAccountsSnapshot {
    /// Les lignes verrouillées, **toutes** — archivées comprises —, par ordre
    /// d'identifiant (Story 15-6a : l'avoir les partage lui-même entre son refus
    /// nommé et la 6 ter). Un identifiant demandé qui n'est pas de la société
    /// en est absent.
    pub fn accounts(&self) -> &[LockedDesignatedAccount] {
        &self.0
    }

    /// **Second temps** de la garde (Story 15-5d ; choix C39, C40, C49) :
    /// contrôle, sur l'instantané verrouillé, **les seuls comptes des rôles
    /// effectivement écrits** par le générateur (`roles`), traduits en
    /// identifiants par les réglages qui ont servi à la génération.
    ///
    /// Appelé **entre** la génération des lignes et
    /// `journal_entries::create_in_tx`. Parcourt **tous** les rôles écrits en
    /// accumulant les fautes, puis refuse, dans cet ordre de priorité :
    ///
    /// 1. un compte d'un rôle écrit **absent de l'instantané** (inexistant, ou
    ///    d'une autre société) **ou inactif** (archivé) →
    ///    [`DbError::InactiveOrInvalidAccounts`] — le refus que `create_in_tx`
    ///    rend pour ce cas, sans nommer le compte (anti-énumération) ;
    /// 2. sinon, les comptes de la société, actifs, dont le **type** n'est pas
    ///    celui que leur réglage exige ([`DesignatedRole::expected_type`] :
    ///    un compte retypé après sa désignation, Story 35-3, #537) →
    ///    [`DbError::DesignatedAccountsWrongType`], un seul refus qui les nomme
    ///    tous, par numéro ([`WrongTypeDesignatedAccounts::new`]). Un compte à la
    ///    fois retypé et non imputable n'est nommé que là ;
    /// 3. sinon, les comptes **non imputables** →
    ///    [`DbError::DesignatedAccountsNotPostable`], un seul refus qui les nomme
    ///    tous. La liste est construite par [`NonPostableAccounts::new`], qui
    ///    **dédoublonne par identifiant** : un même compte désigné pour deux
    ///    rôles est nommé une fois (C40). Depuis la Story 35-3, ce cas ne
    ///    s'atteint plus que par un montage incohérent : dans un même flux, les
    ///    deux rôles attendent des types opposés (créance actif et TVA due
    ///    passif ; créanciers passif et TVA récupérable actif), si bien qu'un
    ///    compte commun est d'abord refusé pour son type.
    ///
    /// Pourquoi accumuler et non rendre au premier compte fautif : un retour
    /// anticipé sur le type masquerait un compte archivé visité plus tard dans
    /// l'ordre `Receivable < VatPayable < Payable < VatRecoverable` — et
    /// l'archivé doit primer.
    ///
    /// Pourquoi ici et non le contrôle de `create_in_tx` : ce dernier est une
    /// lecture **non verrouillante**, qui lit l'instantané REPEATABLE READ ouvert
    /// avant le verrou ; un compte archivé entre les deux y paraîtrait encore
    /// actif. L'instantané de [`lock_designated_accounts_in_tx`], lui, est frais.
    pub(in crate::repositories) fn check_written(
        &self,
        roles: &BTreeSet<DesignatedRole>,
        settings: &CompanyInvoiceSettings,
    ) -> Result<(), DbError> {
        let mut wrong_type: Vec<WrongTypeDesignatedAccount> = Vec::new();
        let mut non_postable: Vec<NonPostableAccount> = Vec::new();
        for role in roles {
            // Un rôle écrit a forcément un compte : le générateur a refusé
            // `ConfigurationRequired` sinon.
            let id = role.designated_id(settings).ok_or_else(|| {
                DbError::Invariant(format!(
                    "rôle de réglage {role:?} écrit par le générateur sans compte désigné"
                ))
            })?;
            let expected = role.expected_type();
            match self.0.iter().find(|a| a.id == id) {
                // Absent ou archivé : la cause la plus forte, rien ne la masque.
                Some(a) if !a.active => return Err(DbError::InactiveOrInvalidAccounts),
                None => return Err(DbError::InactiveOrInvalidAccounts),
                Some(a) if a.account_type != expected => {
                    wrong_type.push(WrongTypeDesignatedAccount {
                        account_id: a.id,
                        account_number: a.number.clone(),
                        setting: role.setting(),
                        expected_type: expected,
                        actual_type: a.account_type,
                    })
                }
                Some(a) if !a.postable => non_postable.push(NonPostableAccount {
                    account_id: a.id,
                    account_number: a.number.clone(),
                }),
                Some(_) => {}
            }
        }
        if !wrong_type.is_empty() {
            return Err(DbError::DesignatedAccountsWrongType(
                WrongTypeDesignatedAccounts::new(wrong_type),
            ));
        }
        if non_postable.is_empty() {
            Ok(())
        } else {
            Err(DbError::DesignatedAccountsNotPostable(
                NonPostableAccounts::new(non_postable),
            ))
        }
    }
}

/// Creates company_invoice_settings with auto-prefill of default accounts resolved
/// by role (`Receivable`, `DefaultRevenue`, `Payable` — Story 14-3b : no longer by
/// hardcoded account number). Called during onboarding finalization (after chart of
/// accounts is loaded).
///
/// **Pool-level variant** (`insert_with_defaults`): Opens its own transaction.
/// `seed_demo` (Path A) ne l'appelle plus depuis la Story 15-7b1 : il emploie la
/// variante `_in_tx` dans sa dernière transaction.
///
/// **Transaction-level variant** (`insert_with_defaults_in_tx`): Works within caller's transaction.
/// Used by the demonstration and the trap sets. *(Story 36-19 : `finalize()` (Path B)
/// appelle directement [`complete_default_accounts_in_tx`], qui porte le corps et
/// rend le refus détaillé ; les deux `insert_with_defaults*` en sont les enveloppes
/// et le ramènent à `InactiveOrInvalidAccounts`.)*
///
/// **Délégation (Story 15-7a1)** : la variante pool n'a plus de corps propre —
/// elle **appelle** [`insert_with_defaults_in_tx`] (`begin`, variante, `commit`) ;
/// sur erreur, rollback *best-effort* puis l'**erreur d'origine rendue telle
/// quelle**. *(Story 15-7b1 : la boucle de retry de `seed_demo` reconnaît
/// désormais la variante rendue par `_in_tx`, dans sa dernière transaction ; ce
/// rejeu se justifiait par la visibilité entre transactions — le plan commité
/// hors verrou. Sous les lectures verrouillantes de `_in_tx`, qui lisent le
/// dernier état commité, `InactiveOrInvalidAccounts` y est permanent : rejeu
/// conservé par prudence, sans effet attendu.)* La duplication et ses marqueurs
/// `MIRROR` ont disparu.
///
/// Résolution des comptes par défaut **par rôle** (Story 14-3b, chantier C) et
/// non plus par numéro codé en dur — le plan comptable reste celui de
/// l'utilisateur (il peut renuméroter). Mapping rôle → compte :
/// - `AccountRole::Receivable` → créances clients (ex-`1100`)
/// - `AccountRole::DefaultRevenue` → produit de facturation par défaut (ex-`3000`)
/// - `AccountRole::Payable` → dettes fournisseurs, optionnel (ex-`2000`)
///
/// Ces trois rôles sont **singleton** : au plus un compte actif par société les
/// porte. Les lookups interrogent la colonne générée `singleton_role` (= le rôle
/// si le compte est actif ET le rôle singleton, NULL sinon) plutôt que la colonne
/// brute `role` : cela exploite directement l'index `uq_accounts_company_singleton_role`
/// (lookup `const` O(1), l'intention documentée de la migration 14-3a) ET encode
/// déjà le filtre `active` (la colonne générée est `active`-aware — inutile de
/// répéter `AND active = true`). `ORDER BY id LIMIT 1` devient sémantiquement
/// redondant (l'unicité garantit ≤ 1 ligne) mais est conservé (cohérence avec
/// `FOR UPDATE`). *(Story 14-3b — ECH code-review pass 2 : `role = ?` scannait
/// l'index, `singleton_role = ?` restaure l'accès `const`.)*
///
/// Template placeholders ({YEAR}, {SEQ:04}, {INVOICE_NUMBER}) are literal database values,
/// not Rust format strings - braces are intentionally single (not {{escaped}}).
///
/// F1 CRITICAL FIX: Lookups use SELECT FOR UPDATE to lock accounts rows,
/// preventing concurrent deletes from creating dangling FKs. Atomic with INSERT.
///
/// F2/F3/F4 CRITICAL FIX: Transaction-level variant keeps account locks within finalize()'s
/// transaction, preserving company and onboarding_state locks until step update completes.
pub async fn insert_with_defaults(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    match insert_with_defaults_in_tx(&mut tx, company_id).await {
        Ok((settings, _inserted)) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(settings)
        }
        Err(e) => {
            // P6-M2: rollback is best-effort. Propagating a rollback error here
            // would hide the original error (InactiveOrInvalidAccounts among
            // them) behind a transient one.
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Transaction-aware variant of insert_with_defaults — **l'enveloppe** de
/// [`complete_default_accounts_in_tx`] (Story 36-19, #1200, D5), que la
/// démonstration (`kesh-seed`) et les jeux de pièges appellent.
///
/// Rend `(réglages, posés)` (Story 15-7a1 ; Story 36-19, revue P1 B1/E5) : le
/// booléen vaut `true` si **cet appel a posé les comptes d'une ligne qui n'en
/// avait aucun** — la ligne créée par cet appel, **ou** une ligne **vierge**
/// ([`is_blank`] : créée avant par `get_or_create_default`, le `GET` des
/// réglages) qu'il vient de compléter — ; `false` si la ligne portait déjà des
/// réglages (complétée ou non). C'est sur ce booléen que la démonstration et les
/// jeux de pièges désignent les trois comptes de TVA
/// (`designate_vat_accounts_in_tx`, `COALESCE`, dans la même transaction) : une
/// démonstration précédée d'un `GET` reçoit donc ses comptes de TVA comme sur
/// une base neuve. Avant la Story 36-19, une telle ligne vierge faisait refuser
/// la démonstration (`InactiveOrInvalidAccounts`) ; désormais elle aboutit, la
/// ligne en `version` 2 (complément d'une ligne existante).
///
/// Le refus détaillé [`DbError::DefaultInvoiceAccountsUnavailable`] est
/// **ramené** à [`DbError::InactiveOrInvalidAccounts`] (boucle de rejeu de la
/// démonstration et tests d'avant la Story 36-19). **Ne commite ni n'annule
/// jamais** : sur erreur, l'appelant annule.
///
/// F2/F3/F4 CRITICAL: Caller must hold SELECT FOR UPDATE lock on onboarding_state
/// and company to prevent deletion races. This function's SELECT FOR UPDATE on accounts
/// complements the higher-level locks, ensuring deterministic ordering of account lookups.
pub async fn insert_with_defaults_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<(CompanyInvoiceSettings, bool), DbError> {
    match complete_default_accounts_in_tx(tx, company_id).await {
        Ok(outcome) => {
            let posed = outcome.posed_on_empty_row();
            Ok((outcome.into_settings(), posed))
        }
        Err(DbError::DefaultInvoiceAccountsUnavailable(_)) => {
            Err(DbError::InactiveOrInvalidAccounts)
        }
        Err(e) => Err(e),
    }
}

/// Ce que [`complete_default_accounts_in_tx`] a fait de la ligne des réglages
/// (Story 36-19, #1200).
#[derive(Debug, Clone)]
pub enum DefaultAccountsOutcome {
    /// La ligne n'existait pas : elle est créée, comptes posés, `version` 1.
    Inserted(CompanyInvoiceSettings),
    /// La ligne existait : des comptes vides ont été complétés, `version` + 1.
    Completed {
        /// La ligne avant le complément, lue sous verrou (en boîte : deux
        /// réglages dans une variante, `clippy::large_enum_variant`).
        before: Box<CompanyInvoiceSettings>,
        /// La ligne relue après le complément.
        after: CompanyInvoiceSettings,
    },
    /// La ligne existait et rien n'était à compléter : aucune écriture.
    Unchanged(CompanyInvoiceSettings),
}

impl DefaultAccountsOutcome {
    /// Les réglages tels qu'ils sont après l'appel.
    pub fn settings(&self) -> &CompanyInvoiceSettings {
        match self {
            Self::Inserted(s) | Self::Unchanged(s) => s,
            Self::Completed { after, .. } => after,
        }
    }

    /// `true` si la ligne a été créée par cet appel.
    pub fn inserted(&self) -> bool {
        matches!(self, Self::Inserted(_))
    }

    /// `true` si cet appel a posé les comptes d'une ligne qui n'en avait aucun :
    /// ligne créée, ou ligne **vierge** complétée (revue P1, B1/E5) — ce que
    /// l'enveloppe [`insert_with_defaults_in_tx`] rend à la démonstration.
    pub fn posed_on_empty_row(&self) -> bool {
        match self {
            Self::Inserted(_) => true,
            Self::Completed { before, .. } => is_blank(before),
            Self::Unchanged(_) => false,
        }
    }

    /// Les réglages tels qu'ils sont après l'appel, par valeur.
    pub fn into_settings(self) -> CompanyInvoiceSettings {
        match self {
            Self::Inserted(s) | Self::Unchanged(s) => s,
            Self::Completed { after, .. } => after,
        }
    }
}

/// Une ligne de réglages **vierge** (Story 36-19, D2) : `version` 1 et ses dix
/// comptes vides — l'état exact que laisse `INSERT IGNORE … (company_id)` (le
/// `GET` des réglages, `get_or_create_default`), et qu'un `PUT` qui change
/// quelque chose quitte (`update` incrémente `version`). Une ligne de `version`
/// 2 aux dix comptes vides est **enregistrée**, non vierge : l'utilisateur a
/// enregistré l'écran avec ses comptes vides.
fn is_blank(s: &CompanyInvoiceSettings) -> bool {
    s.version == 1
        && s.default_receivable_account_id.is_none()
        && s.default_revenue_account_id.is_none()
        && s.default_vat_payable_account_id.is_none()
        && s.default_vat_recoverable_account_id.is_none()
        && s.default_vat_decompte_account_id.is_none()
        && s.default_payable_account_id.is_none()
        && s.default_rounding_account_id.is_none()
        && s.default_discount_account_id.is_none()
        && s.default_bank_fees_account_id.is_none()
        && s.default_bad_debt_account_id.is_none()
}

/// Pourquoi le compte qui porte un rôle ne peut pas être désigné d'office
/// (Story 36-19, D6), `None` s'il le peut.
///
/// ⛔ **Lecture verrouillante** (validation P3, V3-1 ; C-36-19-9) : `number` et
/// `postable` se relisent `FOR UPDATE` sur la ligne que
/// [`super::accounts::find_singleton_role_in_tx`] a déjà verrouillée (aucun
/// verrou neuf). Une lecture simple lirait, sous `REPEATABLE READ`, le cliché de
/// la première lecture non verrouillante de la transaction : un sous-compte créé
/// entre ce cliché et le verrou laisserait le candidat lu imputable.
///
/// `bank_guard` : le compte débiteurs et le compte créanciers ne peuvent pas être
/// le compte du grand livre d'un compte bancaire non archivé (Story 15-6c ; refs
/// #987) — lu par `bank_accounts::first_active_bank_account_linked_to`, verrou
/// partagé, **après** la ligne des réglages (ordre réglages → `bank_accounts`).
async fn role_candidate_rejection(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    account_id: i64,
    bank_guard: bool,
) -> Result<Option<DefaultInvoiceAccountCause>, DbError> {
    let (account_number, postable): (String, bool) = sqlx::query_as(
        "SELECT number, postable FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if !postable {
        return Ok(Some(DefaultInvoiceAccountCause::RoleAccountNotPostable {
            account_number,
        }));
    }
    if bank_guard
        && let Some((_, bank_label)) =
            super::bank_accounts::first_active_bank_account_linked_to(tx, company_id, account_id)
                .await?
    {
        return Ok(Some(
            DefaultInvoiceAccountCause::RoleAccountLinkedToBankAccount {
                account_number,
                bank_label,
            },
        ));
    }
    Ok(None)
}

/// Le compte à poser dans un réglage **obligatoire** (débiteurs, produit) —
/// `None` s'il n'y a rien à poser (réglage déjà désigné, ou refus inscrit dans
/// `unavailable`).
///
/// - Réglage **désigné** : gardé (D3 : le rôle n'est pas cherché), pourvu que le
///   compte soit encore de la société et actif — relu `LOCK IN SHARE MODE`
///   après la ligne des réglages (C-36-19-10) ; sinon cause « archivé », ou
///   « pas un compte de la société » s'il n'est pas de la société. Il n'est pas
///   recontrôlé autrement (le `PUT` l'a contrôlé ; la garde à l'usage reste le
///   filet). ⛔ La lecture est **verrouillante** : dans la route, le cliché
///   `REPEATABLE READ` de la transaction est pris avant (par la lecture simple
///   de `companies.org_type`, `chart_designated_accounts`), et un archivage
///   validé entre ce cliché et cette lecture serait invisible à une lecture
///   simple (revue P1, E2 ; test T-D4a, mutation M15).
/// - Réglage **vide** : le compte de rôle, s'il existe et n'est pas écarté par
///   [`role_candidate_rejection`] ; sinon la cause.
async fn mandatory_account(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    setting: DefaultInvoiceSetting,
    designated: Option<i64>,
    role_account: Option<i64>,
    unavailable: &mut Vec<UnavailableDefaultInvoiceAccount>,
) -> Result<Option<i64>, DbError> {
    let cause = match (designated, role_account) {
        (Some(account_id), _) => {
            let row: Option<(String, bool)> = sqlx::query_as(
                "SELECT number, active FROM accounts WHERE id = ? AND company_id = ? \
                 LOCK IN SHARE MODE",
            )
            .bind(account_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
            match row {
                Some((_, true)) => return Ok(None),
                Some((number, false)) => DefaultInvoiceAccountCause::DesignatedArchived {
                    account_number: number,
                },
                None => DefaultInvoiceAccountCause::DesignatedOutsideCompany,
            }
        }
        (None, None) => DefaultInvoiceAccountCause::NoRoleAccount,
        (None, Some(account_id)) => {
            let bank_guard = setting == DefaultInvoiceSetting::Receivable;
            match role_candidate_rejection(tx, company_id, account_id, bank_guard).await? {
                None => return Ok(Some(account_id)),
                Some(cause) => cause,
            }
        }
    };
    unavailable.push(UnavailableDefaultInvoiceAccount { setting, cause });
    Ok(None)
}

/// Crée ou **complète** la ligne des réglages de facturation de la finalisation
/// de l'assistant (Story 36-19, #1200, D1 à D7 ; refs #987) — la seule fonction
/// qui pose les comptes par défaut que le plan fournit.
///
/// Avant la Story 36-19, la finalisation (`insert_with_defaults_in_tx`) écrivait
/// par `INSERT IGNORE` : sans effet sur une ligne déjà créée par un `GET` des
/// réglages (journal, comptes bancaires, factures ouverts avant la fin de
/// l'assistant), puis refusée faute de comptes. Elle **complète** désormais la
/// ligne qu'elle trouve.
///
/// **Ce qui se complète** (D2, C-36-19-1) — une valeur non vide n'est **jamais**
/// remplacée (`COALESCE`) :
/// - débiteurs et produit (obligatoires) : par leur compte de rôle dès qu'ils
///   sont vides ; désignés, ils sont gardés s'ils sont actifs (D3) ;
/// - créanciers (rôle `Payable`), arrondi, escompte, frais bancaires, pertes
///   (facultatifs) : seulement sur une ligne **vierge** ([`is_blank`]) ;
/// - les trois comptes de TVA : jamais (Story 31-5).
///
/// **Un candidat invalide n'est jamais désigné** (D6) : compte de rôle non
/// imputable, ou — débiteurs et créanciers — lié à un compte bancaire non
/// archivé. Obligatoire → refus ; facultatif → réglage laissé vide, sans refus.
/// Les comptes du plan sont déjà filtrés (`designated_account_id`).
///
/// **Refus** : [`DbError::DefaultInvoiceAccountsUnavailable`], un élément par
/// réglage obligatoire en défaut, débiteurs d'abord, après l'examen des deux.
/// Rien n'est écrit qui ne soit annulé par le `rollback` de l'appelant (la ligne
/// neuve comprise).
///
/// **Un seul ordre, ligne neuve comme existante** (D4, C-36-19-7) :
/// 1. les comptes candidats `FOR UPDATE` — les trois rôles, puis les comptes que
///    le plan marque (`chart_designated_accounts`) : `accounts` avant
///    `company_invoice_settings` ;
/// 2. `INSERT IGNORE INTO company_invoice_settings (company_id)` nu — les
///    colonnes de texte prennent leurs valeurs par défaut, celles qu'écrivait
///    l'`INSERT` d'avant ; `rows_affected == 1` dit « ligne neuve » ;
/// 3. la ligne `FOR UPDATE` ;
/// 4. les contrôles : comptes déjà désignés (`LOCK IN SHARE MODE`), candidats
///    (relus `FOR UPDATE`, déjà tenus), comptes bancaires (`LOCK IN SHARE MODE`)
///    — réglages avant `bank_accounts`, l'ordre du geste bancaire
///    (`claim_accounts_in_share_mode`) ;
/// 5. **un seul** `UPDATE … SET col = COALESCE(col, ?)` si au moins une colonne
///    change, avec `version + 1` **seulement si la ligne existait** (une ligne
///    neuve n'a été lue par personne : elle garde `version` 1). Sinon aucune
///    écriture : rejouée ou appelée deux fois, la fonction rend « inchangé ».
///
/// ⚠️ **Interblocages neufs, bornés** (revue P1, E1/B4), tous deux sur une ligne
/// **existante** — l'ancien code ne la relisait qu'en lecture simple :
/// - **comptes ↔ réglages** : la finalisation tient les comptes de rôle en
///   exclusif et attend la ligne des réglages en exclusif, quand le `PATCH` de
///   liaison d'un compte bancaire (ou la validation d'une facture) tient la
///   ligne et attend un compte ;
/// - **promotion S → X** : l'`INSERT IGNORE` (étape 2) pose sur la ligne
///   existante un verrou **partagé**, que le `FOR UPDATE` (étape 3) promeut en
///   exclusif — le motif que décrit [`update`] : face à un `PUT` des réglages ou
///   à `get_or_create_default_in_tx` (validation d'une facture, avoir, facture
///   fournisseur), qui font de même, deux promotions s'interbloquent (1213).
///
/// Chaque cycle est rejoué des deux côtés — la finalisation et le `PATCH` par
/// `kesh_api::retry::retry_app_on_deadlock`, le `PUT` des réglages, la
/// validation d'une facture, l'avoir et la facture fournisseur par
/// `kesh_db::retry::retry_on_deadlock` —, et une tentative annulée n'écrit rien.
///
/// L'audit est écrit par l'appelant, selon l'issue ([`DefaultAccountsOutcome`]).
/// **Ne commite ni n'annule jamais.**
pub async fn complete_default_accounts_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<DefaultAccountsOutcome, DbError> {
    // 1. Les comptes candidats, verrouillés AVANT la ligne des réglages.
    //
    // Story 34-1 (#966) : les trois lectures passent par la source unique
    // `accounts::find_singleton_role_in_tx`, que l'étape bancaire de l'assistant
    // emploie aussi (sans verrou) ; même requête, même ordre, même `FOR UPDATE`.
    use super::accounts::{RoleLookupLock, find_singleton_role_in_tx};
    let receivable_role = find_singleton_role_in_tx(
        tx,
        company_id,
        AccountRole::Receivable,
        RoleLookupLock::ForUpdate,
    )
    .await?;
    let revenue_role = find_singleton_role_in_tx(
        tx,
        company_id,
        AccountRole::DefaultRevenue,
        RoleLookupLock::ForUpdate,
    )
    .await?;
    // Story 12.2 : compte créanciers (contrepartie achat fournisseur), facultatif.
    let payable_role = find_singleton_role_in_tx(
        tx,
        company_id,
        AccountRole::Payable,
        RoleLookupLock::ForUpdate,
    )
    .await?;
    // Story 25-4-c3-a2 / 25-4-d1 : les comptes désignés par le plan — facultatifs.
    let chart = chart_designated_accounts(tx, company_id).await?;

    // 2. La ligne, créée nue si elle n'existe pas.
    let inserted =
        sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
            .bind(company_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?
            .rows_affected()
            == 1;

    // 3. La ligne sous verrou.
    let before = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ? FOR UPDATE"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // 4. Les contrôles : les deux obligatoires d'abord, débiteurs avant produit.
    let mut unavailable = Vec::new();
    let receivable = mandatory_account(
        tx,
        company_id,
        DefaultInvoiceSetting::Receivable,
        before.default_receivable_account_id,
        receivable_role,
        &mut unavailable,
    )
    .await?;
    let revenue = mandatory_account(
        tx,
        company_id,
        DefaultInvoiceSetting::Revenue,
        before.default_revenue_account_id,
        revenue_role,
        &mut unavailable,
    )
    .await?;
    if !unavailable.is_empty() {
        return Err(DbError::DefaultInvoiceAccountsUnavailable(unavailable));
    }
    // Les facultatifs : sur une ligne vierge seulement (D2) ; un candidat écarté
    // laisse le réglage vide (D6).
    let blank = is_blank(&before);
    let payable = match payable_role {
        Some(account_id)
            if blank
                && role_candidate_rejection(tx, company_id, account_id, true)
                    .await?
                    .is_none() =>
        {
            Some(account_id)
        }
        _ => None,
    };
    let (rounding, discount, bank_fees, bad_debt) = if blank {
        (
            chart.rounding,
            chart.discount,
            chart.bank_fees,
            chart.bad_debt,
        )
    } else {
        (None, None, None, None)
    };

    // 5. Un seul UPDATE, s'il y a quelque chose à poser.
    let fills = [
        receivable, revenue, payable, rounding, discount, bank_fees, bad_debt,
    ];
    if fills.iter().all(Option::is_none) {
        return Ok(if inserted {
            DefaultAccountsOutcome::Inserted(before)
        } else {
            DefaultAccountsOutcome::Unchanged(before)
        });
    }
    sqlx::query(
        "UPDATE company_invoice_settings SET \
         default_receivable_account_id = COALESCE(default_receivable_account_id, ?), \
         default_revenue_account_id = COALESCE(default_revenue_account_id, ?), \
         default_payable_account_id = COALESCE(default_payable_account_id, ?), \
         default_rounding_account_id = COALESCE(default_rounding_account_id, ?), \
         default_discount_account_id = COALESCE(default_discount_account_id, ?), \
         default_bank_fees_account_id = COALESCE(default_bank_fees_account_id, ?), \
         default_bad_debt_account_id = COALESCE(default_bad_debt_account_id, ?), \
         version = version + ? \
         WHERE company_id = ?",
    )
    .bind(receivable)
    .bind(revenue)
    .bind(payable)
    .bind(rounding)
    .bind(discount)
    .bind(bank_fees)
    .bind(bad_debt)
    .bind(if inserted { 0 } else { 1 })
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let after = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ? FOR UPDATE"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(if inserted {
        DefaultAccountsOutcome::Inserted(after)
    } else {
        DefaultAccountsOutcome::Completed {
            before: Box::new(before),
            after,
        }
    })
}

/// Les comptes retenus pour les trois réglages de TVA (Story 31-5, #707), dans
/// l'ordre des colonnes `default_vat_payable_account_id`,
/// `default_vat_recoverable_account_id`, `default_vat_decompte_account_id`.
/// `None` = aucun compte utilisable pour ce réglage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VatRoleAccounts {
    /// TVA due (réglage `default_vat_payable_account_id`).
    pub payable: Option<i64>,
    /// Impôt préalable (réglage `default_vat_recoverable_account_id`).
    pub recoverable: Option<i64>,
    /// Décompte TVA (réglage `default_vat_decompte_account_id`).
    pub settlement: Option<i64>,
}

/// Le type de compte qu'exige chacun des trois réglages de TVA.
#[derive(Debug, Clone, Copy)]
pub struct VatSettingAccountTypes {
    /// TVA due.
    pub payable: AccountType,
    /// Impôt préalable.
    pub recoverable: AccountType,
    /// Décompte TVA.
    pub settlement: AccountType,
}

/// Règle du **réglage** (non du rôle, cf. `accepts_account_type` de
/// `kesh_core::chart_of_accounts`, qui admet `Asset` ou `Liability` pour les trois
/// rôles de TVA), lue par [`lock_vat_role_accounts_in_tx`] et par la route
/// `PUT /api/v1/company/invoice-settings` (`update_invoice_settings`). Écrite une
/// seule fois (Story 31-5, C-31-5-19) : la route et la démonstration ne peuvent
/// plus diverger. Story 35-3 (#537) : la garde à l'usage la lit aussi, pour la
/// TVA due et la TVA récupérable (`DesignatedRole::expected_type`).
pub const VAT_SETTING_ACCOUNT_TYPES: VatSettingAccountTypes = VatSettingAccountTypes {
    payable: AccountType::Liability,
    recoverable: AccountType::Asset,
    settlement: AccountType::Liability,
};

/// Le type de compte qu'exigent les deux réglages de **tiers** : le compte
/// débiteurs et le compte créanciers (Story 35-3, #537).
#[derive(Debug, Clone, Copy)]
pub struct ClaimSettingAccountTypes {
    /// Compte débiteurs (`default_receivable_account_id`).
    pub receivable: AccountType,
    /// Compte créanciers (`default_payable_account_id`).
    pub payable: AccountType,
}

/// Règle des réglages débiteurs et créanciers, écrite **une seule fois**
/// (Story 35-3, #537, D4 ; patron de [`VAT_SETTING_ACCOUNT_TYPES`],
/// C-31-5-19) : la route `PUT /api/v1/company/invoice-settings` la lit à la
/// désignation, et la garde à l'usage ([`DesignatedAccountsSnapshot`]) à la
/// validation d'une facture ou d'une facture fournisseur — la règle de la
/// désignation et celle de l'usage ne peuvent plus diverger.
///
/// Règle du **réglage**, non du rôle : `accepts_account_type` de
/// `kesh_core::chart_of_accounts` admet `Asset` ou `Liability` pour les rôles
/// *Créances clients* et *Dettes fournisseurs*.
pub const CLAIM_SETTING_ACCOUNT_TYPES: ClaimSettingAccountTypes = ClaimSettingAccountTypes {
    receivable: AccountType::Asset,
    payable: AccountType::Liability,
};

/// Lit et **verrouille** (`FOR UPDATE`) les comptes qui portent les trois rôles
/// de TVA de la société, et rend ceux qui sont utilisables pour les réglages de
/// TVA — Story 31-5 (#707, C-31-5-13, C-31-5-18, C-31-5-21, C-31-5-22).
///
/// **Pourquoi la démonstration seulement** (C-31-5-3) : seul `seed_demo` l'appelle.
/// Le chemin production (`finalize`) laisse les trois réglages à `NULL` — un
/// assujetti les désigne dans *Administration → Facturation*, et un non-assujetti
/// qui saisirait un taux par erreur voit la validation refusée
/// (`CONFIGURATION_REQUIRED`) au lieu d'écrire de la TVA due.
///
/// Un rôle n'est retenu que si son compte est **imputable** (`postable`) et du
/// type que [`VAT_SETTING_ACCOUNT_TYPES`] fixe pour le réglage — les exigences de
/// la route de désignation et de `check_written` à la validation. Un compte
/// archivé n'est pas lu (la colonne générée `singleton_role` est `active`-aware).
/// Un rôle sans compte utilisable donne `None`, **sans erreur** : la
/// démonstration se charge quand même.
///
/// **Ordre réel des verrous** : trois lectures ponctuelles
/// `singleton_role = ? FOR UPDATE` sur l'index unique
/// `uq_accounts_company_singleton_role`, dans l'ordre du code — `VatRecoverable`,
/// `VatPayable`, `VatSettlement`. Chacune verrouille l'entrée de l'index puis
/// l'enregistrement du compte ; pour un rôle absent, l'intervalle de l'index où
/// il serait. Au plan standard, cet ordre **coïncide** avec l'ordre croissant des
/// identifiants (1171, 2200, 2206) sans le **garantir** : un rôle réattribué à un
/// autre compte peut le défaire. Le rejeu de `seed_demo` sur interblocage
/// (`is_seed_retryable`) reste le filet.
///
/// **La transaction complète, elle, n'est pas croissante** (Story 32-1a, #716,
/// E7). Dans `final_body`, ces trois verrous **précèdent** les lectures
/// verrouillantes de [`insert_with_defaults_in_tx`] : ses trois rôles
/// (`Receivable`, `DefaultRevenue`, `Payable`), puis les comptes que le plan
/// désigne (`chart_designated_accounts` : l'arrondi, puis les natures d'écart
/// soldé dans l'ordre de `WriteOffNature::ALL`, chacun seulement si le plan le
/// désigne). Ce sont ces deux fonctions qui portent la liste ; elle n'est pas
/// recopiée ici comme exhaustive. Au plan PME, la séquence par numéro est 1171,
/// 2200, 2206, 1100, 3000, 2000, 6940, 3800, 6900, 3805 : un interblocage avec
/// une transaction qui verrouillerait ces comptes dans l'ordre croissant reste
/// possible, et le rejeu de `seed_demo` (`is_seed_retryable`) est le filet.
///
/// À appeler **avant** la création de la ligne de réglages : l'ordre documenté
/// `accounts → company_invoice_settings` (Pattern 5) est ainsi tenu. Ne lit ni
/// n'écrit `company_invoice_settings`. Ne commite ni n'annule.
pub async fn lock_vat_role_accounts_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<VatRoleAccounts, DbError> {
    let types = VAT_SETTING_ACCOUNT_TYPES;
    let recoverable = lock_usable_role_account(
        tx,
        company_id,
        AccountRole::VatRecoverable,
        types.recoverable,
    )
    .await?;
    let payable =
        lock_usable_role_account(tx, company_id, AccountRole::VatPayable, types.payable).await?;
    let settlement =
        lock_usable_role_account(tx, company_id, AccountRole::VatSettlement, types.settlement)
            .await?;
    Ok(VatRoleAccounts {
        payable,
        recoverable,
        settlement,
    })
}

/// Une lecture ponctuelle verrouillante du compte qui porte `role`, retenu s'il
/// est imputable et du type `expected` (cf. [`lock_vat_role_accounts_in_tx`]).
async fn lock_usable_role_account(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    role: AccountRole,
    expected: AccountType,
) -> Result<Option<i64>, DbError> {
    let row = sqlx::query_as::<_, (i64, AccountType, bool)>(
        "SELECT id, account_type, postable FROM accounts \
         WHERE company_id = ? AND singleton_role = ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(role)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(row
        .filter(|(_, account_type, postable)| *postable && *account_type == expected)
        .map(|(id, _, _)| id))
}

/// Désigne les comptes de TVA de `accounts` dans la ligne de réglages de la
/// société — Story 31-5 (#707). Un seul `UPDATE`, `COALESCE` par colonne : une
/// valeur déjà posée n'est **jamais** écrasée, et la fonction est idempotente
/// (rejeu de `seed_demo`). Ne lit **aucun** compte : aucun verrou de compte n'est
/// pris après la ligne de réglages (ordre `accounts → company_invoice_settings`).
///
/// N'incrémente pas `version` : l'appelant (`seed_demo`) ne l'emploie que sur une
/// ligne que la même transaction vient de créer, que personne n'a donc pu lire —
/// ou, depuis la Story 36-19 (revue P1, B1/E5), sur une ligne **vierge** que la
/// même transaction vient de compléter et tient en `FOR UPDATE` : sa `version` a
/// déjà été incrémentée par le complément, et personne ne l'a lue depuis.
/// Ne commite ni n'annule.
pub async fn designate_vat_accounts_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    accounts: &VatRoleAccounts,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE company_invoice_settings SET \
         default_vat_payable_account_id = COALESCE(default_vat_payable_account_id, ?), \
         default_vat_recoverable_account_id = COALESCE(default_vat_recoverable_account_id, ?), \
         default_vat_decompte_account_id = COALESCE(default_vat_decompte_account_id, ?) \
         WHERE company_id = ?",
    )
    .bind(accounts.payable)
    .bind(accounts.recoverable)
    .bind(accounts.settlement)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(())
}

// Reference to avoid unused import warning if Journal is not referenced
// elsewhere in this file (needed for the SQL bind).
#[allow(dead_code)]
const _JOURNAL_TYPE_MARKER: Option<Journal> = None;

#[cfg(test)]
mod designated_changes_tests {
    use super::*;

    /// `default_revenue_account_id` → `defaultRevenueAccountId`.
    fn camel(column: &str) -> String {
        let mut out = String::new();
        let mut upper = false;
        for c in column.chars() {
            if c == '_' {
                upper = true;
            } else if upper {
                out.push(c.to_ascii_uppercase());
                upper = false;
            } else {
                out.push(c);
            }
        }
        out
    }

    /// Story 37-1 (AC-4) : [`designated_changes`] rend les dix colonnes de
    /// [`DESIGNATED_ACCOUNT_COLUMNS`], dans leur ordre, chacune avec SA valeur
    /// demandée et SA valeur en place — un onzième réglage, ou deux colonnes
    /// croisées, rougissent ici.
    #[test]
    fn designated_changes_follows_designated_account_columns() {
        let mut before = serde_json::json!({
            "companyId": 1, "invoiceNumberFormat": "F", "defaultSalesJournal": "Ventes",
            "journalEntryDescriptionTemplate": "T", "creditNoteNumberFormat": "A",
            "supplierInternalNumberFormat": "FF", "roundTo5Centimes": false,
            "minimumInvoiceAmount": null, "version": 1,
            "createdAt": "2026-01-01T00:00:00", "updatedAt": "2026-01-01T00:00:00",
        });
        let mut changes = serde_json::json!({
            "invoiceNumberFormat": "F", "defaultSalesJournal": "Ventes",
            "journalEntryDescriptionTemplate": "T", "creditNoteNumberFormat": "A",
            "supplierInternalNumberFormat": "FF", "roundTo5Centimes": false,
            "minimumInvoiceAmount": null,
        });
        for (i, (column, _)) in DESIGNATED_ACCOUNT_COLUMNS.iter().enumerate() {
            before[camel(column)] = serde_json::json!(i as i64 + 1);
            changes[camel(column)] = serde_json::json!(i as i64 + 101);
        }
        let before: CompanyInvoiceSettings = serde_json::from_value(before).unwrap();
        let changes: CompanyInvoiceSettingsUpdate = serde_json::from_value(changes).unwrap();
        let rendered = designated_changes(&before, &changes);
        for (i, ((_, code), (got_code, requested, current))) in DESIGNATED_ACCOUNT_COLUMNS
            .iter()
            .zip(rendered.iter())
            .enumerate()
        {
            assert_eq!(code, got_code, "rang {i}");
            assert_eq!(*requested, Some(i as i64 + 101), "demandé de {code}");
            assert_eq!(*current, Some(i as i64 + 1), "en place de {code}");
        }
    }
}
