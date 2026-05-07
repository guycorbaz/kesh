//! Repository pour `bank_imports` — entête d'un fichier bancaire importé.
//!
//! La méthode pivot [`create_with_transactions`] prend une `Transaction`
//! en cours pour garantir l'atomicité avec les `bank_transactions` filles
//! ET avec l'entrée audit_log côté handler API (T6.8). Si l'INSERT bulk
//! des transactions échoue (ex. erreur SQL ligne 150 sur 200), le ROLLBACK
//! caller supprime l'entête `bank_imports` ET toutes les transactions
//! déjà insérées (test AC #17 `create_with_transactions_rolls_back_on_constraint_violation`).
//!
//! La contrainte UNIQUE `(company_id, file_hash)` bloque le réimport
//! silencieux d'un même fichier (mappée `409 BANK_IMPORT_DUPLICATE_FILE`
//! côté API via `map_db_error` → `DbError::UniqueConstraintViolation`).

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, QueryBuilder, Transaction};

use crate::entities::bank_import::{BankImport, NewBankImport};
use crate::entities::bank_transaction::{BankTransaction, NewBankTransaction};
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, company_id, bank_account_id, filename, file_hash, source_format, \
     statement_id, period_from, period_to, opening_balance, closing_balance, \
     transaction_count, imported_at, imported_by_user_id";

const TX_COLUMNS: &str = "id, company_id, import_id, bank_account_id, booking_date, value_date, \
     amount, currency, reference, details, end_to_end_id, transaction_id, \
     counterparty_iban, counterparty_name, status, matched_entry_id, \
     auto_match_rejected_at, version, created_at, updated_at";

/// Crée atomiquement un entête `bank_imports` + toutes les
/// `bank_transactions` filles dans une `Transaction` en cours.
///
/// **Atomicité critique** : cette fonction prend `&mut Transaction<MySql>`
/// et **ne commit jamais**. Le caller (handler API) gère le commit pour
/// inclure aussi l'entrée audit_log dans la même transaction.
///
/// Bulk INSERT via `QueryBuilder::push_values` en chunks de **1000**
/// transactions max (limite serveur MySQL `max_allowed_packet`).
/// Pour les fichiers de plus de 1000 lignes, la fonction itère en
/// chunks — c'est extrêmement rare pour CAMT.053 (un statement
/// mensuel typique = 50-300 lignes).
///
/// # Erreurs
///
/// - [`DbError::UniqueConstraintViolation`] si `(company_id, file_hash)`
///   existe déjà → caller mappe vers `409 BANK_IMPORT_DUPLICATE_FILE`.
/// - [`DbError::ForeignKeyViolation`] si `bank_account_id` ou
///   `imported_by_user_id` n'existe pas / appartient à une autre company.
pub async fn create_with_transactions(
    tx: &mut Transaction<'_, MySql>,
    new: NewBankImport,
    transactions: Vec<NewBankTransaction>,
) -> Result<(BankImport, Vec<BankTransaction>), DbError> {
    // Étape 1 : INSERT entête bank_imports.
    let header_result = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, statement_id, \
          period_from, period_to, opening_balance, closing_balance, transaction_count, \
          imported_by_user_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.bank_account_id)
    .bind(&new.filename)
    .bind(&new.file_hash)
    .bind(new.source_format)
    .bind(&new.statement_id)
    .bind(new.period_from)
    .bind(new.period_to)
    .bind(new.opening_balance)
    .bind(new.closing_balance)
    .bind(new.transaction_count)
    .bind(new.imported_by_user_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = header_result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT bank_imports".into(),
        ));
    }
    let import_id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    // Étape 2 : bulk INSERT bank_transactions par chunks de 1000.
    //
    // Pass 1 review H4 : on n'accumule PAS les IDs depuis `last_insert_id()`
    // car (a) la SELECT-back en step 3 est la source de vérité unique et
    // (b) l'arithmétique `first_id + offset` reposait implicitement sur
    // `innodb_autoinc_lock_mode = 1`, qui n'est plus le défaut MariaDB
    // 10.6+ (mode 2 « interleaved » peut casser la séquence). Le SELECT
    // par `(company_id, import_id)` est correct quel que soit le mode.
    for chunk in transactions.chunks(1000) {
        if chunk.is_empty() {
            continue;
        }
        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(
            "INSERT INTO bank_transactions \
             (company_id, import_id, bank_account_id, booking_date, value_date, amount, \
              currency, reference, details, end_to_end_id, transaction_id, counterparty_iban, \
              counterparty_name) ",
        );
        qb.push_values(chunk.iter(), |mut b, t| {
            b.push_bind(t.company_id)
                .push_bind(import_id)
                .push_bind(t.bank_account_id)
                .push_bind(t.booking_date)
                .push_bind(t.value_date)
                .push_bind(t.amount)
                .push_bind(&t.currency)
                .push_bind(&t.reference)
                .push_bind(&t.details)
                .push_bind(&t.end_to_end_id)
                .push_bind(&t.transaction_id)
                .push_bind(&t.counterparty_iban)
                .push_bind(&t.counterparty_name);
        });
        qb.build().execute(&mut **tx).await.map_err(map_db_error)?;
    }

    // Étape 3 : SELECT-back pour récupérer les entités complètes (avec
    // status, version, timestamps DEFAULT). Ordre par id pour stabilité.
    let header = sqlx::query_as::<_, BankImport>(&format!(
        "SELECT {COLUMNS} FROM bank_imports WHERE id = ?"
    ))
    .bind(import_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let inserted_txs = if transactions.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as::<_, BankTransaction>(&format!(
            "SELECT {TX_COLUMNS} FROM bank_transactions \
             WHERE company_id = ? AND import_id = ? \
             ORDER BY id"
        ))
        .bind(new.company_id)
        .bind(import_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?
    };

    Ok((header, inserted_txs))
}

/// Liste paginée des imports bancaires d'une company (multi-tenant scoping).
///
/// Tri par `imported_at DESC` (les plus récents d'abord). Pagination
/// offset/limit cohérente avec `MAX_LIST_LIMIT` (cf. `super::MAX_LIST_LIMIT`).
pub async fn find_by_company_id(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<BankImport>, DbError> {
    let limit = limit.clamp(1, super::MAX_LIST_LIMIT);
    let offset = offset.max(0);

    match bank_account_id {
        Some(bank_id) => sqlx::query_as::<_, BankImport>(&format!(
            "SELECT {COLUMNS} FROM bank_imports \
             WHERE company_id = ? AND bank_account_id = ? \
             ORDER BY imported_at DESC, id DESC \
             LIMIT ? OFFSET ?"
        ))
        .bind(company_id)
        .bind(bank_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error),
        None => sqlx::query_as::<_, BankImport>(&format!(
            "SELECT {COLUMNS} FROM bank_imports \
             WHERE company_id = ? \
             ORDER BY imported_at DESC, id DESC \
             LIMIT ? OFFSET ?"
        ))
        .bind(company_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error),
    }
}

/// Compte total des imports d'une company pour la pagination
/// (review code Pass 1 H5 : `total: 0` hardcoded était un contrat
/// JSON menteur — `BankImportListResponse.total` doit refléter la
/// réalité pour qu'une UI cliente puisse paginer correctement).
pub async fn count_by_company_id(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: Option<i64>,
) -> Result<i64, DbError> {
    match bank_account_id {
        Some(bank_id) => sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM bank_imports \
             WHERE company_id = ? AND bank_account_id = ?",
        )
        .bind(company_id)
        .bind(bank_id)
        .fetch_one(pool)
        .await
        .map_err(map_db_error),
        None => {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bank_imports WHERE company_id = ?")
                .bind(company_id)
                .fetch_one(pool)
                .await
                .map_err(map_db_error)
        }
    }
}

/// Cherche un import existant par `(company_id, file_hash)`. Renvoie
/// `Some(BankImport)` si déjà importé pour cette company (Story 8-3
/// dedup pre-insert), `None` sinon.
///
/// **Multi-tenant safety** : un même hash peut exister sur plusieurs
/// companies sans conflit (l'unique `(company_id, file_hash)` ne porte
/// pas sur le hash seul).
///
/// **Executor générique (M1, Pass 1 review)** : accepte `&MySqlPool`
/// (preview) ou `&mut Transaction<MySql>` (handler create dans une tx
/// ouverte) — pareillement à [`super::bank_transactions::find_in_dedup_window`].
/// Cette généralisation rapproche le code du modèle documenté en
/// spec L11 (« le check applicatif est dans la transaction »).
pub async fn find_by_company_and_hash<'e, E>(
    executor: E,
    company_id: i64,
    file_hash: &str,
) -> Result<Option<BankImport>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankImport>(&format!(
        "SELECT {COLUMNS} FROM bank_imports \
         WHERE company_id = ? AND file_hash = ? \
         LIMIT 1"
    ))
    .bind(company_id)
    .bind(file_hash)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Récupère un import par id, scopé multi-tenant. Renvoie `None` si
/// l'import n'existe pas OU appartient à une autre company (KF-002
/// pattern : pas de leak d'existence cross-tenant).
pub async fn find_by_id_for_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<BankImport>, DbError> {
    sqlx::query_as::<_, BankImport>(&format!(
        "SELECT {COLUMNS} FROM bank_imports \
         WHERE company_id = ? AND id = ? \
         LIMIT 1"
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}
