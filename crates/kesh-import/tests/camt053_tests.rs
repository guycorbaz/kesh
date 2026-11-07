//! Tests d'intégration du parseur CAMT.053 (Story 8-1a, AC #5 et #9).
//!
//! Les 19 fixtures (`ls tests/fixtures/camt053/*.xml | wc -l`) couvrent les cas
//! ci-dessous ; les cinq ajoutées par la Story v017-1 (caractérisation, Latin-1,
//! attribut en double) sont exercées par `camt053_golden.rs` :
//! - v04 namespace par défaut + namespace préfixé (régression H4)
//! - v08 namespace
//! - Sous-transactions (FR49)
//! - Multi-statements (un fichier, plusieurs comptes)
//! - XML tronqué
//! - Namespace inconnu
//! - IBAN counterparty cassé conservé brut (§iban-tolerant)
//! - Devise EUR préservée par le parseur (rejet côté `kesh-core`)
//! - `CdtDbtInd` qui signe le montant
//! - Signe absent de la `Ntry`, repris de chaque `TxDtls`
//! - XML tronqué après la fermeture d'un `Stmt` (garde de fin de fichier)
//! - Chevauchement avec `v04_minimal.xml` (doublon de ligne, pas de fichier ;
//!   exercé côté API)
//! - Montants d'information en devise étrangère (`AmtDtls`, `Chrgs`) : la
//!   devise de la transaction reste celle de son montant (Story 27-2, AC19)

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

    // Review code Pass 2 finding F15 : couverture body-dispatch sur
    // namespace préfixé. Sans ces assertions, un défaut de matching
    // dans `apply_balance` ou `handle_bal_text` pour les tags
    // préfixés laisserait silencieusement opening/closing à None.
    assert_eq!(
        s.opening_balance,
        Some(dec!(5000.00)),
        "OPBD doit être extrait via handle_bal_text + apply_balance avec préfixe"
    );
    assert_eq!(
        s.closing_balance,
        Some(dec!(5500.00)),
        "CLBD doit être extrait via handle_bal_text + apply_balance avec préfixe"
    );

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
fn parse_truncated_after_stmt_close_caught_by_eof_guard() {
    // Régression code review Pass 1 finding F6 : la fixture
    // `v04_truncated.xml` se coupe en plein milieu d'un texte, donc
    // quick-xml peut très bien lever lui-même une erreur sur le texte
    // tronqué — on ne peut pas distinguer si c'est notre check EOF
    // explicite ou quick-xml qui catch. Cette fixture-ci est bien-
    // formée jusqu'après `</Stmt>` puis EOF brutal sans
    // `</BkToCstmrStmt>` ni `</Document>`. quick-xml retourne
    // proprement Event::Eof ; c'est *uniquement* notre guard qui
    // détecte les tags non fermés via `path.is_empty()`.
    let xml = load("v04_truncated_after_stmt_close.xml");
    let err = parse_camt053(&xml).expect_err("EOF avec tags ouverts doit échouer");
    match err {
        CamtError::MalformedXml(msg) => {
            assert!(
                msg.contains("fin de fichier inattendue"),
                "le message doit indiquer la cause EOF (guard explicite), obtenu : {msg}"
            );
            assert!(
                msg.contains("BkToCstmrStmt"),
                "le message doit nommer le tag non fermé, obtenu : {msg}"
            );
        }
        other => panic!("attendu MalformedXml(...), obtenu {other:?}"),
    }
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

#[test]
fn parse_ntry_without_sign_falls_back_to_txdtls_signs() {
    // Régression review code Pass 3 finding BH-Haiku #10 (et patch
    // Pass 2 F13) : un fichier dans lequel `<Ntry>` n'a PAS de
    // `<CdtDbtInd>` mais où chaque `<TxDtls>` porte le sien doit
    // être parsé sans erreur. Le sign Ntry devient `Option::None` et
    // le parseur applique le sign de chaque TxDtls.
    //
    // Ce test verrouille la régression : si une refonte future remet
    // `n.sign.ok_or(...)?` au-dessus du check `txdtls.is_empty()`,
    // cette fixture lèverait MissingRequiredField et le test
    // échouerait.
    let xml = load("v04_ntry_sign_fallback.xml");
    let stmts = parse_camt053(&xml).expect("parse OK avec Ntry sans CdtDbtInd");
    assert_eq!(stmts.len(), 1);

    let s = &stmts[0];
    assert_eq!(s.account_iban, "CH4431999123000889012");
    assert_eq!(s.transactions.len(), 2, "2 TxDtls ⇒ 2 ImportedTransaction");

    let crdt = &s.transactions[0];
    assert_eq!(crdt.amount, dec!(60.00), "TxDtls[0] CRDT ⇒ montant positif");
    assert_eq!(crdt.end_to_end_id.as_deref(), Some("E2E-FALLBACK-CRDT"));
    assert_eq!(crdt.counterparty_name.as_deref(), Some("Client recette"));

    let dbit = &s.transactions[1];
    assert_eq!(
        dbit.amount,
        dec!(-40.00),
        "TxDtls[1] DBIT ⇒ montant négatif (sign TxDtls utilisé même sans Ntry sign)"
    );
    assert_eq!(dbit.end_to_end_id.as_deref(), Some("E2E-FALLBACK-DBIT"));
    assert_eq!(dbit.counterparty_name.as_deref(), Some("Fournisseur"));
}

// ── Story 27-2 (#624), AC19 : la devise d'une transaction est celle de
//    l'élément qui porte son montant, jamais celle d'un montant
//    d'information (`<AmtDtls>`, `<Chrgs>`). Écrits et vus ROUGES sur le
//    code d'avant (cf. Dev Agent Record de la story).

fn amtdtls_statement() -> kesh_import::ImportedStatement {
    let mut stmts = parse_camt053(&load("v04_amtdtls_foreign_currency.xml")).expect("parse OK");
    assert_eq!(stmts.len(), 1);
    let s = stmts.remove(0);
    assert_eq!(s.currency, "CHF");
    assert_eq!(s.transactions.len(), 3, "une transaction par entrée");
    s
}

/// AC19-(1) : entrée 1, `TxDtls/Amt` CHF suivi d'`AmtDtls` et de `Chrgs`
/// en EUR → CHF, montant de `TxDtls/Amt`.
#[test]
fn txdtls_currency_is_that_of_its_amount_not_of_amtdtls_or_charges() {
    let tx = &amtdtls_statement().transactions[0];
    assert_eq!(tx.currency, "CHF");
    assert_eq!(tx.amount, dec!(-54.20));
}

/// AC19-(1-bis) : entrée 3, un `<TxDtls>` sans `Amt` propre et un
/// `AmtDtls` en EUR → devise et montant de l'entrée. Seule à tuer la
/// mutation « la première devise rencontrée gagne » (M25).
#[test]
fn txdtls_without_amount_takes_entry_currency_not_amtdtls() {
    let tx = &amtdtls_statement().transactions[2];
    assert_eq!(tx.currency, "CHF");
    assert_eq!(tx.amount, dec!(-20.00));
}

/// AC19-(2) : entrée 2, sans `<TxDtls>`, `Ntry/Amt` CHF suivi d'un
/// `AmtDtls` en EUR → CHF, montant de `Ntry/Amt`.
#[test]
fn ntry_currency_is_that_of_its_amount_not_of_amtdtls() {
    let tx = &amtdtls_statement().transactions[1];
    assert_eq!(tx.currency, "CHF");
    assert_eq!(tx.amount, dec!(-30.00));
}

// ── Story v0174-correctifs-1 (#819) : le texte d'un élément se lit EN ENTIER,
//    entités (`&amp;`, `&#xE9;`) et sections CDATA comprises. Écrits et vus
//    ROUGES sur le code d'avant (Dev Agent Record de la fiche). XML inventé.

/// Un relevé v04 d'une seule entrée (avec un `TxDtls`) dont on fournit les
/// textes bruts — tels qu'écrits dans le XML, entités et CDATA compris.
/// IBAN, noms et montants inventés.
fn one_entry(ntry_ref: &str, addtl: &str, name: &str, ustrd: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.04">
  <BkToCstmrStmt>
    <Stmt>
      <Id>STMT-819</Id>
      <FrToDt>
        <FrDtTm>2026-10-01T00:00:00</FrDtTm>
        <ToDtTm>2026-10-31T23:59:59</ToDtTm>
      </FrToDt>
      <Acct>
        <Id>
          <IBAN>CH9300762011623852957</IBAN>
        </Id>
        <Ccy>CHF</Ccy>
      </Acct>
      <Ntry>
        <Amt Ccy="CHF">10.00</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt>
          <Dt>2026-10-10</Dt>
        </BookgDt>
        <NtryRef>{ntry_ref}</NtryRef>
        <AcctSvcrRef>TX-819</AcctSvcrRef>
        <NtryDtls>
          <TxDtls>
            <RltdPties>
              <Dbtr>
                <Nm>{name}</Nm>
              </Dbtr>
            </RltdPties>
            <RmtInf>
              <Ustrd>{ustrd}</Ustrd>
            </RmtInf>
          </TxDtls>
        </NtryDtls>
        <AddtlNtryInf>{addtl}</AddtlNtryInf>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>
"#
    )
    .into_bytes()
}

/// Un relevé v04 d'une entrée sans `TxDtls` : `AddtlNtryInf` y devient `details`.
fn one_entry_without_txdtls(ntry_ref: &str, addtl: &str) -> Vec<u8> {
    let xml = String::from_utf8(one_entry(ntry_ref, addtl, "X", "X")).unwrap();
    let start = xml.find("<NtryDtls>").unwrap();
    let end = xml.find("</NtryDtls>").unwrap() + "</NtryDtls>".len();
    format!("{}{}", &xml[..start], &xml[end..]).into_bytes()
}

fn only_tx(xml: &[u8]) -> kesh_import::ImportedTransaction {
    let mut stmts = parse_camt053(xml).expect("parse OK");
    assert_eq!(stmts.len(), 1);
    let mut txs = std::mem::take(&mut stmts[0].transactions);
    assert_eq!(txs.len(), 1);
    txs.remove(0)
}

fn name_of(nm: &str) -> Option<String> {
    only_tx(&one_entry("R", "I", nm, "U")).counterparty_name
}

/// AC-1.1 — les entités prédéfinies se résolvent dans le nom.
#[test]
fn entities_in_name_are_resolved_819() {
    assert_eq!(
        name_of("Dupont &amp; Fils").as_deref(),
        Some("Dupont & Fils")
    );
    assert_eq!(name_of("a &lt; b &gt; c").as_deref(), Some("a < b > c"));
    assert_eq!(
        name_of("&quot;Atelier&quot; l&apos;Ours").as_deref(),
        Some("\"Atelier\" l'Ours")
    );
}

/// AC-1.1 — les références de caractère, hexadécimale et décimale.
#[test]
fn char_refs_in_name_are_resolved_819() {
    assert_eq!(
        name_of("&#xE9;tablissement").as_deref(),
        Some("établissement")
    );
    assert_eq!(
        name_of("&#233;tablissement").as_deref(),
        Some("établissement")
    );
}

/// AC-1.1 — les blancs AUTOUR d'une entité sont gardés ; seuls les blancs
/// extérieurs du texte tombent (C-v0174c-21).
#[test]
fn spaces_around_an_entity_are_kept_819() {
    assert_eq!(name_of("A &amp; B").as_deref(), Some("A & B"));
    assert_eq!(name_of("A&amp;B").as_deref(), Some("A&B"));
    assert_eq!(name_of("  A &amp; B  ").as_deref(), Some("A & B"));
}

/// AC-1.1 — une entité en tête et en queue du texte.
#[test]
fn entity_at_start_and_end_819() {
    assert_eq!(name_of("&amp;Co").as_deref(), Some("&Co"));
    assert_eq!(name_of("Co&amp;").as_deref(), Some("Co&"));
}

/// AC-1.1 — une référence d'entrée (`NtryRef`) entière, et un `Ustrd` en
/// UNE partie.
#[test]
fn entities_in_reference_and_ustrd_819() {
    let tx = only_tx(&one_entry_without_txdtls("REF &amp; Co &lt;1&gt;", "I"));
    assert_eq!(tx.reference.as_deref(), Some("REF & Co <1>"));
    let tx = only_tx(&one_entry("R", "I", "N", "A &amp; B"));
    assert_eq!(tx.details, "A & B", "une seule partie, non « A B »");
}

/// AC-1.2 — le CDATA se lit.
#[test]
fn cdata_is_read_819() {
    let tx = only_tx(&one_entry_without_txdtls(
        "R",
        "<![CDATA[Versement de Müller à Genève]]>",
    ));
    assert_eq!(tx.details, "Versement de Müller à Genève");
    let tx = only_tx(&one_entry(
        "R",
        "I",
        "N",
        "Facture <![CDATA[n° 12]]> octobre",
    ));
    assert_eq!(tx.details, "Facture n° 12 octobre");
    assert_eq!(name_of("<![CDATA[ Dupont ]]>").as_deref(), Some("Dupont"));
    let tx = only_tx(&one_entry("R", "I", "N", "<![CDATA[ligne 1\r\nligne 2]]>"));
    assert_eq!(tx.details, "ligne 1\r\nligne 2", "CRLF du CDATA gardé");
}

/// AC-1.3 — un texte coupé par un commentaire se lit comme en 0.17.3 : le
/// dernier fragment gagne (C-v0174c-3, écartée b). Vert avant : il fige.
#[test]
fn text_split_by_a_comment_reads_as_in_0173_819() {
    assert_eq!(name_of("A<!-- x -->B").as_deref(), Some("B"));
}

/// AC-1.3 — une entité inconnue refuse le fichier, au message de Kesh.
#[test]
fn unknown_entity_is_malformed_819() {
    let r = parse_camt053(&one_entry("R", "I", "Dupont &foo; Fils", "U"));
    match r {
        Err(CamtError::MalformedXml(m)) => assert!(
            m.contains("référence d'entité inconnue : foo"),
            "message de Kesh attendu : {m}"
        ),
        other => panic!("Err(MalformedXml) attendu, obtenu {other:?}"),
    }
}

/// AC-1.3 — un fichier ordinaire garde montants et dates (témoin).
#[test]
fn ordinary_amounts_and_dates_unchanged_819() {
    let tx = only_tx(&one_entry("R", "I", "N", "U"));
    assert_eq!(tx.amount, dec!(10.00));
    assert_eq!(tx.booking_date.to_string(), "2026-10-10");
    assert_eq!(tx.reference.as_deref(), Some("R"));
    assert_eq!(tx.details, "U");
}

/// Revue de code P1 (B-1 = E-4 = A-1) — la lecture corrigée d'un relevé
/// **camt.053.001.08** (`v08::parse`) : entité, référence de caractère, CDATA
/// (les espaces autour d'une entité sont couverts en v04,
/// `spaces_around_an_entity_are_kept_819`). Tue la mutation MA-819-8 (`v08.rs`
/// en lecture héritée).
#[test]
fn v08_corrected_reading_resolves_entities_and_cdata_819() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.08">
  <BkToCstmrStmt>
    <Stmt>
      <Id>STMT-V08-819</Id>
      <FrToDt><FrDtTm>2026-10-01T00:00:00</FrDtTm><ToDtTm>2026-10-31T23:59:59</ToDtTm></FrToDt>
      <Acct><Id><IBAN>CH5604835012345678009</IBAN></Id><Ccy>CHF</Ccy></Acct>
      <Ntry>
        <Amt Ccy="CHF">20.00</Amt>
        <CdtDbtInd>CRDT</CdtDbtInd>
        <BookgDt><Dt>2026-10-12</Dt></BookgDt>
        <NtryRef>REF &amp; Co &lt;8&gt;</NtryRef>
        <NtryDtls>
          <TxDtls>
            <RltdPties><Dbtr><Pty><Nm>Dupont &amp; Fils, &#xE9;tablissement</Nm></Pty></Dbtr></RltdPties>
            <RmtInf><Ustrd>Facture <![CDATA[n° 8]]> octobre</Ustrd></RmtInf>
          </TxDtls>
        </NtryDtls>
      </Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>
"#;
    let tx = only_tx(xml.as_bytes());
    assert_eq!(
        tx.counterparty_name.as_deref(),
        Some("Dupont & Fils, établissement")
    );
    assert_eq!(tx.reference.as_deref(), Some("REF & Co <8>"));
    assert_eq!(tx.details, "Facture n° 8 octobre");
}

/// Revue de code P1 (B-3) — une référence de caractère invalide (`&#0;`, un
/// demi-codet `&#xD800;`) refuse le fichier en `MalformedXml` (relais de
/// quick-xml), au lieu d'être ignorée.
#[test]
fn invalid_char_refs_are_malformed_819() {
    for r in ["&#0;", "&#xD800;"] {
        let res = parse_camt053(&one_entry("R", "I", &format!("Dupont {r} Fils"), "U"));
        assert!(
            matches!(res, Err(CamtError::MalformedXml(_))),
            "{r} : Err(MalformedXml) attendu, obtenu {res:?}"
        );
    }
}

/// Un relevé v04 d'une ligne, sur lequel les tests de la 33-2 greffent un élément
/// du `Stmt` ou la date de comptabilisation de la ligne.
fn releve_33_2(stmt_open: &str, stmt_extra: &str, booking_date: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:camt.053.001.04">
  <BkToCstmrStmt>
    {stmt_open}
      {stmt_extra}
      <Id>STMT-33-2</Id>
      <FrToDt><FrDtTm>2026-10-01T00:00:00</FrDtTm><ToDtTm>2026-10-31T23:59:59</ToDtTm></FrToDt>
      <Acct><Id><IBAN>CH4431999123000889012</IBAN></Id><Ccy>CHF</Ccy></Acct>
      <Ntry><Amt Ccy="CHF">10.00</Amt><CdtDbtInd>CRDT</CdtDbtInd><BookgDt><Dt>{booking_date}</Dt></BookgDt></Ntry>
    </Stmt>
  </BkToCstmrStmt>
</Document>
"#
    )
    .into_bytes()
}

/// AC-L1 (33-2, #923) — une entité inconnue dans le **texte** d'un élément que Kesh
/// ne lit pas est refusée : la portée du refus est le texte du fichier entier.
#[test]
fn unknown_entity_in_an_unread_element_is_refused() {
    let xml = releve_33_2("<Stmt>", "<Foo>&inconnue;</Foo>", "2026-10-10");
    assert!(
        matches!(parse_camt053(&xml), Err(CamtError::MalformedXml(_))),
        "{:?}",
        parse_camt053(&xml)
    );
}

/// AC-L1 (33-2, #923) — la borne de cette portée : dans un **attribut** non lu,
/// l'entité inconnue n'est pas refusée (seul `Ccy` est lu).
#[test]
fn unknown_entity_in_an_unread_attribute_is_not_refused() {
    let xml = releve_33_2(r#"<Stmt Foo="&inconnue;">"#, "", "2026-10-10");
    let stmts = parse_camt053(&xml).expect("attribut non lu : accepté");
    assert_eq!(stmts[0].transactions.len(), 1);
}

/// AC-S5 (a) (33-2, C-e332-22) — la relecture héritée lit comme la 0.17.3, **sans
/// borne d'années** : une année à un, deux ou trois chiffres n'y est pas une erreur.
/// Une borne sur les dates reçues se pose à la lecture corrigée (33-4) ; ce test ne
/// dit rien de celle-ci.
#[test]
fn legacy_reading_has_no_year_bound_922() {
    use chrono::Datelike;
    let read = |date: &str| {
        let xml = releve_33_2("<Stmt>", "", date);
        kesh_import::camt053::parse_legacy_0173(&xml).expect("lecture héritée")[0].transactions[0]
            .booking_date
            .year()
    };
    assert_eq!(read("0999-01-15"), 999);
    assert_eq!(read("&#x32;026-10-10"), 26);
}
