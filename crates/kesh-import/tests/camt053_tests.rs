//! Tests d'intégration du parseur CAMT.053 (Story 8-1a, AC #5 et #9).
//!
//! Les 10 fixtures couvrent :
//! - v04 namespace par défaut + namespace préfixé (régression H4)
//! - v08 namespace
//! - Sous-transactions (FR49)
//! - Multi-statements (un fichier, plusieurs comptes)
//! - XML tronqué
//! - Namespace inconnu
//! - IBAN counterparty cassé conservé brut (§iban-tolerant)
//! - Devise EUR préservée par le parseur (rejet côté `kesh-core`)
//! - `CdtDbtInd` qui signe le montant

use kesh_import::{CamtError, SourceFormat, parse_camt053};
use rust_decimal_macros::dec;

const FIXTURES: &str = "tests/fixtures/camt053";

fn load(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("fixture {} introuvable : {e}", path.display()))
}

#[test]
fn parse_v04_minimal_extracts_all_transactions() {
    let xml = load("v04_minimal.xml");
    let stmts = parse_camt053(&xml).expect("parse OK");
    assert_eq!(stmts.len(), 1);

    let s = &stmts[0];
    assert_eq!(s.statement_id.as_deref(), Some("STMT-001"));
    assert_eq!(s.account_iban, "CH4431999123000889012");
    assert_eq!(s.currency, "CHF");
    assert_eq!(s.opening_balance, Some(dec!(10000.00)));
    assert_eq!(s.closing_balance, Some(dec!(11234.56)));
    assert_eq!(s.period_from.to_string(), "2026-05-01");
    assert_eq!(s.period_to.to_string(), "2026-05-31");
    assert!(matches!(
        &s.source_format,
        SourceFormat::Camt053 { version } if version == "001.04"
    ));

    assert_eq!(s.transactions.len(), 1);
    let tx = &s.transactions[0];
    assert_eq!(tx.amount, dec!(1234.56));
    assert_eq!(tx.currency, "CHF");
    assert_eq!(tx.booking_date.to_string(), "2026-05-15");
    assert_eq!(
        tx.value_date.map(|d| d.to_string()).as_deref(),
        Some("2026-05-15")
    );
    assert_eq!(tx.transaction_id.as_deref(), Some("BANK-TX-42"));
    assert_eq!(tx.end_to_end_id.as_deref(), Some("E2E-2026-05-001"));
    assert_eq!(tx.reference.as_deref(), Some("RF18539007547034"));
    assert_eq!(tx.counterparty_name.as_deref(), Some("Acme SA"));
    assert_eq!(
        tx.counterparty_iban.as_deref(),
        Some("CH9300762011623852957")
    );
}

#[test]
fn parse_v04_prefixed_namespace_extracts_all_transactions() {
    // Régression H4 : le préfixe `ns:` ne doit pas casser la résolution
    // namespace. La forme `<ns:Document xmlns:ns="...">` doit dispatcher
    // sur le parseur v04 exactement comme `<Document xmlns="...">`.
    let xml = load("v04_prefixed_namespace.xml");
    let stmts = parse_camt053(&xml).expect("parse OK avec namespace préfixé");
    assert_eq!(stmts.len(), 1);

    let s = &stmts[0];
    assert_eq!(s.statement_id.as_deref(), Some("STMT-PREFIXED"));
    assert_eq!(s.account_iban, "CH4431999123000889012");
    assert_eq!(s.transactions.len(), 1);

    let tx = &s.transactions[0];
    assert_eq!(tx.amount, dec!(500.00));
    assert_eq!(tx.counterparty_name.as_deref(), Some("Régie Genevoise SA"));
    assert!(matches!(
        &s.source_format,
        SourceFormat::Camt053 { version } if version == "001.04"
    ));
}

#[test]
fn parse_v08_minimal_extracts_all_transactions() {
    let xml = load("v08_minimal.xml");
    let stmts = parse_camt053(&xml).expect("parse OK");
    assert_eq!(stmts.len(), 1);

    let s = &stmts[0];
    assert_eq!(s.account_iban, "CH5604835012345678009");
    assert!(matches!(
        &s.source_format,
        SourceFormat::Camt053 { version } if version == "001.08"
    ));

    let tx = &s.transactions[0];
    assert_eq!(tx.amount, dec!(250.00));
    // Delta v04 → v08 : <Pty> wrapper autour de <Dbtr>
    assert_eq!(tx.counterparty_name.as_deref(), Some("Bobst SA"));
}

#[test]
fn parse_with_subtxs_extracts_individual_transactions() {
    // FR49 : 1 <Ntry> agrégée + 3 <TxDtls> ⇒ 3 ImportedTransaction.
    let xml = load("v04_with_subtxs.xml");
    let stmts = parse_camt053(&xml).expect("parse OK");
    assert_eq!(stmts.len(), 1);
    let s = &stmts[0];

    assert_eq!(s.transactions.len(), 3, "FR49 : une transaction par TxDtls");

    let amounts: Vec<_> = s.transactions.iter().map(|t| t.amount).collect();
    assert_eq!(amounts, vec![dec!(100.00), dec!(150.00), dec!(50.00)]);

    let names: Vec<_> = s
        .transactions
        .iter()
        .map(|t| t.counterparty_name.clone().unwrap_or_default())
        .collect();
    assert_eq!(names, vec!["Client A", "Client B", "Client C"]);

    let e2e: Vec<_> = s
        .transactions
        .iter()
        .map(|t| t.end_to_end_id.clone().unwrap_or_default())
        .collect();
    assert_eq!(e2e, vec!["E2E-SUB-001", "E2E-SUB-002", "E2E-SUB-003"]);

    let bank_refs: Vec<_> = s
        .transactions
        .iter()
        .map(|t| t.transaction_id.clone().unwrap_or_default())
        .collect();
    assert_eq!(
        bank_refs,
        vec!["BANK-SUB-001", "BANK-SUB-002", "BANK-SUB-003"]
    );
}

#[test]
fn parse_multi_stmt_returns_one_per_account() {
    let xml = load("v04_multi_stmt.xml");
    let stmts = parse_camt053(&xml).expect("parse OK");
    assert_eq!(stmts.len(), 2, "deux <Stmt> ⇒ deux ImportedStatement");

    assert_eq!(stmts[0].account_iban, "CH4431999123000889012");
    assert_eq!(stmts[1].account_iban, "CH9300762011623852957");

    // Premier compte : encaissement (CRDT, montant positif).
    assert_eq!(stmts[0].transactions.len(), 1);
    assert_eq!(stmts[0].transactions[0].amount, dec!(100.00));

    // Second compte : paiement (DBIT, montant négatif).
    assert_eq!(stmts[1].transactions.len(), 1);
    assert_eq!(stmts[1].transactions[0].amount, dec!(-50.00));
}

#[test]
fn parse_truncated_returns_malformed_xml_error() {
    let xml = load("v04_truncated.xml");
    let err = parse_camt053(&xml).expect_err("XML tronqué doit échouer");
    assert!(
        matches!(err, CamtError::MalformedXml(_)),
        "attendu MalformedXml, obtenu {err:?}"
    );
}

#[test]
fn parse_unknown_namespace_returns_unsupported_version() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.99">
  <BkToCstmrStmt/>
</Document>"#;
    let err = parse_camt053(xml).expect_err("namespace inconnu doit échouer");
    match err {
        CamtError::UnsupportedVersion(uri) => {
            assert!(uri.contains("camt.053.001.99"));
        }
        other => panic!("attendu UnsupportedVersion, obtenu {other:?}"),
    }
}

#[test]
fn parse_invalid_iban_keeps_transaction() {
    // §iban-tolerant : un IBAN counterparty au checksum cassé est
    // conservé brut comme `Some("CH00000000000000000000")`. Le rejet
    // métier vit côté `kesh-core::types::Iban`. Pas de warning côté
    // parseur.
    let xml = load("v04_invalid_iban.xml");
    let stmts = parse_camt053(&xml).expect("parse OK même avec IBAN cassé");
    assert_eq!(stmts.len(), 1);

    let s = &stmts[0];
    assert_eq!(s.account_iban, "CH4431999123000889012");
    assert_eq!(s.transactions.len(), 1);

    let tx = &s.transactions[0];
    assert_eq!(
        tx.counterparty_iban.as_deref(),
        Some("CH00000000000000000000"),
        "IBAN cassé conservé brut"
    );
    assert_eq!(tx.counterparty_name.as_deref(), Some("Client IBAN cassé"));
}

#[test]
fn parse_eur_currency_preserved() {
    // §currency : le parseur extrait la devise telle qu'elle apparaît.
    // Le rejet « EUR non supporté v0.1 » vit dans
    // `kesh-core::bank_imports::validate_currency_supported_v0_1`.
    let xml = load("v04_eur_currency.xml");
    let stmts = parse_camt053(&xml).expect("parse OK pour EUR");
    assert_eq!(stmts.len(), 1);

    let s = &stmts[0];
    assert_eq!(s.currency, "EUR", "devise du compte préservée");
    assert_eq!(s.transactions.len(), 1);
    assert_eq!(s.transactions[0].currency, "EUR");
}

#[test]
fn parse_credit_debit_indicator_signs_amount_correctly() {
    // <CdtDbtInd>CRDT</> ⇒ montant positif.
    // <CdtDbtInd>DBIT</> ⇒ montant négatif.
    // Les <Amt> dans CAMT.053 sont toujours non-signés.
    let xml = load("v04_credit_debit_indicator.xml");
    let stmts = parse_camt053(&xml).expect("parse OK");
    assert_eq!(stmts.len(), 1);
    let s = &stmts[0];
    assert_eq!(s.transactions.len(), 2);

    let crdt = &s.transactions[0];
    assert_eq!(crdt.amount, dec!(300.00), "CRDT ⇒ montant positif");
    assert_eq!(crdt.counterparty_name.as_deref(), Some("Client recette"));

    let dbit = &s.transactions[1];
    assert_eq!(dbit.amount, dec!(-200.00), "DBIT ⇒ montant négatif");
    assert_eq!(dbit.counterparty_name.as_deref(), Some("Fournisseur"));
}
