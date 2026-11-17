//! Story 43-2 (#823) — **une transaction, plusieurs factures** : l'API du
//! règlement groupé.
//!
//! - `GET /api/v1/reconciliation/transactions/{id}/group-candidates` — les
//!   factures candidates, cherchées par texte ([`get_group_candidates`]) ;
//! - le type `invoice_group` de `POST /api/v1/reconciliation/accept`
//!   ([`accept_one_invoice_group`]), aiguillé par `reconciliation::accept_one`.
//!
//! Le règlement lui-même, ses refus et son audit vivent dans le dépôt
//! (`kesh_db::repositories::grouped_settlements`, Story 43-1a) ; ce module ne
//! fait que traduire. ⛔ **Aucune erreur d'un groupe ne sort au lot** (D2,
//! § *Pattern batch* du `CLAUDE.md`) : chaque refus devient un
//! [`FailedProposal`] par [`group_refusal_to_failed_proposal`], chaque
//! `DbError` par [`group_error_to_failed_proposal`] ; la route n'écrit aucune
//! trace d'audit (D3 : le dépôt les écrit).

use axum::Json;
use axum::extract::{Extension, Query, State};
use chrono::NaiveDate;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::bank_transaction::BankTransaction;
use kesh_db::errors::DbError;
use kesh_db::repositories::company_invoice_settings;
use kesh_db::repositories::grouped_settlements::{
    self, GroupFamily, GroupRefusal, GroupedDocument, GroupedPiece, GroupedSettlement,
};
use kesh_db::repositories::invoice_settlements::{
    amount_due_to_centime, claim_account_refusal_details,
};
use kesh_db::repositories::reconciliation::{InvoiceDateWindow, imputation_order};
use kesh_db::repositories::reconciliation_group_candidates::{
    CandidateFamily, GroupCandidate, GroupCandidatesQuery, find_group_candidates,
};
use kesh_reconciliation::MatchScore;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

use super::reconciliation::{
    AcceptedProposal, FailedProposal, SettlementDirection, bank_account_of_transaction,
    counterparty_is_bank_ledger_failed_proposal, later_fiscal_year_closed_failed_proposal,
    lettering_error_to_failed_proposal, link_preflight, non_postable_failed_proposal,
    payment_before_invoice_details, period_locked_failed_proposal, refuse_wrong_direction,
};

// ─────────────────────────────────────────────────────────────────────────────
// L'acceptation d'un groupe
// ─────────────────────────────────────────────────────────────────────────────

/// Le pré-vol d'un type groupé dans `post_accept` (M7) : un identifiant de
/// facture `≤ 0`, ou un `differenceAccountId` `≤ 0`, rend `400
/// VALIDATION_ERROR` pour le **lot**, comme les types existants. Le nombre de
/// factures et les doublons ne se contrôlent **pas** ici : une seule
/// définition, dans le dépôt, par proposition (`RECONCILIATION_GROUP_INVALID`).
pub(super) fn preflight_group(
    ids: &[i64],
    difference_account_id: Option<i64>,
    field: &str,
    kind: &str,
) -> Result<(), AppError> {
    if ids.iter().any(|id| *id <= 0) {
        return Err(AppError::Validation(format!(
            "{field} : chaque identifiant doit être strictement positif (type='{kind}')"
        )));
    }
    if difference_account_id.is_some_and(|id| id <= 0) {
        return Err(AppError::Validation(format!(
            "differenceAccountId doit être strictement positif (type='{kind}')"
        )));
    }
    Ok(())
}

/// Ce qu'un geste groupé reçoit de `accept_one`, hors identifiants des
/// factures.
#[derive(Debug, Clone, Copy)]
pub(super) struct GroupGesture {
    pub company_id: i64,
    pub bank_account_id: i64,
    pub bank_transaction_id: i64,
    pub difference_account_id: Option<i64>,
    /// La taille du lot **reçu** (trace `reconciliation.accepted`).
    pub batch_size: i64,
    pub acteur: Acteur,
    pub aujourd_hui: NaiveDate,
}

/// Une facture réglée par un groupe, dans `accepted[].settledInvoices`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettledInvoice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier_invoice_id: Option<i64>,
    /// Le montant soldé, brut, à quatre décimales.
    pub amount: String,
}

/// Les champs d'un élément d'`accepted[]` propres à un groupe (M1), aplatis
/// dans [`AcceptedProposal`].
///
/// Invariant : `difference = roundingDifference + gap` (D8 bis).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAcceptedFields {
    /// `d = |montant| − Σ apports bruts`, signé, à quatre décimales.
    pub difference: String,
    /// `r = d − e`, la fraction sous le centime passée au compte d'arrondi,
    /// signée, à quatre décimales.
    pub rounding_difference: String,
    /// `e = centime(d)`, l'écart passé au compte choisi, signé, à deux
    /// décimales (`"0.00"` dans l'arrondi automatique).
    pub gap: String,
    /// Le compte de l'écart ; `null` quand `gap` est nul.
    pub difference_account_id: Option<i64>,
    pub settled_invoices: Vec<SettledInvoice>,
}

/// L'élément d'`accepted[]` d'un groupe réglé : `invoiceId` à `0`
/// (sentinelle), `supplierInvoiceId` à `null`, score conventionnel du type
/// `rule` (total 1.0, sous-scores à 0).
fn accepted_group(bank_transaction_id: i64, done: GroupedSettlement) -> AcceptedProposal {
    let d = done.decomposition;
    AcceptedProposal {
        bank_transaction_id,
        invoice_id: 0,
        supplier_invoice_id: None,
        journal_entry_id: done.journal_entry_id,
        score: MatchScore {
            total: 1.0,
            amount_score: 0.0,
            reference_score: 0.0,
            contact_score: 0.0,
        },
        group: Some(GroupAcceptedFields {
            difference: fixed(d.difference, 4),
            rounding_difference: fixed(d.rounding_difference, 4),
            gap: fixed(d.gap, 2),
            difference_account_id: done.difference_account_id,
            settled_invoices: done
                .invoices
                .into_iter()
                .map(|r| {
                    let (invoice_id, supplier_invoice_id) = match r.kind {
                        GroupFamily::Client => (Some(r.id), None),
                        GroupFamily::Supplier => (None, Some(r.id)),
                    };
                    SettledInvoice {
                        invoice_id,
                        supplier_invoice_id,
                        amount: fixed(r.amount, 4),
                    }
                })
                .collect(),
        }),
    }
}

/// `type: "invoice_group"` — règle les factures clients `invoice_ids` par la
/// transaction au crédit, **sous le verrou du compte** pris par
/// `accept_once`, dans le point de sauvegarde de `accept_batch` (Story 43-2,
/// AC2). Rien n'est écrit pour une proposition refusée (le dépôt revient à son
/// propre point de sauvegarde avant de rendre le refus).
pub(super) async fn accept_one_invoice_group(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    g: GroupGesture,
    invoice_ids: &[i64],
) -> Result<AcceptedProposal, FailedProposal> {
    let outcome = grouped_settlements::settle_invoices_in_tx(
        tx,
        g.company_id,
        g.bank_account_id,
        g.bank_transaction_id,
        invoice_ids,
        g.difference_account_id,
        g.batch_size,
        g.acteur,
        g.aujourd_hui,
    )
    .await;
    conclude(g, SettlementDirection::Credit, outcome)
}

/// `type: "supplier_invoice_group"` (D4, commit retirable) — règle les
/// factures fournisseur `supplier_invoice_ids` par la transaction au débit,
/// symétrique de [`accept_one_invoice_group`] (Story 43-2, AC3), par
/// `grouped_settlements::pay_supplier_invoices_in_tx` (Story 43-1b).
pub(super) async fn accept_one_supplier_invoice_group(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    g: GroupGesture,
    supplier_invoice_ids: &[i64],
) -> Result<AcceptedProposal, FailedProposal> {
    let outcome = grouped_settlements::pay_supplier_invoices_in_tx(
        tx,
        g.company_id,
        g.bank_account_id,
        g.bank_transaction_id,
        supplier_invoice_ids,
        g.difference_account_id,
        g.batch_size,
        g.acteur,
        g.aujourd_hui,
    )
    .await;
    conclude(g, SettlementDirection::Debit, outcome)
}

/// La conclusion commune des deux familles : le règlement en
/// [`AcceptedProposal`], le refus et l'erreur en [`FailedProposal`].
fn conclude(
    g: GroupGesture,
    direction: SettlementDirection,
    outcome: Result<Result<GroupedSettlement, GroupRefusal>, DbError>,
) -> Result<AcceptedProposal, FailedProposal> {
    match outcome {
        Ok(Ok(done)) => Ok(accepted_group(g.bank_transaction_id, done)),
        Ok(Err(refusal)) => Err(group_refusal_to_failed_proposal(
            g.bank_transaction_id,
            direction,
            refusal,
        )),
        Err(err) => Err(group_error_to_failed_proposal(g.bank_transaction_id, err)),
    }
}

/// D2 — une erreur de base d'un groupe en [`FailedProposal`] : **délégation**
/// à `lettering_error_to_failed_proposal`, même classement que l'unitaire —
/// `INTERNAL_ERROR` (après `tracing::error!`) pour `Invariant`, `NotFound` et
/// les refus de lettrage exclus par construction, `DATABASE_ERROR` avec
/// `details.message` pour le reste (un interblocage est rejoué par le lot).
/// Jamais une `AppError`.
pub(super) fn group_error_to_failed_proposal(
    bank_transaction_id: i64,
    err: DbError,
) -> FailedProposal {
    lettering_error_to_failed_proposal(bank_transaction_id, err)
}

/// `invoiceId` ou `supplierInvoiceId`, selon la famille de la pièce.
fn document_key(document: GroupedDocument) -> &'static str {
    match document {
        GroupedDocument::Invoice(_) => "invoiceId",
        GroupedDocument::SupplierInvoice(_) => "supplierInvoiceId",
    }
}

/// Ajoute à `details` l'identifiant de la pièce (`invoiceId` ou
/// `supplierInvoiceId`) et son numéro affichable (`invoiceNumber`).
fn with_piece(mut details: serde_json::Value, piece: &GroupedPiece) -> serde_json::Value {
    if let Some(obj) = details.as_object_mut() {
        obj.insert(
            document_key(piece.document).to_string(),
            serde_json::json!(piece.document.id()),
        );
        obj.insert("invoiceNumber".to_string(), serde_json::json!(piece.number));
    }
    details
}

/// D2, M8 — **la** traduction d'un refus d'un groupe en [`FailedProposal`] :
/// `match` exhaustif sans joker sur [`GroupRefusal`] (une variante ajoutée
/// sans sa ligne ne compile pas). Les constructeurs de l'unitaire sont
/// **appelés**, non recopiés ; chaque code neuf est écrit au site de
/// construction (`error_code: "…"`), pour le décompte de
/// `failed-proposal-label.ts`.
///
/// `direction` : le sens que la famille attend (refus `WrongDirection`).
pub(super) fn group_refusal_to_failed_proposal(
    bank_transaction_id: i64,
    direction: SettlementDirection,
    refusal: GroupRefusal,
) -> FailedProposal {
    let fail = |error_code: &str, details: Option<serde_json::Value>| FailedProposal {
        bank_transaction_id,
        error_code: error_code.to_string(),
        details,
    };
    match refusal {
        GroupRefusal::TooFew { count } => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_GROUP_INVALID".to_string(),
            details: Some(serde_json::json!({ "reason": "too_few", "count": count })),
        },
        GroupRefusal::TooMany { count } => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_GROUP_INVALID".to_string(),
            details: Some(serde_json::json!({ "reason": "too_many", "count": count })),
        },
        GroupRefusal::Duplicate { document } => {
            let mut details = serde_json::json!({ "reason": "duplicate" });
            details[document_key(document)] = serde_json::json!(document.id());
            FailedProposal {
                bank_transaction_id,
                error_code: "RECONCILIATION_GROUP_INVALID".to_string(),
                details: Some(details),
            }
        }
        GroupRefusal::DifferenceAccountRequired { gap } => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_GROUP_INVALID".to_string(),
            details: Some(serde_json::json!({
                "reason": "difference_account_required",
                "gap": fixed(gap, 2),
            })),
        },
        GroupRefusal::DifferenceTooLarge {
            transaction_amount,
            sum_applied,
            gap,
            max_difference,
        } => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE".to_string(),
            details: Some(serde_json::json!({
                "transactionAmount": fixed(transaction_amount, 2),
                "sumApplied": fixed(sum_applied, 4),
                "gap": fixed(gap, 2),
                "maxDifference": fixed(max_difference, 2),
            })),
        },
        GroupRefusal::CurrencyMismatch { currency } => fail(
            "RECONCILIATION_CURRENCY_MISMATCH",
            Some(serde_json::json!({
                "transactionCurrency": currency,
                "expected": "CHF",
            })),
        ),
        GroupRefusal::TransactionNotFound => fail("BANK_TRANSACTION_NOT_FOUND", None),
        GroupRefusal::TransactionNotPending => fail("RECONCILIATION_ALREADY_RECONCILED", None),
        GroupRefusal::WrongDirection { amount } => {
            refuse_wrong_direction(bank_transaction_id, amount, direction).unwrap_or_else(|| {
                // Le dépôt a refusé le sens, la garde de la route l'admet :
                // défaut structurel, jamais un refus métier inventé.
                tracing::error!(
                    %amount,
                    "règlement groupé : sens refusé par le dépôt, admis par la route"
                );
                fail("INTERNAL_ERROR", None)
            })
        }
        GroupRefusal::InvoiceNotFound { document } => {
            let code = match document {
                GroupedDocument::Invoice(_) => "INVOICE_NOT_FOUND",
                GroupedDocument::SupplierInvoice(_) => "SUPPLIER_INVOICE_NOT_FOUND",
            };
            let mut details = serde_json::json!({});
            details[document_key(document)] = serde_json::json!(document.id());
            fail(code, Some(details))
        }
        GroupRefusal::InvoiceNotEligible { piece, reason } => fail(
            "RECONCILIATION_INVOICE_NOT_ELIGIBLE",
            Some(with_piece(
                serde_json::json!({ "reason": reason.code() }),
                &piece,
            )),
        ),
        GroupRefusal::PaymentBeforeInvoiceDate {
            piece,
            tolerance_days,
        } => fail(
            "RECONCILIATION_INVOICE_NOT_ELIGIBLE",
            Some(with_piece(
                payment_before_invoice_details(tolerance_days),
                &piece,
            )),
        ),
        GroupRefusal::HeldByPaymentBatch {
            piece,
            payment_batch_id,
        } => fail(
            "RECONCILIATION_SUPPLIER_PAYMENT_HOLD",
            Some(with_piece(
                serde_json::json!({
                    "reason": "other_invoice_in_payment_batch",
                    "paymentBatchId": payment_batch_id,
                }),
                &piece,
            )),
        ),
        GroupRefusal::SaleEntryMalformed {
            piece,
            sale_entry_id,
        } => fail(
            "INVOICE_SALE_ENTRY_MALFORMED",
            Some(with_piece(
                serde_json::json!({
                    "reason": "no_debit_line_on_sale_entry",
                    "saleEntryId": sale_entry_id,
                }),
                &piece,
            )),
        ),
        GroupRefusal::BankAccountNotConfigured => fail("BANK_ACCOUNT_NOT_CONFIGURED", None),
        GroupRefusal::BankAccountNotFound => fail("BANK_ACCOUNT_NOT_FOUND", None),
        GroupRefusal::BankAccountClosed => fail("BANK_ACCOUNT_CLOSED", None),
        GroupRefusal::ClaimAccount {
            claim,
            role,
            account_id,
            account_number,
            bank_account_id,
            piece,
        } => {
            // D9 — les deux causes de `role: "counterparty"` : le compte de
            // tiers d'une facture égal à la banque (`bankAccountId` présent),
            // ou le compte de l'écart égal à un compte de tiers (`accountId` =
            // `differenceAccountId` envoyé, sans `bankAccountId`).
            let details = claim_account_refusal_details(
                account_id,
                account_number.as_deref(),
                claim,
                role,
                bank_account_id,
            );
            let details = match &piece {
                Some(piece) => with_piece(details, piece),
                None => details,
            };
            fail("SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT", Some(details))
        }
        GroupRefusal::DifferenceAccountIsBankLedger { .. } => {
            counterparty_is_bank_ledger_failed_proposal(bank_transaction_id)
        }
        GroupRefusal::DifferenceAccountInvalid { account_id } => fail(
            "ACCOUNT_NOT_FOUND",
            Some(serde_json::json!({ "missingAccountIds": [account_id] })),
        ),
        GroupRefusal::DifferenceAccountNotPostable {
            account_id,
            account_number,
        } => non_postable_failed_proposal(
            bank_transaction_id,
            vec![kesh_db::errors::NonPostableAccount {
                account_id,
                account_number,
            }],
        ),
        GroupRefusal::RoundingAccountNotConfigured => fail("ROUNDING_ACCOUNT_NOT_CONFIGURED", None),
        GroupRefusal::PeriodLocked {
            locked_through,
            attempted,
        } => period_locked_failed_proposal(bank_transaction_id, None, locked_through, attempted),
        GroupRefusal::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        } => later_fiscal_year_closed_failed_proposal(
            bank_transaction_id,
            None,
            fiscal_year_id,
            &fiscal_year_name,
        ),
        GroupRefusal::FiscalYearInvalid { value_date } => fail(
            "FISCAL_YEAR_INVALID",
            Some(serde_json::json!({ "valueDate": value_date })),
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// La vue et l'annulation (M5)
// ─────────────────────────────────────────────────────────────────────────────

/// Une facture d'un groupe dans la vue (`groupedInvoices`) : montant soldé
/// brut, à quatre décimales.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedInvoiceResponse {
    /// `invoice` ou `supplier_invoice`.
    pub kind: &'static str,
    pub id: i64,
    pub number: Option<String>,
    pub amount: String,
}

/// Une pièce nommée (`cancelBlockedInvoice`, `cancelledInvoices[]`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedPieceResponse {
    /// `invoice` ou `supplier_invoice`.
    pub kind: &'static str,
    pub id: i64,
    pub number: Option<String>,
}

impl GroupedPieceResponse {
    fn from_piece(piece: &GroupedPiece) -> Self {
        Self {
            kind: piece.document.family().code(),
            id: piece.document.id(),
            number: piece.number.clone(),
        }
    }

    /// Une facture d'un groupe défait (`cancelledInvoices[]`).
    pub(super) fn from_view(v: grouped_settlements::GroupedViewInvoice) -> Self {
        Self {
            kind: v.kind.code(),
            id: v.id,
            number: v.number,
        }
    }
}

/// Un compte nommé (`differenceAccount`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRef {
    pub id: i64,
    pub number: String,
    pub name: String,
}

/// Les champs de groupe de `GET /reconciliation/transactions/{id}`.
pub(super) struct GroupViewFields {
    pub grouped_invoices: Vec<GroupedInvoiceResponse>,
    pub gap: Option<String>,
    pub difference_account: Option<AccountRef>,
    pub cancel_blocked_invoice: Option<GroupedPieceResponse>,
}

/// Les champs de groupe de la vue, **chacun à sa source** (M5) :
/// - `groupedInvoices` ← `view.grouped_invoices` (dépôt) ;
/// - `gap` ← **recalculé** par `group_difference(montant, <apports bruts des
///   factures du groupe>, Decimal::MAX).gap`, sur les montants du règlement —
///   jamais sur les réglages, qui ont pu changer depuis ;
/// - `differenceAccount` ← `view.grouped_difference_account_id` (relu par le
///   dépôt sur la ligne d'écart), numéro et nom lus du compte, société de la
///   requête ;
/// - `cancelBlockedInvoice` ← `view.cancel_blocked_invoice`.
///
/// Hors groupe : liste vide et `null` partout.
pub(super) async fn group_view_fields(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    view: &kesh_db::repositories::reconciliation_cancel::ReconciliationView,
) -> Result<GroupViewFields, AppError> {
    use kesh_db::repositories::reconciliation_cancel::ReconciliationKind;
    let cancel_blocked_invoice = view
        .cancel_blocked_invoice
        .as_ref()
        .map(GroupedPieceResponse::from_piece);
    if !matches!(
        view.kind,
        Some(ReconciliationKind::GroupedSettlement { .. })
    ) {
        return Ok(GroupViewFields {
            grouped_invoices: Vec::new(),
            gap: None,
            difference_account: None,
            cancel_blocked_invoice,
        });
    }
    let amounts: Vec<Decimal> = view.grouped_invoices.iter().map(|i| i.amount).collect();
    let gap =
        grouped_settlements::group_difference(view.bank_transaction.amount, &amounts, Decimal::MAX)
            .map(|d| fixed(d.gap, 2))
            .ok();
    let difference_account = match view.grouped_difference_account_id {
        Some(id) => kesh_db::repositories::accounts::find_by_id_in_company(pool, id, company_id)
            .await?
            .map(|a| AccountRef {
                id: a.id,
                number: a.number,
                name: a.name,
            }),
        None => None,
    };
    Ok(GroupViewFields {
        grouped_invoices: view
            .grouped_invoices
            .iter()
            .map(|i| GroupedInvoiceResponse {
                kind: i.kind.code(),
                id: i.id,
                number: i.number.clone(),
                amount: fixed(i.amount, 4),
            })
            .collect(),
        gap,
        difference_account,
        cancel_blocked_invoice,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// La recherche des candidates
// ─────────────────────────────────────────────────────────────────────────────

/// Le nombre de candidates rendues par défaut.
pub const DEFAULT_GROUP_CANDIDATES_LIMIT: usize = 50;

/// Le nombre de candidates rendues au plus (`limit` au-delà est ramené ici).
pub const MAX_GROUP_CANDIDATES_LIMIT: usize = 100;

/// Le nombre d'identifiants que `exclude` admet au plus.
pub const MAX_GROUP_CANDIDATES_EXCLUDE: usize = 50;

/// Un montant à `dp` décimales exactement, demi arrondi en s'éloignant de zéro
/// (`"500.01"`, `"500.0050"`), un zéro négatif écrit sans signe.
pub(crate) fn fixed(amount: Decimal, dp: u32) -> String {
    let mut d = amount.round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero);
    if d.is_zero() {
        d = Decimal::ZERO;
    }
    d.rescale(dp);
    d.to_string()
}

/// Les paramètres de `GET …/group-candidates`.
#[derive(Debug, Deserialize)]
pub struct GroupCandidatesParams {
    /// Le terme de recherche (taillé) ; absent ou vide : aucun filtre.
    pub q: Option<String>,
    /// Le nombre de candidates rendues : 50 par défaut, ramené à 100 au plus ;
    /// une valeur `≤ 0` vaut le défaut (patron de `get_proposals`).
    pub limit: Option<i64>,
    /// Les factures à retirer, `12,15` ; 50 identifiants au plus, chacun `> 0`.
    pub exclude: Option<String>,
}

/// Une facture candidate, telle que l'API la rend (`kind` en snake_case).
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GroupCandidateResponse {
    #[serde(rename_all = "camelCase")]
    Invoice {
        id: i64,
        number: Option<String>,
        contact_name: String,
        date: NaiveDate,
        due_date: Option<NaiveDate>,
        /// Le TTC, au centime.
        total_ttc: String,
        /// Le reste dû au centime (affichage).
        amount_due: String,
        /// Le reste dû brut, à quatre décimales (calcul de l'écart).
        amount_due_raw: String,
    },
    #[serde(rename_all = "camelCase")]
    SupplierInvoice {
        id: i64,
        internal_number: Option<String>,
        supplier_invoice_number: Option<String>,
        contact_name: String,
        date: NaiveDate,
        due_date: Option<NaiveDate>,
        payment_reference: Option<String>,
        /// Le TTC au centime.
        amount_due: String,
        /// Le TTC brut, à quatre décimales.
        amount_due_raw: String,
    },
}

impl GroupCandidateResponse {
    fn from_candidate(family: CandidateFamily, c: GroupCandidate) -> Self {
        let amount_due = fixed(amount_due_to_centime(c.amount_due), 2);
        let amount_due_raw = fixed(c.amount_due, 4);
        match family {
            CandidateFamily::Client => Self::Invoice {
                id: c.id,
                number: c.invoice_number,
                contact_name: c.contact_name,
                date: c.date,
                due_date: c.due_date,
                total_ttc: fixed(amount_due_to_centime(c.total_ttc), 2),
                amount_due,
                amount_due_raw,
            },
            CandidateFamily::Supplier => Self::SupplierInvoice {
                id: c.id,
                internal_number: c.internal_number,
                supplier_invoice_number: c.supplier_invoice_number,
                contact_name: c.contact_name,
                date: c.date,
                due_date: c.due_date,
                payment_reference: c.payment_reference,
                amount_due,
                amount_due_raw,
            },
        }
    }
}

/// La famille cherchée : `client` pour un crédit, `supplier` pour un débit.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroupDirection {
    Client,
    Supplier,
}

/// Réponse de `GET …/group-candidates`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCandidatesResponse {
    /// `null` pour un montant nul ou une transaction hors CHF : la liste est
    /// alors vide.
    pub direction: Option<GroupDirection>,
    /// Le montant de la transaction, **signé**, à deux décimales.
    pub transaction_amount: String,
    /// Le plafond de l'écart de la société, à deux décimales (le réglage
    /// `group_settlement_max_difference`, 5.00 par défaut).
    pub max_difference: String,
    pub candidates: Vec<GroupCandidateResponse>,
    /// Vrai si la troncature à `limit` a coupé, ou si le plafond de lecture
    /// (`GROUP_CANDIDATES_SCAN_LIMIT`) a été dépassé.
    pub has_more: bool,
}

/// `exclude=12,15` → les identifiants ; vide ou absent → aucun.
fn parse_exclude(raw: Option<&str>) -> Result<Vec<i64>, AppError> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(Vec::new());
    };
    let ids = raw
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<i64>()
                .ok()
                .filter(|id| *id > 0)
                .ok_or_else(|| {
                    AppError::Validation(
                        "exclude : identifiants de factures strictement positifs, séparés par des virgules"
                            .into(),
                    )
                })
        })
        .collect::<Result<Vec<i64>, AppError>>()?;
    if ids.len() > MAX_GROUP_CANDIDATES_EXCLUDE {
        return Err(AppError::Validation(format!(
            "exclude : {MAX_GROUP_CANDIDATES_EXCLUDE} identifiants au plus"
        )));
    }
    Ok(ids)
}

/// La famille et la borne de date d'une transaction, ou `None` si aucune
/// facture ne peut la régler : montant nul, ou devise autre que CHF (la règle
/// de `transaction_can_be_linked_to_entry` et de la proposition).
///
/// Un **débit** cherche des factures fournisseur (D4, commit retirable : sans
/// lui, un débit ne rend rien, la liste n'offrant jamais ce que le `POST`
/// refuserait).
fn family_of(t: &BankTransaction) -> Option<(CandidateFamily, NaiveDate)> {
    if t.currency != "CHF" {
        return None;
    }
    if t.amount > Decimal::ZERO {
        Some((
            CandidateFamily::Client,
            InvoiceDateWindow::for_transaction(t).latest_invoice_date,
        ))
    } else if t.amount < Decimal::ZERO {
        Some((
            CandidateFamily::Supplier,
            InvoiceDateWindow::for_supplier_transaction(t).latest_invoice_date,
        ))
    } else {
        None
    }
}

/// `GET /api/v1/reconciliation/transactions/{id}/group-candidates` — les
/// factures qu'une transaction peut régler ensemble (Comptable+, lecture,
/// clés d'API en lecture admises ; Story 43-2, AC5).
///
/// Ordre des refus, celui de `get_link_candidates` : `404 NOT_FOUND`
/// (transaction inexistante ou d'une autre société) → pré-vol
/// (`link_preflight` : compte archivé, `409 BANK_ACCOUNT_CLOSED`, `412
/// BANK_ACCOUNT_NOT_CONFIGURED`) → `404 RECONCILIATION_TRANSACTION_NOT_PENDING`
/// (rapprochée). Une transaction **rejetée** est servie, comme par la route
/// sœur. Un montant nul ou une devise autre que CHF rend `200`,
/// `direction: null` et une liste vide.
///
/// Crédit → factures clients, débit → factures fournisseur, **tous tiers** ;
/// critères et recherche : `kesh_db::repositories::reconciliation_group_candidates`.
/// Au plus `GROUP_CANDIDATES_SCAN_LIMIT` factures sont lues (date puis
/// identifiant), triées ici par l'ordre d'imputation, puis tronquées à
/// `limit`.
pub async fn get_group_candidates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Query(params): Query<GroupCandidatesParams>,
) -> Result<Json<GroupCandidatesResponse>, AppError> {
    let exclude = parse_exclude(params.exclude.as_deref())?;
    let limit = params
        .limit
        .filter(|&l| l > 0)
        .map(|l| usize::try_from(l).unwrap_or(usize::MAX))
        .map(|l| l.min(MAX_GROUP_CANDIDATES_LIMIT))
        .unwrap_or(DEFAULT_GROUP_CANDIDATES_LIMIT);
    let company_id = current_user.company_id;
    let bank_account_id = bank_account_of_transaction(&state.pool, company_id, id).await?;
    link_preflight(&state.pool, company_id, bank_account_id).await?;
    let t = kesh_db::repositories::reconciliation::find_pending_by_id_for_account(
        &state.pool,
        company_id,
        bank_account_id,
        id,
    )
    .await?
    .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;
    if kesh_db::repositories::bank_imports::is_reconciled(t.status.as_str(), t.matched_entry_id) {
        return Err(AppError::ReconciliationTransactionNotPending {
            bank_transaction_id: id,
        });
    }
    let max_difference =
        company_invoice_settings::group_settlement_max_difference(&state.pool, company_id).await?;
    let mut response = GroupCandidatesResponse {
        direction: None,
        transaction_amount: fixed(t.amount, 2),
        max_difference: fixed(max_difference, 2),
        candidates: Vec::new(),
        has_more: false,
    };
    let Some((family, latest_invoice_date)) = family_of(&t) else {
        return Ok(Json(response));
    };
    let found = find_group_candidates(
        &state.pool,
        GroupCandidatesQuery {
            company_id,
            family,
            latest_invoice_date,
            q: params.q.as_deref(),
            exclude: &exclude,
        },
    )
    .await?;
    let mut candidates = found.candidates;
    candidates.sort_by(imputation_order);
    let truncated = candidates.len() > limit;
    candidates.truncate(limit);
    response.direction = Some(match family {
        CandidateFamily::Client => GroupDirection::Client,
        CandidateFamily::Supplier => GroupDirection::Supplier,
    });
    response.has_more = truncated || found.scan_limit_reached;
    response.candidates = candidates
        .into_iter()
        .map(|c| GroupCandidateResponse::from_candidate(family, c))
        .collect();
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn fixed_writes_the_requested_scale() {
        assert_eq!(fixed(dec!(500.005), 2), "500.01");
        assert_eq!(fixed(dec!(-500.005), 2), "-500.01");
        assert_eq!(fixed(dec!(10.005), 4), "10.0050");
        assert_eq!(fixed(dec!(5), 2), "5.00");
        assert_eq!(fixed(dec!(-0.00), 2), "0.00");
        assert_eq!(fixed(dec!(5.0000), 2), "5.00");
    }

    /// AC2, D2 (L3) — une erreur de base d'un groupe se classe comme dans
    /// l'unitaire : un refus de lettrage exclu par construction et un
    /// `Invariant` en `INTERNAL_ERROR` sans détail, une erreur SQL en
    /// `DATABASE_ERROR` avec son message — jamais une `AppError`.
    #[test]
    fn a_database_error_is_classified_like_the_unit_settlement() {
        let lettered = group_error_to_failed_proposal(
            7,
            DbError::LetteringLineAlreadyLettered { code: "L-3".into() },
        );
        assert_eq!(lettered.bank_transaction_id, 7);
        assert_eq!(lettered.error_code, "INTERNAL_ERROR");
        assert!(lettered.details.is_none());
        let invariant =
            group_error_to_failed_proposal(7, DbError::Invariant("reste dû > 0".into()));
        assert_eq!(invariant.error_code, "INTERNAL_ERROR");
        let sql = group_error_to_failed_proposal(7, DbError::Sqlx(sqlx::Error::PoolTimedOut));
        assert_eq!(sql.error_code, "DATABASE_ERROR");
        assert!(sql.details.unwrap()["message"].is_string());
    }

    /// D2 (mutation a) — la conclusion d'un geste groupé rend une `DbError`
    /// dans `failed[]`, jamais une panique ni une erreur du lot ; un refus
    /// aussi, avec le sens de la famille.
    #[test]
    fn the_conclusion_keeps_errors_in_the_proposal() {
        let g = GroupGesture {
            company_id: 1,
            bank_account_id: 2,
            bank_transaction_id: 41,
            difference_account_id: None,
            batch_size: 1,
            acteur: Acteur {
                user_id: 1,
                api_key_id: None,
            },
            aujourd_hui: NaiveDate::from_ymd_opt(2026, 11, 17).unwrap(),
        };
        let failed = conclude(
            g,
            SettlementDirection::Credit,
            Err(DbError::Sqlx(sqlx::Error::PoolTimedOut)),
        )
        .expect_err("une erreur de base est un refus de la proposition");
        assert_eq!(failed.bank_transaction_id, 41);
        assert_eq!(failed.error_code, "DATABASE_ERROR");
        let refused = conclude(
            g,
            SettlementDirection::Credit,
            Ok(Err(GroupRefusal::WrongDirection { amount: dec!(-5) })),
        )
        .expect_err("refus");
        assert_eq!(refused.error_code, "RECONCILIATION_TRANSACTION_NOT_CREDIT");
        assert_eq!(refused.details.unwrap()["reason"], "debit_transaction");
    }

    /// D2, M8 — quelques lignes de la table : le code, et la pièce nommée.
    #[test]
    fn refusals_carry_their_code_and_piece() {
        let piece = GroupedPiece {
            document: GroupedDocument::Invoice(12),
            number: Some("F-12".into()),
        };
        let f = group_refusal_to_failed_proposal(
            1,
            SettlementDirection::Credit,
            GroupRefusal::InvoiceNotEligible {
                piece: piece.clone(),
                reason: grouped_settlements::IneligibleReason::AlreadyPaid,
            },
        );
        assert_eq!(f.error_code, "RECONCILIATION_INVOICE_NOT_ELIGIBLE");
        let d = f.details.unwrap();
        assert_eq!(d["reason"], "invoice_already_paid");
        assert_eq!(d["invoiceId"], 12);
        assert_eq!(d["invoiceNumber"], "F-12");

        let f = group_refusal_to_failed_proposal(
            1,
            SettlementDirection::Credit,
            GroupRefusal::Duplicate {
                document: GroupedDocument::SupplierInvoice(9),
            },
        );
        assert_eq!(f.error_code, "RECONCILIATION_GROUP_INVALID");
        assert_eq!(f.details.unwrap()["supplierInvoiceId"], 9);

        let f = group_refusal_to_failed_proposal(
            1,
            SettlementDirection::Credit,
            GroupRefusal::DifferenceTooLarge {
                transaction_amount: dec!(801.01),
                sum_applied: dec!(800.0000),
                gap: dec!(1.01),
                max_difference: dec!(1.0000),
            },
        );
        assert_eq!(f.error_code, "RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE");
        assert_eq!(
            f.details.unwrap(),
            serde_json::json!({
                "transactionAmount": "801.01",
                "sumApplied": "800.0000",
                "gap": "1.01",
                "maxDifference": "1.00",
            })
        );

        let f = group_refusal_to_failed_proposal(
            1,
            SettlementDirection::Debit,
            GroupRefusal::WrongDirection { amount: dec!(5) },
        );
        assert_eq!(f.error_code, "RECONCILIATION_TRANSACTION_NOT_DEBIT");
        assert_eq!(f.details.unwrap()["reason"], "credit_transaction");
    }

    #[test]
    fn exclude_is_parsed_and_bounded() {
        assert_eq!(parse_exclude(None).unwrap(), Vec::<i64>::new());
        assert_eq!(parse_exclude(Some(" ")).unwrap(), Vec::<i64>::new());
        assert_eq!(parse_exclude(Some("12, 15")).unwrap(), vec![12, 15]);
        assert!(parse_exclude(Some("12,x")).is_err());
        assert!(parse_exclude(Some("12,0")).is_err());
        assert!(parse_exclude(Some("12,,15")).is_err());
        let many: Vec<String> = (1..=51).map(|i| i.to_string()).collect();
        assert!(parse_exclude(Some(&many.join(","))).is_err());
        let fifty: Vec<String> = (1..=50).map(|i| i.to_string()).collect();
        assert_eq!(parse_exclude(Some(&fifty.join(","))).unwrap().len(), 50);
    }
}
