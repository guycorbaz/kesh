//! Rattacher une transaction bancaire au règlement client déjà saisi depuis la
//! fiche de la facture (Story 29-8, #670).
//!
//! ⛔ **Pourquoi ce geste existe.** Un règlement par virement enregistré depuis
//! la fiche de la facture (`invoice_settlements_write::settle_invoice`) passe
//! son écriture — `D banque / C créance` — sans lire ni écrire
//! `bank_transactions` : la transaction du relevé reste `pending`, et la
//! rejeter ne la rapproche pas (le rejet ne pose que `auto_match_rejected_at`).
//! Elle empêchait donc à jamais la clôture du compte bancaire
//! (`bank_accounts::close_in_tx`) et restait au contrôle K2 de la préparation
//! de clôture (`closing_checks`). Le rattachement la passe `reconciled` en
//! pointant l'écriture **existante** du règlement : **aucune** écriture, aucun
//! règlement, aucun lettrage. L'état produit est celui d'une proposition
//! « facture » acceptée (`accept_one_invoice`), à l'écart près de la 29-6.
//!
//! ⛔ **Les deux chemins qui lient une écriture existante.** Ce module en porte
//! deux : le rattachement à un **règlement client** ([`link_in_tx`], Story
//! 29-8) et, depuis la Story v017-3a (#671), le rattachement à une **écriture
//! déjà passée** du journal ([`link_entry_in_tx`]). Les six autres chemins qui
//! posent `matched_entry_id` (`routes/reconciliation.rs` : acceptation
//! facture, acceptation facture fournisseur, proposition éclatée, règle,
//! manuel, éclatement manuel) lient une écriture **qu'ils viennent de créer**.
//! L'invariant « une écriture, une transaction » — qu'aucune contrainte
//! `UNIQUE` ne défend et que supposent
//! `bank_accounts::list_by_company_with_balances` et `reconciliation_cancel`
//! (rang 3) — est tenu par la condition « non rapprochée » de chaque règle,
//! lue sous le verrou de ligne de l'objet lié : le règlement pour
//! [`link_in_tx`], l'écriture pour [`link_entry_in_tx`].
//!
//! # La règle (une seule définition)
//!
//! Un règlement `s` est **rattachable** à une transaction `t` si et seulement si :
//!
//! 1. `t` est un crédit en CHF ([`transaction_can_be_linked`]) — et, pour
//!    l'appelant, `pending` (rejetée ou non) ;
//! 2. `s` est de la même société, `settlement_type = 'bank_transfer'`, sur le
//!    **même** compte bancaire que `t` ;
//! 3. l'écriture de `s` n'est rapprochée d'aucune transaction ;
//! 4. soit `M` le mouvement de l'écriture de `s` sur le compte du grand livre
//!    **actuel** du compte bancaire : `M > 0` et `0 ≤ t.amount − M ≤
//!    tolérance` (la tolérance de la réconciliation, reçue en paramètre).
//!
//! Les conditions 2 à 4 sont **une** requête ([`candidates_sql`]) : la liste
//! des candidats ([`find_link_candidates`]) et le rattachement
//! ([`link_in_tx`], restreint à `s.id`) en lisent **le même texte**. Aucun
//! second test de montant, de compte ou de lien, ni en Rust ni en SQL.
//!
//! # Garantie d'échelle (validation P2, F8)
//!
//! `M` est la **ligne de banque** de l'écriture, au centime **par
//! construction** : un règlement saisi par l'API passe `scale_within(…, 2)`
//! (`kesh-api/src/routes/invoices.rs`, `SettleInvoiceRequest::into_parts`,
//! seul point d'entrée) ; un règlement né d'une acceptation porte `t.amount`
//! (`DECIMAL(18,2)`). ⚠️ Cette garde est **à la route**, pas dans `kesh-db`
//! (`settle_invoice` ne contrôle que la positivité). Les conditions se
//! calculent en décimal exact dans la requête ; la sortie — JSON et audit —
//! est normalisée par `Decimal::rescale(2)` (« 0.0300 » de MariaDB → « 0.03 »).
//! Dans tout état atteignable, condition exacte et sortie coïncident ; un futur
//! appelant de `settle_invoice` sans la garde de route pourrait afficher
//! « 0.00 » pour un écart de 0.004.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{MySql, Transaction};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::bank_transaction::BankTransaction;
use crate::entities::journal_entry::Journal;
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;
use crate::repositories::bank_imports::is_reconciled;
use crate::repositories::reconciliation::{find_by_id_for_account_for_update, payment_date};

/// Nombre maximal de candidats rendus (le filtre et le tri sont dans la
/// requête, **avant** `LIMIT` : leçon C-29-6-3).
pub const LINK_CANDIDATES_LIMIT: i64 = 50;

/// Condition 1 de la règle : la transaction est un **crédit** en CHF.
///
/// Fonction pure, seule définition de la condition (la liste des candidats et
/// le rattachement l'appellent). Un montant nul — `-0.00` compris — n'est pas
/// un crédit.
pub fn transaction_can_be_linked(t: &BankTransaction) -> bool {
    t.amount > Decimal::ZERO && t.currency == "CHF"
}

/// Un règlement rattachable à une transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkCandidate {
    pub settlement_id: i64,
    pub invoice_id: i64,
    pub invoice_number: Option<String>,
    pub contact_name: String,
    pub settled_on: NaiveDate,
    /// `M` — le mouvement de l'écriture du règlement sur le compte du grand
    /// livre actuel du compte bancaire, à deux décimales.
    pub amount: Decimal,
    /// `t.amount − M`, à deux décimales, entre 0 et la tolérance.
    pub gap: Decimal,
    /// `|settled_on − date de paiement de t|`, en jours. Trie, ne filtre pas.
    pub days_apart: i64,
    pub journal_entry_id: i64,
}

#[derive(sqlx::FromRow)]
struct CandidateRow {
    settlement_id: i64,
    invoice_id: i64,
    invoice_number: Option<String>,
    contact_name: String,
    settled_on: NaiveDate,
    journal_entry_id: i64,
    amount: Decimal,
    gap: Decimal,
    days_apart: i64,
}

impl From<CandidateRow> for LinkCandidate {
    fn from(r: CandidateRow) -> Self {
        let mut amount = r.amount;
        amount.rescale(2);
        let mut gap = r.gap;
        gap.rescale(2);
        Self {
            settlement_id: r.settlement_id,
            invoice_id: r.invoice_id,
            invoice_number: r.invoice_number,
            contact_name: r.contact_name,
            settled_on: r.settled_on,
            amount,
            gap,
            days_apart: r.days_apart,
            journal_entry_id: r.journal_entry_id,
        }
    }
}

/// Le texte de la requête des conditions 2 à 4 — `one` : restreinte à `s.id`.
///
/// Paramètres, dans l'ordre : `t.amount` (écart rendu), date de paiement de
/// `t` (proximité), compte du grand livre actuel, `company_id`,
/// `t.bank_account_id`, `t.amount`, `t.amount`, tolérance, et — `one` —
/// l'identifiant du règlement.
///
/// - Le montant `M` se somme dans une **table dérivée par écriture**, en
///   **jointure interne** sur les lignes du compte lié actuel : une écriture
///   sans ligne sur ce compte (règlement passé sur un **ancien** compte lié)
///   ne produit aucune ligne — c'est cette jointure qui écarte le cas `M = 0`.
///   Le `HAVING … > 0` ne décide que d'une écriture qui a des lignes sur ce
///   compte et dont la somme est ≤ 0, ce qu'aucun état courant ne produit.
/// - ⛔ `bank_transactions` n'entre **jamais** dans le `FROM` de la somme (le
///   produit multiplierait les lignes) : la condition 3 est un `NOT EXISTS`.
/// - La proximité lit la date de paiement **liée en paramètre**
///   ([`payment_date`]) : la requête ne réécrit jamais
///   `COALESCE(value_date, booking_date)`.
fn candidates_sql(one: bool) -> String {
    format!(
        "SELECT s.id AS settlement_id, s.invoice_id, i.invoice_number, \
                c.name AS contact_name, s.settled_on, s.journal_entry_id, \
                m.movement AS amount, \
                ? - m.movement AS gap, \
                CAST(ABS(DATEDIFF(s.settled_on, ?)) AS SIGNED) AS days_apart \
         FROM invoice_settlements s \
         JOIN (SELECT l.entry_id, SUM(l.debit) - SUM(l.credit) AS movement \
                 FROM journal_entry_lines l \
                WHERE l.account_id = ? \
                GROUP BY l.entry_id \
               HAVING SUM(l.debit) - SUM(l.credit) > 0) m ON m.entry_id = s.journal_entry_id \
         JOIN invoices i ON i.id = s.invoice_id \
         JOIN contacts c ON c.id = i.contact_id \
         WHERE s.company_id = ? \
           AND s.settlement_type = 'bank_transfer' \
           AND s.settlement_bank_account_id = ? \
           AND NOT EXISTS (SELECT 1 FROM bank_transactions bt WHERE bt.matched_entry_id = s.journal_entry_id) \
           AND ? - m.movement >= 0 \
           AND ? - m.movement <= ?{} \
         ORDER BY gap, days_apart, s.id \
         LIMIT {LINK_CANDIDATES_LIMIT}",
        if one { " AND s.id = ?" } else { "" }
    )
}

/// Exécute [`candidates_sql`] pour `t`.
async fn query_candidates<'e, E>(
    executor: E,
    company_id: i64,
    t: &BankTransaction,
    bank_ledger_account_id: i64,
    amount_tolerance: Decimal,
    settlement_id: Option<i64>,
) -> Result<Vec<LinkCandidate>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let sql = candidates_sql(settlement_id.is_some());
    let mut q = sqlx::query_as::<_, CandidateRow>(&sql)
        .bind(t.amount)
        .bind(payment_date(t))
        .bind(bank_ledger_account_id)
        .bind(company_id)
        .bind(t.bank_account_id)
        .bind(t.amount)
        .bind(t.amount)
        .bind(amount_tolerance);
    if let Some(sid) = settlement_id {
        q = q.bind(sid);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(LinkCandidate::from).collect())
}

/// Les règlements rattachables à `t` — conditions 1 à 4 —, triés par écart
/// croissant, puis jours d'écart croissants, puis identifiant ;
/// [`LINK_CANDIDATES_LIMIT`] au plus. Vide si la condition 1 est fausse.
///
/// `bank_ledger_account_id` : le compte du grand livre **actuel** du compte
/// bancaire de `t` ; `amount_tolerance` : la tolérance de la réconciliation
/// (`AMOUNT_TOLERANCE_HUNDREDTHS` de `kesh-api`, liée en `Decimal`).
/// L'appelant a vérifié que `t` est `pending`. Lecture non verrouillante.
pub async fn find_link_candidates<'e, E>(
    executor: E,
    company_id: i64,
    t: &BankTransaction,
    bank_ledger_account_id: i64,
    amount_tolerance: Decimal,
) -> Result<Vec<LinkCandidate>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if !transaction_can_be_linked(t) {
        return Ok(Vec::new());
    }
    query_candidates(
        executor,
        company_id,
        t,
        bank_ledger_account_id,
        amount_tolerance,
        None,
    )
    .await
}

/// Ce qu'on demande à [`link_in_tx`].
#[derive(Debug, Clone, Copy)]
pub struct LinkRequest {
    pub company_id: i64,
    /// Le compte bancaire de la transaction, lu au pré-vol (il ne change
    /// jamais pour une transaction).
    pub bank_account_id: i64,
    pub bank_transaction_id: i64,
    pub settlement_id: i64,
    /// Le compte du grand livre lié au compte bancaire, lu au pré-vol (comme
    /// `post_manual`, C-29-8-31).
    pub bank_ledger_account_id: i64,
    pub amount_tolerance: Decimal,
    pub user_id: i64,
    pub actor_api_key_id: Option<i64>,
}

/// Un refus du rattachement — à un règlement ([`link_in_tx`]) ou à une
/// écriture ([`link_entry_in_tx`], Story v017-3a) —, sans rien écrire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkRefusal {
    /// Transaction, règlement ou écriture introuvable, ou d'une autre société.
    NotFound,
    /// La transaction est déjà rapprochée.
    NotPending,
    /// Non rattachable : conditions 1 à 4 de la règle des règlements, ou 1 à 6
    /// de celle des écritures.
    NotLinkable,
}

/// Ce qu'a produit un rattachement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Linked {
    pub journal_entry_id: i64,
    pub invoice_id: i64,
    pub settlement_id: i64,
}

/// Rattache la transaction au règlement, **dans une transaction fournie**,
/// sous le verrou nommé du compte que tient l'appelant (`with_account_lock`).
/// Sans `BEGIN` ni `COMMIT`.
///
/// # Ordre des verrous (lock-order), imposé
///
/// verrou nommé (appelant) → `bank_transactions` (`t`) → `invoice_settlements`
/// (`s`) → `journal_entries` (`je`) : **trois verrous de ligne par clé
/// primaire, rien d'autre** ; puis la première lecture ordinaire,
/// `bank_accounts::ensure_open_in_tx`, qui fait naître la vue — **après** le
/// dernier verrou (règle de placement de `ensure_open_in_tx`). Le
/// rattachement ne prend **jamais** la ligne `invoices`, ni `bank_accounts`,
/// ni `fiscal_years` : il n'écrit rien au journal et ne doit pas faire
/// attendre une clôture d'exercice (C-29-8-21). L'ordre `s → je`, en deux
/// instructions, est celui de `cancel_settlement_in_tx`.
///
/// ⛔ **Aucune lecture verrouillante sur `matched_entry_id`** : sur un index
/// secondaire non unique, un `FOR UPDATE` qui ne trouve rien pose un verrou
/// d'intervalle ; deux rattachements simultanés sur deux comptes dont les
/// écritures tombent dans le même intervalle s'interbloqueraient à l'`UPDATE`.
///
/// # L'invariant « une écriture, une transaction » (condition 3)
///
/// Tenu par la lecture ordinaire de l'étape 4.4, **sous le verrou de ligne du
/// règlement** :
///
/// - le verrou nommé ne sérialise **pas** : `with_account_lock` le relâche
///   (`RELEASE_LOCK`) **avant** que la route commite ;
/// - c'est le verrou de ligne de `s` qui sérialise : deux rattachements du
///   même règlement visent deux transactions du **même** compte (condition 2) ;
///   le second attend à l'étape 4.2 (a) que le premier **commite** ; sa vue
///   naît après l'attente, à l'étape 4.3 ; l'étape 4.4 voit donc le
///   `matched_entry_id` commité par le premier : refus, rien d'écrit ;
/// - les autres écrivains ne visent jamais l'écriture d'un règlement
///   existant, sauf un : les six chemins d'acceptation et de rapprochement
///   lient une écriture qu'ils viennent de créer, invisible avant leur commit ;
///   l'annulation (`reconciliation_cancel::cancel_in_tx`) la remet à `NULL`,
///   sous le verrou de `t` puis de `s` — même verrou de `s` ; et le
///   rattachement à une écriture ([`link_entry_in_tx`], Story v017-3a) peut
///   viser l'écriture d'un règlement par virement sur un **autre** compte
///   bancaire partageant le compte du grand livre (fiche v017-3a, AC-2.12) :
///   il ne prend pas `s`, mais prend la ligne de `je` `FOR UPDATE` avant sa
///   première lecture ordinaire, comme ce geste-ci à l'étape 4.2 (b) — c'est
///   ce verrou-là qui les sérialise, sans cycle (aucun des deux ne prend `s`
///   après `je`) ;
/// - ⚠️ `SELECT GET_LOCK` ne fait pas naître la vue (il ne lit aucune table
///   InnoDB), et les lectures du pré-vol se font sur le pool, hors de la
///   transaction. **Raisonné sur le code et la documentation d'InnoDB, non
///   prouvé par un test** (C-29-8-25) : renvoyé à la revue de code ;
/// - corollaire : **toute** lecture ordinaire déplacée avant l'étape 4.2 (a)
///   rouvre le double rattachement (mutation M14).
///
/// # Étapes
///
/// 4.1 `t` `FOR UPDATE` → absente : [`LinkRefusal::NotFound`] ; rapprochée :
/// [`LinkRefusal::NotPending`] ; condition 1 fausse :
/// [`LinkRefusal::NotLinkable`]. 4.2 (a) `s` `FOR UPDATE` → absente ou d'une
/// autre société : `NotFound` ; (b) `je` `FOR UPDATE`. 4.3
/// `ensure_open_in_tx` → `DbError::BankAccountClosed`. 4.4 la requête de la
/// règle restreinte à `s.id` → absente : `NotLinkable`. 5 `UPDATE` gardé
/// (`DbError::OptimisticLockConflict` si aucune ligne). 6 trace
/// `reconciliation.linked`.
pub async fn link_in_tx(
    tx: &mut Transaction<'_, MySql>,
    req: LinkRequest,
) -> Result<Result<Linked, LinkRefusal>, DbError> {
    // 4.1 — la transaction, verrouillée.
    let Some(t) = find_by_id_for_account_for_update(
        &mut **tx,
        req.company_id,
        req.bank_account_id,
        req.bank_transaction_id,
    )
    .await?
    else {
        return Ok(Err(LinkRefusal::NotFound));
    };
    if is_reconciled(t.status.as_str(), t.matched_entry_id) {
        return Ok(Err(LinkRefusal::NotPending));
    }
    if !transaction_can_be_linked(&t) {
        return Ok(Err(LinkRefusal::NotLinkable));
    }

    // 4.2 (a) — le règlement, verrouillé, puis (b) son écriture.
    let entry: Option<i64> = sqlx::query_scalar(
        "SELECT journal_entry_id FROM invoice_settlements WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(req.settlement_id)
    .bind(req.company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let Some(journal_entry_id) = entry else {
        return Ok(Err(LinkRefusal::NotFound));
    };
    let je: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM journal_entries WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(journal_entry_id)
    .bind(req.company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if je.is_none() {
        // Inatteignable : la clé étrangère du règlement tient son écriture.
        return Err(DbError::Invariant(format!(
            "règlement {} sans écriture {journal_entry_id}",
            req.settlement_id
        )));
    }

    // 4.3 — garde non verrouillante, première lecture ordinaire : APRÈS le
    // dernier verrou (règle de placement de `ensure_open_in_tx`).
    super::bank_accounts::ensure_open_in_tx(tx, req.company_id, t.bank_account_id).await?;

    // 4.4 — la règle, restreinte à ce règlement.
    let candidate = query_candidates(
        &mut **tx,
        req.company_id,
        &t,
        req.bank_ledger_account_id,
        req.amount_tolerance,
        Some(req.settlement_id),
    )
    .await?
    .into_iter()
    .next();
    let Some(candidate) = candidate else {
        return Ok(Err(LinkRefusal::NotLinkable));
    };

    // 5 — le lien. Les gardes `status` et `matched_entry_id IS NULL` sont des
    // défenses en profondeur : l'étape 4.1 a lu et refusé ces états sous le
    // verrou de la même ligne, tenu jusqu'au commit.
    let result = sqlx::query(
        "UPDATE bank_transactions \
         SET status = 'reconciled', matched_entry_id = ?, auto_match_rejected_at = NULL, \
             updated_at = NOW(3), version = version + 1 \
         WHERE id = ? AND company_id = ? AND status = 'pending' AND matched_entry_id IS NULL",
    )
    .bind(candidate.journal_entry_id)
    .bind(req.bank_transaction_id)
    .bind(req.company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if result.rows_affected() != 1 {
        return Err(DbError::OptimisticLockConflict);
    }

    // 6 — la trace, dans la même transaction.
    let mut amount = t.amount;
    amount.rescale(2);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            req.user_id,
            req.actor_api_key_id,
            "reconciliation.linked",
            "bank_transaction",
            req.bank_transaction_id,
            Some(serde_json::json!({
                "bank_transaction_id": req.bank_transaction_id,
                "settlement_id": candidate.settlement_id,
                "invoice_id": candidate.invoice_id,
                "journal_entry_id": candidate.journal_entry_id,
                "amount": amount.to_string(),
                "settlement_amount": candidate.amount.to_string(),
                "gap": candidate.gap.to_string(),
                "was_previously_rejected": t.auto_match_rejected_at.is_some(),
            })),
        ),
    )
    .await?;

    Ok(Ok(Linked {
        journal_entry_id: candidate.journal_entry_id,
        invoice_id: candidate.invoice_id,
        settlement_id: candidate.settlement_id,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Story v017-3a (#671) — rattacher une transaction à une écriture déjà passée
// ─────────────────────────────────────────────────────────────────────────────
//
// ⛔ **Pourquoi ce geste existe.** Un mouvement bancaire déjà comptabilisé —
// facture fournisseur payée depuis sa fiche (`supplier_invoices::pay_in_tx_as`),
// lot pain.001 d'une facture, écriture saisie au journal, règlement client par
// compte interne sur le compte de la banque — laissait sa transaction `pending`
// pour toujours : aucun chemin ne lisait l'écriture existante, la rejeter ne la
// rapproche pas, et l'affecter passe une **seconde** écriture. Elle empêchait la
// clôture du compte bancaire et restait au contrôle K2. Le rattachement la passe
// `reconciled` en pointant l'écriture **existante** : aucune écriture, aucun
// règlement, aucun lettrage. Kesh **liste**, l'utilisateur **choisit** : rien
// n'est rattaché d'office (règle « un appariement automatique propose, il ne
// crée jamais »).
//
// # La règle (une seule définition)
//
// Une écriture `e` est **rattachable** à une transaction `t` si et seulement si :
//
// 1. `t` est en CHF, de montant non nul ([`transaction_can_be_linked_to_entry`])
//    — et, pour l'appelant, `pending` (rejetée ou non) ;
// 2. `e` est de la même société ;
// 3. son mouvement `M` sur le compte du grand livre **actuel** du compte
//    bancaire de `t` (`SUM(debit) − SUM(credit)` de ses lignes sur ce compte)
//    **égale** `t.amount`, en décimal exact — un crédit de la banque appelle
//    `M > 0`, un débit `M < 0` ;
// 4. `e` n'est rapprochée d'aucune transaction ;
// 5. `e` ne contre-passe rien et n'est contre-passée par rien ;
// 6. `e` n'est pas l'écriture d'un règlement client par virement **sur le
//    compte bancaire de `t`** — celle-là relève de la Story 29-8
//    ([`find_link_candidates`]).
//
// Les conditions 2 à 6 sont **une** requête ([`entry_candidates_sql`]) : la
// liste ([`find_entry_candidates`]) et le geste ([`link_entry_in_tx`], restreint
// à `e.id`) en lisent le même texte.

/// Nombre maximal d'écritures candidates rendues (filtre et tri dans la
/// requête, **avant** `LIMIT`).
pub const ENTRY_CANDIDATES_LIMIT: i64 = 50;

/// Condition 1 de la règle du rattachement à une écriture : la transaction est
/// en CHF, de montant **non nul** — crédit ou débit. Un montant nul (`-0.00`
/// compris : `Decimal::is_zero` ne regarde pas le signe) n'a pas de mouvement
/// à égaler.
///
/// Fonction pure, seule définition de la condition (la liste et le geste
/// l'appellent ; l'écran en tient le miroir, `canLink`).
pub fn transaction_can_be_linked_to_entry(t: &BankTransaction) -> bool {
    !t.amount.is_zero() && t.currency == "CHF"
}

/// Une écriture déjà passée à laquelle la transaction se rattache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryCandidate {
    pub journal_entry_id: i64,
    pub entry_number: i64,
    pub fiscal_year_id: i64,
    pub entry_date: NaiveDate,
    pub journal: Journal,
    pub description: String,
    /// `M` — le mouvement de l'écriture sur le compte du grand livre actuel du
    /// compte bancaire, à deux décimales, signé (égal à `t.amount`).
    pub amount: Decimal,
    /// `|entry_date − date de paiement de t|`, en jours. Trie, ne filtre pas.
    pub days_apart: i64,
}

#[derive(sqlx::FromRow)]
struct EntryCandidateRow {
    journal_entry_id: i64,
    entry_number: i64,
    fiscal_year_id: i64,
    entry_date: NaiveDate,
    journal: Journal,
    description: String,
    amount: Decimal,
    days_apart: i64,
}

impl From<EntryCandidateRow> for EntryCandidate {
    fn from(r: EntryCandidateRow) -> Self {
        let mut amount = r.amount;
        amount.rescale(2);
        Self {
            journal_entry_id: r.journal_entry_id,
            entry_number: r.entry_number,
            fiscal_year_id: r.fiscal_year_id,
            entry_date: r.entry_date,
            journal: r.journal,
            description: r.description,
            amount,
            days_apart: r.days_apart,
        }
    }
}

/// Le texte de la requête des conditions 2 à 6 — `one` : restreinte à `e.id`.
///
/// Paramètres, dans l'ordre : date de paiement de `t` (proximité), compte du
/// grand livre actuel, `company_id`, `t.amount`, `t.bank_account_id`, et —
/// `one` — l'identifiant de l'écriture.
///
/// - Le mouvement `M` se somme dans une **table dérivée par écriture**, en
///   jointure interne sur les lignes du compte lié actuel. ⚠️ Ici la jointure
///   est **redondante** avec la condition 3 : `M = t.amount ≠ 0` (condition 1)
///   écarte déjà une écriture sans ligne sur ce compte — elle ne porte aucune
///   garde que la condition 3 ne porte (au contraire de la 29-8, où c'est
///   elle qui écarte le cas `M = 0`). Le filtre `l.account_id = ?`, lui, est
///   porteur : sans lui la somme couvrirait toutes les lignes d'une écriture
///   équilibrée, `M = 0` (mutation M2bis).
/// - ⛔ `bank_transactions` n'entre **jamais** dans le `FROM` de la somme (le
///   produit multiplierait les lignes) : la condition 4 est un `NOT EXISTS`.
/// - La proximité lit la date de paiement **liée en paramètre**
///   ([`payment_date`]) : la requête ne réécrit jamais
///   `COALESCE(value_date, booking_date)`.
/// - Tri par jours d'écart, puis identifiant ; [`ENTRY_CANDIDATES_LIMIT`] au
///   plus, filtre et tri **avant** `LIMIT`.
fn entry_candidates_sql(one: bool) -> String {
    format!(
        "SELECT e.id AS journal_entry_id, e.entry_number, e.fiscal_year_id, e.entry_date, \
                e.journal, e.description, m.movement AS amount, \
                CAST(ABS(DATEDIFF(e.entry_date, ?)) AS SIGNED) AS days_apart \
         FROM journal_entries e \
         JOIN (SELECT l.entry_id, SUM(l.debit) - SUM(l.credit) AS movement \
                 FROM journal_entry_lines l \
                WHERE l.account_id = ? \
                GROUP BY l.entry_id) m ON m.entry_id = e.id \
         WHERE e.company_id = ? \
           AND m.movement = ? \
           AND NOT EXISTS (SELECT 1 FROM bank_transactions bt WHERE bt.matched_entry_id = e.id) \
           AND e.reverses_entry_id IS NULL \
           AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id) \
           AND NOT EXISTS (SELECT 1 FROM invoice_settlements s \
                            WHERE s.journal_entry_id = e.id \
                              AND s.settlement_type = 'bank_transfer' \
                              AND s.settlement_bank_account_id = ?){} \
         ORDER BY days_apart, e.id \
         LIMIT {ENTRY_CANDIDATES_LIMIT}",
        if one { " AND e.id = ?" } else { "" }
    )
}

/// Exécute [`entry_candidates_sql`] pour `t`.
async fn query_entry_candidates<'e, E>(
    executor: E,
    company_id: i64,
    t: &BankTransaction,
    bank_ledger_account_id: i64,
    journal_entry_id: Option<i64>,
) -> Result<Vec<EntryCandidate>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let sql = entry_candidates_sql(journal_entry_id.is_some());
    let mut q = sqlx::query_as::<_, EntryCandidateRow>(&sql)
        .bind(payment_date(t))
        .bind(bank_ledger_account_id)
        .bind(company_id)
        .bind(t.amount)
        .bind(t.bank_account_id);
    if let Some(id) = journal_entry_id {
        q = q.bind(id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(EntryCandidate::from).collect())
}

/// Les écritures rattachables à `t` — conditions 1 à 6 —, triées par jours
/// d'écart croissants puis identifiant ; [`ENTRY_CANDIDATES_LIMIT`] au plus.
/// Vide, **sans requête**, si la condition 1 est fausse.
///
/// `bank_ledger_account_id` : le compte du grand livre **actuel** du compte
/// bancaire de `t`. L'appelant a vérifié que `t` est `pending`. Lecture non
/// verrouillante ; rien n'est écrit, rien n'est présélectionné.
pub async fn find_entry_candidates<'e, E>(
    executor: E,
    company_id: i64,
    t: &BankTransaction,
    bank_ledger_account_id: i64,
) -> Result<Vec<EntryCandidate>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if !transaction_can_be_linked_to_entry(t) {
        return Ok(Vec::new());
    }
    query_entry_candidates(executor, company_id, t, bank_ledger_account_id, None).await
}

/// Ce qu'on demande à [`link_entry_in_tx`].
#[derive(Debug, Clone, Copy)]
pub struct LinkEntryRequest {
    pub company_id: i64,
    /// Le compte bancaire de la transaction, lu au pré-vol.
    pub bank_account_id: i64,
    pub bank_transaction_id: i64,
    /// L'écriture désignée par le corps de la requête — candidate ou non.
    pub journal_entry_id: i64,
    /// Le compte du grand livre lié au compte bancaire, lu au pré-vol.
    pub bank_ledger_account_id: i64,
    pub user_id: i64,
    pub actor_api_key_id: Option<i64>,
}

/// Ce qu'a produit un rattachement à une écriture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryLinked {
    pub journal_entry_id: i64,
}

/// Rattache la transaction à l'écriture désignée, **dans une transaction
/// fournie**, sous le verrou nommé du compte que tient l'appelant
/// (`with_account_lock`). Sans `BEGIN` ni `COMMIT`.
///
/// # Ordre des verrous (lock-order), imposé
///
/// verrou nommé (appelant) → `bank_transactions` (`t`) → `journal_entries`
/// (`je`) : **deux verrous de ligne par clé primaire, rien d'autre** ; puis la
/// première lecture ordinaire, `bank_accounts::ensure_open_in_tx`, qui fait
/// naître la vue — **après** le dernier verrou. L'étape 4.2 verrouille
/// l'écriture **désignée par le corps**, candidate ou non : le refus
/// `NotLinkable` des conditions 2 à 6 ne vient qu'en 4.4. Le geste ne prend
/// **jamais** `invoice_settlements`, `supplier_invoices`, `invoices`,
/// `bank_accounts` ni `fiscal_years` : il n'écrit rien au journal.
///
/// ⛔ **Aucune lecture verrouillante sur `matched_entry_id`** : sur cet index
/// secondaire non unique, un `FOR UPDATE` qui ne trouve rien pose un verrou
/// d'intervalle ; deux rattachements simultanés sur deux comptes dont les
/// écritures tombent dans le même intervalle s'interbloqueraient à l'`UPDATE`
/// (raison reprise de [`link_in_tx`]).
///
/// # Qui sérialise : le verrou de ligne de `je`
///
/// - le verrou nommé n'assure qu'une **exclusion mutuelle pendant les
///   instructions** : `with_account_lock` le rend (`RELEASE_LOCK`) **avant**
///   que la route commite ; après lui, le geste n'attend plus rien, il commite ;
/// - deux rattachements de la même écriture — du même compte, ou de deux
///   comptes bancaires partageant le compte du grand livre (fiche, AC-2.12) —
///   se croisent sur la ligne de `je` : le second attend à l'étape 4.2 que le
///   premier **commite** ; sa vue naît après l'attente, à l'étape 4.3 ;
///   l'étape 4.4 voit donc le `matched_entry_id` commité par le premier : refus,
///   rien d'écrit. Le rattachement à un règlement ([`link_in_tx`]) prend lui
///   aussi `je` `FOR UPDATE` avant sa première lecture ordinaire : même
///   sérialisation ;
/// - une contre-passation, une modification ou une suppression de l'écriture
///   (`journal_entries::reverse_in_tx_inner`, `update_in_tx`, `delete_in_tx`)
///   prend `je` `FOR UPDATE` **en premier acte**, puis lit `bank_transactions`
///   sans verrou (`reversal_blockers` → `document_owners`, ou la garde
///   d'écriture) : elle attend le rattachement et voit son lien
///   (`MATCHED_BANK_TRANSACTION`), ou le rattachement l'attend et voit la
///   contre-passation (condition 5) ;
/// - ⚠️ `SELECT GET_LOCK` ne fait pas naître la vue, et les lectures du
///   pré-vol se font sur le pool : **raisonné sur le code et la documentation
///   d'InnoDB** ; l'entrelacement est forcé par le test T-race-3
///   (`reconciliation_link_entry_e2e.rs`) ;
/// - corollaire : **toute** lecture ordinaire déplacée avant l'étape 4.2
///   rouvre le double rattachement (mutation M10).
///
/// # L'absence de cycle `t → je` (fiche v017-3a, AC-3 (b))
///
/// Ce geste prend `t` puis `je`. Un cycle exigerait un geste qui tienne `je`
/// puis attende `t`. Ceux qui créent une écriture **puis** mettent à jour une
/// transaction (les quatre acceptations du lot, le rapprochement et
/// l'éclatement manuels), et le lot qui lettre une facture avant la
/// transaction suivante, tiennent tous le **verrou nommé du compte de la
/// transaction qu'ils écrivent**, pris avant : ils s'excluent avec ce geste sur
/// le même compte, et n'écrivent aucune transaction d'un autre compte. Les
/// gestes sans verrou nommé (routes du journal, suppression d'un import) ne
/// prennent jamais `t` après `je`. La suppression d'une écriture fait poser à
/// InnoDB, au titre de la clé étrangère `ON DELETE SET NULL` de
/// `matched_entry_id`, un verrou d'**intervalle** sur l'index qu'elle parcourt
/// (sans ligne trouvée : sa garde a refusé une écriture rapprochée) ; il
/// n'attend jamais, mais il peut **bloquer** l'`UPDATE` de l'étape 5
/// (insertion dans l'index secondaire) : le rattachement attend alors la fin de
/// la suppression, la suppression n'attend rien. Non prouvé par un test,
/// comme C-29-8-25.
///
/// # Étapes
///
/// 4.1 `t` `FOR UPDATE` → absente : [`LinkRefusal::NotFound`] ; rapprochée :
/// [`LinkRefusal::NotPending`] ; condition 1 fausse :
/// [`LinkRefusal::NotLinkable`] (avant toute lecture de l'écriture). 4.2 `je`
/// `FOR UPDATE` → absente ou d'une autre société : `NotFound`. 4.3
/// `ensure_open_in_tx` → `DbError::BankAccountClosed`. 4.4 la requête de la
/// règle restreinte à `e.id` → absente : `NotLinkable`. 5 `UPDATE` gardé
/// (`DbError::OptimisticLockConflict` si aucune ligne). 6 trace
/// `reconciliation.entry_linked`.
pub async fn link_entry_in_tx(
    tx: &mut Transaction<'_, MySql>,
    req: LinkEntryRequest,
) -> Result<Result<EntryLinked, LinkRefusal>, DbError> {
    // 4.1 — la transaction, verrouillée.
    let Some(t) = find_by_id_for_account_for_update(
        &mut **tx,
        req.company_id,
        req.bank_account_id,
        req.bank_transaction_id,
    )
    .await?
    else {
        return Ok(Err(LinkRefusal::NotFound));
    };
    if is_reconciled(t.status.as_str(), t.matched_entry_id) {
        return Ok(Err(LinkRefusal::NotPending));
    }
    if !transaction_can_be_linked_to_entry(&t) {
        return Ok(Err(LinkRefusal::NotLinkable));
    }

    // 4.2 — l'écriture désignée, verrouillée (candidate ou non).
    let je: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM journal_entries WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(req.journal_entry_id)
    .bind(req.company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if je.is_none() {
        return Ok(Err(LinkRefusal::NotFound));
    }

    // 4.3 — garde non verrouillante, première lecture ordinaire : APRÈS le
    // dernier verrou (règle de placement de `ensure_open_in_tx`).
    super::bank_accounts::ensure_open_in_tx(tx, req.company_id, t.bank_account_id).await?;

    // 4.4 — la règle, restreinte à cette écriture.
    let candidate = query_entry_candidates(
        &mut **tx,
        req.company_id,
        &t,
        req.bank_ledger_account_id,
        Some(req.journal_entry_id),
    )
    .await?
    .into_iter()
    .next();
    let Some(candidate) = candidate else {
        return Ok(Err(LinkRefusal::NotLinkable));
    };

    // 5 — le lien. Les gardes `status` et `matched_entry_id IS NULL` sont des
    // défenses en profondeur : l'étape 4.1 a lu et refusé ces états sous le
    // verrou de la même ligne, tenu jusqu'au commit.
    let result = sqlx::query(
        "UPDATE bank_transactions \
         SET status = 'reconciled', matched_entry_id = ?, auto_match_rejected_at = NULL, \
             updated_at = NOW(3), version = version + 1 \
         WHERE id = ? AND company_id = ? AND status = 'pending' AND matched_entry_id IS NULL",
    )
    .bind(candidate.journal_entry_id)
    .bind(req.bank_transaction_id)
    .bind(req.company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if result.rows_affected() != 1 {
        return Err(DbError::OptimisticLockConflict);
    }

    // 6 — la trace, dans la même transaction.
    let mut amount = t.amount;
    amount.rescale(2);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            req.user_id,
            req.actor_api_key_id,
            "reconciliation.entry_linked",
            "bank_transaction",
            req.bank_transaction_id,
            Some(serde_json::json!({
                "bank_transaction_id": req.bank_transaction_id,
                "journal_entry_id": candidate.journal_entry_id,
                "entry_number": candidate.entry_number,
                "fiscal_year_id": candidate.fiscal_year_id,
                "amount": amount.to_string(),
                "was_previously_rejected": t.auto_match_rejected_at.is_some(),
            })),
        ),
    )
    .await?;

    Ok(Ok(EntryLinked {
        journal_entry_id: candidate.journal_entry_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::bank_transaction::BankTransactionStatus;
    use std::str::FromStr;

    fn tx(amount: &str, currency: &str) -> BankTransaction {
        let now = chrono::Utc::now().naive_utc();
        BankTransaction {
            id: 1,
            company_id: 1,
            import_id: 1,
            bank_account_id: 1,
            booking_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            value_date: None,
            amount: Decimal::from_str(amount).unwrap(),
            currency: currency.into(),
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
            created_at: now,
            updated_at: now,
        }
    }

    /// Condition 1 : un crédit en CHF, et rien d'autre (tue M6).
    #[test]
    fn transaction_can_be_linked_is_a_chf_credit() {
        assert!(transaction_can_be_linked(&tx("0.01", "CHF")));
        assert!(transaction_can_be_linked(&tx("108.10", "CHF")));
        assert!(!transaction_can_be_linked(&tx("0.00", "CHF")));
        assert!(!transaction_can_be_linked(&tx("-0.00", "CHF")));
        assert!(!transaction_can_be_linked(&tx("-108.10", "CHF")));
        assert!(!transaction_can_be_linked(&tx("108.10", "EUR")));
    }

    /// La requête restreinte et la liste ne diffèrent que par `s.id` : un seul
    /// texte pour les conditions 2 à 4 (C-29-8-9).
    #[test]
    fn the_restricted_query_is_the_list_query_plus_the_settlement() {
        let all = candidates_sql(false);
        let one = candidates_sql(true);
        assert_eq!(one.replace(" AND s.id = ?", ""), all);
        assert!(!all.contains("FOR UPDATE"), "lecture ordinaire (C-29-8-17)");
    }

    /// Story v017-3a, condition 1 : une transaction en CHF de montant non nul,
    /// crédit **ou débit** — `-0.00` compris comme nul (tue M8, M27).
    #[test]
    fn transaction_can_be_linked_to_entry_is_a_non_zero_chf_amount() {
        assert!(transaction_can_be_linked_to_entry(&tx("0.01", "CHF")));
        assert!(transaction_can_be_linked_to_entry(&tx("-250.00", "CHF")));
        assert!(transaction_can_be_linked_to_entry(&tx("-0.01", "CHF")));
        assert!(!transaction_can_be_linked_to_entry(&tx("0.00", "CHF")));
        assert!(!transaction_can_be_linked_to_entry(&tx("-0.00", "CHF")));
        assert!(!transaction_can_be_linked_to_entry(&tx("-250.00", "EUR")));
        assert!(!transaction_can_be_linked_to_entry(&tx("250.00", "EUR")));
    }

    /// Story v017-3a (AC-1) : la requête restreinte du geste est la requête de
    /// la liste plus `AND e.id = ?`, sans verrou — un seul texte pour les
    /// conditions 2 à 6.
    #[test]
    fn the_restricted_entry_query_is_the_list_query_plus_the_entry() {
        let all = entry_candidates_sql(false);
        let one = entry_candidates_sql(true);
        assert_eq!(one.replace(" AND e.id = ?", ""), all);
        assert_ne!(one, all);
        assert!(!all.contains("FOR UPDATE"), "lecture ordinaire");
        assert!(!one.contains("FOR UPDATE"), "lecture ordinaire");
    }
}
