//! Routes bank-accounts — Story 8-5a-zero (foundation `journal_account_id`).
//!
//! - `GET  /api/v1/bank-accounts` — liste les bank_accounts de la company
//!   (`authenticated_routes`, tous rôles authentifiés).
//! - `PATCH /api/v1/bank-accounts/{id}` — met à jour le `journalAccountId`
//!   (`comptable_routes`, RBAC Comptable+ requis pour la mutation).
//!
//! Le PATCH valide :
//! - `bank_account` existe et appartient à la company → 404
//!   `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` sinon (variant
//!   `AppError::BankAccountNotFound` réutilisé v0.1, cf. L64).
//! - Si `journalAccountId.is_some()` : compte existe + actif + de type
//!   Asset|Liability (Revenue/Expense rejetés 400 `INVALID_ACCOUNT_TYPE`).
//! - Optimistic lock `version` → 409 `OPTIMISTIC_LOCK_CONFLICT`.
//!
//! Audit log `bank_account.updated` émis depuis le handler (jamais le repo)
//! dans la même transaction que l'UPDATE — atomicité garantie. Court-circuit
//! no-op KF-004 : pas d'audit si le `journal_account_id` ne change pas.

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::bank_account::BankAccount;
use kesh_db::errors::DbError;
use kesh_db::repositories::{accounts, audit_log, bank_accounts};
use serde::Deserialize;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Body du PATCH `/bank-accounts/{id}`.
///
/// Story 8-5a-zero : un seul champ mutable v0.1 (`journalAccountId`). Si
/// besoin futur de patcher `bank_name` / `iban` hors onboarding, créer un
/// handler dédié plutôt qu'un mega-PATCH multi-fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchJournalLinkBody {
    /// `Some(account_id)` lie au compte comptable, `None` délie.
    pub journal_account_id: Option<i64>,
    /// Optimistic lock — version attendue de la row.
    pub version: i32,
}

/// Handler `GET /api/v1/bank-accounts` — liste les bank_accounts de la
/// company courante (multi-tenant scoping via `current_user.company_id`).
pub async fn list_bank_accounts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BankAccount>>, AppError> {
    let accounts = bank_accounts::list_by_company(&state.pool, current_user.company_id).await?;
    Ok(Json(accounts))
}

/// Handler `PATCH /api/v1/bank-accounts/{id}` — met à jour le
/// `journalAccountId` du bank_account.
pub async fn patch_bank_account_journal_link(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<PatchJournalLinkBody>,
) -> Result<Json<BankAccount>, AppError> {
    if body.version < 1 {
        return Err(AppError::Validation("version doit être >= 1".to_string()));
    }

    // Pré-flight validation hors tx — anti-énumération via 404 si compte
    // archivé ou cross-tenant (KF-002 pattern).
    if let Some(account_id) = body.journal_account_id {
        if account_id <= 0 {
            return Err(AppError::Validation(
                "journalAccountId doit être strictement positif".to_string(),
            ));
        }
        let account =
            accounts::find_by_id_in_company(&state.pool, account_id, current_user.company_id)
                .await?
                .ok_or(AppError::AccountNotFound { account_id })?;

        if !account.active {
            // Anti-énumération : compte archivé → 404 (pas 400) pour ne pas leak l'existence.
            return Err(AppError::AccountNotFound { account_id });
        }

        // Validation §validation-account-type : un compte bancaire ne peut être
        // lié qu'à un compte Asset (typique 1020/1030) ou Liability (rare —
        // découvert chronique 2100). Revenue/Expense rejetés.
        match account.account_type {
            AccountType::Asset | AccountType::Liability => {}
            other => {
                return Err(AppError::InvalidAccountType {
                    account_id,
                    account_type: other.as_str().to_string(),
                });
            }
        }
    }

    // Ouverture transaction — UPDATE + audit_log atomiques (pattern Story 3-5
    // / 7-3 / 8-4 — audit_log écrit depuis le handler dans la même tx).
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    let pre_state = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, \
         version, created_at, updated_at FROM bank_accounts \
         WHERE company_id = ? AND id = ? LIMIT 1",
    )
    .bind(current_user.company_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(format!("select bank_account: {e}")))?
    .ok_or(AppError::BankAccountNotFound)?;

    let updated = match bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        current_user.company_id,
        id,
        body.journal_account_id,
        body.version,
    )
    .await
    {
        Ok(b) => b,
        Err(DbError::NotFound) => return Err(AppError::BankAccountNotFound),
        Err(e) => return Err(AppError::Database(e)),
    };

    // KF-004 no-op short-circuit : pas d'audit_log si la version n'a pas
    // été bumpée. `set_journal_account_id_for_company` court-circuite et
    // retourne `existing` quand `journal_account_id` ne change pas.
    if updated.version != pre_state.version {
        let details = serde_json::json!({
            "bank_account_id": id,
            "before": {
                "journal_account_id": pre_state.journal_account_id,
                "version": pre_state.version,
            },
            "after": {
                "journal_account_id": updated.journal_account_id,
                "version": updated.version,
            },
        });
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id: current_user.user_id,
                action: "bank_account.updated".to_string(),
                entity_type: "bank_account".to_string(),
                entity_id: id,
                details_json: Some(details),
            },
        )
        .await?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(updated))
}
