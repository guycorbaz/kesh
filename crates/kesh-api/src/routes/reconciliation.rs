//! Routes API réconciliation bancaire — Story 8-4 (FR44).
//!
//! - `GET /api/v1/reconciliation/proposals?bankAccountId={id}&limit={n}` —
//!   liste les propositions de matching pour les transactions `pending`
//!   du compte. Read-only, **pas de mutex**. Un crédit reçoit des factures
//!   client, un débit des factures fournisseur (Story 30-1b, #686).
//! - `POST /api/v1/reconciliation/accept` — accepte un batch de
//!   propositions sous mutex. Body : `{ bankAccountId, proposals:
//!   [{ bankTransactionId, invoiceId }] }`. Partial success via
//!   savepoints MariaDB. Émet dual audit (`reconciliation.accepted`
//!   + `invoice.paid`).
//! - `POST /api/v1/reconciliation/reject` — marque les transactions
//!   comme manuellement revues (`auto_match_rejected_at = NOW()`)
//!   sous mutex. Body : `{ bankAccountId, bankTransactionIds }`.
//! - `POST /api/v1/reconciliation/transactions/{id}/unreject` — annule le
//!   rejet d'une transaction (`auto_match_rejected_at = NULL`) sous mutex,
//!   sans corps, `204` ; la transaction redevient proposable (Story 28-5a,
//!   #526 — [`post_unreject_transaction`]).
//!
//! **Lots, Story 27-4 (#634)** : une transaction du lot introuvable
//! (supprimée avec son import, d'un autre compte ou d'une autre société),
//! rapprochée ou rejetée est refusée **par proposition** dans `failed[]`
//! (`200`), jamais en refusant le lot entier — § *Pattern batch* du
//! `CLAUDE.md`. Seuls restent globaux le corps invalide (dont plus de
//! `MAX_IN_CLAUSE_IDS` identifiants), le compte bancaire inconnu ou archivé
//! (`404`) et le verrou non obtenu (`409`).
//!
//! **Sub-router** : monté sous `comptable_routes` (lib.rs:90 pattern
//! 8-1b — RBAC `Comptable` requis).

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, FromRequest, Query, Request, State};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::bank_transaction::{BankTransaction, BankTransactionStatus};
use kesh_db::entities::invoice::Invoice;
use kesh_db::errors::{ClaimSide, DbError};
use kesh_db::repositories::invoice_settlements::{ClaimSubject, GapAccountRole, PaymentAgainstDue};
use kesh_db::repositories::reconciliation::{
    InvoiceDateRefusal, InvoiceDateWindow, SupplierInvoiceInAccountBatch, UnpaidInvoiceCandidate,
    invoice_exigibility_date, payment_date,
};
use kesh_db::repositories::supplier_invoices::{self, SupplierSettlementRefusal};
use kesh_db::repositories::{
    accounts as accounts_repo, audit_log, bank_accounts, company_invoice_settings,
    contacts as contacts_repo, fiscal_years, invoice_settlements, journal_entries, letterings,
    projects, reconciliation as reconciliation_repo, reconciliation_rules,
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

/// Limite de résultats par défaut pour `GET /proposals`. Reflet frontend :
/// `PROPOSALS_DEFAULT_LIMIT` (`frontend/src/lib/features/reconciliation/
/// reconciliation.api.ts`, Story 29-9) — les deux valeurs se changent ensemble.
const DEFAULT_PROPOSALS_LIMIT: i64 = 100;

/// Cap maximum de la query param `limit` (M9 Pass 1 code review —
/// défense anti-DoS contre `?limit=999999`). Reflet frontend :
/// `PROPOSALS_MAX_LIMIT` (`reconciliation.api.ts`, Story 29-9), que l'écran
/// demande au plus. Figé par `get_proposals_limit_is_capped_at_500`.
const MAX_PROPOSALS_LIMIT: i64 = 500;

/// Tolérance du **paiement partiel** (5 centimes) au repo : une facture dont
/// le reste dû dépasse la transaction de 5 centimes au plus est candidate
/// (acceptée en règlement partiel, la facture reste ouverte pour la
/// différence). Réduit le candidate set sans accepter le mismatch (le helper
/// `propose_matches` reste binaire 0/1 sur amount, cf. L17/L21).
///
/// L'autre côté — le reste dû **inférieur** à la transaction — n'est pas une
/// tolérance : c'est la borne du trop-perçu,
/// `invoice_settlements::least_due_without_overpayment` (un demi-centime,
/// Story 29-6, #665), lue par `find_unpaid_invoices_for_window`.
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
    /// code review), sans probe count séparé. Depuis la Story 29-9 (#649),
    /// l'écran de réconciliation le lit : un avis sous la liste dit qu'il
    /// reste des transactions plus anciennes, et un bouton redemande la liste
    /// au plafond (`MAX_PROPOSALS_LIMIT`) ; au plafond, l'avis reste, sans
    /// bouton.
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationProposal {
    pub bank_transaction_id: i64,
    pub transaction: TransactionSummary,
    pub candidates: Vec<ReconciliationCandidate>,
    /// Story 30-1b (#686, AC-b5) — pourquoi la ligne ne propose **rien**,
    /// quand c'est une décision et non une absence : `null` sinon (champ
    /// additif). Posée, elle va de pair avec `candidates` vide.
    pub notice: Option<ProposalNotice>,
}

/// Story 30-1b (#686, AC-b5) — l'avis d'une ligne qui ne propose rien. Un
/// seul code : `supplier_invoice_in_payment_batch` — une facture fournisseur
/// ouverte au montant **exact** du débit est engagée dans un lot de paiement
/// en cours **sur ce compte** ; proposer une autre facture (ou une règle)
/// risquerait de régler deux fois une même dépense.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalNotice {
    /// Le code de l'avis (`supplier_invoice_in_payment_batch`).
    pub code: &'static str,
    pub supplier_invoice_id: i64,
    /// Le numéro de la facture chez le fournisseur ; nullable (l'écran affiche
    /// alors `#id`).
    pub supplier_invoice_number: Option<String>,
    pub payment_batch_id: i64,
}

/// Le code de l'unique [`ProposalNotice`] (Story 30-1b).
pub const NOTICE_SUPPLIER_INVOICE_IN_PAYMENT_BATCH: &str = "supplier_invoice_in_payment_batch";

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
/// type. `Invoice` héritée 8-4 ; `Rule` nouveau 8-5b ; `SupplierInvoice`
/// (sérialisé `supplier_invoice`), Story 30-1b (#686) : une facture
/// fournisseur, proposée pour un **débit** seulement.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateType {
    Invoice,
    Rule,
    SupplierInvoice,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationCandidate {
    /// Story 8-5b — discriminator. Les fields ci-dessous sont Some/None
    /// selon ce type.
    pub candidate_type: CandidateType,
    // ----- Invoice candidate fields -----
    /// La facture **client** ; `None` pour une facture fournisseur et une règle.
    pub invoice_id: Option<i64>,
    /// Story 30-1b — la facture **fournisseur** ; `None` sinon. Pour une
    /// facture fournisseur, les champs `invoice_*` qui suivent portent son
    /// numéro (`supplier_invoice_number`), son TTC au centime, sa date ;
    /// `invoice_total_ttc` est `None` (le règlement fournisseur est binaire).
    pub supplier_invoice_id: Option<i64>,
    /// Story 30-1b (C-30-1-15) — le nom du client ou du fournisseur de la
    /// facture, rempli pour les **deux** sortes de factures ; `None` pour une
    /// règle, ou si le contact est introuvable.
    pub contact_name: Option<String>,
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

/// Story 29-3 (#527) — le **rang** d'une candidate dans la priorité écrite
/// de [`order_candidates`] : 0 passe devant 1, qui passe devant 2.
///
/// | rang | candidate |
/// |---|---|
/// | 0 | facture qui porte un **signal d'identité** : montant égal au centime, référence liée à son numéro, ou contrepartie liée au nom de son client — quel que soit son score total |
/// | 1 | la règle d'affectation (une au plus) |
/// | 2 | facture **sans aucun signal** |
///
/// **Pourquoi le signal, et non le score.** Une règle ne sait rien des
/// factures ouvertes : l'accepter pour le paiement d'un client identifié
/// porte ce paiement en **produit** — produit compté deux fois, facture
/// restée due, rappel envoyé à un client qui a payé, et rien ne le signale.
/// C'est le faux rattachement **muet** (`CLAUDE.md`, *Un appariement
/// automatique propose, il ne crée jamais*). Le tri brut par score est écarté :
/// le `1.0` de la règle est une **convention**, pas une mesure, et le comparer
/// aux scores des factures ferait passer la règle devant toute facture faible
/// — c'est-à-dire devant des paiements de clients identifiés.
///
/// Le rang 0 se lit sur les **sous-scores**, jamais sur un seuil du total :
/// `matching.rs` n'y pose que les littéraux `1.0`, `0.5` et `0.0`, comparables
/// exactement.
///
/// ⚠️ **Le rang 2 est inatteignable aujourd'hui** : les trois sous-scores sont
/// positifs ou nuls, et `propose_matches` écarte une facture de total nul —
/// toute facture candidate porte donc un signal. Il existe pour qu'une source
/// future de factures (un choix manuel, #666 ; un score de montant à gradient,
/// la décision L17 de `matching.rs`) ne puisse pas placer **en silence** une
/// facture sans indice devant une règle.
///
/// Le `match` est **exhaustif, sans bras `_`** : un variant ajouté à
/// [`CandidateType`] ne compile pas sans être classé ici.
///
/// Story 30-1b (#686, AC-b5) : une facture **fournisseur** suit la même règle
/// qu'une facture client — elle ne côtoie jamais une facture client (un débit
/// ne reçoit que des factures fournisseur, un crédit que des factures client).
fn candidate_rank(c: &ReconciliationCandidate) -> u8 {
    match c.candidate_type {
        CandidateType::Invoice => invoice_rank(&c.score),
        CandidateType::SupplierInvoice => invoice_rank(&c.score),
        CandidateType::Rule => 1,
    }
}

/// Le rang d'une facture (client ou fournisseur) : 0 si elle porte un signal
/// d'identité, 2 sinon — cf. [`candidate_rank`].
fn invoice_rank(s: &MatchScore) -> u8 {
    if s.amount_score == 1.0 || bears_identity_hint(s) {
        0
    } else {
        2
    }
}

/// Une facture porte-t-elle un **indice** d'identité hors du montant — une
/// référence liée à son numéro (ou, pour un fournisseur, à sa référence de
/// paiement), une contrepartie liée au nom de son contact ? Lu sur les
/// **sous-scores**, jamais sur le total : une facture au montant exact a un
/// total ≥ 0,5 sans aucun indice (Story 30-1b, AC-b5, P3 R11).
fn bears_identity_hint(s: &MatchScore) -> bool {
    s.reference_score > 0.0 || s.contact_score > 0.0
}

/// Story 29-3 (#527) — ordonne les candidates d'une proposition selon la
/// priorité écrite de [`candidate_rank`] : une facture qui porte un signal
/// d'identité passe **toujours** devant la règle d'affectation.
///
/// L'écran n'affiche et n'accepte que `candidates[0]` (une seule candidate
/// visible par ligne, KF-026 #76) : cet ordre est ce que l'utilisateur
/// accepte d'un clic, et la page le dit au-dessus de la liste.
///
/// Le tri est **stable** (`sort_by_key`, jamais `sort_unstable_by_key`) : à
/// rang égal, l'ordre reçu de `propose_matches` est conservé — score
/// décroissant, puis, à score égal, l'ordre d'imputation (art. 87 CO : la
/// facture échue la première d'abord ; Story 29-5, #668), que ce tri garde
/// tel quel. La fonction ne suppose **rien** de l'ordre d'entrée.
///
/// ⚠️ Sur les entrées que produit `get_proposals`, elle ne change rien : la
/// règle y est ajoutée en dernier, et chaque facture qui la côtoie est de
/// rang 0. Son effet ne s'observe qu'aux tests de `candidate_order_tests` et
/// aux mutations du critère de rang. C'est voulu : l'ordre devient **écrit**,
/// au lieu d'être l'effet de bord de la place d'un `push`.
///
/// Les deux angles morts que la Story 29-3 écrivait ici sont résolus : une
/// facture que le paiement dépasse de 1 à 5 centimes n'est plus candidate
/// (Story 29-6, #665 — la proposition ne charge que ce que `classify_payment`
/// admet) ; à score égal, la facture échue la première passe en tête, et non
/// plus celle de plus petit identifiant (C-29-3-12 ; Story 29-5, #668).
fn order_candidates(candidates: &mut [ReconciliationCandidate]) {
    candidates.sort_by_key(candidate_rank);
}

// ============================================================
// Story 30-1b (#686) — les documents d'une transaction, et le masquage de la
// règle d'affectation, partagés par la proposition et l'acceptation
// ============================================================

/// Ce que `get_proposals` a chargé pour une transaction (Story 30-1b, AC-b4).
enum TxDocuments {
    /// Transaction non CHF, ou de montant nul : aucune facture.
    Nothing,
    /// Un **crédit** : les factures client candidates, et celles que le
    /// paiement dépasse de quelques centimes (Story 29-6, jamais proposées).
    Client {
        candidates: Vec<UnpaidInvoiceCandidate>,
        slightly_overpaid: Vec<UnpaidInvoiceCandidate>,
    },
    /// Un **débit** : les factures fournisseur candidates, celles qui masquent
    /// la règle, et la facture du lot en cours sur le compte (AC-b3).
    Supplier {
        candidates: Vec<kesh_db::entities::SupplierInvoice>,
        masking: Vec<kesh_db::entities::SupplierInvoice>,
        account_batch: Option<SupplierInvoiceInAccountBatch>,
    },
}

/// Les triplets `(facture, contact, montant à régler)` de `propose_matches`
/// pour des factures **client** : le montant est le **reste dû** (#420)
/// **au centime** (#476).
fn client_triples(
    invoices: &[UnpaidInvoiceCandidate],
    contacts: &HashMap<i64, kesh_db::entities::Contact>,
) -> Vec<(Invoice, Option<kesh_db::entities::Contact>, Decimal)> {
    invoices
        .iter()
        .map(|c| {
            (
                c.invoice.clone(),
                contacts.get(&c.invoice.contact_id).cloned(),
                invoice_settlements::amount_due_to_centime(c.amount_due),
            )
        })
        .collect()
}

/// Les triplets de `propose_matches` pour des factures **fournisseur**
/// (Story 30-1b) : le montant est le **TTC** — le règlement est binaire
/// (C-30-1-5) ; un TTC à plus de deux décimales n'égale jamais un relevé
/// (angle mort L2, #687 — depuis la v0.17.0, le cas d'une facture enregistrée
/// avant : la création refuse une ligne sous le centime).
fn supplier_triples(
    invoices: &[kesh_db::entities::SupplierInvoice],
    contacts: &HashMap<i64, kesh_db::entities::Contact>,
) -> Vec<(
    kesh_db::entities::SupplierInvoice,
    Option<kesh_db::entities::Contact>,
    Decimal,
)> {
    invoices
        .iter()
        .map(|s| {
            (
                s.clone(),
                contacts.get(&s.contact_id).cloned(),
                s.total_amount,
            )
        })
        .collect()
}

/// Une facture candidate atteint-elle le seuil (0,5) qui écarte la règle
/// d'affectation (AC #114 de la Story 8-5b) ?
fn has_strong_invoice_match(matches: &[kesh_reconciliation::MatchProposal]) -> bool {
    const INVOICE_OVERRIDE_THRESHOLD: f64 = 0.5;
    matches
        .iter()
        .any(|mp| mp.score.total >= INVOICE_OVERRIDE_THRESHOLD)
}

/// Ce qui **défend une règle d'affectation** contre son acceptation (Story
/// 30-1b, #686, AC-b17) : la raison de `RECONCILIATION_RULE_MASKED`.
#[derive(Debug, Clone, PartialEq)]
enum RuleMask {
    /// Débit — une facture fournisseur ouverte au montant exact est engagée
    /// dans un lot `generated` du compte (raison `payment_batch`).
    PaymentBatch {
        supplier_invoice_id: i64,
        payment_batch_id: i64,
    },
    /// Débit — une facture fournisseur **candidate** (raison
    /// `invoice_candidate`).
    InvoiceCandidate { supplier_invoice_id: i64 },
    /// Crédit — une facture client que le paiement dépasse de 1 à 5 centimes,
    /// et qui porte un indice (raison `invoice_slightly_overpaid`, Story 29-6).
    InvoiceSlightlyOverpaid { invoice_id: i64 },
}

/// **Le** critère, partagé par la proposition (`get_proposals`) et
/// l'acceptation d'une règle (`accept_one_rule`), des trois cas où une
/// facture défend la règle (Story 30-1b, AC-b17, DRY). Pure : l'appelant
/// fournit ce qu'il a lu — la facture du lot du compte, les notes des
/// candidates fournisseur, les notes des factures client que le paiement
/// dépasse de quelques centimes (non vide ⇔ indice : leur score de montant
/// est nul, AC-b17).
///
/// ⚠️ **Inventaire clos de ce que l'acceptation ne rejoue pas** (P5, F6),
/// propre à la proposition : côté client, le masquage par une facture
/// **candidate** (`has_strong_invoice_match`) — comportement d'avant la
/// story, gardé pour ne rien retirer à un client existant ; côté
/// fournisseur, la facture presque au montant portant un indice, la facture
/// exacte hors fenêtre portant un indice, la facture exacte dans un lot d'un
/// **autre** compte portant un indice, et la liste masquante pleine. Ni
/// l'une ni l'autre ne lit les factures **payées** (L1, #671).
fn rule_mask_for(
    account_batch: Option<&SupplierInvoiceInAccountBatch>,
    supplier_matches: &[kesh_reconciliation::MatchProposal],
    overpaid_matches: &[kesh_reconciliation::MatchProposal],
) -> Option<RuleMask> {
    if let Some(held) = account_batch {
        return Some(RuleMask::PaymentBatch {
            supplier_invoice_id: held.supplier_invoice_id,
            payment_batch_id: held.payment_batch_id,
        });
    }
    if has_strong_invoice_match(supplier_matches) {
        return Some(RuleMask::InvoiceCandidate {
            supplier_invoice_id: supplier_matches[0].document_id,
        });
    }
    overpaid_matches
        .first()
        .map(|mp| RuleMask::InvoiceSlightlyOverpaid {
            invoice_id: mp.document_id,
        })
}

/// Le refus `RECONCILIATION_RULE_MASKED` d'une règle défendue (AC-b17).
fn rule_masked_failed_proposal(bank_transaction_id: i64, mask: &RuleMask) -> FailedProposal {
    let details = match mask {
        RuleMask::PaymentBatch {
            supplier_invoice_id,
            payment_batch_id,
        } => serde_json::json!({
            "reason": "payment_batch",
            "supplierInvoiceId": supplier_invoice_id,
            "paymentBatchId": payment_batch_id,
        }),
        RuleMask::InvoiceCandidate {
            supplier_invoice_id,
        } => serde_json::json!({
            "reason": "invoice_candidate",
            "supplierInvoiceId": supplier_invoice_id,
        }),
        RuleMask::InvoiceSlightlyOverpaid { invoice_id } => serde_json::json!({
            "reason": "invoice_slightly_overpaid",
            "invoiceId": invoice_id,
        }),
    };
    FailedProposal {
        bank_transaction_id,
        error_code: "RECONCILIATION_RULE_MASKED".to_string(),
        details: Some(details),
    }
}

/// Le **rejeu du masquage** à l'acceptation d'une règle (Story 30-1b, #686,
/// AC-b17) : relit, **en lectures ordinaires**, par les mêmes fonctions de
/// dépôt que `get_proposals`, ce que [`rule_mask_for`] juge — pour un débit,
/// la facture du lot du compte et les candidates fournisseur de la fenêtre
/// (notées avec la même carte de contacts, `find_contacts_by_ids_chunked`) ;
/// pour un crédit, les factures client que le paiement dépasse de 1 à 5
/// centimes. Un lot ou une facture commis après la naissance de la vue de la
/// transaction du lot ne sont pas vus : même tolérance que la proposition.
async fn load_rule_mask(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    bt: &BankTransaction,
) -> Result<Option<RuleMask>, DbError> {
    if bt.currency != "CHF" || bt.amount.is_zero() {
        return Ok(None);
    }
    if bt.amount < Decimal::ZERO {
        let amount = bt.amount.abs();
        let account_batch = reconciliation_repo::find_open_supplier_invoice_in_account_batch(
            &mut **tx,
            company_id,
            bank_account_id,
            amount,
        )
        .await?;
        if account_batch.is_some() {
            return Ok(rule_mask_for(account_batch.as_ref(), &[], &[]));
        }
        let candidates = reconciliation_repo::find_open_supplier_invoices_for_window(
            &mut **tx,
            company_id,
            InvoiceDateWindow::for_supplier_transaction(bt),
            amount,
        )
        .await?;
        let ids: Vec<i64> = candidates.iter().map(|s| s.contact_id).collect();
        let contacts =
            reconciliation_repo::find_contacts_by_ids_chunked(tx, company_id, &ids).await?;
        let matches = propose_matches(bt, &supplier_triples(&candidates, &contacts));
        return Ok(rule_mask_for(None, &matches, &[]));
    }
    let overpaid = reconciliation_repo::find_invoices_slightly_overpaid_for_window(
        &mut **tx,
        company_id,
        InvoiceDateWindow::for_transaction(bt),
        bt.amount,
        Decimal::new(AMOUNT_TOLERANCE_HUNDREDTHS, 2),
    )
    .await?;
    let ids: Vec<i64> = overpaid.iter().map(|c| c.invoice.contact_id).collect();
    let contacts = reconciliation_repo::find_contacts_by_ids_chunked(tx, company_id, &ids).await?;
    let matches = propose_matches(bt, &client_triples(&overpaid, &contacts));
    Ok(rule_mask_for(None, &[], &matches))
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
    /// La facture client réglée ; `0` (sentinelle) pour `split`, `rule` et
    /// `supplier_invoice`.
    pub invoice_id: i64,
    /// Story 30-1b (#686, AC-b6) — la facture fournisseur réglée ; `null`
    /// hors du type `supplier_invoice` (champ additif).
    pub supplier_invoice_id: Option<i64>,
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

/// Story 15-12b (#543, AC 11) — le refus « exercice postérieur clos » dans une
/// acceptation par lot : le filet de `journal_entries::create_in_tx_inner`
/// refuse toute écriture dans un exercice dont un exercice postérieur est clos
/// (état hérité), et le § *Pattern batch* du `CLAUDE.md` exige un **code
/// canonique** dans `failed[]`, jamais un repli `DATABASE_ERROR`.
///
/// ⛔ **Un seul constructeur pour les DEUX sites** (patron de
/// [`period_locked_failed_proposal`]) : le bras de
/// [`project_error_to_failed_proposal`] (voies « facture » et « ventilé ») et
/// la branche en ligne d'`accept_one_rule` (voie « règle »). Le code est un
/// **littéral**, visible au `grep` du décompte des codes de
/// `failed-proposal-label.ts`.
///
/// `details` : `{ fiscalYearId, fiscalYearName }` — l'exercice postérieur clos
/// le plus proche —, plus `projectId` quand il est connu ; **omis** quand il
/// vaut `None` (voie « règle » et « ventilé »), jamais publié à `null`.
fn later_fiscal_year_closed_failed_proposal(
    bank_transaction_id: i64,
    project_id: Option<i64>,
    fiscal_year_id: i64,
    fiscal_year_name: &str,
) -> FailedProposal {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "fiscalYearId".to_string(),
        serde_json::json!(fiscal_year_id),
    );
    obj.insert(
        "fiscalYearName".to_string(),
        serde_json::json!(fiscal_year_name),
    );
    if let Some(pid) = project_id {
        obj.insert("projectId".to_string(), serde_json::json!(pid));
    }
    FailedProposal {
        bank_transaction_id,
        error_code: "LATER_FISCAL_YEAR_CLOSED".to_string(),
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

/// Story 15-6d (#524, choix C-15-6-9, C-15-6-23, C-15-6-28) — la contrepartie
/// d'un rapprochement est-elle **le compte comptable du compte bancaire** ?
/// Une écriture `D banque / C banque` s'équilibre et rien, au grand livre, ne
/// la refuse : seule cette comparaison d'identifiants la ferme.
///
/// La comparaison est celle du helper commun de la 15-6b
/// ([`invoice_settlements::ensure_not_claim_account`], pure) — sa
/// **construction** de refus (variante nommée d'un compte de créance) n'est pas
/// empruntée : le refus reste celui du flux ventilé, construit par
/// [`counterparty_is_bank_ledger_failed_proposal`] (lot) et
/// [`counterparty_is_bank_ledger_error`] (routes directes).
///
/// **Place sur les quatre chemins** (même réponse pour un même défaut) :
/// **après** le contrôle du compte de banque actif (archivé →
/// `BANK_ACCOUNT_NOT_CONFIGURED`), **avant** la validation de la contrepartie
/// (`ACCOUNT_NOT_FOUND`, puis `ACCOUNT_NOT_POSTABLE`) — si bien qu'un compte de
/// banque devenu non imputable pris pour contrepartie rend ce refus, et non
/// `ACCOUNT_NOT_POSTABLE`, qui inviterait à agir sur la mauvaise chose.
///
/// La valeur comparée est celle qui construit l'écriture : aucun verrou n'est
/// nécessaire pour que l'égalité soit sûre.
fn is_bank_ledger(counterparty_account_id: i64, bank_ledger_account_id: i64) -> bool {
    invoice_settlements::ensure_not_claim_account(counterparty_account_id, bank_ledger_account_id)
        .is_err()
}

/// Story 15-6d (AC3) — le refus **par lot** d'une contrepartie égale au compte
/// de banque, pour `accept_one_split` et `accept_one_rule` : `failed[]` avec
/// `errorCode = "VALIDATION_ERROR"` et `details = { "reason":
/// "counterparty_equals_bank_ledger" }` — la forme du flux ventilé existant,
/// que la règle reprend (cf. [`is_bank_ledger`] pour la place).
fn counterparty_is_bank_ledger_failed_proposal(bank_transaction_id: i64) -> FailedProposal {
    FailedProposal {
        bank_transaction_id,
        error_code: "VALIDATION_ERROR".to_string(),
        details: Some(serde_json::json!({ "reason": "counterparty_equals_bank_ledger" })),
    }
}

/// Story 15-6d (AC3) — le refus **direct** d'une contrepartie égale au compte
/// de banque, pour `post_split` et `post_manual` : 400 `VALIDATION_ERROR`, dont
/// le texte nomme le champ en cause (`splits[{idx}].counterpartyAccountId` ou
/// `counterpartyAccountId`). ⚠️ Limite héritée de `post_split` : le message est
/// en français en dur, non traduit ; le code est le champ stable.
fn counterparty_is_bank_ledger_error(field: &str) -> AppError {
    AppError::Validation(format!("{field} ne peut pas être le compte ledger banque"))
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
        // Story 15-12b (#543, AC 11) — le filet sous un bilan clos : un refus
        // MÉTIER (état hérité des exercices), jamais `DATABASE_ERROR`.
        DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        } => later_fiscal_year_closed_failed_proposal(
            bank_transaction_id,
            project_id,
            *fiscal_year_id,
            fiscal_year_name,
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
    /// Story 30-1b (#686, AC-b6) — le règlement d'une facture fournisseur
    /// **ouverte** par un **débit** : `accept_one_supplier_invoice`.
    #[serde(rename = "supplier_invoice", rename_all = "camelCase")]
    SupplierInvoice {
        bank_transaction_id: i64,
        supplier_invoice_id: i64,
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
            }
            | AcceptProposalInput::SupplierInvoice {
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
                        "corps JSON malformé ou champ `type` requis manquant — valeurs acceptées : [\"invoice\", \"split\", \"rule\", \"supplier_invoice\"]".to_string()
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

    // Pass 3.5 avancée (Story 29-6, #665, revue P1) : les règles actives sont
    // lues avant le Pass 2, qui ne cherche les factures trop-perçues que s'il
    // existe une règle à garder.
    let active_rules =
        reconciliation_rules::find_active_for_company(&state.pool, current_user.company_id).await?;

    // Pass 2 : per tx, load candidates + accumulate distinct contact_ids.
    // Un crédit lit les factures client et celles que le paiement dépasse de
    // quelques centimes (jamais proposées ; lues pour garder la règle,
    // Pass 4) ; un débit, les factures fournisseur (Story 30-1b).
    let mut tx_candidates: Vec<(BankTransaction, TxDocuments)> =
        Vec::with_capacity(transactions.len());
    let mut distinct_contact_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let underpayment_tolerance = Decimal::new(AMOUNT_TOLERANCE_HUNDREDTHS, 2);
    for tx in transactions {
        // A6-2 Pass 6 — currency tx-side guard (mono-CHF v0.1, cf. L38).
        // Skip transactions non-CHF. Défense en profondeur : depuis la
        // Story 27-2, aucun import n'enregistre de transaction non CHF
        // (CAMT et CSV à colonne Devise refusés, CSV sans colonne en CHF).
        if tx.currency != "CHF" {
            tx_candidates.push((tx, TxDocuments::Nothing));
            continue;
        }
        // Un montant nul ne reçoit aucune facture, ni client ni fournisseur
        // (Story 30-1b, AC-b4) ; la règle reste calculée pour lui (P-H7).
        if tx.amount.is_zero() {
            tx_candidates.push((tx, TxDocuments::Nothing));
            continue;
        }
        // Story 30-1b (#686, AC-b4) — un DÉBIT reçoit les factures
        // fournisseur, jamais une facture client (#482). Trois lectures
        // ordinaires (AC-b3) : les candidates de la fenêtre fournisseur, les
        // factures qui masquent la règle, la facture du lot en cours sur ce
        // compte.
        if tx.amount < Decimal::ZERO {
            let amount = tx.amount.abs();
            let candidates = reconciliation_repo::find_open_supplier_invoices_for_window(
                &state.pool,
                current_user.company_id,
                InvoiceDateWindow::for_supplier_transaction(&tx),
                amount,
            )
            .await?;
            let masking = reconciliation_repo::find_supplier_invoices_masking(
                &state.pool,
                current_user.company_id,
                amount,
            )
            .await?;
            let account_batch = reconciliation_repo::find_open_supplier_invoice_in_account_batch(
                &state.pool,
                current_user.company_id,
                query.bank_account_id,
                amount,
            )
            .await?;
            // Les contacts des candidates ET des masquantes : l'indice
            // « fournisseur » d'une facture masquante se lit sur eux (P5, F5).
            for si in candidates.iter().chain(&masking) {
                if si.contact_id > 0 {
                    distinct_contact_ids.insert(si.contact_id);
                }
            }
            tx_candidates.push((
                tx,
                TxDocuments::Supplier {
                    candidates,
                    masking,
                    account_batch,
                },
            ));
            continue;
        }
        // Un CRÉDIT garde exactement le chemin d'avant la Story 30-1b : les
        // factures client ; il ne reçoit jamais de facture fournisseur.
        // Story 28-7 (#548) — la fenêtre de dates de l'acceptation, sur la
        // même date (valeur, à défaut comptabilisation) : une facture
        // proposée est une facture que `accept_one_invoice` admet. Story 29-2
        // (#529) : sa borne basse porte sur la date d'ancrage (date, échéance,
        // dernier rappel non annulé), même expression des deux côtés.
        let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
            &state.pool,
            current_user.company_id,
            InvoiceDateWindow::for_transaction(&tx),
            tx.amount,
            underpayment_tolerance,
        )
        .await?;
        // Story 29-6 (#665, revue de code P1, E1) — les factures que la borne
        // basse vient d'écarter (le paiement les dépasse de 1 à 5 centimes).
        // Avant la story, elles étaient candidates et, portant un indice,
        // passaient devant la règle d'affectation ; sans elles, la règle
        // deviendrait la proposition, et *Accepter* porterait en produit le
        // paiement d'un client identifié, la facture restant due. Lues
        // seulement si une règle existe : sans règle, il n'y a rien à garder.
        let slightly_overpaid = if active_rules.is_empty() {
            Vec::new()
        } else {
            reconciliation_repo::find_invoices_slightly_overpaid_for_window(
                &state.pool,
                current_user.company_id,
                InvoiceDateWindow::for_transaction(&tx),
                tx.amount,
                // La tolérance d'avant la story valait des deux côtés : les
                // factures qu'elle admettait au-dessous sont celles-ci.
                underpayment_tolerance,
            )
            .await?
        };
        for cand in candidates.iter().chain(&slightly_overpaid) {
            // M1 Pass 1 — guard `contact_id > 0` : l'entité Invoice
            // expose `contact_id: i64` (pas `Option<i64>`) avec sentinel
            // 0 = pas de contact lié. Insérer 0 dans le set provoquerait
            // un SELECT inutile sur contact_id=0 (jamais existant).
            if cand.invoice.contact_id > 0 {
                distinct_contact_ids.insert(cand.invoice.contact_id);
            }
        }
        tx_candidates.push((
            tx,
            TxDocuments::Client {
                candidates,
                slightly_overpaid,
            },
        ));
    }

    // Pass 3 : batch-load contacts — clients et fournisseurs en un même lot,
    // découpé par tranches de `MAX_IN_CLAUSE_IDS` (Story 30-1b, P5 R1-7 : 500
    // débits peuvent porter plus de 500 contacts).
    let ids_vec: Vec<i64> = distinct_contact_ids.into_iter().collect();
    let contacts_map = {
        let mut conn = state
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
        reconciliation_repo::find_contacts_by_ids_chunked(
            &mut conn,
            current_user.company_id,
            &ids_vec,
        )
        .await?
    };

    // Pass 3.5 (Story 8-5b §rule-application) — accounts_info pour appliquer
    // rule fallback dans Pass 4 (closure sync) ; les règles actives sont lues
    // avant le Pass 2 (Story 29-6).
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
    //
    // Story 15-6d (AC2 bis, #524) : le compte de la banque (s'il est configuré)
    // en est **retiré** — une règle dont la contrepartie est ce compte n'est
    // plus proposée (l'acceptation la refuserait toujours, `accept_one_rule`
    // étape 4 bis) et ne masque plus une règle suivante valable. L'ensemble
    // contient donc les comptes **actifs, imputables, hors compte de la
    // banque** ; le doc-comment de `first_matching_rule` (« non archivé »)
    // reste tel quel, le site d'appel fait foi.
    let bank_ledger_account_id = ba_check.journal_account_id;
    let active_account_ids: std::collections::HashSet<i64> = accounts_info_rows
        .iter()
        .filter(|(id, _, _, postable)| *postable && Some(*id) != bank_ledger_account_id)
        .map(|(id, _, _, _)| *id)
        .collect();
    let accounts_info: HashMap<i64, (String, String)> = accounts_info_rows
        .into_iter()
        .map(|(id, number, name, _)| (id, (number, name)))
        .collect();

    // Pass 4 : score per-tx + rule fallback + build response.
    // AC #114 (Pass 2 Q8) : une facture candidate de score ≥ 0.5 écarte la
    // règle, qui n'est pas même calculée. En dessous, la règle qui correspond
    // rejoint la liste, et `order_candidates` (Story 29-3, #527) fixe la
    // priorité : une facture qui porte un signal d'identité passe devant elle.
    // ⚠️ L'écran n'affiche et n'accepte que `candidates[0]` — une seule des
    // deux se voit.
    // Story 29-6 (#665, revue P1) : une facture que le paiement dépasse de
    // quelques centimes écarte aussi la règle si elle porte un indice, comme
    // sa présence parmi les candidates le faisait avant la story.
    //
    // Story 30-1b (#686, AC-b5) — pour un DÉBIT : une facture fournisseur du
    // lot en cours sur ce compte efface la ligne (`notice`) ; sinon les
    // candidates fournisseur, puis la règle, que masque aussi une facture
    // fournisseur non candidate portant un indice (`find_supplier_invoices_masking`).
    // ⛔ Rien ne lit ici les factures déjà payées : l'angle mort L1 (#671).
    let contact_name_of = |contact_id: i64| contacts_map.get(&contact_id).map(|c| c.name.clone());
    let proposals: Vec<ReconciliationProposal> = tx_candidates
        .into_iter()
        .map(|(tx, documents)| {
            let mut candidates: Vec<ReconciliationCandidate> = Vec::new();
            let mut notice: Option<ProposalNotice> = None;
            // La règle est-elle défendue par une facture ? (les critères
            // partagés avec l'acceptation d'une règle passent par
            // `rule_mask_for`, AC-b17 ; les autres sont propres à la proposition).
            let mut rule_masked = false;
            match documents {
                TxDocuments::Nothing => {}
                TxDocuments::Supplier {
                    candidates: supplier_invoices,
                    masking,
                    account_batch,
                } => {
                    if let Some(held) = account_batch {
                        // AC-b5, première ligne : ni candidate, ni règle — la
                        // sûreté prime sur la commodité. La raison est dite.
                        notice = Some(ProposalNotice {
                            code: NOTICE_SUPPLIER_INVOICE_IN_PAYMENT_BATCH,
                            supplier_invoice_id: held.supplier_invoice_id,
                            supplier_invoice_number: held.supplier_invoice_number,
                            payment_batch_id: held.payment_batch_id,
                        });
                    } else {
                        let matches = propose_matches(
                            &tx,
                            &supplier_triples(&supplier_invoices, &contacts_map),
                        );
                        rule_masked = rule_mask_for(None, &matches, &[]).is_some();
                        // Les masquantes non candidates : la liste pleine
                        // masque par prudence ; sinon, un indice (référence
                        // ou fournisseur) suffit — jamais le montant seul.
                        let candidate_ids: std::collections::HashSet<i64> =
                            supplier_invoices.iter().map(|s| s.id).collect();
                        let rest: Vec<kesh_db::entities::SupplierInvoice> = masking
                            .iter()
                            .filter(|s| !candidate_ids.contains(&s.id))
                            .cloned()
                            .collect();
                        let masking_full =
                            masking.len() as i64 >= reconciliation_repo::SUPPLIER_MASKING_LIMIT;
                        rule_masked = rule_masked
                            || masking_full
                            || propose_matches(&tx, &supplier_triples(&rest, &contacts_map))
                                .iter()
                                .any(|mp| bears_identity_hint(&mp.score));
                        candidates = matches
                            .into_iter()
                            .filter_map(|mp| {
                                let si =
                                    supplier_invoices.iter().find(|s| s.id == mp.document_id)?;
                                Some(ReconciliationCandidate {
                                    candidate_type: CandidateType::SupplierInvoice,
                                    invoice_id: None,
                                    supplier_invoice_id: Some(si.id),
                                    contact_name: contact_name_of(si.contact_id),
                                    invoice_number: si.supplier_invoice_number.clone(),
                                    // Le TTC au centime : le règlement
                                    // fournisseur est binaire, pas de reste dû.
                                    invoice_amount: Some(
                                        invoice_settlements::amount_due_to_centime(si.total_amount)
                                            .normalize()
                                            .to_string(),
                                    ),
                                    invoice_total_ttc: None,
                                    invoice_date: Some(si.invoice_date),
                                    rule_id: None,
                                    rule_label: None,
                                    rule_match_type: None,
                                    counterparty_account_id: None,
                                    counterparty_account_name: None,
                                    score: mp.score,
                                })
                            })
                            .collect();
                    }
                }
                TxDocuments::Client {
                    candidates: candidate_invoices,
                    slightly_overpaid,
                } => {
                    let match_proposals =
                        propose_matches(&tx, &client_triples(&candidate_invoices, &contacts_map));
                    // AC #114 : une facture client de score ≥ 0,5 écarte la règle.
                    // Story 29-6 (#665, revue de code P1, E1) — une facture que le
                    // paiement dépasse de 1 à 5 centimes n'est plus candidate ; si
                    // elle porte un indice (`propose_matches` ne retient que les
                    // factures de score non nul), elle garde la règle comme elle le
                    // faisait avant la story : la ligne reste « Aucune
                    // correspondance », rien n'est passé en produit. Les deux
                    // critères passent par `rule_mask_for` (Story 30-1b) ; seul le
                    // second est rejoué à l'acceptation d'une règle (AC-b17).
                    let overpaid_matches =
                        propose_matches(&tx, &client_triples(&slightly_overpaid, &contacts_map));
                    rule_masked = has_strong_invoice_match(&match_proposals)
                        || rule_mask_for(None, &[], &overpaid_matches).is_some();

                    candidates = match_proposals
                        .into_iter()
                        .filter_map(|mp| {
                            let cand = candidate_invoices
                                .iter()
                                .find(|c| c.invoice.id == mp.document_id)?;
                            let inv = &cand.invoice;
                            Some(ReconciliationCandidate {
                                candidate_type: CandidateType::Invoice,
                                invoice_id: Some(inv.id),
                                supplier_invoice_id: None,
                                contact_name: contact_name_of(inv.contact_id),
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
                                    let due =
                                        invoice_settlements::amount_due_to_centime(cand.amount_due);
                                    let ttc =
                                        invoice_settlements::amount_due_to_centime(cand.total_ttc);
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
                }
            }

            // Story 8-5b rule fallback. Currency CHF only (Pass 3 R1
            // currency filter cohérent L38) — déjà filtré au Pass 2 via
            // `candidate_invoices.is_empty()` quand tx.currency != "CHF",
            // mais rules NE PAS hériter du sign filter 8-4 invoice (P-H7).
            // Story 30-1b : jamais sur une ligne qui porte une `notice`.
            if notice.is_none()
                && !rule_masked
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
                    supplier_invoice_id: None,
                    contact_name: None,
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

            // Story 29-3 (#527) — la priorité écrite, DERNIER geste sur
            // `candidates` : l'écran affiche et accepte `candidates[0]`.
            order_candidates(&mut candidates);

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
                notice,
            }
        })
        .collect();

    Ok(Json(GetProposalsResponse {
        proposals,
        has_more,
    }))
}

// ============================================================
// Story 27-4 — classification d'une transaction d'un lot
// ============================================================

/// Classe une transaction d'un lot de réconciliation (`accept` ou `reject`)
/// à partir de la ligne lue pour `(company_id, bank_account_id, id)`, ou de
/// son absence — Story 27-4, AC1 (#634). Fonction **pure**, partagée par les
/// pré-vols des deux lots et par la reprise sous verrou du rejet.
///
/// Rend `None` si la transaction est **éligible**, sinon le refus à placer
/// dans `failed[]`. L'ordre des bras est la **priorité** :
///
/// | ligne lue | refus |
/// |---|---|
/// | absente — supprimée, d'une autre société, d'un autre compte | `BANK_TRANSACTION_NOT_FOUND` |
/// | rapprochée au sens d'[`is_reconciled`] | `RECONCILIATION_ALREADY_RECONCILED` |
/// | non rapprochée, `auto_match_rejected_at` non nul | `RECONCILIATION_ALREADY_REJECTED` |
/// | sinon | éligible |
///
/// - ⛔ **Anti-énumération** : les trois causes d'absence rendent le même
///   code, sans `details`, et l'appelant les obtient par la même requête
///   scopée société **et** compte.
/// - ⛔ **Une seule définition de « rapprochée »** : la fonction appelle
///   [`is_reconciled`] de la Story 27-1 et n'écrit aucun second test de
///   statut ; un statut ajouté plus tard (≠ `pending`) est refusé — la
///   classification échoue fermée.
/// - Chaque code est écrit **au site de construction**
///   (`error_code: "…".to_string()`), forme que recompte le doc-comment de
///   `failed-proposal-label.ts`.
///
/// [`is_reconciled`]: kesh_db::repositories::bank_imports::is_reconciled
fn classify_batch_transaction(id: i64, row: Option<&BankTransaction>) -> Option<FailedProposal> {
    use kesh_db::repositories::bank_imports::is_reconciled;
    match row {
        None => Some(FailedProposal {
            bank_transaction_id: id,
            error_code: "BANK_TRANSACTION_NOT_FOUND".to_string(),
            details: None,
        }),
        Some(t) if is_reconciled(t.status.as_str(), t.matched_entry_id) => Some(FailedProposal {
            bank_transaction_id: id,
            error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
            details: None,
        }),
        Some(t) if t.auto_match_rejected_at.is_some() => Some(FailedProposal {
            bank_transaction_id: id,
            error_code: "RECONCILIATION_ALREADY_REJECTED".to_string(),
            details: None,
        }),
        Some(_) => None,
    }
}

/// Refuse en `400` un lot de plus de [`MAX_IN_CLAUSE_IDS`] identifiants,
/// **avant** toute lecture (Story 27-4, AC5) — la borne est celle du dépôt,
/// qui rendait sinon une erreur de protocole sortie en `500`.
///
/// [`MAX_IN_CLAUSE_IDS`]: reconciliation_repo::MAX_IN_CLAUSE_IDS
fn check_batch_size(len: usize) -> Result<(), AppError> {
    if len > reconciliation_repo::MAX_IN_CLAUSE_IDS {
        return Err(AppError::Validation(format!(
            "un lot ne peut pas dépasser {} transactions",
            reconciliation_repo::MAX_IN_CLAUSE_IDS
        )));
    }
    Ok(())
}

/// Réunit les refus du pré-vol et ceux du traitement sous verrou dans
/// l'ordre de `ids` (Story 27-4, AC2 et AC4) : l'ordre des propositions
/// reçues pour `accept`, l'ordre croissant pour `reject`. Les doublons étant
/// refusés en `400` avant, chaque identifiant n'a qu'une source.
fn merge_failed_in_order(
    ids: &[i64],
    mut preflight: HashMap<i64, FailedProposal>,
    locked: Vec<FailedProposal>,
) -> Vec<FailedProposal> {
    for f in locked {
        preflight.insert(f.bank_transaction_id, f);
    }
    ids.iter().filter_map(|id| preflight.remove(id)).collect()
}

// ============================================================
// POST /accept
// ============================================================

/// Handler `POST /api/v1/reconciliation/accept`. Acquiert UN seul lock
/// pour tout le batch (H5 Pass 1 patch), itère per-proposal avec
/// savepoints MariaDB pour partial success.
///
/// **Story 27-4 (#634)** : le pré-vol (étape 0ter) **classe** chaque
/// transaction du lot (`classify_batch_transaction`) au lieu de refuser le
/// lot entier en `400` ; seules les propositions éligibles passent sous
/// verrou, et `failed[]` réunit les deux sources **dans l'ordre des
/// propositions reçues**. Un lot sans proposition éligible répond `200`
/// sans transaction ni verrou.
pub async fn post_accept(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    AcceptBodyExtractor(body): AcceptBodyExtractor,
) -> Result<Json<AcceptResponse>, AppError> {
    // Step 0 — validation body (Q2 breaking : enum tagged, dispatch sur variant).
    if body.proposals.is_empty() {
        return Err(AppError::Validation("proposals vide".into()));
    }
    check_batch_size(body.proposals.len())?;
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
            AcceptProposalInput::SupplierInvoice {
                supplier_invoice_id,
                ..
            } => {
                if *supplier_invoice_id <= 0 {
                    return Err(AppError::Validation(
                        "supplierInvoiceId doit être strictement positif (type='supplier_invoice')"
                            .into(),
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
    // Story 28-4a (#642, R1) — compte clos : chaque proposition en `failed[]`,
    // `200`, sans lire les transactions ni prendre de verrou (§ Pattern batch).
    if ba_check.is_closed() {
        let ids: Vec<i64> = body
            .proposals
            .iter()
            .map(|p| p.bank_transaction_id())
            .collect();
        return Ok(Json(AcceptResponse {
            accepted: Vec::new(),
            failed: closed_account_failed(&ids, ba_check.id),
        }));
    }

    // Step 0ter — pré-vol par lot, hors transaction et hors verrou (MP3-3 ;
    // Story 27-4, AC2). Lecture SANS filtre de statut, puis classification de
    // chaque transaction : les refus vont dans `failed[]`, par proposition ;
    // seules les éligibles passent sous verrou.
    let tx_ids: Vec<i64> = body
        .proposals
        .iter()
        .map(|p| p.bank_transaction_id())
        .collect();
    let tx_map = reconciliation_repo::find_by_ids_for_account(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
        &tx_ids,
    )
    .await?;
    let mut preflight_failed: HashMap<i64, FailedProposal> = HashMap::new();
    let mut eligible: Vec<AcceptProposalInput> = Vec::with_capacity(body.proposals.len());
    for p in &body.proposals {
        let id = p.bank_transaction_id();
        match classify_batch_transaction(id, tx_map.get(&id)) {
            Some(refusal) => {
                preflight_failed.insert(id, refusal);
            }
            None => eligible.push(p.clone()),
        }
    }
    // C2 Pass 1 — `tx_map` (instantané du pré-vol) n'est pas passé dans le
    // verrou : `accept_one` recharge la transaction sous verrou (TOCTOU).
    drop(tx_map);

    // Story 27-4, AC3 — rien d'éligible : ni transaction, ni verrou.
    if eligible.is_empty() {
        return Ok(Json(AcceptResponse {
            accepted: Vec::new(),
            failed: merge_failed_in_order(&tx_ids, preflight_failed, Vec::new()),
        }));
    }
    // AC2 — `batch_size` de la trace `reconciliation.accepted` = taille du
    // lot REÇU, pas du sous-lot éligible.
    let batch_size = body.proposals.len() as i64;

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
    let locked = retry_with(
        "reconciliation::accept",
        DEFAULT_MAX_DEADLOCK_ATTEMPTS,
        |err: &AppError| {
            crate::retry::is_app_deadlock(err)
                || matches!(err, AppError::ReconciliationTransactionAborted)
        },
        || {
            let pool = state.pool.clone();
            let proposals = eligible.clone();
            async move {
                accept_once(
                    &pool,
                    company_id,
                    bank_account_id,
                    user_id,
                    actor_api_key_id,
                    proposals,
                    batch_size,
                )
                .await
            }
        },
    )
    .await?;
    Ok(Json(AcceptResponse {
        accepted: locked.accepted,
        failed: merge_failed_in_order(&tx_ids, preflight_failed, locked.failed),
    }))
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
    batch_size: i64,
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
                batch_size,
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
///
/// **Story 27-4** : `proposals` n'est plus le lot reçu mais son sous-lot
/// **éligible** (le pré-vol de `post_accept` a déjà refusé les autres) ;
/// `batch_size` est donc passé explicitement — la taille du lot **reçu**,
/// que porte la trace `reconciliation.accepted` (AC2).
async fn accept_batch(
    tx_outer: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
    proposals: &[AcceptProposalInput],
    batch_size: i64,
) -> Result<AcceptResponse, ReconciliationError> {
    let mut accepted: Vec<AcceptedProposal> = Vec::new();
    let mut failed: Vec<FailedProposal> = Vec::new();

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
        AcceptProposalInput::SupplierInvoice {
            bank_transaction_id,
            supplier_invoice_id,
        } => {
            accept_one_supplier_invoice(
                tx,
                company_id,
                bank_account_id,
                letterings::Actor {
                    user_id,
                    api_key_id: actor_api_key_id,
                },
                *bank_transaction_id,
                *supplier_invoice_id,
                batch_size,
            )
            .await
        }
    }
}

/// Convertit le refus « compte = créance » (Story 15-6b, #474) en
/// `FailedProposal` — **une seule** fonction pour l'étape (b bis) et (c-bis),
/// `details` par le helper commun `claim_account_refusal_details`.
/// `bank_account_id` n'est fourni que pour le rôle `counterparty`. Toute autre
/// erreur — celle de la lecture du numéro — est un `DATABASE_ERROR`, comme aux
/// étapes voisines (pattern batch : jamais d'`AppError` global).
fn claim_account_failed_proposal(
    bank_transaction_id: i64,
    err: DbError,
    bank_account_id: Option<i64>,
) -> FailedProposal {
    match &err {
        DbError::SettlementCounterpartyIsClaimAccount {
            account_id,
            account_number,
            claim,
            role,
            ..
        } => FailedProposal {
            bank_transaction_id,
            error_code: err.error_code().to_string(),
            details: Some(invoice_settlements::claim_account_refusal_details(
                *account_id,
                account_number.as_deref(),
                *claim,
                *role,
                bank_account_id,
            )),
        },
        other => FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": other.to_string() })),
        },
    }
}

/// Convertit une erreur de la **synchronisation du lettrage**
/// (`letterings::sync_invoice_in_tx`, Story 15-1a2-i, P4) en `FailedProposal`
/// — jamais en `AppError` global (pattern batch). Même rôle que
/// [`claim_account_failed_proposal`].
///
/// - Défaut structurel → `INTERNAL_ERROR`, précédé d'un `tracing::error!`,
///   comme l'étape (e) d'`accept_one_invoice` : `Invariant`, et tout refus de
///   lettrage que la synchronisation exclut par construction — dont les refus
///   propres au mode `Manual` (périodes, ligne de pièce, groupe `document`),
///   que le mode `System` n'évalue pas (revue P1, B-3). ⛔
///   `LETTERING_CONCURRENT_CHANGE` en fait partie (C-15-1a2-14) : l'`UPDATE` de
///   la primitive vise des lignes tenues `FOR UPDATE` par la même transaction,
///   le compte est égal par construction — le code n'entre pas dans `failed[]`.
/// - Toute autre erreur (`Sqlx`, interblocage…) → `DATABASE_ERROR`
///   (`details.message`), comme les étapes voisines ; un interblocage est
///   ensuite rejoué par `accept_batch` (`retry_with`).
fn lettering_error_to_failed_proposal(bank_transaction_id: i64, err: DbError) -> FailedProposal {
    match err {
        DbError::Invariant(_)
        | DbError::LetteringConcurrentChange
        | DbError::LetteringLineAlreadyLettered { .. }
        | DbError::LetteringUnbalanced { .. }
        | DbError::LetteringAccountNotLetterable
        | DbError::LetteringAccountsDiffer
        | DbError::LetteringTooFewLines
        | DbError::LetteringTooManyLines { .. }
        | DbError::LetteringAllLinesInClosedPeriods
        | DbError::LetteringLineOwnedByDocument { .. }
        | DbError::LetteringIsDocument
        | DbError::NotFound => {
            tracing::error!("encaissement : lettrage de la facture impossible : {err}");
            FailedProposal {
                bank_transaction_id,
                error_code: "INTERNAL_ERROR".to_string(),
                details: None,
            }
        }
        other => FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": other.to_string() })),
        },
    }
}

/// Story 28-6 (#482, AC1) — un règlement de facture client exige un
/// **encaissement** : refuse une transaction de débit ou de montant nul.
///
/// Fonction pure, appelée par [`accept_one_invoice`] sur la transaction
/// **rechargée sous le verrou du compte**. C'est la même règle que le filtre
/// de [`get_proposals`] (`tx.amount <= Decimal::ZERO` → aucune facture
/// proposée), écrite une seconde fois parce que `POST /accept` est appelable
/// directement — par une clé d'API comme par un appel forgé — avec un couple
/// `(bankTransactionId, invoiceId)` arbitraire. Sans elle, un débit dont la
/// référence et la contrepartie ressemblaient à la facture (cas exécuté au
/// rouge de la story) passait la garde de score, et seule une contrainte de la
/// base (`chk_jel_*`) arrêtait l'écriture : le refus arrivait en
/// `DATABASE_ERROR`, qui invite au rejeu.
///
/// Rend `None` pour un montant strictement positif ; sinon le refus
/// `RECONCILIATION_TRANSACTION_NOT_CREDIT`, avec `details.reason` =
/// `debit_transaction` (montant négatif) ou `zero_amount_transaction`, et
/// `details.transactionAmount` sous la forme du champ homonyme de
/// `RECONCILIATION_OVERPAYMENT`.
///
/// ⚠️ La garde **compare** au zéro, elle ne teste pas le signe : un zéro
/// négatif (`-0.00`, porté par un `Decimal` mais jamais rendu par la colonne
/// `DECIMAL(18,2)`) est un montant nul, et `is_sign_negative()` le classerait
/// en débit (C-28-6-12, C-28-6-15).
///
/// Depuis la Story 30-1b, enveloppe d'une ligne de [`refuse_wrong_direction`]
/// (sens `Credit`) : code, raisons et `details` strictement inchangés.
fn refuse_non_credit_settlement(
    bank_transaction_id: i64,
    amount: Decimal,
) -> Option<FailedProposal> {
    refuse_wrong_direction(bank_transaction_id, amount, SettlementDirection::Credit)
}

/// Le sens qu'un règlement attend de la transaction bancaire (Story 30-1b,
/// #686, AC-b7) : un **crédit** pour une facture client (un encaissement), un
/// **débit** pour une facture fournisseur (un décaissement).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettlementDirection {
    Credit,
    Debit,
}

/// **Une seule** garde de sens pour les deux sortes de factures (Story 30-1b,
/// #686, AC-b7, DRY) : rend `None` si le montant est du sens attendu, sinon le
/// refus — `RECONCILIATION_TRANSACTION_NOT_CREDIT` (client : raisons
/// `debit_transaction`, `zero_amount_transaction`) ou
/// `RECONCILIATION_TRANSACTION_NOT_DEBIT` (fournisseur : raisons
/// `credit_transaction`, `zero_amount_transaction`), avec
/// `details.transactionAmount` tel quel.
///
/// ⚠️ Comparaison **au zéro**, jamais par `is_sign_negative()` : un zéro
/// négatif (`-0.00`) est un montant nul (C-28-6-12).
///
/// Chaque code est écrit **au site de construction** (`error_code: "…"`),
/// forme que recompte le doc-comment de `failed-proposal-label.ts`.
fn refuse_wrong_direction(
    bank_transaction_id: i64,
    amount: Decimal,
    expected: SettlementDirection,
) -> Option<FailedProposal> {
    let details = |opposite_reason: &str| {
        let reason = if amount.is_zero() {
            "zero_amount_transaction"
        } else {
            opposite_reason
        };
        Some(serde_json::json!({
            "reason": reason,
            "transactionAmount": amount,
        }))
    };
    match expected {
        SettlementDirection::Credit if amount > Decimal::ZERO => None,
        SettlementDirection::Credit => Some(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_TRANSACTION_NOT_CREDIT".to_string(),
            details: details("debit_transaction"),
        }),
        SettlementDirection::Debit if amount < Decimal::ZERO => None,
        SettlementDirection::Debit => Some(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_TRANSACTION_NOT_DEBIT".to_string(),
            details: details("credit_transaction"),
        }),
    }
}

/// La traduction **commune** d'un refus de la fenêtre de dates en
/// `FailedProposal` (Story 30-1b, #686, AC-b2, DRY) — pour une facture client
/// (`accept_one_invoice`) comme pour une facture fournisseur
/// (`accept_one_supplier_invoice`) : `RECONCILIATION_INVOICE_NOT_ELIGIBLE`,
/// `details.reason` = `payment_date_before_invoice_date` (avec
/// `tolerance_days`, champ ajouté par la 30-1b : 1 client, 7 fournisseur) ou
/// `payment_date_outside_window` (avec `window_days` : 30 client, 365
/// fournisseur). Les nombres de jours sont lus sur la fenêtre elle-même.
fn date_refusal_failed_proposal(
    bank_transaction_id: i64,
    refusal: InvoiceDateRefusal,
    window: InvoiceDateWindow,
) -> FailedProposal {
    let details = match refusal {
        InvoiceDateRefusal::PaymentBeforeInvoice => serde_json::json!({
            "reason": "payment_date_before_invoice_date",
            "tolerance_days": window.tolerance_days,
        }),
        InvoiceDateRefusal::PaymentTooLate => serde_json::json!({
            "reason": "payment_date_outside_window",
            "window_days": window.window_days,
        }),
    };
    FailedProposal {
        bank_transaction_id,
        error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
        details: Some(details),
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
/// **Sens de la transaction** (Story 28-6, #482) : aussitôt le statut
/// `pending` vérifié (étape 4bis), une transaction de débit ou de montant nul
/// est refusée en `RECONCILIATION_TRANSACTION_NOT_CREDIT`
/// ([`refuse_non_credit_settlement`]) — **avant** la lecture de la facture,
/// parce que le refus ne dépend que de la transaction : un débit proposé
/// contre une facture inexistante rend ce code, non `INVOICE_NOT_FOUND`.
///
/// **Postabilité — pas de garde ici, et c'est voulu** (Story 15-5b, AC6) :
/// aucun compte de cette écriture ne vient du client. Le compte bancaire est
/// un compte de configuration (D-A0, toléré devenu non imputable) ; la créance
/// est lue **sur l'écriture de vente** de la facture
/// (`invoice_settlements::sale_receivable_account`, Story 15-6a) ; le compte d'arrondi vient
/// des réglages, déjà gardé `postable = TRUE` par
/// `company_invoice_settings::rounding_account_for_write`. Les deux autres
/// flux d'acceptation (`split`, `rule`) et les rapprochements manuel et ventilé
/// gardent, eux, le compte de contrepartie venu du client.
///
/// ⚠️ **Mais ne pas venir du client ne veut pas dire ne pas pouvoir être la
/// créance** (Story 15-6b, #474, amendement du classement ci-dessus) : le compte
/// de banque vient de la **configuration** du compte bancaire, et rien n'y
/// interdit le 1100 ; le compte d'arrondi vient des réglages, et la créance
/// retypée en charge passe son contrôle de type. L'un et l'autre écriraient
/// `D 1100 / C 1100`. Ils sont donc **comparés à la créance** — le compte de
/// banque aussitôt la créance lue (étape b bis, avant le trop-perçu), le compte
/// d'arrondi aussitôt lu (c-bis) — et le refus est une proposition de
/// `failed[]` (`SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`), jamais une erreur
/// globale.
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

    // Step 4bis — Story 28-6 (#482) : un débit ou un montant nul ne règle
    // jamais une facture client. Sur la transaction rechargée sous le verrou
    // du compte, avant la lecture de la facture : le refus ne dépend que de
    // la transaction.
    if let Some(refusal) =
        refuse_non_credit_settlement(bank_transaction_id, bank_transaction.amount)
    {
        return Err(refusal);
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
    // Lecture partagée avec l'acceptation fournisseur (`contact_for_score`).
    let contact =
        contact_for_score(tx, company_id, invoice.contact_id, bank_transaction_id).await?;

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
    // Date du règlement écrit : date de valeur, à défaut de comptabilisation.
    let paid_at_candidate = payment_date(&bank_transaction);
    // Story 29-2 (#529) — la date d'ancrage de la facture (la plus tardive
    // de sa date, de son échéance et de son dernier rappel non annulé), lue
    // par la même expression SQL que la proposition. `SELECT` ordinaire, sans
    // verrou, après la lecture ordinaire de la facture (étape 5) : la vue de
    // la transaction est déjà née. Course admise : un rappel annulé ou envoyé
    // entre l'affichage des propositions et l'acceptation déplace l'ancre ;
    // l'acceptation tranche sur ce qu'elle lit, comme pour une facture
    // redatée (Story 28-7).
    let anchor_date =
        match reconciliation_repo::invoice_payment_anchor_date(&mut **tx, company_id, invoice.id)
            .await
        {
            Ok(Some(a)) => a,
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
    // Story 28-7 (#548), Story 29-2 (#529) — la fenêtre de dates que la
    // proposition a déjà appliquée (`get_proposals`) : le paiement au plus un
    // jour avant la date de la facture, au plus 30 jours après sa date
    // d'ancrage. Codes, raisons et `details` inchangés (contrat d'API) ; le
    // refus n'est plus atteint qu'à une facture redatée, ou dont un rappel a
    // été annulé, entre la proposition et l'acceptation, ou par un appel
    // d'API direct.
    // Story 30-1b : la traduction du refus est commune aux deux sortes de
    // factures ; `details` gagne seulement `tolerance_days` (additif).
    let window = InvoiceDateWindow::for_transaction(&bank_transaction);
    if let Err(refusal) = window.check(invoice.date, anchor_date) {
        return Err(date_refusal_failed_proposal(
            bank_transaction_id,
            refusal,
            window,
        ));
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
    //     Lecteur partagé avec le règlement, le solde du reste et l'avoir
    //     (Story 15-6a) : la règle et l'ordre des lignes (créance PREMIÈRE ligne
    //     au débit, arrondi négatif après elle) sont à son doc-comment.
    let receivable_row =
        invoice_settlements::sale_receivable_account(tx, company_id, sale_entry_id)
            .await
            .map_err(|e| FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            })?;
    let receivable_account_id = match receivable_row {
        Some(id) => id,
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

    // (b bis) ⛔ Story 15-6b (#474) — le compte de banque n'est pas la créance.
    //         Après tous les refus antérieurs (score, compte bancaire sans
    //         journal, écriture de vente malformée), avant le trop-perçu.
    invoice_settlements::refuse_if_claim_account(
        tx,
        company_id,
        bank_ledger_account_id,
        receivable_account_id,
        ClaimSubject::Counterparty(ClaimSide::Receivable),
    )
    .await
    .map_err(|e| claim_account_failed_proposal(bank_transaction_id, e, Some(bank_account_id)))?;

    // (c) ⛔ Le trop-perçu est REFUSÉ, il ne s'écrit pas. Sans ce garde, le
    //     compte de créance passerait CRÉDITEUR — un solde contre nature que le
    //     grand livre signalerait, mais après coup.
    //
    //     #476 (25-4-c3-b) : à DOUBLE BORNE, par la classification partagée avec
    //     le règlement manuel. Une transaction égale au reste arrondi au centime
    //     solde la facture : le règlement s'enregistre au reste BRUT et l'écart
    //     passe sur le compte de différences d'arrondi.
    //
    //     #665 (Story 29-6) : la proposition n'offre plus ce que ce garde
    //     refuse (`least_due_without_overpayment`) ; il ne se rencontre plus
    //     que par un appel direct, ou une facture modifiée entre les deux.
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
            // Story 15-6b (AC3 bis) — le compte d'arrondi n'est pas la créance :
            // `details` sans `bankAccountId`, qui n'a pas de sens ici.
            Ok(id) => {
                invoice_settlements::refuse_if_claim_account(
                    tx,
                    company_id,
                    id,
                    receivable_account_id,
                    ClaimSubject::Designated(GapAccountRole::Rounding),
                )
                .await
                .map_err(|e| claim_account_failed_proposal(bank_transaction_id, e, None))?;
                Some(id)
            }
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

    // (g bis) Le lettrage de la pièce (Story 15-1a2-i, P4) — APRÈS (g) : la
    //     ligne `invoices` est désormais tenue par l'`UPDATE`, et une
    //     modification concurrente a déjà été refusée en `race_during_update`
    //     sans qu'aucune marque soit posée. ⛔ Pas dans `create_in_tx` : appelé
    //     avant (g), il prendrait les lignes avant la facture (C-15-1a2-6).
    //     Exercice tenu : celui de (d). Acteur : la clé d'API, s'il y en a une
    //     (seul geste qui la porte, AC10). Toute erreur est per-proposal.
    letterings::sync_invoice_in_tx(
        tx,
        company_id,
        invoice_id,
        fiscal_year.id,
        letterings::Actor {
            user_id,
            api_key_id: actor_api_key_id,
        },
    )
    .await
    .map_err(|e| lettering_error_to_failed_proposal(bank_transaction_id, e))?;

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
        supplier_invoice_id: None,
        journal_entry_id,
        score,
    })
}

// ============================================================
// Story 30-1b (#686) — accepter une facture fournisseur
// ============================================================

/// Un refus de [`supplier_invoices::lock_for_bank_settlement_in_tx`] (étape 3
/// d'[`accept_one_supplier_invoice`]) en `FailedProposal` (AC-b6).
fn supplier_settlement_refusal_failed_proposal(
    bank_transaction_id: i64,
    bank_account_id: i64,
    supplier_invoice_id: i64,
    refusal: SupplierSettlementRefusal,
) -> FailedProposal {
    // Chaque code au site de construction (`error_code: "…"`), pour le
    // décompte de `failed-proposal-label.ts`.
    match refusal {
        // Indiscernable d'une facture d'une autre société (KF-002).
        SupplierSettlementRefusal::NotFound => FailedProposal {
            bank_transaction_id,
            error_code: "SUPPLIER_INVOICE_NOT_FOUND".to_string(),
            details: Some(serde_json::json!({ "supplierInvoiceId": supplier_invoice_id })),
        },
        SupplierSettlementRefusal::NotOpen { .. } => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({ "reason": "supplier_invoice_not_open" })),
        },
        SupplierSettlementRefusal::BankAccountNotFound => FailedProposal {
            bank_transaction_id,
            error_code: "BANK_ACCOUNT_NOT_FOUND".to_string(),
            details: None,
        },
        SupplierSettlementRefusal::BankAccountClosed => FailedProposal {
            bank_transaction_id,
            error_code: "BANK_ACCOUNT_CLOSED".to_string(),
            details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
        },
        SupplierSettlementRefusal::BankAccountNotConfigured => FailedProposal {
            bank_transaction_id,
            error_code: "BANK_ACCOUNT_NOT_CONFIGURED".to_string(),
            details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
        },
        SupplierSettlementRefusal::InGeneratedBatch { payment_batch_id } => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({
                "reason": "supplier_invoice_in_payment_batch",
                "paymentBatchId": payment_batch_id,
            })),
        },
        SupplierSettlementRefusal::HeldByPaymentBatch {
            supplier_invoice_id: held,
            payment_batch_id,
        } => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_SUPPLIER_PAYMENT_HOLD".to_string(),
            details: Some(serde_json::json!({
                "reason": "other_invoice_in_payment_batch",
                "supplierInvoiceId": held,
                "paymentBatchId": payment_batch_id,
            })),
        },
    }
}

/// Une erreur du règlement (`pay_in_tx_as`, étape 6 d'[`accept_one_supplier_invoice`],
/// ou une erreur de base de l'étape 3) en `FailedProposal` — **jamais** une
/// erreur globale (§ *Pattern batch*). Inventaire clos (AC-b6, refait au
/// développement et recopié au Dev Agent Record) : les variantes **nommées**,
/// puis un bras final `DATABASE_ERROR`.
///
/// ⛔ `NotFound` et `IllegalStateTransition` sont **inatteignables** à
/// l'étape 6 — facture et compte sont lus et verrouillés à l'étape 3,
/// l'exercice trouvé sous verrou avant l'écriture, et le règlement ne valide
/// aucun projet : des invariants rompus, classés `INTERNAL_ERROR`, et non par
/// `project_error_to_failed_proposal`, qui les rendrait `PROJECT_*` à tort.
fn supplier_payment_error_to_failed_proposal(
    bank_transaction_id: i64,
    bank_account_id: i64,
    project_id: Option<i64>,
    value_date: NaiveDate,
    err: DbError,
) -> FailedProposal {
    let internal = |err: &DbError| {
        tracing::error!("règlement fournisseur par la réconciliation : invariant rompu : {err}");
        FailedProposal {
            bank_transaction_id,
            error_code: "INTERNAL_ERROR".to_string(),
            details: None,
        }
    };
    match &err {
        DbError::SettlementCounterpartyIsClaimAccount { .. } => {
            claim_account_failed_proposal(bank_transaction_id, err, Some(bank_account_id))
        }
        DbError::InactiveOrInvalidAccounts => FailedProposal {
            bank_transaction_id,
            error_code: err.error_code().to_string(),
            details: None,
        },
        DbError::FiscalYearInvalid | DbError::FiscalYearClosed | DbError::DateOutsideFiscalYear => {
            FailedProposal {
                bank_transaction_id,
                error_code: "FISCAL_YEAR_INVALID".to_string(),
                details: Some(serde_json::json!({ "valueDate": value_date })),
            }
        }
        DbError::PeriodLocked {
            locked_through,
            attempted,
        } => period_locked_failed_proposal(
            bank_transaction_id,
            project_id,
            *locked_through,
            *attempted,
        ),
        DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        } => later_fiscal_year_closed_failed_proposal(
            bank_transaction_id,
            project_id,
            *fiscal_year_id,
            fiscal_year_name,
        ),
        // Défensifs : l'étape 3 tient le compte en exclusif et l'a refusé
        // avant, s'il était clos ou sans compte au grand livre.
        DbError::BankAccountClosed { .. } => FailedProposal {
            bank_transaction_id,
            error_code: "BANK_ACCOUNT_CLOSED".to_string(),
            details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
        },
        DbError::ConfigurationRequired(_) => FailedProposal {
            bank_transaction_id,
            error_code: "BANK_ACCOUNT_NOT_CONFIGURED".to_string(),
            details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
        },
        DbError::OptimisticLockConflict => FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_INVOICE_NOT_ELIGIBLE".to_string(),
            details: Some(serde_json::json!({ "reason": "race_during_update" })),
        },
        DbError::NotFound
        | DbError::IllegalStateTransition(_)
        | DbError::Invariant(_)
        | DbError::LetteringConcurrentChange
        | DbError::LetteringLineAlreadyLettered { .. }
        | DbError::LetteringUnbalanced { .. }
        | DbError::LetteringAccountNotLetterable
        | DbError::LetteringAccountsDiffer
        | DbError::LetteringTooFewLines
        | DbError::LetteringTooManyLines { .. }
        | DbError::LetteringAllLinesInClosedPeriods
        | DbError::LetteringLineOwnedByDocument { .. }
        | DbError::LetteringIsDocument => internal(&err),
        other => FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": other.to_string() })),
        },
    }
}

/// Le contact d'une facture — client ou fournisseur —, lu pour le seul calcul du
/// score de l'acceptation (`accept_one_invoice`, `accept_one_supplier_invoice`).
///
/// `contact_id <= 0` : aucun contact lié (sentinelle 0, M1 Pass 1), aucune
/// lecture. Une erreur de base n'est **plus avalée** (revue de code P1 de la
/// 30-1b, B1 — elle rendait un score de contact nul, que la garde de score
/// minimal du chemin client pouvait changer en refus `score_zero_no_match`
/// trompeur) : elle rend `DATABASE_ERROR` par proposition, comme toute autre
/// lecture de l'acceptation ; un interblocage y est repris par le rejeu du lot
/// (le point de sauvegarde perdu rend `TransactionAborted`).
async fn contact_for_score(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    contact_id: i64,
    bank_transaction_id: i64,
) -> Result<Option<kesh_db::entities::Contact>, FailedProposal> {
    if contact_id <= 0 {
        return Ok(None);
    }
    contacts_repo::find_by_id_in_company(&mut **tx, contact_id, company_id)
        .await
        .map_err(|e| FailedProposal {
            bank_transaction_id,
            error_code: "DATABASE_ERROR".to_string(),
            details: Some(serde_json::json!({ "message": e.to_string() })),
        })
}

/// Story 30-1b (#686, AC-b6) — traite UNE proposition `type='supplier_invoice'`
/// dans son `SAVEPOINT` : le **débit** `bank_transaction_id` règle la facture
/// fournisseur **ouverte** `supplier_invoice_id`.
///
/// Le règlement a **une** définition, celle de la fiche et des lots
/// ([`supplier_invoices::pay_in_tx_as`], appelé, jamais réécrit — C-30-1-10) ;
/// ce que la réconciliation ajoute, dans cet ordre :
///
/// 1. la transaction rechargée (lecture ordinaire : **naissance de la vue**),
///    `pending` ;
/// 2. le **sens** : un crédit ou un montant nul est refusé
///    (`RECONCILIATION_TRANSACTION_NOT_DEBIT`), avant toute lecture de la
///    facture ([`refuse_wrong_direction`]) ;
/// 3. les **verrous et l'éligibilité**, en `kesh-db` :
///    [`supplier_invoices::lock_for_bank_settlement_in_tx`] — facture, compte,
///    lot verrouillant, garde (d) du lot du compte ;
/// 4. la **fenêtre** fournisseur sur la facture verrouillée (`F ≤ D + 7`,
///    exigibilité `≥ D − 365`, [`date_refusal_failed_proposal`]) ;
/// 5. le **montant** : `|tx| = TTC` exactement, sinon
///    `RECONCILIATION_AMOUNT_MISMATCH` ; le score est recalculé pour la trace
///    et la réponse — ≥ 0,5 puisque l'égalité vient d'être vérifiée ; aucune
///    garde de score (C-30-1-16) ;
/// 6. le **règlement** (`pay_in_tx_as`, virement depuis le compte de
///    l'import, daté de la date de paiement, acteur et clé d'API compris) ;
/// 7. le **lien** de la transaction (`… AND status = 'pending' AND version = ?`) ;
/// 8. l'audit `reconciliation.accepted` (`for_actor`).
///
/// ⛔ Aucune garde « dépense déjà réglée par un autre chemin » : l'angle mort
/// L1 (#671).
#[allow(clippy::too_many_arguments)]
async fn accept_one_supplier_invoice(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    actor: letterings::Actor,
    bank_transaction_id: i64,
    supplier_invoice_id: i64,
    batch_size: i64,
) -> Result<AcceptedProposal, FailedProposal> {
    let database_error = |e: &dyn std::fmt::Display| FailedProposal {
        bank_transaction_id,
        error_code: "DATABASE_ERROR".to_string(),
        details: Some(serde_json::json!({ "message": e.to_string() })),
    };

    // (1) La transaction, rechargée sous le verrou du compte.
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
        Err(e) => return Err(database_error(&e)),
    };
    if bank_transaction.status != BankTransactionStatus::Pending {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
            details: None,
        });
    }

    // (2) Le sens : un débit, jamais un crédit ni un montant nul (#482).
    if let Some(refusal) = refuse_wrong_direction(
        bank_transaction_id,
        bank_transaction.amount,
        SettlementDirection::Debit,
    ) {
        return Err(refusal);
    }
    let amount = bank_transaction.amount.abs();
    let value_date = payment_date(&bank_transaction);

    // (3) Verrous et éligibilité.
    let invoice = match supplier_invoices::lock_for_bank_settlement_in_tx(
        tx,
        company_id,
        supplier_invoice_id,
        bank_account_id,
        amount,
    )
    .await
    {
        Ok(Ok(locked)) => locked.invoice,
        Ok(Err(refusal)) => {
            return Err(supplier_settlement_refusal_failed_proposal(
                bank_transaction_id,
                bank_account_id,
                supplier_invoice_id,
                refusal,
            ));
        }
        Err(e) => {
            return Err(supplier_payment_error_to_failed_proposal(
                bank_transaction_id,
                bank_account_id,
                None,
                value_date,
                e,
            ));
        }
    };

    // (4) La fenêtre fournisseur, sur la facture verrouillée.
    let window = InvoiceDateWindow::for_supplier_transaction(&bank_transaction);
    if let Err(refusal) = window.check(invoice.invoice_date, invoice_exigibility_date(&invoice)) {
        return Err(date_refusal_failed_proposal(
            bank_transaction_id,
            refusal,
            window,
        ));
    }

    // (5) Le montant : le règlement est binaire, du TTC entier (C-30-1-5).
    if amount != invoice.total_amount {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_AMOUNT_MISMATCH".to_string(),
            details: Some(serde_json::json!({
                "amountDue": invoice.total_amount,
                "transactionAmount": bank_transaction.amount,
            })),
        });
    }
    let contact =
        contact_for_score(tx, company_id, invoice.contact_id, bank_transaction_id).await?;
    let score = propose_matches(
        &bank_transaction,
        &[(invoice.clone(), contact, invoice.total_amount)],
    )
    .first()
    .map(|p| p.score)
    .unwrap_or(MatchScore {
        total: 0.0,
        amount_score: 0.0,
        reference_score: 0.0,
        contact_score: 0.0,
    });

    // (6) Le règlement — le geste de la fiche, appelé.
    let paid = match supplier_invoices::pay_in_tx_as(
        tx,
        company_id,
        invoice.id,
        kesh_db::entities::SettlementChoice::BankTransfer { bank_account_id },
        value_date,
        actor,
    )
    .await
    {
        Ok(p) => p.invoice,
        Err(e) => {
            return Err(supplier_payment_error_to_failed_proposal(
                bank_transaction_id,
                bank_account_id,
                invoice.project_id,
                value_date,
                e,
            ));
        }
    };
    let Some(journal_entry_id) = paid.settlement_journal_entry_id else {
        tracing::error!(
            "règlement fournisseur sans écriture de règlement : {}",
            paid.id
        );
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "INTERNAL_ERROR".to_string(),
            details: None,
        });
    };

    // (7) Le lien de la transaction.
    let linked = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = ?, status = 'reconciled', updated_at = NOW(3), version = version + 1 \
         WHERE id = ? AND company_id = ? AND status = 'pending' AND version = ?",
    )
    .bind(journal_entry_id)
    .bind(bank_transaction_id)
    .bind(company_id)
    .bind(bank_transaction.version)
    .execute(&mut **tx)
    .await
    .map_err(|e| database_error(&e))?;
    if linked.rows_affected() != 1 {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
            details: Some(serde_json::json!({ "reason": "race_during_update" })),
        });
    }

    // (8) L'audit du rapprochement ; `supplier_invoice.paid` est écrit par
    //     `pay_in_tx_as`.
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            actor.user_id,
            actor.api_key_id,
            "reconciliation.accepted",
            "bank_transaction",
            bank_transaction_id,
            Some(serde_json::json!({
                "type": "supplier_invoice",
                "bank_transaction_id": bank_transaction_id,
                "supplier_invoice_id": invoice.id,
                "score": score,
                "batch_size": batch_size,
                "journal_entry_id": journal_entry_id,
                "purchase_journal_entry_id": invoice.purchase_journal_entry_id,
                "paid_amount": amount,
                "transaction_amount": bank_transaction.amount,
            })),
        ),
    )
    .await
    .map_err(|e| database_error(&e))?;

    Ok(AcceptedProposal {
        bank_transaction_id,
        invoice_id: 0,
        supplier_invoice_id: Some(invoice.id),
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
    // Story 15-6d (AC3) : refus et comparaison partagés avec `accept_one_rule`
    // (même place : après la « Step c », avant la « Step d ») et, pour la
    // comparaison, avec `post_split` et `post_manual` — cf. `is_bank_ledger`.
    for s in splits {
        if is_bank_ledger(s.counterparty_account_id, bank_ledger_account_id) {
            return Err(counterparty_is_bank_ledger_failed_proposal(
                bank_transaction_id,
            ));
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
        supplier_invoice_id: None,
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
///
/// **Ordre des refus** (Story 15-6d, #524) : étape 1, compte bancaire
/// (`DATABASE_ERROR`, `BANK_ACCOUNT_NOT_CONFIGURED`, `BANK_ACCOUNT_NOT_FOUND`)
/// → 1 bis, compte de banque **actif** (`DATABASE_ERROR`,
/// `BANK_ACCOUNT_NOT_CONFIGURED`) → 2 (`RECONCILIATION_RULE_NOT_FOUND`) → 4
/// (`RECONCILIATION_RULE_MISMATCH`) → 4 bis, contrepartie = compte de banque
/// (`VALIDATION_ERROR` / `counterparty_equals_bank_ledger`, forme et place
/// d'`accept_one_split` — cf. [`is_bank_ledger`]) → 5 (`ACCOUNT_NOT_FOUND`,
/// `ACCOUNT_NOT_POSTABLE`) → 6 et la suite.
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

    // Step 1 bis — Story 15-6d (AC2, choix C-15-6-28) : le compte de banque est
    // ACTIF — la « Step c » d'`accept_one_split`. Sans elle, un compte de banque
    // archivé rendait ici `DATABASE_ERROR` (garde `active` de l'écriture) ou
    // `ACCOUNT_NOT_FOUND` (règle sur ce compte), quand les trois autres chemins
    // rendent `BANK_ACCOUNT_NOT_CONFIGURED`.
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
    if !matches!(ledger_active, Some((true,))) {
        return Err(FailedProposal {
            bank_transaction_id,
            error_code: "BANK_ACCOUNT_NOT_CONFIGURED".to_string(),
            details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
        });
    }

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

    // Step 4 bis — Story 15-6d (AC2, #524) : la contrepartie de la règle n'est
    // pas le compte de la banque — même refus et même place que dans
    // `accept_one_split` (après le compte de banque actif, avant la validation
    // de la contrepartie, donc avant `ACCOUNT_NOT_POSTABLE`). `get_proposals` ne
    // propose plus une telle règle (AC2 bis) : cette garde est le filet d'un
    // client d'API ou d'une règle modifiée entre la proposition et
    // l'acceptation.
    if is_bank_ledger(counterparty_account_id, bank_ledger_account_id) {
        return Err(counterparty_is_bank_ledger_failed_proposal(
            bank_transaction_id,
        ));
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

    // Step 7 bis — Story 30-1b (#686, AC-b17) : le rejeu du masquage. La
    // proposition n'offre pas la règle quand une facture la défend ; un appel
    // d'API direct, ou un écran chargé avant la saisie de la facture dans un
    // autre onglet, l'accepterait et compterait la charge (ou le produit)
    // deux fois. Lectures ordinaires, mêmes fonctions que `get_proposals`
    // (`load_rule_mask`, `rule_mask_for`), avant toute écriture. Trois cas
    // seulement : `payment_batch`, `invoice_candidate` (débit),
    // `invoice_slightly_overpaid` (crédit). Le geste attendu est de traiter la
    // facture (l'accepter, ou rattacher la transaction à son règlement) ;
    // `POST /reconciliation/manual` ou l'éclatement ne conviennent qu'à une
    // transaction qui ne règle aucune facture saisie. ⚠️ Changement de contrat
    // des deux côtés : une acceptation autrefois réussie est refusée.
    match load_rule_mask(tx, company_id, bank_account_id, &bt).await {
        Ok(None) => {}
        Ok(Some(mask)) => {
            return Err(rule_masked_failed_proposal(bank_transaction_id, &mask));
        }
        Err(e) => {
            return Err(FailedProposal {
                bank_transaction_id,
                error_code: "DATABASE_ERROR".to_string(),
                details: Some(serde_json::json!({ "message": e.to_string() })),
            });
        }
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
            // ⛔ Story 15-12b (#543, AC 11) : le filet sous un bilan clos, même
            // exception, même constructeur que les deux autres voies. `None`
            // pour `projectId`, comme le `PeriodLocked` ci-dessus : les deux
            // refus de cette branche publient les mêmes `details`.
            if let kesh_db::errors::DbError::LaterFiscalYearClosed {
                fiscal_year_id,
                fiscal_year_name,
            } = &e
            {
                return Err(later_fiscal_year_closed_failed_proposal(
                    bank_transaction_id,
                    None,
                    *fiscal_year_id,
                    fiscal_year_name,
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
        supplier_invoice_id: None,
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
///
/// **Story 27-4 (#634)** : le pré-vol **classe** chaque transaction
/// (`classify_batch_transaction`) au lieu de refuser le lot entier en
/// `400` ; seuls les identifiants éligibles passent sous verrou, triés par
/// identifiant croissant ; `failed[]` réunit les deux sources, **trié en
/// entier**. Un lot sans transaction éligible répond `200` sans transaction
/// ni verrou.
pub async fn post_reject(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<RejectBody>,
) -> Result<Json<RejectResponse>, AppError> {
    if body.bank_transaction_ids.is_empty() {
        return Err(AppError::Validation("bankTransactionIds vide".into()));
    }
    check_batch_size(body.bank_transaction_ids.len())?;
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
    // Story 28-4a (#642, R2) — compte clos : chaque transaction en `failed[]`
    // (triées, comme le reste de la réponse), `200`, sans verrou.
    if ba_check.is_closed() {
        let mut ids = body.bank_transaction_ids.clone();
        ids.sort_unstable();
        return Ok(Json(RejectResponse {
            rejected: Vec::new(),
            failed: closed_account_failed(&ids, ba_check.id),
        }));
    }

    // Story 27-1 (AC4-bis, #632) — verrouiller par identifiant CROISSANT, comme
    // la suppression d'un import (`bank_imports::delete_unreconciled_in_tx`) :
    // un lot `[7, 3]` contre une suppression qui tient 3 et attend 7 formerait
    // un cycle, et cette route n'est pas rejouée (une 1213 rendrait 500). Les
    // doublons sont refusés plus haut : le tri ne change pas l'ensemble traité.
    // Effets : `rejected[]`, `failed[]` et la trace sortent triés.
    let mut sorted_ids = body.bank_transaction_ids.clone();
    sorted_ids.sort_unstable();

    // Story 27-4 (AC4) — pré-vol hors transaction et hors verrou, sans filtre
    // de statut : chaque identifiant est classé, les refus vont dans
    // `failed[]`, seuls les éligibles passent sous verrou.
    let tx_map = reconciliation_repo::find_by_ids_for_account(
        &state.pool,
        current_user.company_id,
        body.bank_account_id,
        &sorted_ids,
    )
    .await?;
    let mut preflight_failed: HashMap<i64, FailedProposal> = HashMap::new();
    let mut ids: Vec<i64> = Vec::with_capacity(sorted_ids.len());
    for &id in &sorted_ids {
        match classify_batch_transaction(id, tx_map.get(&id)) {
            Some(refusal) => {
                preflight_failed.insert(id, refusal);
            }
            None => ids.push(id),
        }
    }
    drop(tx_map);

    // Story 27-4, AC3 — rien d'éligible : ni transaction, ni verrou.
    if ids.is_empty() {
        return Ok(Json(RejectResponse {
            rejected: Vec::new(),
            failed: merge_failed_in_order(&sorted_ids, preflight_failed, Vec::new()),
        }));
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

    let lock_result = with_account_lock(
        &mut tx_outer,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            reject_batch(
                tx_inner,
                company_id,
                bank_account_id,
                user_id,
                actor_api_key_id,
                &ids,
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
            Ok(Json(RejectResponse {
                rejected: response.rejected,
                failed: merge_failed_in_order(&sorted_ids, preflight_failed, response.failed),
            }))
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

/// Rejette, sous le verrou du compte, les identifiants **éligibles** au
/// pré-vol (triés par identifiant croissant, Story 27-1 AC4-bis).
///
/// **Story 27-4 (AC4)** : l'`UPDATE` ne touche la ligne que si elle est
/// encore du compte, `pending`, sans lien et non rejetée. S'il ne touche
/// rien, la ligne est **relue par une lecture verrouillante**
/// ([`reconciliation_repo::find_by_id_for_account_for_update`] — lecture
/// courante, jamais la vue cohérente du lot, qui pourrait dater d'avant une
/// suppression) et **reclassée** par `classify_batch_transaction`. Une
/// relecture éligible n'est pas atteignable (l'`UPDATE` à 0 ligne et la
/// relecture lisent la même version validée sous le même verrou) : repli
/// défensif tracé, `race_during_update`.
async fn reject_batch(
    tx_outer: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
    ids: &[i64],
) -> Result<RejectResponse, ReconciliationError> {
    let mut rejected: Vec<RejectedProposal> = Vec::new();
    let mut failed: Vec<FailedProposal> = Vec::new();

    for &id in ids {
        // MP4-5 Pass 4 — rows_affected check. Story 27-4 (AC4) : gardes
        // `bank_account_id` (une restauration concurrente réinsère les mêmes
        // identifiants, C-27-4-27), `matched_entry_id IS NULL` (« rapprochée »
        // au sens d'`is_reconciled`, C-27-4-30) et `auto_match_rejected_at IS
        // NULL` (un rejet concurrent ne ré-horodate pas la ligne, C-27-4-16).
        let result = sqlx::query(
            "UPDATE bank_transactions \
             SET auto_match_rejected_at = NOW(3), updated_at = NOW(3), version = version + 1 \
             WHERE id = ? AND company_id = ? AND bank_account_id = ? AND status = 'pending' \
               AND matched_entry_id IS NULL AND auto_match_rejected_at IS NULL",
        )
        .bind(id)
        .bind(company_id)
        .bind(bank_account_id)
        .execute(&mut **tx_outer)
        .await?;

        if result.rows_affected() != 1 {
            let current = reconciliation_repo::find_by_id_for_account_for_update(
                &mut **tx_outer,
                company_id,
                bank_account_id,
                id,
            )
            .await
            .map_err(ReconciliationError::Db)?;
            match classify_batch_transaction(id, current.as_ref()) {
                Some(refusal) => failed.push(refusal),
                None => {
                    tracing::warn!(
                        bank_transaction_id = id,
                        "reconciliation reject: UPDATE touched no row but the locking re-read \
                         finds the transaction eligible — defensive refusal"
                    );
                    failed.push(FailedProposal {
                        bank_transaction_id: id,
                        error_code: "RECONCILIATION_ALREADY_RECONCILED".to_string(),
                        details: Some(serde_json::json!({ "reason": "race_during_update" })),
                    });
                }
            }
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

    // Audit log unique pour le batch (HP3-2 Pass 3 — entity_id = plus petit
    // identifiant rejeté, le lot étant trié — Story 27-1, AC4-bis).
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
///    2bis. compte de banque actif → 412 BANK_ACCOUNT_NOT_CONFIGURED.
///    2ter. `counterpartyAccountId` = compte de banque → 400 VALIDATION_ERROR
///    (Story 15-6d, #524) — même refus et même place que `post_split` et que
///    le lot (`accept_one_split`, `accept_one_rule`) : cf. [`is_bank_ledger`].
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
    // Story 28-4a (#642, R3) — compte clos : `409 BANK_ACCOUNT_CLOSED`.
    if bank_account.is_closed() {
        return Err(AppError::BankAccountClosed {
            bank_account_id: bank_account.id,
        });
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

    // Step 2 ter — Story 15-6d (AC1, #524) : la contrepartie ne peut pas être
    // le compte de la banque (l'écriture `D banque / C banque` serait nulle).
    // Même refus et même place que le flux ventilé (`post_split`, choix
    // C-15-6-23) : après le contrôle du compte de banque actif, avant le 404 et
    // `ACCOUNT_NOT_POSTABLE` de la contrepartie, et avant l'état de la
    // transaction (le défaut de la requête prime).
    if is_bank_ledger(body.counterparty_account_id, bank_ledger_account_id) {
        return Err(counterparty_is_bank_ledger_error("counterpartyAccountId"));
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
/// step 4bis compte de banque actif → 412 BANK_ACCOUNT_NOT_CONFIGURED ;
/// garde « contrepartie = compte de banque » → 400 VALIDATION_ERROR (même refus
///   que `post_manual`, Story 15-6d — cf. [`is_bank_ledger`]) ;
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
    // Story 28-4a (#642, R4) — compte clos : `409 BANK_ACCOUNT_CLOSED`.
    if bank_account.is_closed() {
        return Err(AppError::BankAccountClosed {
            bank_account_id: bank_account.id,
        });
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
    //
    // Story 15-6d (AC3) : refus et comparaison partagés avec `post_manual`
    // (même place : après la « Step 4bis », avant la « Step 5 »).
    for (idx, s) in body.splits.iter().enumerate() {
        if is_bank_ledger(s.counterparty_account_id, bank_ledger_account_id) {
            return Err(counterparty_is_bank_ledger_error(&format!(
                "splits[{idx}].counterpartyAccountId"
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

/// Conclut une tentative de `post_manual`, de `post_split` (revue P1 de la
/// Story 15-5e2, B-3 : le `match` était recopié dans les deux) ou de
/// `post_unreject_transaction` (Story 28-5a, C-28-5-18 — d'où la signature
/// générique) : `commit` si la fermeture de `with_account_lock` a réussi,
/// sinon `rollback` et conversion de la [`ReconciliationError`] en
/// [`AppError`]. `flow` nomme la route dans le 500 défensif des variants
/// `Rule*`. Pour un refus de l'annulation de rejet, la fermeture n'a rien
/// écrit : le `commit` vaut un `rollback`.
///
/// ⛔ Un 1213 sort de la branche `Db` en `AppError::Database(DbError::Sqlx(_))`
/// **intact** : c'est la forme que reconnaît le prédicat de
/// [`crate::retry::retry_app_on_deadlock`]. Le `match` est exhaustif (F1''''
/// Pass 6 Opus) : le compilateur force la complétude à tout variant ajouté.
///
/// P-H4 Pass 1 (8-5a-base) : `rollback().await` explicite plutôt que `drop` ;
/// son échec éventuel est ignoré, l'erreur principale étant plus signifiante.
async fn conclude_locked_attempt<T>(
    tx_outer: sqlx::Transaction<'_, sqlx::MySql>,
    lock_result: Result<T, ReconciliationError>,
    flow: &'static str,
) -> Result<T, AppError> {
    match lock_result {
        Ok(value) => {
            tx_outer
                .commit()
                .await
                .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
            Ok(value)
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
        // Story 8-5a-bis — ni `post_manual`, ni `post_split`, ni
        // `post_unreject_transaction` (Story 28-5a) n'ouvrent de savepoint :
        // branche exhaustive, inatteignable en pratique.
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
        // Story 8-5b — branche exhaustive (`post_manual`, `post_split` et
        // `post_unreject_transaction` ne touchent pas aux règles).
        // Inatteignable en pratique.
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
    /// `invoice_settlement`, `supplier_settlement` (Story 30-1a) ou `entry` ;
    /// `None` si la transaction n'est pas rapprochée.
    pub kind: Option<&'static str>,
    /// La facture **client** réglée — nulle pour un règlement fournisseur.
    pub invoice_id: Option<i64>,
    pub invoice_number: Option<String>,
    /// La facture **fournisseur** réglée, pour `supplier_settlement` seulement
    /// (Story 30-1a, AC-a5 : champs ajoutés, contrat additif).
    pub supplier_invoice_id: Option<i64>,
    /// Le numéro (celui du fournisseur, libre et nullable) de cette facture.
    pub supplier_invoice_number: Option<String>,
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
    use kesh_db::repositories::reconciliation_cancel::ReconciliationKind;
    let (invoice_id, supplier_invoice_id) = match view.kind {
        Some(ReconciliationKind::InvoiceSettlement { invoice_id, .. }) => (Some(invoice_id), None),
        Some(ReconciliationKind::SupplierSettlement {
            supplier_invoice_id,
        }) => (None, Some(supplier_invoice_id)),
        Some(ReconciliationKind::Entry) | None => (None, None),
    };
    let hit = view.cancel_blocker;
    Ok(Json(ReconciliationTransactionResponse {
        transaction: view.bank_transaction.into(),
        kind: view.kind.map(ReconciliationKind::code),
        invoice_id,
        invoice_number: view.invoice_number,
        supplier_invoice_id,
        supplier_invoice_number: view.supplier_invoice_number,
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
    /// La facture fournisseur dont le paiement a été retiré, pour un
    /// règlement fournisseur (Story 30-1a, AC-a5 : champ ajouté).
    pub supplier_invoice_id: Option<i64>,
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
    let bank_account_id =
        bank_account_of_transaction(pool, company_id, bank_transaction_id).await?;

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
                supplier_invoice_id: done.supplier_invoice_id,
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

/// Le compte bancaire d'une transaction, pour nommer le verrou — lu hors
/// verrou, scopé par société : il ne change jamais pour une transaction.
/// Absente (ou d'une autre société) → `404 NOT_FOUND`. Partagée par
/// `…/cancel` et `…/unreject` (Story 28-5a, C-28-5-18).
///
/// Lecture non verrouillante en autocommit : elle ne demande aucun verrou et
/// ne peut être victime d'aucun interblocage.
async fn bank_account_of_transaction(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_transaction_id: i64,
) -> Result<i64, AppError> {
    sqlx::query_scalar(
        "SELECT bank_account_id FROM bank_transactions WHERE id = ? AND company_id = ?",
    )
    .bind(bank_transaction_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(DbError::Sqlx(e)))?
    .ok_or(AppError::Database(DbError::NotFound))
}

// ─────────────────────────────────────────────────────────────────────────────
// Story 28-5 (#526) — annuler un rejet
// ─────────────────────────────────────────────────────────────────────────────

/// Ce que décide, pour une annulation de rejet, la ligne lue sous verrou
/// ([`classify_unreject`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnrejectOutcome {
    /// Rejetée et non rapprochée : le rejet est annulé (`204`).
    Unrejected,
    /// Introuvable — d'une autre société, ou supprimée avec son import pendant
    /// l'attente du verrou : `404 NOT_FOUND`.
    NotFound,
    /// Non rejetée (jamais, ou plus), ou rapprochée :
    /// `409 RECONCILIATION_TRANSACTION_NOT_REJECTED`.
    NotRejected,
}

/// Classe une annulation de rejet à partir de la ligne lue **sous verrou**,
/// par une lecture verrouillante, ou de son absence — Story 28-5a, AC3
/// (#526). Fonction **pure** ; l'ordre des bras est la priorité :
///
/// | ligne lue | issue |
/// |---|---|
/// | absente | `NotFound` |
/// | rapprochée au sens d'[`is_reconciled`], marquée ou non | `NotRejected` |
/// | non rapprochée, `auto_match_rejected_at` nul | `NotRejected` |
/// | sinon — rejetée | `Unrejected` |
///
/// ⛔ **Une seule définition de « rapprochée »** : la fonction appelle
/// [`is_reconciled`] (Story 27-1) et n'écrit aucun second test de statut. Une
/// transaction rapprochée **et** marquée — état qu'aucun chemin réel ne
/// produit — est refusée, marqueur intact.
///
/// [`is_reconciled`]: kesh_db::repositories::bank_imports::is_reconciled
fn classify_unreject(row: Option<&BankTransaction>) -> UnrejectOutcome {
    use kesh_db::repositories::bank_imports::is_reconciled;
    match row {
        None => UnrejectOutcome::NotFound,
        Some(t) if is_reconciled(t.status.as_str(), t.matched_entry_id) => {
            UnrejectOutcome::NotRejected
        }
        Some(t) if t.auto_match_rejected_at.is_none() => UnrejectOutcome::NotRejected,
        Some(_) => UnrejectOutcome::Unrejected,
    }
}

/// `POST /api/v1/reconciliation/transactions/{id}/unreject` — annule le rejet
/// d'une transaction bancaire (Comptable+, clés API d'écriture admises, Story
/// 28-5a, #526). Sans corps ; `204`.
///
/// Le marqueur `auto_match_rejected_at` repasse à `NULL` (`version + 1`) : la
/// transaction **redevient proposable** (`GET /reconciliation/proposals`, dans
/// la limite de `limit`). Statut et lien inchangés ; aucune écriture, aucun
/// règlement. Trace `reconciliation.unrejected`, `details.previouslyRejectedAt`
/// = le marqueur lu sous verrou, même chaîne que le champ
/// `autoMatchRejectedAt` de la transaction exposée.
///
/// Refus, sans rien écrire ni tracer : `404 NOT_FOUND` (introuvable, autre
/// société, supprimée pendant l'attente du verrou) ;
/// `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` (non rejetée, ou rapprochée) ;
/// `409 RECONCILIATION_ACCOUNT_LOCKED` ; `409 OPTIMISTIC_LOCK_CONFLICT`,
/// défensif et inatteignable.
///
/// **Pas de rejeu sur interblocage** (`SansEcritureAuJournal`, comme
/// `post_reject`) : la transaction verrouille **une** ligne de
/// `bank_transactions` puis insère une ligne d'audit. Seul cycle connu : la
/// remise à zéro d'onboarding (`kesh_seed::reset_demo`), qui prend `audit_log`
/// par une lecture verrouillante puis supprime `bank_transactions` — l'ordre
/// inverse. Victime, l'annulation rend `500`, sans rien de partiel ; c'est le
/// cas même de `post_reject` (C-28-5-30, point (iv) d'`audit_route_registry.rs`).
pub async fn post_unreject_transaction(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    unreject_once(
        &state.pool,
        current_user.company_id,
        id,
        current_user.user_id,
        current_user.api_key_id,
    )
    .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// La transaction de l'annulation : verrou du compte, lecture verrouillante,
/// classement et — rejetée — remise à `NULL` du marqueur et trace, puis
/// conclusion par [`conclude_locked_attempt`].
async fn unreject_once(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_transaction_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
) -> Result<(), AppError> {
    let bank_account_id =
        bank_account_of_transaction(pool, company_id, bank_transaction_id).await?;
    // Story 28-4a (#642, R8) — compte clos : refus précoce, AVANT la
    // classification « pas rejetée » (sur un compte clos, c'est la clôture qui
    // répond). Lecture neuve, sur le pool, hors transaction : sans effet sur
    // les verrous ni sur la vue de la transaction qui suit.
    ensure_bank_account_open(pool, company_id, bank_account_id).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let lock_result: Result<UnrejectOutcome, ReconciliationError> = with_account_lock(
        &mut tx,
        company_id,
        bank_account_id,
        LOCK_TIMEOUT_SECS,
        async move |tx_inner| {
            // Lecture COURANTE et verrouillante : une `SELECT` ordinaire lirait
            // la vue cohérente ; celle-ci attend une suppression d'import en
            // cours, puis la voit (`404`, B10).
            let row = reconciliation_repo::find_by_id_for_account_for_update(
                &mut **tx_inner,
                company_id,
                bank_account_id,
                bank_transaction_id,
            )
            .await
            .map_err(ReconciliationError::Db)?;
            // Story 28-4a (#642, W30) — garde non verrouillante APRÈS le
            // verrou de la ligne (le dernier du chemin, qui n'a aucune lecture
            // ordinaire hors l'audit), avant la classification et l'écriture ;
            // règle de placement de `bank_accounts::ensure_open_in_tx`.
            bank_accounts::ensure_open_in_tx(tx_inner, company_id, bank_account_id)
                .await
                .map_err(ReconciliationError::Db)?;
            let outcome = classify_unreject(row.as_ref());
            if outcome != UnrejectOutcome::Unrejected {
                return Ok(outcome);
            }
            // `DATETIME(3)` décodé en `NaiveDateTime` : `.and_utc()` comme
            // `reject_batch` et le champ `autoMatchRejectedAt` (même chaîne).
            let previously_rejected_at = row
                .and_then(|t| t.auto_match_rejected_at)
                .map(|d| d.and_utc());

            let result = sqlx::query(
                "UPDATE bank_transactions \
                 SET auto_match_rejected_at = NULL, updated_at = NOW(3), \
                     version = version + 1 \
                 WHERE id = ? AND company_id = ? AND bank_account_id = ?",
            )
            .bind(bank_transaction_id)
            .bind(company_id)
            .bind(bank_account_id)
            .execute(&mut **tx_inner)
            .await
            .map_err(|e| ReconciliationError::Db(DbError::Sqlx(e)))?;
            if result.rows_affected() != 1 {
                // Inatteignable : la ligne est verrouillée par la lecture
                // ci-dessus (C-28-5-19) — jamais un `204` muet.
                return Err(ReconciliationError::Db(DbError::OptimisticLockConflict));
            }

            audit_log::insert_in_tx(
                tx_inner,
                NewAuditLogEntry::for_actor(
                    user_id,
                    actor_api_key_id,
                    "reconciliation.unrejected",
                    "bank_transaction",
                    bank_transaction_id,
                    Some(serde_json::json!({
                        "previouslyRejectedAt": previously_rejected_at,
                    })),
                ),
            )
            .await?;
            Ok(UnrejectOutcome::Unrejected)
        },
    )
    .await;

    match conclude_locked_attempt(tx, lock_result, "post_unreject").await? {
        UnrejectOutcome::Unrejected => Ok(()),
        UnrejectOutcome::NotFound => Err(AppError::Database(DbError::NotFound)),
        UnrejectOutcome::NotRejected => Err(AppError::ReconciliationTransactionNotRejected {
            bank_transaction_id,
        }),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Story 29-8 (#670) — rattacher une transaction au règlement saisi depuis la
// fiche de la facture
// ─────────────────────────────────────────────────────────────────────────────

/// La tolérance d'écart du rattachement, liée en `Decimal` — **la** constante
/// de la réconciliation ([`AMOUNT_TOLERANCE_HUNDREDTHS`]), construite en un
/// seul endroit pour les deux handlers (C-29-8-22). Aucune copie dans
/// `kesh-db`, qui la reçoit en paramètre.
fn link_amount_tolerance() -> Decimal {
    Decimal::new(AMOUNT_TOLERANCE_HUNDREDTHS, 2)
}

/// Le pré-vol commun au `GET` des candidats et au `POST` du rattachement
/// (C-29-8-27), sur le pool, hors transaction, **dans cet ordre** : compte
/// bancaire archivé → `AppError::BankAccountNotFound` (la réponse de
/// `post_manual`, C-29-8-20) ; clos → `409 BANK_ACCOUNT_CLOSED` ; sans compte
/// du grand livre lié → `412 BANK_ACCOUNT_NOT_CONFIGURED`. Rend le compte du
/// grand livre lié.
///
/// Les deux routes l'appellent **avant** toute lecture de statut de la
/// transaction : la liste n'offre jamais ce que le `POST` refuserait, et ne
/// tait pas la cause sous une liste vide. Un compte du grand livre **inactif**
/// n'est pas refusé, à la différence de `post_manual` : le rattachement
/// n'écrit aucune ligne.
async fn link_preflight(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_account_id: i64,
) -> Result<i64, AppError> {
    let account = bank_accounts::find_by_id_for_company(pool, company_id, bank_account_id)
        .await?
        .ok_or(AppError::BankAccountNotFound)?;
    if account.archived {
        return Err(AppError::BankAccountNotFound);
    }
    if account.is_closed() {
        return Err(AppError::BankAccountClosed { bank_account_id });
    }
    account
        .journal_account_id
        .ok_or(AppError::BankAccountNotConfigured { bank_account_id })
}

/// Un règlement rattachable, tel que l'API le rend.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkCandidateResponse {
    pub settlement_id: i64,
    pub invoice_id: i64,
    pub invoice_number: Option<String>,
    pub contact_name: String,
    pub settled_on: NaiveDate,
    /// `M`, à deux décimales.
    pub amount: String,
    /// `transaction − M`, à deux décimales.
    pub gap: String,
    pub days_apart: i64,
    pub journal_entry_id: i64,
}

/// Réponse de `GET …/link-candidates`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkCandidatesResponse {
    pub bank_transaction_id: i64,
    pub candidates: Vec<LinkCandidateResponse>,
}

/// `GET /api/v1/reconciliation/transactions/{id}/link-candidates` — les
/// règlements clients par virement déjà saisis auxquels la transaction se
/// rattache (Comptable+, Story 29-8, #670). Lecture.
///
/// Ordre des refus, **identique à celui du `POST`** : `404 NOT_FOUND`
/// (inexistante ou d'une autre société) → pré-vol ([`link_preflight`] :
/// archivé, clos, non lié) → `404 RECONCILIATION_TRANSACTION_NOT_PENDING`
/// (rapprochée). Une transaction de débit, nulle ou hors CHF rend `200` et une
/// liste vide. La règle et le tri sont ceux de
/// [`kesh_db::repositories::reconciliation_link`] ; rien n'est présélectionné.
pub async fn get_link_candidates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<LinkCandidatesResponse>, AppError> {
    let company_id = current_user.company_id;
    let bank_account_id = bank_account_of_transaction(&state.pool, company_id, id).await?;
    let bank_ledger_account_id = link_preflight(&state.pool, company_id, bank_account_id).await?;
    let t = reconciliation_repo::find_pending_by_id_for_account(
        &state.pool,
        company_id,
        bank_account_id,
        id,
    )
    .await?
    .ok_or(AppError::Database(DbError::NotFound))?;
    if kesh_db::repositories::bank_imports::is_reconciled(t.status.as_str(), t.matched_entry_id) {
        return Err(AppError::ReconciliationTransactionNotPending {
            bank_transaction_id: id,
        });
    }
    let candidates = kesh_db::repositories::reconciliation_link::find_link_candidates(
        &state.pool,
        company_id,
        &t,
        bank_ledger_account_id,
        link_amount_tolerance(),
    )
    .await?
    .into_iter()
    .map(|c| LinkCandidateResponse {
        settlement_id: c.settlement_id,
        invoice_id: c.invoice_id,
        invoice_number: c.invoice_number,
        contact_name: c.contact_name,
        settled_on: c.settled_on,
        amount: c.amount.to_string(),
        gap: c.gap.to_string(),
        days_apart: c.days_apart,
        journal_entry_id: c.journal_entry_id,
    })
    .collect();
    Ok(Json(LinkCandidatesResponse {
        bank_transaction_id: id,
        candidates,
    }))
}

/// Corps de `POST …/link`. `settlementId` absent se lit `None` et rend
/// `400 VALIDATION_ERROR`, comme une valeur `≤ 0`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkBody {
    #[serde(default)]
    pub settlement_id: Option<i64>,
}

/// Réponse de `POST …/link`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkResponse {
    pub bank_transaction_id: i64,
    pub journal_entry_id: i64,
    pub invoice_id: i64,
    pub settlement_id: i64,
}

/// `POST /api/v1/reconciliation/transactions/{id}/link` — rattache la
/// transaction au règlement `settlementId` (Comptable+, clés API d'écriture
/// admises, Story 29-8, #670). La transaction passe `reconciled`, pointant
/// l'écriture du règlement ; **aucune** écriture, aucun règlement, aucun
/// lettrage ; trace `reconciliation.linked`.
///
/// Ordre des refus, chacun sans rien écrire (le `403` du rôle et de la clé
/// `read` vient avant, au middleware) : `400 VALIDATION_ERROR` →
/// `404 NOT_FOUND` (transaction) → pré-vol ([`link_preflight`]) → `409
/// RECONCILIATION_ACCOUNT_LOCKED` (verrou nommé du compte) → sous ce verrou,
/// dans [`kesh_db::repositories::reconciliation_link::link_in_tx`] : `404
/// NOT_FOUND` (transaction disparue), `404
/// RECONCILIATION_TRANSACTION_NOT_PENDING`, `409
/// RECONCILIATION_SETTLEMENT_NOT_LINKABLE` (condition 1, avant toute lecture
/// du règlement), `404 NOT_FOUND` (règlement absent ou d'une autre société),
/// `409 BANK_ACCOUNT_CLOSED` (garde dans la transaction), `409
/// RECONCILIATION_SETTLEMENT_NOT_LINKABLE` (conditions 2 à 4), `409
/// OPTIMISTIC_LOCK_CONFLICT` (défensif). C'est l'ordre de la table de
/// `docs/api-external.md`.
///
/// ⚠️ **Rejouée sur interblocage** (opération `reconciliation::link`, comme
/// `post_manual`) bien qu'elle n'écrive rien au journal : classée
/// `Traced, SansEcritureAuJournal` au registre `audit_route_registry.rs`,
/// point (vi).
pub async fn post_link_transaction(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(body): Json<LinkBody>,
) -> Result<Json<LinkResponse>, AppError> {
    let settlement_id = match body.settlement_id {
        Some(s) if s > 0 => s,
        _ => {
            return Err(AppError::Validation(
                "settlementId doit être strictement positif".into(),
            ));
        }
    };
    let company_id = current_user.company_id;
    let bank_account_id = bank_account_of_transaction(&state.pool, company_id, id).await?;
    let bank_ledger_account_id = link_preflight(&state.pool, company_id, bank_account_id).await?;
    let req = kesh_db::repositories::reconciliation_link::LinkRequest {
        company_id,
        bank_account_id,
        bank_transaction_id: id,
        settlement_id,
        bank_ledger_account_id,
        amount_tolerance: link_amount_tolerance(),
        user_id: current_user.user_id,
        actor_api_key_id: current_user.api_key_id,
    };
    crate::retry::retry_app_on_deadlock("reconciliation::link", || link_once(&state.pool, req))
        .await
        .map(Json)
}

/// Une tentative d'un geste de rattachement (Story 29-8, Story v017-3a) :
/// transaction neuve, verrou nommé du compte, le geste `f`, conclusion par
/// [`conclude_locked_attempt`] (un refus ne rend rien de partiel : rien n'a été
/// écrit avant lui). Factorisée pour [`link_once`] et [`link_entry_once`], qui
/// ne diffèrent que par le geste appelé et la réponse (DRY, fiche v017-3a,
/// AC-6).
async fn locked_link_attempt<T, F>(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    flow: &'static str,
    f: F,
) -> Result<T, AppError>
where
    F: AsyncFnOnce(&mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<T, ReconciliationError>,
{
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let lock_result =
        with_account_lock(&mut tx, company_id, bank_account_id, LOCK_TIMEOUT_SECS, f).await;
    conclude_locked_attempt(tx, lock_result, flow).await
}

/// Le refus d'un rattachement, en erreur de l'API : `NotFound` et `NotPending`
/// communs aux deux gestes, `NotLinkable` propre à chacun (`not_linkable`).
fn link_refusal_error(
    refusal: kesh_db::repositories::reconciliation_link::LinkRefusal,
    bank_transaction_id: i64,
    not_linkable: AppError,
) -> AppError {
    use kesh_db::repositories::reconciliation_link::LinkRefusal;
    match refusal {
        LinkRefusal::NotFound => AppError::Database(DbError::NotFound),
        LinkRefusal::NotPending => AppError::ReconciliationTransactionNotPending {
            bank_transaction_id,
        },
        LinkRefusal::NotLinkable => not_linkable,
    }
}

/// Une tentative du rattachement à un règlement, par [`locked_link_attempt`]
/// et [`kesh_db::repositories::reconciliation_link::link_in_tx`].
async fn link_once(
    pool: &sqlx::MySqlPool,
    req: kesh_db::repositories::reconciliation_link::LinkRequest,
) -> Result<LinkResponse, AppError> {
    use kesh_db::repositories::reconciliation_link::link_in_tx;
    let outcome = locked_link_attempt(
        pool,
        req.company_id,
        req.bank_account_id,
        "post_link",
        async move |tx_inner| {
            link_in_tx(tx_inner, req)
                .await
                .map_err(ReconciliationError::Db)
        },
    )
    .await?;
    match outcome {
        Ok(linked) => Ok(LinkResponse {
            bank_transaction_id: req.bank_transaction_id,
            journal_entry_id: linked.journal_entry_id,
            invoice_id: linked.invoice_id,
            settlement_id: linked.settlement_id,
        }),
        Err(refusal) => Err(link_refusal_error(
            refusal,
            req.bank_transaction_id,
            AppError::ReconciliationSettlementNotLinkable {
                bank_transaction_id: req.bank_transaction_id,
                settlement_id: req.settlement_id,
            },
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Story v017-3a (#671) — rattacher une transaction à une écriture déjà passée
// ─────────────────────────────────────────────────────────────────────────────

/// Une écriture rattachable, telle que l'API la rend.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryCandidateResponse {
    pub journal_entry_id: i64,
    pub entry_number: i64,
    pub fiscal_year_id: i64,
    pub entry_date: NaiveDate,
    /// Le code du journal (`Achats`, `Ventes`, `Banque`, `Caisse`, `OD`).
    pub journal: &'static str,
    pub description: String,
    /// `M`, signé, à deux décimales (égal au montant de la transaction).
    pub amount: String,
    pub days_apart: i64,
    /// Le code du premier motif qui, **au moment de la lecture**, empêcherait
    /// d'annuler ce rattachement — ceux de `SettlementCancelBlocker::code()`
    /// —, ou `null`. Un instantané (angle mort L9 de la fiche) : calculé par
    /// **la** fonction de blocage de l'annulation
    /// (`reconciliation_cancel::entry_cancel_blocker`), sans seconde
    /// précédence. Les trois codes fournisseur de tête ne sortent pas pour une
    /// candidate (le classement exige une facture `paid`, qu'aucun lot
    /// `generated` ne contient).
    pub cancel_blocked_by: Option<&'static str>,
    /// Le numéro du compte archivé (rang 4), comme
    /// `GET /reconciliation/transactions/{id}`.
    pub cancel_blocked_label: Option<String>,
}

/// Réponse de `GET …/entry-candidates`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryCandidatesResponse {
    pub bank_transaction_id: i64,
    pub candidates: Vec<EntryCandidateResponse>,
}

/// `GET /api/v1/reconciliation/transactions/{id}/entry-candidates` — les
/// écritures déjà passées auxquelles la transaction se rattache (Comptable+,
/// Story v017-3a, #671). Lecture.
///
/// Ordre des refus, **identique à celui du `POST`** et à la 29-8 : `404
/// NOT_FOUND` (inexistante ou d'une autre société) → pré-vol
/// ([`link_preflight`], réutilisé : archivé, clos, non lié) → `404
/// RECONCILIATION_TRANSACTION_NOT_PENDING`. Une transaction nulle ou hors CHF
/// rend `200` et une liste vide. La règle et le tri sont ceux de
/// [`kesh_db::repositories::reconciliation_link::find_entry_candidates`] ;
/// rien n'est présélectionné, rien n'est écrit.
///
/// La liste et les motifs d'annulation sont lus dans **une** transaction de
/// lecture (un instantané), sans verrou.
///
/// ⚠️ **Coût** : le motif coûte, par candidate, de huit à douze requêtes (le
/// classement, l'exercice, le lettrage figé, `reversal_blockers`, l'exercice
/// du jour) — de l'ordre de 500 pour 50 candidates. Mesuré au développement
/// (fiche v017-3a, AC-5, Dev Agent Record) ; l'exercice du jour, identique
/// pour toutes, serait factorisable.
pub async fn get_entry_candidates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<EntryCandidatesResponse>, AppError> {
    let company_id = current_user.company_id;
    let bank_account_id = bank_account_of_transaction(&state.pool, company_id, id).await?;
    let bank_ledger_account_id = link_preflight(&state.pool, company_id, bank_account_id).await?;
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let t = reconciliation_repo::find_pending_by_id_for_account(
        &mut *tx,
        company_id,
        bank_account_id,
        id,
    )
    .await?
    .ok_or(AppError::Database(DbError::NotFound))?;
    if kesh_db::repositories::bank_imports::is_reconciled(t.status.as_str(), t.matched_entry_id) {
        return Err(AppError::ReconciliationTransactionNotPending {
            bank_transaction_id: id,
        });
    }
    let found = kesh_db::repositories::reconciliation_link::find_entry_candidates(
        &mut *tx,
        company_id,
        &t,
        bank_ledger_account_id,
    )
    .await?;
    let mut candidates = Vec::with_capacity(found.len());
    for c in found {
        let hit = kesh_db::repositories::reconciliation_cancel::entry_cancel_blocker(
            &mut tx,
            company_id,
            c.journal_entry_id,
            None,
        )
        .await?;
        candidates.push(EntryCandidateResponse {
            journal_entry_id: c.journal_entry_id,
            entry_number: c.entry_number,
            fiscal_year_id: c.fiscal_year_id,
            entry_date: c.entry_date,
            journal: c.journal.as_str(),
            description: c.description,
            amount: c.amount.to_string(),
            days_apart: c.days_apart,
            cancel_blocked_by: hit.as_ref().map(|h| h.0.code()),
            cancel_blocked_label: hit.and_then(|h| h.2),
        });
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    Ok(Json(EntryCandidatesResponse {
        bank_transaction_id: id,
        candidates,
    }))
}

/// Corps de `POST …/link-entry`. `journalEntryId` absent se lit `None` et rend
/// `400 VALIDATION_ERROR`, comme une valeur `≤ 0`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkEntryBody {
    #[serde(default)]
    pub journal_entry_id: Option<i64>,
}

/// Réponse de `POST …/link-entry`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkEntryResponse {
    pub bank_transaction_id: i64,
    pub journal_entry_id: i64,
}

/// `POST /api/v1/reconciliation/transactions/{id}/link-entry` — rattache la
/// transaction à l'écriture `journalEntryId` (Comptable+, clés API d'écriture
/// admises, Story v017-3a, #671). La transaction passe `reconciled`, pointant
/// l'écriture ; **aucune** écriture, aucun règlement, aucun lettrage ; trace
/// `reconciliation.entry_linked`.
///
/// Ordre des refus, chacun sans rien écrire (le `403` du rôle et de la clé
/// `read` vient avant, au middleware) : `400 VALIDATION_ERROR` →
/// `404 NOT_FOUND` (transaction) → pré-vol ([`link_preflight`]) → `409
/// RECONCILIATION_ACCOUNT_LOCKED` → sous ce verrou, dans
/// [`kesh_db::repositories::reconciliation_link::link_entry_in_tx`] : `404
/// NOT_FOUND` (transaction disparue), `404
/// RECONCILIATION_TRANSACTION_NOT_PENDING`, `409
/// RECONCILIATION_ENTRY_NOT_LINKABLE` (condition 1, avant toute lecture de
/// l'écriture), `404 NOT_FOUND` (écriture absente ou d'une autre société),
/// `409 BANK_ACCOUNT_CLOSED`, `409 RECONCILIATION_ENTRY_NOT_LINKABLE`
/// (conditions 2 à 6), `409 OPTIMISTIC_LOCK_CONFLICT` (défensif).
///
/// ⚠️ **Rejouée sur interblocage** (opération `reconciliation::link_entry`)
/// bien qu'elle n'écrive rien au journal : classée `Traced,
/// SansEcritureAuJournal` au registre `audit_route_registry.rs`, point (vi).
pub async fn post_link_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(body): Json<LinkEntryBody>,
) -> Result<Json<LinkEntryResponse>, AppError> {
    let journal_entry_id = match body.journal_entry_id {
        Some(e) if e > 0 => e,
        _ => {
            return Err(AppError::Validation(
                "journalEntryId doit être strictement positif".into(),
            ));
        }
    };
    let company_id = current_user.company_id;
    let bank_account_id = bank_account_of_transaction(&state.pool, company_id, id).await?;
    let bank_ledger_account_id = link_preflight(&state.pool, company_id, bank_account_id).await?;
    let req = kesh_db::repositories::reconciliation_link::LinkEntryRequest {
        company_id,
        bank_account_id,
        bank_transaction_id: id,
        journal_entry_id,
        bank_ledger_account_id,
        user_id: current_user.user_id,
        actor_api_key_id: current_user.api_key_id,
    };
    crate::retry::retry_app_on_deadlock("reconciliation::link_entry", || {
        link_entry_once(&state.pool, req)
    })
    .await
    .map(Json)
}

/// Une tentative du rattachement à une écriture, par [`locked_link_attempt`]
/// et [`kesh_db::repositories::reconciliation_link::link_entry_in_tx`].
async fn link_entry_once(
    pool: &sqlx::MySqlPool,
    req: kesh_db::repositories::reconciliation_link::LinkEntryRequest,
) -> Result<LinkEntryResponse, AppError> {
    use kesh_db::repositories::reconciliation_link::link_entry_in_tx;
    let outcome = locked_link_attempt(
        pool,
        req.company_id,
        req.bank_account_id,
        "post_link_entry",
        async move |tx_inner| {
            link_entry_in_tx(tx_inner, req)
                .await
                .map_err(ReconciliationError::Db)
        },
    )
    .await?;
    match outcome {
        Ok(linked) => Ok(LinkEntryResponse {
            bank_transaction_id: req.bank_transaction_id,
            journal_entry_id: linked.journal_entry_id,
        }),
        Err(refusal) => Err(link_refusal_error(
            refusal,
            req.bank_transaction_id,
            AppError::ReconciliationEntryNotLinkable {
                bank_transaction_id: req.bank_transaction_id,
                journal_entry_id: req.journal_entry_id,
            },
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Story 28-4a (#642) — compte bancaire clos : pré-vols
// ─────────────────────────────────────────────────────────────────────────────

/// Le `failed[]` d'un lot (acceptation R1, rejet R2) visant un compte **clos** :
/// une [`FailedProposal`] par transaction, code `BANK_ACCOUNT_CLOSED`,
/// `details.bankAccountId` — `200`, jamais d'`AppError` globale (§ *Pattern
/// batch* du `CLAUDE.md`). Les acceptations n'ont aucune garde propre
/// (`accept_one_*`, C-28-4-14) : sur un compte clos que la fenêtre de la
/// v0.13.3 (#651, fermée en v0.13.4) aurait laissé avec une transaction en
/// attente, c'est ce pré-vol, lecture fraîche en autocommit, qui les arrête.
fn closed_account_failed(
    bank_transaction_ids: &[i64],
    bank_account_id: i64,
) -> Vec<FailedProposal> {
    bank_transaction_ids
        .iter()
        .map(|&bank_transaction_id| FailedProposal {
            bank_transaction_id,
            error_code: "BANK_ACCOUNT_CLOSED".to_string(),
            details: Some(serde_json::json!({ "bankAccountId": bank_account_id })),
        })
        .collect()
}

/// Pré-vol R8 de l'annulation de rejet : `409 BANK_ACCOUNT_CLOSED` si le compte
/// de la transaction est clos. Lecture non verrouillante sur le pool, hors
/// transaction. Un compte absent (inatteignable : la transaction vient d'être
/// lue) laisse la suite décider.
async fn ensure_bank_account_open(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    bank_account_id: i64,
) -> Result<(), AppError> {
    match bank_accounts::find_by_id_for_company(pool, company_id, bank_account_id).await? {
        Some(account) if account.is_closed() => {
            Err(AppError::BankAccountClosed { bank_account_id })
        }
        _ => Ok(()),
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

    /// Story 15-12b (#543, AC 11 ; revue de code P1, A-2) — la branche
    /// `projectId` du constructeur `later_fiscal_year_closed_failed_proposal`,
    /// par le bras du mapper : **avec** un projet, la clé est portée ; **sans**,
    /// elle est omise. Les trois tests du lot (`reconciliation_e2e.rs`,
    /// `reconciliation_rules_e2e.rs`) n'exercent que des voies sans projet :
    /// retirer l'insertion de `projectId` les laissait verts.
    #[test]
    fn later_fiscal_year_closed_carries_the_project_only_when_known() {
        let refus = || DbError::LaterFiscalYearClosed {
            fiscal_year_id: 12,
            fiscal_year_name: "Exercice 2027".to_string(),
        };

        let avec = project_error_to_failed_proposal(42, Some(7), refus());
        assert_eq!(avec.bank_transaction_id, 42);
        assert_eq!(avec.error_code, "LATER_FISCAL_YEAR_CLOSED");
        assert_eq!(
            avec.details.expect("details attendus"),
            serde_json::json!({
                "fiscalYearId": 12,
                "fiscalYearName": "Exercice 2027",
                "projectId": 7
            })
        );

        let sans = project_error_to_failed_proposal(9, None, refus());
        let details = sans.details.expect("details attendus");
        assert!(
            details.get("projectId").is_none(),
            "la clé doit être ABSENTE, pas nulle : {details}"
        );
        assert_eq!(
            details,
            serde_json::json!({ "fiscalYearId": 12, "fiscalYearName": "Exercice 2027" })
        );
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

    /// Story 15-1a2-i (AC15 d) — une erreur de la synchronisation du lettrage
    /// est TOUJOURS per-proposal : défaut structurel → `INTERNAL_ERROR` sans
    /// détails, `LETTERING_CONCURRENT_CHANGE` compris (C-15-1a2-14 : il n'entre
    /// pas dans `failed[]`) ; toute autre erreur → `DATABASE_ERROR` avec son
    /// message.
    #[test]
    fn lettering_errors_map_to_failed_proposals() {
        let structurels = [
            DbError::Invariant("x".into()),
            DbError::LetteringConcurrentChange,
            DbError::LetteringLineAlreadyLettered { code: "A".into() },
            DbError::LetteringUnbalanced {
                difference: Decimal::ONE,
            },
            DbError::LetteringAccountNotLetterable,
            DbError::LetteringAccountsDiffer,
            DbError::LetteringTooFewLines,
            DbError::LetteringTooManyLines { max: 200 },
            DbError::LetteringAllLinesInClosedPeriods,
            DbError::LetteringIsDocument,
            DbError::LetteringLineOwnedByDocument {
                blocker: kesh_db::errors::ReversalBlocker::OwnedByInvoice,
                document_id: Some(1),
                document_label: None,
            },
            DbError::NotFound,
        ];
        for err in structurels {
            let nom = format!("{err:?}");
            let f = lettering_error_to_failed_proposal(7, err);
            assert_eq!(f.bank_transaction_id, 7, "{nom}");
            assert_eq!(f.error_code, "INTERNAL_ERROR", "{nom}");
            assert!(f.details.is_none(), "{nom}");
        }
        let f = lettering_error_to_failed_proposal(
            7,
            DbError::Sqlx(sqlx::Error::Protocol("panne".into())),
        );
        assert_eq!(f.error_code, "DATABASE_ERROR");
        assert!(
            f.details
                .as_ref()
                .and_then(|d| d["message"].as_str())
                .is_some(),
            "{:?}",
            f.details
        );
    }
}

#[cfg(test)]
mod batch_classification_tests {
    //! Story 27-4, AC1 (#634) — la classification d'une transaction d'un lot.
    //! Codes écrits **en dur**, jamais relus d'une constante de production.
    use super::*;

    /// Partagée avec `unreject_classification_tests` (Story 28-5a).
    pub(super) fn tx(
        status: BankTransactionStatus,
        matched_entry_id: Option<i64>,
        rejected: bool,
    ) -> BankTransaction {
        let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        let at = day.and_hms_opt(12, 0, 0).unwrap();
        BankTransaction {
            id: 7,
            company_id: 1,
            import_id: 1,
            bank_account_id: 1,
            booking_date: day,
            value_date: Some(day),
            amount: Decimal::new(1000, 2),
            currency: "CHF".into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            status,
            matched_entry_id,
            auto_match_rejected_at: rejected.then_some(at),
            version: 1,
            created_at: at,
            updated_at: at,
        }
    }

    fn code(row: Option<&BankTransaction>) -> Option<(String, Option<serde_json::Value>)> {
        classify_batch_transaction(7, row).map(|f| {
            assert_eq!(f.bank_transaction_id, 7);
            (f.error_code, f.details)
        })
    }

    #[test]
    fn absent_row_is_not_found_without_details() {
        assert_eq!(
            code(None),
            Some(("BANK_TRANSACTION_NOT_FOUND".to_string(), None))
        );
    }

    #[test]
    fn reconciled_row_is_already_reconciled() {
        let t = tx(BankTransactionStatus::Reconciled, Some(3), false);
        assert_eq!(
            code(Some(&t)),
            Some(("RECONCILIATION_ALREADY_RECONCILED".to_string(), None))
        );
    }

    #[test]
    fn rejected_pending_row_is_already_rejected() {
        let t = tx(BankTransactionStatus::Pending, None, true);
        assert_eq!(
            code(Some(&t)),
            Some(("RECONCILIATION_ALREADY_REJECTED".to_string(), None))
        );
    }

    #[test]
    fn pending_unrejected_row_is_eligible() {
        let t = tx(BankTransactionStatus::Pending, None, false);
        assert_eq!(code(Some(&t)), None);
    }

    /// Priorité : rapprochée **et** rejetée → rapprochée (tue M5).
    #[test]
    fn reconciled_and_rejected_row_is_already_reconciled() {
        let t = tx(BankTransactionStatus::Reconciled, Some(3), true);
        assert_eq!(
            code(Some(&t)),
            Some(("RECONCILIATION_ALREADY_RECONCILED".to_string(), None))
        );
    }

    /// `pending` avec `matched_entry_id` : rapprochée au sens
    /// d'`is_reconciled` (tue M16 — classification sur `status` seul). État
    /// qu'aucun chemin vivant ne produit, que la suppression d'import refuse
    /// aussi.
    #[test]
    fn pending_row_with_matched_entry_is_already_reconciled() {
        let t = tx(BankTransactionStatus::Pending, Some(3), false);
        assert_eq!(
            code(Some(&t)),
            Some(("RECONCILIATION_ALREADY_RECONCILED".to_string(), None))
        );
    }

    /// `reconciled` sans lien : rapprochée aussi (`status <> 'pending'`).
    #[test]
    fn reconciled_row_without_link_is_already_reconciled() {
        let t = tx(BankTransactionStatus::Reconciled, None, false);
        assert_eq!(
            code(Some(&t)),
            Some(("RECONCILIATION_ALREADY_RECONCILED".to_string(), None))
        );
    }

    /// AC5 — la borne de taille de lot : 500 passent, 501 sont refusés.
    #[test]
    fn batch_size_cap_is_500() {
        assert!(check_batch_size(500).is_ok());
        assert!(matches!(
            check_batch_size(501),
            Err(AppError::Validation(_))
        ));
    }

    /// AC2/AC4 — la fusion suit l'ordre donné, quelle que soit la source.
    #[test]
    fn merge_follows_the_given_order() {
        let f = |id: i64| FailedProposal {
            bank_transaction_id: id,
            // `String::from` : hors du décompte `error_code: "…"` de l'AC7.
            error_code: String::from("TEST"),
            details: None,
        };
        let mut pre = HashMap::new();
        pre.insert(9, f(9));
        pre.insert(1, f(1));
        let merged = merge_failed_in_order(&[9, 5, 1, 3], pre, vec![f(5)]);
        let ids: Vec<i64> = merged.iter().map(|x| x.bank_transaction_id).collect();
        assert_eq!(ids, vec![9, 5, 1]);
    }
}

#[cfg(test)]
mod unreject_classification_tests {
    //! Story 28-5a (#526), B8 — le classement d'une annulation de rejet.
    use super::batch_classification_tests::tx;
    use super::*;

    #[test]
    fn absent_row_is_not_found() {
        assert_eq!(classify_unreject(None), UnrejectOutcome::NotFound);
    }

    #[test]
    fn reconciled_row_is_not_rejected_even_if_marked() {
        let t = tx(BankTransactionStatus::Reconciled, Some(3), true);
        assert_eq!(classify_unreject(Some(&t)), UnrejectOutcome::NotRejected);
    }

    #[test]
    fn unmarked_row_is_not_rejected() {
        let t = tx(BankTransactionStatus::Pending, None, false);
        assert_eq!(classify_unreject(Some(&t)), UnrejectOutcome::NotRejected);
    }

    /// `pending` **avec** lien, marquée : rapprochée au sens d'`is_reconciled`
    /// (tue M18 — bras « rapprochée » sur le statut seul). État qu'aucun chemin
    /// vivant ne produit (revue de code P1, E-1).
    #[test]
    fn pending_row_with_matched_entry_is_not_rejected_even_if_marked() {
        let t = tx(BankTransactionStatus::Pending, Some(3), true);
        assert_eq!(classify_unreject(Some(&t)), UnrejectOutcome::NotRejected);
    }

    /// `reconciled` **sans** lien, marquée : rapprochée aussi
    /// (`status <> 'pending'` ; tue M19 — bras « rapprochée » sur le lien seul).
    #[test]
    fn reconciled_row_without_link_is_not_rejected_even_if_marked() {
        let t = tx(BankTransactionStatus::Reconciled, None, true);
        assert_eq!(classify_unreject(Some(&t)), UnrejectOutcome::NotRejected);
    }

    #[test]
    fn rejected_row_is_unrejected() {
        let t = tx(BankTransactionStatus::Pending, None, true);
        assert_eq!(classify_unreject(Some(&t)), UnrejectOutcome::Unrejected);
    }
}

#[cfg(test)]
mod non_credit_settlement_tests {
    //! Story 28-6 (#482, AC3) — la garde de sens d'`accept_one_invoice`.
    //! Codes et raisons écrits en dur, jamais relus de la production.
    use super::*;
    use rust_decimal_macros::dec;

    /// Le refus attendu, champ par champ.
    fn assert_refused(amount: Decimal, reason: &str) {
        let refusal = refuse_non_credit_settlement(42, amount)
            .unwrap_or_else(|| panic!("{amount} devait être refusé"));
        assert_eq!(
            refusal.bank_transaction_id, 42,
            "identifiant reporté tel quel"
        );
        assert_eq!(refusal.error_code, "RECONCILIATION_TRANSACTION_NOT_CREDIT");
        assert_eq!(
            refusal.details,
            Some(serde_json::json!({ "reason": reason, "transactionAmount": amount })),
        );
    }

    #[test]
    fn a_debit_is_refused() {
        assert_refused(dec!(-100.00), "debit_transaction");
    }

    #[test]
    fn the_smallest_debit_is_refused() {
        assert_refused(dec!(-0.01), "debit_transaction");
    }

    #[test]
    fn a_zero_amount_is_refused() {
        assert_refused(dec!(0.00), "zero_amount_transaction");
    }

    /// Défense en profondeur (C-28-6-12) : la base ne rend jamais `-0.00`,
    /// mais la fonction ne doit pas le classer en débit.
    #[test]
    fn a_negative_zero_is_a_zero_amount_not_a_debit() {
        // ⚠️ `dec!(-0.00)` ne donne PAS un zéro négatif : le macro le
        // normalise, et la précondition ci-dessous l'a montré au premier run
        // (C-28-6-15). Le bit de signe se pose donc explicitement.
        let mut negative_zero = Decimal::new(0, 2);
        negative_zero.set_sign_negative(true);
        // Précondition : sans elle, ce cas serait un doublon de 0.00 et
        // passerait à vide.
        assert!(negative_zero.is_sign_negative(), "montage : zéro négatif");
        assert!(negative_zero.is_zero());
        assert_refused(negative_zero, "zero_amount_transaction");
    }

    #[test]
    fn the_smallest_credit_is_not_refused() {
        assert!(refuse_non_credit_settlement(42, dec!(0.01)).is_none());
    }

    #[test]
    fn an_ordinary_credit_is_not_refused() {
        assert!(refuse_non_credit_settlement(7, dec!(100.00)).is_none());
    }

    #[test]
    fn the_transaction_id_is_carried_as_given() {
        let refusal = refuse_non_credit_settlement(987_654, dec!(-1.00)).unwrap();
        assert_eq!(refusal.bank_transaction_id, 987_654);
    }
}

/// Story 29-3 (#527) — l'ordre des candidates d'une proposition.
///
/// Les entrées et les sorties sont écrites en dur, jamais relues de la
/// production. Plusieurs entrées (règle en tête, facture sans signal) ne
/// sortent pas de `get_proposals` : elles éprouvent la règle de
/// [`order_candidates`], qui ne suppose rien de l'ordre reçu.
#[cfg(test)]
mod candidate_order_tests {
    use super::*;

    /// Facture d'identifiant `id`, sous-scores montant, référence, contact ;
    /// total = la somme pondérée de `matching.rs`.
    fn f(id: i64, amount: f64, reference: f64, contact: f64) -> ReconciliationCandidate {
        ReconciliationCandidate {
            candidate_type: CandidateType::Invoice,
            invoice_id: Some(id),
            supplier_invoice_id: None,
            contact_name: None,
            invoice_number: Some(format!("F-{id}")),
            invoice_amount: Some("100".to_string()),
            invoice_total_ttc: None,
            invoice_date: None,
            rule_id: None,
            rule_label: None,
            rule_match_type: None,
            counterparty_account_id: None,
            counterparty_account_name: None,
            score: MatchScore {
                total: 0.50 * amount + 0.40 * reference + 0.10 * contact,
                amount_score: amount,
                reference_score: reference,
                contact_score: contact,
            },
        }
    }

    /// La règle d'affectation, au score conventionnel de 1,0.
    fn r() -> ReconciliationCandidate {
        ReconciliationCandidate {
            candidate_type: CandidateType::Rule,
            invoice_id: None,
            supplier_invoice_id: None,
            contact_name: None,
            invoice_number: None,
            invoice_amount: None,
            invoice_total_ttc: None,
            invoice_date: None,
            rule_id: Some(7),
            rule_label: Some("Règle".to_string()),
            rule_match_type: Some("counterparty_contains".to_string()),
            counterparty_account_id: Some(3000),
            counterparty_account_name: Some("3000 Ventes".to_string()),
            score: MatchScore {
                total: 1.0,
                amount_score: 0.0,
                reference_score: 0.0,
                contact_score: 0.0,
            },
        }
    }

    /// Identifiant lisible : `Some(id)` d'une facture, `None` de la règle.
    fn ids(candidates: &[ReconciliationCandidate]) -> Vec<Option<i64>> {
        candidates.iter().map(|c| c.invoice_id).collect()
    }

    fn ordered(mut candidates: Vec<ReconciliationCandidate>) -> Vec<Option<i64>> {
        order_candidates(&mut candidates);
        ids(&candidates)
    }

    const R: Option<i64> = None;

    #[test]
    fn cas_1_the_production_order_is_kept() {
        assert_eq!(ordered(vec![f(1, 0.0, 0.0, 1.0), r()]), vec![Some(1), R]);
    }

    #[test]
    fn cas_2_a_rule_ahead_goes_behind_an_identified_invoice() {
        assert_eq!(ordered(vec![r(), f(1, 0.0, 0.0, 1.0)]), vec![Some(1), R]);
    }

    #[test]
    fn cas_3_input_order_is_kept_within_rank_0() {
        assert_eq!(
            ordered(vec![r(), f(1, 0.0, 1.0, 0.0), f(2, 0.0, 0.5, 0.0)]),
            vec![Some(1), Some(2), R]
        );
    }

    #[test]
    fn cas_4_an_ordered_list_is_unchanged() {
        assert_eq!(
            ordered(vec![f(1, 0.0, 0.5, 1.0), f(2, 0.0, 0.5, 0.0), r()]),
            vec![Some(1), Some(2), R]
        );
    }

    #[test]
    fn cas_5_an_invoice_without_signal_goes_behind_the_rule() {
        assert_eq!(ordered(vec![f(1, 0.0, 0.0, 0.0), r()]), vec![R, Some(1)]);
    }

    #[test]
    fn cas_6_the_three_ranks() {
        assert_eq!(
            ordered(vec![r(), f(3, 0.0, 0.0, 1.0), f(4, 0.0, 0.0, 0.0)]),
            vec![Some(3), R, Some(4)]
        );
    }

    #[test]
    fn cas_7_invoices_alone_with_exact_amount_are_unchanged() {
        assert_eq!(
            ordered(vec![f(2, 1.0, 1.0, 0.0), f(1, 1.0, 0.0, 0.0)]),
            vec![Some(2), Some(1)]
        );
    }

    #[test]
    fn cas_8_equal_scores_within_rank_0_are_not_reordered() {
        assert_eq!(
            ordered(vec![f(5, 0.0, 0.0, 1.0), f(3, 0.0, 0.0, 1.0)]),
            vec![Some(5), Some(3)]
        );
    }

    #[test]
    fn cas_9_a_lone_rule_and_an_empty_list() {
        assert_eq!(ordered(vec![r()]), vec![R]);
        assert_eq!(ordered(vec![]), Vec::<Option<i64>>::new());
    }

    #[test]
    fn cas_10_candidate_rank_directly() {
        assert_eq!(candidate_rank(&f(1, 1.0, 0.0, 0.0)), 0);
        assert_eq!(candidate_rank(&f(1, 0.0, 1.0, 0.0)), 0);
        assert_eq!(candidate_rank(&f(1, 0.0, 0.5, 0.0)), 0);
        assert_eq!(candidate_rank(&f(1, 0.0, 0.0, 1.0)), 0);
        assert_eq!(candidate_rank(&f(1, 0.0, 0.0, 0.0)), 2);
        assert_eq!(candidate_rank(&r()), 1);
    }

    /// M6 : 51 éléments — le plafond de 50 factures du dépôt (`LIMIT 50`)
    /// plus la règle — pour qu'un tri instable ait de quoi se trahir.
    #[test]
    fn cas_11_stability_at_a_realistic_size() {
        let mut input = vec![r()];
        input.extend((1..=49).rev().map(|id| f(id, 0.0, 0.0, 1.0)));
        input.push(f(50, 0.0, 0.0, 0.0));

        let mut expected: Vec<Option<i64>> = (1..=49).rev().map(Some).collect();
        expected.push(R);
        expected.push(Some(50));

        assert_eq!(ordered(input), expected);
    }

    /// Une facture fournisseur d'identifiant `id` : `invoice_id` reste nul,
    /// l'identifiant lisible des tests est porté par `supplier_invoice_id`.
    fn s(id: i64, amount: f64, reference: f64, contact: f64) -> ReconciliationCandidate {
        let mut c = f(id, amount, reference, contact);
        c.candidate_type = CandidateType::SupplierInvoice;
        c.invoice_id = None;
        c.supplier_invoice_id = Some(id);
        c
    }

    /// T-b14 (Story 30-1b, AC-b5) — une facture **fournisseur** suit la
    /// priorité écrite d'une facture client : à signal, devant la règle
    /// (M-b11) ; sans signal, derrière.
    #[test]
    fn t_b14_a_supplier_invoice_with_a_signal_goes_before_the_rule() {
        let order = |input: Vec<ReconciliationCandidate>| {
            let mut v = input;
            order_candidates(&mut v);
            v.iter()
                .map(|c| c.supplier_invoice_id.or(c.invoice_id))
                .collect::<Vec<_>>()
        };
        assert_eq!(order(vec![r(), s(9, 1.0, 0.0, 0.0)]), vec![Some(9), R]);
        assert_eq!(order(vec![r(), s(9, 0.0, 0.0, 1.0)]), vec![Some(9), R]);
        assert_eq!(order(vec![s(9, 0.0, 0.0, 0.0), r()]), vec![R, Some(9)]);
        assert_eq!(candidate_rank(&s(9, 1.0, 1.0, 0.0)), 0);
        assert_eq!(candidate_rank(&s(9, 0.0, 0.0, 0.0)), 2);
    }
}

/// Story 30-1b (#686, AC-b7) — T-b21 : la garde de sens côté **fournisseur**
/// (`refuse_wrong_direction`, sens `Debit`). Codes et raisons en dur.
#[cfg(test)]
mod debit_settlement_tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn refused(amount: Decimal) -> Option<FailedProposal> {
        refuse_wrong_direction(42, amount, SettlementDirection::Debit)
    }

    fn assert_refused(amount: Decimal, reason: &str) {
        let refusal = refused(amount).unwrap_or_else(|| panic!("{amount} devait être refusé"));
        assert_eq!(refusal.bank_transaction_id, 42);
        assert_eq!(refusal.error_code, "RECONCILIATION_TRANSACTION_NOT_DEBIT");
        assert_eq!(
            refusal.details,
            Some(serde_json::json!({ "reason": reason, "transactionAmount": amount })),
        );
    }

    #[test]
    fn a_credit_is_refused() {
        assert_refused(dec!(100.00), "credit_transaction");
        assert_refused(dec!(0.01), "credit_transaction");
    }

    #[test]
    fn a_zero_amount_is_refused() {
        assert_refused(dec!(0.00), "zero_amount_transaction");
    }

    /// `-0.00` est un montant nul, pas un débit (M-b13 : `is_sign_negative()`).
    #[test]
    fn a_negative_zero_is_a_zero_amount_not_a_debit() {
        let mut negative_zero = Decimal::new(0, 2);
        negative_zero.set_sign_negative(true);
        assert!(negative_zero.is_sign_negative(), "montage : zéro négatif");
        assert_refused(negative_zero, "zero_amount_transaction");
    }

    #[test]
    fn a_debit_is_not_refused() {
        assert!(refused(dec!(-0.01)).is_none());
        assert!(refused(dec!(-1500.00)).is_none());
    }
}
