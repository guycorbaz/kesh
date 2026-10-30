//! Routes Story v016-2a (#816) — saisir un relevé bancaire à la main.
//!
//! - `POST /api/v1/bank-imports/manual/preview` (Comptable+) — analyse et
//!   contrôle le relevé, rend ses sommes et quatre avertissements ; **n'écrit
//!   rien**, trace comprise.
//! - `POST /api/v1/bank-imports/manual` (Comptable+) — crée le relevé : un
//!   import de `source_format = 'MANUAL'` et ses transactions, par
//!   `bank_imports::create_with_transactions` (la même fonction que l'import
//!   d'un fichier), et **une** trace `bank_import.entered_manually`, dans la
//!   même transaction.
//!
//! Un relevé saisi hérite de tout le circuit d'un import : rapprochement,
//! annulation, suppression (`DELETE /api/v1/bank-imports/{id}`, inchangée),
//! tuile du tableau de bord et contrôle K3 (il porte un solde d'arrivée).
//! Ce module n'écrit **aucune** requête SQL : il compose les aides de
//! `routes/bank_imports.rs` et les lectures de `kesh_db::repositories`.
//!
//! **Ordre des refus**, chacun sans écriture : authentification et rôle
//! (middlewares) ; corps illisible, `Content-Type` absent, corps au-delà de la
//! limite — `400 VALIDATION_ERROR` (extracteur partagé, `crate::extractors`) ;
//! compte absent, d'une autre société ou archivé (`404
//! BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`) ou clos (`409 BANK_ACCOUNT_CLOSED`),
//! pré-vol sur le pool comme l'import ; `400 BANK_STATEMENT_INVALID` ; `422
//! BANK_STATEMENT_UNBALANCED` ; puis, **dans** la transaction : relevé
//! identique (`422 BANK_IMPORT_DUPLICATE_FILE`), doublons de lignes (`422
//! BANK_STATEMENT_DUPLICATE_LINES`), écart de solde de départ (`422
//! BANK_STATEMENT_OPENING_BALANCE_GAP`) — le premier qui manque de sa
//! confirmation l'emporte.

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use rust_decimal::Decimal;
use serde::Serialize;

use kesh_core::bank_imports::manual::{
    ManualStatementError, ManualStatementInput, ManualStatementRaw, ManualStatementSums,
    manual_statement_canonical_bytes, manual_statement_filename, manual_statement_to_drafts,
    parse_manual_statement, validate_manual_statement,
};
use kesh_core::bank_imports::{BankTransactionDraft, detect_duplicate_lines};
use kesh_db::entities::{
    BankAccount, BankImport, BankImportSourceFormat, NewBankImport, NewBankTransaction,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{bank_accounts, bank_imports, bank_transactions};
use kesh_reconciliation::{ReconciliationError, with_account_lock};

use crate::AppState;
use crate::errors::AppError;
use crate::extractors::impl_validation_extractor;
use crate::middleware::auth::CurrentUser;
use crate::routes::bank_imports::{
    BankImportResponse, DuplicateFilePayload, ImportAuditAction, compute_sha256_hex,
    create_import_or_roll_back, duplicate_file_payload, existing_dedup_keys, import_to_response,
    insert_canonical_audit_log,
};

impl_validation_extractor!(
    ManualStatementBodyExtractor,
    ManualStatementRaw,
    "`bankAccountId` i64, `periodFrom`, `periodTo`, `statementNumber?`, `openingBalance`, \
     `closingBalance` str, `lines` tableau de { `bookingDate`, `valueDate?`, `details`, \
     `counterpartyName?`, `reference?`, `amount`, `direction` } str, `confirm*?` bool"
);

// ---------------------------------------------------------------------------
// Réponse de l'aperçu
// ---------------------------------------------------------------------------

/// Réponse de `POST /api/v1/bank-imports/manual/preview` (`200`, aucune
/// écriture). Montants en chaînes à deux décimales.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualStatementPreviewResponse {
    pub total_credits: String,
    pub total_debits: String,
    pub computed_closing_balance: String,
    pub transaction_count: usize,
    pub warnings: ManualStatementWarnings,
}

/// Les quatre avertissements de l'aperçu ; `null` ou `[]` quand absents.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualStatementWarnings {
    pub duplicate_file: Option<DuplicateFilePayload>,
    pub duplicate_lines: Vec<ManualDuplicateLineWarning>,
    pub opening_balance_gap: Option<OpeningBalanceGap>,
    pub overlapping_statements: Vec<OverlappingStatement>,
}

/// Une ligne du relevé (rang 1-based) qui a une transaction existante de même
/// empreinte.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualDuplicateLineWarning {
    pub line: usize,
    pub existing_transaction_id: i64,
    pub key: String,
}

/// Solde de départ ≠ clôture du relevé précédent à solde (C-v016-2-5).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningBalanceGap {
    pub expected: String,
    pub entered: String,
    pub previous_import_id: i64,
    pub previous_period_to: chrono::NaiveDate,
}

/// Relevé existant du compte dont la période chevauche la saisie
/// (information, sans confirmation).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlappingStatement {
    pub import_id: i64,
    pub period_from: chrono::NaiveDate,
    pub period_to: chrono::NaiveDate,
    pub source_format: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /api/v1/bank-imports/manual/preview` — contrôle le relevé et rend ses
/// sommes et ses avertissements. **N'écrit rien** : ni entête, ni ligne, ni
/// trace. Rend `BANK_STATEMENT_INVALID` et `BANK_STATEMENT_UNBALANCED` comme
/// la création (une seule fonction de validation), et les mêmes refus de
/// compte. Les trois drapeaux `confirm*` sont ignorés.
pub async fn preview(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    ManualStatementBodyExtractor(raw): ManualStatementBodyExtractor,
) -> Result<Json<ManualStatementPreviewResponse>, AppError> {
    let checked = check_statement(&state, &current_user, &raw).await?;
    let company_id = current_user.company_id;
    let bank_id = checked.account.id;
    let input = &checked.input;

    let duplicate_file =
        bank_imports::find_by_company_and_hash(&state.pool, company_id, &checked.file_hash)
            .await?
            .map(|existing| duplicate_file_payload(&existing));

    let existing = bank_transactions::find_in_dedup_window(
        &state.pool,
        company_id,
        bank_id,
        input.period_from,
        input.period_to,
    )
    .await?;
    let drafts = line_drafts(input, &checked.file_hash, &current_user);
    let duplicate_lines = detect_duplicate_lines(&drafts, bank_id, &existing_dedup_keys(&existing))
        .into_iter()
        .map(|d| ManualDuplicateLineWarning {
            line: d.new_index + 1,
            existing_transaction_id: d.existing_transaction_id,
            key: d.key,
        })
        .collect();

    let opening_balance_gap = opening_gap(&state.pool, company_id, bank_id, input).await?;

    let overlapping_statements = bank_imports::overlapping_statements(
        &state.pool,
        company_id,
        bank_id,
        input.period_from,
        input.period_to,
    )
    .await?
    .into_iter()
    .map(|i| OverlappingStatement {
        import_id: i.id,
        period_from: i.period_from,
        period_to: i.period_to,
        source_format: i.source_format.as_str().to_string(),
    })
    .collect();

    Ok(Json(ManualStatementPreviewResponse {
        total_credits: amount_string(checked.sums.total_credits),
        total_debits: amount_string(checked.sums.total_debits),
        computed_closing_balance: amount_string(checked.sums.computed_closing_balance),
        transaction_count: checked.sums.transaction_count,
        warnings: ManualStatementWarnings {
            duplicate_file,
            duplicate_lines,
            opening_balance_gap: opening_balance_gap.map(|g| OpeningBalanceGap {
                expected: amount_string(g.expected),
                entered: amount_string(g.entered),
                previous_import_id: g.previous_import_id,
                previous_period_to: g.previous_period_to,
            }),
            overlapping_statements,
        },
    }))
}

/// `POST /api/v1/bank-imports/manual` — crée le relevé saisi (`201`, le
/// `BankImportResponse` de l'import). Entête, lignes et trace
/// `bank_import.entered_manually` dans **la même** transaction : un échec de
/// la trace n'écrit rien. Les doublons de lignes ne sont **jamais** ignorés en
/// silence : sans `confirmDuplicateLines`, refus ; avec, toutes les lignes sont
/// enregistrées (C-v016-2-6).
///
/// **Sous le verrou nommé du compte bancaire** (revue de code P1, E2) : le
/// contrôle « relevé identique », celui des doublons de lignes et l'`INSERT` se
/// font sous `with_account_lock` — le verrou des gestes de rapprochement,
/// `GET_LOCK('reconcile:<base>:<société>:<compte>')` —, pris sur une connexion
/// **dédiée** et rendu seulement **après le commit** de la création : deux
/// enregistrements identiques simultanés donnent un relevé et un refus de
/// doublon, la seconde transaction ne naissant qu'une fois la première
/// commitée. Sans ce verrou, chacune lisait une base où l'autre n'avait rien
/// écrit (aucune unicité de `(company_id, file_hash)` depuis la Story 8-3). Un
/// verrou tenu plus de [`MANUAL_STATEMENT_LOCK_TIMEOUT_SECS`] secondes rend
/// `409 RECONCILIATION_ACCOUNT_LOCKED`, sans écriture.
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    ManualStatementBodyExtractor(raw): ManualStatementBodyExtractor,
) -> Result<(StatusCode, Json<BankImportResponse>), AppError> {
    let checked = check_statement(&state, &current_user, &raw).await?;
    let company_id = current_user.company_id;
    let bank_id = checked.account.id;

    let mut lock_tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let mut outcome: Option<Result<BankImport, AppError>> = None;
    let slot = &mut outcome;
    let (pool, user, body) = (&state.pool, &current_user, &raw);
    let locked = with_account_lock(
        &mut lock_tx,
        company_id,
        bank_id,
        MANUAL_STATEMENT_LOCK_TIMEOUT_SECS,
        async move |_held| {
            *slot = Some(create_under_lock(pool, user, body, &checked).await);
            Ok(())
        },
    )
    .await;
    drop(lock_tx);

    match (outcome, locked) {
        (Some(created), Ok(())) => {
            let header = created?;
            Ok((StatusCode::CREATED, Json(import_to_response(header))))
        }
        // La création a conclu (commitée ou annulée) ; seul le `RELEASE_LOCK`
        // a échoué. Le verrou suit la session de la connexion. Annoncer un
        // échec d'une création commitée serait faux (P2) : on rend son issue.
        (Some(created), Err(e)) => {
            tracing::error!(error = ?e, bank_account_id = bank_id, "saisie d'un relevé : libération du verrou du compte en échec");
            let header = created?;
            Ok((StatusCode::CREATED, Json(import_to_response(header))))
        }
        (
            None,
            Err(ReconciliationError::AccountLocked {
                bank_account_id,
                timeout_secs,
            }),
        ) => Err(AppError::ReconciliationAccountLocked {
            bank_account_id,
            timeout_secs,
        }),
        (None, Err(ReconciliationError::Database(e))) => Err(AppError::Database(DbError::Sqlx(e))),
        (None, other) => {
            tracing::error!(?other, "saisie d'un relevé : verrou du compte non acquis");
            Err(AppError::Internal(
                "verrou du compte bancaire non acquis".to_string(),
            ))
        }
    }
}

/// Délai d'attente du verrou nommé du compte bancaire — celui des gestes de
/// rapprochement (`routes/reconciliation.rs`, `LOCK_TIMEOUT_SECS`).
pub const MANUAL_STATEMENT_LOCK_TIMEOUT_SECS: u32 = 5;

/// Limite de corps **propre** aux deux routes de saisie (revue de code P1,
/// E1), indépendante de `KESH_BANK_IMPORT_MAX_MB` : 500 lignes pleines — un
/// libellé de 500 caractères, une contrepartie et une référence de 255, tous
/// de quatre octets — pèsent environ 2,1 Mo avec les clés JSON ; 3 Mio laissent
/// la marge. Au-delà, l'extracteur partagé rend `400 VALIDATION_ERROR`.
pub const MANUAL_STATEMENT_BODY_LIMIT_BYTES: usize = 3 * 1024 * 1024;

/// La création proprement dite, sous le verrou du compte : sa propre
/// transaction, conclue (commit ou annulation) avant que le verrou soit rendu.
async fn create_under_lock(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    raw: &ManualStatementRaw,
    checked: &CheckedStatement,
) -> Result<BankImport, AppError> {
    let company_id = current_user.company_id;
    let bank_id = checked.account.id;
    let input = &checked.input;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let mut modifiers: Vec<&'static str> = Vec::new();
    let mut details_extra = serde_json::Map::new();

    // 1. Relevé identique déjà enregistré.
    if let Some(existing) =
        bank_imports::find_by_company_and_hash(&mut *tx, company_id, &checked.file_hash).await?
    {
        if !raw.confirm_duplicate_file {
            let _ = tx.rollback().await;
            return Err(AppError::BankImportDuplicateFile {
                existing_import_id: existing.id,
                existing_filename: existing.filename,
            });
        }
        modifiers.push("duplicate_file");
    }

    // 2. Doublons de lignes : confirmer ou corriger, jamais ignorer.
    let existing = bank_transactions::find_in_dedup_window(
        &mut *tx,
        company_id,
        bank_id,
        input.period_from,
        input.period_to,
    )
    .await?;
    let imported_at = chrono::Utc::now().naive_utc();
    let filename = manual_statement_filename(input.period_from, input.period_to);
    let (draft, tx_drafts) = manual_statement_to_drafts(
        input,
        bank_id,
        company_id,
        checked.file_hash.clone(),
        filename,
        imported_at,
        current_user.user_id,
    );
    let duplicates = detect_duplicate_lines(&tx_drafts, bank_id, &existing_dedup_keys(&existing));
    if !duplicates.is_empty() {
        if !raw.confirm_duplicate_lines {
            let _ = tx.rollback().await;
            return Err(AppError::BankStatementDuplicateLines {
                lines: duplicates
                    .iter()
                    .map(|d| (d.new_index + 1, d.existing_transaction_id))
                    .collect(),
            });
        }
        modifiers.push("duplicate_lines_imported");
        details_extra.insert(
            "duplicate_lines_imported".into(),
            serde_json::json!(duplicates.len()),
        );
    }

    // 3. Solde de départ ≠ clôture du relevé précédent à solde.
    if let Some(gap) = opening_gap(&mut *tx, company_id, bank_id, input).await? {
        if !raw.confirm_opening_balance_gap {
            let _ = tx.rollback().await;
            return Err(AppError::BankStatementOpeningBalanceGap {
                expected: amount_string(gap.expected),
                entered: amount_string(gap.entered),
                previous_import_id: gap.previous_import_id,
                previous_period_to: gap.previous_period_to,
            });
        }
        modifiers.push("opening_balance_gap");
        details_extra.insert(
            "expected_opening_balance".into(),
            serde_json::json!(amount_string(gap.expected)),
        );
    }

    details_extra.insert(
        "period_from".into(),
        serde_json::json!(input.period_from.format("%Y-%m-%d").to_string()),
    );
    details_extra.insert(
        "period_to".into(),
        serde_json::json!(input.period_to.format("%Y-%m-%d").to_string()),
    );
    details_extra.insert(
        "opening_balance".into(),
        serde_json::json!(amount_string(input.opening_balance)),
    );
    details_extra.insert(
        "closing_balance".into(),
        serde_json::json!(amount_string(input.closing_balance)),
    );
    details_extra.insert(
        "statement_id".into(),
        serde_json::json!(input.statement_number),
    );

    let new_import = NewBankImport {
        company_id: draft.company_id,
        bank_account_id: draft.bank_account_id,
        filename: draft.filename,
        file_hash: draft.file_hash,
        source_format: BankImportSourceFormat::Manual,
        statement_id: draft.statement_id,
        period_from: draft.period_from,
        period_to: draft.period_to,
        opening_balance: draft.opening_balance.map(|m| m.amount()),
        closing_balance: draft.closing_balance.map(|m| m.amount()),
        transaction_count: draft.transaction_count,
        imported_by_user_id: draft.imported_by_user_id,
    };
    let new_txs: Vec<NewBankTransaction> = tx_drafts
        .into_iter()
        .map(|t| NewBankTransaction {
            company_id,
            bank_account_id: bank_id,
            booking_date: t.booking_date,
            value_date: t.value_date,
            amount: t.amount.amount(),
            currency: t.currency,
            reference: t.reference,
            details: t.details,
            end_to_end_id: t.end_to_end_id,
            transaction_id: t.transaction_id,
            counterparty_iban: t.counterparty_iban,
            counterparty_name: t.counterparty_name,
        })
        .collect();

    let (mut tx, header) = create_import_or_roll_back(tx, new_import, new_txs).await?;

    if let Err(e) = insert_canonical_audit_log(
        &mut tx,
        ImportAuditAction::EnteredManually,
        current_user.user_id,
        current_user.api_key_id,
        header.id,
        &header.filename,
        header.transaction_count,
        header.source_format.as_str(),
        modifiers,
        details_extra,
        None,
    )
    .await
    {
        let _ = tx.rollback().await;
        return Err(e);
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok(header)
}

// ---------------------------------------------------------------------------
// Aides
// ---------------------------------------------------------------------------

/// Un relevé dont le compte, la forme et l'équilibre sont contrôlés.
struct CheckedStatement {
    account: BankAccount,
    input: ManualStatementInput,
    sums: ManualStatementSums,
    /// SHA-256 hexadécimal de l'empreinte canonique (`file_hash`).
    file_hash: String,
}

/// Pré-vol commun à l'aperçu et à la création, sans écriture : compte (`404`
/// absent, d'une autre société ou archivé ; `409` clos), puis analyse et
/// règles du relevé (`400`, `422`), puis l'empreinte.
async fn check_statement(
    state: &AppState,
    current_user: &CurrentUser,
    raw: &ManualStatementRaw,
) -> Result<CheckedStatement, AppError> {
    let account = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        raw.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;
    if account.archived {
        return Err(AppError::BankAccountNotFound);
    }
    if account.is_closed() {
        return Err(AppError::BankAccountClosed {
            bank_account_id: account.id,
        });
    }
    let input = parse_manual_statement(raw).map_err(statement_error)?;
    let sums = validate_manual_statement(&input).map_err(statement_error)?;
    let file_hash = compute_sha256_hex(&manual_statement_canonical_bytes(&input));
    Ok(CheckedStatement {
        account,
        input,
        sums,
        file_hash,
    })
}

/// Traduit l'erreur de kesh-core en `AppError` (codes des ensembles fermés).
fn statement_error(e: ManualStatementError) -> AppError {
    match e {
        ManualStatementError::Invalid {
            line,
            field,
            reason,
        } => AppError::BankStatementInvalid {
            line,
            field: field.code(),
            reason: reason.code(),
        },
        ManualStatementError::Unbalanced {
            opening,
            closing,
            computed_closing,
            diff,
        } => AppError::BankStatementUnbalanced {
            opening: amount_string(opening),
            closing: amount_string(closing),
            computed_closing: amount_string(computed_closing),
            diff: amount_string(diff),
        },
    }
}

/// Les brouillons des lignes, pour la détection de doublons de l'aperçu.
fn line_drafts(
    input: &ManualStatementInput,
    file_hash: &str,
    current_user: &CurrentUser,
) -> Vec<BankTransactionDraft> {
    manual_statement_to_drafts(
        input,
        input.bank_account_id,
        current_user.company_id,
        file_hash.to_string(),
        String::new(),
        chrono::NaiveDateTime::default(),
        current_user.user_id,
    )
    .1
}

/// Écart de solde de départ, s'il existe un relevé précédent à solde.
struct Gap {
    expected: Decimal,
    entered: Decimal,
    previous_import_id: i64,
    previous_period_to: chrono::NaiveDate,
}

async fn opening_gap<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    input: &ManualStatementInput,
) -> Result<Option<Gap>, AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    let previous = bank_imports::previous_statement_closing(
        executor,
        company_id,
        bank_account_id,
        input.period_from,
    )
    .await?;
    Ok(previous.and_then(|(id, period_to, closing)| {
        (closing != input.opening_balance).then_some(Gap {
            expected: closing,
            entered: input.opening_balance,
            previous_import_id: id,
            previous_period_to: period_to,
        })
    }))
}

/// Montant en chaîne à deux décimales (`"50.00"`, `"-0.01"`).
fn amount_string(d: Decimal) -> String {
    // Un zéro négatif (`"-0"` saisi) s'écrit `0.00`, non `-0.00` (revue de code
    // P1, E6).
    let d = if d.is_zero() { Decimal::ZERO } else { d };
    format!("{:.2}", d)
}
