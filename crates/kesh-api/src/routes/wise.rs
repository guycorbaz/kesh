//! Routes de la **revue** et de l'**acceptation** Wise — Story 39-11c (AC-1 à
//! AC-7, amendement du 21.11), gestionnaires minces sur le moteur commun des
//! prestataires ([`super::provider_accept`]) avec le paramétrage Wise
//! ([`kesh_reconciliation::wise::SPEC`]).
//!
//! - `GET /api/v1/wise/review?account=<libellé>&fiscalYearId=<id>` —
//!   Comptable et plus, clés d'API admises en lecture ; lecture seule, une
//!   transaction de lecture ; `404 PROVIDER_ACCOUNT_NOT_FOUND`, `404
//!   FISCAL_YEAR_NOT_FOUND`.
//! - `POST /api/v1/wise/accept` — Comptable et plus, clés d'API `read-write` ;
//!   corps `{ account, proposals: [ {type, …} ] }` (`fee`, `conversion`,
//!   `link_entry`, `rule`, `card_group`), réponse `{ accepted, failed }`
//!   (*Pattern batch*).
//!
//! # Ce que la revue Wise ajoute à la revue commune
//!
//! 1. **Paiements par carte convertis** ([`wise::card_groups`], amendement
//!    A-1) : leurs lignes sortent du classement ligne à ligne ; une
//!    proposition `card_group` par paiement (règle de contrepartie en CHF sur
//!    la ligne principale), sinon « à examiner » (`card_group_no_rule`,
//!    `card_group_unsupported`, `card_group_statements_missing`,
//!    `card_group_partly_reconciled` : [`card_group_blocker`]).
//! 2. **Autre côté déjà passé** (AC-4) — pour un versement Stripe et un
//!    virement : `find_matched_elsewhere` (les écritures déjà rapprochées d'une
//!    transaction d'un autre compte bancaire), filtrée à
//!    [`wise::LINK_MAX_DAYS`] jours au plus **en Rust** (la lecture trie sans
//!    filtrer) : une → `link_entry` ; plusieurs → `ambiguous_link` ; aucune →
//!    `stripe_payout_unmatched` (versement, jamais de règle) ou les règles
//!    (virement : `rule`, sinon `no_rule`).
//! 3. **Écarts** (AC-6) : `fee_without_origin`, `balance_gap` (la règle de
//!    continuité de la Story 39-11b, `bank_imports::continuity_gap`, relue sur
//!    l'état présent des relevés) ; `conversion_half_missing` vient de la
//!    revue commune (paramétrage).
//!
//! # Ce que l'écran lit en plus (Story 39-11d2, AC-1)
//!
//! La revue se sert en [`WiseReviewProposal`], type **de revue** sérialisé
//! seul (patron de `StripeReviewProposal`) : la proposition `card_group` y
//! porte ses lignes (`lines`, les résumés dans l'ordre de
//! `bankTransactionIds`, principale en tête) et le montant du marchand
//! (`merchantAmount`, CHF, deux décimales) ; les quatre motifs `card_group_*`
//! portent `details.lines` ; `card_group_statements_missing` porte
//! `details.missingCurrencies` (posé par [`card_group_blocker`], donc servi
//! aussi dans `failed[].details` d'un refus `PROVIDER_PROPOSAL_STALE` de même
//! raison). Le corps de l'acceptation reste [`WiseProposal`] : un corps qui
//! reprend ces champs d'affichage est admis, ils sont ignorés.
//!
//! ⛔ Le gestionnaire d'acceptation s'appelle [`accept_proposals`], **jamais**
//! `post_accept` : l'exemption `RETRY_WITH_AUTORISE` du registre des routes
//! désigne une fonction par son seul nom. Il rejoue sur interblocage par
//! l'enveloppe `crate::retry::retry_app_on_deadlock` (opération
//! `wise::accept`), non par `retry_with`.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use axum::extract::{Query, State};
use axum::{Extension, Json};
use kesh_db::entities::bank_transaction::BankTransaction;
use kesh_db::entities::{
    ForeignAmount, Journal, NewJournalEntry, NewJournalEntryLine, ReconciliationRule,
};
use kesh_db::errors::{DbError, FxAccountField, FxAccountReason};
use kesh_db::repositories::bank_accounts::BASE_CURRENCY;
use kesh_db::repositories::bank_imports::DateCoverage;
use kesh_db::repositories::reconciliation_link::{EntryCandidate, find_matched_elsewhere};
use kesh_db::repositories::{
    bank_imports, fiscal_years, journal_entries, projects, provider_review as reads,
    reconciliation_rules,
};
use kesh_reconciliation::provider_review::{
    ENTRY_DESCRIPTION_MAX_CHARS, EntryKind, EntrySummary, Proposal, ProviderKind, REVIEW_NO_RULE,
    Review, ReviewFinding, ReviewProposal, TransactionSummary, matching_rule, payment_date,
};
use kesh_reconciliation::rule_matches;
use kesh_reconciliation::wise::{
    self, CardGroupJudgement, CardGroupProposal, KIND_STRIPE_PAYOUT, KIND_TRANSFER, SPEC,
    WiseProposal,
};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySql;

use super::provider_accept::{
    self, AcceptBodyExtractor, AcceptContext, BatchOutcome, CounterpartyRule, EngineProposal,
    PROVIDER_PROPOSAL_STALE, ProviderAcceptResponse, ProviderAccepted, Refusal, ReviewQuery,
    ReviewResponse, check_counterparty, db_refusal, entry_refusal, failed, mark_reconciled,
    month_rate, record_rule_applied, rejudge, stale,
};
use super::reconciliation::{
    counterparty_is_bank_ledger_failed_proposal, project_error_to_failed_proposal,
};
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Borne du libellé d'une écriture, en caractères : celle du moteur commun
/// (revue d'intégration de l'Epic 39, L1 : une seule définition).
const DESCRIPTION_MAX_CHARS: usize = ENTRY_DESCRIPTION_MAX_CHARS;

// ---------------------------------------------------------------------------
// Revue
// ---------------------------------------------------------------------------

/// Une proposition de la **revue** Wise (Story 39-11d2, AC-1), sérialisée à
/// plat (champ `type` compris) : un genre commun, ou un paiement par carte
/// groupé avec ce que l'écran en montre. Type de revue seul : le corps de
/// l'acceptation reste [`WiseProposal`].
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum WiseReviewProposal {
    /// Paiement par carte converti (`type: "card_group"`), avec ses lignes.
    CardGroup(CardGroupReview),
    /// Les genres communs (`link_entry`, `fee`, `rule`, `conversion`).
    Common(Proposal),
}

/// La proposition `card_group` de la revue : les champs du corps
/// d'acceptation, ses lignes et le montant du marchand.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardGroupReview {
    /// Les champs du corps (`type`, `bankTransactionIds`, `ruleId`,
    /// `counterpartyAccountId`).
    #[serde(flatten)]
    pub proposal: CardGroupProposal,
    /// Le résumé de chaque ligne, dans l'ordre de `bankTransactionIds`
    /// (principale en tête).
    pub lines: Vec<TransactionSummary>,
    /// Le montant du marchand en CHF, deux décimales (chaîne décimale).
    pub merchant_amount: String,
}

/// `GET /api/v1/wise/review` — voir l'en-tête du module.
pub async fn review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReviewQuery>,
) -> Result<Json<ReviewResponse<WiseReviewProposal>>, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let built = build_wise_review(&mut tx, current_user.company_id, &query).await;
    // Lecture seule : rien à conclure.
    let _ = tx.rollback().await;
    let review = built?;
    Ok(Json(ReviewResponse::of(
        query.account,
        query.fiscal_year_id,
        review,
    )))
}

/// Une proposition commune, avec la transaction qu'elle montre.
fn common(proposal: Proposal, t: &BankTransaction) -> ReviewProposal<WiseProposal> {
    ReviewProposal {
        proposal: WiseProposal::Common(proposal),
        transaction: TransactionSummary::of(t),
        other_transaction: None,
        entry: None,
    }
}

/// Ce que la revue montre d'une écriture candidate.
fn entry_summary(c: &EntryCandidate) -> EntrySummary {
    EntrySummary {
        journal_entry_id: c.journal_entry_id,
        entry_number: c.entry_number,
        entry_date: c.entry_date,
        description: c.description.clone(),
        ids: Vec::new(),
    }
}

/// Le montant du marchand d'un paiement groupé, en chaîne à deux décimales.
fn merchant_amount(merchant_chf: Decimal) -> String {
    let mut merchant = merchant_chf;
    merchant.rescale(2);
    merchant.to_string()
}

/// La revue Wise, dans la transaction de lecture : la revue commune, puis ce
/// qu'y ajoute Wise (voir l'en-tête du module), convertie à la fin en type de
/// revue ([`WiseReviewProposal`]).
async fn build_wise_review(
    tx: &mut sqlx::Transaction<'_, MySql>,
    company_id: i64,
    query: &ReviewQuery,
) -> Result<Review<WiseReviewProposal>, AppError> {
    let loaded = provider_accept::load_review_in_tx(
        tx,
        company_id,
        &SPEC,
        &query.account,
        query.fiscal_year_id,
    )
    .await?;
    let mut review = loaded.review.map_proposals(WiseProposal::Common);
    let balances: HashMap<i64, _> = loaded.balances.iter().map(|b| (b.account.id, b)).collect();
    let own_ids: Vec<i64> = loaded.balances.iter().map(|b| b.account.id).collect();
    let rules = reconciliation_rules::find_active_for_company(&mut **tx, company_id).await?;
    let rule_accounts = reads::rule_account_ids(&mut **tx, company_id).await?;
    let pending_by_id: HashMap<i64, &BankTransaction> =
        loaded.pending.iter().map(|t| (t.id, t)).collect();

    // 1. Paiements par carte convertis (amendement A-1).
    let groups = wise::card_groups(&loaded.pending);
    let grouped: HashSet<i64> = groups.iter().flat_map(|g| g.ids()).collect();
    remove_transactions(&mut review, &grouped);
    let chf_accounts = wise::chf_rule_accounts(&rule_accounts);
    // Ce que l'écran montre d'une proposition `card_group`, par ligne
    // principale : ses lignes et le montant du marchand (AC-1 de la 39-11d2).
    let mut shown: HashMap<i64, (Vec<TransactionSummary>, String)> = HashMap::new();
    for g in &groups {
        let principal = g.principal();
        let lines: Vec<TransactionSummary> =
            g.lines.iter().map(|t| TransactionSummary::of(t)).collect();
        let mut details = serde_json::json!({
            "cardId": g.card_id,
            "bankTransactionIds": g.ids(),
            "lines": lines,
        });
        match g.judgement {
            CardGroupJudgement::Unsupported(reason) => {
                details["reason"] = serde_json::json!(reason);
                review.push_to_review(ReviewFinding {
                    details: Some(details),
                    ..ReviewFinding::on(wise::REVIEW_CARD_GROUP_UNSUPPORTED, principal)
                });
            }
            CardGroupJudgement::Valid { merchant_chf } => {
                let merchant = merchant_amount(merchant_chf);
                details["merchantAmount"] = serde_json::json!(merchant);
                let blocker = card_group_blocker(
                    &mut *tx,
                    company_id,
                    loaded.balances.iter().map(|b| (b.account.id, b.currency())),
                    g,
                )
                .await?;
                if let Some((code, extra)) = blocker {
                    merge(&mut details, extra);
                    review.push_to_review(ReviewFinding {
                        details: Some(details),
                        ..ReviewFinding::on(code, principal)
                    });
                    continue;
                }
                let ledger = balances
                    .get(&principal.bank_account_id)
                    .and_then(|b| b.account.journal_account_id);
                match matching_rule(&rules, principal, &chf_accounts, ledger) {
                    Some(rule) => {
                        shown.insert(principal.id, (lines, merchant));
                        review.push_proposal(ReviewProposal {
                            proposal: WiseProposal::CardGroup(CardGroupProposal::CardGroup {
                                bank_transaction_ids: g.ids(),
                                rule_id: rule.id,
                                counterparty_account_id: rule.counterparty_account_id,
                            }),
                            transaction: TransactionSummary::of(principal),
                            other_transaction: None,
                            entry: None,
                        })
                    }
                    None => review.push_to_review(ReviewFinding {
                        details: Some(details),
                        ..ReviewFinding::on(wise::REVIEW_CARD_GROUP_NO_RULE, principal)
                    }),
                }
            }
        }
    }

    // 2. Autre côté déjà passé : versements Stripe et virements (AC-4, AC-5).
    let specific = std::mem::take(&mut review.specific);
    for (id, tag) in &specific {
        let Some(t) = pending_by_id.get(id).copied() else {
            continue;
        };
        let balance = balances.get(&t.bank_account_id);
        let ledger = balance.and_then(|b| b.account.journal_account_id);
        let candidates: Vec<EntryCandidate> = match (balance, ledger) {
            (Some(b), Some(l)) => find_matched_elsewhere(&mut **tx, company_id, t, l, b.currency())
                .await?
                .into_iter()
                .filter(|c| c.days_apart <= wise::LINK_MAX_DAYS)
                .collect(),
            _ => Vec::new(),
        };
        match candidates.as_slice() {
            [only] => review.push_proposal(ReviewProposal {
                entry: Some(entry_summary(only)),
                ..common(
                    Proposal::LinkEntry {
                        bank_transaction_id: t.id,
                        journal_entry_id: only.journal_entry_id,
                    },
                    t,
                )
            }),
            [_, _, ..] => review.push_to_review(ReviewFinding {
                entries: candidates.iter().map(entry_summary).collect(),
                ..ReviewFinding::on(wise::REVIEW_AMBIGUOUS_LINK, t)
            }),
            [] if *tag == KIND_STRIPE_PAYOUT => {
                review.push_to_review(ReviewFinding::on(wise::REVIEW_STRIPE_PAYOUT_UNMATCHED, t))
            }
            [] => match matching_rule(&rules, t, &rule_accounts, ledger) {
                Some(rule) => review.push_proposal(common(
                    Proposal::Rule {
                        bank_transaction_id: t.id,
                        rule_id: rule.id,
                        counterparty_account_id: rule.counterparty_account_id,
                    },
                    t,
                )),
                None => review.push_to_review(ReviewFinding::on(REVIEW_NO_RULE, t)),
            },
        }
        debug_assert!(*tag == KIND_STRIPE_PAYOUT || *tag == KIND_TRANSFER);
    }

    // 3. Écarts propres (AC-6).
    let all = reads::transactions_of_accounts(&mut **tx, company_id, &own_ids).await?;
    for t in wise::fees_without_origin(&loaded.pending, &all) {
        review.push_gap(ReviewFinding::on(wise::GAP_FEE_WITHOUT_ORIGIN, t));
    }
    let fiscal_year = reads::find_fiscal_year(&mut **tx, company_id, query.fiscal_year_id)
        .await?
        .ok_or(AppError::ReportFiscalYearNotFound {
            fiscal_year_id: query.fiscal_year_id,
        })?;
    let statements = reads::opening_statements_in_period(
        &mut **tx,
        company_id,
        &own_ids,
        fiscal_year.start_date,
        fiscal_year.end_date,
    )
    .await?;
    for s in statements {
        let gap = bank_imports::continuity_gap(
            &mut *tx,
            company_id,
            s.bank_account_id,
            s.period_from,
            s.opening_balance,
            Some(s.id),
        )
        .await?;
        if let Some(gap) = gap {
            let currency = balances
                .get(&s.bank_account_id)
                .map(|b| b.currency().to_string())
                .unwrap_or_else(|| BASE_CURRENCY.to_string());
            review.push_gap(ReviewFinding {
                code: wise::GAP_BALANCE_GAP,
                transaction: None,
                entries: Vec::new(),
                details: Some(serde_json::json!({
                    "bankAccountId": s.bank_account_id,
                    "importId": s.id,
                    "date": s.period_from.to_string(),
                    "currency": currency,
                    "expected": gap.expected.to_string(),
                    "found": gap.found.to_string(),
                    "previousImportId": gap.previous_import_id,
                })),
            });
        }
    }

    review.sort();
    // `map_proposals` ne sait pas échouer : la première erreur est gardée
    // à part, la revue construite avec elle est jetée.
    let failure = std::cell::RefCell::new(None);
    let mapped = review.map_proposals(|p| {
        let (proposal, error) = to_review_proposal(p, &shown);
        if let Some(e) = error {
            failure.borrow_mut().get_or_insert(e);
        }
        proposal
    });
    match failure.into_inner() {
        Some(e) => Err(e),
        None => Ok(mapped),
    }
}

/// Ce que `shown` tient pour la ligne principale d'un groupe : ses lignes et
/// le montant du marchand. Un groupe sans entrée est un défaut de
/// construction de [`build_wise_review`] (inatteignable : chaque groupe
/// y est inséré avant d'être proposé) : erreur interne tracée, jamais une
/// valeur vide que l'écran montrerait comme un montant absent.
fn card_group_display(
    bank_transaction_ids: &[i64],
    shown: &HashMap<i64, (Vec<TransactionSummary>, String)>,
) -> Result<(Vec<TransactionSummary>, String), AppError> {
    bank_transaction_ids
        .first()
        .and_then(|id| shown.get(id))
        .cloned()
        .ok_or_else(|| {
            tracing::error!(
                ?bank_transaction_ids,
                "revue Wise : groupe de carte sans lignes d'affichage"
            );
            AppError::Internal("revue Wise : groupe de carte sans lignes d'affichage".into())
        })
}

/// Le type de revue d'une proposition : une proposition commune telle
/// quelle ; un `card_group`, avec ce que `shown` tient pour sa ligne
/// principale (lignes et montant du marchand, posés pendant la boucle des
/// groupes de [`build_wise_review`]). Rend aussi l'erreur interne d'un groupe
/// sans lignes d'affichage : la proposition vide rendue avec elle ne sert
/// qu'à finir l'itération, la revue est alors jetée.
fn to_review_proposal(
    p: WiseProposal,
    shown: &HashMap<i64, (Vec<TransactionSummary>, String)>,
) -> (WiseReviewProposal, Option<AppError>) {
    match p {
        WiseProposal::Common(c) => (WiseReviewProposal::Common(c), None),
        WiseProposal::CardGroup(g) => {
            let CardGroupProposal::CardGroup {
                bank_transaction_ids,
                ..
            } = &g;
            let (shown_of, failed) = match card_group_display(bank_transaction_ids, shown) {
                Ok(v) => (v, None),
                Err(e) => ((Vec::new(), String::new()), Some(e)),
            };
            let (lines, merchant_amount) = shown_of;
            let review = WiseReviewProposal::CardGroup(CardGroupReview {
                proposal: g,
                lines,
                merchant_amount,
            });
            (review, failed)
        }
    }
}

/// Les détails du motif
/// [`wise::REVIEW_CARD_GROUP_PARTLY_RECONCILED`]
/// (`reconciledBankTransactionIds`, `journalEntryIds`) quand une ligne **de
/// carte** (`CARD-<id>`, frais exclus) du paiement `card_id` est déjà
/// rapprochée sur un solde de `own`, ou `None`.
///
/// Un frais `FEE-CARD-<id>` déjà rapproché n'y compte pas (revue de code P3
/// de la 39-11c, L1) : passé seul au compte de frais, il n'est pas dans le
/// groupe des lignes en attente, que l'écriture du groupe crédite sans lui —
/// aucun montant compté deux fois.
async fn reconciled_card_lines(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    own: &[i64],
    card_id: &str,
) -> Result<Option<serde_json::Value>, DbError> {
    let done =
        reads::reconciled_by_transaction_ids(&mut *conn, company_id, own, &[card_id.to_string()])
            .await?;
    if done.is_empty() {
        return Ok(None);
    }
    let mut entries: Vec<i64> = done.iter().filter_map(|t| t.matched_entry_id).collect();
    entries.sort_unstable();
    entries.dedup();
    Ok(Some(serde_json::json!({
        "reconciledBankTransactionIds": done.iter().map(|t| t.id).collect::<Vec<_>>(),
        "journalEntryIds": entries,
    })))
}

/// Ce qui empêche d'écrire un paiement par carte groupé, ou `None` : le
/// motif (`code` de la revue, `details.reason` de l'acceptation) et ses
/// détails, **les mêmes** des deux côtés — la revue et l'acceptation
/// l'appellent toutes deux (revues de code P1 et P2 de la 39-11c).
///
/// 1. [`wise::REVIEW_CARD_GROUP_PARTLY_RECONCILED`] : une ligne **de carte**
///    du même paiement (`CARD-<id>`) est déjà rapprochée, sur un solde du
///    compte Wise — une partielle dont une part, importée d'abord, a été
///    passée seule ; écrire le groupe en ferait une seconde écriture du
///    paiement (`details.reconciledBankTransactionIds`,
///    `details.journalEntryIds` : [`reconciled_card_lines`], qui ne compte pas
///    un frais passé seul). Aucun montant n'y serait compté deux fois — le
///    groupe ne réunit que les lignes en attente, son marchand celui de ces
///    seules lignes —, mais le motif garde la proposition à une écriture par
///    paiement. Il ne vaut que pour un groupe : la ligne de carte seule d'un
///    paiement converti, hors groupe (la part en CHF, sans segment, arrivée
///    après le groupe de sa part en devise), suit le classement ligne à ligne
///    et se propose par règle, au montant de sa seule part — comme la reprise
///    à la main de l'amendement A-3 (revue de code P3 de la 39-11c, M1).
/// 2. [`wise::REVIEW_CARD_GROUP_STATEMENTS_MISSING`] : un solde sans ligne dans
///    le groupe peut en avoir une qui n'est pas encore importée
///    (`details.missingBankAccountIds`) — une conversion partielle, sans sa
///    ligne, ressemble à une conversion totale. Couverture jugée par
///    [`bank_imports::date_coverage`] : un relevé couvre la date, ou la
///    continuité des soldes l'atteste. Exigée du solde en CHF toujours (la
///    partielle le débite d'abord) ; d'un solde en devise quand ses relevés
///    encadrent la date sans se suivre (`DateCoverage::Gap`). Avec les soldes,
///    leurs devises (`details.missingCurrencies`, Story 39-11d2 AC-1 : la
///    phrase de l'écran les nomme), triées avec eux par identifiant —
///    `missingCurrencies[i]` est la devise de `missingBankAccountIds[i]`.
///
/// ⚠️ Limites : un solde en devise dont aucun relevé n'encadre la date (rien
/// d'importé après elle, ou rien avant) n'est pas exigé — sinon un solde
/// dormant, ou ouvert après la date, mettrait tout paiement « à examiner » ;
/// une partielle sur deux soldes en devise dont l'une n'est pas encore
/// importée se propose alors sans elle, et sa part arrivée ensuite formerait
/// un second groupe, refusé par le motif 1 (une part rapprochée) ;
/// un frais `FEE-CARD` importé plus tard par un second fichier du **même**
/// solde que la carte se passe seul, au compte de frais (le groupe ne l'a pas
/// compté). La continuité (`closing == opening`) tient pour preuve qu'aucune
/// ligne ne manque, même quand des lignes absentes s'annulent (solde net
/// nul) ; et un solde en devise dont les relevés encadrent la date sans se
/// suivre met « à examiner » tout paiement daté entre eux, même sans rapport
/// avec lui.
async fn card_group_blocker<'b>(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    balances: impl Iterator<Item = (i64, &'b str)>,
    group: &wise::CardGroup<'_>,
) -> Result<Option<(&'static str, serde_json::Value)>, DbError> {
    let balances: Vec<(i64, &str)> = balances.collect();
    let own: Vec<i64> = balances.iter().map(|(id, _)| *id).collect();
    if let Some(details) =
        reconciled_card_lines(&mut *conn, company_id, &own, &group.card_id).await?
    {
        return Ok(Some((wise::REVIEW_CARD_GROUP_PARTLY_RECONCILED, details)));
    }
    let date = payment_date(group.principal());
    let present: HashSet<i64> = group.lines.iter().map(|t| t.bank_account_id).collect();
    let mut missing = Vec::new();
    for (id, currency) in balances {
        if present.contains(&id) {
            continue;
        }
        let coverage = bank_imports::date_coverage(&mut *conn, company_id, id, date).await?;
        let required = match coverage {
            DateCoverage::Covered => false,
            // Des relevés de part et d'autre de la date, qui ne se suivent pas.
            DateCoverage::Gap => true,
            // Le solde en CHF : rien n'atteste la date, il est exigé.
            DateCoverage::NoPrevious | DateCoverage::NothingAfter => currency == BASE_CURRENCY,
        };
        if required {
            missing.push((id, currency));
        }
    }
    if missing.is_empty() {
        return Ok(None);
    }
    // Paires triées ensemble : chaque devise reste celle de son solde.
    missing.sort_unstable();
    let ids: Vec<i64> = missing.iter().map(|(id, _)| *id).collect();
    let currencies: Vec<&str> = missing.iter().map(|(_, c)| *c).collect();
    Ok(Some((
        wise::REVIEW_CARD_GROUP_STATEMENTS_MISSING,
        serde_json::json!({
            "missingBankAccountIds": ids,
            "missingCurrencies": currencies,
        }),
    )))
}

/// Ajoute à l'objet `into` les champs de l'objet `extra`.
fn merge(into: &mut serde_json::Value, extra: serde_json::Value) {
    if let (Some(into), serde_json::Value::Object(extra)) = (into.as_object_mut(), extra) {
        into.extend(extra);
    }
}

/// Retire de la revue toute proposition, tout écart et toute ligne à examiner
/// qui nomment une transaction de `ids` (les lignes d'un paiement groupé, que
/// la revue commune a classées une à une).
fn remove_transactions(review: &mut Review<WiseProposal>, ids: &HashSet<i64>) {
    if ids.is_empty() {
        return;
    }
    review.proposals.retain(|p| {
        let named = match &p.proposal {
            WiseProposal::Common(c) => c.transaction_ids(),
            WiseProposal::CardGroup(_) => Vec::new(),
        };
        !named.iter().any(|id| ids.contains(id))
    });
    let keep = |f: &ReviewFinding| f.transaction.as_ref().is_none_or(|t| !ids.contains(&t.id));
    review.gaps.retain(keep);
    review.to_review.retain(keep);
    review.specific.retain(|(id, _)| !ids.contains(id));
}

// ---------------------------------------------------------------------------
// Acceptation
// ---------------------------------------------------------------------------

/// `POST /api/v1/wise/accept` — voir l'en-tête du module.
pub async fn accept_proposals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    AcceptBodyExtractor(body): AcceptBodyExtractor<WiseProposal>,
) -> Result<Json<ProviderAcceptResponse>, AppError> {
    let prepared =
        provider_accept::prepare_accept(&state.pool, &current_user, &SPEC, body, &[]).await?;
    if prepared.is_empty() {
        return Ok(Json(prepared.finish(BatchOutcome::default())));
    }
    let outcome = crate::retry::retry_app_on_deadlock("wise::accept", || {
        provider_accept::accept_once(&state.pool, &SPEC, &prepared)
    })
    .await?;
    Ok(Json(prepared.finish(outcome)))
}

impl EngineProposal for WiseProposal {
    fn transaction_ids(&self) -> Vec<i64> {
        match self {
            Self::Common(p) => p.transaction_ids(),
            Self::CardGroup(CardGroupProposal::CardGroup {
                bank_transaction_ids,
                ..
            }) => bank_transaction_ids.clone(),
        }
    }

    fn primary_transaction_id(&self) -> i64 {
        match self {
            Self::Common(p) => p.primary_transaction_id(),
            // Le pré-vol refuse une liste vide (`check_shape`).
            Self::CardGroup(CardGroupProposal::CardGroup {
                bank_transaction_ids,
                ..
            }) => bank_transaction_ids.first().copied().unwrap_or(0),
        }
    }

    fn entries_to_lock(&self) -> Vec<i64> {
        match self {
            Self::Common(p) => EngineProposal::entries_to_lock(p),
            Self::CardGroup(_) => Vec::new(),
        }
    }

    fn check_shape(&self) -> Result<(), String> {
        match self {
            Self::Common(p) => EngineProposal::check_shape(p),
            Self::CardGroup(CardGroupProposal::CardGroup {
                bank_transaction_ids,
                rule_id,
                counterparty_account_id,
            }) => {
                if bank_transaction_ids.is_empty() {
                    return Err("bankTransactionIds est vide (type='card_group')".into());
                }
                if bank_transaction_ids.iter().any(|id| *id <= 0)
                    || *rule_id <= 0
                    || *counterparty_account_id <= 0
                {
                    return Err(
                        "chaque identifiant doit être strictement positif (type='card_group')"
                            .into(),
                    );
                }
                Ok(())
            }
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
        match self {
            Self::Common(p) => Box::pin(provider_accept::accept_common(tx, ctx, p)),
            Self::CardGroup(CardGroupProposal::CardGroup {
                bank_transaction_ids,
                rule_id,
                counterparty_account_id,
            }) => Box::pin(accept_card_group(
                tx,
                ctx,
                bank_transaction_ids,
                *rule_id,
                *counterparty_account_id,
            )),
        }
    }
}

/// Un refus d'une ligne du groupe, ramené à la transaction que nomme la
/// proposition (la première de la liste) : sinon il disparaîtrait de la
/// réponse (en-tête du trait `EngineProposal`).
fn naming(primary: i64, r: Refusal) -> Refusal {
    match r {
        Refusal::Failed(mut f) => {
            f.bank_transaction_id = primary;
            Refusal::Failed(f)
        }
        abort => abort,
    }
}

/// Une ligne de l'écriture d'un paiement groupé.
fn entry_line(
    account_id: i64,
    debit: Decimal,
    credit: Decimal,
    foreign: Option<ForeignAmount>,
) -> NewJournalEntryLine {
    NewJournalEntryLine {
        account_id,
        debit,
        credit,
        project_id: None,
        foreign,
    }
}

/// `card_group` : un paiement par carte converti, en **une** écriture
/// (amendement A-1). Rejugement sous les verrous : chaque ligne en attente,
/// sur un compte du compte Wise, de genre frais ou autre ; l'ensemble des
/// lignes en attente du paiement, relu, égal à la proposition et toujours
/// écrivable (sinon périmée, `card_group_changed`) ; la règle active, de même
/// contrepartie, qui correspond toujours à la ligne principale (sinon
/// `rule_no_longer_matches`). Contrepartie : celle d'une règle, en CHF
/// (`COUNTERPARTY_CURRENCY_MISMATCH`), jamais le compte du plan d'un solde.
/// Écriture : la charge au montant du marchand, chaque solde crédité de sa
/// part (cours mensuel retenu pour un solde en devise), la différence sur le
/// compte de perte ou de gain de change de la société.
async fn accept_card_group(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    ids: &[i64],
    rule_id: i64,
    counterparty_account_id: i64,
) -> Result<ProviderAccepted, Refusal> {
    let primary = ids[0];
    let card_or_fee = |k| matches!(k, ProviderKind::Fee | ProviderKind::Other);
    let mut lines: Vec<BankTransaction> = Vec::with_capacity(ids.len());
    let mut ledgers: HashMap<i64, (i64, String)> = HashMap::new();
    for id in ids {
        let r = rejudge(tx, ctx, *id, card_or_fee)
            .await
            .map_err(|r| naming(primary, r))?;
        ledgers.insert(
            r.balance.account.id,
            (r.ledger_account_id, r.balance.currency().to_string()),
        );
        lines.push(r.transaction);
    }
    // Le même paiement, et tout le paiement.
    let card_ids: HashSet<&str> = lines.iter().filter_map(wise::card_id_of).collect();
    let [card_id] = card_ids.into_iter().collect::<Vec<_>>()[..] else {
        return Err(stale(primary, "card_group_changed"));
    };
    let wanted = vec![
        card_id.to_string(),
        format!("{}{card_id}", wise::FEE_PREFIX),
    ];
    let own: Vec<i64> = ctx.balances.keys().copied().collect();
    let current = reads::pending_by_transaction_ids(&mut **tx, ctx.company_id, &own, &wanted)
        .await
        .map_err(|e| db_refusal(primary, e))?;
    let current_ids: HashSet<i64> = current.iter().map(|t| t.id).collect();
    let proposed: HashSet<i64> = ids.iter().copied().collect();
    if current_ids != proposed {
        return Err(stale(primary, "card_group_changed"));
    }
    let groups = wise::card_groups(&lines);
    let [group] = groups.as_slice() else {
        return Err(stale(primary, "card_group_changed"));
    };
    let CardGroupJudgement::Valid { merchant_chf } = group.judgement else {
        return Err(stale(primary, "card_group_changed"));
    };
    let principal = group.principal();
    // Une part déjà rapprochée, une ligne peut-être pas encore importée : le
    // même jugement que la revue.
    let blocker = card_group_blocker(
        &mut *tx,
        ctx.company_id,
        ctx.balances.iter().map(|(id, b)| (*id, b.currency())),
        group,
    )
    .await
    .map_err(|e| db_refusal(primary, e))?;
    if let Some((reason, extra)) = blocker {
        let mut details = serde_json::json!({ "reason": reason });
        merge(&mut details, extra);
        return Err(failed(primary, PROVIDER_PROPOSAL_STALE, details));
    }

    // La règle, rejugée sur la ligne principale.
    let rule: Option<ReconciliationRule> =
        reconciliation_rules::find_by_id_for_company(&mut **tx, ctx.company_id, rule_id)
            .await
            .map_err(|e| db_refusal(primary, e))?;
    let Some(rule) = rule.filter(|r| {
        r.active
            && r.counterparty_account_id == counterparty_account_id
            && rule_matches(r, principal)
    }) else {
        return Err(stale(primary, "rule_no_longer_matches"));
    };
    for (ledger, _) in ledgers.values() {
        if *ledger == counterparty_account_id {
            return Err(Refusal::Failed(
                counterparty_is_bank_ledger_failed_proposal(primary),
            ));
        }
    }
    let principal_ledger = ledgers
        .get(&principal.bank_account_id)
        .map(|(l, _)| *l)
        .unwrap_or_default();
    let currency = check_counterparty(
        tx,
        ctx,
        primary,
        counterparty_account_id,
        principal_ledger,
        CounterpartyRule::Rule,
    )
    .await?;
    if let Some(c) = currency.as_deref()
        && c != BASE_CURRENCY
    {
        return Err(failed(
            primary,
            "COUNTERPARTY_CURRENCY_MISMATCH",
            serde_json::json!({
                "accountId": counterparty_account_id,
                "accountCurrency": c,
                "transactionCurrency": BASE_CURRENCY,
            }),
        ));
    }
    if let Some(pid) = rule.default_project_id
        && let Err(e) = projects::validate_taggable_in_tx(tx, ctx.company_id, &[pid]).await
    {
        if kesh_db::retry::is_deadlock_error(&e) {
            return Err(Refusal::Abort(e));
        }
        return Err(Refusal::Failed(project_error_to_failed_proposal(
            primary,
            Some(pid),
            e,
        )));
    }

    // Les lignes : la charge, la part de chaque solde, la différence.
    let date = payment_date(principal);
    let mut entry_lines = vec![entry_line(
        counterparty_account_id,
        merchant_chf,
        Decimal::ZERO,
        None,
    )];
    let mut credited = Decimal::ZERO;
    for (account, share) in wise::shares_by_account(&group.lines) {
        let Some((ledger, currency)) = ledgers.get(&account) else {
            return Err(stale(primary, "card_group_changed"));
        };
        let (chf, foreign) = if currency == BASE_CURRENCY {
            (share, None)
        } else {
            let rate = month_rate(tx, ctx.company_id, primary, currency, date).await?;
            let chf = kesh_core::accounting::fx::fx_chf(&share, &rate.rate).map_err(|e| {
                failed(
                    primary,
                    "FX_RATE_INVALID",
                    serde_json::json!({ "currency": currency, "message": format!("{e:?}") }),
                )
            })?;
            (
                chf,
                Some(ForeignAmount {
                    declared_currency: None,
                    amount: share,
                    rate: rate.rate,
                    source: Some(rate.source),
                    rate_derived: false,
                }),
            )
        };
        credited += chf;
        entry_lines.push(entry_line(*ledger, Decimal::ZERO, chf, foreign));
    }
    let difference = credited - merchant_chf;
    if !difference.is_zero() {
        let (gain, loss) = reads::fx_difference_accounts(&mut **tx, ctx.company_id)
            .await
            .map_err(|e| db_refusal(primary, e))?;
        let (field, account) = if difference > Decimal::ZERO {
            (FxAccountField::Loss, loss)
        } else {
            (FxAccountField::Gain, gain)
        };
        let Some(account) = account else {
            return Err(entry_refusal(
                primary,
                DbError::FxAccountInvalid {
                    field,
                    reason: FxAccountReason::Missing,
                    holder_account_id: None,
                },
            ));
        };
        entry_lines.push(if difference > Decimal::ZERO {
            entry_line(account, difference, Decimal::ZERO, None)
        } else {
            entry_line(account, Decimal::ZERO, -difference, None)
        });
    }
    let description: String = (ctx.spec.entry_label)(EntryKind::Rule, ctx.account, principal)
        .chars()
        .take(DESCRIPTION_MAX_CHARS)
        .collect();
    let new = NewJournalEntry {
        company_id: ctx.company_id,
        entry_date: date,
        journal: Journal::Banque,
        description,
        project_id: rule.default_project_id,
        lines: entry_lines,
    };
    let fiscal_year = fiscal_years::find_open_covering_date(tx, ctx.company_id, date)
        .await
        .map_err(|e| db_refusal(primary, e))?;
    let Some(fiscal_year) = fiscal_year else {
        return Err(failed(
            primary,
            "RECONCILIATION_FISCAL_YEAR_CLOSED",
            serde_json::json!({ "entryDate": date.to_string() }),
        ));
    };
    // Flux de rapprochement : garde de postabilité désactivée (14-3b, D-A0),
    // la contrepartie est contrôlée avant.
    let entry_id = journal_entries::create_in_tx(tx, fiscal_year.id, ctx.acteur, new, false)
        .await
        .map(|j| j.entry.id)
        .map_err(|e| entry_refusal(primary, e))?;
    for t in &group.lines {
        mark_reconciled(tx, ctx, t, entry_id)
            .await
            .map_err(|r| naming(primary, r))?;
    }
    record_rule_applied(tx, ctx, &rule, principal, entry_id)
        .await
        .map_err(|r| naming(primary, r))?;
    Ok(ProviderAccepted {
        kind: "card_group",
        bank_transaction_ids: group.ids(),
        journal_entry_ids: vec![entry_id],
        rule_id: Some(rule_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un groupe sans entrée dans `shown` (ou sans identifiant) est une
    /// erreur interne, non des lignes vides.
    #[test]
    fn groupe_sans_lignes_d_affichage_est_une_erreur_interne() {
        let shown = HashMap::new();
        assert!(matches!(
            card_group_display(&[7, 8], &shown),
            Err(AppError::Internal(_))
        ));
        assert!(matches!(
            card_group_display(&[], &shown),
            Err(AppError::Internal(_))
        ));
    }

    /// Le groupe connu rend ses lignes et son montant.
    #[test]
    fn groupe_connu_rend_ses_lignes() {
        let mut shown = HashMap::new();
        shown.insert(7, (Vec::new(), "12.50".to_string()));
        let (lines, amount) = card_group_display(&[7, 8], &shown).expect("connu");
        assert!(lines.is_empty());
        assert_eq!(amount, "12.50");
    }

    /// Un groupe absent de `shown` rend l'erreur interne avec la proposition
    /// de repli (revue P2 de la 39-11d2, LOW-A) : c'est elle que
    /// [`build_wise_review`] garde pour jeter la revue ; l'ignorer rendrait
    /// visible une proposition sans lignes ni montant.
    #[test]
    fn groupe_absent_de_shown_rend_l_erreur_interne() {
        let shown = HashMap::new();
        let p = WiseProposal::CardGroup(CardGroupProposal::CardGroup {
            bank_transaction_ids: vec![7, 8],
            rule_id: 3,
            counterparty_account_id: 4,
        });
        let (review, error) = to_review_proposal(p, &shown);
        assert!(
            matches!(error, Some(AppError::Internal(_))),
            "erreur interne attendue, obtenu {error:?}"
        );
        assert!(matches!(
            review,
            WiseReviewProposal::CardGroup(CardGroupReview { ref lines, ref merchant_amount, .. })
                if lines.is_empty() && merchant_amount.is_empty()
        ));

        // Une proposition commune n'a pas besoin de `shown` : aucune erreur.
        let common = WiseProposal::Common(Proposal::Rule {
            bank_transaction_id: 9,
            rule_id: 3,
            counterparty_account_id: 4,
        });
        let (_, error) = to_review_proposal(common, &shown);
        assert!(error.is_none(), "aucune erreur attendue, obtenu {error:?}");
    }
}
