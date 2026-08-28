//! Règlements d'une facture client — Story 24-2 (#371).
//!
//! ⚠️ **Le scoping multi-tenant passe par `company_id`, porté par la table
//! elle-même** (patron `invoice_reminders`, l'autre enfant récent de
//! `invoices`). Toute lecture le filtre explicitement : ne jamais se reposer sur
//! la seule jointure vers `invoices`.
//!
//! ⚠️ **Aucune fonction ici ne dit si une facture est soldée.** Le résiduel se
//! calcule depuis la comptabilité — cf. [`INVOICE_SETTLED_SUBQUERY_SQL`] — et
//! `paid_at` en est la **projection**, posée par l'appelant quand le solde
//! atteint zéro.

use rust_decimal::Decimal;
use sqlx::MySqlPool;

/// Colonnes de la table, dans l'ordre attendu par `FromRow`. Centralisées :
/// une colonne ajoutée sans passer par ici casse la compilation, pas le runtime.
const COLUMNS: &str = "id, company_id, invoice_id, journal_entry_id, amount, settled_on, \
     settlement_type, settlement_bank_account_id, settlement_account_id, created_at";

use crate::entities::{InvoiceSettlement, NewInvoiceSettlement};
use crate::errors::{DbError, map_db_error};

/// Forme **scalaire par facture** du total réglé — sous-requête corrélée.
/// **Prérequis : alias `i` sur `invoices`.**
///
/// Miroir exact de [`INVOICE_SETTLED_DERIVED_JOIN_SQL`], qui en est la forme
/// agrégée. Les deux doivent rester d'accord : c'est ce que vérifie le test de
/// parité.
///
/// ⚠️ Même discipline que `INVOICE_TTC_SUBQUERY_SQL` (`invoices.rs`) : la forme
/// corrélée sert **une** facture, jamais une liste — elle y serait ré-évaluée
/// par ligne, c'est-à-dire un N+1 déguisé en SQL.
pub const INVOICE_SETTLED_SUBQUERY_SQL: &str =
    "(SELECT COALESCE(SUM(s.amount), 0) FROM invoice_settlements s WHERE s.invoice_id = i.id)";

/// Forme **agrégat multi-factures** du total réglé — table dérivée à joindre
/// (alias `st`), puis `COALESCE(st.settled, 0)` côté requête externe.
/// **Prérequis : alias `i` sur `invoices`.**
pub const INVOICE_SETTLED_DERIVED_JOIN_SQL: &str = "LEFT JOIN (SELECT invoice_id, SUM(amount) AS settled \
     FROM invoice_settlements GROUP BY invoice_id) st ON st.invoice_id = i.id";

/// Forme **scalaire par facture** de l'avoir émis — `0` s'il n'y en a pas.
/// **Prérequis : alias `i` sur `invoices`.**
///
/// ⚠️ **Seul un avoir `issued` compte.** Les statuts sont
/// `draft / issued / cancelled` (`chk_credit_notes_status`) : un brouillon n'a
/// pas d'écriture et n'éteint donc rien. `credit_notes.invoice_id` est
/// `NOT NULL UNIQUE` — au plus un avoir par facture, la somme est donc une
/// commodité de forme, pas un cumul réel.
pub const INVOICE_CREDITED_SUBQUERY_SQL: &str = "(SELECT COALESCE(SUM(cn.total_amount), 0) FROM credit_notes cn \
     WHERE cn.invoice_id = i.id AND cn.status = 'issued')";

/// Forme **agrégat multi-factures** de l'avoir émis — alias `cnt`.
/// **Prérequis : alias `i` sur `invoices`.**
pub const INVOICE_CREDITED_DERIVED_JOIN_SQL: &str = "LEFT JOIN (SELECT invoice_id, SUM(total_amount) AS credited \
     FROM credit_notes WHERE status = 'issued' GROUP BY invoice_id) cnt \
     ON cnt.invoice_id = i.id";

/// Enregistre un règlement dans la transaction courante.
///
/// ⚠️ **Ne pose PAS `paid_at`** : c'est à l'appelant de le faire, et seulement
/// si le résiduel atteint zéro. Séparer les deux gestes est délibéré — le
/// premier est un fait comptable, le second une conclusion qu'on en tire.
pub async fn create_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    new: NewInvoiceSettlement,
) -> Result<InvoiceSettlement, DbError> {
    // ⚠️ Le mode et sa contrepartie sortent du MÊME `SettlementChoice` — les
    // dissocier rouvrirait la possibilité d'un `bank_transfer` sans compte
    // bancaire, que `chk_invoice_settlements_counterparty` refuse de toute
    // façon, mais en 500 plutôt qu'en erreur métier.
    let (bank_account_id, account_id) = new.choice.counterparty_refs();
    let id: u64 = sqlx::query(
        "INSERT INTO invoice_settlements \
         (company_id, invoice_id, journal_entry_id, amount, settled_on, \
          settlement_type, settlement_bank_account_id, settlement_account_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.invoice_id)
    .bind(new.journal_entry_id)
    .bind(new.amount)
    .bind(new.settled_on)
    .bind(new.choice.type_str())
    .bind(bank_account_id)
    .bind(account_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .last_insert_id();

    sqlx::query_as::<_, InvoiceSettlement>(&format!(
        "SELECT {COLUMNS} FROM invoice_settlements WHERE id = ?"
    ))
    .bind(id as i64)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Les règlements d'une facture, du plus ancien au plus récent.
pub async fn list_for_invoice(
    pool: &MySqlPool,
    company_id: i64,
    invoice_id: i64,
) -> Result<Vec<InvoiceSettlement>, DbError> {
    sqlx::query_as::<_, InvoiceSettlement>(&format!(
        "SELECT {COLUMNS} FROM invoice_settlements WHERE company_id = ? AND invoice_id = ? \
         ORDER BY settled_on ASC, id ASC"
    ))
    .bind(company_id)
    .bind(invoice_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Ce qui reste dû sur une facture : `TTC − avoir émis − Σ règlements`.
///
/// ⚠️ **Calculé, jamais stocké.** C'est la seule source de vérité du « combien
/// reste-t-il ? », et elle est adossée aux mêmes données que le grand livre.
///
/// Peut être **négatif** si un trop-perçu a été enregistré — ce que
/// l'encaissement refuse d'écrire, mais qu'un import ou une correction manuelle
/// pourrait produire. L'appelant qui affiche ce montant doit donc le traiter
/// comme une anomalie visible, pas l'écrêter à zéro : masquer un solde
/// créditeur, c'est reproduire le défaut que cette vague corrige.
pub async fn amount_due<'e, E>(executor: E, invoice_id: i64) -> Result<Decimal, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    let ttc = crate::repositories::invoices::INVOICE_TTC_SUBQUERY_SQL;
    sqlx::query_scalar::<_, Decimal>(&format!(
        "SELECT {ttc} - {INVOICE_CREDITED_SUBQUERY_SQL} - {INVOICE_SETTLED_SUBQUERY_SQL} \
         FROM invoices i WHERE i.id = ?"
    ))
    .bind(invoice_id)
    .fetch_one(executor)
    .await
    .map_err(map_db_error)
}

/// Total déjà réglé sur une facture — `0` s'il n'y a aucun règlement.
///
/// Distinct d'[`amount_due`] à dessein : l'un dit ce qui est entré, l'autre ce
/// qui manque. Les afficher tous les deux évite au lecteur de faire la
/// soustraction de tête, et rend visible le cas « avoir sans encaissement ».
pub async fn amount_settled<'e, E>(executor: E, invoice_id: i64) -> Result<Decimal, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    sqlx::query_scalar::<_, Decimal>(&format!(
        "SELECT {INVOICE_SETTLED_SUBQUERY_SQL} FROM invoices i WHERE i.id = ?"
    ))
    .bind(invoice_id)
    .fetch_one(executor)
    .await
    .map_err(map_db_error)
}
