//! Un règlement pour plusieurs factures — Story 43-1a (#823), Epic 43.
//!
//! Une transaction bancaire règle **plusieurs factures** d'un seul geste : **une**
//! écriture — une ligne de banque, une ligne de tiers **par facture** (le projet
//! de sa facture), au plus une ligne d'arrondi et une de frais bancaires —, que la
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
use crate::entities::{Invoice, NewInvoiceSettlement, NewJournalEntry, NewJournalEntryLine};
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
    ClaimAccount {
        claim: ClaimSide,
        role: SettlementAccountRole,
        account_id: i64,
        account_number: Option<String>,
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
    /// banque de la transaction.
    DifferenceAccountIsBankLedger { account_id: i64 },
    /// Le compte choisi pour l'écart n'est pas imputable.
    DifferenceAccountNotPostable { account_id: i64 },
    /// Le compte choisi pour l'écart est introuvable, d'une autre société, ou
    /// archivé.
    DifferenceAccountInvalid { account_id: i64 },
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
/// - `difference = |montant| − Σ apports bruts` (signé : négatif quand moins
///   reçu ou payé, positif quand plus ; brut, quatre décimales) ;
/// - `rounding_difference = r = Σ (centime(apport) − apport)` (signé, brut) —
///   les fractions de centime, au compte d'arrondi comme dans l'unitaire ;
/// - `gap = e = |montant| − Σ centime(apport)` (signé, au centime) — le
///   moins-payé (`e < 0`) ou le trop-perçu (`e > 0`), passé au compte que
///   **l'utilisateur choisit** (décision de Guy du 2026-11-17).
///
/// Identité : `difference = e + r`.
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
/// L'écart `e = |amount| − Σ centime(apport)` est admis **dans les deux sens** ;
/// seul `|e| >` plafond est refusé ([`GroupRefusal::DifferenceTooLarge`]). Les
/// fractions de centime (`r`) n'entrent pas dans le plafond.
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
    let centimes: Decimal = gross_contributions
        .iter()
        .map(|c| amount_due_to_centime(*c))
        .sum();
    let e = paid - centimes;
    if e.abs() > max_difference {
        return Err(GroupRefusal::DifferenceTooLarge {
            transaction_amount: amount,
            sum_applied: centimes,
            gap: e,
            max_difference,
        });
    }
    Ok(Decomposition {
        difference: paid - gross,
        rounding_difference: centimes - gross,
        gap: e,
    })
}

/// Le compte choisi pour l'écart, **contrôlé** (décision de Guy du
/// 2026-11-17) : de la société et actif (sinon
/// [`GroupRefusal::DifferenceAccountInvalid`]), imputable (sinon
/// [`GroupRefusal::DifferenceAccountNotPostable`]) — sans restriction de
/// classe —, verrouillé `FOR UPDATE` comme un compte désigné (une désactivation
/// concurrente attend la fin du geste). `None` : le compte convient. La
/// comparaison aux comptes de tiers et à la banque est faite par l'appelant,
/// qui les connaît.
pub(crate) async fn difference_account_refusal(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<Option<GroupRefusal>, DbError> {
    let row: Option<(bool, bool)> = sqlx::query_as(
        "SELECT active, postable FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(account_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    Ok(match row {
        None | Some((false, _)) => Some(GroupRefusal::DifferenceAccountInvalid { account_id }),
        Some((true, false)) => Some(GroupRefusal::DifferenceAccountNotPostable { account_id }),
        Some((true, true)) => None,
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
        Ok(done) => Ok(Ok(done)),
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
    if !super::reconciliation_link::transaction_can_be_linked_to_entry(&bt) {
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
        let Some(sale_entry_id) = invoice.journal_entry_id else {
            return not_eligible(IneligibleReason::JournalEntryNotSet);
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
        let raw_due = super::invoice_settlements::amount_due(&mut **tx, *id).await?;
        if raw_due <= Decimal::ZERO {
            return not_eligible(IneligibleReason::NothingDue);
        }
        let Some(receivable_account_id) =
            super::invoice_settlements::sale_receivable_account(tx, company_id, sale_entry_id)
                .await?
        else {
            return Ok(Err(GroupRefusal::SaleEntryMalformed {
                piece,
                sale_entry_id,
            }));
        };
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
            Some(GroupedPiece {
                document: GroupedDocument::Invoice(m.invoice.id),
                number: m.invoice.invoice_number.clone(),
            }),
        )
        .await?));
    }

    // (6) La différence (C-43-16, D4 bis), puis les comptes qu'elle exige.
    let max =
        super::company_invoice_settings::group_settlement_max_difference(&mut **tx, company_id)
            .await?;
    let gross: Vec<Decimal> = members.iter().map(|m| m.raw_due).collect();
    let decomposition = match group_difference(bt.amount, &gross, max) {
        Ok(d) => d,
        Err(r) => return Ok(Err(r)),
    };
    let r = decomposition.rounding_difference;
    let rounding_account_id = if r.is_zero() {
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
                // Comme l'unitaire (c-bis) : le compte d'arrondi n'est pas la
                // créance.
                if receivables.contains(&id) {
                    return Ok(Err(claim_refusal(
                        tx,
                        company_id,
                        SettlementAccountRole::Rounding,
                        id,
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
    // L'écart `e` (moins-payé ou trop-perçu) : au compte que l'utilisateur
    // choisit — imputable, actif, de la société, ni compte de tiers ni banque.
    let e = decomposition.gap;
    let difference_line_account_id = if e.is_zero() {
        None
    } else {
        let Some(id) = difference_account_id else {
            return Ok(Err(GroupRefusal::DifferenceAccountRequired { gap: e }));
        };
        if let Some(refusal) = difference_account_refusal(tx, company_id, id).await? {
            return Ok(Err(refusal));
        }
        if id == bank_ledger_account_id {
            return Ok(Err(GroupRefusal::DifferenceAccountIsBankLedger {
                account_id: id,
            }));
        }
        if receivables.contains(&id) {
            return Ok(Err(claim_refusal(
                tx,
                company_id,
                SettlementAccountRole::Counterparty,
                id,
                None,
            )
            .await?));
        }
        Some(id)
    };

    // (7) L'écriture : banque, une créance par facture (identifiant croissant),
    //     arrondi, frais (D2). Exercice ouvert couvrant la date de valeur.
    let Some(fiscal_year) =
        super::fiscal_years::find_open_covering_date(tx, company_id, value_date).await?
    else {
        return Ok(Err(GroupRefusal::FiscalYearInvalid { value_date }));
    };
    let mut lines = vec![NewJournalEntryLine {
        account_id: bank_ledger_account_id,
        debit: bt.amount,
        credit: Decimal::ZERO,
        project_id: None,
    }];
    for m in &members {
        lines.push(NewJournalEntryLine {
            account_id: m.receivable_account_id,
            debit: Decimal::ZERO,
            credit: m.raw_due,
            // Le projet de la facture, estampillé sans validation (D7, C-43-20).
            project_id: m.invoice.project_id,
        });
    }
    if let Some(account_id) = rounding_account_id {
        lines.push(NewJournalEntryLine {
            account_id,
            debit: if r < Decimal::ZERO { -r } else { Decimal::ZERO },
            credit: if r > Decimal::ZERO { r } else { Decimal::ZERO },
            project_id: None,
        });
    }
    // Client : moins reçu (`e < 0`) au débit du compte choisi, trop-perçu
    // (`e > 0`) au crédit — le côté qui équilibre l'écriture.
    if let Some(account_id) = difference_line_account_id {
        lines.push(NewJournalEntryLine {
            account_id,
            debit: if e < Decimal::ZERO { -e } else { Decimal::ZERO },
            credit: if e > Decimal::ZERO { e } else { Decimal::ZERO },
            project_id: None,
        });
    }
    let new_entry = NewJournalEntry {
        company_id,
        entry_date: value_date,
        journal: crate::entities::journal_entry::Journal::Banque,
        description: entry_description(&members),
        project_id: None,
        lines,
    };
    let entry = match super::journal_entries::create_copying_projects_in_tx(
        tx,
        fiscal_year.id,
        acteur,
        new_entry,
        false,
    )
    .await
    {
        Ok(e) => e,
        Err(DbError::PeriodLocked {
            locked_through,
            attempted,
        }) => {
            return Ok(Err(GroupRefusal::PeriodLocked {
                locked_through,
                attempted,
            }));
        }
        Err(DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        }) => {
            return Ok(Err(GroupRefusal::LaterFiscalYearClosed {
                fiscal_year_id,
                fiscal_year_name,
            }));
        }
        Err(DbError::FiscalYearClosed | DbError::DateOutsideFiscalYear) => {
            return Ok(Err(GroupRefusal::FiscalYearInvalid { value_date }));
        }
        Err(e) => return Err(e),
    };
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
        super::letterings::sync_invoice_in_tx(tx, company_id, m.invoice.id, fiscal_year.id, acteur)
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
    let full = format!(
        "Encaissement groupé de {} factures : {}",
        members.len(),
        numbers.join(", ")
    );
    if full.chars().count() <= 500 {
        full
    } else {
        let mut cut: String = full.chars().take(497).collect();
        cut.push_str("...");
        cut
    }
}

/// Construit le refus de compte de tiers, numéro du compte lu à l'échec.
async fn claim_refusal(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    role: SettlementAccountRole,
    account_id: i64,
    piece: Option<GroupedPiece>,
) -> Result<GroupRefusal, DbError> {
    let account_number = super::accounts::number_in_company(tx, company_id, account_id).await?;
    Ok(GroupRefusal::ClaimAccount {
        claim: ClaimSide::Receivable,
        role,
        account_id,
        account_number,
        piece,
    })
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

/// Les factures clients d'une écriture groupée, par identifiant croissant —
/// lecture **ordinaire** (vue) ou **verrouillante** (geste).
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

/// Les factures d'une écriture groupée, pour la vue (`get_view`, 43-2).
pub(crate) async fn view_members(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<GroupedViewInvoice>, DbError> {
    Ok(client_members(conn, company_id, entry_id)
        .await?
        .into_iter()
        .map(|(_, id, number, amount)| GroupedViewInvoice {
            kind: GroupFamily::Client,
            id,
            number,
            amount,
        })
        .collect())
}

/// Le compte de l'écart d'une écriture groupée, **relu sur sa ligne d'écart**
/// (pour la vue, 43-2) ; `None` sans écart.
///
/// L'écriture porte, dans l'ordre d'insertion (`line_order`, D2) : la banque,
/// une ligne de tiers par facture, la ligne d'arrondi si `r ≠ 0`, puis la ligne
/// d'écart. `r` se recalcule depuis les montants bruts des règlements (`Σ
/// (centime(montant) − montant)`) : c'est ce qui sépare les deux lignes même
/// quand l'utilisateur a choisi pour l'écart le compte d'arrondi.
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
    let r: Decimal = amounts.iter().map(|a| amount_due_to_centime(*a) - *a).sum();
    let accounts: Vec<i64> = sqlx::query_scalar(
        "SELECT jel.account_id FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? ORDER BY jel.line_order",
    )
    .bind(entry_id)
    .bind(company_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let skip = 1 + amounts.len() + usize::from(!r.is_zero());
    Ok(accounts.get(skip).copied())
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
/// qui le porte (la première par identifiant croissant) — pour la vue et pour
/// le geste. `unlinking` : la transaction que l'on défait (exemption étroite
/// du rang 3).
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
    for (settlement_id, invoice_id, number, _) in client_members(conn, company_id, entry_id).await?
    {
        if let Some(hit) = super::invoice_settlements_write::settlement_cancel_blocker_unlinking(
            conn,
            company_id,
            settlement_id,
            Some(unlinking),
            aujourd_hui,
        )
        .await?
        {
            return Ok(Some((
                hit,
                GroupedPiece {
                    document: GroupedDocument::Invoice(invoice_id),
                    number,
                },
            )));
        }
    }
    Ok(None)
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
    // Les membres (déjà verrouillés par le classement ; relus ici).
    let members = client_members(tx, company_id, entry_id).await?;
    if members.is_empty() {
        return Err(DbError::Invariant(format!(
            "règlement groupé : l'écriture {entry_id} n'a aucun règlement client \
             (groupe fournisseur : Story 43-1b)"
        )));
    }
    // Les factures, par identifiant croissant.
    for (_, invoice_id, _, _) in &members {
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
    super::bank_accounts::ensure_open_in_tx(tx, company_id, bt.bank_account_id).await?;

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
    sqlx::query("DELETE FROM invoice_settlements WHERE company_id = ? AND journal_entry_id = ?")
        .bind(company_id)
        .bind(entry_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

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

// ---------------------------------------------------------------------------
// Le règlement groupé fournisseur — Story 43-1b
// ---------------------------------------------------------------------------

/// **Paie plusieurs factures fournisseur par une transaction au débit** —
/// signature figée à T3 de la 43-1a ; corps : Story 43-1b (même méthode, même
/// point de sauvegarde, même [`group_difference`], écart au compte choisi
/// `difference_account_id`, `settlement_journal_entry_line_id` posée par
/// facture). Côté fournisseur, la banque est au crédit : un écart `e > 0` (plus
/// payé) va au **débit** du compte choisi, `e < 0` au crédit.
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
    let _ = (
        tx,
        company_id,
        bank_account_id,
        bank_transaction_id,
        supplier_invoice_ids,
        difference_account_id,
        batch_size,
        acteur,
        aujourd_hui,
    );
    unimplemented!("Story 43-1b : règlement groupé fournisseur")
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

    /// La fiche : deux restes de 10.0050 (affichés 10.01) — contre 20.02,
    /// arrondi de +0.0100, aucun écart ; contre 20.01, arrondi de +0.0100 et
    /// écart de −0.01 (moins reçu).
    #[test]
    fn sub_centime_remainders_split_rounding_and_gap() {
        let restes = [dec!(10.0050), dec!(10.0050)];
        let d = group_difference(dec!(20.02), &restes, dec!(5)).unwrap();
        assert_eq!(d.rounding_difference, dec!(0.0100));
        assert_eq!(d.gap, dec!(0.00));
        assert_eq!(d.difference, dec!(0.0100));
        let d = group_difference(dec!(20.01), &restes, dec!(5)).unwrap();
        assert_eq!(d.rounding_difference, dec!(0.0100));
        assert_eq!(d.gap, dec!(-0.01));
        assert_eq!(d.difference, dec!(0.0000));
        // L'identité `difference = e + r`.
        assert_eq!(d.difference, d.gap + d.rounding_difference);
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
        let d = group_difference(dec!(300.01), &f, dec!(5)).unwrap();
        assert_eq!(d.gap, dec!(0.01));
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
        // Plafond à zéro : seules les fractions de centime passent.
        assert!(group_difference(dec!(299.99), &f, dec!(0)).is_err());
        let restes = [dec!(10.0050), dec!(10.0050)];
        assert!(group_difference(dec!(20.02), &restes, dec!(0)).is_ok());
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
