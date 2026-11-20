//! Un règlement pour plusieurs factures — Story 43-1a (#823), Epic 43.
//!
//! Une transaction bancaire règle **plusieurs factures** d'un seul geste : **une**
//! écriture — une ligne de banque, une ligne de tiers **par facture** (le projet
//! de sa facture), au plus une ligne d'arrondi et une ligne d'écart au compte
//! choisi par l'utilisateur —, que la
//! transaction pointe (`bank_transactions.matched_entry_id`, C-43-1). Chaque
//! facture sait quelle ligne est la sienne (`invoice_settlements.journal_entry_line_id`,
//! `supplier_invoices.settlement_journal_entry_line_id`, C-43-2) : c'est ce qui
//! permet de lettrer chaque facture avec sa seule ligne de créance.
//!
//! ⛔ **Un groupe ne se défait que par le dé-rapprochement de sa transaction, en
//! entier** ([`cancel_in_tx`], C-43-6). Les gestes unitaires sur une facture du
//! groupe butent sur les gardes existantes (`MATCHED_BANK_TRANSACTION`, facture
//! payée).
//!
//! **Le côté client** est ici ([`settle_invoices_in_tx`]) ; le côté fournisseur
//! ([`pay_supplier_invoices_in_tx`]) est la Story 43-1b ; l'API, la 43-2.
//!
//! **La différence** (décision de Guy du 2026-11-17 : écart admis dans les deux
//! sens, au compte que l'utilisateur choisit, refusé au-delà du plafond seul)
//! est calculée par une seule fonction pure, [`group_difference`], appelée
//! par les deux familles : un changement de la règle ne change qu'elle, ses tests,
//! les variantes de refus et les libellés.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::entities::audit_log::{Acteur, NewAuditLogEntry};
use crate::entities::{
    Invoice, NewInvoiceSettlement, NewJournalEntry, NewJournalEntryLine, SupplierInvoice,
};
use crate::errors::{ClaimSide, DbError, SettlementAccountRole, map_db_error};
use crate::repositories::invoice_settlements::amount_due_to_centime;

/// Le nombre minimal de factures d'un groupe (C-43-7).
pub const GROUP_MIN_INVOICES: usize = 2;
/// Le nombre maximal de factures d'un groupe (C-43-7).
pub const GROUP_MAX_INVOICES: usize = 50;

/// Le nom **constant** du point de sauvegarde du dépôt (D6). Il s'emboîte dans
/// celui de la route (`sp_<transaction>`, `accept_batch`).
const SAVEPOINT: &str = "grouped_settlement";

// ---------------------------------------------------------------------------
// Types publics — le contrat consommé par les 43-1b, 43-2 et 43-3 (T3)
// ---------------------------------------------------------------------------

/// La famille d'un groupe : le sens de la transaction la choisit (C-43-15) — un
/// crédit règle des factures **clients**, un débit des factures **fournisseur**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupFamily {
    /// Factures clients (`invoices`), transaction au crédit.
    Client,
    /// Factures fournisseur (`supplier_invoices`), transaction au débit.
    Supplier,
}

impl GroupFamily {
    /// Le code exposé (`kind` des listes de la vue, 43-2).
    pub fn code(self) -> &'static str {
        match self {
            Self::Client => "invoice",
            Self::Supplier => "supplier_invoice",
        }
    }
}

/// Une pièce en cause dans un refus, ou membre d'un groupe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupedDocument {
    /// Une facture client.
    Invoice(i64),
    /// Une facture fournisseur.
    SupplierInvoice(i64),
}

impl GroupedDocument {
    /// L'identifiant de la pièce.
    pub fn id(self) -> i64 {
        match self {
            Self::Invoice(id) | Self::SupplierInvoice(id) => id,
        }
    }

    /// Sa famille.
    pub fn family(self) -> GroupFamily {
        match self {
            Self::Invoice(_) => GroupFamily::Client,
            Self::SupplierInvoice(_) => GroupFamily::Supplier,
        }
    }

    fn of(family: GroupFamily, id: i64) -> Self {
        match family {
            GroupFamily::Client => Self::Invoice(id),
            GroupFamily::Supplier => Self::SupplierInvoice(id),
        }
    }
}

/// La pièce d'un refus, avec son **numéro affichable** (client :
/// `invoice_number` ; fournisseur : numéro interne, à défaut celui du
/// fournisseur) — pour que l'écran la nomme (43-3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedPiece {
    pub document: GroupedDocument,
    pub number: Option<String>,
}

/// La raison d'une facture non admise (`details.reason` de
/// `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, table de la 43-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IneligibleReason {
    /// Client : la facture n'est pas `validated` (`invoice_not_validated`).
    NotValidated,
    /// Client : déjà payée (`invoice_already_paid`).
    AlreadyPaid,
    /// Client : sans écriture de vente (`invoice_journal_entry_not_set`).
    JournalEntryNotSet,
    /// Client : reste dû nul ou négatif (`nothing_due`).
    NothingDue,
    /// Fournisseur : pas `open` (`supplier_invoice_not_open`) — 43-1b.
    SupplierNotOpen,
    /// Fournisseur : dans un lot `generated` (`supplier_invoice_in_payment_batch`) — 43-1b.
    SupplierInPaymentBatch,
    /// La pièce a changé depuis sa lecture (`race_during_update`) : son
    /// `UPDATE … AND version = ?` n'a touché aucune ligne (C-43-19).
    RaceDuringUpdate,
}

impl IneligibleReason {
    /// Le code exposé.
    pub fn code(self) -> &'static str {
        match self {
            Self::NotValidated => "invoice_not_validated",
            Self::AlreadyPaid => "invoice_already_paid",
            Self::JournalEntryNotSet => "invoice_journal_entry_not_set",
            Self::NothingDue => "nothing_due",
            Self::SupplierNotOpen => "supplier_invoice_not_open",
            Self::SupplierInPaymentBatch => "supplier_invoice_in_payment_batch",
            Self::RaceDuringUpdate => "race_during_update",
        }
    }
}

/// Un refus d'un règlement groupé — **une variante par refus nommé** ; aucune
/// n'est une erreur de base. La route (43-2) traduit chacune en `FailedProposal`
/// par `group_refusal_to_failed_proposal`, `match` exhaustif sans joker.
///
/// ⛔ Tout autre problème reste un `Err(DbError)` — en particulier les erreurs
/// SQL (1213, 1305) et celles du **lettrage**, jamais traduites en refus (D6).
#[derive(Debug, Clone, PartialEq)]
pub enum GroupRefusal {
    /// Moins de [`GROUP_MIN_INVOICES`] factures.
    TooFew { count: usize },
    /// Plus de [`GROUP_MAX_INVOICES`] factures.
    TooMany { count: usize },
    /// La même facture deux fois.
    Duplicate { document: GroupedDocument },
    /// La transaction n'existe pas pour ce compte bancaire.
    TransactionNotFound,
    /// La transaction n'est plus `pending` (lue, ou à son `UPDATE`).
    TransactionNotPending,
    /// La transaction est dans le sens contraire de la famille, ou de montant
    /// nul (route : `refuse_wrong_direction`). Contrôlé **avant** la devise.
    WrongDirection { amount: Decimal },
    /// La transaction n'est pas en CHF (`RECONCILIATION_CURRENCY_MISMATCH`, C-43-22).
    CurrencyMismatch { currency: String },
    /// Facture introuvable, ou d'une autre société.
    InvoiceNotFound { document: GroupedDocument },
    /// Facture non admise ; sa raison.
    InvoiceNotEligible {
        piece: GroupedPiece,
        reason: IneligibleReason,
    },
    /// Fournisseur (43-1b) : la facture est tenue par un lot du compte.
    HeldByPaymentBatch {
        piece: GroupedPiece,
        payment_batch_id: Option<i64>,
    },
    /// L'écriture de vente n'a pas de ligne au débit (`INVOICE_SALE_ENTRY_MALFORMED`).
    SaleEntryMalformed {
        piece: GroupedPiece,
        sale_entry_id: i64,
    },
    /// Le compte bancaire n'est lié à aucun compte du grand livre.
    BankAccountNotConfigured,
    /// Le compte bancaire est introuvable ou archivé.
    BankAccountNotFound,
    /// Le compte bancaire est clos.
    BankAccountClosed,
    /// Un compte que l'écriture écrirait coïncide avec un compte de tiers
    /// (`D X / C X`) — `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, avec les
    /// rôles réels de [`SettlementAccountRole`] : `Counterparty` (le compte de
    /// banque, ou le compte choisi pour l'écart, égal à une créance ou à une
    /// dette), `Rounding` (le compte de différences d'arrondi). `claim` : le
    /// côté soldé.
    ///
    /// Deux causes pour `Counterparty`, que les champs distinguent :
    /// le compte de créance d'une facture (`piece`) égal au compte de banque
    /// (`bank_account_id` renseigné, `account_id` = ce compte) ; le compte
    /// choisi pour l'écart égal à une créance (`bank_account_id` vide,
    /// `account_id` = le compte choisi).
    ClaimAccount {
        claim: ClaimSide,
        role: SettlementAccountRole,
        /// Le compte en cause.
        account_id: i64,
        account_number: Option<String>,
        /// Le compte bancaire de la transaction, quand c'est son compte du
        /// grand livre qui coïncide avec une créance.
        bank_account_id: Option<i64>,
        piece: Option<GroupedPiece>,
    },
    /// Le paiement précède la facture au-delà de la tolérance (C-43-7, la borne
    /// basse de l'unitaire ; la borne haute n'est pas appliquée).
    PaymentBeforeInvoiceDate {
        piece: GroupedPiece,
        tolerance_days: i64,
    },
    /// `|e|` dépasse le plafond de la société (C-43-24) — **le seul** refus de
    /// la différence (décision de Guy du 2026-11-17) : moins-payé et
    /// trop-perçu sont admis dans les deux familles.
    DifferenceTooLarge {
        transaction_amount: Decimal,
        sum_applied: Decimal,
        /// L'écart `e`, signé.
        gap: Decimal,
        max_difference: Decimal,
    },
    /// Un écart `e ≠ 0` sans compte choisi par l'utilisateur
    /// (`difference_account_id` absent).
    DifferenceAccountRequired {
        /// L'écart `e`, signé.
        gap: Decimal,
    },
    /// Le compte choisi pour l'écart est le compte du grand livre de la
    /// banque de la transaction. Contrôle **1** du compte choisi (convention de
    /// la contrepartie choisie : banque, manquant, non imputable, tiers).
    DifferenceAccountIsBankLedger { account_id: i64 },
    /// Contrôle **2** : le compte choisi est introuvable, d'une autre société,
    /// ou archivé (route : `ACCOUNT_NOT_FOUND`, `missingAccountIds`).
    DifferenceAccountInvalid { account_id: i64 },
    /// Contrôle **3** : le compte choisi n'est pas imputable (route :
    /// `non_postable_failed_proposal`, qui nomme le numéro).
    DifferenceAccountNotPostable {
        account_id: i64,
        account_number: String,
    },
    /// Contrôle **3 bis** (reprise de `main` après l'Epic 37, revue
    /// d'intégration M2) : le compte choisi est tenu **en devise** — le
    /// règlement groupé s'écrit en CHF. Route (43-2) :
    /// `ACCOUNT_IN_FOREIGN_CURRENCY`, `details: { accountId, currency }`, le
    /// code du règlement unitaire (`invoice_settlements_write`, Story 37-3a,
    /// AC-5). Sans ce refus, la ligne CHF tomberait au contrôle central de
    /// l'écriture (`MULTI_CURRENCY_DISABLED` ou `ACCOUNT_REQUIRES_CURRENCY`),
    /// une erreur de base au lieu d'un refus nommé.
    DifferenceAccountInForeignCurrency { account_id: i64, currency: String },
    /// Une fraction de centime à passer sans compte de différences d'arrondi.
    RoundingAccountNotConfigured,
    /// La date de l'écriture est dans une période verrouillée.
    PeriodLocked {
        locked_through: NaiveDate,
        attempted: NaiveDate,
    },
    /// Un exercice postérieur est clos.
    LaterFiscalYearClosed {
        fiscal_year_id: i64,
        fiscal_year_name: String,
    },
    /// Aucun exercice ouvert ne couvre la date de valeur.
    FiscalYearInvalid { value_date: NaiveDate },
}

/// La décomposition de la différence d'un groupe (D4 bis) — **une seule
/// définition pour les deux familles**.
///
/// - `difference = d = |montant| − Σ apports bruts` (signé : négatif quand moins
///   reçu ou payé, positif quand plus ; brut, quatre décimales) ;
/// - `gap = e = centime(d)` (signé, au centime, demi-centime loin de zéro) — le
///   moins-payé (`e < 0`) ou le trop-perçu (`e > 0`), passé au compte que
///   **l'utilisateur choisit** (décision de Guy du 2026-11-17) ;
/// - `rounding_difference = r = d − e` (signé, `|r| ≤ 0.005`) — la fraction
///   sous le centime, au compte d'arrondi comme dans l'unitaire.
///
/// Identité : `difference = rounding_difference + gap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decomposition {
    pub difference: Decimal,
    pub rounding_difference: Decimal,
    pub gap: Decimal,
}

/// **La règle de la différence** (C-43-16, D4 bis remplacée par la décision de
/// Guy du 2026-11-17) — pure, sans base, **la même pour les deux familles**.
///
/// `amount` : le montant de la transaction (signé, deux décimales) ;
/// `gross_contributions` : l'apport **brut** de chaque facture (client : son
/// reste dû ; fournisseur : son `total_amount`) ; `max_difference` : le plafond
/// de la société.
///
/// **Décomposition définitive** (`kesh-gate-logs/e43/decision-difference.md`,
/// § « Décomposition de l'écart ») : `d = |amount| − Σ apports bruts` (quatre
/// décimales) ; `e = centime(d)`, demi-centime arrondi en s'éloignant de zéro
/// (comme [`amount_due_to_centime`]) ; `r = d − e`, toujours `|r| ≤ 0.005`.
/// `r` va au compte d'arrondi, `e` au compte choisi. L'**arrondi automatique**
/// est le cas `e = 0` (`|d| < 0.005`) : aucun compte choisi. L'écart `e` est
/// admis **dans les deux sens** ; seul `|e| >` plafond est refusé
/// ([`GroupRefusal::DifferenceTooLarge`]) ; `r` n'y est jamais soumis.
///
/// Guy : « le trop perçu ou le moins doit pouvoir être comptabilisé à la main :
/// un moins peut être une perte sur débiteur ou des frais bancaires : on ne sait
/// pas a priori. Un trop perçu peut être un arrondi vers le haut de la part du
/// client » (`kesh-gate-logs/e43/decision-difference.md`).
pub fn group_difference(
    amount: Decimal,
    gross_contributions: &[Decimal],
    max_difference: Decimal,
) -> Result<Decomposition, GroupRefusal> {
    let paid = amount.abs();
    let gross: Decimal = gross_contributions.iter().copied().sum();
    let d = paid - gross;
    let e = amount_due_to_centime(d);
    let r = d - e;
    if e.abs() > max_difference {
        return Err(GroupRefusal::DifferenceTooLarge {
            transaction_amount: amount,
            sum_applied: gross,
            gap: e,
            max_difference,
        });
    }
    Ok(Decomposition {
        difference: d,
        rounding_difference: r,
        gap: e,
    })
}

/// Le compte choisi pour l'écart, **contrôlé** (décision de Guy du
/// 2026-11-17) : de la société et actif (sinon
/// [`GroupRefusal::DifferenceAccountInvalid`]), imputable (sinon
/// [`GroupRefusal::DifferenceAccountNotPostable`]), en CHF (sinon
/// [`GroupRefusal::DifferenceAccountInForeignCurrency`] — le règlement groupé
/// s'écrit en CHF ; même ordre que le règlement unitaire de l'Epic 37 :
/// invalide, non imputable, en devise) — sans restriction de classe —,
/// verrouillé `FOR UPDATE` comme un compte désigné (une désactivation, ou un
/// changement de devise, concurrents attendent la fin du geste). `None` : le
/// compte convient. La comparaison aux comptes de tiers et à la banque est
/// faite par l'appelant, qui les connaît.
pub(crate) async fn difference_account_refusal(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<GroupRefusal>, DbError> {
    let row: Option<(bool, bool, String, Option<String>)> = sqlx::query_as(
        "SELECT active, postable, number, currency FROM accounts \
         WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    Ok(match row {
        None | Some((false, _, _, _)) => {
            Some(GroupRefusal::DifferenceAccountInvalid { account_id })
        }
        Some((true, false, account_number, _)) => {
            Some(GroupRefusal::DifferenceAccountNotPostable {
                account_id,
                account_number,
            })
        }
        Some((true, true, _, Some(currency))) => {
            Some(GroupRefusal::DifferenceAccountInForeignCurrency {
                account_id,
                currency,
            })
        }
        Some((true, true, _, None)) => None,
    })
}

/// Une facture d'un groupe, telle que le règlement l'a soldée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedInvoiceRef {
    pub kind: GroupFamily,
    pub id: i64,
    /// Le numéro affichable.
    pub number: Option<String>,
    /// La ligne `invoice_settlements` ; `None` côté fournisseur (la facture
    /// **est** la ligne de règlement).
    pub settlement_id: Option<i64>,
    /// La ligne de tiers de la facture dans l'écriture.
    pub line_id: i64,
    /// Le montant soldé, **brut** (quatre décimales).
    pub amount: Decimal,
}

/// Ce que rend un règlement groupé réussi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedSettlement {
    /// L'écriture du règlement.
    pub journal_entry_id: i64,
    /// L'entrée `reconciliation.accepted` (les `invoice.paid` la citent).
    pub reconciliation_audit_id: i64,
    /// La différence décomposée (D4 bis, décision de Guy du 2026-11-17).
    pub decomposition: Decomposition,
    /// Le compte de l'écart `gap`, choisi par l'utilisateur ; `None` sans
    /// écart.
    pub difference_account_id: Option<i64>,
    /// Les factures soldées, par identifiant croissant.
    pub invoices: Vec<GroupedInvoiceRef>,
}

// ---------------------------------------------------------------------------
// Le point de sauvegarde (D6)
// ---------------------------------------------------------------------------

async fn savepoint(tx: &mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<(), DbError> {
    sqlx::query(&format!("SAVEPOINT {SAVEPOINT}"))
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    Ok(())
}

/// Revient au point de sauvegarde **avant** de rendre un refus : un refus ne
/// laisse rien dans la transaction de l'appelant. Un échec (1305 : la
/// transaction a été annulée sous le geste, interblocage absorbé plus bas)
/// remonte en `Err`, jamais suivi du refus.
async fn refuse<T>(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    refusal: GroupRefusal,
) -> Result<Result<T, GroupRefusal>, DbError> {
    sqlx::query(&format!("ROLLBACK TO SAVEPOINT {SAVEPOINT}"))
        .execute(&mut **tx)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(Err(refusal))
}

/// Ce que la règle des bornes et du doublon refuse, **avant toute lecture**
/// (étape 2) — la seule définition : la route n'en fait aucune copie.
fn check_bounds(family: GroupFamily, ids: &[i64]) -> Option<GroupRefusal> {
    if ids.len() < GROUP_MIN_INVOICES {
        return Some(GroupRefusal::TooFew { count: ids.len() });
    }
    if ids.len() > GROUP_MAX_INVOICES {
        return Some(GroupRefusal::TooMany { count: ids.len() });
    }
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(*id) {
            return Some(GroupRefusal::Duplicate {
                document: GroupedDocument::of(family, *id),
            });
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Le règlement groupé client (AC5)
// ---------------------------------------------------------------------------

/// Une facture lue à l'étape 4, ce que la suite en emploie.
struct ClientMember {
    invoice: Invoice,
    raw_due: Decimal,
    receivable_account_id: i64,
}

/// **Règle plusieurs factures clients par une transaction au crédit**, dans la
/// transaction de l'appelant et sous son verrou de compte (Story 43-1a, AC5).
///
/// Par la méthode de `accept_one_invoice` (`routes/reconciliation.rs`), **à deux
/// écarts près** (C-43-19) : (1) les factures sont mises à jour (étape 9)
/// **avant** la transaction (étape 10), pour qu'un refus d'une facture tombe
/// avant toute écriture sur la transaction — sans effet sur le résultat (même
/// transaction, verrou de compte, point de sauvegarde) ; (2) ni score ni fenêtre
/// haute de 30 jours (C-43-7).
///
/// ⛔ **Aucun `FOR UPDATE` sur les factures en amont** : chaque facture, sa
/// version et son reste dû sont lus sur l'**instantané** ; c'est
/// l'`UPDATE invoices … AND version = <lue>` de l'étape 9 qui interdit de régler
/// deux fois (Story 25-4-c2). Un `FOR UPDATE` lirait la version à jour et
/// désarmerait le contrôle.
///
/// **Point de sauvegarde (D6)** : `SAVEPOINT grouped_settlement` en tête ; tout
/// `Ok(Err(refus))` est rendu après `ROLLBACK TO SAVEPOINT grouped_settlement` —
/// il s'emboîte dans celui de `accept_batch`. Un `Err(DbError)` laisse
/// l'appelant revenir à **son** point de sauvegarde. Un `Err(DbError::Sqlx)`
/// (1213, 1305) n'est **jamais** traduit en refus : il remonte, et la route
/// rejoue (`accept_batch`, `is_savepoint_lost`). Les erreurs du lettrage
/// restent des `Err`, comme dans l'unitaire.
///
/// `invoice_ids` dans l'ordre reçu ; le traitement va par identifiant
/// croissant. `difference_account_id` : le compte que l'utilisateur choisit
/// pour l'écart `e` (moins-payé ou trop-perçu) — obligatoire dès que `e ≠ 0`,
/// ignoré sinon (décision de Guy du 2026-11-17). L'audit
/// (`reconciliation.accepted` puis `invoice.paid` par facture) est écrit ici,
/// avec l'acteur (D5) ; la route n'écrit rien.
///
/// `aujourd_hui` n'est **pas lu** (revue P1, B-4) : l'écriture est datée de la
/// date de valeur de la transaction, comme l'unitaire. Il reste dans la
/// signature figée à T3 (contrat de la 43-2) pour la symétrie avec
/// [`cancel_in_tx`] et la 43-1b, sans effet.
///
/// Au succès, le point de sauvegarde est **relâché** (`RELEASE SAVEPOINT`,
/// revue P1, B-3) : il ne survit pas dans la transaction de l'appelant. Après
/// un **refus**, il subsiste (`ROLLBACK TO` le garde ; revue P2, LOW-7) : sans
/// effet — le suivant, de même nom, le remplace, et un `ROLLBACK TO` du point
/// de l'appelant efface ceux posés après lui.
#[allow(clippy::too_many_arguments)]
pub async fn settle_invoices_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    bank_transaction_id: i64,
    invoice_ids: &[i64],
    difference_account_id: Option<i64>,
    batch_size: i64,
    acteur: Acteur,
    aujourd_hui: NaiveDate,
) -> Result<Result<GroupedSettlement, GroupRefusal>, DbError> {
    let _ = aujourd_hui;
    // (1) Le point de sauvegarde propre.
    savepoint(tx).await?;
    // (2) Bornes et doublon, avant toute lecture.
    if let Some(r) = check_bounds(GroupFamily::Client, invoice_ids) {
        return refuse(tx, r).await;
    }
    match settle_client_inner(
        tx,
        company_id,
        bank_account_id,
        bank_transaction_id,
        invoice_ids,
        difference_account_id,
        batch_size,
        acteur,
    )
    .await?
    {
        Ok(done) => {
            sqlx::query(&format!("RELEASE SAVEPOINT {SAVEPOINT}"))
                .execute(&mut **tx)
                .await
                .map_err(DbError::Sqlx)?;
            Ok(Ok(done))
        }
        Err(r) => refuse(tx, r).await,
    }
}

/// Le corps de [`settle_invoices_in_tx`], étapes 3 à 11 : rend le refus sans
/// revenir au point de sauvegarde (l'enveloppe s'en charge).
#[allow(clippy::too_many_arguments)]
async fn settle_client_inner(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    bank_transaction_id: i64,
    invoice_ids: &[i64],
    difference_account_id: Option<i64>,
    batch_size: i64,
    acteur: Acteur,
) -> Result<Result<GroupedSettlement, GroupRefusal>, DbError> {
    use super::reconciliation::{
        InvoiceDateRefusal, InvoiceDateWindow, find_invoice_by_id_for_company,
        find_pending_by_id_for_account, payment_date,
    };
    use crate::entities::bank_transaction::BankTransactionStatus;

    // (3) La transaction, rechargée sans verrou (patron de l'unitaire).
    let Some(bt) =
        find_pending_by_id_for_account(&mut **tx, company_id, bank_account_id, bank_transaction_id)
            .await?
    else {
        return Ok(Err(GroupRefusal::TransactionNotFound));
    };
    if bt.status != BankTransactionStatus::Pending {
        return Ok(Err(GroupRefusal::TransactionNotPending));
    }
    // Le sens et le montant nul d'abord, la devise ensuite (sinon un montant
    // nul serait masqué par la devise).
    if bt.amount <= Decimal::ZERO {
        return Ok(Err(GroupRefusal::WrongDirection { amount: bt.amount }));
    }
    if !super::reconciliation_link::transaction_can_be_linked_to_entry(
        &bt,
        super::bank_accounts::BASE_CURRENCY,
    ) {
        return Ok(Err(GroupRefusal::CurrencyMismatch {
            currency: bt.currency.clone(),
        }));
    }
    let value_date = payment_date(&bt);
    let window = InvoiceDateWindow::for_transaction(&bt);

    // (4) Chaque facture, par identifiant croissant, sur l'instantané.
    let mut ids: Vec<i64> = invoice_ids.to_vec();
    ids.sort_unstable();
    let mut members: Vec<ClientMember> = Vec::with_capacity(ids.len());
    for id in &ids {
        let Some(invoice) = find_invoice_by_id_for_company(&mut **tx, company_id, *id).await?
        else {
            return Ok(Err(GroupRefusal::InvoiceNotFound {
                document: GroupedDocument::Invoice(*id),
            }));
        };
        let piece = GroupedPiece {
            document: GroupedDocument::Invoice(*id),
            number: invoice.invoice_number.clone(),
        };
        let not_eligible = |reason| {
            Ok(Err(GroupRefusal::InvoiceNotEligible {
                piece: piece.clone(),
                reason,
            }))
        };
        if invoice.status != "validated" {
            return not_eligible(IneligibleReason::NotValidated);
        }
        if invoice.paid_at.is_some() {
            return not_eligible(IneligibleReason::AlreadyPaid);
        }
        // L'ordre d'AC5.4 (revue P1, A LOW-4) : reste dû nul, sans écriture de
        // vente, écriture de vente sans débit, puis la date.
        let raw_due = super::invoice_settlements::amount_due(&mut **tx, *id).await?;
        if raw_due <= Decimal::ZERO {
            return not_eligible(IneligibleReason::NothingDue);
        }
        let Some(sale_entry_id) = invoice.journal_entry_id else {
            return not_eligible(IneligibleReason::JournalEntryNotSet);
        };
        let Some(receivable_account_id) =
            super::invoice_settlements::sale_receivable_account(tx, company_id, sale_entry_id)
                .await?
        else {
            return Ok(Err(GroupRefusal::SaleEntryMalformed {
                piece,
                sale_entry_id,
            }));
        };
        // La borne basse seule : l'ancre n'est pas lue (`check` rend
        // `PaymentBeforeInvoice` avant `PaymentTooLate`, borne haute ignorée).
        if let Err(InvoiceDateRefusal::PaymentBeforeInvoice) =
            window.check(invoice.date, invoice.date)
        {
            return Ok(Err(GroupRefusal::PaymentBeforeInvoiceDate {
                piece,
                tolerance_days: window.tolerance_days,
            }));
        }
        members.push(ClientMember {
            invoice,
            raw_due,
            receivable_account_id,
        });
    }

    // (5) Le compte bancaire : lié, ouvert, non archivé ; le compte de banque
    //     n'est le compte de créance d'aucune facture.
    let bank_row: Option<(Option<i64>, bool)> = sqlx::query_as(
        "SELECT journal_account_id, archived FROM bank_accounts WHERE company_id = ? AND id = ?",
    )
    .bind(company_id)
    .bind(bank_account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let bank_ledger_account_id = match bank_row {
        None | Some((_, true)) => return Ok(Err(GroupRefusal::BankAccountNotFound)),
        Some((None, false)) => return Ok(Err(GroupRefusal::BankAccountNotConfigured)),
        Some((Some(id), false)) => id,
    };
    match super::bank_accounts::ensure_open_in_tx(tx, company_id, bank_account_id).await {
        Ok(()) => {}
        Err(DbError::BankAccountClosed { .. }) => {
            return Ok(Err(GroupRefusal::BankAccountClosed));
        }
        Err(DbError::NotFound) => return Ok(Err(GroupRefusal::BankAccountNotFound)),
        Err(e) => return Err(e),
    }
    let receivables: BTreeSet<i64> = members.iter().map(|m| m.receivable_account_id).collect();
    if let Some(m) = members
        .iter()
        .find(|m| m.receivable_account_id == bank_ledger_account_id)
    {
        return Ok(Err(claim_refusal(
            tx,
            company_id,
            SettlementAccountRole::Counterparty,
            bank_ledger_account_id,
            Some(bank_account_id),
            Some(GroupedPiece {
                document: GroupedDocument::Invoice(m.invoice.id),
                number: m.invoice.invoice_number.clone(),
            }),
        )
        .await?));
    }

    // (6) La différence (C-43-16, D4 bis), puis les comptes qu'elle exige — la
    //     règle commune aux deux familles (`difference_accounts`), côté créance.
    let gross: Vec<Decimal> = members.iter().map(|m| m.raw_due).collect();
    let accounts = match difference_accounts(
        tx,
        company_id,
        bt.amount,
        &gross,
        &receivables,
        ClaimSide::Receivable,
        bank_ledger_account_id,
        difference_account_id,
    )
    .await?
    {
        Ok(a) => a,
        Err(r) => return Ok(Err(r)),
    };

    // (7) L'écriture : banque (au débit), une créance par facture (identifiant
    //     croissant), arrondi, écart (D2) ; exercice ouvert couvrant la date de
    //     valeur.
    let mut lines = vec![NewJournalEntryLine {
        account_id: bank_ledger_account_id,
        debit: bt.amount,
        credit: Decimal::ZERO,
        project_id: None,
        // Chemin automatique : CHF seulement (Epic 37, `invoice_settlements`).
        foreign: None,
    }];
    for m in &members {
        lines.push(NewJournalEntryLine {
            account_id: m.receivable_account_id,
            debit: Decimal::ZERO,
            credit: m.raw_due,
            // Le projet de la facture, estampillé sans validation (D7, C-43-20).
            project_id: m.invoice.project_id,
            foreign: None,
        });
    }
    // Client : moins reçu (`r`, `e` < 0) au débit, trop-perçu au crédit.
    push_difference_lines(&mut lines, &accounts, true);
    let (entry, fiscal_year_id) = match create_group_entry(
        tx,
        company_id,
        acteur,
        value_date,
        entry_description(&members),
        lines,
    )
    .await?
    {
        Ok(done) => done,
        Err(r) => return Ok(Err(r)),
    };
    let DifferenceAccounts {
        decomposition,
        difference_account_id: difference_line_account_id,
        ..
    } = accounts;
    let journal_entry_id = entry.entry.id;
    // Attribution ligne → facture par la position d'insertion (D2) : la ligne
    // `k + 1` (0 = banque) est celle de la facture `k`. `create_in_tx` rend les
    // lignes dans l'ordre fourni (`ORDER BY line_order`).
    if entry.lines.len() < members.len() + 1 {
        return Err(DbError::Invariant(
            "règlement groupé : l'écriture n'a pas une ligne par facture".into(),
        ));
    }
    let line_ids: Vec<i64> = entry.lines[1..=members.len()]
        .iter()
        .map(|l| l.id)
        .collect();

    // (8) Une ligne `invoice_settlements` par facture.
    let mut refs: Vec<GroupedInvoiceRef> = Vec::with_capacity(members.len());
    for (m, line_id) in members.iter().zip(&line_ids) {
        let s = super::invoice_settlements::create_in_tx(
            tx,
            NewInvoiceSettlement {
                company_id,
                invoice_id: m.invoice.id,
                journal_entry_id,
                journal_entry_line_id: Some(*line_id),
                amount: m.raw_due,
                settled_on: value_date,
                kind: crate::entities::SettlementKind::Choice(
                    crate::entities::SettlementChoice::BankTransfer { bank_account_id },
                ),
            },
        )
        .await?;
        refs.push(GroupedInvoiceRef {
            kind: GroupFamily::Client,
            id: m.invoice.id,
            number: m.invoice.invoice_number.clone(),
            settlement_id: Some(s.id),
            line_id: *line_id,
            amount: m.raw_due,
        });
    }

    // (9) Chaque facture : reste dû à zéro, puis `paid_at` et `version + 1` à
    //     la version LUE (étape 4) — le verrou qui interdit de régler deux fois.
    let paid_at = value_date
        .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).expect("minuit est une heure valide"));
    for m in &members {
        let due_after = super::invoice_settlements::amount_due(&mut **tx, m.invoice.id).await?;
        if due_after > Decimal::ZERO {
            return Err(DbError::Invariant(format!(
                "règlement groupé : la facture {} garde un reste dû de {due_after}",
                m.invoice.id
            )));
        }
        let updated = sqlx::query(
            "UPDATE invoices SET paid_at = ?, version = version + 1, updated_at = NOW(3) \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'validated'",
        )
        .bind(paid_at)
        .bind(m.invoice.id)
        .bind(company_id)
        .bind(m.invoice.version)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
        if updated.rows_affected() != 1 {
            return Ok(Err(GroupRefusal::InvoiceNotEligible {
                piece: GroupedPiece {
                    document: GroupedDocument::Invoice(m.invoice.id),
                    number: m.invoice.invoice_number.clone(),
                },
                reason: IneligibleReason::RaceDuringUpdate,
            }));
        }
    }

    // (10) La transaction.
    let updated = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = ?, status = 'reconciled', updated_at = NOW(3), \
             version = version + 1 \
         WHERE id = ? AND company_id = ? AND status = 'pending' AND matched_entry_id IS NULL \
           AND version = ?",
    )
    .bind(journal_entry_id)
    .bind(bank_transaction_id)
    .bind(company_id)
    .bind(bt.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if updated.rows_affected() != 1 {
        return Ok(Err(GroupRefusal::TransactionNotPending));
    }

    // (11) Le lettrage de chaque facture, la revérification du compte, l'audit.
    for m in &members {
        super::letterings::sync_invoice_in_tx(tx, company_id, m.invoice.id, fiscal_year_id, acteur)
            .await?;
    }
    super::bank_accounts::recheck_open_locked_in_tx(tx, company_id, bank_account_id).await?;

    let accepted = super::audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::par(
            acteur,
            "reconciliation.accepted",
            "bank_transaction",
            bank_transaction_id,
            Some(serde_json::json!({
                "proposal_type": "invoice_group",
                "bank_transaction_id": bank_transaction_id,
                "journal_entry_id": journal_entry_id,
                "batch_size": batch_size,
                "paid_amount": bt.amount,
                "difference": decomposition.difference,
                "rounding_difference": decomposition.rounding_difference,
                "gap": decomposition.gap,
                "difference_account_id": difference_line_account_id,
                "invoices": refs
                    .iter()
                    .map(|i| serde_json::json!({
                        "invoice_id": i.id,
                        "settled_amount": i.amount,
                    }))
                    .collect::<Vec<_>>(),
            })),
        ),
    )
    .await?;
    for m in &members {
        let version_pre = m.invoice.version;
        super::audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::par(
                acteur,
                "invoice.paid",
                "invoice",
                m.invoice.id,
                Some(serde_json::json!({
                    "paid_at": paid_at.and_utc(),
                    "paid_by_user_id": acteur.user_id,
                    "paid_via": "reconciliation",
                    "reconciliation_audit_id": accepted.id,
                    "settlement_journal_entry_id": journal_entry_id,
                    "settled_amount": m.raw_due,
                    "amount_due_after": Decimal::ZERO,
                    "grouped": true,
                    "before": { "paid_at": null, "version": version_pre },
                    "after": { "paid_at": paid_at.and_utc(), "version": version_pre + 1 },
                })),
            ),
        )
        .await?;
    }

    Ok(Ok(GroupedSettlement {
        journal_entry_id,
        reconciliation_audit_id: accepted.id,
        decomposition,
        difference_account_id: difference_line_account_id,
        invoices: refs,
    }))
}

/// Le libellé de l'écriture : « Encaissement groupé de N factures : n1, n2, … »,
/// borné à la colonne (500 caractères).
fn entry_description(members: &[ClientMember]) -> String {
    let numbers: Vec<String> = members
        .iter()
        .map(|m| {
            m.invoice
                .invoice_number
                .clone()
                .unwrap_or_else(|| m.invoice.id.to_string())
        })
        .collect();
    bounded_description(format!(
        "Encaissement groupé de {} factures : {}",
        members.len(),
        numbers.join(", ")
    ))
}

/// Un libellé d'écriture borné à la colonne (500 caractères), coupé avec
/// « ... » — une seule définition pour les deux familles.
fn bounded_description(full: String) -> String {
    if full.chars().count() <= 500 {
        full
    } else {
        let mut cut: String = full.chars().take(497).collect();
        cut.push_str("...");
        cut
    }
}

/// Construit le refus de compte de tiers, numéro du compte lu à l'échec — côté
/// client (la créance soldée).
async fn claim_refusal(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    role: SettlementAccountRole,
    account_id: i64,
    bank_account_id: Option<i64>,
    piece: Option<GroupedPiece>,
) -> Result<GroupRefusal, DbError> {
    claim_refusal_on(
        tx,
        company_id,
        ClaimSide::Receivable,
        role,
        account_id,
        bank_account_id,
        piece,
    )
    .await
}

/// [`claim_refusal`], pour le côté soldé `claim` — la dette d'un groupe
/// fournisseur (Story 43-1b) passe `ClaimSide::Payable`.
async fn claim_refusal_on(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    claim: ClaimSide,
    role: SettlementAccountRole,
    account_id: i64,
    bank_account_id: Option<i64>,
    piece: Option<GroupedPiece>,
) -> Result<GroupRefusal, DbError> {
    let account_number = super::accounts::number_in_company(tx, company_id, account_id).await?;
    Ok(GroupRefusal::ClaimAccount {
        claim,
        role,
        account_id,
        account_number,
        bank_account_id,
        piece,
    })
}

// ---------------------------------------------------------------------------
// La règle commune aux deux familles : la différence, ses comptes, l'écriture
// (Story 43-1b, revue P1 B-3 — extraite du bras client de la 43-1a, à
// comportement égal)
// ---------------------------------------------------------------------------

/// La différence d'un groupe et les comptes qu'elle exige, contrôlés.
struct DifferenceAccounts {
    decomposition: Decomposition,
    /// Le compte d'arrondi, si `r ≠ 0`.
    rounding_account_id: Option<i64>,
    /// Le compte choisi pour l'écart, si `e ≠ 0`.
    difference_account_id: Option<i64>,
}

/// **La règle de la différence au geste, écrite une seule fois pour les deux
/// familles** (C-43-16, C-43-25) : [`group_difference`] — **le plafond
/// d'abord** — puis, si `r ≠ 0`, le compte d'arrondi (requis, et jamais un
/// compte de tiers du groupe, C-e431a-7) ; puis, si `e ≠ 0`, le compte choisi,
/// dans l'ordre : requis (`DifferenceAccountRequired`), pas la banque, existant
/// et actif, imputable, en CHF ([`difference_account_refusal`]), pas un compte
/// de tiers du groupe.
///
/// `claims` : les comptes de tiers des factures (créances ou dettes) ; `claim` :
/// leur côté, porté par le refus `ClaimAccount`.
#[allow(clippy::too_many_arguments)]
async fn difference_accounts(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    amount: Decimal,
    gross: &[Decimal],
    claims: &BTreeSet<i64>,
    claim: ClaimSide,
    bank_ledger_account_id: i64,
    difference_account_id: Option<i64>,
) -> Result<Result<DifferenceAccounts, GroupRefusal>, DbError> {
    let max =
        super::company_invoice_settings::group_settlement_max_difference(&mut **tx, company_id)
            .await?;
    let decomposition = match group_difference(amount, gross, max) {
        Ok(d) => d,
        Err(r) => return Ok(Err(r)),
    };
    let rounding_account_id = if decomposition.rounding_difference.is_zero() {
        None
    } else {
        match super::company_invoice_settings::rounding_account_for_write(
            tx,
            company_id,
            crate::errors::RoundingContext::Payment,
        )
        .await
        {
            Ok(id) => {
                if claims.contains(&id) {
                    return Ok(Err(claim_refusal_on(
                        tx,
                        company_id,
                        claim,
                        SettlementAccountRole::Rounding,
                        id,
                        None,
                        None,
                    )
                    .await?));
                }
                Some(id)
            }
            Err(DbError::RoundingAccountNotConfigured { .. }) => {
                return Ok(Err(GroupRefusal::RoundingAccountNotConfigured));
            }
            Err(e) => return Err(e),
        }
    };
    let e = decomposition.gap;
    let difference_line_account_id = if e.is_zero() {
        None
    } else {
        let Some(id) = difference_account_id else {
            return Ok(Err(GroupRefusal::DifferenceAccountRequired { gap: e }));
        };
        if id == bank_ledger_account_id {
            return Ok(Err(GroupRefusal::DifferenceAccountIsBankLedger {
                account_id: id,
            }));
        }
        if let Some(refusal) = difference_account_refusal(tx, company_id, id).await? {
            return Ok(Err(refusal));
        }
        if claims.contains(&id) {
            return Ok(Err(claim_refusal_on(
                tx,
                company_id,
                claim,
                SettlementAccountRole::Counterparty,
                id,
                None,
                None,
            )
            .await?));
        }
        Some(id)
    };
    Ok(Ok(DifferenceAccounts {
        decomposition,
        rounding_account_id,
        difference_account_id: difference_line_account_id,
    }))
}

/// Ajoute à l'écriture la ligne d'arrondi puis la ligne d'écart, du côté qui
/// l'équilibre : la banque au **débit** (client) met `r > 0` et `e > 0` au
/// **crédit** ; la banque au crédit (fournisseur), au débit.
fn push_difference_lines(
    lines: &mut Vec<NewJournalEntryLine>,
    accounts: &DifferenceAccounts,
    bank_on_debit: bool,
) {
    let mut push = |account_id: i64, value: Decimal| {
        let (positive, negative) = (value.max(Decimal::ZERO), (-value).max(Decimal::ZERO));
        let (debit, credit) = if bank_on_debit {
            (negative, positive)
        } else {
            (positive, negative)
        };
        lines.push(NewJournalEntryLine {
            account_id,
            debit,
            credit,
            project_id: None,
            // CHF seulement : un compte d'écart en devise est refusé en amont
            // (`DifferenceAccountInForeignCurrency`) ; l'arrondi est un compte
            // désigné, que l'Epic 37 interdit en devise.
            foreign: None,
        });
    };
    if let Some(id) = accounts.rounding_account_id {
        push(id, accounts.decomposition.rounding_difference);
    }
    if let Some(id) = accounts.difference_account_id {
        push(id, accounts.decomposition.gap);
    }
}

/// L'écriture d'un groupe : l'exercice ouvert qui couvre la date de valeur,
/// puis la création (`enforce_postable = false`, les comptes étant ceux des
/// pièces et des réglages, le compte choisi contrôlé à part), les erreurs de
/// période et d'exercice traduites en refus. Rend l'écriture et son exercice.
async fn create_group_entry(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    acteur: Acteur,
    value_date: NaiveDate,
    description: String,
    lines: Vec<NewJournalEntryLine>,
) -> Result<Result<(crate::entities::JournalEntryWithLines, i64), GroupRefusal>, DbError> {
    let Some(fiscal_year) =
        super::fiscal_years::find_open_covering_date(tx, company_id, value_date).await?
    else {
        return Ok(Err(GroupRefusal::FiscalYearInvalid { value_date }));
    };
    let new_entry = NewJournalEntry {
        company_id,
        entry_date: value_date,
        journal: crate::entities::journal_entry::Journal::Banque,
        description,
        project_id: None,
        lines,
    };
    match super::journal_entries::create_copying_projects_in_tx(
        tx,
        fiscal_year.id,
        acteur,
        new_entry,
        false,
    )
    .await
    {
        Ok(e) => Ok(Ok((e, fiscal_year.id))),
        Err(DbError::PeriodLocked {
            locked_through,
            attempted,
        }) => Ok(Err(GroupRefusal::PeriodLocked {
            locked_through,
            attempted,
        })),
        Err(DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        }) => Ok(Err(GroupRefusal::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        })),
        Err(DbError::FiscalYearClosed | DbError::DateOutsideFiscalYear) => {
            Ok(Err(GroupRefusal::FiscalYearInvalid { value_date }))
        }
        Err(e) => Err(e),
    }
}

// ---------------------------------------------------------------------------
// Le dé-rapprochement d'un groupe (AC7)
// ---------------------------------------------------------------------------

/// Une facture d'un groupe, telle que la vue de la transaction la rend
/// (`groupedInvoices`, 43-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedViewInvoice {
    pub kind: GroupFamily,
    pub id: i64,
    pub number: Option<String>,
    /// Le montant soldé, brut.
    pub amount: Decimal,
}

/// Les factures clients d'une écriture groupée, par identifiant croissant, avec
/// leur numéro — lecture **ordinaire** (jointure sur `invoices`, sans verrou).
///
/// ⛔ **Dans le geste, jamais avant le verrou de l'écriture et de son
/// exercice** (revue P1, B-1 / E-1) : sous `REPEATABLE READ`, la première
/// lecture ordinaire fige l'instantané ; faite plus tôt, elle ferait juger le
/// rang 2 (« exercice clos ») et le rang 2 bis sur l'état d'avant une clôture
/// concurrente. Le geste prend d'abord ses membres par [`locked_members`].
pub(crate) async fn client_members(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<(i64, i64, Option<String>, Decimal)>, DbError> {
    sqlx::query_as(
        "SELECT s.id, s.invoice_id, i.invoice_number, s.amount \
         FROM invoice_settlements s \
         JOIN invoices i ON i.id = s.invoice_id AND i.company_id = s.company_id \
         WHERE s.company_id = ? AND s.journal_entry_id = ? ORDER BY s.invoice_id",
    )
    .bind(company_id)
    .bind(entry_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)
}

/// `(règlement, facture)` d'une écriture groupée, par une lecture
/// **verrouillante** de `invoice_settlements` seule, dans l'ordre de la clé
/// `(journal_entry_id, invoice_id)` — la forme du geste (revue P1, B-1) : elle
/// ne fige pas l'instantané, et ne prend aucun verrou que le classement
/// verrouillant (`reconciliation_cancel::classify`) ne tienne déjà.
async fn locked_members(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<(i64, i64)>, DbError> {
    sqlx::query_as(
        "SELECT id, invoice_id FROM invoice_settlements \
         WHERE company_id = ? AND journal_entry_id = ? \
         ORDER BY journal_entry_id, invoice_id FOR UPDATE",
    )
    .bind(company_id)
    .bind(entry_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Un membre fournisseur d'une écriture groupée, lu par [`supplier_members`].
#[derive(Debug, Clone)]
pub(crate) struct SupplierGroupMember {
    pub id: i64,
    /// Le numéro affichable : interne, à défaut celui du fournisseur.
    pub number: Option<String>,
    /// Le `total_amount` de la facture : son apport au groupe.
    pub amount: Decimal,
    /// Sa ligne de dette dans l'écriture (`settlement_journal_entry_line_id`).
    pub line_id: Option<i64>,
}

/// Le filtre des membres fournisseur d'une écriture groupée : les factures
/// `paid` qui la pointent — le prédicat du classement
/// (`reconciliation_cancel::SUPPLIER_SETTLEMENT_CLASSIFY_SQL`), par l'index de
/// `settlement_journal_entry_id`. Une facture **détachée** (annulée, colonnes
/// vidées, AC7) n'en est plus.
const SUPPLIER_MEMBERS_WHERE: &str =
    "WHERE company_id = ? AND settlement_journal_entry_id = ? AND status = 'paid' ORDER BY id";

/// Les factures fournisseur d'une écriture groupée, par identifiant croissant
/// (Story 43-1b, AC6 bis) — lecture **ordinaire**, sous la même règle que
/// [`client_members`] : dans le geste, jamais avant le verrou de l'écriture et
/// de son exercice.
pub(crate) async fn supplier_members(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<SupplierGroupMember>, DbError> {
    /// `(facture, numéro interne, numéro du fournisseur, total, ligne)`.
    type Row = (i64, Option<String>, Option<String>, Decimal, Option<i64>);
    let rows: Vec<Row> = sqlx::query_as(&format!(
        "SELECT id, internal_number, supplier_invoice_number, total_amount, \
             settlement_journal_entry_line_id FROM supplier_invoices {SUPPLIER_MEMBERS_WHERE}"
    ))
    .bind(company_id)
    .bind(entry_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?;
    Ok(rows
        .into_iter()
        .map(
            |(id, interne, fournisseur, amount, line_id)| SupplierGroupMember {
                id,
                number: interne.or(fournisseur),
                amount,
                line_id,
            },
        )
        .collect())
}

/// `(facture, version)` des membres fournisseur, par une lecture
/// **verrouillante** — les lignes que le classement verrouillant tient déjà
/// (`reconciliation_cancel::classify`, même prédicat, même index) : elle ne
/// fige pas l'instantané et ne prend aucun verrou neuf.
async fn locked_supplier_members(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<(i64, i64, Option<chrono::NaiveDateTime>)>, DbError> {
    sqlx::query_as(&format!(
        "SELECT id, version, paid_at FROM supplier_invoices {SUPPLIER_MEMBERS_WHERE} FOR UPDATE"
    ))
    .bind(company_id)
    .bind(entry_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Les factures d'une écriture groupée, pour la vue (`get_view`, 43-2) : les
/// membres clients, à défaut les membres fournisseur (Story 43-1b, AC6 bis) —
/// une écriture groupée n'a qu'une famille (C-43-15).
pub(crate) async fn view_members(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<GroupedViewInvoice>, DbError> {
    let clients = client_members(conn, company_id, entry_id).await?;
    if !clients.is_empty() {
        return Ok(clients
            .into_iter()
            .map(|(_, id, number, amount)| GroupedViewInvoice {
                kind: GroupFamily::Client,
                id,
                number,
                amount,
            })
            .collect());
    }
    Ok(supplier_members(conn, company_id, entry_id)
        .await?
        .into_iter()
        .map(|m| GroupedViewInvoice {
            kind: GroupFamily::Supplier,
            id: m.id,
            number: m.number,
            amount: m.amount,
        })
        .collect())
}

/// Le compte de l'écart d'une écriture groupée, **relu sur sa ligne d'écart**
/// (pour la vue, 43-2) ; `None` sans écart.
///
/// L'écriture porte, dans l'ordre d'insertion (`line_order`, D2) : la banque,
/// une ligne de tiers par facture, la ligne d'arrondi si l'arrondi est non nul,
/// puis la ligne d'écart si l'écart est non nul. Les deux se **recalculent**
/// par [`group_difference`] — déterministe — depuis le montant de la ligne de
/// banque et les montants bruts des règlements (plafond sans objet ici) : c'est
/// ce qui sépare les deux lignes même quand l'utilisateur a choisi pour l'écart
/// le compte d'arrondi, et même sous l'arrondi automatique.
pub(crate) async fn difference_account_of(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Option<i64>, DbError> {
    let amounts: Vec<Decimal> = sqlx::query_scalar(
        "SELECT amount FROM invoice_settlements WHERE company_id = ? AND journal_entry_id = ?",
    )
    .bind(company_id)
    .bind(entry_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?;
    // Story 43-1b (AC6 bis) : sans règlement client, le groupe est fournisseur.
    if amounts.is_empty() {
        return supplier_difference_account_of(conn, company_id, entry_id).await;
    }
    let lines: Vec<(i64, Decimal)> = sqlx::query_as(
        "SELECT jel.account_id, jel.debit + jel.credit FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? ORDER BY jel.line_order",
    )
    .bind(entry_id)
    .bind(company_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?;
    // Revue P1 (B-2, E-3, E-4) : une écriture qui n'a pas la forme d'un groupe
    // client — aucune ligne, ligne d'écart absente — rend `Invariant`, jamais
    // une valeur fausse ni un « sans écart » silencieux.
    let malformee = |quoi: &str| {
        DbError::Invariant(format!(
            "écriture groupée {entry_id} : {quoi} (relecture du compte d'écart)"
        ))
    };
    let Some((_, paid)) = lines.first() else {
        return Err(malformee("aucune ligne"));
    };
    let d = group_difference(*paid, &amounts, Decimal::MAX)
        .map_err(|r| malformee(&format!("décomposition {r:?}")))?;
    if d.gap.is_zero() {
        return Ok(None);
    }
    let skip = 1 + amounts.len() + usize::from(!d.rounding_difference.is_zero());
    lines
        .get(skip)
        .map(|(account, _)| Some(*account))
        .ok_or_else(|| malformee("ligne d'écart absente"))
}

/// [`difference_account_of`] pour un groupe **fournisseur** (Story 43-1b,
/// AC6 bis) : même ordre des lignes (banque, une dette par facture, arrondi si
/// `r ≠ 0`, écart si `e ≠ 0`), même [`group_difference`] (plafond sans objet).
///
/// ⚠️ **Les lignes de dette se lisent nommément, jamais devinées** (revue P1,
/// A MEDIUM-1, B-2, E1 ; C-e431b-4) : une facture du groupe peut avoir été
/// **détachée** (annulée après coup, AC7) — elle n'est plus membre, mais sa
/// ligne de dette reste dans l'écriture, et sa colonne
/// `settlement_journal_entry_line_id` est vidée. Les lignes de dette sont donc
/// celles des membres (leur colonne) **et** celles des factures détachées,
/// nommées par la trace de leur détachement (`supplier_invoice.cancelled`,
/// `details.settlementJournalEntryLineId`, écrite par la 43-1b — la seule
/// trace qui en reste, comme le gel « paiement détaché » lit déjà
/// `settlementJournalEntryId` dans la même trace). Leur nombre doit égaler celui
/// des factures que le paiement a soldées, recoupé sur son entrée
/// `reconciliation.accepted` ([`group_size_at_payment`], revue P2, LOW-1 : sans
/// lui, une trace manquante pour la **dernière** facture d'un groupe sans écart
/// passerait inaperçue) ; ensemble, elles doivent occuper exactement les
/// positions 1 à `n` ; les apports sont leurs montants (le `total_amount` de
/// chaque facture, contrôlé égal au paiement), la queue (arrondi, écart) se
/// contrôle en nombre. Toute autre forme — une ligne de dette qu'aucune facture
/// ne nomme, une trace manquante — rend `Invariant`, jamais une valeur fausse.
async fn supplier_difference_account_of(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Option<i64>, DbError> {
    let malformee = |quoi: &str| {
        DbError::Invariant(format!(
            "écriture groupée fournisseur {entry_id} : {quoi} (relecture du compte d'écart)"
        ))
    };
    let members = supplier_members(conn, company_id, entry_id).await?;
    if members.is_empty() {
        return Err(malformee("aucun règlement"));
    }
    let lines: Vec<(i64, i64, Decimal)> = sqlx::query_as(
        "SELECT jel.id, jel.account_id, jel.debit + jel.credit FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? ORDER BY jel.line_order",
    )
    .bind(entry_id)
    .bind(company_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let Some((_, _, paid)) = lines.first() else {
        return Err(malformee("aucune ligne"));
    };
    // Les lignes de dette : celles des membres, puis celles des détachées.
    let mut debt_lines: BTreeSet<i64> = BTreeSet::new();
    for m in &members {
        debt_lines.insert(
            m.line_id
                .ok_or_else(|| malformee("un membre sans ligne de règlement"))?,
        );
    }
    debt_lines.extend(detached_debt_lines(conn, company_id, entry_id).await?);
    // Autant de lignes nommées que de factures payées par le geste (revue P2,
    // LOW-1) : sans ce recoupement, une trace de détachement manquante pour la
    // DERNIÈRE facture d'un groupe sans écart ferait lire sa dette comme ligne
    // d'écart, la forme restant cohérente.
    let n = debt_lines.len();
    if group_size_at_payment(conn, company_id, entry_id).await? != Some(n) {
        return Err(malformee(
            "les lignes de dette nommées ne recoupent pas les factures du paiement",
        ));
    }
    // Elles occupent exactement les positions 1 à n (D2 : une par facture,
    // juste après la banque).
    let block_ok = lines.len() > n
        && lines[1..=n]
            .iter()
            .all(|(id, _, _)| debt_lines.contains(id));
    if !block_ok {
        return Err(malformee(
            "les lignes de dette nommées n'occupent pas les positions 1 à n",
        ));
    }
    let contributions: Vec<Decimal> = lines[1..=n].iter().map(|(_, _, a)| *a).collect();
    let d = group_difference(*paid, &contributions, Decimal::MAX)
        .map_err(|r| malformee(&format!("décomposition {r:?}")))?;
    let tail = usize::from(!d.rounding_difference.is_zero()) + usize::from(!d.gap.is_zero());
    if lines.len() != n + 1 + tail {
        return Err(malformee("lignes d'arrondi et d'écart inattendues"));
    }
    if d.gap.is_zero() {
        return Ok(None);
    }
    Ok(lines.last().map(|(_, account, _)| *account))
}

/// Les lignes de dette des factures **détachées** de l'écriture groupée
/// `entry_id` (AC7) : celles que nomment les traces `supplier_invoice.cancelled`
/// de la société dont `settlementJournalEntryId` est l'écriture
/// ([`super::journal_entries::DETACHED_SETTLEMENT_TRACES`], la lecture du gel
/// « paiement détaché »), **sur les seules factures `cancelled`** — le filtre de
/// `modification_guard` (revue P2, LOW-6) : il borne le balayage, et écarte une
/// trace étrangère restaurée qui se joindrait par `entity_id` à une facture qui
/// n'est pas annulée. Une facture détachée puis repassée en brouillon n'est plus
/// `cancelled`, mais ce repassage exige l'écriture contre-passée, donc une
/// transaction qui n'est plus rapprochée : sa vue ne se lit plus.
async fn detached_debt_lines(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<i64>, DbError> {
    use super::journal_entries::{DETACHED_SETTLEMENT_ENTRY, DETACHED_SETTLEMENT_TRACES};
    let lines: Vec<Option<i64>> = sqlx::query_scalar(&format!(
        "SELECT DISTINCT CAST(JSON_VALUE(al.details_json, '$.settlementJournalEntryLineId') \
         AS SIGNED) FROM {DETACHED_SETTLEMENT_TRACES} \
           AND si.status = 'cancelled' AND {DETACHED_SETTLEMENT_ENTRY} = ?"
    ))
    .bind(company_id)
    .bind(company_id)
    .bind(entry_id.to_string())
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?;
    Ok(lines.into_iter().flatten().collect())
}

/// Le nombre de factures que le paiement groupé `entry_id` a soldées, lu sur
/// son entrée `reconciliation.accepted` (`details.supplier_invoices`, écrite
/// dans la même transaction que l'écriture) — un **recoupement**, jamais la
/// source d'une valeur (revue P2, LOW-1) : le compte d'écart se relit toujours
/// sur l'écriture. `None` si l'entrée manque ou n'est pas unique.
async fn group_size_at_payment(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Option<usize>, DbError> {
    let sizes: Vec<Option<i64>> = sqlx::query_scalar(
        "SELECT JSON_LENGTH(details_json, '$.supplier_invoices') FROM audit_log \
         WHERE company_id = ? AND entity_type = 'bank_transaction' \
           AND action = 'reconciliation.accepted' \
           AND JSON_VALUE(details_json, '$.proposal_type') = 'supplier_invoice_group' \
           AND JSON_VALUE(details_json, '$.journal_entry_id') = ?",
    )
    .bind(company_id)
    .bind(entry_id.to_string())
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?;
    Ok(match sizes.as_slice() {
        [Some(n)] => usize::try_from(*n).ok(),
        _ => None,
    })
}

/// Le point de sauvegarde du lot a-t-il disparu — la transaction annulée sous
/// le geste (interblocage absorbé plus bas) ? Lu sur le **code** MySQL 1305,
/// jamais sur le texte : c'est l'`Err(DbError)` que rend un `ROLLBACK TO
/// SAVEPOINT grouped_settlement` qui échoue (D6), et que la route rejoue.
pub fn is_savepoint_lost(err: &DbError) -> bool {
    match err {
        DbError::Sqlx(e) => e
            .as_database_error()
            .and_then(|db| db.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
            .is_some_and(|my| my.number() == 1305),
        _ => false,
    }
}

/// Le premier motif qui empêche de défaire le groupe `entry_id`, et la facture
/// qui le porte — pour la vue et pour le geste (AC7.1). `unlinking` : la
/// transaction que l'on défait (exemption étroite du rang 3).
///
/// Dans cet ordre : la **tête** de chaque facture, par identifiant croissant
/// (rang 1, facture créditée ; rang 1 bis, solde existant —
/// `invoice_settlements_write::settlement_cancel_head`), puis la **queue de
/// l'écriture commune, une seule fois** (rangs 2 à 5 —
/// `settlement_entry_cancel_blocker` ; revue P1, B-5 / E-5 : elle était
/// réévaluée par facture). Un motif de l'écriture nomme la première facture.
///
/// Un groupe **fournisseur** (Story 43-1b, AC6) : la tête fournisseur de chaque
/// facture (rang 1, `supplier_invoices::supplier_settlement_cancel_head`), puis
/// la même queue une fois. Une écriture sans aucun membre →
/// [`DbError::Invariant`] : aucune réponse fausse (revue P1, B-2, E-3).
pub(crate) async fn group_cancel_blocker(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
    unlinking: i64,
    aujourd_hui: NaiveDate,
) -> Result<
    Option<(
        crate::repositories::settlement_cancellation::SettlementCancelHit,
        GroupedPiece,
    )>,
    DbError,
> {
    let members = client_members(conn, company_id, entry_id).await?;
    let Some((_, first_invoice, first_number, _)) = members.first().cloned() else {
        return supplier_group_cancel_blocker(conn, company_id, entry_id, unlinking, aujourd_hui)
            .await;
    };
    for (settlement_id, invoice_id, number, _) in members {
        let (_, head) = super::invoice_settlements_write::settlement_cancel_head(
            conn,
            company_id,
            settlement_id,
        )
        .await?;
        if let Some(hit) = head {
            return Ok(Some((
                hit,
                GroupedPiece {
                    document: GroupedDocument::Invoice(invoice_id),
                    number,
                },
            )));
        }
    }
    Ok(
        super::settlement_cancellation::settlement_entry_cancel_blocker(
            conn,
            company_id,
            entry_id,
            Some(unlinking),
            aujourd_hui,
        )
        .await?
        .map(|hit| {
            (
                hit,
                GroupedPiece {
                    document: GroupedDocument::Invoice(first_invoice),
                    number: first_number,
                },
            )
        }),
    )
}

/// Le bras fournisseur de [`group_cancel_blocker`] (Story 43-1b, AC6) : la
/// tête de chaque facture, par identifiant croissant, puis la queue de
/// l'écriture commune **une fois** ; un motif de l'écriture nomme la première
/// facture.
async fn supplier_group_cancel_blocker(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
    unlinking: i64,
    aujourd_hui: NaiveDate,
) -> Result<
    Option<(
        crate::repositories::settlement_cancellation::SettlementCancelHit,
        GroupedPiece,
    )>,
    DbError,
> {
    let members = supplier_members(conn, company_id, entry_id).await?;
    let Some(first) = members.first().cloned() else {
        return Err(DbError::Invariant(format!(
            "règlement groupé : l'écriture {entry_id} n'a aucun règlement, client ni fournisseur"
        )));
    };
    let piece = |m: &SupplierGroupMember| GroupedPiece {
        document: GroupedDocument::SupplierInvoice(m.id),
        number: m.number.clone(),
    };
    for m in &members {
        if let Err(hit) =
            super::supplier_invoices::supplier_settlement_cancel_head(conn, company_id, m.id)
                .await?
        {
            return Ok(Some((hit, piece(m))));
        }
    }
    Ok(
        super::settlement_cancellation::settlement_entry_cancel_blocker(
            conn,
            company_id,
            entry_id,
            Some(unlinking),
            aujourd_hui,
        )
        .await?
        .map(|hit| (hit, piece(&first))),
    )
}

/// Ce que rend le dé-rapprochement d'un groupe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedCancellation {
    /// L'écriture inverse, datée du jour.
    pub reversal_journal_entry_id: i64,
    /// Les factures dont le règlement a été retiré (identifiant croissant).
    pub invoices: Vec<GroupedViewInvoice>,
}

/// **Défait un règlement groupé** — appelé par
/// [`super::reconciliation_cancel::cancel_in_tx`] quand le classement reconnaît
/// le groupe (AC7, C-43-6), après le verrou de la transaction `bt` et la lecture
/// **verrouillante** des lignes `invoice_settlements` de l'écriture.
///
/// Dans l'ordre de l'unitaire : factures verrouillées par identifiant
/// croissant, écriture et exercice verrouillés, `ensure_open_in_tx` ; puis :
///
/// 1. **tous les refus, avant toute écriture** — par facture (créditée, solde
///    existant) et sur l'écriture (exercice clos, lettrage figé par la période,
///    autre transaction liée, compte archivé, pas d'exercice ouvert au jour) ;
///    le premier rendu, en [`DbError::ReconciliationNotCancellable`].
///    `expect_unlink_only = Some(true)` → `ReconciliationCancelChanged` : un
///    groupe n'est jamais un retrait seul ;
/// 2. le lien défait, comme l'unitaire ;
/// 3. le délettrage de chaque facture ;
/// 4. la suppression des lignes `invoice_settlements` de l'écriture ;
/// 5. la contre-passation de l'écriture devenue sans propriétaire
///    (`journal_entries::reverse_in_tx`, D4 : aucune autorité neuve) ;
/// 6. chaque facture : `paid_at` projeté, `version + 1` ;
/// 7. `recheck_open_locked_in_tx`, puis l'audit — une
///    `invoice.settlement_cancelled` par facture et `reconciliation.cancelled`.
///
/// N'exécute pas de rollback : l'appelant en est responsable.
pub async fn cancel_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bt: &crate::entities::bank_transaction::BankTransaction,
    entry_id: i64,
    acteur: Acteur,
    expect_unlink_only: Option<bool>,
    aujourd_hui: NaiveDate,
) -> Result<GroupedCancellation, DbError> {
    // ⛔ L'ordre de l'unitaire (`reconciliation_cancel::cancel_in_tx`, étape 2) :
    // **aucune lecture non verrouillante avant le verrou de l'exercice** (revue
    // P1, B-1 / E-1). Les membres d'abord, par une lecture VERROUILLANTE de
    // `invoice_settlements` seule — lignes déjà tenues par le classement —, puis
    // les factures, puis l'écriture et son exercice ; les numéros (jointure
    // ordinaire) seulement après.
    let tenus = locked_members(tx, company_id, entry_id).await?;
    if tenus.is_empty() {
        // Story 43-1b (AC6) : sans règlement client, le groupe est fournisseur
        // (C-e431a-9 : l'`Invariant` d'avant est remplacé par ce bras).
        return supplier_cancel_in_tx(
            tx,
            company_id,
            bt,
            entry_id,
            acteur,
            expect_unlink_only,
            aujourd_hui,
        )
        .await;
    }
    // Les factures, par identifiant croissant.
    for (_, invoice_id) in &tenus {
        sqlx::query("SELECT id FROM invoices WHERE id = ? AND company_id = ? FOR UPDATE")
            .bind(invoice_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?
            .ok_or(DbError::NotFound)?;
    }
    // L'écriture et son exercice, avant de juger.
    let fiscal_year_id: i64 = sqlx::query_scalar(
        "SELECT fy.id FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? FOR UPDATE",
    )
    .bind(entry_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .ok_or(DbError::NotFound)?;
    // Lecture non verrouillante APRÈS le verrou de l'exercice (règle de
    // placement de `bank_accounts::ensure_open_in_tx`, comme l'unitaire).
    super::bank_accounts::ensure_open_in_tx(tx, company_id, bt.bank_account_id).await?;
    // Les numéros, maintenant que l'instantané naît après le verrou.
    let members = client_members(tx, company_id, entry_id).await?;
    let lus: Vec<(i64, i64)> = members.iter().map(|(s, i, _, _)| (*s, *i)).collect();
    if lus != tenus {
        return Err(DbError::Invariant(format!(
            "dé-rapprochement groupé : les règlements lus ({lus:?}) ne sont pas ceux \
             tenus ({tenus:?})"
        )));
    }

    // (1) Les refus, avant toute écriture.
    if expect_unlink_only == Some(true) {
        return Err(DbError::ReconciliationCancelChanged { unlink_only: false });
    }
    if let Some(((blocker, _, _), _)) =
        group_cancel_blocker(tx, company_id, entry_id, bt.id, aujourd_hui).await?
    {
        return Err(DbError::ReconciliationNotCancellable { blocker });
    }

    // (2) Le lien défait.
    let updated = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL, \
             version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ? AND status = 'reconciled' AND version = ?",
    )
    .bind(bt.id)
    .bind(company_id)
    .bind(bt.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if updated.rows_affected() != 1 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (3) Le délettrage de chaque pièce.
    for (_, invoice_id, _, _) in &members {
        super::letterings::dissolve_invoice_document_group_in_tx(
            tx,
            company_id,
            *invoice_id,
            fiscal_year_id,
            acteur,
        )
        .await?;
    }

    // (4) Les règlements retirés : l'écriture n'a plus de propriétaire.
    let retires = super::invoice_settlements_write::delete_grouped_settlements_in_tx(
        tx, company_id, entry_id,
    )
    .await?;
    if retires != members.len() as u64 {
        return Err(DbError::Invariant(format!(
            "dé-rapprochement groupé : {retires} règlements retirés pour {} lus",
            members.len()
        )));
    }

    // (5) La contre-passation.
    let reversal =
        super::journal_entries::reverse_in_tx(tx, company_id, entry_id, acteur, aujourd_hui)
            .await?;

    // (6) Chaque facture : `paid_at` projeté, version.
    let mut invoices = Vec::with_capacity(members.len());
    let mut dues = Vec::with_capacity(members.len());
    for (_, invoice_id, number, amount) in &members {
        let due_after = super::invoice_settlements::amount_due(&mut **tx, *invoice_id).await?;
        let sql = if due_after > Decimal::ZERO {
            "UPDATE invoices SET paid_at = NULL, version = version + 1, updated_at = NOW(3) \
             WHERE id = ? AND company_id = ?"
        } else {
            "UPDATE invoices SET version = version + 1, updated_at = NOW(3) \
             WHERE id = ? AND company_id = ?"
        };
        let marked = sqlx::query(sql)
            .bind(invoice_id)
            .bind(company_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;
        if marked.rows_affected() != 1 {
            return Err(DbError::Invariant(
                "dé-rapprochement groupé : une facture n'a pas pu être marquée modifiée".into(),
            ));
        }
        invoices.push(GroupedViewInvoice {
            kind: GroupFamily::Client,
            id: *invoice_id,
            number: number.clone(),
            amount: *amount,
        });
        dues.push(due_after);
    }

    // (7) La revérification du compte, puis l'audit.
    super::bank_accounts::recheck_open_locked_in_tx(tx, company_id, bt.bank_account_id).await?;
    for ((settlement_id, invoice_id, _, amount), due_after) in members.iter().zip(&dues) {
        super::audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::par(
                acteur,
                "invoice.settlement_cancelled",
                "invoice",
                *invoice_id,
                Some(serde_json::json!({
                    "settlementId": settlement_id,
                    "settledAmount": amount,
                    "settlementJournalEntryId": entry_id,
                    "reversalJournalEntryId": reversal.entry.id,
                    "amountDueAfter": due_after,
                    "grouped": true,
                })),
            ),
        )
        .await?;
    }
    super::audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::par(
            acteur,
            "reconciliation.cancelled",
            "bank_transaction",
            bt.id,
            Some(serde_json::json!({
                "kind": "grouped_settlement",
                "matchedEntryId": entry_id,
                "reversalJournalEntryId": reversal.entry.id,
                "unlinkOnly": false,
                "invoiceId": null,
                "settlementId": null,
                "supplierInvoiceId": null,
                "invoices": members
                    .iter()
                    .map(|(s, i, n, _)| serde_json::json!({
                        "invoice_id": i,
                        "settlement_id": s,
                        "number": n,
                    }))
                    .collect::<Vec<_>>(),
                "amount": bt.amount,
                "wasPreviouslyRejected": bt.auto_match_rejected_at.is_some(),
            })),
        ),
    )
    .await?;

    Ok(GroupedCancellation {
        reversal_journal_entry_id: reversal.entry.id,
        invoices,
    })
}

/// Le bras **fournisseur** de [`cancel_in_tx`] (Story 43-1b, AC6) — l'ordre de
/// l'unitaire fournisseur (`supplier_invoices::cancel_settlement_in_tx`) :
/// **les factures avant l'écriture**. Le classement verrouillant les tient déjà
/// (la ligne `supplier_invoices` **est** la ligne de règlement) ; la lecture
/// verrouillante des membres le redit sans verrou neuf. Puis l'écriture et son
/// exercice, `ensure_open_in_tx`, et seulement alors les lectures ordinaires.
///
/// 1. tous les refus avant toute écriture : `expect_unlink_only = Some(true)`
///    → `ReconciliationCancelChanged` (un groupe n'est jamais un retrait seul,
///    C-e431a-6) ; puis la tête de chaque facture et la queue de l'écriture
///    une fois ([`supplier_group_cancel_blocker`]) ;
/// 2. le lien défait ;
/// 3. le délettrage de chaque facture ;
/// 4. chaque facture rouverte (`open`, colonnes de règlement **ligne
///    comprise** et `paid_at` à `NULL` par `CLEAR_SETTLEMENT_SQL`, `version +
///    1`, à la version lue sous verrou) : l'écriture n'a plus de propriétaire ;
/// 5. la contre-passation de l'écriture entière — y compris la ligne d'une
///    facture **détachée** auparavant (AC7), qui n'est plus rouverte ;
/// 6. `recheck_open_locked_in_tx`, puis l'audit : une
///    `supplier_invoice.settlement_cancelled` par facture rouverte, puis
///    `reconciliation.cancelled`.
async fn supplier_cancel_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bt: &crate::entities::bank_transaction::BankTransaction,
    entry_id: i64,
    acteur: Acteur,
    expect_unlink_only: Option<bool>,
    aujourd_hui: NaiveDate,
) -> Result<GroupedCancellation, DbError> {
    use super::supplier_invoices::CLEAR_SETTLEMENT_SQL;

    let tenus = locked_supplier_members(tx, company_id, entry_id).await?;
    if tenus.is_empty() {
        return Err(DbError::Invariant(format!(
            "règlement groupé : l'écriture {entry_id} n'a aucun règlement, client ni fournisseur"
        )));
    }
    // L'écriture et son exercice, avant de juger.
    let fiscal_year_id: i64 = sqlx::query_scalar(
        "SELECT fy.id FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? FOR UPDATE",
    )
    .bind(entry_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .ok_or(DbError::NotFound)?;
    super::bank_accounts::ensure_open_in_tx(tx, company_id, bt.bank_account_id).await?;
    // Les numéros et les lignes, maintenant que l'instantané naît après le verrou.
    let members = supplier_members(tx, company_id, entry_id).await?;
    let lus: Vec<i64> = members.iter().map(|m| m.id).collect();
    let ids_tenus: Vec<i64> = tenus.iter().map(|(id, _, _)| *id).collect();
    if lus != ids_tenus {
        return Err(DbError::Invariant(format!(
            "dé-rapprochement groupé fournisseur : les factures lues ({lus:?}) ne sont pas \
             celles tenues ({ids_tenus:?})"
        )));
    }

    // (1) Les refus, avant toute écriture.
    if expect_unlink_only == Some(true) {
        return Err(DbError::ReconciliationCancelChanged { unlink_only: false });
    }
    if let Some(((blocker, _, _), _)) =
        supplier_group_cancel_blocker(tx, company_id, entry_id, bt.id, aujourd_hui).await?
    {
        return Err(DbError::ReconciliationNotCancellable { blocker });
    }

    // (2) Le lien défait.
    let updated = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL, \
             version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ? AND status = 'reconciled' AND version = ?",
    )
    .bind(bt.id)
    .bind(company_id)
    .bind(bt.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if updated.rows_affected() != 1 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (3) Le délettrage de chaque pièce (l'exercice tenu : celui du règlement).
    for m in &members {
        super::letterings::dissolve_supplier_invoice_document_group_in_tx(
            tx,
            company_id,
            m.id,
            fiscal_year_id,
            acteur,
        )
        .await?;
    }

    // (4) Chaque facture rouverte, à la version lue sous verrou : l'écriture
    //     n'a plus de propriétaire.
    for (id, version, _) in &tenus {
        let rows = sqlx::query(&format!(
            "UPDATE supplier_invoices SET status = 'open', {CLEAR_SETTLEMENT_SQL}, \
             version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'paid'"
        ))
        .bind(id)
        .bind(company_id)
        .bind(version)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows != 1 {
            return Err(DbError::OptimisticLockConflict);
        }
    }

    // (5) La contre-passation de l'écriture entière.
    let reversal =
        super::journal_entries::reverse_in_tx(tx, company_id, entry_id, acteur, aujourd_hui)
            .await?;

    // (6) La revérification du compte, puis l'audit.
    super::bank_accounts::recheck_open_locked_in_tx(tx, company_id, bt.bank_account_id).await?;
    for (m, (_, _, paid_at)) in members.iter().zip(&tenus) {
        super::audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::par(
                acteur,
                "supplier_invoice.settlement_cancelled",
                "supplier_invoice",
                m.id,
                Some(serde_json::json!({
                    "settlementType": "bank_transfer",
                    "settlementJournalEntryId": entry_id,
                    "settlementJournalEntryLineId": m.line_id,
                    "reversalJournalEntryId": reversal.entry.id,
                    "paidAt": paid_at,
                    "grouped": true,
                })),
            ),
        )
        .await?;
    }
    super::audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::par(
            acteur,
            "reconciliation.cancelled",
            "bank_transaction",
            bt.id,
            Some(serde_json::json!({
                "kind": "grouped_settlement",
                "matchedEntryId": entry_id,
                "reversalJournalEntryId": reversal.entry.id,
                "unlinkOnly": false,
                "invoiceId": null,
                "settlementId": null,
                "supplierInvoiceId": null,
                "supplier_invoices": members
                    .iter()
                    .map(|m| serde_json::json!({
                        "supplier_invoice_id": m.id,
                        "number": m.number,
                    }))
                    .collect::<Vec<_>>(),
                "amount": bt.amount,
                "wasPreviouslyRejected": bt.auto_match_rejected_at.is_some(),
            })),
        ),
    )
    .await?;

    Ok(GroupedCancellation {
        reversal_journal_entry_id: reversal.entry.id,
        invoices: members
            .into_iter()
            .map(|m| GroupedViewInvoice {
                kind: GroupFamily::Supplier,
                id: m.id,
                number: m.number,
                amount: m.amount,
            })
            .collect(),
    })
}

// ---------------------------------------------------------------------------
// Le règlement groupé fournisseur — Story 43-1b
// ---------------------------------------------------------------------------

/// Une facture fournisseur lue et verrouillée à l'étape 4, ce que la suite en
/// emploie.
struct SupplierMember {
    invoice: SupplierInvoice,
    /// Le compte de dette, lu sur l'écriture d'achat (`purchase_payable_line`).
    payable_account_id: i64,
    /// Le numéro affichable : interne, à défaut celui du fournisseur.
    number: Option<String>,
}

impl SupplierMember {
    fn piece(&self) -> GroupedPiece {
        GroupedPiece {
            document: GroupedDocument::SupplierInvoice(self.invoice.id),
            number: self.number.clone(),
        }
    }
}

/// Le numéro affichable d'une facture fournisseur : le numéro interne, à
/// défaut celui du fournisseur (`GroupedPiece`, 43-3).
fn supplier_number(invoice: &SupplierInvoice) -> Option<String> {
    invoice
        .internal_number
        .clone()
        .or_else(|| invoice.supplier_invoice_number.clone())
}

/// **Paie plusieurs factures fournisseur par une transaction au débit**, dans
/// la transaction de l'appelant et sous son verrou de compte (Story 43-1b,
/// #823) — signature figée à T3 de la 43-1a.
///
/// La forme est celle de [`settle_invoices_in_tx`] — même point de sauvegarde
/// (D6), mêmes bornes et doublon, même [`group_difference`] —, la méthode celle
/// de **l'unitaire fournisseur** (`accept_one_supplier_invoice`, C-43-19) :
/// chaque facture est verrouillée et admise par
/// [`super::supplier_invoices::lock_for_bank_settlement_in_tx`] (**réemployée**),
/// par identifiant croissant ; son `UPDATE … AND version = ? AND status =
/// 'open'` porte la version lue **sous ce verrou**.
///
/// ⛔ **Ordre des verrous : facture, compte, lot** — celui de `pay_in_tx` et de
/// `create_batch` (revue P1, B-1, E2) : **toutes** les factures sont
/// verrouillées d'abord, par identifiant croissant (la lecture de tête de
/// `create_batch`), avant que la première `lock_for_bank_settlement_in_tx` ne
/// prenne le compte ; sans cela, le groupe tiendrait le compte en attendant la
/// facture suivante, et un paiement depuis sa fiche formerait un cycle (1213).
/// Le compte du grand livre de la banque est celui lu sous le verrou du compte
/// (`LockedForSettlement::bank_ledger_account_id`).
///
/// L'écriture (sans projet d'en-tête) : la banque **au crédit** de |montant| ;
/// par facture, le **débit** de son compte de dette (lu sur l'écriture d'achat)
/// de son `total_amount`, avec le projet de la facture estampillé sans
/// validation (C-43-20) ; l'arrondi `r` (au débit si `r > 0`) ; l'écart `e` au
/// compte choisi, **au débit si `e > 0`** (plus payé que les factures), au
/// crédit sinon — le symétrique du côté client. Chaque facture porte sa ligne
/// (`settlement_journal_entry_line_id`), ce qui fait d'elle un membre du
/// groupe (`reconciliation_cancel::classify`).
///
/// Audit : `reconciliation.accepted` (`proposal_type: "supplier_invoice_group"`)
/// puis `supplier_invoice.paid` par facture, avec l'acteur. `aujourd_hui`
/// n'est pas lu, comme dans [`settle_invoices_in_tx`].
#[allow(clippy::too_many_arguments)]
pub async fn pay_supplier_invoices_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    bank_transaction_id: i64,
    supplier_invoice_ids: &[i64],
    difference_account_id: Option<i64>,
    batch_size: i64,
    acteur: Acteur,
    aujourd_hui: NaiveDate,
) -> Result<Result<GroupedSettlement, GroupRefusal>, DbError> {
    let _ = aujourd_hui;
    // (1) Le point de sauvegarde propre.
    savepoint(tx).await?;
    // (2) Bornes et doublon, avant toute lecture.
    if let Some(r) = check_bounds(GroupFamily::Supplier, supplier_invoice_ids) {
        return refuse(tx, r).await;
    }
    match pay_supplier_inner(
        tx,
        company_id,
        bank_account_id,
        bank_transaction_id,
        supplier_invoice_ids,
        difference_account_id,
        batch_size,
        acteur,
    )
    .await?
    {
        Ok(done) => {
            sqlx::query(&format!("RELEASE SAVEPOINT {SAVEPOINT}"))
                .execute(&mut **tx)
                .await
                .map_err(DbError::Sqlx)?;
            Ok(Ok(done))
        }
        Err(r) => refuse(tx, r).await,
    }
}

/// Le corps de [`pay_supplier_invoices_in_tx`], étapes 3 à 11 : rend le refus
/// sans revenir au point de sauvegarde (l'enveloppe s'en charge).
#[allow(clippy::too_many_arguments)]
async fn pay_supplier_inner(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    bank_transaction_id: i64,
    supplier_invoice_ids: &[i64],
    difference_account_id: Option<i64>,
    batch_size: i64,
    acteur: Acteur,
) -> Result<Result<GroupedSettlement, GroupRefusal>, DbError> {
    use super::reconciliation::{
        InvoiceDateRefusal, InvoiceDateWindow, find_pending_by_id_for_account, payment_date,
    };
    use crate::entities::bank_transaction::BankTransactionStatus;

    // (3) La transaction, rechargée sans verrou (patron de l'unitaire).
    let Some(bt) =
        find_pending_by_id_for_account(&mut **tx, company_id, bank_account_id, bank_transaction_id)
            .await?
    else {
        return Ok(Err(GroupRefusal::TransactionNotFound));
    };
    if bt.status != BankTransactionStatus::Pending {
        return Ok(Err(GroupRefusal::TransactionNotPending));
    }
    // Un débit : ni crédit ni montant nul ; la devise ensuite.
    if bt.amount >= Decimal::ZERO {
        return Ok(Err(GroupRefusal::WrongDirection { amount: bt.amount }));
    }
    if !super::reconciliation_link::transaction_can_be_linked_to_entry(
        &bt,
        super::bank_accounts::BASE_CURRENCY,
    ) {
        return Ok(Err(GroupRefusal::CurrencyMismatch {
            currency: bt.currency.clone(),
        }));
    }
    let paid = bt.amount.abs();
    let value_date = payment_date(&bt);
    let window = InvoiceDateWindow::for_supplier_transaction(&bt);

    // (4) L'ordre des verrous de l'unitaire et de `create_batch` — **facture,
    //     compte, lot** (revue P1, B-1, E2) : toutes les factures d'abord, par
    //     identifiant croissant (la lecture de tête de `create_batch`, étape
    //     −1, réemployée) ; `lock_for_bank_settlement_in_tx` les redemande
    //     ensuite, déjà tenues, avant le compte. Sans cela, le groupe tiendrait
    //     le compte (pris avec la facture 1) en attendant la facture 2, et un
    //     paiement depuis la fiche de la facture 2 formerait un cycle (1213).
    let mut ids: Vec<i64> = supplier_invoice_ids.to_vec();
    ids.sort_unstable();
    {
        let sql = super::payment_batches::lock_invoices_sql(ids.len());
        let mut q = sqlx::query(&sql);
        for id in &ids {
            q = q.bind(id);
        }
        q.bind(company_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(map_db_error)?;
    }
    //     Puis chaque facture, admise par la méthode de l'unitaire ; la borne
    //     basse de date ; sa ligne de dette.
    let mut members: Vec<SupplierMember> = Vec::with_capacity(ids.len());
    let mut bank_ledger_account_id = None;
    for id in &ids {
        let locked = super::supplier_invoices::lock_for_bank_settlement_in_tx(
            tx,
            company_id,
            *id,
            bank_account_id,
            paid,
        )
        .await?;
        let invoice = match locked {
            Ok(l) => {
                bank_ledger_account_id = Some(l.bank_ledger_account_id);
                l.invoice
            }
            Err(refusal) => {
                return Ok(Err(
                    supplier_lock_refusal(tx, company_id, *id, refusal).await?
                ));
            }
        };
        let number = supplier_number(&invoice);
        let piece = GroupedPiece {
            document: GroupedDocument::SupplierInvoice(*id),
            number: number.clone(),
        };
        // La borne basse seule (C-43-7) : `check` rend `PaymentBeforeInvoice`
        // avant `PaymentTooLate`, borne haute ignorée.
        if let Err(InvoiceDateRefusal::PaymentBeforeInvoice) =
            window.check(invoice.invoice_date, invoice.invoice_date)
        {
            return Ok(Err(GroupRefusal::PaymentBeforeInvoiceDate {
                piece,
                tolerance_days: window.tolerance_days,
            }));
        }
        // La dette, lue sur l'écriture d'achat (comme `pay_in_tx_as`) ; son
        // absence est un invariant rompu, comme dans l'unitaire.
        let purchase_entry_id = invoice.purchase_journal_entry_id.ok_or_else(|| {
            DbError::Invariant("facture fournisseur validée sans écriture d'achat".into())
        })?;
        let (payable_account_id, payable_amount) =
            super::supplier_invoices::purchase_payable_line(tx, company_id, purchase_entry_id)
                .await?
                .ok_or_else(|| {
                    DbError::Invariant("écriture d'achat sans ligne de crédit créanciers".into())
                })?;
        // La ligne de la dette doit solder ce que la facture doit : le lettrage
        // (achat, règlement) et la vue (apports = `total_amount`) en dépendent.
        if payable_amount != invoice.total_amount {
            return Err(DbError::Invariant(format!(
                "règlement groupé : la facture fournisseur {id} doit {} mais son écriture \
                 d'achat crédite {payable_amount}",
                invoice.total_amount
            )));
        }
        members.push(SupplierMember {
            invoice,
            payable_account_id,
            number,
        });
    }

    // (5) Le compte bancaire a été verrouillé et contrôlé (archivé, clos, sans
    //     compte du grand livre) par `lock_for_bank_settlement_in_tx` : son
    //     compte du grand livre est celui **lu sous ce verrou** (revue P1, E3).
    //     `ensure_open_in_tx` (lecture ordinaire) ne refuse rien de plus que
    //     cette lecture verrouillante — sauf, à tort, un compte rouvert après la
    //     naissance de l'instantané (fenêtre étroite) ; il reste parce que
    //     l'inventaire du compte clos n'admet la revérification de l'AC3
    //     (`recheck_open_locked_in_tx`, en dernier) que derrière une garde non
    //     verrouillante (C-e431b-6). Puis la banque n'est la dette d'aucune
    //     facture.
    match super::bank_accounts::ensure_open_in_tx(tx, company_id, bank_account_id).await {
        Ok(()) => {}
        Err(DbError::BankAccountClosed { .. }) => {
            return Ok(Err(GroupRefusal::BankAccountClosed));
        }
        Err(DbError::NotFound) => return Ok(Err(GroupRefusal::BankAccountNotFound)),
        Err(e) => return Err(e),
    }
    let Some(bank_ledger_account_id) = bank_ledger_account_id else {
        return Err(DbError::Invariant(
            "règlement groupé fournisseur : aucune facture verrouillée".into(),
        ));
    };
    let payables: BTreeSet<i64> = members.iter().map(|m| m.payable_account_id).collect();
    if let Some(m) = members
        .iter()
        .find(|m| m.payable_account_id == bank_ledger_account_id)
    {
        return Ok(Err(claim_refusal_on(
            tx,
            company_id,
            ClaimSide::Payable,
            SettlementAccountRole::Counterparty,
            bank_ledger_account_id,
            Some(bank_account_id),
            Some(m.piece()),
        )
        .await?));
    }

    // (6) La différence et ses comptes — la règle commune aux deux familles
    //     (`difference_accounts` : le plafond d'abord), côté dette.
    let gross: Vec<Decimal> = members.iter().map(|m| m.invoice.total_amount).collect();
    let accounts = match difference_accounts(
        tx,
        company_id,
        bt.amount,
        &gross,
        &payables,
        ClaimSide::Payable,
        bank_ledger_account_id,
        difference_account_id,
    )
    .await?
    {
        Ok(a) => a,
        Err(r) => return Ok(Err(r)),
    };

    // (7) L'écriture : banque (au crédit), une dette par facture (identifiant
    //     croissant), arrondi, écart ; exercice ouvert couvrant la date de
    //     valeur.
    let mut lines = vec![NewJournalEntryLine {
        account_id: bank_ledger_account_id,
        debit: Decimal::ZERO,
        credit: paid,
        project_id: None,
        // Chemin automatique : CHF seulement (Epic 37, `supplier_invoices`).
        foreign: None,
    }];
    for m in &members {
        lines.push(NewJournalEntryLine {
            account_id: m.payable_account_id,
            debit: m.invoice.total_amount,
            credit: Decimal::ZERO,
            // Le projet de la facture, estampillé sans validation (C-43-20).
            project_id: m.invoice.project_id,
            foreign: None,
        });
    }
    // Fournisseur : la banque est au crédit ; `r > 0` et `e > 0` (plus payé
    // que les factures) au débit, négatifs au crédit — le côté qui équilibre.
    push_difference_lines(&mut lines, &accounts, false);
    let numbers: Vec<String> = members
        .iter()
        .map(|m| m.number.clone().unwrap_or_else(|| m.invoice.id.to_string()))
        .collect();
    let description = bounded_description(format!(
        "Paiement groupé de {} factures fournisseur : {}",
        members.len(),
        numbers.join(", ")
    ));
    let (entry, fiscal_year_id) =
        match create_group_entry(tx, company_id, acteur, value_date, description, lines).await? {
            Ok(done) => done,
            Err(r) => return Ok(Err(r)),
        };
    let DifferenceAccounts {
        decomposition,
        difference_account_id: difference_line_account_id,
        ..
    } = accounts;
    let journal_entry_id = entry.entry.id;
    // Attribution ligne → facture par la position d'insertion (D2) : la ligne
    // `k + 1` (0 = banque) est celle de la facture `k`.
    if entry.lines.len() < members.len() + 1 {
        return Err(DbError::Invariant(
            "règlement groupé fournisseur : l'écriture n'a pas une ligne par facture".into(),
        ));
    }
    let line_ids: Vec<i64> = entry.lines[1..=members.len()]
        .iter()
        .map(|l| l.id)
        .collect();

    // (8) Chaque facture : `paid`, son règlement et SA ligne, à la version lue
    //     sous le verrou de l'étape 4 (AC3).
    let paid_at = value_date
        .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).expect("minuit est une heure valide"));
    let mut refs: Vec<GroupedInvoiceRef> = Vec::with_capacity(members.len());
    for (m, line_id) in members.iter().zip(&line_ids) {
        let updated = sqlx::query(
            "UPDATE supplier_invoices SET status = 'paid', settlement_type = 'bank_transfer', \
             settlement_bank_account_id = ?, settlement_account_id = NULL, \
             settlement_journal_entry_id = ?, settlement_journal_entry_line_id = ?, \
             paid_at = ?, version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'open'",
        )
        .bind(bank_account_id)
        .bind(journal_entry_id)
        .bind(line_id)
        .bind(paid_at)
        .bind(m.invoice.id)
        .bind(company_id)
        .bind(m.invoice.version)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
        if updated.rows_affected() != 1 {
            return Ok(Err(GroupRefusal::InvoiceNotEligible {
                piece: m.piece(),
                reason: IneligibleReason::RaceDuringUpdate,
            }));
        }
        refs.push(GroupedInvoiceRef {
            kind: GroupFamily::Supplier,
            id: m.invoice.id,
            number: m.number.clone(),
            settlement_id: None,
            line_id: *line_id,
            amount: m.invoice.total_amount,
        });
    }

    // (9) La transaction.
    let updated = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = ?, status = 'reconciled', updated_at = NOW(3), \
             version = version + 1 \
         WHERE id = ? AND company_id = ? AND status = 'pending' AND matched_entry_id IS NULL \
           AND version = ?",
    )
    .bind(journal_entry_id)
    .bind(bank_transaction_id)
    .bind(company_id)
    .bind(bt.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if updated.rows_affected() != 1 {
        return Ok(Err(GroupRefusal::TransactionNotPending));
    }

    // (10) Le lettrage de chaque facture (sa ligne d'achat, sa ligne de dette),
    //      puis la revérification du compte, dernière acquisition de verrou.
    for m in &members {
        super::letterings::sync_supplier_invoice_in_tx(
            tx,
            company_id,
            m.invoice.id,
            fiscal_year_id,
            acteur,
        )
        .await?;
    }
    super::bank_accounts::recheck_open_locked_in_tx(tx, company_id, bank_account_id).await?;

    // (11) L'audit : le rapprochement d'abord, puis le paiement de chaque
    //      facture (les détails de `pay_in_tx_as`, et le groupe).
    let accepted = super::audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::par(
            acteur,
            "reconciliation.accepted",
            "bank_transaction",
            bank_transaction_id,
            Some(serde_json::json!({
                "proposal_type": "supplier_invoice_group",
                "bank_transaction_id": bank_transaction_id,
                "journal_entry_id": journal_entry_id,
                "batch_size": batch_size,
                "paid_amount": paid,
                "transaction_amount": bt.amount,
                "difference": decomposition.difference,
                "rounding_difference": decomposition.rounding_difference,
                "gap": decomposition.gap,
                "difference_account_id": difference_line_account_id,
                "supplier_invoices": refs
                    .iter()
                    .map(|i| serde_json::json!({
                        "supplier_invoice_id": i.id,
                        "settled_amount": i.amount,
                    }))
                    .collect::<Vec<_>>(),
            })),
        ),
    )
    .await?;
    for r in &refs {
        super::audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::par(
                acteur,
                "supplier_invoice.paid",
                "supplier_invoice",
                r.id,
                Some(serde_json::json!({
                    "settlementType": "bank_transfer",
                    "settlementJournalEntryId": journal_entry_id,
                    "settlementJournalEntryLineId": r.line_id,
                    "amount": r.amount.to_string(),
                    "reconciliationAuditId": accepted.id,
                    "grouped": true,
                })),
            ),
        )
        .await?;
    }

    Ok(Ok(GroupedSettlement {
        journal_entry_id,
        reconciliation_audit_id: accepted.id,
        decomposition,
        difference_account_id: difference_line_account_id,
        invoices: refs,
    }))
}

/// Un refus de [`super::supplier_invoices::lock_for_bank_settlement_in_tx`] en
/// refus du groupe : la facture `id` en cause (ou, pour la garde du lot du
/// compte, l'**autre** facture qui la tient, avec son numéro).
async fn supplier_lock_refusal(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    refusal: super::supplier_invoices::SupplierSettlementRefusal,
) -> Result<GroupRefusal, DbError> {
    use super::supplier_invoices::SupplierSettlementRefusal as R;
    // Le numéro d'une pièce verrouillée (ou lue) — pour que l'écran la nomme.
    async fn piece_of(
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        company_id: i64,
        id: i64,
    ) -> Result<GroupedPiece, DbError> {
        let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT internal_number, supplier_invoice_number FROM supplier_invoices \
             WHERE id = ? AND company_id = ?",
        )
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
        Ok(GroupedPiece {
            document: GroupedDocument::SupplierInvoice(id),
            number: row.and_then(|(interne, fournisseur)| interne.or(fournisseur)),
        })
    }
    Ok(match refusal {
        R::NotFound => GroupRefusal::InvoiceNotFound {
            document: GroupedDocument::SupplierInvoice(id),
        },
        R::NotOpen { .. } => GroupRefusal::InvoiceNotEligible {
            piece: piece_of(tx, company_id, id).await?,
            reason: IneligibleReason::SupplierNotOpen,
        },
        R::BankAccountNotFound => GroupRefusal::BankAccountNotFound,
        R::BankAccountClosed => GroupRefusal::BankAccountClosed,
        R::BankAccountNotConfigured => GroupRefusal::BankAccountNotConfigured,
        R::InGeneratedBatch { .. } => GroupRefusal::InvoiceNotEligible {
            piece: piece_of(tx, company_id, id).await?,
            reason: IneligibleReason::SupplierInPaymentBatch,
        },
        R::HeldByPaymentBatch {
            supplier_invoice_id,
            payment_batch_id,
        } => GroupRefusal::HeldByPaymentBatch {
            piece: piece_of(tx, company_id, supplier_invoice_id).await?,
            payment_batch_id: Some(payment_batch_id),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn client_exact_amount_has_neither_rounding_nor_fees() {
        let d = group_difference(dec!(300.00), &[dec!(100.00), dec!(200.00)], dec!(5)).unwrap();
        assert_eq!(
            d,
            Decomposition {
                difference: Decimal::ZERO,
                rounding_difference: Decimal::ZERO,
                gap: Decimal::ZERO,
            }
        );
    }

    /// Les six cas du tableau de la décision (`kesh-gate-logs/e43/
    /// decision-difference.md`, § « Décomposition de l'écart ») :
    /// `(restes, payé, d, e, r)`, et l'identité `d = r + e`.
    #[test]
    fn the_six_cases_of_the_decision_table() {
        let cas: [(Vec<Decimal>, Decimal, Decimal, Decimal, Decimal); 6] = [
            (
                vec![dec!(10.0050), dec!(10.0050)],
                dec!(20.01),
                dec!(0),
                dec!(0),
                dec!(0),
            ),
            (
                vec![dec!(10.0050), dec!(10.0050)],
                dec!(20.02),
                dec!(0.0100),
                dec!(0.01),
                dec!(0),
            ),
            (
                vec![dec!(500.00), dec!(300.00)],
                dec!(799.99),
                dec!(-0.0100),
                dec!(-0.01),
                dec!(0),
            ),
            (
                vec![dec!(10.0050), dec!(10.0050), dec!(10.0050)],
                dec!(30.06),
                dec!(0.0450),
                dec!(0.05),
                dec!(-0.0050),
            ),
            (
                vec![dec!(10.0049), dec!(10.0000)],
                dec!(20.00),
                dec!(-0.0049),
                dec!(0),
                dec!(-0.0049),
            ),
            (
                vec![dec!(10.0050), dec!(10.0000)],
                dec!(20.00),
                dec!(-0.0050),
                dec!(-0.01),
                dec!(0.0050),
            ),
        ];
        for (restes, paye, d, e, r) in cas {
            let got = group_difference(paye, &restes, dec!(5)).unwrap();
            assert_eq!(got.difference, d, "d pour {paye} sur {restes:?}");
            assert_eq!(got.gap, e, "e pour {paye} sur {restes:?}");
            assert_eq!(got.rounding_difference, r, "r pour {paye} sur {restes:?}");
            assert_eq!(got.difference, got.rounding_difference + got.gap);
            assert!(got.rounding_difference.abs() <= dec!(0.005));
            // L'arrondi automatique est le cas `e = 0` : il passe sous un
            // plafond nul ; tout `e ≠ 0` y est refusé.
            assert_eq!(
                group_difference(paye, &restes, dec!(0)).is_ok(),
                e.is_zero(),
                "plafond nul pour {paye}"
            );
        }
        // Un centime n'est jamais absorbé, même à cinquante factures.
        let cinquante = vec![dec!(10.00); 50];
        assert_eq!(
            group_difference(dec!(499.99), &cinquante, dec!(5))
                .unwrap()
                .gap,
            dec!(-0.01)
        );
    }

    /// Décision de Guy du 2026-11-17 : moins reçu ET trop-perçu admis, dans la
    /// limite du plafond ; seul le plafond refuse.
    #[test]
    fn both_directions_are_admitted_within_the_ceiling() {
        let f = [dec!(100.00), dec!(200.00)];
        // Moins reçu.
        let d = group_difference(dec!(299.60), &f, dec!(5)).unwrap();
        assert_eq!(d.gap, dec!(-0.40));
        assert_eq!(d.difference, dec!(-0.40));
        assert_eq!(
            group_difference(dec!(295.00), &f, dec!(5)).unwrap().gap,
            dec!(-5.00)
        );
        // Trop-perçu.
        let d = group_difference(dec!(300.05), &f, dec!(5)).unwrap();
        assert_eq!(d.gap, dec!(0.05));
        assert_eq!(
            group_difference(dec!(305.00), &f, dec!(5)).unwrap().gap,
            dec!(5.00)
        );
        // Au-delà du plafond, dans les deux sens.
        assert_eq!(
            group_difference(dec!(294.99), &f, dec!(5)),
            Err(GroupRefusal::DifferenceTooLarge {
                transaction_amount: dec!(294.99),
                sum_applied: dec!(300.00),
                gap: dec!(-5.01),
                max_difference: dec!(5),
            })
        );
        assert!(matches!(
            group_difference(dec!(305.01), &f, dec!(5)),
            Err(GroupRefusal::DifferenceTooLarge { .. })
        ));
        // Plafond à zéro : tout écart d'un centime ou plus est refusé.
        assert!(group_difference(dec!(299.99), &f, dec!(0)).is_err());
    }

    /// Fournisseur : la même règle, sur `|montant|` (transaction au débit).
    #[test]
    fn supplier_amount_is_taken_in_absolute_value() {
        let f = [dec!(300.00), dec!(500.00)];
        let d = group_difference(dec!(-800.40), &f, dec!(5)).unwrap();
        assert_eq!(d.gap, dec!(0.40));
        assert_eq!(d.difference, dec!(0.40));
        let d = group_difference(dec!(-799.60), &f, dec!(5)).unwrap();
        assert_eq!(d.gap, dec!(-0.40));
        assert!(matches!(
            group_difference(dec!(-805.01), &f, dec!(5)),
            Err(GroupRefusal::DifferenceTooLarge { .. })
        ));
    }

    /// D6 — un `ROLLBACK TO SAVEPOINT grouped_settlement` qui échoue (point de
    /// sauvegarde disparu : 1305) remonte en `Err(DbError)`, jamais suivi du
    /// refus, et [`is_savepoint_lost`] le reconnaît. Sonde : la transaction
    /// n'a jamais posé le point de sauvegarde.
    #[sqlx::test(migrations = "./test-schema")]
    async fn a_lost_savepoint_surfaces_as_an_error(pool: sqlx::MySqlPool) {
        let mut tx = pool.begin().await.expect("transaction");
        let out = refuse::<()>(&mut tx, GroupRefusal::TransactionNotFound).await;
        let err = out.expect_err("le refus ne doit pas masquer la perte du point de sauvegarde");
        assert!(is_savepoint_lost(&err), "erreur inattendue : {err:?}");
        // Sonde positive inverse : un point de sauvegarde posé se rejoint, le
        // refus est rendu.
        savepoint(&mut tx).await.expect("point de sauvegarde");
        let out = refuse::<()>(&mut tx, GroupRefusal::TransactionNotFound)
            .await
            .expect("retour au point de sauvegarde");
        assert_eq!(out, Err(GroupRefusal::TransactionNotFound));
        assert!(!is_savepoint_lost(&DbError::NotFound));
    }

    #[test]
    fn bounds_and_duplicates_are_refused_before_any_read() {
        assert_eq!(
            check_bounds(GroupFamily::Client, &[1]),
            Some(GroupRefusal::TooFew { count: 1 })
        );
        let many: Vec<i64> = (1..=51).collect();
        assert_eq!(
            check_bounds(GroupFamily::Client, &many),
            Some(GroupRefusal::TooMany { count: 51 })
        );
        assert_eq!(
            check_bounds(GroupFamily::Supplier, &[3, 4, 3]),
            Some(GroupRefusal::Duplicate {
                document: GroupedDocument::SupplierInvoice(3)
            })
        );
        assert_eq!(check_bounds(GroupFamily::Client, &[1, 2]), None);
    }
}
