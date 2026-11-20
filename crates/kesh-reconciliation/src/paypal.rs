//! Paramétrage **PayPal** de la revue et de l'acceptation communes — Story
//! 38-11c2 (AC-4).
//!
//! Fonctions pures : le classement d'une transaction PayPal ([`classify`]),
//! l'extraction des identifiants `TXN …` d'un libellé ([`txn_ids_in`]), les
//! libellés des écritures ([`entry_label`]) et le paramétrage [`SPEC`] que la
//! route PayPal passe au moteur commun ([`crate::provider_review`]).

use kesh_db::entities::BankTransaction;
use rust_decimal::Decimal;

use crate::provider_review::{
    EntryKind, Placement, ProviderKind, ProviderSpec, REVIEW_CONVERSION_UNPAIRED,
};

/// Valeur de `bank_accounts.provider` d'un compte PayPal.
pub const PROVIDER: &str = "paypal";

/// Suffixe de l'identifiant de la transaction des **frais** d'une ligne
/// PayPal : `<id>:FEE` (Story 38-11b). Un identifiant PayPal n'a jamais de
/// `:` (38-11a, AC-3), si bien que le suffixe ne peut heurter un identifiant
/// réel. L'import (`kesh-api`, `routes::paypal_imports`) le reprend d'ici.
pub const PAYPAL_FEE_SUFFIX: &str = ":FEE";

/// Types PayPal d'un **encaissement** de vente (montant > 0). Liste fermée,
/// comparée sans casse.
pub const SALE_RECEIPT_TYPES: [&str; 6] = [
    "Website Payment",
    "Subscription Payment",
    "Express Checkout Payment",
    "Mobile Payment",
    "Shopping Cart Payment Received",
    "Payment Received",
];

/// Types PayPal d'une **conversion** de devise (chaque moitié). Liste fermée,
/// comparée sans casse.
pub const CONVERSION_TYPES: [&str; 2] = [
    "General Currency Conversion",
    "User Initiated Currency Conversion",
];

/// Types PayPal d'un **retrait** (montant < 0). Liste fermée, comparée sans
/// casse.
pub const WITHDRAWAL_TYPES: [&str; 3] = [
    "General Withdrawal",
    "Withdraw Funds to Bank Account",
    "User Initiated Withdrawal",
];

/// Le type est-il dans la liste (sans casse, blancs de bord ignorés) ?
fn type_in(kind: &str, list: &[&str]) -> bool {
    let kind = kind.trim();
    list.iter().any(|t| t.eq_ignore_ascii_case(kind))
}

/// Le genre d'une transaction PayPal (AC-4), sur le suffixe `:FEE` de
/// `transaction_id`, le `provider_type` et le signe, dans cet ordre :
///
/// - `Fee` — suffixe `:FEE`, **quel que soit le signe** ;
/// - `SaleReceipt` — montant > 0 et type de [`SALE_RECEIPT_TYPES`] ;
/// - `Conversion` — type de [`CONVERSION_TYPES`] ;
/// - `Withdrawal` — montant < 0 et type de [`WITHDRAWAL_TYPES`] ;
/// - `Other` — le reste ; un type inconnu y tombe, jamais d'erreur.
pub fn classify(t: &BankTransaction) -> ProviderKind {
    if t.transaction_id
        .as_deref()
        .is_some_and(|id| id.ends_with(PAYPAL_FEE_SUFFIX))
    {
        return ProviderKind::Fee;
    }
    let kind = t.provider_type.as_deref().unwrap_or("");
    if t.amount > Decimal::ZERO && type_in(kind, &SALE_RECEIPT_TYPES) {
        ProviderKind::SaleReceipt
    } else if type_in(kind, &CONVERSION_TYPES) {
        ProviderKind::Conversion
    } else if t.amount < Decimal::ZERO && type_in(kind, &WITHDRAWAL_TYPES) {
        ProviderKind::Withdrawal
    } else {
        ProviderKind::Other
    }
}

/// Mots-clés qui annoncent un identifiant PayPal dans un libellé.
const KEYWORDS: [&str; 2] = ["txn", "transaction"];
/// Longueur d'un identifiant extrait, en caractères.
const ID_MIN: usize = 10;
const ID_MAX: usize = 20;

/// Séparateur admis entre le mot-clé et l'identifiant.
fn is_separator(c: char) -> bool {
    matches!(c, ' ' | ':' | '#' | '-')
}

/// Les identifiants PayPal d'un libellé (AC-4) : ceux qui suivent `TXN` ou
/// `transaction` (sans casse, mot entier), séparés par au moins un espace,
/// `:`, `#` ou `-` ; `[0-9A-Z]{10,20}` sans casse, en mot entier ; rendus en
/// majuscules, sans doublon, dans l'ordre du libellé.
pub fn txn_ids_in(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < n {
        if i > 0 && chars[i - 1].is_alphanumeric() {
            i += 1;
            continue;
        }
        let Some(len) = KEYWORDS.iter().find_map(|k| {
            let kc: Vec<char> = k.chars().collect();
            let end = i + kc.len();
            (end <= n
                && chars[i..end]
                    .iter()
                    .zip(&kc)
                    .all(|(a, b)| a.to_ascii_lowercase() == *b))
            .then_some(kc.len())
        }) else {
            i += 1;
            continue;
        };
        let after_keyword = i + len;
        if after_keyword < n && chars[after_keyword].is_alphanumeric() {
            i += 1;
            continue;
        }
        let mut start = after_keyword;
        while start < n && is_separator(chars[start]) {
            start += 1;
        }
        if start == after_keyword {
            i = after_keyword;
            continue;
        }
        let mut end = start;
        while end < n && chars[end].is_ascii_alphanumeric() {
            end += 1;
        }
        let whole_word = end == n || !chars[end].is_alphanumeric();
        let len_ok = (ID_MIN..=ID_MAX).contains(&(end - start));
        if whole_word && len_ok {
            let id: String = chars[start..end]
                .iter()
                .map(|c| c.to_ascii_uppercase())
                .collect();
            if !out.contains(&id) {
                out.push(id);
            }
        }
        i = end.max(after_keyword);
    }
    out
}

/// Clé de rattachement d'une transaction PayPal : son `transaction_id`.
fn match_key(t: &BankTransaction) -> Option<&str> {
    t.transaction_id.as_deref()
}

/// Appariement des deux moitiés d'une conversion PayPal, par le `Reference
/// Txn ID` (rangé dans `end_to_end_id` par l'import, 38-11b) : **même**
/// référence non vide (la forme des relevés de 2024, AC-5), ou chacune
/// pointant l'identifiant de l'autre (la forme du fichier fictif de 38-11a).
pub fn conversion_match(a: &BankTransaction, b: &BankTransaction) -> bool {
    fn reference(t: &BankTransaction) -> Option<&str> {
        t.end_to_end_id
            .as_deref()
            .map(str::trim)
            .filter(|r| !r.is_empty())
    }
    fn id(t: &BankTransaction) -> Option<&str> {
        t.transaction_id.as_deref().map(str::trim)
    }
    match (reference(a), reference(b)) {
        (Some(ra), Some(rb)) if ra == rb => true,
        (Some(ra), Some(rb)) => id(b) == Some(ra) && id(a) == Some(rb),
        _ => false,
    }
}

/// L'identifiant PayPal d'une transaction, sans le suffixe des frais.
fn paypal_id(t: &BankTransaction) -> &str {
    let id = t.transaction_id.as_deref().unwrap_or("");
    id.strip_suffix(PAYPAL_FEE_SUFFIX).unwrap_or(id)
}

/// Libellé d'une écriture PayPal (AC-8) : « Frais PayPal <devise> · TXN
/// <id> » pour des frais, « PayPal <compte> · <type> · TXN <id> » sinon.
pub fn entry_label(kind: EntryKind, account: &str, t: &BankTransaction) -> String {
    match kind {
        EntryKind::Fee => format!("Frais PayPal {} · TXN {}", t.currency, paypal_id(t)),
        _ => format!(
            "PayPal {account} · {} · TXN {}",
            t.provider_type.as_deref().unwrap_or("").trim(),
            paypal_id(t)
        ),
    }
}

/// Le paramétrage PayPal de la revue et de l'acceptation communes.
pub const SPEC: ProviderSpec = ProviderSpec {
    provider: PROVIDER,
    classify,
    match_key,
    extract_ids: txn_ids_in,
    conversion_match,
    conversion_unpaired: (REVIEW_CONVERSION_UNPAIRED, Placement::ToReview),
    entry_label,
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use kesh_db::entities::bank_transaction::BankTransactionStatus;
    use std::str::FromStr;

    fn tx(amount: &str, kind: &str, id: &str) -> BankTransaction {
        let now = chrono::Utc::now().naive_utc();
        BankTransaction {
            id: 1,
            company_id: 1,
            import_id: 1,
            bank_account_id: 1,
            booking_date: NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
            value_date: None,
            amount: Decimal::from_str(amount).unwrap(),
            currency: "EUR".into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            transaction_id: Some(id.into()),
            counterparty_iban: None,
            counterparty_name: None,
            provider_type: Some(kind.into()),
            counterparty_email: None,
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn frais_quel_que_soit_le_signe_et_le_type() {
        assert_eq!(
            classify(&tx("-1.20", "Website Payment", "0A:FEE")),
            ProviderKind::Fee
        );
        assert_eq!(
            classify(&tx("0.40", "Payment Refund", "0A:FEE")),
            ProviderKind::Fee
        );
    }

    #[test]
    fn chaque_type_d_encaissement() {
        for kind in SALE_RECEIPT_TYPES {
            assert_eq!(
                classify(&tx("10", kind, "0A")),
                ProviderKind::SaleReceipt,
                "{kind}"
            );
            assert_eq!(
                classify(&tx("10", &kind.to_uppercase(), "0A")),
                ProviderKind::SaleReceipt,
                "{kind} sans casse"
            );
            // Négatif : un achat, pas un encaissement.
            assert_eq!(
                classify(&tx("-10", kind, "0A")),
                ProviderKind::Other,
                "{kind} négatif"
            );
        }
    }

    #[test]
    fn chaque_type_de_conversion_et_de_retrait() {
        for kind in CONVERSION_TYPES {
            assert_eq!(classify(&tx("-10", kind, "0A")), ProviderKind::Conversion);
            assert_eq!(classify(&tx("10", kind, "0A")), ProviderKind::Conversion);
        }
        for kind in WITHDRAWAL_TYPES {
            assert_eq!(
                classify(&tx("-10", kind, "0A")),
                ProviderKind::Withdrawal,
                "{kind}"
            );
            assert_eq!(
                classify(&tx("10", kind, "0A")),
                ProviderKind::Other,
                "{kind} positif"
            );
        }
    }

    #[test]
    fn type_inconnu_ou_absent_en_autre() {
        assert_eq!(
            classify(&tx("-10", "Payment Refund", "0A")),
            ProviderKind::Other
        );
        assert_eq!(
            classify(&tx("10", "Truc inconnu", "0A")),
            ProviderKind::Other
        );
        let mut t = tx("10", "", "0A");
        t.provider_type = None;
        assert_eq!(classify(&t), ProviderKind::Other);
    }

    #[test]
    fn identifiants_d_un_libelle() {
        assert_eq!(
            txn_ids_in("Vente boutique TXN 0TEST0000000000AB"),
            ["0TEST0000000000AB"]
        );
        assert_eq!(
            txn_ids_in("transaction: 0test0000000000ab"),
            ["0TEST0000000000AB"]
        );
        assert_eq!(txn_ids_in("txn#1234567890 payée"), ["1234567890"]);
        assert_eq!(
            txn_ids_in("Commande 42 (TXN-ABCDEFGHIJ) livrée"),
            ["ABCDEFGHIJ"]
        );
        assert_eq!(
            txn_ids_in("TXN 1111111111 et transaction 2222222222, TXN 1111111111"),
            ["1111111111", "2222222222"]
        );
    }

    #[test]
    fn faux_positifs_ecartes() {
        assert!(txn_ids_in("TXN").is_empty());
        assert!(txn_ids_in("TXN seul").is_empty());
        // Trop court, trop long, collé au mot-clé, mot-clé dans un mot.
        assert!(txn_ids_in("TXN 123456789").is_empty());
        assert!(txn_ids_in("TXN 123456789012345678901").is_empty());
        assert!(txn_ids_in("TXN1234567890").is_empty());
        assert!(txn_ids_in("XTXN 1234567890").is_empty());
        assert!(txn_ids_in("transactions 1234567890").is_empty());
        // Identifiant suivi d'une lettre accentuée : pas un mot entier.
        assert!(txn_ids_in("TXN 1234567890é").is_empty());
        assert!(txn_ids_in("").is_empty());
    }

    #[test]
    fn conversion_appariee_par_reference_commune_ou_croisee() {
        let mut a = tx("-40", "User Initiated Currency Conversion", "0A4D");
        let mut b = tx("44.12", "User Initiated Currency Conversion", "0A5E");
        // Croisée : chacune pointe l'autre (fichier de 38-11a).
        a.end_to_end_id = Some("0A5E".into());
        b.end_to_end_id = Some("0A4D".into());
        assert!(conversion_match(&a, &b));
        // Commune : même référence (relevés de 2024).
        a.end_to_end_id = Some("0PAY".into());
        b.end_to_end_id = Some("0PAY".into());
        assert!(conversion_match(&a, &b));
        // Différentes, sans croisement ; vide ; absente.
        b.end_to_end_id = Some("0AUTRE".into());
        assert!(!conversion_match(&a, &b));
        a.end_to_end_id = Some("  ".into());
        b.end_to_end_id = Some("  ".into());
        assert!(!conversion_match(&a, &b));
        a.end_to_end_id = None;
        b.end_to_end_id = None;
        assert!(!conversion_match(&a, &b));
    }

    #[test]
    fn libelles_des_ecritures() {
        let fee = tx("-1.20", "Website Payment", "0ABC:FEE");
        assert_eq!(
            entry_label(EntryKind::Fee, "PayPal www", &fee),
            "Frais PayPal EUR · TXN 0ABC"
        );
        let rule = tx("-20", "Express Checkout Payment", "0XYZ");
        assert_eq!(
            entry_label(EntryKind::Rule, "www", &rule),
            "PayPal www · Express Checkout Payment · TXN 0XYZ"
        );
    }
}
