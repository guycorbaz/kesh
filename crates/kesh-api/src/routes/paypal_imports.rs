//! Route `POST /api/v1/paypal-imports` — Story 38-11b (Epic 38, phase 1).
//!
//! Importe d'un geste le CSV PayPal « Balance affecting transactions » d'un
//! **compte PayPal** (multipart : `file`, `account`), **ventilé par devise** :
//! chaque devise du fichier va au compte bancaire Kesh, solde de ce compte
//! PayPal dans cette devise (`bank_accounts.provider = 'paypal'`,
//! `provider_account` = le libellé, devise = celle du compte du plan lié).
//!
//! Comptable+, clés d'API `read-write` admises, comme `POST
//! /api/v1/bank-imports`. Taille maximale : celle de l'import bancaire.
//!
//! **Ordre des refus**, rien d'écrit à aucun : `400 VALIDATION_ERROR` (champ
//! manquant) → `404 PROVIDER_ACCOUNT_NOT_FOUND` → `422 PAYPAL_CSV_INVALID` →
//! `422 PROVIDER_CURRENCY_ACCOUNT_MISSING` → `409 PROVIDER_CURRENCY_DUPLICATE`
//! → `409 BANK_ACCOUNT_CLOSED` → `422 BANK_IMPORT_DUPLICATE_FILE` (ce dernier
//! jugé sous les verrous, dans la transaction d'écriture). Un verrou de compte
//! non obtenu rend `409 RECONCILIATION_ACCOUNT_LOCKED`.
//!
//! **Ventilation** ([`ventilate`], fonction pure) : lignes `Status ≠
//! Completed` (sans casse), lignes `Memo` et **paires neutres** de retenue
//! écartées et comptées ; chaque ligne restante donne une transaction pour son
//! brut et une pour ses frais (chacune de son signe, [`transactions_of_row`]).
//! **Doublons** : une transaction du même compte bancaire porte déjà
//! l'identifiant (brut ou `:FEE`), ou une ligne précédente du fichier porte le
//! même identifiant → la ligne est ignorée. La fenêtre date-montant-référence
//! des imports bancaires ne s'applique pas : deux ventes du même montant le
//! même jour sont deux transactions.
//!
//! **Verrous** : ceux de **chaque** compte visé, par identifiant croissant
//! (`with_account_locks`), sur une connexion dédiée ; l'écriture se fait dans
//! une **seconde** transaction, une seule pour toutes les devises (tout ou
//! rien), patron de `create_under_account_lock`.

use std::collections::{BTreeMap, HashSet};

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDate;
use serde::Serialize;

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::bank_account::PROVIDER_PAYPAL;
use kesh_db::entities::bank_import::{BankImportSourceFormat, NewBankImport};
use kesh_db::entities::bank_transaction::NewBankTransaction;
use kesh_db::errors::DbError;
use kesh_db::repositories::bank_accounts::{self, ProviderBalance};
use kesh_db::repositories::{audit_log, bank_imports, bank_transactions};
use kesh_import::paypal::{BalanceImpact, PaypalCsvError, PaypalRow, parse_paypal_csv};
use kesh_reconciliation::{ReconciliationError, with_account_locks};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::bank_imports::{
    ImportAuditAction, compute_sha256_hex, import_create_error, insert_canonical_audit_log,
};
use crate::routes::bank_statements_manual::MANUAL_STATEMENT_LOCK_TIMEOUT_SECS;

/// Suffixe de l'identifiant de la transaction des **frais** d'une ligne
/// PayPal : `<id>:FEE`. Un identifiant PayPal n'a jamais de `:` (38-11a,
/// AC-3), si bien que le suffixe ne peut heurter un identifiant réel.
pub const PAYPAL_FEE_SUFFIX: &str = ":FEE";

/// Types PayPal d'une **retenue neutre** : une paire de deux lignes de ces
/// types (même devise, même `Reference Txn ID` non vide, brut opposé, frais
/// nuls) ne change pas le solde et s'écarte de l'import. Liste fermée.
pub const PAYPAL_NEUTRAL_PAIR_TYPES: [&str; 6] = [
    "Payment Review Hold",
    "Payment Review Release",
    "Hold on Balance for Dispute Investigation",
    "Reversal of General Account Hold",
    "Account Hold for Open Authorization",
    "Reversal of Account Hold",
];

/// Le seul statut importé (comparé sans casse).
const COMPLETED: &str = "Completed";

/// Bornes des colonnes texte de `bank_transactions`, en caractères (le libellé
/// suit la borne de la saisie manuelle).
const DETAILS_MAX_CHARS: usize = 500;
const NAME_MAX_CHARS: usize = 255;
const PROVIDER_TYPE_MAX_CHARS: usize = 64;
const EMAIL_MAX_CHARS: usize = 254;
const FILENAME_MAX_BYTES: usize = 255;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// Une ligne écartée parce que son statut n'est pas `Completed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotCompletedLine {
    pub transaction_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub status: String,
}

/// Les lignes écartées avant tout, par motif (AC-5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedLines {
    pub not_completed: Vec<NotCompletedLine>,
    /// Lignes `Balance Impact = Memo` (statut `Completed`).
    pub memo: usize,
    /// Paires de retenue neutres (une paire = deux lignes).
    pub neutral_pairs: usize,
}

/// Compte rendu d'une devise : son compte bancaire, l'entrée `bank_imports`
/// créée (`null` si toutes ses lignes étaient des doublons), et les nombres de
/// **transactions** insérées et ignorées comme doublons.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyReport {
    pub currency: String,
    pub bank_account_id: i64,
    pub import_id: Option<i64>,
    pub inserted: usize,
    pub duplicates: usize,
}

/// Réponse `201` de l'import (AC-6).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaypalImportResponse {
    pub account: String,
    pub currencies: Vec<CurrencyReport>,
    pub skipped: SkippedLines,
}

// ---------------------------------------------------------------------------
// Ventilation (pure)
// ---------------------------------------------------------------------------

/// Les lignes du fichier, filtrées et rangées par devise (ordre du fichier).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ventilation {
    pub by_currency: BTreeMap<String, Vec<PaypalRow>>,
    pub skipped: SkippedLines,
}

/// Ventile les lignes lues par 38-11a — Story 38-11b (AC-5). Dans l'ordre :
/// statut ≠ `Completed` (sans casse) → `notCompleted` ; `Memo` → `memo` ;
/// paires neutres → `neutralPairs` (une retenue sans sa levée dans le fichier
/// reste importée). Les lignes restantes sont rangées par devise.
pub fn ventilate(rows: &[PaypalRow]) -> Ventilation {
    let mut skipped = SkippedLines::default();
    let mut kept: Vec<&PaypalRow> = Vec::new();
    for row in rows {
        if !row.status.trim().eq_ignore_ascii_case(COMPLETED) {
            skipped.not_completed.push(NotCompletedLine {
                transaction_id: row.transaction_id.clone(),
                kind: row.kind.clone(),
                status: row.status.clone(),
            });
        } else if row.balance_impact == BalanceImpact::Memo {
            skipped.memo += 1;
        } else {
            kept.push(row);
        }
    }

    let neutral = neutral_pair_indices(&kept);
    skipped.neutral_pairs = neutral.len() / 2;

    let mut by_currency: BTreeMap<String, Vec<PaypalRow>> = BTreeMap::new();
    for (i, row) in kept.into_iter().enumerate() {
        if !neutral.contains(&i) {
            by_currency
                .entry(row.currency.clone())
                .or_default()
                .push(row.clone());
        }
    }
    Ventilation {
        by_currency,
        skipped,
    }
}

/// Indices (dans `rows`) des lignes appariées en **paires neutres** : deux
/// lignes de types [`PAYPAL_NEUTRAL_PAIR_TYPES`], même devise, même `Reference
/// Txn ID` non vide, brut opposé non nul, frais nuls. Chaque ligne entre dans
/// une paire au plus ; on apparie à la première ligne libre qui convient, dans
/// l'ordre du fichier.
fn neutral_pair_indices(rows: &[&PaypalRow]) -> HashSet<usize> {
    let candidate = |r: &PaypalRow| {
        PAYPAL_NEUTRAL_PAIR_TYPES.contains(&r.kind.trim())
            && r.fee.is_zero()
            && !r.gross.is_zero()
            && r.reference_txn_id.as_deref().is_some_and(|s| !s.is_empty())
    };
    let mut paired = HashSet::new();
    for i in 0..rows.len() {
        if paired.contains(&i) || !candidate(rows[i]) {
            continue;
        }
        let a = rows[i];
        let partner = (i + 1..rows.len()).find(|&j| {
            let b = rows[j];
            !paired.contains(&j)
                && candidate(b)
                && b.currency == a.currency
                && b.reference_txn_id == a.reference_txn_id
                && b.gross == -a.gross
        });
        if let Some(j) = partner {
            paired.insert(i);
            paired.insert(j);
        }
    }
    paired
}

/// Coupe `s` à `max` caractères.
fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Coupe `s` à `max` octets, sur une frontière de caractère.
fn truncate_bytes(s: &str, max: usize) -> String {
    let mut end = s.len().min(max);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Les identifiants des transactions qu'une ligne produit : le brut (si non
/// nul) et les frais (si non nuls, `<id>:FEE`).
pub fn row_transaction_ids(row: &PaypalRow) -> Vec<String> {
    let mut ids = Vec::with_capacity(2);
    if !row.gross.is_zero() {
        ids.push(row.transaction_id.clone());
    }
    if !row.fee.is_zero() {
        ids.push(format!("{}{PAYPAL_FEE_SUFFIX}", row.transaction_id));
    }
    ids
}

/// Les transactions d'une ligne — Story 38-11b (AC-5) : le **brut** (si non
/// nul) et les **frais** (si non nuls, quel que soit leur signe : négatifs
/// d'ordinaire, positifs sur certains remboursements). La somme des deux est le
/// `Net` de la ligne (contrôlé par 38-11a).
pub fn transactions_of_row(
    row: &PaypalRow,
    company_id: i64,
    bank_account_id: i64,
) -> Vec<NewBankTransaction> {
    let opt = |v: &Option<String>, max: usize| {
        v.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| truncate_chars(s, max))
    };
    let email = if row.gross.is_sign_negative() {
        opt(&row.to_email, EMAIL_MAX_CHARS)
    } else {
        opt(&row.from_email, EMAIL_MAX_CHARS)
    };
    let kind = row.kind.trim().to_string();
    let base = |amount, transaction_id: String, details: String| NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date: row.date,
        value_date: None,
        amount,
        currency: row.currency.clone(),
        reference: Some(format!("TXN {}", row.transaction_id)),
        details: truncate_chars(&details, DETAILS_MAX_CHARS),
        end_to_end_id: opt(&row.reference_txn_id, NAME_MAX_CHARS),
        transaction_id: Some(transaction_id),
        counterparty_iban: None,
        counterparty_name: opt(&row.name, NAME_MAX_CHARS),
        provider_type: Some(truncate_chars(&kind, PROVIDER_TYPE_MAX_CHARS)),
        counterparty_email: email.clone(),
    };

    let mut out = Vec::with_capacity(2);
    if !row.gross.is_zero() {
        let details = [
            Some(kind.as_str()),
            row.item_title.as_deref(),
            row.subject.as_deref(),
            row.note.as_deref(),
        ]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        out.push(base(row.gross, row.transaction_id.clone(), details));
    }
    if !row.fee.is_zero() {
        out.push(base(
            row.fee,
            format!("{}{PAYPAL_FEE_SUFFIX}", row.transaction_id),
            format!("Frais PayPal · {kind}"),
        ));
    }
    out
}

/// Écarte les **doublons** d'une devise — Story 38-11b (AC-6) : une ligne dont
/// l'identifiant a déjà paru plus haut dans le fichier, ou dont une transaction
/// (brut ou frais) existe déjà sur le compte (`existing`), est ignorée
/// entière. Rend les lignes gardées et le nombre de **transactions** ignorées.
pub fn dedupe_rows<'a>(
    rows: &'a [PaypalRow],
    existing: &HashSet<String>,
) -> (Vec<&'a PaypalRow>, usize) {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut kept = Vec::new();
    let mut duplicates = 0;
    for row in rows {
        let ids = row_transaction_ids(row);
        let in_file = !seen.insert(row.transaction_id.as_str());
        if in_file || ids.iter().any(|id| existing.contains(id)) {
            duplicates += ids.len();
        } else {
            kept.push(row);
        }
    }
    (kept, duplicates)
}

// ---------------------------------------------------------------------------
// Multipart
// ---------------------------------------------------------------------------

struct PaypalImportForm {
    file_bytes: axum::body::Bytes,
    filename: String,
    account: String,
}

async fn parse_form(mut multipart: Multipart) -> Result<PaypalImportForm, AppError> {
    let mut file: Option<(axum::body::Bytes, String)> = None;
    let mut account: Option<String> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart : {e}")))?
    {
        match field.name() {
            Some("file") => {
                if file.is_some() {
                    return Err(AppError::Validation("champ `file` en double".into()));
                }
                let name = field.file_name().unwrap_or("paypal.csv").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                file = Some((bytes, name));
            }
            Some("account") => {
                if account.is_some() {
                    return Err(AppError::Validation("champ `account` en double".into()));
                }
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                account = Some(text);
            }
            _ => {}
        }
    }
    let (file_bytes, filename) =
        file.ok_or_else(|| AppError::Validation("champ `file` manquant".into()))?;
    let account = account
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::Validation("champ `account` manquant".into()))?;
    let filename = truncate_bytes(filename.trim(), FILENAME_MAX_BYTES);
    Ok(PaypalImportForm {
        file_bytes,
        filename: if filename.is_empty() {
            "paypal.csv".into()
        } else {
            filename
        },
        account,
    })
}

/// Traduit le refus du lecteur (38-11a) en `422 PAYPAL_CSV_INVALID`.
fn csv_error(e: PaypalCsvError) -> AppError {
    let reason = e.code();
    let details = match &e {
        PaypalCsvError::MissingColumns(columns) => serde_json::json!({ "columns": columns }),
        PaypalCsvError::DuplicateColumn(column) => serde_json::json!({ "columns": [column] }),
        PaypalCsvError::Lines(lines) => serde_json::json!({ "lines": lines }),
        PaypalCsvError::Encoding(inner) => serde_json::json!({ "cause": inner.to_string() }),
        PaypalCsvError::Empty
        | PaypalCsvError::DateOrderConflict
        | PaypalCsvError::DateOrderAmbiguous => serde_json::json!({}),
    };
    AppError::PaypalCsvInvalid { reason, details }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// Une devise du fichier et son compte bancaire.
struct Target<'a> {
    currency: String,
    balance: &'a ProviderBalance,
    rows: Vec<PaypalRow>,
    file_hash: String,
}

/// `POST /api/v1/paypal-imports` — voir l'en-tête du module.
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<PaypalImportResponse>), AppError> {
    let form = parse_form(multipart).await?;
    let company_id = current_user.company_id;

    let balances = bank_accounts::list_provider_balances(
        &state.pool,
        company_id,
        PROVIDER_PAYPAL,
        &form.account,
    )
    .await?;
    if balances.is_empty() {
        return Err(AppError::ProviderAccountNotFound {
            account: form.account,
        });
    }

    let csv = parse_paypal_csv(&form.file_bytes).map_err(csv_error)?;
    let ventilation = ventilate(&csv.rows);

    let missing: Vec<String> = ventilation
        .by_currency
        .keys()
        .filter(|c| !balances.iter().any(|b| b.currency() == c.as_str()))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(AppError::ProviderCurrencyAccountMissing {
            currencies: missing,
        });
    }

    let mut targets: Vec<Target<'_>> = Vec::new();
    for (currency, rows) in &ventilation.by_currency {
        let mut matching = balances
            .iter()
            .filter(|b| b.currency() == currency.as_str());
        let first = matching
            .next()
            .expect("devise couverte, contrôlée ci-dessus");
        if let Some(second) = matching.next() {
            return Err(AppError::ProviderCurrencyDuplicate {
                account: form.account.clone(),
                currency: currency.clone(),
                bank_account_id: second.account.id,
            });
        }
        let mut hashed = form.file_bytes.to_vec();
        hashed.push(b':');
        hashed.extend_from_slice(currency.as_bytes());
        targets.push(Target {
            currency: currency.clone(),
            balance: first,
            rows: rows.clone(),
            file_hash: compute_sha256_hex(&hashed),
        });
    }
    if let Some(closed) = targets.iter().find(|t| t.balance.account.is_closed()) {
        return Err(AppError::BankAccountClosed {
            bank_account_id: closed.balance.account.id,
        });
    }

    let ids: Vec<i64> = targets.iter().map(|t| t.balance.account.id).collect();
    let audit_entity_id = balances
        .iter()
        .map(|b| b.account.id)
        .min()
        .expect("au moins un compte, contrôlé ci-dessus");
    let mut lock_tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let mut outcome: Option<Result<Vec<CurrencyReport>, AppError>> = None;
    let slot = &mut outcome;
    let (pool, user, form_ref, targets_ref, skipped) = (
        &state.pool,
        &current_user,
        &form,
        &targets,
        &ventilation.skipped,
    );
    let locked = with_account_locks(
        &mut lock_tx,
        company_id,
        &ids,
        MANUAL_STATEMENT_LOCK_TIMEOUT_SECS,
        async move |_held| {
            *slot = Some(
                write_under_locks(pool, user, form_ref, targets_ref, skipped, audit_entity_id)
                    .await,
            );
            Ok(())
        },
    )
    .await;
    drop(lock_tx);

    let currencies = match (outcome, locked) {
        (Some(written), Ok(())) => written?,
        // L'écriture a conclu ; seul un `RELEASE_LOCK` a échoué (verrou rendu
        // au retour de la connexion au pool, #842) : on rend son issue.
        (Some(written), Err(e)) => {
            tracing::error!(error = ?e, "import PayPal : libération des verrous en échec");
            written?
        }
        (
            None,
            Err(ReconciliationError::AccountLocked {
                bank_account_id,
                timeout_secs,
            }),
        ) => {
            return Err(AppError::ReconciliationAccountLocked {
                bank_account_id,
                timeout_secs,
            });
        }
        (None, Err(ReconciliationError::Database(e))) => {
            return Err(AppError::Database(DbError::Sqlx(e)));
        }
        (None, other) => {
            tracing::error!(?other, "import PayPal : verrous des comptes non acquis");
            return Err(AppError::Internal(
                "verrous des comptes bancaires non acquis".to_string(),
            ));
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(PaypalImportResponse {
            account: form.account,
            currencies,
            skipped: ventilation.skipped,
        }),
    ))
}

/// L'écriture, **sous les verrous** des comptes visés : une seule transaction
/// pour toutes les devises (tout ou rien), conclue avant que les verrous soient
/// rendus. Empreinte du fichier par devise (`422 BANK_IMPORT_DUPLICATE_FILE`
/// si l'une est déjà importée), doublons lus dans la transaction, une entrée
/// `bank_imports` par devise qui insère au moins une transaction, traces
/// `bank_import.created` (une par entrée) et `paypal_import.created`.
async fn write_under_locks(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    form: &PaypalImportForm,
    targets: &[Target<'_>],
    skipped: &SkippedLines,
    audit_entity_id: i64,
) -> Result<Vec<CurrencyReport>, AppError> {
    let company_id = current_user.company_id;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    for target in targets {
        if let Some(existing) =
            bank_imports::find_by_company_and_hash(&mut *tx, company_id, &target.file_hash).await?
        {
            let _ = tx.rollback().await;
            return Err(AppError::BankImportDuplicateFile {
                existing_import_id: existing.id,
                existing_filename: existing.filename,
            });
        }
    }

    let mut reports = Vec::with_capacity(targets.len());
    for target in targets {
        let bank_account_id = target.balance.account.id;
        let candidate_ids: Vec<String> = target.rows.iter().flat_map(row_transaction_ids).collect();
        let existing = bank_transactions::existing_transaction_ids(
            &mut tx,
            company_id,
            bank_account_id,
            &candidate_ids,
        )
        .await?;
        let (kept, duplicates) = dedupe_rows(&target.rows, &existing);
        let new_txs: Vec<NewBankTransaction> = kept
            .iter()
            .flat_map(|row| transactions_of_row(row, company_id, bank_account_id))
            .collect();
        if new_txs.is_empty() {
            reports.push(CurrencyReport {
                currency: target.currency.clone(),
                bank_account_id,
                import_id: None,
                inserted: 0,
                duplicates,
            });
            continue;
        }
        let period_from = kept.iter().map(|r| r.date).min().unwrap_or(NaiveDate::MIN);
        let period_to = kept.iter().map(|r| r.date).max().unwrap_or(NaiveDate::MIN);
        // Le solde de la dernière ligne (date la plus tardive ; à date égale, la
        // dernière du fichier) qui en porte un.
        let closing_balance = kept
            .iter()
            .filter(|r| r.balance.is_some())
            .max_by_key(|r| (r.date, r.line))
            .and_then(|r| r.balance);
        let inserted = new_txs.len();
        let transaction_count = i32::try_from(inserted)
            .map_err(|_| AppError::Internal("import PayPal : trop de transactions".into()))?;
        let new_import = NewBankImport {
            company_id,
            bank_account_id,
            filename: form.filename.clone(),
            file_hash: target.file_hash.clone(),
            source_format: BankImportSourceFormat::Csv,
            statement_id: None,
            period_from,
            period_to,
            opening_balance: None,
            closing_balance,
            transaction_count,
            imported_by_user_id: current_user.user_id,
        };
        let header =
            match bank_imports::create_with_transactions(&mut tx, new_import, new_txs).await {
                Ok((header, _)) => header,
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(import_create_error(e));
                }
            };
        let mut details_extra = serde_json::Map::new();
        details_extra.insert("provider".into(), serde_json::json!(PROVIDER_PAYPAL));
        details_extra.insert("provider_account".into(), serde_json::json!(form.account));
        details_extra.insert("currency".into(), serde_json::json!(target.currency));
        details_extra.insert("duplicates_skipped".into(), serde_json::json!(duplicates));
        insert_canonical_audit_log(
            &mut tx,
            ImportAuditAction::Created,
            current_user.user_id,
            current_user.api_key_id,
            header.id,
            &header.filename,
            header.transaction_count,
            "CSV",
            vec!["paypal"],
            details_extra,
            None,
        )
        .await?;
        reports.push(CurrencyReport {
            currency: target.currency.clone(),
            bank_account_id,
            import_id: Some(header.id),
            inserted,
            duplicates,
        });
    }

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "paypal_import.created",
            "bank_account",
            audit_entity_id,
            Some(serde_json::json!({
                "provider": PROVIDER_PAYPAL,
                "provider_account": form.account,
                "filename": form.filename,
                "currencies": reports,
                "skipped": skipped,
            })),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    Ok(reports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    const FIXTURE: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv");

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn fixture_rows() -> Vec<PaypalRow> {
        parse_paypal_csv(FIXTURE).expect("fixture lisible").rows
    }

    fn row(id: &str, kind: &str, currency: &str, gross: &str, fee: &str) -> PaypalRow {
        let mut r = fixture_rows().remove(0);
        r.transaction_id = id.into();
        r.kind = kind.into();
        r.currency = currency.into();
        r.gross = dec(gross);
        r.fee = dec(fee);
        r.net = r.gross + r.fee;
        r.reference_txn_id = None;
        r.balance_impact = if r.gross.is_sign_negative() {
            BalanceImpact::Debit
        } else {
            BalanceImpact::Credit
        };
        r
    }

    #[test]
    fn la_fixture_se_ventile_par_devise_et_par_motif() {
        let v = ventilate(&fixture_rows());
        assert_eq!(
            v.by_currency.keys().cloned().collect::<Vec<_>>(),
            ["CHF", "EUR", "USD"]
        );
        let ids = |c: &str| {
            v.by_currency[c]
                .iter()
                .map(|r| r.transaction_id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids("EUR"),
            [
                "0TEST00000000001A",
                "0TEST00000000003C",
                "0TEST00000000004D",
                "0TEST0000000000AK",
                "0TEST0000000000BL"
            ]
        );
        assert_eq!(ids("USD"), ["0TEST00000000002B", "0TEST00000000005E"]);
        assert_eq!(ids("CHF"), ["0TEST00000000006F", "0TEST00000000007G"]);
        assert_eq!(
            v.skipped.not_completed,
            [
                NotCompletedLine {
                    transaction_id: "0TEST0000000000CM".into(),
                    kind: "Website Payment".into(),
                    status: "Pending".into()
                },
                NotCompletedLine {
                    transaction_id: "0TEST0000000000DN".into(),
                    kind: "General Payment".into(),
                    status: "Denied".into()
                }
            ]
        );
        assert_eq!(v.skipped.memo, 1);
        assert_eq!(v.skipped.neutral_pairs, 1);
    }

    #[test]
    fn la_somme_des_transactions_d_une_devise_est_celle_de_ses_nets() {
        let v = ventilate(&fixture_rows());
        for (currency, rows) in &v.by_currency {
            let txs: Decimal = rows
                .iter()
                .flat_map(|r| transactions_of_row(r, 1, 1))
                .map(|t| t.amount)
                .sum();
            let nets: Decimal = rows.iter().map(|r| r.net).sum();
            assert_eq!(txs, nets, "devise {currency}");
        }
        // 9 transactions EUR (dont des frais positifs), 3 USD, 3 CHF.
        let count = |c: &str| {
            v.by_currency[c]
                .iter()
                .flat_map(|r| transactions_of_row(r, 1, 1))
                .count()
        };
        assert_eq!((count("EUR"), count("USD"), count("CHF")), (9, 3, 3));
    }

    #[test]
    fn brut_et_frais_chacun_de_son_signe() {
        let rows = fixture_rows();
        let refund = rows
            .iter()
            .find(|r| r.transaction_id == "0TEST0000000000BL")
            .unwrap();
        let txs = transactions_of_row(refund, 7, 9);
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].amount, dec("-30.00"));
        assert_eq!(txs[0].transaction_id.as_deref(), Some("0TEST0000000000BL"));
        assert_eq!(txs[0].reference.as_deref(), Some("TXN 0TEST0000000000BL"));
        assert_eq!(txs[0].end_to_end_id.as_deref(), Some("0TEST0000000000AK"));
        assert_eq!(txs[0].counterparty_name.as_deref(), Some("Daniel Fictif"));
        // Débit : l'adresse du destinataire.
        assert_eq!(
            txs[0].counterparty_email.as_deref(),
            Some("daniel@example.invalid")
        );
        assert_eq!(txs[0].provider_type.as_deref(), Some("Payment Refund"));
        assert_eq!(
            txs[0].details,
            "Payment Refund · Remboursement complet\nà la demande du client"
        );
        assert_eq!(
            txs[1].amount,
            dec("0.96"),
            "frais positifs d'un remboursement"
        );
        assert_eq!(
            txs[1].transaction_id.as_deref(),
            Some("0TEST0000000000BL:FEE")
        );
        assert_eq!(txs[1].details, "Frais PayPal · Payment Refund");
        assert!(
            txs.iter()
                .all(|t| t.bank_account_id == 9 && t.company_id == 7)
        );

        let sale = rows
            .iter()
            .find(|r| r.transaction_id == "0TEST00000000001A")
            .unwrap();
        let txs = transactions_of_row(sale, 1, 1);
        // Crédit : l'adresse de l'émetteur.
        assert_eq!(
            txs[0].counterparty_email.as_deref(),
            Some("alice@example.invalid")
        );
        assert_eq!(txs[0].details, "Website Payment · Commande 1001");
        assert_eq!(txs[1].amount, dec("-3.20"));
    }

    #[test]
    fn une_ligne_sans_frais_donne_une_transaction() {
        let r = row(
            "0TESTSANSFRAIS",
            "General Withdrawal",
            "CHF",
            "-10.00",
            "0.00",
        );
        assert_eq!(transactions_of_row(&r, 1, 1).len(), 1);
        assert_eq!(row_transaction_ids(&r), ["0TESTSANSFRAIS"]);
    }

    #[test]
    fn une_retenue_sans_sa_levee_reste_importee() {
        let mut hold = row("0TESTHOLD", "Payment Review Hold", "EUR", "-5.00", "0.00");
        hold.reference_txn_id = Some("0TESTREF".into());
        let v = ventilate(std::slice::from_ref(&hold));
        assert_eq!(v.skipped.neutral_pairs, 0);
        assert_eq!(v.by_currency["EUR"].len(), 1);
    }

    #[test]
    fn une_paire_se_forme_seulement_aux_conditions_ecrites() {
        let pair = |kind_b: &str, currency_b: &str, gross_b: &str, fee_b: &str| {
            let mut a = row("0TESTA", "Payment Review Hold", "EUR", "-5.00", "0.00");
            a.reference_txn_id = Some("0TESTREF".into());
            let mut b = row("0TESTB", kind_b, currency_b, gross_b, fee_b);
            b.reference_txn_id = Some("0TESTREF".into());
            ventilate(&[a, b]).skipped.neutral_pairs
        };
        assert_eq!(pair("Payment Review Release", "EUR", "5.00", "0.00"), 1);
        assert_eq!(
            pair("Website Payment", "EUR", "5.00", "0.00"),
            0,
            "type hors liste"
        );
        assert_eq!(
            pair("Payment Review Release", "USD", "5.00", "0.00"),
            0,
            "autre devise"
        );
        assert_eq!(
            pair("Payment Review Release", "EUR", "4.00", "0.00"),
            0,
            "brut non opposé"
        );
        assert_eq!(
            pair("Payment Review Release", "EUR", "5.10", "-0.10"),
            0,
            "frais non nuls"
        );
        // Référence vide : pas de paire.
        let a = row("0TESTA", "Payment Review Hold", "EUR", "-5.00", "0.00");
        let b = row("0TESTB", "Payment Review Release", "EUR", "5.00", "0.00");
        assert_eq!(ventilate(&[a, b]).skipped.neutral_pairs, 0);
    }

    #[test]
    fn le_statut_se_compare_sans_casse() {
        let mut r = row("0TESTC", "Website Payment", "EUR", "5.00", "0.00");
        r.status = "COMPLETED".into();
        assert!(ventilate(&[r]).skipped.not_completed.is_empty());
    }

    #[test]
    fn doublon_dans_le_fichier_et_en_base() {
        let a = row("0TESTDUP", "Website Payment", "EUR", "10.00", "-0.50");
        let b = a.clone();
        let c = row("0TESTNEW", "Website Payment", "EUR", "20.00", "-1.00");
        let d = row("0TESTOLD", "Website Payment", "EUR", "30.00", "0.00");
        let rows = vec![a, b, c, d];
        let existing: HashSet<String> = ["0TESTOLD".to_string()].into_iter().collect();
        let (kept, duplicates) = dedupe_rows(&rows, &existing);
        let ids: Vec<&str> = kept.iter().map(|r| r.transaction_id.as_str()).collect();
        assert_eq!(ids, ["0TESTDUP", "0TESTNEW"]);
        // La seconde 0TESTDUP (brut + frais) et 0TESTOLD (brut seul).
        assert_eq!(duplicates, 3);
        // Les frais seuls déjà en base suffisent à écarter la ligne.
        let existing: HashSet<String> = ["0TESTNEW:FEE".to_string()].into_iter().collect();
        let (kept, duplicates) = dedupe_rows(&rows[2..3], &existing);
        assert!(kept.is_empty());
        assert_eq!(duplicates, 2);
    }

    #[test]
    fn libelles_et_contreparties_bornes() {
        let mut r = row("0TESTLONG", "Website Payment", "EUR", "10.00", "0.00");
        r.note = Some("é".repeat(600));
        r.name = Some("n".repeat(300));
        let t = &transactions_of_row(&r, 1, 1)[0];
        assert_eq!(t.details.chars().count(), DETAILS_MAX_CHARS);
        assert_eq!(
            t.counterparty_name.as_ref().unwrap().chars().count(),
            NAME_MAX_CHARS
        );
    }
}
