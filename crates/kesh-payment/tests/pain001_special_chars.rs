//! Référence pain.001 à caractères spéciaux (Story v017-1, #789, AC12).
//!
//! La montée de `quick-xml` (0.39 → 0.42) ne doit pas changer un octet du
//! fichier pain.001 que Kesh écrit. `golden_file_matches`
//! (`src/pain001/mod.rs`) tient l'exemple ordinaire ; ce test tient les
//! caractères que l'écrivain XML échappe : `&`, `<`, `>`, `"`, `'`, et un
//! caractère accentué (`é`), dans les noms et la référence non structurée.
//!
//! La référence `tests/fixtures/pain001_special_chars.xml` a été **produite
//! sous `quick-xml` 0.39.4**, avant la montée. Sa régénération
//! (`KESH_GOLDEN_REGEN=1`) est verrouillée comme celle de la caractérisation
//! CAMT.053 (`kesh-import/tests/camt053_golden.rs`) : refusée en CI et hors
//! d'un `Cargo.lock` qui résout `quick-xml` en `0.39.x`.
//!
//! **Écart assumé (C-v017-1-9)** : aucun texte de la référence ne contient de
//! `\r`. `quick-xml` 0.42 l'échappe en `&#13;` (#990), là où 0.39 l'écrivait
//! brut ; un lecteur XML normalise d'ailleurs un `\r` brut en `\n`. L'écriture
//! 0.42 est la plus fidèle ; le fichier pain.001 est régénéré à la demande,
//! jamais stocké.

use std::path::Path;

use chrono::NaiveDate;
use kesh_payment::pain001::{
    CreditorAccount, Pain001Batch, Pain001Tx, PaymentReference, generate_pain001,
};
use rust_decimal_macros::dec;

const REFERENCE: &str = "tests/fixtures/pain001_special_chars.xml";

/// Version de `quick-xml` résolue par le `Cargo.lock` du workspace.
fn locked_quick_xml_version() -> String {
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let lock = std::fs::read_to_string(&lock_path).expect("Cargo.lock lisible");
    let mut lines = lock.lines();
    let mut versions = Vec::new();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"quick-xml\"" {
            let v = lines
                .next()
                .and_then(|l| l.trim().strip_prefix("version = \""))
                .and_then(|l| l.strip_suffix('"'))
                .expect("ligne version après name = \"quick-xml\"");
            versions.push(v.to_string());
        }
    }
    assert_eq!(
        versions.len(),
        1,
        "une seule version de quick-xml : {versions:?}"
    );
    versions.remove(0)
}

/// Mode de régénération, verrouillé.
fn regen_mode() -> bool {
    if std::env::var("KESH_GOLDEN_REGEN").as_deref() != Ok("1") {
        return false;
    }
    assert!(
        std::env::var_os("CI").is_none(),
        "KESH_GOLDEN_REGEN interdit en CI"
    );
    let v = locked_quick_xml_version();
    assert!(
        v.starts_with("0.39."),
        "KESH_GOLDEN_REGEN refusé : la référence se produit sous quick-xml 0.39.x, le Cargo.lock résout {v}"
    );
    true
}

fn batch() -> Pain001Batch {
    Pain001Batch {
        msg_id: "MSG-SPECIAL-0001".into(),
        payment_info_id: "PMT-SPECIAL-0001".into(),
        creation_dt: NaiveDate::from_ymd_opt(2026, 10, 30)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap(),
        initiating_party: "Démo & Fils <SA>".into(),
        debtor_name: "Démo & Fils <SA>".into(),
        debtor_iban: "CH9300762011623852957".into(),
        requested_execution_date: NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
        transactions: vec![
            Pain001Tx {
                end_to_end_id: "PAY-SPECIAL-00000001".into(),
                amount: dec!(1234.50),
                creditor_name: "L'Atelier \"Été\" & Cie > Genève".into(),
                creditor_account: CreditorAccount::Iban("CH5604835012345678009".into()),
                reference: PaymentReference::Unstructured(
                    "Facture n° 42 : <urgent> & \"payée\" d'avance, été 2026".into(),
                ),
            },
            Pain001Tx {
                end_to_end_id: "PAY-SPECIAL-00000002".into(),
                amount: dec!(99.95),
                creditor_name: "Régie <Genève> & Co".into(),
                creditor_account: CreditorAccount::QrIban("CH4431999123000889012".into()),
                reference: PaymentReference::Qrr("210000000003139471430009017".into()),
            },
        ],
    }
}

#[test]
fn special_characters_are_written_as_under_quick_xml_039() {
    let xml = generate_pain001(&batch()).expect("génération pain.001");
    assert!(
        !xml.contains('\r'),
        "aucun \\r dans la référence (C-v017-1-9)"
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(REFERENCE);
    if regen_mode() {
        std::fs::write(&path, format!("{xml}\n")).expect("écriture de la référence");
        return;
    }
    let reference = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "référence {} absente ({e}) : elle se produit sous quick-xml 0.39",
            path.display()
        )
    });
    assert_eq!(xml.trim_end(), reference.trim_end());
    // La référence porte bien les formes échappées qu'elle fige.
    for forme in ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;", "é"] {
        assert!(
            reference.contains(forme),
            "forme {forme:?} absente de la référence"
        );
    }
}
