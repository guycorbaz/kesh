//! Traduction des réponses de l'API Stripe en lignes du rapport — Story 40-8a
//! (AC-3).
//!
//! Un versement automatique payé (`GET /v1/payouts`) et ses transactions de
//! solde (`GET /v1/balance_transactions?payout=…`) deviennent des
//! [`StripePayoutRow`] — la forme que rend le lecteur du CSV « Itemized payout
//! reconciliation » (Story 40-7a) —, si bien que l'import, les doublons et le
//! rapprochement (40-7b, 40-7c) s'appliquent sans changement.
//!
//! Règles, chacune tenue par un test :
//!
//! - **jours de Zurich** et non UTC ([`kesh_core::horloge::jour_suisse`]) :
//!   `payout_date` est le jour de l'`arrival_date` du versement, `date` celui de
//!   `created`, `available_on` celui de `available_on` ; un instant du
//!   31 décembre à 23:30 UTC est le 1er janvier à Zurich ;
//! - `created` (texte) est l'heure de Zurich `AAAA-MM-JJ HH:MM:SS`, la forme du
//!   rapport du tableau de bord (constatée identique sur 336 transactions
//!   réelles de 2024) ;
//! - montants lus **en entiers** (plus petites unités, jamais `f64`) puis
//!   placés à l'exposant de la devise ([`exponent`]) ; une devise à trois
//!   décimales est refusée comme le lecteur CSV la refuse (erreur de ligne
//!   `AMOUNT`) ;
//! - `net = amount − fee` (erreur `NET_MISMATCH`), un versement n'a qu'une
//!   devise (`PAYOUT_CURRENCY_MIXED`), identifiants au contrôle du lecteur CSV
//!   ([`kesh_import::stripe::valid_id`]) ;
//! - la transaction du versement lui-même (`type = payout`, `source` = le
//!   versement), que `?payout=` rend, **n'est pas une ligne** ;
//! - **Σ net des lignes = `amount` de l'objet versement**, sinon le
//!   téléchargement échoue en `PROVIDER_RESPONSE_INVALID` (`payoutId`,
//!   `expected`, `found`) et rien n'est écrit : une liste incomplète
//!   écrirait un versement faux.
//!
//! Fonctions pures, sans I/O.

use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};
use kesh_core::dates::annee_stockable;
use kesh_core::horloge::{heure_suisse, jour_suisse};
use kesh_import::stripe::valid_id;
use kesh_import::{MAX_CSV_LINE_ERRORS, StripeLineError, StripeLineErrorCode, StripePayoutRow};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::errors::AppError;

use super::PROVIDER;

/// Devises à **zéro** décimale de Stripe, en minuscules — liste de
/// `docs.stripe.com/currencies`, reprise le 2026-11-22 (amendement de la
/// fiche, point 4). ISK et UGX n'y sont **pas** : la même page dit que leurs
/// montants s'écrivent, pour la compatibilité, en centièmes dont la partie
/// décimale vaut toujours `00` ; ils se lisent donc à deux décimales, comme
/// HUF et TWD.
pub const ZERO_DECIMAL_CURRENCIES: [&str; 15] = [
    "bif", "clp", "djf", "gnf", "jpy", "kmf", "krw", "mga", "pyg", "rwf", "vnd", "vuv", "xaf",
    "xof", "xpf",
];

/// Devises à **trois** décimales de Stripe, en minuscules (même page, même
/// date) : refusées, comme le lecteur du CSV les refuse (Story 40-7a, AC-5).
pub const THREE_DECIMAL_CURRENCIES: [&str; 5] = ["bhd", "jod", "kwd", "omr", "tnd"];

/// Champs de l'API nommés par les erreurs de ligne.
pub const FIELD_ID: &str = "id";
/// Devise d'une transaction.
pub const FIELD_CURRENCY: &str = "currency";
/// Montant brut.
pub const FIELD_AMOUNT: &str = "amount";
/// Net.
pub const FIELD_NET: &str = "net";
/// Instant de création.
pub const FIELD_CREATED: &str = "created";
/// Instant de disponibilité.
pub const FIELD_AVAILABLE_ON: &str = "available_on";
/// Catégorie de rapport.
pub const FIELD_CATEGORY: &str = "reporting_category";
/// Identifiant du versement.
pub const FIELD_PAYOUT_ID: &str = "payout.id";
/// Devise du versement.
pub const FIELD_PAYOUT_CURRENCY: &str = "payout.currency";
/// Montant du versement.
pub const FIELD_PAYOUT_AMOUNT: &str = "payout.amount";
/// Arrivée du versement.
pub const FIELD_PAYOUT_ARRIVAL: &str = "payout.arrival_date";

/// Type de la transaction de solde du versement lui-même.
pub const PAYOUT_TRANSACTION_TYPE: &str = "payout";

/// Un versement, tel que `GET /v1/payouts` le rend (champs lus seulement).
/// Les montants sont des **entiers** en plus petites unités : un nombre à
/// virgule ou démesuré rend la réponse illisible, jamais arrondie.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ApiPayout {
    /// Identifiant (`po_…`).
    pub id: String,
    /// Montant versé, en plus petites unités.
    pub amount: i64,
    /// Devise, en minuscules.
    pub currency: String,
    /// Arrivée prévue ou constatée à la banque (secondes d'époque).
    pub arrival_date: i64,
    /// Versement automatique (`true`) ou manuel.
    pub automatic: bool,
    /// Statut (`paid`, `pending`, `in_transit`, `canceled`, `failed`).
    pub status: String,
}

/// Une transaction de solde, telle que `GET /v1/balance_transactions` la
/// rend (champs lus seulement). `source` non développé : un identifiant.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ApiBalanceTransaction {
    /// Identifiant (`txn_…`).
    pub id: String,
    /// Brut, en plus petites unités.
    pub amount: i64,
    /// Frais (positifs quand Stripe prélève).
    pub fee: i64,
    /// Net = brut − frais.
    pub net: i64,
    /// Devise, en minuscules.
    pub currency: String,
    /// Création (secondes d'époque).
    pub created: i64,
    /// Disponibilité des fonds (secondes d'époque).
    pub available_on: Option<i64>,
    /// Catégorie de rapport (`charge`, `refund`, `fee`, `payout_reversal`…).
    pub reporting_category: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Identifiant de l'objet lié (`ch_…`, `re_…`, `po_…`).
    pub source: Option<String>,
    /// Type (`charge`, `refund`, `stripe_fee`, `payout`…).
    #[serde(rename = "type")]
    pub kind: String,
}

/// L'exposant d'une devise (minuscules) : 0, 2, ou `None` pour une devise à
/// trois décimales (refusée).
pub fn exponent(currency: &str) -> Option<u32> {
    if ZERO_DECIMAL_CURRENCIES.contains(&currency) {
        Some(0)
    } else if THREE_DECIMAL_CURRENCIES.contains(&currency) {
        None
    } else {
        Some(2)
    }
}

/// La transaction de solde du versement lui-même (`type = payout` **et**
/// `source` = le versement), que `?payout=` rend et qui n'est pas une ligne.
pub fn is_own_payout_transaction(t: &ApiBalanceTransaction, payout: &ApiPayout) -> bool {
    t.kind == PAYOUT_TRANSACTION_TYPE && t.source.as_deref() == Some(payout.id.as_str())
}

/// Un montant en plus petites unités, en unités de la devise.
fn units(minor: i64, exp: u32) -> Decimal {
    Decimal::new(minor, exp)
}

/// L'instant d'un nombre de secondes d'époque, `None` hors de la plage de la
/// base (année 1000 à 9999 à Zurich).
fn instant(secs: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(secs, 0).filter(|t| annee_stockable(jour_suisse(*t)))
}

/// Devise valide : trois lettres ASCII minuscules ou majuscules.
fn valid_currency(c: &str) -> bool {
    c.len() == 3 && c.bytes().all(|b| b.is_ascii_alphabetic())
}

/// Texte rogné, vide = absent (comme une cellule du CSV).
fn trimmed(v: &Option<String>) -> Option<String> {
    v.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Refus des lignes d'un versement.
#[derive(Debug, Clone, PartialEq)]
pub enum RowsError {
    /// Erreurs de ligne (forme du lecteur CSV), au plus
    /// [`MAX_CSV_LINE_ERRORS`].
    Lines(Vec<StripeLineError>),
    /// Σ net des lignes ≠ montant du versement.
    PayoutMismatch {
        /// Le versement.
        payout_id: String,
        /// Montant de l'objet versement.
        expected: Decimal,
        /// Σ net des lignes.
        found: Decimal,
    },
}

/// Collecte plafonnée des erreurs de ligne.
#[derive(Default)]
struct Errors(Vec<StripeLineError>);

impl Errors {
    fn push(&mut self, line: usize, field: &'static str, code: StripeLineErrorCode, value: &str) {
        if self.0.len() < MAX_CSV_LINE_ERRORS {
            let mut v: String = value
                .chars()
                .take(kesh_import::MAX_CSV_LINE_ERROR_VALUE_CHARS)
                .collect();
            if v.len() < value.len() {
                v.push('…');
            }
            self.0.push(StripeLineError {
                line,
                field: Some(field),
                code,
                value: Some(v),
            });
        }
    }
}

/// Les lignes d'**un** versement, numérotées à partir de `first_line` — voir
/// le module pour les règles. La transaction du versement lui-même est
/// écartée ; les autres sont rangées par (`created`, `id`).
pub fn payout_rows(
    payout: &ApiPayout,
    transactions: &[ApiBalanceTransaction],
    first_line: usize,
) -> Result<Vec<StripePayoutRow>, RowsError> {
    let mut errors = Errors::default();

    // Le versement : identifiant, devise, arrivée. Ses erreurs se nomment à la
    // première ligne du versement.
    if !valid_id(&payout.id) {
        errors.push(
            first_line,
            FIELD_PAYOUT_ID,
            StripeLineErrorCode::Id,
            &payout.id,
        );
    }
    let payout_currency = payout.currency.to_ascii_lowercase();
    let payout_exp = if valid_currency(&payout_currency) {
        let e = exponent(&payout_currency);
        if e.is_none() {
            errors.push(
                first_line,
                FIELD_PAYOUT_AMOUNT,
                StripeLineErrorCode::Amount,
                &payout.currency,
            );
        }
        e
    } else {
        errors.push(
            first_line,
            FIELD_PAYOUT_CURRENCY,
            StripeLineErrorCode::Currency,
            &payout.currency,
        );
        None
    };
    let payout_date = instant(payout.arrival_date).map(jour_suisse);
    if payout_date.is_none() {
        errors.push(
            first_line,
            FIELD_PAYOUT_ARRIVAL,
            StripeLineErrorCode::Date,
            &payout.arrival_date.to_string(),
        );
    }

    let mut kept: Vec<&ApiBalanceTransaction> = transactions
        .iter()
        .filter(|t| !is_own_payout_transaction(t, payout))
        .collect();
    kept.sort_by(|a, b| (a.created, &a.id).cmp(&(b.created, &b.id)));

    let mut rows = Vec::with_capacity(kept.len());
    let mut sum_net: i128 = 0;
    for (i, t) in kept.iter().enumerate() {
        let line = first_line + i;
        let before = errors.0.len();
        if !valid_id(&t.id) {
            errors.push(line, FIELD_ID, StripeLineErrorCode::Id, &t.id);
        }
        let currency = t.currency.to_ascii_lowercase();
        let exp = if !valid_currency(&currency) {
            errors.push(
                line,
                FIELD_CURRENCY,
                StripeLineErrorCode::Currency,
                &t.currency,
            );
            None
        } else if currency != payout_currency {
            errors.push(
                line,
                FIELD_CURRENCY,
                StripeLineErrorCode::PayoutCurrencyMixed,
                &t.currency,
            );
            None
        } else {
            payout_exp
        };
        if t.amount.checked_sub(t.fee) != Some(t.net) {
            errors.push(
                line,
                FIELD_NET,
                StripeLineErrorCode::NetMismatch,
                &t.net.to_string(),
            );
        }
        let created = instant(t.created);
        if created.is_none() {
            errors.push(
                line,
                FIELD_CREATED,
                StripeLineErrorCode::Date,
                &t.created.to_string(),
            );
        }
        let available_on = match t.available_on {
            None => Some(None),
            Some(s) => match instant(s) {
                Some(a) => Some(Some(jour_suisse(a))),
                None => {
                    errors.push(
                        line,
                        FIELD_AVAILABLE_ON,
                        StripeLineErrorCode::Date,
                        &s.to_string(),
                    );
                    None
                }
            },
        };
        let category = trimmed(&t.reporting_category).map(|c| c.to_lowercase());
        if category.is_none() {
            errors.push(line, FIELD_CATEGORY, StripeLineErrorCode::Id, "");
        }
        sum_net += i128::from(t.net);
        if errors.0.len() > before {
            continue;
        }
        let (Some(exp), Some(created), Some(available_on), Some(category), Some(payout_date)) =
            (exp, created, available_on, category, payout_date)
        else {
            // Une erreur du versement lui-même (déjà notée) : pas de ligne.
            continue;
        };
        rows.push(StripePayoutRow {
            line,
            payout_id: payout.id.clone(),
            payout_date,
            balance_transaction_id: t.id.clone(),
            date: jour_suisse(created),
            created: heure_suisse(created)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            available_on,
            currency: currency.to_ascii_uppercase(),
            gross: units(t.amount, exp),
            fee: units(t.fee, exp),
            net: units(t.net, exp),
            category,
            description: trimmed(&t.description),
            source_id: trimmed(&t.source),
            charge_id: None,
        });
    }

    if !errors.0.is_empty() {
        errors.0.sort_by_key(|e| e.line);
        return Err(RowsError::Lines(errors.0));
    }
    // Sans erreur, la devise du versement a un exposant.
    let exp = payout_exp.unwrap_or(2);
    if sum_net != i128::from(payout.amount) {
        return Err(RowsError::PayoutMismatch {
            payout_id: payout.id.clone(),
            expected: units(payout.amount, exp),
            found: i64::try_from(sum_net).map_or(Decimal::MAX, |n| units(n, exp)),
        });
    }
    Ok(rows)
}

/// Le premier numéro de ligne : 2, comme la première ligne d'un fichier sous
/// son en-tête (le lecteur CSV numérote ainsi).
pub const FIRST_LINE: usize = 2;

/// Les lignes de plusieurs versements, rangés par (`arrival_date`, `id`),
/// numérotées à la suite à partir de [`FIRST_LINE`]. Refus, **rien n'étant
/// rendu** : erreurs de ligne → `PROVIDER_RESPONSE_INVALID` `details.lines`
/// (au plus [`MAX_CSV_LINE_ERRORS`]) ; Σ net ≠ montant d'un versement →
/// `details: { payoutId, expected, found }` ; une transaction de solde rendue
/// par deux versements → `details: { reason: "duplicate_transaction",
/// transactionId }`.
pub fn rows_of(
    payouts: &[(ApiPayout, Vec<ApiBalanceTransaction>)],
) -> Result<Vec<StripePayoutRow>, AppError> {
    let mut ordered: Vec<&(ApiPayout, Vec<ApiBalanceTransaction>)> = payouts.iter().collect();
    ordered.sort_by(|a, b| (a.0.arrival_date, &a.0.id).cmp(&(b.0.arrival_date, &b.0.id)));
    let mut rows = Vec::new();
    let mut lines: Vec<StripeLineError> = Vec::new();
    let mut next = FIRST_LINE;
    for (payout, txs) in ordered {
        let count = txs
            .iter()
            .filter(|t| !is_own_payout_transaction(t, payout))
            .count()
            .max(1);
        match payout_rows(payout, txs, next) {
            Ok(r) => rows.extend(r),
            Err(RowsError::Lines(e)) => {
                lines.extend(e);
            }
            Err(RowsError::PayoutMismatch {
                payout_id,
                expected,
                found,
            }) => {
                if lines.is_empty() {
                    return Err(AppError::ProviderResponseInvalid {
                        provider: PROVIDER,
                        details: serde_json::json!({
                            "payoutId": payout_id,
                            "expected": expected.to_string(),
                            "found": found.to_string(),
                        }),
                    });
                }
            }
        }
        next += count;
    }
    if !lines.is_empty() {
        lines.truncate(MAX_CSV_LINE_ERRORS);
        return Err(AppError::ProviderResponseInvalid {
            provider: PROVIDER,
            details: serde_json::json!({ "lines": lines }),
        });
    }
    let mut seen: HashSet<&str> = HashSet::new();
    for r in &rows {
        if !seen.insert(&r.balance_transaction_id) {
            return Err(AppError::ProviderResponseInvalid {
                provider: PROVIDER,
                details: serde_json::json!({
                    "reason": "duplicate_transaction",
                    "transactionId": r.balance_transaction_id,
                }),
            });
        }
    }
    Ok(rows)
}

/// Le jour de Zurich d'un nombre de secondes d'époque (tests et appelants).
pub fn zurich_day(secs: i64) -> Option<NaiveDate> {
    instant(secs).map(jour_suisse)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;

    fn secs(y: i32, m: u32, d: u32, h: u32, mi: u32) -> i64 {
        Utc.with_ymd_and_hms(y, m, d, h, mi, 0).unwrap().timestamp()
    }

    fn payout(amount: i64, currency: &str) -> ApiPayout {
        ApiPayout {
            id: "po_TEST0001".into(),
            amount,
            currency: currency.into(),
            arrival_date: secs(2024, 3, 4, 23, 0),
            automatic: true,
            status: "paid".into(),
        }
    }

    fn tx(id: &str, amount: i64, fee: i64, currency: &str, created: i64) -> ApiBalanceTransaction {
        ApiBalanceTransaction {
            id: id.into(),
            amount,
            fee,
            net: amount - fee,
            currency: currency.into(),
            created,
            available_on: Some(created),
            reporting_category: Some("charge".into()),
            description: Some("  Vente fictive ".into()),
            source: Some("ch_TEST0001".into()),
            kind: "charge".into(),
        }
    }

    fn the_payout_tx(amount: i64) -> ApiBalanceTransaction {
        ApiBalanceTransaction {
            kind: PAYOUT_TRANSACTION_TYPE.into(),
            source: Some("po_TEST0001".into()),
            reporting_category: Some("payout".into()),
            ..tx("txn_TESTPO", -amount, 0, "usd", secs(2024, 3, 2, 10, 0))
        }
    }

    fn lines_of(e: RowsError) -> Vec<StripeLineError> {
        match e {
            RowsError::Lines(l) => l,
            other => panic!("erreurs de ligne attendues : {other:?}"),
        }
    }

    #[test]
    fn deux_decimales_et_transaction_du_versement_ecartee() {
        let txs = [
            the_payout_tx(10061),
            tx("txn_TEST0001", 10430, 369, "usd", secs(2024, 3, 1, 9, 0)),
        ];
        let rows = payout_rows(&payout(10061, "usd"), &txs, FIRST_LINE).unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.line, 2);
        assert_eq!(r.currency, "USD");
        assert_eq!(
            (r.gross, r.fee, r.net),
            (dec!(104.30), dec!(3.69), dec!(100.61))
        );
        assert_eq!(r.category, "charge");
        assert_eq!(r.description.as_deref(), Some("Vente fictive"));
        assert_eq!(r.source_id.as_deref(), Some("ch_TEST0001"));
        assert_eq!(r.charge_id, None);
        // Arrivée le 4 mars à 23:00 UTC : le 5 mars à Zurich.
        assert_eq!(r.payout_date, NaiveDate::from_ymd_opt(2024, 3, 5).unwrap());
        assert_eq!(r.created, "2024-03-01 10:00:00");
    }

    /// La transaction d'un **autre** versement, même de type `payout`, n'est
    /// pas écartée : elle reste une ligne (et fausse la somme, refusée).
    #[test]
    fn seule_la_transaction_du_versement_lui_meme_est_ecartee() {
        let p = payout(10061, "usd");
        assert!(is_own_payout_transaction(&the_payout_tx(10061), &p));
        let mut autre = the_payout_tx(10061);
        autre.source = Some("po_TEST0009".into());
        assert!(!is_own_payout_transaction(&autre, &p));
        let mut charge = the_payout_tx(10061);
        charge.kind = "charge".into();
        assert!(!is_own_payout_transaction(&charge, &p));
    }

    #[test]
    fn zero_decimale_et_trois_decimales_refusees() {
        let rows = payout_rows(
            &payout(970, "jpy"),
            &[tx("txn_TEST0001", 1000, 30, "jpy", secs(2024, 3, 1, 9, 0))],
            FIRST_LINE,
        )
        .unwrap();
        assert_eq!((rows[0].gross, rows[0].fee), (dec!(1000), dec!(30)));
        // ISK : deux décimales (centièmes à `00`), amendement point 4.
        let rows = payout_rows(
            &payout(50000, "isk"),
            &[tx("txn_TEST0001", 50000, 0, "isk", secs(2024, 3, 1, 9, 0))],
            FIRST_LINE,
        )
        .unwrap();
        assert_eq!(rows[0].gross, dec!(500));
        let e = lines_of(
            payout_rows(
                &payout(1000, "kwd"),
                &[tx("txn_TEST0001", 1000, 0, "kwd", secs(2024, 3, 1, 9, 0))],
                FIRST_LINE,
            )
            .unwrap_err(),
        );
        assert_eq!(e[0].code, StripeLineErrorCode::Amount);
        assert_eq!(e[0].field, Some(FIELD_PAYOUT_AMOUNT));
    }

    #[test]
    fn remboursement_a_frais_negatifs() {
        let mut r = tx("txn_TEST0003", -2000, -60, "chf", secs(2024, 3, 1, 9, 0));
        r.kind = "refund".into();
        r.reporting_category = Some("Refund".into());
        let rows = payout_rows(&payout(-1940, "chf"), &[r], FIRST_LINE).unwrap();
        assert_eq!(
            (rows[0].gross, rows[0].fee, rows[0].net),
            (dec!(-20.00), dec!(-0.60), dec!(-19.40))
        );
        assert_eq!(rows[0].category, "refund");
    }

    #[test]
    fn trente_et_un_decembre_utc_est_le_premier_janvier_a_zurich() {
        let created = secs(2024, 12, 31, 23, 30);
        let mut p = payout(100, "chf");
        // Versement créé fin décembre, arrivé le 1er janvier à Zurich.
        p.arrival_date = secs(2024, 12, 31, 23, 15);
        let rows = payout_rows(&p, &[tx("txn_TEST0001", 100, 0, "chf", created)], 2).unwrap();
        let jan1 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_eq!(rows[0].date, jan1);
        assert_eq!(rows[0].available_on, Some(jan1));
        assert_eq!(rows[0].payout_date, jan1);
        assert_eq!(rows[0].created, "2025-01-01 00:30:00");
        // Et à 22:59 UTC (23:59 à Zurich en hiver), c'est encore le 31.
        assert_eq!(
            zurich_day(secs(2024, 12, 31, 22, 59)),
            NaiveDate::from_ymd_opt(2024, 12, 31)
        );
    }

    #[test]
    fn somme_des_nets_differente_du_versement() {
        let e = payout_rows(
            &payout(10000, "usd"),
            &[tx(
                "txn_TEST0001",
                10430,
                369,
                "usd",
                secs(2024, 3, 1, 9, 0),
            )],
            FIRST_LINE,
        )
        .unwrap_err();
        assert_eq!(
            e,
            RowsError::PayoutMismatch {
                payout_id: "po_TEST0001".into(),
                expected: dec!(100.00),
                found: dec!(100.61),
            }
        );
        match rows_of(&[(
            payout(10000, "usd"),
            vec![tx(
                "txn_TEST0001",
                10430,
                369,
                "usd",
                secs(2024, 3, 1, 9, 0),
            )],
        )]) {
            Err(AppError::ProviderResponseInvalid { details, .. }) => {
                assert_eq!(details["payoutId"], "po_TEST0001");
                assert_eq!(details["expected"], "100.00");
                assert_eq!(details["found"], "100.61");
            }
            other => panic!("refus attendu : {other:?}"),
        }
    }

    #[test]
    fn controles_de_ligne() {
        let mut bad_net = tx("txn_TEST0001", 100, 3, "usd", secs(2024, 3, 1, 9, 0));
        bad_net.net = 98;
        let mixed = tx("txn_TEST0002", 100, 0, "eur", secs(2024, 3, 1, 10, 0));
        let bad_id = tx("txn:TEST", 100, 0, "usd", secs(2024, 3, 1, 11, 0));
        let mut no_cat = tx("txn_TEST0004", 100, 0, "usd", secs(2024, 3, 1, 12, 0));
        no_cat.reporting_category = Some(" ".into());
        let mut bad_date = tx("txn_TEST0005", 100, 0, "usd", 999_999_999_999);
        bad_date.available_on = None;
        let e = lines_of(
            payout_rows(
                &payout(0, "usd"),
                &[bad_net, mixed, bad_id, no_cat, bad_date],
                FIRST_LINE,
            )
            .unwrap_err(),
        );
        let codes: Vec<_> = e.iter().map(|l| (l.line, l.code)).collect();
        assert!(
            codes.contains(&(2, StripeLineErrorCode::NetMismatch)),
            "{codes:?}"
        );
        assert!(codes.contains(&(3, StripeLineErrorCode::PayoutCurrencyMixed)));
        assert!(codes.contains(&(4, StripeLineErrorCode::Id)));
        assert!(codes.contains(&(5, StripeLineErrorCode::Id)));
        assert!(codes.contains(&(6, StripeLineErrorCode::Date)), "{codes:?}");
    }

    #[test]
    fn versements_ranges_par_arrivee_et_lignes_a_la_suite() {
        let mut p2 = payout(100, "chf");
        p2.id = "po_TEST0002".into();
        p2.arrival_date = secs(2024, 3, 10, 0, 0);
        let p1 = payout(300, "chf");
        let rows = rows_of(&[
            (
                p2,
                vec![tx("txn_TEST0009", 100, 0, "chf", secs(2024, 3, 8, 9, 0))],
            ),
            (
                p1,
                vec![
                    tx("txn_TEST0002", 200, 0, "chf", secs(2024, 3, 2, 9, 0)),
                    tx("txn_TEST0001", 100, 0, "chf", secs(2024, 3, 1, 9, 0)),
                ],
            ),
        ])
        .unwrap();
        let ids: Vec<_> = rows
            .iter()
            .map(|r| (r.line, r.balance_transaction_id.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                (2, "txn_TEST0001"),
                (3, "txn_TEST0002"),
                (4, "txn_TEST0009")
            ]
        );
    }

    #[test]
    fn transaction_rendue_par_deux_versements() {
        let mut p2 = payout(100, "chf");
        p2.id = "po_TEST0002".into();
        let t = tx("txn_TEST0001", 100, 0, "chf", secs(2024, 3, 1, 9, 0));
        match rows_of(&[(payout(100, "chf"), vec![t.clone()]), (p2, vec![t])]) {
            Err(AppError::ProviderResponseInvalid { details, .. }) => {
                assert_eq!(details["reason"], "duplicate_transaction");
            }
            other => panic!("refus attendu : {other:?}"),
        }
    }

    #[test]
    fn montant_non_entier_illisible() {
        let ok: Result<ApiBalanceTransaction, _> = serde_json::from_str(
            r#"{"id":"txn_A","amount":10,"fee":0,"net":10,"currency":"chf","created":1,"type":"charge"}"#,
        );
        assert!(ok.is_ok());
        for bad in ["10.5", "1e3", "99999999999999999999"] {
            let json = format!(
                r#"{{"id":"txn_A","amount":{bad},"fee":0,"net":10,"currency":"chf","created":1,"type":"charge"}}"#
            );
            assert!(
                serde_json::from_str::<ApiBalanceTransaction>(&json).is_err(),
                "{bad}"
            );
        }
    }
}
