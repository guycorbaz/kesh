//! Génère le golden file pain.001 de référence (test golden_file_matches).
use kesh_payment::pain001::*;
use rust_decimal_macros::dec;
fn main() {
    let batch = Pain001Batch {
        msg_id: "MSG-2026-0001".into(),
        payment_info_id: "PMT-2026-0001".into(),
        creation_dt: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap().and_hms_opt(9, 0, 0).unwrap(),
        initiating_party: "Demo PME SA".into(),
        debtor_name: "Demo PME SA".into(),
        debtor_iban: "CH9300762011623852957".into(),
        requested_execution_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
        transactions: vec![
            Pain001Tx { end_to_end_id: "PAY-00000001-00000001".into(), amount: dec!(1081.00), creditor_name: "Fournisseur Alpha".into(), creditor_account: CreditorAccount::Iban("CH5604835012345678009".into()), reference: PaymentReference::Unstructured("Facture A-2026-042".into()) },
            Pain001Tx { end_to_end_id: "PAY-00000001-00000002".into(), amount: dec!(540.50), creditor_name: "Fournisseur Beta".into(), creditor_account: CreditorAccount::QrIban("CH4431999123000889012".into()), reference: PaymentReference::Qrr("210000000003139471430009017".into()) },
        ],
    };
    print!("{}", generate_pain001(&batch).unwrap());
}
