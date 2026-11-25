//! Paramétrage **Stripe** de la revue et de l'acceptation communes — Story
//! 40-7c (Epic 40, phase 1).
//!
//! Fonctions **pures**, sans I/O : le classement d'une transaction importée du
//! rapport Stripe « Itemized payout reconciliation » (Story 40-7b,
//! [`classify`]), l'extraction des identifiants `ch_…`, `py_…` et `po_…` d'un
//! libellé d'écriture ([`stripe_ids_in`]), le libellé des écritures
//! ([`entry_label`]), le jugement d'un **versement** ([`judge_payout`]) et la
//! revue Stripe complète, bâtie sur la revue commune ([`complete_review`]).
//!
//! # Ce qui revient au moteur commun, et ce qui est propre à Stripe
//!
//! - **Charge** (catégorie `charge`, montant > 0) : un encaissement
//!   ([`ProviderKind::SaleReceipt`]), décidé par la revue commune — proposition
//!   `link_entry` vers l'écriture de vente de la boutique dont le libellé porte
//!   l'identifiant de la charge (clé : `reference`, `source_id` à défaut
//!   `charge_id`, 40-7b AC-5).
//! - **Frais** (suffixe `:FEE` de `transaction_id`, quel que soit le signe, ou
//!   catégorie `fee`) : [`ProviderKind::Fee`], proposition `fee` commune.
//! - **Versement** (catégorie `payout`) : genre propre
//!   ([`KIND_PAYOUT`]), jugé ici ([`judge_payout`]) — proposition `payout`
//!   ([`StripeReviewProposal::Payout`]), à examiner, ou écart.
//! - **Remboursement** et **toute autre catégorie** (`payout_reversal`,
//!   litiges…) : genres propres, toujours **à examiner**
//!   ([`REVIEW_REFUND`], [`REVIEW_OTHER_CATEGORY`]) — jamais une règle
//!   d'affectation (FR-40-7c-5).
//!
//! Rien n'est écrit : la revue **propose** (`CLAUDE.md`, « Un appariement
//! automatique propose, il ne crée jamais »).

use std::collections::HashMap;

use kesh_db::entities::BankTransaction;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::paypal::PAYPAL_FEE_SUFFIX;
use crate::provider_review::{
    ENTRY_DESCRIPTION_MAX_CHARS, EntryKind, Placement, Proposal, ProviderKind, ProviderSpec,
    REVIEW_CONVERSION_UNPAIRED, Review, ReviewFinding, ReviewProposal, TransactionSummary,
    no_extra_kind,
};

/// Valeur de `bank_accounts.provider` d'un compte Stripe.
pub const PROVIDER: &str = "stripe";

/// Suffixe de l'identifiant de la transaction des **frais** d'une ligne du
/// rapport : `<balance_transaction_id>:FEE` — la constante de la 38-11b, que
/// l'import Stripe (40-7b) reprend.
pub const FEE_SUFFIX: &str = PAYPAL_FEE_SUFFIX;

/// Catégorie (`reporting_category`, rangée dans `provider_type`) d'une
/// **charge**.
pub const CHARGE_CATEGORY: &str = "charge";
/// Catégorie d'un **frais seul** (frais de plateforme, de versement…).
pub const FEE_CATEGORY: &str = "fee";
/// Catégorie d'un **remboursement**.
pub const REFUND_CATEGORY: &str = "refund";
/// Catégorie d'un **retour de versement**.
pub const PAYOUT_REVERSAL_CATEGORY: &str = "payout_reversal";
/// Catégorie de la transaction d'un **versement**, posée par l'import (40-7b,
/// une transaction par versement, de `− Σ net`).
pub const PAYOUT_CATEGORY: &str = "payout";

/// Genre propre : un versement.
pub const KIND_PAYOUT: &str = "stripe_payout";
/// Genre propre : un remboursement.
pub const KIND_REFUND: &str = "stripe_refund";
/// Genre propre : toute autre catégorie (retour de versement, litige…).
pub const KIND_OTHER: &str = "stripe_other";

/// À examiner : un remboursement (`details: { category }`).
pub const REVIEW_REFUND: &str = "refund";
/// À examiner : une autre catégorie (`details: { category }`).
pub const REVIEW_OTHER_CATEGORY: &str = "other_category";
/// À examiner : un versement qui porte un retour de versement.
pub const REVIEW_PAYOUT_REVERSAL: &str = "payout_reversal";
/// À examiner : un versement de montant positif ou nul (Σ net ≤ 0).
pub const REVIEW_PAYOUT_NOT_NEGATIVE: &str = "payout_not_negative";
/// À examiner : aucun compte de versement réglé, ou un compte de versement
/// inutilisable (`details: { payoutId, reason }`, `reason` ∈
/// [`PAYOUT_ACCOUNT_NOT_SET`], [`PAYOUT_ACCOUNT_NOT_CONFIGURED`], ou un motif
/// de la règle des destinations — `not_found`, `archived`, `closed`, `stripe`,
/// `paypal`, `currency`, que la route lit de
/// `kesh_db::repositories::bank_accounts::PayoutDestination::refusal`).
pub const REVIEW_PAYOUT_ACCOUNT_MISSING: &str = "payout_account_missing";
/// Motif d'un `payout_account_missing` : le compte Stripe n'a pas de compte de
/// versement réglé.
pub const PAYOUT_ACCOUNT_NOT_SET: &str = "not_set";
/// Motif d'un `payout_account_missing` : le compte de versement n'est lié à
/// aucun compte du plan.
pub const PAYOUT_ACCOUNT_NOT_CONFIGURED: &str = "not_configured";
/// Écart : la somme des lignes d'un versement ne vaut pas son montant
/// (`details: { payoutId, expected, found }`).
pub const GAP_PAYOUT_TOTAL_MISMATCH: &str = "payout_total_mismatch";
/// Écart : un versement rapproché dont l'écriture n'est pointée par aucune
/// transaction d'un autre compte bancaire (`details: { payoutId,
/// journalEntryId, payoutBankAccountId }`).
pub const GAP_PAYOUT_WITHOUT_RECEIPT: &str = "payout_without_receipt";

/// Préfixes des identifiants Stripe cherchés dans un libellé d'écriture :
/// charge (`ch_`) et paiement (`py_`), les clés des encaissements que nomme
/// l'écriture de vente d'une boutique ; **versement** (`po_`), la clé d'un
/// versement, que nomme l'écriture de son acceptation (« Versement Stripe
/// po_… <net> <devise> »). Sans `po_`, l'écriture d'un versement ne nommait
/// pas son versement : la règle commune « déjà comptabilisée »
/// ([`crate::provider_review::first_naming_entry`]) ne retenait plus que le
/// repli prudent — deux versements de même montant tous deux bloqués —, et
/// rien du tout si le libellé portait aussi un `ch_…` ou un `py_…` (correctif
/// L3 de la revue de la #1534). `txn_` (la transaction de solde) n'est la clé
/// d'aucun genre : il n'en est pas.
const ID_PREFIXES: [&str; 3] = ["ch_", "py_", "po_"];

/// Caractère d'un identifiant Stripe : `[A-Za-z0-9_]`.
fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// La catégorie Stripe d'une transaction (`provider_type`), rognée.
fn category(t: &BankTransaction) -> &str {
    t.provider_type.as_deref().unwrap_or("").trim()
}

/// Le genre d'une transaction Stripe, dans cet ordre :
///
/// - `Fee` — suffixe `:FEE` de `transaction_id`, **quel que soit le signe**
///   (frais prélevé ou rendu), ou catégorie `fee` (frais seul) ;
/// - `SaleReceipt` — catégorie `charge` et montant > 0 ;
/// - `Specific(KIND_PAYOUT)` — catégorie `payout` ;
/// - `Specific(KIND_REFUND)` — catégorie `refund` ;
/// - `Specific(KIND_OTHER)` — le reste (`payout_reversal`, une charge de
///   montant nul ou négatif, toute catégorie inconnue), jamais d'erreur.
///
/// Les catégories se comparent **exactement** (Stripe les écrit en
/// minuscules).
pub fn classify(t: &BankTransaction) -> ProviderKind {
    if t.transaction_id
        .as_deref()
        .is_some_and(|id| id.ends_with(FEE_SUFFIX))
    {
        return ProviderKind::Fee;
    }
    match category(t) {
        FEE_CATEGORY => ProviderKind::Fee,
        CHARGE_CATEGORY if t.amount > Decimal::ZERO => ProviderKind::SaleReceipt,
        PAYOUT_CATEGORY => ProviderKind::Specific(KIND_PAYOUT),
        REFUND_CATEGORY => ProviderKind::Specific(KIND_REFUND),
        _ => ProviderKind::Specific(KIND_OTHER),
    }
}

/// Les identifiants Stripe d'un libellé : les jetons `ch_…`, `py_…` et `po_…`, faits de
/// `[A-Za-z0-9_]`, en **mot entier** (bornés par le début, la fin ou un
/// caractère hors de cet ensemble), au moins un caractère après le préfixe ;
/// **casse comprise** (les identifiants Stripe y sont sensibles) ; sans
/// doublon, dans l'ordre du libellé. `xch_1` n'en est pas un (le préfixe est
/// dans un mot plus long).
pub fn stripe_ids_in(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split(|c: char| !is_id_char(c)) {
        let is_id = ID_PREFIXES
            .iter()
            .any(|p| word.len() > p.len() && word.starts_with(p));
        if is_id && !out.iter().any(|w| w == word) {
            out.push(word.to_string());
        }
    }
    out
}

/// Clé de rattachement d'une transaction Stripe : sa `reference`, posée par
/// l'import (40-7b AC-5), selon le genre :
///
/// - **encaissement** (charge) : l'objet lié, `source_id` à défaut
///   `charge_id` — `ch_…` ou `py_…`, que porte le libellé de l'écriture de
///   vente de la boutique ;
/// - **versement** : l'identifiant du versement, `po_…` — celui qu'écrit le
///   libellé de son acceptation ([`entry_label`] : « Versement Stripe po_…
///   <net> <devise> », ou « Stripe <compte> · po_… » sans libellé importé) ;
///   c'est ce qui fait **nommer** le versement par son écriture ;
/// - **frais** (`<txn_…>:FEE`) : l'objet lié de la ligne, comme la charge
///   qu'ils accompagnent. Le libellé de leur écriture, la ligne du rapport
///   (`Frais Stripe · … BRUT - txn_… (…) - po_…`), ne le porte pas, sauf si
///   la description de la vente le cite, et nomme le **versement** (`po_…`) ;
///   le garde est la **présélection**, non le nom : l'écriture des frais n'a
///   de ligne sur aucun autre compte bancaire, et la présélection « déjà
///   comptabilisée » exige une transaction d'un autre compte qui lui soit
///   rapprochée, et un mouvement sur le compte du solde égal au montant de la
///   transaction, comme pour les frais PayPal (`…/link-entry` refuse d'y
///   rattacher une réception : l'écriture des frais n'a pas de mouvement sur
///   le compte de la banque) ;
/// - remboursement, autre catégorie : l'objet lié (`re_…`…), ou aucune ; ces
///   genres n'écrivent pas.
fn match_key(t: &BankTransaction) -> Option<&str> {
    t.reference
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty())
}

/// Le genre de clé d'un identifiant Stripe : `po_` (versement), `ch_` ou
/// `py_` (encaissement : la charge et son paiement désignent la même vente) ;
/// `None` pour tout autre jeton.
fn key_kind(id: &str) -> Option<&'static str> {
    if id.starts_with("po_") {
        Some("payout")
    } else if id.starts_with("ch_") || id.starts_with("py_") {
        Some("charge")
    } else {
        None
    }
}

/// Correctif de l'angle mort de la #1534 (#1539) : un identifiant cité dans un
/// libellé ne fait de l'écriture celle d'**une autre** transaction
/// ([`crate::provider_review::unnamed_and_free`]) que s'il est du genre de la
/// clé de `t` ([`match_key`]) : un `po_…` pour un versement, un `ch_…`/`py_…`
/// pour des frais. Une écriture de versement passée à la main qui ne cite
/// qu'une charge ou un paiement ne nomme donc aucun autre versement : elle
/// tombe dans le repli prudent, sous sa borne (montant, compte du plan, côté
/// Stripe libre, rapprochée de l'autre côté). Une transaction sans clé d'un
/// genre connu : aucun identifiant ne compte (le repli prudent, jamais une
/// seconde écriture).
fn same_key_kind(t: &BankTransaction, id: &str) -> bool {
    match match_key(t).and_then(key_kind) {
        Some(kind) => key_kind(id) == Some(kind),
        None => false,
    }
}

/// Stripe n'a pas de conversion : aucune paire.
fn no_conversion(_: &BankTransaction, _: &BankTransaction) -> bool {
    false
}

/// Borne un libellé à `max` caractères en gardant sa **fin** à partir du
/// dernier `) - ` (le versement `po_…` du format de la 40-7b) : seul le milieu
/// (la description) cède, remplacé par `…`. Sans ce repère, ou si la fin seule
/// dépasse, le libellé est coupé à `max`.
pub fn bounded_label(label: &str, max: usize) -> String {
    let total = label.chars().count();
    if total <= max {
        return label.to_string();
    }
    if let Some(pos) = label.rfind(") - ") {
        let tail = &label[pos..];
        let tail_chars = tail.chars().count();
        if tail_chars < max {
            let head: String = label.chars().take(max - 1 - tail_chars).collect();
            return format!("{head}…{tail}");
        }
    }
    label.chars().take(max).collect()
}

/// Libellé d'une écriture Stripe (frais, versement) — amendement du 21.11 de la
/// fiche : le **libellé de la transaction** que l'import a écrit au format du
/// tableur comptable (40-7b, `details` : « Frais Stripe · <Encaissement|
/// catégorie> Stripe de <brut> <DEV> -<frais>=<net> <DEV> BRUT - <txn_…>
/// (<created>- <description>) - <po_…> », « Versement Stripe <po_…> <net>
/// <DEV> »), borné à [`ENTRY_DESCRIPTION_MAX_CHARS`] par [`bounded_label`].
/// Un libellé vide (transaction écrite hors de l'import) se remplace par
/// « Stripe <compte> · <identifiant> ».
pub fn entry_label(_kind: EntryKind, account: &str, t: &BankTransaction) -> String {
    let details = t.details.trim();
    if details.is_empty() {
        return bounded_label(
            &format!(
                "Stripe {account} · {}",
                t.transaction_id.as_deref().unwrap_or("")
            ),
            ENTRY_DESCRIPTION_MAX_CHARS,
        );
    }
    bounded_label(details, ENTRY_DESCRIPTION_MAX_CHARS)
}

/// Le paramétrage Stripe de la revue et de l'acceptation communes.
pub const SPEC: ProviderSpec = ProviderSpec {
    provider: PROVIDER,
    classify,
    match_key,
    extract_ids: stripe_ids_in,
    same_key_kind,
    conversion_match: no_conversion,
    // Jamais atteint : `classify` ne rend jamais `Conversion`.
    conversion_unpaired: (REVIEW_CONVERSION_UNPAIRED, Placement::ToReview),
    entry_label,
    linkable: no_extra_kind,
    ruleable: no_extra_kind,
};

// ---------------------------------------------------------------------------
// Versements
// ---------------------------------------------------------------------------

/// Les **lignes** du versement `payout` parmi `candidates` : les transactions
/// du **même compte bancaire** qui portent `end_to_end_id` = l'identifiant du
/// versement, **autres que le versement lui-même** (qui le porte aussi, 40-7b
/// AC-5), quels que soient leur état et leur date.
pub fn payout_lines<'a>(
    payout: &BankTransaction,
    candidates: &'a [BankTransaction],
) -> Vec<&'a BankTransaction> {
    let Some(payout_id) = payout.transaction_id.as_deref() else {
        return Vec::new();
    };
    candidates
        .iter()
        .filter(|l| {
            l.id != payout.id
                && l.bank_account_id == payout.bank_account_id
                && l.end_to_end_id.as_deref() == Some(payout_id)
        })
        .collect()
}

/// Le verdict sur un versement ([`judge_payout`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayoutVerdict {
    /// À examiner, sous ce motif.
    ToReview(&'static str),
    /// Écart : la somme des lignes (`found`) ne vaut pas `expected` (= −
    /// montant du versement).
    TotalMismatch {
        /// Ce que les lignes devraient faire.
        expected: Decimal,
        /// Ce qu'elles font.
        found: Decimal,
    },
    /// Proposable.
    Proposable,
}

/// Le jugement d'un versement (AC-4), **seule** définition : la revue et le
/// rejugement de l'acceptation l'appellent. Dans cet ordre :
///
/// - (a) une ligne est un retour de versement → [`REVIEW_PAYOUT_REVERSAL`] ;
///   montant ≥ 0 → [`REVIEW_PAYOUT_NOT_NEGATIVE`] ;
/// - (b) pas de compte de versement utilisable (jugé par la route,
///   `destination_refusal`, sur la règle du réglage) →
///   [`REVIEW_PAYOUT_ACCOUNT_MISSING`] ;
/// - (c) Σ montants des lignes ≠ − montant → [`PayoutVerdict::TotalMismatch`] ;
/// - (d) sinon [`PayoutVerdict::Proposable`].
pub fn judge_payout(
    payout: &BankTransaction,
    lines: &[&BankTransaction],
    destination_usable: bool,
) -> PayoutVerdict {
    if lines
        .iter()
        .any(|l| category(l) == PAYOUT_REVERSAL_CATEGORY)
    {
        return PayoutVerdict::ToReview(REVIEW_PAYOUT_REVERSAL);
    }
    if payout.amount >= Decimal::ZERO {
        return PayoutVerdict::ToReview(REVIEW_PAYOUT_NOT_NEGATIVE);
    }
    if !destination_usable {
        return PayoutVerdict::ToReview(REVIEW_PAYOUT_ACCOUNT_MISSING);
    }
    let found: Decimal = lines.iter().map(|l| l.amount).sum();
    let expected = -payout.amount;
    if found != expected {
        return PayoutVerdict::TotalMismatch { expected, found };
    }
    PayoutVerdict::Proposable
}

// ---------------------------------------------------------------------------
// Revue
// ---------------------------------------------------------------------------

/// La proposition `payout` de la revue : le versement et, pour l'affichage
/// (40-7d), le compte de versement réglé et son libellé. Le corps de
/// l'acceptation peut reprendre `payoutBankAccountId` : une destination
/// changée depuis la revue rend alors la proposition périmée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum PayoutProposal {
    /// Écrire le versement vers le compte de versement.
    Payout {
        /// Le versement.
        bank_transaction_id: i64,
        /// Le compte bancaire de versement.
        payout_bank_account_id: i64,
        /// Son libellé.
        payout_account_label: String,
    },
}

/// Une proposition de la revue Stripe : commune (`link_entry`, `fee`) ou
/// `payout`, sérialisée à plat (champ `type` compris).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum StripeReviewProposal {
    /// Une proposition commune.
    Common(Proposal),
    /// Un versement.
    Payout(PayoutProposal),
}

/// Le compte de versement d'un compte bancaire Stripe, **utilisable** :
/// réglé, non archivé, ouvert, ni Stripe ni PayPal, lié à un compte du plan de
/// la devise du solde (la règle du réglage, rejugée par la route).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoutDestination {
    /// Le compte bancaire de versement.
    pub bank_account_id: i64,
    /// Son libellé.
    pub label: String,
}

/// Ce que lit la route Stripe en plus de la revue commune.
#[derive(Debug, Clone, Copy)]
pub struct StripeReviewInput<'a> {
    /// Les transactions pending revues (celles de la revue commune).
    pub pending: &'a [BankTransaction],
    /// Les transactions des comptes Stripe qui portent l'identifiant d'un des
    /// versements pending (`end_to_end_id`), tous états et toutes dates.
    pub payout_candidates: &'a [BankTransaction],
    /// Par compte bancaire Stripe : son compte de versement utilisable, ou
    /// `None`.
    pub destinations: &'a HashMap<i64, PayoutDestination>,
    /// Par compte bancaire Stripe : le compte de versement réglé, utilisable
    /// ou non (`details.payoutBankAccountId` d'un écart de réception).
    pub configured_destinations: &'a HashMap<i64, i64>,
    /// Par compte bancaire Stripe dont le compte de versement réglé est
    /// **inutilisable** : le motif (`details.reason` d'un
    /// `payout_account_missing`). Un compte Stripe sans réglage n'y figure pas
    /// ([`PAYOUT_ACCOUNT_NOT_SET`]).
    pub destination_refusals: &'a HashMap<i64, &'static str>,
    /// Les versements de l'exercice rapprochés dont l'écriture n'est pointée
    /// par aucune transaction d'un autre compte bancaire.
    pub unreceived_payouts: &'a [BankTransaction],
    /// Les transactions pending **déjà comptabilisées** de l'autre côté :
    /// transaction → écriture, lues par la revue commune
    /// (`LoadedReview::already_posted` de la route). Un versement qui y figure
    /// va « à examiner » sous [`crate::provider_review::REVIEW_ALREADY_POSTED`],
    /// avant tout autre motif (correctif des prestataires).
    pub already_posted: &'a HashMap<i64, i64>,
}

/// Un motif sur une transaction, avec des détails.
fn finding(code: &'static str, t: &BankTransaction, details: serde_json::Value) -> ReviewFinding {
    ReviewFinding {
        details: Some(details),
        ..ReviewFinding::on(code, t)
    }
}

/// Un montant en chaîne décimale à deux décimales.
fn money(mut d: Decimal) -> String {
    d.rescale(2);
    d.to_string()
}

/// La revue Stripe : la revue commune, passée au type de proposition Stripe,
/// puis les genres propres décidés ici (versements, remboursements, autres
/// catégories) et les versements sans réception (AC-5), triée.
pub fn complete_review(
    common: Review,
    input: &StripeReviewInput<'_>,
) -> Review<StripeReviewProposal> {
    let specific = common.specific.clone();
    let mut review = common.map_proposals(StripeReviewProposal::Common);
    let by_id: HashMap<i64, &BankTransaction> = input.pending.iter().map(|t| (t.id, t)).collect();
    for (id, kind) in specific {
        let Some(t) = by_id.get(&id).copied() else {
            continue;
        };
        match kind {
            KIND_PAYOUT => decide_payout(&mut review, t, input),
            KIND_REFUND => review.push_to_review(finding(
                REVIEW_REFUND,
                t,
                serde_json::json!({ "category": category(t) }),
            )),
            _ => review.push_to_review(finding(
                REVIEW_OTHER_CATEGORY,
                t,
                serde_json::json!({ "category": category(t) }),
            )),
        }
    }
    for t in input.unreceived_payouts {
        review.push_gap(finding(
            GAP_PAYOUT_WITHOUT_RECEIPT,
            t,
            serde_json::json!({
                "payoutId": t.transaction_id,
                "journalEntryId": t.matched_entry_id,
                "payoutBankAccountId": input.configured_destinations.get(&t.bank_account_id),
            }),
        ));
    }
    review.sort();
    review
}

/// Décision d'un versement pending (voir [`judge_payout`]).
///
/// **Avant tout autre motif**, la règle commune « écrite une seule fois » : un
/// versement **déjà comptabilisé** — son écriture vivante, rapprochée de la
/// réception d'un autre compte bancaire, alors que le versement lui-même a
/// été détaché (annulation du seul côté Stripe, qui ne retire que le lien), et
/// qui le **nomme** (son libellé porte son `po_…`, [`match_key`]) ou, par
/// prudence, ne porte aucun `po_…` (un `ch_…` ou un `py_…` cité ne compte
/// pas : [`same_key_kind`] ; `first_naming_entry` de la revue commune) — va « à examiner » sous `already_posted`, l'écriture nommée
/// ([`ReviewFinding::already_posted`]) : jamais proposé, jamais sous un motif
/// qui conseillerait de l'écrire de nouveau. La route rejuge la même règle à
/// l'acceptation (`rejudge_writing`).
fn decide_payout(
    review: &mut Review<StripeReviewProposal>,
    t: &BankTransaction,
    input: &StripeReviewInput<'_>,
) {
    if let Some(entry_id) = input.already_posted.get(&t.id).copied() {
        review.push_to_review(ReviewFinding::already_posted(t, entry_id));
        return;
    }
    let lines = payout_lines(t, input.payout_candidates);
    let destination = input.destinations.get(&t.bank_account_id);
    let payout_id = t.transaction_id.clone();
    match judge_payout(t, &lines, destination.is_some()) {
        PayoutVerdict::ToReview(REVIEW_PAYOUT_ACCOUNT_MISSING) => {
            let reason = input
                .destination_refusals
                .get(&t.bank_account_id)
                .copied()
                .unwrap_or(PAYOUT_ACCOUNT_NOT_SET);
            review.push_to_review(finding(
                REVIEW_PAYOUT_ACCOUNT_MISSING,
                t,
                serde_json::json!({ "payoutId": payout_id, "reason": reason }),
            ))
        }
        PayoutVerdict::ToReview(code) => review.push_to_review(finding(
            code,
            t,
            serde_json::json!({ "payoutId": payout_id }),
        )),
        PayoutVerdict::TotalMismatch { expected, found } => review.push_gap(finding(
            GAP_PAYOUT_TOTAL_MISMATCH,
            t,
            serde_json::json!({
                "payoutId": payout_id,
                "expected": money(expected),
                "found": money(found),
            }),
        )),
        PayoutVerdict::Proposable => {
            // `destination` est `Some` : `judge_payout` l'a exigé.
            if let Some(d) = destination {
                review.push_proposal(ReviewProposal {
                    proposal: StripeReviewProposal::Payout(PayoutProposal::Payout {
                        bank_transaction_id: t.id,
                        payout_bank_account_id: d.bank_account_id,
                        payout_account_label: d.label.clone(),
                    }),
                    transaction: TransactionSummary::of(t),
                    other_transaction: None,
                    entry: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_review::{
        GAP_AMOUNT_OR_CURRENCY_MISMATCH, GAP_RECEIPT_WITHOUT_SALE, GAP_SALE_WITHOUT_RECEIPT,
        ProviderAccount, REVIEW_ALREADY_POSTED, REVIEW_AMBIGUOUS_SALE, REVIEW_FEE_ACCOUNT_MISSING,
        ReviewInput, SaleEntry, SaleMovement, build_review,
    };
    use chrono::NaiveDate;
    use kesh_db::entities::bank_transaction::BankTransactionStatus;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn tx(id: i64, amount: &str, category: &str, txid: &str) -> BankTransaction {
        let now = chrono::Utc::now().naive_utc();
        BankTransaction {
            id,
            company_id: 1,
            import_id: 1,
            bank_account_id: 10,
            booking_date: NaiveDate::from_ymd_opt(2020, 2, 3).unwrap(),
            value_date: None,
            amount: dec(amount),
            currency: "EUR".into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            transaction_id: Some(txid.into()),
            counterparty_iban: None,
            counterparty_name: None,
            provider_type: Some(category.into()),
            counterparty_email: None,
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    fn line(id: i64, amount: &str, category: &str, payout: &str) -> BankTransaction {
        let mut t = tx(id, amount, category, &format!("txn_{id}"));
        t.end_to_end_id = Some(payout.into());
        t
    }

    fn payout(id: i64, amount: &str, payout_id: &str) -> BankTransaction {
        let mut t = tx(id, amount, PAYOUT_CATEGORY, payout_id);
        t.end_to_end_id = Some(payout_id.into());
        t.reference = Some(payout_id.into());
        t
    }

    // --- Classement ---

    #[test]
    fn chaque_genre() {
        assert_eq!(
            classify(&tx(1, "100", "charge", "txn_1")),
            ProviderKind::SaleReceipt
        );
        // Frais `:FEE`, prélevé ou rendu, quelle que soit la catégorie.
        assert_eq!(
            classify(&tx(1, "-3.20", "charge", "txn_1:FEE")),
            ProviderKind::Fee
        );
        assert_eq!(
            classify(&tx(1, "0.60", "refund", "txn_1:FEE")),
            ProviderKind::Fee
        );
        // Frais seul : catégorie `fee`, de tout signe.
        assert_eq!(classify(&tx(1, "-0.50", "fee", "txn_4")), ProviderKind::Fee);
        assert_eq!(
            classify(&tx(1, "-96.80", "payout", "po_1")),
            ProviderKind::Specific(KIND_PAYOUT)
        );
        assert_eq!(
            classify(&tx(1, "-20", "refund", "txn_3")),
            ProviderKind::Specific(KIND_REFUND)
        );
        assert_eq!(
            classify(&tx(1, "-38.54", "payout_reversal", "txn_9")),
            ProviderKind::Specific(KIND_OTHER)
        );
        assert_eq!(
            classify(&tx(1, "-5", "dispute", "txn_9")),
            ProviderKind::Specific(KIND_OTHER)
        );
        // Une charge de montant nul ou négatif n'est pas un encaissement.
        assert_eq!(
            classify(&tx(1, "-5", "charge", "txn_9")),
            ProviderKind::Specific(KIND_OTHER)
        );
        assert_eq!(
            classify(&tx(1, "0", "charge", "txn_9")),
            ProviderKind::Specific(KIND_OTHER),
            "une charge de montant nul n'est pas un encaissement"
        );
        // Catégorie absente : une autre catégorie, jamais d'erreur.
        let mut t = tx(1, "5", "", "txn_9");
        t.provider_type = None;
        assert_eq!(classify(&t), ProviderKind::Specific(KIND_OTHER));
        // Casse : `Charge` n'est pas une catégorie Stripe.
        assert_eq!(
            classify(&tx(1, "5", "Charge", "txn_9")),
            ProviderKind::Specific(KIND_OTHER)
        );
    }

    // --- Identifiants ---

    #[test]
    fn identifiants_d_un_libelle() {
        assert_eq!(
            stripe_ids_in("Vente boutique TXN ch_TEST0001"),
            ["ch_TEST0001"]
        );
        assert_eq!(
            stripe_ids_in("Commande 12 (py_3Abc_9), puis ch_X1; ch_X1"),
            ["py_3Abc_9", "ch_X1"]
        );
        assert_eq!(stripe_ids_in("ch_A-ch_B"), ["ch_A", "ch_B"]);
        assert_eq!(stripe_ids_in("début:ch_Z"), ["ch_Z"]);
        // Le versement, que nomme l'écriture de son acceptation.
        assert_eq!(
            stripe_ids_in("Versement Stripe po_TEST0001 125.15 CHF"),
            ["po_TEST0001"]
        );
        assert_eq!(
            stripe_ids_in("Versement Stripe po_TEST0001 ch_TEST0002"),
            ["po_TEST0001", "ch_TEST0002"]
        );
    }

    #[test]
    fn faux_positifs_et_casse() {
        // Préfixe dans un mot plus long.
        assert!(stripe_ids_in("xch_TEST0001").is_empty());
        assert!(stripe_ids_in("1py_TEST").is_empty());
        // Préfixe seul.
        assert!(stripe_ids_in("ch_ et py_ et po_").is_empty());
        assert!(stripe_ids_in("xpo_1 PO_1").is_empty());
        // Casse comprise : `CH_` n'est pas un identifiant, et la casse est gardée.
        assert!(stripe_ids_in("CH_TEST0001").is_empty());
        assert_eq!(stripe_ids_in("ch_AbC"), ["ch_AbC"]);
        assert_ne!(stripe_ids_in("ch_abc"), ["ch_ABC"]);
        // Autres objets Stripe : ni la transaction de solde (`txn_`), clé
        // d'aucun genre, ni un remboursement ou une intention de paiement.
        assert!(stripe_ids_in("txn_1 re_3 pi_4").is_empty());
        assert!(stripe_ids_in("").is_empty());
    }

    #[test]
    fn cle_de_rattachement() {
        let mut t = tx(1, "100", "charge", "txn_1");
        assert_eq!(match_key(&t), None);
        t.reference = Some("ch_TEST0001".into());
        assert_eq!(match_key(&t), Some("ch_TEST0001"));
        t.reference = Some("  ".into());
        assert_eq!(match_key(&t), None);
        // Un versement : son identifiant, celui qu'écrit son libellé.
        assert_eq!(match_key(&payout(2, "-96.80", "po_1")), Some("po_1"));
    }

    // --- Libellés ---

    #[test]
    fn libelle_de_l_ecriture_repris_de_la_transaction() {
        let mut t = tx(1, "-3.20", "charge", "txn_1:FEE");
        t.details = "Frais Stripe · Encaissement Stripe de 100.00 CHF -3.20=96.80 CHF BRUT - txn_1 (2020-01-29 10:15:04- Vente A) - po_1".into();
        assert_eq!(entry_label(EntryKind::Fee, "Stripe", &t), t.details);
        let mut p = payout(2, "-96.80", "po_1");
        p.details = "Versement Stripe po_1 96.80 CHF".into();
        assert_eq!(
            entry_label(EntryKind::Withdrawal, "Stripe", &p),
            "Versement Stripe po_1 96.80 CHF"
        );
        p.details = String::new();
        assert_eq!(
            entry_label(EntryKind::Withdrawal, "Stripe boutique", &p),
            "Stripe Stripe boutique · po_1"
        );
    }

    #[test]
    fn libelle_borne_en_gardant_le_versement() {
        let long = format!(
            "Frais Stripe · Encaissement Stripe de 1.00 EUR -0.10=0.90 EUR BRUT - txn_1 (2020-01-01- {}) - po_FIN",
            "d".repeat(400)
        );
        let b = entry_label(EntryKind::Fee, "S", &{
            let mut t = tx(1, "-0.10", "charge", "txn_1:FEE");
            t.details = long.clone();
            t
        });
        assert_eq!(b.chars().count(), ENTRY_DESCRIPTION_MAX_CHARS);
        assert!(b.ends_with(") - po_FIN"), "{b}");
        assert!(b.starts_with("Frais Stripe · Encaissement Stripe de 1.00 EUR"));
        assert!(b.contains('…'));
        // Sans repère : coupé net.
        let sans = "x".repeat(300);
        assert_eq!(bounded_label(&sans, 200), "x".repeat(200));
        // Court : inchangé.
        assert_eq!(bounded_label("abc", 200), "abc");
        // Multioctet : compté en caractères.
        let accents = format!("{} ) - po_é", "é".repeat(250));
        let b = bounded_label(&accents, 200);
        assert_eq!(b.chars().count(), 200);
        assert!(b.ends_with(") - po_é"));
    }

    /// Correctif L3 (revue de la #1534) : avec les libellés de Stripe,
    /// l'écriture d'un versement **nomme** ce versement — sa clé `po_…` est
    /// dans son libellé —, jamais un autre versement de même montant, ni des
    /// frais ; et cela tient quand le libellé porte aussi un `ch_…` ou un
    /// `py_…`. L'écriture des frais ne nomme pas les frais. Un libellé qui ne
    /// porte qu'une charge ou un paiement tombe dans le repli prudent (#1539).
    #[test]
    fn libelle_stripe_nomme_le_seul_versement_ecrit() {
        use crate::provider_review::{entry_names, first_naming_entry};
        let mut versement = payout(1, "-96.80", "po_1");
        versement.details = "Versement Stripe po_1 96.80 CHF".into();
        let autre = payout(2, "-96.80", "po_2");
        let mut frais = tx(3, "-3.20", "charge", "txn_1:FEE");
        frais.reference = Some("ch_1".into());
        frais.details = "Frais Stripe · Encaissement Stripe de 100.00 CHF -3.20=96.80 CHF BRUT - txn_1 (2020-01-29 10:15:04- Vente A) - po_1".into();
        let du_versement = entry_label(EntryKind::Withdrawal, "S", &versement);
        let des_frais = entry_label(EntryKind::Fee, "S", &frais);
        assert!(entry_names(&SPEC, &versement, &du_versement, &[]));
        assert!(!entry_names(&SPEC, &autre, &du_versement, &[]));
        assert!(!entry_names(&SPEC, &frais, &du_versement, &[]));
        assert!(!entry_names(&SPEC, &frais, &des_frais, &[]), "{des_frais}");
        // L'écriture des frais nomme le versement (« … - po_1 ») : la
        // présélection l'écarte, elle n'a de ligne sur aucun autre compte bancaire.
        assert!(entry_names(&SPEC, &versement, &des_frais, &[]));
        // Sans libellé importé, le libellé de repli porte aussi le `po_…`.
        let mut nu = payout(4, "-96.80", "po_4");
        nu.details = String::new();
        let repli = entry_label(EntryKind::Withdrawal, "Stripe boutique", &nu);
        assert!(entry_names(&SPEC, &nu, &repli, &[]), "{repli}");
        // Deux versements de même montant : chacun la seule écriture qui le
        // nomme ; aucune pour un versement jamais écrit.
        let candidats = [
            (901, du_versement.as_str()),
            (902, "Versement Stripe po_2 96.80 CHF"),
        ];
        assert_eq!(
            first_naming_entry(&SPEC, &versement, candidats, &[]),
            Some(901)
        );
        assert_eq!(first_naming_entry(&SPEC, &autre, candidats, &[]), Some(902));
        assert_eq!(first_naming_entry(&SPEC, &nu, [candidats[0]], &[]), None);
        // #1539 : un libellé qui ne porte qu'une charge ou un paiement, sans le
        // `po_…`, ne nomme aucun versement — la charge est d'un autre genre de
        // clé : le repli prudent le retient, pour l'un et l'autre versement de
        // même montant (le côté Stripe est libre) ; non s'il porte encore un
        // versement de ce compte.
        for jeton in ["ch_TEST0002", "py_TEST0003"] {
            let seul = format!("Versement Stripe {jeton} 96.80 CHF");
            for v in [&versement, &autre] {
                assert!(!entry_names(&SPEC, v, &seul, &[]), "{seul}");
                assert_eq!(
                    first_naming_entry(&SPEC, v, [(904, seul.as_str())], &[]),
                    Some(904),
                    "{seul}"
                );
            }
            let mut porte = payout(5, "-96.80", "po_5");
            porte.matched_entry_id = Some(904);
            assert_eq!(
                first_naming_entry(&SPEC, &versement, [(904, seul.as_str())], &[porte]),
                None,
                "{seul}"
            );
            // Un `po_…` d'un autre versement, même à côté de la charge : exclu.
            let autre_po = format!("Versement Stripe po_2 {jeton} 96.80 CHF");
            assert_eq!(
                first_naming_entry(&SPEC, &versement, [(905, autre_po.as_str())], &[]),
                None,
                "{autre_po}"
            );
        }
        // Le genre de clé : `po_` d'un versement, `ch_`/`py_` des frais (leur
        // clé est la charge) ; un jeton d'aucun genre ne compte pour personne.
        assert!(same_key_kind(&versement, "po_9"));
        assert!(!same_key_kind(&versement, "ch_9"));
        assert!(!same_key_kind(&versement, "py_9"));
        assert!(same_key_kind(&frais, "ch_9") && same_key_kind(&frais, "py_9"));
        assert!(!same_key_kind(&frais, "po_9"));
        let mut sans_cle = payout(6, "-1.00", "po_6");
        sans_cle.reference = None;
        assert!(!same_key_kind(&sans_cle, "po_6"));
        // Le libellé porte aussi une charge ou un paiement : le versement reste
        // nommé, et seul lui (avant L3, ni le nom ni le repli prudent).
        for jeton in ["ch_TEST0002", "py_TEST0003"] {
            let mixte = format!("Versement Stripe po_1 96.80 CHF {jeton}");
            assert_eq!(
                first_naming_entry(&SPEC, &versement, [(903, mixte.as_str())], &[]),
                Some(903),
                "{mixte}"
            );
            assert_eq!(
                first_naming_entry(&SPEC, &autre, [(903, mixte.as_str())], &[]),
                None,
                "{mixte}"
            );
        }
    }

    /// Frontière de la fin gardée : une fin qui tient (150 < 200) est gardée
    /// entière, seul le milieu cède ; une fin d'exactement `max` caractères
    /// ne laisse aucune place au milieu et se coupe net, sans `…`.
    #[test]
    fn libelle_borne_frontiere_de_la_fin() {
        let fin_longue = format!(") - {}", "t".repeat(146));
        assert_eq!(fin_longue.chars().count(), 150);
        let b = bounded_label(&format!("{}{fin_longue}", "d".repeat(300)), 200);
        assert_eq!(b.chars().count(), 200);
        assert!(b.ends_with(&fin_longue), "{b}");
        assert!(b.contains('…'), "{b}");
        let fin_pleine = format!(") - {}", "t".repeat(196));
        assert_eq!(fin_pleine.chars().count(), 200);
        let label = format!("{}{fin_pleine}", "h".repeat(50));
        let b = bounded_label(&label, 200);
        assert_eq!(b, label.chars().take(200).collect::<String>());
        assert!(!b.contains('…'), "{b}");
    }

    // --- Versements ---

    #[test]
    fn lignes_d_un_versement_sans_lui_meme_ni_un_autre_compte() {
        let p = payout(1, "-125.65", "po_1");
        let mut ailleurs = line(4, "10", "charge", "po_1");
        ailleurs.bank_account_id = 11;
        let candidates = vec![
            p.clone(),
            line(2, "100", "charge", "po_1"),
            line(3, "-3.20", "charge", "po_1"),
            ailleurs,
            line(5, "7", "charge", "po_2"),
        ];
        let ids: Vec<i64> = payout_lines(&p, &candidates).iter().map(|l| l.id).collect();
        assert_eq!(ids, [2, 3]);
    }

    #[test]
    fn jugement_d_un_versement() {
        let p = payout(1, "-96.80", "po_1");
        let a = line(2, "100", "charge", "po_1");
        let f = line(3, "-3.20", "charge", "po_1");
        // Ordinaire.
        assert_eq!(judge_payout(&p, &[&a, &f], true), PayoutVerdict::Proposable);
        // Sans compte de versement.
        assert_eq!(
            judge_payout(&p, &[&a, &f], false),
            PayoutVerdict::ToReview(REVIEW_PAYOUT_ACCOUNT_MISSING)
        );
        // Total faux (une ligne manque).
        assert_eq!(
            judge_payout(&p, &[&a], true),
            PayoutVerdict::TotalMismatch {
                expected: dec("96.80"),
                found: dec("100")
            }
        );
        // À retour de versement : avant tout le reste.
        let r = line(4, "-38.54", "payout_reversal", "po_1");
        assert_eq!(
            judge_payout(&p, &[&a, &f, &r], false),
            PayoutVerdict::ToReview(REVIEW_PAYOUT_REVERSAL)
        );
        // De net positif ou nul.
        let positif = payout(5, "38.54", "po_2");
        assert_eq!(
            judge_payout(&positif, &[], true),
            PayoutVerdict::ToReview(REVIEW_PAYOUT_NOT_NEGATIVE)
        );
        let nul = payout(6, "0", "po_3");
        assert_eq!(
            judge_payout(&nul, &[], true),
            PayoutVerdict::ToReview(REVIEW_PAYOUT_NOT_NEGATIVE)
        );
    }

    // --- Revue sur des jeux construits à la main ---

    struct Jeu {
        pending: Vec<BankTransaction>,
        all: Vec<BankTransaction>,
        sales: Vec<SaleEntry>,
        destinations: HashMap<i64, PayoutDestination>,
        configured: HashMap<i64, i64>,
        refusals: HashMap<i64, &'static str>,
        unreceived: Vec<BankTransaction>,
        fee_account: Option<i64>,
        posted: HashMap<i64, i64>,
    }

    fn sale(id: i64, description: &str, ledger: i64, currency: &str, amount: &str) -> SaleEntry {
        SaleEntry {
            journal_entry_id: id,
            entry_number: id,
            entry_date: NaiveDate::from_ymd_opt(2020, 1, 29).unwrap(),
            description: description.into(),
            movements: vec![SaleMovement {
                ledger_account_id: ledger,
                currency: currency.into(),
                amount: dec(amount),
            }],
            linked: false,
        }
    }

    fn charge(id: i64, amount: &str, reference: &str) -> BankTransaction {
        let mut t = tx(id, amount, "charge", &format!("txn_{id}"));
        t.reference = Some(reference.into());
        t
    }

    impl Jeu {
        fn new() -> Self {
            Self {
                pending: Vec::new(),
                all: Vec::new(),
                sales: Vec::new(),
                destinations: HashMap::new(),
                configured: HashMap::new(),
                refusals: HashMap::new(),
                unreceived: Vec::new(),
                fee_account: Some(6941),
                posted: HashMap::new(),
            }
        }

        fn revue(&self) -> Review<StripeReviewProposal> {
            let accounts = vec![
                ProviderAccount {
                    bank_account_id: 10,
                    ledger_account_id: Some(1031),
                    currency: "EUR".into(),
                    fee_account_id: self.fee_account,
                },
                ProviderAccount {
                    bank_account_id: 12,
                    ledger_account_id: Some(1030),
                    currency: "CHF".into(),
                    fee_account_id: None,
                },
            ];
            let rules = Vec::new();
            let rule_ids = HashMap::new();
            let mut all = self.all.clone();
            all.extend(self.pending.iter().cloned());
            let common = build_review(
                &ReviewInput {
                    accounts: &accounts,
                    pending: &self.pending,
                    all_linked_accounts_transactions: &all,
                    sale_entries: &self.sales,
                    rules: &rules,
                    rule_account_ids: &rule_ids,
                    primary_ledger_account_id: None,
                    already_posted: &self.posted,
                },
                &SPEC,
            );
            complete_review(
                common,
                &StripeReviewInput {
                    pending: &self.pending,
                    payout_candidates: &all,
                    destinations: &self.destinations,
                    configured_destinations: &self.configured,
                    destination_refusals: &self.refusals,
                    unreceived_payouts: &self.unreceived,
                    already_posted: &self.posted,
                },
            )
        }
    }

    fn codes(l: &[ReviewFinding]) -> Vec<&'static str> {
        l.iter().map(|f| f.code).collect()
    }

    #[test]
    fn charge_concordante_proposee_au_rattachement() {
        let mut j = Jeu::new();
        j.pending.push(charge(1, "200.00", "ch_TEST0003"));
        j.sales
            .push(sale(50, "Boutique TXN ch_TEST0003", 1031, "EUR", "200.00"));
        let r = j.revue();
        assert_eq!(r.proposals.len(), 1);
        assert_eq!(
            r.proposals[0].proposal,
            StripeReviewProposal::Common(Proposal::LinkEntry {
                bank_transaction_id: 1,
                journal_entry_id: 50
            })
        );
        assert!(r.gaps.is_empty() && r.to_review.is_empty());
    }

    #[test]
    fn ecarts_des_charges() {
        let mut j = Jeu::new();
        // Sans vente.
        j.pending.push(charge(1, "80.00", "ch_TEST0099"));
        // Autre devise (vente au compte CHF).
        j.pending.push(charge(2, "50.00", "ch_B"));
        j.sales.push(sale(51, "TXN ch_B", 1030, "CHF", "50.00"));
        // Montant différent.
        j.pending.push(charge(3, "40.00", "ch_C"));
        j.sales.push(sale(52, "TXN ch_C", 1031, "EUR", "41.00"));
        // Déjà rattachée.
        j.pending.push(charge(4, "30.00", "ch_D"));
        let mut deja = sale(53, "TXN ch_D", 1031, "EUR", "30.00");
        deja.linked = true;
        j.sales.push(deja);
        // Deux ventes.
        j.pending.push(charge(5, "20.00", "ch_E"));
        j.sales.push(sale(54, "TXN ch_E", 1031, "EUR", "20.00"));
        j.sales.push(sale(55, "Doublon ch_E", 1031, "EUR", "20.00"));
        // Vente sans charge.
        j.sales.push(sale(56, "TXN ch_SANS", 1031, "EUR", "9.00"));
        let r = j.revue();
        assert!(r.proposals.is_empty(), "{:?}", r.proposals);
        // Tri par date : la vente sans charge (29 janvier) d'abord.
        assert_eq!(
            codes(&r.gaps),
            [
                GAP_SALE_WITHOUT_RECEIPT,
                GAP_RECEIPT_WITHOUT_SALE,
                GAP_AMOUNT_OR_CURRENCY_MISMATCH,
                GAP_AMOUNT_OR_CURRENCY_MISMATCH,
                GAP_AMOUNT_OR_CURRENCY_MISMATCH,
            ]
        );
        assert_eq!(codes(&r.to_review), [REVIEW_AMBIGUOUS_SALE]);
    }

    #[test]
    fn frais_remboursement_et_autres_categories() {
        let mut j = Jeu::new();
        j.pending.push(tx(1, "-3.20", "charge", "txn_1:FEE"));
        j.pending.push(tx(2, "0.60", "refund", "txn_3:FEE"));
        j.pending.push(tx(3, "-0.50", "fee", "txn_4"));
        j.pending.push(tx(4, "-20.00", "refund", "txn_3"));
        j.pending.push(tx(5, "-38.54", "payout_reversal", "txn_9"));
        let r = j.revue();
        let kinds: Vec<&str> = r
            .proposals
            .iter()
            .map(|p| match &p.proposal {
                StripeReviewProposal::Common(c) => c.kind(),
                StripeReviewProposal::Payout(_) => "payout",
            })
            .collect();
        assert_eq!(kinds, ["fee", "fee", "fee"]);
        assert_eq!(codes(&r.to_review), [REVIEW_REFUND, REVIEW_OTHER_CATEGORY]);
        assert_eq!(
            r.to_review[1].details,
            Some(serde_json::json!({ "category": "payout_reversal" }))
        );
        // Sans compte de frais : à examiner.
        j.fee_account = None;
        let r = j.revue();
        assert_eq!(
            codes(&r.to_review),
            [
                REVIEW_FEE_ACCOUNT_MISSING,
                REVIEW_FEE_ACCOUNT_MISSING,
                REVIEW_FEE_ACCOUNT_MISSING,
                REVIEW_REFUND,
                REVIEW_OTHER_CATEGORY
            ]
        );
    }

    #[test]
    fn versements_proposes_examines_ou_en_ecart() {
        let mut j = Jeu::new();
        j.destinations.insert(
            10,
            PayoutDestination {
                bank_account_id: 77,
                label: "Banque EUR".into(),
            },
        );
        // Ordinaire : les lignes, rapprochées, peuvent précéder l'exercice.
        j.pending.push(payout(1, "-194.00", "po_A"));
        let mut vieux = line(11, "200", "charge", "po_A");
        vieux.status = BankTransactionStatus::Reconciled;
        vieux.booking_date = NaiveDate::from_ymd_opt(2019, 12, 30).unwrap();
        j.all.push(vieux);
        j.all.push(line(12, "-6.00", "charge", "po_A"));
        // Total faux.
        j.pending.push(payout(2, "-50.00", "po_B"));
        j.all.push(line(13, "45", "charge", "po_B"));
        // À retour.
        j.pending.push(payout(3, "38.54", "po_C"));
        j.all.push(line(14, "-38.54", "payout_reversal", "po_C"));
        let r = j.revue();
        assert_eq!(r.proposals.len(), 1);
        let json = serde_json::to_value(&r.proposals[0]).unwrap();
        assert_eq!(json["type"], "payout");
        assert_eq!(json["bankTransactionId"], 1);
        assert_eq!(json["payoutBankAccountId"], 77);
        assert_eq!(json["payoutAccountLabel"], "Banque EUR");
        assert_eq!(codes(&r.gaps), [GAP_PAYOUT_TOTAL_MISMATCH]);
        assert_eq!(
            r.gaps[0].details,
            Some(serde_json::json!({ "payoutId": "po_B", "expected": "50.00", "found": "45.00" }))
        );
        // Le versement à retour (sa ligne de retour, rapprochée ou non, est
        // hors de la liste pending de ce jeu).
        assert_eq!(codes(&r.to_review), [REVIEW_PAYOUT_REVERSAL]);
        // Sans compte de versement : le motif passe avant le total.
        j.destinations.clear();
        let r = j.revue();
        assert!(r.proposals.is_empty());
        assert!(r.gaps.is_empty());
        assert_eq!(
            codes(&r.to_review),
            [
                REVIEW_PAYOUT_ACCOUNT_MISSING,
                REVIEW_PAYOUT_ACCOUNT_MISSING,
                REVIEW_PAYOUT_REVERSAL
            ]
        );
        // Le motif nommé : aucun réglage, puis le refus que la route a jugé.
        let po_a = |r: &Review<StripeReviewProposal>| {
            r.to_review
                .iter()
                .find(|f| {
                    f.code == REVIEW_PAYOUT_ACCOUNT_MISSING
                        && f.transaction.as_ref().is_some_and(|t| t.id == 1)
                })
                .and_then(|f| f.details.clone())
        };
        assert_eq!(
            po_a(&r),
            Some(serde_json::json!({ "payoutId": "po_A", "reason": PAYOUT_ACCOUNT_NOT_SET }))
        );
        j.refusals.insert(10, "paypal");
        assert_eq!(
            po_a(&j.revue()),
            Some(serde_json::json!({ "payoutId": "po_A", "reason": "paypal" }))
        );
    }

    /// Correctif des prestataires (#1539) : un versement déjà comptabilisé de
    /// l'autre côté va « à examiner » sous `already_posted`, l'écriture nommée,
    /// **avant tout autre motif** — compte de versement absent, total faux —,
    /// jamais proposé. La sonde positive : sans entrée dans `already_posted`,
    /// ou une entrée pour une autre transaction, le versement reste proposé.
    #[test]
    fn versement_deja_comptabilise_a_examiner_avant_tout_autre_motif() {
        let mut j = Jeu::new();
        j.destinations.insert(
            10,
            PayoutDestination {
                bank_account_id: 77,
                label: "Banque EUR".into(),
            },
        );
        j.pending.push(payout(1, "-194.00", "po_A"));
        j.all.push(line(11, "200", "charge", "po_A"));
        j.all.push(line(12, "-6.00", "charge", "po_A"));
        // Sonde positive : une écriture déjà comptabilisée pour une autre
        // transaction ne touche pas le versement.
        j.posted.insert(99, 901);
        let r = j.revue();
        assert_eq!(r.proposals.len(), 1, "{:?}", r.to_review);
        assert!(r.to_review.is_empty());
        // Déjà comptabilisé : à examiner, l'écriture nommée, plus proposé.
        j.posted.insert(1, 900);
        let r = j.revue();
        assert!(r.proposals.is_empty());
        assert!(r.gaps.is_empty());
        assert_eq!(codes(&r.to_review), [REVIEW_ALREADY_POSTED]);
        assert_eq!(r.to_review[0].transaction.as_ref().map(|t| t.id), Some(1));
        assert_eq!(
            r.to_review[0].details,
            Some(serde_json::json!({ "journalEntryId": 900 }))
        );
        // Avant le compte de versement absent…
        j.destinations.clear();
        assert_eq!(codes(&j.revue().to_review), [REVIEW_ALREADY_POSTED]);
        // … et avant le total faux.
        j.all.pop();
        let r = j.revue();
        assert!(r.gaps.is_empty(), "{:?}", r.gaps);
        assert_eq!(codes(&r.to_review), [REVIEW_ALREADY_POSTED]);
    }

    #[test]
    fn versement_sans_reception() {
        let mut j = Jeu::new();
        let mut p = payout(1, "-194.00", "po_A");
        p.status = BankTransactionStatus::Reconciled;
        p.matched_entry_id = Some(900);
        j.unreceived.push(p);
        j.configured.insert(10, 77);
        let r = j.revue();
        assert_eq!(codes(&r.gaps), [GAP_PAYOUT_WITHOUT_RECEIPT]);
        assert_eq!(
            r.gaps[0].details,
            Some(
                serde_json::json!({ "payoutId": "po_A", "journalEntryId": 900, "payoutBankAccountId": 77 })
            )
        );
        // Sans réglage : `null`.
        j.configured.clear();
        let r = j.revue();
        assert_eq!(
            r.gaps[0].details.as_ref().unwrap()["payoutBankAccountId"],
            serde_json::Value::Null
        );
    }
}
