//! **Moteur commun** de la revue et de l'acceptation d'un compte de
//! prestataire de paiement — Story 38-11c2 (AC-5 à AC-9), servi tel quel à
//! PayPal (`routes::paypal`), Wise (39-11c) et Stripe (40-7c).
//!
//! ⛔ **Aucun nom de prestataire et aucune route ici** (NFR-38-11c2-3) : chaque
//! prestataire a son module de routes, mince, qui passe son paramétrage
//! ([`ProviderSpec`]) aux fonctions de ce module. Les codes d'erreur propres au
//! moteur sont génériques (`PROVIDER_*`).
//!
//! # Revue ([`load_review_in_tx`], [`review`])
//!
//! Lectures ordinaires dans **une** transaction de lecture, puis
//! [`build_review`] (pur). Refus : `404 PROVIDER_ACCOUNT_NOT_FOUND`, `404
//! FISCAL_YEAR_NOT_FOUND`. Rien n'est écrit.
//!
//! # Acceptation ([`prepare_accept`], [`accept_once`], [`PreparedAccept::finish`])
//!
//! 1. **Pré-vol**, sur le pool, hors verrou ([`prepare_accept`]) : `400` global
//!    pour un corps vide, de plus de `MAX_IN_CLAUSE_IDS` propositions, portant
//!    deux fois la même transaction (les deux d'une conversion comptent) ou
//!    mal formé ; `404 PROVIDER_ACCOUNT_NOT_FOUND`. Les transactions inconnues,
//!    d'une autre société ou hors du compte de prestataire vont en `failed[]`
//!    (`BANK_TRANSACTION_NOT_FOUND`). Les comptes bancaires à verrouiller sont
//!    ceux des transactions restantes, plus les comptes supplémentaires que
//!    déclare le prestataire.
//! 2. **Une tentative** ([`accept_once`]) : une transaction de base,
//!    `with_account_locks` **sur cette transaction** (identifiants croissants),
//!    relecture des réglages du prestataire sous les verrous, puis un **point de
//!    sauvegarde par proposition** ; chaque proposition **rejuge** son
//!    classement sous les verrous (sinon `failed[]`
//!    [`PROVIDER_PROPOSAL_STALE`]) et écrit. Commit, verrous rendus.
//! 3. Le gestionnaire du prestataire enveloppe la tentative dans
//!    `crate::retry::retry_app_on_deadlock` (jamais `retry_with`, réservée à
//!    `reconciliation::post_accept`).
//!
//! ⚠️ **Interblocage, piège du patron par proposition.** Un 1213 levé *dans*
//! une proposition a annulé **toute** la transaction (InnoDB) : le ranger en
//! `failed[]` puis faire `ROLLBACK TO SAVEPOINT` échouerait en 1305, que
//! `retry_app_on_deadlock` ne rejoue pas. Toute erreur de base d'une
//! proposition passe donc par [`db_refusal`], qui teste
//! `kesh_db::retry::is_deadlock_error` **avant** tout `ROLLBACK TO SAVEPOINT`
//! et la rend [`Refusal::Abort`] ; [`run_batch`] la remonte aussitôt en
//! `ReconciliationError::Db(DbError::Sqlx(_))`, traverse `with_account_locks`,
//! et [`accept_once`] la rend `AppError::Database(DbError::Sqlx(_))`, que
//! `is_app_deadlock` reconnaît : l'enveloppe rejoue toute l'opération, verrous
//! repris ; **jamais** un interblocage dans `failed[]`. Contrepartie : sans le
//! filet du 1305 de `post_accept`, un interblocage converti en refus avant ce
//! test finirait en 500 au `ROLLBACK TO SAVEPOINT` (test d'interblocage par
//! genre, `paypal_reconciliation_e2e.rs`).
//!
//! # Écritures ([`write_provider_entry`])
//!
//! Journal Banque, date de la transaction, deux lignes : compte du plan du
//! compte de prestataire et contrepartie, sens selon le signe. La ligne du
//! compte de prestataire est **en devise** si son compte du plan en a une, au
//! **cours mensuel retenu** par la société pour le mois (cours absent →
//! `RATE_UNAVAILABLE`) ; la contrepartie porte le même CHF, et, si elle est de
//! la même devise, le même montant en devise et le même cours. Aucun code TVA.
//! Après l'écriture, la transaction passe `reconciled` par
//! `reconciliation::mark_reconciled_in_tx`. Une trace
//! `provider.proposal_accepted` par proposition acceptée, sous l'acteur.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use kesh_core::exchange_rates::{RateKind, first_of_month};
use kesh_db::entities::audit_log::{Acteur, NewAuditLogEntry};
use kesh_db::entities::bank_transaction::BankTransaction;
use kesh_db::entities::{
    ForeignAmount, Journal, NewJournalEntry, NewJournalEntryLine, RateProvenance,
};
use kesh_db::errors::{DbError, FxAccountField, FxAccountReason};
use kesh_db::repositories::bank_accounts::{self, BASE_CURRENCY, ProviderBalance};
use kesh_db::repositories::exchange_rates::find_company_rate_in_tx;
use kesh_db::repositories::journal_entry_conversions::{
    ConversionErrorKind, create_provider_conversion_in_tx,
};
use kesh_db::repositories::reconciliation_link::{LinkEntryRequest, LinkRefusal, link_entry_in_tx};
use kesh_db::repositories::{
    audit_log, fiscal_years, journal_entries, projects, provider_review as reads,
    reconciliation as reconciliation_repo, reconciliation_rules,
};
use kesh_db::retry::is_deadlock_error;
use kesh_reconciliation::conversion::{ConversionSide, PlanError, SourceRate, plan_conversion};
use kesh_reconciliation::provider_review::{
    EntryKind, Proposal, ProviderAccount, ProviderKind, ProviderSpec, Review, ReviewFinding,
    ReviewInput, ReviewProposal, SaleEntry, SaleMovement, build_review, conversion_halves, counts,
    payment_date,
};
use kesh_reconciliation::{ReconciliationError, rule_matches, with_account_locks};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::MySql;

use super::reconciliation::{
    FailedProposal, closed_account_failed, counterparty_is_bank_ledger_failed_proposal,
    fx_failed_proposal, later_fiscal_year_closed_failed_proposal, non_postable_failed_proposal,
    period_locked_failed_proposal, project_error_to_failed_proposal,
};
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Code générique d'une proposition **périmée** : son classement ne tient plus
/// sous les verrous (transaction plus en attente, autre genre, autre écriture
/// ou paire, règle qui ne correspond plus, réglages du prestataire changés).
/// Servi tel quel à Wise et Stripe (Q-39-10).
pub const PROVIDER_PROPOSAL_STALE: &str = "PROVIDER_PROPOSAL_STALE";

/// Délai d'attente du verrou nommé de chaque compte, en secondes (celui de la
/// réconciliation).
const LOCK_TIMEOUT_SECS: u32 = 5;

/// Borne du libellé d'une écriture, en caractères (celle des écritures de la
/// réconciliation, `accept_one_rule`).
const DESCRIPTION_MAX_CHARS: usize = 200;

/// Action d'audit d'une proposition acceptée.
pub const AUDIT_PROPOSAL_ACCEPTED: &str = "provider.proposal_accepted";

// ---------------------------------------------------------------------------
// Revue
// ---------------------------------------------------------------------------

/// Paramètres de `GET /api/v1/<prestataire>/review`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewQuery {
    /// Libellé du compte du prestataire.
    pub account: String,
    /// Exercice revu.
    pub fiscal_year_id: i64,
}

/// Réponse de la revue — la forme que reprennent Wise et Stripe.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResponse {
    /// Libellé du compte, tel que demandé.
    pub account: String,
    /// Exercice revu.
    pub fiscal_year_id: i64,
    /// Propositions.
    pub proposals: Vec<ReviewProposal>,
    /// Écarts.
    pub gaps: Vec<ReviewFinding>,
    /// Lignes à examiner.
    pub to_review: Vec<ReviewFinding>,
    /// Décomptes.
    pub counts: BTreeMap<&'static str, usize>,
}

impl ReviewResponse {
    /// La réponse d'une revue (triée par l'appelant, [`Review::sort`]).
    pub fn of(account: String, fiscal_year_id: i64, review: Review) -> Self {
        let counts = counts(&review);
        Self {
            account,
            fiscal_year_id,
            proposals: review.proposals,
            gaps: review.gaps,
            to_review: review.to_review,
            counts,
        }
    }
}

/// Ce qu'a lu une revue, rendu au prestataire avec elle pour qu'il décide ses
/// genres propres ([`Review::specific`]).
#[derive(Debug)]
pub struct LoadedReview {
    /// La revue commune.
    pub review: Review,
    /// Les comptes bancaires du compte de prestataire.
    pub balances: Vec<ProviderBalance>,
    /// Les transactions pending revues.
    pub pending: Vec<BankTransaction>,
}

/// Les comptes du prestataire, en paramètre de la revue pure.
fn provider_accounts(balances: &[ProviderBalance]) -> Vec<ProviderAccount> {
    balances
        .iter()
        .map(|b| ProviderAccount {
            bank_account_id: b.account.id,
            ledger_account_id: b.account.journal_account_id,
            currency: b.currency().to_string(),
            fee_account_id: b.account.provider_settings.fee_account_id,
        })
        .collect()
}

/// Les écritures de vente (AC-5) à partir des lignes lues : regroupées par
/// écriture, gardées si l'une de leurs lignes est au débit d'un des comptes du
/// plan, mouvement compté dans la devise de chaque compte.
fn sale_entries_of(
    lines: Vec<reads::SaleEntryLine>,
    currencies: &HashMap<i64, String>,
) -> Vec<SaleEntry> {
    let mut out: Vec<SaleEntry> = Vec::new();
    let mut has_debit: Vec<bool> = Vec::new();
    for l in lines {
        let currency = currencies
            .get(&l.account_id)
            .cloned()
            .unwrap_or_else(|| BASE_CURRENCY.to_string());
        let amount = if currency == BASE_CURRENCY {
            l.chf_movement
        } else {
            l.currency_movement
        };
        let movement = SaleMovement {
            ledger_account_id: l.account_id,
            currency,
            amount,
        };
        let debit = l.debit_total > Decimal::ZERO;
        match out.last_mut() {
            Some(e) if e.journal_entry_id == l.journal_entry_id => {
                e.movements.push(movement);
                if let Some(flag) = has_debit.last_mut() {
                    *flag |= debit;
                }
            }
            _ => {
                out.push(SaleEntry {
                    journal_entry_id: l.journal_entry_id,
                    entry_number: l.entry_number,
                    entry_date: l.entry_date,
                    description: l.description,
                    movements: vec![movement],
                    linked: l.linked != 0,
                });
                has_debit.push(debit);
            }
        }
    }
    out.into_iter()
        .zip(has_debit)
        .filter_map(|(e, d)| d.then_some(e))
        .collect()
}

/// Les lectures et la revue commune, **dans la transaction de lecture** de
/// l'appelant (Wise et Stripe y ajoutent leurs lectures propres, dans le même
/// instantané). Refus : `404 PROVIDER_ACCOUNT_NOT_FOUND`, `404
/// FISCAL_YEAR_NOT_FOUND`.
pub async fn load_review_in_tx(
    tx: &mut sqlx::Transaction<'_, MySql>,
    company_id: i64,
    spec: &ProviderSpec,
    account: &str,
    fiscal_year_id: i64,
) -> Result<LoadedReview, AppError> {
    let balances =
        bank_accounts::list_provider_balances(&mut **tx, company_id, spec.provider, account)
            .await?;
    if balances.is_empty() {
        return Err(AppError::ProviderAccountNotFound {
            account: account.to_string(),
        });
    }
    let fiscal_year = reads::find_fiscal_year(&mut **tx, company_id, fiscal_year_id)
        .await?
        .ok_or(AppError::ReportFiscalYearNotFound { fiscal_year_id })?;

    let accounts = provider_accounts(&balances);
    let ledger_currencies: HashMap<i64, String> = balances
        .iter()
        .filter_map(|b| {
            b.account
                .journal_account_id
                .map(|l| (l, b.currency().to_string()))
        })
        .collect();
    let mut ledger_ids: Vec<i64> = ledger_currencies.keys().copied().collect();
    ledger_ids.sort_unstable();
    let own_ids: Vec<i64> = balances.iter().map(|b| b.account.id).collect();

    let pending = reads::pending_in_period(
        &mut **tx,
        company_id,
        &own_ids,
        fiscal_year.start_date,
        fiscal_year.end_date,
    )
    .await?;
    let linked_accounts =
        reads::bank_accounts_on_ledgers(&mut **tx, company_id, &ledger_ids).await?;
    let all_linked =
        reads::transactions_of_accounts(&mut **tx, company_id, &linked_accounts).await?;
    let lines = reads::sale_entries(&mut **tx, company_id, fiscal_year.id, &ledger_ids).await?;
    let sales = sale_entries_of(lines, &ledger_currencies);
    let rules = reconciliation_rules::find_active_for_company(&mut **tx, company_id).await?;
    let rule_account_ids = reads::rule_account_ids(&mut **tx, company_id).await?;
    let primary = reads::primary_ledger_account_id(&mut **tx, company_id).await?;

    let review = build_review(
        &ReviewInput {
            accounts: &accounts,
            pending: &pending,
            all_linked_accounts_transactions: &all_linked,
            sale_entries: &sales,
            rules: &rules,
            rule_account_ids: &rule_account_ids,
            primary_ledger_account_id: primary,
        },
        spec,
    );
    Ok(LoadedReview {
        review,
        balances,
        pending,
    })
}

/// La revue complète d'un prestataire **sans genre propre** (PayPal) : une
/// transaction de lecture, [`load_review_in_tx`], la réponse.
pub async fn review(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    spec: &ProviderSpec,
    query: ReviewQuery,
) -> Result<ReviewResponse, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let loaded = load_review_in_tx(
        &mut tx,
        current_user.company_id,
        spec,
        &query.account,
        query.fiscal_year_id,
    )
    .await;
    // Lecture seule : rien à conclure.
    let _ = tx.rollback().await;
    let loaded = loaded?;
    Ok(ReviewResponse::of(
        query.account,
        query.fiscal_year_id,
        loaded.review,
    ))
}

// ---------------------------------------------------------------------------
// Acceptation : corps, réponse, pré-vol
// ---------------------------------------------------------------------------

/// Corps de `POST /api/v1/<prestataire>/accept`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptBody<P = Proposal> {
    /// Libellé du compte du prestataire.
    pub account: String,
    /// Propositions choisies (formes de la revue).
    pub proposals: Vec<P>,
}

/// Extracteur du corps : un JSON mal formé rend `400 VALIDATION_ERROR`
/// (*Pattern batch* : corps mal formé), non le `422` d'axum.
pub struct AcceptBodyExtractor<P = Proposal>(pub AcceptBody<P>);

impl<S, P> FromRequest<S> for AcceptBodyExtractor<P>
where
    S: Send + Sync,
    P: serde::de::DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<AcceptBody<P>>::from_request(req, state).await {
            Ok(Json(body)) => Ok(Self(body)),
            Err(rej) => {
                let message = match &rej {
                    JsonRejection::JsonDataError(_) | JsonRejection::JsonSyntaxError(_) => {
                        "corps JSON invalide : `account` et `proposals` attendus, chaque proposition avec son `type` et ses champs".to_string()
                    }
                    JsonRejection::MissingJsonContentType(_) => {
                        "Content-Type attendu : application/json".to_string()
                    }
                    _ => "requête invalide (corps non lisible)".to_string(),
                };
                Err(AppError::Validation(message).into_response())
            }
        }
    }
}

/// Une proposition acceptée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAccepted {
    /// Le genre (`link_entry`, `fee`…).
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// Les transactions rapprochées.
    pub bank_transaction_ids: Vec<i64>,
    /// Les écritures passées ou rattachées.
    pub journal_entry_ids: Vec<i64>,
    /// La règle appliquée (proposition `rule`), pour la trace.
    #[serde(skip)]
    pub rule_id: Option<i64>,
}

/// Réponse de l'acceptation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAcceptResponse {
    /// Propositions acceptées, dans l'ordre du corps.
    pub accepted: Vec<ProviderAccepted>,
    /// Propositions refusées, dans l'ordre du corps.
    pub failed: Vec<FailedProposal>,
}

/// Le refus d'une proposition dans le lot.
#[derive(Debug)]
pub enum Refusal {
    /// Refus par proposition, rangé dans `failed[]`.
    Failed(FailedProposal),
    /// Interblocage : remonté en erreur pour que l'enveloppe rejoue toute
    /// l'opération — **jamais** dans `failed[]`.
    Abort(DbError),
}

/// Une erreur de base d'une proposition : un interblocage (1213) →
/// [`Refusal::Abort`] ; toute autre → `failed[]` `DATABASE_ERROR`. **Le seul**
/// passage des erreurs de base du moteur vers un refus.
pub fn db_refusal(bank_transaction_id: i64, e: DbError) -> Refusal {
    if is_deadlock_error(&e) {
        return Refusal::Abort(e);
    }
    Refusal::Failed(FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": match &e {
            DbError::Sqlx(inner) => inner.to_string(),
            other => other.to_string(),
        } })),
    })
}

/// Une proposition périmée ([`PROVIDER_PROPOSAL_STALE`], `details.reason`).
pub fn stale(bank_transaction_id: i64, reason: &str) -> Refusal {
    Refusal::Failed(FailedProposal {
        bank_transaction_id,
        error_code: PROVIDER_PROPOSAL_STALE.to_string(),
        details: Some(serde_json::json!({ "reason": reason })),
    })
}

/// Un refus par proposition, construit.
fn failed(bank_transaction_id: i64, code: &str, details: serde_json::Value) -> Refusal {
    Refusal::Failed(FailedProposal {
        bank_transaction_id,
        error_code: code.to_string(),
        details: Some(details),
    })
}

/// Une proposition que sait accepter le moteur. [`Proposal`] (les genres
/// communs) l'implémente ; un prestataire qui a des propositions propres
/// (Stripe : `payout`) définit son type, délègue les genres communs à
/// [`accept_common`] et décide les siens avec les briques de ce module.
pub trait EngineProposal: Clone + Send + Sync {
    /// Toutes les transactions de la proposition.
    fn transaction_ids(&self) -> Vec<i64>;
    /// Celle que nomme un refus.
    fn primary_transaction_id(&self) -> i64;
    /// Contrôle de forme du pré-vol (`Err` : message du `400` global).
    fn check_shape(&self) -> Result<(), String>;
    /// Acceptation sous les verrous, dans le point de sauvegarde du lot.
    fn accept<'a, 't>(
        &'a self,
        tx: &'a mut sqlx::Transaction<'t, MySql>,
        ctx: &'a AcceptContext<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderAccepted, Refusal>> + Send + 'a>>
    where
        't: 'a;
}

impl EngineProposal for Proposal {
    fn transaction_ids(&self) -> Vec<i64> {
        Proposal::transaction_ids(self)
    }

    fn primary_transaction_id(&self) -> i64 {
        Proposal::primary_transaction_id(self)
    }

    fn check_shape(&self) -> Result<(), String> {
        let positive = |v: i64, field: &str| {
            if v > 0 {
                Ok(())
            } else {
                Err(format!(
                    "{field} doit être strictement positif (type='{}')",
                    self.kind()
                ))
            }
        };
        for id in self.transaction_ids() {
            positive(id, "l'identifiant de transaction")?;
        }
        match self {
            Proposal::LinkEntry {
                journal_entry_id, ..
            } => positive(*journal_entry_id, "journalEntryId"),
            Proposal::Fee { fee_account_id, .. } => positive(*fee_account_id, "feeAccountId"),
            Proposal::Rule {
                rule_id,
                counterparty_account_id,
                ..
            } => {
                positive(*rule_id, "ruleId")?;
                positive(*counterparty_account_id, "counterpartyAccountId")
            }
            Proposal::Withdrawal {
                counterparty_account_id,
                ..
            } => match counterparty_account_id {
                Some(id) => positive(*id, "counterpartyAccountId"),
                None => Err("counterpartyAccountId est obligatoire (type='withdrawal')".into()),
            },
            Proposal::Conversion { .. } => Ok(()),
        }
    }

    fn accept<'a, 't>(
        &'a self,
        tx: &'a mut sqlx::Transaction<'t, MySql>,
        ctx: &'a AcceptContext<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<ProviderAccepted, Refusal>> + Send + 'a>>
    where
        't: 'a,
    {
        Box::pin(accept_common(tx, ctx, self))
    }
}

/// Un lot prêt pour la tentative sous verrous.
#[derive(Debug)]
pub struct PreparedAccept<P = Proposal> {
    /// Société.
    pub company_id: i64,
    /// Acteur (utilisateur, clé d'API).
    pub acteur: Acteur,
    /// Libellé du compte du prestataire.
    pub account: String,
    /// Propositions éligibles (pré-vol passé), dans l'ordre du corps.
    pub eligible: Vec<P>,
    /// Comptes bancaires à verrouiller.
    pub lock_ids: Vec<i64>,
    /// Ordre du corps (transaction nommée par chaque proposition).
    order: Vec<i64>,
    /// Refus du pré-vol.
    preflight_failed: HashMap<i64, FailedProposal>,
}

/// Issue d'une tentative : acceptées et refusées, dans l'ordre du lot éligible.
#[derive(Debug, Default)]
pub struct BatchOutcome {
    /// Acceptées.
    pub accepted: Vec<ProviderAccepted>,
    /// Refusées sous les verrous.
    pub failed: Vec<FailedProposal>,
}

impl<P> PreparedAccept<P> {
    /// Rien d'éligible : ni transaction, ni verrou.
    pub fn is_empty(&self) -> bool {
        self.eligible.is_empty()
    }

    /// La réponse : `failed[]` réunit le pré-vol et la tentative, dans
    /// l'ordre du corps.
    pub fn finish(self, outcome: BatchOutcome) -> ProviderAcceptResponse {
        let mut by_id = self.preflight_failed;
        for f in outcome.failed {
            by_id.insert(f.bank_transaction_id, f);
        }
        let failed = self
            .order
            .iter()
            .filter_map(|id| by_id.remove(id))
            .collect();
        ProviderAcceptResponse {
            accepted: outcome.accepted,
            failed,
        }
    }
}

/// Le pré-vol de l'acceptation (voir l'en-tête du module), sur le pool, hors
/// verrou. `extra_lock_accounts` : comptes bancaires que le prestataire fait
/// verrouiller en plus (Stripe : le compte de versement).
pub async fn prepare_accept<P: EngineProposal>(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    spec: &ProviderSpec,
    body: AcceptBody<P>,
    extra_lock_accounts: &[i64],
) -> Result<PreparedAccept<P>, AppError> {
    if body.proposals.is_empty() {
        return Err(AppError::Validation("proposals vide".into()));
    }
    if body.proposals.len() > reconciliation_repo::MAX_IN_CLAUSE_IDS {
        return Err(AppError::Validation(format!(
            "un lot ne peut pas dépasser {} propositions",
            reconciliation_repo::MAX_IN_CLAUSE_IDS
        )));
    }
    let mut seen: HashSet<i64> = HashSet::new();
    for p in &body.proposals {
        p.check_shape().map_err(AppError::Validation)?;
        for id in p.transaction_ids() {
            if !seen.insert(id) {
                return Err(AppError::Validation(format!(
                    "transaction {id} portée deux fois dans le lot"
                )));
            }
        }
    }

    let company_id = current_user.company_id;
    let balances =
        bank_accounts::list_provider_balances(pool, company_id, spec.provider, &body.account)
            .await?;
    if balances.is_empty() {
        return Err(AppError::ProviderAccountNotFound {
            account: body.account,
        });
    }
    let own: HashSet<i64> = balances.iter().map(|b| b.account.id).collect();
    let ids: Vec<i64> = seen.into_iter().collect();
    let transactions = reads::transactions_by_ids(pool, company_id, &ids).await?;

    let mut order = Vec::with_capacity(body.proposals.len());
    let mut preflight_failed = HashMap::new();
    let mut eligible = Vec::with_capacity(body.proposals.len());
    let mut lock_ids: Vec<i64> = extra_lock_accounts.to_vec();
    for p in body.proposals {
        let primary = p.primary_transaction_id();
        order.push(primary);
        let accounts: Option<Vec<i64>> = p
            .transaction_ids()
            .iter()
            .map(|id| {
                transactions
                    .get(id)
                    .map(|t| t.bank_account_id)
                    .filter(|a| own.contains(a))
            })
            .collect();
        match accounts {
            Some(accounts) => {
                lock_ids.extend(accounts);
                eligible.push(p);
            }
            None => {
                preflight_failed.insert(
                    primary,
                    FailedProposal {
                        bank_transaction_id: primary,
                        error_code: "BANK_TRANSACTION_NOT_FOUND".to_string(),
                        details: None,
                    },
                );
            }
        }
    }
    lock_ids.sort_unstable();
    lock_ids.dedup();
    Ok(PreparedAccept {
        company_id,
        acteur: Acteur {
            user_id: current_user.user_id,
            api_key_id: current_user.api_key_id,
        },
        account: body.account,
        eligible,
        lock_ids,
        order,
        preflight_failed,
    })
}

// ---------------------------------------------------------------------------
// Acceptation : la tentative
// ---------------------------------------------------------------------------

/// Ce que voit chaque proposition sous les verrous.
pub struct AcceptContext<'a> {
    /// Le paramétrage du prestataire.
    pub spec: &'a ProviderSpec,
    /// Société.
    pub company_id: i64,
    /// Acteur.
    pub acteur: Acteur,
    /// Libellé du compte du prestataire.
    pub account: &'a str,
    /// Les comptes bancaires du compte de prestataire, **relus sous les
    /// verrous**, par identifiant.
    pub balances: HashMap<i64, ProviderBalance>,
}

/// **Une tentative** de l'acceptation (voir l'en-tête du module), rejouée sur
/// interblocage par le gestionnaire du prestataire. Une erreur sqlx sort en
/// `AppError::Database(DbError::Sqlx(_))`, que reconnaît
/// `crate::retry::is_app_deadlock`.
pub async fn accept_once<P: EngineProposal>(
    pool: &sqlx::MySqlPool,
    spec: &ProviderSpec,
    prepared: &PreparedAccept<P>,
) -> Result<BatchOutcome, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let locked = with_account_locks(
        &mut tx,
        prepared.company_id,
        &prepared.lock_ids,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| run_batch(tx_inner, spec, prepared).await,
    )
    .await;
    match locked {
        Ok(outcome) => {
            tx.commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(outcome)
        }
        Err(e) => {
            drop(tx);
            Err(lock_error(e))
        }
    }
}

/// L'erreur de la tentative, rendue en `AppError` ; les verrous nommés sont
/// rendus au retour de la fermeture (ou de la connexion au pool, #842).
fn lock_error(e: ReconciliationError) -> AppError {
    match e {
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
        ReconciliationError::Database(e) => AppError::Database(DbError::Sqlx(e)),
        ReconciliationError::Db(e) => AppError::Database(e),
        other => {
            tracing::error!(?other, "acceptation d'un prestataire : erreur inattendue");
            AppError::Internal("acceptation d'un prestataire : erreur inattendue".into())
        }
    }
}

/// Le lot, sous les verrous : relecture des réglages, puis un point de
/// sauvegarde par proposition. Un [`Refusal::Abort`] remonte aussitôt, sans
/// `ROLLBACK TO SAVEPOINT` (voir l'en-tête du module).
async fn run_batch<P: EngineProposal>(
    tx: &mut sqlx::Transaction<'_, MySql>,
    spec: &ProviderSpec,
    prepared: &PreparedAccept<P>,
) -> Result<BatchOutcome, ReconciliationError> {
    let balances = bank_accounts::list_provider_balances(
        &mut **tx,
        prepared.company_id,
        spec.provider,
        &prepared.account,
    )
    .await
    .map_err(ReconciliationError::Db)?;
    let ctx = AcceptContext {
        spec,
        company_id: prepared.company_id,
        acteur: prepared.acteur,
        account: &prepared.account,
        balances: balances.into_iter().map(|b| (b.account.id, b)).collect(),
    };
    let mut outcome = BatchOutcome::default();
    for p in &prepared.eligible {
        let primary = p.primary_transaction_id();
        let savepoint = format!("sp_{primary}");
        sqlx::query(&format!("SAVEPOINT {savepoint}"))
            .execute(&mut **tx)
            .await
            .map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?;
        let result = match p.accept(tx, &ctx).await {
            Ok(accepted) => match audit_accepted(tx, &ctx, &accepted).await {
                Ok(()) => Ok(accepted),
                Err(e) => Err(db_refusal(primary, e)),
            },
            Err(r) => Err(r),
        };
        match result {
            Ok(accepted) => {
                sqlx::query(&format!("RELEASE SAVEPOINT {savepoint}"))
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?;
                outcome.accepted.push(accepted);
            }
            Err(Refusal::Abort(e)) => return Err(ReconciliationError::Db(e)),
            Err(Refusal::Failed(f)) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {savepoint}"))
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?;
                outcome.failed.push(f);
            }
        }
    }
    Ok(outcome)
}

/// La trace `provider.proposal_accepted` d'une proposition acceptée, sous
/// l'acteur.
async fn audit_accepted(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    accepted: &ProviderAccepted,
) -> Result<(), DbError> {
    let entity_id = accepted.bank_transaction_ids.first().copied().unwrap_or(0);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            ctx.acteur.user_id,
            ctx.acteur.api_key_id,
            AUDIT_PROPOSAL_ACCEPTED,
            "bank_transaction",
            entity_id,
            Some(serde_json::json!({
                "provider": ctx.spec.provider,
                "provider_account": ctx.account,
                "type": accepted.kind,
                "bank_transaction_ids": accepted.bank_transaction_ids,
                "journal_entry_ids": accepted.journal_entry_ids,
                "rule_id": accepted.rule_id,
            })),
        ),
    )
    .await
    .map(|_| ())
}

/// L'acceptation d'une proposition commune.
pub async fn accept_common(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    p: &Proposal,
) -> Result<ProviderAccepted, Refusal> {
    match p {
        Proposal::LinkEntry {
            bank_transaction_id,
            journal_entry_id,
        } => accept_link_entry(tx, ctx, *bank_transaction_id, *journal_entry_id).await,
        Proposal::Fee {
            bank_transaction_id,
            fee_account_id,
        } => accept_fee(tx, ctx, *bank_transaction_id, *fee_account_id).await,
        Proposal::Rule {
            bank_transaction_id,
            rule_id,
            counterparty_account_id,
        } => {
            accept_rule(
                tx,
                ctx,
                *bank_transaction_id,
                *rule_id,
                *counterparty_account_id,
            )
            .await
        }
        Proposal::Withdrawal {
            bank_transaction_id,
            counterparty_account_id,
        } => {
            let Some(counterparty) = counterparty_account_id else {
                // Refusé au pré-vol (`check_shape`) : défense en profondeur.
                return Err(failed(
                    *bank_transaction_id,
                    "VALIDATION_ERROR",
                    serde_json::json!({ "reason": "counterparty_required" }),
                ));
            };
            accept_withdrawal(tx, ctx, *bank_transaction_id, *counterparty).await
        }
        Proposal::Conversion {
            debit_transaction_id,
            credit_transaction_id,
        } => accept_conversion(tx, ctx, *debit_transaction_id, *credit_transaction_id).await,
    }
}

// ---------------------------------------------------------------------------
// Briques : rejugement, contrepartie, écriture, rapprochement
// ---------------------------------------------------------------------------

/// Une transaction relue sous les verrous, avec son compte bancaire.
pub struct Rejudged<'a> {
    /// La transaction.
    pub transaction: BankTransaction,
    /// Son compte bancaire (compte de prestataire).
    pub balance: &'a ProviderBalance,
    /// Son compte du plan.
    pub ledger_account_id: i64,
}

/// Rejuge une transaction sous les verrous : encore **pending**, sur un compte
/// du compte de prestataire relu, de genre admis par `kind_ok` ; sinon
/// [`PROVIDER_PROPOSAL_STALE`]. Compte clos → `BANK_ACCOUNT_CLOSED` ; compte du
/// plan absent → `BANK_ACCOUNT_NOT_CONFIGURED`.
pub async fn rejudge<'a>(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &'a AcceptContext<'_>,
    id: i64,
    kind_ok: impl Fn(ProviderKind) -> bool,
) -> Result<Rejudged<'a>, Refusal> {
    let found = reads::transactions_by_ids(&mut **tx, ctx.company_id, &[id])
        .await
        .map_err(|e| db_refusal(id, e))?;
    let Some(t) = found.into_iter().next().map(|(_, t)| t) else {
        return Err(stale(id, "transaction_not_found"));
    };
    let Some(balance) = ctx.balances.get(&t.bank_account_id) else {
        return Err(stale(id, "provider_settings_changed"));
    };
    if kesh_db::repositories::bank_imports::is_reconciled(t.status.as_str(), t.matched_entry_id) {
        return Err(stale(id, "transaction_not_pending"));
    }
    if !kind_ok((ctx.spec.classify)(&t)) {
        return Err(stale(id, "kind_changed"));
    }
    if balance.account.is_closed() {
        return Err(Refusal::Failed(
            closed_account_failed(&[id], balance.account.id).remove(0),
        ));
    }
    let Some(ledger_account_id) = balance.account.journal_account_id else {
        return Err(failed(
            id,
            "BANK_ACCOUNT_NOT_CONFIGURED",
            serde_json::json!({ "bankAccountId": balance.account.id }),
        ));
    };
    Ok(Rejudged {
        transaction: t,
        balance,
        ledger_account_id,
    })
}

/// Le contrôle d'une contrepartie, **paramétré par le genre** (AC-8), fait
/// avant l'écriture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CounterpartyRule {
    /// Frais : compte de charge (38-11b AC-2).
    Expense,
    /// Règle : le contrôle d'`accept_one_rule` (actif, imputable).
    Rule,
    /// Retrait : compte d'actif en CHF.
    AssetChf,
    /// Compte d'actif de la devise donnée (versement Stripe, 40-7c AC-4 d).
    AssetInCurrency(String),
}

/// Contrôle la contrepartie `account_id` (voir [`CounterpartyRule`]) : jamais
/// le compte du plan du compte de prestataire (`VALIDATION_ERROR`,
/// `counterparty_equals_bank_ledger`), active (`ACCOUNT_NOT_FOUND`),
/// imputable (`ACCOUNT_NOT_POSTABLE`), du type et de la devise attendus
/// (`VALIDATION_ERROR`, `counterparty_wrong_type`, `counterparty_wrong_currency`).
/// Rend sa devise (`None` : CHF).
pub async fn check_counterparty(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    bank_transaction_id: i64,
    account_id: i64,
    ledger_account_id: i64,
    rule: CounterpartyRule,
) -> Result<Option<String>, Refusal> {
    if account_id == ledger_account_id {
        return Err(Refusal::Failed(
            counterparty_is_bank_ledger_failed_proposal(bank_transaction_id),
        ));
    }
    let account = reads::counterparty_account(&mut **tx, ctx.company_id, account_id)
        .await
        .map_err(|e| db_refusal(bank_transaction_id, e))?;
    let Some(account) = account.filter(|a| a.active) else {
        return Err(failed(
            bank_transaction_id,
            "ACCOUNT_NOT_FOUND",
            serde_json::json!({ "missingAccountIds": [account_id] }),
        ));
    };
    if !account.postable {
        return Err(Refusal::Failed(non_postable_failed_proposal(
            bank_transaction_id,
            vec![kesh_db::errors::NonPostableAccount {
                account_id,
                account_number: account.number,
            }],
        )));
    }
    let (expected_type, expected_currency): (Option<&str>, Option<Option<&str>>) = match &rule {
        CounterpartyRule::Expense => (Some("Expense"), None),
        CounterpartyRule::Rule => (None, None),
        CounterpartyRule::AssetChf => (Some("Asset"), Some(None)),
        CounterpartyRule::AssetInCurrency(c) => (
            Some("Asset"),
            Some(if c == BASE_CURRENCY {
                None
            } else {
                Some(c.as_str())
            }),
        ),
    };
    if let Some(t) = expected_type
        && account.account_type != t
    {
        return Err(failed(
            bank_transaction_id,
            "VALIDATION_ERROR",
            serde_json::json!({ "reason": "counterparty_wrong_type", "expected": t, "accountId": account_id }),
        ));
    }
    if let Some(c) = expected_currency
        && account.currency.as_deref() != c
    {
        return Err(failed(
            bank_transaction_id,
            "VALIDATION_ERROR",
            serde_json::json!({ "reason": "counterparty_wrong_currency", "expected": c.unwrap_or(BASE_CURRENCY), "accountId": account_id }),
        ));
    }
    Ok(account.currency)
}

/// Une erreur de l'écriture : refus métier sous son code canonique
/// (verrou de période, exercice postérieur clos, contrôle central des lignes
/// en devise, compte clos, conflit de version), interblocage remonté, le reste
/// en `DATABASE_ERROR` ([`db_refusal`]).
pub fn entry_refusal(bank_transaction_id: i64, e: DbError) -> Refusal {
    if is_deadlock_error(&e) {
        return Refusal::Abort(e);
    }
    match &e {
        DbError::PeriodLocked {
            locked_through,
            attempted,
        } => Refusal::Failed(period_locked_failed_proposal(
            bank_transaction_id,
            None,
            *locked_through,
            *attempted,
        )),
        DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        } => Refusal::Failed(later_fiscal_year_closed_failed_proposal(
            bank_transaction_id,
            None,
            *fiscal_year_id,
            fiscal_year_name,
        )),
        DbError::BankAccountClosed { bank_account_id } => Refusal::Failed(
            closed_account_failed(&[bank_transaction_id], *bank_account_id).remove(0),
        ),
        DbError::OptimisticLockConflict => failed(
            bank_transaction_id,
            "OPTIMISTIC_LOCK_CONFLICT",
            serde_json::json!({}),
        ),
        DbError::FxAccountInvalid {
            field,
            reason,
            holder_account_id,
        } => fx_account_invalid(bank_transaction_id, *field, *reason, *holder_account_id),
        _ => match fx_failed_proposal(bank_transaction_id, &e) {
            Some(fp) => Refusal::Failed(fp),
            None => db_refusal(bank_transaction_id, e),
        },
    }
}

/// `FX_ACCOUNT_INVALID` par proposition, `details` de la réponse HTTP
/// (`field`, `reason`, `holderAccountId`).
fn fx_account_invalid(
    bank_transaction_id: i64,
    field: FxAccountField,
    reason: FxAccountReason,
    holder_account_id: Option<i64>,
) -> Refusal {
    let mut details = serde_json::json!({
        "field": field.as_api_field(),
        "reason": reason.as_str(),
    });
    if let Some(holder) = holder_account_id {
        details["holderAccountId"] = serde_json::json!(holder);
    }
    failed(bank_transaction_id, "FX_ACCOUNT_INVALID", details)
}

/// Le cours mensuel retenu par la société pour `currency` au mois de `date`,
/// et sa provenance ; absent → `RATE_UNAVAILABLE` (`details: { currency,
/// month }`, forme de `fx_failed_proposal`).
pub async fn month_rate(
    tx: &mut sqlx::Transaction<'_, MySql>,
    company_id: i64,
    bank_transaction_id: i64,
    currency: &str,
    date: NaiveDate,
) -> Result<SourceRate, Refusal> {
    let month = first_of_month(date);
    let found = find_company_rate_in_tx(tx, company_id, currency, RateKind::Month, month)
        .await
        .map_err(|e| db_refusal(bank_transaction_id, e))?;
    match found {
        Some(r) => Ok(SourceRate {
            rate: r.rate,
            source: if r.source.is_afc() {
                RateProvenance::Afc
            } else {
                RateProvenance::Manual
            },
        }),
        None => Err(entry_refusal(
            bank_transaction_id,
            DbError::RateUnavailable {
                currency: currency.to_string(),
                month,
            },
        )),
    }
}

/// Ce qu'écrit [`write_provider_entry`].
pub struct ProviderEntry<'a> {
    /// La transaction (relue sous les verrous).
    pub transaction: &'a BankTransaction,
    /// Le compte du plan du compte de prestataire.
    pub ledger_account_id: i64,
    /// Sa devise.
    pub ledger_currency: &'a str,
    /// La contrepartie, contrôlée avant ([`check_counterparty`]).
    pub counterparty_account_id: i64,
    /// Sa devise (`None` : CHF).
    pub counterparty_currency: Option<&'a str>,
    /// Le genre, pour le libellé.
    pub kind: EntryKind,
    /// Projet analytique (règle).
    pub project_id: Option<i64>,
}

/// **La** fonction d'écriture du moteur (AC-8) — voir l'en-tête du module.
/// Rend l'identifiant de l'écriture.
pub async fn write_provider_entry(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    entry: ProviderEntry<'_>,
) -> Result<i64, Refusal> {
    let t = entry.transaction;
    let id = t.id;
    if t.amount.is_zero() {
        return Err(failed(
            id,
            "VALIDATION_ERROR",
            serde_json::json!({ "reason": "zero_amount_transaction" }),
        ));
    }
    let date = payment_date(t);
    let amount = t.amount.abs();
    let (chf, foreign) = if entry.ledger_currency == BASE_CURRENCY {
        (amount, None)
    } else {
        let rate = month_rate(tx, ctx.company_id, id, entry.ledger_currency, date).await?;
        let chf = kesh_core::accounting::fx::fx_chf(&amount, &rate.rate).map_err(|e| {
            failed(
                id,
                "FX_RATE_INVALID",
                serde_json::json!({ "currency": entry.ledger_currency, "message": format!("{e:?}") }),
            )
        })?;
        (
            chf,
            Some(ForeignAmount {
                declared_currency: None,
                amount,
                rate: rate.rate,
                source: Some(rate.source),
                rate_derived: false,
            }),
        )
    };
    let counterparty_foreign = match (entry.counterparty_currency, &foreign) {
        (Some(c), Some(f)) if c == entry.ledger_currency => Some(f.clone()),
        _ => None,
    };
    // Crédit du prestataire (entrée) : débit de son compte du plan.
    let incoming = t.amount > Decimal::ZERO;
    let line = |account_id: i64, debit: bool, foreign: Option<ForeignAmount>| NewJournalEntryLine {
        account_id,
        debit: if debit { chf } else { Decimal::ZERO },
        credit: if debit { Decimal::ZERO } else { chf },
        project_id: None,
        foreign,
    };
    let description: String = (ctx.spec.entry_label)(entry.kind, ctx.account, t)
        .chars()
        .take(DESCRIPTION_MAX_CHARS)
        .collect();
    let new = NewJournalEntry {
        company_id: ctx.company_id,
        entry_date: date,
        journal: Journal::Banque,
        description,
        project_id: entry.project_id,
        lines: vec![
            line(entry.ledger_account_id, incoming, foreign),
            line(
                entry.counterparty_account_id,
                !incoming,
                counterparty_foreign,
            ),
        ],
    };
    let fiscal_year = fiscal_years::find_open_covering_date(tx, ctx.company_id, date)
        .await
        .map_err(|e| db_refusal(id, e))?;
    let Some(fiscal_year) = fiscal_year else {
        return Err(failed(
            id,
            "RECONCILIATION_FISCAL_YEAR_CLOSED",
            serde_json::json!({ "entryDate": date.to_string() }),
        ));
    };
    // Flux de rapprochement : garde de postabilité désactivée (14-3b, D-A0),
    // la contrepartie est contrôlée avant.
    journal_entries::create_in_tx(tx, fiscal_year.id, ctx.acteur, new, false)
        .await
        .map(|j| j.entry.id)
        .map_err(|e| entry_refusal(id, e))
}

/// Passe la transaction `reconciled` sur l'écriture (version lue sous les
/// verrous) ; ligne non mise à jour → `RECONCILIATION_ALREADY_RECONCILED`.
pub async fn mark_reconciled(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    t: &BankTransaction,
    journal_entry_id: i64,
) -> Result<(), Refusal> {
    let updated = reconciliation_repo::mark_reconciled_in_tx(
        tx,
        ctx.company_id,
        t.id,
        journal_entry_id,
        t.version,
    )
    .await
    .map_err(|e| db_refusal(t.id, e))?;
    if updated {
        Ok(())
    } else {
        Err(failed(
            t.id,
            "RECONCILIATION_ALREADY_RECONCILED",
            serde_json::json!({ "reason": "race_during_update" }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Les genres communs
// ---------------------------------------------------------------------------

/// `link_entry` : rattachement à une écriture déjà passée, par
/// `link_entry_in_tx` (aucune écriture). Rejugement : transaction pending,
/// d'encaissement (l'identifiant figure toujours au libellé de l'écriture) ou
/// d'un genre propre au prestataire ; une vente déjà rattachée rend la
/// proposition périmée. Après le lien d'un encaissement, une
/// lecture **verrouillante** vérifie qu'aucune autre transaction ne pointe la
/// vente (rattachée entre-temps depuis un autre compte : l'instantané de la
/// transaction, ouvert avant le verrou de l'écriture, ne le montrerait pas à
/// `link_entry_in_tx`).
pub async fn accept_link_entry(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    bank_transaction_id: i64,
    journal_entry_id: i64,
) -> Result<ProviderAccepted, Refusal> {
    let r = rejudge(tx, ctx, bank_transaction_id, |k| {
        matches!(k, ProviderKind::SaleReceipt | ProviderKind::Specific(_))
    })
    .await?;
    let sale = (ctx.spec.classify)(&r.transaction) == ProviderKind::SaleReceipt;
    if sale {
        let key = (ctx.spec.match_key)(&r.transaction).map(str::to_string);
        let description = reads::entry_description(&mut **tx, ctx.company_id, journal_entry_id)
            .await
            .map_err(|e| db_refusal(bank_transaction_id, e))?;
        let still = match (key, description) {
            (Some(k), Some(d)) => (ctx.spec.extract_ids)(&d).contains(&k),
            _ => false,
        };
        if !still {
            return Err(stale(bank_transaction_id, "entry_changed"));
        }
        let links = reads::entry_link_count(&mut **tx, ctx.company_id, journal_entry_id)
            .await
            .map_err(|e| db_refusal(bank_transaction_id, e))?;
        if links > 0 {
            return Err(stale(bank_transaction_id, "entry_linked"));
        }
    }
    let req = LinkEntryRequest {
        company_id: ctx.company_id,
        bank_account_id: r.balance.account.id,
        bank_transaction_id,
        journal_entry_id,
        bank_ledger_account_id: r.ledger_account_id,
        bank_ledger_currency: r.balance.currency(),
        user_id: ctx.acteur.user_id,
        actor_api_key_id: ctx.acteur.api_key_id,
    };
    let linked = match link_entry_in_tx(tx, req).await {
        Ok(Ok(l)) => l,
        Ok(Err(refusal)) => {
            let code = match refusal {
                LinkRefusal::NotFound => "NOT_FOUND",
                LinkRefusal::NotPending => "RECONCILIATION_TRANSACTION_NOT_PENDING",
                LinkRefusal::NotLinkable => "RECONCILIATION_ENTRY_NOT_LINKABLE",
            };
            return Err(failed(
                bank_transaction_id,
                code,
                serde_json::json!({ "journalEntryId": journal_entry_id }),
            ));
        }
        Err(e) => return Err(entry_refusal(bank_transaction_id, e)),
    };
    if sale {
        let others = reads::other_links_locked(
            tx,
            ctx.company_id,
            linked.journal_entry_id,
            bank_transaction_id,
        )
        .await
        .map_err(|e| db_refusal(bank_transaction_id, e))?;
        if others > 0 {
            return Err(stale(bank_transaction_id, "entry_linked_elsewhere"));
        }
    }
    Ok(ProviderAccepted {
        kind: "link_entry",
        bank_transaction_ids: vec![bank_transaction_id],
        journal_entry_ids: vec![linked.journal_entry_id],
        rule_id: None,
    })
}

/// `fee` : frais au compte de frais réglé (inchangé depuis la revue, sinon
/// périmée), compte de charge actif et imputable ; frais prélevé : débit
/// frais / crédit prestataire, frais rendu : l'inverse.
pub async fn accept_fee(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    bank_transaction_id: i64,
    fee_account_id: i64,
) -> Result<ProviderAccepted, Refusal> {
    let r = rejudge(tx, ctx, bank_transaction_id, |k| k == ProviderKind::Fee).await?;
    if r.balance.account.provider_settings.fee_account_id != Some(fee_account_id) {
        return Err(stale(bank_transaction_id, "provider_settings_changed"));
    }
    let currency = check_counterparty(
        tx,
        ctx,
        bank_transaction_id,
        fee_account_id,
        r.ledger_account_id,
        CounterpartyRule::Expense,
    )
    .await?;
    let entry_id = write_provider_entry(
        tx,
        ctx,
        ProviderEntry {
            transaction: &r.transaction,
            ledger_account_id: r.ledger_account_id,
            ledger_currency: r.balance.currency(),
            counterparty_account_id: fee_account_id,
            counterparty_currency: currency.as_deref(),
            kind: EntryKind::Fee,
            project_id: None,
        },
    )
    .await?;
    mark_reconciled(tx, ctx, &r.transaction, entry_id).await?;
    Ok(ProviderAccepted {
        kind: "fee",
        bank_transaction_ids: vec![bank_transaction_id],
        journal_entry_ids: vec![entry_id],
        rule_id: None,
    })
}

/// `rule` : la règle (active, qui correspond toujours, même contrepartie ;
/// sinon périmée), contrepartie contrôlée comme `accept_one_rule`, projet par
/// défaut revalidé ; écriture, compteur d'application et trace
/// `reconciliation_rule.applied`, comme `accept_one_rule`.
pub async fn accept_rule(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    bank_transaction_id: i64,
    rule_id: i64,
    counterparty_account_id: i64,
) -> Result<ProviderAccepted, Refusal> {
    let r = rejudge(tx, ctx, bank_transaction_id, |k| k == ProviderKind::Other).await?;
    let rule = reconciliation_rules::find_by_id_for_company(&mut **tx, ctx.company_id, rule_id)
        .await
        .map_err(|e| db_refusal(bank_transaction_id, e))?;
    let Some(rule) = rule.filter(|rule| {
        rule.active
            && rule.counterparty_account_id == counterparty_account_id
            && rule_matches(rule, &r.transaction)
    }) else {
        return Err(stale(bank_transaction_id, "rule_no_longer_matches"));
    };
    let currency = check_counterparty(
        tx,
        ctx,
        bank_transaction_id,
        counterparty_account_id,
        r.ledger_account_id,
        CounterpartyRule::Rule,
    )
    .await?;
    if let Some(pid) = rule.default_project_id
        && let Err(e) = projects::validate_taggable_in_tx(tx, ctx.company_id, &[pid]).await
    {
        if is_deadlock_error(&e) {
            return Err(Refusal::Abort(e));
        }
        return Err(Refusal::Failed(project_error_to_failed_proposal(
            bank_transaction_id,
            Some(pid),
            e,
        )));
    }
    let entry_id = write_provider_entry(
        tx,
        ctx,
        ProviderEntry {
            transaction: &r.transaction,
            ledger_account_id: r.ledger_account_id,
            ledger_currency: r.balance.currency(),
            counterparty_account_id,
            counterparty_currency: currency.as_deref(),
            kind: EntryKind::Rule,
            project_id: rule.default_project_id,
        },
    )
    .await?;
    mark_reconciled(tx, ctx, &r.transaction, entry_id).await?;
    let applied_count_after =
        reconciliation_rules::increment_applied_count_in_tx(tx, ctx.company_id, rule_id)
            .await
            .map_err(|e| db_refusal(bank_transaction_id, e))?;
    let entry_date = payment_date(&r.transaction);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            ctx.acteur.user_id,
            ctx.acteur.api_key_id,
            "reconciliation_rule.applied",
            "reconciliation_rules",
            rule_id,
            Some(serde_json::json!({
                "rule_id": rule_id,
                "rule_label": rule.label,
                "match_type": rule.match_type.as_str(),
                "match_value": rule.match_value,
                "bank_transaction_id": bank_transaction_id,
                "counterparty_account_id": counterparty_account_id,
                "journal_entry_id": entry_id,
                "applied_count_after": applied_count_after,
                "value_date": r.transaction.value_date.map(|d| d.to_string()),
                "entry_date": entry_date.to_string(),
                "was_previously_rejected": r.transaction.auto_match_rejected_at.is_some(),
            })),
        ),
    )
    .await
    .map_err(|e| db_refusal(bank_transaction_id, e))?;
    Ok(ProviderAccepted {
        kind: "rule",
        bank_transaction_ids: vec![bank_transaction_id],
        journal_entry_ids: vec![entry_id],
        rule_id: Some(rule_id),
    })
}

/// `withdrawal` : retrait d'un solde **CHF** (sinon périmé) vers le compte
/// choisi, compte d'actif imputable en CHF : débit contrepartie / crédit
/// prestataire. Le mouvement de la banque, importé par CAMT, se rattache à la
/// même écriture comme l'autre côté d'un virement (34-11b).
pub async fn accept_withdrawal(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    bank_transaction_id: i64,
    counterparty_account_id: i64,
) -> Result<ProviderAccepted, Refusal> {
    let r = rejudge(tx, ctx, bank_transaction_id, |k| {
        k == ProviderKind::Withdrawal
    })
    .await?;
    if r.balance.currency() != BASE_CURRENCY {
        return Err(stale(bank_transaction_id, "withdrawal_foreign_currency"));
    }
    let currency = check_counterparty(
        tx,
        ctx,
        bank_transaction_id,
        counterparty_account_id,
        r.ledger_account_id,
        CounterpartyRule::AssetChf,
    )
    .await?;
    let entry_id = write_provider_entry(
        tx,
        ctx,
        ProviderEntry {
            transaction: &r.transaction,
            ledger_account_id: r.ledger_account_id,
            ledger_currency: r.balance.currency(),
            counterparty_account_id,
            counterparty_currency: currency.as_deref(),
            kind: EntryKind::Withdrawal,
            project_id: None,
        },
    )
    .await?;
    mark_reconciled(tx, ctx, &r.transaction, entry_id).await?;
    Ok(ProviderAccepted {
        kind: "withdrawal",
        bank_transaction_ids: vec![bank_transaction_id],
        journal_entry_ids: vec![entry_id],
        rule_id: None,
    })
}

/// `conversion` : les deux moitiés rejugées (pending, genre conversion, du
/// compte de prestataire, débitée < 0 < créditée, devises différentes,
/// reconnues par `conversion_halves` ; sinon périmée), plan pur
/// (`plan_conversion`), écriture par `create_provider_conversion_in_tx`
/// (une écriture pointée par les deux transactions si un côté est en CHF ;
/// deux sinon, chaque transaction sur celle de son compte).
pub async fn accept_conversion(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    debit_transaction_id: i64,
    credit_transaction_id: i64,
) -> Result<ProviderAccepted, Refusal> {
    let id = debit_transaction_id;
    let is_conversion = |k| k == ProviderKind::Conversion;
    let debit = rejudge(tx, ctx, debit_transaction_id, is_conversion).await?;
    let credit = rejudge(tx, ctx, credit_transaction_id, is_conversion)
        .await
        .map_err(|r| match r {
            // Le refus nomme toujours la transaction débitée.
            Refusal::Failed(mut f) => {
                f.bank_transaction_id = id;
                Refusal::Failed(f)
            }
            abort => abort,
        })?;
    let paired = debit.transaction.amount < Decimal::ZERO
        && credit.transaction.amount > Decimal::ZERO
        && debit.balance.currency() != credit.balance.currency()
        && conversion_halves(&debit.transaction, &credit.transaction, ctx.spec);
    if !paired {
        return Err(stale(id, "conversion_pair_changed"));
    }
    let source = ConversionSide {
        ledger_account_id: debit.ledger_account_id,
        currency: debit.balance.currency().to_string(),
        amount: debit.transaction.amount.abs(),
    };
    let target = ConversionSide {
        ledger_account_id: credit.ledger_account_id,
        currency: credit.balance.currency().to_string(),
        amount: credit.transaction.amount.abs(),
    };
    let source_date = payment_date(&debit.transaction);
    let source_rate = if source.currency != BASE_CURRENCY && target.currency != BASE_CURRENCY {
        Some(month_rate(tx, ctx.company_id, id, &source.currency, source_date).await?)
    } else {
        None
    };
    let plan = plan_conversion(&source, &target, source_rate).map_err(|e| match e {
        PlanError::RateInvalid(message) => failed(
            id,
            "FX_RATE_INVALID",
            serde_json::json!({ "currency": target.currency, "message": message }),
        ),
        other => stale(id, &format!("{other:?}")),
    })?;
    let description: String =
        (ctx.spec.entry_label)(EntryKind::Conversion, ctx.account, &debit.transaction)
            .chars()
            .take(DESCRIPTION_MAX_CHARS)
            .collect();
    let entries = create_provider_conversion_in_tx(
        tx,
        ctx.company_id,
        ctx.acteur,
        plan,
        source_date,
        payment_date(&credit.transaction),
        description,
    )
    .await
    .map_err(|e| match e.kind {
        ConversionErrorKind::Db(db) => entry_refusal(id, db),
        ConversionErrorKind::NoFiscalYear(date) | ConversionErrorKind::FiscalYearClosed(date) => {
            failed(
                id,
                "RECONCILIATION_FISCAL_YEAR_CLOSED",
                serde_json::json!({ "entryDate": date.to_string() }),
            )
        }
        other => failed(
            id,
            "VALIDATION_ERROR",
            serde_json::json!({ "reason": "conversion_refused", "message": format!("{other:?}") }),
        ),
    })?;
    let (debit_entry, credit_entry) = match entries.as_slice() {
        [single] => (*single, *single),
        [first, second] => (*first, *second),
        _ => {
            return Err(db_refusal(
                id,
                DbError::Invariant("conversion de prestataire : écritures manquantes".into()),
            ));
        }
    };
    mark_reconciled(tx, ctx, &debit.transaction, debit_entry).await?;
    mark_reconciled(tx, ctx, &credit.transaction, credit_entry)
        .await
        .map_err(|r| match r {
            Refusal::Failed(mut f) => {
                f.bank_transaction_id = id;
                Refusal::Failed(f)
            }
            abort => abort,
        })?;
    let mut journal_entry_ids = vec![debit_entry];
    if credit_entry != debit_entry {
        journal_entry_ids.push(credit_entry);
    }
    Ok(ProviderAccepted {
        kind: "conversion",
        bank_transaction_ids: vec![debit_transaction_id, credit_transaction_id],
        journal_entry_ids,
        rule_id: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(entry: i64, account: i64, debit: &str, chf: &str, cur: &str) -> reads::SaleEntryLine {
        reads::SaleEntryLine {
            journal_entry_id: entry,
            entry_number: entry,
            entry_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            description: format!("e{entry}"),
            account_id: account,
            debit_total: debit.parse().unwrap(),
            chf_movement: chf.parse().unwrap(),
            currency_movement: cur.parse().unwrap(),
            linked: 0,
        }
    }

    #[test]
    fn ecritures_de_vente_regroupees_et_comptees_dans_la_devise_du_compte() {
        let currencies: HashMap<i64, String> =
            [(1020, "CHF".to_string()), (1021, "USD".to_string())].into();
        let sales = sale_entries_of(
            vec![
                line(1, 1021, "22.50", "22.50", "25.00"),
                line(2, 1020, "0", "-10", "0"),
                line(3, 1020, "5", "5", "0"),
                line(3, 1021, "0", "-4.50", "-5.00"),
            ],
            &currencies,
        );
        // L'écriture 2 n'a aucune ligne au débit : écartée.
        assert_eq!(
            sales.iter().map(|s| s.journal_entry_id).collect::<Vec<_>>(),
            [1, 3]
        );
        assert_eq!(sales[0].movements[0].amount, Decimal::new(2500, 2));
        assert_eq!(sales[0].movements[0].currency, "USD");
        assert_eq!(sales[1].movements.len(), 2);
        assert_eq!(sales[1].movements[0].amount, Decimal::new(5, 0));
        assert_eq!(sales[1].movements[1].amount, Decimal::new(-500, 2));
    }

    #[test]
    fn un_interblocage_n_est_jamais_un_refus_par_proposition() {
        // Une erreur de base quelconque : `DATABASE_ERROR`.
        match db_refusal(7, DbError::Invariant("x".into())) {
            Refusal::Failed(f) => {
                assert_eq!(f.error_code, "DATABASE_ERROR");
                assert_eq!(f.bank_transaction_id, 7);
            }
            Refusal::Abort(_) => panic!("pas un interblocage"),
        }
        // Un refus métier garde son code.
        match entry_refusal(7, DbError::OptimisticLockConflict) {
            Refusal::Failed(f) => assert_eq!(f.error_code, "OPTIMISTIC_LOCK_CONFLICT"),
            Refusal::Abort(_) => panic!("pas un interblocage"),
        }
        match entry_refusal(
            7,
            DbError::RateUnavailable {
                currency: "USD".into(),
                month: NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            },
        ) {
            Refusal::Failed(f) => {
                assert_eq!(f.error_code, "RATE_UNAVAILABLE");
                assert_eq!(
                    f.details.unwrap(),
                    serde_json::json!({ "currency": "USD", "month": "2025-03-01" })
                );
            }
            Refusal::Abort(_) => panic!("pas un interblocage"),
        }
    }

    #[test]
    fn forme_des_propositions_au_pre_vol() {
        let ok = Proposal::Fee {
            bank_transaction_id: 1,
            fee_account_id: 2,
        };
        assert!(EngineProposal::check_shape(&ok).is_ok());
        let sans_contrepartie = Proposal::Withdrawal {
            bank_transaction_id: 1,
            counterparty_account_id: None,
        };
        assert!(EngineProposal::check_shape(&sans_contrepartie).is_err());
        let negatif = Proposal::Conversion {
            debit_transaction_id: 1,
            credit_transaction_id: -2,
        };
        assert!(EngineProposal::check_shape(&negatif).is_err());
    }
}
