//! Repository pour `bank_imports` — entête d'un fichier bancaire importé.
//!
//! La méthode pivot [`create_with_transactions`] prend une `Transaction`
//! en cours pour garantir l'atomicité avec les `bank_transactions` filles
//! ET avec l'entrée audit_log côté handler API (T6.8). Si l'INSERT bulk
//! des transactions échoue (ex. erreur SQL ligne 150 sur 200), le ROLLBACK
//! caller supprime l'entête `bank_imports` ET toutes les transactions
//! déjà insérées (test AC #17 `create_with_transactions_rolls_back_on_constraint_violation`).
//!
//! Le réimport d'un même fichier n'est **pas** bloqué par une contrainte de
//! base : l'unicité `(company_id, file_hash)` a été retirée par la Story 8-3
//! (`20260507000001_bank_imports_relax_hash_unique.sql`, index non unique
//! `idx_bank_imports_company_hash`). Le blocage est **applicatif** : la route
//! cherche l'empreinte par [`find_by_company_and_hash`] et refuse en
//! `422 BANK_IMPORT_DUPLICATE_FILE` sans `confirmDuplicateFile=true`.
//!
//! Story 27-1 (#632) : [`delete_unreconciled_in_tx`] supprime un import entier,
//! avec ses transactions, tant qu'aucune n'est rapprochée — ce qui lève aussi ce
//! blocage applicatif pour le même fichier.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, QueryBuilder, Transaction};

use chrono::NaiveDateTime;

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

/// Story 9-2b T3.2.6 — Liste **tous** les imports bancaires d'une company sans
/// pagination ni filtre `bank_account_id`, pour l'export global ZIP.
///
/// Distincte de [`find_by_company_id`] (paginée + filtrée). Tri stable `id ASC`
/// (les `imported_at` peuvent être identiques sur des imports batch).
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<BankImport>, DbError> {
    sqlx::query_as::<_, BankImport>(&format!(
        "SELECT {COLUMNS} FROM bank_imports \
         WHERE company_id = ? \
         ORDER BY id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
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

/// Issue de [`delete_unreconciled_in_tx`] (Story 27-1, #632).
#[derive(Debug)]
pub enum DeleteBankImportOutcome {
    /// L'import et ses transactions sont supprimés.
    Deleted(DeletedBankImport),
    /// Import inconnu, ou d'une autre société.
    NotFound,
    /// Au moins une transaction est rapprochée : rien n'est supprimé.
    HasReconciled {
        /// Transactions rapprochées au sens de [`is_reconciled`].
        reconciled_count: i64,
        /// Transactions de l'import, comptées sous verrou.
        transaction_count: i64,
    },
}

/// Ce qu'une suppression réussie a retiré, lu sous verrou.
#[derive(Debug)]
pub struct DeletedBankImport {
    /// L'entête tel qu'il était, lu sous verrou.
    pub import: BankImport,
    /// Lignes supprimées (par la cascade), comptées sous verrou.
    pub transaction_count: i64,
    /// Dont rejetées (`auto_match_rejected_at` non nul).
    pub rejected_count: i64,
}

/// **« Rapprochée »** — la définition fermée de la Story 27-1 (AC1) : une
/// transaction bancaire est rapprochée si et seulement si `status <> 'pending'`
/// **ou** `matched_entry_id IS NOT NULL`.
///
/// Elle s'appuie sur l'inventaire clos des écrivains de `bank_transactions` :
/// les cinq chemins qui rapprochent posent les deux colonnes ensemble, le seul
/// qui les retire (`reconciliation_cancel::cancel_in_tx`) les retire ensemble,
/// et le **rejet** (`reject_batch`) ne touche aucune des deux — une transaction
/// rejetée n'est donc **pas** rapprochée. La disjonction couvre en plus deux
/// états qu'aucun chemin vivant ne produit (`reconciled` sans lien, `pending`
/// avec lien).
///
/// ⚠️ **Écrite deux fois, et seulement deux** : ici, et côté écran dans
/// `frontend/src/lib/features/bank-import/bank-import-deletion.ts`
/// (`isReconciled`). L'écran n'est qu'une commodité : le serveur seul décide.
pub fn is_reconciled(status: &str, matched_entry_id: Option<i64>) -> bool {
    status != "pending" || matched_entry_id.is_some()
}

/// Supprime l'import `import_id` de la société `company_id` **avec toutes ses
/// transactions**, à condition qu'**aucune** ne soit rapprochée (au sens de
/// [`is_reconciled`]) — Story 27-1, #632. Un import sans transaction est donc
/// toujours supprimable.
///
/// **Ne commit pas** (comme [`create_with_transactions`]) : l'appelant y ajoute
/// la trace d'audit `bank_import.deleted` et conclut. Sur `NotFound` et
/// `HasReconciled`, rien n'a été écrit ; l'appelant doit `rollback` sans
/// tarder, la transaction tenant l'entête et les lignes sous `FOR UPDATE`.
///
/// # Ordre des verrous, et pourquoi
///
/// Aucune lecture ordinaire avant le premier verrou (règle de la 15-8a : elle
/// ouvrirait la vue cohérente avant l'attente, et la suite ne verrait pas le
/// commit d'une acceptation concurrente) :
///
/// 1. l'entête, `FOR UPDATE` (scopé société) — absent ⇒ `NotFound` ;
/// 2. les transactions de l'import, `FOR UPDATE`, **par identifiant
///    croissant**, par l'index `idx_bank_transactions_import` (sous
///    REPEATABLE READ, un plan par l'index `company_id` verrouillerait toutes
///    les lignes de la société examinées). Une lecture **verrouillante** lit la
///    version validée la plus récente : si une acceptation a écrit avant, on
///    attend son commit puis on la voit rapprochée ;
/// 3. le comptage des rapprochées se fait **sur ce résultat**, pas par une
///    seconde requête — > 0 ⇒ `HasReconciled`, sans rien écrire ;
/// 4. `DELETE` de l'entête (`rows_affected() != 1` ⇒ `NotFound`) ; les
///    transactions partent par `fk_bank_transactions_import … ON DELETE
///    CASCADE`, sans `DELETE` explicite.
///
/// Le verrou de l'entête n'est pas nécessaire à la correction (le contrôle de
/// `rows_affected()` suffit contre une seconde suppression) : il fixe l'ordre
/// import → transactions et réduit la fréquence des interblocages. Un
/// interblocage (1213) reste possible contre un lot d'acceptation, qui
/// verrouille ses transactions dans l'ordre des propositions : `map_db_error`
/// le garde en `DbError::Sqlx`, que reconnaît `crate::retry::is_deadlock_error`,
/// et la route le rejoue. Le lot de rejet, lui, verrouille par identifiant
/// croissant (Story 27-1, AC4-bis) : pas de cycle avec lui.
///
/// Conséquence connue, sans cycle : quand l'import supprimé est le plus récent
/// de l'installation, le verrou de trou de l'étape 2 sur `import_id` va
/// jusqu'au supremum d'un index non scopé par société — toute création
/// d'import, toutes sociétés confondues, attend alors la fin de la suppression.
///
/// Ce raisonnement sur les lectures verrouillantes vaut sous **MariaDB 10.11**
/// (`docker-compose.yml`, CI). Avec `innodb_snapshot_isolation` (actif par
/// défaut depuis 11.6), une lecture verrouillante qui rencontre une version
/// postérieure à une vue ouverte rend `ER_CHECKREAD` (1020) — sans effet sur le
/// code nominal (aucune vue n'est ouverte avant les verrous), à revoir à la
/// montée de version.
///
/// # Angles morts assumés
///
/// - Un import créé pendant la suppression n'est pas concerné (autre
///   identifiant) ; deux imports du même fichier se suppriment un par un.
/// - Un lot d'acceptation ou de rejet préparé avant la suppression : validée
///   avant son pré-vol, le lot entier rend `400 VALIDATION_ERROR` ; validée
///   après, la ligne sort en `failed[]` (`BANK_TRANSACTION_NOT_FOUND` ou
///   `RECONCILIATION_ALREADY_RECONCILED`). Non corrigé ici (#634).
/// - Une création concurrente du même compte juge sa détection de doublons sur
///   une vue fixée avant la suppression : elle peut ignorer des lignes qui
///   n'existent plus. Fenêtre de quelques millisecondes, non fermée.
/// - La restauration supprime les lignes de `bank_transactions` puis celles de
///   `bank_imports` (`backup.rs`, boucle sur `TABLES_TO_TRUNCATE`), la
///   suppression verrouille l'entête puis les lignes : un interblocage est
///   possible, rejoué côté suppression par `retry_app_on_deadlock`. La
///   restauration n'étant exclusive que des autres imports (Story 15-12b), une
///   suppression concurrente d'une restauration n'est pas gardée au-delà.
/// - Une sauvegarde prise pendant une suppression peut porter des transactions
///   orphelines (#633 ; export global CSV, #465).
pub async fn delete_unreconciled_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    import_id: i64,
) -> Result<DeleteBankImportOutcome, DbError> {
    // Étape 1 — l'entête, sous verrou : première instruction de la transaction.
    let import = sqlx::query_as::<_, BankImport>(&format!(
        "SELECT {COLUMNS} FROM bank_imports WHERE id = ? AND company_id = ? FOR UPDATE"
    ))
    .bind(import_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let Some(import) = import else {
        return Ok(DeleteBankImportOutcome::NotFound);
    };

    // Étape 2 — les transactions, sous verrou, par identifiant croissant.
    let rows: Vec<(i64, String, Option<i64>, Option<NaiveDateTime>)> = sqlx::query_as(
        "SELECT id, status, matched_entry_id, auto_match_rejected_at \
         FROM bank_transactions FORCE INDEX (idx_bank_transactions_import) \
         WHERE company_id = ? AND import_id = ? ORDER BY id FOR UPDATE",
    )
    .bind(company_id)
    .bind(import_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Étape 3 — comptage sur le résultat de la lecture verrouillante.
    let transaction_count = rows.len() as i64;
    let reconciled_count = rows
        .iter()
        .filter(|(_, status, matched, _)| is_reconciled(status, *matched))
        .count() as i64;
    if reconciled_count > 0 {
        return Ok(DeleteBankImportOutcome::HasReconciled {
            reconciled_count,
            transaction_count,
        });
    }
    let rejected_count = rows
        .iter()
        .filter(|(_, _, _, rejected)| rejected.is_some())
        .count() as i64;

    // Étape 4 — l'entête ; les transactions suivent par la cascade.
    let deleted = sqlx::query("DELETE FROM bank_imports WHERE id = ? AND company_id = ?")
        .bind(import_id)
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    if deleted.rows_affected() != 1 {
        return Ok(DeleteBankImportOutcome::NotFound);
    }

    Ok(DeleteBankImportOutcome::Deleted(DeletedBankImport {
        import,
        transaction_count,
        rejected_count,
    }))
}
