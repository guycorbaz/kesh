//! Lecture d'une page de l'API PayPal « Transaction Search » et conversion vers
//! le modèle du CSV (Story 38-12a, Epic 38 phase 2).
//!
//! `GET /v1/reporting/transactions` rend, par page, des transactions au schéma
//! `transaction_detail` de PayPal. Ce module les lit ([`parse_transactions_page`])
//! et les convertit en [`PaypalRow`] ([`to_paypal_row`]) — la forme que rend le
//! lecteur du CSV (38-11a) —, si bien que la ventilation, les doublons et le
//! rapprochement (38-11b, 38-11c1, 38-11c2) s'appliquent sans changement.
//!
//! Les contrôles d'une ligne sont **ceux du CSV** (devise, identifiant, montant,
//! fonctions partagées de [`crate::paypal`]) ; les erreurs ont la forme de
//! [`PaypalLineError`], le champ étant le nom du champ de l'API.
//!
//! Relevés du 2026-11-20 sur un compte réel (appels en lecture seule, métadonnées
//! seules, aucune donnée personnelle conservée) :
//!
//! - avec l'en-tête `PayPal-Enforce-ISO8601-Format: true`, la date d'une
//!   transaction arrive **en UTC** (`+00:00`) et non dans le fuseau du compte
//!   que dit le schéma : la conversion la ramène à l'heure locale que lui donne
//!   l'appelant (Zurich, côté `kesh-api`), celle du CSV ;
//! - les frais sont **négatifs** (encaissements, et paiements `T0000` qui en
//!   portent) ou **absents** ; aucun frais positif n'a été observé (aucun
//!   remboursement avec frais sur la période) — le signe est admis tel quel ;
//! - `payer_info` d'un **débit** désigne la **contrepartie** (adresses variées
//!   d'un débit à l'autre), non le titulaire du compte ;
//! - une fenêtre sans transaction rend `total_pages = 0` et aucun champ
//!   `transaction_details` ;
//! - `paypal_reference_id_type` vaut aussi `RP` (paiements d'abonnement), absent
//!   de la liste du schéma ; la référence est reprise telle quelle.
//!
//! Fonction pure, sans I/O, sans dépendance interne au workspace.

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::csv::parse_api_amount;
use crate::paypal::{
    BalanceImpact, PaypalLineError, PaypalLineErrorCode, PaypalRow, normalize_currency,
    normalize_transaction_id,
};

/// Champs de l'API nommés par les erreurs de ligne.
pub const FIELD_TRANSACTION_ID: &str = "transaction_info.transaction_id";
/// Date d'initiation de la transaction.
pub const FIELD_DATE: &str = "transaction_info.transaction_initiation_date";
/// Code d'événement (type de transaction).
pub const FIELD_EVENT_CODE: &str = "transaction_info.transaction_event_code";
/// Statut (`S`, `P`, `D`, `V`).
pub const FIELD_STATUS: &str = "transaction_info.transaction_status";
/// Devise du montant.
pub const FIELD_CURRENCY: &str = "transaction_info.transaction_amount.currency_code";
/// Valeur du montant brut.
pub const FIELD_GROSS: &str = "transaction_info.transaction_amount.value";
/// Devise des frais.
pub const FIELD_FEE_CURRENCY: &str = "transaction_info.fee_amount.currency_code";
/// Valeur des frais.
pub const FIELD_FEE: &str = "transaction_info.fee_amount.value";

/// Code d'événement PayPal → libellé du CSV (colonne `Type`), table **fermée**.
///
/// Les libellés sont ceux que lit la phase 1 (listes de classement de 38-11c2 et
/// `PAYPAL_NEUTRAL_PAIR_TYPES` de 38-11b, côté `kesh-api` ; le test de couverture y vit). État de vérification :
///
/// - **vérifiés** (✔) par concordance sur l'année 2024 d'un compte réel : les
///   codes que rend l'API (341 transactions : `T0000`, `T0002`, `T0003`,
///   `T0006`, `T0007`, `T0201`, `T1107`, `T1110`, `T1111`, `T2105`, `T2106`),
///   rapprochés des types relevés au CSV de la même année (341 lignes, même
///   ventilation des statuts : 339 `Completed`, un `Pending`, un `Denied`) et
///   du sens de chaque type (`Credit`/`Debit`, frais). La jointure par
///   identifiant de transaction n'a pas pu se faire : le CSV de 2024 a été
///   supprimé avant la phase 2 ;
/// - **non vérifiés** (doc) : libellé tiré de la page publique des codes
///   d'événement de PayPal, mis à la casse des libellés du CSV ; aucun n'est
///   paru dans les données réelles consultées.
///
/// Un code absent de la table garde son code pour libellé (`T9999`) : la ligne
/// est importée et tombe « à examiner », jamais ignorée.
pub const PAYPAL_EVENT_TYPES: &[(&str, &str)] = &[
    ("T0000", "General Payment"),      // ✔ débits avec frais
    ("T0001", "Mass Payment"),         // doc
    ("T0002", "Subscription Payment"), // ✔ crédits, référence RP
    ("T0003", "PreApproved Payment Bill User Payment"), // ✔ débit, référence PAP
    ("T0006", "Express Checkout Payment"), // ✔ débits sans frais
    ("T0007", "Website Payment"),      // ✔ crédits avec frais
    ("T0011", "Mobile Payment"),       // doc
    ("T0013", "Donation Payment"),     // doc
    ("T0200", "General Currency Conversion"), // doc
    ("T0201", "User Initiated Currency Conversion"), // ✔ paires débit/crédit
    ("T0400", "General Withdrawal"),   // doc
    ("T0403", "User Initiated Withdrawal"), // doc (« Manual withdrawal »)
    ("T1105", "Reversal of General Account Hold"), // doc
    ("T1106", "Payment Reversal"),     // doc
    ("T1107", "Payment Refund"),       // ✔ débit
    ("T1110", "Hold on Balance for Dispute Investigation"), // ✔ débit, statut P
    ("T1111", "Cancellation of Hold for Dispute Resolution"), // ✔ crédit, statut D
    ("T1500", "General Account Hold"), // doc
    ("T1501", "Account Hold for Open Authorization"), // doc
    ("T1502", "Account Hold for ACH Deposit"), // doc
    ("T1503", "Hold on Available Balance"), // doc
    ("T2105", "Payment Review Hold"),  // ✔ débit sans frais
    ("T2106", "Payment Review Release"), // ✔ crédit sans frais
];

/// Libellés du CSV que lit la phase 1 et qu'**aucun** code de
/// [`PAYPAL_EVENT_TYPES`] ne rend — inventaire **fermé** des non-résolus,
/// chacun avec sa raison. Le test de couverture exige que tout libellé lu par
/// la phase 1 soit l'image d'un code **ou** figure ici.
pub const PAYPAL_CSV_LABELS_WITHOUT_CODE: &[(&str, &str)] = &[
    (
        "Shopping Cart Payment Received",
        "ancien libellé d'encaissement du CSV ; l'API rend ces ventes en T0007 (Website Payment), de la même liste",
    ),
    (
        "Payment Received",
        "ancien libellé d'encaissement du CSV ; aucun code d'événement de PayPal ne le porte",
    ),
    (
        "Withdraw Funds to Bank Account",
        "synonyme du CSV de General Withdrawal (T0400), de la même liste",
    ),
    (
        "Reversal of Account Hold",
        "aucun code de la page de PayPal ne le porte ; sa paire (Account Hold for Open Authorization, T1501) reste importée sans être neutralisée",
    ),
];

/// Le libellé du CSV d'un code d'événement ; le code lui-même s'il est inconnu.
pub fn event_label(code: &str) -> String {
    PAYPAL_EVENT_TYPES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or_else(|| code.to_string(), |(_, label)| (*label).to_string())
}

/// Un montant de l'API (`money`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PaypalApiMoney {
    /// Code ISO 4217.
    pub currency_code: Option<String>,
    /// Valeur décimale à point.
    pub value: Option<String>,
}

/// `transaction_info` : les champs lus.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PaypalApiTransactionInfo {
    /// Identifiant de la transaction.
    pub transaction_id: Option<String>,
    /// Identifiant de la transaction liée.
    pub paypal_reference_id: Option<String>,
    /// Code d'événement (`T0007`…).
    pub transaction_event_code: Option<String>,
    /// Date d'initiation (RFC 3339).
    pub transaction_initiation_date: Option<String>,
    /// Montant brut.
    pub transaction_amount: Option<PaypalApiMoney>,
    /// Frais (absents s'il n'y en a pas).
    pub fee_amount: Option<PaypalApiMoney>,
    /// Statut (`S`, `P`, `D`, `V`).
    pub transaction_status: Option<String>,
    /// Objet.
    pub transaction_subject: Option<String>,
    /// Note.
    pub transaction_note: Option<String>,
    /// Numéro de facture du marchand.
    pub invoice_id: Option<String>,
}

/// Nom d'une personne (`name`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PaypalApiName {
    /// Nom complet.
    pub full_name: Option<String>,
    /// Autre nom complet (celui que l'API rend en pratique).
    pub alternate_full_name: Option<String>,
    /// Prénom.
    pub given_name: Option<String>,
    /// Nom de famille.
    pub surname: Option<String>,
}

/// `payer_info` : la contrepartie.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PaypalApiPayerInfo {
    /// Adresse électronique.
    pub email_address: Option<String>,
    /// Nom.
    pub payer_name: Option<PaypalApiName>,
}

/// Un article du panier.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PaypalApiItem {
    /// Nom de l'article.
    pub item_name: Option<String>,
}

/// `cart_info`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PaypalApiCartInfo {
    /// Articles.
    pub item_details: Option<Vec<PaypalApiItem>>,
    /// Numéro de facture PayPal.
    pub paypal_invoice_id: Option<String>,
}

/// Une transaction de l'API (`transaction_detail`), champs lus seulement.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PaypalApiTx {
    /// Informations de la transaction.
    #[serde(default)]
    pub transaction_info: PaypalApiTransactionInfo,
    /// Contrepartie.
    pub payer_info: Option<PaypalApiPayerInfo>,
    /// Panier.
    pub cart_info: Option<PaypalApiCartInfo>,
}

/// Une page de réponse (`search_response`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaypalApiPage {
    /// Transactions de la page (aucune pour une fenêtre vide).
    pub transactions: Vec<PaypalApiTx>,
    /// Nombre de pages (0 pour une fenêtre vide).
    pub total_pages: u32,
    /// « Données à jour jusqu'à » (RFC 3339), tel que donné.
    pub last_refreshed_datetime: Option<String>,
}

/// Refus d'une page : le corps n'a pas la forme attendue.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PaypalApiError {
    /// Corps qui n'est pas du JSON, ou d'une autre forme que `search_response`.
    #[error("réponse PayPal illisible")]
    Malformed,
    /// `total_pages` absent.
    #[error("réponse PayPal sans total_pages")]
    MissingTotalPages,
}

#[derive(Deserialize)]
struct RawPage {
    transaction_details: Option<Vec<PaypalApiTx>>,
    total_pages: Option<u32>,
    last_refreshed_datetime: Option<String>,
}

/// Lit une page de `GET /v1/reporting/transactions`. Une fenêtre vide
/// (`total_pages = 0`, pas de `transaction_details`) rend une page vide.
pub fn parse_transactions_page(bytes: &[u8]) -> Result<PaypalApiPage, PaypalApiError> {
    let raw: RawPage = serde_json::from_slice(bytes).map_err(|_| PaypalApiError::Malformed)?;
    Ok(PaypalApiPage {
        transactions: raw.transaction_details.unwrap_or_default(),
        total_pages: raw.total_pages.ok_or(PaypalApiError::MissingTotalPages)?,
        last_refreshed_datetime: raw.last_refreshed_datetime,
    })
}

/// Texte non vide après `trim`, ou `None`.
fn non_empty(v: &Option<String>) -> Option<String> {
    v.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Le nom de la contrepartie : `full_name`, sinon `alternate_full_name`,
/// sinon `given_name` et `surname` joints ; aucun → `None`.
fn payer_name(tx: &PaypalApiTx) -> Option<String> {
    let name = tx.payer_info.as_ref()?.payer_name.as_ref()?;
    non_empty(&name.full_name)
        .or_else(|| non_empty(&name.alternate_full_name))
        .or_else(|| {
            let parts: Vec<String> = [non_empty(&name.given_name), non_empty(&name.surname)]
                .into_iter()
                .flatten()
                .collect();
            (!parts.is_empty()).then(|| parts.join(" "))
        })
}

/// Convertit une transaction de l'API en [`PaypalRow`] — Story 38-12a (AC-5).
///
/// `line` est le rang de la transaction dans le téléchargement (à partir de 1),
/// pour les messages. `local` ramène l'instant de la transaction à l'heure
/// locale du compte (Zurich côté `kesh-api`) : la date et l'heure de la ligne
/// sont celles de cette heure locale, comme au CSV, et `time_zone` porte son
/// décalage (`+01:00`), là où le CSV écrit un nom (`CET`).
///
/// Rend toutes les erreurs de la ligne (au plus une par champ), à la manière du
/// lecteur du CSV.
pub fn to_paypal_row(
    tx: &PaypalApiTx,
    line: usize,
    local: &dyn Fn(DateTime<FixedOffset>) -> DateTime<FixedOffset>,
) -> Result<PaypalRow, Vec<PaypalLineError>> {
    let info = &tx.transaction_info;
    let mut errors = Vec::new();
    let mut push = |field, code, value: Option<&str>| {
        errors.push(PaypalLineError::new(line, Some(field), code, value));
    };

    let raw_id = info.transaction_id.as_deref();
    let transaction_id = raw_id.and_then(normalize_transaction_id);
    if transaction_id.is_none() {
        push(
            FIELD_TRANSACTION_ID,
            PaypalLineErrorCode::TransactionId,
            raw_id,
        );
    }

    let raw_date = info.transaction_initiation_date.as_deref();
    // L'instant ramené à l'heure locale, et sa date (celle du CSV), dans la
    // plage des années de la base.
    let instant = raw_date
        .and_then(|d| DateTime::parse_from_rfc3339(d.trim()).ok())
        .map(local)
        .map(|t| (t, t.date_naive()))
        .filter(|(_, date)| crate::annee_recevable(*date));
    if instant.is_none() {
        push(FIELD_DATE, PaypalLineErrorCode::Date, raw_date);
    }

    let kind = non_empty(&info.transaction_event_code).map(|c| event_label(&c));
    if kind.is_none() {
        push(FIELD_EVENT_CODE, PaypalLineErrorCode::Required, None);
    }

    let raw_status = info.transaction_status.as_deref();
    let status = match raw_status.map(str::trim) {
        Some("S") => Some("Completed"),
        Some("P") => Some("Pending"),
        Some("D") => Some("Denied"),
        Some("V") => Some("Reversed"),
        _ => None,
    };
    if status.is_none() {
        push(FIELD_STATUS, PaypalLineErrorCode::Status, raw_status);
    }

    let amount = info.transaction_amount.clone().unwrap_or_default();
    let raw_currency = amount.currency_code.as_deref();
    let currency = raw_currency.and_then(|c| normalize_currency(c.trim()));
    if currency.is_none() {
        push(FIELD_CURRENCY, PaypalLineErrorCode::Currency, raw_currency);
    }
    let raw_gross = amount.value.as_deref();
    let gross = raw_gross.and_then(parse_api_amount);
    if gross.is_none() {
        push(FIELD_GROSS, PaypalLineErrorCode::Amount, raw_gross);
    }

    // Frais : absents → 0 ; présents, de tout signe (positifs sur certains
    // remboursements, admis par 38-11a et ventilés par 38-11b), dans la devise
    // du montant.
    let fee = match &info.fee_amount {
        None => Some(Decimal::ZERO),
        Some(fee) => {
            let raw_fee = fee.value.as_deref();
            let value = raw_fee.and_then(parse_api_amount);
            if value.is_none() {
                push(FIELD_FEE, PaypalLineErrorCode::Amount, raw_fee);
            }
            let fee_currency = fee
                .currency_code
                .as_deref()
                .and_then(|c| normalize_currency(c.trim()));
            if fee_currency.is_some() && currency.is_some() && fee_currency != currency {
                push(
                    FIELD_FEE_CURRENCY,
                    PaypalLineErrorCode::Currency,
                    fee.currency_code.as_deref(),
                );
                None
            } else {
                value
            }
        }
    };

    if !errors.is_empty() {
        return Err(errors);
    }
    // Tous présents : chaque `None` a poussé son erreur ci-dessus.
    let (
        Some(transaction_id),
        Some((instant, local_date)),
        Some(kind),
        Some(status),
        Some(currency),
    ) = (transaction_id, instant, kind, status, currency)
    else {
        return Err(errors);
    };
    let (Some(gross), Some(fee)) = (gross, fee) else {
        return Err(errors);
    };

    // La demande n'admet que les lignes qui affectent le solde : pas de `Memo`.
    let balance_impact = if gross.is_sign_negative() && !gross.is_zero() {
        BalanceImpact::Debit
    } else {
        BalanceImpact::Credit
    };
    let email = tx
        .payer_info
        .as_ref()
        .and_then(|p| non_empty(&p.email_address));
    let (from_email, to_email) = match balance_impact {
        BalanceImpact::Debit => (None, email),
        _ => (email, None),
    };
    let item_title = tx
        .cart_info
        .as_ref()
        .and_then(|c| c.item_details.as_ref())
        .map(|items| {
            items
                .iter()
                .filter_map(|i| non_empty(&i.item_name))
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .filter(|s| !s.is_empty());
    let invoice_number = non_empty(&info.invoice_id).or_else(|| {
        tx.cart_info
            .as_ref()
            .and_then(|c| non_empty(&c.paypal_invoice_id))
    });

    Ok(PaypalRow {
        line,
        date: local_date,
        time: Some(instant.format("%H:%M:%S").to_string()),
        time_zone: Some(instant.offset().to_string()),
        name: payer_name(tx),
        kind,
        status: status.to_string(),
        currency,
        gross,
        fee,
        net: gross + fee,
        from_email,
        to_email,
        transaction_id,
        reference_txn_id: non_empty(&info.paypal_reference_id).map(|r| r.to_ascii_uppercase()),
        balance_impact,
        balance: None,
        item_title,
        subject: non_empty(&info.transaction_subject),
        note: non_empty(&info.transaction_note),
        invoice_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Offset};
    use rust_decimal_macros::dec;

    /// Heure locale fictive des tests : UTC+1 fixe.
    fn plus_one(t: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
        t.with_timezone(&FixedOffset::east_opt(3600).unwrap())
    }

    fn tx(json: &str) -> PaypalApiTx {
        serde_json::from_str(json).expect("transaction de test")
    }

    const SALE: &str = r#"{
        "transaction_info": {
            "transaction_id": "0test00000000001a",
            "paypal_reference_id": "0testref0000001",
            "transaction_event_code": "T0007",
            "transaction_initiation_date": "2024-01-31T23:30:00+00:00",
            "transaction_amount": { "currency_code": "eur", "value": "25.00" },
            "fee_amount": { "currency_code": "EUR", "value": "-1.20" },
            "transaction_status": "S",
            "transaction_subject": " Commande 1001 ",
            "transaction_note": "",
            "invoice_id": "INV-1"
        },
        "payer_info": {
            "email_address": "alice@example.invalid",
            "payer_name": { "alternate_full_name": "Alice Fictive", "given_name": "Alice", "surname": "Fictive" }
        },
        "cart_info": { "item_details": [ { "item_name": "Article A" }, { "item_name": "Article B" } ] }
    }"#;

    #[test]
    fn une_vente_se_convertit_champ_a_champ() {
        let r = to_paypal_row(&tx(SALE), 3, &plus_one).unwrap();
        assert_eq!(r.line, 3);
        // 23:30 UTC le 31 = 00:30 le 1er février à l'heure locale.
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(r.time.as_deref(), Some("00:30:00"));
        assert_eq!(r.time_zone.as_deref(), Some("+01:00"));
        assert_eq!(r.name.as_deref(), Some("Alice Fictive"));
        assert_eq!(r.kind, "Website Payment");
        assert_eq!(r.status, "Completed");
        assert_eq!(r.currency, "EUR");
        assert_eq!(
            (r.gross, r.fee, r.net),
            (dec!(25.00), dec!(-1.20), dec!(23.80))
        );
        assert_eq!(r.from_email.as_deref(), Some("alice@example.invalid"));
        assert_eq!(r.to_email, None);
        assert_eq!(r.transaction_id, "0TEST00000000001A");
        assert_eq!(r.reference_txn_id.as_deref(), Some("0TESTREF0000001"));
        assert_eq!(r.balance_impact, BalanceImpact::Credit);
        assert_eq!(r.balance, None);
        assert_eq!(r.item_title.as_deref(), Some("Article A · Article B"));
        assert_eq!(r.subject.as_deref(), Some("Commande 1001"));
        assert_eq!(r.note, None, "note vide");
        assert_eq!(r.invoice_number.as_deref(), Some("INV-1"));
    }

    #[test]
    fn un_debit_place_l_adresse_en_destinataire_et_n_a_pas_de_frais() {
        let t = tx(r#"{
            "transaction_info": {
                "transaction_id": "0TESTDEBIT01",
                "transaction_event_code": "T0006",
                "transaction_initiation_date": "2024-03-01T10:00:00Z",
                "transaction_amount": { "currency_code": "USD", "value": "-40.00" },
                "transaction_status": "S"
            },
            "payer_info": { "email_address": "shop@example.invalid",
                            "payer_name": { "given_name": "Bob", "surname": "Fictif" } },
            "cart_info": { "paypal_invoice_id": "PPINV-9" }
        }"#);
        let r = to_paypal_row(&t, 1, &plus_one).unwrap();
        assert_eq!(r.balance_impact, BalanceImpact::Debit);
        assert_eq!(r.from_email, None);
        assert_eq!(r.to_email.as_deref(), Some("shop@example.invalid"));
        assert_eq!(r.fee, Decimal::ZERO);
        assert_eq!(r.net, dec!(-40.00));
        assert_eq!(r.name.as_deref(), Some("Bob Fictif"));
        assert_eq!(r.invoice_number.as_deref(), Some("PPINV-9"));
        assert_eq!(r.kind, "Express Checkout Payment");
        assert_eq!(r.item_title, None);
    }

    #[test]
    fn des_frais_positifs_sont_admis() {
        let mut t = tx(SALE);
        t.transaction_info.transaction_amount = Some(PaypalApiMoney {
            currency_code: Some("EUR".into()),
            value: Some("-30.00".into()),
        });
        t.transaction_info.fee_amount = Some(PaypalApiMoney {
            currency_code: Some("EUR".into()),
            value: Some("0.96".into()),
        });
        t.transaction_info.transaction_event_code = Some("T1107".into());
        let r = to_paypal_row(&t, 1, &plus_one).unwrap();
        assert_eq!((r.fee, r.net), (dec!(0.96), dec!(-29.04)));
        assert_eq!(r.kind, "Payment Refund");
        assert_eq!(r.to_email.as_deref(), Some("alice@example.invalid"));
    }

    #[test]
    fn statuts_et_code_inconnu() {
        for (letter, label) in [
            ("S", "Completed"),
            ("P", "Pending"),
            ("D", "Denied"),
            ("V", "Reversed"),
        ] {
            let mut t = tx(SALE);
            t.transaction_info.transaction_status = Some(letter.into());
            assert_eq!(to_paypal_row(&t, 1, &plus_one).unwrap().status, label);
        }
        let mut t = tx(SALE);
        t.transaction_info.transaction_status = Some("X".into());
        let e = to_paypal_row(&t, 4, &plus_one).unwrap_err();
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].field, Some(FIELD_STATUS));
        assert_eq!(e[0].code, PaypalLineErrorCode::Status);
        assert_eq!(e[0].value.as_deref(), Some("X"));

        let mut t = tx(SALE);
        t.transaction_info.transaction_event_code = Some("T9999".into());
        assert_eq!(to_paypal_row(&t, 1, &plus_one).unwrap().kind, "T9999");
    }

    #[test]
    fn les_controles_du_csv_s_appliquent() {
        let mut t = tx(SALE);
        t.transaction_info.transaction_id = Some("0TEST_UNDERSCORE".into());
        t.transaction_info.transaction_amount = Some(PaypalApiMoney {
            currency_code: Some("EURO".into()),
            value: Some("25,00".into()),
        });
        t.transaction_info.transaction_initiation_date = Some("hier".into());
        t.transaction_info.transaction_event_code = None;
        let e = to_paypal_row(&t, 9, &plus_one).unwrap_err();
        let codes: Vec<(&str, PaypalLineErrorCode)> =
            e.iter().map(|x| (x.field.unwrap(), x.code)).collect();
        assert_eq!(
            codes,
            [
                (FIELD_TRANSACTION_ID, PaypalLineErrorCode::TransactionId),
                (FIELD_DATE, PaypalLineErrorCode::Date),
                (FIELD_EVENT_CODE, PaypalLineErrorCode::Required),
                (FIELD_CURRENCY, PaypalLineErrorCode::Currency),
                (FIELD_GROSS, PaypalLineErrorCode::Amount),
            ]
        );
        assert!(e.iter().all(|x| x.line == 9));
    }

    #[test]
    fn frais_d_une_autre_devise_ou_illisibles() {
        let mut t = tx(SALE);
        t.transaction_info.fee_amount = Some(PaypalApiMoney {
            currency_code: Some("USD".into()),
            value: Some("-1.00".into()),
        });
        let e = to_paypal_row(&t, 1, &plus_one).unwrap_err();
        assert_eq!(e[0].field, Some(FIELD_FEE_CURRENCY));
        let mut t = tx(SALE);
        t.transaction_info.fee_amount = Some(PaypalApiMoney {
            currency_code: Some("EUR".into()),
            value: Some("-1.234".into()),
        });
        let e = to_paypal_row(&t, 1, &plus_one).unwrap_err();
        assert_eq!(e[0].field, Some(FIELD_FEE));
        assert_eq!(e[0].code, PaypalLineErrorCode::Amount);
    }

    /// Revue P1, F13 — l'API n'écrit pas de virgule des milliers : un montant
    /// qui en porte est refusé (le CSV, lui, l'admet), brut comme frais.
    #[test]
    fn une_virgule_des_milliers_est_refusee_a_l_api() {
        let mut t = tx(SALE);
        t.transaction_info.transaction_amount = Some(PaypalApiMoney {
            currency_code: Some("EUR".into()),
            value: Some("1,234.00".into()),
        });
        let e = to_paypal_row(&t, 1, &plus_one).unwrap_err();
        assert_eq!(
            (e[0].field, e[0].code),
            (Some(FIELD_GROSS), PaypalLineErrorCode::Amount)
        );
        let mut t = tx(SALE);
        t.transaction_info.fee_amount = Some(PaypalApiMoney {
            currency_code: Some("EUR".into()),
            value: Some("-1,000".into()),
        });
        let e = to_paypal_row(&t, 1, &plus_one).unwrap_err();
        assert_eq!(
            (e[0].field, e[0].code),
            (Some(FIELD_FEE), PaypalLineErrorCode::Amount)
        );
    }

    #[test]
    fn le_nom_suit_l_ordre_ecrit() {
        let mut t = tx(SALE);
        t.payer_info.as_mut().unwrap().payer_name = Some(PaypalApiName {
            full_name: Some("Plein".into()),
            alternate_full_name: Some("Autre".into()),
            ..Default::default()
        });
        assert_eq!(payer_name(&t).as_deref(), Some("Plein"));
        t.payer_info.as_mut().unwrap().payer_name = Some(PaypalApiName::default());
        assert_eq!(payer_name(&t), None);
        t.payer_info = None;
        assert_eq!(payer_name(&t), None);
    }

    #[test]
    fn la_date_garde_l_heure_locale_donnee() {
        let utc = |t: DateTime<FixedOffset>| t.with_timezone(&chrono::Utc.fix());
        let mut t = tx(SALE);
        t.transaction_info.transaction_initiation_date = Some("2024-06-30T23:30:00+02:00".into());
        let r = to_paypal_row(&t, 1, &utc).unwrap();
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2024, 6, 30).unwrap());
        assert_eq!(r.time.as_deref(), Some("21:30:00"));
        assert_eq!(r.time_zone.as_deref(), Some("+00:00"));
    }

    #[test]
    fn une_page_se_lit_et_une_fenetre_vide_aussi() {
        let page = parse_transactions_page(
            format!(
                r#"{{"transaction_details":[{SALE}],"total_pages":2,"page":1,
                     "last_refreshed_datetime":"2024-02-01T08:00:00+00:00"}}"#
            )
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(page.transactions.len(), 1);
        assert_eq!(page.total_pages, 2);
        assert_eq!(
            page.last_refreshed_datetime.as_deref(),
            Some("2024-02-01T08:00:00+00:00")
        );
        // Forme relevée sur l'API réelle : fenêtre vide.
        let empty = parse_transactions_page(
            br#"{"start_date":"2025-01-01T00:00:00+00:00","page":1,"total_items":0,"total_pages":0,"links":[]}"#,
        )
        .unwrap();
        assert!(empty.transactions.is_empty());
        assert_eq!(empty.total_pages, 0);
        assert_eq!(
            parse_transactions_page(b"<html>"),
            Err(PaypalApiError::Malformed)
        );
        assert_eq!(
            parse_transactions_page(br#"{"transaction_details":[]}"#),
            Err(PaypalApiError::MissingTotalPages)
        );
    }

    /// Montants en **chaînes** JSON (forme de l'API, relevée sur le compte réel) :
    /// lus sans passer par un nombre flottant, au chiffre près, même à 17
    /// chiffres significatifs. Un montant en **nombre** JSON est refusé (page
    /// illisible), jamais arrondi en silence (`serde_json` sans
    /// `arbitrary_precision` perdrait des chiffres au-delà de 15).
    #[test]
    fn un_montant_a_dix_sept_chiffres_se_lit_exactement_et_un_nombre_json_est_refuse() {
        let page = br#"{"total_pages":1,"transaction_details":[{"transaction_info":{
            "transaction_id":"0TESTGRAND","transaction_event_code":"T0007",
            "transaction_initiation_date":"2024-01-02T10:00:00Z","transaction_status":"S",
            "transaction_amount":{"currency_code":"CHF","value":"999999999999999.99"},
            "fee_amount":{"currency_code":"CHF","value":"-0.01"}}}]}"#;
        let p = parse_transactions_page(page).unwrap();
        let r = to_paypal_row(&p.transactions[0], 1, &plus_one).unwrap();
        assert_eq!(r.gross.to_string(), "999999999999999.99");
        assert_eq!(r.net.to_string(), "999999999999999.98");
        let nombre = br#"{"total_pages":1,"transaction_details":[{"transaction_info":{
            "transaction_id":"0TESTGRAND",
            "transaction_amount":{"currency_code":"CHF","value":999999999999999.99}}}]}"#;
        assert_eq!(
            parse_transactions_page(nombre),
            Err(PaypalApiError::Malformed)
        );
    }

    #[test]
    fn la_table_est_fermee_et_sans_doublon() {
        let mut codes: Vec<&str> = PAYPAL_EVENT_TYPES.iter().map(|(c, _)| *c).collect();
        let n = codes.len();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), n, "code en double");
        assert!(
            PAYPAL_EVENT_TYPES
                .iter()
                .all(|(c, l)| c.len() == 5 && c.starts_with('T') && !l.is_empty())
        );
        // Un libellé sans code n'est pas aussi l'image d'un code.
        for (label, reason) in PAYPAL_CSV_LABELS_WITHOUT_CODE {
            assert!(!reason.is_empty());
            assert!(
                !PAYPAL_EVENT_TYPES.iter().any(|(_, l)| l == label),
                "{label} a un code : à retirer de l'inventaire"
            );
        }
        assert_eq!(event_label("T0201"), "User Initiated Currency Conversion");
    }
}
