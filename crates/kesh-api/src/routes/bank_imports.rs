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
    self as core_bank_imports, BankImportDraft, BankTransactionDraft, DuplicateKey, DuplicateLine,
    SourceFormatTag, dedup_key_scalar, detect_duplicate_lines,
};
use kesh_db::entities::{
    BankImport, BankImportSourceFormat, NewAuditLogEntry, NewBankImport, NewBankTransaction,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{audit_log, bank_accounts, bank_imports, bank_transactions};
use kesh_import::{
    CamtError, CsvError, CsvLineError, ImportedStatement, ParseCsvOutcome, parse_camt053,
    parse_csv_collect,
};

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
    /// AC #3b — F1 validate Pass 1. Conservé top-level pour
    /// backward-compat 8-1b (test `post_preview_returns_ignored_statements_for_multi_stmt_file`).
    pub ignored_statements: Vec<IgnoredStatement>,
    /// Story 8-3 — warnings non-bloquants structurés. Cf. spec
    /// §preview-warnings-shape : forme JSON stable, champs `null` ou
    /// vides (`[]`) quand absent.
    pub warnings: PreviewWarnings,
    pub transactions: Vec<PreviewTransaction>,
    /// Story 8-3 KF #70 — métadonnées de résolution de profil CSV
    /// (None pour CAMT). Permet à la UI de pré-sélectionner le profil
    /// auto-matché et de basculer vers une sélection explicite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csv_profile_match: Option<CsvProfileMatch>,
}

/// Warnings non-bloquants exposés par `POST /preview` (Story 8-3
/// §preview-warnings-shape). Tous les champs sont optionnels
/// (`Option<...>`) ou vides (`Vec` vide) quand l'analyse ne détecte
/// rien.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PreviewWarnings {
    /// CR-010 #62 / 8-1b — opening + Σ ≠ closing.
    pub balance_mismatch: Option<BalanceMismatchPayload>,
    /// 8-1b — devise ≠ CHF (rejet bloquant final, info en preview).
    pub unsupported_currency: Option<UnsupportedCurrencyPayload>,
    /// 8-2 — encoding détecté ≠ encoding profil (overridable
    /// `confirmEncodingMismatch`).
    pub encoding_mismatch: Option<EncodingMismatchPayload>,
    /// Story 8-3 — fichier déjà importé par le passé (`(company_id,
    /// file_hash)` matching). Overridable via `confirmDuplicateFile`.
    pub duplicate_file: Option<DuplicateFilePayload>,
    /// Story 8-3 — transactions chevauchant un import précédent
    /// (clé composite `(date, amount, ref_normalized, account)`).
    pub duplicate_lines: Vec<DuplicateLineWarning>,
    /// Story 8-3 — lignes CSV invalides (parse partial mode).
    /// Overridable via `confirmPartialImport`.
    pub invalid_lines: Option<InvalidLinesPayload>,
    /// Warnings informationnels CSV (`bank_csv_multiple_profile_matches`,
    /// `bank_csv_profile_auto_matched`, etc.) — non-bloquants, mappés
    /// vers i18n côté frontend.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub informational: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceMismatchPayload {
    pub opening: String,
    pub closing: String,
    pub sum: String,
    pub diff: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsupportedCurrencyPayload {
    pub currency: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncodingMismatchPayload {
    pub profile: String,
    pub detected: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFilePayload {
    pub existing_import_id: i64,
    pub existing_filename: String,
    /// L5 (Pass 1 review) — sérialisé en RFC3339 UTC (`...Z`) plutôt
    /// que `NaiveDateTime` sans timezone, pour matcher la spec
    /// §preview-warnings-shape qui montre `"2026-04-12T10:30:00Z"`.
    /// La DB stocke des timestamps naïfs en convention UTC ; la
    /// conversion `naive_utc.and_utc()` est sémantiquement sûre.
    pub existing_imported_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateLineWarning {
    pub new_index: usize,
    pub existing_transaction_id: i64,
    pub key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidLinesPayload {
    pub lines: Vec<crate::errors::CsvLineErrorPayload>,
    pub total_errors: usize,
    pub truncated: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvProfileMatch {
    pub profile_id: i64,
    pub profile_name: String,
    /// `true` si le profil a été matché via le filename pattern
    /// (auto-match), `false` si l'utilisateur a explicitement passé
    /// `bankProfileId` dans le multipart.
    pub auto_matched: bool,
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

const FILENAME_MAX_BYTES: usize = 255;

struct MultipartFields {
    /// Bytes du fichier uploadé. Conservé en `Bytes` (zero-copy depuis
    /// axum) au lieu de `Vec<u8>` (review code Pass 1 H1) pour éviter
    /// le doublement RSS sur upload (10 MiB → 20 MiB).
    file_bytes: axum::body::Bytes,
    filename: String,
    content_type: Option<String>,
    bank_account_id: i64,
    confirm_balance_mismatch: bool,
    /// Story 8-2 — bankProfileId explicite (CSV uniquement).
    bank_profile_id: Option<i64>,
    /// Story 8-2 — Pass 1 H5 + Pass 2 H'3 confirmation explicite.
    /// Wired dans `create_csv` (Pass 1 review G2 H7) : si `parse_csv`
    /// retourne `EncodingMismatch` ET ce flag est `true`, on retry avec
    /// l'encoding détecté + audit log spécial.
    confirm_encoding_mismatch: bool,
    /// Story 8-3 — autorise l'INSERT d'un nouvel import malgré un
    /// fichier déjà importé (`(company_id, file_hash)` matching).
    confirm_duplicate_file: bool,
    /// Story 8-3 — comportement face aux lignes doublons :
    /// `Skip` (default) ignore les doublons, `Import` les persiste.
    confirm_duplicate_lines: ConfirmDuplicateLines,
    /// Story 8-3 — autorise la persistance des lignes valides d'un
    /// CSV partiellement défaillant (warnings.invalidLines retourné).
    confirm_partial_import: bool,
}

/// Story 8-3 — comportement face aux doublons ligne-par-ligne.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfirmDuplicateLines {
    /// Default (sans flag ou `confirmDuplicateLines=skip`) — les
    /// transactions doublons ne sont pas persistées.
    #[default]
    Skip,
    /// `confirmDuplicateLines=import` — toutes les transactions sont
    /// persistées, doublons inclus (audit log discriminant).
    Import,
}

/// Tronque `s` à `max_bytes` octets en respectant les frontières de char
/// UTF-8 (review code Pass 1 M1) — `chars().take(N).collect()` ne borne
/// que le nombre de scalaires Unicode, pas les octets ; un nom de
/// fichier plein d'emoji 4-byte produit jusqu'à 1020 octets et casse
/// `VARCHAR(255)` MariaDB qui mesure en bytes.
fn truncate_to_byte_len(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

async fn parse_multipart(mut multipart: Multipart) -> Result<MultipartFields, AppError> {
    let mut file_bytes: Option<axum::body::Bytes> = None;
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut bank_account_id: Option<i64> = None;
    let mut confirm_balance_mismatch = false;
    let mut bank_profile_id: Option<i64> = None;
    let mut confirm_encoding_mismatch = false;
    let mut confirm_duplicate_file = false;
    // M2 (Pass 1 review) — _seen guard parity with confirmDuplicateLines:
    // reject duplicate confirm-flag fields in multipart so an attacker
    // cannot flip `true` → `false` by appending a second occurrence.
    let mut confirm_duplicate_file_seen = false;
    let mut confirm_duplicate_lines = ConfirmDuplicateLines::Skip;
    let mut confirm_duplicate_lines_seen = false;
    let mut confirm_partial_import = false;
    // M3 (Pass 1 review) — same as M2 for confirmPartialImport.
    let mut confirm_partial_import_seen = false;
    // L9 (Pass 1 review) — same _seen guard for confirmBalanceMismatch
    // and confirmEncodingMismatch (parity with the rest of the family).
    let mut confirm_balance_mismatch_seen = false;
    let mut confirm_encoding_mismatch_seen = false;

    while let Some(field) = multipart.next_field().await.map_err(map_multipart_err)? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                if file_bytes.is_some() {
                    // Review code Pass 1 (multipart hardening) : refuser
                    // les champs dupliqués sur les champs sécurité-sensibles
                    // pour éviter les attaques par duplication.
                    return Err(AppError::Validation(
                        "Champ 'file' dupliqué dans le multipart".into(),
                    ));
                }
                let original = field.file_name().unwrap_or("upload.xml").to_string();
                filename = Some(truncate_to_byte_len(&original, FILENAME_MAX_BYTES));
                content_type = field.content_type().map(|s| s.to_string());
                file_bytes = Some(field.bytes().await.map_err(map_multipart_err)?);
            }
            "bankAccountId" => {
                if bank_account_id.is_some() {
                    return Err(AppError::Validation(
                        "Champ 'bankAccountId' dupliqué dans le multipart".into(),
                    ));
                }
                let text = field.text().await.map_err(map_multipart_err)?;
                let id: i64 = text.trim().parse().map_err(|_| {
                    AppError::Validation(format!("bankAccountId invalide : '{text}'"))
                })?;
                if id <= 0 {
                    // Defense-in-depth (review code Pass 1) : rejeter
                    // 0/négatif au boundary plutôt que de laisser
                    // find_by_id_for_company faire la requête DB inutile.
                    return Err(AppError::Validation(
                        "bankAccountId doit être strictement positif".into(),
                    ));
                }
                bank_account_id = Some(id);
            }
            "confirmBalanceMismatch" => {
                if confirm_balance_mismatch_seen {
                    return Err(AppError::Validation(
                        "Champ 'confirmBalanceMismatch' dupliqué dans le multipart".into(),
                    ));
                }
                confirm_balance_mismatch_seen = true;
                let text = field.text().await.map_err(map_multipart_err)?;
                // Review code Pass 1 M3 : case-insensitive — accepter
                // "true"/"True"/"TRUE"/"1" pour les clients non-browser
                // (curl, scripts) qui peuvent envoyer en majuscules.
                confirm_balance_mismatch =
                    matches!(text.trim().to_ascii_lowercase().as_str(), "true" | "1");
            }
            "bankProfileId" => {
                // Story 8-2 — Pass 2 M'8 duplicate guard.
                if bank_profile_id.is_some() {
                    return Err(AppError::Validation(
                        "Champ 'bankProfileId' dupliqué dans le multipart".into(),
                    ));
                }
                let text = field.text().await.map_err(map_multipart_err)?;
                let id: i64 = text.trim().parse().map_err(|_| {
                    AppError::Validation(format!("bankProfileId invalide : '{text}'"))
                })?;
                if id <= 0 {
                    return Err(AppError::Validation(
                        "bankProfileId doit être strictement positif".into(),
                    ));
                }
                bank_profile_id = Some(id);
            }
            "confirmEncodingMismatch" => {
                if confirm_encoding_mismatch_seen {
                    return Err(AppError::Validation(
                        "Champ 'confirmEncodingMismatch' dupliqué dans le multipart".into(),
                    ));
                }
                confirm_encoding_mismatch_seen = true;
                let text = field.text().await.map_err(map_multipart_err)?;
                confirm_encoding_mismatch =
                    matches!(text.trim().to_ascii_lowercase().as_str(), "true" | "1");
            }
            "confirmDuplicateFile" => {
                if confirm_duplicate_file_seen {
                    return Err(AppError::Validation(
                        "Champ 'confirmDuplicateFile' dupliqué dans le multipart".into(),
                    ));
                }
                confirm_duplicate_file_seen = true;
                let text = field.text().await.map_err(map_multipart_err)?;
                confirm_duplicate_file =
                    matches!(text.trim().to_ascii_lowercase().as_str(), "true" | "1");
            }
            "confirmDuplicateLines" => {
                if confirm_duplicate_lines_seen {
                    return Err(AppError::Validation(
                        "Champ 'confirmDuplicateLines' dupliqué dans le multipart".into(),
                    ));
                }
                confirm_duplicate_lines_seen = true;
                let text = field.text().await.map_err(map_multipart_err)?;
                confirm_duplicate_lines = match text.trim().to_ascii_lowercase().as_str() {
                    "skip" => ConfirmDuplicateLines::Skip,
                    "import" => ConfirmDuplicateLines::Import,
                    other => {
                        return Err(AppError::Validation(format!(
                            "confirmDuplicateLines invalide : '{other}' (attendu 'skip' ou 'import')"
                        )));
                    }
                };
            }
            "confirmPartialImport" => {
                if confirm_partial_import_seen {
                    return Err(AppError::Validation(
                        "Champ 'confirmPartialImport' dupliqué dans le multipart".into(),
                    ));
                }
                confirm_partial_import_seen = true;
                let text = field.text().await.map_err(map_multipart_err)?;
                confirm_partial_import =
                    matches!(text.trim().to_ascii_lowercase().as_str(), "true" | "1");
            }
            _ => {
                // Review code Pass 1 M2 : propager l'erreur sur les
                // champs inconnus aussi — sinon un PAYLOAD_TOO_LARGE
                // déclenché en lisant un champ inconnu serait
                // silencieusement masqué et le caller verrait un
                // confusing "champ 'file' manquant".
                field.bytes().await.map_err(map_multipart_err)?;
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
        content_type,
        bank_account_id,
        confirm_balance_mismatch,
        bank_profile_id,
        confirm_encoding_mismatch,
        confirm_duplicate_file,
        confirm_duplicate_lines,
        confirm_partial_import,
    })
}

// ---------------------------------------------------------------------------
// Story 8-2 — Format detection + CSV pipeline helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportFormat {
    Camt053,
    Csv,
}

/// Détecte le format upload (CAMT.053 vs CSV) sur les raw bytes
/// (pas decoded — encoding non encore connu).
///
/// Priorités : extension > MIME > sniff content (cf. spec §csv-detection).
fn detect_import_format(
    filename: &str,
    content_type: Option<&str>,
    raw_first_bytes: &[u8],
) -> Result<ImportFormat, AppError> {
    let lower = filename.to_ascii_lowercase();
    // Priorité 1 — extension
    if lower.ends_with(".xml") {
        return Ok(ImportFormat::Camt053);
    }
    if lower.ends_with(".csv") || lower.ends_with(".txt") {
        return Ok(ImportFormat::Csv);
    }
    // Priorité 2 — MIME
    if let Some(ct) = content_type {
        let ct_lower = ct.to_ascii_lowercase();
        if ct_lower.contains("xml") {
            return Ok(ImportFormat::Camt053);
        }
        if ct_lower.contains("csv") || ct_lower == "text/plain" {
            return Ok(ImportFormat::Csv);
        }
    }
    // Priorité 3 — sniff content sur **raw bytes ASCII-safe**.
    // Pass 1 review G2-BH-4 + G2-EH-9 + G2-AA-3 : un fichier CAMT.053
    // encodé en ISO-8859-1 (cas réel observé) ferait échouer `from_utf8`
    // → `unwrap_or("")` retournerait "" → marqueur XML jamais trouvé.
    // Skip leading whitespace + BOM bytes pour matcher uniquement au
    // début du fichier (un CSV avec `<?xml` dans une cellule au milieu
    // ne déclenche plus le faux positif CAMT).
    let sniff = &raw_first_bytes[..raw_first_bytes.len().min(256)];
    let head_start = sniff
        .iter()
        .position(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0xEF | 0xBB | 0xBF))
        .unwrap_or(sniff.len());
    let head: Vec<u8> = sniff[head_start..]
        .iter()
        .take(50)
        .map(|b| b.to_ascii_lowercase())
        .collect();
    if head.starts_with(b"<?xml") || head.starts_with(b"<document") {
        return Ok(ImportFormat::Camt053);
    }
    // Heuristique CSV : présence de séparateur courant dans la fenêtre.
    if sniff.iter().any(|b| matches!(b, b',' | b';' | b'\t')) {
        return Ok(ImportFormat::Csv);
    }
    Err(AppError::BankImportUnsupportedFormat)
}

/// Convertit un `kesh_import::CsvLineError` vers payload sérialisable.
fn csv_line_error_to_payload(e: CsvLineError) -> crate::errors::CsvLineErrorPayload {
    crate::errors::CsvLineErrorPayload {
        line: e.line,
        code: format!("{:?}", e.code).to_uppercase(),
        value: e.value,
        message_i18n_key: e.message_i18n_key,
    }
}

/// Mappe `CsvError` vers `AppError`.
fn map_csv_error(err: CsvError) -> AppError {
    match err {
        CsvError::EmptyFile { reason } => AppError::BankCsvEmptyFile { reason },
        CsvError::UnsupportedEncoding { detected } => {
            AppError::BankCsvUnsupportedEncoding { detected }
        }
        CsvError::EncodingMismatch { profile, detected } => {
            AppError::BankCsvEncodingMismatch { profile, detected }
        }
        CsvError::DecodingFailed {
            encoding,
            byte_offset,
        } => AppError::BankCsvProfileMisconfigured(format!(
            "Decoding {} failed at byte {}",
            encoding, byte_offset
        )),
        CsvError::MissingHeader => AppError::BankCsvEmptyFile {
            reason: "missing header".into(),
        },
        CsvError::ProfileMisconfigured(reason) => AppError::BankCsvProfileMisconfigured(reason),
        CsvError::PartialFailure {
            errors,
            total_errors,
            truncated,
        } => AppError::BankCsvParsePartialFailure {
            lines: errors.into_iter().map(csv_line_error_to_payload).collect(),
            total_errors,
            truncated,
            reason: None,
        },
        // Pour les variantes non-PartialFailure inline (InvalidDate etc.)
        // — ne devraient pas apparaître au top-level (collectées dans
        // PartialFailure), mais defense-in-depth.
        other => AppError::BankCsvProfileMisconfigured(other.to_string()),
    }
}

/// Résolution du profil CSV (cf. §profile-matching).
///
/// **Pass 1 review G2 H6 — transaction-bound** : signature passe le
/// `pool` séparément (utilisé pour `list_available_profiles` sur le
/// chemin no-match) et un `executor` dédié pour les SELECT critiques
/// (étapes 1 et 2). Le caller `create_csv` passe `&mut **tx` comme
/// executor pour que la résolution + INSERT bank_imports + audit_log
/// vivent dans la même transaction (Pass 2 H'2 + Pass 3 M''4
/// Interprétation A).
///
/// Pass 1 review G2 H8 / G2-AA-6 : `available_profiles` enrichi
/// systématiquement dans tous les chemins d'erreur (vs `Vec::new()`
/// fragile qui dépendait du caller).
///
/// Pass 1 review G2-AA-2 : warning `bank_csv_multiple_profile_matches`
/// retourné via `WarningCollector` mécanisme — multi-match prend le
/// plus récent + push warning (AC #8).
///
/// Returns `(profile, warnings)` — warnings vide si single match
/// ou explicit ID.
///
/// 1. Si `bank_profile_id` explicite → `find_by_id_for_company` (404 si
///    cross-tenant ou inexistant — Pass 1 H8).
/// 2. Sinon → auto-match par `filename_pattern` regex (warning si plusieurs).
/// 3. Sinon → 404 avec `available_profiles` (cap 50, Pass 1 M11).
async fn resolve_csv_profile<'e, E>(
    executor: E,
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_profile_id: Option<i64>,
    filename: &str,
) -> Result<(kesh_db::entities::bank_profile::BankProfile, Vec<String>), AppError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    use kesh_db::repositories::bank_profiles;

    // Branche unique : selon `bank_profile_id`, on appelle UN seul SELECT
    // sur l'executor (pour permettre transaction-bound). Le helper
    // `list_available_profiles` est appelé sur le pool dans le path
    // d'erreur uniquement (pas critique d'être dans la tx).
    if let Some(id) = bank_profile_id {
        match bank_profiles::find_by_id_for_company(executor, company_id, id).await? {
            Some(profile) => return Ok((profile, Vec::new())),
            None => {
                return Err(AppError::BankCsvProfileNotFound {
                    available_profiles: list_available_profiles(pool, company_id).await,
                });
            }
        }
    }

    let matches =
        bank_profiles::find_matching_profiles_for_filename(executor, company_id, filename).await?;
    let count = matches.len();
    match matches.into_iter().next() {
        Some(profile) => {
            let mut warnings = Vec::new();
            if count > 1 {
                // Pass 1 review G2-AA-2 : warning AC #8.
                warnings.push("bank_csv_multiple_profile_matches".to_string());
            }
            Ok((profile, warnings))
        }
        None => Err(AppError::BankCsvProfileNotFound {
            available_profiles: list_available_profiles(pool, company_id).await,
        }),
    }
}

async fn list_available_profiles(
    pool: &sqlx::MySqlPool,
    company_id: i64,
) -> Vec<crate::errors::BankProfileSummary> {
    use kesh_db::repositories::bank_profiles;
    match bank_profiles::list_by_company(pool, company_id, 50, 0).await {
        Ok((profiles, _)) => profiles
            .into_iter()
            .map(|p| crate::errors::BankProfileSummary {
                id: p.id,
                bank_name: p.bank_name,
            })
            .collect(),
        Err(_) => Vec::new(),
    }
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
        // Story 8-2 T5.0.c — CSV désormais accepté.
        kesh_import::SourceFormat::Csv { .. } => Ok(BankImportSourceFormat::Csv),
    }
}

fn source_format_tag_for_kesh_core(fmt: BankImportSourceFormat) -> SourceFormatTag {
    match fmt {
        BankImportSourceFormat::Camt053V04 => SourceFormatTag::Camt053V04,
        BankImportSourceFormat::Camt053V08 => SourceFormatTag::Camt053V08,
        BankImportSourceFormat::Csv => SourceFormatTag::Csv,
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

    // Story 8-2 — détection format upload.
    let format = detect_import_format(
        &fields.filename,
        fields.content_type.as_deref(),
        &fields.file_bytes,
    )?;
    if format == ImportFormat::Csv {
        return preview_csv(&state, current_user.company_id, &bank_account, &fields).await;
    }

    // Hash + parse CAMT.053.
    let file_hash = compute_sha256_hex(&fields.file_bytes);
    let stmts = parse_camt053(&fields.file_bytes).map_err(map_camt_error)?;
    let selection = select_statement_by_iban(stmts, &bank_account.iban)?;
    let stmt = selection.selected;

    // Warnings non-bloquants — structurés (Story 8-3).
    let mut warnings = PreviewWarnings::default();

    // CR-010 #62 / 8-1b — balance_mismatch.
    match core_bank_imports::validate_balance(&stmt) {
        Ok(()) => {}
        Err(kesh_core::errors::CoreError::BankImportBalanceMismatch {
            opening,
            closing,
            sum,
            diff,
        }) => {
            warnings.balance_mismatch = Some(BalanceMismatchPayload {
                opening: money_to_string(opening),
                closing: money_to_string(closing),
                sum: money_to_string(sum),
                diff: money_to_string(diff),
            });
        }
        Err(other) => {
            tracing::warn!("validate_balance unexpected CoreError: {other}");
            return Err(AppError::Internal(format!(
                "validate_balance retourne une variante inattendue : {other}"
            )));
        }
    }
    match core_bank_imports::validate_currency_supported_v0_1(&stmt) {
        Ok(()) => {}
        Err(kesh_core::errors::CoreError::BankImportUnsupportedCurrency(currency)) => {
            warnings.unsupported_currency = Some(UnsupportedCurrencyPayload { currency });
        }
        Err(other) => {
            tracing::warn!("validate_currency unexpected CoreError: {other}");
            return Err(AppError::Internal(format!(
                "validate_currency retourne une variante inattendue : {other}"
            )));
        }
    }

    // Story 8-3 — détection fichier déjà importé (FR43 partie 1).
    if let Some(existing) =
        bank_imports::find_by_company_and_hash(&state.pool, current_user.company_id, &file_hash)
            .await?
    {
        warnings.duplicate_file = Some(duplicate_file_payload(&existing));
    }

    // Story 8-3 — détection ligne-par-ligne (FR43 partie 2).
    // L8 (Pass 1 review) — garde `is_empty()` parité avec preview_csv :
    // si le statement n'a pas de transactions, ne lance pas la requête
    // SQL inutile sur `bank_transactions`.
    if !stmt.transactions.is_empty() {
        warnings.duplicate_lines = compute_duplicate_lines_warnings(
            &state.pool,
            current_user.company_id,
            fields.bank_account_id,
            stmt.period_from,
            stmt.period_to,
            &stmt.transactions,
        )
        .await?;
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
        csv_profile_match: None,
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

    // Story 8-2 — détection format upload + dispatch CSV.
    let format = detect_import_format(
        &fields.filename,
        fields.content_type.as_deref(),
        &fields.file_bytes,
    )?;
    if format == ImportFormat::Csv {
        return create_csv(&state, &current_user, &bank_account, &fields).await;
    }

    let file_hash = compute_sha256_hex(&fields.file_bytes);

    // Story 8-3 — ouvre la transaction AVANT le check applicatif
    // duplicate file pour que le check + INSERT vivent dans la même tx
    // (le UNIQUE a été retiré par migration, donc le check est notre
    // seule barrière — cf. Limitations connues v0.1 L11 race acceptée).
    //
    // M1 (Pass 1 review) — `find_by_company_and_hash` ET
    // `find_in_dedup_window` sont désormais exécutés via `&mut *tx`
    // au lieu de `&state.pool`, conformément à la spec L11 + T6.2 step 5
    // (« le check applicatif est dans la transaction »). Sous MariaDB
    // REPEATABLE READ la fenêtre de race reste théoriquement ouverte
    // par snapshot ; mais sortir le check de la transaction l'élargit
    // arbitrairement et désaligne le code de la documentation.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let mut modifiers: Vec<&'static str> = Vec::new();
    let mut details_extra = serde_json::Map::new();

    if let Some(existing) =
        bank_imports::find_by_company_and_hash(&mut *tx, current_user.company_id, &file_hash)
            .await?
    {
        if !fields.confirm_duplicate_file {
            return Err(AppError::BankImportDuplicateFile {
                existing_import_id: existing.id,
                existing_filename: existing.filename,
            });
        }
        modifiers.push("duplicate_file");
    }

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
    match core_bank_imports::validate_balance(&stmt) {
        Ok(()) => {}
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
            modifiers.push("balance_mismatch");
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

    // Story 8-3 — détection ligne-par-ligne (FR43 partie 2). On charge
    // la fenêtre dans la transaction ouverte (M1, Pass 1 review) et on
    // filtre les drafts selon `confirm_duplicate_lines`.
    let existing = bank_transactions::find_in_dedup_window(
        &mut *tx,
        current_user.company_id,
        fields.bank_account_id,
        draft.period_from,
        draft.period_to,
    )
    .await?;
    let duplicate_lines =
        detect_duplicate_lines_for_imported(&stmt.transactions, fields.bank_account_id, &existing);

    // M5 (Pass 1 review) — invariant d'alignement positionnel :
    // `tx_drafts` (sortie `from_imported`) et `stmt.transactions`
    // (entrée `detect_duplicate_lines_for_imported`) doivent indexer
    // les mêmes transactions dans le même ordre, sinon
    // `apply_duplicate_lines_filter` skipperait les mauvaises rangées.
    debug_assert_eq!(
        tx_drafts.len(),
        stmt.transactions.len(),
        "M5 alignement: tx_drafts.len() doit == stmt.transactions.len()"
    );

    let (final_drafts, final_count) =
        apply_duplicate_lines_filter(tx_drafts, &duplicate_lines, fields.confirm_duplicate_lines);
    if !duplicate_lines.is_empty() {
        match fields.confirm_duplicate_lines {
            ConfirmDuplicateLines::Skip => {
                modifiers.push("duplicate_lines_skipped");
                details_extra.insert(
                    "duplicate_lines_skipped".into(),
                    serde_json::json!(duplicate_lines.len()),
                );
            }
            ConfirmDuplicateLines::Import => {
                modifiers.push("duplicate_lines_imported");
                details_extra.insert(
                    "duplicate_lines_imported".into(),
                    serde_json::json!(duplicate_lines.len()),
                );
            }
        }
    }

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
        transaction_count: final_count,
        imported_by_user_id: draft.imported_by_user_id,
    };
    let new_txs: Vec<NewBankTransaction> = final_drafts
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

    let (header, _txs) =
        bank_imports::create_with_transactions(&mut tx, new_import, new_txs).await?;

    insert_canonical_audit_log(
        &mut tx,
        current_user.user_id,
        header.id,
        &header.filename,
        header.transaction_count,
        source_format_tag_for_kesh_core(source_format).as_db_str(),
        modifiers,
        details_extra,
        None, // CAMT path : pas de profile_id
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok((StatusCode::CREATED, Json(import_to_response(header))))
}

/// `GET /api/v1/bank-imports` — liste paginée multi-tenant (KF-002).
///
/// Review code Pass 1 H5 : `total` retourné via `count_by_company_id`
/// (au lieu de `0` hardcodé) pour respecter le contrat
/// `ListResponse<T>.total` honnête vis-à-vis du client TypeScript.
///
/// **Review code Pass 2 L5 — race count/items** : les deux requêtes
/// (`count_by_company_id` puis `find_by_company_id`) ne partagent pas
/// de transaction. Un INSERT concurrent entre les deux peut faire
/// diverger légèrement `total` et `items.len()`. Pour v0.1 c'est
/// acceptable (le frontend recharge périodiquement et l'écart est
/// borné à 1 par tx concurrente). Si Story 8-3 ajoute une UI
/// pagination strictement consistante, wrapper les deux queries dans
/// une `Transaction` `READ COMMITTED`.
pub async fn list(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListBankImportsQuery>,
) -> Result<Json<ListResponse<BankImportResponse>>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let total = bank_imports::count_by_company_id(
        &state.pool,
        current_user.company_id,
        query.bank_account_id,
    )
    .await?;

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
        total,
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

/// Story 8-3 — sérialise un `BankImport` existant en payload preview.
fn duplicate_file_payload(existing: &BankImport) -> DuplicateFilePayload {
    DuplicateFilePayload {
        existing_import_id: existing.id,
        existing_filename: existing.filename.clone(),
        // L5 — naive→UTC en convention DB (timestamps stockés en UTC).
        existing_imported_at: existing.imported_at.and_utc(),
    }
}

/// Story 8-3 — calcule les warnings `duplicateLines` en chargeant la
/// fenêtre `[period_from, period_to]` et en comparant via
/// [`detect_duplicate_lines`]. Retourne `Vec` vide si aucun doublon.
///
/// Variante préview (utilise `&MySqlPool`). Pour `create`, voir la
/// version transaction-bound dans le handler.
async fn compute_duplicate_lines_warnings(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    period_from: chrono::NaiveDate,
    period_to: chrono::NaiveDate,
    new_txs: &[kesh_import::ImportedTransaction],
) -> Result<Vec<DuplicateLineWarning>, AppError> {
    let existing = bank_transactions::find_in_dedup_window(
        pool,
        company_id,
        bank_account_id,
        period_from,
        period_to,
    )
    .await?;
    Ok(detect_duplicate_lines_for_imported(
        new_txs,
        bank_account_id,
        &existing,
    ))
}

/// Story 8-3 — filtre les drafts selon `confirmDuplicateLines` :
/// `Skip` retire les drafts dont `new_index ∈ duplicate_lines`,
/// `Import` les conserve tous.
///
/// Retourne `(filtered_drafts, count i32)` — le compteur est calculé
/// avec `i32::try_from` saturé à `i32::MAX` pour matcher le contrat
/// `transaction_count: INT NOT NULL` côté DB.
fn apply_duplicate_lines_filter(
    drafts: Vec<BankTransactionDraft>,
    duplicate_lines: &[DuplicateLineWarning],
    mode: ConfirmDuplicateLines,
) -> (Vec<BankTransactionDraft>, i32) {
    let filtered: Vec<BankTransactionDraft> = match mode {
        ConfirmDuplicateLines::Import => drafts,
        ConfirmDuplicateLines::Skip if duplicate_lines.is_empty() => drafts,
        ConfirmDuplicateLines::Skip => {
            let dup_set: std::collections::HashSet<usize> =
                duplicate_lines.iter().map(|d| d.new_index).collect();
            drafts
                .into_iter()
                .enumerate()
                .filter_map(|(i, d)| if dup_set.contains(&i) { None } else { Some(d) })
                .collect()
        }
    };
    let count = i32::try_from(filtered.len()).unwrap_or(i32::MAX);
    (filtered, count)
}

/// Story 8-3 §audit-log-actions — insère une **action canonique unique**
/// `bank_import.created` avec `details_json.modifiers: [..]` triés
/// alphabétiquement pour discriminer les variantes (balance_mismatch,
/// duplicate_file, partial, encoding_mismatch, etc.).
///
/// Une seule entrée audit par import, quel que soit le nombre de
/// modifiers actifs. Cf. spec §audit-log-actions.
#[allow(clippy::too_many_arguments)]
async fn insert_canonical_audit_log(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    import_id: i64,
    filename: &str,
    transaction_count: i32,
    source_format_db_str: &str,
    mut modifiers: Vec<&'static str>,
    details_extra: serde_json::Map<String, serde_json::Value>,
    csv_profile: Option<&kesh_db::entities::BankProfile>,
) -> Result<(), AppError> {
    modifiers.sort();
    modifiers.dedup();
    let mut details = serde_json::json!({
        "filename": filename,
        "transaction_count": transaction_count,
        "source_format": source_format_db_str,
        "modifiers": modifiers,
    });
    let details_obj = details
        .as_object_mut()
        .expect("details_json est toujours un object");
    for (k, v) in details_extra {
        details_obj.insert(k, v);
    }
    if let Some(p) = csv_profile {
        details_obj.insert("bank_profile_id".into(), serde_json::json!(p.id));
        details_obj.insert(
            "bank_profile_name".into(),
            serde_json::json!(p.bank_name.clone()),
        );
    }

    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry {
            user_id,
            action: "bank_import.created".to_string(),
            entity_type: "bank_imports".to_string(),
            entity_id: import_id,
            details_json: Some(details),
        },
    )
    .await?;
    Ok(())
}

/// Story 8-3 — wrapper qui mappe `&[ImportedTransaction]` → drafts +
/// `&[BankTransaction]` → `(id, DuplicateKey)` puis appelle
/// [`detect_duplicate_lines`]. Retourne les warnings sérialisables.
fn detect_duplicate_lines_for_imported(
    new_txs: &[kesh_import::ImportedTransaction],
    bank_account_id: i64,
    existing: &[kesh_db::entities::BankTransaction],
) -> Vec<DuplicateLineWarning> {
    let drafts: Vec<BankTransactionDraft> =
        new_txs.iter().map(BankTransactionDraft::from).collect();
    let existing_keys: Vec<(i64, DuplicateKey)> = existing
        .iter()
        .map(|t| {
            (
                t.id,
                dedup_key_scalar(
                    t.booking_date,
                    t.amount,
                    t.reference.as_deref(),
                    t.end_to_end_id.as_deref(),
                    t.transaction_id.as_deref(),
                    t.bank_account_id,
                ),
            )
        })
        .collect();
    detect_duplicate_lines(&drafts, bank_account_id, &existing_keys)
        .into_iter()
        .map(|d: DuplicateLine| DuplicateLineWarning {
            new_index: d.new_index,
            existing_transaction_id: d.existing_transaction_id,
            key: d.key,
        })
        .collect()
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

// ---------------------------------------------------------------------------
// Story 8-2 — CSV preview + create handlers
// ---------------------------------------------------------------------------

/// Construit un `kesh_import::CsvProfile` depuis l'entité DB.
fn db_profile_to_csv_profile(
    profile: &kesh_db::entities::bank_profile::BankProfile,
) -> Result<kesh_import::CsvProfile, AppError> {
    let column_mapping = profile.parse_column_mapping().map_err(AppError::Database)?;
    Ok(kesh_import::CsvProfile {
        bank_name: profile.bank_name.clone(),
        filename_pattern: profile.filename_pattern.clone(),
        column_mapping,
        date_format: profile.date_format.clone(),
        decimal_separator: profile.decimal_separator_char(),
        field_separator: profile.field_separator_char(),
        encoding: profile.encoding.clone(),
        header_row_count: profile.header_row_count,
    })
}

async fn preview_csv(
    state: &AppState,
    company_id: i64,
    bank_account: &kesh_db::entities::BankAccount,
    fields: &MultipartFields,
) -> Result<Json<BankImportPreviewResponse>, AppError> {
    // Preview ne persiste pas — on utilise le pool directement (pas
    // besoin de transaction-bound). Pass 1 review G2 H6 : la race
    // condition n'est critique que pour create_csv.
    let (profile, informational) = resolve_csv_profile(
        &state.pool,
        &state.pool,
        company_id,
        fields.bank_profile_id,
        &fields.filename,
    )
    .await?;

    let csv_profile = db_profile_to_csv_profile(&profile)?;

    // Story 8-3 — `parse_csv_collect` retourne `ParseCsvOutcome` qui
    // discrimine `AllValid` / `PartialFailure` / `HardFailure`. Sur
    // EncodingMismatch (HardFailure), 8-2 Pass 3 BH3-1 préserve le
    // comportement preview = 200 OK + warning + retry sans encoding
    // forcé.
    let mut warnings = PreviewWarnings {
        informational,
        ..Default::default()
    };

    let outcome = parse_csv_collect(&fields.file_bytes, &csv_profile);
    let (stmt, invalid_lines_payload) = match outcome {
        ParseCsvOutcome::AllValid(s) => (s, None),
        ParseCsvOutcome::PartialFailure {
            valid,
            errors,
            total_errors,
            truncated,
        } => {
            let payload = InvalidLinesPayload {
                lines: errors.into_iter().map(csv_line_error_to_payload).collect(),
                total_errors,
                truncated,
            };
            (valid, Some(payload))
        }
        ParseCsvOutcome::HardFailure(kesh_import::CsvError::EncodingMismatch {
            profile: p,
            detected,
        }) => {
            // Pass 3 review BH3-1 — retry avec auto-détection.
            warnings.encoding_mismatch = Some(EncodingMismatchPayload {
                profile: p,
                detected,
            });
            let mut forced = csv_profile.clone();
            forced.encoding = None;
            match parse_csv_collect(&fields.file_bytes, &forced) {
                ParseCsvOutcome::AllValid(s) => (s, None),
                ParseCsvOutcome::PartialFailure {
                    valid,
                    errors,
                    total_errors,
                    truncated,
                } => {
                    let payload = InvalidLinesPayload {
                        lines: errors.into_iter().map(csv_line_error_to_payload).collect(),
                        total_errors,
                        truncated,
                    };
                    (valid, Some(payload))
                }
                ParseCsvOutcome::HardFailure(e) => return Err(map_csv_error(e)),
            }
        }
        ParseCsvOutcome::HardFailure(e) => return Err(map_csv_error(e)),
    };
    warnings.invalid_lines = invalid_lines_payload;

    let auto_matched = fields.bank_profile_id.is_none();
    let csv_profile_match = Some(CsvProfileMatch {
        profile_id: profile.id,
        profile_name: profile.bank_name.clone(),
        auto_matched,
    });
    if auto_matched {
        warnings
            .informational
            .push("bank_csv_profile_auto_matched".to_string());
    }

    let file_hash = compute_sha256_hex(&fields.file_bytes);

    // Story 8-3 — détection fichier déjà importé (FR43 partie 1).
    if let Some(existing) =
        bank_imports::find_by_company_and_hash(&state.pool, company_id, &file_hash).await?
    {
        warnings.duplicate_file = Some(duplicate_file_payload(&existing));
    }

    // Story 8-3 — détection ligne-par-ligne (FR43 partie 2). Skip si
    // pas de transactions valides (cas all-invalid CSV).
    if !stmt.transactions.is_empty() {
        warnings.duplicate_lines = compute_duplicate_lines_warnings(
            &state.pool,
            company_id,
            fields.bank_account_id,
            stmt.period_from,
            stmt.period_to,
            &stmt.transactions,
        )
        .await?;
    }

    let preview_txs: Vec<PreviewTransaction> = stmt
        .transactions
        .iter()
        .map(|t| PreviewTransaction {
            booking_date: t.booking_date,
            value_date: t.value_date,
            amount: t.amount,
            currency: t.currency.clone(),
            reference: t.reference.clone(),
            details: t.details.clone(),
            counterparty_iban: t.counterparty_iban.clone(),
            counterparty_name: t.counterparty_name.clone(),
        })
        .collect();

    Ok(Json(BankImportPreviewResponse {
        file_hash,
        filename: fields.filename.clone(),
        source_format: "CSV".to_string(),
        selected_statement: PreviewStatement {
            statement_id: None,
            account_iban: bank_account.iban.clone(),
            currency: stmt.currency,
            period_from: stmt.period_from,
            period_to: stmt.period_to,
            opening_balance: None,
            closing_balance: None,
        },
        ignored_statements: Vec::new(),
        warnings,
        transactions: preview_txs,
        csv_profile_match,
    }))
}

async fn create_csv(
    state: &AppState,
    current_user: &CurrentUser,
    bank_account: &kesh_db::entities::BankAccount,
    fields: &MultipartFields,
) -> Result<(StatusCode, Json<BankImportResponse>), AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let file_hash = compute_sha256_hex(&fields.file_bytes);
    let mut modifiers: Vec<&'static str> = Vec::new();
    let mut details_extra = serde_json::Map::new();

    // Story 8-3 — duplicate file check applicatif (FR43 partie 1)
    // AVANT le parse CSV (fail-fast applicatif §error-precedence-order
    // ordre 6, plus précoce que le parse).
    //
    // M1 (Pass 1 review) — passe `&mut *tx` plutôt que `&state.pool`
    // pour aligner le code sur le modèle « check applicatif dans la tx »
    // (spec L11 + T6.2 step 5).
    if let Some(existing) =
        bank_imports::find_by_company_and_hash(&mut *tx, current_user.company_id, &file_hash)
            .await?
    {
        if !fields.confirm_duplicate_file {
            return Err(AppError::BankImportDuplicateFile {
                existing_import_id: existing.id,
                existing_filename: existing.filename,
            });
        }
        modifiers.push("duplicate_file");
    }

    // Pass 1 review G2 H6 — transaction-bound profile resolution.
    let (profile, _info) = resolve_csv_profile(
        &mut *tx,
        &state.pool,
        current_user.company_id,
        fields.bank_profile_id,
        &fields.filename,
    )
    .await?;

    let csv_profile = db_profile_to_csv_profile(&profile)?;

    // Story 8-3 — `parse_csv_collect` retourne `ParseCsvOutcome`. Le
    // partial commit est autorisé via `confirmPartialImport=true`.
    // EncodingMismatch wiring 8-2 H7 préservé.
    let outcome = parse_csv_collect(&fields.file_bytes, &csv_profile);

    let outcome = match outcome {
        ParseCsvOutcome::HardFailure(kesh_import::CsvError::EncodingMismatch { .. })
            if fields.confirm_encoding_mismatch =>
        {
            modifiers.push("encoding_mismatch");
            let mut forced = csv_profile.clone();
            forced.encoding = None;
            parse_csv_collect(&fields.file_bytes, &forced)
        }
        other => other,
    };

    let stmt = match outcome {
        ParseCsvOutcome::AllValid(s) => s,
        ParseCsvOutcome::PartialFailure {
            valid,
            errors,
            total_errors,
            truncated,
        } => {
            if !fields.confirm_partial_import {
                return Err(AppError::BankCsvParsePartialFailure {
                    lines: errors.into_iter().map(csv_line_error_to_payload).collect(),
                    total_errors,
                    truncated,
                    reason: None,
                });
            }
            // AC #16 — partial commit avec 0 lignes valides → reject 422
            // discriminant `reason = "no_valid_lines_to_commit"`.
            if valid.transactions.is_empty() {
                return Err(AppError::BankCsvParsePartialFailure {
                    lines: errors.into_iter().map(csv_line_error_to_payload).collect(),
                    total_errors,
                    truncated,
                    reason: Some("no_valid_lines_to_commit"),
                });
            }
            modifiers.push("partial");
            details_extra.insert(
                "partial_invalid_lines".into(),
                serde_json::json!(errors.len()),
            );
            details_extra.insert(
                "partial_total_errors".into(),
                serde_json::json!(total_errors),
            );
            details_extra.insert("partial_truncated".into(), serde_json::json!(truncated));
            valid
        }
        ParseCsvOutcome::HardFailure(e) => return Err(map_csv_error(e)),
    };

    let imported_at = chrono::Utc::now().naive_utc();
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

    // Story 8-3 — détection ligne-par-ligne (FR43 partie 2). M1 (Pass 1
    // review) — find_in_dedup_window via `&mut *tx` au lieu de
    // `&state.pool` (alignement modèle spec L11).
    //
    // M9 (Pass 1 review) — defensive guard : si `parse_csv_collect`
    // retourne PartialFailure { valid: empty }, la sentinel
    // `empty_valid_sentinel_date()` (1970-01-01) atterrit dans
    // draft.period_from. Le check `valid.transactions.is_empty()` plus
    // haut a déjà rejeté ce cas avec `reason = "no_valid_lines_to_commit"`,
    // mais on ajoute un debug_assert pour matérialiser l'invariant.
    debug_assert_ne!(
        draft.period_from,
        kesh_import::empty_valid_sentinel_date(),
        "M9: période sentinel ne doit pas atteindre find_in_dedup_window"
    );
    let existing = bank_transactions::find_in_dedup_window(
        &mut *tx,
        current_user.company_id,
        fields.bank_account_id,
        draft.period_from,
        draft.period_to,
    )
    .await?;
    let duplicate_lines =
        detect_duplicate_lines_for_imported(&stmt.transactions, fields.bank_account_id, &existing);

    // M5 (Pass 1 review) — invariant d'alignement positionnel ; cf. CAMT path.
    debug_assert_eq!(
        tx_drafts.len(),
        stmt.transactions.len(),
        "M5 alignement: tx_drafts.len() doit == stmt.transactions.len()"
    );

    let (final_drafts, final_count) =
        apply_duplicate_lines_filter(tx_drafts, &duplicate_lines, fields.confirm_duplicate_lines);
    if !duplicate_lines.is_empty() {
        match fields.confirm_duplicate_lines {
            ConfirmDuplicateLines::Skip => {
                modifiers.push("duplicate_lines_skipped");
                details_extra.insert(
                    "duplicate_lines_skipped".into(),
                    serde_json::json!(duplicate_lines.len()),
                );
            }
            ConfirmDuplicateLines::Import => {
                modifiers.push("duplicate_lines_imported");
                details_extra.insert(
                    "duplicate_lines_imported".into(),
                    serde_json::json!(duplicate_lines.len()),
                );
            }
        }
    }

    let new_import = NewBankImport {
        company_id: draft.company_id,
        bank_account_id: draft.bank_account_id,
        filename: draft.filename,
        file_hash: draft.file_hash,
        source_format: BankImportSourceFormat::Csv,
        statement_id: None,
        period_from: draft.period_from,
        period_to: draft.period_to,
        opening_balance: draft.opening_balance.map(|m| m.amount()),
        closing_balance: draft.closing_balance.map(|m| m.amount()),
        transaction_count: final_count,
        imported_by_user_id: draft.imported_by_user_id,
    };
    let new_txs: Vec<NewBankTransaction> = final_drafts
        .into_iter()
        .map(|d| NewBankTransaction {
            company_id: current_user.company_id,
            bank_account_id: fields.bank_account_id,
            booking_date: d.booking_date,
            value_date: d.value_date,
            amount: d.amount.amount(),
            currency: d.currency,
            reference: d.reference,
            details: d.details,
            end_to_end_id: d.end_to_end_id,
            transaction_id: d.transaction_id,
            counterparty_iban: d.counterparty_iban,
            counterparty_name: d.counterparty_name,
        })
        .collect();

    let (header, _txs) =
        bank_imports::create_with_transactions(&mut tx, new_import, new_txs).await?;

    insert_canonical_audit_log(
        &mut tx,
        current_user.user_id,
        header.id,
        &header.filename,
        header.transaction_count,
        "CSV",
        modifiers,
        details_extra,
        Some(&profile),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let _ = bank_account; // validé en amont
    Ok((StatusCode::CREATED, Json(import_to_response(header))))
}
