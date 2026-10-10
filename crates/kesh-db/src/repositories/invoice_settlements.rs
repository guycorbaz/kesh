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
     settlement_type, settlement_bank_account_id, settlement_account_id, \
     write_off_nature, write_off_vat, created_at";

use crate::entities::{InvoiceSettlement, NewInvoiceSettlement, NewJournalEntryLine};
use crate::errors::{ClaimSide, DbError, SettlementAccountRole, map_db_error};
use crate::repositories::invoices::line_ttc_sql;

/// Forme **scalaire par facture** du total réglé — sous-requête corrélée.
/// **Prérequis : alias `i` sur `invoices`.**
///
/// Miroir exact de [`INVOICE_SETTLED_DERIVED_JOIN_SQL`], qui en est la forme
/// agrégée. Les deux doivent rester d'accord : c'est ce que vérifie le test de
/// parité (`tests/invoice_amount_due_parity.rs`).
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

/// Forme **scalaire par facture** de l'avoir émis, **TTC** — `0` s'il n'y en
/// a pas. **Prérequis : alias `i` sur `invoices`.**
///
/// ⛔ **TTC, arrondi ligne par ligne** — la grandeur que l'écriture de l'avoir
/// porte au crédit de la créance (`credit_notes::generate_credit_note_journal_lines` :
/// `total_ht + total_vat`, TVA par [`kesh_core::accounting::vat::line_vat_amount`]).
/// Le résiduel soustrait ce terme d'un TTC : il doit être dans la même unité.
/// **Jamais `credit_notes.total_amount`**, qui est **HT** (miroir de
/// `invoices.total_amount`) : le lire ici laissait la TVA d'une facture
/// créditée en reste dû (#455, Story 25-4-a).
///
/// ⚠️ **Seul un avoir `issued` compte.** Les statuts sont
/// `draft / issued / cancelled` (`chk_credit_notes_status`) : un brouillon n'a
/// pas d'écriture et n'éteint donc rien. `credit_notes.invoice_id` est
/// `NOT NULL UNIQUE` — au plus un avoir par facture, la somme est donc une
/// commodité de forme, pas un cumul réel.
///
/// ⛔ **Arrondi figé de l'avoir compris** (Story 25-4-c4-a, #494) : l'avoir
/// recopie `rounding_amount` de sa facture, si bien qu'un avoir total éteint
/// exactement le TTC arrondi. Le terme d'arrondi se somme sur `credit_notes`,
/// **hors** de la somme des lignes — qui le compterait une fois par ligne.
pub const INVOICE_CREDITED_SUBQUERY_SQL: &str = concat!(
    "((SELECT COALESCE(SUM(",
    line_ttc_sql!("cl."),
    "), 0) FROM credit_note_lines cl \
     INNER JOIN credit_notes cn ON cn.id = cl.credit_note_id \
     WHERE cn.invoice_id = i.id AND cn.status = 'issued') \
     + (SELECT COALESCE(SUM(cn.rounding_amount), 0) FROM credit_notes cn \
     WHERE cn.invoice_id = i.id AND cn.status = 'issued'))"
);

/// Forme **agrégat multi-factures** de l'avoir émis, **TTC** — table dérivée
/// à joindre (alias `cnt`), puis `COALESCE(cnt.credited, 0)` côté requête
/// externe. **Prérequis : alias `i` sur `invoices`.**
///
/// Miroir exact de [`INVOICE_CREDITED_SUBQUERY_SQL`] — même unité, même
/// arrondi ; les deux sont tenues d'accord par le test de parité
/// (`tests/invoice_amount_due_parity.rs`).
///
/// L'arrondi figé de l'avoir (Story 25-4-c4-a) s'ajoute **par avoir**, après la
/// somme de ses lignes (table dérivée `cl_ttc`), puis se somme par facture.
/// ⛔ `LEFT JOIN` + `COALESCE` : un avoir sans ligne garde son arrondi, comme dans
/// la forme scalaire — un `INNER JOIN` l'écarterait d'une seule des deux formes
/// (revue de code P1 de la 25-4-c4-a).
pub const INVOICE_CREDITED_DERIVED_JOIN_SQL: &str = concat!(
    "LEFT JOIN (SELECT cn.invoice_id, SUM(COALESCE(cl_ttc.ttc, 0) + cn.rounding_amount) AS credited \
     FROM credit_notes cn LEFT JOIN (SELECT cl.credit_note_id, SUM(",
    line_ttc_sql!("cl."),
    ") AS ttc FROM credit_note_lines cl GROUP BY cl.credit_note_id) cl_ttc \
     ON cl_ttc.credit_note_id = cn.id \
     WHERE cn.status = 'issued' GROUP BY cn.invoice_id) cnt ON cnt.invoice_id = i.id"
);

/// Les trois tables dérivées du **reste dû sous forme jointe** — TTC (`lt`),
/// avoir émis (`cnt`), réglé (`st`) — à placer après `FROM invoices i …`
/// (Story 25-4-b1, #416). **Prérequis : alias `i` sur `invoices`.**
///
/// ⛔ Forme des **listes et agrégats** : la forme scalaire
/// ([`amount_due`]) y serait réévaluée par ligne — un N+1 déguisé (24-2, D3).
/// Les deux formes sont tenues d'accord par `tests/invoice_amount_due_parity.rs`.
///
/// Une fonction et non une constante : les trois jointures sont elles-mêmes des
/// constantes, que `concat!` ne sait pas assembler.
pub fn amount_due_derived_joins() -> String {
    format!(
        "{} {INVOICE_CREDITED_DERIVED_JOIN_SQL} {INVOICE_SETTLED_DERIVED_JOIN_SQL}",
        crate::repositories::invoices::INVOICE_TTC_DERIVED_JOIN_SQL
    )
}

/// Le **reste dû** d'une ligne, sur les tables de
/// [`amount_due_derived_joins`] : `TTC − avoir émis − Σ règlements`, trois
/// termes TTC. Miroir exact de [`amount_due`]. ⛔ Ne pas le réécrire à la main
/// dans une requête : c'est ainsi que les agrégats ont sommé le TTC pendant
/// un mois après la 24-2 (#416).
pub const INVOICE_AMOUNT_DUE_DERIVED_SQL: &str = "(COALESCE(lt.ttc, 0) + i.rounding_amount - COALESCE(cnt.credited, 0) - COALESCE(st.settled, 0))";

/// Le **total réglé** d'une ligne, sur les tables de [`amount_due_derived_joins`].
pub const INVOICE_AMOUNT_SETTLED_DERIVED_SQL: &str = "COALESCE(st.settled, 0)";

// ---------------------------------------------------------------------------
// Story 25-4-c3-b (#476) — le reste dû au centime, et l'écart d'arrondi
// ---------------------------------------------------------------------------

/// Le reste dû **au centime** — la seule définition (AC 1).
///
/// `line_total` porte quatre décimales, la TVA seule est arrondie à deux : le
/// reste dû calculé ([`amount_due`], [`INVOICE_AMOUNT_DUE_DERIVED_SQL`]) peut
/// valoir 10.0050, alors que la QR, les rappels et le dialogue réclament 10.01.
/// Toute comparaison d'un paiement au reste dû, et tout affichage de ce reste,
/// passe par ici.
///
/// ⛔ Le reste dû **calculé** n'est jamais arrondi : seules les comparaisons et
/// l'affichage le sont. Et la stratégie n'est pas recopiée : c'est celle de
/// [`kesh_core::types::Money::round_to_centimes`] (`MidpointAwayFromZero`).
pub fn amount_due_to_centime(raw: Decimal) -> Decimal {
    kesh_core::types::Money::new(raw)
        .round_to_centimes()
        .amount()
}

/// Ce que devient un paiement face au reste dû **brut** (AC 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentAgainstDue {
    /// Règlement au montant payé, deux lignes : partiel, ou solde exact quand le
    /// paiement égale le reste brut.
    Ordinary,
    /// Le paiement égale le reste **arrondi**, qui diffère du brut : le règlement
    /// s'enregistre au **brut** — le reste dû tombe à zéro exactement — et
    /// `paid − raw_due` passe en troisième ligne, sur le compte de différences
    /// d'arrondi.
    SettlesWithRounding { raw_due: Decimal },
    /// Trop-perçu : refusé, jamais écrit.
    Overpayment,
}

/// Classe un paiement contre le reste dû brut (AC 3), dans cet ordre :
///
/// 1. `paid == arrondi` et `arrondi != brut` → [`PaymentAgainstDue::SettlesWithRounding`],
///    y compris quand l'arrondi est **inférieur** au brut (10.00 sur 10.004) ;
/// 2. `paid > brut` → [`PaymentAgainstDue::Overpayment`] ;
/// 3. sinon → [`PaymentAgainstDue::Ordinary`].
///
/// ⛔ **Les deux bornes.** Comparer à l'arrondi seul laissait un paiement
/// **strictement entre** le brut et l'arrondi (10.008 sur 10.0050) passer en
/// partiel : le reste dû devenait négatif, `paid_at` se posait sans écart, et la
/// créance restait créditrice (validation P1 de la story, F1).
///
/// Un reste brut nul ou négatif, ou inférieur au demi-centime (arrondi nul),
/// refuse tout paiement positif en trop-perçu — limite connue, #490.
pub fn classify_payment(paid: Decimal, raw_due: Decimal) -> PaymentAgainstDue {
    let rounded = amount_due_to_centime(raw_due);
    if paid == rounded && rounded != raw_due {
        PaymentAgainstDue::SettlesWithRounding { raw_due }
    } else if paid > raw_due {
        PaymentAgainstDue::Overpayment
    } else {
        PaymentAgainstDue::Ordinary
    }
}

/// Les lignes de l'écriture d'un règlement (AC 3) : débit de la contrepartie du
/// montant **payé**, crédit de la créance du montant **réglé**, et — s'ils
/// diffèrent — l'écart sur `rounding_account_id` : au **crédit** si le paiement
/// dépasse le réglé, au **débit** sinon. Équilibrée par construction, sans ligne
/// à zéro (`chk_jel_debit_credit_exclusive`).
///
/// Un écart sans compte d'arrondi est une erreur de l'appelant
/// ([`DbError::Invariant`]) : c'est à lui d'avoir obtenu le compte, ou refusé.
///
/// Si le compte d'arrondi est aussi la contrepartie (compte interne choisi au
/// règlement), l'écriture porte deux lignes sur le même compte : équilibrée,
/// comptablement exacte, laissée telle quelle (revue P1 de la 25-4-c3-b).
pub fn settlement_journal_lines(
    counterparty_account_id: i64,
    receivable_account_id: i64,
    paid: Decimal,
    settled: Decimal,
    rounding_account_id: Option<i64>,
) -> Result<Vec<NewJournalEntryLine>, DbError> {
    let mut lines = vec![
        NewJournalEntryLine {
            account_id: counterparty_account_id,
            debit: paid,
            credit: Decimal::ZERO,
            project_id: None,
        },
        NewJournalEntryLine {
            account_id: receivable_account_id,
            debit: Decimal::ZERO,
            credit: settled,
            project_id: None,
        },
    ];
    let gap = paid - settled;
    if !gap.is_zero() {
        let account_id = rounding_account_id.ok_or_else(|| {
            DbError::Invariant("écart d'arrondi sans compte de différences d'arrondi".into())
        })?;
        lines.push(NewJournalEntryLine {
            account_id,
            debit: if gap < Decimal::ZERO {
                -gap
            } else {
                Decimal::ZERO
            },
            credit: if gap > Decimal::ZERO {
                gap
            } else {
                Decimal::ZERO
            },
            project_id: None,
        });
    }
    Ok(lines)
}

/// Les lignes de l'écriture d'un **solde** (Story 25-4-d2a, #384) : débit du
/// compte de la nature `arrondi(amount) − Σ TVA`, débit de la TVA due par taux
/// (une ligne par part, toutes sur `vat_account_id`), crédit de la créance
/// `amount` — le reste **exact**, qui peut porter quatre décimales.
///
/// La **fraction de centime** (`amount − arrondi(amount)`, un demi-centime au
/// plus) va au compte de différences d'arrondi `rounding_account_id`, au débit
/// si elle est positive, au crédit sinon — même convention que le règlement au
/// centime ([`settlement_journal_lines`]) : un compte de charge ou de produit ne
/// reçoit pas de fraction de centime. Équilibrée par construction ; le reliquat
/// de centimes du prorata reste sur le compte de la nature.
///
/// ⛔ Garde `Σ TVA < amount` → [`DbError::Invariant`] : le débit du compte de la
/// nature doit rester strictement positif (`chk_jel_debit_credit_exclusive`).
/// Inatteignable par le prorata ([`kesh_core::accounting::vat::write_off_vat_shares`]),
/// elle protège la fonction de ce qu'on lui passe. Des parts sans compte de TVA
/// sont de même une erreur de l'appelant.
pub fn write_off_journal_lines(
    nature_account_id: i64,
    receivable_account_id: i64,
    vat_account_id: Option<i64>,
    rounding_account_id: Option<i64>,
    amount: Decimal,
    vat_shares: &[kesh_core::accounting::vat::VatRateShare],
) -> Result<Vec<NewJournalEntryLine>, DbError> {
    let total_vat: Decimal = vat_shares.iter().map(|s| s.vat_amount).sum();
    if total_vat >= amount {
        return Err(DbError::Invariant(format!(
            "solde : la TVA corrigée ({total_vat}) atteint le montant soldé ({amount})"
        )));
    }
    // La fraction de centime sort du compte de la nature — sauf quand celui-ci
    // EST le compte de différences d'arrondi (nature `rounding`) : elle y reste.
    let gap = amount - amount_due_to_centime(amount);
    let separate_gap = !gap.is_zero() && rounding_account_id != Some(nature_account_id);
    let nature_debit = if separate_gap {
        amount - gap - total_vat
    } else {
        amount - total_vat
    };
    // Un reste inférieur au demi-centime (0.0040) s'arrondit à zéro : il n'y a
    // rien à imputer à la nature, tout le reste est une fraction de centime et
    // va au compte d'arrondi (#490 — revue de code P2). Le cas « débit nul sans
    // écart séparé » est inatteignable — il vaudrait `total_vat == amount`,
    // déjà refusé plus haut — et gardé par défense (revue de code P3).
    if nature_debit < Decimal::ZERO || (nature_debit.is_zero() && !separate_gap) {
        return Err(DbError::Invariant(format!(
            "solde : le débit du compte de la nature ({nature_debit}) n'est pas positif"
        )));
    }
    let mut lines = Vec::new();
    if !nature_debit.is_zero() {
        lines.push(NewJournalEntryLine {
            account_id: nature_account_id,
            debit: nature_debit,
            credit: Decimal::ZERO,
            project_id: None,
        });
    }
    if separate_gap {
        let account_id = rounding_account_id.ok_or_else(|| {
            DbError::Invariant(
                "solde : fraction de centime sans compte de différences d'arrondi".into(),
            )
        })?;
        lines.push(NewJournalEntryLine {
            account_id,
            debit: if gap > Decimal::ZERO {
                gap
            } else {
                Decimal::ZERO
            },
            credit: if gap < Decimal::ZERO {
                -gap
            } else {
                Decimal::ZERO
            },
            project_id: None,
        });
    }
    if !vat_shares.is_empty() {
        let vat_account_id = vat_account_id.ok_or_else(|| {
            DbError::Invariant("solde : des parts de TVA sans compte de TVA due".into())
        })?;
        lines.extend(vat_shares.iter().map(|share| NewJournalEntryLine {
            account_id: vat_account_id,
            debit: share.vat_amount,
            credit: Decimal::ZERO,
            project_id: None,
        }));
    }
    lines.push(NewJournalEntryLine {
        account_id: receivable_account_id,
        debit: Decimal::ZERO,
        credit: amount,
        project_id: None,
    });
    Ok(lines)
}

/// La ventilation figée d'un solde, telle que persistée en `write_off_vat` :
/// `[{ratePercent, baseHt, vatAmount}]`, montants en chaînes décimales.
pub fn write_off_vat_json(
    shares: &[kesh_core::accounting::vat::VatRateShare],
) -> serde_json::Value {
    serde_json::Value::Array(
        shares
            .iter()
            .map(|s| {
                serde_json::json!({
                    "ratePercent": s.rate_percent.to_string(),
                    "baseHt": s.base_ht.to_string(),
                    "vatAmount": s.vat_amount.to_string(),
                })
            })
            .collect(),
    )
}

/// Relit la ventilation figée d'un solde (`write_off_vat`), telle que l'écrit
/// [`write_off_vat_json`] — la forme ne vit qu'ici (Story 25-4-d2c, #384).
///
/// Une forme fausse — pas un tableau, clé absente, valeur non chaîne, décimal
/// illisible — est une **erreur** (message décrivant le défaut), jamais un zéro :
/// la syntaxe JSON, elle, est garantie par la contrainte `json_valid` de la base.
pub fn parse_write_off_vat(
    value: &serde_json::Value,
) -> Result<Vec<kesh_core::accounting::vat::VatRateShare>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("write_off_vat n'est pas un tableau : {value}"))?;
    items
        .iter()
        .map(|item| {
            let field = |key: &str| -> Result<Decimal, String> {
                let raw = item.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
                    format!("write_off_vat : clé « {key} » absente ou non chaîne dans {item}")
                })?;
                raw.parse::<Decimal>()
                    .map_err(|e| format!("write_off_vat : « {key} » illisible ({raw}) : {e}"))
            };
            Ok(kesh_core::accounting::vat::VatRateShare {
                rate_percent: field("ratePercent")?,
                base_ht: field("baseHt")?,
                vat_amount: field("vatAmount")?,
            })
        })
        .collect()
}

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
    let (bank_account_id, account_id) = new.kind.counterparty_refs();
    // Story 25-4-d2a : nature et ventilation, ssi c'est un solde
    // (`chk_invoice_settlements_write_off_nature`).
    let (nature, vat) = match &new.kind {
        crate::entities::SettlementKind::WriteOff { nature, vat, .. } => {
            (Some(nature.as_str()), Some(vat.clone()))
        }
        crate::entities::SettlementKind::Choice(_) => (None, None),
    };
    let id: u64 = sqlx::query(
        "INSERT INTO invoice_settlements \
         (company_id, invoice_id, journal_entry_id, amount, settled_on, \
          settlement_type, settlement_bank_account_id, settlement_account_id, \
          write_off_nature, write_off_vat) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.invoice_id)
    .bind(new.journal_entry_id)
    .bind(new.amount)
    .bind(new.settled_on)
    .bind(new.kind.type_str())
    .bind(bank_account_id)
    .bind(account_id)
    .bind(nature)
    .bind(vat)
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
pub async fn list_for_invoice<'e, E>(
    executor: E,
    company_id: i64,
    invoice_id: i64,
) -> Result<Vec<InvoiceSettlement>, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    sqlx::query_as::<_, InvoiceSettlement>(&format!(
        "SELECT {COLUMNS} FROM invoice_settlements WHERE company_id = ? AND invoice_id = ? \
         ORDER BY settled_on ASC, id ASC"
    ))
    .bind(company_id)
    .bind(invoice_id)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// L'expression **scalaire par facture** du reste dû — `TTC − avoir émis −
/// Σ règlements`, trois sous-requêtes corrélées, **prérequis : alias `i` sur
/// `invoices`**. Celle d'[`amount_due`], écrite une fois ; la vue des postes
/// ouverts (Story 15-1b, AC8) l'emploie pour les factures d'une page, où les
/// tables dérivées de [`amount_due_derived_joins`] agrégeraient des tables
/// entières (`EXPLAIN` mesuré : `credit_note_lines` lue en entier). Miroir de
/// [`INVOICE_AMOUNT_DUE_DERIVED_SQL`], tenu d'accord par
/// `tests/invoice_amount_due_parity.rs` (par [`amount_due`]).
pub fn amount_due_scalar_sql() -> String {
    format!(
        "({} - {INVOICE_CREDITED_SUBQUERY_SQL} - {INVOICE_SETTLED_SUBQUERY_SQL})",
        crate::repositories::invoices::INVOICE_TTC_SUBQUERY_SQL
    )
}

/// Ce qui reste dû sur une facture : `TTC − avoir émis − Σ règlements`.
///
/// ⛔ **Trois termes TTC.** Le TTC de la facture et celui de l'avoir sont
/// arrondis ligne par ligne par la même formule (`line_ttc_sql!`) ; les
/// règlements sont des montants encaissés, donc TTC par nature.
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
    sqlx::query_scalar::<_, Decimal>(&format!(
        "SELECT {} FROM invoices i WHERE i.id = ?",
        amount_due_scalar_sql()
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

// ---------------------------------------------------------------------------
// Story 15-6a (#473, #523) — les comptes de la vente, lus sur son écriture
// ---------------------------------------------------------------------------

/// Le compte de **créance** d'une facture, lu sur son **écriture de vente** —
/// jamais sur les réglages (Story 15-6a, #473).
///
/// ⛔ **La créance se lit sur l'écriture de vente, jamais sur les réglages.**
/// Les réglages disent quel compte la *prochaine* facture débitera ; une
/// facture déjà validée a débité le compte que portait son écriture, et tout
/// geste qui la solde — règlement, solde du reste, encaissement par
/// rapprochement, avoir — doit créditer **ce** compte-là, pour qu'il revienne
/// à zéro quoi qu'il soit arrivé à la configuration entre-temps.
///
/// La créance est la **première** ligne au débit de l'écriture — d'où
/// `ORDER BY jel.id LIMIT 1`. Pas la **seule** : un arrondi à 5 centimes
/// **négatif** (Story 25-4-c4-a) ajoute une ligne de débit, toujours **après**
/// elle (`invoices::generate_invoice_journal_lines_rounded`). L'ordre est celui
/// de `jel.id`, qui suit l'ordre d'insertion des lignes ; une écriture
/// enregistrée ne se modifie plus (gardes de `journal_entries::update` et
/// `delete_in_tx` sur une écriture possédée par une pièce).
///
/// Portée par `je.company_id` (anti-IDOR) : une écriture d'une autre société
/// rend `None`, comme une écriture sans ligne de débit. Chaque appelant garde
/// **son** refus pour `None` (`DbError::Invariant` pour le règlement, le solde
/// du reste et l'avoir ; `INVOICE_SALE_ENTRY_MALFORMED` en `failed[]` pour le
/// rapprochement). Ne prend aucun verrou : les lignes d'une écriture de vente
/// sont gelées.
pub async fn sale_receivable_account(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    sale_entry_id: i64,
) -> Result<Option<i64>, DbError> {
    sqlx::query_scalar(
        "SELECT jel.account_id FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? AND jel.debit > 0 \
         ORDER BY jel.id LIMIT 1",
    )
    .bind(sale_entry_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)
}

/// Le compte d'**arrondi à 5 centimes** d'une facture, lu sur la **dernière**
/// ligne de son écriture de vente et **recoupé** avec l'arrondi figé sur la
/// facture (Story 15-6a, #523).
///
/// La validation pousse l'écart d'arrondi en **dernière** ligne
/// (`invoices::generate_invoice_journal_lines_rounded`) : au **crédit** si
/// l'arrondi est positif, au **débit** s'il est négatif. L'avoir le contre-passe
/// sur **ce** compte, non sur celui que les réglages désignent au moment de
/// l'avoir — sinon l'ancien compte garderait l'écart et le nouveau prendrait
/// l'opposé.
///
/// **Pourquoi le recoupement** : un lecteur positionnel qui ne vérifierait rien
/// rendrait en silence n'importe quelle ligne (une écriture retouchée, une
/// ligne ajoutée après l'arrondi). La dernière ligne doit donc porter
/// exactement l'arrondi : `rounding_amount > 0` ⇒ `credit = rounding_amount`
/// et `debit = 0` ; `rounding_amount < 0` ⇒ `debit = −rounding_amount` et
/// `credit = 0`. Toute autre forme — pas de ligne, sens ou montant différent —
/// est un [`DbError::Invariant`].
///
/// **Pourquoi pas `usable_designated_account`** : celle-ci contrôle un compte
/// **désigné** (type charge ou produit, actif, imputable). La doctrine de la
/// contre-passation est « mêmes comptes que l'origine, seule l'inactivité
/// bloque » (6 ter de la Story 16-1a) : l'état actif du compte rendu ici est
/// contrôlé par l'appelant, sous verrou.
///
/// À n'appeler que si `rounding_amount ≠ 0` : une facture émise sans arrondi
/// n'a pas de ligne d'arrondi. Portée par `je.company_id`. Aucun verrou.
pub async fn sale_rounding_account(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    sale_entry_id: i64,
    rounding_amount: Decimal,
) -> Result<i64, DbError> {
    let last: Option<(i64, Decimal, Decimal)> = sqlx::query_as(
        "SELECT jel.account_id, jel.debit, jel.credit FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? \
         ORDER BY jel.id DESC LIMIT 1",
    )
    .bind(sale_entry_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let matches = |debit: Decimal, credit: Decimal| {
        if rounding_amount > Decimal::ZERO {
            credit == rounding_amount && debit.is_zero()
        } else if rounding_amount < Decimal::ZERO {
            debit == -rounding_amount && credit.is_zero()
        } else {
            false
        }
    };
    match last {
        Some((account_id, debit, credit)) if matches(debit, credit) => Ok(account_id),
        _ => Err(DbError::Invariant(format!(
            "écriture de vente {sale_entry_id} : la dernière ligne n'est pas l'arrondi de la \
             facture ({rounding_amount})"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Story 15-6b (#474) — un règlement ne vise pas le compte qu'il solde
// ---------------------------------------------------------------------------

/// Constat **pur** qu'un compte coïncide avec le compte soldé — rendu par
/// [`ensure_not_claim_account`], consommé par [`claim_account_refusal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimAccountClash {
    /// Le compte en cause (égal au compte soldé).
    pub account_id: i64,
}

/// Le **compte d'écart** désigné dans les réglages qu'un geste écrit en face de
/// la créance (Story 15-6b, AC3 bis). Sous-ensemble de
/// [`SettlementAccountRole`] : un compte désigné est toujours comparé à une
/// **créance** client.
///
/// ⚠️ Nommé `GapAccountRole`, et non `DesignatedRole` comme l'écrivait la fiche
/// (signature indicative ; choix C-15-6b-2) : un `DesignatedRole` existe déjà
/// dans `company_invoice_settings` (Story 15-5d), qui désigne les quatre champs
/// que la validation et la saisie fournisseur écrivent — un autre ensemble.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapAccountRole {
    /// `default_rounding_account_id`, quel que soit le geste.
    Rounding,
    /// Le compte d'une nature `discount`, `bank_fees` ou `bad_debt`.
    WriteOffNature,
    /// `default_vat_payable_account_id` (solde du reste).
    VatPayable,
}

/// Le **sujet** d'une comparaison au compte soldé (Story 15-6b, AC1 ; finding
/// F5 de la P5) : la combinaison « dette fournisseur + compte désigné » est
/// **irreprésentable** — le règlement fournisseur n'écrit aucun compte d'écart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimSubject {
    /// La contrepartie choisie, ou celle que porte le compte bancaire du geste.
    Counterparty(ClaimSide),
    /// Un compte désigné dans les réglages — comparé à la créance client.
    Designated(GapAccountRole),
}

impl ClaimSubject {
    /// Le côté soldé : celui de la contrepartie, ou la créance pour un compte
    /// désigné.
    pub fn claim(self) -> ClaimSide {
        match self {
            Self::Counterparty(side) => side,
            Self::Designated(_) => ClaimSide::Receivable,
        }
    }

    /// Le rôle exposé (`details.role`), qui porte le remède.
    pub fn role(self) -> SettlementAccountRole {
        match self {
            Self::Counterparty(_) => SettlementAccountRole::Counterparty,
            Self::Designated(GapAccountRole::Rounding) => SettlementAccountRole::Rounding,
            Self::Designated(GapAccountRole::WriteOffNature) => {
                SettlementAccountRole::WriteOffNature
            }
            Self::Designated(GapAccountRole::VatPayable) => SettlementAccountRole::VatPayable,
        }
    }
}

/// Compare un compte que le geste écrit **en face** du compte soldé à ce compte
/// soldé (Story 15-6b, #474). **Pure** : ni I/O ni transaction — c'est la seule
/// partie que la 15-6d emprunte (sa garde compare la contrepartie au compte de
/// banque).
///
/// ⛔ Une écriture `D X / C X` s'équilibre : rien, au grand livre, ne la refuse.
/// Seule cette comparaison d'**identifiants** la ferme — le contrôle de type
/// d'un compte désigné ne suffit pas, le type d'un compte qui porte des
/// écritures se changeant avec confirmation.
pub fn ensure_not_claim_account(
    account_id: i64,
    claim_account_id: i64,
) -> Result<(), ClaimAccountClash> {
    if account_id == claim_account_id {
        Err(ClaimAccountClash { account_id })
    } else {
        Ok(())
    }
}

/// Construit le refus [`DbError::SettlementCounterpartyIsClaimAccount`] —
/// **seule** construction de la variante hors de l'interception de
/// `payment_batches::confirm_batch` (qui la reconstruit avec son lot).
///
/// Appelée **à l'échec seulement** : elle lit le numéro du compte dans la
/// transaction de l'appelant (`accounts::number_in_company`, partagé avec les
/// refus de configuration de la Story 15-6c). Une erreur SQL de cette lecture est rendue
/// telle quelle — c'est alors elle, et non le refus, que l'appelant propage.
/// `batch` vaut toujours `None` ici.
pub async fn claim_account_refusal(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    clash: ClaimAccountClash,
    subject: ClaimSubject,
) -> DbError {
    match super::accounts::number_in_company(conn, company_id, clash.account_id).await {
        Ok(account_number) => DbError::SettlementCounterpartyIsClaimAccount {
            account_id: clash.account_id,
            account_number,
            claim: subject.claim(),
            role: subject.role(),
            batch: None,
        },
        Err(e) => e,
    }
}

/// Garde complète d'un site : [`ensure_not_claim_account`], puis, sur
/// coïncidence **seulement**, [`claim_account_refusal`] (Story 15-6b ; choix
/// C-15-6b-1). Rend `Err` avec le refus — ou avec l'erreur SQL de la lecture du
/// numéro.
pub async fn refuse_if_claim_account(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    account_id: i64,
    claim_account_id: i64,
    subject: ClaimSubject,
) -> Result<(), DbError> {
    match ensure_not_claim_account(account_id, claim_account_id) {
        Ok(()) => Ok(()),
        Err(clash) => Err(claim_account_refusal(conn, company_id, clash, subject).await),
    }
}

/// Les `details` du refus, construits **une fois** pour ses trois
/// consommateurs — le mapping HTTP, le `FailedProposal` du rapprochement et le
/// `PaymentBatchFailedItem` de la création d'un lot — pour que leurs clés ne
/// divergent pas.
///
/// `bankAccountId` n'est posé que s'il est fourni (rôle `counterparty` d'un
/// geste qui porte un compte bancaire). Les clés **de lot**
/// (`paymentBatchId`, `supplierInvoiceId`) sont hors de ce helper : le mapping
/// HTTP, seul consommateur qui en ait, les ajoute.
///
/// ⚠️ **Ce qui est construit une fois, ce sont les CLÉS, pas l'extraction** :
/// les consommateurs du rapprochement
/// (`routes/reconciliation.rs::claim_account_failed_proposal`) et de la
/// création d'un lot (`payment_batches.rs::validate_invoice_for_batch`)
/// déstructurent chacun la variante `SettlementCounterpartyIsClaimAccount`
/// par leur propre `match` avant d'appeler ce helper. Une méthode sur
/// `DbError` qui rende directement les `details` supprimerait ces deux `match`
/// — dette écrite au Change Log de la 15-6b (revue de code P1, finding B-6).
pub fn claim_account_refusal_details(
    account_id: i64,
    account_number: Option<&str>,
    claim: ClaimSide,
    role: SettlementAccountRole,
    bank_account_id: Option<i64>,
) -> serde_json::Value {
    let mut details = serde_json::json!({
        "accountId": account_id,
        "accountNumber": account_number,
        "claim": claim.as_str(),
        "role": role.as_str(),
    });
    if let Some(bank_account_id) = bank_account_id {
        details["bankAccountId"] = serde_json::json!(bank_account_id);
    }
    details
}

// ---------------------------------------------------------------------------
// Story 25-5-a (#386) — lecture exhaustive pour l'export de souveraineté
// ---------------------------------------------------------------------------

/// Tous les règlements de factures d'une société (Story 25-5-a, #386).
///
/// ⛔ **Non bornée** : un export de souveraineté qui tronque ment sur ce qu'il
/// contient. ⚠️ Cette table est née à l'**Epic 24** — c'est-à-dire APRÈS
/// l'écriture de l'issue #386, qui ne la mentionne donc pas. *L'export se
/// périmait à chaque epic ; c'est ce que la garde d'exhaustivité ferme.*
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<InvoiceSettlement>, DbError> {
    sqlx::query_as::<_, InvoiceSettlement>(&format!(
        "SELECT {COLUMNS} FROM invoice_settlements WHERE company_id = ? ORDER BY id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn le_reste_au_centime_arrondit_le_demi_centime_loin_de_zero() {
        assert_eq!(amount_due_to_centime(dec!(10.0050)), dec!(10.01));
        assert_eq!(amount_due_to_centime(dec!(10.004)), dec!(10.00));
        assert_eq!(amount_due_to_centime(dec!(-10.0050)), dec!(-10.01));
        assert_eq!(amount_due_to_centime(dec!(68.1000)), dec!(68.10));
    }

    #[test]
    fn classification_a_double_borne() {
        use PaymentAgainstDue::*;
        // Solde avec écart, arrondi au-dessus puis au-dessous du brut.
        assert_eq!(
            classify_payment(dec!(10.01), dec!(10.0050)),
            SettlesWithRounding {
                raw_due: dec!(10.0050)
            }
        );
        assert_eq!(
            classify_payment(dec!(10.00), dec!(10.004)),
            SettlesWithRounding {
                raw_due: dec!(10.004)
            }
        );
        // ⛔ Entre le brut et l'arrondi : trop-perçu, jamais un partiel.
        assert_eq!(classify_payment(dec!(10.008), dec!(10.0050)), Overpayment);
        assert_eq!(classify_payment(dec!(10.02), dec!(10.0050)), Overpayment);
        // Ordinaire : partiel, solde exact, reste déjà au centime.
        assert_eq!(classify_payment(dec!(5.00), dec!(10.0050)), Ordinary);
        assert_eq!(classify_payment(dec!(10.00), dec!(10.0000)), Ordinary);
        assert_eq!(classify_payment(dec!(10.01), dec!(10.00)), Overpayment);
        // Reste nul, négatif ou sous le demi-centime (#490) : tout est trop-perçu.
        assert_eq!(classify_payment(dec!(0.01), dec!(0)), Overpayment);
        assert_eq!(classify_payment(dec!(0.01), dec!(-0.004)), Overpayment);
        assert_eq!(classify_payment(dec!(0.01), dec!(0.004)), Overpayment);
    }

    #[test]
    fn lignes_ecart_au_credit_ou_au_debit() {
        let l = settlement_journal_lines(1, 2, dec!(10.01), dec!(10.0050), Some(3)).unwrap();
        assert_eq!(l.len(), 3);
        assert_eq!((l[0].account_id, l[0].debit), (1, dec!(10.01)));
        assert_eq!((l[1].account_id, l[1].credit), (2, dec!(10.0050)));
        assert_eq!(
            (l[2].account_id, l[2].debit, l[2].credit),
            (3, dec!(0), dec!(0.0050))
        );

        let l = settlement_journal_lines(1, 2, dec!(10.00), dec!(10.004), Some(3)).unwrap();
        assert_eq!(
            (l[2].account_id, l[2].debit, l[2].credit),
            (3, dec!(0.004), dec!(0))
        );
        let debit: Decimal = l.iter().map(|x| x.debit).sum();
        let credit: Decimal = l.iter().map(|x| x.credit).sum();
        assert_eq!(debit, credit, "écriture équilibrée");

        let l = settlement_journal_lines(1, 2, dec!(5), dec!(5), None).unwrap();
        assert_eq!(
            l.len(),
            2,
            "sans écart, deux lignes et aucun compte d'arrondi requis"
        );

        assert!(matches!(
            settlement_journal_lines(1, 2, dec!(10.01), dec!(10.0050), None),
            Err(DbError::Invariant(_))
        ));
    }

    /// Story 25-4-d2c — la lecture relit exactement ce que l'écriture produit.
    #[test]
    fn la_ventilation_d_un_solde_se_relit_telle_qu_ecrite() {
        use kesh_core::accounting::vat::VatRateShare;
        let shares = vec![
            VatRateShare {
                rate_percent: dec!(8.10),
                base_ht: dec!(123.45),
                vat_amount: dec!(10.00),
            },
            VatRateShare {
                rate_percent: dec!(2.60),
                base_ht: dec!(50.0000),
                vat_amount: dec!(1.30),
            },
        ];
        assert_eq!(
            parse_write_off_vat(&write_off_vat_json(&shares)),
            Ok(shares)
        );
        assert_eq!(parse_write_off_vat(&serde_json::json!([])), Ok(vec![]));
    }

    /// Story 25-4-d2c — chaque forme fausse est une erreur, jamais un zéro.
    #[test]
    fn une_ventilation_de_forme_fausse_est_une_erreur() {
        let ok = serde_json::json!({"ratePercent": "8.1", "baseHt": "1", "vatAmount": "0.08"});
        assert!(parse_write_off_vat(&serde_json::json!([ok])).is_ok());
        for bad in [
            serde_json::json!({"items": []}),
            serde_json::json!(null),
            serde_json::json!([{"baseHt": "1", "vatAmount": "0.08"}]),
            serde_json::json!([{"ratePercent": 8.1, "baseHt": "1", "vatAmount": "0.08"}]),
            serde_json::json!([{"ratePercent": "8.1", "baseHt": "1", "vatAmount": "abc"}]),
            serde_json::json!(["8.1"]),
        ] {
            assert!(parse_write_off_vat(&bad).is_err(), "accepté : {bad}");
        }
    }

    /// Story 15-6b (#474) — la comparaison est une égalité d'identifiants, et
    /// rien d'autre.
    #[test]
    fn la_comparaison_au_compte_solde_est_une_egalite_d_identifiants() {
        assert_eq!(ensure_not_claim_account(7, 8), Ok(()));
        assert_eq!(
            ensure_not_claim_account(8, 8),
            Err(ClaimAccountClash { account_id: 8 })
        );
    }

    /// Story 15-6b — le sujet rend le côté et le rôle ; un compte désigné est
    /// toujours comparé à la créance.
    #[test]
    fn le_sujet_rend_le_cote_et_le_role() {
        let cases = [
            (
                ClaimSubject::Counterparty(ClaimSide::Receivable),
                ClaimSide::Receivable,
                SettlementAccountRole::Counterparty,
            ),
            (
                ClaimSubject::Counterparty(ClaimSide::Payable),
                ClaimSide::Payable,
                SettlementAccountRole::Counterparty,
            ),
            (
                ClaimSubject::Designated(GapAccountRole::Rounding),
                ClaimSide::Receivable,
                SettlementAccountRole::Rounding,
            ),
            (
                ClaimSubject::Designated(GapAccountRole::WriteOffNature),
                ClaimSide::Receivable,
                SettlementAccountRole::WriteOffNature,
            ),
            (
                ClaimSubject::Designated(GapAccountRole::VatPayable),
                ClaimSide::Receivable,
                SettlementAccountRole::VatPayable,
            ),
        ];
        for (subject, claim, role) in cases {
            assert_eq!(subject.claim(), claim, "{subject:?}");
            assert_eq!(subject.role(), role, "{subject:?}");
        }
    }

    /// Story 15-6b — forme des `details`, avec et sans compte bancaire.
    #[test]
    fn les_details_du_refus_ont_une_seule_forme() {
        assert_eq!(
            claim_account_refusal_details(
                3,
                Some("1100"),
                ClaimSide::Receivable,
                SettlementAccountRole::Counterparty,
                Some(9),
            ),
            serde_json::json!({
                "accountId": 3,
                "accountNumber": "1100",
                "claim": "receivable",
                "role": "counterparty",
                "bankAccountId": 9,
            })
        );
        let sans_banque = claim_account_refusal_details(
            3,
            None,
            ClaimSide::Receivable,
            SettlementAccountRole::Rounding,
            None,
        );
        assert_eq!(
            sans_banque,
            serde_json::json!({
                "accountId": 3,
                "accountNumber": null,
                "claim": "receivable",
                "role": "rounding",
            })
        );
        assert!(sans_banque.get("bankAccountId").is_none());
    }
}
