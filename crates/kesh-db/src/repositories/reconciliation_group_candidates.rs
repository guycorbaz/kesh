//! Story 43-2 (#823) — les factures **candidates** d'un règlement groupé : la
//! recherche de l'écran qui impute une transaction sur plusieurs factures.
//!
//! **Une lecture, deux familles** : un crédit cherche des factures clients
//! `validated` à reste dû positif, un débit des factures fournisseur `open`
//! hors lot de paiement `generated`. **Tous tiers** : la liste mêle les
//! clients (ou les fournisseurs), c'est l'objet du geste (une banque qui
//! verse, pour plusieurs débiteurs, un seul montant).
//!
//! **La liste n'offre jamais ce que le règlement refuserait** (principe de
//! `link_preflight`, `kesh-api`) : mêmes critères d'éligibilité que
//! `grouped_settlements::settle_invoices_in_tx` (client : statut, `paid_at`,
//! écriture de vente, reste dû brut `> 0`) et la **borne haute** de la fenêtre
//! de dates (`InvoiceDateWindow::latest_invoice_date`), seule borne que le
//! règlement groupé contrôle (`PaymentBeforeInvoiceDate`). La liste est plus
//! étroite que le règlement sur un point, écrit : une facture fournisseur à
//! TTC nul n'est pas offerte.
//!
//! **Recherche** (`docs/search-patterns.md` § 1 à 4 et § 6) : le nom du tiers
//! en `FULLTEXT` (`ft_contacts_name`, préfixe sur le dernier jeton), avec un
//! repli `LIKE` quand le terme n'a aucun jeton de trois caractères ou plus
//! (sinon « BL SA » ne trouverait rien, l'index ignorant les jetons courts) ;
//! les numéros et la référence en `LIKE … ESCAPE`. La disjonction est la même
//! dans les deux branches.
//!
//! **Échantillon déterminé** : au plus [`GROUP_CANDIDATES_SCAN_LIMIT`]
//! factures, `ORDER BY` date puis identifiant ; l'appelant les trie par
//! l'ordre d'imputation (`reconciliation::imputation_order`, que
//! [`GroupCandidate`] implémente par [`PayableDocument`]) puis les tronque.
//! [`GroupCandidates::scan_limit_reached`] dit si le plafond a coupé.
//!
//! Lecture ordinaire, sans verrou ; scopée par société (KF-002).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{MySql, QueryBuilder};

use crate::errors::{DbError, map_db_error};
use crate::repositories::invoice_settlements::{
    INVOICE_AMOUNT_DUE_DERIVED_SQL, amount_due_derived_joins,
};
use crate::repositories::invoices::INVOICE_TTC_DERIVED_SQL;
use crate::repositories::reconciliation::PayableDocument;
use crate::util::search::{escape_boolean_ft, escape_like};

/// Le plafond de lecture de [`find_group_candidates`] : au-delà, la liste est
/// un échantillon (les plus anciennes par date, puis identifiant) et
/// [`GroupCandidates::scan_limit_reached`] le dit.
pub const GROUP_CANDIDATES_SCAN_LIMIT: i64 = 500;

/// La longueur minimale d'un jeton que l'index `FULLTEXT` retient
/// (`innodb_ft_min_token_size`, 3 par défaut, `docs/search-patterns.md` § 3.1).
const FULLTEXT_MIN_TOKEN_CHARS: usize = 3;

/// La famille des factures cherchées : celle que règle le sens de la
/// transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateFamily {
    /// Crédit : factures clients.
    Client,
    /// Débit : factures fournisseur.
    Supplier,
}

/// Une facture candidate.
///
/// Les champs propres à une famille valent `None` pour l'autre :
/// `invoice_number` pour un client ; `internal_number`,
/// `supplier_invoice_number` et `payment_reference` pour un fournisseur.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct GroupCandidate {
    pub id: i64,
    /// Numéro de la facture client.
    pub invoice_number: Option<String>,
    /// Numéro interne de la facture fournisseur (Story 33-1b).
    pub internal_number: Option<String>,
    /// Numéro de la facture chez le fournisseur.
    pub supplier_invoice_number: Option<String>,
    /// Référence de paiement de la facture fournisseur.
    pub payment_reference: Option<String>,
    pub contact_name: String,
    /// `invoices.date` ou `supplier_invoices.invoice_date`.
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    /// Le TTC (arrondi figé compris pour un client) ; le TTC de la facture
    /// fournisseur pour l'autre famille.
    pub total_ttc: Decimal,
    /// Le reste dû **brut** (quatre décimales) ; pour une facture fournisseur,
    /// son TTC (le règlement fournisseur est binaire).
    pub amount_due: Decimal,
}

impl PayableDocument for GroupCandidate {
    fn document_id(&self) -> i64 {
        self.id
    }
    fn document_date(&self) -> NaiveDate {
        self.date
    }
    fn due_date(&self) -> Option<NaiveDate> {
        self.due_date
    }
}

/// Ce que demande [`find_group_candidates`].
#[derive(Debug, Clone, Copy)]
pub struct GroupCandidatesQuery<'a> {
    pub company_id: i64,
    pub family: CandidateFamily,
    /// La borne haute de la fenêtre de dates de la transaction
    /// (`InvoiceDateWindow::latest_invoice_date`, client ou fournisseur).
    pub latest_invoice_date: NaiveDate,
    /// Le terme de recherche, tel que reçu (taillé ici) ; vide ou absent :
    /// aucun filtre.
    pub q: Option<&'a str>,
    /// Les factures à retirer (déjà imputées par l'écran).
    pub exclude: &'a [i64],
}

/// Le résultat de [`find_group_candidates`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupCandidates {
    /// Au plus [`GROUP_CANDIDATES_SCAN_LIMIT`] factures, par date puis
    /// identifiant croissants.
    pub candidates: Vec<GroupCandidate>,
    /// Vrai si d'autres factures répondaient aux critères au-delà du plafond.
    pub scan_limit_reached: bool,
}

/// Le terme compte-t-il au moins un jeton que l'index `FULLTEXT` retient ?
/// Sinon la recherche du nom passe par `LIKE` (`docs/search-patterns.md`
/// § 3.1) — y compris quand le terme échappé est vide.
fn has_fulltext_token(escaped: &str) -> bool {
    escaped
        .split_whitespace()
        .any(|t| t.chars().count() >= FULLTEXT_MIN_TOKEN_CHARS)
}

/// Ajoute la disjonction de recherche, ` AND (nom OR numéros…)`, si le terme
/// n'est pas vide après taille.
fn push_search(qb: &mut QueryBuilder<'_, MySql>, family: CandidateFamily, q: Option<&str>) {
    let Some(trimmed) = q.map(str::trim).filter(|t| !t.is_empty()) else {
        return;
    };
    let escaped = escape_boolean_ft(trimmed);
    let like_pattern = format!("%{}%", escape_like(trimmed));
    if has_fulltext_token(&escaped) {
        qb.push(" AND (MATCH(c.name) AGAINST(");
        qb.push_bind(format!("{escaped}*"));
        qb.push(" IN BOOLEAN MODE)");
    } else {
        qb.push(" AND (c.name LIKE ");
        qb.push_bind(like_pattern.clone());
        qb.push(" ESCAPE '\\\\'");
    }
    let number_columns: &[&str] = match family {
        CandidateFamily::Client => &["i.invoice_number"],
        CandidateFamily::Supplier => &[
            "si.internal_number",
            "si.supplier_invoice_number",
            "si.payment_reference",
        ],
    };
    for column in number_columns {
        qb.push(format!(" OR COALESCE({column}, '') LIKE "));
        qb.push_bind(like_pattern.clone());
        qb.push(" ESCAPE '\\\\'");
    }
    qb.push(")");
}

/// Ajoute ` AND <colonne> NOT IN (…)` si `exclude` n'est pas vide.
fn push_exclude(qb: &mut QueryBuilder<'_, MySql>, column: &str, exclude: &[i64]) {
    if exclude.is_empty() {
        return;
    }
    qb.push(format!(" AND {column} NOT IN ("));
    let mut ids = qb.separated(", ");
    for id in exclude {
        ids.push_bind(*id);
    }
    qb.push(")");
}

/// Les factures candidates d'un règlement groupé (Story 43-2, AC5) — voir la
/// documentation du module pour les critères, la recherche et le plafond.
///
/// # Errors
///
/// `DbError` de la lecture.
pub async fn find_group_candidates<'e, E>(
    executor: E,
    query: GroupCandidatesQuery<'_>,
) -> Result<GroupCandidates, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    let mut qb: QueryBuilder<'_, MySql> = QueryBuilder::new("");
    match query.family {
        CandidateFamily::Client => {
            // Alias `i` exigé par les tables dérivées du reste dû.
            qb.push(format!(
                "SELECT i.id, i.invoice_number, CAST(NULL AS CHAR) AS internal_number, \
                 CAST(NULL AS CHAR) AS supplier_invoice_number, \
                 CAST(NULL AS CHAR) AS payment_reference, c.name AS contact_name, \
                 i.date AS date, i.due_date, {ttc} AS total_ttc, {due} AS amount_due \
                 FROM invoices i \
                 JOIN contacts c ON c.id = i.contact_id AND c.company_id = i.company_id \
                 {joins} \
                 WHERE i.company_id = ",
                ttc = INVOICE_TTC_DERIVED_SQL,
                due = INVOICE_AMOUNT_DUE_DERIVED_SQL,
                joins = amount_due_derived_joins(),
            ));
            qb.push_bind(query.company_id);
            qb.push(
                " AND i.status = 'validated' AND i.paid_at IS NULL \
                 AND i.journal_entry_id IS NOT NULL AND i.date <= ",
            );
            qb.push_bind(query.latest_invoice_date);
            push_exclude(&mut qb, "i.id", query.exclude);
            push_search(&mut qb, query.family, query.q);
            qb.push(" HAVING amount_due > 0 ORDER BY i.date ASC, i.id ASC LIMIT ");
        }
        CandidateFamily::Supplier => {
            qb.push(
                "SELECT si.id, CAST(NULL AS CHAR) AS invoice_number, si.internal_number, \
                 si.supplier_invoice_number, si.payment_reference, c.name AS contact_name, \
                 si.invoice_date AS date, si.due_date, si.total_amount AS total_ttc, \
                 si.total_amount AS amount_due \
                 FROM supplier_invoices si \
                 JOIN contacts c ON c.id = si.contact_id AND c.company_id = si.company_id \
                 WHERE si.company_id = ",
            );
            qb.push_bind(query.company_id);
            qb.push(
                " AND si.status = 'open' AND si.total_amount > 0 \
                 AND NOT EXISTS (SELECT 1 FROM payment_batch_items pbi \
                   JOIN payment_batches pb ON pb.id = pbi.payment_batch_id \
                   WHERE pbi.supplier_invoice_id = si.id AND pb.status = 'generated') \
                 AND si.invoice_date <= ",
            );
            qb.push_bind(query.latest_invoice_date);
            push_exclude(&mut qb, "si.id", query.exclude);
            push_search(&mut qb, query.family, query.q);
            qb.push(" ORDER BY si.invoice_date ASC, si.id ASC LIMIT ");
        }
    }
    // Une ligne de plus que le plafond : dit, sans compter, s'il a coupé.
    qb.push_bind(GROUP_CANDIDATES_SCAN_LIMIT + 1);
    let mut candidates = qb
        .build_query_as::<GroupCandidate>()
        .fetch_all(executor)
        .await
        .map_err(map_db_error)?;
    let scan_limit = usize::try_from(GROUP_CANDIDATES_SCAN_LIMIT).unwrap_or(usize::MAX);
    let scan_limit_reached = candidates.len() > scan_limit;
    candidates.truncate(scan_limit);
    Ok(GroupCandidates {
        candidates,
        scan_limit_reached,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_term_without_a_three_char_token_falls_back_to_like() {
        assert!(!has_fulltext_token(""));
        assert!(!has_fulltext_token("BL"));
        assert!(!has_fulltext_token("BL SA"));
        assert!(has_fulltext_token("BL Holding"));
        assert!(has_fulltext_token("Müh"));
    }
}
