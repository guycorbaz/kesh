//! Revue commune d'un compte de **prestataire de paiement** — Story 38-11c2
//! (AC-4, AC-5), servie à PayPal (38-11c2), Wise (39-11c) et Stripe (40-7c).
//!
//! Fonctions **pures**, sans I/O : la route lit les transactions, les écritures
//! de vente, les règles et les réglages (`kesh-api`, module
//! `routes::provider_accept`), puis appelle [`build_review`] avec le
//! **paramétrage du prestataire** ([`ProviderSpec`]). Rien n'est écrit : la
//! revue **propose** ; tout rattachement est une proposition que l'utilisateur
//! accepte (`CLAUDE.md`, « Un appariement automatique propose, il ne crée
//! jamais »).
//!
//! ⛔ **Aucun nom de prestataire dans ce module** (NFR-38-11c2-3) : ce qui
//! distingue PayPal, Wise et Stripe tient dans leur [`ProviderSpec`] —
//! classement, clé de rattachement, extraction des identifiants d'un libellé,
//! appariement des conversions, libellés des écritures. Un prestataire
//! qui a des genres propres (versement Stripe vu de Wise, versement Stripe) les
//! classe en [`ProviderKind::Specific`] : la revue commune les lui rend sans
//! décision ([`Review::specific`]), il les décide et les range lui-même
//! ([`Review::push_proposal`], [`Review::push_gap`], [`Review::push_to_review`])
//! avant [`Review::sort`].
//!
//! # Ce que décide [`build_review`]
//!
//! Pour chaque transaction **pending** du compte de prestataire, selon son genre :
//!
//! - **Encaissement** ([`ProviderKind::SaleReceipt`]) — les écritures de vente
//!   dont un identifiant égale la clé de la transaction : exactement une, sur le
//!   compte du plan du **même compte bancaire**, de mouvement en devise égal au
//!   montant, non rattachée → [`Proposal::LinkEntry`] ; aucune → écart
//!   [`GAP_RECEIPT_WITHOUT_SALE`] ; une, mais sur un autre compte du plan, de
//!   mouvement différent ou déjà rattachée → écart [`GAP_AMOUNT_OR_CURRENCY_MISMATCH`] ;
//!   plusieurs → à examiner [`REVIEW_AMBIGUOUS_SALE`].
//! - **Frais** ([`ProviderKind::Fee`]) — compte de frais réglé sur le compte
//!   bancaire → [`Proposal::Fee`] ; sinon [`REVIEW_FEE_ACCOUNT_MISSING`].
//! - **Conversion** ([`ProviderKind::Conversion`]) — l'**unique** autre
//!   transaction `Conversion` pending, de signe opposé, d'une autre devise, que
//!   le prédicat d'appariement du prestataire ([`ProviderSpec::conversion_match`])
//!   reconnaît, et dont la transaction est elle-même l'unique partenaire → [`Proposal::Conversion`]
//!   (une seule par paire) ; sinon le motif [`ProviderSpec::conversion_unpaired`].
//! - **Retrait** ([`ProviderKind::Withdrawal`]) — compte du plan en CHF →
//!   [`Proposal::Withdrawal`], contrepartie préremplie au compte du plan du
//!   compte bancaire principal (ou `None`) ; en devise → à examiner
//!   [`REVIEW_WITHDRAWAL_FOREIGN_CURRENCY`].
//! - **Autre** ([`ProviderKind::Other`]) — première règle qui correspond
//!   ([`crate::rules::rule_matches`], contrepartie dans l'ensemble admis, jamais
//!   le compte du plan de la transaction) → [`Proposal::Rule`] ; sinon
//!   [`REVIEW_NO_RULE`].
//!
//! Et pour chaque écriture de vente **non rattachée** dont **aucun**
//! identifiant n'est la clé d'une transaction (pending ou rapprochée) des
//! comptes bancaires liés aux mêmes comptes du plan → écart
//! [`GAP_SALE_WITHOUT_RECEIPT`].
//!
//! Chaque liste est triée par date, puis identifiant de la transaction (ou de
//! l'écriture pour un écart sans transaction).

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;
use kesh_db::entities::{BankTransaction, ReconciliationRule};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::rules::rule_matches;

/// La monnaie de base de Kesh (un compte du plan sans devise est en CHF).
pub const BASE_CURRENCY: &str = "CHF";

/// Borne du libellé d'une écriture passée à l'acceptation, en caractères
/// (celle des écritures de la réconciliation, `accept_one_rule`). Le moteur
/// (`kesh-api`, `routes::provider_accept`) y coupe tout libellé ; un
/// prestataire peut borner le sien avant, pour choisir ce qui cède (Stripe,
/// [`crate::stripe::bounded_label`]).
pub const ENTRY_DESCRIPTION_MAX_CHARS: usize = 200;

/// Écart : encaissement sans écriture de vente de même identifiant.
pub const GAP_RECEIPT_WITHOUT_SALE: &str = "receipt_without_sale";
/// Écart : écriture de vente non rattachée qu'aucune transaction n'encaisse.
pub const GAP_SALE_WITHOUT_RECEIPT: &str = "sale_without_receipt";
/// Écart : une écriture de vente, mais sur un autre compte (autre devise), de
/// mouvement différent, ou déjà rattachée.
pub const GAP_AMOUNT_OR_CURRENCY_MISMATCH: &str = "amount_or_currency_mismatch";
/// À examiner : plusieurs écritures de vente portent l'identifiant.
pub const REVIEW_AMBIGUOUS_SALE: &str = "ambiguous_sale";
/// À examiner : frais sans compte de frais réglé sur le compte bancaire.
pub const REVIEW_FEE_ACCOUNT_MISSING: &str = "fee_account_missing";
/// À examiner : conversion sans sa paire (motif de PayPal).
pub const REVIEW_CONVERSION_UNPAIRED: &str = "conversion_unpaired";
/// À examiner : retrait d'un solde en devise (phase 1 : jamais écrit, le CHF
/// reçu par la banque diffère du cours de Kesh).
pub const REVIEW_WITHDRAWAL_FOREIGN_CURRENCY: &str = "withdrawal_foreign_currency";
/// À examiner : aucune règle ne correspond.
pub const REVIEW_NO_RULE: &str = "no_rule";
/// À examiner : la transaction est **déjà comptabilisée** — une écriture
/// vivante porte son mouvement sur le compte du plan du solde et est déjà
/// rapprochée de **l'autre côté** (`details: { journalEntryId }`, la plus
/// proche par date ; [`ReviewInput::already_posted`]). Typiquement : le côté
/// prestataire d'un virement annulé seul (`reconciliation_cancel` ne retire
/// que son lien, l'écriture reste rapprochée de la réception). L'écrire de
/// nouveau la compterait deux fois : la transaction se **rattache** à cette
/// écriture. Remplace, pour les genres qui **écrivent** (frais, conversion,
/// retrait, autre : voir `writes`), la proposition ou le motif qu'ils
/// auraient reçus, avant tout autre motif ; à l'acceptation,
/// la même règle rend [`FAILED_ALREADY_POSTED`].
pub const REVIEW_ALREADY_POSTED: &str = "already_posted";
/// Code de refus (`failed[].errorCode`) de l'acceptation d'une transaction
/// déjà comptabilisée ([`REVIEW_ALREADY_POSTED`]) : rien n'est écrit
/// (`details: { journalEntryId }`).
pub const FAILED_ALREADY_POSTED: &str = "PROVIDER_TRANSACTION_ALREADY_POSTED";

/// Le genre d'une transaction de prestataire, rendu par
/// [`ProviderSpec::classify`]. Commun aux prestataires ; un prestataire range
/// ses genres propres en [`ProviderKind::Specific`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// Encaissement d'une vente faite ailleurs (boutique) : se rattache à
    /// l'écriture de vente.
    SaleReceipt,
    /// Frais du prestataire (prélevé ou rendu).
    Fee,
    /// Une moitié d'une conversion entre deux devises du prestataire.
    Conversion,
    /// Retrait vers un compte bancaire.
    Withdrawal,
    /// Le reste : achats, remboursements, litiges… (règles).
    Other,
    /// Genre propre au prestataire, décidé par lui hors de la revue commune ;
    /// la chaîne le nomme (`"stripe_payout"`…).
    Specific(&'static str),
}

/// Le genre de l'écriture dont un prestataire écrit le libellé
/// ([`ProviderSpec::entry_label`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// Écriture des frais.
    Fee,
    /// Écriture d'une règle.
    Rule,
    /// Écriture d'un retrait.
    Withdrawal,
    /// Écriture(s) d'une conversion.
    Conversion,
}

/// Où va une conversion sans sa paire : à examiner (PayPal) ou en écart
/// (Wise, `conversion_half_missing`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Dans `toReview`.
    ToReview,
    /// Dans `gaps`.
    Gap,
}

/// Le **paramétrage d'un prestataire** : tout ce qui distingue PayPal, Wise et
/// Stripe pour la revue et l'acceptation communes (NFR-38-11c2-3).
#[derive(Debug, Clone, Copy)]
pub struct ProviderSpec {
    /// Valeur de `bank_accounts.provider` (`"paypal"`…).
    pub provider: &'static str,
    /// Classement d'une transaction.
    pub classify: fn(&BankTransaction) -> ProviderKind,
    /// Clé de rattachement d'une transaction (identifiant que portent les
    /// libellés des écritures de vente) : `transaction_id` pour PayPal et Wise,
    /// `reference` pour Stripe. `None` : la transaction n'en a pas.
    pub match_key: fn(&BankTransaction) -> Option<&str>,
    /// Extraction des identifiants d'un libellé d'écriture, en forme
    /// comparable à [`ProviderSpec::match_key`].
    pub extract_ids: fn(&str) -> Vec<String>,
    /// L'identifiant `id`, extrait d'un libellé ([`ProviderSpec::extract_ids`]),
    /// est-il du **même genre de clé** que la transaction `t` — celui d'une
    /// transaction que l'écriture pourrait porter à la place de `t` ? Seul un
    /// tel identifiant fait d'une écriture celle d'**une autre** transaction et
    /// l'exclut du repli prudent ([`unnamed_and_free`]). PayPal : un seul genre
    /// ([`one_key_kind`]). Stripe : pour un versement, seul un `po_` — une
    /// charge citée au libellé d'un versement ne fait pas de son écriture celle
    /// d'un autre versement ; pour des frais, tout identifiant Stripe. Wise :
    /// le préfixe de l'identifiant (`TRANSFER`, `CARD`, `FEE-TRANSFER`…) — une
    /// carte citée au libellé d'un virement ne fait pas de son écriture celle
    /// d'un autre virement.
    pub same_key_kind: fn(&BankTransaction, &str) -> bool,
    /// Les deux transactions sont-elles les moitiés d'une même conversion,
    /// par leurs références (PayPal : même `Reference Txn ID`, ou chacune
    /// pointant l'autre ; Wise : même `transaction_id`) ? Le signe opposé et la
    /// devise différente sont jugés par le moteur.
    pub conversion_match: fn(&BankTransaction, &BankTransaction) -> bool,
    /// Motif et place d'une conversion sans paire.
    pub conversion_unpaired: (&'static str, Placement),
    /// Libellé d'une écriture passée à l'acceptation (`account` : le libellé
    /// du compte du prestataire). Tronqué par l'appelant à la borne existante.
    pub entry_label: fn(EntryKind, &str, &BankTransaction) -> String,
    /// Genres admis **en plus** d'un encaissement ([`ProviderKind::SaleReceipt`],
    /// toujours admis) pour la proposition [`Proposal::LinkEntry`] : l'autre
    /// côté déjà passé d'un virement ou d'un versement (Wise : `Other` de
    /// préfixe `TRANSFER` et versement Stripe, 39-11c AC-4). PayPal et Stripe : aucun.
    /// Voir [`admits_link`].
    pub linkable: fn(ProviderKind) -> bool,
    /// Genres admis **en plus** de [`ProviderKind::Other`] (toujours admis)
    /// pour la proposition [`Proposal::Rule`] (un genre propre que le
    /// prestataire règle, selon ses candidates, par une règle). PayPal :
    /// aucun. Voir [`admits_rule`].
    pub ruleable: fn(ProviderKind) -> bool,
}

/// Un seul genre de clé : tout identifiant d'un libellé est du genre de toute
/// transaction (paramétrage de PayPal pour [`ProviderSpec::same_key_kind`]).
pub fn one_key_kind(_: &BankTransaction, _: &str) -> bool {
    true
}

/// Aucun genre en plus (paramétrage de PayPal pour [`ProviderSpec::linkable`]
/// et [`ProviderSpec::ruleable`]).
pub fn no_extra_kind(_: ProviderKind) -> bool {
    false
}

/// Le genre `kind` admet-il la proposition [`Proposal::LinkEntry`] ?
/// Encaissement, ou genre que le prestataire déclare
/// ([`ProviderSpec::linkable`]). **Seule** règle : la revue d'un prestataire et
/// le rejugement de l'acceptation l'appellent. Pour un genre autre que
/// l'encaissement, seule la règle du rattachement (`link_entry_in_tx` :
/// mouvement, devise, contre-passation, liens) juge l'écriture.
pub fn admits_link(spec: &ProviderSpec, kind: ProviderKind) -> bool {
    kind == ProviderKind::SaleReceipt || (spec.linkable)(kind)
}

/// Le genre `kind` admet-il la proposition [`Proposal::Rule`] ? `Other`, ou
/// genre que le prestataire déclare ([`ProviderSpec::ruleable`]). Seule
/// règle, comme [`admits_link`].
pub fn admits_rule(spec: &ProviderSpec, kind: ProviderKind) -> bool {
    kind == ProviderKind::Other || (spec.ruleable)(kind)
}

/// Un compte bancaire du compte de prestataire (solde d'une devise).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAccount {
    /// Identifiant du compte bancaire.
    pub bank_account_id: i64,
    /// Son compte du plan (`None` : non configuré).
    pub ledger_account_id: Option<i64>,
    /// Devise de son compte du plan (`CHF` pour la monnaie de base).
    pub currency: String,
    /// Compte de frais réglé.
    pub fee_account_id: Option<i64>,
}

/// Une écriture de vente candidate, telle que lue par la route : écriture de
/// l'exercice, ni contre-passée ni contre-passation, avec au moins une ligne
/// au **débit** d'un des comptes du plan lus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaleEntry {
    /// Identifiant.
    pub journal_entry_id: i64,
    /// Numéro.
    pub entry_number: i64,
    /// Date.
    pub entry_date: NaiveDate,
    /// Libellé (d'où se tirent les identifiants).
    pub description: String,
    /// Mouvement de l'écriture sur chacun des comptes du plan lus, dans la
    /// devise du compte (CHF pour un compte sans devise), signé (débit > 0).
    pub movements: Vec<SaleMovement>,
    /// Une transaction bancaire la pointe déjà.
    pub linked: bool,
}

/// Mouvement d'une écriture de vente sur un compte du plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaleMovement {
    /// Le compte du plan.
    pub ledger_account_id: i64,
    /// Sa devise.
    pub currency: String,
    /// Le mouvement, dans cette devise (débit − crédit).
    pub amount: Decimal,
}

/// Ce que lit la route pour une revue.
#[derive(Debug, Clone, Copy)]
pub struct ReviewInput<'a> {
    /// Les comptes bancaires du compte de prestataire.
    pub accounts: &'a [ProviderAccount],
    /// Les transactions **pending** de ces comptes, datées dans l'exercice.
    pub pending: &'a [BankTransaction],
    /// **Toutes** les transactions (pending et rapprochées) des comptes
    /// bancaires liés aux mêmes comptes du plan, pour les clés des ventes
    /// encaissées.
    pub all_linked_accounts_transactions: &'a [BankTransaction],
    /// Les écritures de vente de l'exercice sur ces comptes du plan.
    pub sale_entries: &'a [SaleEntry],
    /// Les règles actives, dans l'ordre `priority, id`.
    pub rules: &'a [ReconciliationRule],
    /// Les comptes admis comme contrepartie d'une règle (actifs, imputables) et
    /// leur devise (`None` : CHF) : une règle ne s'applique à une transaction
    /// que si sa contrepartie est en CHF ou dans la devise de la transaction.
    pub rule_account_ids: &'a HashMap<i64, Option<String>>,
    /// Compte du plan du compte bancaire principal de la société.
    pub primary_ledger_account_id: Option<i64>,
    /// Les transactions pending **déjà comptabilisées** de l'autre côté :
    /// identifiant de la transaction → écriture ([`REVIEW_ALREADY_POSTED`]),
    /// lues par `kesh_db::repositories::reconciliation_link::posted_elsewhere_for_page`
    /// puis [`first_naming_entry`].
    pub already_posted: &'a HashMap<i64, i64>,
}

/// Une proposition — servie par la revue, et corps de l'acceptation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum Proposal {
    /// Rattacher la transaction à une écriture déjà passée (vente).
    LinkEntry {
        bank_transaction_id: i64,
        journal_entry_id: i64,
    },
    /// Écrire les frais au compte de frais.
    Fee {
        bank_transaction_id: i64,
        fee_account_id: i64,
    },
    /// Écrire par une règle.
    Rule {
        bank_transaction_id: i64,
        rule_id: i64,
        counterparty_account_id: i64,
    },
    /// Écrire une conversion (les deux moitiés).
    Conversion {
        debit_transaction_id: i64,
        credit_transaction_id: i64,
    },
    /// Écrire un retrait vers le compte choisi (obligatoire à l'acceptation).
    Withdrawal {
        bank_transaction_id: i64,
        counterparty_account_id: Option<i64>,
    },
}

impl Proposal {
    /// La transaction que nomme un refus (`FailedProposal`) : la **débitée**
    /// pour une conversion.
    pub fn primary_transaction_id(&self) -> i64 {
        match self {
            Self::LinkEntry {
                bank_transaction_id,
                ..
            }
            | Self::Fee {
                bank_transaction_id,
                ..
            }
            | Self::Rule {
                bank_transaction_id,
                ..
            }
            | Self::Withdrawal {
                bank_transaction_id,
                ..
            } => *bank_transaction_id,
            Self::Conversion {
                debit_transaction_id,
                ..
            } => *debit_transaction_id,
        }
    }

    /// Toutes les transactions de la proposition (deux pour une conversion).
    pub fn transaction_ids(&self) -> Vec<i64> {
        match self {
            Self::Conversion {
                debit_transaction_id,
                credit_transaction_id,
            } => vec![*debit_transaction_id, *credit_transaction_id],
            other => vec![other.primary_transaction_id()],
        }
    }

    /// Le genre, tel que sérialisé (`"link_entry"`…).
    pub fn kind(&self) -> &'static str {
        match self {
            Self::LinkEntry { .. } => "link_entry",
            Self::Fee { .. } => "fee",
            Self::Rule { .. } => "rule",
            Self::Conversion { .. } => "conversion",
            Self::Withdrawal { .. } => "withdrawal",
        }
    }
}

/// Ce que la revue montre d'une transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSummary {
    /// Identifiant.
    pub id: i64,
    /// Compte bancaire.
    pub bank_account_id: i64,
    /// Date ([`payment_date`] : `booking_date`).
    pub date: NaiveDate,
    /// Montant signé, deux décimales (chaîne décimale en JSON).
    pub amount: Decimal,
    /// Devise.
    pub currency: String,
    /// Type chez le prestataire.
    pub provider_type: Option<String>,
    /// Nom de la contrepartie.
    pub name: Option<String>,
    /// Identifiant chez le prestataire.
    pub transaction_id: Option<String>,
    /// E-mail de la contrepartie (colonne importée par 38-11b) — Story 38-11d
    /// (revue P1, M5) : l'écran le montre aux lignes « à examiner », pour écrire
    /// une règle à filtre « E-mail exact ».
    pub counterparty_email: Option<String>,
}

impl TransactionSummary {
    /// Le résumé de `t`.
    pub fn of(t: &BankTransaction) -> Self {
        let mut amount = t.amount;
        amount.rescale(2);
        Self {
            id: t.id,
            bank_account_id: t.bank_account_id,
            date: payment_date(t),
            amount,
            currency: t.currency.clone(),
            provider_type: t.provider_type.clone(),
            name: t.counterparty_name.clone(),
            transaction_id: t.transaction_id.clone(),
            counterparty_email: t.counterparty_email.clone(),
        }
    }
}

/// Ce que la revue montre d'une écriture de vente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySummary {
    /// Identifiant.
    pub journal_entry_id: i64,
    /// Numéro.
    pub entry_number: i64,
    /// Date.
    pub entry_date: NaiveDate,
    /// Libellé.
    pub description: String,
    /// Identifiants tirés du libellé.
    pub ids: Vec<String>,
}

impl EntrySummary {
    fn of(e: &SaleEntry, ids: Vec<String>) -> Self {
        Self {
            journal_entry_id: e.journal_entry_id,
            entry_number: e.entry_number,
            entry_date: e.entry_date,
            description: e.description.clone(),
            ids,
        }
    }
}

/// Une proposition de la revue, avec ce qu'elle montre.
///
/// **Générique** en `P`, le type de proposition (les genres communs,
/// [`Proposal`], par défaut) : un prestataire qui a des propositions propres
/// (Stripe : `payout`, 40-7c AC-4 d) sert les siennes dans la même revue,
/// comme le corps de l'acceptation (`AcceptBody<P>`), en convertissant la
/// revue commune ([`Review::map_proposals`]). Les champs d'affichage propres à
/// sa proposition (Stripe : `payoutBankAccountId`, libellé du compte de
/// versement) se portent dans `P`, aplati avec elle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewProposal<P = Proposal> {
    /// La proposition (champs aplatis, `type` compris).
    #[serde(flatten)]
    pub proposal: P,
    /// La transaction (la débitée pour une conversion).
    pub transaction: TransactionSummary,
    /// L'autre moitié d'une conversion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_transaction: Option<TransactionSummary>,
    /// L'écriture de vente d'un rattachement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<EntrySummary>,
}

/// Un écart ou une ligne à examiner.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFinding {
    /// Le motif (constantes de ce module, ou d'un prestataire).
    pub code: &'static str,
    /// La transaction (absente pour une vente sans encaissement).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<TransactionSummary>,
    /// Les écritures en cause.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<EntrySummary>,
    /// Détails propres au motif.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ReviewFinding {
    /// Le motif [`REVIEW_ALREADY_POSTED`] de `t`, qui nomme l'écriture
    /// `journal_entry_id`. Public : un prestataire qui décide d'un genre propre
    /// qui écrit (le versement Stripe) le sert de la même façon.
    pub fn already_posted(t: &BankTransaction, journal_entry_id: i64) -> Self {
        Self {
            details: Some(serde_json::json!({ "journalEntryId": journal_entry_id })),
            ..Self::on(REVIEW_ALREADY_POSTED, t)
        }
    }

    /// Un motif sur une transaction, sans détail.
    pub fn on(code: &'static str, t: &BankTransaction) -> Self {
        Self {
            code,
            transaction: Some(TransactionSummary::of(t)),
            entries: Vec::new(),
            details: None,
        }
    }

    /// Clé de tri : date, puis identifiant (transaction, sinon écriture).
    fn sort_key(&self) -> (NaiveDate, i64, i64) {
        match (&self.transaction, self.entries.first()) {
            (Some(t), _) => (t.date, t.id, 0),
            (None, Some(e)) => (e.entry_date, i64::MAX, e.journal_entry_id),
            (None, None) => (NaiveDate::MIN, i64::MAX, i64::MAX),
        }
    }
}

/// Le résultat d'une revue, générique en `P` comme [`ReviewProposal`].
#[derive(Debug, Clone, PartialEq)]
pub struct Review<P = Proposal> {
    /// Propositions.
    pub proposals: Vec<ReviewProposal<P>>,
    /// Écarts.
    pub gaps: Vec<ReviewFinding>,
    /// Lignes à examiner.
    pub to_review: Vec<ReviewFinding>,
    /// Transactions de genre [`ProviderKind::Specific`], laissées au
    /// prestataire (identifiant, genre).
    pub specific: Vec<(i64, &'static str)>,
}

impl<P> Default for Review<P> {
    fn default() -> Self {
        Self {
            proposals: Vec::new(),
            gaps: Vec::new(),
            to_review: Vec::new(),
            specific: Vec::new(),
        }
    }
}

impl<P> Review<P> {
    /// La même revue, chaque proposition convertie par `f` (un prestataire
    /// passe la revue commune à son propre type de proposition, puis y ajoute
    /// les siennes, [`Review::push_proposal`]). Écarts, lignes à examiner et
    /// genres propres sont gardés tels quels.
    pub fn map_proposals<Q>(self, f: impl Fn(P) -> Q) -> Review<Q> {
        Review {
            proposals: self
                .proposals
                .into_iter()
                .map(|p| ReviewProposal {
                    proposal: f(p.proposal),
                    transaction: p.transaction,
                    other_transaction: p.other_transaction,
                    entry: p.entry,
                })
                .collect(),
            gaps: self.gaps,
            to_review: self.to_review,
            specific: self.specific,
        }
    }

    /// Ajoute une proposition.
    pub fn push_proposal(&mut self, p: ReviewProposal<P>) {
        self.proposals.push(p);
    }

    /// Ajoute un écart.
    pub fn push_gap(&mut self, f: ReviewFinding) {
        self.gaps.push(f);
    }

    /// Ajoute une ligne à examiner.
    pub fn push_to_review(&mut self, f: ReviewFinding) {
        self.to_review.push(f);
    }

    /// Trie chaque liste : date, puis identifiant.
    pub fn sort(&mut self) {
        self.proposals
            .sort_by_key(|p| (p.transaction.date, p.transaction.id));
        self.gaps.sort_by_key(ReviewFinding::sort_key);
        self.to_review.sort_by_key(ReviewFinding::sort_key);
        self.specific.sort();
    }
}

/// La date d'une transaction de prestataire : sa `booking_date`, le jour du
/// mouvement chez le prestataire — celle de la revue (période, tri, résumé) et
/// des écritures de l'acceptation.
///
/// ⚠️ **Jamais `value_date`** (amendement du 21.11 de la Story 40-7c) : seul
/// l'import Stripe la remplit (`available_on`, la date où les fonds se
/// libèrent, quelques jours après la vente) ; une vente du 30 décembre aurait
/// daté ses frais de janvier, dans l'exercice suivant. Les imports PayPal et
/// Wise la laissent vide : pour eux, rien ne change. La période de la revue
/// (`kesh_db::repositories::provider_review::pending_in_period`) suit la même
/// date.
pub fn payment_date(t: &BankTransaction) -> NaiveDate {
    t.booking_date
}

/// La revue — voir l'en-tête du module.
pub fn build_review(input: &ReviewInput<'_>, spec: &ProviderSpec) -> Review {
    let mut review = Review::default();
    let accounts: HashMap<i64, &ProviderAccount> = input
        .accounts
        .iter()
        .map(|a| (a.bank_account_id, a))
        .collect();

    // Les identifiants de chaque écriture de vente, extraits une fois ; les
    // écritures sans identifiant ne sont pas des ventes.
    let sales: Vec<(&SaleEntry, Vec<String>)> = input
        .sale_entries
        .iter()
        .map(|e| (e, (spec.extract_ids)(&e.description)))
        .filter(|(_, ids)| !ids.is_empty())
        .collect();
    let mut sales_by_id: HashMap<&str, Vec<usize>> = HashMap::new();
    for (idx, (_, ids)) in sales.iter().enumerate() {
        for id in ids {
            let list = sales_by_id.entry(id.as_str()).or_default();
            if !list.contains(&idx) {
                list.push(idx);
            }
        }
    }

    let classified: Vec<(&BankTransaction, ProviderKind)> = input
        .pending
        .iter()
        .map(|t| (t, (spec.classify)(t)))
        .collect();
    // Une moitié de conversion déjà comptabilisée ne s'apparie pas : sa
    // partenaire, si elle est pending, reste sans paire.
    let conversions: Vec<&BankTransaction> = classified
        .iter()
        .filter(|(t, k)| {
            *k == ProviderKind::Conversion && !input.already_posted.contains_key(&t.id)
        })
        .map(|(t, _)| *t)
        .collect();

    for (t, kind) in &classified {
        let account = accounts.get(&t.bank_account_id);
        // Revue P1 du correctif (M1) : avant tout autre motif — une transaction
        // déjà comptabilisée ne se montre jamais sous « aucune règle » ou
        // « compte de frais non réglé », qui conseillent d'écrire de nouveau.
        if let Some(entry_id) = input.already_posted.get(&t.id).copied()
            && writes(spec, *kind)
        {
            review.push_to_review(ReviewFinding::already_posted(t, entry_id));
            continue;
        }
        match kind {
            ProviderKind::SaleReceipt => {
                decide_receipt(&mut review, t, account.copied(), &sales, &sales_by_id, spec)
            }
            ProviderKind::Fee => match account.and_then(|a| a.fee_account_id) {
                Some(fee_account_id) => review.push_proposal(ReviewProposal {
                    proposal: Proposal::Fee {
                        bank_transaction_id: t.id,
                        fee_account_id,
                    },
                    transaction: TransactionSummary::of(t),
                    other_transaction: None,
                    entry: None,
                }),
                None => review.push_to_review(ReviewFinding::on(REVIEW_FEE_ACCOUNT_MISSING, t)),
            },
            ProviderKind::Conversion => decide_conversion(&mut review, t, &conversions, spec),
            ProviderKind::Withdrawal => {
                if account.is_some_and(|a| a.currency == BASE_CURRENCY) {
                    review.push_proposal(ReviewProposal {
                        proposal: Proposal::Withdrawal {
                            bank_transaction_id: t.id,
                            counterparty_account_id: input.primary_ledger_account_id,
                        },
                        transaction: TransactionSummary::of(t),
                        other_transaction: None,
                        entry: None,
                    });
                } else {
                    review.push_to_review(ReviewFinding::on(REVIEW_WITHDRAWAL_FOREIGN_CURRENCY, t));
                }
            }
            ProviderKind::Other => {
                let ledger = account.and_then(|a| a.ledger_account_id);
                match matching_rule(input.rules, t, input.rule_account_ids, ledger) {
                    Some(rule) => review.push_proposal(ReviewProposal {
                        proposal: Proposal::Rule {
                            bank_transaction_id: t.id,
                            rule_id: rule.id,
                            counterparty_account_id: rule.counterparty_account_id,
                        },
                        transaction: TransactionSummary::of(t),
                        other_transaction: None,
                        entry: None,
                    }),
                    None => review.push_to_review(ReviewFinding::on(REVIEW_NO_RULE, t)),
                }
            }
            ProviderKind::Specific(tag) => review.specific.push((t.id, tag)),
        }
    }

    // Ventes sans encaissement : aucune clé de transaction (pending ou
    // rapprochée) des comptes bancaires liés aux mêmes comptes du plan.
    let keys: HashSet<&str> = input
        .all_linked_accounts_transactions
        .iter()
        .chain(input.pending.iter())
        .filter_map(|t| (spec.match_key)(t))
        .collect();
    for (entry, ids) in &sales {
        if entry.linked || ids.iter().any(|id| keys.contains(id.as_str())) {
            continue;
        }
        let first = entry.movements.iter().find(|m| m.amount > Decimal::ZERO);
        let mut details = serde_json::Map::new();
        if let Some(m) = first {
            let mut amount = m.amount;
            amount.rescale(2);
            details.insert("amount".into(), serde_json::json!(amount.to_string()));
            details.insert("currency".into(), serde_json::json!(m.currency));
            details.insert(
                "ledgerAccountId".into(),
                serde_json::json!(m.ledger_account_id),
            );
        }
        review.push_gap(ReviewFinding {
            code: GAP_SALE_WITHOUT_RECEIPT,
            transaction: None,
            entries: vec![EntrySummary::of(entry, ids.clone())],
            details: Some(serde_json::Value::Object(details)),
        });
    }

    review.sort();
    review
}

/// Le genre `kind` **écrit-il** à l'acceptation ? Frais, conversion, retrait,
/// autre (règle) — sauf un genre que le prestataire admet au **rattachement**
/// ([`admits_link`] : l'encaissement, et ce qu'il déclare par
/// [`ProviderSpec::linkable`]) : sa proposition rattache à une écriture déjà
/// passée, ce que la règle [`REVIEW_ALREADY_POSTED`] ne doit pas empêcher.
/// Les genres propres ([`ProviderKind::Specific`]) restent au prestataire.
fn writes(spec: &ProviderSpec, kind: ProviderKind) -> bool {
    matches!(
        kind,
        ProviderKind::Fee
            | ProviderKind::Conversion
            | ProviderKind::Withdrawal
            | ProviderKind::Other
    ) && !admits_link(spec, kind)
}

/// La première règle qui correspond à `t`, de contrepartie admise — en CHF ou
/// dans la devise de `t` — et autre que le compte du plan de la transaction
/// (`ledger`). Même ordre et même
/// prédicat que [`crate::rules::first_matching_rule`].
pub fn matching_rule<'a>(
    rules: &'a [ReconciliationRule],
    t: &BankTransaction,
    rule_account_ids: &HashMap<i64, Option<String>>,
    ledger: Option<i64>,
) -> Option<&'a ReconciliationRule> {
    rules
        .iter()
        .filter(|r| r.active)
        .filter(|r| {
            rule_account_ids
                .get(&r.counterparty_account_id)
                .is_some_and(|currency| currency.as_deref().is_none_or(|c| c == t.currency))
        })
        .filter(|r| Some(r.counterparty_account_id) != ledger)
        .find(|r| rule_matches(r, t))
}

/// Décision d'un encaissement (voir l'en-tête du module).
fn decide_receipt(
    review: &mut Review,
    t: &BankTransaction,
    account: Option<&ProviderAccount>,
    sales: &[(&SaleEntry, Vec<String>)],
    sales_by_id: &HashMap<&str, Vec<usize>>,
    spec: &ProviderSpec,
) {
    let matches: &[usize] = (spec.match_key)(t)
        .and_then(|k| sales_by_id.get(k))
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    match matches {
        [] => review.push_gap(ReviewFinding::on(GAP_RECEIPT_WITHOUT_SALE, t)),
        [only] => {
            let (entry, ids) = &sales[*only];
            let ledger = account.and_then(|a| a.ledger_account_id);
            let movement =
                ledger.and_then(|l| entry.movements.iter().find(|m| m.ledger_account_id == l));
            let concordant = movement.is_some_and(|m| m.amount == t.amount) && !entry.linked;
            if concordant {
                review.push_proposal(ReviewProposal {
                    proposal: Proposal::LinkEntry {
                        bank_transaction_id: t.id,
                        journal_entry_id: entry.journal_entry_id,
                    },
                    transaction: TransactionSummary::of(t),
                    other_transaction: None,
                    entry: Some(EntrySummary::of(entry, ids.clone())),
                });
                return;
            }
            // Trouvé : le mouvement du même compte du plan s'il existe, sinon
            // le premier débit d'un autre compte (autre devise).
            let found =
                movement.or_else(|| entry.movements.iter().find(|m| m.amount > Decimal::ZERO));
            let mut expected = t.amount;
            expected.rescale(2);
            let mut details = serde_json::Map::new();
            details.insert(
                "journalEntryId".into(),
                serde_json::json!(entry.journal_entry_id),
            );
            details.insert(
                "expectedAmount".into(),
                serde_json::json!(expected.to_string()),
            );
            details.insert("expectedCurrency".into(), serde_json::json!(t.currency));
            if let Some(m) = found {
                let mut amount = m.amount;
                amount.rescale(2);
                details.insert("foundAmount".into(), serde_json::json!(amount.to_string()));
                details.insert("foundCurrency".into(), serde_json::json!(m.currency));
            } else {
                details.insert("foundAmount".into(), serde_json::Value::Null);
                details.insert("foundCurrency".into(), serde_json::Value::Null);
            }
            details.insert("alreadyLinked".into(), serde_json::json!(entry.linked));
            review.push_gap(ReviewFinding {
                code: GAP_AMOUNT_OR_CURRENCY_MISMATCH,
                transaction: Some(TransactionSummary::of(t)),
                entries: vec![EntrySummary::of(entry, ids.clone())],
                details: Some(serde_json::Value::Object(details)),
            });
        }
        several => review.push_to_review(ReviewFinding {
            code: REVIEW_AMBIGUOUS_SALE,
            transaction: Some(TransactionSummary::of(t)),
            entries: several
                .iter()
                .map(|i| EntrySummary::of(sales[*i].0, sales[*i].1.clone()))
                .collect(),
            details: None,
        }),
    }
}

/// Deux moitiés d'une même conversion ? Signe opposé (montants non nuls),
/// devises différentes, et le prédicat du prestataire. **Seule** définition :
/// la revue et le rejugement de l'acceptation l'appellent.
pub fn conversion_halves(a: &BankTransaction, b: &BankTransaction, spec: &ProviderSpec) -> bool {
    a.id != b.id
        && !a.amount.is_zero()
        && !b.amount.is_zero()
        && a.amount.is_sign_negative() != b.amount.is_sign_negative()
        && a.currency != b.currency
        && (spec.conversion_match)(a, b)
}

/// L'écriture de libellé `description` **nomme-t-elle** la transaction `t` ?
/// Correctif « une transaction de prestataire n'est écrite qu'une fois »
/// (#1538) : la condition qui distingue, parmi les transactions de même
/// montant d'un même solde, celle dont une écriture est **déjà passée** —
/// l'écriture la nomme. Deux formes, sans autre lien structuré entre une
/// écriture et une transaction détachée (`matched_entry_id`, effacé par le
/// détachement, est le seul) :
///
/// - **par son libellé** : les identifiants qu'il porte
///   ([`ProviderSpec::extract_ids`]) comprennent la clé de `t`
///   ([`ProviderSpec::match_key`]) — la règle du rattachement d'une vente. Le
///   libellé est celui de l'acceptation ([`ProviderSpec::entry_label`] : PayPal
///   « … · TXN <id> » ; Wise « Wise <compte> · <type> · <id> », ses frais
///   compris, « Frais Wise <devise> · FEE-<id> »). Les frais PayPal ont pour clé `<id>:FEE`, que leur
///   écriture ne porte pas (« Frais PayPal … · TXN <id> ») : ils ne sont
///   jamais nommés, par l'écriture principale ni par la leur. L'écriture des
///   frais nomme la transaction principale, mais n'est jamais rapprochée de
///   l'autre côté (elle n'a de ligne sur aucun autre compte bancaire) : la
///   présélection de l'appelant l'écarte ;
/// - **par sa paire de conversion** : une conversion en une écriture ne
///   nomme que sa moitié débitée ; l'autre moitié est nommée si l'écriture
///   est rapprochée (`linked` : les transactions rapprochées de l'écriture)
///   de sa paire ([`conversion_halves`]), que l'écriture nomme.
///
/// Une écriture sans aucun identifiant n'en nomme aucune : voir
/// [`unnamed_and_free`] et [`first_naming_entry`].
pub fn entry_names(
    spec: &ProviderSpec,
    t: &BankTransaction,
    description: &str,
    linked: &[&BankTransaction],
) -> bool {
    let ids = (spec.extract_ids)(description);
    let named =
        |x: &BankTransaction| (spec.match_key)(x).is_some_and(|k| ids.iter().any(|i| i == k));
    named(t)
        || ((spec.classify)(t) == ProviderKind::Conversion
            && linked
                .iter()
                .any(|p| conversion_halves(t, p, spec) && named(p)))
}

/// L'écriture de libellé `description` ne porte **aucun** identifiant du
/// genre de clé de `t` ([`ProviderSpec::same_key_kind`] ; une écriture saisie
/// à la main, puis rattachée, sans identifiant ou avec seulement des
/// identifiants d'un autre genre — une charge `ch_…` citée au libellé d'un
/// versement Stripe) **et** son côté prestataire est **libre** : aucune
/// transaction du compte bancaire de `t` ne lui est rapprochée (`linked` :
/// celles qui le sont). C'est l'écriture d'une transaction détachée de ce
/// compte, sans dire laquelle : le repli prudent de [`first_naming_entry`]
/// (revue P1 de la #1560, M1 ; identifiant d'un autre genre : #1539). Une
/// écriture dont le côté prestataire est encore rapproché (une transaction
/// qu'elle porte toujours, par exemple un retrait d'un mois passé du même
/// montant) n'est l'écriture d'aucune transaction en attente.
pub fn unnamed_and_free(
    spec: &ProviderSpec,
    t: &BankTransaction,
    description: &str,
    linked: &[&BankTransaction],
) -> bool {
    !(spec.extract_ids)(description)
        .iter()
        .any(|id| (spec.same_key_kind)(t, id))
        && !linked
            .iter()
            .any(|p| p.bank_account_id == t.bank_account_id)
}

/// L'écriture de libellé `description` nomme-t-elle **une autre** transaction
/// que `t`, du même genre de clé : un identifiant qu'elle porte est du genre
/// de `t` ([`ProviderSpec::same_key_kind`]) et n'est pas sa clé
/// ([`ProviderSpec::match_key`]) ? C'est alors l'écriture de cette autre
/// transaction, jamais celle de `t` : [`first_naming_entry`] ne la retient
/// pas, et un prestataire qui propose de rattacher `t` à l'autre côté déjà
/// passé d'un virement (Wise, 39-11c AC-4) l'écarte de ses candidates.
pub fn names_another(spec: &ProviderSpec, t: &BankTransaction, description: &str) -> bool {
    let key = (spec.match_key)(t);
    (spec.extract_ids)(description)
        .iter()
        .any(|id| (spec.same_key_kind)(t, id) && key != Some(id.as_str()))
}

/// L'écriture déjà passée de `t` parmi les `candidates` (déjà triées : la plus
/// proche par date, puis la plus ancienne), ou `None`. `linked` : les
/// transactions rapprochées de ces écritures, toutes ensemble (chacune porte
/// son `matched_entry_id`). Deux passes :
///
/// 1. la première qui **nomme** `t` ([`entry_names`]) — une écriture qui nomme
///    `t` passe toujours avant une écriture sans identifiant, même plus proche ;
/// 2. à défaut, **par prudence**, la première sans identifiant du genre de
///    clé de `t` dont le côté prestataire est libre ([`unnamed_and_free`]) :
///    elle ne dit pas quelle transaction elle porte, et vaut pour toute
///    transaction de ce montant du compte — en refuser une de trop (« à
///    examiner », écrite à la main) coûte moins que d'en écrire une seconde
///    fois.
///
/// Une écriture qui nomme une **autre** transaction du même genre de clé n'est
/// jamais retenue. La **seule** décision « déjà comptabilisée » d'une
/// transaction de prestataire (#1538) : la revue (sur la lecture par page) et
/// l'acceptation (sous les verrous du lot) l'appellent sur la même présélection
/// (`kesh_db::repositories::reconciliation_link::find_posted_elsewhere`).
pub fn first_naming_entry<'a>(
    spec: &ProviderSpec,
    t: &BankTransaction,
    candidates: impl IntoIterator<Item = (i64, &'a str)>,
    linked: &[BankTransaction],
) -> Option<i64> {
    let candidates: Vec<(i64, &str, Vec<&BankTransaction>)> = candidates
        .into_iter()
        .map(|(entry_id, description)| {
            let on_entry = linked
                .iter()
                .filter(|p| p.matched_entry_id == Some(entry_id))
                .collect();
            (entry_id, description, on_entry)
        })
        .collect();
    let pass = |keep: &dyn Fn(&str, &[&BankTransaction]) -> bool| {
        candidates
            .iter()
            .find(|(_, d, on)| keep(d, on))
            .map(|(id, _, _)| *id)
    };
    pass(&|d, on| entry_names(spec, t, d, on))
        .or_else(|| pass(&|d, on| unnamed_and_free(spec, t, d, on)))
}

/// Les partenaires possibles de `t` parmi les conversions pending
/// ([`conversion_halves`]).
fn conversion_partners<'a>(
    t: &BankTransaction,
    conversions: &[&'a BankTransaction],
    spec: &ProviderSpec,
) -> Vec<&'a BankTransaction> {
    conversions
        .iter()
        .copied()
        .filter(|o| conversion_halves(t, o, spec))
        .collect()
}

/// L'appariement d'une conversion, **commun** aux prestataires (la clé
/// d'appariement est au paramétrage) : pure, sans I/O.
///
/// Rend la paire `(débitée, créditée)` quand `t` a un seul partenaire et en est
/// lui-même le seul partenaire.
pub fn pair_conversion<'a>(
    t: &'a BankTransaction,
    conversions: &[&'a BankTransaction],
    spec: &ProviderSpec,
) -> Option<(&'a BankTransaction, &'a BankTransaction)> {
    let partners = conversion_partners(t, conversions, spec);
    let [other] = partners.as_slice() else {
        return None;
    };
    let back = conversion_partners(other, conversions, spec);
    if back.len() != 1 || back[0].id != t.id {
        return None;
    }
    if t.amount < Decimal::ZERO {
        Some((t, *other))
    } else {
        Some((*other, t))
    }
}

/// Décision d'une moitié de conversion (une proposition par paire, rangée
/// depuis sa moitié débitée).
fn decide_conversion(
    review: &mut Review,
    t: &BankTransaction,
    conversions: &[&BankTransaction],
    spec: &ProviderSpec,
) {
    match pair_conversion(t, conversions, spec) {
        Some((debit, credit)) => {
            if debit.id == t.id {
                review.push_proposal(ReviewProposal {
                    proposal: Proposal::Conversion {
                        debit_transaction_id: debit.id,
                        credit_transaction_id: credit.id,
                    },
                    transaction: TransactionSummary::of(debit),
                    other_transaction: Some(TransactionSummary::of(credit)),
                    entry: None,
                });
            }
        }
        None => {
            let (code, placement) = spec.conversion_unpaired;
            let finding = ReviewFinding::on(code, t);
            match placement {
                Placement::ToReview => review.push_to_review(finding),
                Placement::Gap => review.push_gap(finding),
            }
        }
    }
}

/// Décomptes de la revue, servis tels quels (`counts`).
pub fn counts<P>(review: &Review<P>) -> BTreeMap<&'static str, usize> {
    let mut c = BTreeMap::new();
    c.insert("proposals", review.proposals.len());
    c.insert("gaps", review.gaps.len());
    c.insert("toReview", review.to_review.len());
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_db::entities::ReconciliationMatchType;
    use kesh_db::entities::bank_transaction::BankTransactionStatus;
    use std::str::FromStr;

    pub(crate) fn tx(id: i64, account: i64, amount: &str, currency: &str) -> BankTransaction {
        let now = chrono::Utc::now().naive_utc();
        BankTransaction {
            id,
            company_id: 1,
            import_id: 1,
            bank_account_id: account,
            booking_date: NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            value_date: None,
            amount: Decimal::from_str(amount).unwrap(),
            currency: currency.into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            provider_type: None,
            counterparty_email: None,
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    /// Un paramétrage d'essai : le genre est le `provider_type` (« sale »,
    /// « fee », « conv », « wd », « spec », autre), la clé est
    /// `transaction_id`, l'extraction prend les mots `ID…` du libellé.
    fn classify(t: &BankTransaction) -> ProviderKind {
        match t.provider_type.as_deref() {
            Some("sale") => ProviderKind::SaleReceipt,
            Some("fee") => ProviderKind::Fee,
            Some("conv") => ProviderKind::Conversion,
            Some("wd") => ProviderKind::Withdrawal,
            Some("spec") => ProviderKind::Specific("special"),
            _ => ProviderKind::Other,
        }
    }
    fn key(t: &BankTransaction) -> Option<&str> {
        t.transaction_id.as_deref()
    }
    fn reference_key(t: &BankTransaction) -> Option<&str> {
        t.reference.as_deref()
    }
    fn extract(s: &str) -> Vec<String> {
        s.split_whitespace()
            .filter(|w| w.starts_with("ID"))
            .map(str::to_string)
            .collect()
    }
    fn conv_match(a: &BankTransaction, b: &BankTransaction) -> bool {
        a.end_to_end_id.as_deref().is_some_and(|k| !k.is_empty())
            && a.end_to_end_id == b.end_to_end_id
    }
    fn label(_: EntryKind, _: &str, _: &BankTransaction) -> String {
        String::new()
    }
    const SPEC: ProviderSpec = ProviderSpec {
        provider: "test",
        classify,
        match_key: key,
        extract_ids: extract,
        same_key_kind: one_key_kind,
        conversion_match: conv_match,
        conversion_unpaired: (REVIEW_CONVERSION_UNPAIRED, Placement::ToReview),
        entry_label: label,
        linkable: no_extra_kind,
        ruleable: no_extra_kind,
    };

    fn typed(mut t: BankTransaction, kind: &str, id: &str) -> BankTransaction {
        t.provider_type = Some(kind.into());
        t.transaction_id = Some(id.into());
        t
    }

    fn account(id: i64, ledger: i64, currency: &str, fee: Option<i64>) -> ProviderAccount {
        ProviderAccount {
            bank_account_id: id,
            ledger_account_id: Some(ledger),
            currency: currency.into(),
            fee_account_id: fee,
        }
    }

    fn sale(id: i64, description: &str, ledger: i64, currency: &str, amount: &str) -> SaleEntry {
        SaleEntry {
            journal_entry_id: id,
            entry_number: id,
            entry_date: NaiveDate::from_ymd_opt(2025, 2, 27).unwrap(),
            description: description.into(),
            movements: vec![SaleMovement {
                ledger_account_id: ledger,
                currency: currency.into(),
                amount: Decimal::from_str(amount).unwrap(),
            }],
            linked: false,
        }
    }

    fn rule(id: i64, value: &str, counterparty: i64) -> ReconciliationRule {
        let now = chrono::Utc::now().naive_utc();
        ReconciliationRule {
            id,
            company_id: 1,
            label: format!("r{id}"),
            match_type: ReconciliationMatchType::CounterpartyContains,
            match_value: value.into(),
            counterparty_account_id: counterparty,
            default_project_id: None,
            priority: 0,
            active: true,
            applied_count: 0,
            last_applied_at: None,
            provider_type: None,
            counterparty_email: None,
            direction: None,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    struct Fixture {
        accounts: Vec<ProviderAccount>,
        pending: Vec<BankTransaction>,
        others: Vec<BankTransaction>,
        sales: Vec<SaleEntry>,
        rules: Vec<ReconciliationRule>,
        rule_accounts: HashMap<i64, Option<String>>,
        primary: Option<i64>,
        posted: HashMap<i64, i64>,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                // 10 : USD (plan 1021, frais 6940) ; 11 : CHF (plan 1020, sans frais).
                accounts: vec![
                    account(10, 1021, "USD", Some(6940)),
                    account(11, 1020, "CHF", None),
                ],
                pending: Vec::new(),
                others: Vec::new(),
                sales: Vec::new(),
                rules: Vec::new(),
                rule_accounts: [(6500, None), (4000, None)].into_iter().collect(),
                primary: Some(1010),
                posted: HashMap::new(),
            }
        }
        fn review(&self, spec: &ProviderSpec) -> Review {
            build_review(
                &ReviewInput {
                    accounts: &self.accounts,
                    pending: &self.pending,
                    all_linked_accounts_transactions: &self.others,
                    sale_entries: &self.sales,
                    rules: &self.rules,
                    rule_account_ids: &self.rule_accounts,
                    primary_ledger_account_id: self.primary,
                    already_posted: &self.posted,
                },
                spec,
            )
        }
    }

    fn codes(list: &[ReviewFinding]) -> Vec<&'static str> {
        list.iter().map(|f| f.code).collect()
    }

    #[test]
    fn vente_concordante_proposee_au_rattachement() {
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "25.00", "USD"), "sale", "IDA"));
        f.sales.push(sale(100, "Vente IDA", 1021, "USD", "25.00"));
        let r = f.review(&SPEC);
        assert_eq!(
            r.proposals
                .iter()
                .map(|p| p.proposal.clone())
                .collect::<Vec<_>>(),
            vec![Proposal::LinkEntry {
                bank_transaction_id: 1,
                journal_entry_id: 100
            }]
        );
        assert!(r.gaps.is_empty() && r.to_review.is_empty());
    }

    #[test]
    fn encaissement_sans_vente() {
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "25.00", "USD"), "sale", "IDA"));
        let r = f.review(&SPEC);
        assert_eq!(codes(&r.gaps), [GAP_RECEIPT_WITHOUT_SALE]);
        assert!(r.proposals.is_empty());
    }

    #[test]
    fn vente_sur_l_autre_devise_montant_different_ou_deja_rattachee() {
        // Sur l'autre compte du plan (CHF) : écart.
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "25.00", "USD"), "sale", "IDA"));
        f.sales.push(sale(100, "Vente IDA", 1020, "CHF", "25.00"));
        let r = f.review(&SPEC);
        assert_eq!(codes(&r.gaps), [GAP_AMOUNT_OR_CURRENCY_MISMATCH]);
        let d = r.gaps[0].details.as_ref().unwrap();
        assert_eq!(d["foundCurrency"], "CHF");
        assert_eq!(d["expectedCurrency"], "USD");
        assert_eq!(d["expectedAmount"], "25.00");

        // Montant différent.
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "25.00", "USD"), "sale", "IDA"));
        f.sales.push(sale(100, "Vente IDA", 1021, "USD", "24.00"));
        let r = f.review(&SPEC);
        assert_eq!(codes(&r.gaps), [GAP_AMOUNT_OR_CURRENCY_MISMATCH]);
        assert_eq!(r.gaps[0].details.as_ref().unwrap()["foundAmount"], "24.00");

        // Déjà rattachée.
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "25.00", "USD"), "sale", "IDA"));
        let mut s = sale(100, "Vente IDA", 1021, "USD", "25.00");
        s.linked = true;
        f.sales.push(s);
        let r = f.review(&SPEC);
        assert_eq!(codes(&r.gaps), [GAP_AMOUNT_OR_CURRENCY_MISMATCH]);
        assert_eq!(r.gaps[0].details.as_ref().unwrap()["alreadyLinked"], true);
    }

    #[test]
    fn deux_ventes_a_examiner() {
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "25.00", "USD"), "sale", "IDA"));
        f.sales.push(sale(100, "Vente IDA", 1021, "USD", "25.00"));
        f.sales.push(sale(101, "Autre IDA", 1021, "USD", "25.00"));
        let r = f.review(&SPEC);
        assert_eq!(codes(&r.to_review), [REVIEW_AMBIGUOUS_SALE]);
        assert_eq!(r.to_review[0].entries.len(), 2);
        assert!(r.proposals.is_empty());
    }

    #[test]
    fn vente_sans_encaissement_et_vente_encaissee_par_un_autre_compte() {
        let mut f = Fixture::new();
        f.sales.push(sale(100, "Vente IDA", 1021, "USD", "25.00"));
        f.sales.push(sale(101, "Vente IDB", 1021, "USD", "30.00"));
        f.sales
            .push(sale(102, "Sans identifiant", 1021, "USD", "30.00"));
        // IDB est encaissée par un autre compte bancaire du même compte du plan
        // (transaction rapprochée, hors de la revue).
        f.others
            .push(typed(tx(9, 99, "30.00", "USD"), "sale", "IDB"));
        let r = f.review(&SPEC);
        assert_eq!(codes(&r.gaps), [GAP_SALE_WITHOUT_RECEIPT]);
        assert_eq!(r.gaps[0].entries[0].journal_entry_id, 100);
        assert_eq!(r.gaps[0].details.as_ref().unwrap()["amount"], "25.00");
        // Une vente déjà rattachée n'est pas un écart.
        let mut f2 = Fixture::new();
        let mut s = sale(100, "Vente IDA", 1021, "USD", "25.00");
        s.linked = true;
        f2.sales.push(s);
        assert!(f2.review(&SPEC).gaps.is_empty());
    }

    #[test]
    fn la_cle_de_rattachement_est_au_parametrage() {
        let mut f = Fixture::new();
        let mut t = typed(tx(1, 10, "25.00", "USD"), "sale", "AUTRE");
        t.reference = Some("IDREF".into());
        f.pending.push(t);
        f.sales.push(sale(100, "Vente IDREF", 1021, "USD", "25.00"));
        // Par `transaction_id` : aucune vente, et la vente n'est encaissée par
        // aucune clé.
        assert_eq!(
            codes(&f.review(&SPEC).gaps),
            [GAP_SALE_WITHOUT_RECEIPT, GAP_RECEIPT_WITHOUT_SALE]
        );
        // Par `reference` : rattachée.
        let spec = ProviderSpec {
            match_key: reference_key,
            ..SPEC
        };
        let r = f.review(&spec);
        assert_eq!(r.proposals.len(), 1);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn frais_avec_et_sans_compte_negatifs_et_positifs() {
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "-1.20", "USD"), "fee", "IDA:FEE"));
        f.pending
            .push(typed(tx(2, 10, "0.50", "USD"), "fee", "IDB:FEE"));
        f.pending
            .push(typed(tx(3, 11, "-0.80", "CHF"), "fee", "IDC:FEE"));
        let r = f.review(&SPEC);
        assert_eq!(
            r.proposals
                .iter()
                .map(|p| p.proposal.clone())
                .collect::<Vec<_>>(),
            vec![
                Proposal::Fee {
                    bank_transaction_id: 1,
                    fee_account_id: 6940
                },
                Proposal::Fee {
                    bank_transaction_id: 2,
                    fee_account_id: 6940
                },
            ]
        );
        assert_eq!(codes(&r.to_review), [REVIEW_FEE_ACCOUNT_MISSING]);
    }

    fn conv(
        id: i64,
        account: i64,
        amount: &str,
        currency: &str,
        reference: &str,
    ) -> BankTransaction {
        let mut t = typed(tx(id, account, amount, currency), "conv", &format!("C{id}"));
        t.end_to_end_id = Some(reference.into());
        t
    }

    #[test]
    fn conversion_appariee_par_reference_et_orpheline() {
        let mut f = Fixture::new();
        f.pending.push(conv(1, 10, "-100.00", "USD", "REF1"));
        f.pending.push(conv(2, 11, "90.00", "CHF", "REF1"));
        f.pending.push(conv(3, 10, "-50.00", "USD", "REF2"));
        // Même devise : pas une paire.
        f.pending.push(conv(4, 10, "50.00", "USD", "REF3"));
        f.pending.push(conv(5, 10, "-50.00", "USD", "REF3"));
        // Sans référence.
        f.pending.push(conv(6, 10, "-10.00", "USD", ""));
        let r = f.review(&SPEC);
        assert_eq!(
            r.proposals
                .iter()
                .map(|p| p.proposal.clone())
                .collect::<Vec<_>>(),
            vec![Proposal::Conversion {
                debit_transaction_id: 1,
                credit_transaction_id: 2
            }]
        );
        assert_eq!(r.proposals[0].other_transaction.as_ref().unwrap().id, 2);
        assert_eq!(
            codes(&r.to_review),
            vec![REVIEW_CONVERSION_UNPAIRED; 4],
            "3, 4, 5 et 6 sans paire"
        );
        // La place est au paramétrage.
        let spec = ProviderSpec {
            conversion_unpaired: ("conversion_half_missing", Placement::Gap),
            ..SPEC
        };
        let r = f.review(&spec);
        assert_eq!(r.gaps.len(), 4);
        assert!(r.to_review.is_empty());
    }

    #[test]
    fn conversion_a_trois_moities_non_appariee() {
        let mut f = Fixture::new();
        f.pending.push(conv(1, 10, "-100.00", "USD", "REF1"));
        f.pending.push(conv(2, 11, "90.00", "CHF", "REF1"));
        f.pending.push(conv(3, 11, "91.00", "CHF", "REF1"));
        let r = f.review(&SPEC);
        assert!(r.proposals.is_empty());
        assert_eq!(r.to_review.len(), 3);
    }

    #[test]
    fn retrait_chf_et_en_devise() {
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 11, "-500.00", "CHF"), "wd", "W1"));
        f.pending
            .push(typed(tx(2, 10, "-500.00", "USD"), "wd", "W2"));
        let r = f.review(&SPEC);
        assert_eq!(
            r.proposals
                .iter()
                .map(|p| p.proposal.clone())
                .collect::<Vec<_>>(),
            vec![Proposal::Withdrawal {
                bank_transaction_id: 1,
                counterparty_account_id: Some(1010)
            }]
        );
        assert_eq!(codes(&r.to_review), [REVIEW_WITHDRAWAL_FOREIGN_CURRENCY]);
        // Sans compte principal : contrepartie à choisir.
        f.primary = None;
        let r = f.review(&SPEC);
        assert_eq!(
            r.proposals[0].proposal,
            Proposal::Withdrawal {
                bank_transaction_id: 1,
                counterparty_account_id: None
            }
        );
    }

    #[test]
    fn regles_filtres_sens_et_sans_regle() {
        let mut f = Fixture::new();
        let mut achat = typed(tx(1, 10, "-20.00", "USD"), "Express Checkout Payment", "A1");
        achat.counterparty_name = Some("Hostpoint AG".into());
        achat.counterparty_email = Some("billing@hostpoint.ch".into());
        f.pending.push(achat);
        let mut autre = typed(tx(2, 10, "-5.00", "USD"), "Express Checkout Payment", "A2");
        autre.counterparty_name = Some("Inconnu".into());
        f.pending.push(autre);

        // Règle filtrée par type, par e-mail, par sens : correspond.
        let mut r1 = rule(1, "hostpoint", 6500);
        r1.provider_type = Some("express checkout payment".into());
        r1.counterparty_email = Some("Billing@Hostpoint.ch".into());
        r1.direction = Some("debit".into());
        f.rules.push(r1);
        let r = f.review(&SPEC);
        assert_eq!(
            r.proposals[0].proposal,
            Proposal::Rule {
                bank_transaction_id: 1,
                rule_id: 1,
                counterparty_account_id: 6500
            }
        );
        assert_eq!(codes(&r.to_review), [REVIEW_NO_RULE]);

        // Sens contraire : ne correspond plus.
        f.rules[0].direction = Some("credit".into());
        assert_eq!(f.review(&SPEC).to_review.len(), 2);
        // Filtre de type différent.
        f.rules[0].direction = None;
        f.rules[0].provider_type = Some("Website Payment".into());
        assert_eq!(f.review(&SPEC).to_review.len(), 2);
        // Règle non filtrée.
        f.rules[0].provider_type = None;
        f.rules[0].counterparty_email = None;
        assert_eq!(f.review(&SPEC).proposals.len(), 1);
        // Contrepartie en devise : admise pour la devise de la transaction
        // (décision du responsable de l'epic du 2026-11-21), écartée pour toute autre.
        let devise = f.review(&SPEC).proposals.len();
        f.rule_accounts.insert(6500, Some("EUR".into()));
        assert!(f.review(&SPEC).proposals.is_empty());
        f.rule_accounts.insert(6500, Some("USD".into()));
        assert_eq!(f.review(&SPEC).proposals.len(), devise);
        f.rule_accounts.insert(6500, None);
        // Contrepartie hors de l'ensemble admis, ou compte du plan de la
        // transaction : écartée.
        f.rules[0].counterparty_account_id = 7777;
        assert!(f.review(&SPEC).proposals.is_empty());
        f.rule_accounts.insert(1021, None);
        f.rules[0].counterparty_account_id = 1021;
        assert!(f.review(&SPEC).proposals.is_empty());
    }

    /// Correctif #1538 : seule l'écriture qui **nomme** la transaction la
    /// retient ; deux transactions de même montant ne se confondent pas.
    #[test]
    fn premiere_ecriture_qui_nomme_la_transaction() {
        let a = typed(tx(1, 10, "-50.00", "CHF"), "wd", "IDA");
        let b = typed(tx(2, 10, "-50.00", "CHF"), "wd", "IDB");
        // La plus proche (901) nomme A ; la suivante (902) nomme B.
        let candidates = [(901, "Retrait IDA"), (902, "Retrait IDB")];
        assert_eq!(first_naming_entry(&SPEC, &a, candidates, &[]), Some(901));
        assert_eq!(first_naming_entry(&SPEC, &b, candidates, &[]), Some(902));
        // Une écriture qui ne nomme personne (saisie à la main), côté
        // prestataire libre, retient par prudence toute transaction de la
        // présélection ; une écriture qui nomme `t`, même plus loin, passe
        // avant elle.
        let sans_nom = (903, "Retrait sans identifiant");
        assert_eq!(first_naming_entry(&SPEC, &a, [sans_nom], &[]), Some(903));
        assert_eq!(first_naming_entry(&SPEC, &b, [sans_nom], &[]), Some(903));
        assert_eq!(
            first_naming_entry(&SPEC, &a, [sans_nom, (901, "Retrait IDA")], &[]),
            Some(901),
            "l'écriture qui nomme passe avant la plus proche sans nom"
        );
        // Côté prestataire encore rapproché (une transaction du même compte) :
        // l'écriture n'est celle d'aucune transaction en attente.
        let mut portee = typed(tx(9, 10, "-50.00", "CHF"), "wd", "IDZ");
        portee.matched_entry_id = Some(903);
        assert_eq!(
            first_naming_entry(&SPEC, &a, [sans_nom], &[portee.clone()]),
            None
        );
        // Une transaction d'un autre compte (l'autre côté) ne l'occupe pas.
        let mut autre_cote = tx(10, 99, "50.00", "CHF");
        autre_cote.matched_entry_id = Some(903);
        assert_eq!(
            first_naming_entry(&SPEC, &a, [sans_nom], &[autre_cote]),
            Some(903)
        );
        let c = typed(tx(3, 10, "-50.00", "CHF"), "wd", "IDC");
        assert_eq!(first_naming_entry(&SPEC, &c, candidates, &[]), None);
    }

    /// Correctif #1538 : une conversion en une écriture ne nomme que sa
    /// moitié débitée ; l'autre moitié est nommée par sa paire, rapprochée de
    /// l'écriture — jamais par une conversion étrangère de même montant.
    #[test]
    fn conversion_nommee_par_sa_paire_rapprochee() {
        let mut debit = typed(tx(1, 11, "-90.00", "CHF"), "conv", "IDD");
        debit.end_to_end_id = Some("REF1".into());
        debit.matched_entry_id = Some(901);
        let mut credit = typed(tx(2, 10, "100.00", "USD"), "conv", "IDC");
        credit.end_to_end_id = Some("REF1".into());
        let mut autre = typed(tx(3, 10, "100.00", "USD"), "conv", "IDX");
        autre.end_to_end_id = Some("REF2".into());
        let linked = [debit.clone()];
        assert_eq!(
            first_naming_entry(&SPEC, &credit, [(901, "Conversion IDD")], &linked),
            Some(901)
        );
        assert_eq!(
            first_naming_entry(&SPEC, &autre, [(901, "Conversion IDD")], &linked),
            None,
            "une autre conversion de même montant"
        );
        // La paire doit être rapprochée de CETTE écriture.
        assert_eq!(
            first_naming_entry(&SPEC, &credit, [(902, "Conversion IDD")], &linked),
            None
        );
        // Une transaction qui n'est pas une conversion n'est pas nommée par une paire.
        let mut retrait = typed(tx(4, 10, "100.00", "USD"), "wd", "IDW");
        retrait.end_to_end_id = Some("REF1".into());
        assert_eq!(
            first_naming_entry(&SPEC, &retrait, [(901, "Conversion IDD")], &linked),
            None
        );
    }

    /// Correctif « écrite une seule fois » : une transaction déjà
    /// comptabilisée de l'autre côté n'est proposée à aucun genre qui écrit
    /// (frais, retrait, règle, conversion) ; elle est à examiner, l'écriture
    /// nommée. La sonde positive : sans entrée dans `already_posted`, les
    /// mêmes transactions sont proposées. Un encaissement n'est pas touché (sa
    /// proposition rattache, elle n'écrit pas), ni un genre propre.
    #[test]
    fn deja_comptabilisee_a_examiner_pour_les_genres_qui_ecrivent() {
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "-1.20", "USD"), "fee", "IDA:FEE"));
        f.pending
            .push(typed(tx(2, 11, "-500.00", "CHF"), "wd", "W1"));
        let mut achat = typed(tx(3, 10, "-20.00", "USD"), "Express Checkout Payment", "A1");
        achat.counterparty_name = Some("Hostpoint AG".into());
        f.pending.push(achat);
        f.rules.push(rule(1, "hostpoint", 6500));
        f.pending.push(conv(4, 10, "-100.00", "USD", "REF1"));
        f.pending.push(conv(5, 11, "90.00", "CHF", "REF1"));
        f.pending
            .push(typed(tx(6, 10, "30.00", "USD"), "sale", "IDS"));
        f.sales.push(sale(50, "Vente IDS", 1021, "USD", "30.00"));
        f.pending
            .push(typed(tx(7, 10, "-3.00", "USD"), "spec", "P1"));
        // Sans compte de frais (compte 11) et sans règle : leur motif propre, sauf
        // déjà comptabilisées (revue P1 du correctif, M1).
        f.pending
            .push(typed(tx(8, 11, "-0.80", "CHF"), "fee", "IDF:FEE"));
        let mut inconnu = typed(tx(9, 10, "-7.00", "USD"), "Express Checkout Payment", "A9");
        inconnu.counterparty_name = Some("Inconnu".into());
        f.pending.push(inconnu);
        // Sonde positive : rien de déjà comptabilisé, tout est proposé.
        let r = f.review(&SPEC);
        let kinds: Vec<&str> = r.proposals.iter().map(|p| p.proposal.kind()).collect();
        assert_eq!(
            kinds,
            ["fee", "withdrawal", "rule", "conversion", "link_entry"],
            "{kinds:?}"
        );
        assert_eq!(
            codes(&r.to_review),
            [REVIEW_FEE_ACCOUNT_MISSING, REVIEW_NO_RULE],
            "{:?}",
            r.to_review
        );
        // Toutes déjà comptabilisées (la conversion par sa seule moitié débitée).
        for (t, e) in [
            (1, 901),
            (2, 902),
            (3, 903),
            (4, 904),
            (6, 906),
            (7, 907),
            (8, 908),
            (9, 909),
        ] {
            f.posted.insert(t, e);
        }
        let r = f.review(&SPEC);
        assert_eq!(
            r.proposals
                .iter()
                .map(|p| p.proposal.kind())
                .collect::<Vec<_>>(),
            ["link_entry"],
            "l'encaissement se rattache toujours"
        );
        assert_eq!(
            r.specific,
            vec![(7, "special")],
            "un genre propre reste au prestataire"
        );
        let deja: Vec<(i64, serde_json::Value)> = r
            .to_review
            .iter()
            .filter(|f| f.code == REVIEW_ALREADY_POSTED)
            .map(|f| {
                (
                    f.transaction.as_ref().unwrap().id,
                    f.details.clone().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            deja,
            vec![
                (1, serde_json::json!({ "journalEntryId": 901 })),
                (2, serde_json::json!({ "journalEntryId": 902 })),
                (3, serde_json::json!({ "journalEntryId": 903 })),
                (4, serde_json::json!({ "journalEntryId": 904 })),
                (8, serde_json::json!({ "journalEntryId": 908 })),
                (9, serde_json::json!({ "journalEntryId": 909 })),
            ]
        );
        assert!(
            !r.to_review
                .iter()
                .any(|f| f.code == REVIEW_FEE_ACCOUNT_MISSING || f.code == REVIEW_NO_RULE),
            "le motif « déjà comptabilisée » passe avant : {:?}",
            r.to_review
        );
        // La moitié créditée, privée de sa paire déjà comptabilisée : sans paire.
        assert!(
            r.to_review
                .iter()
                .any(|f| f.code == REVIEW_CONVERSION_UNPAIRED
                    && f.transaction.as_ref().unwrap().id == 5),
            "{:?}",
            r.to_review
        );
    }

    #[test]
    fn genre_propre_laisse_au_prestataire_et_ordre() {
        let mut f = Fixture::new();
        let mut late = typed(tx(5, 10, "-1.00", "USD"), "fee", "X:FEE");
        late.booking_date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
        f.pending.push(late);
        f.pending
            .push(typed(tx(7, 10, "-1.00", "USD"), "fee", "Y:FEE"));
        f.pending
            .push(typed(tx(6, 10, "-1.00", "USD"), "fee", "Z:FEE"));
        f.pending.push(typed(tx(8, 10, "3.00", "USD"), "spec", "S"));
        let r = f.review(&SPEC);
        assert_eq!(r.specific, vec![(8, "special")]);
        let ids: Vec<i64> = r.proposals.iter().map(|p| p.transaction.id).collect();
        assert_eq!(ids, [6, 7, 5], "date, puis identifiant");
        let c = counts(&r);
        assert_eq!(c["proposals"], 3);
    }

    #[test]
    fn proposition_serialisee_et_relue() {
        let p = Proposal::Conversion {
            debit_transaction_id: 1,
            credit_transaction_id: 2,
        };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "type": "conversion", "debitTransactionId": 1, "creditTransactionId": 2 })
        );
        let back: Proposal = serde_json::from_value(serde_json::json!({
            "type": "withdrawal", "bankTransactionId": 3, "counterpartyAccountId": 4
        }))
        .unwrap();
        assert_eq!(
            back,
            Proposal::Withdrawal {
                bank_transaction_id: 3,
                counterparty_account_id: Some(4)
            }
        );
        assert_eq!(p.transaction_ids(), [1, 2]);
        assert_eq!(p.primary_transaction_id(), 1);
        assert_eq!(p.kind(), "conversion");
        let review = ReviewProposal {
            proposal: Proposal::Fee {
                bank_transaction_id: 1,
                fee_account_id: 2,
            },
            transaction: TransactionSummary::of(&tx(1, 10, "-1.2", "USD")),
            other_transaction: None,
            entry: None,
        };
        let json = serde_json::to_value(&review).unwrap();
        assert_eq!(json["type"], "fee");
        assert_eq!(json["feeAccountId"], 2);
        assert_eq!(json["transaction"]["amount"], "-1.20");
    }

    /// M2 (revue P1) : `link_entry` et `rule` admis selon le paramétrage du
    /// prestataire — encaissement et `Other` toujours, un genre propre ou
    /// `Other` en rattachement seulement si le prestataire le déclare.
    #[test]
    fn genres_admis_au_rattachement_et_a_la_regle_selon_le_parametrage() {
        let specific = ProviderKind::Specific("special");
        // PayPal (aucun genre en plus).
        assert!(admits_link(&SPEC, ProviderKind::SaleReceipt));
        for k in [
            ProviderKind::Other,
            specific,
            ProviderKind::Fee,
            ProviderKind::Conversion,
            ProviderKind::Withdrawal,
        ] {
            assert!(!admits_link(&SPEC, k), "{k:?}");
        }
        assert!(admits_rule(&SPEC, ProviderKind::Other));
        for k in [
            ProviderKind::SaleReceipt,
            specific,
            ProviderKind::Fee,
            ProviderKind::Conversion,
            ProviderKind::Withdrawal,
        ] {
            assert!(!admits_rule(&SPEC, k), "{k:?}");
        }
        // Un prestataire (Wise, 39-11c AC-4 et AC-5) qui rattache ou règle
        // `Other` et son genre propre selon les candidates.
        fn other_or_specific(k: ProviderKind) -> bool {
            matches!(k, ProviderKind::Other | ProviderKind::Specific(_))
        }
        let wise_like = ProviderSpec {
            linkable: other_or_specific,
            ruleable: other_or_specific,
            ..SPEC
        };
        for k in [ProviderKind::SaleReceipt, ProviderKind::Other, specific] {
            assert!(admits_link(&wise_like, k), "{k:?}");
        }
        for k in [ProviderKind::Other, specific] {
            assert!(admits_rule(&wise_like, k), "{k:?}");
        }
        assert!(!admits_link(&wise_like, ProviderKind::Fee));
        assert!(!admits_rule(&wise_like, ProviderKind::SaleReceipt));
    }

    /// M3 (revue P1) : une revue porte une proposition propre au prestataire
    /// (forme du `payout` de Stripe, 40-7c AC-4 d), servie à côté des genres
    /// communs, triée et comptée comme eux.
    #[test]
    fn revue_avec_une_proposition_propre_au_prestataire() {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
        #[serde(
            tag = "type",
            rename_all = "snake_case",
            rename_all_fields = "camelCase"
        )]
        enum Own {
            Payout {
                bank_transaction_id: i64,
                payout_bank_account_id: i64,
                payout_account_label: String,
            },
        }
        #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
        #[serde(untagged)]
        enum WithPayout {
            Common(Proposal),
            Own(Own),
        }
        let mut f = Fixture::new();
        f.pending
            .push(typed(tx(1, 10, "-1.20", "USD"), "fee", "IDA:FEE"));
        let payout = tx(2, 10, "-50.00", "USD");
        f.pending.push(typed(payout.clone(), "spec", "PO1"));
        let common = f.review(&SPEC);
        assert_eq!(common.specific, [(2, "special")]);
        let mut review: Review<WithPayout> = common.map_proposals(WithPayout::Common);
        review.push_proposal(ReviewProposal {
            proposal: WithPayout::Own(Own::Payout {
                bank_transaction_id: 2,
                payout_bank_account_id: 77,
                payout_account_label: "Banque".into(),
            }),
            transaction: TransactionSummary::of(&payout),
            other_transaction: None,
            entry: None,
        });
        review.sort();
        assert_eq!(counts(&review)["proposals"], 2);
        let json = serde_json::to_value(&review.proposals).unwrap();
        assert_eq!(json[0]["type"], "fee");
        assert_eq!(json[0]["feeAccountId"], 6940);
        assert_eq!(json[1]["type"], "payout");
        assert_eq!(json[1]["bankTransactionId"], 2);
        assert_eq!(json[1]["payoutBankAccountId"], 77);
        assert_eq!(json[1]["payoutAccountLabel"], "Banque");
        assert_eq!(json[1]["transaction"]["amount"], "-50.00");
    }
}
