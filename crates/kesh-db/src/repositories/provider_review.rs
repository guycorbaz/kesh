//! Lectures de la **revue d'un compte de prestataire** et du pré-vol de son
//! acceptation — Story 38-11c2 (AC-5, AC-6, AC-7), communes à PayPal, Wise et
//! Stripe (aucun nom de prestataire ici).
//!
//! Lectures **ordinaires**, sans verrou, génériques sur l'exécuteur : la revue
//! les fait toutes dans **une** transaction de lecture (instantané unique),
//! l'acceptation en fait certaines sur le pool avant ses verrous, puis relit
//! sous les verrous ce qu'elle rejuge. Rien n'est écrit.
//!
//! Les écritures de vente se lisent par **une** requête bornée à l'exercice et
//! aux comptes du plan demandés ([`sale_entries`]) ; la recherche de
//! l'identifiant dans le libellé se fait en Rust par la fonction d'extraction
//! du prestataire, **jamais** par un `LIKE`.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySql;

use crate::entities::bank_transaction::BankTransaction;
use crate::entities::fiscal_year::FiscalYear;
use crate::errors::{DbError, map_db_error};

const BANK_TX_COLUMNS: &str = super::bank_transactions::COLUMNS;

/// `?, ?, …` pour `n` paramètres (`n ≥ 1`).
fn placeholders(n: usize) -> String {
    vec!["?"; n].join(", ")
}

/// L'exercice `fiscal_year_id` de la société, ou `None` (absent ou d'une autre
/// société).
pub async fn find_fiscal_year<'e, E>(
    executor: E,
    company_id: i64,
    fiscal_year_id: i64,
) -> Result<Option<FiscalYear>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, multi_currency, \
                opening_rate_kind, closing_rate_kind, created_at, updated_at \
         FROM fiscal_years WHERE id = ? AND company_id = ?",
    )
    .bind(fiscal_year_id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Les comptes bancaires de la société (archivés compris) liés à l'un des
/// comptes du plan `ledger_ids` : ceux du compte de prestataire et tout autre
/// compte qui partagerait l'un de ces comptes du plan (AC-5). Triés par
/// identifiant.
pub async fn bank_accounts_on_ledgers<'e, E>(
    executor: E,
    company_id: i64,
    ledger_ids: &[i64],
) -> Result<Vec<i64>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ledger_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT id FROM bank_accounts WHERE company_id = ? AND journal_account_id IN ({}) ORDER BY id",
        placeholders(ledger_ids.len())
    );
    let mut q = sqlx::query_scalar::<_, i64>(&sql).bind(company_id);
    for id in ledger_ids {
        q = q.bind(*id);
    }
    q.fetch_all(executor).await.map_err(map_db_error)
}

/// Les transactions **pending** des comptes bancaires `bank_account_ids`, dont
/// la date (`value_date`, à défaut `booking_date`) tombe dans `[from, to]`,
/// triées par date puis identifiant.
pub async fn pending_in_period<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_ids: &[i64],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if bank_account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND status = 'pending' AND bank_account_id IN ({}) \
           AND COALESCE(value_date, booking_date) BETWEEN ? AND ? \
         ORDER BY COALESCE(value_date, booking_date), id",
        placeholders(bank_account_ids.len())
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql).bind(company_id);
    for id in bank_account_ids {
        q = q.bind(*id);
    }
    q.bind(from)
        .bind(to)
        .fetch_all(executor)
        .await
        .map_err(map_db_error)
}

/// **Toutes** les transactions (pending et rapprochées, toutes dates) des
/// comptes bancaires `bank_account_ids` — les clés des ventes encaissées
/// (AC-5, vente sans encaissement).
pub async fn transactions_of_accounts<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_ids: &[i64],
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if bank_account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id IN ({}) ORDER BY id",
        placeholders(bank_account_ids.len())
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql).bind(company_id);
    for id in bank_account_ids {
        q = q.bind(*id);
    }
    q.fetch_all(executor).await.map_err(map_db_error)
}

/// Les transactions `ids` de la société, quel que soit leur compte ou leur
/// statut, par identifiant (pré-vol de l'acceptation). Au plus
/// [`super::reconciliation::MAX_IN_CLAUSE_IDS`] identifiants : l'appelant borne
/// le lot avant.
pub async fn transactions_by_ids<'e, E>(
    executor: E,
    company_id: i64,
    ids: &[i64],
) -> Result<HashMap<i64, BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions WHERE company_id = ? AND id IN ({})",
        placeholders(ids.len())
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(*id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(|t| (t.id, t)).collect())
}

/// Une ligne de [`sale_entries`] : une écriture et son mouvement sur **un**
/// des comptes du plan demandés.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SaleEntryLine {
    /// Identifiant de l'écriture.
    pub journal_entry_id: i64,
    /// Numéro.
    pub entry_number: i64,
    /// Date.
    pub entry_date: NaiveDate,
    /// Libellé.
    pub description: String,
    /// Le compte du plan.
    pub account_id: i64,
    /// Somme des débits sur ce compte, en CHF.
    pub debit_total: Decimal,
    /// Mouvement en CHF (débit − crédit).
    pub chf_movement: Decimal,
    /// Mouvement en devise du compte (montant en devise de chaque ligne, du
    /// signe de son côté ; 0 pour des lignes en CHF).
    pub currency_movement: Decimal,
    /// Une transaction bancaire pointe l'écriture (0 ou 1).
    pub linked: i64,
}

/// Les **écritures de vente candidates** de l'exercice (AC-5) : écritures de
/// la société dans l'exercice `fiscal_year_id`, ni contre-passées ni
/// contre-passation, avec leur mouvement sur chacun des comptes du plan
/// `ledger_ids` qu'elles touchent. **Une** requête ; le filtre « au moins une
/// ligne au débit d'un de ces comptes » et l'extraction des identifiants se
/// font en Rust chez l'appelant.
pub async fn sale_entries<'e, E>(
    executor: E,
    company_id: i64,
    fiscal_year_id: i64,
    ledger_ids: &[i64],
) -> Result<Vec<SaleEntryLine>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ledger_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT e.id AS journal_entry_id, e.entry_number, e.entry_date, e.description, \
                l.account_id, \
                SUM(l.debit) AS debit_total, \
                SUM(l.debit) - SUM(l.credit) AS chf_movement, \
                SUM(CASE WHEN l.debit > 0 THEN COALESCE(l.amount_currency, 0) \
                         ELSE -COALESCE(l.amount_currency, 0) END) AS currency_movement, \
                CAST(EXISTS (SELECT 1 FROM bank_transactions bt WHERE bt.matched_entry_id = e.id) \
                     AS SIGNED) AS linked \
         FROM journal_entries e \
         JOIN journal_entry_lines l ON l.entry_id = e.id \
         WHERE e.company_id = ? AND e.fiscal_year_id = ? AND l.account_id IN ({}) \
           AND e.reverses_entry_id IS NULL \
           AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id) \
         GROUP BY e.id, e.entry_number, e.entry_date, e.description, l.account_id \
         ORDER BY e.id, l.account_id",
        placeholders(ledger_ids.len())
    );
    let mut q = sqlx::query_as::<_, SaleEntryLine>(&sql)
        .bind(company_id)
        .bind(fiscal_year_id);
    for id in ledger_ids {
        q = q.bind(*id);
    }
    q.fetch_all(executor).await.map_err(map_db_error)
}

/// Les comptes admis comme contrepartie d'une règle de prestataire : actifs,
/// imputables, **en CHF** (phase 1 : aucune règle à compte en devise, AC-10).
pub async fn rule_account_ids<'e, E>(executor: E, company_id: i64) -> Result<HashSet<i64>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM accounts \
         WHERE company_id = ? AND active = TRUE AND postable = TRUE AND currency IS NULL",
    )
    .bind(company_id)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)?;
    Ok(ids.into_iter().collect())
}

/// Le compte du plan du compte bancaire **principal** de la société (non
/// archivé), ou `None` — la contrepartie préremplie d'un retrait (AC-5).
pub async fn primary_ledger_account_id<'e, E>(
    executor: E,
    company_id: i64,
) -> Result<Option<i64>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let row: Option<Option<i64>> = sqlx::query_scalar(
        "SELECT journal_account_id FROM bank_accounts \
         WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)?;
    Ok(row.flatten())
}

/// Un compte du plan lu pour le contrôle d'une contrepartie (AC-8).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CounterpartyAccount {
    /// Identifiant.
    pub id: i64,
    /// Numéro.
    pub number: String,
    /// Actif.
    pub active: bool,
    /// Imputable.
    pub postable: bool,
    /// Type (`Asset`, `Liability`, `Revenue`, `Expense`).
    pub account_type: String,
    /// Devise (`None` : CHF).
    pub currency: Option<String>,
}

/// Le compte du plan `account_id` de la société, pour le contrôle d'une
/// contrepartie, ou `None`. Lecture ordinaire, dans la transaction de
/// l'acceptation.
pub async fn counterparty_account<'e, E>(
    executor: E,
    company_id: i64,
    account_id: i64,
) -> Result<Option<CounterpartyAccount>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, CounterpartyAccount>(
        "SELECT id, number, active, postable, account_type, currency \
         FROM accounts WHERE id = ? AND company_id = ?",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(DbError::Sqlx)
}

/// Le libellé de l'écriture `journal_entry_id` de la société (rejugement d'un
/// rattachement : l'identifiant y figure-t-il toujours ?), ou `None`.
pub async fn entry_description<'e, E>(
    executor: E,
    company_id: i64,
    journal_entry_id: i64,
) -> Result<Option<String>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_scalar("SELECT description FROM journal_entries WHERE id = ? AND company_id = ?")
        .bind(journal_entry_id)
        .bind(company_id)
        .fetch_optional(executor)
        .await
        .map_err(DbError::Sqlx)
}

/// Le nombre de transactions qui pointent l'écriture `journal_entry_id`
/// (lecture ordinaire) : le rejugement d'un rattachement d'encaissement (la
/// vente doit être encore non rattachée, sinon la proposition est périmée).
pub async fn entry_link_count<'e, E>(
    executor: E,
    company_id: i64,
    journal_entry_id: i64,
) -> Result<i64, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_transactions WHERE company_id = ? AND matched_entry_id = ?",
    )
    .bind(company_id)
    .bind(journal_entry_id)
    .fetch_one(executor)
    .await
    .map_err(DbError::Sqlx)
}

/// Les **autres** transactions qui pointent l'écriture `journal_entry_id`, en
/// lecture **verrouillante** (`LOCK IN SHARE MODE`) : le contrôle, après un
/// rattachement d'encaissement, qu'aucune transaction d'un autre compte n'a
/// rattaché la même vente entre-temps (une lecture ordinaire lirait
/// l'instantané de la transaction, ouvert avant le verrou de l'écriture).
pub async fn other_links_locked(
    tx: &mut sqlx::Transaction<'_, MySql>,
    company_id: i64,
    journal_entry_id: i64,
    bank_transaction_id: i64,
) -> Result<i64, DbError> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_transactions \
         WHERE company_id = ? AND matched_entry_id = ? AND id <> ? LOCK IN SHARE MODE",
    )
    .bind(company_id)
    .bind(journal_entry_id)
    .bind(bank_transaction_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(DbError::Sqlx)
}
