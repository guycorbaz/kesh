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

use std::collections::HashMap;

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
/// la date (`booking_date`, jamais `value_date` : amendement du 21.11 de la
/// Story 40-7c, voir `kesh_reconciliation::provider_review::payment_date`)
/// tombe dans `[from, to]`, triées par date puis identifiant.
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
           AND booking_date BETWEEN ? AND ? \
         ORDER BY booking_date, id",
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

/// Les transactions des comptes bancaires `bank_account_ids` qui portent l'un
/// des identifiants de regroupement `group_ids` (`end_to_end_id`), **tous
/// états et toutes dates**, par identifiant — Story 40-7c (AC-4) : les lignes
/// des versements Stripe, une requête pour tous les comptes. Au plus
/// [`super::reconciliation::MAX_IN_CLAUSE_IDS`] identifiants par appel : au-delà,
/// la lecture se fait par tranches.
pub async fn transactions_of_groups(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    bank_account_ids: &[i64],
    group_ids: &[String],
) -> Result<Vec<BankTransaction>, DbError> {
    let mut out = Vec::new();
    if bank_account_ids.is_empty() {
        return Ok(out);
    }
    for chunk in group_ids.chunks(super::reconciliation::MAX_IN_CLAUSE_IDS) {
        let sql = format!(
            "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
             WHERE company_id = ? AND bank_account_id IN ({}) AND end_to_end_id IN ({}) \
             ORDER BY id",
            placeholders(bank_account_ids.len()),
            placeholders(chunk.len())
        );
        let mut q = sqlx::query_as::<_, BankTransaction>(&sql).bind(company_id);
        for id in bank_account_ids {
            q = q.bind(*id);
        }
        for g in chunk {
            q = q.bind(g);
        }
        out.extend(q.fetch_all(&mut *conn).await.map_err(map_db_error)?);
    }
    Ok(out)
}

/// Les transactions **rapprochées** des comptes bancaires `bank_account_ids`,
/// de type `provider_type`, datées (`booking_date`) dans `[from, to]`, dont
/// l'écriture n'est pointée (`matched_entry_id`) par **aucune** transaction
/// d'un autre compte bancaire que ceux-là — Story 40-7c (AC-5) : les
/// versements Stripe dont la réception n'est pas rattachée. Triées par date
/// puis identifiant.
pub async fn reconciled_without_other_side<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_ids: &[i64],
    provider_type: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if bank_account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let ids = placeholders(bank_account_ids.len());
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions bt \
         WHERE bt.company_id = ? AND bt.status = 'reconciled' \
           AND bt.bank_account_id IN ({ids}) AND bt.provider_type = ? \
           AND bt.matched_entry_id IS NOT NULL AND bt.booking_date BETWEEN ? AND ? \
           AND NOT EXISTS (SELECT 1 FROM bank_transactions o \
                            WHERE o.company_id = bt.company_id \
                              AND o.matched_entry_id = bt.matched_entry_id \
                              AND o.bank_account_id NOT IN ({ids})) \
         ORDER BY bt.booking_date, bt.id"
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql).bind(company_id);
    for id in bank_account_ids {
        q = q.bind(*id);
    }
    q = q.bind(provider_type).bind(from).bind(to);
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

/// Ceux des identifiants `ids` qui sont une écriture de la société (lecture
/// ordinaire, sur le pool : pré-vol de l'acceptation). Le lot ne pré-verrouille
/// que ceux-là : un identifiant absent ou d'une autre société ne pose aucun
/// verrou d'intervalle. Au plus
/// [`super::reconciliation::MAX_IN_CLAUSE_IDS`] identifiants.
pub async fn existing_entry_ids<'e, E>(
    executor: E,
    company_id: i64,
    ids: &[i64],
) -> Result<Vec<i64>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT id FROM journal_entries WHERE company_id = ? AND id IN ({}) ORDER BY id",
        placeholders(ids.len())
    );
    let mut q = sqlx::query_scalar::<_, i64>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(*id);
    }
    q.fetch_all(executor).await.map_err(map_db_error)
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

/// Les comptes admis comme contrepartie d'une règle de prestataire — actifs,
/// imputables, en CHF **ou en devise** — avec leur devise (`None` : CHF). La
/// revue ne retient une règle que si la devise de sa contrepartie est celle de
/// la transaction, ou le CHF (décision du responsable de l'epic du 2026-11-21).
pub async fn rule_account_ids<'e, E>(
    executor: E,
    company_id: i64,
) -> Result<HashMap<i64, Option<String>>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let rows: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, currency FROM accounts \
         WHERE company_id = ? AND active = TRUE AND postable = TRUE",
    )
    .bind(company_id)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)?;
    Ok(rows.into_iter().collect())
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

/// Les transactions **pending** des comptes bancaires `bank_account_ids` dont
/// l'identifiant chez le prestataire (`transaction_id`) est l'un de
/// `transaction_ids`, triées par identifiant — le rejugement d'un paiement par
/// carte groupé (Story 39-11c, amendement A-1) : l'ensemble des lignes en
/// attente du paiement, relu sous les verrous. Lecture ordinaire.
pub async fn pending_by_transaction_ids<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_ids: &[i64],
    transaction_ids: &[String],
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    select_by_transaction_ids(
        executor,
        company_id,
        bank_account_ids,
        transaction_ids,
        "AND status = 'pending'",
    )
    .await
}

/// Les transactions **déjà rapprochées** des comptes bancaires
/// `bank_account_ids` dont l'identifiant chez le prestataire est l'un de
/// `transaction_ids`, triées par identifiant — la garde du paiement par carte
/// groupé (Story 39-11c, revue de code P2, M1) : une ligne du même paiement
/// déjà rapprochée seule. Lecture ordinaire.
pub async fn reconciled_by_transaction_ids<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_ids: &[i64],
    transaction_ids: &[String],
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    select_by_transaction_ids(
        executor,
        company_id,
        bank_account_ids,
        transaction_ids,
        "AND status <> 'pending'",
    )
    .await
}

/// La lecture commune de [`pending_by_transaction_ids`] et de
/// [`reconciled_by_transaction_ids`] ; `status_clause` est un fragment SQL
/// constant (jamais une entrée).
async fn select_by_transaction_ids<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_ids: &[i64],
    transaction_ids: &[String],
    status_clause: &'static str,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if bank_account_ids.is_empty() || transaction_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? {status_clause} AND bank_account_id IN ({}) \
           AND transaction_id IN ({}) ORDER BY id",
        placeholders(bank_account_ids.len()),
        placeholders(transaction_ids.len())
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql).bind(company_id);
    for id in bank_account_ids {
        q = q.bind(*id);
    }
    for id in transaction_ids {
        q = q.bind(id);
    }
    q.fetch_all(executor).await.map_err(map_db_error)
}

/// Un relevé à solde d'ouverture (Story 39-11c, AC-6, écart `balance_gap`).
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct OpeningStatement {
    /// Identifiant de l'import.
    pub id: i64,
    /// Compte bancaire.
    pub bank_account_id: i64,
    /// Premier jour du relevé.
    pub period_from: NaiveDate,
    /// Solde d'ouverture.
    pub opening_balance: Decimal,
}

/// Les relevés **à solde d'ouverture** des comptes bancaires
/// `bank_account_ids` dont le premier jour tombe dans `[from, to]`, triés par
/// premier jour puis identifiant — ceux dont la revue relit la continuité
/// (`bank_imports::continuity_gap`, la règle de la Story 39-11b).
pub async fn opening_statements_in_period<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_ids: &[i64],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<OpeningStatement>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if bank_account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT id, bank_account_id, period_from, opening_balance FROM bank_imports \
         WHERE company_id = ? AND bank_account_id IN ({}) AND opening_balance IS NOT NULL \
           AND period_from BETWEEN ? AND ? \
         ORDER BY period_from, id",
        placeholders(bank_account_ids.len())
    );
    let mut q = sqlx::query_as::<_, OpeningStatement>(&sql).bind(company_id);
    for id in bank_account_ids {
        q = q.bind(*id);
    }
    q.bind(from)
        .bind(to)
        .fetch_all(executor)
        .await
        .map_err(map_db_error)
}

/// Les comptes de **gain** et de **perte de change** de la société
/// (`companies.fx_gain_account_id`, `fx_loss_account_id`), chacun `None` s'il
/// n'est pas réglé — la ligne de différence d'un paiement par carte groupé
/// (Story 39-11c, amendement A-1). Lecture ordinaire.
pub async fn fx_difference_accounts<'e, E>(
    executor: E,
    company_id: i64,
) -> Result<(Option<i64>, Option<i64>), DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let row: Option<(Option<i64>, Option<i64>)> =
        sqlx::query_as("SELECT fx_gain_account_id, fx_loss_account_id FROM companies WHERE id = ?")
            .bind(company_id)
            .fetch_optional(executor)
            .await
            .map_err(map_db_error)?;
    Ok(row.unwrap_or((None, None)))
}
