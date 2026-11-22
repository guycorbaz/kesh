//! Routes de la **revue** et de l'**acceptation** Stripe — Story 40-7c (Epic
//! 40, phase 1), gestionnaires minces sur le moteur commun des prestataires
//! ([`super::provider_accept`]) avec le paramétrage Stripe
//! ([`kesh_reconciliation::stripe::SPEC`]).
//!
//! - `GET /api/v1/stripe/review?account=<libellé>&fiscalYearId=<id>` —
//!   Comptable et plus, clés d'API admises en lecture ; lecture seule, **une**
//!   transaction de lecture : la revue commune (charges → ventes, frais), puis
//!   les lectures propres à Stripe dans le même instantané (lignes des
//!   versements, comptes de versement, versements sans réception) et la revue
//!   Stripe ([`kesh_reconciliation::stripe::complete_review`]). `404
//!   PROVIDER_ACCOUNT_NOT_FOUND`, `404 FISCAL_YEAR_NOT_FOUND`.
//! - `POST /api/v1/stripe/accept` — Comptable et plus, clés d'API
//!   `read-write` ; corps `{ account, proposals: [ {type, …} ] }` (`link_entry`,
//!   `fee`, `payout` ; tout autre type → `400 VALIDATION_ERROR`), réponse `{
//!   accepted, failed }` (*Pattern batch*). Les comptes de versement réglés sur
//!   le compte Stripe sont lus **avant** les verrous et verrouillés avec les
//!   comptes des transactions ; la proposition `payout` relit sous les verrous
//!   le compte de versement réglé ([`accept_payout`]).
//!
//! ⛔ Le gestionnaire d'acceptation s'appelle [`accept_proposals`], **jamais**
//! `post_accept` : l'exemption `RETRY_WITH_AUTORISE` du registre des routes
//! désigne une fonction par son seul nom. Il rejoue sur interblocage par
//! l'enveloppe `crate::retry::retry_app_on_deadlock` (opération
//! `stripe::accept`), non par `retry_with`.
//!
//! Une proposition qu'on n'accepte pas n'est pas perdue (décision du
//! responsable de l'epic du 21.11, (c)) : sa transaction reste `pending`, la
//! revue suivante la propose de nouveau, et elle se traite à la main par
//! l'API — écriture en devise (`POST /journal-entries`), puis rattachement
//! (`POST /reconciliation/transactions/{id}/link-entry`).

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use axum::extract::{Query, State};
use axum::{Extension, Json};
use kesh_db::entities::BankAccount;
use kesh_db::errors::DbError;
use kesh_db::repositories::{bank_accounts, provider_review as reads};
use kesh_reconciliation::provider_review::{EntryKind, Proposal, ProviderKind, Review};
use kesh_reconciliation::stripe::{
    KIND_PAYOUT, PAYOUT_CATEGORY, PayoutDestination, PayoutVerdict, SPEC, StripeReviewInput,
    StripeReviewProposal, classify, complete_review, judge_payout, payout_lines,
};
use serde::Deserialize;
use sqlx::MySql;

use super::provider_accept::{
    self, AcceptBodyExtractor, AcceptContext, BatchOutcome, CounterpartyRule, EngineProposal,
    ProviderAcceptResponse, ProviderAccepted, ProviderEntry, Refusal, ReviewQuery, ReviewResponse,
    accept_common, check_counterparty, db_refusal, mark_reconciled, rejudge, stale,
    write_provider_entry,
};
use super::reconciliation::{FailedProposal, closed_account_failed};
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

// ---------------------------------------------------------------------------
// Revue
// ---------------------------------------------------------------------------

/// `GET /api/v1/stripe/review` — voir l'en-tête du module.
pub async fn review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReviewQuery>,
) -> Result<Json<ReviewResponse<StripeReviewProposal>>, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let review = load_review(&mut tx, current_user.company_id, &query).await;
    // Lecture seule : rien à conclure.
    let _ = tx.rollback().await;
    Ok(Json(ReviewResponse::of(
        query.account,
        query.fiscal_year_id,
        review?,
    )))
}

/// Le libellé d'un compte bancaire de versement : sa banque, et sa
/// description si elle existe.
fn destination_label(a: &BankAccount) -> String {
    match a.description.as_deref() {
        Some(d) => format!("{} · {d}", a.bank_name),
        None => a.bank_name.clone(),
    }
}

/// Un compte de versement est-il **utilisable** pour une écriture : non
/// archivé, ouvert, lié à un compte du plan. (Type et devise de son compte du
/// plan se contrôlent à l'acceptation, `check_counterparty`.)
fn destination_usable(a: &BankAccount) -> bool {
    !a.archived && !a.is_closed() && a.journal_account_id.is_some()
}

/// Les lectures et la revue Stripe, dans la transaction de lecture.
async fn load_review(
    tx: &mut sqlx::Transaction<'_, MySql>,
    company_id: i64,
    query: &ReviewQuery,
) -> Result<Review<StripeReviewProposal>, AppError> {
    let loaded = provider_accept::load_review_in_tx(
        tx,
        company_id,
        &SPEC,
        &query.account,
        query.fiscal_year_id,
    )
    .await?;
    // Relu dans le même instantané : `load_review_in_tx` l'a trouvé.
    let fiscal_year = reads::find_fiscal_year(&mut **tx, company_id, query.fiscal_year_id)
        .await?
        .ok_or(AppError::ReportFiscalYearNotFound {
            fiscal_year_id: query.fiscal_year_id,
        })?;
    let own_ids: Vec<i64> = loaded.balances.iter().map(|b| b.account.id).collect();

    // Les lignes des versements pending : une requête pour tous les comptes.
    let payout_ids: Vec<String> = loaded
        .pending
        .iter()
        .filter(|t| classify(t) == ProviderKind::Specific(KIND_PAYOUT))
        .filter_map(|t| t.transaction_id.clone())
        .collect();
    let payout_candidates =
        reads::transactions_of_groups(tx, company_id, &own_ids, &payout_ids).await?;

    // Les comptes de versement réglés, et ceux qui sont utilisables.
    let configured: HashMap<i64, i64> = loaded
        .balances
        .iter()
        .filter_map(|b| {
            b.account
                .provider_settings
                .payout_bank_account_id
                .map(|d| (b.account.id, d))
        })
        .collect();
    let mut destination_ids: Vec<i64> = configured.values().copied().collect();
    destination_ids.sort_unstable();
    destination_ids.dedup();
    let found: HashMap<i64, BankAccount> =
        bank_accounts::find_by_ids(&mut **tx, company_id, &destination_ids)
            .await?
            .into_iter()
            .map(|a| (a.id, a))
            .collect();
    let destinations: HashMap<i64, PayoutDestination> = configured
        .iter()
        .filter_map(|(stripe_id, dest_id)| {
            found
                .get(dest_id)
                .filter(|a| destination_usable(a))
                .map(|a| {
                    (
                        *stripe_id,
                        PayoutDestination {
                            bank_account_id: a.id,
                            label: destination_label(a),
                        },
                    )
                })
        })
        .collect();

    let unreceived = reads::reconciled_without_other_side(
        &mut **tx,
        company_id,
        &own_ids,
        PAYOUT_CATEGORY,
        fiscal_year.start_date,
        fiscal_year.end_date,
    )
    .await?;

    Ok(complete_review(
        loaded.review,
        &StripeReviewInput {
            pending: &loaded.pending,
            payout_candidates: &payout_candidates,
            destinations: &destinations,
            configured_destinations: &configured,
            unreceived_payouts: &unreceived,
        },
    ))
}

// ---------------------------------------------------------------------------
// Acceptation
// ---------------------------------------------------------------------------

/// La proposition propre à Stripe, telle que le corps de l'acceptation la
/// porte : le versement, et — facultatif — le compte de versement que la revue
/// a montré : s'il n'est plus celui des réglages, la proposition est périmée.
/// Les champs d'affichage de la revue (`payoutAccountLabel`, `transaction`)
/// sont ignorés : la proposition de la revue se renvoie telle quelle.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum OwnProposal {
    /// Écrire le versement vers le compte de versement réglé.
    Payout {
        /// Le versement.
        bank_transaction_id: i64,
        /// Le compte de versement que la revue a montré.
        #[serde(default)]
        payout_bank_account_id: Option<i64>,
    },
}

/// Une proposition du corps de l'acceptation Stripe : commune (`link_entry`,
/// `fee` ; les autres genres communs sont refusés au pré-vol) ou `payout`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StripeProposal {
    /// Une proposition commune.
    Common(Proposal),
    /// Un versement.
    Own(OwnProposal),
}

impl EngineProposal for StripeProposal {
    fn transaction_ids(&self) -> Vec<i64> {
        match self {
            Self::Common(p) => EngineProposal::transaction_ids(p),
            Self::Own(OwnProposal::Payout {
                bank_transaction_id,
                ..
            }) => vec![*bank_transaction_id],
        }
    }

    fn primary_transaction_id(&self) -> i64 {
        match self {
            Self::Common(p) => EngineProposal::primary_transaction_id(p),
            Self::Own(OwnProposal::Payout {
                bank_transaction_id,
                ..
            }) => *bank_transaction_id,
        }
    }

    fn entries_to_lock(&self) -> Vec<i64> {
        match self {
            Self::Common(p) => EngineProposal::entries_to_lock(p),
            Self::Own(_) => Vec::new(),
        }
    }

    fn check_shape(&self) -> Result<(), String> {
        match self {
            Self::Common(p @ (Proposal::LinkEntry { .. } | Proposal::Fee { .. })) => {
                EngineProposal::check_shape(p)
            }
            Self::Common(p) => Err(format!(
                "type '{}' non admis pour un compte Stripe (admis : link_entry, fee, payout)",
                p.kind()
            )),
            Self::Own(OwnProposal::Payout {
                bank_transaction_id,
                payout_bank_account_id,
            }) => {
                if *bank_transaction_id <= 0 {
                    return Err(
                        "l'identifiant de transaction doit être strictement positif (type='payout')"
                            .into(),
                    );
                }
                if payout_bank_account_id.is_some_and(|d| d <= 0) {
                    return Err(
                        "payoutBankAccountId doit être strictement positif (type='payout')".into(),
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
            Self::Common(p) => Box::pin(accept_common(tx, ctx, p)),
            Self::Own(OwnProposal::Payout {
                bank_transaction_id,
                payout_bank_account_id,
            }) => Box::pin(accept_payout(
                tx,
                ctx,
                *bank_transaction_id,
                *payout_bank_account_id,
            )),
        }
    }
}

/// `payout` (AC-4 d) : le versement rejugé sous les verrous — pending, genre
/// versement, compte de versement réglé, **verrouillé** pour le lot (lu au
/// pré-vol) et égal à celui que la revue a montré s'il est donné, sinon périmé
/// (`provider_settings_changed`) ; même jugement que la revue
/// ([`judge_payout`], lignes relues sous les verrous : retour de versement,
/// montant positif, total faux → périmé sous le motif) ; compte de versement
/// relu : absent ou archivé → périmé, clos → `BANK_ACCOUNT_CLOSED`, sans
/// compte du plan → `BANK_ACCOUNT_NOT_CONFIGURED` ; son compte du plan
/// contrôlé comme une contrepartie d'actif **de la devise du solde**
/// (`CounterpartyRule::AssetInCurrency`). Écriture par la fonction commune
/// (débit compte de versement, crédit solde Stripe, net en devise, deux lignes
/// au même cours), puis rapprochement du versement.
///
/// Le compte de versement n'est pas gardé par `ensure_open_in_tx` : aucune
/// ligne ne lui est rattachée (seule l'écriture touche son compte du plan).
/// Le lot en tient le verrou nommé (`with_account_locks` : il exclut un
/// import, une acceptation ou un rattachement concurrents sur ce compte), mais
/// la relecture (`find_by_id_in_tx`) est cohérente et **non verrouillante** :
/// les routes de clôture, d'archivage et de réglage du compte ne prennent pas
/// ce verrou, et l'une d'elles peut passer entre la lecture et la validation.
/// L'écriture ne touche que le compte du plan : l'inventaire des comptes
/// clos reste juste.
async fn accept_payout(
    tx: &mut sqlx::Transaction<'_, MySql>,
    ctx: &AcceptContext<'_>,
    bank_transaction_id: i64,
    claimed_destination: Option<i64>,
) -> Result<ProviderAccepted, Refusal> {
    let id = bank_transaction_id;
    let r = rejudge(tx, ctx, id, |k| k == ProviderKind::Specific(KIND_PAYOUT)).await?;
    let configured = r.balance.account.provider_settings.payout_bank_account_id;
    let Some(destination_id) = configured.filter(|d| ctx.locked_accounts.contains(d)) else {
        return Err(stale(id, "provider_settings_changed"));
    };
    if claimed_destination.is_some_and(|c| c != destination_id) {
        return Err(stale(id, "provider_settings_changed"));
    }
    let payout_id = r.transaction.transaction_id.clone().unwrap_or_default();
    let candidates =
        reads::transactions_of_groups(tx, ctx.company_id, &[r.balance.account.id], &[payout_id])
            .await
            .map_err(|e| db_refusal(id, e))?;
    let lines = payout_lines(&r.transaction, &candidates);
    match judge_payout(&r.transaction, &lines, true) {
        PayoutVerdict::Proposable => {}
        PayoutVerdict::ToReview(code) => return Err(stale(id, code)),
        PayoutVerdict::TotalMismatch { .. } => {
            return Err(stale(
                id,
                kesh_reconciliation::stripe::GAP_PAYOUT_TOTAL_MISMATCH,
            ));
        }
    }
    let destination =
        match bank_accounts::find_by_id_in_tx(tx, ctx.company_id, destination_id).await {
            Ok(a) => a,
            Err(DbError::NotFound) => return Err(stale(id, "provider_settings_changed")),
            Err(e) => return Err(db_refusal(id, e)),
        };
    if destination.archived {
        return Err(stale(id, "payout_account_archived"));
    }
    if destination.is_closed() {
        return Err(Refusal::Failed(
            closed_account_failed(&[id], destination_id).remove(0),
        ));
    }
    let Some(destination_ledger) = destination.journal_account_id else {
        return Err(Refusal::Failed(FailedProposal {
            bank_transaction_id: id,
            error_code: "BANK_ACCOUNT_NOT_CONFIGURED".to_string(),
            details: Some(serde_json::json!({ "bankAccountId": destination_id })),
        }));
    };
    let currency = check_counterparty(
        tx,
        ctx,
        id,
        destination_ledger,
        r.ledger_account_id,
        CounterpartyRule::AssetInCurrency(r.balance.currency().to_string()),
    )
    .await?;
    let entry_id = write_provider_entry(
        tx,
        ctx,
        ProviderEntry {
            transaction: &r.transaction,
            ledger_account_id: r.ledger_account_id,
            ledger_currency: r.balance.currency(),
            counterparty_account_id: destination_ledger,
            counterparty_currency: currency.as_deref(),
            // Un versement est un retrait vers une banque ; le libellé Stripe
            // ne dépend pas du genre (`stripe::entry_label`).
            kind: EntryKind::Withdrawal,
            project_id: None,
            known_chf: None,
        },
    )
    .await?;
    mark_reconciled(tx, ctx, &r.transaction, entry_id).await?;
    Ok(ProviderAccepted {
        kind: "payout",
        bank_transaction_ids: vec![id],
        journal_entry_ids: vec![entry_id],
        rule_id: None,
    })
}

/// Les comptes de versement à verrouiller en plus pour le lot : ceux des
/// réglages (`configured`) **si et seulement si** le lot contient un
/// `payout`. Un lot de frais ou de rattachements ne les touche pas : les
/// verrouiller ferait attendre (et échouer en `409
/// RECONCILIATION_ACCOUNT_LOCKED`) un lot sans rapport avec un import qui
/// tient le compte de destination.
fn payout_destinations(
    configured: impl Iterator<Item = i64>,
    proposals: &[StripeProposal],
) -> Vec<i64> {
    if proposals
        .iter()
        .any(|p| matches!(p, StripeProposal::Own(_)))
    {
        configured.collect()
    } else {
        Vec::new()
    }
}

/// `POST /api/v1/stripe/accept` — voir l'en-tête du module.
pub async fn accept_proposals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    AcceptBodyExtractor(body): AcceptBodyExtractor<StripeProposal>,
) -> Result<Json<ProviderAcceptResponse>, AppError> {
    // Les comptes de versement réglés, lus avant les verrous : verrouillés
    // avec les comptes des transactions (un compte inconnu rend 404 au pré-vol).
    let balances = bank_accounts::list_provider_balances(
        &state.pool,
        current_user.company_id,
        SPEC.provider,
        &body.account,
    )
    .await?;
    let destinations = payout_destinations(
        balances
            .iter()
            .filter_map(|b| b.account.provider_settings.payout_bank_account_id),
        &body.proposals,
    );
    let prepared =
        provider_accept::prepare_accept(&state.pool, &current_user, &SPEC, body, &destinations)
            .await?;
    if prepared.is_empty() {
        return Ok(Json(prepared.finish(BatchOutcome::default())));
    }
    let outcome = crate::retry::retry_app_on_deadlock("stripe::accept", || {
        provider_accept::accept_once(&state.pool, &SPEC, &prepared)
    })
    .await?;
    Ok(Json(prepared.finish(outcome)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les constantes que la revue partage avec l'import (40-7b) et le schéma
    /// ne dérivent pas.
    #[test]
    fn constantes_partagees_avec_l_import() {
        assert_eq!(
            SPEC.provider,
            kesh_db::entities::bank_account::PROVIDER_STRIPE
        );
        assert_eq!(
            kesh_reconciliation::stripe::FEE_SUFFIX,
            super::super::provider_imports::FEE_SUFFIX
        );
        assert_eq!(
            super::super::stripe_imports::PAYOUT_PROVIDER_TYPE,
            PAYOUT_CATEGORY
        );
    }

    /// Les comptes de versement ne se verrouillent que pour un lot qui porte un
    /// `payout` (revue P1 L7).
    #[test]
    fn destinations_verrouillees_seulement_pour_un_lot_avec_versement() {
        let versement = StripeProposal::Own(OwnProposal::Payout {
            bank_transaction_id: 1,
            payout_bank_account_id: None,
        });
        let frais: StripeProposal = serde_json::from_value(serde_json::json!({
            "type": "fee", "bankTransactionId": 2, "feeAccountId": 3
        }))
        .unwrap();
        assert!(matches!(frais, StripeProposal::Common(_)));
        assert_eq!(
            payout_destinations([7, 8].into_iter(), std::slice::from_ref(&frais)),
            Vec::<i64>::new()
        );
        assert_eq!(
            payout_destinations([7, 8].into_iter(), &[frais, versement]),
            vec![7, 8]
        );
    }

    #[test]
    fn corps_d_une_proposition_stripe() {
        let p: StripeProposal = serde_json::from_value(serde_json::json!({
            "type": "payout", "bankTransactionId": 5, "payoutBankAccountId": 7,
            "payoutAccountLabel": "Banque", "transaction": { "id": 5 }
        }))
        .unwrap();
        assert_eq!(
            p,
            StripeProposal::Own(OwnProposal::Payout {
                bank_transaction_id: 5,
                payout_bank_account_id: Some(7)
            })
        );
        assert!(p.check_shape().is_ok());
        let sans: StripeProposal =
            serde_json::from_value(serde_json::json!({ "type": "payout", "bankTransactionId": 5 }))
                .unwrap();
        assert_eq!(
            sans,
            StripeProposal::Own(OwnProposal::Payout {
                bank_transaction_id: 5,
                payout_bank_account_id: None
            })
        );
        let fee: StripeProposal = serde_json::from_value(serde_json::json!({
            "type": "fee", "bankTransactionId": 1, "feeAccountId": 2
        }))
        .unwrap();
        assert!(matches!(fee, StripeProposal::Common(Proposal::Fee { .. })));
        assert!(fee.check_shape().is_ok());
        assert_eq!(fee.transaction_ids(), [1]);
        let lien: StripeProposal = serde_json::from_value(serde_json::json!({
            "type": "link_entry", "bankTransactionId": 1, "journalEntryId": 9
        }))
        .unwrap();
        assert_eq!(lien.entries_to_lock(), [9]);
        // Genres communs non admis pour Stripe : refusés au pré-vol (400).
        for refuse in [
            serde_json::json!({ "type": "rule", "bankTransactionId": 1, "ruleId": 2, "counterpartyAccountId": 3 }),
            serde_json::json!({ "type": "withdrawal", "bankTransactionId": 1, "counterpartyAccountId": 3 }),
            serde_json::json!({ "type": "conversion", "debitTransactionId": 1, "creditTransactionId": 2 }),
        ] {
            let p: StripeProposal = serde_json::from_value(refuse.clone()).unwrap();
            assert!(p.check_shape().is_err(), "{refuse}");
        }
        // Forme : identifiants positifs ; type inconnu illisible.
        let negatif: StripeProposal =
            serde_json::from_value(serde_json::json!({ "type": "payout", "bankTransactionId": 0 }))
                .unwrap();
        assert!(negatif.check_shape().is_err());
        let dest_negative: StripeProposal = serde_json::from_value(
            serde_json::json!({ "type": "payout", "bankTransactionId": 1, "payoutBankAccountId": -1 }),
        )
        .unwrap();
        assert!(dest_negative.check_shape().is_err());
        assert!(
            serde_json::from_value::<StripeProposal>(serde_json::json!({ "type": "inconnu" }))
                .is_err()
        );
    }

    #[test]
    fn libelle_et_usage_d_un_compte_de_versement() {
        let now = chrono::Utc::now().naive_utc();
        let mut a = BankAccount {
            id: 7,
            company_id: 1,
            bank_name: "Banque".into(),
            description: None,
            iban: None,
            qr_iban: None,
            account_kind: kesh_db::entities::bank_account::BankAccountKind::Bank,
            card_last4: None,
            is_primary: false,
            journal_account_id: Some(1),
            version: 1,
            archived: false,
            closed_on: None,
            provider_settings: Default::default(),
            created_at: now,
            updated_at: now,
        };
        assert_eq!(destination_label(&a), "Banque");
        assert!(destination_usable(&a));
        a.description = Some("EUR".into());
        assert_eq!(destination_label(&a), "Banque · EUR");
        a.journal_account_id = None;
        assert!(!destination_usable(&a));
        a.journal_account_id = Some(1);
        a.archived = true;
        assert!(!destination_usable(&a));
        a.archived = false;
        a.closed_on = Some(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert!(!destination_usable(&a));
    }
}
