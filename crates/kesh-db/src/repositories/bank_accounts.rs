//! Repository CRUD pour `BankAccount`.
//!
//! ## Sémantique du flag `archived` (Story v014-1, FINDING-6 Pass 3 Opus)
//!
//! - [`find_primary`] : filtre `archived = FALSE` (garantit que les routes
//!   PDF QR Bill ne retournent jamais un primary archivé).
//! - [`find_by_id_for_company`] : **ne filtre PAS** `archived` (contrat
//!   intentionnel — c'est aux call sites de mutation post-archivage
//!   d'inspecter `bank_account.archived` après le find).
//! - [`list_by_company`] : filtre `archived = FALSE` quand
//!   `include_archived=false` (défaut), retourne tout quand `true`
//!   (export ZIP souveraineté + toggle UI « afficher archivés »).
//! - [`set_journal_account_id_for_company`], [`update_for_company`],
//!   [`archive_for_company`] : SELECT FOR UPDATE filtre `archived = FALSE`
//!   → `DbError::NotFound` (anti-énumération KF-002) si row archivée.
//! - [`count_transactions_for_bank_account`] : utilisé par DELETE pour
//!   refuser 412 BANK_ACCOUNT_HAS_TRANSACTIONS si historique présent.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::bank_account::{BankAccount, NewBankAccount};
use crate::errors::{DbError, map_db_error};

const FIND_BY_ID_SQL: &str = "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, created_at, updated_at \
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
///
/// Story v014-1 (FINDING-1 Pass 3 Opus) — filtre `archived = FALSE` pour ne
/// pas retourner un primary archivé. Cross-fichier : utilisé par
/// `routes/invoice_pdf.rs` pour générer le QR Bill — sinon le PDF utiliserait
/// un IBAN archivé alors que le compte est invisible côté UI.
pub async fn find_primary(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Option<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE LIMIT 1",
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
///
/// **Story v014-1 (FINDING-6 Pass 3 Opus)** — la fonction ne filtre PAS
/// `archived` (contrat intentionnel). Les call sites doivent inspecter
/// `bank_account.archived` post-find et rejeter avec `BankAccountNotFound`
/// (anti-énumération KF-002) pour les mutations post-archivage.
pub async fn find_by_id_for_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND id = ? LIMIT 1",
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les comptes bancaires d'une company.
///
/// Story v014-1 — paramètre `include_archived` :
/// - `false` (défaut UI/handler GET) : filtre `archived = FALSE`.
/// - `true` (export ZIP souveraineté + toggle UI « afficher archivés ») :
///   retourne **tous** les comptes, archivés inclus.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    include_archived: bool,
) -> Result<Vec<BankAccount>, DbError> {
    let sql = if include_archived {
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? ORDER BY is_primary DESC, id"
    } else {
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND archived = FALSE ORDER BY is_primary DESC, id"
    };
    sqlx::query_as::<_, BankAccount>(sql)
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
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE LIMIT 1 FOR UPDATE",
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
/// **Story v014-1 (FINDING-2 Pass 3 Opus)** — le SELECT FOR UPDATE filtre
/// `archived = FALSE` ; une row archivée → `DbError::NotFound` (handler
/// retourne `AppError::BankAccountNotFound` anti-énumération). Un compte
/// archivé est immuable hors `un-archive` workflow (L1 v0.1 — pas de
/// restoration UI).
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
         version, archived, created_at, updated_at FROM bank_accounts \
         WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
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

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // NOTE : pas de tx.commit() ici — c'est le caller (route handler) qui
    // commit après avoir écrit l'audit_log dans la même tx.
    Ok((updated, existing))
}

/// Met à jour TOUS les champs métier d'un bank_account (PUT v014-1).
///
/// Différent de [`set_journal_account_id_for_company`] qui ne touche que
/// `journal_account_id` (PATCH legacy 8-5a-zero) : ce helper écrit
/// `bank_name`, `iban`, `qr_iban`, `is_primary`, `journal_account_id`.
///
/// La transition `is_primary` (un compte devient primary alors qu'un autre
/// l'était) est **déléguée au caller** : ce repo flippe simplement
/// `is_primary` en accord avec le payload. Le caller doit appeler
/// [`flip_primary_off_for_company`] sur l'ancien primary AVANT (ou APRÈS,
/// indifférent dans la même tx) pour préserver l'invariant « au plus 1
/// primary par company ».
///
/// Filtre `archived = FALSE` au SELECT FOR UPDATE (un PUT sur compte archivé
/// → 404 anti-énumération).
///
/// Retourne `(updated, before)` cohérent avec `set_journal_account_id_for_company`.
pub async fn update_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    new: &NewBankAccount,
    new_journal_account_id: Option<i64>,
    expected_version: i32,
) -> Result<(BankAccount, BankAccount), DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, \
         version, archived, created_at, updated_at FROM bank_accounts \
         WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
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

    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE bank_accounts SET bank_name = ?, iban = ?, qr_iban = ?, is_primary = ?, journal_account_id = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(&new.bank_name)
    .bind(&new.iban)
    .bind(&new.qr_iban)
    .bind(new.is_primary)
    .bind(new_journal_account_id)
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

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    Ok((updated, existing))
}

/// Soft-delete (archive) un bank_account.
///
/// Story v014-1 — passe `archived = TRUE` + bump version. Préserve audit +
/// traçabilité (pas de DELETE physique). Filtre `archived = FALSE` au SELECT
/// FOR UPDATE : un DELETE sur compte déjà archivé → 404 anti-énumération
/// (idempotence non-supportée v0.1, L1 pas de restoration UI).
pub async fn archive_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
) -> Result<(BankAccount, BankAccount), DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, \
         version, archived, created_at, updated_at FROM bank_accounts \
         WHERE company_id = ? AND id = ? AND archived = FALSE FOR UPDATE",
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

    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE bank_accounts SET archived = TRUE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
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

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    Ok((updated, existing))
}

/// Helper transition primary : flip un éventuel autre primary à `is_primary=FALSE`
/// (sans bump audit log — c'est le caller qui décide).
///
/// Story v014-1 (FINDING-3 Pass 3 Opus) — politique uniforme POST + PUT :
/// quand un client crée/édite un compte avec `is_primary=true` alors qu'un
/// autre primary existe, on flip silencieusement l'ancien (transition).
///
/// Filtre `archived = FALSE` + exclut `excluded_id` (le compte qu'on est en
/// train d'updater — ne pas se flip soi-même).
///
/// **F15 Pass 1 code review** : retourne `Option<(updated, existing)>` cohérent
/// avec `update_for_company` et `archive_for_company`. Le `existing` est le
/// snapshot pré-flip (capturé par le SELECT FOR UPDATE) — le caller l'utilise
/// directement comme source `before` de l'audit log `primary_transition` sans
/// arithmétique fragile sur `version`.
pub async fn flip_primary_off_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    excluded_id: i64,
) -> Result<Option<(BankAccount, BankAccount)>, DbError> {
    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, \
         version, archived, created_at, updated_at FROM bank_accounts \
         WHERE company_id = ? AND is_primary = TRUE AND archived = FALSE AND id != ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(excluded_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let Some(old_primary) = existing else {
        return Ok(None);
    };

    let rows = sqlx::query(
        "UPDATE bank_accounts SET is_primary = FALSE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(old_primary.id)
    .bind(company_id)
    .bind(old_primary.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Race : un autre tx a flippé le primary entre notre SELECT FOR UPDATE
        // (qui prend le lock) et notre UPDATE. Impossible en pratique car
        // FOR UPDATE bloque. Mais défense en profondeur.
        return Err(DbError::OptimisticLockConflict);
    }

    let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(old_primary.id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;

    Ok(Some((updated, old_primary)))
}

/// Compte le nombre de `bank_transactions` associées à un bank_account
/// **dans une transaction fournie par le caller** (F2 Pass 1 code review —
/// élimine la fenêtre TOCTOU entre count hors-tx et archive dans tx).
///
/// Story v014-1 (AC#8) — utilisé par DELETE pour refuser 412
/// BANK_ACCOUNT_HAS_TRANSACTIONS si une transaction (pending ou reconciled)
/// existe. `bank_transactions` n'a pas de colonne `archived` — comptage
/// inconditionnel (auditabilité CO Art. 958f).
pub async fn count_transactions_for_bank_account(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    bank_account_id: i64,
) -> Result<i64, DbError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bank_transactions WHERE bank_account_id = ? AND company_id = ?",
    )
    .bind(bank_account_id)
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(row.0)
}

/// Compte le nombre d'autres bank_accounts non-archivés du même company
/// (exclut le compte courant) **dans une transaction fournie par le caller**.
/// Utilisé par DELETE pour décider si on autorise l'archivage d'un primary
/// (autorisé seulement si c'est le dernier compte non-archivé — AC#10).
pub async fn count_other_active_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    excluded_id: i64,
) -> Result<i64, DbError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bank_accounts \
         WHERE company_id = ? AND archived = FALSE AND id != ?",
    )
    .bind(company_id)
    .bind(excluded_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(row.0)
}

/// Acquiert un advisory lock sentinel sur la row `companies.id` pour
/// serializer les mutations CRUD bank_accounts d'un même tenant (mitigation
/// L5 race condition primary applicatif-only, FINDING-9 Pass 3 Opus).
///
/// À appeler au début des handlers POST/PUT avant tout autre SELECT FOR
/// UPDATE sur `bank_accounts`. Le lock est libéré au commit/rollback de la
/// tx. SELECT … FOR UPDATE sur companies n'impacte pas les autres handlers
/// qui ne mutent pas `companies` (lecture seule via `read uncommitted` ou
/// SELECT sans FOR UPDATE).
pub async fn acquire_company_sentinel_lock(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<(), DbError> {
    let _: Option<(i64,)> = sqlx::query_as("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    Ok(())
}

/// Liste les comptes bancaires avec leurs soldes calculés + date dernière
/// transaction depuis `journal_entry_lines`.
///
/// Story v014-1 T5 (FINDING-10 Pass 3 Opus — périmètre du calcul) :
/// - **Pas de filtre de status** : `journal_entries` n'a pas de colonne
///   `status` ; toute écriture insérée est par construction validée (la
///   double-partie est balanced et toutes les FK existent — il n'y a pas
///   de notion de draft v0.1). Donc toutes les `journal_entry_lines`
///   participent au solde.
/// - Pas de filtre `fiscal_year_id` : « solde depuis création » v0.1.
/// - Pas de rollup hiérarchique (uniquement sur `journal_account_id` exact —
///   pas sur les enfants `parent_id`). Tooltip helper text AC#27 recommande
///   de lier le bank_account au sous-compte spécifique.
/// - **F13 Pass 1 code review (AC#30)** : retourne aussi `last_transaction_date`
///   = `MAX(je.entry_date)` agrégé sur le `journal_account_id` du compte.
///   `None` si journal_account_id NULL OU aucune écriture sur ce compte.
///
/// Retourne `Vec<(BankAccount, Option<Decimal>, Option<NaiveDate>)>` — le
/// solde et la date sont `None` si le bank_account n'a pas de
/// `journal_account_id` configuré (`L4`).
pub async fn list_by_company_with_balances(
    pool: &MySqlPool,
    company_id: i64,
    include_archived: bool,
) -> Result<
    Vec<(
        BankAccount,
        Option<rust_decimal::Decimal>,
        Option<chrono::NaiveDate>,
    )>,
    DbError,
> {
    let accounts = list_by_company(pool, company_id, include_archived).await?;

    if accounts.is_empty() {
        return Ok(Vec::new());
    }

    let account_ids: Vec<i64> = accounts
        .iter()
        .filter_map(|b| b.journal_account_id)
        .collect();

    if account_ids.is_empty() {
        return Ok(accounts.into_iter().map(|b| (b, None, None)).collect());
    }

    let mut builder = sqlx::QueryBuilder::<MySql>::new(
        "SELECT jel.account_id, \
                COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) AS balance, \
                MAX(je.entry_date) AS last_entry_date \
         FROM journal_entry_lines jel \
         INNER JOIN journal_entries je ON jel.entry_id = je.id \
         WHERE je.company_id = ",
    );
    builder.push_bind(company_id);
    builder.push(" AND jel.account_id IN (");
    let mut sep = builder.separated(", ");
    for id in &account_ids {
        sep.push_bind(*id);
    }
    builder.push(") GROUP BY jel.account_id");

    let rows: Vec<(i64, rust_decimal::Decimal, Option<chrono::NaiveDate>)> = builder
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    use std::collections::HashMap;
    let agg_by_account: HashMap<i64, (rust_decimal::Decimal, Option<chrono::NaiveDate>)> = rows
        .into_iter()
        .map(|(aid, bal, dt)| (aid, (bal, dt)))
        .collect();

    Ok(accounts
        .into_iter()
        .map(|b| match b.journal_account_id {
            Some(aid) => match agg_by_account.get(&aid) {
                Some((balance, date)) => (b, Some(*balance), *date),
                // journal_account_id existe mais aucune ligne → solde = 0.00,
                // date = None.
                None => (b, Some(rust_decimal::Decimal::ZERO), None),
            },
            None => (b, None, None),
        })
        .collect())
}
