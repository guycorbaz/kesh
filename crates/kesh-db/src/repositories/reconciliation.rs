//! Repository pour la réconciliation bancaire (Story 8-4).
//!
//! Helpers :
//!
//! - [`InvoiceDateWindow`] : **la** fenêtre de dates dans laquelle une
//!   facture peut être réglée par une transaction (Story 28-7, #548) —
//!   seule définition du dépôt, lue par la proposition
//!   ([`find_unpaid_invoices_for_window`]) **et** par l'acceptation
//!   (`accept_one_invoice`, `kesh-api`).
//! - [`find_unpaid_invoices_for_window`] : charge les factures
//!   candidates pour le matching d'une transaction bancaire — filtre
//!   multi-tenant + [`InvoiceDateWindow`] + tolérance amount
//!   ± `amount_tolerance`, par `(date, id)` croissants, 50 au plus.
//!   **PAS de filtre currency** (S4-1 Pass 4 revert : colonne
//!   `invoices.currency` n'existe pas v0.1 mono-CHF, cf. L38).
//! - [`find_pending_transactions_for_account`] : charge les
//!   transactions `status='pending'` non-rejetées (`auto_match_rejected_at IS NULL`)
//!   pour un compte donné, multi-tenant scoped, paginées (limit
//!   hardcodé 100 au caller, cf. L24).
//! - [`find_contacts_by_ids`] : batch-load Contacts par liste d'IDs
//!   (M5 Pass 1 + MP4-3 Pass 4 — évite N×M queries dans le handler
//!   GET proposals 4-pass architecture).
//! - [`find_by_ids_for_account`] : pré-vol par lot de POST /accept et
//!   POST /reject (1 SELECT IN avant le verrou). **Story 27-4** : sans
//!   filtre de statut ni de rejet (remplace `find_pending_by_ids`, qui
//!   filtrait et faisait refuser tout le lot en 400) — l'appelant classe
//!   chaque ligne et refuse par proposition dans `failed[]`.
//! - [`find_by_id_for_account_for_update`] : relecture **verrouillante**
//!   (lecture courante) d'une transaction par le rejet, quand son
//!   `UPDATE` n'a touché aucune ligne (Story 27-4, AC4), et lecture de
//!   l'annulation d'un rejet, sous verrou, avant son `UPDATE` (Story 28-5a).
//! - [`find_pending_by_id_for_account`] : recharge UNE
//!   `BankTransaction` pending par id (C2 Pass 1 — TOCTOU fix : appelé
//!   inside lock dans `accept_one` pour fermer la fenêtre entre le
//!   pré-flight batch et l'UPDATE).
//!
//! Tous les helpers utilisent l'Executor générique pour accepter
//! `&MySqlPool` (preview) ou `&mut Transaction<MySql>` (handler create
//! dans tx ouverte).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySql;
use std::collections::HashMap;

use crate::entities::bank_transaction::BankTransaction;
use crate::entities::contact::Contact;
use crate::entities::invoice::Invoice;
use crate::errors::{DbError, map_db_error};
use crate::repositories::invoice_settlements::{
    INVOICE_AMOUNT_DUE_DERIVED_SQL, amount_due_derived_joins,
};

// Colonnes Invoice pour SELECT — la liste unique de `repositories::invoices`
// (Story 25-6-b).
use super::invoices::INVOICE_COLUMNS;

/// Facture candidate à la réconciliation, accompagnée de son **reste dû** et
/// de son **TTC**.
///
/// - `amount_due` (#420, Story 25-4-c) — le montant réellement comparé à
///   l'encaissement bancaire : `TTC − avoir émis − Σ règlements`. Une facture
///   déjà réglée en partie attend son **solde**, pas son TTC ; comparer le TTC
///   l'excluait des candidates du virement qui la soldait.
/// - `total_ttc` (#246, Story 21-2b) — conservé pour l'**affichage** seulement
///   (« reste dû sur 1 000.00 »), jamais pour comparer. `invoice.total_amount`
///   est le HT et ne matcherait jamais une facture avec TVA.
///
/// Le reste dû est **brut** (jusqu'à 4 décimales) : l'arrondi au centime est
/// la Story 25-4-c3 (#476), pas celle-ci.
///
/// `#[sqlx(flatten)]` : **premier usage du workspace** — `Invoice` dérive
/// `FromRow` et le SELECT liste ses colonnes via `INVOICE_COLUMNS`, donc le
/// flatten hydrate proprement. `pub` (pas `pub(crate)`) : consommé et
/// destructuré par `kesh-api` (crate séparé).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UnpaidInvoiceCandidate {
    #[sqlx(flatten)]
    pub invoice: Invoice,
    pub amount_due: Decimal,
    pub total_ttc: Decimal,
}

/// Colonnes BankTransaction pour SELECT (cohérent bank_transactions::COLUMNS).
const BANK_TX_COLUMNS: &str = "id, company_id, import_id, bank_account_id, booking_date, value_date, \
     amount, currency, reference, details, end_to_end_id, transaction_id, \
     counterparty_iban, counterparty_name, status, matched_entry_id, \
     auto_match_rejected_at, version, created_at, updated_at";

/// Un paiement règle une facture au plus **30 jours après** la date de
/// celle-ci (borne de [`InvoiceDateWindow::earliest_invoice_date`]).
///
/// Au-delà, le paiement n'est ni proposé ni accepté : c'est la borne de
/// #529 (paiement tardif), que la Story 28-7 laisse où elle est.
pub const INVOICE_WINDOW_DAYS_AFTER_INVOICE: i64 = 30;

/// Un paiement peut précéder la facture qu'il règle d'**un jour** au plus
/// (borne de [`InvoiceDateWindow::latest_invoice_date`]).
///
/// La date de paiement est une date bancaire, la date de facture une date
/// métier locale : l'écart de fuseau suffit à faire paraître un paiement
/// « la veille ». Un paiement antérieur de plus d'un jour — un acompte, par
/// exemple — ne règle pas cette facture : Kesh n'a pas de modèle d'acompte,
/// il se rapproche à la main.
pub const PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS: i64 = 1;

/// La date de paiement d'une transaction bancaire : sa **date de valeur**,
/// à défaut sa date de comptabilisation.
///
/// C'est la date que lisent la fenêtre ([`InvoiceDateWindow::for_transaction`])
/// et le règlement écrit à l'acceptation (`paid_at`, `settled_on`, date de
/// l'écriture d'encaissement).
pub fn payment_date(tx: &BankTransaction) -> NaiveDate {
    tx.value_date.unwrap_or(tx.booking_date)
}

/// La fenêtre de dates de facture qu'une transaction peut régler (Story 28-7,
/// #548) : `D − 30 jours ≤ F ≤ D + 1 jour`, `D` étant la
/// [date de paiement](payment_date) et `F` la date de la facture, bornes
/// incluses.
///
/// **Une seule définition, deux lecteurs.** La proposition
/// ([`find_unpaid_invoices_for_window`], appelée par `get_proposals`) et
/// l'acceptation (`accept_one_invoice`, `kesh-api`) la lisent toutes deux :
/// une facture proposée est une facture que l'acceptation admet, et
/// réciproquement. Avant la Story 28-7, la proposition cherchait
/// `± 30 jours` autour de la date de comptabilisation, et proposait des
/// factures que l'acceptation refusait ensuite.
///
/// **Pourquoi l'acceptation fait foi** (C-28-7-1) : admettre un paiement
/// antérieur à sa facture changerait ce que Kesh écrit au journal — un
/// règlement daté avant la créance —, romprait l'accord avec le règlement
/// saisi depuis la fiche de la facture, et se heurterait à la base.
///
/// **Sites qui portent la même borne basse sans lire cette fenêtre** — non
/// résolus, assumés (C-28-7-7) :
/// - `settle_invoice` et `write_off_invoice`
///   (`repositories::invoice_settlements_write`, le règlement et le solde par
///   escompte, perte, frais ou arrondi) : même tolérance d'un jour sur une
///   date **saisie**, sans borne haute — ils ne règlent pas une transaction ;
/// - la contrainte `chk_invoices_paid_at_after_date` (migration
///   `20260419000001_invoice_paid_at.sql`, copiée dans le squash de
///   `test-schema`) : `DATE(paid_at) >= date - INTERVAL 1 DAY`. La base
///   double la tolérance d'un jour, qui ne se desserre que par une migration.
///
/// Aucune borne n'est ramenée à la plage d'une colonne `DATE` : `check`
/// compare exactement ce que comparait l'acceptation. Une transaction datée
/// aux extrémités de cette plage est un angle mort écrit (C-28-7-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvoiceDateWindow {
    /// Date de facture la plus ancienne admise : `D − 30 jours`.
    pub earliest_invoice_date: NaiveDate,
    /// Date de facture la plus récente admise : `D + 1 jour`.
    pub latest_invoice_date: NaiveDate,
}

/// Pourquoi une facture sort de l'[`InvoiceDateWindow`] d'une transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceDateRefusal {
    /// La facture est datée de plus d'un jour **après** le paiement : le
    /// paiement la précède (raison d'API `payment_date_before_invoice_date`).
    PaymentBeforeInvoice,
    /// Le paiement arrive plus de 30 jours après la facture (raison d'API
    /// `payment_date_outside_window`).
    PaymentTooLate,
}

impl InvoiceDateWindow {
    /// La fenêtre d'une date de paiement `d` : `[d − 30, d + 1]`.
    pub fn for_payment_date(d: NaiveDate) -> Self {
        Self {
            earliest_invoice_date: d - chrono::Duration::days(INVOICE_WINDOW_DAYS_AFTER_INVOICE),
            latest_invoice_date: d + chrono::Duration::days(PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS),
        }
    }

    /// La fenêtre d'une transaction, sur sa [date de paiement](payment_date).
    pub fn for_transaction(tx: &BankTransaction) -> Self {
        Self::for_payment_date(payment_date(tx))
    }

    /// Une facture datée `invoice_date` est-elle dans la fenêtre ?
    ///
    /// # Errors
    ///
    /// - [`InvoiceDateRefusal::PaymentBeforeInvoice`] si
    ///   `invoice_date > latest_invoice_date` ;
    /// - [`InvoiceDateRefusal::PaymentTooLate`] si
    ///   `invoice_date < earliest_invoice_date`.
    pub fn check(&self, invoice_date: NaiveDate) -> Result<(), InvoiceDateRefusal> {
        if invoice_date > self.latest_invoice_date {
            return Err(InvoiceDateRefusal::PaymentBeforeInvoice);
        }
        if invoice_date < self.earliest_invoice_date {
            return Err(InvoiceDateRefusal::PaymentTooLate);
        }
        Ok(())
    }
}

/// Charge les factures candidates pour la réconciliation d'une
/// transaction bancaire (Story 8-4 §candidate-window).
///
/// Filtres :
/// 1. `company_id` (KF-002 Pattern 1, multi-tenant scoping).
/// 2. `status = 'validated' AND paid_at IS NULL AND journal_entry_id IS NOT NULL`
///    (factures éligibles à la réconciliation v0.1).
/// 3. `date BETWEEN window.earliest_invoice_date AND window.latest_invoice_date`
///    — la fenêtre de l'acceptation ([`InvoiceDateWindow`], Story 28-7).
///    Résultat trié par `date ASC, id ASC` (C-28-7-10) **avant** le
///    `LIMIT 50` : au-delà de 50 candidates, ce sont toujours les plus
///    récentes qui sont écartées. Un index peut servir cet ordre si
///    l'optimiseur le choisit ; c'est l'`ORDER BY` qui le garantit.
/// 4. **Reste dû** `BETWEEN tx_amount - amount_tolerance AND tx_amount + amount_tolerance`
///    (#420, Story 25-4-c) — après le HT (#246 : une facture avec TVA ne
///    matchait jamais) puis le TTC (une facture réglée en partie ne matchait
///    jamais son solde). Le reste dû est pris **brut** : la tolérance couvre
///    l'écart à son arrondi au centime (au plus 0.005). ⛔ Pas de `ROUND` ici :
///    l'arrondi a UNE définition, `invoice_settlements::amount_due_to_centime`,
///    appliquée par l'appelant au score et à l'affichage (Story 25-4-c3-b).
///
/// ⛔ **Forme jointe** ([`amount_due_derived_joins`] +
/// [`INVOICE_AMOUNT_DUE_DERIVED_SQL`]) : c'est une liste, la forme corrélée y
/// serait réévaluée par ligne — un N+1 déguisé.
///
/// **Pas de filtre currency v0.1** — colonne inexistante (cf. L38 + S4-1 Pass 4).
///
/// **Multi-tenant safety (KF-002 Pattern 1)** : filtrage systématique
/// par `company_id`.
///
/// **Index** : `idx_invoices_company_validated_unpaid_date` créé en
/// migration `20260507100001_reconciliation_8_4.sql` (couvre le `WHERE` ;
/// le filtre du reste dû est appliqué en `HAVING` après retrait des non-éligibles).
pub async fn find_unpaid_invoices_for_window<'e, E>(
    executor: E,
    company_id: i64,
    window: InvoiceDateWindow,
    tx_amount: Decimal,
    amount_tolerance: Decimal,
) -> Result<Vec<UnpaidInvoiceCandidate>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    // Alias `i` requis par les tables dérivées du reste dû (`… ON … = i.id`).
    // Elles n'exposent que `invoice_id` et leur agrégat : les colonnes de
    // `INVOICE_COLUMNS` restent sans ambiguïté. Filtre en HAVING pour
    // référencer l'alias `amount_due` sans dupliquer l'expression (SQL
    // n'autorise pas un alias de SELECT dans le WHERE).
    sqlx::query_as::<_, UnpaidInvoiceCandidate>(&format!(
        "SELECT {INVOICE_COLUMNS}, {due} AS amount_due, {ttc} AS total_ttc \
         FROM invoices i {joins} \
         WHERE i.company_id = ? \
           AND i.status = 'validated' \
           AND i.paid_at IS NULL \
           AND i.journal_entry_id IS NOT NULL \
           AND i.date BETWEEN ? AND ? \
         HAVING amount_due BETWEEN ? - ? AND ? + ? \
         ORDER BY i.date ASC, i.id ASC \
         LIMIT 50",
        due = INVOICE_AMOUNT_DUE_DERIVED_SQL,
        ttc = crate::repositories::invoices::INVOICE_TTC_DERIVED_SQL,
        joins = amount_due_derived_joins(),
    ))
    .bind(company_id)
    .bind(window.earliest_invoice_date)
    .bind(window.latest_invoice_date)
    .bind(tx_amount)
    .bind(amount_tolerance)
    .bind(tx_amount)
    .bind(amount_tolerance)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// Charge les transactions bancaires `pending` non-rejetées pour un
/// compte donné, scopées multi-tenant, paginées via `limit`.
///
/// Filtres :
/// - `company_id` (KF-002 Pattern 1).
/// - `bank_account_id` (scope local).
/// - `status = 'pending'`.
/// - `auto_match_rejected_at IS NULL` (exclut les transactions
///   rejetées manuellement, réservées pour 8-5 manual).
///
/// Tri par `booking_date DESC, id DESC` pour présenter les
/// transactions récentes d'abord (UX nominal).
pub async fn find_pending_transactions_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    limit: i64,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? \
           AND bank_account_id = ? \
           AND status = 'pending' \
           AND auto_match_rejected_at IS NULL \
         ORDER BY booking_date DESC, id DESC \
         LIMIT ?",
    ))
    .bind(company_id)
    .bind(bank_account_id)
    .bind(limit)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// Batch-load Contacts par liste d'IDs (MP4-3 Pass 4 — éviter N×M
/// queries dans le handler GET proposals 4-pass architecture).
/// Retourne une `HashMap<i64, Contact>` keyed par `contact.id`.
///
/// **Multi-tenant safety** : filtre `company_id`.
/// **Soft-delete** (cf. L37) : ne filtre PAS `active = false` v0.1
/// — si un contact est désactivé entre import et reconciliation, le
/// matching produit quand même un contact_score (poids 0.10 donc
/// impact mineur).
pub async fn find_contacts_by_ids<'e, E>(
    executor: E,
    company_id: i64,
    ids: &[i64],
) -> Result<HashMap<i64, Contact>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    if ids.len() > MAX_IN_CLAUSE_IDS {
        return Err(DbError::Sqlx(sqlx::Error::Protocol(format!(
            "find_contacts_by_ids: ids.len()={} exceeds MAX_IN_CLAUSE_IDS={MAX_IN_CLAUSE_IDS}",
            ids.len()
        ))));
    }
    // Build IN clause manuellement avec placeholders.
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {cols} FROM contacts \
         WHERE company_id = ? AND id IN ({placeholders})",
        cols = super::contacts::COLUMNS,
    );
    let mut q = sqlx::query_as::<_, Contact>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(|c| (c.id, c)).collect())
}

/// Helper inline pour charger une `Invoice` par id scopée multi-tenant.
/// Utilisé par le handler accept (step 5) — équivalent à
/// `FIND_INVOICE_SCOPED_SQL` d'`invoices.rs:41` mais accessible
/// publiquement.
pub async fn find_invoice_by_id_for_company<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<Invoice>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Invoice>(&format!(
        "SELECT {INVOICE_COLUMNS} FROM invoices WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Cap maximum de la liste `ids` pour éviter un IN clause unbounded
/// (M9 Pass 1 code review). MariaDB tolère des IN larges mais le
/// chemin nominal de réconciliation traite des batches < 100 ; cap
/// défensif à 500 pour éviter un payload abusif.
///
/// **Publique depuis la Story 27-4** (C-27-4-20) : les routes
/// `POST /reconciliation/accept` et `/reject` refusent en `400` un lot
/// de plus de `MAX_IN_CLAUSE_IDS` identifiants **avant** toute lecture,
/// en lisant cette constante — une seule borne, pas de second littéral.
/// Elle borne aussi [`find_contacts_by_ids`], d'où son nom générique.
pub const MAX_IN_CLAUSE_IDS: usize = 500;

/// Lecture par lot des transactions d'un compte, **sans filtre de
/// statut ni de rejet** (Story 27-4, AC11 — remplace
/// `find_pending_by_ids`).
///
/// Scopée `company_id` **et** `bank_account_id` : un identifiant
/// supprimé, d'une autre société ou d'un autre compte est simplement
/// absent de la `HashMap` rendue, par le **même** chemin — c'est ce qui
/// garantit l'anti-énumération du refus `BANK_TRANSACTION_NOT_FOUND`.
///
/// Pourquoi aucun filtre : la **classification** (éligible, rapprochée,
/// rejetée, introuvable) est faite par l'appelant, une seule fois, par
/// la fonction pure des routes de réconciliation, qui appelle
/// [`super::bank_imports::is_reconciled`]. Filtrer ici ferait de toute
/// transaction non éligible une « introuvable ».
///
/// Appelants : le pré-vol de `post_accept` et celui de `post_reject`
/// (`kesh-api/src/routes/reconciliation.rs`), hors transaction et hors
/// verrou.
///
/// Retourne `Err(DbError::Sqlx(Protocol))` si
/// `ids.len() > MAX_IN_CLAUSE_IDS` (les routes refusent le lot en `400`
/// avant d'arriver ici).
pub async fn find_by_ids_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    ids: &[i64],
) -> Result<HashMap<i64, BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    if ids.len() > MAX_IN_CLAUSE_IDS {
        return Err(DbError::Sqlx(sqlx::Error::Protocol(format!(
            "find_by_ids_for_account: ids.len()={} exceeds MAX_IN_CLAUSE_IDS={MAX_IN_CLAUSE_IDS}",
            ids.len()
        ))));
    }
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? \
           AND id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql)
        .bind(company_id)
        .bind(bank_account_id);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(|t| (t.id, t)).collect())
}

/// Recharge UNE `BankTransaction` `status='pending'` (sans filtre
/// `auto_match_rejected_at`) par id pour un compte donné, scopée
/// multi-tenant. Utilisé par `accept_one` (C2 Pass 1 — TOCTOU fix)
/// pour rafraîchir l'état dans le lock après le pré-flight batch
/// hors-lock.
///
/// **Comme [`find_by_ids_for_account`]** (qui a remplacé
/// `find_pending_by_ids` à la Story 27-4), ce helper ne filtre
/// PAS `auto_match_rejected_at IS NULL`. Une transaction rejetée
/// manuellement entre le pré-flight et le lock peut encore être
/// acceptée (pattern « j'ai rejeté trop vite, je rétracte ») — le
/// caller laisse le step 4 status check trancher.
///
/// **Naming caveat (dette `dette-naming-reconciliation-helpers` —
/// F8'' Pass 3 Opus 8-5a-base)** : malgré le nom, ce helper ne
/// filtre PAS `status='pending'` côté SQL. Il appelle juste « par
/// id pour un compte ». Le filtre status est délégué au caller
/// (8-4 `accept_one` step 4). 8-5a-base introduit
/// [`find_strictly_pending_by_id_for_account`] qui, lui, filtre
/// strictement `status='pending'` côté SQL — utilisé par le flow
/// manual où l'on ne veut pas réutiliser une transaction déjà
/// `reconciled`.
pub async fn find_pending_by_id_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    id: i64,
) -> Result<Option<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&bank_tx_by_id_for_account_sql(""))
        .bind(company_id)
        .bind(bank_account_id)
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_error)
}

/// Texte SQL de la lecture d'une transaction par identifiant, scopée
/// `(company_id, bank_account_id)`, sans filtre de statut — partagé par
/// [`find_pending_by_id_for_account`] (`suffix` vide) et
/// [`find_by_id_for_account_for_update`] (`suffix` = `" FOR UPDATE"`),
/// pour que les deux lectures ne divergent pas (Story 27-4, C-27-4-32).
fn bank_tx_by_id_for_account_sql(suffix: &str) -> String {
    format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? AND id = ?{suffix}"
    )
}

/// Relit UNE transaction par identifiant, scopée `(company_id,
/// bank_account_id)`, sans filtre de statut, par une **lecture
/// verrouillante** (`FOR UPDATE`) — Story 27-4, AC4 (C-27-4-21).
///
/// **Lecture courante, et c'est tout son objet.** Sous REPEATABLE READ,
/// une `SELECT` ordinaire dans la transaction d'un lot lit la vue
/// cohérente ouverte par la première lecture du lot ; une transaction
/// supprimée (avec son import) **après** cette ouverture y apparaîtrait
/// encore. `FOR UPDATE` lit la dernière version validée et la verrouille
/// jusqu'à la fin de la transaction (verrou d'intervalle si la ligne est
/// absente) : l'appelant sait ce que la ligne est **maintenant**.
///
/// Deux appelants, dans `kesh-api/src/routes/reconciliation.rs` :
///
/// - `reject_batch`, quand l'`UPDATE` du rejet n'a touché aucune ligne — la
///   ligne relue est reclassée (introuvable, rapprochée, rejetée). Le verrou
///   porte sur une ligne que l'`UPDATE` qui précède visait déjà : même ordre,
///   aucun cycle neuf ;
/// - `unreject_once` (Story 28-5a, #526), qui lit la ligne sous le verrou du
///   compte **avant** son `UPDATE` et la classe (`classify_unreject`) : une
///   transaction supprimée avec son import pendant l'attente est vue absente
///   (`404`), là où la vue cohérente la verrait encore.
///
/// ⚠️ Appelée **hors** transaction (autocommit), elle ne verrouille rien
/// au-delà de l'instruction : réservée à un exécuteur transactionnel.
pub async fn find_by_id_for_account_for_update<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    id: i64,
) -> Result<Option<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&bank_tx_by_id_for_account_sql(" FOR UPDATE"))
        .bind(company_id)
        .bind(bank_account_id)
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_error)
}

/// Charge UNE `BankTransaction` **strictement** `status='pending'` par
/// id, scopée multi-tenant `(company_id, bank_account_id, id)`. Utilisé
/// par le flow `/manual` (8-5a-base) et `/split` (8-5a-bis) pour
/// pré-flight ownership AVANT lock ET inside lock.
///
/// Retourne `None` si introuvable, status ≠ 'pending', cross-tenant,
/// ou cross-account. Couvre en un seul code 4 cas — le caller mappe
/// vers `RECONCILIATION_TRANSACTION_NOT_PENDING` 404.
///
/// **Distinct de `find_pending_by_id_for_account` (8-4)** qui ne
/// filtre PAS `status` (cf. `dette-naming-reconciliation-helpers`).
pub async fn find_strictly_pending_by_id_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    id: i64,
) -> Result<Option<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? AND id = ? \
           AND status = 'pending'"
    ))
    .bind(company_id)
    .bind(bank_account_id)
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

#[cfg(test)]
mod tests {
    //! Tests purs de la fenêtre de dates (Story 28-7, AC6) — aucune base.

    use super::*;
    use crate::entities::bank_transaction::BankTransactionStatus;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn days(n: i64) -> chrono::Duration {
        chrono::Duration::days(n)
    }

    fn tx(booking_date: NaiveDate, value_date: Option<NaiveDate>) -> BankTransaction {
        let at = date(2026, 6, 1).and_hms_opt(0, 0, 0).unwrap();
        BankTransaction {
            id: 1,
            company_id: 1,
            import_id: 1,
            bank_account_id: 1,
            booking_date,
            value_date,
            amount: Decimal::new(10_000, 2),
            currency: "CHF".into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: at,
            updated_at: at,
        }
    }

    #[test]
    fn the_window_of_a_payment_date_is_thirty_days_before_to_one_day_after() {
        let d = date(2026, 6, 15);
        let w = InvoiceDateWindow::for_payment_date(d);
        assert_eq!(w.earliest_invoice_date, d - days(30));
        assert_eq!(w.latest_invoice_date, d + days(1));
    }

    #[test]
    fn check_admits_the_bounds_and_refuses_one_day_beyond_each() {
        let d = date(2026, 6, 15);
        let w = InvoiceDateWindow::for_payment_date(d);
        assert_eq!(w.check(d + days(1)), Ok(()), "D + 1");
        assert_eq!(
            w.check(d + days(2)),
            Err(InvoiceDateRefusal::PaymentBeforeInvoice),
            "D + 2"
        );
        assert_eq!(w.check(d), Ok(()), "D");
        assert_eq!(w.check(d - days(1)), Ok(()), "D − 1");
        assert_eq!(w.check(d - days(30)), Ok(()), "D − 30");
        assert_eq!(
            w.check(d - days(31)),
            Err(InvoiceDateRefusal::PaymentTooLate),
            "D − 31"
        );
    }

    #[test]
    fn the_payment_date_is_the_value_date_when_there_is_one() {
        let booking = date(2026, 6, 15);
        let value = date(2026, 6, 10);
        let t = tx(booking, Some(value));
        assert_eq!(payment_date(&t), value);
        assert_eq!(
            InvoiceDateWindow::for_transaction(&t),
            InvoiceDateWindow::for_payment_date(value)
        );
    }

    #[test]
    fn the_payment_date_falls_back_to_the_booking_date() {
        let booking = date(2026, 6, 15);
        let t = tx(booking, None);
        assert_eq!(payment_date(&t), booking);
        assert_eq!(
            InvoiceDateWindow::for_transaction(&t),
            InvoiceDateWindow::for_payment_date(booking)
        );
    }
}
