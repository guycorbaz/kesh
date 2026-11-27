//! Repository pour `bank_transactions` — transactions bancaires
//! individuelles d'un import.
//!
//! Story 8-1b ne fait que lire les transactions (la création est faite
//! atomiquement par `bank_imports::create_with_transactions`). Story 8-4
//! ajoutera `update_status`, `match_with_journal_entry` etc.
//!
//! **Multi-tenant double-scope** : `list_by_import` filtre par
//! `(company_id, import_id)` même si `import_id` est unique — le scoping
//! par `company_id` reste appliqué pour Pattern 1 KF-002 (pas de leak
//! même si un attaquant devine un `import_id` valide d'une autre
//! company).

use chrono::NaiveDate;
use sqlx::MySql;
use sqlx::mysql::MySqlPool;

use crate::entities::bank_transaction::BankTransaction;
use crate::errors::{DbError, map_db_error};

/// Colonnes de `bank_transactions` qui hydratent [`BankTransaction`] —
/// **source unique** depuis la Story 38-11b : `bank_imports` et
/// `reconciliation` la reprennent (deux copies divergeaient en silence à
/// chaque colonne ajoutée ; `FromRow` ne rougit qu'à l'exécution).
pub(crate) const COLUMNS: &str = "id, company_id, import_id, bank_account_id, booking_date, value_date, \
     amount, currency, reference, details, end_to_end_id, transaction_id, \
     counterparty_iban, counterparty_name, provider_type, counterparty_email, status, matched_entry_id, \
     auto_match_rejected_at, version, created_at, updated_at";

/// Story 9-2b T3.2.7 — Liste **toutes** les transactions bancaires d'une
/// company sans filtre `import_id`, pour l'export global ZIP. Tri stable
/// `id ASC` (ordre d'insertion).
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<BankTransaction>, DbError> {
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {COLUMNS} FROM bank_transactions \
         WHERE company_id = ? \
         ORDER BY id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les transactions d'un import donné, scopées multi-tenant.
///
/// Tri par `id` (= ordre d'insertion = ordre du fichier source pour
/// CAMT.053 où le parser émet les `<Ntry>` dans l'ordre). Renvoie un
/// `Vec` vide si l'import n'existe pas / appartient à une autre company.
pub async fn list_by_import(
    pool: &MySqlPool,
    company_id: i64,
    import_id: i64,
) -> Result<Vec<BankTransaction>, DbError> {
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND import_id = ? \
         ORDER BY id"
    ))
    .bind(company_id)
    .bind(import_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Charge les transactions existantes dans la fenêtre
/// `[period_from, period_to]` (bornes incluses) pour le compte
/// donné, scopées multi-tenant (Story 8-3 T3.1).
///
/// Utilisé par le handler `bank_imports::create` pour la détection de
/// doublons ligne-par-ligne (cf. `kesh_core::bank_imports::detect_duplicate_lines`).
///
/// **Multi-tenant safety (KF-002 Pattern 1)** : filtrage systématique
/// par `(company_id, bank_account_id)` — les transactions cross-tenant
/// ne sont jamais retournées.
///
/// **Index** : utilise
/// `idx_bank_transactions_company_account_date (company_id, bank_account_id, booking_date)`
/// (créé en migration `20260504000001_bank_imports.sql`).
///
/// **Executor générique** : accepte `&MySqlPool` (preview) ou
/// `&mut Transaction<MySql>` (handler create dans une tx ouverte).
pub async fn find_in_dedup_window<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    period_from: NaiveDate,
    period_to: NaiveDate,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    // L-4 (Pass 2 review) — bornes inversées (`from > to`) résulteraient
    // en BETWEEN MariaDB silencieusement vide ; surfacing en debug.
    debug_assert!(
        period_from <= period_to,
        "find_in_dedup_window: period_from {} > period_to {}",
        period_from,
        period_to
    );
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? \
           AND booking_date BETWEEN ? AND ?"
    ))
    .bind(company_id)
    .bind(bank_account_id)
    .bind(period_from)
    .bind(period_to)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// Une transaction déjà sur le compte, vue par son identifiant de prestataire
/// ([`existing_transactions_by_ids`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingTransaction {
    /// Identifiant de transaction, tel que stocké.
    pub transaction_id: String,
    /// Montant signé.
    pub amount: rust_decimal::Decimal,
    /// Date de comptabilisation.
    pub booking_date: NaiveDate,
}

/// Les transactions du compte bancaire dont l'identifiant est parmi
/// `transaction_ids`, avec leur montant et leur date (Story 39-11a2).
///
/// Lue par le cœur commun des imports de prestataire
/// (`kesh-api`, `routes/provider_imports.rs`, Story 39-14) pour **tous** les
/// prestataires : il juge un doublon par (identifiant, sens) quand la ligne
/// importée porte un sens — un même identifiant Wise peut porter un débit
/// **et** un crédit (réserve de carte expirée, vérification de carte), le
/// sens se lisant au signe du montant —, par identifiant seul sinon.
///
/// Contrat : lecture **dans la transaction de l'import**, sous les verrous
/// nommés des comptes visés (deux imports du même compte se sérialisent, et la
/// lecture voit le relevé concurrent commité ou absent) ; par l'index
/// `idx_bank_transactions_account_txid` ; comparaison selon la collation de
/// la colonne (sans casse) ; valeurs **telles que stockées** ; par paquets de
/// 500 identifiants ; une ligne rendue par transaction (un identifiant peut
/// donc revenir plusieurs fois). Ordre non garanti.
pub async fn existing_transactions_by_ids(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    bank_account_id: i64,
    transaction_ids: &[String],
) -> Result<Vec<ExistingTransaction>, DbError> {
    let mut found = Vec::new();
    for chunk in transaction_ids.chunks(500) {
        let mut qb: sqlx::QueryBuilder<MySql> = sqlx::QueryBuilder::new(
            "SELECT transaction_id, amount, booking_date FROM bank_transactions WHERE company_id = ",
        );
        qb.push_bind(company_id)
            .push(" AND bank_account_id = ")
            .push_bind(bank_account_id)
            .push(" AND transaction_id IN (");
        let mut separated = qb.separated(", ");
        for id in chunk {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        let rows: Vec<(Option<String>, rust_decimal::Decimal, NaiveDate)> = qb
            .build_query_as()
            .fetch_all(&mut *conn)
            .await
            .map_err(map_db_error)?;
        found.extend(rows.into_iter().filter_map(|(id, amount, booking_date)| {
            id.map(|transaction_id| ExistingTransaction {
                transaction_id,
                amount,
                booking_date,
            })
        }));
    }
    Ok(found)
}

/// Les identifiants de transaction, parmi `transaction_ids`, que le compte
/// bancaire porte déjà, chacun une fois (Story 38-11b, AC-6) : lecture par
/// identifiant seul, sans sens ni montant.
///
/// Même contrat qu'[`existing_transactions_by_ids`], sur laquelle elle est
/// écrite (Story 39-13). Elle n'a plus d'appelant dans `kesh-api` depuis la
/// Story 39-14 (le cœur commun lit `existing_transactions_by_ids`) ; elle
/// reste publique parce qu'une branche d'epic non encore fusionnée l'appelle.
pub async fn existing_transaction_ids(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    bank_account_id: i64,
    transaction_ids: &[String],
) -> Result<std::collections::HashSet<String>, DbError> {
    Ok(
        existing_transactions_by_ids(conn, company_id, bank_account_id, transaction_ids)
            .await?
            .into_iter()
            .map(|t| t.transaction_id)
            .collect(),
    )
}
