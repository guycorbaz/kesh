//! Route `POST /api/v1/paypal-imports` — Story 38-11b (Epic 38, phase 1).
//!
//! Importe d'un geste le CSV PayPal « Balance affecting transactions » d'un
//! **compte PayPal** (multipart : `file`, `account`), **ventilé par devise** :
//! chaque devise du fichier va au compte bancaire Kesh, solde de ce compte
//! PayPal dans cette devise (`bank_accounts.provider = 'paypal'`,
//! `provider_account` = le libellé, devise = celle du compte du plan lié).
//!
//! Comptable+, clés d'API `read-write` admises, comme `POST
//! /api/v1/bank-imports`. Taille maximale : celle de l'import bancaire.
//!
//! **Ordre des refus**, rien d'écrit à aucun : `400 VALIDATION_ERROR` (champ
//! manquant) → `404 PROVIDER_ACCOUNT_NOT_FOUND` → `422 PAYPAL_CSV_INVALID` →
//! `422 PROVIDER_CURRENCY_ACCOUNT_MISSING` → `409 PROVIDER_CURRENCY_DUPLICATE`
//! → `409 BANK_ACCOUNT_CLOSED` → `422 BANK_IMPORT_DUPLICATE_FILE` (ce dernier
//! jugé sous les verrous, dans la transaction d'écriture). Un verrou de compte
//! non obtenu rend `409 RECONCILIATION_ACCOUNT_LOCKED`.
//!
//! **Ventilation** ([`ventilate`], fonction pure) : lignes `Status ≠
//! Completed` (sans casse), lignes `Memo` et **paires neutres** de retenue
//! écartées et comptées ; chaque ligne restante donne une transaction pour son
//! brut et une pour ses frais (chacune de son signe, [`transactions_of_row`]).
//! **Doublons** : une transaction du même compte bancaire porte déjà
//! l'identifiant (brut ou `:FEE`), ou une ligne précédente du fichier porte le
//! même identifiant → la ligne est ignorée. La fenêtre date-montant-référence
//! des imports bancaires ne s'applique pas : deux ventes du même montant le
//! même jour sont deux transactions.
//!
//! **Écriture** : par le **cœur commun** ([`crate::routes::provider_imports`],
//! Story 38-12a, extrait d'ici) — verrous de chaque compte visé, une seule
//! transaction pour toutes les devises (tout ou rien), relecture verrouillante
//! des soldes, empreinte, doublons, traces, rejeu sur interblocage (opération
//! `paypal_imports::create`). Cette route lit le fichier, le ventile
//! ([`ventilate`]) et le traduit en lignes neutres ([`provider_lines`]) ; le
//! téléchargement par l'API ([`crate::routes::paypal_downloads`]) fait de même.

use std::collections::{BTreeMap, HashSet};

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Serialize;

use kesh_db::entities::bank_account::PROVIDER_PAYPAL;
use kesh_db::entities::bank_transaction::NewBankTransaction;
use kesh_import::paypal::{
    BalanceImpact, COL_TRANSACTION_ID, PaypalCsvError, PaypalLineError, PaypalLineErrorCode,
    PaypalRow, parse_paypal_csv,
};
use kesh_import::{MAX_CSV_LINE_ERROR_VALUE_CHARS, MAX_CSV_LINE_ERRORS};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
pub use crate::routes::provider_imports::CurrencyReport;
use crate::routes::provider_imports::{
    ImportRequest, ImportSource, ProviderLine, import_lines, load_balances,
};

/// Suffixe de l'identifiant de la transaction des **frais** d'une ligne
/// PayPal : `<id>:FEE`. Un identifiant PayPal n'a jamais de `:` (38-11a,
/// AC-3), si bien que le suffixe ne peut heurter un identifiant réel.
pub const PAYPAL_FEE_SUFFIX: &str = ":FEE";

/// Types PayPal d'une **retenue neutre** : une paire de deux lignes de ces
/// types (même devise, même `Reference Txn ID` non vide, brut opposé, frais
/// nuls) ne change pas le solde et s'écarte de l'import. Liste fermée.
pub const PAYPAL_NEUTRAL_PAIR_TYPES: [&str; 6] = [
    "Payment Review Hold",
    "Payment Review Release",
    "Hold on Balance for Dispute Investigation",
    "Reversal of General Account Hold",
    "Account Hold for Open Authorization",
    "Reversal of Account Hold",
];

/// Le seul statut importé (comparé sans casse).
const COMPLETED: &str = "Completed";

/// Bornes des colonnes texte de `bank_transactions`, en caractères (le libellé
/// suit la borne de la saisie manuelle).
const DETAILS_MAX_CHARS: usize = 500;
const NAME_MAX_CHARS: usize = 255;
const PROVIDER_TYPE_MAX_CHARS: usize = 64;
const EMAIL_MAX_CHARS: usize = 254;
const FILENAME_MAX_BYTES: usize = 255;
/// Longueur de la colonne `bank_transactions.transaction_id`, et de
/// `reference`, en caractères.
const TXID_COLUMN_MAX_CHARS: usize = 255;
/// Préfixe de la référence d'une transaction PayPal : `TXN <id>`.
const REFERENCE_PREFIX: &str = "TXN ";
/// Longueur maximale d'un identifiant de transaction PayPal admis à l'import —
/// Story 38-11b, revue P1 (L2) : l'identifiant des frais (`<id>:FEE`) et la
/// référence (`TXN <id>`) doivent tenir dans leur colonne de 255. Le lecteur
/// (38-11a) ne borne pas la longueur ; au-delà, la ligne est refusée en `422
/// PAYPAL_CSV_INVALID` (erreur de ligne `transaction_id`), et non par une
/// erreur de la base (500). Un identifiant PayPal réel compte 17 caractères.
pub const PAYPAL_TRANSACTION_ID_MAX_CHARS: usize = TXID_COLUMN_MAX_CHARS - PAYPAL_FEE_SUFFIX.len();
const _: () =
    assert!(REFERENCE_PREFIX.len() + PAYPAL_TRANSACTION_ID_MAX_CHARS <= TXID_COLUMN_MAX_CHARS);

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// Une ligne écartée parce que son statut n'est pas `Completed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotCompletedLine {
    pub transaction_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub status: String,
}

/// Les lignes écartées avant tout, par motif (AC-5).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedLines {
    pub not_completed: Vec<NotCompletedLine>,
    /// Lignes `Balance Impact = Memo` (statut `Completed`).
    pub memo: usize,
    /// Paires de retenue neutres (une paire = deux lignes).
    pub neutral_pairs: usize,
}

/// Réponse `201` de l'import (AC-6).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaypalImportResponse {
    pub account: String,
    pub currencies: Vec<CurrencyReport>,
    pub skipped: SkippedLines,
}

// ---------------------------------------------------------------------------
// Ventilation (pure)
// ---------------------------------------------------------------------------

/// Les lignes du fichier, filtrées et rangées par devise (ordre du fichier).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ventilation {
    pub by_currency: BTreeMap<String, Vec<PaypalRow>>,
    pub skipped: SkippedLines,
}

/// Ventile les lignes lues par 38-11a — Story 38-11b (AC-5). Dans l'ordre :
/// statut ≠ `Completed` (sans casse) → `notCompleted` ; `Memo` → `memo` ;
/// paires neutres → `neutralPairs` (une retenue sans sa levée dans le fichier
/// reste importée). Les lignes restantes sont rangées par devise.
pub fn ventilate(rows: &[PaypalRow]) -> Ventilation {
    let mut skipped = SkippedLines::default();
    let mut kept: Vec<&PaypalRow> = Vec::new();
    for row in rows {
        if !row.status.trim().eq_ignore_ascii_case(COMPLETED) {
            skipped.not_completed.push(NotCompletedLine {
                transaction_id: row.transaction_id.clone(),
                kind: row.kind.clone(),
                status: row.status.clone(),
            });
        } else if row.balance_impact == BalanceImpact::Memo {
            skipped.memo += 1;
        } else {
            kept.push(row);
        }
    }

    let neutral = neutral_pair_indices(&kept);
    skipped.neutral_pairs = neutral.len() / 2;

    let mut by_currency: BTreeMap<String, Vec<PaypalRow>> = BTreeMap::new();
    for (i, row) in kept.into_iter().enumerate() {
        if !neutral.contains(&i) {
            by_currency
                .entry(row.currency.clone())
                .or_default()
                .push(row.clone());
        }
    }
    Ventilation {
        by_currency,
        skipped,
    }
}

/// Indices (dans `rows`) des lignes appariées en **paires neutres** : deux
/// lignes de types [`PAYPAL_NEUTRAL_PAIR_TYPES`], même devise, même `Reference
/// Txn ID` non vide, brut opposé non nul, frais nuls. Chaque ligne entre dans
/// une paire au plus ; on apparie à la première ligne libre qui convient, dans
/// l'ordre du fichier.
fn neutral_pair_indices(rows: &[&PaypalRow]) -> HashSet<usize> {
    let candidate = |r: &PaypalRow| {
        PAYPAL_NEUTRAL_PAIR_TYPES.contains(&r.kind.trim())
            && r.fee.is_zero()
            && !r.gross.is_zero()
            && r.reference_txn_id.as_deref().is_some_and(|s| !s.is_empty())
    };
    let mut paired = HashSet::new();
    for i in 0..rows.len() {
        if paired.contains(&i) || !candidate(rows[i]) {
            continue;
        }
        let a = rows[i];
        let partner = (i + 1..rows.len()).find(|&j| {
            let b = rows[j];
            !paired.contains(&j)
                && candidate(b)
                && b.currency == a.currency
                && b.reference_txn_id == a.reference_txn_id
                && b.gross == -a.gross
        });
        if let Some(j) = partner {
            paired.insert(i);
            paired.insert(j);
        }
    }
    paired
}

/// Coupe `s` à `max` caractères.
fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Coupe `s` à `max` octets, sur une frontière de caractère.
fn truncate_bytes(s: &str, max: usize) -> String {
    let mut end = s.len().min(max);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Refuse les lignes dont l'identifiant de transaction dépasse
/// [`PAYPAL_TRANSACTION_ID_MAX_CHARS`] — Story 38-11b, revue P1 (L2) : erreurs
/// de ligne `transaction_id`, au plus [`MAX_CSV_LINE_ERRORS`], valeur tronquée
/// comme celles du lecteur. Toutes les lignes du fichier, quel que soit leur
/// statut (un identifiant aussi long n'est pas un identifiant PayPal).
pub fn check_transaction_id_lengths(rows: &[PaypalRow]) -> Result<(), PaypalCsvError> {
    let errors: Vec<PaypalLineError> = rows
        .iter()
        .filter(|r| r.transaction_id.chars().count() > PAYPAL_TRANSACTION_ID_MAX_CHARS)
        .take(MAX_CSV_LINE_ERRORS)
        .map(|r| {
            let mut value = truncate_chars(&r.transaction_id, MAX_CSV_LINE_ERROR_VALUE_CHARS);
            if value.len() < r.transaction_id.len() {
                value.push('…');
            }
            PaypalLineError {
                line: r.line,
                field: Some(COL_TRANSACTION_ID),
                code: PaypalLineErrorCode::TransactionId,
                value: Some(value),
            }
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(PaypalCsvError::Lines(errors))
    }
}

/// Les identifiants des transactions qu'une ligne produit : le brut (si non
/// nul) et les frais (si non nuls, `<id>:FEE`).
pub fn row_transaction_ids(row: &PaypalRow) -> Vec<String> {
    let mut ids = Vec::with_capacity(2);
    if !row.gross.is_zero() {
        ids.push(row.transaction_id.clone());
    }
    if !row.fee.is_zero() {
        ids.push(format!("{}{PAYPAL_FEE_SUFFIX}", row.transaction_id));
    }
    ids
}

/// Les lignes neutres d'une ligne PayPal — Story 38-11b (AC-5), traduites pour
/// le cœur commun par la Story 38-12a : le **brut** (si non nul) et les
/// **frais** (si non nuls, quel que soit leur signe : négatifs d'ordinaire,
/// positifs sur certains remboursements, identifiant `<id>:FEE`). La somme des
/// deux est le `Net` de la ligne (contrôlé par 38-11a). Fonction pure.
pub fn provider_lines_of_row(row: &PaypalRow) -> Vec<ProviderLine> {
    let opt = |v: &Option<String>, max: usize| {
        v.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| truncate_chars(s, max))
    };
    let email = if row.gross.is_sign_negative() {
        opt(&row.to_email, EMAIL_MAX_CHARS)
    } else {
        opt(&row.from_email, EMAIL_MAX_CHARS)
    };
    let kind = row.kind.trim().to_string();
    let base = |amount, transaction_id: String, details: String| ProviderLine {
        group_id: row.transaction_id.clone(),
        transaction_id,
        booking_date: row.date,
        value_date: None,
        amount,
        currency: row.currency.clone(),
        reference: Some(format!("{REFERENCE_PREFIX}{}", row.transaction_id)),
        details: truncate_chars(&details, DETAILS_MAX_CHARS),
        end_to_end_id: opt(&row.reference_txn_id, NAME_MAX_CHARS),
        counterparty_name: opt(&row.name, NAME_MAX_CHARS),
        counterparty_email: email.clone(),
        provider_type: Some(truncate_chars(&kind, PROVIDER_TYPE_MAX_CHARS)),
        rank: row.line,
        balance_after: row.balance,
    };

    let mut out = Vec::with_capacity(2);
    if !row.gross.is_zero() {
        let details = [
            Some(kind.as_str()),
            row.item_title.as_deref(),
            row.subject.as_deref(),
            row.note.as_deref(),
        ]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        out.push(base(row.gross, row.transaction_id.clone(), details));
    }
    if !row.fee.is_zero() {
        out.push(base(
            row.fee,
            format!("{}{PAYPAL_FEE_SUFFIX}", row.transaction_id),
            format!("Frais PayPal · {kind}"),
        ));
    }
    out
}

/// Les transactions bancaires d'une ligne ([`provider_lines_of_row`]).
pub fn transactions_of_row(
    row: &PaypalRow,
    company_id: i64,
    bank_account_id: i64,
) -> Vec<NewBankTransaction> {
    provider_lines_of_row(row)
        .into_iter()
        .map(|l| l.into_new_bank_transaction(company_id, bank_account_id))
        .collect()
}

/// Les lignes neutres d'une ventilation, par devise (ordre du fichier) :
/// l'entrée du cœur commun (Story 38-12a). Fonction pure.
pub fn provider_lines(ventilation: &Ventilation) -> BTreeMap<String, Vec<ProviderLine>> {
    ventilation
        .by_currency
        .iter()
        .map(|(currency, rows)| {
            (
                currency.clone(),
                rows.iter().flat_map(provider_lines_of_row).collect(),
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Multipart
// ---------------------------------------------------------------------------

struct PaypalImportForm {
    file_bytes: axum::body::Bytes,
    filename: String,
    account: String,
}

async fn parse_form(mut multipart: Multipart) -> Result<PaypalImportForm, AppError> {
    let mut file: Option<(axum::body::Bytes, String)> = None;
    let mut account: Option<String> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart : {e}")))?
    {
        match field.name() {
            Some("file") => {
                if file.is_some() {
                    return Err(AppError::Validation("champ `file` en double".into()));
                }
                let name = field.file_name().unwrap_or("paypal.csv").to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                file = Some((bytes, name));
            }
            Some("account") => {
                if account.is_some() {
                    return Err(AppError::Validation("champ `account` en double".into()));
                }
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                account = Some(text);
            }
            _ => {}
        }
    }
    let (file_bytes, filename) =
        file.ok_or_else(|| AppError::Validation("champ `file` manquant".into()))?;
    let account = account
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::Validation("champ `account` manquant".into()))?;
    let filename = truncate_bytes(filename.trim(), FILENAME_MAX_BYTES);
    Ok(PaypalImportForm {
        file_bytes,
        filename: if filename.is_empty() {
            "paypal.csv".into()
        } else {
            filename
        },
        account,
    })
}

/// Traduit le refus du lecteur (38-11a) en `422 PAYPAL_CSV_INVALID`.
fn csv_error(e: PaypalCsvError) -> AppError {
    let reason = e.code();
    let details = match &e {
        PaypalCsvError::MissingColumns(columns) => serde_json::json!({ "columns": columns }),
        PaypalCsvError::DuplicateColumn(column) => serde_json::json!({ "columns": [column] }),
        PaypalCsvError::Lines(lines) => serde_json::json!({ "lines": lines }),
        PaypalCsvError::Encoding(inner) => serde_json::json!({ "cause": inner.to_string() }),
        PaypalCsvError::Empty
        | PaypalCsvError::DateOrderConflict
        | PaypalCsvError::DateOrderAmbiguous => serde_json::json!({}),
    };
    AppError::PaypalCsvInvalid { reason, details }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// `POST /api/v1/paypal-imports` — voir l'en-tête du module.
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<PaypalImportResponse>), AppError> {
    let form = parse_form(multipart).await?;
    let balances = load_balances(
        &state.pool,
        current_user.company_id,
        PROVIDER_PAYPAL,
        &form.account,
    )
    .await?;

    let csv = parse_paypal_csv(&form.file_bytes).map_err(csv_error)?;
    check_transaction_id_lengths(&csv.rows).map_err(csv_error)?;
    let ventilation = ventilate(&csv.rows);

    let mut audit_details = serde_json::Map::new();
    audit_details.insert("provider".into(), serde_json::json!(PROVIDER_PAYPAL));
    audit_details.insert("provider_account".into(), serde_json::json!(form.account));
    audit_details.insert("filename".into(), serde_json::json!(form.filename));
    audit_details.insert("skipped".into(), serde_json::json!(ventilation.skipped));
    let request = ImportRequest {
        provider: PROVIDER_PAYPAL,
        account: &form.account,
        filename: &form.filename,
        source: ImportSource::File(&form.file_bytes),
        retry_operation: "paypal_imports::create",
        audit_action: "paypal_import.created",
        audit_details,
        by_currency: provider_lines(&ventilation),
    };
    let currencies = import_lines(&state.pool, &current_user, &balances, &request).await?;

    Ok((
        StatusCode::CREATED,
        Json(PaypalImportResponse {
            account: form.account,
            currencies,
            skipped: ventilation.skipped,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::provider_imports::dedupe_lines;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    const FIXTURE: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv");

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn fixture_rows() -> Vec<PaypalRow> {
        parse_paypal_csv(FIXTURE).expect("fixture lisible").rows
    }

    fn row(id: &str, kind: &str, currency: &str, gross: &str, fee: &str) -> PaypalRow {
        let mut r = fixture_rows().remove(0);
        r.transaction_id = id.into();
        r.kind = kind.into();
        r.currency = currency.into();
        r.gross = dec(gross);
        r.fee = dec(fee);
        r.net = r.gross + r.fee;
        r.reference_txn_id = None;
        r.balance_impact = if r.gross.is_sign_negative() {
            BalanceImpact::Debit
        } else {
            BalanceImpact::Credit
        };
        r
    }

    #[test]
    fn la_fixture_se_ventile_par_devise_et_par_motif() {
        let v = ventilate(&fixture_rows());
        assert_eq!(
            v.by_currency.keys().cloned().collect::<Vec<_>>(),
            ["CHF", "EUR", "USD"]
        );
        let ids = |c: &str| {
            v.by_currency[c]
                .iter()
                .map(|r| r.transaction_id.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids("EUR"),
            [
                "0TEST00000000001A",
                "0TEST00000000003C",
                "0TEST00000000004D",
                "0TEST0000000000AK",
                "0TEST0000000000BL"
            ]
        );
        assert_eq!(ids("USD"), ["0TEST00000000002B", "0TEST00000000005E"]);
        assert_eq!(ids("CHF"), ["0TEST00000000006F", "0TEST00000000007G"]);
        assert_eq!(
            v.skipped.not_completed,
            [
                NotCompletedLine {
                    transaction_id: "0TEST0000000000CM".into(),
                    kind: "Website Payment".into(),
                    status: "Pending".into()
                },
                NotCompletedLine {
                    transaction_id: "0TEST0000000000DN".into(),
                    kind: "General Payment".into(),
                    status: "Denied".into()
                }
            ]
        );
        assert_eq!(v.skipped.memo, 1);
        assert_eq!(v.skipped.neutral_pairs, 1);
    }

    #[test]
    fn la_somme_des_transactions_d_une_devise_est_celle_de_ses_nets() {
        let v = ventilate(&fixture_rows());
        for (currency, rows) in &v.by_currency {
            let txs: Decimal = rows
                .iter()
                .flat_map(|r| transactions_of_row(r, 1, 1))
                .map(|t| t.amount)
                .sum();
            let nets: Decimal = rows.iter().map(|r| r.net).sum();
            assert_eq!(txs, nets, "devise {currency}");
        }
        // 9 transactions EUR (dont des frais positifs), 3 USD, 3 CHF.
        let count = |c: &str| {
            v.by_currency[c]
                .iter()
                .flat_map(|r| transactions_of_row(r, 1, 1))
                .count()
        };
        assert_eq!((count("EUR"), count("USD"), count("CHF")), (9, 3, 3));
    }

    #[test]
    fn brut_et_frais_chacun_de_son_signe() {
        let rows = fixture_rows();
        let refund = rows
            .iter()
            .find(|r| r.transaction_id == "0TEST0000000000BL")
            .unwrap();
        let txs = transactions_of_row(refund, 7, 9);
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].amount, dec("-30.00"));
        assert_eq!(txs[0].transaction_id.as_deref(), Some("0TEST0000000000BL"));
        assert_eq!(txs[0].reference.as_deref(), Some("TXN 0TEST0000000000BL"));
        assert_eq!(txs[0].end_to_end_id.as_deref(), Some("0TEST0000000000AK"));
        assert_eq!(txs[0].counterparty_name.as_deref(), Some("Daniel Fictif"));
        // Débit : l'adresse du destinataire.
        assert_eq!(
            txs[0].counterparty_email.as_deref(),
            Some("daniel@example.invalid")
        );
        assert_eq!(txs[0].provider_type.as_deref(), Some("Payment Refund"));
        assert_eq!(
            txs[0].details,
            "Payment Refund · Remboursement complet\nà la demande du client"
        );
        assert_eq!(
            txs[1].amount,
            dec("0.96"),
            "frais positifs d'un remboursement"
        );
        assert_eq!(
            txs[1].transaction_id.as_deref(),
            Some("0TEST0000000000BL:FEE")
        );
        assert_eq!(txs[1].details, "Frais PayPal · Payment Refund");
        assert!(
            txs.iter()
                .all(|t| t.bank_account_id == 9 && t.company_id == 7)
        );

        let sale = rows
            .iter()
            .find(|r| r.transaction_id == "0TEST00000000001A")
            .unwrap();
        let txs = transactions_of_row(sale, 1, 1);
        // Crédit : l'adresse de l'émetteur.
        assert_eq!(
            txs[0].counterparty_email.as_deref(),
            Some("alice@example.invalid")
        );
        assert_eq!(txs[0].details, "Website Payment · Commande 1001");
        assert_eq!(txs[1].amount, dec("-3.20"));
    }

    #[test]
    fn une_ligne_sans_frais_donne_une_transaction() {
        let r = row(
            "0TESTSANSFRAIS",
            "General Withdrawal",
            "CHF",
            "-10.00",
            "0.00",
        );
        assert_eq!(transactions_of_row(&r, 1, 1).len(), 1);
        assert_eq!(row_transaction_ids(&r), ["0TESTSANSFRAIS"]);
    }

    #[test]
    fn une_retenue_sans_sa_levee_reste_importee() {
        let mut hold = row("0TESTHOLD", "Payment Review Hold", "EUR", "-5.00", "0.00");
        hold.reference_txn_id = Some("0TESTREF".into());
        let v = ventilate(std::slice::from_ref(&hold));
        assert_eq!(v.skipped.neutral_pairs, 0);
        assert_eq!(v.by_currency["EUR"].len(), 1);
    }

    #[test]
    fn une_paire_se_forme_seulement_aux_conditions_ecrites() {
        let pair = |kind_b: &str, currency_b: &str, gross_b: &str, fee_b: &str| {
            let mut a = row("0TESTA", "Payment Review Hold", "EUR", "-5.00", "0.00");
            a.reference_txn_id = Some("0TESTREF".into());
            let mut b = row("0TESTB", kind_b, currency_b, gross_b, fee_b);
            b.reference_txn_id = Some("0TESTREF".into());
            ventilate(&[a, b]).skipped.neutral_pairs
        };
        assert_eq!(pair("Payment Review Release", "EUR", "5.00", "0.00"), 1);
        assert_eq!(
            pair("Website Payment", "EUR", "5.00", "0.00"),
            0,
            "type hors liste"
        );
        assert_eq!(
            pair("Payment Review Release", "USD", "5.00", "0.00"),
            0,
            "autre devise"
        );
        assert_eq!(
            pair("Payment Review Release", "EUR", "4.00", "0.00"),
            0,
            "brut non opposé"
        );
        assert_eq!(
            pair("Payment Review Release", "EUR", "5.10", "-0.10"),
            0,
            "frais non nuls"
        );
        // Référence vide : pas de paire.
        let a = row("0TESTA", "Payment Review Hold", "EUR", "-5.00", "0.00");
        let b = row("0TESTB", "Payment Review Release", "EUR", "5.00", "0.00");
        assert_eq!(ventilate(&[a, b]).skipped.neutral_pairs, 0);
    }

    #[test]
    fn le_statut_se_compare_sans_casse() {
        let mut r = row("0TESTC", "Website Payment", "EUR", "5.00", "0.00");
        r.status = "COMPLETED".into();
        assert!(ventilate(&[r]).skipped.not_completed.is_empty());
    }

    #[test]
    fn doublon_dans_le_fichier_et_en_base() {
        let a = row("0TESTDUP", "Website Payment", "EUR", "10.00", "-0.50");
        let b = a.clone();
        let c = row("0TESTNEW", "Website Payment", "EUR", "20.00", "-1.00");
        let d = row("0TESTOLD", "Website Payment", "EUR", "30.00", "0.00");
        let rows = [a, b, c, d];
        // Story 38-12a : les doublons se jugent au cœur commun, sur les lignes
        // neutres groupées par ligne du fichier ; assertions inchangées.
        let lines: Vec<ProviderLine> = rows.iter().flat_map(provider_lines_of_row).collect();
        let existing: HashSet<String> = ["0TESTOLD".to_string()].into_iter().collect();
        let (kept, duplicates) = dedupe_lines(&lines, &existing);
        let mut ids: Vec<&str> = kept.iter().map(|l| l.group_id.as_str()).collect();
        ids.dedup();
        assert_eq!(ids, ["0TESTDUP", "0TESTNEW"]);
        // La seconde 0TESTDUP (brut + frais) et 0TESTOLD (brut seul).
        assert_eq!(duplicates, 3);
        // Les frais seuls déjà en base suffisent à écarter la ligne.
        let existing: HashSet<String> = ["0TESTNEW:FEE".to_string()].into_iter().collect();
        let lignes = provider_lines_of_row(&rows[2]);
        let (kept, duplicates) = dedupe_lines(&lignes, &existing);
        assert!(kept.is_empty());
        assert_eq!(duplicates, 2);
    }

    #[test]
    fn libelles_et_contreparties_bornes() {
        let mut r = row("0TESTLONG", "Website Payment", "EUR", "10.00", "0.00");
        r.note = Some("é".repeat(600));
        r.name = Some("n".repeat(300));
        let t = &transactions_of_row(&r, 1, 1)[0];
        assert_eq!(t.details.chars().count(), DETAILS_MAX_CHARS);
        assert_eq!(
            t.counterparty_name.as_ref().unwrap().chars().count(),
            NAME_MAX_CHARS
        );
    }

    /// Revue P1 (L2) — un identifiant de 251 caractères passe, et ses frais
    /// (`:FEE`) comme sa référence (`TXN `) tiennent dans 255 ; à 252, la ligne
    /// est refusée en erreur de ligne `transaction_id`, valeur tronquée.
    #[test]
    fn identifiant_de_transaction_borne_a_la_colonne() {
        let max = "A".repeat(PAYPAL_TRANSACTION_ID_MAX_CHARS);
        let ok = row(&max, "Website Payment", "EUR", "10.00", "-1.00");
        assert!(check_transaction_id_lengths(std::slice::from_ref(&ok)).is_ok());
        for t in transactions_of_row(&ok, 1, 1) {
            assert!(t.transaction_id.unwrap().chars().count() <= TXID_COLUMN_MAX_CHARS);
            assert!(t.reference.unwrap().chars().count() <= TXID_COLUMN_MAX_CHARS);
        }
        let mut long = ok.clone();
        long.transaction_id.push('B');
        long.line = 7;
        match check_transaction_id_lengths(&[ok, long]) {
            Err(PaypalCsvError::Lines(lines)) => {
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0].line, 7);
                assert_eq!(lines[0].field, Some(COL_TRANSACTION_ID));
                assert_eq!(lines[0].code, PaypalLineErrorCode::TransactionId);
                assert!(lines[0].value.as_deref().unwrap().ends_with('…'));
            }
            other => panic!("refus attendu : {other:?}"),
        }
    }
}
