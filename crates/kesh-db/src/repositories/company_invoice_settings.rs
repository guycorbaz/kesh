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

use sqlx::mysql::MySqlPool;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{
    AccountRole, AccountType, CompanyInvoiceSettings, CompanyInvoiceSettingsUpdate, Journal,
};
use crate::errors::{DbError, RoundingContext, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str = "company_id, invoice_number_format, default_receivable_account_id, \
    default_revenue_account_id, default_vat_payable_account_id, \
    default_vat_recoverable_account_id, default_vat_decompte_account_id, \
    default_sales_journal, journal_entry_description_template, \
    credit_note_number_format, default_payable_account_id, \
    default_rounding_account_id, round_to_5_centimes, minimum_invoice_amount, \
    default_discount_account_id, default_bank_fees_account_id, default_bad_debt_account_id, \
    version, created_at, updated_at";

fn settings_snapshot_json(s: &CompanyInvoiceSettings) -> serde_json::Value {
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
        && before.default_payable_account_id == changes.default_payable_account_id
        && before.default_rounding_account_id == changes.default_rounding_account_id
        && before.round_to_5_centimes == changes.round_to_5_centimes
        && before.minimum_invoice_amount == changes.minimum_invoice_amount
        && before.default_discount_account_id == changes.default_discount_account_id
        && before.default_bank_fees_account_id == changes.default_bank_fees_account_id
        && before.default_bad_debt_account_id == changes.default_bad_debt_account_id
}

/// Met à jour la config (tous les champs) avec verrou optimiste et audit.
///
/// Le caller doit avoir validé les données en amont (format, comptes,
/// journal). Audit log wrapper `{before, after}`.
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    expected_version: i32,
    user_id: i64,
    changes: CompanyInvoiceSettingsUpdate,
) -> Result<CompanyInvoiceSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // S'assurer que la row existe (création lazy si absente).
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    if before.version != expected_version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    // KF-004 : court-circuit no-op AVANT toute mutation.
    // NOTE concurrence (KF-004): sous REPEATABLE READ + plain SELECT, si une tx
    // parallèle commit entre notre BEGIN et ce check, on retourne notre snapshot
    // stale au lieu d'un 409. Race acceptée v0.1 (cf. spec 7-3 §race-condition).
    // Mitigation future: SELECT FOR UPDATE partout (non v0.1).
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
/// Partagé par les deux `insert_with_defaults*` (DRY). Le plan est **relu** ici
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
    let designated: Option<Option<i64>> = sqlx::query_scalar(
        "SELECT default_rounding_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let Some(account_id) = designated.flatten() else {
        return Err(DbError::RoundingAccountNotConfigured { context });
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
    .map_err(map_db_error)?
    .ok_or(DbError::RoundingAccountNotConfigured { context })
}

/// Creates company_invoice_settings with auto-prefill of default accounts resolved
/// by role (`Receivable`, `DefaultRevenue`, `Payable` — Story 14-3b : no longer by
/// hardcoded account number). Called during onboarding finalization (after chart of
/// accounts is loaded).
///
/// **Pool-level variant** (`insert_with_defaults`): Opens its own transaction.
/// Used by seed_demo (Path A), which doesn't need locking coordination.
///
/// **Transaction-level variant** (`insert_with_defaults_in_tx`): Works within caller's transaction.
/// Used by finalize() (Path B), which holds locks on company and onboarding_state.
/// Caller must pass open transaction to keep locks alive during account lookup and INSERT.
///
/// **Duplication note**: Code is intentionally duplicated between these variants rather than
/// using a generic executor macro due to SQLx 0.8 HRTB fragility (see repository docstring P13).
/// The 5-line body at lines marked MIRROR must stay synchronized.
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

    // MIRROR: Keep synchronized with insert_with_defaults_in_tx
    // F1 CRITICAL FIX: Lock accounts rows during lookup to prevent concurrent deletes.
    // SELECT FOR UPDATE prevents other transactions from modifying these rows until commit.
    // ORDER BY id LIMIT 1 ensures deterministic single-row lock (schema uniqueness guarantee).
    let receivable = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE"
    )
    .bind(company_id)
    .bind(AccountRole::Receivable)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    let revenue = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE"
    )
    .bind(company_id)
    .bind(AccountRole::DefaultRevenue)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    // Story 12.2 : compte créanciers 2000 (contrepartie achat fournisseur).
    // OPTIONNEL (non fail-fast) — présent dans les charts standards mais
    // l'absence ne doit pas bloquer la finalisation d'onboarding.
    let payable = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE"
    )
    .bind(company_id)
    .bind(AccountRole::Payable)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    // P1-004 + P1-007: Early NULL validation before INSERT (fail-fast pattern).
    // P6-M2: rollback is best-effort. The previous `.map_err(map_db_error)?` would
    // hide InactiveOrInvalidAccounts behind a transient rollback error and break
    // the retry-loop matching in seed_demo (which keys on this exact variant).
    if receivable.is_none() || revenue.is_none() {
        let _ = tx.rollback().await;
        return Err(DbError::InactiveOrInvalidAccounts);
    }

    // Story 25-4-c3-a2 / 25-4-d1 : les comptes désignés par le plan — facultatifs.
    let designated = chart_designated_accounts(&mut tx, company_id).await?;

    // P1-C1: Check rows_affected to distinguish newly inserted vs pre-existing rows
    // INSERT IGNORE suppresses errors but returns rows_affected=0 if DUPLICATE KEY
    let rows = sqlx::query(
        "INSERT IGNORE INTO company_invoice_settings \
         (company_id, invoice_number_format, default_receivable_account_id, \
          default_revenue_account_id, default_payable_account_id, \
          default_rounding_account_id, default_discount_account_id, \
          default_bank_fees_account_id, default_bad_debt_account_id, \
          default_sales_journal, journal_entry_description_template) \
         VALUES (?, 'F-{YEAR}-{SEQ:04}', ?, ?, ?, ?, ?, ?, ?, 'Ventes', '{YEAR}-{INVOICE_NUMBER}')",
    )
    .bind(company_id)
    .bind(receivable)
    .bind(revenue)
    .bind(payable)
    .bind(designated.rounding)
    .bind(designated.discount)
    .bind(designated.bank_fees)
    .bind(designated.bad_debt)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    // If rows==0, row already existed (DUPLICATE KEY).
    // P16: validate that the referenced accounts are still alive (not soft-deleted).
    // Pure NULL re-check on the row would be dead defense — the new fail-fast path
    // can no longer insert NULLs. Joining on accounts.active=TRUE catches the case
    // where a previously-good FK now points to a deactivated account.
    // CI fix: explicit `cis.` prefix on the SELECT list — `accounts` also has a
    // `company_id` column, so `{COLUMNS}` (unprefixed) yields "Column ambiguous".
    if rows == 0 {
        let existing = sqlx::query_as::<_, CompanyInvoiceSettings>(
            "SELECT cis.company_id, cis.invoice_number_format, cis.default_receivable_account_id, \
                    cis.default_revenue_account_id, cis.default_vat_payable_account_id, \
                    cis.default_vat_recoverable_account_id, cis.default_vat_decompte_account_id, \
                    cis.default_sales_journal, \
                    cis.journal_entry_description_template, cis.credit_note_number_format, \
                    cis.default_payable_account_id, cis.default_rounding_account_id, \
                    cis.round_to_5_centimes, cis.minimum_invoice_amount, \
                    cis.default_discount_account_id, cis.default_bank_fees_account_id, \
                    cis.default_bad_debt_account_id, \
                    cis.version, cis.created_at, cis.updated_at \
             FROM company_invoice_settings cis \
             JOIN accounts ar ON ar.id = cis.default_receivable_account_id AND ar.active = TRUE \
             JOIN accounts av ON av.id = cis.default_revenue_account_id AND av.active = TRUE \
             WHERE cis.company_id = ?",
        )
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

        match existing {
            Some(row) => {
                tx.commit().await.map_err(map_db_error)?;
                return Ok(row);
            }
            None => {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::InactiveOrInvalidAccounts);
            }
        }
    }

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

/// Transaction-aware variant of insert_with_defaults for finalize() to use.
/// Keeps account locks within the caller's transaction scope.
///
/// F2/F3/F4 CRITICAL: Caller must hold SELECT FOR UPDATE lock on onboarding_state
/// and company to prevent deletion races. This function's SELECT FOR UPDATE on accounts
/// complements the higher-level locks, ensuring deterministic ordering of account lookups.
pub async fn insert_with_defaults_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, DbError> {
    // MIRROR: Keep synchronized with insert_with_defaults
    // F1 CRITICAL FIX: Lock accounts rows during lookup to prevent concurrent deletes.
    // SELECT FOR UPDATE prevents other transactions from modifying these rows until commit.
    // ORDER BY id LIMIT 1 ensures deterministic single-row lock (schema uniqueness guarantee).
    let receivable = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE"
    )
    .bind(company_id)
    .bind(AccountRole::Receivable)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    let revenue = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE"
    )
    .bind(company_id)
    .bind(AccountRole::DefaultRevenue)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    // Story 12.2 : compte créanciers 2000 (contrepartie achat fournisseur).
    // OPTIONNEL (non fail-fast) — cf. variante pool.
    let payable = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE"
    )
    .bind(company_id)
    .bind(AccountRole::Payable)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    // P1-004 + P1-007: Early NULL validation before INSERT (fail-fast pattern).
    // Si aucun compte actif ne porte les rôles Receivable/DefaultRevenue (Story 14-3b),
    // rejeter immédiatement au lieu de créer des lignes NULL — prévient la corruption
    // et fournit un message clair aux appelants.
    if receivable.is_none() || revenue.is_none() {
        return Err(DbError::InactiveOrInvalidAccounts);
    }

    // Story 25-4-c3-a2 / 25-4-d1 : les comptes désignés par le plan — facultatifs.
    let designated = chart_designated_accounts(tx, company_id).await?;

    // P1-C1: Check rows_affected to distinguish newly inserted vs pre-existing rows
    // INSERT IGNORE suppresses errors but returns rows_affected=0 if DUPLICATE KEY
    let rows = sqlx::query(
        "INSERT IGNORE INTO company_invoice_settings \
         (company_id, invoice_number_format, default_receivable_account_id, \
          default_revenue_account_id, default_payable_account_id, \
          default_rounding_account_id, default_discount_account_id, \
          default_bank_fees_account_id, default_bad_debt_account_id, \
          default_sales_journal, journal_entry_description_template) \
         VALUES (?, 'F-{YEAR}-{SEQ:04}', ?, ?, ?, ?, ?, ?, ?, 'Ventes', '{YEAR}-{INVOICE_NUMBER}')",
    )
    .bind(company_id)
    .bind(receivable)
    .bind(revenue)
    .bind(payable)
    .bind(designated.rounding)
    .bind(designated.discount)
    .bind(designated.bank_fees)
    .bind(designated.bad_debt)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    // If rows==0, row already existed (DUPLICATE KEY).
    // P16: validate FK liveness via JOIN on accounts.active = TRUE (cf. pool variant).
    // CI fix: explicit `cis.` prefix — `accounts` also has a `company_id` column.
    if rows == 0 {
        let existing = sqlx::query_as::<_, CompanyInvoiceSettings>(
            "SELECT cis.company_id, cis.invoice_number_format, cis.default_receivable_account_id, \
                    cis.default_revenue_account_id, cis.default_vat_payable_account_id, \
                    cis.default_vat_recoverable_account_id, cis.default_vat_decompte_account_id, \
                    cis.default_sales_journal, \
                    cis.journal_entry_description_template, cis.credit_note_number_format, \
                    cis.default_payable_account_id, cis.default_rounding_account_id, \
                    cis.round_to_5_centimes, cis.minimum_invoice_amount, \
                    cis.default_discount_account_id, cis.default_bank_fees_account_id, \
                    cis.default_bad_debt_account_id, \
                    cis.version, cis.created_at, cis.updated_at \
             FROM company_invoice_settings cis \
             JOIN accounts ar ON ar.id = cis.default_receivable_account_id AND ar.active = TRUE \
             JOIN accounts av ON av.id = cis.default_revenue_account_id AND av.active = TRUE \
             WHERE cis.company_id = ?",
        )
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

        return match existing {
            Some(row) => Ok(row),
            None => Err(DbError::InactiveOrInvalidAccounts),
        };
    }

    let settings = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    Ok(settings)
}

// Reference to avoid unused import warning if Journal is not referenced
// elsewhere in this file (needed for the SQL bind).
#[allow(dead_code)]
const _JOURNAL_TYPE_MARKER: Option<Journal> = None;
