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
///
/// **Ordre des erreurs** (Story 15-5b, AC12) : `DbError::NotFound` →
/// `DbError::OptimisticLockConflict` → court-circuit no-op (valeur inchangée,
/// rien n'est contrôlé) → `DbError::AccountsNotPostable` (nouveau compte lié
/// actif et non imputable).
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

    // Story 15-5b (AC12, choix C10) — postabilité du nouveau compte lié, sous
    // le verrou de la ligne et seulement si la valeur change (court-circuit
    // no-op ci-dessus) : un compte lié devenu non imputable après coup reste
    // accepté tel quel (D-A0).
    if let Some(account_id) = journal_account_id {
        super::accounts::ensure_postable_if_active_in_tx(tx, company_id, account_id).await?;
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
///
/// **Ordre des erreurs** (Story 15-5b, AC12) : `DbError::NotFound` (compte
/// bancaire inconnu, d'une autre société ou archivé) →
/// `DbError::OptimisticLockConflict` (version périmée) →
/// `DbError::AccountsNotPostable` (nouveau compte lié actif et non imputable,
/// contrôlé seulement s'il change). Un refus abandonne la transaction du
/// caller : une démotion de l'ancien principal faite dans la même transaction
/// est annulée avec elle.
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

    // Story 15-5b (AC12, choix C10) — postabilité du compte lié, sous le
    // verrou, **seulement s'il change** et n'est pas `NULL` : un PUT qui
    // renvoie le compte en place, devenu non imputable après coup, passe.
    if new_journal_account_id != existing.journal_account_id
        && let Some(account_id) = new_journal_account_id
    {
        super::accounts::ensure_postable_if_active_in_tx(tx, company_id, account_id).await?;
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

/// Soldes d'un compte bancaire, tels que la liste les rend (Story v014-1,
/// Story 25-6-a #389).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BankAccountBalances {
    /// **Solde comptable** du compte de grand livre lié — `SUM(debit) −
    /// SUM(credit)`, toutes dates. `None` sans compte lié ; `0` si le compte lié
    /// n'a aucune ligne.
    pub current_balance: Option<rust_decimal::Decimal>,
    /// `MAX(entry_date)` des lignes du compte lié.
    pub last_transaction_date: Option<chrono::NaiveDate>,
    /// Solde de clôture du **dernier relevé** importé qui en porte un (CAMT ;
    /// un import CSV n'en a jamais) — `period_to` le plus récent, départagé par
    /// `imported_at` puis `id`.
    pub statement_closing_balance: Option<rust_decimal::Decimal>,
    /// `period_to` de ce relevé.
    pub statement_date: Option<chrono::NaiveDate>,
    /// Solde comptable du compte lié **à `statement_date` inclus**, corrigé des
    /// dates de valeur (voir [`list_by_company_with_balances`]). `None` sans
    /// relevé, sans compte lié, ou quand **plusieurs** comptes bancaires
    /// partagent le compte de grand livre (son solde ne s'attribue à aucun).
    pub ledger_balance_at_statement: Option<rust_decimal::Decimal>,
}

/// Agrégats d'un compte bancaire, lus en une requête (voir
/// [`list_by_company_with_balances`]).
#[derive(Debug, sqlx::FromRow)]
struct BalancesRow {
    bank_account_id: i64,
    balance: Option<rust_decimal::Decimal>,
    last_entry_date: Option<chrono::NaiveDate>,
    closing_balance: Option<rust_decimal::Decimal>,
    period_to: Option<chrono::NaiveDate>,
    ledger_at: Option<rust_decimal::Decimal>,
    booked_before_entered_after: Option<rust_decimal::Decimal>,
    booked_after_entered_before: Option<rust_decimal::Decimal>,
    sharing: Option<i64>,
}

/// Liste les comptes bancaires avec leurs soldes calculés + date dernière
/// transaction depuis `journal_entry_lines`, et le dernier relevé.
///
/// Story v014-1 T5 (FINDING-10 Pass 3 Opus — périmètre du calcul) :
/// - **Pas de filtre de status** : `journal_entries` n'a pas de colonne
///   `status` ; toute écriture insérée est par construction validée. Donc
///   toutes les `journal_entry_lines` participent au solde.
/// - Pas de filtre `fiscal_year_id` : « solde depuis création ».
/// - Pas de rollup hiérarchique (uniquement sur `journal_account_id` exact).
/// - `last_transaction_date` = `MAX(je.entry_date)` sur le compte lié.
///
/// # Le dernier relevé et l'écart (Story 25-6-a, #389)
///
/// Le solde comptable n'est **pas** le solde bancaire. Pour les confronter, la
/// liste rend le solde de clôture du dernier relevé portant un solde, et le
/// solde comptable **à sa date**. ⚠️ Le relevé reflète les mouvements par date
/// de **comptabilisation bancaire** (`booking_date`), alors que les écritures de
/// rapprochement sont datées à la date de **valeur** : le solde à la date du
/// relevé est donc **corrigé** des transactions rapprochées dont les deux dates
/// tombent de part et d'autre de `period_to` — `+` le mouvement que leur
/// écriture porte sur le compte lié quand la banque l'a comptabilisé avant (ou
/// le jour même) et l'écriture après, `−` dans le cas inverse. On somme la
/// **ligne de l'écriture sur le compte lié actuel**, pas `amount` : une écriture
/// posée sur un ancien compte lié sort d'elle-même de la correction. Une
/// transaction **non rapprochée** n'y entre pas — son absence du grand livre est
/// l'écart à montrer.
///
/// ⚠️ **Invariant supposé : une transaction, une écriture.** Les cinq chemins de
/// rapprochement (`kesh-api/src/routes/reconciliation.rs`) lient chacun une
/// écriture qu'ils viennent de créer ; aucune écriture n'est rapprochée de
/// plusieurs transactions. Si cela changeait, la correction ajouterait la ligne
/// entière de l'écriture là où seule la part d'une transaction franchit la date
/// du relevé (revue P1, lentille B).
///
/// # Forme
///
/// Une requête agrégée **par compte bancaire** ; chaque somme dans sa propre
/// table dérivée ou sous-requête corrélée. ⛔ Ne jamais joindre
/// `bank_transactions` dans le même `FROM` que les lignes du grand livre : le
/// produit multiplierait ces lignes et fausserait les sommes.
pub async fn list_by_company_with_balances(
    pool: &MySqlPool,
    company_id: i64,
    include_archived: bool,
) -> Result<Vec<(BankAccount, BankAccountBalances)>, DbError> {
    let accounts = list_by_company(pool, company_id, include_archived).await?;

    if accounts.is_empty() {
        return Ok(Vec::new());
    }

    let rows: Vec<BalancesRow> = sqlx::query_as(
        "SELECT ba.id AS bank_account_id, \
                agg.balance, agg.last_entry_date, \
                st.closing_balance, st.period_to, \
                CASE WHEN st.period_to IS NULL OR ba.journal_account_id IS NULL THEN NULL ELSE \
                  (SELECT COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) \
                   FROM journal_entry_lines jel \
                   INNER JOIN journal_entries je ON jel.entry_id = je.id \
                   WHERE je.company_id = ba.company_id \
                     AND jel.account_id = ba.journal_account_id \
                     AND je.entry_date <= st.period_to) END AS ledger_at, \
                CASE WHEN st.period_to IS NULL OR ba.journal_account_id IS NULL THEN NULL ELSE \
                  (SELECT COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) \
                   FROM journal_entry_lines jel \
                   INNER JOIN journal_entries je ON jel.entry_id = je.id \
                   WHERE je.company_id = ba.company_id \
                     AND jel.account_id = ba.journal_account_id \
                     AND je.entry_date > st.period_to \
                     AND je.id IN (SELECT bt.matched_entry_id FROM bank_transactions bt \
                                   WHERE bt.bank_account_id = ba.id AND bt.company_id = ba.company_id \
                                     AND bt.status = 'reconciled' AND bt.matched_entry_id IS NOT NULL \
                                     AND bt.booking_date <= st.period_to)) END \
                  AS booked_before_entered_after, \
                CASE WHEN st.period_to IS NULL OR ba.journal_account_id IS NULL THEN NULL ELSE \
                  (SELECT COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) \
                   FROM journal_entry_lines jel \
                   INNER JOIN journal_entries je ON jel.entry_id = je.id \
                   WHERE je.company_id = ba.company_id \
                     AND jel.account_id = ba.journal_account_id \
                     AND je.entry_date <= st.period_to \
                     AND je.id IN (SELECT bt.matched_entry_id FROM bank_transactions bt \
                                   WHERE bt.bank_account_id = ba.id AND bt.company_id = ba.company_id \
                                     AND bt.status = 'reconciled' AND bt.matched_entry_id IS NOT NULL \
                                     AND bt.booking_date > st.period_to)) END \
                  AS booked_after_entered_before, \
                CAST(shared.n AS SIGNED) AS sharing \
         FROM bank_accounts ba \
         LEFT JOIN ( \
             SELECT jel.account_id, \
                    COALESCE(SUM(jel.debit) - SUM(jel.credit), 0) AS balance, \
                    MAX(je.entry_date) AS last_entry_date \
             FROM journal_entry_lines jel \
             INNER JOIN journal_entries je ON jel.entry_id = je.id \
             WHERE je.company_id = ? \
               AND jel.account_id IN (SELECT journal_account_id FROM bank_accounts \
                                      WHERE company_id = ? AND journal_account_id IS NOT NULL) \
             GROUP BY jel.account_id \
         ) agg ON agg.account_id = ba.journal_account_id \
         LEFT JOIN ( \
             SELECT bank_account_id, closing_balance, period_to, \
                    ROW_NUMBER() OVER (PARTITION BY bank_account_id \
                                       ORDER BY period_to DESC, imported_at DESC, id DESC) AS rn \
             FROM bank_imports \
             WHERE company_id = ? AND closing_balance IS NOT NULL \
         ) st ON st.bank_account_id = ba.id AND st.rn = 1 \
         LEFT JOIN ( \
             SELECT journal_account_id, COUNT(*) AS n \
             FROM bank_accounts \
             WHERE company_id = ? AND journal_account_id IS NOT NULL \
             GROUP BY journal_account_id \
         ) shared ON shared.journal_account_id = ba.journal_account_id \
         WHERE ba.company_id = ?",
    )
    .bind(company_id) // agg : je.company_id
    .bind(company_id) // agg : bornée aux comptes liés de la société
    .bind(company_id) // st
    .bind(company_id) // shared
    .bind(company_id) // ba.company_id
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    let by_bank_account: std::collections::HashMap<i64, BalancesRow> =
        rows.into_iter().map(|r| (r.bank_account_id, r)).collect();

    Ok(accounts
        .into_iter()
        .map(|b| {
            let balances = match (b.journal_account_id, by_bank_account.get(&b.id)) {
                (Some(_), Some(r)) => {
                    // Le partage se compte sur TOUS les comptes de la société,
                    // archivés compris : la réponse ne dépend pas d'`include_archived`.
                    let shared = r.sharing.unwrap_or(1) > 1;
                    let ledger_balance_at_statement = match (
                        r.ledger_at,
                        r.booked_before_entered_after,
                        r.booked_after_entered_before,
                    ) {
                        (Some(at), Some(plus), Some(minus)) if !shared => Some(at + plus - minus),
                        _ => None,
                    };
                    BankAccountBalances {
                        // Compte lié sans aucune ligne → 0 (comportement v014-1).
                        current_balance: Some(r.balance.unwrap_or(rust_decimal::Decimal::ZERO)),
                        last_transaction_date: r.last_entry_date,
                        statement_closing_balance: r.closing_balance,
                        statement_date: r.period_to,
                        ledger_balance_at_statement,
                    }
                }
                // Sans compte lié : ni solde comptable ni écart ; le relevé reste.
                (None, Some(r)) => BankAccountBalances {
                    statement_closing_balance: r.closing_balance,
                    statement_date: r.period_to,
                    ..Default::default()
                },
                (_, None) => BankAccountBalances::default(),
            };
            (b, balances)
        })
        .collect())
}
