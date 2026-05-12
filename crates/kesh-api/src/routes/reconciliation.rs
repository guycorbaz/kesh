//! Routes API réconciliation bancaire — Story 8-4 (FR44).
//!
//! - `GET /api/v1/reconciliation/proposals?bankAccountId={id}&limit={n}` —
//!   liste les propositions de matching pour les transactions `pending`
//!   du compte. Read-only, **pas de mutex**.
//! - `POST /api/v1/reconciliation/accept` — accepte un batch de
//!   propositions sous mutex. Body : `{ bankAccountId, proposals:
//!   [{ bankTransactionId, invoiceId }] }`. Partial success via
//!   savepoints MariaDB. Émet dual audit (`reconciliation.accepted`
//!   + `invoice.paid`).
//! - `POST /api/v1/reconciliation/reject` — marque les transactions
//!   comme manuellement revues (`auto_match_rejected_at = NOW()`)
//!   sous mutex. Body : `{ bankAccountId, bankTransactionIds }`.
//!
//! **Sub-router** : monté sous `comptable_routes` (lib.rs:90 pattern
//! 8-1b — RBAC `Comptable` requis).

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, FromRequest, Query, Request, State};
use axum::response::{IntoResponse, Response};
use chrono::{Duration, NaiveDate};
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::bank_transaction::{BankTransaction, BankTransactionStatus};
use kesh_db::entities::invoice::Invoice;
use kesh_db::errors::DbError;
use kesh_db::repositories::{
    accounts as accounts_repo, audit_log, bank_accounts, contacts as contacts_repo, fiscal_years,
    journal_entries, reconciliation as reconciliation_repo,
};
use kesh_reconciliation::{
    MatchScore, ReconciliationError, SplitDetail, build_journal_entry_for_counterparty,
    build_split_journal_entry, propose_matches, validate_split_balance, with_account_lock,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Limite de résultats par défaut pour `GET /proposals` (cf. L24).
const DEFAULT_PROPOSALS_LIMIT: i64 = 100;

/// Cap maximum de la query param `limit` (M9 Pass 1 code review —
/// défense anti-DoS contre `?limit=999999`).
const MAX_PROPOSALS_LIMIT: i64 = 500;

/// Fenêtre temporelle ± `WINDOW_DAYS` autour de `tx.booking_date` pour
/// le filtre `find_unpaid_invoices_for_window` (§candidate-window).
const WINDOW_DAYS: i64 = 30;

/// Tolérance amount ± `AMOUNT_TOLERANCE` CHF (5 centimes) au repo
/// — réduit le candidate set sans accepter le mismatch (le helper
/// `propose_matches` reste binaire 0/1 sur amount, cf. L17/L21).
const AMOUNT_TOLERANCE_HUNDREDTHS: i64 = 5;

/// Timeout `GET_LOCK` MariaDB pour les flows accept/reject (5s par
/// défaut, cf. spec §mutex-account + L36).
const LOCK_TIMEOUT_SECS: u32 = 5;

// ============================================================
// Response shapes (camelCase JSON pour le frontend)
// ============================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetProposalsResponse {
    pub proposals: Vec<ReconciliationProposal>,
    /// Indique si la query SQL a renvoyé `limit + 1` lignes (H6 Pass 1
    /// code review). Permet au frontend v0.2 d'afficher un bouton
    /// « Charger plus » sans probe count séparé. v0.1 : structure
    /// présente mais pas de UI dédiée.
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationProposal {
    pub bank_transaction_id: i64,
    pub transaction: TransactionSummary,
    pub candidates: Vec<ReconciliationCandidate>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSummary {
    pub booking_date: chrono::NaiveDate,
    /// P-M7 Pass 1 code review : exposer `value_date` (Option<NaiveDate>)
    /// pour pré-remplir le champ datepicker du `ManualMatchModal` avec la
    /// date de valeur (si présente) plutôt que le booking_date. Camelcase
    /// JSON : `valueDate`.
    pub value_date: Option<chrono::NaiveDate>,
    pub amount: String,
    pub currency: String,
    pub counterparty_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationCandidate {
    pub invoice_id: i64,
    pub invoice_number: Option<String>,
    pub invoice_amount: String,
    pub invoice_date: chrono::NaiveDate,
    pub score: MatchScore,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptResponse {
    pub accepted: Vec<AcceptedProposal>,
    pub failed: Vec<FailedProposal>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedProposal {
    pub bank_transaction_id: i64,
    pub invoice_id: i64,
    pub journal_entry_id: i64,
    pub score: MatchScore,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedProposal {
    pub bank_transaction_id: i64,
    pub error_code: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectResponse {
    pub rejected: Vec<RejectedProposal>,
    pub failed: Vec<FailedProposal>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectedProposal {
    pub bank_transaction_id: i64,
    pub rejected_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================
// Request shapes
// ============================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetProposalsQuery {
    pub bank_account_id: i64,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptBody {
    pub bank_account_id: i64,
    pub proposals: Vec<AcceptProposalInput>,
}

/// Story 8-5a-bis Q2 breaking — enum tagged Serde sur le champ `type`.
///
/// - `type: "invoice"` (8-4 héritée) : matching tx ↔ facture validée.
/// - `type: "split"` (8-5a-bis) : équivalent batch de `/split` standalone
///   (éclate tx en N+1 lignes JE). Path-dep `bank_account.journal_account_id`
///   résolu serveur-side inside `accept_one_split` (H1 Pass 4).
/// - `type: "manual"` (réservé v0.2 — Manual standalone via `/manual`).
/// - `type: "rule"` (réservé 8-5b).
///
/// Pas de `derive(Copy)` (F11'' Pass 3) car `Vec<SplitProposalLine>` +
/// `Decimal` ne sont pas `Copy`. Les call-sites internes utilisent
/// `clone()` quand nécessaire.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AcceptProposalInput {
    #[serde(rename = "invoice", rename_all = "camelCase")]
    Invoice {
        bank_transaction_id: i64,
        invoice_id: i64,
    },
    #[serde(rename = "split", rename_all = "camelCase")]
    Split {
        bank_transaction_id: i64,
        splits: Vec<SplitProposalLine>,
        /// M6''' Pass 3 Opus — `value_date` optional cohérent avec `/split`
        /// standalone. Si absent → handler default = 3 couches
        /// `body.value_date.or(tx.value_date).unwrap_or(tx.booking_date)`.
        #[serde(default)]
        value_date: Option<NaiveDate>,
    },
}

impl AcceptProposalInput {
    /// Helper d'extraction du `bank_transaction_id` indépendamment du
    /// variant — utilisé par le pré-flight `post_accept` (validation
    /// surface + batch ownership) avant le dispatch dans `accept_one`.
    fn bank_transaction_id(&self) -> i64 {
        match self {
            AcceptProposalInput::Invoice {
                bank_transaction_id,
                ..
            }
            | AcceptProposalInput::Split {
                bank_transaction_id,
                ..
            } => *bank_transaction_id,
        }
    }
}

/// Story 8-5a-bis — ligne de split dans le body `/accept type='split'`
/// (équivalent du `SplitLineInput` côté `/split` standalone).
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SplitProposalLine {
    pub counterparty_account_id: i64,
    pub amount: Decimal,
    pub description: String,
}

/// Extracteur custom pour `AcceptBody` — convertit les rejets serde en
/// `AppError::Validation` (400 `VALIDATION_ERROR`) au lieu du 422 Axum
/// natif.
///
/// **F9 Pass 1** : sans cet extracteur, un body absent du champ `type`
/// retourne 422 (Axum default sur `serde(tag)` désérialisation échouée).
/// AC #100 exige 400 explicite. Pattern repris de
/// `routes::bank_accounts::PatchJournalLinkBodyExtractor`.
pub struct AcceptBodyExtractor(pub AcceptBody);

impl<S> FromRequest<S> for AcceptBodyExtractor
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<AcceptBody>::from_request(req, state).await {
            Ok(Json(body)) => Ok(Self(body)),
            Err(rej) => {
                let message = match &rej {
                    JsonRejection::JsonDataError(_) | JsonRejection::JsonSyntaxError(_) => {
                        "corps JSON malformé ou champ `type` requis manquant — valeurs acceptées v0.1 : [\"invoice\", \"split\"]".to_string()
                    }
                    JsonRejection::MissingJsonContentType(_) => {
                        "Content-Type attendu : application/json".to_string()
                    }
                    _ => {
                        tracing::warn!(
                            rejection = %rej,
                            "AcceptBodyExtractor: unhandled JsonRejection variant"
                        );
                        "requête invalide (corps non-parsable)".to_string()
                    }
                };
                Err(AppError::Validation(message).into_response())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectBody {
    pub bank_account_id: i64,
    pub bank_transaction_ids: Vec<i64>,
}

// ============================================================
// GET /proposals
// ============================================================

/// Handler `GET /api/v1/reconciliation/proposals?bankAccountId={id}&limit={n}`.
/// Read-only — pas de mutex acquis (cf. spec MP5-3 Pass 5).
///
/// Architecture 4-pass (cf. T5.2 step 3) :
/// 1. Load all pending transactions (1 query).
/// 2. Pour chaque tx : load candidates (1 query par tx = N queries).
/// 3. Collect distinct contact_ids → batch-load contacts (1 query).
/// 4. Score per-tx in-memory.
pub async fn get_proposals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<GetProposalsQuery>,
) -> Result<Json<GetProposalsResponse>, AppError> {
    if query.bank_account_id <= 0 {
        return Err(AppError::Validation(
            "bankAccountId doit être strictement positif".into(),
        ));
    }
    // M9 Pass 1 — cap limit à `MAX_PROPOSALS_LIMIT` pour éviter DoS via
    // `?limit=999999`.
    let limit = query
        .limit
        .filter(|&l| l > 0)
        .map(|l| l.min(MAX_PROPOSALS_LIMIT))
        .unwrap_or(DEFAULT_PROPOSALS_LIMIT);

    // Pré-flight HP3-4 — bank_account ownership check.
    bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        query.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;

    // Pass 1 : load all pending transactions (1 query).
    // H6 Pass 1 — fetch `limit + 1` pour détecter la présence de plus
    // de résultats (pagination indicator `has_more`). On truncate à
    // `limit` avant le scoring pour ne pas exposer la sentinelle.
    let mut transactions = reconciliation_repo::find_pending_transactions_for_account(
        &state.pool,
        current_user.company_id,
        query.bank_account_id,
        limit + 1,
    )
    .await?;
    let has_more = transactions.len() as i64 > limit;
    if has_more {
        transactions.truncate(limit as usize);
    }

    // Pass 2 : per tx, load candidates + accumulate distinct contact_ids.
    let mut tx_candidates: Vec<(BankTransaction, Vec<Invoice>)> =
        Vec::with_capacity(transactions.len());
    let mut distinct_contact_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let amount_tolerance = Decimal::new(AMOUNT_TOLERANCE_HUNDREDTHS, 2);
    for tx in transactions {
        // A6-2 Pass 6 — currency tx-side guard (mono-CHF v0.1, cf. L38).
        // Skip transactions non-CHF (parser CSV peut techniquement insérer
        // EUR/USD via custom profile pre-Story 11 multi-currency).
        if tx.currency != "CHF" {
            tx_candidates.push((tx, Vec::new()));
            continue;
        }
        // MP3-1 Pass 3 — sign filter (tx débit → invoices toujours positives).
        if tx.amount <= Decimal::ZERO {
            tx_candidates.push((tx, Vec::new()));
            continue;
        }
        let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
            &state.pool,
            current_user.company_id,
            tx.booking_date,
            tx.amount,
            WINDOW_DAYS,
            amount_tolerance,
        )
        .await?;
        for inv in &candidates {
            // M1 Pass 1 — guard `contact_id > 0` : l'entité Invoice
            // expose `contact_id: i64` (pas `Option<i64>`) avec sentinel
            // 0 = pas de contact lié. Insérer 0 dans le set provoquerait
            // un SELECT inutile sur contact_id=0 (jamais existant).
            if inv.contact_id > 0 {
                distinct_contact_ids.insert(inv.contact_id);
            }
        }
        tx_candidates.push((tx, candidates));
    }

    // Pass 3 : batch-load contacts (1 query).
    let ids_vec: Vec<i64> = distinct_contact_ids.into_iter().collect();
    let contacts_map =
        reconciliation_repo::find_contacts_by_ids(&state.pool, current_user.company_id, &ids_vec)
            .await?;

    // Pass 4 : score per-tx + build response.
    let proposals: Vec<ReconciliationProposal> = tx_candidates
        .into_iter()
        .map(|(tx, candidate_invoices)| {
            let candidates_with_contacts: Vec<(Invoice, Option<kesh_db::entities::Contact>)> =
                candidate_invoices
                    .iter()
                    .map(|inv| (inv.clone(), contacts_map.get(&inv.contact_id).cloned()))
                    .collect();
            let match_proposals = propose_matches(&tx, &candidates_with_contacts);
            let candidates: Vec<ReconciliationCandidate> = match_proposals
                .into_iter()
                .filter_map(|mp| {
                    let inv = candidate_invoices.iter().find(|i| i.id == mp.invoice_id)?;
                    Some(ReconciliationCandidate {
                        invoice_id: inv.id,
                        invoice_number: inv.invoice_number.clone(),
                        invoice_amount: inv.total_amount.normalize().to_string(),
                        invoice_date: inv.date,
                        score: mp.score,
                    })
                })
                .collect();
            ReconciliationProposal {
                bank_transaction_id: tx.id,
                transaction: TransactionSummary {
                    booking_date: tx.booking_date,
                    value_date: tx.value_date,
                    amount: tx.amount.normalize().to_string(),
                    currency: tx.currency.clone(),
                    counterparty_name: tx.counterparty_name.clone(),
                },
                candidates,
            }
        })
        .collect();

    Ok(Json(GetProposalsResponse {
        proposals,
        has_more,
    }))
}

// ============================================================
// POST /accept
// ============================================================

/// Handler `POST /api/v1/reconciliation/accept`. Acquiert UN seul lock
/// pour tout le batch (H5 Pass 1 patch), itère per-proposal avec
/// savepoints MariaDB pour partial success.
pub async fn post_accept(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    AcceptBodyExtractor(body): AcceptBodyExtractor,
) -> Result<Json<AcceptResponse>, AppError> {
    // Step 0 — validation body (Q2 breaking : enum tagged, dispatch sur variant).
    if body.proposals.is_empty() {
        return Err(AppError::Validation("proposals vide".into()));
    }
    if body.bank_account_id <= 0 {
        return Err(AppError::Validation(
            "bankAccountId doit être strictement positif".into(),
        ));
    }
    let mut seen_tx_ids = std::collections::HashSet::new();
    for p in &body.proposals {
        let bt_id = p.bank_transaction_id();
        if !seen_tx_ids.insert(bt_id) {
            return Err(AppError::Validation(format!(
                "bankTransactionId dupliqué dans le batch : {bt_id}"
            )));
        }
        if bt_id <= 0 {
            return Err(AppError::Validation(
                "bankTransactionId doit être strictement positif".into(),
            ));
        }
        match p {
            AcceptProposalInput::Invoice { invoice_id, .. } => {
                if *invoice_id <= 0 {
                    return Err(AppError::Validation(
                        "invoiceId doit être strictement positif (type='invoice')".into(),
                    ));
                }
            }
            AcceptProposalInput::Split { splits, .. } => {
                // Story 8-5a-bis — pré-validation surface des splits inline
                // (cohérent §validation-handler-side-split steps 1, 1bis, 2).
                if splits.len() < MIN_SPLIT_LINES {
                    return Err(AppError::Validation(format!(
                        "splits doit contenir au moins {MIN_SPLIT_LINES} lignes — utilisez /accept type='invoice' ou /manual"
                    )));
                }
                if splits.len() > MAX_SPLIT_LINES {
                    return Err(AppError::Validation(format!(
                        "splits ne peut pas dépasser {MAX_SPLIT_LINES} lignes"
                    )));
                }
                for (idx, s) in splits.iter().enumerate() {
                    if s.description.chars().count() > MAX_MANUAL_DESCRIPTION_LEN {
                        return Err(AppError::Validation(format!(
                            "splits[{idx}].description trop longue (max {MAX_MANUAL_DESCRIPTION_LEN} caractères)"
                        )));
                    }
                    if s.amount <= Decimal::ZERO {
                        return Err(AppError::Validation(format!(
                            "splits[{idx}].amount doit être strictement positif (> 0)"
                        )));
                    }
                    // P1 (ECH-01) — voir commentaire post_split step 2.
                    if s.amount.scale() > 2 {
                        return Err(AppError::Validation(format!(
                            "splits[{idx}].amount précision invalide (max 2 décimales pour CHF)"
                        )));
                    }
                    if s.counterparty_account_id <= 0 {
                        return Err(AppError::Validation(format!(
                            "splits[{idx}].counterpartyAccountId doit être strictement positif"
                        )));
                    }
                }
            }
        }
    }

    // Step 0bis — bank_account ownership pré-flight (HP3-4).
    bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;

    // Step 0ter — bank_transactions ownership pré-flight batch (MP3-3).
    let tx_ids: Vec<i64> = body
        .proposals
        .iter()
        .map(|p| p.bank_transaction_id())
        .collect();
    let tx_map = reconciliation_repo::find_pending_by_ids(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
        &tx_ids,
    )
    .await?;
    if tx_map.len() != tx_ids.len() {
        let missing: Vec<i64> = tx_ids
            .iter()
            .copied()
            .filter(|id| !tx_map.contains_key(id))
            .collect();
        return Err(AppError::Validation(format!(
            "bankTransactions [{:?}] n'appartiennent pas au bankAccountId={}",
            missing, body.bank_account_id
        )));
    }

    // Step 1 — Acquire UN seul lock pour tout le batch (H5).
    let mut tx_outer = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let bank_account_id = body.bank_account_id;
    let proposals = body.proposals.clone();
    let user_id = current_user.user_id;
    let company_id = current_user.company_id;

    // C2 Pass 1 — `tx_map` (snapshot pré-flight 0ter) n'est plus passé
    // dans le lock : `accept_one` recharge la BankTransaction inside
    // lock pour fermer la fenêtre TOCTOU entre le pré-flight (hors-lock)
    // et l'UPDATE step 8. Le pré-flight 0ter reste utile pour valider
    // le batch ownership avant d'acquérir le lock (fail-fast 400).
    drop(tx_map);
    let lock_result = with_account_lock(
        &mut tx_outer,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            accept_batch(tx_inner, company_id, bank_account_id, user_id, &proposals).await
        },
    )
    .await;

    match lock_result {
        Ok(response) => {
            tx_outer
                .commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(Json(response))
        }
        Err(ReconciliationError::AccountLocked {
            bank_account_id,
            timeout_secs,
        }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationAccountLocked {
                bank_account_id,
                timeout_secs,
            })
        }
        Err(ReconciliationError::LockReleaseFailed {
            bank_account_id, ..
        }) => {
            // HP4-1 Pass 4 + HP5-1 Pass 5 : drop tx_outer pour rollback +
            // retour au pool. Lock advisory libéré à fin de session.
            drop(tx_outer);
            Err(AppError::ReconciliationLockReleaseFailed { bank_account_id })
        }
        Err(ReconciliationError::Database(e)) => {
            drop(tx_outer);
            Err(AppError::Database(DbError::Sqlx(e)))
        }
        // Story 8-5a-base T2.3 — variant `Db(DbError)` ajouté à
        // `ReconciliationError`. La closure `accept_batch` n'émet PAS
        // ce variant en pratique (elle continue d'utiliser
        // `.map_err(|e| match e { Sqlx → Database, other → Database+Protocol })`
        // — F2 Pass 7 Sonnet directive). Cette branche est unreachable
        // en pratique mais requise pour l'exhaustivité du compilateur.
        Err(ReconciliationError::Db(db_err)) => {
            drop(tx_outer);
            Err(AppError::Database(db_err))
        }
        // Idem : `accept_batch` n'émet pas `FiscalYearClosed`. Branche
        // exhaustive uniquement.
        Err(ReconciliationError::FiscalYearClosed { entry_date }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationFiscalYearClosed { entry_date })
        }
        // Story 8-5a-bis — `SplitImbalance` peut être émis par
        // `accept_one_split` (cf. T3 H1 Pass 4). Mapping → 400 cohérent
        // avec `/split` standalone.
        Err(ReconciliationError::SplitImbalance {
            expected,
            actual,
            difference,
        }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationSplitImbalance {
                expected,
                actual,
                difference,
            })
        }
    }
}

/// Helper interne — itère les proposals dans des savepoints MariaDB
/// pour partial success.
///
/// **C2 Pass 1 code review** : signature simplifiée — ne reçoit plus
/// `_pool` ni `tx_map`. `accept_one` recharge la `BankTransaction`
/// inside lock via `find_pending_by_id_for_account` (TOCTOU fix),
/// et reçoit `bank_account_id` pour scope le SELECT.
async fn accept_batch(
    tx_outer: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    proposals: &[AcceptProposalInput],
) -> Result<AcceptResponse, ReconciliationError> {
    let mut accepted: Vec<AcceptedProposal> = Vec::new();
    let mut failed: Vec<FailedProposal> = Vec::new();
    let batch_size = proposals.len() as i64;

    for proposal in proposals {
        let savepoint = format!("sp_{}", proposal.bank_transaction_id());
        sqlx::query(&format!("SAVEPOINT {savepoint}"))
            .execute(&mut **tx_outer)
            .await?;

        match accept_one(
            tx_outer,
            company_id,
            bank_account_id,
            user_id,
            proposal,
            batch_size,
        )
        .await
        {
            Ok(entry) => {
                sqlx::query(&format!("RELEASE SAVEPOINT {savepoint}"))
                    .execute(&mut **tx_outer)
                    .await?;
                accepted.push(entry);
            }
            Err(failure) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {savepoint}"))
                    .execute(&mut **tx_outer)
                    .await?;
                failed.push(failure);
            }
        }
    }

    Ok(AcceptResponse { accepted, failed })
}

/// Helper interne — dispatch sur le variant `AcceptProposalInput` Q2.
///
/// Story 8-5a-bis F3''' Pass 3 Opus + H1 Pass 4 : pattern match top-level
/// pour briser les 29 accès directs `proposal.bank_transaction_id` /
/// `.invoice_id` (qui cassent avec l'enum tagged).
async fn accept_one(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    proposal: &AcceptProposalInput,
    batch_size: i64,
) -> Result<AcceptedProposal, FailedProposal> {
    match proposal {
        AcceptProposalInput::Invoice {
            bank_transaction_id,
            invoice_id,
        } => {
            accept_one_invoice(
                tx,
                company_id,
                bank_account_id,
                user_id,
                *bank_transaction_id,
                *invoice_id,
                batch_size,
            )
            .await
        }
        AcceptProposalInput::Split {
            bank_transaction_id,
            splits,
            value_date,
        } => {
            accept_one_split(
                tx,
                company_id,
                bank_account_id,
                user_id,
                *bank_transaction_id,
                splits,
                *value_date,
            )
            .await
        }
    }
}

/// Helper interne — traite UNE proposal `type='invoice'` dans son savepoint.
///
/// **C2 Pass 1 code review (TOCTOU fix)** : la BankTransaction est
/// rechargée INSIDE le lock via `find_pending_by_id_for_account`. Le
/// snapshot `tx_map` du pré-flight 0ter (hors-lock) est ignoré ici
/// pour fermer la fenêtre TOCTOU. `bank_account_id` est nécessaire
/// pour scope le SELECT (l'invariant batch « tous les tx du même
/// bank_account_id » a été validé au pré-flight 0ter).
async fn accept_one_invoice(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    bank_transaction_id: i64,
    invoice_id: i64,
    batch_size: i64,
) -> Result<AcceptedProposal, FailedProposal> {
    // Step 2 — recharger BankTransaction INSIDE le lock (C2 Pass 1
    // TOCTOU fix). Le snapshot `tx_map` pré-lock peut être caduc si un
    // autre flow a updated la transaction entre le pré-flight 0ter et
    // l'acquisition du lock.
    let bank_transaction = match reconciliation_repo::find_pending_by_id_for_account(
        &mut **tx,
        company_id,
        bank_account_id,
        bank_transaction_id,
    )
    .await
    {
        Ok(Some(t)) => t,
        Ok(None) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "BANK_TRANSACTION_NOT_FOUND".to_string(),
                details: None,
            });
        }
        Err(e) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
    };

    // Step 4 — status pending (post-rechargement inside lock).
    if bank_transaction.status != BankTransactionStatus::Pending {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
            details: None,
        });
    }

    // Step 5 — load Invoice via reconciliation_repo helper.
    let invoice = match reconciliation_repo::find_invoice_by_id_for_company(
        &mut **tx, company_id, invoice_id,
    )
    .await
    {
        Ok(Some(inv)) => inv,
        Ok(None) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "INVOICE_NOT_FOUND".to_string(),
                details: Some(serde_json::json!({ "invoiceId": invoice_id })),
            });
        }
        Err(e) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
    };

    // Step 5bis — load Contact si invoice.contact_id valid (MP6-1 Pass 6).
    // M1 Pass 1 — guard `contact_id > 0` : sentinel 0 = pas de contact lié.
    // M7 Pass 1 — utilisation directe de `contacts_repo::find_by_id_in_company`
    // (Executor générique, helper dupliqué `find_contact_by_id_for_company`
    // supprimé du `reconciliation_repo`).
    // Sur erreur DB, contact = None → score contact = 0.0 (graceful).
    let contact = if invoice.contact_id > 0 {
        contacts_repo::find_by_id_in_company(&mut **tx, invoice.contact_id, company_id)
            .await
            .unwrap_or(None)
    } else {
        None
    };

    // Step 6 — éligibilité invoice (HP3-3 Pass 3 + HP4-2 Pass 4 enum 4 reasons).
    if invoice.status != "validated" {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({ "reason": "invoice_not_validated" })),
        });
    }
    if invoice.paid_at.is_some() {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({ "reason": "invoice_already_paid" })),
        });
    }
    if invoice.journal_entry_id.is_none() {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({ "reason": "invoice_journal_entry_not_set" })),
        });
    }
    let paid_at_candidate = bank_transaction
        .value_date
        .unwrap_or(bank_transaction.booking_date);
    // Lower bound (existant) — paiement >= invoice.date - 1 day.
    if paid_at_candidate < invoice.date - Duration::days(1) {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({ "reason": "payment_date_before_invoice_date" })),
        });
    }
    // Upper bound (P3-M2 Pass 3) — paiement <= invoice.date + WINDOW_DAYS.
    // Aligne `accept_one_invoice` sur le candidate window ±30j de
    // `find_unpaid_invoices_for_window`. Empêche d'accepter une tx
    // récente contre une invoice très ancienne ou une tx future contre
    // invoice récente. L27 (cf. spec) : paiements tardifs > 30j sont
    // reportés Story 8-5 manual.
    if paid_at_candidate > invoice.date + Duration::days(WINDOW_DAYS) {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({
                "reason": "payment_date_outside_window",
                "window_days": WINDOW_DAYS,
            })),
        });
    }
    let journal_entry_id = invoice.journal_entry_id.unwrap();

    // Step 7 — re-calculer score serveur-side (M7 Pass 1).
    let candidates_for_score: Vec<(Invoice, Option<kesh_db::entities::Contact>)> =
        vec![(invoice.clone(), contact.clone())];
    let proposals_score = propose_matches(&bank_transaction, &candidates_for_score);
    let score = proposals_score
        .first()
        .map(|p| p.score)
        .unwrap_or(MatchScore {
            total: 0.0,
            amount_score: 0.0,
            reference_score: 0.0,
            contact_score: 0.0,
        });

    // Step 7bis — P3-H4 Pass 3 : min-score guard.
    // Refuse les couples (tx, invoice) avec score total <= 0.0 —
    // protection server-side contre les bypass UI où un Comptable
    // forgerait un POST `{bankTransactionId, invoiceId}` arbitraire.
    // Le score est calculé serveur-side, pas trusté du client. v0.1
    // pas d'override manuel ; story 8-5 ajoutera la création manuelle
    // FR45 avec override explicite.
    if score.total <= 0.0 {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_SCORE_TOO_LOW".to_string(),
            details: Some(serde_json::json!({
                "reason": "score_zero_no_match",
                "score": score,
            })),
        });
    }

    // Step 8 — UPDATE bank_transactions + invoices inline.
    // M2 Pass 1 — `NaiveDate::and_hms_opt` est deprecated ; utiliser
    // `and_time(NaiveTime::from_hms_opt(0,0,0))` pour produire le
    // `NaiveDateTime` minuit (l'unwrap est safe : 00:00:00 est constant).
    let paid_at_dt = paid_at_candidate
        .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is constant valid"));
    let invoice_version_pre = invoice.version;

    // C1 Pass 1 — UPDATE avec guard `status='pending'` (defense-in-depth
    // contre une race entre step 4 et step 8 : le check status au step 4
    // est lu inside lock mais l'UPDATE reste séparé) + check
    // `rows_affected() == 1` pour détecter le no-op silencieux.
    //
    // P3-H1 Pass 3 — symétrie optimistic lock avec UPDATE invoices :
    // ajoute `AND version = ?` pour défense-in-depth contre futures
    // mutations `bank_transactions` hors flow réconciliation (Story 8-5
    // manual matching, retroactive parser updates). `bank_tx_version_pre`
    // est la version courante DB au moment du recharge inside lock par
    // `find_pending_by_id_for_account` (C2 Pass 1 TOCTOU fix).
    let bank_tx_version_pre = bank_transaction.version;
    let bank_tx_update = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = ?, status = 'reconciled', updated_at = NOW(3), version = version + 1 \
         WHERE id = ? AND company_id = ? AND status = 'pending' AND version = ?",
    )
    .bind(journal_entry_id)
    .bind(bank_transaction_id)
    .bind(company_id)
    .bind(bank_tx_version_pre)
    .execute(&mut **tx)
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;

    if bank_tx_update.rows_affected() != 1 {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
            details: Some(serde_json::json!({ "reason": "race_during_update" })),
        });
    }

    let invoice_update = sqlx::query(
        "UPDATE invoices \
         SET paid_at = ?, version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'validated'",
    )
    .bind(paid_at_dt)
    .bind(invoice_id)
    .bind(company_id)
    .bind(invoice_version_pre)
    .execute(&mut **tx)
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;

    if invoice_update.rows_affected() != 1 {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({ "reason": "race_during_update" })),
        });
    }

    // Step 9 — audit log reconciliation.accepted.
    let details_accepted = serde_json::json!({
        "bank_transaction_id": bank_transaction_id,
        "invoice_id": invoice_id,
        "score": score,
        "batch_size": batch_size,
        "journal_entry_id": journal_entry_id,
    });
    let entry_accepted = audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry {
            user_id,
            action: "reconciliation.accepted".to_string(),
            entity_type: "bank_transaction".to_string(),
            entity_id: bank_transaction_id,
            details_json: Some(details_accepted),
        },
    )
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;

    // Step 10 — dual audit log invoice.paid (HP3-3 Pass 3 + MP4-4 Pass 4).
    let details_paid = serde_json::json!({
        "paid_at": paid_at_dt.and_utc(),
        "paid_by_user_id": user_id,
        "paid_via": "reconciliation",
        "reconciliation_audit_id": entry_accepted.id,
        "before": { "paid_at": null, "version": invoice_version_pre },
        "after": { "paid_at": paid_at_dt.and_utc(), "version": invoice_version_pre + 1 },
    });
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry {
            user_id,
            action: "invoice.paid".to_string(),
            entity_type: "invoice".to_string(),
            entity_id: invoice_id,
            details_json: Some(details_paid),
        },
    )
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;

    Ok(AcceptedProposal {
        bank_transaction_id,
        invoice_id,
        journal_entry_id,
        score,
    })
}

/// Helper interne Story 8-5a-bis — traite UNE proposal `type='split'`
/// dans son savepoint. Pattern per-proposal `FailedProposal` (cohérent
/// pattern 8-4 `accept_one_invoice`).
///
/// **H1 Pass 4 — Résolution `journal_account_id` inside lock** :
/// le lookup `bank_accounts::find_by_id_for_company` est fait inside la
/// closure (SELECT non-mutant, OK dans le lock advisory). Si NULL →
/// `FailedProposal { error_code: "BANK_ACCOUNT_NOT_CONFIGURED" }`
/// per-proposal (PAS un `AppError` global qui casserait le batch).
///
/// **Validations répliquées (M2 Pass 4)** : les validations pré-flight
/// surface (1bis, 2, 6bis, 7) sont garanties handler-side (`post_accept`
/// step 0) mais répliquées defense-in-depth ici en `FailedProposal`.
#[allow(clippy::too_many_arguments)]
async fn accept_one_split(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    bank_transaction_id: i64,
    splits: &[SplitProposalLine],
    value_date: Option<NaiveDate>,
) -> Result<AcceptedProposal, FailedProposal> {
    // Step a — re-validation defense-in-depth (1bis, 2).
    if splits.len() < MIN_SPLIT_LINES || splits.len() > MAX_SPLIT_LINES {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "VALIDATION_ERROR".to_string(),
            details: Some(serde_json::json!({ "reason": "splits_count_out_of_range" })),
        });
    }
    for s in splits {
        if s.description.chars().count() > MAX_MANUAL_DESCRIPTION_LEN {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "VALIDATION_ERROR".to_string(),
                details: Some(serde_json::json!({ "reason": "split_description_too_long" })),
            });
        }
        if s.amount <= Decimal::ZERO {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "VALIDATION_ERROR".to_string(),
                details: Some(serde_json::json!({ "reason": "split_amount_not_positive" })),
            });
        }
        // P1 (ECH-01) defense-in-depth — surface check existe déjà dans
        // `post_accept` step 0, on réplique ici pour éviter 500 DATABASE_ERROR
        // si bypass surface (clients API directs).
        if s.amount.scale() > 2 {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "VALIDATION_ERROR".to_string(),
                details: Some(serde_json::json!({ "reason": "split_amount_scale_too_high" })),
            });
        }
    }

    // Step b — bank_account.journal_account_id lookup inside lock (H1 Pass 4).
    // SELECT inline (les helpers `bank_accounts::find_by_id_for_company` et
    // `accounts::find_by_id_in_company` prennent un `&MySqlPool` pas un
    // Executor générique — kesh-db extension hors scope 8-5a-bis).
    let journal_account_row: Option<(Option<i64>,)> = match sqlx::query_as(
        "SELECT journal_account_id FROM bank_accounts WHERE company_id = ? AND id = ? LIMIT 1",
    )
    .bind(company_id)
    .bind(bank_account_id)
    .fetch_optional(&mut **tx)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
    };
    let bank_ledger_account_id = match journal_account_row {
        Some((Some(id),)) => id,
        Some((None,)) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "BANK_ACCOUNT_NOT_CONFIGURED".to_string(),
                details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
            });
        }
        None => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "BANK_ACCOUNT_NOT_FOUND".to_string(),
                details: None,
            });
        }
    };

    // Step c — vérifier que le ledger banque est actif.
    let ledger_active: Option<(bool,)> =
        match sqlx::query_as("SELECT active FROM accounts WHERE id = ? AND company_id = ? LIMIT 1")
            .bind(bank_ledger_account_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "DATABASE_ERROR".to_string(),
                    details: Some(serde_json::json!({ "message": e.to_string() })),
                });
            }
        };
    match ledger_active {
        Some((true,)) => {}
        _ => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "BANK_ACCOUNT_NOT_CONFIGURED".to_string(),
                details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
            });
        }
    }

    // P2 (ECH-02) defense-in-depth — counterparty != bank_ledger inside lock.
    for s in splits {
        if s.counterparty_account_id == bank_ledger_account_id {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "VALIDATION_ERROR".to_string(),
                details: Some(serde_json::json!({ "reason": "counterparty_equals_bank_ledger" })),
            });
        }
    }

    // Step d — batch validation des counterparty accounts (itération séquentielle).
    let mut missing: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
    for s in splits {
        let row: Option<(bool,)> = match sqlx::query_as(
            "SELECT active FROM accounts WHERE id = ? AND company_id = ? LIMIT 1",
        )
        .bind(s.counterparty_account_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "DATABASE_ERROR".to_string(),
                    details: Some(serde_json::json!({ "message": e.to_string() })),
                });
            }
        };
        match row {
            Some((true,)) => {}
            _ => {
                missing.insert(s.counterparty_account_id);
            }
        }
    }
    if !missing.is_empty() {
        let ids: Vec<i64> = missing.into_iter().collect();
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "ACCOUNT_NOT_FOUND".to_string(),
            details: Some(serde_json::json!({ "missingAccountIds": ids })),
        });
    }

    // Step e — re-fetch tx INSIDE lock (TOCTOU).
    let bt = match reconciliation_repo::find_strictly_pending_by_id_for_account(
        &mut **tx,
        company_id,
        bank_account_id,
        bank_transaction_id,
    )
    .await
    {
        Ok(Some(t)) => t,
        Ok(None) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "RECONCILIATION_TRANSACTION_NOT_PENDING".to_string(),
                details: None,
            });
        }
        Err(e) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
    };

    // Step f — defense-in-depth tx.amount != 0 (6bis).
    if bt.amount.is_zero() {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "VALIDATION_ERROR".to_string(),
            details: Some(serde_json::json!({ "reason": "zero_amount_transaction" })),
        });
    }

    // Step g — validate_split_balance.
    let amounts: Vec<Decimal> = splits.iter().map(|s| s.amount).collect();
    if let Err(imb) = validate_split_balance(bt.amount, &amounts) {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_SPLIT_IMBALANCE".to_string(),
            details: Some(serde_json::json!({
                "expected": imb.expected.to_string(),
                "actual": imb.actual.to_string(),
                "difference": imb.difference.to_string(),
            })),
        });
    }

    // Step h — résoudre entry_date 3 couches (F4''' Pass 3).
    let entry_date = value_date.or(bt.value_date).unwrap_or(bt.booking_date);

    // Step i — fiscal year.
    let fiscal_year = match fiscal_years::find_open_covering_date(tx, company_id, entry_date).await
    {
        Ok(Some(fy)) => fy,
        Ok(None) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "RECONCILIATION_FISCAL_YEAR_CLOSED".to_string(),
                details: Some(serde_json::json!({ "entryDate": entry_date.to_string() })),
            });
        }
        Err(e) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
    };

    // Step j — build_split_journal_entry + create_in_tx.
    let split_details: Vec<SplitDetail> = splits
        .iter()
        .map(|s| SplitDetail {
            account_id: s.counterparty_account_id,
            amount: s.amount,
            description: s.description.clone(),
        })
        .collect();
    let je_description = format!("Éclatement transaction agrégée ({} lignes)", splits.len());
    let new_je = build_split_journal_entry(
        &bt,
        bank_ledger_account_id,
        &split_details,
        je_description,
        entry_date,
    );
    let je = match journal_entries::create_in_tx(tx, fiscal_year.id, user_id, new_je).await {
        Ok(j) => j,
        Err(e) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
    };
    let journal_entry_id = je.entry.id;

    // Step k — UPDATE bank_transactions optimistic lock + reset auto_match_rejected_at.
    let bank_tx_version_pre = bt.version;
    let bank_tx_update = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = ?, status = 'reconciled', \
             auto_match_rejected_at = NULL, updated_at = NOW(3), \
             version = version + 1 \
         WHERE id = ? AND company_id = ? AND status = 'pending' \
           AND version = ?",
    )
    .bind(journal_entry_id)
    .bind(bank_transaction_id)
    .bind(company_id)
    .bind(bank_tx_version_pre)
    .execute(&mut **tx)
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;
    if bank_tx_update.rows_affected() != 1 {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
            details: Some(serde_json::json!({ "reason": "race_during_update" })),
        });
    }

    // Step l — audit log reconciliation.split_applied.
    let was_previously_rejected = bt.auto_match_rejected_at.is_some();
    // P5 (Pass 1) — normaliser scale 2 décimales cohérent /split standalone.
    // `Decimal::rescale(2)` force scale 2 dans la sortie.
    let splits_for_audit: Vec<serde_json::Value> = splits
        .iter()
        .map(|s| {
            let mut amount = s.amount;
            amount.rescale(2);
            serde_json::json!({
                "counterparty_account_id": s.counterparty_account_id,
                "amount": amount.to_string(),
                "description": s.description,
            })
        })
        .collect();
    let details = serde_json::json!({
        "bank_transaction_id": bank_transaction_id,
        "splits": splits_for_audit,
        "total_amount": bt.amount.abs().to_string(),
        "journal_entry_id": journal_entry_id,
        "value_date": entry_date.to_string(),
        "was_previously_rejected": was_previously_rejected,
    });
    if let Err(e) = audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry {
            user_id,
            action: "reconciliation.split_applied".to_string(),
            entity_type: "bank_transaction".to_string(),
            entity_id: bank_transaction_id,
            details_json: Some(details),
        },
    )
    .await
    {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": e.to_string() })),
        });
    }

    Ok(AcceptedProposal {
        bank_transaction_id,
        // Pour un split, pas d'invoice ciblée — sentinel 0 (la structure
        // est commune avec invoice flow).
        invoice_id: 0,
        journal_entry_id,
        score: MatchScore {
            total: 0.0,
            amount_score: 0.0,
            reference_score: 0.0,
            contact_score: 0.0,
        },
    })
}

// ============================================================
// POST /reject
// ============================================================

/// Handler `POST /api/v1/reconciliation/reject`. Marque les
/// transactions comme manuellement revues (`auto_match_rejected_at`),
/// sous mutex partagé avec accept (M2 Pass 1).
pub async fn post_reject(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<RejectBody>,
) -> Result<Json<RejectResponse>, AppError> {
    if body.bank_transaction_ids.is_empty() {
        return Err(AppError::Validation("bankTransactionIds vide".into()));
    }
    if body.bank_account_id <= 0 {
        return Err(AppError::Validation(
            "bankAccountId doit être strictement positif".into(),
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for &id in &body.bank_transaction_ids {
        if !seen.insert(id) {
            return Err(AppError::Validation(format!(
                "bankTransactionId dupliqué : {id}"
            )));
        }
        if id <= 0 {
            return Err(AppError::Validation(
                "bankTransactionIds doivent être strictement positifs".into(),
            ));
        }
    }

    // Pré-flight ownership.
    bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;

    let tx_map = reconciliation_repo::find_pending_by_ids(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
        &body.bank_transaction_ids,
    )
    .await?;
    if tx_map.len() != body.bank_transaction_ids.len() {
        let missing: Vec<i64> = body
            .bank_transaction_ids
            .iter()
            .copied()
            .filter(|id| !tx_map.contains_key(id))
            .collect();
        return Err(AppError::Validation(format!(
            "bankTransactions [{:?}] n'appartiennent pas au bankAccountId={}",
            missing, body.bank_account_id
        )));
    }

    let mut tx_outer = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let bank_account_id = body.bank_account_id;
    let user_id = current_user.user_id;
    let company_id = current_user.company_id;
    let ids = body.bank_transaction_ids.clone();

    let lock_result = with_account_lock(
        &mut tx_outer,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| reject_batch(tx_inner, company_id, user_id, &ids, &tx_map).await,
    )
    .await;

    match lock_result {
        Ok(response) => {
            tx_outer
                .commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(Json(response))
        }
        Err(ReconciliationError::AccountLocked {
            bank_account_id,
            timeout_secs,
        }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationAccountLocked {
                bank_account_id,
                timeout_secs,
            })
        }
        Err(ReconciliationError::LockReleaseFailed {
            bank_account_id, ..
        }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationLockReleaseFailed { bank_account_id })
        }
        Err(ReconciliationError::Database(e)) => {
            drop(tx_outer);
            Err(AppError::Database(DbError::Sqlx(e)))
        }
        // Story 8-5a-base T2.3 — branches exhaustives sur les nouveaux
        // variants `Db(DbError)` + `FiscalYearClosed { entry_date }`.
        // Unreachable en pratique pour `reject_batch` (qui n'émet PAS
        // ces variants — pattern `.map_err` manuel conservé) mais
        // requises pour l'exhaustivité du match.
        Err(ReconciliationError::Db(db_err)) => {
            drop(tx_outer);
            Err(AppError::Database(db_err))
        }
        Err(ReconciliationError::FiscalYearClosed { entry_date }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationFiscalYearClosed { entry_date })
        }
        // Story 8-5a-bis — branche exhaustive uniquement (reject_batch
        // n'émet pas SplitImbalance). Unreachable en pratique.
        Err(ReconciliationError::SplitImbalance {
            expected,
            actual,
            difference,
        }) => {
            drop(tx_outer);
            Err(AppError::ReconciliationSplitImbalance {
                expected,
                actual,
                difference,
            })
        }
    }
}

async fn reject_batch(
    tx_outer: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    user_id: i64,
    ids: &[i64],
    tx_map: &HashMap<i64, BankTransaction>,
) -> Result<RejectResponse, ReconciliationError> {
    let mut rejected: Vec<RejectedProposal> = Vec::new();
    let mut failed: Vec<FailedProposal> = Vec::new();

    for &id in ids {
        let bank_tx = match tx_map.get(&id) {
            Some(t) => t,
            None => {
                failed.push(FailedProposal {
                    bank_transaction_id: id,
                    error_code: "BANK_TRANSACTION_NOT_FOUND".to_string(),
                    details: None,
                });
                continue;
            }
        };
        if bank_tx.status != BankTransactionStatus::Pending {
            failed.push(FailedProposal {
                bank_transaction_id: id,
                error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
                details: None,
            });
            continue;
        }

        // MP4-5 Pass 4 — rows_affected check.
        let result = sqlx::query(
            "UPDATE bank_transactions \
             SET auto_match_rejected_at = NOW(3), updated_at = NOW(3), version = version + 1 \
             WHERE id = ? AND company_id = ? AND status = 'pending'",
        )
        .bind(id)
        .bind(company_id)
        .execute(&mut **tx_outer)
        .await?;

        if result.rows_affected() != 1 {
            failed.push(FailedProposal {
                bank_transaction_id: id,
                error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
                details: Some(serde_json::json!({ "reason": "race_during_update" })),
            });
            continue;
        }

        // M4 Pass 1 — récupérer `auto_match_rejected_at` depuis la DB
        // (NOW(3) appliqué côté serveur) au lieu de `chrono::Utc::now()`
        // côté app : évite le clock skew entre l'horloge applicative et
        // celle de MariaDB. SELECT séparé après UPDATE car MariaDB
        // RETURNING n'est pas systématiquement supporté sur le sub-set
        // de versions ciblé (10.5+ requis pour RETURNING avec UPDATE).
        //
        // P3-C1 Pass 3 — `auto_match_rejected_at` colonne `DATETIME(3)`
        // (sans TZ DB-side, cf. migration `20260507100001_reconciliation_8_4.sql:15`)
        // — sqlx-mysql décode en `NaiveDateTime`. Le type `DateTime<Utc>`
        // est réservé aux colonnes `TIMESTAMP`. La sérialisation JSON
        // applique `.and_utc()` car MariaDB stocke en UTC (convention
        // projet) — `.and_utc()` ré-attache juste le marqueur Utc à la
        // valeur naïve, pas de conversion timezone applicative.
        let rejected_at_naive: chrono::NaiveDateTime = sqlx::query_scalar(
            "SELECT auto_match_rejected_at FROM bank_transactions \
             WHERE id = ? AND company_id = ?",
        )
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut **tx_outer)
        .await?;

        rejected.push(RejectedProposal {
            bank_transaction_id: id,
            rejected_at: rejected_at_naive.and_utc(),
        });
    }

    // Audit log unique pour le batch (HP3-2 Pass 3 — entity_id = first id).
    if !rejected.is_empty() {
        let success_ids: Vec<i64> = rejected.iter().map(|r| r.bank_transaction_id).collect();
        let details = serde_json::json!({
            "bank_transaction_ids": success_ids,
            "count": success_ids.len(),
        });
        audit_log::insert_in_tx(
            tx_outer,
            NewAuditLogEntry {
                user_id,
                action: "reconciliation.rejected".to_string(),
                entity_type: "bank_transaction".to_string(),
                entity_id: rejected[0].bank_transaction_id,
                details_json: Some(details),
            },
        )
        .await
        // C3 Pass 1 — toute `DbError` non-Sqlx (e.g. `NotFound`,
        // `OptimisticLockConflict`, `UniqueConstraintViolation`) est
        // wrapped dans `sqlx::Error::Protocol` pour préserver la
        // sémantique `Database` côté handler (mappée 500). Le fake
        // `AccountLocked{0,0}` précédent était mappé 409 par erreur,
        // induisant un retry-after côté client sans cause valide.
        .map_err(|e| match e {
            DbError::Sqlx(sqlx_err) => ReconciliationError::Database(sqlx_err),
            other => ReconciliationError::Database(sqlx::Error::Protocol(format!(
                "audit_log insert_in_tx failed (non-Sqlx DbError): {other:?}"
            ))),
        })?;
    }

    Ok(RejectResponse { rejected, failed })
}

// ============================================================
// Story 8-5a-base — POST /reconciliation/manual (FR45)
// ============================================================

/// Cap maximum pour le field `description` du body POST /manual
/// (8-5a-base). Distinct de `MAX_DESCRIPTION_LEN = 500` de
/// `routes/journal_entries.rs` — business rule modal manual
/// (libellé court UX, F4 Pass 4 Sonnet).
const MAX_MANUAL_DESCRIPTION_LEN: usize = 200;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualMatchBody {
    pub bank_account_id: i64,
    pub bank_transaction_id: i64,
    pub counterparty_account_id: i64,
    pub description: Option<String>,
    pub value_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualMatchResponse {
    pub bank_transaction_id: i64,
    pub journal_entry_id: i64,
}

/// Handler `POST /api/v1/reconciliation/manual` (Story 8-5a-base FR45).
///
/// Crée à la volée une `journal_entry` à 2 lignes (banque + contrepartie)
/// pour réconcilier une `bank_transaction` `pending` sans facture
/// pré-existante. Le compte ledger banque est résolu **serveur-side**
/// via `bank_account.journal_account_id` (foundation 8-5a-zero) — pas
/// de body field `bankLedgerAccountId`.
///
/// Lève L19/L20 héritées 8-4 (matching journal_entries non-invoice +
/// création écriture sans facture). L23 partiellement levée (manual
/// reset `auto_match_rejected_at = NULL`).
///
/// Sub-router `comptable_routes` → RBAC Comptable+ enforcé.
///
/// **Ordre de validation** (cf. spec §validation-handler-side) :
/// 1. `bankAccountId` ownership multi-tenant → 404 BANK_ACCOUNT_NOT_FOUND.
/// 2. `bank_account.journal_account_id` configuré → 412 BANK_ACCOUNT_NOT_CONFIGURED.
/// 3. `counterpartyAccountId` ownership + active → 404 ACCOUNT_NOT_FOUND.
/// 4. `find_strictly_pending_by_id_for_account` → 404 RECONCILIATION_TRANSACTION_NOT_PENDING.
///    4bis. `tx.amount != 0` → 400 VALIDATION_ERROR (zero_amount_transaction, F7''' Pass 3).
/// 5. (inside lock 5-9) : re-fetch tx (TOCTOU) + fiscal_year + create_in_tx
///    + UPDATE bank_transactions optimistic lock + audit log.
pub async fn post_manual(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<ManualMatchBody>,
) -> Result<Json<ManualMatchResponse>, AppError> {
    // Step 0 — validation body de surface.
    if body.bank_account_id <= 0 {
        return Err(AppError::Validation(
            "bankAccountId doit être strictement positif".into(),
        ));
    }
    if body.bank_transaction_id <= 0 {
        return Err(AppError::Validation(
            "bankTransactionId doit être strictement positif".into(),
        ));
    }
    if body.counterparty_account_id <= 0 {
        return Err(AppError::Validation(
            "counterpartyAccountId doit être strictement positif".into(),
        ));
    }
    if let Some(d) = &body.description {
        if d.chars().count() > MAX_MANUAL_DESCRIPTION_LEN {
            return Err(AppError::Validation(format!(
                "description trop longue (max {MAX_MANUAL_DESCRIPTION_LEN} caractères)"
            )));
        }
    }

    // Step 1 — bank_account ownership pré-flight (KF-002 multi-tenant).
    // F1''' Pass 3 Opus : `AppError::BankAccountNotFound` est unit-struct
    // (pas `{ bank_account_id }`) — code HTTP réel
    // `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` v0.1, dette héritée 8-1b.
    let bank_account = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;

    // Step 2 — bank_account.journal_account_id configuré (foundation 8-5a-zero).
    // → 412 BANK_ACCOUNT_NOT_CONFIGURED si NULL.
    let bank_ledger_account_id =
        bank_account
            .journal_account_id
            .ok_or(AppError::BankAccountNotConfigured {
                bank_account_id: body.bank_account_id,
            })?;

    // Step 2 bis — P-H5 Pass 1 code review : vérifier que le compte ledger
    // banque résolu est ACTIF. Race fenêtre : un Admin peut archiver le
    // compte 1020 entre le PATCH /bank-accounts/{id} (8-5a-zero) et
    // l'appel manual. Sans ce check, `journal_entries::create_in_tx`
    // step 7 retourne 400 INACTIVE_OR_INVALID_ACCOUNTS (générique). On
    // remonte 412 BANK_ACCOUNT_NOT_CONFIGURED qui pointe vers la racine
    // (le compte lié n'est plus exploitable, reconfigurer dans
    // /bank-accounts) — UX cohérente avec step 2.
    let bank_ledger_account = accounts_repo::find_by_id_in_company(
        &state.pool,
        bank_ledger_account_id,
        current_user.company_id,
    )
    .await?;
    match bank_ledger_account {
        Some(a) if a.active => {}
        _ => {
            return Err(AppError::BankAccountNotConfigured {
                bank_account_id: body.bank_account_id,
            });
        }
    }

    // Step 3 — counterpartyAccountId ownership + active check.
    // anti-énumération KF-002 : 404 même si cross-tenant ou archivé.
    let counterparty = accounts_repo::find_by_id_in_company(
        &state.pool,
        body.counterparty_account_id,
        current_user.company_id,
    )
    .await?;
    let counterparty = match counterparty {
        Some(a) if a.active => a,
        _ => {
            return Err(AppError::AccountNotFound {
                account_id: body.counterparty_account_id,
                missing_account_ids: None,
            });
        }
    };

    // Step 4 — find_strictly_pending : 404 si introuvable / cross-tenant /
    // cross-account / déjà reconciled (4 cas en un seul code).
    let bank_transaction = reconciliation_repo::find_strictly_pending_by_id_for_account(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
        body.bank_transaction_id,
    )
    .await?
    .ok_or(AppError::ReconciliationTransactionNotPending {
        bank_transaction_id: body.bank_transaction_id,
    })?;

    // Step 4bis — F7''' Pass 3 Opus : pré-validation `tx.amount != 0`.
    // Évite que `build_journal_entry_for_counterparty` produise 2 lignes
    // 0/0 sémantiquement vides (cf. L48). Marqueur "zero_amount_transaction"
    // dans `error.message` (F5'''' Pass 6 — shape réelle `AppError::Validation`).
    if bank_transaction.amount.is_zero() {
        return Err(AppError::Validation("zero_amount_transaction".to_string()));
    }

    // Capture les inputs handler-side avant de move dans la closure.
    // P-H3 Pass 1 code review : `resolved_value_date` calculé handler-side
    // était dead code (la closure recalcule indépendamment depuis ses
    // captures `body.value_date` + `tx.value_date` + `booking_date`).
    // Supprimé.
    let was_previously_rejected = bank_transaction.auto_match_rejected_at.is_some();
    let bank_transaction_amount = bank_transaction.amount;
    let description = body.description.clone().unwrap_or_default();

    // Step 5+ — tout le reste inside `with_account_lock` (atomicité
    // §validation-handler-side step 9 : steps 5-9 ne PAS sortir audit_log
    // de la closure — F2'''' Pass 6 Opus).
    let mut tx_outer = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let bank_account_id = body.bank_account_id;
    let bank_transaction_id = body.bank_transaction_id;
    let counterparty_account_id = body.counterparty_account_id;
    // P-H3 Pass 1 code review : suppression `counterparty_number` (dead
    // code spéculatif pour logging futur).
    let user_id = current_user.user_id;
    let company_id = current_user.company_id;
    // `counterparty` (Account complet) consommé jusque ici par les checks
    // step 3 (active=true). Drop explicite pour signaler la fin d'usage.
    drop(counterparty);

    let lock_result: Result<i64, ReconciliationError> = with_account_lock(
        &mut tx_outer,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            // Step 5 — re-fetch tx INSIDE le lock (TOCTOU defense
            // pattern 8-4 — un autre flow concurrent peut avoir mis à
            // jour la tx entre le pré-flight step 4 et l'acquisition
            // du lock).
            let tx = reconciliation_repo::find_strictly_pending_by_id_for_account(
                &mut **tx_inner,
                company_id,
                bank_account_id,
                bank_transaction_id,
            )
            .await?
            .ok_or_else(|| {
                // Race avec un autre flow : la tx a été reconciled
                // entre step 4 et step 5. On émet un DbError typé
                // wrappé en variant `Db` pour préserver la fidélité
                // jusqu'au mapping HTTP. `OptimisticLockConflict`
                // (mappé 409) reflète bien la sémantique de race.
                ReconciliationError::Db(DbError::OptimisticLockConflict)
            })?;
            let bank_tx_version_pre = tx.version;
            let booking_date = tx.booking_date;
            let entry_date = body.value_date.or(tx.value_date).unwrap_or(booking_date);

            // Step 6 — find_open_covering_date avec FOR UPDATE row lock
            // intra-tx (advisory lock orthogonal). Si None (NoFiscalYear
            // OR Closed unifiés v0.1, cf. L46), traduire en
            // `ReconciliationError::FiscalYearClosed { entry_date }` —
            // F3''' Pass 3 Opus : sans cette traduction, le `Ok(None)`
            // ne propagerait pas en erreur.
            let fiscal_year =
                fiscal_years::find_open_covering_date(tx_inner, company_id, entry_date)
                    .await
                    .map_err(ReconciliationError::Db)?
                    .ok_or(ReconciliationError::FiscalYearClosed { entry_date })?;

            // Step 7 — build_journal_entry_for_counterparty (helper
            // 8-5a-base T2) puis create_in_tx atomique.
            let new_je = build_journal_entry_for_counterparty(
                &tx,
                bank_ledger_account_id,
                counterparty_account_id,
                description.clone(),
                entry_date,
            );
            let je =
                journal_entries::create_in_tx(tx_inner, fiscal_year.id, user_id, new_je).await?;
            let journal_entry_id = je.entry.id;

            // Step 8 — UPDATE bank_transactions optimistic lock + status
            // guard + multi-tenant defense (F3'''' Pass 6 Opus complète
            // cohérent pattern 8-4 ligne 691).
            let update_result = sqlx::query(
                "UPDATE bank_transactions \
                 SET status = 'reconciled', matched_entry_id = ?, \
                     auto_match_rejected_at = NULL, updated_at = NOW(3), \
                     version = version + 1 \
                 WHERE id = ? AND company_id = ? AND status = 'pending' \
                   AND version = ?",
            )
            .bind(journal_entry_id)
            .bind(bank_transaction_id)
            .bind(company_id)
            .bind(bank_tx_version_pre)
            .execute(&mut **tx_inner)
            .await
            // P-M5 Pass 1 code review : cohérence avec le reste de la
            // closure qui wrap les erreurs sqlx via `Db(DbError::Sqlx(_))`
            // (variant `From<DbError>` pour `?`). Si step 8 est extrait en
            // helper kesh-db retournant `DbError`, le `?` continuera à
            // fonctionner. Le variant `Database(sqlx::Error)` reste pour
            // compat 8-4 mais n'est plus émis par 8-5a-base.
            .map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?;

            if update_result.rows_affected() != 1 {
                // Race version OR status `pending → reconciled` par
                // un autre flow concurrent. Mapper en
                // `OptimisticLockConflict` → 409.
                return Err(ReconciliationError::Db(DbError::OptimisticLockConflict));
            }

            // Step 9 — audit log `reconciliation.manual_matched` snake_case
            // top-level (Q4a action distincte cohérent F4'' Pass 3).
            let details = serde_json::json!({
                "bank_transaction_id": bank_transaction_id,
                "counterparty_account_id": counterparty_account_id,
                "journal_entry_id": journal_entry_id,
                "amount": bank_transaction_amount.to_string(),
                "description": description,
                "value_date": entry_date.to_string(),
                "was_previously_rejected": was_previously_rejected,
            });
            audit_log::insert_in_tx(
                tx_inner,
                NewAuditLogEntry {
                    user_id,
                    action: "reconciliation.manual_matched".to_string(),
                    entity_type: "bank_transaction".to_string(),
                    entity_id: bank_transaction_id,
                    details_json: Some(details),
                },
            )
            .await?;

            Ok(journal_entry_id)
        },
    )
    .await;

    // F1'''' Pass 6 Opus — match exhaustif sur tous les variants
    // `ReconciliationError`. Le compilateur Rust force la complétude.
    match lock_result {
        Ok(journal_entry_id) => {
            tx_outer
                .commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(Json(ManualMatchResponse {
                bank_transaction_id,
                journal_entry_id,
            }))
        }
        // P-H4 Pass 1 code review : remplacement `drop(tx_outer)` par
        // `rollback().await` explicite pour signaler l'intention. Le
        // rollback peut échouer (connexion DB perdue) — on ignore
        // l'erreur car l'erreur principale (AccountLocked, FiscalYearClosed,
        // Db, ...) est plus signifiante. Pattern idiomatique Rust + plus
        // explicite que le drop implicite.
        // Note résiduelle : les handlers 8-4 (`post_accept`/`post_reject`)
        // utilisent encore `drop(tx_outer)`. Cohérence reportée Story 11+
        // (out-of-scope 8-5a-base).
        Err(ReconciliationError::AccountLocked {
            bank_account_id,
            timeout_secs,
        }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationAccountLocked {
                bank_account_id,
                timeout_secs,
            })
        }
        Err(ReconciliationError::LockReleaseFailed {
            bank_account_id, ..
        }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationLockReleaseFailed { bank_account_id })
        }
        Err(ReconciliationError::FiscalYearClosed { entry_date }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationFiscalYearClosed { entry_date })
        }
        Err(ReconciliationError::Db(db_err)) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::Database(db_err))
        }
        Err(ReconciliationError::Database(e)) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::Database(DbError::Sqlx(e)))
        }
        // Story 8-5a-bis — branche exhaustive (post_manual ne fait pas
        // de split, unreachable en pratique).
        Err(ReconciliationError::SplitImbalance {
            expected,
            actual,
            difference,
        }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationSplitImbalance {
                expected,
                actual,
                difference,
            })
        }
    }
}

// ============================================================
// Story 8-5a-bis — POST /reconciliation/split (FR48)
// ============================================================

/// Bornes split (§split-flow Min/max splits) :
/// - `< 2` → 400 Validation (utiliser /manual pour 1 ligne).
/// - `> 50` → 400 Validation (cap raisonnable v0.1).
const MIN_SPLIT_LINES: usize = 2;
const MAX_SPLIT_LINES: usize = 50;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitLineInput {
    pub counterparty_account_id: i64,
    pub amount: Decimal,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitBody {
    pub bank_account_id: i64,
    pub bank_transaction_id: i64,
    pub splits: Vec<SplitLineInput>,
    pub value_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitResponse {
    pub bank_transaction_id: i64,
    pub journal_entry_id: i64,
}

/// Handler `POST /api/v1/reconciliation/split` (Story 8-5a-bis FR48).
///
/// Éclate une `bank_transaction` `pending` agrégée en N+1 lignes de
/// `journal_entry` (1 ligne banque agrégée + N lignes contreparties).
/// Le compte ledger banque est résolu **serveur-side** via
/// `bank_account.journal_account_id` (foundation 8-5a-zero) — pas de
/// body field `bankLedgerAccountId`.
///
/// Sub-router `comptable_routes` → RBAC Comptable+ enforcé.
///
/// **Ordre de validation** (cf. spec §validation-handler-side-split) :
/// step 1 `splits.len()` ∈ [2, 50] → 400 Validation ;
/// step 1bis `splits[i].description` ≤ 200 chars (M4''') → 400 Validation ;
/// step 2 `splits[i].amount > 0` strict (C1''') → 400 Validation ;
/// step 3 `bankAccountId` ownership multi-tenant → 404 BANK_ACCOUNT_NOT_FOUND ;
/// step 4 `bank_account.journal_account_id` configuré → 412 BANK_ACCOUNT_NOT_CONFIGURED ;
/// step 5 batch ownership/active des `counterpartyAccountId` → 404 ACCOUNT_NOT_FOUND ;
/// step 6 `find_strictly_pending_by_id_for_account` → 404 RECONCILIATION_TRANSACTION_NOT_PENDING ;
/// step 6bis `tx.amount != 0` (M2''') → 400 VALIDATION_ERROR ;
/// step 7 `validate_split_balance` → 400 RECONCILIATION_SPLIT_IMBALANCE ;
/// steps 8-13 inside lock : re-fetch tx + fiscal_year + create_in_tx + UPDATE optimistic + audit.
pub async fn post_split(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<SplitBody>,
) -> Result<Json<SplitResponse>, AppError> {
    // Step 0 — validation surface body.
    if body.bank_account_id <= 0 {
        return Err(AppError::Validation(
            "bankAccountId doit être strictement positif".into(),
        ));
    }
    if body.bank_transaction_id <= 0 {
        return Err(AppError::Validation(
            "bankTransactionId doit être strictement positif".into(),
        ));
    }

    // Step 1 — splits.len() ∈ [2, 50].
    if body.splits.len() < MIN_SPLIT_LINES {
        return Err(AppError::Validation(format!(
            "splits doit contenir au moins {MIN_SPLIT_LINES} lignes — utilisez /manual pour 1 ligne"
        )));
    }
    if body.splits.len() > MAX_SPLIT_LINES {
        return Err(AppError::Validation(format!(
            "splits ne peut pas dépasser {MAX_SPLIT_LINES} lignes"
        )));
    }

    // Step 1bis — longueur description ≤ 200 chars (M4''' Pass 3 Opus,
    // defense-in-depth backend — frontend cap aussi).
    for (idx, s) in body.splits.iter().enumerate() {
        if s.description.chars().count() > MAX_MANUAL_DESCRIPTION_LEN {
            return Err(AppError::Validation(format!(
                "splits[{idx}].description trop longue (max {MAX_MANUAL_DESCRIPTION_LEN} caractères)"
            )));
        }

        // Step 2 — splits[i].amount > 0 strict (C1''' Pass 3 Opus).
        // Rejet montants 0 et négatifs pour empêcher lignes JE 0/0 vides.
        if s.amount <= Decimal::ZERO {
            return Err(AppError::Validation(format!(
                "splits[{idx}].amount doit être strictement positif (> 0)"
            )));
        }
        // Code-review P1 (ECH-01 Pass 1 Sonnet) — `bank_transactions.amount`
        // est DECIMAL(18,2) (centimes CHF) ; les `journal_entry_lines.debit/credit`
        // sont DECIMAL(19,4). Sans cette validation, un split avec scale > 2
        // (ex. "3.33333") passerait `validate_split_balance` (qui compare la
        // valeur mathématique) mais échouerait à l'INSERT avec strict mode
        // MariaDB → HTTP 500 au lieu de 400. Cap à 2 décimales (centimes).
        if s.amount.scale() > 2 {
            return Err(AppError::Validation(format!(
                "splits[{idx}].amount précision invalide (max 2 décimales pour CHF)"
            )));
        }
        if s.counterparty_account_id <= 0 {
            return Err(AppError::Validation(format!(
                "splits[{idx}].counterpartyAccountId doit être strictement positif"
            )));
        }
    }

    // Step 3 — bank_account ownership pré-flight (KF-002 multi-tenant).
    let bank_account = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;

    // Step 4 — bank_account.journal_account_id configuré (foundation 8-5a-zero).
    let bank_ledger_account_id =
        bank_account
            .journal_account_id
            .ok_or(AppError::BankAccountNotConfigured {
                bank_account_id: body.bank_account_id,
            })?;

    // Step 4bis — ledger banque actif (P-H5 Pass 1 manual pattern réutilisé).
    let bank_ledger_account = accounts_repo::find_by_id_in_company(
        &state.pool,
        bank_ledger_account_id,
        current_user.company_id,
    )
    .await?;
    match bank_ledger_account {
        Some(a) if a.active => {}
        _ => {
            return Err(AppError::BankAccountNotConfigured {
                bank_account_id: body.bank_account_id,
            });
        }
    }

    // P2 (ECH-02 Pass 1 Sonnet) — interdire counterparty == bank_ledger_account_id.
    // Sinon le JE résultant aurait des lignes débit + crédit sur le même compte
    // (self-referential balance-sheet no-op) sans signification comptable. Le
    // frontend filtre client-side classes 5/6/7 (le bank ledger est classe 1/2),
    // donc en UX normal pas de collision possible — mais un client API direct
    // pourrait bypass. Defense-in-depth backend.
    for (idx, s) in body.splits.iter().enumerate() {
        if s.counterparty_account_id == bank_ledger_account_id {
            return Err(AppError::Validation(format!(
                "splits[{idx}].counterpartyAccountId ne peut pas être le compte ledger banque"
            )));
        }
    }

    // Step 5 — batch validation accounts via itération séquentielle (cap 50,
    // cf. §validation-handler-side-split step 5 + L3 Pass 1 source tree).
    // Collecte les IDs manquants triés et distincts pour body 404 batch.
    let mut missing: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
    for s in &body.splits {
        match accounts_repo::find_by_id_in_company(
            &state.pool,
            s.counterparty_account_id,
            current_user.company_id,
        )
        .await?
        {
            Some(a) if a.active => {}
            _ => {
                missing.insert(s.counterparty_account_id);
            }
        }
    }
    if !missing.is_empty() {
        let ids: Vec<i64> = missing.into_iter().collect();
        let first = ids[0];
        return Err(AppError::AccountNotFound {
            account_id: first,
            missing_account_ids: Some(ids),
        });
    }

    // Step 6 — find_strictly_pending.
    let bank_transaction = reconciliation_repo::find_strictly_pending_by_id_for_account(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
        body.bank_transaction_id,
    )
    .await?
    .ok_or(AppError::ReconciliationTransactionNotPending {
        bank_transaction_id: body.bank_transaction_id,
    })?;

    // Step 6bis — pré-validation `tx.amount != 0` (M2''' Pass 3 Opus).
    if bank_transaction.amount.is_zero() {
        return Err(AppError::Validation("zero_amount_transaction".to_string()));
    }

    // Step 7 — validate_split_balance Decimal exact (F7 Pass 1 .collect::<Vec<_>>()).
    let amounts: Vec<Decimal> = body.splits.iter().map(|s| s.amount).collect();
    if let Err(imbalance) = validate_split_balance(bank_transaction.amount, &amounts) {
        return Err(AppError::ReconciliationSplitImbalance {
            expected: imbalance.expected,
            actual: imbalance.actual,
            difference: imbalance.difference,
        });
    }

    // Capture inputs handler-side avant move dans closure.
    let was_previously_rejected = bank_transaction.auto_match_rejected_at.is_some();
    let bank_transaction_amount = bank_transaction.amount;
    let splits_for_lock: Vec<SplitDetail> = body
        .splits
        .iter()
        .map(|s| SplitDetail {
            account_id: s.counterparty_account_id,
            amount: s.amount,
            description: s.description.clone(),
        })
        .collect();
    // P5 (Pass 1 LOW merged BH-M4/ECH-07/AA-F3) — normaliser scale 2 décimales
    // pour cohérence avec total_amount + spec §audit-log-shape (`"5000.00"`).
    // `Decimal::rescale(2)` force la scale 2 dans le résultat (round_dp(2)
    // ne padd pas les zéros trailing si scale d'entrée < 2).
    let splits_for_audit: Vec<serde_json::Value> = body
        .splits
        .iter()
        .map(|s| {
            let mut amount = s.amount;
            amount.rescale(2);
            serde_json::json!({
                "counterparty_account_id": s.counterparty_account_id,
                "amount": amount.to_string(),
                "description": s.description,
            })
        })
        .collect();
    let je_description = format!(
        "Éclatement transaction agrégée ({} lignes)",
        body.splits.len()
    );

    // Steps 8-13 inside `with_account_lock`.
    let mut tx_outer = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let bank_account_id = body.bank_account_id;
    let bank_transaction_id = body.bank_transaction_id;
    let body_value_date = body.value_date;
    let user_id = current_user.user_id;
    let company_id = current_user.company_id;

    let lock_result: Result<i64, ReconciliationError> = with_account_lock(
        &mut tx_outer,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            // Step 8 — re-fetch tx INSIDE le lock (TOCTOU).
            let tx = reconciliation_repo::find_strictly_pending_by_id_for_account(
                &mut **tx_inner,
                company_id,
                bank_account_id,
                bank_transaction_id,
            )
            .await?
            .ok_or_else(|| ReconciliationError::Db(DbError::OptimisticLockConflict))?;
            let bank_tx_version_pre = tx.version;
            let booking_date = tx.booking_date;
            let entry_date = body_value_date.or(tx.value_date).unwrap_or(booking_date);

            // Step 9 — find_open_covering_date.
            let fiscal_year =
                fiscal_years::find_open_covering_date(tx_inner, company_id, entry_date)
                    .await
                    .map_err(ReconciliationError::Db)?
                    .ok_or(ReconciliationError::FiscalYearClosed { entry_date })?;

            // Step 10 — build_split_journal_entry (N+1 lignes).
            let new_je = build_split_journal_entry(
                &tx,
                bank_ledger_account_id,
                &splits_for_lock,
                je_description.clone(),
                entry_date,
            );

            // Step 11 — create_in_tx atomique.
            let je =
                journal_entries::create_in_tx(tx_inner, fiscal_year.id, user_id, new_je).await?;
            let journal_entry_id = je.entry.id;

            // Step 12 — UPDATE bank_transactions optimistic lock + status guard
            // + reset auto_match_rejected_at = NULL (M3 Pass 2).
            let update_result = sqlx::query(
                "UPDATE bank_transactions \
                 SET status = 'reconciled', matched_entry_id = ?, \
                     auto_match_rejected_at = NULL, updated_at = NOW(3), \
                     version = version + 1 \
                 WHERE id = ? AND company_id = ? AND status = 'pending' \
                   AND version = ?",
            )
            .bind(journal_entry_id)
            .bind(bank_transaction_id)
            .bind(company_id)
            .bind(bank_tx_version_pre)
            .execute(&mut **tx_inner)
            .await
            .map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?;

            if update_result.rows_affected() != 1 {
                return Err(ReconciliationError::Db(DbError::OptimisticLockConflict));
            }

            // Step 13 — audit log `reconciliation.split_applied` snake_case
            // top-level (cohérent F4'' Pass 3 + Q4a action distincte).
            let details = serde_json::json!({
                "bank_transaction_id": bank_transaction_id,
                "splits": splits_for_audit,
                "total_amount": bank_transaction_amount.abs().to_string(),
                "journal_entry_id": journal_entry_id,
                "value_date": entry_date.to_string(),
                "was_previously_rejected": was_previously_rejected,
            });
            audit_log::insert_in_tx(
                tx_inner,
                NewAuditLogEntry {
                    user_id,
                    action: "reconciliation.split_applied".to_string(),
                    entity_type: "bank_transaction".to_string(),
                    entity_id: bank_transaction_id,
                    details_json: Some(details),
                },
            )
            .await?;

            Ok(journal_entry_id)
        },
    )
    .await;

    match lock_result {
        Ok(journal_entry_id) => {
            tx_outer
                .commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(Json(SplitResponse {
                bank_transaction_id,
                journal_entry_id,
            }))
        }
        Err(ReconciliationError::AccountLocked {
            bank_account_id,
            timeout_secs,
        }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationAccountLocked {
                bank_account_id,
                timeout_secs,
            })
        }
        Err(ReconciliationError::LockReleaseFailed {
            bank_account_id, ..
        }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationLockReleaseFailed { bank_account_id })
        }
        Err(ReconciliationError::FiscalYearClosed { entry_date }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationFiscalYearClosed { entry_date })
        }
        Err(ReconciliationError::Db(db_err)) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::Database(db_err))
        }
        Err(ReconciliationError::Database(e)) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::Database(DbError::Sqlx(e)))
        }
        Err(ReconciliationError::SplitImbalance {
            expected,
            actual,
            difference,
        }) => {
            let _ = tx_outer.rollback().await;
            Err(AppError::ReconciliationSplitImbalance {
                expected,
                actual,
                difference,
            })
        }
    }
}
