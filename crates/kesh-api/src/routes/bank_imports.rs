//! Routes Story 8-1b — Import bancaire CAMT.053.
//!
//! - `POST /api/v1/bank-imports/preview` (Comptable+) — parse + validate,
//!   pas de persistance, retourne preview avec warnings.
//! - `POST /api/v1/bank-imports` (Comptable+) — parse + validate +
//!   persist atomique (entête + transactions + audit log dans une seule
//!   `Transaction`).
//! - `GET /api/v1/bank-imports` (tout rôle authentifié) — liste paginée
//!   multi-tenant (KF-002).
//! - `GET /api/v1/bank-imports/{id}` (tout rôle) — détail import +
//!   transactions filles.
//!
//! **Multi-tenant scoping** : tous les handlers utilisent
//! `current_user.company_id` du JWT. Le `bankAccountId` du formulaire
//! est validé via `bank_accounts::find_by_id_for_company` avant tout
//! traitement (Pass 1 H5 / T6.3 — anti-IDOR).
//!
//! **Multipart** : `axum::extract::Multipart` (feature `multipart`).
//! Le `DefaultBodyLimit` est appliqué au sub-router au moment du mount
//! dans `lib.rs::build_router` (T6.10).

use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use kesh_core::bank_imports::{
    self as core_bank_imports, BankImportDraft, BankTransactionDraft, SourceFormatTag,
};
use kesh_db::entities::{
    BankImportSourceFormat, NewAuditLogEntry, NewBankImport, NewBankTransaction,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{audit_log, bank_accounts, bank_imports, bank_transactions};
use kesh_import::{CamtError, ImportedStatement, parse_camt053};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankImportPreviewResponse {
    /// Hash SHA-256 hex du fichier uploadé (passé tel quel au confirm).
    pub file_hash: String,
    /// Filename original (pour ré-affichage UX, pas pour persistance).
    pub filename: String,
    pub source_format: String,
    /// Statement sélectionné (matché par IBAN du `bankAccountId`).
    pub selected_statement: PreviewStatement,
    /// Statements ignorés (autres IBAN dans le fichier multi-stmt).
    /// AC #3b — F1 validate Pass 1.
    pub ignored_statements: Vec<IgnoredStatement>,
    /// Warnings non-bloquants (`balance_mismatch`, `unsupported_currency`).
    pub warnings: Vec<String>,
    pub transactions: Vec<PreviewTransaction>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewStatement {
    pub statement_id: Option<String>,
    pub account_iban: String,
    pub currency: String,
    pub period_from: chrono::NaiveDate,
    pub period_to: chrono::NaiveDate,
    pub opening_balance: Option<Decimal>,
    pub closing_balance: Option<Decimal>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredStatement {
    pub statement_id: Option<String>,
    pub account_iban: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTransaction {
    pub booking_date: chrono::NaiveDate,
    pub value_date: Option<chrono::NaiveDate>,
    pub amount: Decimal,
    pub currency: String,
    pub reference: Option<String>,
    pub details: String,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankImportResponse {
    pub id: i64,
    pub bank_account_id: i64,
    pub filename: String,
    pub file_hash: String,
    pub source_format: String,
    pub statement_id: Option<String>,
    pub period_from: chrono::NaiveDate,
    pub period_to: chrono::NaiveDate,
    pub opening_balance: Option<Decimal>,
    pub closing_balance: Option<Decimal>,
    pub transaction_count: i32,
    pub imported_at: chrono::NaiveDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankImportDetailResponse {
    #[serde(flatten)]
    pub import: BankImportResponse,
    pub transactions: Vec<TransactionResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub id: i64,
    pub booking_date: chrono::NaiveDate,
    pub value_date: Option<chrono::NaiveDate>,
    pub amount: Decimal,
    pub currency: String,
    pub reference: Option<String>,
    pub details: String,
    pub end_to_end_id: Option<String>,
    pub transaction_id: Option<String>,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBankImportsQuery {
    #[serde(default)]
    pub bank_account_id: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Multipart parsing helper
// ---------------------------------------------------------------------------

const FILENAME_MAX_LEN: usize = 255;

struct MultipartFields {
    file_bytes: Vec<u8>,
    filename: String,
    bank_account_id: i64,
    confirm_balance_mismatch: bool,
}

async fn parse_multipart(mut multipart: Multipart) -> Result<MultipartFields, AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut bank_account_id: Option<i64> = None;
    let mut confirm_balance_mismatch = false;

    while let Some(field) = multipart.next_field().await.map_err(map_multipart_err)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let original = field.file_name().unwrap_or("upload.xml").to_string();
                let truncated: String = original.chars().take(FILENAME_MAX_LEN).collect();
                filename = Some(truncated);
                let bytes = field.bytes().await.map_err(map_multipart_err)?;
                file_bytes = Some(bytes.to_vec());
            }
            "bankAccountId" => {
                let text = field.text().await.map_err(map_multipart_err)?;
                let id: i64 = text.trim().parse().map_err(|_| {
                    AppError::Validation(format!("bankAccountId invalide : '{text}'"))
                })?;
                bank_account_id = Some(id);
            }
            "confirmBalanceMismatch" => {
                let text = field.text().await.map_err(map_multipart_err)?;
                confirm_balance_mismatch = matches!(text.trim(), "true" | "1");
            }
            _ => {
                // Ignore unknown fields (forward-compat).
                let _ = field.bytes().await;
            }
        }
    }

    let file_bytes = file_bytes
        .ok_or_else(|| AppError::Validation("Champ 'file' manquant dans le multipart".into()))?;
    let filename = filename.unwrap_or_else(|| "upload.xml".into());
    let bank_account_id = bank_account_id.ok_or_else(|| {
        AppError::Validation("Champ 'bankAccountId' manquant dans le multipart".into())
    })?;

    Ok(MultipartFields {
        file_bytes,
        filename,
        bank_account_id,
        confirm_balance_mismatch,
    })
}

fn map_multipart_err(e: axum::extract::multipart::MultipartError) -> AppError {
    // axum 0.8 expose `status()` qui retourne `StatusCode::PAYLOAD_TOO_LARGE`
    // pour l'overflow `DefaultBodyLimit`. C'est la signal canonique.
    if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return AppError::BankImportTooLarge;
    }
    let msg = e.to_string();
    if msg.to_ascii_lowercase().contains("too large")
        || msg.to_ascii_lowercase().contains("body limit")
    {
        return AppError::BankImportTooLarge;
    }
    AppError::Validation(format!("Multipart invalide : {msg}"))
}

// ---------------------------------------------------------------------------
// CamtError → AppError mapping (F7 + F6 validate Pass 1)
// ---------------------------------------------------------------------------

fn map_camt_error(err: CamtError) -> AppError {
    match err {
        CamtError::MalformedXml(msg) => AppError::BankImportParseFailed {
            kind: "MALFORMED_XML",
            message: msg,
        },
        CamtError::UnsupportedVersion(uri) => AppError::BankImportParseFailed {
            kind: "UNSUPPORTED_VERSION",
            message: uri,
        },
        CamtError::MissingRequiredField(path) => AppError::BankImportParseFailed {
            kind: "MISSING_FIELD",
            message: path,
        },
        CamtError::InvalidAmount(s) => AppError::BankImportParseFailed {
            kind: "INVALID_AMOUNT",
            message: s,
        },
        CamtError::InvalidDate(s) => AppError::BankImportParseFailed {
            kind: "INVALID_DATE",
            message: s,
        },
    }
}

fn money_to_string(m: kesh_core::types::Money) -> String {
    format!("{}", m.amount())
}

// ---------------------------------------------------------------------------
// Multi-statement filtering (§multi-stmt)
// ---------------------------------------------------------------------------

/// Normalise un IBAN pour comparaison stricte (espaces retirés, uppercase).
fn normalize_iban(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase()
}

struct StatementSelection {
    selected: ImportedStatement,
    ignored: Vec<ImportedStatement>,
}

/// Filtre `Vec<ImportedStatement>` par IBAN du `bank_account` sélectionné
/// (§multi-stmt + AC #3b/3c).
fn select_statement_by_iban(
    statements: Vec<ImportedStatement>,
    target_iban: &str,
) -> Result<StatementSelection, AppError> {
    let target = normalize_iban(target_iban);
    let mut selected: Option<ImportedStatement> = None;
    let mut ignored: Vec<ImportedStatement> = Vec::new();
    for stmt in statements {
        if normalize_iban(&stmt.account_iban) == target {
            if selected.is_none() {
                selected = Some(stmt);
            } else {
                // Plus d'un statement avec le même IBAN — fusionner ou
                // garder le premier ? Pour v0.1 on garde le premier et
                // ignore les suivants (rare en pratique : un même IBAN
                // multiple statements = sub-périodes consécutives).
                ignored.push(stmt);
            }
        } else {
            ignored.push(stmt);
        }
    }
    match selected {
        Some(stmt) => Ok(StatementSelection {
            selected: stmt,
            ignored,
        }),
        None => Err(AppError::BankImportNoMatchingStatement {
            found_ibans: ignored.into_iter().map(|s| s.account_iban).collect(),
        }),
    }
}

fn version_to_source_format(stmt: &ImportedStatement) -> Result<BankImportSourceFormat, AppError> {
    match &stmt.source_format {
        kesh_import::SourceFormat::Camt053 { version } if version == "001.04" => {
            Ok(BankImportSourceFormat::Camt053V04)
        }
        kesh_import::SourceFormat::Camt053 { version } if version == "001.08" => {
            Ok(BankImportSourceFormat::Camt053V08)
        }
        kesh_import::SourceFormat::Camt053 { version } => Err(AppError::BankImportParseFailed {
            kind: "UNSUPPORTED_VERSION",
            message: version.clone(),
        }),
        kesh_import::SourceFormat::Csv { .. } => Err(AppError::BankImportParseFailed {
            kind: "UNSUPPORTED_VERSION",
            message: "csv".into(),
        }),
    }
}

fn source_format_tag_for_kesh_core(fmt: BankImportSourceFormat) -> SourceFormatTag {
    match fmt {
        BankImportSourceFormat::Camt053V04 => SourceFormatTag::Camt053V04,
        BankImportSourceFormat::Camt053V08 => SourceFormatTag::Camt053V08,
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /api/v1/bank-imports/preview` — parse + validate sans persistance.
///
/// Renvoie le statement matché, les statements ignorés (multi-stmt) et la
/// liste des warnings non-bloquants. Aucune mutation DB. RBAC Comptable+
/// (mounting via `comptable_routes` dans `lib.rs`).
pub async fn preview(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<Json<BankImportPreviewResponse>, AppError> {
    let fields = parse_multipart(multipart).await?;

    // Validation IDOR du bank_account_id (Pass 1 H5).
    let bank_account = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        fields.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;

    // Hash + parse.
    let file_hash = compute_sha256_hex(&fields.file_bytes);
    let stmts = parse_camt053(&fields.file_bytes).map_err(map_camt_error)?;
    let selection = select_statement_by_iban(stmts, &bank_account.iban)?;
    let stmt = selection.selected;

    // Warnings non-bloquants.
    let mut warnings: Vec<String> = Vec::new();
    if let Err(DbError::NotFound) = Ok::<_, DbError>(()) {
        // (placeholder — la branche n'arrive jamais ; structure pour permettre
        // l'extension future avec d'autres warnings côté preview.)
    }
    if let Err(_e) = core_bank_imports::validate_balance(&stmt) {
        warnings.push("balance_mismatch".into());
    }
    if let Err(_e) = core_bank_imports::validate_currency_supported_v0_1(&stmt) {
        warnings.push("unsupported_currency".into());
    }

    let source_format = version_to_source_format(&stmt)?;

    let preview_txs: Vec<PreviewTransaction> = stmt
        .transactions
        .iter()
        .map(|tx| PreviewTransaction {
            booking_date: tx.booking_date,
            value_date: tx.value_date,
            amount: tx.amount,
            currency: tx.currency.clone(),
            reference: tx.reference.clone(),
            details: tx.details.clone(),
            counterparty_iban: tx.counterparty_iban.clone(),
            counterparty_name: tx.counterparty_name.clone(),
        })
        .collect();

    let ignored: Vec<IgnoredStatement> = selection
        .ignored
        .into_iter()
        .map(|s| IgnoredStatement {
            statement_id: s.statement_id,
            account_iban: s.account_iban,
        })
        .collect();

    Ok(Json(BankImportPreviewResponse {
        file_hash,
        filename: fields.filename,
        source_format: source_format.as_str().to_string(),
        selected_statement: PreviewStatement {
            statement_id: stmt.statement_id.clone(),
            account_iban: stmt.account_iban.clone(),
            currency: stmt.currency.clone(),
            period_from: stmt.period_from,
            period_to: stmt.period_to,
            opening_balance: stmt.opening_balance,
            closing_balance: stmt.closing_balance,
        },
        ignored_statements: ignored,
        warnings,
        transactions: preview_txs,
    }))
}

/// `POST /api/v1/bank-imports` — persistance atomique + audit log.
///
/// Sans `confirmBalanceMismatch=true` : balance mismatch → 422.
/// Avec : audit log `bank_import.created_with_balance_mismatch` (CR-010).
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<BankImportResponse>), AppError> {
    let fields = parse_multipart(multipart).await?;

    let bank_account = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        fields.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;

    let file_hash = compute_sha256_hex(&fields.file_bytes);
    let stmts = parse_camt053(&fields.file_bytes).map_err(map_camt_error)?;
    let selection = select_statement_by_iban(stmts, &bank_account.iban)?;
    let stmt = selection.selected;

    // Validation devise (always blocking — pas de bypass v0.1).
    if let Err(kesh_core::errors::CoreError::BankImportUnsupportedCurrency(c)) =
        core_bank_imports::validate_currency_supported_v0_1(&stmt)
    {
        return Err(AppError::BankImportUnsupportedCurrency(c));
    }

    // Validation balance (bloquante sauf confirm explicite).
    let balance_mismatch_action = match core_bank_imports::validate_balance(&stmt) {
        Ok(()) => None,
        Err(kesh_core::errors::CoreError::BankImportBalanceMismatch {
            opening,
            closing,
            sum,
            diff,
        }) => {
            if !fields.confirm_balance_mismatch {
                return Err(AppError::BankImportBalanceMismatch {
                    opening: money_to_string(opening),
                    closing: money_to_string(closing),
                    sum: money_to_string(sum),
                    diff: money_to_string(diff),
                });
            }
            Some("created_with_balance_mismatch")
        }
        Err(other) => return Err(AppError::Database(other.into_db_error_or_internal())),
    };

    let source_format = version_to_source_format(&stmt)?;
    let imported_at = chrono::Utc::now().naive_utc();

    // kesh-core conversion (FK injection F8 architecture).
    let (draft, tx_drafts): (BankImportDraft, Vec<BankTransactionDraft>) =
        core_bank_imports::from_imported(
            &stmt,
            fields.bank_account_id,
            current_user.company_id,
            file_hash.clone(),
            fields.filename.clone(),
            imported_at,
            current_user.user_id,
        )
        .map_err(|e| AppError::Database(e.into_db_error_or_internal()))?;

    // Conversion vers les Inserts kesh-db.
    let new_import = NewBankImport {
        company_id: draft.company_id,
        bank_account_id: draft.bank_account_id,
        filename: draft.filename,
        file_hash: draft.file_hash,
        source_format,
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
            company_id: current_user.company_id,
            bank_account_id: fields.bank_account_id,
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

    // Persistance atomique : entête + transactions + audit log dans 1 tx.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let (header, _txs) =
        match bank_imports::create_with_transactions(&mut tx, new_import, new_txs).await {
            Ok(out) => out,
            Err(DbError::UniqueConstraintViolation(_)) => {
                return Err(AppError::BankImportDuplicateFile);
            }
            Err(e) => return Err(AppError::Database(e)),
        };

    let action = balance_mismatch_action.unwrap_or("created");
    let action_str = format!("bank_import.{action}");
    let details_json = serde_json::json!({
        "filename": header.filename,
        "transaction_count": header.transaction_count,
        "source_format": source_format_tag_for_kesh_core(source_format).as_db_str(),
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id: current_user.user_id,
            action: action_str,
            entity_type: "bank_imports".into(),
            entity_id: header.id,
            details_json: Some(details_json),
        },
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok((StatusCode::CREATED, Json(import_to_response(header))))
}

/// `GET /api/v1/bank-imports` — liste paginée multi-tenant (KF-002).
pub async fn list(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListBankImportsQuery>,
) -> Result<Json<ListResponse<BankImportResponse>>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let imports = bank_imports::find_by_company_id(
        &state.pool,
        current_user.company_id,
        query.bank_account_id,
        limit,
        offset,
    )
    .await?;

    Ok(Json(ListResponse {
        items: imports.into_iter().map(import_to_response).collect(),
        total: 0, // count séparé hors scope v0.1 — TODO Story 8-3 si pagination cliente l'exige
        offset,
        limit,
    }))
}

/// `GET /api/v1/bank-imports/{id}` — détail import + transactions filles.
pub async fn detail(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<BankImportDetailResponse>, AppError> {
    let import = bank_imports::find_by_id_for_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let txs =
        bank_transactions::list_by_import(&state.pool, current_user.company_id, import.id).await?;

    let import_resp = import_to_response(import);

    let tx_resps: Vec<TransactionResponse> = txs
        .into_iter()
        .map(|t| TransactionResponse {
            id: t.id,
            booking_date: t.booking_date,
            value_date: t.value_date,
            amount: t.amount,
            currency: t.currency,
            reference: t.reference,
            details: t.details,
            end_to_end_id: t.end_to_end_id,
            transaction_id: t.transaction_id,
            counterparty_iban: t.counterparty_iban,
            counterparty_name: t.counterparty_name,
            status: t.status.as_str().to_string(),
        })
        .collect();

    Ok(Json(BankImportDetailResponse {
        import: import_resp,
        transactions: tx_resps,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn compute_sha256_hex(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hex_encode(&hash)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn import_to_response(import: kesh_db::entities::BankImport) -> BankImportResponse {
    BankImportResponse {
        id: import.id,
        bank_account_id: import.bank_account_id,
        filename: import.filename,
        file_hash: import.file_hash,
        source_format: import.source_format.as_str().to_string(),
        statement_id: import.statement_id,
        period_from: import.period_from,
        period_to: import.period_to,
        opening_balance: import.opening_balance,
        closing_balance: import.closing_balance,
        transaction_count: import.transaction_count,
        imported_at: import.imported_at,
    }
}

// Helper local pour convertir certains CoreError vers DbError. Les
// variantes BankImport* portent une sémantique propre exposée déjà via
// AppError::BankImport* directement ; ce fallback Internal couvre les
// cas inattendus (ex. autres CoreError remontés par kesh-core).
trait CoreErrorIntoDbError {
    fn into_db_error_or_internal(self) -> DbError;
}
impl CoreErrorIntoDbError for kesh_core::errors::CoreError {
    fn into_db_error_or_internal(self) -> DbError {
        DbError::Invariant(format!(
            "CoreError inattendue dans bank_imports route : {self}"
        ))
    }
}
