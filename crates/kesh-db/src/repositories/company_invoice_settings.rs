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

use std::collections::BTreeSet;

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, QueryBuilder};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{
    AccountRole, AccountType, CompanyInvoiceSettings, CompanyInvoiceSettingsUpdate, Journal,
};
use crate::errors::{
    DbError, NonPostableAccount, NonPostableAccounts, RoundingContext, map_db_error,
};
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
/// relu au moment d'écrire : actif, imputable, de la société, `FOR UPDATE` —
/// comme le compte de la nature, sans contrainte de type (un compte de TVA due
/// est un passif). Absent ou inutilisable → `ConfigurationRequired`, le même
/// refus que l'avoir pour un réglage absent.
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
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM accounts WHERE id = ? AND company_id = ? AND active = TRUE \
         AND postable = TRUE FOR UPDATE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?
    .ok_or_else(missing)
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

/// Un compte désigné tel que l'a lu le verrou de
/// [`lock_designated_accounts_in_tx`].
#[derive(Debug, Clone, sqlx::FromRow)]
struct LockedDesignatedAccount {
    id: i64,
    number: String,
    active: bool,
    postable: bool,
}

/// L'instantané **verrouillé** des comptes candidats d'un flux, rendu par
/// [`lock_designated_accounts_in_tx`] ; ses lignes restent verrouillées (en
/// partagé) jusqu'au commit, il n'y a donc rien à relire pour le contrôle.
#[derive(Debug)]
pub(in crate::repositories) struct DesignatedAccountsSnapshot(Vec<LockedDesignatedAccount>);

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
/// # Le verrou : partagé, une requête, `ORDER BY id`
///
/// `SELECT id, number, active, postable FROM accounts FORCE INDEX (PRIMARY)
/// WHERE company_id = ? AND id IN (…) ORDER BY id LOCK IN SHARE MODE` (syntaxe
/// MariaDB 10.11 : pas de `FOR SHARE`). L'ordre de verrouillage entre les comptes est celui des
/// identifiants.
///
/// **Pourquoi partagé suffit** (C87) : le verrou n'a qu'un but, qu'aucun
/// archivage ni passage à non imputable ne s'insère entre le contrôle et
/// l'insertion de l'écriture. Or ces gestes **écrivent** la ligne du compte —
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
/// - le **type** du compte (angle mort assumé, préexistant — un compte désigné
///   retypé après sa désignation reste utilisé ; le compte d'arrondi, lui,
///   contrôle son type) ;
/// - le **compte bancaire** (D-A0, C6), le **compte de produit par défaut**
///   (D3-bis de la 16-1a), le compte de **décompte TVA** (lu par aucun flux
///   d'écriture) ;
/// - **l'avoir**, qui relit la créance et la TVA due dans les réglages du moment
///   sans ce contrôle, délibérément (C35 : ces lectures sont elles-mêmes le
///   défaut de #473 et #525) ;
/// - le **solde du reste** garde son refus `ConfigurationRequired` pour une TVA
///   due inutilisable ([`vat_payable_account_for_write`]) — divergence assumée
///   (C28).
///
/// `journal_entries::create_in_tx` garde `enforce_postable = false` (D-A0) : la
/// garde est en amont, sur les seuls comptes de réglage. Elle révise la limite
/// **L2** de D-A0 (`14-3b-consommateurs-roles.md`) pour ces quatre comptes.
pub(in crate::repositories) async fn lock_designated_accounts_in_tx(
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
        "SELECT id, number, active, postable FROM accounts FORCE INDEX (PRIMARY) WHERE company_id = ",
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
    /// **Second temps** de la garde (Story 15-5d ; choix C39, C40, C49) :
    /// contrôle, sur l'instantané verrouillé, **les seuls comptes des rôles
    /// effectivement écrits** par le générateur (`roles`), traduits en
    /// identifiants par les réglages qui ont servi à la génération.
    ///
    /// Appelé **entre** la génération des lignes et
    /// `journal_entries::create_in_tx`. Refuse, dans cet ordre :
    ///
    /// 1. un compte d'un rôle écrit **absent de l'instantané** (inexistant, ou
    ///    d'une autre société) **ou inactif** (archivé) →
    ///    [`DbError::InactiveOrInvalidAccounts`] — le refus que `create_in_tx`
    ///    rend pour ce cas, sans nommer le compte (anti-énumération) ;
    /// 2. sinon, les comptes de la société, actifs et **non imputables** →
    ///    [`DbError::DesignatedAccountsNotPostable`], un seul refus qui les nomme
    ///    tous. La liste est construite par [`NonPostableAccounts::new`], qui
    ///    **dédoublonne par identifiant** : un même compte désigné pour deux
    ///    rôles est nommé une fois (C40).
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
        let mut non_postable: Vec<NonPostableAccount> = Vec::new();
        for role in roles {
            // Un rôle écrit a forcément un compte : le générateur a refusé
            // `ConfigurationRequired` sinon.
            let id = role.designated_id(settings).ok_or_else(|| {
                DbError::Invariant(format!(
                    "rôle de réglage {role:?} écrit par le générateur sans compte désigné"
                ))
            })?;
            match self.0.iter().find(|a| a.id == id) {
                Some(a) if a.active && a.postable => {}
                Some(a) if a.active => non_postable.push(NonPostableAccount {
                    account_id: a.id,
                    account_number: a.number.clone(),
                }),
                _ => return Err(DbError::InactiveOrInvalidAccounts),
            }
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
/// Used by seed_demo (Path A), which doesn't need locking coordination.
///
/// **Transaction-level variant** (`insert_with_defaults_in_tx`): Works within caller's transaction.
/// Used by finalize() (Path B), which holds locks on company and onboarding_state.
/// Caller must pass open transaction to keep locks alive during account lookup and INSERT.
///
/// **Délégation (Story 15-7a1)** : la variante pool n'a plus de corps propre —
/// elle **appelle** [`insert_with_defaults_in_tx`] (`begin`, variante, `commit`) ;
/// sur erreur, rollback *best-effort* puis l'**erreur d'origine rendue telle
/// quelle** (P6-M2 : la boucle de retry de `seed_demo` reconnaît exactement
/// `DbError::InactiveOrInvalidAccounts`). La duplication et ses marqueurs
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
            // would hide InactiveOrInvalidAccounts behind a transient error and
            // break the retry-loop matching in seed_demo (which keys on this
            // exact variant).
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Transaction-aware variant of insert_with_defaults for finalize() to use.
/// Keeps account locks within the caller's transaction scope.
///
/// Rend `(réglages, inséré)` (Story 15-7a1) : le booléen vaut `true` si
/// l'`INSERT IGNORE` a réellement inséré la ligne (`rows_affected == 1`),
/// `false` si elle existait déjà (réglages relus, comptes vivants vérifiés).
/// **Ne commite ni n'annule jamais** : sur erreur, l'appelant annule.
///
/// F2/F3/F4 CRITICAL: Caller must hold SELECT FOR UPDATE lock on onboarding_state
/// and company to prevent deletion races. This function's SELECT FOR UPDATE on accounts
/// complements the higher-level locks, ensuring deterministic ordering of account lookups.
pub async fn insert_with_defaults_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<(CompanyInvoiceSettings, bool), DbError> {
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
    // OPTIONNEL (non fail-fast) — présent dans les charts standards mais
    // l'absence ne doit pas bloquer la finalisation d'onboarding.
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
    // Story 15-7a1 : « inséré » = l'INSERT IGNORE a réellement écrit la ligne.
    let inserted = rows == 1;

    // If rows==0, row already existed (DUPLICATE KEY).
    // P16: validate that the referenced accounts are still alive (not soft-deleted).
    // Pure NULL re-check on the row would be dead defense — the fail-fast path
    // above can no longer insert NULLs. Joining on accounts.active=TRUE catches the
    // case where a previously-good FK now points to a deactivated account.
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
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

        return match existing {
            Some(row) => Ok((row, inserted)),
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

    Ok((settings, inserted))
}

// Reference to avoid unused import warning if Journal is not referenced
// elsewhere in this file (needed for the SQL bind).
#[allow(dead_code)]
const _JOURNAL_TYPE_MARKER: Option<Journal> = None;
