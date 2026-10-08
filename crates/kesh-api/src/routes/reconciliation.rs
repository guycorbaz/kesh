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
use kesh_db::repositories::invoice_settlements::PaymentAgainstDue;
use kesh_db::repositories::reconciliation::UnpaidInvoiceCandidate;
use kesh_db::repositories::{
    accounts as accounts_repo, audit_log, bank_accounts, company_invoice_settings,
    contacts as contacts_repo, fiscal_years, invoice_settlements, journal_entries, projects,
    reconciliation as reconciliation_repo, reconciliation_rules,
};
use kesh_reconciliation::{
    MatchScore, ReconciliationError, SplitDetail, build_journal_entry_for_counterparty,
    build_split_journal_entry, propose_matches, rule_matches, validate_split_balance,
    with_account_lock,
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

/// Story 8-5b §api-response-shapes Pass 1 P-H6 — discriminator candidate
/// type. `Invoice` héritée 8-4 ; `Rule` nouveau 8-5b.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateType {
    Invoice,
    Rule,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationCandidate {
    /// Story 8-5b — discriminator. Les fields ci-dessous sont Some/None
    /// selon ce type.
    pub candidate_type: CandidateType,
    // ----- Invoice candidate fields -----
    pub invoice_id: Option<i64>,
    pub invoice_number: Option<String>,
    /// Montant à régler de la facture : son **reste dû** (#420, Story 25-4-c),
    /// le montant comparé à la transaction — le TTC tant que rien n'est réglé.
    pub invoice_amount: Option<String>,
    /// TTC de la facture, **seulement** si elle est déjà réglée en partie
    /// (reste dû ≠ TTC) — pour la mention « reste dû sur <TTC> » qui la fait
    /// reconnaître (Story 25-4-c, Q3). `None` sinon : rien à ajouter.
    pub invoice_total_ttc: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    // ----- Rule candidate fields (Story 8-5b) -----
    pub rule_id: Option<i64>,
    pub rule_label: Option<String>,
    /// Pass 1 code review LOW AA5 fix : match_type exposed for audit
    /// replay + frontend display (§api-response-shapes spec).
    pub rule_match_type: Option<String>,
    pub counterparty_account_id: Option<i64>,
    /// Display name `"{number} {name}"` du compte de contrepartie résolu
    /// via accounts_info (1 query batch, cf. §rule-application R5/Q5).
    pub counterparty_account_name: Option<String>,
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

/// Story 19-5 — mappe une [`DbError`] issue de la **validation du projet
/// analytique** (`validate_taggable_in_tx`, ou `create_in_tx` per-ligne pour le
/// split) en [`FailedProposal`] avec un `error_code` canonique, plutôt qu'un
/// `DATABASE_ERROR` opaque. Respecte le pattern batch (erreur per-proposition,
/// jamais d'`AppError` globale).
///
/// `project_id` (si connu du caller) est joint dans `details` (`{ "projectId":
/// <id> }`, AC11). Pour le split, le projet fautif exact n'est pas isolable →
/// `None`.
///
/// **Attention** : n'appeler ce mapper QUE sur un chemin où un `DbError::NotFound`
/// / `IllegalStateTransition("...projet...")` provient réellement de la
/// validation projet — sinon un `NotFound` non-projet (ex. fiscal_year absent)
/// serait mal étiqueté. Le chemin `create_in_tx` d'`accept_one_rule` (projet
/// déjà validé en amont) utilise donc le mapping générique, pas ce mapper.
/// Story 24-4c (#380) — le refus de verrou de période, sous la forme que le
/// § *Pattern batch* du `CLAUDE.md` impose : un **code canonique**, jamais un
/// repli générique.
///
/// ⛔ **Un seul constructeur pour les DEUX sites.** `project_error_to_failed_proposal`
/// et le mappage en ligne d'`accept_one_rule` en avaient chacun une copie ; deux
/// copies du même mappage divergent, et une seule des deux était testée.
///
/// ⚠️ `projectId` est **omis** quand il n'y a pas de projet, jamais publié à
/// `null` — c'est la convention de tous les autres codes du batch, et le chemin
/// `split` appelle précisément avec `None`.
fn period_locked_failed_proposal(
    bank_transaction_id: i64,
    project_id: Option<i64>,
    locked_through: chrono::NaiveDate,
    attempted: chrono::NaiveDate,
) -> FailedProposal {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "lockedThrough".to_string(),
        serde_json::json!(locked_through.to_string()),
    );
    obj.insert(
        "attempted".to_string(),
        serde_json::json!(attempted.to_string()),
    );
    if let Some(pid) = project_id {
        obj.insert("projectId".to_string(), serde_json::json!(pid));
    }
    FailedProposal {
        bank_transaction_id,
        error_code: "PERIOD_LOCKED".to_string(),
        details: Some(serde_json::Value::Object(obj)),
    }
}

/// Story 15-5b (AC3, AC4, #427) — le refus d'un compte de contrepartie **non
/// imputable** dans une acceptation par lot, sous la forme du § *Pattern batch*
/// du `CLAUDE.md` : `failed[]` avec `errorCode = "ACCOUNT_NOT_POSTABLE"` et
/// `details = { "rejected": [{ "accountId", "accountNumber" }] }`.
///
/// Le code et le détail sont ceux du HTTP 400 de la variante
/// [`DbError::AccountsNotPostable`] : le code est lu par
/// [`DbError::error_code`], le détail par
/// [`kesh_db::errors::NonPostableAccounts::details`] — seul constructeur du JSON
/// `rejected` (choix C29) ; la liste est triée et dédoublonnée.
///
/// **Précondition** : `accounts` non vide, et chaque compte est de la société
/// et actif (le manquant est rendu avant, en `ACCOUNT_NOT_FOUND`).
fn non_postable_failed_proposal(
    bank_transaction_id: i64,
    accounts: Vec<kesh_db::errors::NonPostableAccount>,
) -> FailedProposal {
    let list = kesh_db::errors::NonPostableAccounts::new(accounts);
    let details = list.details();
    FailedProposal {
        bank_transaction_id,
        error_code: DbError::AccountsNotPostable(list).error_code().to_string(),
        details: Some(details),
    }
}

fn project_error_to_failed_proposal(
    bank_transaction_id: i64,
    project_id: Option<i64>,
    err: DbError,
) -> FailedProposal {
    let details = |extra: Option<&str>| {
        let mut obj = serde_json::Map::new();
        if let Some(pid) = project_id {
            obj.insert("projectId".to_string(), serde_json::json!(pid));
        }
        if let Some(msg) = extra {
            obj.insert("message".to_string(), serde_json::json!(msg));
        }
        if obj.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(obj))
        }
    };
    match &err {
        DbError::IllegalStateTransition(msg) if msg.contains("projet") => FailedProposal {
            bank_transaction_id,
            error_code: "PROJECT_ARCHIVED".to_string(),
            details: details(Some(msg)),
        },
        DbError::NotFound => FailedProposal {
            bank_transaction_id,
            error_code: "PROJECT_NOT_FOUND".to_string(),
            details: details(None),
        },
        // ⛔ Story 24-4c (#380) — le verrou de période. SANS ce bras, un
        // rapprochement antidaté tomberait dans le `_` ci-dessous et serait
        // rapporté au client comme `DATABASE_ERROR` : une panne de base là où
        // il s'agit d'un refus MÉTIER parfaitement légitime.
        //
        // ⚠️ C'est le § *Pattern batch* du `CLAUDE.md` qui l'exige — « error_code :
        // constante canonique », jamais un repli générique — et il déclare les
        // trois `accept_one_*` inviolables sur ce point.
        DbError::PeriodLocked {
            locked_through,
            attempted,
        } => period_locked_failed_proposal(
            bank_transaction_id,
            project_id,
            *locked_through,
            *attempted,
        ),
        _ => FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": err.to_string() })),
        },
    }
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
    /// Story 8-5b FR47 — application d'une `reconciliation_rule` pour
    /// créer un journal_entry sans facture pré-existante. Le serveur
    /// résout serveur-side : (a) ledger banque via
    /// `bank_account.journal_account_id`, (b) re-validation match au
    /// step 7 anti-race, (c) optimistic version check `AND version = ?`
    /// au step 13 (Pass 1 code review HIGH EC1 fix — pas de `SELECT FOR
    /// UPDATE` sur bank_transactions ; le serialization repose
    /// exclusivement sur l'advisory lock `with_account_lock`).
    /// Pas de body `value_date` v0.1
    /// (Pass 3 R3 — l'utilisateur fait confiance à `tx.value_date`).
    #[serde(rename = "rule", rename_all = "camelCase")]
    Rule {
        bank_transaction_id: i64,
        rule_id: i64,
        counterparty_account_id: i64,
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
            }
            | AcceptProposalInput::Rule {
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
    /// Projet analytique de cette ligne de ventilation (Story 19-5) —
    /// optionnel. Validé per-ligne par `create_in_tx`.
    #[serde(default)]
    pub project_id: Option<i64>,
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
                        "corps JSON malformé ou champ `type` requis manquant — valeurs acceptées v0.1 : [\"invoice\", \"split\", \"rule\"]".to_string()
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

    // Pré-flight HP3-4 — bank_account ownership check + archived guard
    // (Story v014-1, FINDING-6 Pass 3 Opus — anti-énumération KF-002).
    let ba_check = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        query.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;
    if ba_check.archived {
        return Err(AppError::BankAccountNotFound);
    }

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
    let mut tx_candidates: Vec<(BankTransaction, Vec<UnpaidInvoiceCandidate>)> =
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
        for cand in &candidates {
            // M1 Pass 1 — guard `contact_id > 0` : l'entité Invoice
            // expose `contact_id: i64` (pas `Option<i64>`) avec sentinel
            // 0 = pas de contact lié. Insérer 0 dans le set provoquerait
            // un SELECT inutile sur contact_id=0 (jamais existant).
            if cand.invoice.contact_id > 0 {
                distinct_contact_ids.insert(cand.invoice.contact_id);
            }
        }
        tx_candidates.push((tx, candidates));
    }

    // Pass 3 : batch-load contacts (1 query).
    let ids_vec: Vec<i64> = distinct_contact_ids.into_iter().collect();
    let contacts_map =
        reconciliation_repo::find_contacts_by_ids(&state.pool, current_user.company_id, &ids_vec)
            .await?;

    // Pass 3.5 (Story 8-5b §rule-application) — pre-load active rules +
    // accounts_info pour appliquer rule fallback dans Pass 4 (closure sync).
    let active_rules =
        reconciliation_rules::find_active_for_company(&state.pool, current_user.company_id).await?;
    let accounts_info_rows: Vec<(i64, String, String, bool)> = sqlx::query_as(
        "SELECT id, number, name, postable FROM accounts WHERE company_id = ? AND active = TRUE",
    )
    .bind(current_user.company_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    // Story 15-5b (AC5, #427) — l'ensemble passé à `first_matching_rule` ne
    // retient que les comptes **actifs ET imputables** : une règle dont le
    // compte est devenu non imputable (scindé en sous-comptes, compte de
    // résultat ou de clôture) n'est plus proposée, et la règle suivante qui
    // correspond l'est à sa place. Son acceptation serait de toute façon
    // refusée (`accept_one_rule`, étape 5). `accounts_info`, qui sert à
    // l'affichage, garde tous les comptes actifs. Le paramètre de
    // `kesh_reconciliation` s'appelle encore `active_account_ids` (crate hors
    // périmètre) : ce qu'il reçoit ici est plus étroit que son nom.
    let active_account_ids: std::collections::HashSet<i64> = accounts_info_rows
        .iter()
        .filter(|(_, _, _, postable)| *postable)
        .map(|(id, _, _, _)| *id)
        .collect();
    let accounts_info: HashMap<i64, (String, String)> = accounts_info_rows
        .into_iter()
        .map(|(id, number, name, _)| (id, (number, name)))
        .collect();

    // Pass 4 : score per-tx + rule fallback + build response.
    // AC #114 (Pass 2 Q8) : un invoice candidate avec score ≥ 0.5 prime
    // sur les rules ; en dessous, rule + invoice coexistent (les 2
    // s'affichent). Aucun invoice candidate → rule en fallback seul.
    const INVOICE_OVERRIDE_THRESHOLD: f64 = 0.5;
    let proposals: Vec<ReconciliationProposal> = tx_candidates
        .into_iter()
        .map(|(tx, candidate_invoices)| {
            // #420 (25-4-c) : le triplet candidat porte le RESTE DÛ — le TTC
            // excluait le virement qui soldait une facture réglée en partie.
            // #476 (25-4-c3-b) : AU CENTIME — un virement de 10.01 sur un reste
            // de 10.0050 marquait 0 au score de montant.
            let candidates_with_contacts: Vec<(
                Invoice,
                Option<kesh_db::entities::Contact>,
                rust_decimal::Decimal,
            )> = candidate_invoices
                .iter()
                .map(|c| {
                    (
                        c.invoice.clone(),
                        contacts_map.get(&c.invoice.contact_id).cloned(),
                        invoice_settlements::amount_due_to_centime(c.amount_due),
                    )
                })
                .collect();
            let match_proposals = propose_matches(&tx, &candidates_with_contacts);
            let has_strong_invoice_match = match_proposals
                .iter()
                .any(|mp| mp.score.total >= INVOICE_OVERRIDE_THRESHOLD);

            let mut candidates: Vec<ReconciliationCandidate> = match_proposals
                .into_iter()
                .filter_map(|mp| {
                    let cand = candidate_invoices
                        .iter()
                        .find(|c| c.invoice.id == mp.invoice_id)?;
                    let inv = &cand.invoice;
                    Some(ReconciliationCandidate {
                        candidate_type: CandidateType::Invoice,
                        invoice_id: Some(inv.id),
                        invoice_number: inv.invoice_number.clone(),
                        // #420 (25-4-c) : montant affiché dans l'UI candidats
                        // = reste dû, la grandeur comparée à la tx (après le
                        // HT, #246, puis le TTC, qui affichait 1 000 pour un
                        // solde de 600). Le TTC ne suit que si les deux diffèrent.
                        // #476 (25-4-c3-b) : l'un et l'autre AU CENTIME, comparés
                        // comme affichés — « 10.005 » ne se lit sur aucun relevé.
                        invoice_amount: Some(
                            invoice_settlements::amount_due_to_centime(cand.amount_due)
                                .normalize()
                                .to_string(),
                        ),
                        invoice_total_ttc: {
                            let due = invoice_settlements::amount_due_to_centime(cand.amount_due);
                            let ttc = invoice_settlements::amount_due_to_centime(cand.total_ttc);
                            (due != ttc).then(|| ttc.normalize().to_string())
                        },
                        invoice_date: Some(inv.date),
                        rule_id: None,
                        rule_label: None,
                        rule_match_type: None,
                        counterparty_account_id: None,
                        counterparty_account_name: None,
                        score: mp.score,
                    })
                })
                .collect();

            // Story 8-5b rule fallback. Currency CHF only (Pass 3 R1
            // currency filter cohérent L38) — déjà filtré au Pass 2 via
            // `candidate_invoices.is_empty()` quand tx.currency != "CHF",
            // mais rules NE PAS hériter du sign filter 8-4 invoice (P-H7).
            if !has_strong_invoice_match
                && tx.currency == "CHF"
                && let Some(rule) = kesh_reconciliation::first_matching_rule(
                    &active_rules,
                    &tx,
                    &active_account_ids,
                )
            {
                let counterparty_display = accounts_info
                    .get(&rule.counterparty_account_id)
                    .map(|(num, name)| format!("{num} {name}"))
                    .unwrap_or_else(|| format!("compte #{}", rule.counterparty_account_id));
                candidates.push(ReconciliationCandidate {
                    candidate_type: CandidateType::Rule,
                    invoice_id: None,
                    invoice_number: None,
                    invoice_amount: None,
                    invoice_total_ttc: None,
                    invoice_date: None,
                    rule_id: Some(rule.id),
                    rule_label: Some(rule.label.clone()),
                    rule_match_type: Some(rule.match_type.as_str().to_string()),
                    counterparty_account_id: Some(rule.counterparty_account_id),
                    counterparty_account_name: Some(counterparty_display),
                    score: MatchScore {
                        total: 1.0,
                        amount_score: 0.0,
                        reference_score: 0.0,
                        contact_score: 0.0,
                    },
                });
            }

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
            AcceptProposalInput::Rule {
                rule_id,
                counterparty_account_id,
                ..
            } => {
                // Story 8-5b — pré-validation surface du variant Rule.
                if *rule_id <= 0 {
                    return Err(AppError::Validation(
                        "ruleId doit être strictement positif (type='rule')".into(),
                    ));
                }
                if *counterparty_account_id <= 0 {
                    return Err(AppError::Validation(
                        "counterpartyAccountId doit être strictement positif (type='rule')".into(),
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

    // Step 0bis — bank_account ownership pré-flight (HP3-4) + archived guard
    // (Story v014-1, FINDING-6 Pass 3 Opus — anti-énumération KF-002).
    let ba_check = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;
    if ba_check.archived {
        return Err(AppError::BankAccountNotFound);
    }

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

    // C2 Pass 1 — `tx_map` (snapshot pré-flight 0ter) n'est plus passé
    // dans le lock : `accept_one` recharge la BankTransaction inside
    // lock pour fermer la fenêtre TOCTOU entre le pré-flight (hors-lock)
    // et l'UPDATE step 8. Le pré-flight 0ter reste utile pour valider
    // le batch ownership avant d'acquérir le lock (fail-fast 400).
    drop(tx_map);

    // Step 1 — UN seul lock pour tout le batch (H5), dans une tentative que
    // `retry_with` rejoue en entier sur interblocage (Story 25-4-c2, #480).
    //
    // ⛔ **Rejeu au plus dehors** — transaction neuve, verrou de compte repris —,
    // patron de `post_cancel_reconciliation`. Deux causes, deux formes :
    // - un 1213 qui remonte **directement** par un `?` (`SAVEPOINT`, `RELEASE`)
    //   arrive en `AppError::Database(DbError::Sqlx(_))`, que `is_app_deadlock`
    //   reconnaît ;
    // - un 1213 levé **dans** une proposition y est absorbé en `FailedProposal` ;
    //   InnoDB a pourtant annulé toute la transaction, et c'est le
    //   `ROLLBACK TO SAVEPOINT` suivant qui échoue (1305) — `accept_batch` le
    //   remonte en `ReconciliationError::TransactionAborted`, rendu ici en
    //   `AppError::ReconciliationTransactionAborted`.
    // ⚠️ Prédicat LOCAL à cette route : `is_app_deadlock` reste 1213 seul, un
    // 1305 ailleurs dans le crate n'a pas ce sens. C'est pourquoi cette route
    // garde `retry_with` au lieu de l'enveloppe `retry_app_on_deadlock` — seule
    // exception, tenue par le registre (`RETRY_WITH_AUTORISE`).
    use kesh_db::retry::{DEFAULT_MAX_DEADLOCK_ATTEMPTS, retry_with};
    let company_id = current_user.company_id;
    let user_id = current_user.user_id;
    // Story 17-2a (DC5 cat ii) — attribution PAT propagée dans les helpers/closures.
    let actor_api_key_id = current_user.api_key_id;
    let bank_account_id = body.bank_account_id;
    retry_with(
        "reconciliation::accept",
        DEFAULT_MAX_DEADLOCK_ATTEMPTS,
        |err: &AppError| {
            crate::retry::is_app_deadlock(err)
                || matches!(err, AppError::ReconciliationTransactionAborted)
        },
        || {
            let pool = state.pool.clone();
            let proposals = body.proposals.clone();
            async move {
                accept_once(
                    &pool,
                    company_id,
                    bank_account_id,
                    user_id,
                    actor_api_key_id,
                    proposals,
                )
                .await
            }
        },
    )
    .await
    .map(Json)
}

/// Une tentative de `POST /accept` : transaction neuve, verrou du compte
/// bancaire, lot, commit (Story 25-4-c2 — extraite de `post_accept` pour que
/// `retry_with` puisse la rejouer en entier).
async fn accept_once(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
    proposals: Vec<AcceptProposalInput>,
) -> Result<AcceptResponse, AppError> {
    let mut tx_outer = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let lock_result = with_account_lock(
        &mut tx_outer,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            accept_batch(
                tx_inner,
                company_id,
                bank_account_id,
                user_id,
                actor_api_key_id,
                &proposals,
            )
            .await
        },
    )
    .await;

    match lock_result {
        Ok(response) => {
            tx_outer
                .commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(response)
        }
        // Story 25-4-c2 — la transaction du lot a été annulée sous lui par
        // InnoDB (interblocage) : rejouable, cf. `post_accept`.
        Err(ReconciliationError::TransactionAborted { source }) => {
            drop(tx_outer);
            tracing::warn!(error = %source, "reconciliation accept: transaction aborted underneath (deadlock), retrying");
            Err(AppError::ReconciliationTransactionAborted)
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
        // Story 8-5b — variants Rule ajoutés à `ReconciliationError`
        // pour les handlers `post_create_rule`/`post_update_rule` +
        // `accept_one_rule` step 2. Ce flow (`accept_batch`) ne les émet
        // pas : les conflits Rule sont gérés en handlers CRUD séparés ou
        // en `FailedProposal` per-proposal (cf. §accept-with-rule-flow).
        // Branche exhaustive uniquement.
        Err(
            ReconciliationError::RuleNotFound { .. }
            | ReconciliationError::RuleNoLongerMatches { .. }
            | ReconciliationError::RuleMismatch { .. }
            | ReconciliationError::RuleDuplicate { .. },
        ) => {
            // Pass 1 code review HIGH BH1 fix : defensive 500 au lieu
            // de unreachable!() qui crash le task Tokio.
            drop(tx_outer);
            tracing::error!(
                "Story 8-5b Rule variant propagated to accept_batch — \
                 defensive 500 fallback (should not reach: Rule conflicts \
                 handled in CRUD handlers or per-proposal FailedProposal)"
            );
            Err(AppError::Internal(
                "internal: unexpected Rule variant in accept_batch".into(),
            ))
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
    actor_api_key_id: Option<i64>,
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
            actor_api_key_id,
            proposal,
            batch_size,
        )
        .await
        {
            Ok(entry) => {
                // Même lecture que la branche d'échec : un point de sauvegarde
                // disparu signale une transaction annulée sous le lot.
                if let Err(e) = sqlx::query(&format!("RELEASE SAVEPOINT {savepoint}"))
                    .execute(&mut **tx_outer)
                    .await
                {
                    if is_savepoint_lost(&e) {
                        return Err(ReconciliationError::TransactionAborted { source: e });
                    }
                    return Err(ReconciliationError::Database(e));
                }
                accepted.push(entry);
            }
            Err(failure) => {
                // Story 25-4-c2 (#480) — un 1213 absorbé dans la proposition a
                // annulé TOUTE la transaction : le point de sauvegarde n'existe
                // plus (1305). Le signaler comme tel, pour que la route rejoue
                // le lot au lieu de rendre un 500 opaque.
                if let Err(e) = sqlx::query(&format!("ROLLBACK TO SAVEPOINT {savepoint}"))
                    .execute(&mut **tx_outer)
                    .await
                {
                    if is_savepoint_lost(&e) {
                        return Err(ReconciliationError::TransactionAborted { source: e });
                    }
                    return Err(ReconciliationError::Database(e));
                }
                failed.push(failure);
            }
        }
    }

    Ok(AcceptResponse { accepted, failed })
}

/// `TransactionAborted` n'est posée que par `accept_batch` : ailleurs, elle
/// signale un défaut, rendu en `500` (Story 25-4-c2 — bras défensif partagé par
/// `post_reject`, `post_manual` et `post_split`).
fn transaction_aborted_outside_accept(source: &sqlx::Error) -> AppError {
    tracing::error!(error = %source, "unexpected TransactionAborted outside accept_batch");
    AppError::ReconciliationTransactionAborted
}

/// Code MariaDB `ER_SP_DOES_NOT_EXIST` — ici, un point de sauvegarde disparu
/// avec la transaction qu'InnoDB a annulée.
const MARIADB_SAVEPOINT_DOES_NOT_EXIST: u16 = 1305;

/// Le point de sauvegarde du lot a-t-il disparu ? Lu sur le **code** MySQL,
/// jamais sur le texte du message (Story 25-4-c2).
fn is_savepoint_lost(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db| db.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .is_some_and(|my| my.number() == MARIADB_SAVEPOINT_DOES_NOT_EXIST)
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
    actor_api_key_id: Option<i64>,
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
                actor_api_key_id,
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
                actor_api_key_id,
                *bank_transaction_id,
                splits,
                *value_date,
            )
            .await
        }
        AcceptProposalInput::Rule {
            bank_transaction_id,
            rule_id,
            counterparty_account_id,
        } => {
            accept_one_rule(
                tx,
                company_id,
                bank_account_id,
                user_id,
                actor_api_key_id,
                *bank_transaction_id,
                *rule_id,
                *counterparty_account_id,
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
///
/// **Postabilité — pas de garde ici, et c'est voulu** (Story 15-5b, AC6) :
/// aucun compte de cette écriture ne vient du client. Le compte bancaire est
/// un compte de configuration (D-A0, toléré devenu non imputable) ; la créance
/// est lue **sur l'écriture de vente** de la facture ; le compte d'arrondi vient
/// des réglages, déjà gardé `postable = TRUE` par
/// `company_invoice_settings::rounding_account_for_write`. Les deux autres
/// flux d'acceptation (`split`, `rule`) et les rapprochements manuel et ventilé
/// gardent, eux, le compte de contrepartie venu du client.
#[allow(clippy::too_many_arguments)]
async fn accept_one_invoice(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
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
    let sale_entry_id = invoice.journal_entry_id.unwrap();

    // Step 7 — re-calculer score serveur-side (M7 Pass 1).
    // #420 (25-4-c) : re-score sur le RESTE DÛ, la grandeur du filtre de
    // `find_unpaid_invoices_for_window` — forme scalaire (UNE facture), dans
    // la transaction ; les deux formes sont tenues d'accord par le test de
    // parité (`invoice_amount_due_parity.rs`).
    let invoice_amount_due = invoice_settlements::amount_due(&mut **tx, invoice.id)
        .await
        .map_err(|e| FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": e.to_string() })),
        })?;
    let candidates_for_score: Vec<(
        Invoice,
        Option<kesh_db::entities::Contact>,
        rust_decimal::Decimal,
    )> = vec![(
        invoice.clone(),
        contact.clone(),
        // #476 (25-4-c3-b) : au centime, comme le triplet des propositions.
        invoice_settlements::amount_due_to_centime(invoice_amount_due),
    )];
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

    // =====================================================================
    // Step 7ter — L'ÉCRITURE D'ENCAISSEMENT (Story 24-2, #371).
    //
    // ⚠️ Avant cette story, la transaction bancaire était rapprochée de
    // l'écriture de VENTE et aucune écriture de règlement n'était passée : le
    // compte débiteurs accumulait des débits jamais crédités, la banque était
    // sous-évaluée du même montant, et le bilan restait équilibré — donc muet.
    //
    // Le gabarit est `supplier_invoices::pay_in_tx`, symétrique et éprouvé.
    // =====================================================================

    // (a) Compte de banque au grand livre — inline, comme le chemin `split`
    //     (les helpers `bank_accounts::*` prennent un `&MySqlPool`, pas un
    //     Executor générique).
    let bank_ledger_row: Option<(Option<i64>,)> = sqlx::query_as(
        "SELECT journal_account_id FROM bank_accounts WHERE company_id = ? AND id = ? LIMIT 1",
    )
    .bind(company_id)
    .bind(bank_account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;
    let bank_ledger_account_id = match bank_ledger_row {
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

    // (b) ⛔ Le compte de créance se lit SUR L'ÉCRITURE DE VENTE, jamais sur les
    //     réglages. C'est ce qui garantit que le compte se solde exactement,
    //     quoi qu'il soit arrivé à la configuration entre l'émission de la
    //     facture et son encaissement. Miroir strict de l'étape (2) de
    //     `pay_in_tx`, qui lit la ligne de CRÉDIT de l'écriture d'achat.
    //
    //     La créance est la PREMIÈRE ligne au débit de l'écriture de vente, en
    //     position 0, pour le TTC — d'où `ORDER BY jel.id LIMIT 1`. ⚠️ Pas la
    //     SEULE : un arrondi à 5 centimes négatif (Story 25-4-c4-a) ajoute une
    //     ligne de débit, toujours APRÈS la créance
    //     (`generate_invoice_journal_lines_rounded`).
    let receivable_row: Option<(i64,)> = sqlx::query_as(
        "SELECT jel.account_id FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? AND jel.debit > 0 \
         ORDER BY jel.id LIMIT 1",
    )
    .bind(sale_entry_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;
    let receivable_account_id = match receivable_row {
        Some((id,)) => id,
        None => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "INVOICE_SALE_ENTRY_MALFORMED".to_string(),
                details: Some(serde_json::json!({
                    "reason": "no_debit_line_on_sale_entry",
                    "saleEntryId": sale_entry_id,
                })),
            });
        }
    };

    // (c) ⛔ Le trop-perçu est REFUSÉ, il ne s'écrit pas. Sans ce garde, le
    //     compte de créance passerait CRÉDITEUR — un solde contre nature que le
    //     grand livre signalerait, mais après coup.
    //
    //     #476 (25-4-c3-b) : à DOUBLE BORNE, par la classification partagée avec
    //     le règlement manuel. Une transaction égale au reste arrondi au centime
    //     solde la facture : le règlement s'enregistre au reste BRUT et l'écart
    //     passe sur le compte de différences d'arrondi.
    let due_before = invoice_settlements::amount_due(&mut **tx, invoice_id)
        .await
        .map_err(|e| FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": e.to_string() })),
        })?;
    let settled_amount =
        match invoice_settlements::classify_payment(bank_transaction.amount, due_before) {
            PaymentAgainstDue::Overpayment => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "RECONCILIATION_OVERPAYMENT".to_string(),
                    details: Some(serde_json::json!({
                        "amountDue": due_before,
                        "transactionAmount": bank_transaction.amount,
                    })),
                });
            }
            PaymentAgainstDue::Ordinary => bank_transaction.amount,
            PaymentAgainstDue::SettlesWithRounding { raw_due } => raw_due,
        };
    // (c-bis) Le compte d'arrondi, exigé SEULEMENT s'il y a un écart, et vérifié
    //         au moment d'écrire (AC 4) — refus per-proposal, pattern batch.
    let rounding_account_id = if settled_amount != bank_transaction.amount {
        match company_invoice_settings::rounding_account_for_write(
            tx,
            company_id,
            kesh_db::errors::RoundingContext::Payment,
        )
        .await
        {
            Ok(id) => Some(id),
            Err(DbError::RoundingAccountNotConfigured { .. }) => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "ROUNDING_ACCOUNT_NOT_CONFIGURED".to_string(),
                    details: Some(serde_json::json!({
                        "amountDue": due_before,
                        "transactionAmount": bank_transaction.amount,
                    })),
                });
            }
            Err(e) => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "DATABASE_ERROR".to_string(),
                    details: Some(serde_json::json!({ "message": e.to_string() })),
                });
            }
        }
    } else {
        None
    };

    // (d) Exercice OUVERT couvrant la date de valeur. Jamais d'écriture dans un
    //     exercice clos — c'est le verrou de bouclement, pas une commodité.
    let fiscal_year =
        match fiscal_years::find_open_covering_date(tx, company_id, paid_at_candidate).await {
            Ok(Some(fy)) => fy,
            Ok(None) => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "FISCAL_YEAR_INVALID".to_string(),
                    details: Some(serde_json::json!({ "valueDate": paid_at_candidate })),
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

    // (e) `D banque / C créance`, du MONTANT DE LA TRANSACTION — jamais du total
    //     de la facture. L'écriture existe pour que le compte bancaire de Kesh
    //     égale le relevé. #476 : la créance est créditée du montant RÉGLÉ, et
    //     l'écart éventuel passe en troisième ligne.
    let lines = match invoice_settlements::settlement_journal_lines(
        bank_ledger_account_id,
        receivable_account_id,
        bank_transaction.amount,
        settled_amount,
        rounding_account_id,
    ) {
        Ok(lines) => lines,
        // Seule erreur possible : un écart sans compte d'arrondi — impossible
        // tant que (c-bis) exige le compte dès que réglé ≠ payé. Si un refactor
        // défaisait ce couplage, c'est un BUG STRUCTUREL : tracé, et rendu sous
        // un code qui ne se confond pas avec une panne de base (revue P1, C).
        Err(e) => {
            tracing::error!("encaissement : lignes d'écriture impossibles à construire : {e}");
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "INTERNAL_ERROR".to_string(),
                details: None,
            });
        }
    };
    let label = invoice
        .invoice_number
        .clone()
        .unwrap_or_else(|| invoice.id.to_string());
    let contact_name = contact.as_ref().map(|c| c.name.clone()).unwrap_or_default();
    let new_je = kesh_db::entities::NewJournalEntry {
        company_id,
        entry_date: paid_at_candidate,
        journal: kesh_db::entities::Journal::Banque,
        description: format!("Encaissement facture {label} - {contact_name}"),
        // Le règlement hérite du projet de la facture (cohérence analytique,
        // patron `pay_in_tx`).
        project_id: invoice.project_id,
        lines,
    };
    // Flux automatique (réconciliation) : garde de postabilité désactivée
    // (Story 14-3b, D-A0) — cohérent `pay_in_tx` et chemin `split`.
    let settlement_je =
        match journal_entries::create_in_tx(tx, fiscal_year.id, user_id, new_je, false).await {
            Ok(j) => j,
            Err(e) => {
                return Err(project_error_to_failed_proposal(
                    bank_transaction_id,
                    invoice.project_id,
                    e,
                ));
            }
        };
    let journal_entry_id = settlement_je.entry.id;

    // (f) La liaison facture ↔ écriture. `UNIQUE (journal_entry_id)` en base :
    //     une écriture d'encaissement règle UNE facture.
    if let Err(e) = invoice_settlements::create_in_tx(
        tx,
        kesh_db::entities::NewInvoiceSettlement {
            company_id,
            invoice_id,
            journal_entry_id,
            // #476 : le reste BRUT quand la transaction solde avec un écart.
            amount: settled_amount,
            settled_on: paid_at_candidate,
            // La réconciliation bancaire est, par définition, un virement — et
            // le compte est celui de l'import, pas un choix de l'utilisateur
            // (Story 24-3).
            kind: kesh_db::entities::SettlementKind::Choice(
                kesh_db::entities::SettlementChoice::BankTransfer { bank_account_id },
            ),
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

    // (g) ⛔ `paid_at` est la PROJECTION d'un solde tombé à zéro, pas un drapeau
    //     posé par le premier virement. Une facture partiellement réglée garde
    //     `paid_at` à NULL — et reste donc relançable et rapprochable pour le
    //     solde, sans qu'aucun des cinq sites qui lisent `paid_at IS NULL` ait à
    //     changer.
    let due_after = invoice_settlements::amount_due(&mut **tx, invoice_id)
        .await
        .map_err(|e| FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": e.to_string() })),
        })?;
    let fully_settled = due_after <= Decimal::ZERO;

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

    // ⛔ `paid_at` seulement si le solde est tombé à zéro (Story 24-2). Un
    // encaissement partiel bump la version et `updated_at` — la facture a bien
    // changé d'état — mais laisse `paid_at` à NULL.
    //
    // ⛔ **Ce `version = ?` est le verrou qui interdit de régler deux fois**
    // (Story 25-4-c2, #480). Le reste dû (`due_before`, garde (c)) est lu sur
    // l'**instantané** de la transaction, fixé dès sa première lecture : un
    // écrit validé depuis est invisible. Cet `UPDATE`, lui, lit la version
    // **courante** : il ne touche aucune ligne si la facture a changé depuis
    // l'instantané, et la proposition est refusée sans rien écrire. Cela ne
    // vaut que par l'invariant : **tout écrit qui change le reste dû incrémente
    // `version`** — l'acceptation (ici), le règlement manuel
    // (`invoice_settlements_write::settle_invoice`, partiel compris), l'annulation
    // d'un règlement (`cancel_settlement_in_tx`), l'émission d'un avoir
    // (`credit_notes::create_credit_note`), la dévalidation (`invoices::unvalidate`).
    //
    // ⛔ **Pas de `FOR UPDATE` sur la facture en amont** : il lirait `version` à
    // jour — ce contrôle passerait toujours — alors que le reste dû resterait lu
    // sur l'instantané. Il désarmerait le verrou au lieu de le renforcer.
    //
    // ⚠️ MariaDB 10.11, `innodb_snapshot_isolation = OFF`. À partir de la 11.6,
    // la valeur par défaut passe à `ON` : un `UPDATE` sur une ligne modifiée
    // depuis l'instantané échoue alors en 1020 (*Record has changed since last
    // read*) au lieu de lire la version courante. Le refus reste juste, mais il
    // sortirait en `DATABASE_ERROR` et non en `race_during_update`.
    let paid_at_to_set: Option<chrono::NaiveDateTime> = if fully_settled {
        Some(paid_at_dt)
    } else {
        None
    };
    let invoice_update = sqlx::query(
        "UPDATE invoices \
         SET paid_at = ?, version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'validated'",
    )
    .bind(paid_at_to_set)
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
        // Story 24-2 : l'écriture d'ENCAISSEMENT, plus celle de vente.
        "journal_entry_id": journal_entry_id,
        "sale_journal_entry_id": sale_entry_id,
        // #476 : réglé ≠ payé quand la transaction solde avec un écart d'arrondi.
        "settled_amount": settled_amount,
        "paid_amount": bank_transaction.amount,
        "rounding_difference": bank_transaction.amount - settled_amount,
        "amount_due_after": due_after,
        "fully_settled": fully_settled,
    });
    let entry_accepted = audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
            "reconciliation.accepted",
            "bank_transaction",
            bank_transaction_id,
            Some(details_accepted),
        ),
    )
    .await
    .map_err(|e| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    })?;

    // Step 10 — dual audit log (HP3-3 Pass 3 + MP4-4 Pass 4).
    //
    // ⚠️ Story 24-2 : l'action dépend désormais de l'effet réel. Un encaissement
    // partiel n'est PAS `invoice.paid` — écrire « payée » sur une facture qui ne
    // l'est pas remettrait un mensonge dans la piste d'audit, à l'endroit précis
    // où elle doit faire foi.
    let action = if fully_settled {
        "invoice.paid"
    } else {
        "invoice.partially_settled"
    };
    let details_paid = serde_json::json!({
        "paid_at": paid_at_to_set.map(|d| d.and_utc()),
        "paid_by_user_id": user_id,
        "paid_via": "reconciliation",
        "reconciliation_audit_id": entry_accepted.id,
        "settlement_journal_entry_id": journal_entry_id,
        // #476 : réglé ≠ payé quand la transaction solde avec un écart d'arrondi.
        "settled_amount": settled_amount,
        "paid_amount": bank_transaction.amount,
        "rounding_difference": bank_transaction.amount - settled_amount,
        "amount_due_after": due_after,
        "before": { "paid_at": null, "version": invoice_version_pre },
        "after": {
            "paid_at": paid_at_to_set.map(|d| d.and_utc()),
            "version": invoice_version_pre + 1
        },
    });
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
            action,
            "invoice",
            invoice_id,
            Some(details_paid),
        ),
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
    actor_api_key_id: Option<i64>,
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
    //
    // Story 15-5b (AC3, #427) : un compte de contrepartie **non imputable**
    // (regroupement, résultat, clôture) est refusé en `ACCOUNT_NOT_POSTABLE`.
    // Les manquants (inconnu, autre société, archivé) restent prioritaires en
    // `ACCOUNT_NOT_FOUND` (anti-énumération KF-002) : la variante ne nomme
    // qu'un compte de la société, actif. Pas d'exemption — le compte vient de
    // la proposition, pas d'un réglage en place.
    let mut missing: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
    let mut not_postable: Vec<kesh_db::errors::NonPostableAccount> = Vec::new();
    for s in splits {
        let row: Option<(bool, bool, String)> = match sqlx::query_as(
            "SELECT active, postable, number FROM accounts WHERE id = ? AND company_id = ? LIMIT 1",
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
            Some((true, true, _)) => {}
            Some((true, false, number)) => {
                not_postable.push(kesh_db::errors::NonPostableAccount {
                    account_id: s.counterparty_account_id,
                    account_number: number,
                });
            }
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
    if !not_postable.is_empty() {
        return Err(non_postable_failed_proposal(
            bank_transaction_id,
            not_postable,
        ));
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
    // Story 19-5 : chaque ligne de ventilation porte son projet analytique
    // (multi-usage). La validation projet (existant/company/non archivé) est
    // faite per-ligne par create_in_tx (étape 0) — mappée en FailedProposal
    // ci-dessous plutôt qu'en DATABASE_ERROR opaque.
    // ⚠️ Ordre des verrous : l'exercice est DÉJÀ verrouillé (Step i) ; l'étape
    // 0 de `create_in_tx` prend ensuite la sentinelle et les projets des
    // lignes — l'ordre INVERSE de `journal_entries::create`. Un cycle peut se
    // former : `post_accept` est rejouée, comme l'autre côté du cycle ; l'ordre
    // n'en réduit que la fréquence (Pattern 5, « Global Lock Order »).
    let split_details: Vec<SplitDetail> = splits
        .iter()
        .map(|s| SplitDetail {
            account_id: s.counterparty_account_id,
            amount: s.amount,
            description: s.description.clone(),
            project_id: s.project_id,
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
    // Flux automatique (réconciliation) : garde de postabilité désactivée (14-3b, D-A0).
    let je = match journal_entries::create_in_tx(tx, fiscal_year.id, user_id, new_je, false).await {
        Ok(j) => j,
        Err(e) => {
            // Split : la validation projet per-ligne se fait DANS create_in_tx,
            // donc un NotFound/IllegalStateTransition("...projet...") est bien un
            // projet fautif (id exact non isolable → None).
            return Err(project_error_to_failed_proposal(
                bank_transaction_id,
                None,
                e,
            ));
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
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
            "reconciliation.split_applied",
            "bank_transaction",
            bank_transaction_id,
            Some(details),
        ),
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
// Story 8-5b — accept_one_rule (FR47)
// ============================================================

/// Story 8-5b — traite UNE proposal `type='rule'` dans son savepoint.
///
/// Pattern strict `accept_one_split` (cf. spec ECH4-1 Pass 4) :
/// retourne `Result<AcceptedProposal, FailedProposal>` pour TOUTES les
/// erreurs per-proposal. Aucune escalade en `AppError` global — le
/// batch handler accumule les `FailedProposal` et retourne 200 OK avec
/// `failed[]` non vide.
///
/// 16 steps documentés §accept-with-rule-flow. Décisions Pass 1-4 :
/// - Step 0 : currency CHF defense-in-depth (Pass 3 R1).
/// - Step 4 : counterparty mismatch RULE_MISMATCH per-proposal (AC #120).
/// - Step 7 : re-validation `rule_matches` anti-race (AC #119).
/// - Step 11 : description handler-side `"Règle '{label}' — {counterparty}"`
///   tronquée 200 chars UTF-8-safe (Pass 2 Q9).
/// - Step 12 : `manual::build_journal_entry_for_counterparty` réutilisé
///   (signature stable 8-5a-base).
#[allow(clippy::too_many_arguments)]
async fn accept_one_rule(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
    bank_transaction_id: i64,
    rule_id: i64,
    counterparty_account_id: i64,
) -> Result<AcceptedProposal, FailedProposal> {
    // Step 1 — bank_account.journal_account_id lookup (inline, transaction-bound).
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
            // Step 3 (Pass 4 ECH4-1) — per-proposal FailedProposal (PAS 412 global).
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

    // Step 2 — SELECT rule (transaction-bound). RuleNotFound si None ou inactive.
    let rule =
        match reconciliation_rules::find_by_id_for_company(&mut **tx, company_id, rule_id).await {
            Ok(Some(r)) if r.active => r,
            Ok(_) => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "RECONCILIATION_RULE_NOT_FOUND".to_string(),
                    details: Some(serde_json::json!({ "ruleId": rule_id })),
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

    // Step 4 — counterparty mismatch check (AC #120).
    if rule.counterparty_account_id != counterparty_account_id {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_RULE_MISMATCH".to_string(),
            details: Some(serde_json::json!({
                "ruleId": rule_id,
                "expectedAccount": rule.counterparty_account_id,
                "actualAccount": counterparty_account_id,
            })),
        });
    }

    // Step 5 — counterparty account actif ET imputable (SELECT inline).
    //
    // Story 15-5b (AC4, #427) : un compte **non imputable** est refusé en
    // `ACCOUNT_NOT_POSTABLE`, **sans exemption** (choix C4) — une règle dont le
    // compte a été scindé en sous-comptes est périmée ; `get_proposals` ne la
    // propose déjà plus (AC5), cette garde couvre la proposition rejouée par un
    // client. Archivé / inconnu → `ACCOUNT_NOT_FOUND`, prioritaire (KF-002).
    let counterparty_row: Option<(bool, bool, String, String)> = match sqlx::query_as(
        "SELECT active, postable, name, number FROM accounts WHERE id = ? AND company_id = ? LIMIT 1",
    )
    .bind(counterparty_account_id)
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
    let counterparty_name = match counterparty_row {
        Some((true, true, name, _)) => name,
        Some((true, false, _, number)) => {
            return Err(non_postable_failed_proposal(
                bank_transaction_id,
                vec![kesh_db::errors::NonPostableAccount {
                    account_id: counterparty_account_id,
                    account_number: number,
                }],
            ));
        }
        _ => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "ACCOUNT_NOT_FOUND".to_string(),
                details: Some(serde_json::json!({
                    "missingAccountIds": [counterparty_account_id],
                })),
            });
        }
    };

    // Step 6 — refetch tx INSIDE lock (TOCTOU, pattern accept_one_split).
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

    // Step 0/6.5 — currency filter CHF (Pass 3 R1 — defense-in-depth, le
    // get_proposals filter aussi côté upstream).
    if bt.currency != "CHF" {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_CURRENCY_MISMATCH".to_string(),
            details: Some(serde_json::json!({
                "transactionCurrency": bt.currency,
                "expected": "CHF",
            })),
        });
    }

    // Step 7 — re-validation rule_matches anti-race (AC #119).
    if !rule_matches(&rule, &bt) {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_RULE_NO_LONGER_MATCHES".to_string(),
            details: Some(serde_json::json!({ "ruleId": rule_id })),
        });
    }

    // Step 8 — tx.amount != 0 (helper panic guard 8-5a-base).
    if bt.amount.is_zero() {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "VALIDATION_ERROR".to_string(),
            details: Some(serde_json::json!({ "reason": "zero_amount_transaction" })),
        });
    }

    // Step 9 — entry_date = tx.value_date.unwrap_or(tx.booking_date) (Pass 1 P-H1).
    let entry_date = bt.value_date.unwrap_or(bt.booking_date);

    // Step 10 — fiscal_year covering.
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

    // Step 11 — description handler-side (Pass 2 Q9), UTF-8-safe truncate 200.
    let raw_description = format!("Règle '{}' — {}", rule.label, counterparty_name);
    let description: String = raw_description.chars().take(200).collect();

    // Step 11bis (Story 19-5) — résout le projet analytique par défaut de la
    // règle et le RE-VALIDE ici (le projet a pu être archivé depuis la
    // création de la règle ; toute nouvelle écriture au grand livre doit être
    // sur projet actif, DC3). Le repo ne valide pas `new.project_id`
    // document-level (19-2 DC2) → validation explicite avant create_in_tx.
    // Projet archivé/absent → FailedProposal per-proposition (PROJECT_ARCHIVED
    // / PROJECT_NOT_FOUND), jamais d'AppError globale.
    // ⚠️ Ordre des verrous : l'exercice est DÉJÀ verrouillé (Step 10) ;
    // `validate_taggable_in_tx` prend ensuite la sentinelle `companies` puis le
    // projet — l'ordre INVERSE de `journal_entries::create` et de
    // `create_opening_entry`. Un cycle peut se former : `post_accept` est
    // rejouée (son `retry_with`, 1213 et 1305), comme l'autre côté du cycle ;
    // l'ordre n'en réduit que la fréquence (Pattern 5, « Global Lock Order »).
    let default_project_id = rule.default_project_id;
    if let Some(pid) = default_project_id
        && let Err(e) = projects::validate_taggable_in_tx(tx, company_id, &[pid]).await
    {
        // Erreur issue exclusivement de la validation projet → mapping
        // canonique avec projectId dans details (AC11).
        return Err(project_error_to_failed_proposal(
            bank_transaction_id,
            Some(pid),
            e,
        ));
    }

    // Step 12 — build_journal_entry_for_counterparty + create_in_tx.
    let new_je = build_journal_entry_for_counterparty(
        &bt,
        bank_ledger_account_id,
        counterparty_account_id,
        description,
        entry_date,
        default_project_id,
    );
    // Flux automatique (réconciliation) : garde de postabilité désactivée (14-3b, D-A0).
    let je = match journal_entries::create_in_tx(tx, fiscal_year.id, user_id, new_je, false).await {
        Ok(j) => j,
        Err(e) => {
            // Le projet par défaut (document-level) a déjà été validé au step
            // 11bis ; un NotFound ici provient d'une autre cause (jamais projet,
            // les lignes portent project_id=None) → mapping générique pour ne
            // pas mal étiqueter en PROJECT_NOT_FOUND (Pass 1 LOW BH/ECH).
            //
            // ⛔ Story 24-4c (#380) : le verrou de période fait EXCEPTION au
            // repli générique — c'est un refus métier, pas une panne de base.
            if let kesh_db::errors::DbError::PeriodLocked {
                locked_through,
                attempted,
            } = &e
            {
                return Err(period_locked_failed_proposal(
                    bank_transaction_id,
                    None,
                    *locked_through,
                    *attempted,
                ));
            }
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
    };
    let journal_entry_id = je.entry.id;

    // Step 13 — UPDATE bank_transactions optimistic lock (pattern accept_one_split).
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
    .bind(bt.version)
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

    // Step 14 — increment_applied_count_in_tx (atomique avec version+1
    // dans le SET — pas de WHERE version=? per Pass 1 ECH-10). Pass 1
    // code review HIGH AA1 fix : version doit être bumped pour cohérence
    // optimistic lock côté PATCH suivant.
    let applied_count_after =
        match reconciliation_rules::increment_applied_count_in_tx(tx, company_id, rule_id).await {
            Ok(count) => count,
            Err(e) => {
                return Err(FailedProposal {
                    bank_transaction_id,
                    error_code: "DATABASE_ERROR".to_string(),
                    details: Some(serde_json::json!({ "message": e.to_string() })),
                });
            }
        };

    // Step 15 — audit reconciliation_rule.applied (action distincte Q4b).
    let was_previously_rejected = bt.auto_match_rejected_at.is_some();
    // Pass 3 R4 — `value_date` brut nullable (peut être null), `entry_date`
    // résolu non-null (distinct du value_date) pour traçabilité.
    // Pass 1 code review HIGH AA2 fix : `applied_count_after` requis par spec.
    let rule_applied_details = serde_json::json!({
        "rule_id": rule_id,
        "rule_label": rule.label,
        "match_type": rule.match_type.as_str(),
        "match_value": rule.match_value,
        "bank_transaction_id": bank_transaction_id,
        "counterparty_account_id": counterparty_account_id,
        "journal_entry_id": journal_entry_id,
        "applied_count_after": applied_count_after,
        "value_date": bt.value_date.map(|d| d.to_string()),
        "entry_date": entry_date.to_string(),
        "was_previously_rejected": was_previously_rejected,
    });
    if let Err(e) = audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
            "reconciliation_rule.applied",
            "reconciliation_rules",
            rule_id,
            Some(rule_applied_details),
        ),
    )
    .await
    {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": e.to_string() })),
        });
    }

    // Step 16 — audit reconciliation.accepted avec details.type='rule' (extension Pass 1 P-M).
    let mut amount_audit = bt.amount;
    amount_audit.rescale(2);
    // Pass 1 code review HIGH AA3 fix : `match_type` requis par spec step 15.
    let accepted_details = serde_json::json!({
        "type": "rule",
        "bank_transaction_id": bank_transaction_id,
        "rule_id": rule_id,
        "match_type": rule.match_type.as_str(),
        "counterparty_account_id": counterparty_account_id,
        "journal_entry_id": journal_entry_id,
        "amount": amount_audit.to_string(),
        "value_date": bt.value_date.map(|d| d.to_string()),
        "entry_date": entry_date.to_string(),
        "was_previously_rejected": was_previously_rejected,
    });
    if let Err(e) = audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
            "reconciliation.accepted",
            "bank_transaction",
            bank_transaction_id,
            Some(accepted_details),
        ),
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
        // Sentinel invoice_id=0 (dette transverse 8-5a-bis BH-H1 v0.2).
        invoice_id: 0,
        journal_entry_id,
        // Pass 1 code review LOW EC9 fix : score.total = 1.0 cohérent
        // avec celui exposé par get_proposals pour candidate type=rule
        // (`total: 1.0` au lieu de `0.0` qui pouvait être interprété
        // comme "no match" par un consumer).
        score: MatchScore {
            total: 1.0,
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

    // Pré-flight ownership + archived guard (Story v014-1, FINDING-6 Pass 3 Opus).
    let ba_check = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;
    if ba_check.archived {
        return Err(AppError::BankAccountNotFound);
    }

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
    // Story 17-2a (DC5 cat ii) — attribution PAT propagée dans les helpers/closures.
    let actor_api_key_id = current_user.api_key_id;
    let company_id = current_user.company_id;
    let ids = body.bank_transaction_ids.clone();

    let lock_result = with_account_lock(
        &mut tx_outer,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            reject_batch(
                tx_inner,
                company_id,
                user_id,
                actor_api_key_id,
                &ids,
                &tx_map,
            )
            .await
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
        Err(ReconciliationError::TransactionAborted { source }) => {
            drop(tx_outer);
            Err(transaction_aborted_outside_accept(&source))
        }
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
        // Story 8-5b — branche exhaustive (reject_batch ne touche pas
        // aux rules). Unreachable en pratique.
        Err(
            ReconciliationError::RuleNotFound { .. }
            | ReconciliationError::RuleNoLongerMatches { .. }
            | ReconciliationError::RuleMismatch { .. }
            | ReconciliationError::RuleDuplicate { .. },
        ) => {
            drop(tx_outer);
            tracing::error!("Story 8-5b Rule variant propagated to reject_batch — defensive 500");
            Err(AppError::Internal(
                "internal: unexpected Rule variant in reject_batch".into(),
            ))
        }
    }
}

async fn reject_batch(
    tx_outer: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
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
            NewAuditLogEntry::for_actor(
                user_id,
                actor_api_key_id,
                "reconciliation.rejected",
                "bank_transaction",
                rejected[0].bank_transaction_id,
                Some(details),
            ),
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
    /// Projet analytique document-level (Story 19-5) — optionnel. Recopié sur
    /// les 2 lignes de l'écriture. Validé handler-side avant `create_in_tx`.
    #[serde(default)]
    pub project_id: Option<i64>,
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
///    3bis. compte de contrepartie non imputable → 400 ACCOUNT_NOT_POSTABLE
///    (Story 15-5b, #427) — après le 404, prioritaire (anti-énumération).
/// 4. `find_strictly_pending_by_id_for_account` → 404 RECONCILIATION_TRANSACTION_NOT_PENDING.
///    4bis. `tx.amount != 0` → 400 VALIDATION_ERROR (zero_amount_transaction, F7''' Pass 3).
/// 5. (inside lock 5-9) : re-fetch tx (TOCTOU) + fiscal_year + create_in_tx
///    + UPDATE bank_transactions optimistic lock + audit log.
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe
/// [`crate::retry::retry_app_on_deadlock`], opération `reconciliation::manual` ;
/// la transaction vit dans [`post_manual_once`].
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
    if let Some(d) = &body.description
        && d.chars().count() > MAX_MANUAL_DESCRIPTION_LEN
    {
        return Err(AppError::Validation(format!(
            "description trop longue (max {MAX_MANUAL_DESCRIPTION_LEN} caractères)"
        )));
    }

    // Step 1 — bank_account ownership pré-flight (KF-002 multi-tenant)
    // + archived guard (Story v014-1, FINDING-6 Pass 3 Opus).
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
    if bank_account.archived {
        return Err(AppError::BankAccountNotFound);
    }

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
    // Step 3 bis — Story 15-5b (AC1, #427) : un compte de contrepartie **non
    // imputable** (regroupement, résultat, clôture) est refusé en 400
    // `ACCOUNT_NOT_POSTABLE`, après le 404 ci-dessus (prioritaire) et avant
    // toute écriture. Le compte bancaire, compte de configuration, n'est pas
    // concerné (D-A0) : `create_in_tx` reçoit toujours `enforce_postable =
    // false`, la garde porte sur le seul compte venu du client.
    if !counterparty.postable {
        return Err(AppError::Database(DbError::accounts_not_postable([
            kesh_db::errors::NonPostableAccount {
                account_id: counterparty.id,
                account_number: counterparty.number.clone(),
            },
        ])));
    }

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

    // Revue P1 de la 15-5e2 (L-1/B-2) : ce que l'audit écrit
    // (`was_previously_rejected`, montant) n'est plus capturé ici mais relu
    // DANS la tentative, sur la transaction bancaire re-lue à l'étape 5 — une
    // valeur lue avant la première tentative pouvait être périmée à la
    // suivante. `bank_transaction` ne sert qu'aux pré-contrôles.
    drop(bank_transaction);

    // `counterparty` (Account complet) consommé jusque ici par les checks
    // step 3 (active=true). Drop explicite pour signaler la fin d'usage.
    drop(counterparty);

    // Step 5+ — tout le reste dans UNE tentative (`post_manual_once`), que
    // l'enveloppe `AppError` rejoue en entier sur interblocage (Story 15-5e2) :
    // transaction neuve, verrou de compte repris. Les contrôles 0 à 4bis
    // ci-dessus lisent hors transaction et restent hors de la fermeture ; ceux
    // qui lisent ce que la transaction verrouille (re-fetch, exercice, projet)
    // sont dans la tentative, dans leur ordre.
    crate::retry::retry_app_on_deadlock("reconciliation::manual", || {
        post_manual_once(&state.pool, &body, &current_user, bank_ledger_account_id)
    })
    .await
    .map(Json)
}

/// Une tentative de `POST /reconciliation/manual` (Story 15-5e2) : transaction
/// neuve, verrou nommé du compte bancaire, étapes 5 à 9 (re-fetch de la
/// transaction bancaire, exercice, projet, écriture, `UPDATE`, audit), commit.
///
/// ⛔ Un 1213 levé sous le verrou nommé remonte en
/// `ReconciliationError::Db(DbError::Sqlx(_))`, rendu par le `match` en
/// `AppError::Database(DbError::Sqlx(_))` **après** `rollback` — `with_account_lock`
/// a déjà relâché le verrou nommé — : c'est la forme que reconnaît le prédicat
/// de [`crate::retry::retry_app_on_deadlock`]. Le corps est reçu **par
/// référence** ; ce que la tentative consomme (le libellé) est cloné en elle,
/// et ce que l'audit écrit (`was_previously_rejected`, montant) est lu sur la
/// transaction bancaire re-lue à l'étape 5, jamais avant la tentative.
async fn post_manual_once(
    pool: &sqlx::MySqlPool,
    body: &ManualMatchBody,
    current_user: &CurrentUser,
    bank_ledger_account_id: i64,
) -> Result<ManualMatchResponse, AppError> {
    let description = body.description.clone().unwrap_or_default();
    let mut tx_outer = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let bank_account_id = body.bank_account_id;
    let bank_transaction_id = body.bank_transaction_id;
    let counterparty_account_id = body.counterparty_account_id;
    let user_id = current_user.user_id;
    // Story 17-2a (DC5 cat ii) — attribution PAT propagée dans les helpers/closures.
    let actor_api_key_id = current_user.api_key_id;
    let company_id = current_user.company_id;

    // Étapes 5 à 9 dans `with_account_lock` (atomicité §validation-handler-side
    // step 9 : l'audit ne sort PAS de la closure — F2'''' Pass 6 Opus).
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
            // Valeurs de l'audit (étape 9), lues dans la tentative (revue P1
            // de la 15-5e2, L-1/B-2).
            let was_previously_rejected = tx.auto_match_rejected_at.is_some();
            let bank_transaction_amount = tx.amount;

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

            // Step 6bis (Story 19-5) — valide le projet analytique document-level
            // AVANT create_in_tx (le repo ne valide pas `new.project_id`, 19-2
            // DC2). Projet inconnu → 404, archivé → 409 (mapping DbError).
            // ⚠️ Ordre des verrous : l'exercice est DÉJÀ verrouillé (étape 6) ;
            // `validate_taggable_in_tx` prend ensuite la sentinelle `companies`
            // puis les projets — l'ordre INVERSE de `journal_entries::create`
            // (étape 0 puis étape 1) et de `create_opening_entry`. Un cycle peut
            // donc se former ; la route est rejouée (enveloppe `AppError`, Story
            // 15-5e2), et l'ordre ne fait qu'en réduire la fréquence (Pattern 5,
            // « Global Lock Order »).
            if let Some(pid) = body.project_id {
                projects::validate_taggable_in_tx(tx_inner, company_id, &[pid])
                    .await
                    .map_err(ReconciliationError::Db)?;
            }

            // Step 7 — build_journal_entry_for_counterparty (helper
            // 8-5a-base T2) puis create_in_tx atomique.
            let new_je = build_journal_entry_for_counterparty(
                &tx,
                bank_ledger_account_id,
                counterparty_account_id,
                description.clone(),
                entry_date,
                body.project_id,
            );
            let je =
                // Flux automatique (réconciliation) : garde postabilité off (14-3b, D-A0).
                journal_entries::create_in_tx(tx_inner, fiscal_year.id, user_id, new_je, false)
                    .await?;
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
                NewAuditLogEntry::for_actor(
                    user_id,
                    actor_api_key_id,
                    "reconciliation.manual_matched",
                    "bank_transaction",
                    bank_transaction_id,
                    Some(details),
                ),
            )
            .await?;

            Ok(journal_entry_id)
        },
    )
    .await;

    // F1'''' Pass 6 Opus — match exhaustif, factorisé avec `post_split_once`
    // (revue P1 de la 15-5e2, B-3) dans [`conclude_locked_attempt`].
    let journal_entry_id = conclude_locked_attempt(tx_outer, lock_result, "post_manual").await?;
    Ok(ManualMatchResponse {
        bank_transaction_id,
        journal_entry_id,
    })
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
    /// Projet analytique de cette ligne de ventilation (Story 19-5) —
    /// optionnel. Validé per-ligne par `create_in_tx`.
    #[serde(default)]
    pub project_id: Option<i64>,
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
/// step 5bis comptes de contrepartie non imputables → 400 ACCOUNT_NOT_POSTABLE,
///   tous nommés (Story 15-5b, #427) — après le 404, prioritaire ;
/// step 6 `find_strictly_pending_by_id_for_account` → 404 RECONCILIATION_TRANSACTION_NOT_PENDING ;
/// step 6bis `tx.amount != 0` (M2''') → 400 VALIDATION_ERROR ;
/// step 7 `validate_split_balance` → 400 RECONCILIATION_SPLIT_IMBALANCE ;
/// steps 8-13 inside lock : re-fetch tx + fiscal_year + create_in_tx + UPDATE optimistic + audit.
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe
/// [`crate::retry::retry_app_on_deadlock`], opération `reconciliation::split` ;
/// la transaction vit dans [`post_split_once`].
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

    // Step 3 — bank_account ownership pré-flight (KF-002 multi-tenant)
    // + archived guard (Story v014-1, FINDING-6 Pass 3 Opus).
    let bank_account = bank_accounts::find_by_id_for_company(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
    )
    .await?
    .ok_or(AppError::BankAccountNotFound)?;
    if bank_account.archived {
        return Err(AppError::BankAccountNotFound);
    }

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
    //
    // Story 15-5b (AC2, #427) : les comptes **non imputables** sont collectés
    // à part et refusés en un seul 400 `ACCOUNT_NOT_POSTABLE` nommant tous
    // ceux de la requête — **après** les manquants, qui restent prioritaires
    // en 404 (anti-énumération KF-002).
    let mut missing: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
    let mut not_postable: Vec<kesh_db::errors::NonPostableAccount> = Vec::new();
    for s in &body.splits {
        match accounts_repo::find_by_id_in_company(
            &state.pool,
            s.counterparty_account_id,
            current_user.company_id,
        )
        .await?
        {
            Some(a) if a.active && a.postable => {}
            Some(a) if a.active => {
                not_postable.push(kesh_db::errors::NonPostableAccount {
                    account_id: a.id,
                    account_number: a.number,
                });
            }
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
    if !not_postable.is_empty() {
        return Err(AppError::Database(DbError::accounts_not_postable(
            not_postable,
        )));
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

    // Revue P1 de la 15-5e2 (L-1/B-2) : ce que l'audit écrit est relu DANS la
    // tentative, sur la transaction bancaire re-lue à l'étape 8. Le montant
    // d'une transaction importée ne change pas : l'équilibre contrôlé à
    // l'étape 7 sur la pré-lecture reste valable sous verrou.
    drop(bank_transaction);

    // Steps 8-13 dans UNE tentative (`post_split_once`), que l'enveloppe
    // `AppError` rejoue en entier sur interblocage (Story 15-5e2) : transaction
    // neuve, verrou de compte repris. Les contrôles 0 à 7 ci-dessus lisent hors
    // transaction et restent hors de la fermeture.
    crate::retry::retry_app_on_deadlock("reconciliation::split", || {
        post_split_once(&state.pool, &body, &current_user, bank_ledger_account_id)
    })
    .await
    .map(Json)
}

/// Une tentative de `POST /reconciliation/split` (Story 15-5e2) : transaction
/// neuve, verrou nommé du compte bancaire, étapes 8 à 13 (re-fetch, exercice,
/// écriture, `UPDATE`, audit), commit.
///
/// ⛔ Même chemin d'erreur que [`post_manual_once`] : un 1213 levé sous le
/// verrou nommé est rendu par le `match` en `AppError::Database(DbError::Sqlx(_))`
/// après `rollback`, la forme que reconnaît le prédicat de
/// [`crate::retry::retry_app_on_deadlock`]. Le corps est reçu **par
/// référence** ; ce que la tentative consomme (lignes, détails d'audit,
/// libellé) est reconstruit en elle, et `was_previously_rejected` comme le
/// montant de l'audit sont lus sur la transaction bancaire re-lue à l'étape 8.
async fn post_split_once(
    pool: &sqlx::MySqlPool,
    body: &SplitBody,
    current_user: &CurrentUser,
    bank_ledger_account_id: i64,
) -> Result<SplitResponse, AppError> {
    let splits_for_lock: Vec<SplitDetail> = body
        .splits
        .iter()
        .map(|s| SplitDetail {
            account_id: s.counterparty_account_id,
            amount: s.amount,
            description: s.description.clone(),
            // Story 19-5 — projet par ligne de ventilation (validé per-ligne
            // par create_in_tx).
            project_id: s.project_id,
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
    let mut tx_outer = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let bank_account_id = body.bank_account_id;
    let bank_transaction_id = body.bank_transaction_id;
    let body_value_date = body.value_date;
    let user_id = current_user.user_id;
    // Story 17-2a (DC5 cat ii) — attribution PAT propagée dans les helpers/closures.
    let actor_api_key_id = current_user.api_key_id;
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
            // Valeurs de l'audit (étape 13), lues dans la tentative (revue P1
            // de la 15-5e2, L-1/B-2).
            let was_previously_rejected = tx.auto_match_rejected_at.is_some();
            let bank_transaction_amount = tx.amount;

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
            // ⚠️ Ordre des verrous : l'exercice est DÉJÀ verrouillé (étape 9) ;
            // l'étape 0 de `create_in_tx` prend ensuite la sentinelle et les
            // projets des lignes — l'ordre INVERSE de `journal_entries::create`.
            // Un cycle peut se former ; la route est rejouée (enveloppe
            // `AppError`, Story 15-5e2), l'ordre n'en réduit que la fréquence.
            let je =
                // Flux automatique (réconciliation) : garde postabilité off (14-3b, D-A0).
                journal_entries::create_in_tx(tx_inner, fiscal_year.id, user_id, new_je, false)
                    .await?;
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
                NewAuditLogEntry::for_actor(
                    user_id,
                    actor_api_key_id,
                    "reconciliation.split_applied",
                    "bank_transaction",
                    bank_transaction_id,
                    Some(details),
                ),
            )
            .await?;

            Ok(journal_entry_id)
        },
    )
    .await;

    let journal_entry_id = conclude_locked_attempt(tx_outer, lock_result, "post_split").await?;
    Ok(SplitResponse {
        bank_transaction_id,
        journal_entry_id,
    })
}

/// Conclut une tentative de `post_manual` ou `post_split` (revue P1 de la
/// Story 15-5e2, B-3 : le `match` était recopié dans les deux) : `commit` si la
/// fermeture de `with_account_lock` a réussi, sinon `rollback` et conversion de
/// la [`ReconciliationError`] en [`AppError`]. `flow` nomme la route dans le
/// 500 défensif des variants `Rule*`.
///
/// ⛔ Un 1213 sort de la branche `Db` en `AppError::Database(DbError::Sqlx(_))`
/// **intact** : c'est la forme que reconnaît le prédicat de
/// [`crate::retry::retry_app_on_deadlock`]. Le `match` est exhaustif (F1''''
/// Pass 6 Opus) : le compilateur force la complétude à tout variant ajouté.
///
/// P-H4 Pass 1 (8-5a-base) : `rollback().await` explicite plutôt que `drop` ;
/// son échec éventuel est ignoré, l'erreur principale étant plus signifiante.
async fn conclude_locked_attempt(
    tx_outer: sqlx::Transaction<'_, sqlx::MySql>,
    lock_result: Result<i64, ReconciliationError>,
    flow: &'static str,
) -> Result<i64, AppError> {
    match lock_result {
        Ok(journal_entry_id) => {
            tx_outer
                .commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(journal_entry_id)
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
        // Story 8-5a-bis — ni `post_manual` ni `post_split` n'ouvrent de
        // savepoint : branche exhaustive, inatteignable en pratique.
        Err(ReconciliationError::TransactionAborted { source }) => {
            drop(tx_outer);
            Err(transaction_aborted_outside_accept(&source))
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
        // Story 8-5b — branche exhaustive (ces routes ne touchent pas aux
        // règles). Inatteignable en pratique.
        Err(
            ReconciliationError::RuleNotFound { .. }
            | ReconciliationError::RuleNoLongerMatches { .. }
            | ReconciliationError::RuleMismatch { .. }
            | ReconciliationError::RuleDuplicate { .. },
        ) => {
            let _ = tx_outer.rollback().await;
            tracing::error!("Story 8-5b Rule variant propagated to {flow} — defensive 500");
            Err(AppError::Internal(format!(
                "internal: unexpected Rule variant in {flow}"
            )))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Story 25-3-b (#418) — annuler un rapprochement
// ─────────────────────────────────────────────────────────────────────────────

/// Une transaction bancaire et ce qui décide de l'annulation de son
/// rapprochement — lus dans un seul instantané
/// ([`kesh_db::repositories::reconciliation_cancel::get_view`]).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationTransactionResponse {
    #[serde(flatten)]
    pub transaction: crate::routes::bank_imports::TransactionResponse,
    /// `invoice_settlement` ou `entry` ; `None` si la transaction n'est pas
    /// rapprochée.
    pub kind: Option<&'static str>,
    pub invoice_id: Option<i64>,
    pub invoice_number: Option<String>,
    pub cancellable: bool,
    /// Code du premier motif qui refuse l'annulation.
    pub cancel_blocked_by: Option<&'static str>,
    /// Le numéro du compte archivé (rang 4).
    pub cancel_blocked_label: Option<String>,
    /// L'**autre** transaction qui pointe la même écriture (rang 3).
    pub cancel_blocked_document_id: Option<i64>,
}

/// `GET /api/v1/reconciliation/transactions/{id}` — la transaction et ce qui
/// empêche d'annuler son rapprochement (Comptable+, Story 25-3-b).
///
/// ⚠️ **Une route, et non des champs du détail d'import** : un import porte des
/// centaines de transactions, et la précédence coûte quelques requêtes par
/// transaction — elle se calcule au clic, pour une seule.
pub async fn get_reconciliation_transaction(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ReconciliationTransactionResponse>, AppError> {
    let view = kesh_db::repositories::reconciliation_cancel::get_view(
        &state.pool,
        current_user.company_id,
        id,
    )
    .await?
    .ok_or(AppError::Database(DbError::NotFound))?;
    let (invoice_id, kind) = match view.kind {
        Some(k @ kesh_db::repositories::reconciliation_cancel::ReconciliationKind::InvoiceSettlement {
            invoice_id,
            ..
        }) => (Some(invoice_id), Some(k.code())),
        Some(k) => (None, Some(k.code())),
        None => (None, None),
    };
    let hit = view.cancel_blocker;
    Ok(Json(ReconciliationTransactionResponse {
        transaction: view.bank_transaction.into(),
        kind,
        invoice_id,
        invoice_number: view.invoice_number,
        cancellable: hit.is_none(),
        cancel_blocked_by: hit.as_ref().map(|h| h.0.code()),
        cancel_blocked_label: hit.as_ref().and_then(|h| h.2.clone()),
        cancel_blocked_document_id: hit.as_ref().and_then(|h| h.1),
    }))
}

/// Réponse de l'annulation d'un rapprochement.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelReconciliationResponse {
    /// La transaction relue, revenue « à rapprocher ».
    pub bank_transaction: crate::routes::bank_imports::TransactionResponse,
    /// L'écriture inverse, datée du jour.
    pub reversal_journal_entry_id: i64,
    /// La facture dont le règlement a été retiré, pour un règlement client.
    pub invoice_id: Option<i64>,
}

/// `POST /api/v1/reconciliation/transactions/{id}/cancel` — annule un
/// rapprochement par contre-passation (Comptable+, clés API d'écriture
/// admises, Story 25-3-b, #418).
///
/// ⛔ **Rejeu sur interblocage, au plus dehors.** Une acceptation de
/// proposition verrouille l'exercice du jour avant la facture ; ce geste, la
/// facture avant l'exercice du jour : sur deux comptes bancaires, ils peuvent
/// s'interbloquer (1213). L'enveloppe `AppError`
/// [`crate::retry::retry_app_on_deadlock`] rejoue alors **toute** l'opération —
/// transaction neuve, verrou de compte repris —, patron de
/// `onboarding::finalize` (KF-002-H-002, #43). ⚠️ Le prédicat porte sur
/// `AppError` : le mapping ci-dessous **préserve** `DbError::Sqlx`, sans quoi
/// le rejeu serait muet.
pub async fn post_cancel_reconciliation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<CancelReconciliationResponse>, AppError> {
    let company_id = current_user.company_id;
    let user_id = current_user.user_id;
    let actor_api_key_id = current_user.api_key_id;
    crate::retry::retry_app_on_deadlock("reconciliation::cancel", || {
        cancel_reconciliation_once(&state.pool, company_id, id, user_id, actor_api_key_id)
    })
    .await
    .map(Json)
}

/// Une tentative : transaction neuve, verrou du compte bancaire, geste, commit.
async fn cancel_reconciliation_once(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_transaction_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
) -> Result<CancelReconciliationResponse, AppError> {
    // Le compte, pour nommer le verrou — lu hors verrou, scopé par société :
    // il ne change jamais pour une transaction.
    let bank_account_id: i64 = sqlx::query_scalar(
        "SELECT bank_account_id FROM bank_transactions WHERE id = ? AND company_id = ?",
    )
    .bind(bank_transaction_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(DbError::Sqlx(e)))?
    .ok_or(AppError::Database(DbError::NotFound))?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let result = with_account_lock(
        &mut tx,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            Ok(kesh_db::repositories::reconciliation_cancel::cancel_in_tx(
                tx_inner,
                company_id,
                bank_transaction_id,
                user_id,
                actor_api_key_id,
            )
            .await?)
        },
    )
    .await;
    match result {
        Ok(done) => {
            tx.commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(CancelReconciliationResponse {
                bank_transaction: done.bank_transaction.into(),
                reversal_journal_entry_id: done.reversal_journal_entry_id,
                invoice_id: done.invoice_id,
            })
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(match e {
                ReconciliationError::AccountLocked {
                    bank_account_id,
                    timeout_secs,
                } => AppError::ReconciliationAccountLocked {
                    bank_account_id,
                    timeout_secs,
                },
                ReconciliationError::LockReleaseFailed {
                    bank_account_id, ..
                } => AppError::ReconciliationLockReleaseFailed { bank_account_id },
                // ⛔ Les deux formes d'erreur de base PRÉSERVENT `DbError::Sqlx` :
                // c'est ce que lit le prédicat du rejeu.
                ReconciliationError::Db(db) => AppError::Database(db),
                ReconciliationError::Database(e) => AppError::Database(DbError::Sqlx(e)),
                // Story 25-4-c2 — posée par `accept_batch` seulement ; défensif ici.
                ReconciliationError::TransactionAborted { .. } => {
                    AppError::ReconciliationTransactionAborted
                }
                other => {
                    tracing::error!(error = %other, "variante inattendue au dé-rapprochement");
                    AppError::Internal("internal: unexpected reconciliation error".into())
                }
            })
        }
    }
}

#[cfg(test)]
mod period_lock_tests {
    use super::*;

    /// Story 24-4c (#380) — ⛔ **un rapprochement antidaté est un refus MÉTIER,
    /// jamais une panne de base.**
    ///
    /// Sans le bras dédié, `DbError::PeriodLocked` tombait dans le repli `_` et
    /// sortait en `DATABASE_ERROR` : le client d'un `accept_batch` aurait lu une
    /// erreur d'infrastructure là où il fallait lui dire que la période est
    /// verrouillée et jusqu'à quand.
    ///
    /// ⚠️ C'est le § *Pattern batch* du `CLAUDE.md` qui l'exige — « error_code :
    /// constante canonique », jamais un repli générique — et il déclare les
    /// trois `accept_one_*` **inviolables** sur ce point. Relevé en passe 1 de
    /// revue de code, la spec ayant demandé le test sans qu'il soit écrit.
    #[test]
    fn period_locked_is_reported_as_a_business_refusal_not_a_database_error() {
        let locked_through = chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
        let attempted = chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

        let failed = project_error_to_failed_proposal(
            42,
            Some(7),
            DbError::PeriodLocked {
                locked_through,
                attempted,
            },
        );

        assert_eq!(failed.bank_transaction_id, 42);
        assert_eq!(
            failed.error_code, "PERIOD_LOCKED",
            "un repli en DATABASE_ERROR rapporterait une panne là où il y a un refus métier"
        );

        // ⛔ Les DEUX dates voyagent dans `details` : un refus qui ne dit pas
        // jusqu'où les livres sont fermés n'est pas utilisable par le client.
        let details = failed.details.expect("details attendus");
        assert_eq!(details["lockedThrough"], "2026-03-31");
        assert_eq!(details["attempted"], "2026-01-15");
        assert_eq!(
            details["projectId"], 7,
            "le contexte projet reste porté, comme pour les autres codes"
        );
    }

    /// ⛔ **Sans projet, la clé est OMISE — jamais publiée à `null`.** C'est la
    /// convention de tous les autres codes du batch, et le chemin `split`
    /// appelle précisément avec `None` : le seul code à publier une clé nulle
    /// serait celui-ci. *Relevé en passe 2 de revue, le bras neuf ayant codé
    /// `projectId` en dur au lieu de passer par la closure existante.*
    #[test]
    fn period_locked_omits_the_project_key_when_there_is_no_project() {
        let failed = project_error_to_failed_proposal(
            9,
            None,
            DbError::PeriodLocked {
                locked_through: chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
                attempted: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            },
        );
        let details = failed.details.expect("details attendus");
        assert!(
            details.get("projectId").is_none(),
            "la clé doit être ABSENTE, pas nulle : {details}"
        );
        assert_eq!(details["lockedThrough"], "2026-03-31");
    }

    /// Verrouille les deux mappages voisins, `PROJECT_NOT_FOUND` et
    /// `PROJECT_ARCHIVED`.
    ///
    /// ⚠️ **Ce test ne garde PAS l'ordre du `match`**, contrairement à ce que sa
    /// première rédaction affirmait : un bras à variante concrète
    /// (`DbError::PeriodLocked { .. }`) ne peut capturer ni `NotFound` ni
    /// `IllegalStateTransition`, quelle que soit sa position. *Vérifié par
    /// mutation en passe 2 de revue — le test principal rougit sans le bras,
    /// celui-ci passe inchangé.* Une justification fausse est ce qui fait qu'on
    /// cesse d'entretenir un test : elle est corrigée plutôt que le test retiré,
    /// car les deux mappages qu'il épingle valent d'être tenus.
    #[test]
    fn other_errors_keep_their_mapping() {
        let f = project_error_to_failed_proposal(1, None, DbError::NotFound);
        assert_eq!(f.error_code, "PROJECT_NOT_FOUND");

        let f = project_error_to_failed_proposal(
            1,
            None,
            DbError::IllegalStateTransition("le projet analytique est archivé".into()),
        );
        assert_eq!(f.error_code, "PROJECT_ARCHIVED");
    }
}
