//! Repository CRUD pour `BankAccount`.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::bank_account::{BankAccount, NewBankAccount};
use crate::errors::{DbError, map_db_error};

const FIND_BY_ID_SQL: &str = "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, created_at, updated_at \
     FROM bank_accounts WHERE id = ?";

/// Crée un nouveau compte bancaire et retourne l'entité persistée.
pub async fn create(pool: &MySqlPool, new: NewBankAccount) -> Result<BankAccount, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.bank_name)
    .bind(&new.iban)
    .bind(&new.qr_iban)
    .bind(new.is_primary)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT bank_accounts".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let account = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant(format!("bank_account {id} introuvable après INSERT")))?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Retourne le compte bancaire principal d'une company (ou None).
pub async fn find_primary(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Option<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Cherche un compte bancaire par id **scopé multi-tenant**.
///
/// Story 8-1b T6.3 (review code Pass 1 H5) : utilisé par le handler
/// `POST /bank-imports/preview` pour valider que le `bankAccountId`
/// fourni par le client appartient bien à la company courante (sinon
/// un attaquant pourrait passer un id appartenant à une autre company
/// et persister ses transactions sous ce bank_account — IDOR).
///
/// Renvoie `None` si le compte n'existe pas OU appartient à une autre
/// company (pas de leak d'existence cross-tenant — pattern KF-002).
pub async fn find_by_id_for_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND id = ? LIMIT 1",
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les comptes bancaires d'une company.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? ORDER BY is_primary DESC, id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Compare le compte existant au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
/// Compare uniquement les champs effectivement écrits par l'UPDATE de
/// `upsert_primary` (`bank_name`, `iban`, `qr_iban`).
fn is_no_op_change(existing: &BankAccount, new: &NewBankAccount) -> bool {
    existing.bank_name == new.bank_name
        && existing.iban == new.iban
        && existing.qr_iban == new.qr_iban
}

/// Upsert du compte bancaire principal (idempotent pour retries).
///
/// Utilise SELECT FOR UPDATE dans une transaction unique pour éviter le
/// TOCTOU entre la lecture et l'écriture.
pub async fn upsert_primary(pool: &MySqlPool, new: NewBankAccount) -> Result<BankAccount, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE LIMIT 1 FOR UPDATE",
    )
    .bind(new.company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    match existing {
        Some(account) => {
            // KF-004 : court-circuit no-op AVANT toute mutation.
            // Note technique : le SELECT FOR UPDATE ci-dessus tient déjà un X-lock
            // sur la row ; tx.rollback() libère ce lock identiquement à tx.commit()
            // côté InnoDB (pas de différence sémantique pour les verrous). Choix
            // rollback() pour cohérence inter-repos + clarté « rien n'a été modifié ».
            if is_no_op_change(&account, &new) {
                tx.rollback().await.map_err(map_db_error)?;
                return Ok(account);
            }

            let rows = sqlx::query(
                "UPDATE bank_accounts SET bank_name = ?, iban = ?, qr_iban = ?, version = version + 1 \
                 WHERE id = ? AND version = ?",
            )
            .bind(&new.bank_name)
            .bind(&new.iban)
            .bind(&new.qr_iban)
            .bind(account.id)
            .bind(account.version)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();

            if rows == 0 {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::OptimisticLockConflict);
            }

            let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
                .bind(account.id)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_error)?;

            tx.commit().await.map_err(map_db_error)?;
            Ok(updated)
        }
        None => {
            let result = sqlx::query(
                "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(new.company_id)
            .bind(&new.bank_name)
            .bind(&new.iban)
            .bind(&new.qr_iban)
            .bind(new.is_primary)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

            let id = i64::try_from(result.last_insert_id())
                .map_err(|_| DbError::Invariant("last_insert_id overflow".into()))?;

            let account = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_error)?;

            tx.commit().await.map_err(map_db_error)?;
            Ok(account)
        }
    }
}

/// Met à jour le `journal_account_id` d'un bank_account scopé multi-tenant
/// **dans une transaction fournie par le caller**.
///
/// Story 8-5a-zero — pose le pattern `bank_account.journal_account_id` qui sera
/// consommé par 8-5a-base (manual match) et 8-5a-bis (split) sans body field
/// `bankLedgerAccountId` (résolu serveur-side via cette colonne).
///
/// **Pass 3 Opus 4.7 — F1''' fix** : la fonction prend `&mut Transaction<MySql>`
/// au lieu d'ouvrir sa propre transaction. Cela permet au handler de partager
/// la tx avec `audit_log::insert_in_tx` et de garantir l'atomicité UPDATE +
/// audit (pattern Story 3-5 + 7-3 + 8-4 — audit_log écrit depuis le route
/// handler, jamais depuis le repo).
///
/// **Pass 1 code review Sonnet 4.6 — P-C1** : retourne `(updated, before)`
/// atomiquement. Le caller utilise `before` comme source `before` de
/// l'audit_log (pas un SELECT séparé hors-FOR UPDATE qui ouvrirait une
/// fenêtre TOCTOU avec un SELECT FOR UPDATE concurrent).
///
/// **Pass 1 code review Sonnet 4.6 — P-H2** : la version est validée AVANT
/// le court-circuit no-op. Un client avec version stale obtient
/// `OptimisticLockConflict` même si `journal_account_id` ne change pas
/// (pas de 200 OK silencieux sur version périmée).
///
/// Optimistic lock sur `version` (cohérent KF-004). Court-circuit no-op : si
/// `journal_account_id` ne change pas ET version match, retourne l'entité
/// inchangée sans bump version. Le caller doit comparer
/// `before.version == updated.version` pour détecter le no-op et skipper
/// l'audit_log.
///
/// Erreurs :
/// - `DbError::NotFound` : bank_account introuvable ou cross-tenant.
/// - `DbError::OptimisticLockConflict` : `expected_version` ne match pas
///   (vérifié AVANT court-circuit no-op).
///
/// **Note transactional safety** : le SELECT FOR UPDATE initial est scopé par
/// `(company_id, id)`, l'UPDATE final est scopé par `(id, company_id, version)`
/// (defense-in-depth M4). Le post-fetch via `FIND_BY_ID_SQL` (filtre par `id`
/// seul) est sûr dans la même tx car la row appartient nécessairement à la
/// company (sinon le SELECT FOR UPDATE aurait retourné None). Cohérent avec
/// le pattern `upsert_primary`.
pub async fn set_journal_account_id_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    journal_account_id: Option<i64>,
    expected_version: i32,
) -> Result<(BankAccount, BankAccount), DbError> {
    // SELECT FOR UPDATE scopé multi-tenant + verrou X sur la row.
    // Le `existing` retourné sert également de source `before` pour
    // l'audit_log côté handler (P-C1 : pas de SELECT séparé qui ouvrirait
    // une fenêtre TOCTOU).
    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, \
         version, created_at, updated_at FROM bank_accounts \
         WHERE company_id = ? AND id = ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let existing = match existing {
        Some(b) => b,
        None => return Err(DbError::NotFound),
    };

    // P-H2 : validation optimistic lock AVANT court-circuit no-op. Un client
    // avec version stale doit obtenir `OptimisticLockConflict` même si la
    // valeur cible (`journal_account_id`) coïncide avec l'état persisté.
    // Sinon il pourrait croire son écriture acceptée alors qu'une mutation
    // concurrente a entre-temps changé la row puis l'a remise à la même
    // valeur — état invisible côté client.
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    // KF-004 court-circuit no-op : pas de bump version, pas d'audit_log côté
    // handler. Le caller (handler) doit checker `before.version ==
    // updated.version` pour détecter le no-op et skipper
    // `audit_log::insert_in_tx`.
    if existing.journal_account_id == journal_account_id {
        return Ok((existing.clone(), existing));
    }

    // M4 defense-in-depth : ajout `AND company_id = ?` au scope de l'UPDATE.
    // Le SELECT FOR UPDATE précédent garantit déjà l'appartenance, mais
    // l'UPDATE explicite multi-tenant rend l'invariant local au statement.
    let rows = sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(journal_account_id)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    // Post-fetch : FIND_BY_ID_SQL filtre par `id` seul, mais le SELECT FOR
    // UPDATE initial + l'UPDATE scopé garantissent qu'on ne peut arriver ici
    // que si la row appartient à la company (cohérent `upsert_primary`).
    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // NOTE : pas de tx.commit() ici — c'est le caller (route handler) qui
    // commit après avoir écrit l'audit_log dans la même tx.
    Ok((updated, existing))
}
