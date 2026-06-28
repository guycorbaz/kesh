//! Générateur `pain.001.001.09` (CustomerCreditTransferInitiation) — Story 12.3.
//!
//! Variante Swiss Payment Standards (SIX). Génère un ordre de virement groupé :
//! un seul `PmtInf` (même débiteur + même date d'exécution) contenant N
//! `CdtTrfTxInf`. Types de paiement supportés (DC2) :
//! - **Type 1** (IBAN domestique CH/LI) → `RmtInf/Ustrd` (référence libre) ;
//! - **Type 3** (QR-IBAN) → `RmtInf/Strd/CdtrRefInf` avec référence QRR.
//!
//! Pur (zéro I/O) : les coordonnées (IBAN/QR-IBAN/QRR) sont supposées déjà
//! validées par l'appelant (`kesh-db`). Écriture via `quick_xml::Writer`
//! (échappement XML automatique). Ordre des éléments conforme au schéma XSD.

use chrono::{NaiveDate, NaiveDateTime};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use rust_decimal::Decimal;
use std::io::Cursor;
use thiserror::Error;

/// Namespace XSD de `pain.001.001.09` (le suffixe « ch.03 » est une version des
/// guidelines SIX, pas un namespace distinct).
pub const PAIN001_NS: &str = "urn:iso:std:iso:20022:tech:xsd:pain.001.001.09";

/// Devise unique supportée (v0.4).
const CCY: &str = "CHF";

/// Longueurs maximales ISO 20022 (au-delà → erreur, refus XSD).
const MAX_ID: usize = 35; // MsgId, PmtInfId, EndToEndId
const MAX_NAME: usize = 70; // Dbtr/Nm, Cdtr/Nm, InitgPty/Nm
const MAX_USTRD: usize = 140; // RmtInf/Ustrd

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PaymentError {
    #[error("le lot de paiement ne contient aucune transaction")]
    EmptyBatch,
    #[error("champ '{field}' trop long ({len} > {max})")]
    TooLong {
        field: &'static str,
        len: usize,
        max: usize,
    },
    #[error("erreur d'écriture XML : {0}")]
    Xml(String),
}

/// Compte créancier : IBAN classique (type 1) ou QR-IBAN (type 3).
#[derive(Debug, Clone)]
pub enum CreditorAccount {
    Iban(String),
    QrIban(String),
}

/// Référence de paiement associée à la transaction.
#[derive(Debug, Clone)]
pub enum PaymentReference {
    /// Référence QR (QRR, 27 chiffres) — émise en `RmtInf/Strd/CdtrRefInf`.
    Qrr(String),
    /// Référence libre — émise en `RmtInf/Ustrd`. Vide → pas de `RmtInf`.
    Unstructured(String),
}

/// Une transaction de crédit (un virement vers un créancier).
#[derive(Debug, Clone)]
pub struct Pain001Tx {
    pub end_to_end_id: String,
    pub amount: Decimal,
    pub creditor_name: String,
    pub creditor_account: CreditorAccount,
    pub reference: PaymentReference,
}

/// Lot pain.001 : en-tête + transactions.
#[derive(Debug, Clone)]
pub struct Pain001Batch {
    pub msg_id: String,
    pub payment_info_id: String,
    pub creation_dt: NaiveDateTime,
    pub initiating_party: String,
    pub debtor_name: String,
    pub debtor_iban: String,
    pub requested_execution_date: NaiveDate,
    pub transactions: Vec<Pain001Tx>,
}

type W = Writer<Cursor<Vec<u8>>>;

fn xml_err(e: impl std::fmt::Display) -> PaymentError {
    PaymentError::Xml(e.to_string())
}

fn check_len(field: &'static str, s: &str, max: usize) -> Result<(), PaymentError> {
    if s.chars().count() > max {
        return Err(PaymentError::TooLong {
            field,
            len: s.chars().count(),
            max,
        });
    }
    Ok(())
}

/// Élément feuille `<name>text</name>` (texte échappé automatiquement).
fn leaf(w: &mut W, name: &str, text: &str) -> Result<(), PaymentError> {
    w.write_event(Event::Start(BytesStart::new(name)))
        .map_err(xml_err)?;
    w.write_event(Event::Text(BytesText::new(text)))
        .map_err(xml_err)?;
    w.write_event(Event::End(BytesEnd::new(name)))
        .map_err(xml_err)?;
    Ok(())
}

fn open(w: &mut W, name: &str) -> Result<(), PaymentError> {
    w.write_event(Event::Start(BytesStart::new(name)))
        .map_err(xml_err)
}

fn close(w: &mut W, name: &str) -> Result<(), PaymentError> {
    w.write_event(Event::End(BytesEnd::new(name))).map_err(xml_err)
}

/// Formate un montant en 2 décimales, point décimal, sans séparateur de milliers.
fn amt(d: Decimal) -> String {
    format!("{:.2}", d.round_dp(2))
}

/// Génère le XML `pain.001.001.09` d'un lot. Le résultat commence par la
/// déclaration `<?xml version="1.0" encoding="UTF-8"?>`.
pub fn generate_pain001(batch: &Pain001Batch) -> Result<String, PaymentError> {
    if batch.transactions.is_empty() {
        return Err(PaymentError::EmptyBatch);
    }
    check_len("MsgId", &batch.msg_id, MAX_ID)?;
    check_len("PmtInfId", &batch.payment_info_id, MAX_ID)?;
    check_len("InitgPty", &batch.initiating_party, MAX_NAME)?;
    check_len("Dbtr/Nm", &batch.debtor_name, MAX_NAME)?;
    for tx in &batch.transactions {
        check_len("EndToEndId", &tx.end_to_end_id, MAX_ID)?;
        check_len("Cdtr/Nm", &tx.creditor_name, MAX_NAME)?;
        if let PaymentReference::Unstructured(r) = &tx.reference {
            check_len("RmtInf/Ustrd", r, MAX_USTRD)?;
        }
    }

    // NbOfTxs + CtrlSum (Σ des InstdAmt, mêmes valeurs au niveau GrpHdr et PmtInf, DC5).
    let nb_of_txs = batch.transactions.len();
    let ctrl_sum: Decimal = batch.transactions.iter().map(|t| t.amount).sum();
    let nb = nb_of_txs.to_string();
    let cs = amt(ctrl_sum);

    let mut w: W = Writer::new(Cursor::new(Vec::new()));
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_err)?;

    let mut doc = BytesStart::new("Document");
    doc.push_attribute(("xmlns", PAIN001_NS));
    w.write_event(Event::Start(doc)).map_err(xml_err)?;
    open(&mut w, "CstmrCdtTrfInitn")?;

    // --- GrpHdr (ordre XSD : MsgId, CreDtTm, NbOfTxs, CtrlSum, InitgPty) ---
    open(&mut w, "GrpHdr")?;
    leaf(&mut w, "MsgId", &batch.msg_id)?;
    leaf(
        &mut w,
        "CreDtTm",
        &batch.creation_dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
    )?;
    leaf(&mut w, "NbOfTxs", &nb)?;
    leaf(&mut w, "CtrlSum", &cs)?;
    open(&mut w, "InitgPty")?;
    leaf(&mut w, "Nm", &batch.initiating_party)?;
    close(&mut w, "InitgPty")?;
    close(&mut w, "GrpHdr")?;

    // --- PmtInf (un seul, DC5) ---
    open(&mut w, "PmtInf")?;
    leaf(&mut w, "PmtInfId", &batch.payment_info_id)?;
    leaf(&mut w, "PmtMtd", "TRF")?;
    leaf(&mut w, "NbOfTxs", &nb)?;
    leaf(&mut w, "CtrlSum", &cs)?;
    open(&mut w, "ReqdExctnDt")?;
    leaf(
        &mut w,
        "Dt",
        &batch.requested_execution_date.format("%Y-%m-%d").to_string(),
    )?;
    close(&mut w, "ReqdExctnDt")?;
    // Dbtr
    open(&mut w, "Dbtr")?;
    leaf(&mut w, "Nm", &batch.debtor_name)?;
    close(&mut w, "Dbtr")?;
    // DbtrAcct/Id/IBAN
    open(&mut w, "DbtrAcct")?;
    open(&mut w, "Id")?;
    leaf(&mut w, "IBAN", &batch.debtor_iban)?;
    close(&mut w, "Id")?;
    close(&mut w, "DbtrAcct")?;
    // DbtrAgt/FinInstnId/Othr/Id=NOTPROVIDED (DC3)
    open(&mut w, "DbtrAgt")?;
    open(&mut w, "FinInstnId")?;
    open(&mut w, "Othr")?;
    leaf(&mut w, "Id", "NOTPROVIDED")?;
    close(&mut w, "Othr")?;
    close(&mut w, "FinInstnId")?;
    close(&mut w, "DbtrAgt")?;
    leaf(&mut w, "ChrgBr", "SLEV")?;

    for tx in &batch.transactions {
        write_tx(&mut w, tx)?;
    }

    close(&mut w, "PmtInf")?;
    close(&mut w, "CstmrCdtTrfInitn")?;
    w.write_event(Event::End(BytesEnd::new("Document")))
        .map_err(xml_err)?;

    let bytes = w.into_inner().into_inner();
    String::from_utf8(bytes).map_err(xml_err)
}

/// Écrit un `CdtTrfTxInf` (ordre XSD : PmtId, Amt, Cdtr, CdtrAcct, RmtInf).
fn write_tx(w: &mut W, tx: &Pain001Tx) -> Result<(), PaymentError> {
    open(w, "CdtTrfTxInf")?;
    // PmtId/EndToEndId
    open(w, "PmtId")?;
    leaf(w, "EndToEndId", &tx.end_to_end_id)?;
    close(w, "PmtId")?;
    // Amt/InstdAmt Ccy="CHF"
    open(w, "Amt")?;
    let mut instd = BytesStart::new("InstdAmt");
    instd.push_attribute(("Ccy", CCY));
    w.write_event(Event::Start(instd)).map_err(xml_err)?;
    w.write_event(Event::Text(BytesText::new(&amt(tx.amount))))
        .map_err(xml_err)?;
    w.write_event(Event::End(BytesEnd::new("InstdAmt")))
        .map_err(xml_err)?;
    close(w, "Amt")?;
    // Cdtr/Nm
    open(w, "Cdtr")?;
    leaf(w, "Nm", &tx.creditor_name)?;
    close(w, "Cdtr")?;
    // CdtrAcct/Id/IBAN (IBAN ou QR-IBAN, même balise IBAN)
    open(w, "CdtrAcct")?;
    open(w, "Id")?;
    let iban = match &tx.creditor_account {
        CreditorAccount::Iban(s) | CreditorAccount::QrIban(s) => s,
    };
    leaf(w, "IBAN", iban)?;
    close(w, "Id")?;
    close(w, "CdtrAcct")?;
    // RmtInf : Strd/CdtrRefInf (QRR) ou Ustrd (libre non vide)
    match &tx.reference {
        PaymentReference::Qrr(qrr) => {
            open(w, "RmtInf")?;
            open(w, "Strd")?;
            open(w, "CdtrRefInf")?;
            open(w, "Tp")?;
            open(w, "CdOrPrtry")?;
            leaf(w, "Prtry", "QRR")?;
            close(w, "CdOrPrtry")?;
            close(w, "Tp")?;
            leaf(w, "Ref", qrr)?;
            close(w, "CdtrRefInf")?;
            close(w, "Strd")?;
            close(w, "RmtInf")?;
        }
        PaymentReference::Unstructured(r) if !r.trim().is_empty() => {
            open(w, "RmtInf")?;
            leaf(w, "Ustrd", r)?;
            close(w, "RmtInf")?;
        }
        PaymentReference::Unstructured(_) => {} // pas de RmtInf si vide
    }
    close(w, "CdtTrfTxInf")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_batch(txs: Vec<Pain001Tx>) -> Pain001Batch {
        Pain001Batch {
            msg_id: "MSG-001".into(),
            payment_info_id: "PMT-001".into(),
            creation_dt: NaiveDate::from_ymd_opt(2026, 6, 28)
                .unwrap()
                .and_hms_opt(10, 30, 0)
                .unwrap(),
            initiating_party: "Ma PME SA".into(),
            debtor_name: "Ma PME SA".into(),
            debtor_iban: "CH9300762011623852957".into(),
            requested_execution_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            transactions: txs,
        }
    }

    fn tx_iban() -> Pain001Tx {
        Pain001Tx {
            end_to_end_id: "PAY-00000001-00000042".into(),
            amount: dec!(1081.00),
            creditor_name: "Fournisseur A".into(),
            creditor_account: CreditorAccount::Iban("CH9300762011623852957".into()),
            reference: PaymentReference::Unstructured("Facture 2026-042".into()),
        }
    }

    fn tx_qr() -> Pain001Tx {
        Pain001Tx {
            end_to_end_id: "PAY-00000001-00000043".into(),
            amount: dec!(500.50),
            creditor_name: "Fournisseur B".into(),
            creditor_account: CreditorAccount::QrIban("CH4431999123000889012".into()),
            reference: PaymentReference::Qrr("210000000003139471430009017".into()),
        }
    }

    #[test]
    fn empty_batch_rejected() {
        let b = sample_batch(vec![]);
        assert_eq!(generate_pain001(&b).unwrap_err(), PaymentError::EmptyBatch);
    }

    #[test]
    fn declares_xml_and_namespace() {
        let xml = generate_pain001(&sample_batch(vec![tx_iban()])).unwrap();
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains(&format!("xmlns=\"{PAIN001_NS}\"")));
        assert!(xml.contains("<PmtMtd>TRF</PmtMtd>"));
        assert!(xml.contains("<ChrgBr>SLEV</ChrgBr>"));
        assert!(xml.contains("<Othr><Id>NOTPROVIDED</Id></Othr>"));
    }

    #[test]
    fn nb_and_ctrlsum_consistent_grphdr_and_pmtinf() {
        let xml = generate_pain001(&sample_batch(vec![tx_iban(), tx_qr()])).unwrap();
        // 2 transactions, somme 1081.00 + 500.50 = 1581.50, présent 2× (GrpHdr + PmtInf).
        assert_eq!(xml.matches("<NbOfTxs>2</NbOfTxs>").count(), 2);
        assert_eq!(xml.matches("<CtrlSum>1581.50</CtrlSum>").count(), 2);
    }

    #[test]
    fn qr_iban_emits_structured_qrr() {
        let xml = generate_pain001(&sample_batch(vec![tx_qr()])).unwrap();
        assert!(xml.contains("<Strd><CdtrRefInf><Tp><CdOrPrtry><Prtry>QRR</Prtry>"));
        assert!(xml.contains("<Ref>210000000003139471430009017</Ref>"));
        assert!(!xml.contains("<Ustrd>"));
    }

    #[test]
    fn iban_emits_unstructured() {
        let xml = generate_pain001(&sample_batch(vec![tx_iban()])).unwrap();
        assert!(xml.contains("<Ustrd>Facture 2026-042</Ustrd>"));
        assert!(!xml.contains("<Strd>"));
        assert!(xml.contains("<InstdAmt Ccy=\"CHF\">1081.00</InstdAmt>"));
    }

    #[test]
    fn escapes_special_characters() {
        let mut tx = tx_iban();
        tx.creditor_name = "Müller & Co <SA>".into();
        let xml = generate_pain001(&sample_batch(vec![tx])).unwrap();
        assert!(xml.contains("Müller &amp; Co &lt;SA&gt;"));
        assert!(!xml.contains("& Co")); // le & nu ne doit pas subsister
    }

    #[test]
    fn name_too_long_rejected() {
        let mut b = sample_batch(vec![tx_iban()]);
        b.debtor_name = "X".repeat(71);
        assert!(matches!(
            generate_pain001(&b).unwrap_err(),
            PaymentError::TooLong { field: "Dbtr/Nm", .. }
        ));
    }

    #[test]
    fn empty_unstructured_omits_rmtinf() {
        let mut tx = tx_iban();
        tx.reference = PaymentReference::Unstructured("  ".into());
        let xml = generate_pain001(&sample_batch(vec![tx])).unwrap();
        assert!(!xml.contains("<RmtInf>"));
    }
}

#[cfg(test)]
mod golden {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn golden_file_matches() {
        let batch = Pain001Batch {
            msg_id: "MSG-2026-0001".into(),
            payment_info_id: "PMT-2026-0001".into(),
            creation_dt: chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
                .unwrap()
                .and_hms_opt(9, 0, 0)
                .unwrap(),
            initiating_party: "Demo PME SA".into(),
            debtor_name: "Demo PME SA".into(),
            debtor_iban: "CH9300762011623852957".into(),
            requested_execution_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
            transactions: vec![
                Pain001Tx {
                    end_to_end_id: "PAY-00000001-00000001".into(),
                    amount: dec!(1081.00),
                    creditor_name: "Fournisseur Alpha".into(),
                    creditor_account: CreditorAccount::Iban("CH5604835012345678009".into()),
                    reference: PaymentReference::Unstructured("Facture A-2026-042".into()),
                },
                Pain001Tx {
                    end_to_end_id: "PAY-00000001-00000002".into(),
                    amount: dec!(540.50),
                    creditor_name: "Fournisseur Beta".into(),
                    creditor_account: CreditorAccount::QrIban("CH4431999123000889012".into()),
                    reference: PaymentReference::Qrr("210000000003139471430009017".into()),
                },
            ],
        };
        let xml = generate_pain001(&batch).unwrap();
        let golden = include_str!("../../tests/fixtures/pain001_sample.xml");
        assert_eq!(xml.trim_end(), golden.trim_end());
    }
}
