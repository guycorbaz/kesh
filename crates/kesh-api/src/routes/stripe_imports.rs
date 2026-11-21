//! Route `POST /api/v1/stripe-imports` — Story 40-7b (Epic 40, phase 1).
//!
//! Importe d'un geste le rapport Stripe « Itemized payout reconciliation » d'un
//! **compte Stripe** (multipart : `file`, `account`), **ventilé par devise** :
//! chaque devise du fichier va au compte bancaire Kesh, solde de ce compte
//! Stripe dans cette devise (`bank_accounts.provider = 'stripe'`,
//! `provider_account` = le libellé, devise = celle du compte du plan lié).
//!
//! Comptable+, clés d'API `read-write` admises, comme `POST
//! /api/v1/bank-imports`. Taille maximale : celle de l'import bancaire. La
//! route n'écrit **aucune écriture** : la revue et l'acceptation sont d'autres
//! routes (Story 40-7c).
//!
//! **Ordre des refus**, rien d'écrit à aucun : `400 VALIDATION_ERROR` (champ
//! manquant) → `404 PROVIDER_ACCOUNT_NOT_FOUND` → `422 STRIPE_CSV_INVALID` →
//! `422 PROVIDER_CURRENCY_ACCOUNT_MISSING` → `409 PROVIDER_CURRENCY_DUPLICATE`
//! → `409 BANK_ACCOUNT_CLOSED` → `422 BANK_IMPORT_DUPLICATE_FILE` — les quatre
//! derniers rendus par le cœur commun
//! ([`crate::routes::provider_imports`], Story 40-7b AC-7, partagé avec
//! l'import PayPal : verrous, empreinte, doublons, rejeu sur interblocage,
//! audit).
//!
//! **Ventilation** ([`lines_of_currency`], fonction pure, aucune ligne
//! écartée) : chaque ligne du rapport donne une transaction pour son **brut**
//! (si non nul) et une pour ses **frais** (si non nuls, de signe opposé au
//! champ `fee`) ; chaque **versement** (`automatic_payout_id`) donne une
//! transaction de `− Σ net` de ses lignes **dans le fichier**, quand cette somme
//! n'est pas nulle. Aucun contrôle ne voit un fichier **filtré** par catégorie
//! dans Stripe (le total du versement est calculé sur les mêmes lignes) : le
//! seul garde-fou est la réception du versement, qui ne retrouve pas son
//! montant (écart `payout_without_receipt`, Story 40-7c) ; le manuel dit de ne
//! pas filtrer le rapport.
//!
//! **Continuité** : aucune règle. Le rapport est découpé par versement et ne
//! porte aucun solde (les entrées `bank_imports` ont des soldes `NULL`).
//! **Doublons** : une transaction du même compte bancaire porte déjà
//! l'identifiant (brut, `:FEE` ou versement) → ignorée ; un rapport par devise
//! puis le rapport toutes devises de la même période s'importent sans rien
//! doubler.

use std::collections::BTreeMap;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use rust_decimal::Decimal;
use serde::Serialize;

use kesh_db::entities::bank_account::PROVIDER_STRIPE;
use kesh_import::stripe::{
    COL_BALANCE_TXN_ID, COL_PAYOUT_ID, LINKED_OBJECT_COLUMNS, StripeCsvError, StripeLineError,
    StripeLineErrorCode, StripePayoutRow, parse_stripe_payout_csv,
};
use kesh_import::{MAX_CSV_LINE_ERROR_VALUE_CHARS, MAX_CSV_LINE_ERRORS};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::provider_imports::{
    self, CurrencyReport, DETAILS_MAX_CHARS, FEE_SUFFIX, ImportRequest, ImportSource,
    NAME_MAX_CHARS, PROVIDER_TYPE_MAX_CHARS, ProviderGroup, ProviderLine, TXID_COLUMN_MAX_CHARS,
    import_lines, load_balances, truncate_chars,
};

/// Longueur maximale d'un identifiant (transaction de solde, versement) admis
/// à l'import : l'identifiant des frais (`<id>:FEE`) doit tenir dans la colonne
/// `transaction_id` de 255. Au-delà, la ligne est refusée en `422
/// STRIPE_CSV_INVALID` (erreur de ligne `ID`), et non par une erreur de la base
/// (500) ; un identifiant Stripe réel compte moins de 40 caractères.
pub const STRIPE_ID_MAX_CHARS: usize = TXID_COLUMN_MAX_CHARS - FEE_SUFFIX.len();

/// Catégorie de la transaction d'un versement (`bank_transactions.provider_type`),
/// celle que la revue Stripe classe en versement (Story 40-7c).
pub const PAYOUT_PROVIDER_TYPE: &str = kesh_reconciliation::stripe::PAYOUT_CATEGORY;

/// Début du libellé d'une transaction de frais.
const FEE_DETAILS_PREFIX: &str = "Frais Stripe · ";
/// Début du libellé d'une transaction de versement.
const PAYOUT_DETAILS_PREFIX: &str = "Versement Stripe";
/// Catégorie dont le libellé dit « Encaissement » (les autres disent leur
/// catégorie) : celle que la revue Stripe classe en encaissement (40-7c).
const CHARGE_CATEGORY: &str = kesh_reconciliation::stripe::CHARGE_CATEGORY;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// Un versement du fichier, dans une devise : sa somme nette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayoutReport {
    pub payout_id: String,
    pub currency: String,
    /// Nombre de lignes du fichier dans ce versement.
    pub lines: usize,
    /// Somme des nets de ces lignes.
    pub net: Decimal,
}

/// Réponse `201` de l'import (AC-6).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StripeImportResponse {
    pub account: String,
    pub currencies: Vec<CurrencyReport>,
    pub payouts: Vec<PayoutReport>,
}

// ---------------------------------------------------------------------------
// Ventilation (pure)
// ---------------------------------------------------------------------------

/// Les lignes du fichier rangées par devise (ordre du fichier) — aucune ligne
/// n'est écartée.
pub fn ventilate(rows: &[StripePayoutRow]) -> BTreeMap<String, Vec<&StripePayoutRow>> {
    let mut by_currency: BTreeMap<String, Vec<&StripePayoutRow>> = BTreeMap::new();
    for row in rows {
        by_currency
            .entry(row.currency.clone())
            .or_default()
            .push(row);
    }
    by_currency
}

/// Refuse les lignes dont un identifiant dépasse [`STRIPE_ID_MAX_CHARS`] :
/// erreurs de ligne `ID`, au plus [`MAX_CSV_LINE_ERRORS`], valeur tronquée comme
/// celles du lecteur. Le lecteur (40-7a) ne borne pas la longueur.
pub fn check_id_lengths(rows: &[StripePayoutRow]) -> Result<(), StripeCsvError> {
    let truncated = |v: &str| {
        let mut value = truncate_chars(v, MAX_CSV_LINE_ERROR_VALUE_CHARS);
        if value.len() < v.len() {
            value.push('…');
        }
        value
    };
    let mut errors = Vec::new();
    for row in rows {
        for (field, value) in [
            (COL_BALANCE_TXN_ID, row.balance_transaction_id.as_str()),
            (COL_PAYOUT_ID, row.payout_id.as_str()),
        ] {
            if value.chars().count() > STRIPE_ID_MAX_CHARS {
                errors.push(StripeLineError {
                    line: row.line,
                    field: Some(field),
                    code: StripeLineErrorCode::Id,
                    value: Some(truncated(value)),
                });
            }
        }
        if errors.len() >= MAX_CSV_LINE_ERRORS {
            errors.truncate(MAX_CSV_LINE_ERRORS);
            break;
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(StripeCsvError::Lines(errors))
    }
}

/// Le **libellé** d'une ligne du rapport — amendement de la 40-7b du 2026-11-20
/// (demande du Project Lead, format de son tableur Crésus) :
///
/// `<Encaissement si catégorie = charge, sinon la catégorie> Stripe de <brut>
/// <DEVISE> -<frais>=<net> <DEVISE> BRUT - <balance_transaction_id> (<created>-
/// <description>) - <versement>`
///
/// Les montants sont ceux **lus** (formes `Decimal` du lecteur, devise en
/// majuscules), `<created>` la cellule de création telle qu'écrite, la
/// description vide si absente. Le signe de `-<frais>` est littéral : des frais
/// rendus (`-0.60`) s'écrivent `--0.60`. Les identifiants `txn_…` et `po_…`
/// restent cherchables dans le texte (rapprochement, dépannage ; repris par le
/// libellé des écritures, Story 40-7c).
pub fn row_label(row: &StripePayoutRow) -> String {
    label(row, 0)
}

/// Le libellé de [`row_label`], la **description bornée** pour que le libellé
/// entier — précédé de `prefix_chars` caractères (le préfixe des frais) — tienne
/// dans [`DETAILS_MAX_CHARS`] : la fin `) - <versement>`, que le rapprochement
/// et le dépannage cherchent, n'est jamais coupée (relevé L1 de la revue P1 de
/// la 40-7b). Seule la description cède ; des identifiants de 251 caractères
/// chacun (la borne de `check_id_lengths`) pourraient encore dépasser, ce qui
/// n'arrive pas avec des identifiants Stripe réels (moins de 40).
fn label(row: &StripePayoutRow, prefix_chars: usize) -> String {
    let compose = |description: &str| {
        let kind = if row.category == CHARGE_CATEGORY {
            "Encaissement"
        } else {
            row.category.as_str()
        };
        format!(
            "{kind} Stripe de {gross} {cur} -{fee}={net} {cur} BRUT - {txn} ({created}- {description}) - {payout}",
            gross = row.gross,
            fee = row.fee,
            net = row.net,
            cur = row.currency,
            txn = row.balance_transaction_id,
            created = row.created,
            payout = row.payout_id,
        )
    };
    let budget = DETAILS_MAX_CHARS.saturating_sub(prefix_chars + compose("").chars().count());
    let description = row.description.as_deref().map(str::trim).unwrap_or("");
    compose(&truncate_chars(description, budget))
}

/// Les lignes neutres d'une ligne du rapport — Story 40-7b (AC-5) : le **brut**
/// (si non nul, `amount = gross`) et les **frais** (si non nuls, `amount =
/// −fee` : le champ `fee` est positif quand Stripe prélève, négatif quand il
/// rend). La somme des deux est le `net` de la ligne (contrôlé par 40-7a). Le
/// groupe de doublons est l'identifiant de la transaction de solde.
pub fn provider_lines_of_row(row: &StripePayoutRow) -> Vec<ProviderLine> {
    let reference = row
        .source_id
        .as_ref()
        .or(row.charge_id.as_ref())
        .map(|r| truncate_chars(r, TXID_COLUMN_MAX_CHARS));
    let category = truncate_chars(&row.category, PROVIDER_TYPE_MAX_CHARS);
    let base = |amount: Decimal, transaction_id: String, details: String| ProviderLine {
        group_id: row.balance_transaction_id.clone(),
        transaction_id,
        booking_date: row.date,
        value_date: row.available_on,
        amount,
        currency: row.currency.clone(),
        reference: reference.clone(),
        details: truncate_chars(&details, DETAILS_MAX_CHARS),
        end_to_end_id: Some(truncate_chars(&row.payout_id, NAME_MAX_CHARS)),
        counterparty_name: None,
        counterparty_email: None,
        provider_type: Some(category.clone()),
        rank: row.line,
        balance_after: None,
    };

    let mut out = Vec::with_capacity(2);
    if !row.gross.is_zero() {
        out.push(base(
            row.gross,
            row.balance_transaction_id.clone(),
            row_label(row),
        ));
    }
    if !row.fee.is_zero() {
        out.push(base(
            -row.fee,
            format!("{}{FEE_SUFFIX}", row.balance_transaction_id),
            format!(
                "{FEE_DETAILS_PREFIX}{}",
                label(row, FEE_DETAILS_PREFIX.chars().count())
            ),
        ));
    }
    out
}

/// La ligne neutre d'un versement : `− Σ net` de ses lignes, à la date du
/// versement, identifiée par `payout_id` (groupe `payout:<id>`, espace de noms
/// distinct de celui des transactions de solde).
fn payout_line(payout_id: &str, first: &StripePayoutRow, net: Decimal) -> ProviderLine {
    ProviderLine {
        group_id: format!("payout:{payout_id}"),
        transaction_id: payout_id.to_string(),
        booking_date: first.payout_date,
        value_date: None,
        amount: -net,
        currency: first.currency.clone(),
        reference: Some(truncate_chars(payout_id, TXID_COLUMN_MAX_CHARS)),
        details: format!(
            "{PAYOUT_DETAILS_PREFIX} {payout_id} {net} {cur}",
            cur = first.currency
        ),
        end_to_end_id: Some(truncate_chars(payout_id, NAME_MAX_CHARS)),
        counterparty_name: None,
        counterparty_email: None,
        provider_type: Some(PAYOUT_PROVIDER_TYPE.to_string()),
        rank: first.line,
        balance_after: None,
    }
}

/// Les lignes neutres d'**une devise** et ses versements — Story 40-7b
/// (AC-5) : les lignes de chaque ligne du rapport, puis une par versement dont
/// la somme des nets n'est pas nulle (un versement de somme nulle ne déplace
/// rien : pas de transaction, mais il reste au compte rendu).
pub fn lines_of_currency(rows: &[&StripePayoutRow]) -> (Vec<ProviderGroup>, Vec<PayoutReport>) {
    // Un groupe par ligne du rapport (brut + frais) : l'unité des doublons,
    // connue de la source (forme du socle de la 38-12a).
    let mut groups: Vec<ProviderGroup> = rows
        .iter()
        .map(|r| provider_lines_of_row(r))
        .filter(|g| !g.is_empty())
        .collect();

    // Versements, dans l'ordre de première apparition au fichier.
    let mut order: Vec<&str> = Vec::new();
    let mut by_payout: BTreeMap<&str, (&StripePayoutRow, usize, Decimal)> = BTreeMap::new();
    for row in rows {
        let slot = by_payout.entry(row.payout_id.as_str()).or_insert_with(|| {
            order.push(row.payout_id.as_str());
            (*row, 0, Decimal::ZERO)
        });
        slot.1 += 1;
        slot.2 += row.net;
    }
    let mut payouts = Vec::with_capacity(order.len());
    for payout_id in order {
        let (first, count, net) = by_payout[payout_id];
        payouts.push(PayoutReport {
            payout_id: payout_id.to_string(),
            currency: first.currency.clone(),
            lines: count,
            net,
        });
        if !net.is_zero() {
            groups.push(vec![payout_line(payout_id, first, net)]);
        }
    }
    (groups, payouts)
}

// ---------------------------------------------------------------------------
// Lecteur
// ---------------------------------------------------------------------------

/// Traduit le refus du lecteur (40-7a) en `422 STRIPE_CSV_INVALID`. Pour
/// l'objet lié absent ([`LINKED_OBJECT_COLUMNS`]), `details.hint` dit la marche
/// à suivre dans Stripe.
fn csv_error(e: StripeCsvError) -> AppError {
    let reason = e.code();
    let details = match &e {
        StripeCsvError::MissingColumns(columns) => {
            let mut details = serde_json::json!({ "columns": columns });
            if columns.iter().any(|c| c == LINKED_OBJECT_COLUMNS) {
                details["hint"] = serde_json::json!(crate::errors::t(
                    "error-stripe-csv-linked-object-hint",
                    "Dans Stripe, Rapports → Rapprochement des versements → Détaillé, ajoutez la colonne source_id (ou charge_id), puis exportez de nouveau.",
                ));
            }
            details
        }
        StripeCsvError::DuplicateColumn(column) => serde_json::json!({ "columns": [column] }),
        StripeCsvError::Lines(lines) => serde_json::json!({ "lines": lines }),
        StripeCsvError::Encoding(inner) => serde_json::json!({ "cause": inner.to_string() }),
        StripeCsvError::Empty => serde_json::json!({}),
    };
    AppError::StripeCsvInvalid { reason, details }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// `POST /api/v1/stripe-imports` — voir l'en-tête du module.
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<StripeImportResponse>), AppError> {
    let form = provider_imports::parse_form(multipart, "stripe.csv").await?;
    let balances = load_balances(
        &state.pool,
        current_user.company_id,
        PROVIDER_STRIPE,
        &form.account,
    )
    .await?;

    let csv = parse_stripe_payout_csv(&form.file_bytes).map_err(csv_error)?;
    check_id_lengths(&csv.rows).map_err(csv_error)?;

    let mut by_currency = BTreeMap::new();
    let mut payouts = Vec::new();
    for (currency, rows) in ventilate(&csv.rows) {
        let (groups, mut currency_payouts) = lines_of_currency(&rows);
        by_currency.insert(currency, groups);
        payouts.append(&mut currency_payouts);
    }
    let mut audit_details = serde_json::Map::new();
    audit_details.insert("provider".into(), serde_json::json!(PROVIDER_STRIPE));
    audit_details.insert("provider_account".into(), serde_json::json!(form.account));
    audit_details.insert("filename".into(), serde_json::json!(form.filename));
    audit_details.insert("payouts".into(), serde_json::json!(payouts));
    let request = ImportRequest {
        provider: PROVIDER_STRIPE,
        account: &form.account,
        filename: &form.filename,
        source: ImportSource::File(&form.file_bytes),
        retry_operation: "stripe_imports::create",
        audit_action: "stripe_import.created",
        audit_details,
        by_currency,
    };
    let currencies = import_lines(&state.pool, &current_user, &balances, &request).await?;

    Ok((
        StatusCode::CREATED,
        Json(StripeImportResponse {
            account: form.account,
            currencies,
            payouts,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::collections::HashSet;
    use std::str::FromStr;

    const FIXTURE: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv");

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn fixture_rows() -> Vec<StripePayoutRow> {
        parse_stripe_payout_csv(FIXTURE)
            .expect("fixture lisible")
            .rows
    }

    /// Une ligne synthétique d'un versement `po_X`, en CHF.
    fn row(id: &str, payout: &str, gross: &str, fee: &str) -> StripePayoutRow {
        let (gross, fee) = (dec(gross), dec(fee));
        StripePayoutRow {
            line: 2,
            payout_id: payout.into(),
            payout_date: NaiveDate::from_ymd_opt(2020, 2, 3).unwrap(),
            balance_transaction_id: id.into(),
            date: NaiveDate::from_ymd_opt(2020, 1, 30).unwrap(),
            created: "2020-01-30 14:02:41".into(),
            available_on: None,
            currency: "CHF".into(),
            gross,
            fee,
            net: gross - fee,
            category: "charge".into(),
            description: None,
            source_id: None,
            charge_id: None,
        }
    }

    fn lines_of(
        rows: &[StripePayoutRow],
        currency: &str,
    ) -> (Vec<ProviderLine>, Vec<PayoutReport>) {
        let par_devise = ventilate(rows);
        let (groups, payouts) = lines_of_currency(&par_devise[currency]);
        (groups.into_iter().flatten().collect(), payouts)
    }

    fn groups_of(rows: &[StripePayoutRow], currency: &str) -> Vec<ProviderGroup> {
        lines_of_currency(&ventilate(rows)[currency]).0
    }

    fn montants(lines: &[ProviderLine]) -> Vec<Decimal> {
        lines.iter().map(|l| l.amount).collect()
    }

    /// AC-5 — le rapport de la fixture se ventile par devise, rien n'est
    /// écarté ; la somme des transactions de chaque devise est nulle.
    #[test]
    fn la_fixture_se_ventile_par_devise_et_chaque_somme_est_nulle() {
        let rows = fixture_rows();
        let par_devise = ventilate(&rows);
        assert_eq!(
            par_devise.keys().map(String::as_str).collect::<Vec<_>>(),
            ["CHF", "EUR", "USD"]
        );
        for (devise, attendu) in [("CHF", 8), ("EUR", 7), ("USD", 5)] {
            let (lines, _) = lines_of(&rows, devise);
            let txs = montants(&lines);
            assert_eq!(txs.len(), attendu, "{devise}");
            assert_eq!(txs.iter().sum::<Decimal>(), Decimal::ZERO, "{devise}");
        }
    }

    /// Brut et frais, chacun de son signe : les frais sont l'opposé du champ
    /// `fee`, positifs quand Stripe les rend (remboursement).
    #[test]
    fn brut_et_frais_chacun_de_son_signe() {
        let txs = provider_lines_of_row(&row("txn_A", "po_X", "100.00", "3.20"));
        assert_eq!(txs.len(), 2);
        assert_eq!(
            (txs[0].amount, txs[0].transaction_id.as_str()),
            (dec("100.00"), "txn_A")
        );
        assert_eq!(
            (txs[1].amount, txs[1].transaction_id.as_str()),
            (dec("-3.20"), "txn_A:FEE")
        );
        assert!(txs[1].details.starts_with("Frais Stripe · "));
        // Frais rendus d'un remboursement.
        let txs = provider_lines_of_row(&row("txn_B", "po_X", "-20.00", "-0.60"));
        assert_eq!((txs[0].amount, txs[1].amount), (dec("-20.00"), dec("0.60")));
        // Une ligne sans frais ne donne que son brut ; une ligne de frais seuls
        // (brut nul) que ses frais.
        assert_eq!(
            provider_lines_of_row(&row("txn_C", "po_X", "-0.50", "0")).len(),
            1
        );
        let seuls = provider_lines_of_row(&row("txn_D", "po_X", "0", "1.00"));
        assert_eq!(seuls.len(), 1);
        assert_eq!(seuls[0].transaction_id, "txn_D:FEE");
        assert!(provider_lines_of_row(&row("txn_E", "po_X", "0", "0")).is_empty());
    }

    /// La référence est l'objet lié `source_id`, à défaut `charge_id` ; le
    /// versement et la catégorie suivent ; sans objet lié, aucune référence.
    #[test]
    fn la_reference_est_source_id_puis_charge_id() {
        let mut r = row("txn_A", "po_X", "10", "0");
        assert_eq!(provider_lines_of_row(&r)[0].reference, None);
        r.charge_id = Some("ch_9".into());
        assert_eq!(
            provider_lines_of_row(&r)[0].reference.as_deref(),
            Some("ch_9")
        );
        r.source_id = Some("re_1".into());
        let t = &provider_lines_of_row(&r)[0];
        assert_eq!(t.reference.as_deref(), Some("re_1"));
        assert_eq!(t.end_to_end_id.as_deref(), Some("po_X"));
        assert_eq!(t.provider_type.as_deref(), Some("charge"));
    }

    /// Le versement : `− Σ net` de ses lignes, à sa date, identifié par son
    /// identifiant ; un versement de somme nulle ne crée rien mais reste au
    /// compte rendu ; le retour d'un versement (somme négative) est un crédit.
    #[test]
    fn le_versement_reprend_la_somme_des_nets() {
        let rows = vec![
            row("txn_A", "po_1", "100.00", "3.20"),
            row("txn_B", "po_1", "-20.00", "-0.60"),
            row("txn_C", "po_2", "5.00", "0"),
            row("txn_D", "po_2", "-5.00", "0"),
            row("txn_E", "po_3", "-8.00", "0"),
        ];
        let (lines, payouts) = lines_of(&rows, "CHF");
        let po = |id: &str| {
            lines
                .iter()
                .find(|l| l.group_id == format!("payout:{id}"))
                .cloned()
        };
        let po1 = po("po_1").unwrap();
        assert_eq!(po1.amount, dec("-77.40"), "−(96.80 − 19.40)");
        assert_eq!(po1.transaction_id, "po_1");
        assert_eq!(po1.provider_type.as_deref(), Some("payout"));
        assert_eq!(
            po1.booking_date,
            NaiveDate::from_ymd_opt(2020, 2, 3).unwrap()
        );
        assert!(po("po_2").is_none(), "somme nulle : aucune transaction");
        assert_eq!(po("po_3").unwrap().amount, dec("8.00"));
        assert_eq!(
            payouts
                .iter()
                .map(|p| (p.payout_id.as_str(), p.lines, p.net))
                .collect::<Vec<_>>(),
            [
                ("po_1", 2, dec("77.40")),
                ("po_2", 2, dec("0.00")),
                ("po_3", 1, dec("-8.00"))
            ]
        );
    }

    /// AC-6 — doublons : un identifiant déjà en base ou déjà paru dans le
    /// fichier ignore l'entrée entière (ses transactions comptées).
    #[test]
    fn doublon_dans_le_fichier_et_en_base() {
        let rows = vec![
            row("txn_A", "po_1", "100.00", "3.20"),
            row("txn_A", "po_1", "100.00", "3.20"),
            row("txn_B", "po_1", "50.00", "0"),
        ];
        let groups = groups_of(&rows, "CHF");
        let mut existing = HashSet::new();
        existing.insert("po_1".to_string());
        let (kept, duplicates) = provider_imports::dedupe_groups(&groups, &existing);
        // txn_A (2 lignes) en double dans le fichier, et le versement déjà connu.
        assert_eq!(
            kept.iter()
                .map(|l| l.transaction_id.as_str())
                .collect::<Vec<_>>(),
            ["txn_A", "txn_A:FEE", "txn_B"]
        );
        assert_eq!(duplicates, 2 + 1);
    }

    /// Deux lignes identiques **côte à côte** dans le rapport sont deux groupes :
    /// la seconde est un doublon (les groupes sont ceux de la source, forme du
    /// socle de la 38-12a — le correctif de l'adjacence n'a plus d'objet).
    #[test]
    fn deux_lignes_identiques_voisines_sont_deux_groupes() {
        let rows = vec![
            row("txn_A", "po_1", "10.00", "1.00"),
            row("txn_A", "po_1", "10.00", "1.00"),
        ];
        let groups = groups_of(&rows, "CHF");
        // Deux lignes du rapport, plus le versement : trois groupes.
        assert_eq!(groups.len(), 3);
        let (kept, duplicates) = provider_imports::dedupe_groups(&groups, &HashSet::new());
        assert_eq!(kept.len(), 3, "brut, frais, versement");
        assert_eq!(duplicates, 2, "la seconde ligne : brut et frais");
    }

    /// L1 (revue P1) — une description très longue ne coupe ni la fin du
    /// libellé (`) - po_…`), ni celle du libellé des frais : seule la
    /// description cède, et le libellé tient dans la borne.
    #[test]
    fn une_longue_description_ne_coupe_pas_la_fin_du_libelle() {
        let mut r = row("txn_L", "po_L", "100.00", "3.20");
        r.description = Some("x".repeat(900));
        let lignes = provider_lines_of_row(&r);
        assert_eq!(lignes.len(), 2);
        for l in &lignes {
            assert!(l.details.ends_with(") - po_L"), "{}", l.details);
            assert!(l.details.chars().count() <= DETAILS_MAX_CHARS);
        }
        // Les identifiants restent cherchables dans le texte.
        assert!(lignes[0].details.contains("txn_L"));
        // Une description courte reste entière.
        r.description = Some("courte".into());
        assert!(row_label(&r).contains("(2020-01-30 14:02:41- courte) - po_L"));
    }

    /// Amendement du 2026-11-20 — le libellé suit le format du tableur Crésus du
    /// Project Lead, au caractère près, pour le brut, les frais et le versement
    /// (fixture fictive de 40-7a).
    #[test]
    fn les_libelles_suivent_le_format_du_tableur() {
        let rows = fixture_rows();
        let ligne = |id: &str| {
            rows.iter()
                .find(|r| r.balance_transaction_id == id)
                .unwrap()
        };
        // Encaissement (catégorie `charge`), brut puis frais.
        let lignes = provider_lines_of_row(ligne("txn_TEST0001"));
        assert_eq!(
            lignes[0].details,
            "Encaissement Stripe de 100.00 CHF -3.20=96.80 CHF BRUT - txn_TEST0001 (2020-01-29 10:15:04- Vente fictive A) - po_TEST0001"
        );
        assert_eq!(
            lignes[1].details,
            "Frais Stripe · Encaissement Stripe de 100.00 CHF -3.20=96.80 CHF BRUT - txn_TEST0001 (2020-01-29 10:15:04- Vente fictive A) - po_TEST0001"
        );
        // Une autre catégorie dit sa catégorie ; frais nuls : « -0.00 ».
        assert_eq!(
            provider_lines_of_row(ligne("txn_TEST0004"))[0].details,
            "fee Stripe de -0.50 CHF -0.00=-0.50 CHF BRUT - txn_TEST0004 (2020-02-01 00:00:00- Frais fictifs de la plateforme) - po_TEST0001"
        );
        // Frais rendus : le signe de `-<frais>` est littéral.
        assert_eq!(
            provider_lines_of_row(ligne("txn_TEST0003"))[0].details,
            "refund Stripe de -20.00 CHF --0.60=-19.40 CHF BRUT - txn_TEST0003 (2020-01-31 09:40:12- Remboursement fictif de la vente A) - po_TEST0001"
        );
        // Devise en majuscules, description absente : parenthèse « (… - ) ».
        let mut r = row("txn_X", "po_X", "5.00", "0");
        r.currency = "EUR".into();
        assert_eq!(
            row_label(&r),
            "Encaissement Stripe de 5.00 EUR -0=5.00 EUR BRUT - txn_X (2020-01-30 14:02:41- ) - po_X"
        );
        // Versement : identifiant et somme des nets.
        let (lines, _) = lines_of(&rows, "CHF");
        let versement = lines
            .iter()
            .find(|l| l.transaction_id == "po_TEST0001")
            .unwrap();
        assert_eq!(versement.details, "Versement Stripe po_TEST0001 125.15 CHF");
    }

    /// Un identifiant plus long que sa colonne (frais `:FEE` compris) est une
    /// erreur de ligne `ID`, jamais une erreur de base.
    #[test]
    fn identifiant_borne_a_la_colonne() {
        let ok = row(&"a".repeat(STRIPE_ID_MAX_CHARS), "po_1", "1", "0");
        assert!(check_id_lengths(std::slice::from_ref(&ok)).is_ok());
        let long = row(&"a".repeat(STRIPE_ID_MAX_CHARS + 1), "po_1", "1", "0");
        match check_id_lengths(&[ok, long]) {
            Err(StripeCsvError::Lines(lignes)) => {
                assert_eq!(lignes.len(), 1);
                assert_eq!(lignes[0].code, StripeLineErrorCode::Id);
                assert_eq!(lignes[0].field, Some("balance_transaction_id"));
            }
            autre => panic!("attendu une erreur de ligne, reçu {autre:?}"),
        }
    }
}
