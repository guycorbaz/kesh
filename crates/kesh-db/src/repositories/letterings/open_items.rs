//! Les postes ouverts d'un compte à une date, et les rapprochements proposés
//! (Story 15-1b, #518) — **lecture seule** : rien ici n'écrit la marque.
//!
//! # « Ouvert à la date `X` »
//!
//! Une ligne du compte, d'une écriture datée **≤ X**, telle qu'elle n'est pas
//! lettrée, **ou** que son groupe contient une ligne d'une écriture datée
//! **> X** (le lettrage n'était pas acquis à `X`). **Invariant** : la somme
//! `Σ(débit − crédit)` des lignes ouvertes à `X` égale le solde cumulatif du
//! compte à `X` — tous exercices confondus, celui de la Balance et du bilan pour
//! un compte de bilan, seul lettrable. Un groupe entièrement daté ≤ X se nette à
//! zéro (R3) et n'est pas compté ; tout autre groupe a ses lignes ≤ X comptées.
//!
//! ⚠️ **« Au X » n'est pas un instantané** : la vue se calcule sur les marques
//! d'**aujourd'hui**. Elle est stable pour un `X` en période close tant qu'aucun
//! administrateur ne déverrouille, ne rouvre un exercice ni ne restaure une
//! sauvegarde ; au-dessus de la borne, un délettrage ou une annulation fait
//! réapparaître des lignes « au X ».
//!
//! # Deux requêtes (AC2)
//!
//! - **A — la sélection** ([`OPEN_ITEMS_BALANCE_SQL`], [`OPEN_ITEMS_TOTALS_SQL`],
//!   [`OPEN_ITEMS_PAGE_SQL`]) : solde, total ouvert, nombre de lignes, page. ⛔
//!   Elle ne lit **aucune pièce** — ni facture, ni avoir, ni facture
//!   fournisseur, ni règlement, ni transaction bancaire — ni `paid_at` :
//!   « ouvert » = « non lettré », un point c'est tout (arbitrage du 2026-08-26).
//!   Garanti en valeur (l'invariant) et lexicalement (liste blanche de tables).
//! - **B — l'enrichissement**, sur les seules écritures de la **page** : la
//!   pièce propriétaire ([`journal_entries::document_owners`], un lot), l'état
//!   de la facture client et son reste dû, la règle des périodes
//!   ([`super::open_period_rule`], une fois pour les exercices de la page).
//!
//! Les deux, et les trois agrégats, sont lus dans **une** transaction ouverte
//! par [`open_items`] elle-même (`begin`, lectures, `rollback`), pour qu'un
//! écrivain concurrent ne désaccorde pas le total et la page. ⚠️ La vue unique
//! repose sur le niveau d'isolation **par défaut** d'InnoDB (`REPEATABLE
//! READ`), que Kesh ne configure pas.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use kesh_core::lettering::proposals::{self, Candidate, MAX_PROPOSAL_CANDIDATES};
use rust_decimal::Decimal;
use sqlx::{Connection, MySqlConnection};

use super::{Origin, code_of, letterable_account, open_period_rule, with_placeholders};
use crate::errors::{DbError, map_db_error};
use crate::repositories::invoice_settlements::{amount_due_scalar_sql, amount_due_to_centime};
use crate::repositories::journal_entries::{self, DocumentKind, DocumentOwner};

/// La fin commune des requêtes A sur les lignes **ouvertes à `X`** : la table
/// dérivée des groupes du compte (date de leur ligne la plus tardive =
/// `letteredOn`) jointe en `LEFT JOIN`, puis le filtre.
///
/// ⛔ La dérivée est **bornée au compte** (`jel2.account_id = ?` — un groupe est
/// mono-compte) et à la société, et lit la date **sans** filtre `≤ X` : la date
/// du groupe doit voir ses lignes postérieures à `X`. Sans la borne, MariaDB
/// matérialiserait les groupes de toutes les sociétés, la clé étant un
/// identifiant de ligne global.
///
/// Marqueurs, dans l'ordre : compte et société (dérivée), compte, société,
/// `X` (date ≤), `X` (groupe acquis après).
macro_rules! open_items_tail {
    () => {
        "LEFT JOIN (SELECT jel2.lettering_key, MAX(je2.entry_date) AS lettered_on \
           FROM journal_entry_lines jel2 FORCE INDEX (idx_jel_account_lettering) \
           JOIN journal_entries je2 ON je2.id = jel2.entry_id \
          WHERE jel2.account_id = ? AND je2.company_id = ? AND jel2.lettering_key IS NOT NULL \
          GROUP BY jel2.lettering_key) g ON g.lettering_key = jel.lettering_key \
         WHERE jel.account_id = ? AND je.company_id = ? AND je.entry_date <= ? \
           AND (g.lettered_on IS NULL OR g.lettered_on > ?)"
    };
}

/// Requête A — le **solde** cumulatif du compte à `X` : toutes ses lignes datées
/// ≤ X, tous exercices confondus. Marqueurs : compte, société, `X`.
pub const OPEN_ITEMS_BALANCE_SQL: &str = "SELECT COALESCE(SUM(jel.debit - jel.credit), 0) \
     FROM journal_entry_lines jel FORCE INDEX (idx_jel_account_lettering) \
     JOIN journal_entries je ON je.id = jel.entry_id \
     WHERE jel.account_id = ? AND je.company_id = ? AND je.entry_date <= ?";

/// Requête A — le **nombre** et le **total** `Σ(débit − crédit)` des lignes
/// ouvertes à `X`, sur tout l'ensemble (pas la page).
pub const OPEN_ITEMS_TOTALS_SQL: &str = concat!(
    "SELECT COUNT(*), COALESCE(SUM(jel.debit - jel.credit), 0) \
     FROM journal_entry_lines jel FORCE INDEX (idx_jel_account_lettering) \
     JOIN journal_entries je ON je.id = jel.entry_id ",
    open_items_tail!()
);

/// Requête A — la **page**, dans l'ordre du Grand livre : date, exercice,
/// numéro d'écriture, `line_order`, puis `id` (`general_ledger::fetch_lines`).
/// Marqueurs : ceux de la fin commune, puis `LIMIT`, `OFFSET`.
pub const OPEN_ITEMS_PAGE_SQL: &str = concat!(
    "SELECT jel.id, jel.entry_id, je.entry_number, je.fiscal_year_id, fy.name, je.entry_date, \
            je.journal, je.description, jel.debit, jel.credit, jel.lettering_key, \
            jel.lettering_origin, g.lettered_on \
     FROM journal_entry_lines jel FORCE INDEX (idx_jel_account_lettering) \
     JOIN journal_entries je ON je.id = jel.entry_id \
     JOIN fiscal_years fy ON fy.id = je.fiscal_year_id ",
    open_items_tail!(),
    " ORDER BY je.entry_date ASC, je.fiscal_year_id ASC, je.entry_number ASC, \
       jel.line_order ASC, jel.id ASC \
     LIMIT ? OFFSET ?"
);

/// Requête B — l'état des factures client de la page : `paid_at` posé, au moins
/// un règlement, reste dû par la forme **scalaire par facture**
/// ([`amount_due_scalar_sql`], #416 : la formule d'`amount_due`, jamais
/// réécrite). ⚠️ Pas les tables dérivées d'`amount_due_derived_joins` : mesuré
/// (AC8, `EXPLAIN` sur 20 000 factures), celle des lignes d'avoir est lue en
/// entier pour une page de 50 factures (C-15-1b-16). `{ids}` : autant de `?`
/// que de factures ; puis la société.
fn invoice_states_sql() -> String {
    format!(
        "SELECT i.id, i.paid_at IS NOT NULL, \
                EXISTS (SELECT 1 FROM invoice_settlements s WHERE s.invoice_id = i.id), \
                {due} \
         FROM invoices i \
         WHERE i.id IN ({{ids}}) AND i.company_id = ?",
        due = amount_due_scalar_sql()
    )
}

/// Requête des **candidates** aux propositions : les lignes du compte ouvertes
/// **aujourd'hui** (non lettrées), toutes dates, par `idx_jel_account_lettering`.
/// Marqueurs : compte, société.
const PROPOSAL_LINES_SQL: &str = "SELECT jel.id, jel.entry_id, je.entry_number, je.fiscal_year_id, \
            fy.name, je.entry_date, je.journal, je.description, jel.debit, jel.credit, \
            je.reverses_entry_id \
     FROM journal_entry_lines jel FORCE INDEX (idx_jel_account_lettering) \
     JOIN journal_entries je ON je.id = jel.entry_id \
     JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
     WHERE jel.account_id = ? AND je.company_id = ? AND jel.lettering_key IS NULL \
     ORDER BY jel.id";

/// Pourquoi une ligne est ouverte **à `X`** (AC4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenReason {
    /// La ligne n'est pas lettrée aujourd'hui.
    Unlettered,
    /// La ligne est lettrée, mais son groupe porte une ligne datée après `X`.
    LetteredAfterAsOf,
}

impl OpenReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unlettered => "unlettered",
            Self::LetteredAfterAsOf => "letteredAfterAsOf",
        }
    }
}

/// Où en est **aujourd'hui** la facture client d'une ligne (AC4), dans l'ordre
/// de précédence — la première qui s'applique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentState {
    /// `paid_at` posé et aucun règlement : héritage d'avant la v0.12.0 — le grand
    /// livre porte réellement la créance.
    PaidWithoutSettlementEntry,
    /// Reste dû au centime ≤ 0 (réglée, créditée ou soldée).
    NothingDue,
    /// Au moins un règlement, reste dû au centime > 0.
    PartiallySettled,
    /// Aucun règlement, reste dû au centime > 0.
    Unpaid,
}

impl DocumentState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PaidWithoutSettlementEntry => "paidWithoutSettlementEntry",
            Self::NothingDue => "nothingDue",
            Self::PartiallySettled => "partiallySettled",
            Self::Unpaid => "unpaid",
        }
    }

    /// La précédence d'AC4, sur le reste dû **au centime** (#490 : un reste brut
    /// de 0,004 n'est pas `partiallySettled`).
    fn of(paid_at: bool, has_settlement: bool, due_centime: Decimal) -> Self {
        if paid_at && !has_settlement {
            Self::PaidWithoutSettlementEntry
        } else if due_centime <= Decimal::ZERO {
            Self::NothingDue
        } else if has_settlement {
            Self::PartiallySettled
        } else {
            Self::Unpaid
        }
    }
}

/// Une ligne ouverte à `X`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenItem {
    pub line_id: i64,
    pub entry_id: i64,
    /// ⚠️ Unique par exercice seulement : se lit avec `fiscal_year_name`.
    pub entry_number: i64,
    pub fiscal_year_id: i64,
    pub fiscal_year_name: String,
    pub date: NaiveDate,
    pub journal: String,
    pub description: String,
    pub debit: Decimal,
    pub credit: Decimal,
    /// La pièce qui possède l'écriture — la **première** dans l'ordre de
    /// [`DocumentKind`] ; `None` si aucune.
    pub document: Option<DocumentOwner>,
    /// Le lettrage d'**aujourd'hui** (nuls si la ligne est ouverte aujourd'hui).
    pub lettering_key: Option<i64>,
    pub lettering_code: Option<String>,
    pub lettering_origin: Option<Origin>,
    /// Date de la ligne la plus tardive du groupe : celle où le lettrage est
    /// acquis. Non nul **et** > `X` ⇔ [`OpenReason::LetteredAfterAsOf`].
    pub lettered_on: Option<NaiveDate>,
    pub reason: OpenReason,
    /// Pour une ligne de facture client (`invoice`, ou `settlement` → sa
    /// facture) ; `None` pour tout autre, avoir compris (C-15-1b-12).
    pub document_state: Option<DocumentState>,
    /// Reste dû d'aujourd'hui, **au centime** ; `None` hors facture client.
    pub amount_due: Option<Decimal>,
    /// Ouverte aujourd'hui, et aucun propriétaire ne fait d'elle une ligne de
    /// pièce ([`DocumentKind::blocks_manual_lettering`]).
    pub manually_letterable: bool,
    /// « En période ouverte » (R7), lu sans verrou — indicatif.
    pub in_open_period: bool,
}

/// La vue des postes ouverts d'un compte à `X` (AC1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenItems {
    pub account_id: i64,
    pub account_number: String,
    pub as_of: NaiveDate,
    /// Solde cumulatif du compte à `X`, `Σ(débit − crédit)`.
    pub balance: Decimal,
    /// `Σ(débit − crédit)` des lignes ouvertes à `X` — égal à `balance`.
    pub open_total: Decimal,
    /// Nombre de lignes ouvertes à `X` (tout l'ensemble).
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
    pub items: Vec<OpenItem>,
}

/// Une ligne d'une paire proposée — sous-ensemble d'[`OpenItem`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalLine {
    pub line_id: i64,
    pub entry_id: i64,
    pub entry_number: i64,
    pub fiscal_year_name: String,
    pub date: NaiveDate,
    pub journal: String,
    pub description: String,
    pub document: Option<DocumentOwner>,
    pub in_open_period: bool,
}

/// Une paire proposée (AC5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetteringProposal {
    pub amount: Decimal,
    pub days_apart: i64,
    pub reversal_pair: bool,
    pub debit: ProposalLine,
    pub credit: ProposalLine,
}

/// Les propositions d'un compte (AC5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetteringProposals {
    pub account_id: i64,
    /// Lignes candidates (ouvertes, lettrables à la main), ≤ le plafond.
    pub candidate_count: usize,
    /// Paires retenues avant `limit`.
    pub total: usize,
    pub limit: usize,
    pub items: Vec<LetteringProposal>,
}

/// AC7 — le compte de la vue : introuvable dans la société → [`DbError::NotFound`]
/// (indiscernable d'un compte d'une autre société) ; non lettrable, y compris
/// **devenu** non lettrable (C104) → [`DbError::LetteringAccountNotLetterable`].
/// Rend le numéro du compte.
async fn require_viewable_account(
    conn: &mut MySqlConnection,
    company_id: i64,
    account_id: i64,
) -> Result<String, DbError> {
    match letterable_account(conn, company_id, account_id).await? {
        None => Err(DbError::NotFound),
        Some((false, _)) => Err(DbError::LetteringAccountNotLetterable),
        Some((true, number)) => Ok(number),
    }
}

/// R5 sur le lot de [`journal_entries::document_owners`] : une ligne **ouverte**
/// est lettrable à la main si aucun propriétaire de son écriture n'a
/// [`DocumentKind::blocks_manual_lettering`] — la liste écrite une fois, celle
/// de `first_document_owner`. ⚠️ Une transaction bancaire seule n'ôte pas la
/// lettrabilité.
fn free_of_document(owners: Option<&Vec<DocumentOwner>>) -> bool {
    !owners.is_some_and(|v| v.iter().any(|o| o.kind.blocks_manual_lettering()))
}

/// La facture client que décrit `documentState` : celle d'un propriétaire
/// `invoice`, ou la facture réglée d'un `settlement` ; `None` pour tout autre
/// type, avoir compris (C-15-1b-12).
fn client_invoice_of(document: &DocumentOwner) -> Option<i64> {
    match document.kind {
        DocumentKind::Invoice => Some(document.id),
        DocumentKind::Settlement => document.invoice_id,
        DocumentKind::CreditNote
        | DocumentKind::SupplierInvoice
        | DocumentKind::BankTransaction => None,
    }
}

#[derive(sqlx::FromRow)]
struct PageRow {
    id: i64,
    entry_id: i64,
    entry_number: i64,
    fiscal_year_id: i64,
    name: String,
    entry_date: NaiveDate,
    journal: String,
    description: String,
    debit: Decimal,
    credit: Decimal,
    lettering_key: Option<i64>,
    lettering_origin: Option<String>,
    lettered_on: Option<NaiveDate>,
}

/// **Les postes ouverts du compte `account_id` à la date `as_of`** (AC1-AC4).
///
/// `limit` et `offset` sont les valeurs **effectives** (l'appelant les écrête :
/// `limit` dans `1..=500`, `offset ≥ 0`). Refus : [`DbError::NotFound`] (compte
/// introuvable dans la société), [`DbError::LetteringAccountNotLetterable`].
///
/// Ouvre **sa** transaction de lecture sur la connexion prêtée, la referme par
/// `rollback` — une vue unique pour les agrégats et la page (`REPEATABLE READ`
/// par défaut d'InnoDB, non configuré par Kesh).
pub async fn open_items(
    conn: &mut MySqlConnection,
    company_id: i64,
    account_id: i64,
    as_of: NaiveDate,
    limit: i64,
    offset: i64,
) -> Result<OpenItems, DbError> {
    let mut tx = conn.begin().await.map_err(map_db_error)?;
    let account_number = require_viewable_account(&mut tx, company_id, account_id).await?;

    // Requête A — solde, total ouvert, nombre, page.
    let balance: Decimal = sqlx::query_scalar(OPEN_ITEMS_BALANCE_SQL)
        .bind(account_id)
        .bind(company_id)
        .bind(as_of)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
    let (total, open_total): (i64, Decimal) = sqlx::query_as(OPEN_ITEMS_TOTALS_SQL)
        .bind(account_id)
        .bind(company_id)
        .bind(account_id)
        .bind(company_id)
        .bind(as_of)
        .bind(as_of)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
    let rows: Vec<PageRow> = sqlx::query_as(OPEN_ITEMS_PAGE_SQL)
        .bind(account_id)
        .bind(company_id)
        .bind(account_id)
        .bind(company_id)
        .bind(as_of)
        .bind(as_of)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Requête B — sur les seules écritures de la page.
    let entry_ids: Vec<i64> = rows.iter().map(|r| r.entry_id).collect();
    let owners = journal_entries::document_owners(&mut tx, company_id, &entry_ids).await?;
    let fiscal_years: Vec<i64> = rows.iter().map(|r| r.fiscal_year_id).collect();
    let regle = open_period_rule(&mut tx, company_id, &fiscal_years).await?;
    let invoice_ids: BTreeSet<i64> = rows
        .iter()
        .filter_map(|r| owners.get(&r.entry_id)?.first())
        .filter_map(client_invoice_of)
        .collect();
    let states = invoice_states(&mut tx, company_id, &invoice_ids).await?;
    tx.rollback().await.map_err(map_db_error)?;

    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        let entry_owners = owners.get(&r.entry_id);
        let document = entry_owners.and_then(|v| v.first()).cloned();
        let lettering_origin = r
            .lettering_origin
            .as_deref()
            .map(|o| {
                Origin::parse(o).ok_or_else(|| {
                    DbError::Invariant(format!("postes ouverts : origine de lettrage inconnue {o}"))
                })
            })
            .transpose()?;
        let (document_state, amount_due) = match document
            .as_ref()
            .and_then(client_invoice_of)
            .and_then(|id| states.get(&id))
        {
            Some((state, due)) => (Some(*state), Some(*due)),
            None => (None, None),
        };
        items.push(OpenItem {
            line_id: r.id,
            entry_id: r.entry_id,
            entry_number: r.entry_number,
            fiscal_year_id: r.fiscal_year_id,
            fiscal_year_name: r.name,
            date: r.entry_date,
            journal: r.journal,
            description: r.description,
            debit: r.debit,
            credit: r.credit,
            reason: if r.lettering_key.is_some() {
                OpenReason::LetteredAfterAsOf
            } else {
                OpenReason::Unlettered
            },
            manually_letterable: r.lettering_key.is_none() && free_of_document(entry_owners),
            in_open_period: regle.line_in_open_period(r.fiscal_year_id, r.entry_date)?,
            document,
            lettering_code: r.lettering_key.map(code_of),
            lettering_key: r.lettering_key,
            lettering_origin,
            lettered_on: r.lettered_on,
            document_state,
            amount_due,
        });
    }

    Ok(OpenItems {
        account_id,
        account_number,
        as_of,
        balance,
        open_total,
        total,
        offset,
        limit,
        items,
    })
}

/// Requête B — `facture → (état, reste dû au centime)` pour les factures
/// nommées de la société ; vide sans requête si aucune.
async fn invoice_states(
    conn: &mut MySqlConnection,
    company_id: i64,
    invoice_ids: &BTreeSet<i64>,
) -> Result<BTreeMap<i64, (DocumentState, Decimal)>, DbError> {
    if invoice_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let sql = with_placeholders(&invoice_states_sql(), invoice_ids.len());
    let mut q = sqlx::query_as::<_, (i64, bool, bool, Decimal)>(&sql);
    for id in invoice_ids {
        q = q.bind(*id);
    }
    let rows = q
        .bind(company_id)
        .fetch_all(conn)
        .await
        .map_err(map_db_error)?;
    Ok(rows
        .into_iter()
        .map(|(id, paid_at, has_settlement, due)| {
            let due = amount_due_to_centime(due);
            (id, (DocumentState::of(paid_at, has_settlement, due), due))
        })
        .collect())
}

/// La requête B des états de facture pour `n` factures — exposée pour
/// l'`EXPLAIN` d'AC8 (test ignoré de `tests/open_items.rs`) ; marqueurs : les
/// `n` factures, puis la société.
#[doc(hidden)]
pub fn open_items_invoice_states_sql(n: usize) -> String {
    with_placeholders(&invoice_states_sql(), n)
}

#[derive(sqlx::FromRow)]
struct CandidateRow {
    id: i64,
    entry_id: i64,
    entry_number: i64,
    fiscal_year_id: i64,
    name: String,
    entry_date: NaiveDate,
    journal: String,
    description: String,
    debit: Decimal,
    credit: Decimal,
    reverses_entry_id: Option<i64>,
}

/// **Les rapprochements que Kesh propose** sur le compte `account_id` (AC5) :
/// paires de lignes ouvertes aujourd'hui, lettrables à la main, de sens opposés
/// et de montants égaux, dont une au moins est en période ouverte — classées
/// par [`proposals::propose_pairs`]. Kesh **n'écrit rien**.
///
/// - Candidates : lignes non lettrées du compte, moins les lignes de pièce
///   (R5, par [`journal_entries::document_owners`], découpé par 500 écritures :
///   coût linéaire en lignes ouvertes, **sans plafond propre**) ; au-delà de
///   [`MAX_PROPOSAL_CANDIDATES`] candidates → [`DbError::LetteringProposalsTooManyLines`]
///   (jamais une troncature muette).
/// - `limit` effectif (l'appelant l'écrête dans `1..=500`) ; **pas d'`offset`** :
///   les paires suivantes se lisent après avoir accepté les premières.
/// - Refus du compte : comme [`open_items`] (404, 409).
///
/// Lecture sans verrou, dans une transaction de lecture (`rollback`) : une
/// borne posée entre la proposition et le clic reste refusée par le lettrage.
pub async fn lettering_proposals(
    conn: &mut MySqlConnection,
    company_id: i64,
    account_id: i64,
    limit: usize,
) -> Result<LetteringProposals, DbError> {
    let mut tx = conn.begin().await.map_err(map_db_error)?;
    require_viewable_account(&mut tx, company_id, account_id).await?;
    let rows: Vec<CandidateRow> = sqlx::query_as(PROPOSAL_LINES_SQL)
        .bind(account_id)
        .bind(company_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;
    let entry_ids: Vec<i64> = rows.iter().map(|r| r.entry_id).collect();
    let owners = journal_entries::document_owners(&mut tx, company_id, &entry_ids).await?;
    let rows: Vec<CandidateRow> = rows
        .into_iter()
        .filter(|r| free_of_document(owners.get(&r.entry_id)))
        .collect();
    if rows.len() > MAX_PROPOSAL_CANDIDATES {
        return Err(DbError::LetteringProposalsTooManyLines {
            max: MAX_PROPOSAL_CANDIDATES,
        });
    }
    let fiscal_years: Vec<i64> = rows.iter().map(|r| r.fiscal_year_id).collect();
    let regle = open_period_rule(&mut tx, company_id, &fiscal_years).await?;
    tx.rollback().await.map_err(map_db_error)?;

    let mut candidates = Vec::with_capacity(rows.len());
    for r in &rows {
        candidates.push(Candidate {
            line_id: r.id,
            entry_id: r.entry_id,
            reverses_entry_id: r.reverses_entry_id,
            date: r.entry_date,
            debit: r.debit,
            credit: r.credit,
            in_open_period: regle.line_in_open_period(r.fiscal_year_id, r.entry_date)?,
        });
    }
    let paires = proposals::propose_pairs(&candidates);
    let line = |i: usize| {
        let (r, c) = (&rows[i], &candidates[i]);
        ProposalLine {
            line_id: r.id,
            entry_id: r.entry_id,
            entry_number: r.entry_number,
            fiscal_year_name: r.name.clone(),
            date: r.entry_date,
            journal: r.journal.clone(),
            description: r.description.clone(),
            document: owners.get(&r.entry_id).and_then(|v| v.first()).cloned(),
            in_open_period: c.in_open_period,
        }
    };
    let items = paires
        .iter()
        .take(limit)
        .map(|p| LetteringProposal {
            amount: p.amount,
            days_apart: p.days_apart,
            reversal_pair: p.reversal_pair,
            debit: line(p.debit),
            credit: line(p.credit),
        })
        .collect();
    Ok(LetteringProposals {
        account_id,
        candidate_count: candidates.len(),
        total: paires.len(),
        limit,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 3 (AC2) — garde **lexicale** en liste blanche sur les constantes de
    /// la requête A : tout nom de table qui suit `FROM` ou `JOIN` est l'un de
    /// `journal_entry_lines`, `journal_entries`, `fiscal_years` — une table
    /// dérivée (`(SELECT …`) est parcourue pour ses propres `FROM`/`JOIN` —, et
    /// `paid_at` n'y figure pas. Une pièce réintroduite dans A rougit ici.
    #[test]
    fn selection_query_reads_no_document() {
        const ADMISES: [&str; 3] = ["journal_entry_lines", "journal_entries", "fiscal_years"];
        let requetes = [
            ("balance", OPEN_ITEMS_BALANCE_SQL),
            ("totaux", OPEN_ITEMS_TOTALS_SQL),
            ("page", OPEN_ITEMS_PAGE_SQL),
        ];
        // Défense complémentaire de la liste blanche (revue P1, A) : aucun nom
        // de table de pièce n'apparaît comme mot, sous quelque forme que ce soit
        // (jointure à virgule, `FROM(` accolé, sous-requête `EXISTS`).
        const PIECES: [&str; 6] = [
            "invoices",
            "invoice_lines",
            "credit_notes",
            "supplier_invoices",
            "invoice_settlements",
            "bank_transactions",
        ];
        for (nom, sql) in requetes {
            let normalise: String = sql
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '_' {
                        c
                    } else {
                        ' '
                    }
                })
                .collect();
            for mot in normalise.split_whitespace() {
                assert!(
                    !PIECES.contains(&mot),
                    "requête A ({nom}) : la table de pièce « {mot} » est nommée"
                );
            }
            let mots: Vec<&str> = sql.split_whitespace().collect();
            let mut tables = 0;
            for w in mots.windows(2) {
                if w[0].eq_ignore_ascii_case("FROM") || w[0].eq_ignore_ascii_case("JOIN") {
                    if w[1].starts_with('(') {
                        continue; // table dérivée : ses FROM/JOIN sont lus à leur tour
                    }
                    assert!(
                        ADMISES.contains(&w[1]),
                        "requête A ({nom}) : la table « {} » n'est pas admise",
                        w[1]
                    );
                    tables += 1;
                }
            }
            assert!(
                tables >= 2,
                "montage : la requête {nom} doit nommer ses tables"
            );
            assert!(!sql.contains("paid_at"), "requête A ({nom}) : paid_at lu");
        }
    }

    /// Revue P1, E-6 (réfuté) — un reste brut de −0,004 s'affiche « 0.00 » :
    /// l'arrondi au centime ne garde pas de zéro négatif (mutation « redressement
    /// retiré » verte : aucun redressement n'est nécessaire). Un trop-perçu réel
    /// reste négatif.
    #[test]
    fn rounded_due_has_no_negative_zero() {
        use rust_decimal_macros::dec;
        assert_eq!(amount_due_to_centime(dec!(-0.004)).to_string(), "0.00");
        assert_eq!(amount_due_to_centime(dec!(-5.004)).to_string(), "-5.00");
        assert_eq!(amount_due_to_centime(dec!(10.005)).to_string(), "10.01");
    }

    /// AC4 — la précédence de `documentState`, sur le reste au centime.
    #[test]
    fn document_state_precedence() {
        use rust_decimal_macros::dec;
        let s = DocumentState::of;
        assert_eq!(
            s(true, false, dec!(10)),
            DocumentState::PaidWithoutSettlementEntry
        );
        assert_eq!(
            s(true, false, dec!(0)),
            DocumentState::PaidWithoutSettlementEntry
        );
        assert_eq!(s(true, true, dec!(0)), DocumentState::NothingDue);
        assert_eq!(s(false, true, dec!(-1)), DocumentState::NothingDue);
        assert_eq!(s(false, true, dec!(0.01)), DocumentState::PartiallySettled);
        assert_eq!(s(false, false, dec!(0.01)), DocumentState::Unpaid);
        assert_eq!(s(false, false, dec!(0)), DocumentState::NothingDue);
    }
}
